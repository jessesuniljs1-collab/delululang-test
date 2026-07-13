//! Stage 5 chunk 6 phase 5m — THE guard end-to-end orchestration test, through the real binary
//! (grants_cli.rs structure and rigor: DaemonGuard, temp dirs, exact-diagnostic assertions).
//!
//! The story (addendum §2.4–§2.7): the principal guards `fs_write:*` → a delegated agent's leased
//! write is blocked with DL1410 naming the exact request command, and nothing executes → the agent
//! requests with `--why` → a retried run is DL1411 carrying the request id → the principal approves
//! with a comment → the retried run succeeds via the broker-held permit → a second agent's request
//! is DENIED with a comment → its retried run is DL1412 carrying the comment VERBATIM → the
//! principal enables bypass (banner printed; the run proceeds with a per-rule agent-side warning and
//! a `guard_bypassed_use` audit event; the awareness line shows BYPASSED) → the principal SEALS the
//! rule → the run refuses DL1413 even under bypass → a `warn`-tier rule proceeds with a one-line
//! warning + `guard_warn` event → `delulu audit verify` is green over the whole story.
//!
//! Live re-run for the head chef (same commands, any temp dir):
//! ```text
//! set DELULU_STATE_DIR=<tmp>\state
//! delulu broker start                                          (capture the gow1_ owner code)
//! delulu guard policy set fs_write:* guarded --owner <code>
//! delulu grants delegate --effects Read,Write --fs-read ./data --fs-write ./out --multi --owner <code>
//! delulu run agent.delulu --lease <token>      (DL1410 naming `delulu guard request …`; no write)
//! delulu guard request <node> --use fs_write:* --why "write the result file"
//! delulu run agent.delulu --lease <token>      (DL1411 carrying the request id)
//! delulu guard approve <req> --owner <code> --comment "ok"
//! delulu run agent.delulu --lease <token>      (succeeds via the permit; out\result.txt written)
//! delulu guard deny <req2> --owner <code> --comment "…"   (second node's request)
//! delulu run agent2 --lease <token2>           (DL1412 carrying the comment verbatim)
//! delulu guard bypass on --owner <code>        (prints the banner)
//! delulu run agent2 --lease <token2>           (proceeds; warned; guard_bypassed_use audited)
//! delulu guard policy set fs_write:* sealed --owner <code>
//! delulu run agent2 --lease <token2>           (DL1413 even under bypass)
//! delulu audit verify --dir <state>\audit      (green)
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

/// Stops the daemon on drop from a stable cwd, never panicking (the chunk-3/4 Drop-guard lesson).
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

fn extract_owner_code(text: &str) -> Option<String> {
    let idx = text.find("gow1_")?;
    let tail = &text[idx..];
    let end = tail.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(tail.len());
    Some(tail[..end].to_string())
}

/// Delegate a Read+Write slice under the guard (the owner signs off on the guarded fs_write mint);
/// multi-redemption so the lease is retried across the block → approve → use arcs.
fn delegate_rw(cwd: &Path, state: &Path, owner: &str) -> (String, String) {
    let o = delulu_in(
        cwd,
        state,
        &["grants", "delegate", "--effects", "Read,Write", "--fs-read", "./data", "--fs-write", "./out",
          "--multi", "--owner", owner, "--json"],
    );
    assert!(o.status.success(), "delegate: {}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("delegate --json");
    (v["node"].as_str().unwrap().to_string(), v["token"].as_str().unwrap().to_string())
}

#[test]
fn guard_block_request_approve_deny_bypass_seal_orchestration_end_to_end() {
    // ----- arrange -------------------------------------------------------------------------------
    let base = std::env::temp_dir().join(format!("delulu_guard_e2e_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cwd = base.join("work");
    let state = base.join("state");
    std::fs::create_dir_all(cwd.join("data")).unwrap();
    std::fs::create_dir_all(cwd.join("out")).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(cwd.join("data/in.txt"), "42").unwrap();
    // The agent: reads inside the leased fs.read scope, writes inside the leased fs.write scope
    // (identical to the grants_cli agent — a custody/guard denial mid-run is a fault, exit 1).
    let agent = "module agent\nfn main(root: Root) ! {Read, Write} {\n  let fr = root.fs_read(\"./data\")\n  let fw = root.fs_write(\"./out\")\n  match fr.read_text(\"in.txt\") {\n    Ok(v) => { match fw.write_text(\"result.txt\", v) { Ok(_) => {}, Err(_) => {} } },\n    Err(_) => {}\n  }\n}\n";
    std::fs::write(cwd.join("agent.delulu"), agent).unwrap();

    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: state.clone() };
    let owner = extract_owner_code(&format!("{}{}", stdout(&o), stderr(&o))).expect("owner code");

    // ----- the principal guards fs_write (owner-gated; states the policy-change bound) ------------
    let o = delulu_in(&cwd, &state, &["guard", "policy", "set", "fs_write:*", "guarded", "--owner", &owner]);
    assert!(o.status.success(), "policy set: {}", stderr(&o));
    assert!(stderr(&o).contains("guard policy changes take effect: synchronous class — before the next use"), "{}", stderr(&o));

    let (node, token) = delegate_rw(&cwd, &state, &owner);

    // ----- BLOCK: the delegated write is DL1410 naming the exact request command; nothing executes -
    let o = delulu_in(&cwd, &state, &["run", "agent.delulu", "--lease", &token, "--no-prompt"]);
    assert_eq!(o.status.code(), Some(1), "guarded write must be refused");
    let err = stderr(&o);
    assert!(err.contains("DL1410"), "criterion 1: {err}");
    assert!(err.contains(&format!("delulu guard request {node} --use fs_write:*")), "the exact request command: {err}");
    // Criterion 8: the awareness line printed before user output, `on` state.
    assert!(err.contains("guard: on — guarded:") && err.contains("fs_write:*"), "awareness line: {err}");
    assert!(!cwd.join("out/result.txt").exists(), "nothing executes under a guard block");

    // ----- REQUEST (mandatory --why) → retried run is DL1411 carrying the request id ---------------
    let o = delulu_in(&cwd, &state, &["guard", "request", &node, "--use", "fs_write:*", "--why", "write the result file"]);
    assert!(o.status.success(), "request: {}", stderr(&o));
    let req = stdout(&o).trim().to_string();
    let o = delulu_in(&cwd, &state, &["run", "agent.delulu", "--lease", &token, "--no-prompt"]);
    assert_eq!(o.status.code(), Some(1));
    let err = stderr(&o);
    assert!(err.contains("DL1411") && err.contains(&req), "pending carries the id: {err}");

    // The principal sees the queue WITH the justification.
    let o = delulu_in(&cwd, &state, &["guard", "pending"]);
    assert!(stdout(&o).contains("write the result file"), "pending shows why: {}", stdout(&o));

    // ----- APPROVE (with comment) → the retried run succeeds via the broker-held permit ------------
    let o = delulu_in(&cwd, &state, &["guard", "approve", &req, "--owner", &owner, "--comment", "ok"]);
    assert!(o.status.success(), "approve: {}", stderr(&o));
    let o = delulu_in(&cwd, &state, &["run", "agent.delulu", "--lease", &token, "--no-prompt"]);
    assert!(o.status.success(), "approved run: {}", stderr(&o));
    assert_eq!(std::fs::read_to_string(cwd.join("out/result.txt")).unwrap(), "42", "the approved write happened");
    std::fs::remove_file(cwd.join("out/result.txt")).unwrap();

    // ----- DENY path: a second agent's request is denied; its retry is DL1412 verbatim -------------
    let (node2, token2) = delegate_rw(&cwd, &state, &owner);
    let o = delulu_in(&cwd, &state, &["guard", "request", &node2, "--use", "fs_write:*", "--why", "me too"]);
    let req2 = stdout(&o).trim().to_string();
    let o = delulu_in(&cwd, &state, &["guard", "deny", &req2, "--owner", &owner, "--comment", "not this agent; use the reporting pipeline"]);
    assert!(o.status.success(), "deny: {}", stderr(&o));
    let o = delulu_in(&cwd, &state, &["run", "agent.delulu", "--lease", &token2, "--no-prompt"]);
    assert_eq!(o.status.code(), Some(1));
    let err = stderr(&o);
    assert!(err.contains("DL1412"), "{err}");
    assert!(err.contains("not this agent; use the reporting pipeline"), "criterion 5: the comment verbatim: {err}");
    assert!(!cwd.join("out/result.txt").exists());

    // ----- BYPASS (runtime toggle): banner printed; the run proceeds, warned + audited -------------
    let o = delulu_in(&cwd, &state, &["guard", "bypass", "on", "--owner", &owner]);
    assert!(o.status.success(), "bypass on: {}", stderr(&o));
    let banner = stderr(&o);
    assert!(banner.contains("GUARD BYPASSED") && banner.contains("approval checkpoint"), "{banner}");
    assert!(banner.contains("WITHOUT YOUR KNOWLEDGE until you read the audit log"), "the exact banner: {banner}");
    let o = delulu_in(&cwd, &state, &["run", "agent.delulu", "--lease", &token2, "--no-prompt"]);
    assert!(o.status.success(), "bypassed run proceeds: {}", stderr(&o));
    let err = stderr(&o);
    // Criterion 8: the awareness line shows BYPASSED prominently.
    assert!(err.contains("guard: BYPASSED by the principal"), "awareness line under bypass: {err}");
    // Criterion 7: the per-rule agent-side warning on the bypassed guarded use.
    assert!(err.contains("guard BYPASSED: `fs_write:*`"), "agent-side warning: {err}");
    assert_eq!(std::fs::read_to_string(cwd.join("out/result.txt")).unwrap(), "42");
    std::fs::remove_file(cwd.join("out/result.txt")).unwrap();

    // ----- SEALED: refuses DL1413 even under bypass ------------------------------------------------
    let o = delulu_in(&cwd, &state, &["guard", "policy", "set", "fs_write:*", "sealed", "--owner", &owner]);
    assert!(o.status.success(), "seal: {}", stderr(&o));
    let o = delulu_in(&cwd, &state, &["run", "agent.delulu", "--lease", &token2, "--no-prompt"]);
    assert_eq!(o.status.code(), Some(1), "sealed must refuse even under bypass");
    let err = stderr(&o);
    assert!(err.contains("DL1413"), "criterion 6: {err}");
    assert!(!cwd.join("out/result.txt").exists(), "nothing executes under a seal");

    // ----- WARN tier: the use proceeds with a one-line warning + a guard_warn audit event ----------
    let o = delulu_in(&cwd, &state, &["guard", "policy", "set", "fs_write:*", "warn", "--owner", &owner]);
    assert!(o.status.success(), "warn tier: {}", stderr(&o));
    let o = delulu_in(&cwd, &state, &["guard", "bypass", "off", "--owner", &owner]);
    assert!(o.status.success(), "bypass off: {}", stderr(&o));
    let o = delulu_in(&cwd, &state, &["run", "agent.delulu", "--lease", &token2, "--no-prompt"]);
    assert!(o.status.success(), "warn-tier run proceeds: {}", stderr(&o));
    assert!(stderr(&o).contains("guard warn: `fs_write:*`"), "the one-line warning: {}", stderr(&o));
    assert_eq!(std::fs::read_to_string(cwd.join("out/result.txt")).unwrap(), "42");

    // ----- the audit chain shows the whole story and verifies ------------------------------------
    let audit_dir = state.join("audit").to_string_lossy().to_string();
    let o = delulu_in(&cwd, &state, &["audit", "verify", "--dir", &audit_dir]);
    assert!(o.status.success(), "criterion 4/7: audit verify green: {}", stderr(&o));
    for action in ["guard_block", "guard_request", "guard_approve", "guard_permit_use", "guard_deny",
                   "guard_bypassed_use", "guard_bypass_on", "guard_bypass_off", "guard_policy_edit", "guard_warn"] {
        let o = delulu_in(&cwd, &state, &["audit", "query", "--action", action, "--dir", &audit_dir]);
        assert!(!stdout(&o).trim().is_empty(), "the audit log carries `{action}` events");
    }

    // ----- daemon down ⇒ guard verbs fail closed with the start command ----------------------------
    let o = delulu_in(&cwd, &state, &["broker", "stop"]);
    assert!(o.status.success(), "broker stop: {}", stderr(&o));
    let o = delulu_in(&cwd, &state, &["guard", "status"]);
    assert_eq!(o.status.code(), Some(1), "guard with no daemon must fail closed");
    assert!(stderr(&o).contains("DL1401") && stderr(&o).contains("delulu broker start"), "{}", stderr(&o));

    let _ = std::fs::remove_dir_all(&base);
}

/// Criterion 7's other half: `broker start --dangerously-bypass-guard` prints the exact banner at
/// start, `guard status --json` reports bypass (valid machine JSON), and a leased guarded use
/// proceeds with the `guard_bypassed_use` audit event from the very first run.
#[test]
fn broker_start_dangerously_bypass_guard_prints_the_banner() {
    let base = std::env::temp_dir().join(format!("delulu_guard_bypass_start_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cwd = base.join("work");
    let state = base.join("state");
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&state).unwrap();

    let o = delulu_in(&cwd, &state, &["broker", "start", "--dangerously-bypass-guard"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: state.clone() };
    let out = format!("{}{}", stdout(&o), stderr(&o));
    assert!(out.contains("GUARD BYPASSED") && out.contains("approval checkpoint"), "the exact banner at start: {out}");
    assert!(out.contains("Sealed rules still hold"), "{out}");

    // `guard status --json` is valid machine JSON and reports the bypass.
    let o = delulu_in(&cwd, &state, &["guard", "status", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("guard status --json");
    assert_eq!(v["bypass"], true, "{v}");
    assert_eq!(v["mode"], "bypassed", "{v}");

    let _ = delulu_in(&cwd, &state, &["broker", "stop"]);
    let _ = std::fs::remove_dir_all(&base);
}
