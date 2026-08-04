//! P17-C1 — **FIXED.** The audit chain detected modification and reordering but not TRUNCATION.
//! It does now.
//!
//! ## What was observed, before the fix
//!
//! `audit::verify` walks the day files forward from `GENESIS_HASH`, checking each record's
//! `prev_hash` against the running head and recomputing its own hash. **Every one of those checks
//! is local to a link.** Deleting the last *k* records therefore left a chain in which every
//! remaining link was still correct, so `verify` returned `Ok` — with a shorter count and an
//! earlier head, and no complaint. Nothing anchored the head: `AuditLog::open` *recovered* it by
//! reading the files, so the broker resumed chaining from the truncated head and every later record
//! was genuinely valid.
//!
//! Five records written, last two deleted, `verify` reported three and passed.
//!
//! ## The fix
//!
//! `ANCHOR.json` records the head hash and the record count **outside the log**, refreshed on every
//! append. `verify` compares the chain it computed against it, and `AuditLog::open` refuses to
//! start on a log that disagrees with its own anchor — because silently rewriting a disagreeing
//! anchor would erase the evidence it exists to keep.
//!
//! ## What this does NOT buy, stated because an anchor is easy to oversell
//!
//! The anchor lives in the same directory as the log, so **an attacker who can delete records can
//! also rewrite the anchor.** This is not tamper-proofing. What it buys is real but bounded:
//!
//! * **Accidental truncation is caught** — partial write, full disk, botched rotation, half-done
//!   sync. An ordinary operational failure that used to pass silently.
//! * **Naive tampering is caught** — deleting lines needs a text editor; a consistent anchor needs
//!   the format.
//! * **The head is now exportable.** `verify` returns it and it is written down, so an operator can
//!   witness it elsewhere and check a later run against a value the attacker never held. That is
//!   the only route to genuine tamper-evidence, and it needs an **external** witness — which no
//!   file inside this directory can be.
//!
//! `an_attacker_who_also_rewrites_the_anchor_is_not_caught` pins that limit deliberately.

use std::fs;

use delulu_broker::audit::{self, AuditLog, AuditSink};

fn write_chain(dir: &std::path::Path, n: u64) {
    let mut log = AuditLog::open(dir).expect("open audit log");
    for i in 0..n {
        log.append(audit::AuditEntry {
            seq: i,
            ts: 1_000 + i as i64,
            actor_node: Some(format!("node-{i}")),
            action: "delegate".into(),
            target: None,
            authority: None,
            span: None,
            decision: "allow".into(),
        })
        .expect("append succeeds");
    }
}

fn day_file(dir: &std::path::Path) -> std::path::PathBuf {
    fs::read_dir(dir)
        .expect("read dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "jsonl"))
        .expect("a day file exists")
}

fn tmp(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-audit-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).expect("temp audit dir");
    d
}

fn truncate_by(dir: &std::path::Path, k: usize) -> Vec<String> {
    let day = day_file(dir);
    let text = fs::read_to_string(&day).expect("read day file");
    let mut lines: Vec<String> = text.lines().filter(|l| !l.trim().is_empty()).map(String::from).collect();
    lines.truncate(lines.len() - k);
    fs::write(&day, lines.join("\n") + "\n").expect("rewrite truncated day file");
    lines
}

/// **The regression witness.** This asserted the opposite before the anchor existed: `verify`
/// returned `Ok` on a chain whose last records had been deleted.
#[test]
fn deleting_the_last_records_is_now_detected() {
    let dir = tmp("trunc");
    write_chain(&dir, 5);

    let before = audit::verify(&dir).expect("the freshly written chain verifies");
    assert_eq!(before.records, 5, "five records written");

    truncate_by(&dir, 2);

    let err = audit::verify(&dir)
        .expect_err("TRUNCATION MUST BE DETECTED — the anchor still says five records");
    let msg = format!("{err}");
    assert!(
        msg.contains("anchor"),
        "the error must name the anchor disagreement, got: {msg}"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Reopening a truncated log must refuse too — otherwise the broker would resume chaining from the
/// shortened head and every later record would be genuinely valid, laundering the deletion.
#[test]
fn reopening_a_truncated_log_refuses() {
    let dir = tmp("reopen");
    write_chain(&dir, 4);
    truncate_by(&dir, 1);

    assert!(
        AuditLog::open(&dir).is_err(),
        "opening a log that disagrees with its own anchor must fail, not silently re-anchor"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// The control that must keep passing: MODIFICATION was always caught, and still is.
#[test]
fn modifying_a_record_in_place_is_caught() {
    let dir = tmp("tamper");
    write_chain(&dir, 4);
    audit::verify(&dir).expect("baseline verifies");

    let day = day_file(&dir);
    let text = fs::read_to_string(&day).expect("read");
    let tampered = text.replacen("\"allow\"", "\"deny\"", 1);
    assert_ne!(tampered, text, "the fixture must actually change something");
    fs::write(&day, tampered).expect("write");

    assert!(
        audit::verify(&dir).is_err(),
        "an in-place edit MUST be detected — that is what the chain is for"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// An honest limit, pinned so nobody mistakes the anchor for tamper-proofing: an attacker who also
/// rewrites the anchor to match the truncated chain is NOT caught. If this ever starts failing,
/// someone has added an external witness and the documentation above must be updated to match.
#[test]
fn an_attacker_who_also_rewrites_the_anchor_is_not_caught() {
    let dir = tmp("both");
    write_chain(&dir, 5);
    let lines = truncate_by(&dir, 2);

    // Recompute a consistent anchor from the truncated chain — what a competent attacker does.
    let surviving = lines.last().expect("a record survives");
    let v: serde_json::Value = serde_json::from_str(surviving).expect("record parses");
    let head = v["hash"].as_str().expect("hash present");
    fs::write(dir.join("ANCHOR.json"), format!("{{\"head\":\"{head}\",\"records\":3}}\n"))
        .expect("rewrite anchor");

    assert!(
        audit::verify(&dir).is_ok(),
        "the anchor lives beside the log, so rewriting both is not detectable from inside the \
         directory — this is the documented limit, and only an EXTERNAL witness closes it"
    );

    let _ = fs::remove_dir_all(&dir);
}
