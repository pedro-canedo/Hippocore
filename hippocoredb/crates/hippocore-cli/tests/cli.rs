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
