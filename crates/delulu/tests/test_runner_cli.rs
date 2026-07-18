//! Stage 8, phase 8g: `delulu test` — the authority-isolated runner (criterion 5),
//! driven through the real binary. Pure tests hold nothing; an undeclared effect is
//! refused; every test's actual effect list is in its JSON report even on pass; and the
//! broker lane mints a `test-session` node that is transitively revoked at session end.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_testrun_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn delulu(dir: &Path, args: &[&str], state: Option<&PathBuf>) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_delulu"));
    c.current_dir(dir).args(args).env("DELULU_NO_FIRST_RUN", "1");
    if let Some(s) = state {
        c.env("DELULU_STATE_DIR", s);
    } else {
        // A private, broker-less state dir so the runner takes the embedded lane
        // deterministically (never a stray daemon from the dev box).
        c.env("DELULU_STATE_DIR", dir.join("no-broker"));
    }
    c.output().expect("run delulu")
}

/// A package whose test ceiling allows Write + a fixtures read scope.
fn scaffold(dir: &Path) {
    std::fs::write(
        dir.join("delulu.toml"),
        "[package]\nname = \"suite\"\nversion = \"0.1.0\"\n\n\
         [test-authority]\neffects = [\"Write\", \"Read\"]\nfs.read = [\"./tests/fixtures\"]\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("tests").join("fixtures")).unwrap();
    // Pure test + a within-ceiling Write test.
    std::fs::write(
        dir.join("tests").join("pass.delulu"),
        "module pass\nfn double(n: Int) -> Int { n * 2 }\n\
         test \"pure doubles\" {\n  assert_eq(double(21), 42)\n}\n\
         test \"writes within the ceiling\" ! {Write} {\n  let out = test_root.console()\n  out.println(\"hi from a test\")\n}\n",
    )
    .unwrap();
}

#[test]
fn criterion5_pure_tests_run_effects_traced_and_undeclared_is_refused() {
    let dir = tmp("crit5");
    scaffold(&dir);

    // A test that performs Net without declaring it → DL0501 at check (its file fails).
    std::fs::write(
        dir.join("tests").join("net.delulu"),
        "module net\ntest \"sneaky net\" {\n  let h = test_root.http([\"x\"])\n  let r = h.get(\"http://x\")\n}\n",
    )
    .unwrap();

    let o = delulu(&dir, &["test", "--json"], None);
    let v: Value = serde_json::from_slice(&o.stdout).expect("test --json is valid JSON");

    let by_name = |name: &str| -> Value {
        v["tests"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["name"] == name)
            .cloned()
            .unwrap_or(Value::Null)
    };

    // The pure test passed carrying ZERO traced effects (invariant 41: it held nothing).
    let pure = by_name("pure doubles");
    assert_eq!(pure["status"], "pass", "{pure}");
    assert_eq!(pure["effects_traced"].as_array().unwrap().len(), 0, "a pure test traces nothing");

    // The Write test passed and its actual effect list is IN the report (review surface).
    let wrote = by_name("writes within the ceiling");
    assert_eq!(wrote["status"], "pass", "{wrote}");
    let traced: Vec<&str> =
        wrote["effects_traced"].as_array().unwrap().iter().map(|e| e.as_str().unwrap()).collect();
    assert!(traced.contains(&"Write"), "effects_traced carries Write even on pass: {wrote}");

    // The undeclared-Net file was refused (DL0501 at check) — the runner reports it failed
    // and never ran it.
    let net = by_name("sneaky net");
    assert!(
        net.is_null() || net["status"] == "check-failed",
        "the undeclared-Net test must be refused, not run"
    );
    assert!(
        v["tests"].as_array().unwrap().iter().any(|t| t["status"] == "check-failed"),
        "the net.delulu file failed to check: {}",
        String::from_utf8_lossy(&o.stdout)
    );

    // Overall: at least one failure ⇒ exit 1.
    assert_eq!(o.status.code(), Some(1));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("DL0501"), "the real refusal surfaces on stderr: {err}");
}

#[test]
fn a_test_exceeding_the_package_ceiling_is_dl1703() {
    let dir = tmp("ceiling");
    // Ceiling allows only Write; a test declaring Read exceeds it.
    std::fs::write(
        dir.join("delulu.toml"),
        "[package]\nname = \"s\"\nversion = \"0.1.0\"\n\n[test-authority]\neffects = [\"Write\"]\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("tests").join("over.delulu"),
        "module over\ntest \"reads too much\" ! {Read} {\n  let fs = test_root.fs_read(\"./x\")\n  let r = fs.read_text(\"a\")\n}\n",
    )
    .unwrap();
    let o = delulu(&dir, &["test", "--json"], None);
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("DL1703"), "exceeding the ceiling is DL1703: {err}");
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn determinism_repeated_runs_are_byte_identical_modulo_timing() {
    let dir = tmp("determ");
    scaffold(&dir);
    let strip_ms = |bytes: &[u8]| -> Value {
        let mut v: Value = serde_json::from_slice(bytes).unwrap();
        // Timing is the only non-deterministic field — blank it before comparison.
        if let Some(tests) = v["tests"].as_array_mut() {
            for t in tests {
                t["ms"] = Value::Null;
            }
        }
        v["summary"]["ms"] = Value::Null;
        v
    };
    let a = delulu(&dir, &["test", "--json"], None);
    let b = delulu(&dir, &["test", "--json"], None);
    assert_eq!(strip_ms(&a.stdout), strip_ms(&b.stdout), "deterministic by default");
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

/// Criterion 5's broker half: with a live daemon the run lives under a `test-session`
/// node that is transitively revoked at session end — nothing a test leaked survives.
#[test]
fn criterion5_broker_session_is_revoked_at_end() {
    let dir = tmp("session");
    scaffold(&dir);
    let state = dir.join("state");
    std::fs::create_dir_all(&state).unwrap();

    let start = delulu(&dir, &["broker", "start"], Some(&state));
    if !start.status.success() {
        // No broker on this platform/lane — the embedded lane is covered above; skip.
        eprintln!("broker start unavailable, skipping the daemon lane: {}", String::from_utf8_lossy(&start.stderr));
        return;
    }
    let _guard = DaemonGuard { state: state.clone() };

    let o = delulu(&dir, &["test", "--json"], Some(&state));
    let v: Value = serde_json::from_slice(&o.stdout).expect("json");
    assert_eq!(v["custody"]["mode"], "daemon", "the daemon lane engaged: {}", v["custody"]);
    let session = v["custody"]["session"].as_str().expect("a session node was minted");
    assert_eq!(v["custody"]["revoked_at_end"], true, "the session was revoked at end");

    // And the tree shows it REVOKED afterward — session-end transitive revocation,
    // observed on the node itself (revoked nodes are marked, not erased — the audit
    // trail is the point). Its per-file child is revoked in the same sweep.
    let tree = delulu(&dir, &["grants", "tree"], Some(&state));
    let tree_out = String::from_utf8_lossy(&tree.stdout);
    let session_line = tree_out
        .lines()
        .find(|l| l.contains(session))
        .unwrap_or_else(|| panic!("the session node is in the tree: {tree_out}"));
    assert!(
        session_line.contains("revoked"),
        "the test-session node must be revoked at run end: {session_line}"
    );
    assert!(
        tree_out.lines().any(|l| l.contains("test-file") && l.contains("revoked")),
        "the per-file child was transitively revoked too: {tree_out}"
    );
}
