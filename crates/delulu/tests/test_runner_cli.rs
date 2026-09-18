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
    // P1-06 (NE-08): the refusal used to surface as a HUMAN render on stderr while the envelope
    // said only `status: "check-failed"` — the machine channel named a failure and withheld its
    // cause. The diagnostics now ride in the envelope, with code, span and repairs, and under
    // `--json` nothing human-rendered goes to stderr at all.
    let codes: Vec<&str> =
        v["diagnostics"].as_array().expect("the test envelope carries diagnostics")
            .iter().map(|d| d["code"].as_str().unwrap_or("")).collect();
    assert!(codes.contains(&"DL0501"), "the real refusal is in the envelope: {}", v["diagnostics"]);
    let first = &v["diagnostics"][0];
    assert!(first["spans"][0]["start"]["byte"].is_number(), "with a byte span: {first}");
    assert!(first["repairs"].is_array(), "and its repairs: {first}");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        !err.contains("DL0501"),
        "under --json the human render must not also go to stderr: {err}"
    );
    assert_eq!(v["summary"]["errors"], v["summary"]["failed"], "errors tracks the run's verdict");
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
    // The human channel still renders it; the machine channel now carries it as a diagnostic with
    // a code and a span rather than as a message string with the code spliced into prose (NE-08).
    let human = delulu(&dir, &["test"], None);
    assert!(
        String::from_utf8_lossy(&human.stderr).contains("DL1703"),
        "exceeding the ceiling is DL1703 on the human channel"
    );
    let v: Value = serde_json::from_slice(&o.stdout).expect("one envelope");
    let codes: Vec<&str> = v["diagnostics"].as_array().expect("diagnostics array")
        .iter().map(|d| d["code"].as_str().unwrap_or("")).collect();
    assert!(codes.contains(&"DL1703"), "exceeding the ceiling is DL1703: {}", v["diagnostics"]);
    assert!(
        v["diagnostics"][0]["spans"][0]["start"]["line"].is_number(),
        "and it names where: {}",
        v["diagnostics"][0]
    );
    let failed = v["tests"].as_array().unwrap().iter().find(|t| t["status"] == "fail").expect("a failed test");
    assert_eq!(failed["failure"]["code"], "DL1703", "the failure names the code as a field: {failed}");
    assert!(
        !String::from_utf8_lossy(&o.stderr).contains("DL1703"),
        "under --json nothing human-rendered goes to stderr"
    );
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

/// P1-F4: `delulu test <package directory>` resolves the PACKAGE — every module, as `run` and bare
/// `delulu test` inside it do — instead of walking it as loose files, where a test calling a name
/// another module exports was `check-failed`. Reached from outside the package, by path.
#[test]
fn a_package_reached_by_path_is_tested_with_all_its_modules() {
    let dir = tmp("pkgpath");
    let pkg = dir.join("pk2");
    std::fs::create_dir_all(pkg.join("src")).unwrap();
    std::fs::write(
        pkg.join("delulu.toml"),
        "[package]\nname = \"pk2\"\nversion = \"0.1.0\"\n\n[authority]\neffects = []\n",
    )
    .unwrap();
    std::fs::write(pkg.join("src").join("util.delulu"), "module pk2.util\n\npub fn double(x: Int) -> Int {\n    x * 2\n}\n")
        .unwrap();
    std::fs::write(
        pkg.join("src").join("main.delulu"),
        "module pk2\n\nimport pk2.util\n\nfn quad(x: Int) -> Int {\n    double(double(x))\n}\n\n\
         test \"quad uses the other module\" {\n    assert_eq(str(quad(3)), \"12\")\n}\n",
    )
    .unwrap();

    let o = delulu(&dir, &["test", "pk2", "--json"], None);
    let out = String::from_utf8_lossy(&o.stdout).to_string();
    assert_eq!(o.status.code(), Some(0), "the package's test must pass:\n{out}\n{}", String::from_utf8_lossy(&o.stderr));
    let v: Value = serde_json::from_str(&out).expect("one envelope");
    assert_eq!(v["summary"]["passed"], 1, "{out}");
    let t = &v["tests"][0];
    assert_eq!(t["status"], "pass", "{out}");
    // Reported where it is WRITTEN, not where the flattened copy put it.
    assert!(t["file"].as_str().unwrap_or("").ends_with("main.delulu"), "{out}");
    assert_eq!(t["span"]["line"], 9, "{out}");

    // The shipped two-module example, by path from the repository root.
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let o = delulu(&repo, &["test", "examples/greeter"], Some(&dir.join("no-broker")));
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    let _ = std::fs::remove_dir_all(&dir);
}

/// D-NE-17 (the owner, 2026-09-18): `delulu test --test-authority <row>`, in the manifest's own
/// `[test-authority]` syntax. For a file outside any package it is the only ceiling; inside a
/// package it may only narrow the package's ceiling, and a wider row is DL1703 before any test
/// runs. Either way a test holds exactly its declared row (invariant 41) — the flag is a ceiling.
#[test]
fn test_authority_flag_is_a_ceiling_that_can_only_narrow_a_package() {
    let dir = tmp("testauth");
    let loose = dir.join("loose");
    std::fs::create_dir_all(&loose).unwrap();
    std::fs::write(
        loose.join("w.delulu"),
        "module w\n\ntest \"writes\" ! {Write} {\n    let out = test_root.console()\n    out.println(\"hi\")\n}\n\n\
         test \"pure\" {\n    assert_eq(str(1 + 1), \"2\")\n}\n",
    )
    .unwrap();
    let run = |cwd: &Path, args: &[&str]| -> (Option<i32>, Value, String) {
        let o = delulu(cwd, args, None);
        let out = String::from_utf8_lossy(&o.stdout).to_string();
        (o.status.code(), serde_json::from_str(&out).unwrap_or(Value::Null), out)
    };

    // Outside a package: no flag, no ceiling — the effectful test is DL1703, as before.
    let (code, _, out) = run(&loose, &["test", "w.delulu", "--json"]);
    assert_eq!(code, Some(1), "{out}");
    // The flag is the only ceiling: the test runs, holding exactly its declared row.
    let (code, v, out) = run(&loose, &["test", "w.delulu", "--test-authority", "effects = [\"Write\", \"Clock\"]", "--json"]);
    assert_eq!(code, Some(0), "{out}");
    let writes = v["tests"].as_array().unwrap().iter().find(|t| t["name"] == "writes").cloned().unwrap();
    assert_eq!(writes["effects_traced"], serde_json::json!(["Write"]), "only the declared row: {out}");
    let pure = v["tests"].as_array().unwrap().iter().find(|t| t["name"] == "pure").cloned().unwrap();
    assert_eq!(pure["effects_traced"], serde_json::json!([]), "a pure test holds nothing: {out}");
    // A row the flag does not cover is refused, and the refusal names the flag's ceiling.
    let (code, v, out) = run(&loose, &["test", "w.delulu", "--test-authority", "effects = [\"Clock\"]", "--json"]);
    assert_eq!(code, Some(1), "{out}");
    assert!(v["diagnostics"].to_string().contains("--test-authority ceiling"), "{out}");
    // Not the manifest syntax: refused as usage, never ignored.
    let (code, _, out) = run(&loose, &["test", "w.delulu", "--test-authority", "Write"]);
    assert_eq!(code, Some(2), "{out}");

    // Inside a package whose ceiling is {Write} with one fs.read scope.
    let pkg = dir.join("pkg");
    std::fs::create_dir_all(pkg.join("src")).unwrap();
    std::fs::create_dir_all(pkg.join("fixtures").join("sub")).unwrap();
    std::fs::create_dir_all(dir.join("other")).unwrap();
    std::fs::write(
        pkg.join("delulu.toml"),
        "[package]\nname = \"pkg\"\nversion = \"0.1.0\"\n\n[authority]\neffects = []\n\n\
         [test-authority]\neffects = [\"Write\"]\nfs.read = [\"./fixtures\"]\n",
    )
    .unwrap();
    std::fs::write(
        pkg.join("src").join("main.delulu"),
        "module pkg\n\ntest \"writes\" ! {Write} {\n    let out = test_root.console()\n    out.println(\"ran\")\n}\n",
    )
    .unwrap();
    // Equal, and narrower-in-scope: accepted.
    let (code, _, out) = run(&pkg, &["test", "--test-authority", "effects = [\"Write\"]", "--json"]);
    assert_eq!(code, Some(0), "{out}");
    let (code, _, out) =
        run(&pkg, &["test", "--test-authority", "effects = [\"Write\"]", "--test-authority", "fs.read = [\"./fixtures/sub\"]", "--json"]);
    assert_eq!(code, Some(0), "{out}");
    // Wider — by an effect, or by a scope outside the package's (spelled with `..` too) — is DL1703
    // BEFORE any test runs: no `tests` array at all.
    for row in [
        vec!["--test-authority", "effects = [\"Write\", \"Net\"]"],
        vec!["--test-authority", "fs.read = [\"../other\"]"],
        vec!["--test-authority", "fs.read = [\"./fixtures/../../other\"]"],
    ] {
        let mut args = vec!["test"];
        args.extend(row.iter().copied());
        args.push("--json");
        let (code, v, out) = run(&pkg, &args);
        assert_eq!(code, Some(1), "{row:?}: {out}");
        assert_eq!(v["diagnostics"][0]["code"], "DL1703", "{row:?}: {out}");
        assert!(v.get("tests").is_none(), "{row:?}: refused before any test ran: {out}");
    }
    // And the same from outside, reaching the package by path.
    let (code, v, out) = run(&dir, &["test", "pkg", "--test-authority", "effects = [\"Net\"]", "--json"]);
    assert_eq!(code, Some(1), "{out}");
    assert_eq!(v["diagnostics"][0]["code"], "DL1703", "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// Head-chef verification of P1-F: a test file is governed by its NEAREST ENCLOSING package's
/// ceiling however it is reached. A file reached by its own path from outside used to escape it —
/// `delulu test pk/tests/t.delulu --test-authority 'effects = ["Write"]'` ran a Write test in a
/// package whose ceiling is pure. Four routes, one answer, with and without the flag.
#[test]
fn every_route_to_a_test_file_meets_its_packages_ceiling() {
    let dir = tmp("routes");
    let pk = dir.join("pk");
    std::fs::create_dir_all(pk.join("tests")).unwrap();
    std::fs::create_dir_all(pk.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("other")).unwrap();
    std::fs::write(
        pk.join("delulu.toml"),
        "[package]\nname = \"pk\"\nversion = \"0.1.0\"\n\n[authority]\neffects = []\n\n[test-authority]\neffects = []\n",
    )
    .unwrap();
    std::fs::write(pk.join("src").join("main.delulu"), "module pk\n\nfn one() -> Int {\n    1\n}\n").unwrap();
    std::fs::write(
        pk.join("tests").join("t.delulu"),
        "module t\n\ntest \"writes\" ! {Write} {\n    let out = test_root.console()\n    out.println(\"hi\")\n}\n",
    )
    .unwrap();
    let routes: [(&Path, &str); 4] = [
        (&dir, "pk"),
        (&pk, "tests/t.delulu"),
        (&dir, "pk/tests/t.delulu"),
        (&dir, "other/../pk/tests/t.delulu"),
    ];
    for flag in [true, false] {
        for (cwd, target) in routes {
            let mut args = vec!["test", target];
            if flag {
                args.extend(["--test-authority", "effects = [\"Write\"]"]);
            }
            let o = delulu(cwd, &args, None);
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            assert_eq!(o.status.code(), Some(1), "flag={flag} `{}` must be refused:\n{err}", args.join(" "));
            assert!(err.contains("DL1703"), "flag={flag} `{}`:\n{err}", args.join(" "));
            assert!(!String::from_utf8_lossy(&o.stdout).contains("hi"), "the Write test must not run");
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}
