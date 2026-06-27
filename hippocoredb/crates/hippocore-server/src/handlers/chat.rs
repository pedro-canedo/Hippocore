use std::convert::Infallible;

use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
    Json,
};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;

use crate::{types::ServerError, AppState};
use hippocore::{BuildContextRequest, SearchMode};

#[derive(Deserialize)]
pub struct ChatRequest {
    pub query: String,
    pub system_prompt_id: Option<String>,
    pub system_prompt: Option<String>,
    pub collection: Option<String>,
    pub max_tokens: Option<usize>,
    pub top_k: Option<usize>,
}

pub async fn chat(
    State(state): State<AppState>,
    Path(tid): Path<String>,
    Json(body): Json<ChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ServerError> {
    let max_tokens = body.max_tokens.unwrap_or(2048);
    let top_k = body.top_k.unwrap_or(5);

    let ctx_req = BuildContextRequest {
        tenant_id: tid.clone(),
        query: body.query.clone(),
        collection: body.collection.clone(),
        max_tokens,
        top_k_candidates: top_k,
        mode: SearchMode::Hybrid,
        user_id: None,
        metadata_filter: hippocore::Metadata::new(),
        include_related: false,
        related_limit: 8,
    };

    let (context_text, sources) = {
        let db = state.db.lock().unwrap();
        let ctx = db.build_context(ctx_req).map_err(ServerError::from)?;
        let text = ctx.text.clone();
        let srcs: Vec<String> = ctx.items_included.iter().map(|i| i.id.clone()).collect();
        (text, srcs)
    };

    let system_prompt = {
        let explicit = body.system_prompt.clone();
        if let Some(s) = explicit {
            s
        } else if let Some(ref pid) = body.system_prompt_id {
            let db = state.db.lock().unwrap();
            let prompts = db.list_prompts(Some(&tid));
            prompts
                .into_iter()
                .find(|p| &p.id == pid)
                .map(|p| p.content)
                .unwrap_or_default()
        } else {
            String::new()
        }
    };

    let full_prompt = if system_prompt.is_empty() {
        format!(
            "You are a helpful AI assistant. Use the context below to answer the question.\n\nContext:\n{context_text}\n\nQuestion: {}",
            body.query
        )
    } else {
        system_prompt
            .replace("{{context}}", &context_text)
            .replace("{{query}}", &body.query)
    };

    let provider = {
        let providers = state.llm_providers.lock().unwrap();
        providers
            .iter()
            .find(|p| p.is_default)
            .or_else(|| providers.first())
            .cloned()
    };

    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(64);
    let http_client = state.http_client.clone();

    tokio::spawn(async move {
        match provider {
            None => {
                let _ = tx
                    .send(Ok(Event::default()
                        .event("error")
                        .data("No LLM provider configured")))
                    .await;
            }
            Some(p) => {
                let result = call_llm_stream(&http_client, &p, &full_prompt, &tx).await;
                if let Err(e) = result {
                    let _ = tx.send(Ok(Event::default().event("error").data(e))).await;
                }
                let done_data = serde_json::json!({ "sources": sources }).to_string();
                let _ = tx
                    .send(Ok(Event::default().event("done").data(done_data)))
                    .await;
            }
        }
    });

    Ok(Sse::new(ReceiverStream::new(rx)))
}

async fn call_llm_stream(
    client: &reqwest::Client,
    provider: &crate::admin::LlmProviderConfig,
    prompt: &str,
    tx: &mpsc::Sender<Result<Event, Infallible>>,
) -> Result<(), String> {
    match provider.kind.as_str() {
        "ollama" => call_ollama(client, provider, prompt, tx).await,
        "openrouter" | "openai" => call_openai_compat(client, provider, prompt, tx).await,
        other => Err(format!("unsupported provider kind: {other}")),
    }
}

async fn call_ollama(
    client: &reqwest::Client,
    provider: &crate::admin::LlmProviderConfig,
    prompt: &str,
    tx: &mpsc::Sender<Result<Event, Infallible>>,
) -> Result<(), String> {
    let url = format!("{}/api/generate", provider.base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": provider.model,
        "prompt": prompt,
        "stream": true,
    });
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Ollama HTTP {}", resp.status()));
    }

    let mut stream = resp.bytes_stream();
    use tokio_stream::StreamExt as _;
    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| e.to_string())?;
        if let Ok(text) = std::str::from_utf8(&bytes) {
            for line in text.lines() {
                if line.is_empty() {
                    continue;
                }
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(token) = val["response"].as_str() {
                        if !token.is_empty() {
                            let _ = tx.send(Ok(Event::default().data(token))).await;
                        }
                    }
                    if val["done"].as_bool().unwrap_or(false) {
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}

async fn call_openai_compat(
    client: &reqwest::Client,
    provider: &crate::admin::LlmProviderConfig,
    prompt: &str,
    tx: &mpsc::Sender<Result<Event, Infallible>>,
) -> Result<(), String> {
    let url = format!(
        "{}/chat/completions",
        provider.base_url.trim_end_matches('/')
    );
    let body = serde_json::json!({
        "model": provider.model,
        "messages": [{"role": "user", "content": prompt}],
        "stream": true,
    });
    let mut req = client.post(&url).json(&body);
    if let Some(ref key) = provider.api_key {
        req = req.bearer_auth(key);
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("LLM HTTP {}", resp.status()));
    }

    let mut stream = resp.bytes_stream();
    use tokio_stream::StreamExt as _;
    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| e.to_string())?;
        if let Ok(text) = std::str::from_utf8(&bytes) {
            for line in text.lines() {
                let line = line.trim();
                if !line.starts_with("data:") {
                    continue;
                }
                let payload = line.trim_start_matches("data:").trim();
                if payload == "[DONE]" {
                    break;
                }
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(payload) {
                    if let Some(token) = val["choices"][0]["delta"]["content"].as_str() {
                        if !token.is_empty() {
                            let _ = tx.send(Ok(Event::default().data(token))).await;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
