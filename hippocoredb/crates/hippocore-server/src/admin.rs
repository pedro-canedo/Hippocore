use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Json, Redirect},
};
use serde::{Deserialize, Serialize};

use crate::{types::ServerError, AppState};

pub async fn root() -> Redirect {
    Redirect::to("/admin")
}

pub async fn page() -> Html<&'static str> {
    Html(ADMIN_HTML)
}

#[derive(Serialize)]
pub struct AdminConfigResponse {
    pub api_key_set: bool,
    pub api_key_length: usize,
    pub data_dir: String,
}

#[derive(Serialize)]
pub struct AdminBootstrapResponse {
    pub config: AdminConfigResponse,
    pub stats: serde_json::Value,
    pub tenants: Vec<hippocore::Tenant>,
}

#[derive(Serialize)]
pub struct RotatedKeyResponse {
    pub api_key: String,
}

#[derive(Deserialize, Default)]
pub struct CollectionQuery {
    pub tenant_id: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct EntityQuery {
    pub tenant_id: Option<String>,
    pub collection: Option<String>,
    pub table: Option<String>,
    pub from_id: Option<String>,
}

#[derive(Deserialize)]
pub struct RotateKeyRequest {
    pub length: Option<usize>,
}

pub async fn config(
    State(state): State<AppState>,
) -> Result<Json<AdminConfigResponse>, ServerError> {
    let key = state.api_key.lock().unwrap();
    let db = state.db.lock().unwrap();
    let data_dir = db.config().data_dir.to_string_lossy().to_string();
    Ok(Json(AdminConfigResponse {
        api_key_set: !key.is_empty(),
        api_key_length: key.len(),
        data_dir,
    }))
}

pub async fn bootstrap(
    State(state): State<AppState>,
) -> Result<Json<AdminBootstrapResponse>, ServerError> {
    let config = {
        let key = state.api_key.lock().unwrap();
        let db = state.db.lock().unwrap();
        AdminConfigResponse {
            api_key_set: !key.is_empty(),
            api_key_length: key.len(),
            data_dir: db.config().data_dir.to_string_lossy().to_string(),
        }
    };
    let (stats, tenants) = {
        let db = state.db.lock().unwrap();
        let stats = db.stats().map_err(ServerError::from)?;
        (stats, db.tenants())
    };
    Ok(Json(AdminBootstrapResponse {
        stats: serde_json::json!({
            "tenants": stats.tenants,
            "collections": stats.collections,
            "documents": stats.documents,
            "chunks": stats.chunks,
            "memories": stats.memories,
            "records": stats.records,
            "files": stats.files,
            "graph_edges": stats.graph_edges,
            "indexed_entries": stats.indexed_entries,
            "wal_entries": stats.wal_entries,
            "disk_bytes": stats.disk_bytes,
            "audit_records": stats.audit_records,
            "audit_log_bytes": stats.audit_log_bytes,
        }),
        config,
        tenants,
    }))
}

pub async fn rotate_api_key(
    State(state): State<AppState>,
    Json(body): Json<RotateKeyRequest>,
) -> Result<Json<RotatedKeyResponse>, ServerError> {
    let len = body.length.unwrap_or(32).max(16);
    let api_key = generate_api_key(len)?;
    *state.api_key.lock().unwrap() = api_key.clone();
    Ok(Json(RotatedKeyResponse { api_key }))
}

pub async fn tenants(
    State(state): State<AppState>,
) -> Result<Json<Vec<hippocore::Tenant>>, ServerError> {
    let db = state.db.lock().unwrap();
    Ok(Json(db.tenants()))
}

pub async fn collections(
    State(state): State<AppState>,
    Query(query): Query<CollectionQuery>,
) -> Result<Json<Vec<hippocore::Collection>>, ServerError> {
    let db = state.db.lock().unwrap();
    let items = match query.tenant_id {
        Some(ref tenant_id) => db.collections(tenant_id),
        None => db.all_collections(),
    };
    Ok(Json(items))
}

pub async fn memories(
    State(state): State<AppState>,
    Query(query): Query<EntityQuery>,
) -> Result<Json<Vec<hippocore::Memory>>, ServerError> {
    let tenant_id = query
        .tenant_id
        .ok_or_else(|| ServerError(StatusCode::BAD_REQUEST, "tenant_id required".into()))?;
    let db = state.db.lock().unwrap();
    Ok(Json(
        db.list_memories(&tenant_id, query.collection.as_deref()),
    ))
}

pub async fn documents(
    State(state): State<AppState>,
    Query(query): Query<EntityQuery>,
) -> Result<Json<Vec<hippocore::Document>>, ServerError> {
    let tenant_id = query
        .tenant_id
        .ok_or_else(|| ServerError(StatusCode::BAD_REQUEST, "tenant_id required".into()))?;
    let db = state.db.lock().unwrap();
    Ok(Json(
        db.list_documents(&tenant_id, query.collection.as_deref()),
    ))
}

pub async fn records(
    State(state): State<AppState>,
    Query(query): Query<EntityQuery>,
) -> Result<Json<Vec<hippocore::Record>>, ServerError> {
    let tenant_id = query
        .tenant_id
        .ok_or_else(|| ServerError(StatusCode::BAD_REQUEST, "tenant_id required".into()))?;
    let db = state.db.lock().unwrap();
    Ok(Json(db.list_records(
        &tenant_id,
        query.collection.as_deref(),
        query.table.as_deref(),
    )))
}

pub async fn files(
    State(state): State<AppState>,
    Query(query): Query<EntityQuery>,
) -> Result<Json<Vec<hippocore::FileObject>>, ServerError> {
    let tenant_id = query
        .tenant_id
        .ok_or_else(|| ServerError(StatusCode::BAD_REQUEST, "tenant_id required".into()))?;
    let db = state.db.lock().unwrap();
    Ok(Json(db.list_files(&tenant_id, query.collection.as_deref())))
}

pub async fn graph_edges(
    State(state): State<AppState>,
    Query(query): Query<EntityQuery>,
) -> Result<Json<Vec<hippocore::GraphEdge>>, ServerError> {
    let tenant_id = query
        .tenant_id
        .ok_or_else(|| ServerError(StatusCode::BAD_REQUEST, "tenant_id required".into()))?;
    let db = state.db.lock().unwrap();
    Ok(Json(
        db.list_graph_edges(&tenant_id, query.from_id.as_deref()),
    ))
}

fn generate_api_key(len: usize) -> Result<String, ServerError> {
    let byte_len = len.max(16);
    let mut bytes = vec![0u8; byte_len];
    getrandom::fill(&mut bytes).map_err(|e| {
        ServerError(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to generate api key: {e}"),
        )
    })?;
    Ok(bytes.into_iter().map(|b| format!("{:02x}", b)).collect())
}

const ADMIN_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Hippocore Studio</title>
  <style>
    :root { color-scheme: dark; --bg: #0c1116; --panel: #121923; --line: #233042; --text: #e8eef6; --muted: #9ba8b7; --accent: #7dd3fc; --good: #4ade80; --warn: #f59e0b; }
    body { margin: 0; font: 14px/1.4 system-ui, sans-serif; background: linear-gradient(180deg, #091018, #0c1116); color: var(--text); }
    header { padding: 20px 24px; border-bottom: 1px solid var(--line); display: flex; justify-content: space-between; align-items: center; gap: 16px; }
    main { padding: 24px; display: grid; gap: 16px; grid-template-columns: 1.2fr 1fr; }
    section { background: rgba(18,25,35,.92); border: 1px solid var(--line); border-radius: 10px; padding: 16px; }
    h1,h2,h3 { margin: 0 0 12px; font-weight: 650; }
    h1 { font-size: 20px; }
    h2 { font-size: 15px; }
    label { display: block; font-size: 12px; color: var(--muted); margin: 10px 0 4px; }
    input, textarea, button { width: 100%; box-sizing: border-box; border-radius: 8px; border: 1px solid var(--line); background: #0b121a; color: var(--text); padding: 10px 12px; }
    textarea { min-height: 88px; resize: vertical; }
    button { cursor: pointer; background: #132033; }
    button.primary { background: #1c3348; border-color: #2c5879; }
    .grid { display: grid; gap: 10px; grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .muted { color: var(--muted); }
    .row { display: flex; gap: 8px; align-items: center; }
    .row > * { flex: 1; }
    pre { margin: 0; white-space: pre-wrap; word-break: break-word; background: #091018; border: 1px solid var(--line); border-radius: 8px; padding: 10px; overflow: auto; }
    .span-2 { grid-column: span 2; }
    .small { font-size: 12px; }
    .pill { display: inline-block; padding: 3px 8px; border-radius: 999px; background: #132033; border: 1px solid var(--line); margin-right: 6px; margin-bottom: 6px; }
    .stack { display: grid; gap: 10px; }
  </style>
</head>
<body>
  <header>
    <div>
      <h1>Hippocore Studio</h1>
      <div class="muted">Local-first admin console for core data, recall, context, and API key rotation.</div>
    </div>
    <div class="row" style="max-width: 360px;">
      <input id="apiKey" placeholder="API key" />
      <button id="saveKey" class="primary">Use key</button>
    </div>
  </header>
  <main>
    <section class="span-2">
      <h2>Status</h2>
      <div class="grid">
        <div><div class="muted small">API key</div><div id="apiKeyStatus">not loaded</div></div>
        <div><div class="muted small">Data dir</div><div id="dataDir">-</div></div>
        <div><div class="muted small">Tenants</div><div id="tenantCount">-</div></div>
        <div><div class="muted small">Documents / memories / records</div><div id="counts">-</div></div>
      </div>
      <pre id="stats">Connect with an API key to load status.</pre>
    </section>

    <section>
      <h2>Browse</h2>
      <div class="stack">
        <div class="row">
          <button id="refresh">Refresh</button>
          <button id="rotate">Rotate key</button>
        </div>
        <div>
          <label>Tenants</label>
          <div id="tenants" class="stack"></div>
        </div>
        <div>
          <label>Collections</label>
          <div id="collections" class="stack"></div>
        </div>
        <div>
          <label>Objects</label>
          <div class="row">
            <button data-kind="memories">Memories</button>
            <button data-kind="documents">Documents</button>
            <button data-kind="records">Records</button>
            <button data-kind="files">Files</button>
          </div>
          <pre id="objects">Select a tenant, then a kind.</pre>
        </div>
      </div>
    </section>

    <section>
      <h2>Create & Test</h2>
      <div class="stack">
        <div class="grid">
          <div>
            <label>New tenant id</label>
            <input id="newTenantId" placeholder="acme" />
          </div>
          <div>
            <label>New tenant name</label>
            <input id="newTenantName" placeholder="Acme Corp" />
          </div>
        </div>
        <button id="createTenant">Create tenant</button>
        <div class="grid">
          <div>
            <label>New collection name</label>
            <input id="newCollectionName" placeholder="support" />
          </div>
          <div>
            <label>New collection description</label>
            <input id="newCollectionDescription" placeholder="support knowledge base" />
          </div>
        </div>
        <button id="createCollection">Create collection</button>
        <div class="grid">
          <div>
            <label>Tenant id</label>
            <input id="tenantId" placeholder="acme" />
          </div>
          <div>
            <label>Collection</label>
            <input id="collection" placeholder="support" />
          </div>
        </div>
        <label>Memory text</label>
        <textarea id="memoryText" placeholder="Oracle ORA-12514 means ..."></textarea>
        <button id="createMemory" class="primary">Store memory</button>
        <label>Document text</label>
        <textarea id="documentText" placeholder="Paste document text here"></textarea>
        <button id="createDocument" class="primary">Store document</button>
        <label>Recall query</label>
        <input id="query" placeholder="oracle connection" />
        <button id="runRecall">Run recall</button>
        <button id="runContext">Build context</button>
        <pre id="result">Ready.</pre>
      </div>
    </section>

    <section>
      <h2>Ollama integration helper</h2>
      <div class="grid">
        <div>
          <label>Ollama URL</label>
          <input id="ollamaUrl" placeholder="http://localhost:11434" />
        </div>
        <div>
          <label>Embed model</label>
          <input id="embedModel" placeholder="nomic-embed-text" />
        </div>
        <div>
          <label>Chat model</label>
          <input id="chatModel" placeholder="llama3.2" />
        </div>
        <div>
          <label>API key</label>
          <input id="ollamaKey" placeholder="optional" />
        </div>
        <div class="span-2">
          <button id="copyEnv">Copy env snippet</button>
        </div>
      </div>
      <pre id="envSnippet">Set the fields above to generate an integration snippet.</pre>
    </section>
  </main>
  <script>
    const els = {
      apiKey: document.getElementById('apiKey'),
      apiKeyStatus: document.getElementById('apiKeyStatus'),
      dataDir: document.getElementById('dataDir'),
      tenantCount: document.getElementById('tenantCount'),
      counts: document.getElementById('counts'),
      stats: document.getElementById('stats'),
      tenants: document.getElementById('tenants'),
      collections: document.getElementById('collections'),
      objects: document.getElementById('objects'),
      result: document.getElementById('result'),
      tenantId: document.getElementById('tenantId'),
      collection: document.getElementById('collection'),
      newTenantId: document.getElementById('newTenantId'),
      newTenantName: document.getElementById('newTenantName'),
      newCollectionName: document.getElementById('newCollectionName'),
      newCollectionDescription: document.getElementById('newCollectionDescription'),
      memoryText: document.getElementById('memoryText'),
      documentText: document.getElementById('documentText'),
      query: document.getElementById('query'),
      ollamaUrl: document.getElementById('ollamaUrl'),
      embedModel: document.getElementById('embedModel'),
      chatModel: document.getElementById('chatModel'),
      ollamaKey: document.getElementById('ollamaKey'),
      envSnippet: document.getElementById('envSnippet'),
    };
    const state = { key: localStorage.getItem('hippocore.apiKey') || '', tenant: '', kind: 'memories' };
    els.apiKey.value = state.key;
    els.ollamaUrl.value = localStorage.getItem('hippocore.ollamaUrl') || 'http://localhost:11434';
    els.embedModel.value = localStorage.getItem('hippocore.embedModel') || 'nomic-embed-text';
    els.chatModel.value = localStorage.getItem('hippocore.chatModel') || 'llama3.2';
    els.ollamaKey.value = localStorage.getItem('hippocore.ollamaKey') || '';

    function headers(extra={}) {
      return Object.assign({'content-type': 'application/json', 'x-api-key': state.key}, extra);
    }
    function saveSettings() {
      localStorage.setItem('hippocore.apiKey', state.key);
      localStorage.setItem('hippocore.ollamaUrl', els.ollamaUrl.value.trim());
      localStorage.setItem('hippocore.embedModel', els.embedModel.value.trim());
      localStorage.setItem('hippocore.chatModel', els.chatModel.value.trim());
      localStorage.setItem('hippocore.ollamaKey', els.ollamaKey.value.trim());
    }
    function show(obj) { els.result.textContent = typeof obj === 'string' ? obj : JSON.stringify(obj, null, 2); }
    function envSnippet() {
      return [
        `HIPPOCORE_API_KEY=${state.key || '<set-a-key>'}`,
        `OLLAMA_URL=${els.ollamaUrl.value.trim()}`,
        `EMBED_MODEL=${els.embedModel.value.trim()}`,
        `CHAT_MODEL=${els.chatModel.value.trim()}`,
      ].join('\n');
    }
    async function api(path, options = {}) {
      const res = await fetch(path, Object.assign({ headers: headers() }, options));
      const text = await res.text();
      const body = text ? JSON.parse(text) : null;
      if (!res.ok) throw new Error(body?.error || text || `HTTP ${res.status}`);
      return body;
    }
    async function loadBootstrap() {
      if (!state.key) {
        els.apiKeyStatus.textContent = 'enter API key';
        return;
      }
      const data = await api('/admin/bootstrap');
      els.apiKeyStatus.textContent = data.config.api_key_set ? `loaded (${data.config.api_key_length} chars)` : 'missing';
      els.dataDir.textContent = data.config.data_dir;
      els.tenantCount.textContent = data.tenants.length;
      els.counts.textContent = `${data.stats.documents} docs, ${data.stats.memories} memories, ${data.stats.records} records`;
      els.stats.textContent = JSON.stringify(data.stats, null, 2);
      renderTenants(data.tenants);
      show({ bootstrap: data });
    }
    function renderTenants(tenants) {
      els.tenants.innerHTML = '';
      tenants.forEach(t => {
        const b = document.createElement('button');
        b.textContent = `${t.id} — ${t.name}`;
        b.onclick = async () => {
          state.tenant = t.id;
          els.tenantId.value = t.id;
          await loadCollections();
        };
        els.tenants.appendChild(b);
      });
    }
    async function loadCollections() {
      if (!state.tenant) return;
      const items = await api(`/admin/collections?tenant_id=${encodeURIComponent(state.tenant)}`);
      els.collections.innerHTML = '';
      items.forEach(c => {
        const pill = document.createElement('div');
        pill.className = 'pill';
        pill.textContent = `${c.name}${c.description ? ' — ' + c.description : ''}`;
        els.collections.appendChild(pill);
      });
    }
    async function loadObjects(kind) {
      if (!state.tenant) throw new Error('select a tenant first');
      const q = new URLSearchParams({ tenant_id: state.tenant });
      if (els.collection.value.trim()) q.set('collection', els.collection.value.trim());
      if (kind === 'records' && document.getElementById('table')) q.set('table', document.getElementById('table').value.trim());
      const items = await api(`/admin/${kind}?${q.toString()}`);
      els.objects.textContent = JSON.stringify(items, null, 2);
    }
    document.getElementById('saveKey').onclick = async () => {
      state.key = els.apiKey.value.trim();
      saveSettings();
      try { await loadBootstrap(); } catch (e) { show(String(e)); }
    };
    document.getElementById('refresh').onclick = async () => {
      saveSettings();
      try { await loadBootstrap(); } catch (e) { show(String(e)); }
    };
    document.getElementById('rotate').onclick = async () => {
      const data = await api('/admin/api-key/rotate', { method: 'POST', body: JSON.stringify({ length: 32 }) });
      state.key = data.api_key;
      els.apiKey.value = data.api_key;
      saveSettings();
      await loadBootstrap();
      show({ rotated: true, api_key: data.api_key });
    };
    document.getElementById('createTenant').onclick = async () => {
      const body = { id: els.newTenantId.value.trim(), name: els.newTenantName.value.trim() };
      show(await api('/tenants', { method: 'POST', body: JSON.stringify(body) }));
      await loadBootstrap();
    };
    document.getElementById('createCollection').onclick = async () => {
      const tenant = els.tenantId.value.trim() || els.newTenantId.value.trim();
      const body = { name: els.newCollectionName.value.trim(), description: els.newCollectionDescription.value.trim() };
      show(await api(`/tenants/${encodeURIComponent(tenant)}/collections`, { method: 'POST', body: JSON.stringify(body) }));
      await loadBootstrap();
    };
    document.getElementById('createMemory').onclick = async () => {
      const body = { collection: els.collection.value.trim(), text: els.memoryText.value.trim(), memory_type: 'semantic' };
      const data = await api(`/tenants/${encodeURIComponent(els.tenantId.value.trim())}/memories`, { method: 'POST', body: JSON.stringify(body) });
      show(data);
    };
    document.getElementById('createDocument').onclick = async () => {
      const body = { collection: els.collection.value.trim(), text: els.documentText.value.trim() };
      const data = await api(`/tenants/${encodeURIComponent(els.tenantId.value.trim())}/documents`, { method: 'POST', body: JSON.stringify(body) });
      show(data);
    };
    document.getElementById('runRecall').onclick = async () => {
      const body = { query: els.query.value.trim(), collection: els.collection.value.trim() || null, top_k: 5 };
      show(await api(`/tenants/${encodeURIComponent(els.tenantId.value.trim())}/recall`, { method: 'POST', body: JSON.stringify(body) }));
    };
    document.getElementById('runContext').onclick = async () => {
      const body = { query: els.query.value.trim(), max_tokens: 1024, include_related: true };
      show(await api(`/tenants/${encodeURIComponent(els.tenantId.value.trim())}/context`, { method: 'POST', body: JSON.stringify(body) }));
    };
    document.querySelectorAll('button[data-kind]').forEach(btn => {
      btn.onclick = () => loadObjects(btn.dataset.kind).catch(e => show(String(e)));
    });
    document.getElementById('copyEnv').onclick = async () => {
      const text = envSnippet();
      await navigator.clipboard.writeText(text);
      els.envSnippet.textContent = text;
    };
    [els.ollamaUrl, els.embedModel, els.chatModel, els.ollamaKey].forEach(el => {
      el.addEventListener('input', () => {
        saveSettings();
        els.envSnippet.textContent = envSnippet();
      });
    });
    els.envSnippet.textContent = envSnippet();
    if (state.key) loadBootstrap().catch(e => show(String(e)));
  </script>
</body>
</html>
"#;
