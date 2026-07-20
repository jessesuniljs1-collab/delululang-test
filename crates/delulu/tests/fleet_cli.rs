//! Stage 10 phase 10j — Track H, §9.3: `delulu fleet update` at the CLI boundary.
//!
//! The pure staged-rollout state machine (stage → health gate → rollback-or-continue) is unit
//! tested in `crates/delulu/src/fleet.rs` itself, fast and independent of any process. This file
//! covers the four things criterion 9's fleet-update drill names that only exist at the process
//! boundary: a real artifact file, a real approval record on disk, and the actual exit codes and
//! `--json` shape a caller gets back — a clean approved rollout, a hash mismatch, a missing
//! approval record (the skip branch), and a health-gate failure that rolls back and stops
//! staging (checked via the JOURNAL, not just the exit code).
//!
//! Approval records here are hand-constructed via `delulu_runtime::Approval { .. }.to_json()`
//! rather than shelled out through any signing machinery: an approval record is not a signature,
//! it is a human's plain-text sign-off (see `Approval`'s own doc comment), so building one
//! directly is both the more realistic case and the less fragile one — no keypair, no subprocess
//! round-trip, just the exact struct `fleet update` itself parses.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(repo_root())
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-fleet-cli-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Hand-construct a sign-off record on disk, reusing `delulu_runtime::Approval` exactly as the
/// device gate does — this is the SAME record type `--broker-profile hw:<adapter> --approved`
/// reads, just written directly rather than produced by a `sim` run.
fn write_approval(dir: &Path, artifact: &str, hash: &str) -> PathBuf {
    let approval = delulu_runtime::Approval {
        artifact: artifact.to_string(),
        hash: hash.to_string(),
        profile: "fleet-test".to_string(),
    };
    let path = dir.join("approved.json");
    std::fs::write(&path, approval.to_json()).unwrap();
    path
}

// ----- the control case: a clean, fully-approved rollout ---------------------------------------

/// The control case, and the one that gives every refusal test below its meaning: a correctly
/// approved artifact rolls out to every member with no failures at all.
#[test]
fn a_clean_approved_rollout_completes_all_members_with_no_failures() {
    let dir = scratch("clean");
    let artifact = dir.join("release.bin");
    std::fs::write(&artifact, b"fleet payload v1").unwrap();
    let hash = delulu_broker::content_hash(b"fleet payload v1");
    let approved = write_approval(&dir, &artifact.to_string_lossy(), &hash);
    let previous = dir.join("previous.bin");
    std::fs::write(&previous, b"fleet payload v0").unwrap();

    let o = delulu(&[
        "fleet",
        "update",
        &artifact.to_string_lossy(),
        "--members",
        "5",
        "--approved",
        &approved.to_string_lossy(),
        "--previous",
        &previous.to_string_lossy(),
        "--json",
    ]);
    let out = stdout(&o);
    assert!(o.status.success(), "a clean rollout must succeed:\n{out}\n{}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("valid JSON");
    assert_eq!(v["outcome"].as_str(), Some("completed"));
    assert_eq!(v["members"].as_u64(), Some(5));
    let events = v["events"].as_array().expect("events array");
    let staged = events.iter().filter(|e| e["op"] == "member.staged").count();
    assert_eq!(staged, 5, "all five members staged:\n{out}");
    let passed = events.iter().filter(|e| e["op"] == "member.health_pass").count();
    assert_eq!(passed, 5, "all five members passed their health gate — a REAL reachable path:\n{out}");
    assert!(events.iter().any(|e| e["op"] == "rollout.completed"), "{out}");
    assert!(!events.iter().any(|e| e["op"] == "rollout.rolled_back"), "{out}");
}

/// The same control case in human-readable mode: one line per member, then a final verdict line.
#[test]
fn the_human_readable_summary_has_one_line_per_member_and_a_verdict() {
    let dir = scratch("clean-human");
    let artifact = dir.join("release.bin");
    std::fs::write(&artifact, b"fleet payload human").unwrap();
    let hash = delulu_broker::content_hash(b"fleet payload human");
    let approved = write_approval(&dir, &artifact.to_string_lossy(), &hash);
    let previous = dir.join("previous.bin");
    std::fs::write(&previous, b"fleet payload human v0").unwrap();

    let o = delulu(&[
        "fleet",
        "update",
        &artifact.to_string_lossy(),
        "--members",
        "3",
        "--approved",
        &approved.to_string_lossy(),
        "--previous",
        &previous.to_string_lossy(),
    ]);
    let out = stdout(&o);
    assert!(o.status.success(), "{out}\n{}", stderr(&o));
    let member_lines = out.lines().filter(|l| l.starts_with("member ")).count();
    assert_eq!(member_lines, 3, "one line per member:\n{out}");
    assert!(out.lines().any(|l| l.contains("completed")), "a final verdict line:\n{out}");
}

// ----- DL1905: the approved-hash gate, reused ---------------------------------------------------

/// A hash mismatch refuses `DL1905` before any member is staged. Checked via the ABSENCE of an
/// `events` array in the JSON output: the rollout function that would produce staged events is
/// never even called on this path.
#[test]
fn a_hash_mismatch_refuses_dl1905_before_any_member_is_staged() {
    let dir = scratch("mismatch");
    let artifact = dir.join("release.bin");
    std::fs::write(&artifact, b"fleet payload v1").unwrap();
    // Approve a hash that does NOT match — as if the artifact were edited after sign-off.
    let wrong_hash = delulu_broker::content_hash(b"a completely different payload");
    let approved = write_approval(&dir, &artifact.to_string_lossy(), &wrong_hash);
    let previous = dir.join("previous.bin");
    std::fs::write(&previous, b"fleet payload v0").unwrap();

    let o = delulu(&[
        "fleet",
        "update",
        &artifact.to_string_lossy(),
        "--members",
        "5",
        "--approved",
        &approved.to_string_lossy(),
        "--previous",
        &previous.to_string_lossy(),
        "--json",
    ]);
    let out = stdout(&o);
    assert_eq!(o.status.code(), Some(1), "{out}\n{}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("valid JSON");
    assert_eq!(v["outcome"].as_str(), Some("refused"));
    assert_eq!(v["code"].as_str(), Some("DL1905"));
    assert!(v.get("events").is_none(), "the refusal fires before any rollout event exists:\n{out}");
}

/// The skip branch: a missing approval FILE (the flag names a path, but nothing is there) is a
/// refusal, never a pass.
#[test]
fn a_missing_approval_file_refuses_dl1905_not_a_pass() {
    let dir = scratch("missing-approval");
    let artifact = dir.join("release.bin");
    std::fs::write(&artifact, b"fleet payload v1").unwrap();
    let previous = dir.join("previous.bin");
    std::fs::write(&previous, b"fleet payload v0").unwrap();
    let missing = dir.join("does-not-exist.json");

    let o = delulu(&[
        "fleet",
        "update",
        &artifact.to_string_lossy(),
        "--members",
        "3",
        "--approved",
        &missing.to_string_lossy(),
        "--previous",
        &previous.to_string_lossy(),
    ]);
    let err = stderr(&o);
    assert_eq!(o.status.code(), Some(1), "a missing approval is a refusal, not a pass:\n{err}");
    assert!(err.contains("DL1905"), "{err}");
}

/// The other half of "missing": the `--approved` flag omitted entirely. Symmetric with the device
/// case's own CLI (`resolve_device_profile`), which treats an absent `--approved` the same as an
/// absent record — both are DL1905, not a bare usage error.
#[test]
fn omitting_approved_entirely_is_also_a_dl1905_refusal() {
    let dir = scratch("no-approved-flag");
    let artifact = dir.join("release.bin");
    std::fs::write(&artifact, b"fleet payload v1").unwrap();
    let previous = dir.join("previous.bin");
    std::fs::write(&previous, b"fleet payload v0").unwrap();

    let o = delulu(&[
        "fleet",
        "update",
        &artifact.to_string_lossy(),
        "--members",
        "3",
        "--previous",
        &previous.to_string_lossy(),
    ]);
    let err = stderr(&o);
    assert_eq!(o.status.code(), Some(1), "{err}");
    assert!(err.contains("DL1905"), "{err}");
}

/// A corrupt (unparseable) approval record is refused exactly like an absent one — never read as
/// "nothing to check", matching `Approval::parse`'s own stated law.
#[test]
fn a_corrupt_approval_record_refuses_dl1905_not_an_empty_approval() {
    let dir = scratch("corrupt-approval");
    let artifact = dir.join("release.bin");
    std::fs::write(&artifact, b"fleet payload v1").unwrap();
    let previous = dir.join("previous.bin");
    std::fs::write(&previous, b"fleet payload v0").unwrap();
    let corrupt = dir.join("corrupt.json");
    std::fs::write(&corrupt, b"{ not json at all").unwrap();

    let o = delulu(&[
        "fleet",
        "update",
        &artifact.to_string_lossy(),
        "--members",
        "3",
        "--approved",
        &corrupt.to_string_lossy(),
        "--previous",
        &previous.to_string_lossy(),
    ]);
    let err = stderr(&o);
    assert_eq!(o.status.code(), Some(1), "a corrupt approval is a refusal, not nothing to check:\n{err}");
    assert!(err.contains("DL1905"), "{err}");
}

// ----- the health gate and rollback --------------------------------------------------------------

/// The criterion's centerpiece: a health-gate failure at a specific member triggers rollback and
/// STOPS further staging. Checked via the journal, not just the exit code — a bug that kept
/// staging past the failure would still exit 1, and only the journal catches it.
#[test]
fn a_health_gate_failure_triggers_rollback_and_stops_further_staging() {
    let dir = scratch("rollback");
    let artifact = dir.join("release.bin");
    std::fs::write(&artifact, b"fleet payload v2").unwrap();
    let hash = delulu_broker::content_hash(b"fleet payload v2");
    let approved = write_approval(&dir, &artifact.to_string_lossy(), &hash);
    let previous = dir.join("previous.bin");
    std::fs::write(&previous, b"fleet payload v1").unwrap();
    let previous_hash = delulu_broker::content_hash(b"fleet payload v1");

    let o = delulu(&[
        "fleet",
        "update",
        &artifact.to_string_lossy(),
        "--members",
        "5",
        "--approved",
        &approved.to_string_lossy(),
        "--previous",
        &previous.to_string_lossy(),
        "--fail-health-at",
        "2",
        "--json",
    ]);
    let out = stdout(&o);
    assert_eq!(o.status.code(), Some(1), "a rolled-back update is not a success:\n{out}\n{}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("valid JSON");
    assert_eq!(v["outcome"].as_str(), Some("rolled_back"));
    assert_eq!(v["failed_member"].as_u64(), Some(2));

    let events = v["events"].as_array().expect("events array");
    let staged: Vec<u64> = events
        .iter()
        .filter(|e| e["op"] == "member.staged")
        .map(|e| e["member"].as_u64().unwrap())
        .collect();
    assert_eq!(staged, vec![0, 1, 2], "members 3 and 4 must never be staged:\n{out}");
    assert!(
        !events.iter().any(|e| e["member"].as_u64() == Some(3) || e["member"].as_u64() == Some(4)),
        "no event at all — staged, passed, failed — may mention members 3 or 4:\n{out}"
    );
    let rollback = events
        .iter()
        .find(|e| e["op"] == "rollout.rolled_back")
        .expect("the rollback is journaled");
    assert_eq!(rollback["member"].as_u64(), Some(2), "the rollback names which member triggered it");
    assert!(
        rollback["detail"].as_str().unwrap().contains(&previous_hash),
        "the rollback names what it rolled back to: {}",
        rollback["detail"]
    );
    assert!(!events.iter().any(|e| e["op"] == "rollout.completed"), "{out}");
}

/// The same rollback, in human-readable mode: the rollback and verdict lines are both present and
/// say ROLLED BACK, not completed.
#[test]
fn the_human_readable_rollback_reports_rolled_back_not_completed() {
    let dir = scratch("rollback-human");
    let artifact = dir.join("release.bin");
    std::fs::write(&artifact, b"fleet payload v3").unwrap();
    let hash = delulu_broker::content_hash(b"fleet payload v3");
    let approved = write_approval(&dir, &artifact.to_string_lossy(), &hash);
    let previous = dir.join("previous.bin");
    std::fs::write(&previous, b"fleet payload v2").unwrap();

    let o = delulu(&[
        "fleet",
        "update",
        &artifact.to_string_lossy(),
        "--members",
        "4",
        "--approved",
        &approved.to_string_lossy(),
        "--previous",
        &previous.to_string_lossy(),
        "--fail-health-at",
        "1",
    ]);
    let out = stdout(&o);
    assert_eq!(o.status.code(), Some(1), "{out}\n{}", stderr(&o));
    assert!(out.contains("ROLLED BACK"), "{out}");
    assert!(!out.contains("completed"), "a rolled-back run must not also claim completion:\n{out}");
}

// ----- usage errors -------------------------------------------------------------------------------

/// `--members 0`, a missing `--members`, and a missing `--previous` are usage errors (exit 2),
/// distinct from the DL1905 safety refusal (exit 1) — the CLI never confuses "you forgot to tell
/// me something" with "I have what you told me, and it does not check out".
#[test]
fn malformed_or_missing_required_flags_are_usage_errors_not_dl1905() {
    let dir = scratch("usage");
    let artifact = dir.join("release.bin");
    std::fs::write(&artifact, b"x").unwrap();
    let hash = delulu_broker::content_hash(b"x");
    let approved = write_approval(&dir, &artifact.to_string_lossy(), &hash);
    let previous = dir.join("previous.bin");
    std::fs::write(&previous, b"y").unwrap();

    let o = delulu(&[
        "fleet",
        "update",
        &artifact.to_string_lossy(),
        "--members",
        "0",
        "--approved",
        &approved.to_string_lossy(),
        "--previous",
        &previous.to_string_lossy(),
    ]);
    assert_eq!(o.status.code(), Some(2), "--members 0 must be a usage error:\n{}", stderr(&o));

    let o = delulu(&[
        "fleet",
        "update",
        &artifact.to_string_lossy(),
        "--approved",
        &approved.to_string_lossy(),
        "--previous",
        &previous.to_string_lossy(),
    ]);
    assert_eq!(o.status.code(), Some(2), "a missing --members must be a usage error:\n{}", stderr(&o));

    let o = delulu(&[
        "fleet",
        "update",
        &artifact.to_string_lossy(),
        "--members",
        "3",
        "--approved",
        &approved.to_string_lossy(),
    ]);
    let err = stderr(&o);
    assert_eq!(o.status.code(), Some(2), "a missing --previous must be a usage error, not DL1905:\n{err}");
    assert!(!err.contains("DL1905"), "{err}");
}

#[test]
fn an_unknown_fleet_subcommand_is_a_usage_error() {
    let o = delulu(&["fleet", "frobnicate"]);
    assert_eq!(o.status.code(), Some(2), "{}", stderr(&o));
}
