//! Phase A1 — the Palette, through the real `delulu` binary (Surface addendum §2.5).
//!
//! Witnesses acceptance criteria 8 (precedence + JSON-never-colored + piped-colorless), 9 (three
//! themes, `mono` has no color SGR, `theme.toml` role override, invalid theme ⇒ DL1790 + fallback),
//! and — by never editing any pre-existing test — 10 (zero regression: color is off for the piped,
//! non-TTY streams these tests run under, so the whole existing suite is untouched).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

const REJECT: &str = "tests/conformance/reject/DL0501_undeclared_effect.delulu";
const CLEAN: &str = "examples/demo.delulu";
const ESC: char = '\u{1b}';

/// Run `delulu` with a CLEAN color environment plus the given env overrides, so a stray
/// `NO_COLOR`/`DELULU_COLOR` in the runner's env can never make a color assertion flaky.
fn run(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_delulu"));
    c.current_dir(workspace_root())
        .env_remove("NO_COLOR")
        .env_remove("DELULU_COLOR")
        .env_remove("DELULU_THEME")
        .env_remove("DELULU_THEME_FILE");
    for (k, v) in envs {
        c.env(k, v);
    }
    c.args(args).output().expect("failed to run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

// ----- criterion 8: precedence, JSON-never-colored, piped-colorless -------------------------------

#[test]
fn piped_output_is_colorless_by_default() {
    // No flag, no env, captured through a pipe (non-TTY) ⇒ auto resolves to OFF.
    let o = run(&["check", REJECT], &[]);
    let e = stderr(&o);
    assert!(e.contains("error[DL0501]"), "diagnostic present: {e}");
    assert!(!e.contains(ESC), "piped output must carry zero SGR bytes: {e:?}");
}

#[test]
fn color_always_flag_paints_the_diagnostic() {
    let e = stderr(&run(&["check", REJECT, "--color", "always"], &[]));
    assert!(e.contains(&format!("{ESC}[1;31merror{ESC}[0m")), "severity painted red: {e:?}");
    assert!(e.contains(&format!("{ESC}[1mDL0501{ESC}[0m")), "code painted bold: {e:?}");
}

#[test]
fn json_is_never_colored_even_under_color_always() {
    let o = run(&["check", REJECT, "--json", "--color", "always"], &[]);
    let s = stdout(&o);
    assert!(!s.contains(ESC), "JSON is a machine channel — zero SGR bytes: {s:?}");
    let v: serde_json::Value = serde_json::from_str(&s).expect("still valid JSON");
    assert_eq!(v["diagnostics"][0]["code"], "DL0501");
}

#[test]
fn delulu_color_env_forces_color_when_piped() {
    let o = run(&["check", REJECT], &[("DELULU_COLOR", "always")]);
    assert!(stderr(&o).contains(ESC), "DELULU_COLOR=always colors even a pipe");
}

#[test]
fn no_color_beats_delulu_color_always() {
    let o = run(&["check", REJECT], &[("DELULU_COLOR", "always"), ("NO_COLOR", "1")]);
    assert!(!stderr(&o).contains(ESC), "NO_COLOR beats DELULU_COLOR=always");
}

#[test]
fn color_always_flag_beats_no_color() {
    // The explicit per-invocation flag is the user speaking now — it beats NO_COLOR.
    let o = run(&["check", REJECT, "--color", "always"], &[("NO_COLOR", "1")]);
    assert!(stderr(&o).contains(ESC), "--color always beats NO_COLOR");
}

#[test]
fn color_never_flag_disables_even_with_delulu_color_always() {
    let o = run(&["check", REJECT, "--color", "never"], &[("DELULU_COLOR", "always")]);
    assert!(!stderr(&o).contains(ESC), "--color never wins");
}

#[test]
fn ok_success_line_is_painted() {
    let e = stderr(&run(&["check", CLEAN, "--color", "always"], &[]));
    assert!(e.contains(&format!("{ESC}[32mok:")), "ok: line painted in the success role: {e:?}");
}

// ----- criterion 9: themes ------------------------------------------------------------------------

#[test]
fn mono_theme_emits_no_color_sgr() {
    let e = stderr(&run(&["check", REJECT, "--color", "always", "--theme", "mono"], &[]));
    // mono styles the severity in BOLD only …
    assert!(e.contains(&format!("{ESC}[1merror{ESC}[0m")), "mono severity is bold-only: {e:?}");
    // … and never emits a color parameter (30-37/40-47/90-97/38/48).
    for color in ["31m", "32m", "33m", "34m", "35m", "36m", "37m", "91m", "92m", "93m", "38;", "48;"] {
        assert!(!e.contains(color), "mono leaked a color SGR `{color}`: {e:?}");
    }
}

#[test]
fn bright_theme_uses_bright_colors() {
    let e = stderr(&run(&["check", REJECT, "--color", "always", "--theme", "bright"], &[]));
    assert!(e.contains(&format!("{ESC}[1;91merror{ESC}[0m")), "bright severity is bright red: {e:?}");
}

#[test]
fn theme_toml_role_override_is_honored() {
    let dir = std::env::temp_dir().join(format!("delulu_palette_override_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("theme.toml");
    std::fs::write(&path, "theme = \"default\"\n[roles]\nerror = \"green\"\n").unwrap();
    let e = stderr(&run(
        &["check", REJECT, "--color", "always"],
        &[("DELULU_THEME_FILE", path.to_str().unwrap())],
    ));
    assert!(e.contains(&format!("{ESC}[32merror{ESC}[0m")), "error overridden to green: {e:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn invalid_theme_is_dl1790_warning_and_falls_back() {
    // A bad theme name never hard-fails: DL1790 warning, default theme, command still runs.
    let o = run(&["check", CLEAN, "--theme", "neon"], &[]);
    let e = stderr(&o);
    assert!(e.contains("DL1790"), "invalid theme warns with DL1790: {e:?}");
    assert!(e.contains("warning[DL1790]"), "it is a warning severity: {e:?}");
    assert!(e.contains("checked clean"), "the check still ran to completion: {e:?}");
    assert_eq!(o.status.code(), Some(0), "a bad theme does not change the exit code");
}

#[test]
fn malformed_theme_file_is_dl1790_and_still_runs() {
    let dir = std::env::temp_dir().join(format!("delulu_palette_bad_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("theme.toml");
    std::fs::write(&path, "this is = = not valid\n[[[\n").unwrap();
    let o = run(&["check", CLEAN], &[("DELULU_THEME_FILE", path.to_str().unwrap())]);
    let e = stderr(&o);
    assert!(e.contains("DL1790"), "malformed theme.toml warns with DL1790: {e:?}");
    assert_eq!(o.status.code(), Some(0));
    let _ = std::fs::remove_dir_all(&dir);
}

// ----- explain surface ----------------------------------------------------------------------------

#[test]
fn explain_e_palette_describes_the_model() {
    let s = stdout(&run(&["explain", "E-PALETTE"], &[]));
    assert!(s.contains("Palette"), "{s}");
    assert!(s.contains("NO_COLOR"), "resolution order documented: {s}");
    assert!(s.contains("mono"), "themes documented: {s}");
}
