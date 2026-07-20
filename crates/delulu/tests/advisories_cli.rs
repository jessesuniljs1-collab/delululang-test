//! Stage 10 phase 10k (Track C, spec §4): the registry advisory feed as `delulu build` sees it.
//! A resolved dependency on a version named in a published advisory is DL1903 — a WARNING by
//! default, an ERROR under `--deny-advisories` (the CI gate). Driven through the real binary end
//! to end, house style shared with `deploy_cli.rs` (fixture packages on disk, `CARGO_BIN_EXE_delulu`).
//!
//! The skip branch is the point of most of these tests: WITHOUT the gate an absent feed is silence,
//! but WITH `--deny-advisories` an absent, unreadable, or partly-unparseable feed is a refusal — a
//! gate that opens because it could not find its evidence is the gate opening on damage.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_adv_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn delulu(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(dir)
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .output()
        .expect("run delulu")
}

fn combined(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

/// A minimal, clean package at a chosen version. The advisory scan iterates every workspace
/// package, so naming the package itself in an advisory is the simplest possible affected-version
/// fixture — the same code path a vulnerable *dependency* takes, without needing a dependency graph.
fn scaffold_pkg(dir: &Path, name: &str, version: &str) {
    let src = dir.join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(
        dir.join("delulu.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"{version}\"\n\n[authority]\neffects = []\n"),
    )
    .unwrap();
    std::fs::write(src.join("main.delulu"), "module x\nfn main(root: Root) {\n}\n").unwrap();
}

/// Write a `delulu.advisories.json` feed at the package's default location.
fn write_feed(dir: &Path, body: &str) {
    std::fs::write(dir.join("delulu.advisories.json"), body).unwrap();
}

fn advisory_feed(package: &str, affected: &[&str], patched: &str) -> String {
    let affected = affected.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(", ");
    format!(
        "{{ \"advisories\": [ {{ \"id\": \"DLSA-2026-0001\", \"package\": \"{package}\", \
         \"affected\": [{affected}], \"patched\": \"{patched}\", \"severity\": \"high\", \
         \"summary\": \"request smuggling via header folding\" }} ] }}"
    )
}

// ===== the DL1903 witness pair =================================================================

/// NEGATIVE witness for `ref.diag.DL1903`: a build whose dependency is on an advised version is
/// flagged. By default this is a warning — the build still succeeds (exit 0) — because an advisory
/// is information, not a wall.
#[test]
fn an_advised_dependency_is_flagged_dl1903() {
    let dir = tmp("advised");
    scaffold_pkg(&dir, "myapp", "1.0.0");
    write_feed(&dir, &advisory_feed("myapp", &["1.0.0"], "1.0.1"));

    let out = delulu(&dir, &["build", "."]);
    let all = combined(&out);
    assert!(out.status.success(), "a plain build only WARNS on an advisory; it must still succeed:\n{all}");
    assert!(all.contains("DL1903"), "expected a DL1903 advisory warning:\n{all}");
    assert!(all.contains("DLSA-2026-0001"), "the advisory id should be named:\n{all}");
    assert!(all.contains("1.0.1"), "the patched-version upgrade hint should be shown:\n{all}");
}

/// POSITIVE witness for `ref.diag.DL1903`: the gate SEEN TO PASS. A build on a version no advisory
/// names clears `--deny-advisories` with exit 0 and no DL1903. A gate witnessed only by its
/// refusals is indistinguishable from a gate that is simply shut (the DL1905/DL1907 precedent).
#[test]
fn a_clean_version_passes_the_deny_advisories_gate() {
    let dir = tmp("clean");
    // The package is on 1.0.1; the advisory only covers 1.0.0. Exact membership: not affected.
    scaffold_pkg(&dir, "myapp", "1.0.1");
    write_feed(&dir, &advisory_feed("myapp", &["1.0.0"], "1.0.1"));

    let out = delulu(&dir, &["build", ".", "--deny-advisories"]);
    let all = combined(&out);
    assert!(out.status.success(), "a clean version must pass the gate:\n{all}");
    assert!(!all.contains("DL1903"), "no advisory should fire for an unaffected version:\n{all}");
}

// ===== the gate: --deny-advisories turns a match into a failure ================================

#[test]
fn deny_advisories_turns_a_match_into_an_error() {
    let dir = tmp("deny_match");
    scaffold_pkg(&dir, "myapp", "1.0.0");
    write_feed(&dir, &advisory_feed("myapp", &["1.0.0"], "1.0.1"));

    let out = delulu(&dir, &["build", ".", "--deny-advisories"]);
    let all = combined(&out);
    assert!(!out.status.success(), "the gate must FAIL the build on a match:\n{all}");
    assert!(all.contains("DL1903"), "the failure must be the coded DL1903:\n{all}");
}

// ===== the skip branch: a gate that cannot find its evidence refuses ============================

#[test]
fn deny_advisories_with_no_feed_refuses_rather_than_passing() {
    let dir = tmp("deny_nofeed");
    scaffold_pkg(&dir, "myapp", "1.0.0");
    // No feed written at all.
    let out = delulu(&dir, &["build", ".", "--deny-advisories"]);
    let all = combined(&out);
    assert!(
        !out.status.success(),
        "a CI gate with no feed to check must not report a clean pass:\n{all}"
    );
    assert!(
        all.contains("no advisory feed"),
        "the refusal should say the feed was missing:\n{all}"
    );
}

#[test]
fn without_the_gate_no_feed_is_silence_not_a_warning() {
    let dir = tmp("silence");
    scaffold_pkg(&dir, "myapp", "1.0.0");
    // No feed, no --deny-advisories.
    let out = delulu(&dir, &["build", "."]);
    let all = combined(&out);
    assert!(out.status.success(), "a plain build with no feed must succeed:\n{all}");
    assert!(!all.contains("DL1903"), "nothing is known, so nothing is said:\n{all}");
}

#[test]
fn deny_advisories_with_an_unparseable_record_refuses() {
    let dir = tmp("deny_malformed");
    scaffold_pkg(&dir, "myapp", "1.0.0");
    // A feed that parses as JSON but whose single record is missing `affected` — a half-record the
    // scanner will not silently treat as "matches nothing" under the gate.
    write_feed(
        &dir,
        "{ \"advisories\": [ { \"id\": \"DLSA-x\", \"package\": \"myapp\" } ] }",
    );
    let out = delulu(&dir, &["build", ".", "--deny-advisories"]);
    let all = combined(&out);
    assert!(
        !out.status.success(),
        "an unparseable record under the gate must refuse, not pass:\n{all}"
    );
    assert!(all.contains("cannot parse") || all.contains("unparseable"), "reason should name the bad record:\n{all}");
}

// ===== --advisory-feed points the scan elsewhere ================================================

#[test]
fn advisory_feed_flag_points_at_an_explicit_file() {
    let dir = tmp("feedflag");
    scaffold_pkg(&dir, "myapp", "1.0.0");
    // The feed lives somewhere the default discovery would never find it.
    let feed = dir.join("elsewhere").join("feed.json");
    std::fs::create_dir_all(feed.parent().unwrap()).unwrap();
    std::fs::write(&feed, advisory_feed("myapp", &["1.0.0"], "1.0.1")).unwrap();

    let out = delulu(
        &dir,
        &["build", ".", "--deny-advisories", "--advisory-feed", &feed.to_string_lossy()],
    );
    let all = combined(&out);
    assert!(!out.status.success(), "the explicit feed should be consulted and the gate should fail:\n{all}");
    assert!(all.contains("DL1903"), "the match from the explicit feed should fire:\n{all}");
}
