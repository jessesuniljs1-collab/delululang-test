//! End-to-end tests for `delulu audit tail|query|verify` (Stage 5 phase 5d). The CLI reads the
//! hash-chained log files directly — no daemon (chunk 3). Logs are produced here through the real
//! `delulu-broker` stack (Broker + AuditLog) into a temp `--dir`, then inspected via the binary.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::rc::Rc;

use delulu_broker::{AuditLog, Authority, Broker, Holder, ManualClock, Op, Scopes, SeqIdSource};
use delulu_check::Effect;
use serde_json::Value;

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        .output()
        .expect("failed to run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

fn eff(names: &[&str]) -> BTreeSet<Effect> {
    names.iter().map(|n| Effect::core_from_name(n).unwrap()).collect()
}
fn names(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// Populate a fresh temp audit dir via the real broker stack: issue → allowed write → denied write
/// → revoke (4 records), then return the dir.
fn seeded_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_audit_cli_{}_{}", std::process::id(), tag));
    let _ = std::fs::remove_dir_all(&dir);
    let log = AuditLog::open(&dir).unwrap();
    let clock = Rc::new(ManualClock::new(1_752_192_000_000)); // 2025-07-11-ish, any fixed epoch ms
    let mut b = Broker::with_sources(Box::new(SeqIdSource::new()), Box::new(clock))
        .with_sink(Box::new(log));
    let root = b.issue(
        Holder::new("process", "cli-test", "pid:0"),
        Authority::new(eff(&["Write"]), Scopes { fs_write: names(&["./out"]), ..Default::default() }),
        None,
    );
    assert!(b.check(&root, Op::FsWrite, Some("./out/a.txt")).is_allow());
    assert!(!b.check(&root, Op::FsWrite, Some("./secret")).is_allow());
    b.revoke(&root, &root).unwrap();
    dir
}

#[test]
fn audit_verify_passes_on_a_clean_log_and_json_has_the_stats() {
    let dir = seeded_dir("verify_ok");
    let d = dir.to_string_lossy().to_string();

    let o = delulu(&["audit", "verify", "--dir", &d]);
    assert!(o.status.success(), "verify should pass: {}", stderr(&o));
    assert!(stderr(&o).contains("audit chain verified"), "{}", stderr(&o));

    let o = delulu(&["audit", "verify", "--dir", &d, "--json"]);
    let v: Value = serde_json::from_str(&stdout(&o)).expect("verify --json is valid JSON");
    assert_eq!(v["ok"], true);
    assert_eq!(v["records"], 4, "issue + allowed use + denied use + revoke");
    assert_eq!(v["files"], 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn audit_verify_fails_dl1405_on_a_corrupted_record() {
    let dir = seeded_dir("verify_bad");
    let d = dir.to_string_lossy().to_string();

    // Corrupt one byte of the seq-2 record on disk (keeps the line valid JSON).
    let file = std::fs::read_dir(&dir).unwrap().next().unwrap().unwrap().path();
    let text = std::fs::read_to_string(&file).unwrap();
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let idx = lines.iter().position(|l| l.contains("\"seq\":2")).expect("seq-2 record exists");
    lines[idx] = lines[idx].replacen("./out/a.txt", "./out/b.txt", 1);
    std::fs::write(&file, lines.join("\n") + "\n").unwrap();

    let o = delulu(&["audit", "verify", "--dir", &d]);
    assert_eq!(o.status.code(), Some(1), "a broken chain exits 1");
    assert!(stderr(&o).contains("DL1405"), "human output names DL1405: {}", stderr(&o));

    let o = delulu(&["audit", "verify", "--dir", &d, "--json"]);
    let v: Value = serde_json::from_str(&stdout(&o)).expect("verify --json is valid JSON");
    assert_eq!(v["diagnostics"][0]["code"], "DL1405");
    let msg = v["diagnostics"][0]["message"].as_str().unwrap();
    assert!(msg.contains("seq 2"), "DL1405 registers the failing seq: {msg}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn audit_tail_and_query_read_the_records() {
    let dir = seeded_dir("tail_query");
    let d = dir.to_string_lossy().to_string();

    // tail 2 → the last two records (denied use + revoke), oldest first.
    let o = delulu(&["audit", "tail", "2", "--dir", &d, "--json"]);
    let v: Value = serde_json::from_str(&stdout(&o)).expect("tail --json is valid JSON");
    assert_eq!(v["count"], 2);
    assert_eq!(v["records"][0]["action"], "use");
    assert_eq!(v["records"][0]["decision"], "deny");
    assert_eq!(v["records"][1]["action"], "revoke");

    // Human tail prints one line per record with seq + action.
    let o = delulu(&["audit", "tail", "--dir", &d]);
    let out = stdout(&o);
    assert!(out.contains("issue") && out.contains("revoke"), "{out}");

    // query --action use → exactly the two synchronous-use records.
    let o = delulu(&["audit", "query", "--action", "use", "--dir", &d, "--json"]);
    let v: Value = serde_json::from_str(&stdout(&o)).expect("query --json is valid JSON");
    assert_eq!(v["count"], 2);

    // query --effect Write → the one record whose authority carries Write (the issue).
    let o = delulu(&["audit", "query", "--effect", "Write", "--dir", &d, "--json"]);
    let v: Value = serde_json::from_str(&stdout(&o)).expect("query --json is valid JSON");
    assert_eq!(v["count"], 1);
    assert_eq!(v["records"][0]["action"], "issue");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn audit_log_header_states_observability_not_enforcement() {
    // Spec §7: the FIRST line of every log file states the log is observability, not enforcement.
    let dir = seeded_dir("header");
    let file = std::fs::read_dir(&dir).unwrap().next().unwrap().unwrap().path();
    let text = std::fs::read_to_string(&file).unwrap();
    let first = text.lines().next().unwrap();
    assert!(
        first.contains("observability, not enforcement"),
        "header line must state the exact phrase: {first}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn audit_needs_a_subcommand_and_rejects_unknown_ones() {
    let o = delulu(&["audit"]);
    assert_eq!(o.status.code(), Some(2));
    let o = delulu(&["audit", "bogus"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(stderr(&o).contains("unknown audit subcommand"), "{}", stderr(&o));
}
