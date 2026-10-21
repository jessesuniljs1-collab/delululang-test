//! Stage 5 phase 5j — THE end-to-end orchestration test, through the real binary (chunk 5's one
//! process-spawning test, like chunk 3's daemon lifecycle and chunk 4's foreign worker).
//!
//! The story (spec §3.3, the stage's payoff): a human/orchestrator delegates a slice of authority
//! and receives a portable lease token → an agent runs a program under `--lease <token>` and can
//! do exactly what the slice allows → a second use of the single-use token is DL1407 (criterion
//! 5) → a widening delegation under the node is DL0802 carrying the computed intersection
//! (criterion 3) → `grants tree`/`list`/`inspect` show the tree → `grants revoke` kills the node
//! AND its delegated child transitively (criterion 4) → a post-revocation run fails DL1403 with
//! the revoking audit seq (criterion 1's shape) → with the daemon down, every `grants` verb is
//! DL1401 with the exact start command (invariant 27).
//!
//! Live re-run for the head chef (same commands, any temp dir):
//! ```text
//! set DELULU_STATE_DIR=<tmp>\state
//! delulu broker start
//! delulu grants delegate --effects Read,Write --fs-read ./data --fs-write ./out --ttl 1h --multi
//! delulu run agent.delulu --lease <token>     (succeeds; out\result.txt written)
//! delulu grants tree                          (root → delegated node, live)
//! delulu grants revoke <node>                 (prints the §4.2 bound)
//! delulu run agent.delulu --lease <token>     (fails DL1403, carries the revoking seq)
//! delulu broker stop
//! delulu grants tree                          (fails DL1401 with the start command)
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

/// Stops the daemon on drop from a stable cwd, never panicking (the chunk-3/4 Drop-guard lesson:
/// a Drop panic masks the real assertion, and the test's work dir may already be gone).
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

#[test]
fn delegate_lease_run_revoke_orchestration_end_to_end() {
    // ----- arrange -------------------------------------------------------------------------------
    let base = std::env::temp_dir().join(format!("delulu_grants_cli_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cwd = base.join("work");
    let state = base.join("state");
    std::fs::create_dir_all(cwd.join("data")).unwrap();
    std::fs::create_dir_all(cwd.join("out")).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(cwd.join("data/in.txt"), "42").unwrap();
    // The agent: reads inside the leased fs.read scope, writes inside the leased fs.write scope.
    let agent = "module agent\nfn main(root: Root) ! {Read, Write} {\n  let fr = root.fs_read(\"./data\")\n  let fw = root.fs_write(\"./out\")\n  match fr.read_text(\"in.txt\") {\n    Ok(v) => { match fw.write_text(\"result.txt\", v) { Ok(_) => {}, Err(_) => {} } },\n    Err(_) => {}\n  }\n}\n";
    std::fs::write(cwd.join("agent.delulu"), agent).unwrap();

    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: state.clone() };

    // ----- delegate: the orchestrator mints a slice + token (spec §3.2/§3.3) ----------------------
    // Single-use token first, to prove criterion 5.
    let o = delulu_in(
        &cwd,
        &state,
        &["grants", "delegate", "--effects", "Read,Write", "--fs-read", "./data", "--fs-write", "./out", "--ttl", "1h", "--json"],
    );
    assert!(o.status.success(), "delegate: {}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("delegate --json");
    let node = v["node"].as_str().expect("delegated node id").to_string();
    let parent = v["parent"].as_str().expect("auto-issued root id").to_string();
    let token = v["token"].as_str().expect("lease token").to_string();
    assert!(token.starts_with("dlt1_"), "portable versioned token: {token}");
    assert_ne!(node, parent);

    // ----- the agent runs under the lease — and ONLY under the lease ------------------------------
    let o = delulu_in(&cwd, &state, &["run", "agent.delulu", "--lease", &token, "--no-prompt"]);
    assert!(o.status.success(), "lease run failed: {}\n{}", stderr(&o), stdout(&o));
    assert!(stderr(&o).contains("custody: daemon"), "custody label: {}", stderr(&o));
    assert!(stderr(&o).contains(&node), "the run names its delegated node: {}", stderr(&o));
    assert_eq!(std::fs::read_to_string(cwd.join("out/result.txt")).unwrap(), "42", "the leased write happened");

    // Criterion 5: the second use of the single-redemption token is DL1407 — and performs nothing.
    std::fs::remove_file(cwd.join("out/result.txt")).unwrap();
    let o = delulu_in(&cwd, &state, &["run", "agent.delulu", "--lease", &token, "--no-prompt"]);
    assert_eq!(o.status.code(), Some(1), "second redemption must fail");
    assert!(stderr(&o).contains("DL1407"), "criterion 5 (single redemption): {}", stderr(&o));
    assert!(!cwd.join("out/result.txt").exists(), "a refused lease performs nothing");

    // ----- criterion 3: a widening delegation under the node is DL0802 with the intersection ------
    let o = delulu_in(
        &cwd,
        &state,
        &["grants", "delegate", "--parent", &node, "--effects", "Read,Net", "--net", "evil.example", "--fs-read", "./data"],
    );
    assert_eq!(o.status.code(), Some(1), "widening must be refused");
    let err = stderr(&o);
    assert!(err.contains("DL0802"), "criterion 3: {err}");
    assert!(err.contains("effects={Read}"), "the refusal carries the computed intersection (never widens): {err}");

    // A (reflexive) delegation under the node succeeds — this child proves transitive revocation
    // next. Multi-redemption, so the post-revocation run exercises the DL1403 path (a dead lease),
    // not the single-use DL1407 path; same effect set as the agent, so the failure below is the
    // custody revocation, never a missing root slice (DL0703).
    let o = delulu_in(
        &cwd,
        &state,
        &[
            "grants",
            "delegate",
            "--parent",
            &node,
            "--effects",
            "Read,Write",
            "--fs-read",
            "./data",
            "--fs-write",
            "./out",
            "--multi",
            "--json",
        ],
    );
    assert!(o.status.success(), "narrowing delegate: {}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("delegate --json");
    let child = v["node"].as_str().unwrap().to_string();
    let child_token = v["token"].as_str().unwrap().to_string();

    // ----- the read surface: tree / list / inspect ------------------------------------------------
    let o = delulu_in(&cwd, &state, &["grants", "tree"]);
    assert!(o.status.success(), "tree: {}", stderr(&o));
    let tree = stdout(&o);
    assert!(tree.contains(&parent) && tree.contains(&node) && tree.contains(&child), "{tree}");

    let o = delulu_in(&cwd, &state, &["grants", "list", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("list --json");
    assert!(v["count"].as_u64().unwrap() >= 3, "root + node + child listed: {v}");

    let o = delulu_in(&cwd, &state, &["grants", "inspect", &node]);
    assert!(o.status.success(), "inspect: {}", stderr(&o));
    let ins = stdout(&o);
    // The fs scope carries the absolutized, lexically-normalized path (the same frame the agent's
    // runtime resolves against), so assert on the shape rather than the raw `./data` literal.
    assert!(ins.contains(&node) && ins.contains("live") && ins.contains("fs.read=["), "{ins}");
    assert!(ins.contains("data"), "the granted subtree is named: {ins}");

    // ----- revoke the delegated node: transitive over its child (criterion 4) ---------------------
    let o = delulu_in(&cwd, &state, &["grants", "revoke", &node, "--json"]);
    assert!(o.status.success(), "revoke: {}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("revoke --json");
    let revoked: Vec<String> = v["newly_revoked"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
    assert!(revoked.contains(&node) && revoked.contains(&child), "transitive: {revoked:?}");
    let by_seq = v["by_seq"].as_u64().unwrap();
    // The revocation reply states the honest §4.2 bound — never "immediate" (playbook trap 3).
    assert!(v["revocation_takes_effect"].as_str().unwrap().contains("before the next use"), "{v}");

    // `grants tree` shows the whole subtree revoked, the root still live (criterion 4's shape).
    let o = delulu_in(&cwd, &state, &["grants", "tree"]);
    let tree = stdout(&o);
    assert_eq!(tree.matches(&format!("revoked@{by_seq}")).count(), 2, "node + child revoked by seq {by_seq}: {tree}");

    // ----- the next effectful use under the dead subtree is DL1403 with the revoking seq ----------
    let o = delulu_in(&cwd, &state, &["run", "agent.delulu", "--lease", &child_token, "--no-prompt"]);
    assert_eq!(o.status.code(), Some(1), "a revoked lease must not run");
    let err = stderr(&o);
    assert!(err.contains("DL1403"), "criterion 1/4 shape (revoked lease): {err}");
    assert!(err.contains(&by_seq.to_string()), "DL1403 carries the revoking audit seq: {err}");
    assert!(!cwd.join("out/result.txt").exists(), "nothing ran under the revoked lease");

    // The whole daemon-written audit chain verifies (criterion 10's live half).
    let audit_dir = state.join("audit").to_string_lossy().to_string();
    let o = delulu_in(&cwd, &state, &["audit", "verify", "--dir", &audit_dir]);
    assert!(o.status.success(), "audit verify: {}", stderr(&o));

    // ----- daemon down ⇒ every grants verb is DL1401 with the exact start command -----------------
    let o = delulu_in(&cwd, &state, &["broker", "stop"]);
    assert!(o.status.success(), "broker stop: {}", stderr(&o));
    let o = delulu_in(&cwd, &state, &["grants", "tree"]);
    assert_eq!(o.status.code(), Some(1), "grants with no daemon must fail closed");
    let err = stderr(&o);
    assert!(err.contains("DL1401"), "{err}");
    assert!(err.contains("delulu broker start"), "the exact start command: {err}");

    let _ = std::fs::remove_dir_all(&base);
}

/// RW 6.15 (found by on-demand routine run 5): a lease run's DL0703 told its holder to `pass --grant fs.read=…` — a flag
/// the same run refuses ("a `--lease` run derives its authority from the delegated node"), on both engines. It now names
/// the delegation that lacks the grant; an ordinary run beside it still names the `--grant`.
#[test]
fn a_lease_runs_refusal_names_its_delegation_on_both_engines() {
    let base = std::env::temp_dir().join(format!("delulu_grants_remedy_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cwd = base.join("w");
    let state = base.join("s");
    std::fs::create_dir_all(cwd.join("data/pub")).unwrap();
    std::fs::create_dir_all(cwd.join("data/priv")).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    let agent = "module agent\nfn main(root: Root) ! {Read} {\n  let fr = root.fs_read(\"./data/priv\")\n  match fr.read_text(\"f.txt\") { Ok(_) => {}, Err(_) => {} }\n}\n";
    std::fs::write(cwd.join("agent.delulu"), agent).unwrap();

    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: state.clone() };
    let o = delulu_in(&cwd, &state, &["grants", "delegate", "--effects", "Read", "--fs-read", "./data/pub", "--ttl", "1h", "--multi", "--json"]);
    assert!(o.status.success(), "delegate: {}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("delegate --json");
    let (node, token) = (v["node"].as_str().unwrap().to_string(), v["token"].as_str().unwrap().to_string());

    for engine in [&[][..], &["--engine", "wasm"][..]] {
        let mut args = vec!["run", "agent.delulu", "--lease", &token, "--no-prompt"];
        args.extend_from_slice(engine);
        let o = delulu_in(&cwd, &state, &args);
        let err = stderr(&o);
        assert_eq!(o.status.code(), Some(1), "{engine:?}: {err}");
        assert!(err.contains("DL0703"), "{engine:?}: {err}");
        assert!(
            err.contains(&format!("node `{node}`")) && err.contains("does not carry `fs.read=./data/priv`"),
            "{engine:?}: the refusal names the delegation that lacks the grant: {err}"
        );
        assert!(!err.contains("pass `--grant"), "{engine:?}: a lease run is never told to pass a `--grant` it refuses: {err}");
    }
    let o = delulu_in(&cwd, &state, &["run", "agent.delulu", "--no-prompt", "--grant", "fs.read=./data/pub"]);
    assert!(stderr(&o).contains("pass `--grant fs.read=./data/priv`"), "an ordinary run is still told what to grant: {}", stderr(&o));
    drop(_guard);
    let _ = std::fs::remove_dir_all(&base);
}

/// RW 4.42 (routine run 7, AUDIT-TEXT-1's next surface): a delegation's scopes are the delegating party's
/// strings, and `grants tree`, `grants list` and `grants inspect` printed them raw on the owner's terminal —
/// witnessed on `634e9f0`: an escape set the title and cleared the screen, and a line break in a path began
/// a forged line of its own. Each field is now shown escaped on its line; `--json` stays exact.
#[test]
fn a_delegations_strings_are_shown_escaped_in_tree_list_and_inspect() {
    let base = std::env::temp_dir().join(format!("delulu_grants_text_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cwd = base.join("w");
    let state = base.join("s");
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: state.clone() };
    let path = "./out/\u{1b}]0;PWNED\u{7}\u{1b}[2J\ng_forged [live] effects={Read,Write,Net}";
    let host = "evil\u{1b}[31m.example";
    let o = delulu_in(
        &cwd,
        &state,
        &["grants", "delegate", "--effects", "Write,Net", "--fs-write", path, "--net", host, "--ttl", "1h", "--json"],
    );
    assert!(o.status.success(), "delegate: {}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("delegate --json");
    let node = v["node"].as_str().expect("node").to_string();

    for args in [vec!["grants", "tree"], vec!["grants", "list"], vec!["grants", "inspect", &node]] {
        let o = delulu_in(&cwd, &state, &args);
        assert!(o.status.success(), "{args:?}: {}", stderr(&o));
        let out = stdout(&o);
        assert!(!out.contains('\u{1b}') && !out.contains('\u{7}'), "{args:?}: raw control characters reached the owner: {out:?}");
        assert!(!out.lines().any(|l| l.starts_with("g_forged")), "{args:?}: the path's line break forged a line: {out:?}");
        assert!(out.contains("\\u{1b}]0;PWNED"), "{args:?}: the string is shown, escaped: {out:?}");
    }
    let o = delulu_in(&cwd, &state, &["grants", "inspect", &node, "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("inspect --json");
    assert!(v.to_string().contains("evil\\u001b[31m.example"), "the JSON carries the delegation's own bytes: {v}");
}

/// **RW 4.70: `grants list` and `grants tree` read a tree past one frame.** Each asked the daemon for the whole tree in one
/// answer, and every answer is capped at one 16 MiB frame — the tree keeps every node with its holder's description, words of
/// any length — so past it both were refused (before D-V2-161, a dropped connection read as `broker unreachable`). They read
/// it page by page now: every node listed once, and every node drawn in the tree under its parent.
#[test]
fn grants_list_and_tree_read_a_tree_past_one_frame() {
    let base = std::env::temp_dir().join(format!("dlgr-{}-big", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("w"), base.join("s"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let guard = DaemonGuard { state: state.clone() };
    let o = delulu_in(&cwd, &state, &["grants", "delegate", "--effects", "Write", "--json", "--holder-desc", "the parent"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let parent = serde_json::from_str::<serde_json::Value>(&stdout(&o)).unwrap()["node"].as_str().unwrap().to_string();
    // Under each OS's limit on one argument (Windows: 32,767 characters a command line).
    let desc_len: usize = if cfg!(windows) { 30_000 } else { 120_000 };
    let count = (16 * 1024 * 1024) / desc_len + 2;
    let desc = "x".repeat(desc_len);
    std::thread::scope(|s| {
        for w in 0..4 {
            let (cwd, state, desc, parent) = (&cwd, &state, &desc, &parent);
            s.spawn(move || {
                for _ in (w..count).step_by(4) {
                    let o = delulu_in(
                        cwd,
                        state,
                        &["grants", "delegate", "--effects", "Write", "--parent", parent, "--holder-desc", desc],
                    );
                    assert!(o.status.success(), "grants delegate: {}", stderr(&o));
                }
            });
        }
    });

    let o = delulu_in(&cwd, &state, &["grants", "list", "--json"]);
    assert!(o.status.success(), "grants list over {count} nodes of {desc_len}-byte descriptions: {}{}", stdout(&o), stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("grants list --json");
    let nodes = v["nodes"].as_array().unwrap();
    // The parent, its root (`grants delegate` mints one), and the described nodes.
    assert_eq!((v["count"].as_u64(), nodes.len()), (Some(count as u64 + 2), count + 2), "every node, once");
    let described: usize = nodes.iter().map(|n| n["holder_desc"].as_str().unwrap_or("").len()).sum();
    assert!(described > 16 * 1024 * 1024, "the state: the descriptions alone are more than one frame ({described} bytes)");
    let ids: std::collections::BTreeSet<&str> = nodes.iter().filter_map(|n| n["id"].as_str()).collect();
    assert_eq!(ids.len(), nodes.len(), "no node twice");

    let o = delulu_in(&cwd, &state, &["grants", "tree"]);
    assert!(o.status.success(), "grants tree: {}", stderr(&o));
    let text = stdout(&o);
    assert_eq!(text.lines().count(), count + 2, "one line a node");
    assert_eq!(text.lines().filter(|l| l.starts_with("    g_") && l.ends_with(&desc)).count(), count, "each under the parent");
    drop(guard);
    let _ = std::fs::remove_dir_all(&base);
}
