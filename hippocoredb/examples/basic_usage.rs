//! End-to-end example of the Hippocore memory database.
//!
//! Run with: `cargo run -p hippocore --example basic_usage`

use hippocore::model::MemoryType;
use hippocore::{Config, Hippocore, RecallRequest, RememberRequest, StoreDocumentRequest};

fn main() -> hippocore::Result<()> {
    let dir = std::env::temp_dir().join("hippocore-basic-usage");
    let _ = std::fs::remove_dir_all(&dir);

    let mut db = Hippocore::open(Config::new(&dir))?;

    db.create_tenant("acme", "Acme Corp")?;
    db.create_collection("acme", "support", "support knowledge base")?;

    // Store a document (chunked + embedded automatically).
    let mut doc = StoreDocumentRequest::new(
        "acme",
        "support",
        "Oracle ORA-12514 means the listener does not know the requested service. \
         Check the service name in tnsnames.ora and the listener status.",
    );
    doc.metadata
        .insert("ambiente".to_string(), "HML".to_string());
    db.store_document(doc)?;

    // Store a long-term memory.
    let mut mem = RememberRequest::new(
        "acme",
        "support",
        MemoryType::Semantic,
        "A customer running Oracle in the staging environment hit ORA-12514.",
    );
    mem.user_id = Some("agent-1".into());
    db.remember(mem)?;

    // Hybrid recall.
    let hits = db.recall(RecallRequest::new("acme", "oracle ORA-12514 in HML"))?;
    println!("recall returned {} hit(s):", hits.len());
    for h in &hits {
        println!(
            "  [{:?}] {} score={:.3} ({})",
            h.kind, h.id, h.score, h.reason
        );
    }

    println!("{}", db.stats()?);
    db.close()?;
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}
