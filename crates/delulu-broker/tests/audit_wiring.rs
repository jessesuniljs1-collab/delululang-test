//! Phase 5d/5e wiring tests (invariant 26): a broker with an attached sink emits EXACTLY ONE audit
//! record per issue/attenuate/delegate/revoke/redeem/deny and per synchronous-class ALLOWED use —
//! reusing the seq the operation already consumed (chunk-1 ruling 5) — and epoch-class uses emit
//! none. Also: daily rotation driven by the INJECTED clock (no sleeps), through the whole
//! Broker → AuditLog → verify stack.

use std::collections::BTreeSet;
use std::rc::Rc;

use delulu_broker::{
    verify, AuditLog, Authority, Broker, Decision, Holder, ManualClock, MemSink, Op, Scopes,
    SeqIdSource,
};
use delulu_check::Effect;

fn eff(names: &[&str]) -> BTreeSet<Effect> {
    names.iter().map(|n| Effect::core_from_name(n).unwrap()).collect()
}
fn names(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}
fn holder() -> Holder {
    Holder::new("process", "orchestrator", "pid:1")
}

/// A broker with a shared in-memory sink and deterministic ids/clock. The MemSink clone shares
/// state, so the test reads back what the broker wrote.
fn broker_with_sink(clock: Rc<ManualClock>) -> (Broker, MemSink) {
    let sink = MemSink::new();
    let b = Broker::with_sources(Box::new(SeqIdSource::new()), Box::new(clock))
        .with_sink(Box::new(sink.clone()))
        .with_key([9u8; 32]);
    (b, sink)
}

#[test]
fn exactly_one_record_per_op_including_denies_and_none_for_epoch() {
    let clock = Rc::new(ManualClock::new(1_000));
    let (mut b, sink) = broker_with_sink(clock);

    // 1. issue → one "issue" record.
    let root = b.issue(
        holder(),
        Authority::new(
            eff(&["Read", "Net", "Write"]),
            Scopes {
                fs_read: names(&["./data"]),
                fs_write: names(&["./out"]),
                net: names(&["a.com"]),
                ..Default::default()
            },
        ),
        None,
    );
    assert_eq!(sink.len(), 1);

    // 2. attenuate (ok) → one "attenuate" allow record.
    let child = b
        .attenuate(
            &root,
            Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data/sub"]), ..Default::default() }),
            holder(),
            None,
        )
        .unwrap();
    assert_eq!(sink.len(), 2);

    // 3. attenuate (widening deny) → one "attenuate" deny record.
    let _ = b.attenuate(&root, Authority::new(eff(&["Declassify"]), Scopes::default()), holder(), None);
    assert_eq!(sink.len(), 3);

    // 4. delegate → EXACTLY one "delegate" record (never attenuate + delegate double-logged).
    let (_d, token) = b
        .delegate(
            &root,
            Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data/d"]), ..Default::default() }),
            holder(),
            None,
            false,
        )
        .unwrap();
    assert_eq!(sink.len(), 4);

    // 5. redeem → one "redeem" record.
    b.redeem(&token, "pid:4711").unwrap();
    assert_eq!(sink.len(), 5);

    // 6. denied redeem (second redemption) → one "redeem" record with decision "deny".
    let _ = b.redeem(&token, "pid:4712").unwrap_err();
    assert_eq!(sink.len(), 6);

    // 7. synchronous-class ALLOWED use → one "use" record carrying the SAME seq check() returned.
    let d = b.check(&root, Op::FsWrite, Some("./out/log.txt"));
    let Decision::Allow { audit_seq: Some(use_seq) } = d else {
        panic!("expected a synchronous allow with a seq")
    };
    assert_eq!(sink.len(), 7);

    // 8. synchronous-class DENIED use (out of scope) → one record, decision "deny".
    let _ = b.check(&root, Op::FsWrite, Some("./secret"));
    assert_eq!(sink.len(), 8);

    // 9. epoch-class use (live path AND snapshot path) → NO record (chunk-1 semantics).
    assert!(b.check(&root, Op::FsRead, Some("./data/x")).is_allow());
    let snap = b.snapshot();
    assert!(snap.check(&child, Op::FsRead, Some("./data/sub/y")).is_allow());
    assert_eq!(sink.len(), 8, "epoch-class uses emit no audit record");

    // 10. revoke → one "revoke" record; denied revoke (lateral) → one deny record.
    b.revoke(&root, &child).unwrap();
    assert_eq!(sink.len(), 9);
    let _ = b.revoke(&child, &root).unwrap_err();
    assert_eq!(sink.len(), 10);

    // 11. rotate_key → one "rotate_key" record.
    b.rotate_key();
    assert_eq!(sink.len(), 11);

    // The actions, in order, one per op.
    let actions: Vec<String> = sink.records().iter().map(|r| r.action.clone()).collect();
    assert_eq!(
        actions,
        vec![
            "issue",
            "attenuate",
            "attenuate",
            "delegate",
            "redeem",
            "redeem",
            "use",
            "use",
            "revoke",
            "revoke",
            "rotate_key"
        ]
    );
    let decisions: Vec<String> = sink.records().iter().map(|r| r.decision.clone()).collect();
    assert_eq!(
        decisions,
        vec!["allow", "allow", "deny", "allow", "allow", "deny", "allow", "deny", "allow", "deny", "allow"]
    );

    // Ruling 5 made visible: the records carry the broker's own monotone seqs with NO renumbering
    // and no gaps — every seq-consuming op logged exactly once.
    let seqs: Vec<u64> = sink.records().iter().map(|r| r.seq).collect();
    assert_eq!(seqs, (1..=11).collect::<Vec<u64>>());
    // And the "use" record reused the very seq check() handed back.
    assert_eq!(sink.records()[6].seq, use_seq);
}

#[test]
fn no_sink_means_no_records_and_chunk1_behavior() {
    // Default broker (no sink): everything works exactly as chunk 1 — this is the backward-compat
    // guarantee (head-chef ruling 4). Smoke-tested here; the full chunk-1 suite is the real proof.
    let clock = Rc::new(ManualClock::new(1_000));
    let mut b = Broker::with_sources(Box::new(SeqIdSource::new()), Box::new(clock));
    let root = b.issue(holder(), Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./d"]), ..Default::default() }), None);
    assert!(b.check(&root, Op::FsRead, Some("./d/x")).is_allow());
    b.revoke(&root, &root).unwrap();
}

#[test]
fn broker_driven_daily_rotation_cross_links_and_verifies() {
    // Drive rotation with the INJECTED clock — no sleeps. Day 1 = 1970-01-02, day 2 = 1970-01-03.
    let dir = std::env::temp_dir().join(format!("delulu_audit_broker_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    let day1 = 86_400_000 + 500;
    let clock = Rc::new(ManualClock::new(day1));
    let log = AuditLog::open(&dir).unwrap();
    let mut b = Broker::with_sources(Box::new(SeqIdSource::new()), Box::new(clock.clone()))
        .with_sink(Box::new(log));

    let root = b.issue(
        holder(),
        Authority::new(eff(&["Write"]), Scopes { fs_write: names(&["./out"]), ..Default::default() }),
        None,
    );
    assert!(b.check(&root, Op::FsWrite, Some("./out/a")).is_allow());

    // Cross the day boundary via the injected clock; the next record lands in a new day file whose
    // first record cross-links the previous day's head.
    clock.advance(86_400_000);
    assert!(b.check(&root, Op::FsWrite, Some("./out/b")).is_allow());

    let mut files: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    files.sort();
    assert_eq!(files, vec!["19700102.jsonl", "19700103.jsonl"], "one JSONL per day");

    // The whole cross-linked chain verifies end to end.
    let stats = verify(&dir).unwrap();
    assert_eq!(stats.files, 2);
    assert_eq!(stats.records, 3, "issue + two synchronous uses");

    let _ = std::fs::remove_dir_all(&dir);
}
