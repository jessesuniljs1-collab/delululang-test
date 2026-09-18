//! Phase A3 — the Atlas end-to-end orchestration test, through the real binary (guard_e2e.rs
//! structure: DaemonGuard, temp state dir, exact assertions).
//!
//! The story: build a program's atlas in every format (byte-identical across runs) → start a real
//! broker and delegate a grant → `atlas --custody` overlays the live grant tree (grant nodes,
//! `delegates` edges, the verbatim custody caveat, `custody` in `atlas/1`) → stop the broker →
//! `atlas --custody` degrades to a DL1781 note and STILL emits the atlas without the overlay
//! (criterion 6's second half) → the HTML artifact is self-contained (criterion 7's e2e witness).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

const DEMO: &str = "examples/demo.delulu";

fn delulu_state(state: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(workspace_root())
        .env("DELULU_STATE_DIR", state)
        .env_remove("NO_COLOR")
        .env_remove("DELULU_COLOR")
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

#[test]
fn atlas_custody_overlay_live_then_degraded_end_to_end() {
    let tmp = std::env::temp_dir().join(format!("delulu_atlas_e2e_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let state = tmp.join("state");
    // The daemon spawns with the state dir as its cwd — it must exist first (guard_e2e pattern).
    std::fs::create_dir_all(&state).unwrap();

    // ----- 1. every format is byte-identical across runs (criterion 1, e2e form) ---------------
    for fmt in ["tree", "digest", "json", "dot", "mermaid", "html"] {
        let a = stdout(&delulu_state(&state, &["atlas", DEMO, "--format", fmt]));
        let b = stdout(&delulu_state(&state, &["atlas", DEMO, "--format", fmt]));
        assert_eq!(a, b, "format {fmt} must be byte-identical across runs");
        assert!(!a.is_empty(), "format {fmt} produced output");
    }

    // ----- 2. live broker: delegate a grant, then overlay it ------------------------------------
    let start = delulu_state(&state, &["broker", "start"]);
    assert!(start.status.success(), "broker start: {}", stderr(&start));
    let _guard = DaemonGuard { state: state.clone() };
    let banner = format!("{}{}", stdout(&start), stderr(&start));
    let owner = extract_owner_code(&banner).expect("owner code in start output");

    let del = delulu_state(
        &state,
        &["grants", "delegate", "--effects", "Read", "--fs-read", "./data", "--json"],
    );
    assert!(del.status.success(), "delegate: {}", stderr(&del));
    let delegated: Value = serde_json::from_str(&stdout(&del)).expect("delegate --json");
    let node_id = delegated["node"].as_str().expect("node id").to_string();
    let _ = owner; // the overlay is read-only — no owner code involved (ruling 7)

    let o = delulu_state(&state, &["atlas", DEMO, "--custody", "--json"]);
    assert!(o.status.success(), "atlas --custody: {}", stderr(&o));
    let env: Value = serde_json::from_str(&stdout(&o)).expect("atlas --custody --json is valid JSON");
    let v: Value = env["atlas"].clone();
    assert_eq!(v["atlas"], "atlas/1");
    // The overlay is present and carries the delegated grant.
    assert!(!v["custody"].is_null(), "custody overlay attached");
    let grants = v["custody"]["grants"].as_array().expect("grants array");
    assert!(
        grants.iter().any(|g| g["id"] == node_id.as_str()),
        "the delegated node {node_id} appears in the overlay: {grants:?}"
    );
    // Grant nodes + delegates edge in the graph itself.
    let node_ids: Vec<&str> = v["nodes"].as_array().unwrap().iter().map(|n| n["id"].as_str().unwrap()).collect();
    assert!(node_ids.contains(&format!("grant:{node_id}").as_str()), "grant node in the graph");
    assert!(
        v["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["kind"] == "delegates" && e["to"] == format!("grant:{node_id}")),
        "a delegates edge reaches the delegated grant"
    );
    // The verbatim custody caveat ships (addendum §2.6).
    let caveats: Vec<&str> = v["caveats"].as_array().unwrap().iter().map(|c| c.as_str().unwrap()).collect();
    assert!(
        caveats.iter().any(|c| c.contains("awareness, not enforcement")),
        "the custody caveat ships verbatim: {caveats:?}"
    );

    // ----- 3. broker down: DL1781 note + the atlas is STILL emitted (criterion 6) ---------------
    let stop = delulu_state(&state, &["broker", "stop"]);
    assert!(stop.status.success(), "broker stop: {}", stderr(&stop));
    let o = delulu_state(&state, &["atlas", DEMO, "--custody", "--json"]);
    assert_eq!(o.status.code(), Some(0), "broker down never blocks the atlas");
    let e = stderr(&o);
    assert!(e.contains("DL1781"), "the degradation is a DL1781 note: {e}");
    assert!(e.contains("atlas emitted without it"), "the note says the atlas still ships: {e}");
    let env: Value =
        serde_json::from_str(&stdout(&o)).expect("stdout is still the clean machine channel");
    let v: Value = env["atlas"].clone();
    assert_eq!(v["atlas"], "atlas/1", "the atlas is emitted without the overlay");
    assert!(v["custody"].is_null(), "no overlay when the broker is down");

    // ----- 4. the HTML artifact is fully self-contained (criterion 7, e2e witness) --------------
    let out_dir = tmp.join("bundle");
    let o = delulu_state(
        &state,
        &["atlas", DEMO, "--format", "html", "--out", out_dir.to_str().unwrap()],
    );
    assert!(o.status.success(), "atlas --out: {}", stderr(&o));
    let html = std::fs::read_to_string(out_dir.join("atlas.html")).expect("atlas.html written");
    assert!(html.contains("const ATLAS="), "data embedded inline");
    assert!(!html.contains("http://") && !html.contains("https://"), "zero external URLs");
    assert!(std::fs::metadata(out_dir.join("ATLAS.md")).is_ok(), "ATLAS.md written");
    assert!(std::fs::metadata(out_dir.join("atlas.json")).is_ok(), "atlas.json written");

    let _ = std::fs::remove_dir_all(&tmp);
}
