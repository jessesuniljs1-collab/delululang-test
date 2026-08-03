//! P17-7 — the audit chain detects modification and reordering, but **NOT suffix truncation**.
//!
//! `audit::verify` walks the day files forward from `GENESIS_HASH`, checking each record's
//! `prev_hash` against the running head and recomputing each record's own hash
//! (`audit.rs:354-396`). Every one of those checks is *local to a link*. Deleting the LAST k
//! records therefore leaves a chain in which every remaining link is still correct, so `verify`
//! returns `Ok` — with a smaller record count and an earlier head.
//!
//! Nothing anchors the head. `AuditLog::open` **recovers** the head by reading the existing day
//! files (`audit.rs:216-227`), so after a truncation the broker resumes chaining from the truncated
//! head and every subsequent record is genuinely valid. There is no stored expectation of the head
//! or the record count anywhere in the workspace to compare against.
//!
//! ## Why this is worth stating rather than shrugging at
//!
//! The threat model does bound it: the audit directory lives under the operator's own state
//! directory, and this project already records that it does not provide multi-tenancy or same-user
//! isolation. An attacker who can delete audit files can usually do worse.
//!
//! But a hash chain is sold as **tamper evidence**, and the attack this one does not detect is the
//! attractive one: you do not modify the record of what you did, you delete it. The honest claim is
//! "the chain detects modification and reordering", not "the chain is tamper-evident", and the
//! difference is exactly what an investigator would be relying on.
//!
//! ## What would close it
//!
//! Anchoring the head outside the log — the broker's state file, a monotonic counter checked at
//! open, or an external witness. `AuditBundle::verify` already accepts an `expected_start`
//! (`audit.rs:485`), which is the same idea applied to the *start* of a bundle; the *end* of the
//! live chain has no equivalent. Not implemented here: it is a persistence-format change and
//! belongs in an RFC.

use std::fs;

use delulu_broker::audit::{self, AuditLog, AuditSink};

/// Build a small real chain, then cut records off the end and re-verify.
#[test]
fn deleting_the_last_records_leaves_a_chain_that_still_verifies() {
    let dir = std::env::temp_dir().join(format!("delulu-audit-trunc-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp audit dir");

    // --- write a real chain through the public sink -----------------------------------------
    {
        let mut log = AuditLog::open(&dir).expect("open audit log");
        for i in 0..5u64 {
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
            // A dropped append would silently shrink the fixture and make the truncation result
            // meaningless, so the write is checked rather than discarded.
            .expect("append succeeds");
        }
    }

    let before = audit::verify(&dir).expect("the freshly written chain verifies");
    assert_eq!(before.records, 5, "five records written");

    // --- the attack: delete the last two lines of the day file ------------------------------
    let day = fs::read_dir(&dir)
        .expect("read dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "jsonl"))
        .expect("a day file exists");

    let text = fs::read_to_string(&day).expect("read day file");
    let mut lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    lines.truncate(lines.len() - 2);
    fs::write(&day, lines.join("\n") + "\n").expect("rewrite truncated day file");

    // --- THE FINDING: the truncated chain still verifies -------------------------------------
    let after = audit::verify(&dir).expect(
        "TRUNCATION IS UNDETECTED: verify() returns Ok on a chain whose last records were deleted, \
         because every check it performs is local to a link and the deleted suffix left no trace",
    );

    assert_eq!(after.records, before.records - 2, "two records are simply gone");
    assert_ne!(after.head, before.head, "the head moved backwards, and nothing notices");

    let _ = fs::remove_dir_all(&dir);
}

/// The control: MODIFICATION is caught. This is what the chain genuinely provides, and stating the
/// limitation above must not be read as "the chain does nothing".
#[test]
fn modifying_a_record_in_place_is_caught() {
    let dir = std::env::temp_dir().join(format!("delulu-audit-tamper-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp audit dir");

    {
        let mut log = AuditLog::open(&dir).expect("open audit log");
        for i in 0..4u64 {
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
            // A dropped append would silently shrink the fixture and make the truncation result
            // meaningless, so the write is checked rather than discarded.
            .expect("append succeeds");
        }
    }
    audit::verify(&dir).expect("baseline verifies");

    let day = fs::read_dir(&dir)
        .expect("read dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "jsonl"))
        .expect("a day file exists");

    // Flip an "allow" to a "deny" in the middle of the chain.
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
