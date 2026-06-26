//! Smoke tests driving the compiled `hippocore` binary as a subprocess.

use std::process::Command;

use tempfile::TempDir;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hippocore"))
}

#[test]
fn cli_full_flow_smoke_test() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().to_str().unwrap();

    let init = bin().args(["init", "--db", db]).output().unwrap();
    assert!(init.status.success(), "init failed: {init:?}");

    let remember = bin()
        .args([
            "remember",
            "--db",
            db,
            "--tenant",
            "acme",
            "--collection",
            "support",
            "--type",
            "semantic",
            "--text",
            "Oracle ORA-12514 connection issue in HML",
            "--meta",
            "ambiente=HML",
        ])
        .output()
        .unwrap();
    assert!(remember.status.success(), "remember failed: {remember:?}");

    let put = bin()
        .args([
            "put-document",
            "--db",
            db,
            "--tenant",
            "acme",
            "--collection",
            "support",
            "--text",
            "Oracle listener configuration and tnsnames troubleshooting guide",
        ])
        .output()
        .unwrap();
    assert!(put.status.success(), "put-document failed: {put:?}");

    let recall = bin()
        .args([
            "recall",
            "--db",
            db,
            "--tenant",
            "acme",
            "--query",
            "oracle HML",
            "--top-k",
            "5",
        ])
        .output()
        .unwrap();
    assert!(recall.status.success(), "recall failed: {recall:?}");
    let out = String::from_utf8_lossy(&recall.stdout);
    assert!(out.contains("result(s)"), "unexpected recall output: {out}");

    let stats = bin().args(["stats", "--db", db]).output().unwrap();
    assert!(stats.status.success());
    assert!(String::from_utf8_lossy(&stats.stdout).contains("memories"));

    let inspect = bin().args(["inspect", "--db", db]).output().unwrap();
    assert!(inspect.status.success());
    assert!(String::from_utf8_lossy(&inspect.stdout).contains("acme"));
}

#[test]
fn cli_compact_preserves_data() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().to_str().unwrap();

    for i in 0..3 {
        let out = bin()
            .args([
                "remember",
                "--db",
                db,
                "--tenant",
                "t",
                "--collection",
                "c",
                "--type",
                "note",
                "--id",
                "m1",
                "--text",
                &format!("oracle memory {i}"),
                "--embedding",
                "1,0,0",
            ])
            .output()
            .unwrap();
        assert!(out.status.success());
    }

    let compact = bin().args(["compact", "--db", db]).output().unwrap();
    assert!(compact.status.success(), "compact failed: {compact:?}");
    assert!(String::from_utf8_lossy(&compact.stdout).contains("compacted"));

    // Data is still recallable after compaction.
    let recall = bin()
        .args([
            "recall", "--db", db, "--tenant", "t", "--query", "oracle", "--json",
        ])
        .output()
        .unwrap();
    assert!(recall.status.success());
    assert!(String::from_utf8_lossy(&recall.stdout).contains("\"id\": \"m1\""));
}

#[test]
fn cli_forget_removes_memory() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().to_str().unwrap();

    let store = bin()
        .args([
            "remember",
            "--db",
            db,
            "--tenant",
            "t",
            "--collection",
            "c",
            "--type",
            "note",
            "--id",
            "m1",
            "--text",
            "oracle memory",
            "--embedding",
            "1,0,0",
        ])
        .output()
        .unwrap();
    assert!(store.status.success());

    let forget = bin()
        .args([
            "forget",
            "--db",
            db,
            "--tenant",
            "t",
            "--collection",
            "c",
            "--id",
            "m1",
        ])
        .output()
        .unwrap();
    assert!(forget.status.success(), "forget failed: {forget:?}");

    // It is gone from recall.
    let recall = bin()
        .args([
            "recall", "--db", db, "--tenant", "t", "--query", "oracle", "--json",
        ])
        .output()
        .unwrap();
    assert!(recall.status.success());
    assert_eq!(String::from_utf8_lossy(&recall.stdout).trim(), "[]");
}

#[test]
fn cli_put_and_delete_record() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().to_str().unwrap();

    let put = bin()
        .args([
            "put-record",
            "--db",
            db,
            "--tenant",
            "t",
            "--collection",
            "c",
            "--table",
            "systems",
            "--id",
            "sys-1",
            "--json",
            r#"{"name":"billing-db","engine":"postgresql","port":5432}"#,
            "--meta",
            "env=prod",
        ])
        .output()
        .unwrap();
    assert!(put.status.success(), "put-record failed: {put:?}");

    let recall = bin()
        .args([
            "recall",
            "--db",
            db,
            "--tenant",
            "t",
            "--query",
            "billing postgresql",
            "--kind",
            "record",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(recall.status.success(), "recall failed: {recall:?}");
    let out = String::from_utf8_lossy(&recall.stdout);
    assert!(out.contains("\"kind\": \"record\""), "{out}");
    assert!(out.contains("\"record_table\": \"systems\""), "{out}");

    let delete = bin()
        .args([
            "delete-record",
            "--db",
            db,
            "--tenant",
            "t",
            "--collection",
            "c",
            "--table",
            "systems",
            "--id",
            "sys-1",
        ])
        .output()
        .unwrap();
    assert!(delete.status.success(), "delete-record failed: {delete:?}");

    let recall = bin()
        .args([
            "recall",
            "--db",
            db,
            "--tenant",
            "t",
            "--query",
            "billing postgresql",
            "--kind",
            "record",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(recall.status.success());
    assert_eq!(String::from_utf8_lossy(&recall.stdout).trim(), "[]");
}

#[test]
fn cli_import_and_delete_file() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("db");
    let db = db.to_str().unwrap();
    let file_path = dir.path().join("postgres.md");
    std::fs::write(
        &file_path,
        "PostgreSQL does not use Oracle-style lsnrctl listener.",
    )
    .unwrap();
    let path = file_path.to_str().unwrap();

    let import = bin()
        .args([
            "import-file",
            "--db",
            db,
            "--tenant",
            "t",
            "--collection",
            "c",
            "--id",
            "pg-file",
            "--path",
            path,
            "--meta",
            "topic=postgres",
        ])
        .output()
        .unwrap();
    assert!(import.status.success(), "import-file failed: {import:?}");

    let recall = bin()
        .args([
            "recall",
            "--db",
            db,
            "--tenant",
            "t",
            "--query",
            "postgresql listener",
            "--kind",
            "chunk",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(recall.status.success(), "recall failed: {recall:?}");
    let out = String::from_utf8_lossy(&recall.stdout);
    assert!(out.contains("\"file_id\": \"pg-file\""), "{out}");

    let delete = bin()
        .args([
            "delete-file",
            "--db",
            db,
            "--tenant",
            "t",
            "--collection",
            "c",
            "--id",
            "pg-file",
        ])
        .output()
        .unwrap();
    assert!(delete.status.success(), "delete-file failed: {delete:?}");

    let recall = bin()
        .args([
            "recall",
            "--db",
            db,
            "--tenant",
            "t",
            "--query",
            "postgresql listener",
            "--kind",
            "chunk",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(recall.status.success());
    assert_eq!(String::from_utf8_lossy(&recall.stdout).trim(), "[]");
}

#[test]
fn cli_eval_quality() {
    let dir = TempDir::new().unwrap();

    // Write a minimal fixture.
    let fixture = serde_json::json!({
        "version": 1,
        "memories": [
            {
                "id": "pg-port",
                "text": "PostgreSQL listens on TCP port 5432 by default",
                "metadata": { "product": "postgresql" }
            },
            {
                "id": "ora-port",
                "text": "Oracle listener uses port 1521 by default",
                "metadata": { "product": "oracle" }
            }
        ],
        "cases": [
            {
                "name": "postgres_port",
                "query": "What port does PostgreSQL use?",
                "relevant_ids": ["pg-port"],
                "expected_first_ids": ["pg-port"],
                "forbidden_first_ids": ["ora-port"],
                "answer_must_include": ["5432"]
            }
        ],
        "thresholds": {
            "min_hit_at_1": 0.8,
            "min_hit_at_k": 0.8,
            "min_mrr": 0.8
        }
    });
    let fixture_path = dir.path().join("fixture.json");
    std::fs::write(&fixture_path, fixture.to_string()).unwrap();

    // Run eval-quality (temp DB, no --db).
    let out = bin()
        .args(["eval-quality", "--fixture", fixture_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "eval-quality failed: {out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("PASS"), "expected PASS: {stdout}");

    // Run with --json and check structure.
    let out = bin()
        .args([
            "eval-quality",
            "--fixture",
            fixture_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert_eq!(json["pass"], true);
    assert!(json["scenarios"].as_array().unwrap().len() == 1);
    assert_eq!(json["scenarios"][0]["name"], "postgres_port");
    assert_eq!(json["scenarios"][0]["pass"], true);
}

#[test]
fn cli_eval_quality_fails_on_bad_thresholds() {
    let dir = TempDir::new().unwrap();
    // Fixture with impossible thresholds to ensure FAIL exit code.
    let fixture = serde_json::json!({
        "version": 1,
        "memories": [
            { "id": "m1", "text": "generic text about systems", "metadata": {} }
        ],
        "cases": [
            {
                "name": "impossible",
                "query": "something completely unrelated to the memories",
                "relevant_ids": ["never-exists"],
                "expected_first_ids": [],
                "forbidden_first_ids": [],
                "answer_must_include": []
            }
        ],
        "thresholds": {
            "min_hit_at_1": 1.0,
            "min_hit_at_k": 1.0,
            "min_mrr": 1.0
        }
    });
    let fixture_path = dir.path().join("impossible.json");
    std::fs::write(&fixture_path, fixture.to_string()).unwrap();

    let out = bin()
        .args(["eval-quality", "--fixture", fixture_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success(), "should fail when thresholds not met");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("FAIL"), "expected FAIL output: {stdout}");
}

#[test]
fn cli_bad_metadata_returns_nonzero() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().to_str().unwrap();
    let out = bin()
        .args([
            "remember",
            "--db",
            db,
            "--tenant",
            "t",
            "--collection",
            "c",
            "--text",
            "hello",
            "--meta",
            "bad_pair_without_equals",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn cli_admin_list_and_show() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().to_str().unwrap();

    // Seed a memory and a document.
    let mem = bin()
        .args([
            "remember",
            "--db",
            db,
            "--tenant",
            "acme",
            "--collection",
            "kb",
            "--type",
            "semantic",
            "--id",
            "m1",
            "--text",
            "PostgreSQL listens on port 5432 by default",
        ])
        .output()
        .unwrap();
    assert!(mem.status.success(), "remember: {mem:?}");

    let put = bin()
        .args([
            "put-document",
            "--db",
            db,
            "--tenant",
            "acme",
            "--collection",
            "kb",
            "--id",
            "doc1",
            "--text",
            "PostgreSQL connection guide",
        ])
        .output()
        .unwrap();
    assert!(put.status.success(), "put-document: {put:?}");

    // list-tenants
    let out = bin().args(["list-tenants", "--db", db]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("acme"), "list-tenants: {stdout}");

    // list-tenants --json
    let out = bin()
        .args(["list-tenants", "--db", db, "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert!(json.is_array());
    assert_eq!(json.as_array().unwrap().len(), 1);

    // list-collections
    let out = bin()
        .args(["list-collections", "--db", db, "--tenant", "acme"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("kb"),
        "list-collections"
    );

    // list-documents
    let out = bin()
        .args(["list-documents", "--db", db, "--tenant", "acme"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("doc1"),
        "list-documents"
    );

    // list-memories
    let out = bin()
        .args(["list-memories", "--db", db, "--tenant", "acme"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("m1"), "list-memories: {stdout}");

    // show-document
    let out = bin()
        .args([
            "show-document",
            "--db",
            db,
            "--tenant",
            "acme",
            "--collection",
            "kb",
            "--id",
            "doc1",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("PostgreSQL connection guide"),
        "show-document: {stdout}"
    );
    assert!(stdout.contains("chunk"), "show-document chunks: {stdout}");

    // show-document --json
    let out = bin()
        .args([
            "show-document",
            "--db",
            db,
            "--tenant",
            "acme",
            "--collection",
            "kb",
            "--id",
            "doc1",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert!(json["document"]["id"] == "doc1");
    assert!(json["chunks"].is_array());

    // show-memory
    let out = bin()
        .args([
            "show-memory",
            "--db",
            db,
            "--tenant",
            "acme",
            "--collection",
            "kb",
            "--id",
            "m1",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("PostgreSQL"),
        "show-memory"
    );

    // show-document unknown id → nonzero exit
    let out = bin()
        .args([
            "show-document",
            "--db",
            db,
            "--tenant",
            "acme",
            "--collection",
            "kb",
            "--id",
            "nonexistent",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success(), "should fail for unknown id");
}

#[test]
fn cli_admin_list_records_and_show() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().to_str().unwrap();

    let put = bin()
        .args([
            "put-record",
            "--db",
            db,
            "--tenant",
            "t",
            "--collection",
            "c",
            "--table",
            "systems",
            "--id",
            "db1",
            "--json",
            r#"{"engine":"postgresql","port":5432,"env":"prod"}"#,
        ])
        .output()
        .unwrap();
    assert!(put.status.success(), "put-record: {put:?}");

    // list-records
    let out = bin()
        .args([
            "list-records",
            "--db",
            db,
            "--tenant",
            "t",
            "--table",
            "systems",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("db1"),
        "list-records"
    );

    // show-record
    let out = bin()
        .args([
            "show-record",
            "--db",
            db,
            "--tenant",
            "t",
            "--collection",
            "c",
            "--table",
            "systems",
            "--id",
            "db1",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("postgresql"), "show-record: {stdout}");
    assert!(
        stdout.contains("projection"),
        "show-record projection: {stdout}"
    );

    // show-record --json
    let out = bin()
        .args([
            "show-record",
            "--db",
            db,
            "--tenant",
            "t",
            "--collection",
            "c",
            "--table",
            "systems",
            "--id",
            "db1",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert_eq!(json["id"], "db1");
    assert_eq!(json["table"], "systems");
    assert_eq!(json["payload"]["engine"], "postgresql");
}

#[test]
fn cli_admin_list_files_and_show() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("db");
    let db = db.to_str().unwrap();
    let file_path = dir.path().join("guide.md");
    std::fs::write(&file_path, "PostgreSQL guide for operators").unwrap();
    let path = file_path.to_str().unwrap();

    let import = bin()
        .args([
            "import-file",
            "--db",
            db,
            "--tenant",
            "t",
            "--collection",
            "c",
            "--id",
            "f1",
            "--path",
            path,
        ])
        .output()
        .unwrap();
    assert!(import.status.success(), "import-file: {import:?}");

    // list-files
    let out = bin()
        .args(["list-files", "--db", db, "--tenant", "t"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("f1"),
        "list-files"
    );

    // show-file
    let out = bin()
        .args([
            "show-file",
            "--db",
            db,
            "--tenant",
            "t",
            "--collection",
            "c",
            "--id",
            "f1",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("guide.md"), "show-file name: {stdout}");
    assert!(stdout.contains("document"), "show-file doc: {stdout}");

    // show-file --json
    let out = bin()
        .args([
            "show-file",
            "--db",
            db,
            "--tenant",
            "t",
            "--collection",
            "c",
            "--id",
            "f1",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert_eq!(json["id"], "f1");
    assert_eq!(json["name"], "guide.md");
    assert!(json["document_id"].as_str().unwrap().starts_with("file:"));
}

#[test]
fn cli_bad_memory_type_returns_nonzero() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().to_str().unwrap();
    let out = bin()
        .args([
            "remember",
            "--db",
            db,
            "--tenant",
            "t",
            "--collection",
            "c",
            "--type",
            "nonsense",
            "--text",
            "hello",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn cli_studio_exits_without_panic() {
    // The TUI cannot be driven headlessly, but we verify the binary doesn't
    // panic when `studio` is invoked without a real terminal. It will fail at
    // `enable_raw_mode` (no TTY in CI) — we just assert no signal termination.
    let dir = TempDir::new().unwrap();
    let db = dir.path().to_str().unwrap();

    let out = bin().args(["studio", "--db", db]).output().unwrap();
    assert!(
        out.status.code().is_some(),
        "studio exited via signal (panic?): {out:?}"
    );
}
