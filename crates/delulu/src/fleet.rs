//! Stage 10 phase 10j — Track H, §9.3: fleet/OTA updates (spec §9.3, criterion 9's fleet-update
//! drill).
//!
//! **Fleet updates ride the existing machinery rather than inventing a new one.** The rule the
//! spec states — "a fleet never receives an artifact whose hash differs from what was approved,
//! without explicit human re-approval" — is the sim-to-hardware artifact gate from Track D
//! (10f, spec §5.4), generalized. So this module fires the SAME diagnostic code, `DL1905`, and
//! reads the SAME [`delulu_runtime::Approval`] record type the device gate already uses. A fleet
//! member's approval and an actuator's sign-off answer the identical question — "did a human
//! approve exactly these bytes?" — and a second code here would just be two names for one rule.
//!
//! **The rollout is staged and health-gated, and a failure stops it rather than degrading it.**
//! `delulu fleet update` rolls one artifact out to `N` simulated members, one at a time: stage,
//! then run the member's health gate. The first health-gate failure halts the rollout immediately
//! — no member after the failing one is ever staged — and rolls back to a target PINNED before
//! the rollout started (`--previous`, always required; see [`cmd_fleet_update`]). "Rollback
//! artifacts are pinned at rollout start" (spec §9.3) is a statement about ordering: deciding what
//! to roll back to AFTER a failure has already happened is deciding it too late.
//!
//! **This is a simulation, honestly.** There is no real fleet, no real network, and no real
//! member here — `--members N` is a count, staging is a journal entry, and the health gate is a
//! deterministic function of the member index (`--fail-health-at`) rather than a probe of
//! anything real. Nothing in this module performs I/O beyond reading the artifact and the
//! approval record and writing the journal; it exists to exercise the STATE MACHINE — staged
//! rollout, health gate, approved-hash gate, rollback — the same way the robotics and satellite
//! demos exercise the device broker's state machine without a real arm or a real spacecraft.

use serde_json::json;

use delulu_diag::{render_human, Diagnostic, SourceMap};

/// One thing that happened during a staged rollout, for the journal. Mirrors the shape
/// `device::DeviceEvent` and `compute::DispatchRecord` already use for "one thing that happened,
/// for the record": an op, the id(s) it is about, and a detail string a human can read without
/// cross-referencing anything else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FleetEvent {
    /// `member.staged`, `member.health_pass`, `member.health_fail`, `rollout.rolled_back`,
    /// `rollout.completed`.
    pub op: String,
    /// The member index this event is about, when it is about exactly one member.
    pub member: Option<u32>,
    pub detail: String,
}

/// How a staged rollout ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RolloutOutcome {
    /// Every member updated; nothing rolled back.
    Completed,
    /// A health gate failed at this member index; the rollout stopped there and rolled back.
    /// Members after this index were never staged.
    RolledBack { at_member: u32 },
}

/// Run the staged rollout over `members` simulated fleet members (spec §9.3, criterion 9's
/// fleet-update drill). Pure: no filesystem, no network, no CLI — `health_check` decides each
/// member's fate and `journal` records every event, so this whole state machine is testable in
/// microseconds and independently of any process boundary. The CLI layer ([`cmd_fleet_update`])
/// is the only caller that knows about files, hashes, or `--fail-health-at`; this function knows
/// only members, a health-check function, and a place to write what happened.
///
/// **Order matters and is part of the contract.** A member is staged, THEN its health gate runs.
/// The first health-gate failure stops the rollout immediately: no member after the failing one
/// is ever staged. "Stop the rollout" means stop — continuing to roll a build out that a health
/// gate just rejected would make the gate a suggestion rather than a gate.
///
/// `previous` is the rollback target, resolved by the CALLER before this function is ever entered
/// (spec §9.3: "rollback artifacts are pinned at rollout start"). This function does not compute
/// or validate it; it only names it in the rollback event, so the journal is self-describing
/// without a reader having to cross-reference the invocation that produced it.
pub fn run_rollout(
    members: u32,
    previous: &str,
    mut health_check: impl FnMut(u32) -> bool,
    mut journal: impl FnMut(FleetEvent),
) -> RolloutOutcome {
    for m in 0..members {
        journal(FleetEvent {
            op: "member.staged".to_string(),
            member: Some(m),
            detail: format!("member {m} staged for update"),
        });
        if health_check(m) {
            journal(FleetEvent {
                op: "member.health_pass".to_string(),
                member: Some(m),
                detail: format!("member {m} passed its health gate"),
            });
            continue;
        }
        journal(FleetEvent {
            op: "member.health_fail".to_string(),
            member: Some(m),
            detail: format!("member {m} FAILED its health gate"),
        });
        let remaining = if m + 1 < members {
            format!("members {}..{members} were never staged", m + 1)
        } else {
            "no members remained after the failure".to_string()
        };
        journal(FleetEvent {
            op: "rollout.rolled_back".to_string(),
            member: Some(m),
            detail: format!(
                "member {m}'s health failure stopped the rollout; rolled back to {previous}; {remaining}"
            ),
        });
        return RolloutOutcome::RolledBack { at_member: m };
    }
    journal(FleetEvent {
        op: "rollout.completed".to_string(),
        member: None,
        detail: format!("all {members} member(s) updated successfully"),
    });
    RolloutOutcome::Completed
}

/// Resolve `--previous <hash-or-artifact-path>` to a `blake3:<hex>` content hash. Accepts either
/// form so a caller can name the rollback target however they have it on hand: a literal hash
/// (carried over from a prior rollout's own `--approved` record, say) or a path to the actual
/// previous artifact, which this hashes with the exact same [`delulu_broker::content_hash`] the
/// approved-hash gate below uses — one hashing implementation, every call site.
fn resolve_hash_or_path(s: &str) -> Result<String, String> {
    if let Some(hex) = s.strip_prefix("blake3:") {
        // Mirrors `Approval::parse`'s own validation depth exactly: length-checked, not
        // independently re-validated as hex, so a literal hash is held to the same bar the
        // approval record already is, no stricter and no looser.
        if hex.len() != 64 {
            return Err(format!("`{s}` starts with `blake3:` but is not 64 hex characters long"));
        }
        return Ok(s.to_string());
    }
    let bytes = std::fs::read(s).map_err(|e| format!("cannot read `{s}`: {e}"))?;
    Ok(delulu_broker::content_hash(&bytes))
}

/// `delulu fleet <subcommand>`. Only `update` exists today.
pub fn cmd_fleet(rest: &[String]) -> i32 {
    let Some(sub) = rest.first() else {
        eprintln!("error: `fleet` needs a subcommand (update)");
        return 2;
    };
    match sub.as_str() {
        "update" => cmd_fleet_update(&rest[1..]),
        other => {
            eprintln!("error: unknown `fleet` subcommand `{other}` (expected: update)");
            2
        }
    }
}

/// `delulu fleet update <artifact-path> --members N --approved <signoff-record-path>
/// [--fail-health-at I] --previous <hash-or-artifact-path> [--json]` (spec §9.3).
///
/// Flags:
/// - `--members N`: how many simulated fleet members to roll the update across. A positive
///   integer; zero or missing is a usage error (exit 2).
/// - `--approved <path>`: a [`delulu_runtime::Approval`] sign-off record — the SAME record shape
///   `delulu run --broker-profile hw:<adapter> --approved <record>` reads for the device gate.
///   Missing, unreadable, or unparseable — same as a hash mismatch — is `DL1905` (exit 1): a
///   corrupt or absent record is a refusal, never "nothing to check".
/// - `--fail-health-at I`: optional. When present, the health gate fails for exactly member `I`
///   and passes for every other member. Absent, every member passes — the control case that
///   gives the failure case meaning.
/// - `--previous <hash-or-artifact-path>`: **always required, unconditionally** — not only on
///   invocations that expect a failure. The rollback target must be pinned before the rollout
///   starts (spec §9.3), and a target that is only sometimes required is exactly the kind of
///   bound nobody enforces (10f's "a rate bound nobody enforces is a comfort, not a control"
///   lesson, applied here): the moment a health gate fails is the moment this stops being
///   optional, and by then it is too late to ask for it. Missing is a usage error (exit 2), not
///   `DL1905` — this is "you did not tell the CLI something it needs", not a safety refusal.
/// - `--json`: machine-readable output.
///
/// Exit codes: 0 every member completed; 1 `DL1905` refused before rollout started; 1 a rollback
/// occurred (a rolled-back update is not a success even though the rollback itself worked); 2 a
/// usage error.
fn cmd_fleet_update(rest: &[String]) -> i32 {
    let json = rest.iter().any(|a| a == "--json");
    let Some(artifact) = positional(rest) else {
        eprintln!("error: `fleet update` needs an artifact path");
        return 2;
    };
    let members: u32 = match flag(rest, "--members") {
        None => {
            eprintln!("error: `fleet update` needs `--members <N>` (a positive integer)");
            return 2;
        }
        Some(s) => match s.parse::<u32>() {
            Ok(n) if n > 0 => n,
            _ => {
                eprintln!("error: `--members` must be a positive integer, got `{s}`");
                return 2;
            }
        },
    };
    let Some(previous_raw) = flag(rest, "--previous") else {
        eprintln!(
            "error: `fleet update` needs `--previous <hash-or-artifact-path>` — the rollback \
             target pinned before this rollout starts (spec §9.3: rollback artifacts are pinned \
             at rollout start). Required on every invocation, not only ones that expect to fail: \
             a rollback target that is only sometimes supplied is a bound nobody enforces"
        );
        return 2;
    };
    let fail_health_at: Option<u32> = match flag(rest, "--fail-health-at") {
        None => None,
        Some(s) => match s.parse::<u32>() {
            Ok(n) => Some(n),
            Err(_) => {
                eprintln!("error: `--fail-health-at` must be a non-negative integer, got `{s}`");
                return 2;
            }
        },
    };

    let bytes = match std::fs::read(&artifact) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: cannot read `{artifact}`: {e}");
            return 2;
        }
    };
    let actual_hash = delulu_broker::content_hash(&bytes);

    // ----- DL1905, reused: the approved-hash rule generalized to a fleet member (spec §9.3) -----
    // Order: no sign-off flag at all, an unreadable record, and an unparseable record are all the
    // SAME refusal as a mismatched hash. "I could not tell whether this was approved" is the skip
    // branch, and the skip branch says no — exactly the law `Approval::parse`'s own doc comment
    // states for the device case, applied here without exception.
    let Some(approved_path) = flag(rest, "--approved") else {
        return refuse_dl1905(
            &artifact,
            format!(
                "`fleet update {artifact}` was requested with no `--approved` sign-off record — \
                 pass `--approved <record>` naming the rollout a human approved for these exact \
                 bytes. This artifact hashes to {actual_hash}. A fleet never receives an artifact \
                 whose hash differs from what was approved, without explicit human re-approval \
                 [a human must approve this rollout; see `delulu explain DL1905` and spec §9.3]"
            ),
            json,
        );
    };
    let src = match std::fs::read_to_string(&approved_path) {
        Ok(s) => s,
        Err(e) => {
            return refuse_dl1905(
                &artifact,
                format!(
                    "the sign-off record `{approved_path}` could not be read ({e}) — a corrupt \
                     or absent approval is a refusal, never nothing to check [see `delulu explain \
                     DL1905`]"
                ),
                json,
            );
        }
    };
    let approval = match delulu_runtime::Approval::parse(&src) {
        Ok(a) => a,
        Err(e) => {
            return refuse_dl1905(
                &artifact,
                format!(
                    "the sign-off record `{approved_path}` is not readable as an approval: {e} \
                     [see `delulu explain DL1905`]"
                ),
                json,
            );
        }
    };
    if approval.hash != actual_hash {
        return refuse_dl1905(
            &artifact,
            format!(
                "`{artifact}` does not match its approved hash: approved {}, actual {actual_hash} \
                 — re-approve this exact artifact, or restore the one that was approved. A fleet \
                 never receives an artifact whose hash differs from what was approved, without \
                 explicit human re-approval [a human must re-approve; see `delulu explain DL1905` \
                 and spec §9.3]",
                approval.hash
            ),
            json,
        );
    }

    // The gate passed. Resolve the rollback target now — AFTER the gate, not before, so a broken
    // `--previous` never masks a hash refusal that would have stopped the rollout anyway.
    let previous_hash = match resolve_hash_or_path(&previous_raw) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("error: cannot resolve `--previous {previous_raw}`: {e}");
            return 2;
        }
    };

    let mut events: Vec<FleetEvent> = Vec::new();
    let outcome =
        run_rollout(members, &previous_hash, |m| Some(m) != fail_health_at, |ev| events.push(ev));

    print_rollout_summary(&artifact, &actual_hash, members, &previous_hash, outcome, &events, json);

    match outcome {
        RolloutOutcome::Completed => 0,
        RolloutOutcome::RolledBack { .. } => 1,
    }
}

/// Refuse a fleet update before rollout starts (`DL1905`, spec §9.3 — the approved-hash rule
/// generalized from the device sim-to-hardware gate, §5.4). A real [`Diagnostic`], rendered
/// through the same [`render_human`] this crate's `deploy plan` (`DL1909`) uses for an identical
/// reason: neither refusal has a source span (a file's bytes, not a program's source), so an empty
/// [`SourceMap`] is the honest input, and both commands render consistently with each other rather
/// than each inventing their own error format.
fn refuse_dl1905(artifact: &str, message: String, json: bool) -> i32 {
    let d = Diagnostic::error("DL1905", message);
    if json {
        println!(
            "{}",
            json!({
                "command": "fleet",
                "subcommand": "update",
                "outcome": "refused",
                "artifact": artifact,
                "code": d.code,
                "error": d.message,
            })
        );
    } else {
        eprint!("{}", render_human(&d, &SourceMap::new()));
    }
    1
}

fn print_rollout_summary(
    artifact: &str,
    hash: &str,
    members: u32,
    previous: &str,
    outcome: RolloutOutcome,
    events: &[FleetEvent],
    json: bool,
) {
    if json {
        let events_json: Vec<serde_json::Value> = events
            .iter()
            .map(|e| json!({ "op": e.op, "member": e.member, "detail": e.detail }))
            .collect();
        let mut obj = json!({
            "command": "fleet",
            "subcommand": "update",
            "outcome": match outcome {
                RolloutOutcome::Completed => "completed",
                RolloutOutcome::RolledBack { .. } => "rolled_back",
            },
            "artifact": artifact,
            "hash": hash,
            "members": members,
            "previous": previous,
            "events": events_json,
        });
        if let RolloutOutcome::RolledBack { at_member } = outcome {
            obj["failed_member"] = json!(at_member);
        }
        println!("{obj}");
        return;
    }
    for ev in events {
        match ev.op.as_str() {
            "member.health_pass" => {
                println!("member {}: staged, health OK", ev.member.unwrap_or_default())
            }
            "member.health_fail" => {
                println!("member {}: staged, health FAILED", ev.member.unwrap_or_default())
            }
            "rollout.rolled_back" => println!("rollback: {}", ev.detail),
            _ => {}
        }
    }
    match outcome {
        RolloutOutcome::Completed => {
            println!("fleet update: completed — {members}/{members} member(s) updated to {hash}");
        }
        RolloutOutcome::RolledBack { at_member } => {
            println!(
                "fleet update: ROLLED BACK at member {at_member} — rolled back to {previous} \
                 (the update to {hash} did not complete)"
            );
        }
    }
}

// ----- tiny helpers (no new deps, mirrors signing.rs's) --------------------

fn flag(rest: &[String], name: &str) -> Option<String> {
    let eq = format!("{name}=");
    let mut it = rest.iter();
    while let Some(a) = it.next() {
        if a == name {
            return it.next().cloned();
        }
        if let Some(v) = a.strip_prefix(&eq) {
            return Some(v.to_string());
        }
    }
    None
}

fn positional(rest: &[String]) -> Option<String> {
    let mut i = 0;
    while i < rest.len() {
        let a = &rest[i];
        if a == "--members" || a == "--approved" || a == "--fail-health-at" || a == "--previous" {
            i += 2; // skip the flag AND its value
            continue;
        }
        if a.starts_with("--") {
            i += 1;
            continue;
        }
        return Some(a.clone());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The control case, and a REAL reachable path rather than an untested default: every member
    /// passes a health check that is exercised exactly the same way a failing one would be, and
    /// nothing rolls back.
    #[test]
    fn a_clean_rollout_completes_every_member_with_no_rollback() {
        let mut events = Vec::new();
        let outcome = run_rollout(5, "blake3:prev", |_m| true, |ev| events.push(ev));
        assert_eq!(outcome, RolloutOutcome::Completed);
        let staged: Vec<u32> =
            events.iter().filter(|e| e.op == "member.staged").filter_map(|e| e.member).collect();
        assert_eq!(staged, vec![0, 1, 2, 3, 4], "every member is staged, in order");
        let passed: Vec<u32> = events
            .iter()
            .filter(|e| e.op == "member.health_pass")
            .filter_map(|e| e.member)
            .collect();
        assert_eq!(passed, vec![0, 1, 2, 3, 4], "every member passes its health gate");
        assert!(
            events.iter().any(|e| e.op == "rollout.completed"),
            "the completion event is journaled: {events:?}"
        );
        assert!(
            !events.iter().any(|e| e.op.contains("fail") || e.op.contains("rolled_back")),
            "a clean rollout journals nothing about failure: {events:?}"
        );
    }

    /// The failure case the acceptance criterion names directly: a health-gate failure at a
    /// specific member triggers rollback and stops further staging. Checking the JOURNAL, not
    /// just the returned outcome, is the point — a bug that kept staging after the failure would
    /// still return `RolledBack { at_member: 2 }`, and only the journal would catch it.
    #[test]
    fn a_health_failure_stops_the_rollout_and_nothing_after_it_is_ever_staged() {
        let mut events = Vec::new();
        let outcome = run_rollout(5, "blake3:prev", |m| m != 2, |ev| events.push(ev));
        assert_eq!(outcome, RolloutOutcome::RolledBack { at_member: 2 });
        let staged: Vec<u32> =
            events.iter().filter(|e| e.op == "member.staged").filter_map(|e| e.member).collect();
        assert_eq!(staged, vec![0, 1, 2], "members 3 and 4 must never be staged: {events:?}");
        assert!(!events.iter().any(|e| e.member == Some(3) || e.member == Some(4)),
            "no event at all — staged, passed, failed — may mention members 3 or 4: {events:?}");
        assert!(events.iter().any(|e| e.op == "member.health_fail" && e.member == Some(2)));
        let rollback = events
            .iter()
            .find(|e| e.op == "rollout.rolled_back")
            .expect("the rollback is journaled");
        assert_eq!(rollback.member, Some(2), "the rollback names which member triggered it");
        assert!(rollback.detail.contains("blake3:prev"), "and what it rolled back to: {}", rollback.detail);
        assert!(
            !events.iter().any(|e| e.op == "rollout.completed"),
            "a rolled-back rollout never also completes: {events:?}"
        );
    }

    /// The edge at the front of the list: failure at member 0 still stages that member first
    /// (staging always precedes the health check), then rolls back with zero members finished.
    #[test]
    fn a_failure_at_the_first_member_still_stages_it_before_rolling_back() {
        let mut events = Vec::new();
        let outcome = run_rollout(3, "blake3:prev", |m| m != 0, |ev| events.push(ev));
        assert_eq!(outcome, RolloutOutcome::RolledBack { at_member: 0 });
        let staged: Vec<u32> =
            events.iter().filter(|e| e.op == "member.staged").filter_map(|e| e.member).collect();
        assert_eq!(staged, vec![0], "member 0 is staged even though it immediately fails");
    }

    /// The edge at the back of the list: failure at the LAST member leaves no remainder to name,
    /// and the rollback detail must say so rather than printing an empty/nonsensical range.
    #[test]
    fn a_failure_at_the_last_member_names_no_remainder() {
        let mut events = Vec::new();
        let outcome = run_rollout(3, "blake3:prev", |m| m != 2, |ev| events.push(ev));
        assert_eq!(outcome, RolloutOutcome::RolledBack { at_member: 2 });
        let rollback = events.iter().find(|e| e.op == "rollout.rolled_back").unwrap();
        assert!(
            rollback.detail.contains("no members remained"),
            "an empty range must not be printed as a range: {}",
            rollback.detail
        );
    }

    #[test]
    fn resolve_hash_or_path_accepts_a_wellformed_literal_hash_and_rejects_a_short_one() {
        let good = format!("blake3:{}", "a".repeat(64));
        assert_eq!(resolve_hash_or_path(&good).unwrap(), good);
        assert!(resolve_hash_or_path("blake3:tooshort").is_err());
    }

    #[test]
    fn resolve_hash_or_path_hashes_a_real_file_with_the_same_function_the_gate_uses() {
        let dir = std::env::temp_dir().join(format!("delulu-fleet-unit-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("prev.bin");
        std::fs::write(&path, b"previous artifact bytes").unwrap();
        let want = delulu_broker::content_hash(b"previous artifact bytes");
        assert_eq!(resolve_hash_or_path(path.to_str().unwrap()).unwrap(), want);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn flag_and_positional_agree_on_which_tokens_are_values() {
        let rest = vec![
            "art.bin".to_string(),
            "--members".to_string(),
            "5".to_string(),
            "--json".to_string(),
        ];
        assert_eq!(positional(&rest), Some("art.bin".to_string()));
        assert_eq!(flag(&rest, "--members"), Some("5".to_string()));
    }
}
