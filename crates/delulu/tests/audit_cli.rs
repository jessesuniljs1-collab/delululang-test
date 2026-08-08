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

/// The single `.jsonl` day file in an audit directory.
///
/// **Not** `read_dir(..).next()`. That took whatever entry the filesystem happened to yield first,
/// which is unspecified order — and once `ANCHOR.json` (the truncation anchor, P17-C1) started
/// living alongside the log, "first entry" began resolving to the anchor on Linux while still
/// resolving to the day file on Windows. Two tests here passed on one platform and failed on the
/// other for that reason alone. Filter by extension and require exactly one match, so an unexpected
/// second file is a loud failure rather than a coin toss.
fn day_file(dir: &std::path::Path) -> PathBuf {
    let mut days: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("read audit dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
        .collect();
    days.sort();
    assert_eq!(days.len(), 1, "expected exactly one day file, found {days:?}");
    days.pop().expect("one day file")
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
    let root = b
        .issue_root(
            Holder::new("process", "cli-test", "pid:0"),
            Authority::new(eff(&["Write"]), Scopes { fs_write: names(&["./out"]), ..Default::default() }),
            None,
        )
        .expect("a non-strict broker issues a root");
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
    let file = day_file(&dir);
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
    let file = day_file(&dir);
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

// ----- C30: a read surface must not present a broken chain as authentic ------------------------

/// Rewrite the (single) day file in `dir` by mapping every line through `f`.
fn mangle_day_file(dir: &PathBuf, f: impl Fn(&str) -> String) {
    let day = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().and_then(|s| s.to_str()) == Some("jsonl"))
        .expect("a day file exists");
    let text = std::fs::read_to_string(&day).unwrap();
    let out: Vec<String> = text.lines().map(f).collect();
    std::fs::write(&day, out.join("\n") + "\n").unwrap();
}

#[test]
fn a_tampered_record_is_not_shown_as_authentic() {
    // C30. `audit tail` read records without checking a single hash. Flipping a `decision` from
    // "allow" to "deny" and leaving the hash alone produced a listing that displayed the FORGED
    // value with no warning — on the one surface an operator uses after an incident. The chain
    // verifier already detected this; nothing on the read path called it.
    let dir = seeded_dir("c30_tamper");
    mangle_day_file(&dir, |l| {
        if l.contains("\"seq\":2") { l.replace("\"allow\"", "\"deny\"") } else { l.to_string() }
    });
    let o = delulu(&["audit", "tail", "--dir", dir.to_str().unwrap()]);
    let err = stderr(&o);
    assert!(err.contains("WARNING"), "a broken chain must warn loudly:\n{err}");
    assert!(err.contains("MUST NOT be trusted"), "{err}");
    assert_ne!(o.status.code(), Some(0), "a corrupt read must not exit 0");
    // The records are still SHOWN — an operator investigating a tampered log needs to see it.
    assert!(!stdout(&o).is_empty(), "records must still be displayed");
}

#[test]
fn a_record_corrupted_beyond_parsing_does_not_vanish_silently() {
    // The sharper half of C30. A record mangled into non-JSON was silently DROPPED from the
    // listing: records 1, 2 and 4 were shown, 3 was simply absent, with no gap marker, no error,
    // and no hint that the list was shorter than the log. That is an omission attack against the
    // record of what happened, executed by corrupting one line.
    let dir = seeded_dir("c30_omit");
    let full = delulu(&["audit", "tail", "--dir", dir.to_str().unwrap()]);
    let before = stdout(&full).lines().count();
    mangle_day_file(&dir, |l| {
        if l.contains("\"seq\":3") { "{ not json".to_string() } else { l.to_string() }
    });
    let o = delulu(&["audit", "tail", "--dir", dir.to_str().unwrap()]);
    let after = stdout(&o).lines().count();
    assert!(after < before, "the probe must actually drop a record ({before} -> {after})");
    let err = stderr(&o);
    assert!(err.contains("WARNING"), "the loss must be announced:\n{err}");
    assert!(
        err.contains("MISSING from this listing"),
        "the warning must say entries may be missing entirely: {err}"
    );
    assert_ne!(o.status.code(), Some(0));
}

#[test]
fn the_json_read_surface_always_states_whether_the_chain_verified() {
    // A machine consumer must never have to infer integrity from the absence of a field, so
    // `chain_verified` is present on every tail/query response — true on a clean log, false with
    // `chain_error` beside it on a broken one.
    let dir = seeded_dir("c30_json");
    let clean = delulu(&["audit", "tail", "--dir", dir.to_str().unwrap(), "--json"]);
    let v: Value = serde_json::from_str(&stdout(&clean)).expect("valid JSON");
    assert_eq!(v["chain_verified"], Value::Bool(true), "{}", stdout(&clean));
    assert!(v.get("chain_error").is_none(), "no error on a clean chain");

    mangle_day_file(&dir, |l| {
        if l.contains("\"seq\":2") { l.replace("\"allow\"", "\"deny\"") } else { l.to_string() }
    });
    let broken = delulu(&["audit", "tail", "--dir", dir.to_str().unwrap(), "--json"]);
    let v2: Value = serde_json::from_str(&stdout(&broken)).expect("valid JSON even when broken");
    assert_eq!(v2["chain_verified"], Value::Bool(false), "{}", stdout(&broken));
    assert!(
        v2["chain_error"].as_str().unwrap_or_default().contains("DL1405")
            || v2["chain_error"].as_str().unwrap_or_default().contains("tamper"),
        "the error must name the failure: {}",
        stdout(&broken)
    );
}

/// **An audit tool may not silently ignore an argument.**
///
/// `audit` reads a log directory and takes `--dir`; every sibling custody command (`grants`,
/// `guard`, `secrets`) talks to the broker and takes `--state-dir`. The parser's final arm was
/// `_ => {}`, so an operator who typed the habitual flag had it dropped and got records from the
/// DEFAULT store — `~/.delulu/audit` — printed as the answer to a question about a different one.
///
/// Investigating an incident with evidence from somewhere else is not a lesser failure than showing
/// none. This is the campaign's own rule about the skip branch, landing on the one subsystem whose
/// entire purpose is accountability (C75, ruling D72).
#[test]
fn audit_refuses_an_option_it_does_not_know_instead_of_reading_the_wrong_store() {
    let empty = std::env::temp_dir().join(format!("delulu-audit-flag-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&empty);
    std::fs::create_dir_all(&empty).unwrap();

    let o = delulu(&["audit", "tail", "2", "--state-dir", empty.to_str().unwrap()]);
    let out = format!("{}{}", stdout(&o), stderr(&o));
    assert!(
        out.contains("does not know the option"),
        "the habitual flag must be REFUSED, not dropped: {out}"
    );
    assert!(out.contains("--dir"), "and the refusal must name the flag that does work: {out}");
    assert!(
        !out.contains("seq "),
        "no records may be printed for a store the caller did not ask for: {out}"
    );

    // A flag nobody could mistake for a real one is refused too — the fix is not special-cased.
    let o = delulu(&["audit", "tail", "2", "--totally-bogus"]);
    let out = format!("{}{}", stdout(&o), stderr(&o));
    assert!(out.contains("does not know the option"), "any unknown option is refused: {out}");
    assert!(!out.contains("seq "), "and nothing is read: {out}");
}

/// THE SKIP-BRANCH CASE: the real flag still works, and an empty directory says so by name rather
/// than printing nothing. A refusal that fired on every invocation would pass the test above and
/// break the command.
#[test]
fn audit_still_reads_the_directory_it_was_actually_given() {
    let empty = std::env::temp_dir().join(format!("delulu-audit-ok-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&empty);
    std::fs::create_dir_all(&empty).unwrap();

    let o = delulu(&["audit", "tail", "2", "--dir", empty.to_str().unwrap()]);
    let out = format!("{}{}", stdout(&o), stderr(&o));
    assert!(out.contains("no matching audit records"), "an empty log says so: {out}");
    assert!(
        out.contains(&empty.display().to_string()) || out.contains("delulu-audit-ok"),
        "and it names the directory it looked in, so the reader can tell which store: {out}"
    );
}

/// **`secrets list` on an empty store says so, rather than printing nothing.**
///
/// Silence is ambiguous, and here it was ambiguous about a security store: a reader could not tell
/// "there are no secrets" from "the store could not be read" from "the command did nothing", and it
/// exited 0 either way. `grants list` one command over already said `(no grants — the tree is
/// empty)`. This is the "nothing done, success claimed" shape of C26 and C66 (C74, ruling D72).
#[test]
fn secrets_list_on_an_empty_store_says_so() {
    let empty = std::env::temp_dir().join(format!("delulu-secrets-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&empty);
    std::fs::create_dir_all(&empty).unwrap();

    let o = delulu(&["secrets", "list", "--state-dir", empty.to_str().unwrap()]);
    let out = format!("{}{}", stdout(&o), stderr(&o));
    assert!(o.status.success(), "an empty store is not an error: {out}");
    assert!(
        !out.trim().is_empty(),
        "a security command that prints NOTHING leaves the reader unable to tell success from a \
         no-op — that is C26's shape"
    );
    assert!(out.contains("no secrets"), "and it says which condition it is: {out}");
}
