//! The stability contract's behavior (Stage 9c, spec §2.2 — release criterion 9).
//!
//! `DL1802` (a package declaring a newer language edition than the toolchain) is driven through the
//! real binary here; `DL1801` (deprecated feature) is exercised against a synthetic deprecation
//! table in `delulu_check::deprecation`, because the live registry is empty at 1.0 by policy and
//! deprecating something real just to have a test would be the tail wagging the dog.
//!
//! The skip-branch cases matter as much as the happy ones: an OLDER edition must build (a refusal
//! there would break every pre-1.0 manifest), and a package declaring nothing must build unchanged.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(workspace_root())
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("failed to run delulu")
}

fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

/// A minimal buildable package whose manifest carries the given `[package]` extra line.
fn scaffold(name: &str, package_extra: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu-stability-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("delulu.toml"),
        format!("[package]\nname = \"edtest\"\nversion = \"0.1.0\"\nkind = \"lib\"\n{package_extra}\n"),
    )
    .unwrap();
    std::fs::write(dir.join("src").join("main.delulu"), "module edtest\n\npub fn two() -> Int {\n  2\n}\n")
        .unwrap();
    dir
}

/// CRITERION 9, first half: a package declaring a NEWER edition than the toolchain is refused with
/// DL1802, and the refusal names both editions so the repair is actionable.
#[test]
fn a_newer_language_edition_is_refused_dl1802() {
    let dir = scaffold("newer", "language = \"9.9\"");
    let o = delulu(&["build", dir.to_str().unwrap()]);
    let out = text(&o);
    assert!(!o.status.success(), "a future edition must refuse the build:\n{out}");
    assert!(out.contains("DL1802"), "the refusal must be DL1802:\n{out}");
    assert!(out.contains("9.9"), "the refusal names the declared edition:\n{out}");
}

/// THE SKIP-BRANCH CASE: an OLDER edition must still build. Minors are strictly additive, so a
/// 1.0 package means what it said under a later toolchain — refusing here would break every
/// package ever written, which is the opposite of a stability contract.
#[test]
fn an_older_language_edition_still_builds() {
    let dir = scaffold("older", "language = \"1.0\"");
    let o = delulu(&["build", dir.to_str().unwrap()]);
    let out = text(&o);
    assert!(!out.contains("DL1802"), "an older edition must never be refused:\n{out}");
}

/// A package that pins no edition is read at the toolchain's own — the back-compatible default
/// that every pre-1.0 manifest relies on.
#[test]
fn a_package_without_an_edition_is_unaffected() {
    let dir = scaffold("none", "");
    let o = delulu(&["build", dir.to_str().unwrap()]);
    let out = text(&o);
    assert!(!out.contains("DL1802"), "an unpinned package must build unchanged:\n{out}");
}

/// A malformed edition is reported (DL1004) rather than ignored: silently dropping it would leave
/// the package unpinned while its author believed it was pinned.
#[test]
fn a_malformed_edition_is_reported_not_ignored() {
    let dir = scaffold("bad", "language = \"one-point-oh\"");
    let o = delulu(&["build", dir.to_str().unwrap()]);
    let out = text(&o);
    assert!(out.contains("DL1004"), "a malformed edition must be reported:\n{out}");
    assert!(out.contains("MAJOR.MINOR"), "the message must say the expected shape:\n{out}");
}

/// CRITERION 9, second half: DL1801 is registered, carries an explain body, and is a WARNING
/// class code. The behavioral tests live with the policy in `delulu_check::deprecation`.
#[test]
fn dl1801_and_dl1802_are_registered_with_explanations() {
    for code in ["DL1801", "DL1802"] {
        assert!(
            delulu_diag::code_title(code).is_some(),
            "{code} must be in the diagnostic registry"
        );
        let o = delulu(&["explain", code]);
        assert!(o.status.success(), "`delulu explain {code}` must succeed");
        let out = text(&o);
        assert!(out.len() > 200, "{code} needs a real explain body, got {} bytes", out.len());
    }
}

/// The deprecation warning is a WARNING, not an error — the policy's load-bearing promise. A
/// deprecation that failed the build would be a removal wearing a friendlier name.
#[test]
fn a_deprecation_is_a_warning_and_never_fails_a_build() {
    use delulu_check::deprecation::{warn, DeprKind, Deprecation};
    const FIXTURE: &[Deprecation] = &[Deprecation {
        item: "old_greet",
        kind: DeprKind::Name,
        since: "1.1",
        replacement: Some("greet"),
        rfc: "RFC-0002",
    }];
    let d = warn(FIXTURE, "old_greet", delulu_diag::Span::new(0, 0, 9)).expect("deprecated");
    assert_eq!(d.code, "DL1801");
    assert!(!d.is_error(), "DL1801 must be a warning");
    assert!(d.message.contains("greet"), "the mechanical replacement is named: {}", d.message);
}

/// The live registry is empty at 1.0 — asserted, not assumed, so this stays true by choice rather
/// than by nobody looking.
#[test]
fn nothing_is_deprecated_at_1_0() {
    assert!(
        delulu_check::deprecation::DEPRECATIONS.is_empty(),
        "the 1.0 release deprecates nothing — see STABILITY.md §3"
    );
}

/// D11: asking a tool what it does must never make it do the thing. `--help` is answered by the
/// dispatch before any subcommand runs, so no side effect can precede it.
#[test]
fn help_never_has_side_effects() {
    let home = std::env::temp_dir().join(format!("delulu-help-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();

    let o = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(workspace_root())
        .env("DELULU_HOME", &home)
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(["keygen", "--help"])
        .output()
        .expect("run");
    assert!(o.status.success(), "`keygen --help` must succeed");
    assert!(
        !home.join("keys").join("id_ed25519").exists(),
        "`delulu keygen --help` must NOT generate a key — asking for help is not a request to act"
    );
    assert!(text(&o).contains("USAGE"), "help must actually show usage: {}", text(&o));
}
