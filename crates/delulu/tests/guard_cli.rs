//! Stage 5 chunk 6 phase 5k — the Guard CLI surface, through the real binary.
//!
//! Covers the criteria that are CLI/daemon-shaped:
//! - criterion 9: admin verbs (`guard policy set`) refuse DL1414 without a valid owner code; the
//!   owner code appears exactly once in `broker start` output and NEVER on disk (grep the state dir);
//! - the `guard status` / `guard policy show|set` read+admin surface + the policy-change bound.
//!
//! Live re-run (any temp dir):
//! ```text
//! set DELULU_STATE_DIR=<tmp>\state
//! delulu broker start                      (prints the owner code once, to the terminal)
//! delulu guard status                      (guarded: declassify:*, foreign_c:*, foreign_python:*)
//! delulu guard policy set net:* guarded    (DL1414 — no owner)
//! delulu guard policy set net:* guarded --owner <code>   (ok; states the policy-change bound)
//! delulu broker stop
//! ```

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn delulu_in(cwd: &Path, state: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(cwd)
        .env("DELULU_STATE_DIR", state)
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

struct DaemonGuard {
    state: PathBuf,
}
impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = Command::new(env!("CARGO_BIN_EXE_delulu"))
            .current_dir(std::env::temp_dir())
            .env("DELULU_STATE_DIR", &self.state)
            .args(["broker", "stop"])
            .output();
    }
}

/// Pull the `gow1_…` owner code out of `broker start` output.
fn extract_owner_code(text: &str) -> Option<String> {
    let idx = text.find("gow1_")?;
    let tail = &text[idx..];
    let end = tail.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(tail.len());
    Some(tail[..end].to_string())
}

/// Recursively read every file under `dir` and return whether any contains `needle`.
fn any_file_contains(dir: &Path, needle: &str) -> bool {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(p) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&p) else { continue };
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(bytes) = std::fs::read(&path) {
                if String::from_utf8_lossy(&bytes).contains(needle) {
                    return true;
                }
            }
        }
    }
    false
}

#[test]
fn guard_cli_owner_code_once_never_on_disk_and_policy_admin() {
    let base = std::env::temp_dir().join(format!("delulu_guard_cli_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cwd = base.join("work");
    let state = base.join("state");
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&state).unwrap();

    // ----- broker start prints the owner code exactly once --------------------------------------
    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: state.clone() };
    let start_out = format!("{}{}", stdout(&o), stderr(&o));
    let owner = extract_owner_code(&start_out).expect("broker start prints the guard owner code");
    assert!(owner.starts_with("gow1_"), "owner code shape: {owner}");
    assert_eq!(start_out.matches(&owner).count(), 1, "owner code appears exactly once in broker-start output");

    // ----- guard status shows the default guarded classes ---------------------------------------
    let o = delulu_in(&cwd, &state, &["guard", "status"]);
    assert!(o.status.success(), "guard status: {}", stderr(&o));
    let status = stdout(&o);
    for rule in ["declassify:*", "foreign_c:*", "foreign_python:*"] {
        assert!(status.contains(rule), "guard status lists the default rule `{rule}`: {status}");
    }

    // ----- criterion 9: an admin verb without a valid owner code is DL1414 ------------------------
    let o = delulu_in(&cwd, &state, &["guard", "policy", "set", "net:*", "guarded"]);
    assert_eq!(o.status.code(), Some(1), "admin verb without owner must fail");
    assert!(stderr(&o).contains("DL1414"), "no owner → DL1414: {}", stderr(&o));

    // ----- with the owner code the edit succeeds and states the policy-change bound --------------
    let o = delulu_in(&cwd, &state, &["guard", "policy", "set", "net:*", "guarded", "--owner", &owner]);
    assert!(o.status.success(), "policy set with owner: {}", stderr(&o));
    assert!(
        stderr(&o).contains("guard policy changes take effect: synchronous class — before the next use"),
        "the policy-change bound is stated verbatim: {}",
        stderr(&o)
    );
    let o = delulu_in(&cwd, &state, &["guard", "policy", "show"]);
    assert!(stdout(&o).contains("net:*"), "the new rule shows: {}", stdout(&o));

    // ----- guard status --json is valid machine JSON ---------------------------------------------
    let o = delulu_in(&cwd, &state, &["guard", "status", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("guard status --json is valid JSON");
    assert_eq!(v["command"], "guard");
    assert_eq!(v["bypass"], false);

    // ----- criterion 9: the owner code is NEVER written anywhere under the state dir --------------
    let o = delulu_in(&cwd, &state, &["broker", "stop"]);
    assert!(o.status.success(), "broker stop: {}", stderr(&o));
    assert!(
        !any_file_contains(&state, &owner),
        "criterion 9: the owner code must never be written to disk (found it under {})",
        state.display()
    );

    let _ = std::fs::remove_dir_all(&base);
}
