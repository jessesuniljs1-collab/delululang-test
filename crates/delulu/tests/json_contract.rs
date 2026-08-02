//! The `--json` envelope contract, enforced across the whole CLI surface.
//!
//! `docs/for-agents.md` promises: **"Every `--json` command emits one object."** Before campaign
//! finding C2 that was false on essentially every subcommand: a usage or I/O error printed a human
//! sentence to stderr and exited nonzero with **nothing on stdout**. A programmatic caller — a shell
//! script, a CI job, an agent — got an exit code and no parseable answer. `delulu test` had the
//! opposite bug once the fallback landed, emitting *two* objects.
//!
//! So this file tests the exact promise: not "some JSON", not "at least one object", but **exactly
//! one**. It sweeps every subcommand against the argument shapes a caller actually gets wrong, which
//! is why it catches both directions of the defect.
//!
//! Excluded, with reasons rather than silently: `lsp` is a JSON-RPC server that blocks on stdin by
//! design, `repl` is interactive, and `broker` can start a daemon — none of the three is a
//! run-once-and-exit command, so a sweep is the wrong instrument for them.

use std::process::{Command, Output};

/// Every run-once subcommand. Kept explicit so that adding a subcommand and forgetting the contract
/// shows up as a missing entry in review, rather than as silence.
const SUBCOMMANDS: &[&str] = &[
    "add", "atlas", "audit", "authority", "build", "check", "deploy", "explain", "fix", "fleet",
    "fmt", "grants", "guard", "keygen", "locale", "lock", "login", "morph", "plugin", "publish",
    "run", "secrets", "sign", "test", "why",
];

/// Every run-once subcommand the dispatcher accepts must appear in [`SUBCOMMANDS`] AND in `--help`.
///
/// `deploy` and `fleet` were both working top-level commands that `--help` never mentioned, which is
/// how they escaped the first sweep — and `deploy` was double-emitting JSON on its refusal paths, found
/// only because an older test happened to parse its output. An undocumented command is a command
/// nothing sweeps.
const NOT_SWEPT: &[&str] = &[
    "lsp",    // a JSON-RPC server: blocks on stdin by design
    "repl",   // interactive
    "broker", // can start a daemon; not run-once
];

/// Argument shapes that make a command fail. Each is something a real caller produces: no argument
/// at all, a path that does not exist, a duplicated flag, a malformed flag.
const FAILING_SHAPES: &[&[&str]] = &[
    &[],
    &["/nonexistent/delulu-json-contract/x.delulu"],
    &["--json"],
    &["---"],
];

fn run(args: &[&str]) -> Output {
    let home = std::env::temp_dir().join(format!("delulu_json_contract_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&home);
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        // An isolated home so a sweep can never touch the developer's keys, credentials, or state.
        .env("DELULU_HOME", &home)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("the delulu binary must run")
}

/// How many complete JSON values `text` consists of. `0` for empty, `usize::MAX` for unparseable —
/// distinguished so a failure message can say which of the two went wrong.
/// Did this invocation crash? **Exit code alone cannot answer that**, and that is the whole reason a
/// panic on an empty `delulu.toml` survived a breadth sweep, a front-door pass and a dedicated crash
/// hunt (`HARDENING_CAMPAIGN.md` C49).
///
/// `main.rs` runs the entire CLI on a spawned thread with a 512 MiB stack, because a tree-walking
/// interpreter on a 1 MiB main stack died of stack overflow before `MAX_DEPTH` could fire. When that
/// worker panics, `main` joins it and returns **2** — deliberately, and documented: "exit codes are
/// part of the stable contract: 0 ok / 1 diagnostics / 2 internal". A panic *is* an internal error,
/// so 2 is the honest code and must not change.
///
/// But exit 2 is also what an ordinary usage error returns, so a sweep keying on 101 was blind to
/// every crash in the work path — which is where all the work happens. The panic message itself is
/// the only reliable signal, so that is what this checks.
fn panicked(out: &Output) -> bool {
    let blob = String::from_utf8_lossy(&out.stderr);
    blob.contains("panicked at") || blob.contains("RUST_BACKTRACE")
}

fn count_json_values(text: &str) -> usize {
    let t = text.trim();
    if t.is_empty() {
        return 0;
    }
    let mut de = serde_json::Deserializer::from_str(t).into_iter::<serde_json::Value>();
    let mut n = 0usize;
    for item in de.by_ref() {
        match item {
            Ok(_) => n += 1,
            Err(_) => return usize::MAX,
        }
    }
    n
}

#[test]
fn every_failing_json_invocation_emits_exactly_one_object() {
    let mut broken: Vec<String> = Vec::new();
    for sub in SUBCOMMANDS {
        for shape in FAILING_SHAPES {
            let mut args: Vec<&str> = vec![sub];
            args.extend_from_slice(shape);
            if !args.contains(&"--json") {
                args.push("--json");
            }
            let out = run(&args);
            let code = out.status.code().unwrap_or(-1);
            if code == 0 {
                continue; // this shape happened to succeed; the contract concerns failures
            }
            let stdout = String::from_utf8_lossy(&out.stdout);
            match count_json_values(&stdout) {
                1 => {}
                0 => broken.push(format!("{args:?} exited {code} with NO json on stdout")),
                usize::MAX => broken.push(format!("{args:?} exited {code} with unparseable stdout")),
                n => broken.push(format!("{args:?} exited {code} with {n} json values, want 1")),
            }
        }
    }
    assert!(
        broken.is_empty(),
        "the `--json` contract is one object per invocation, including on failure:\n  {}",
        broken.join("\n  ")
    );
}

#[test]
fn a_failure_envelope_carries_the_fields_a_caller_keys_on() {
    // The documented envelope: `command`, `schema`, `delulu_version`, `diagnostics`, `summary`.
    // `summary.errors == 0` is the documented definition of "this passed", so a failure must not
    // report zero — that is the field a caller branches on before reading anything else.
    let out = run(&["check", "/nonexistent/delulu-json-contract/x.delulu", "--json"]);
    assert_ne!(out.status.code(), Some(0));
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("a failure must still be one JSON object");
    assert_eq!(v["command"], "check");
    assert_eq!(v["schema"], 1);
    assert!(v["delulu_version"].is_string(), "{v}");
    assert!(v["diagnostics"].is_array(), "{v}");
    assert_ne!(
        v["summary"]["errors"], 0,
        "a failed command must not report summary.errors == 0: {v}"
    );
}

#[test]
fn no_invocation_panics_or_leaves_the_process_signalled() {
    // Exit 101 is a Rust panic; anything at or above 132 is a signal death on Unix. Either means the
    // CLI stopped being a program with a contract and became a crash — which for a caller is
    // indistinguishable from the toolchain being broken. Swept over both the JSON and human paths,
    // because a panic is not a formatting concern.
    let mut bad: Vec<String> = Vec::new();
    for sub in SUBCOMMANDS {
        for shape in FAILING_SHAPES {
            for json in [false, true] {
                let mut args: Vec<&str> = vec![sub];
                args.extend_from_slice(shape);
                if json {
                    args.push("--json");
                }
                let out = run(&args);
                let code = out.status.code().unwrap_or(-1);
                if code == 101 || !(0..132).contains(&code) || panicked(&out) {
                    bad.push(format!("{args:?} exited {code}{}", if panicked(&out) { " WITH A PANIC" } else { "" }));
                }
            }
        }
    }
    assert!(bad.is_empty(), "the CLI must fail with a diagnostic, never a crash:\n  {}", bad.join("\n  "));
}

#[test]
fn every_dispatched_subcommand_is_documented_and_swept() {
    // The help text is the only list a human or an agent can discover commands from, so a command
    // missing from it is invisible — including to this file's own sweep.
    let help = String::from_utf8_lossy(&run(&["--help"]).stdout).to_string();
    let mut missing_from_help: Vec<&str> = Vec::new();
    for sub in SUBCOMMANDS.iter().chain(NOT_SWEPT.iter()) {
        if !help.contains(&format!("delulu {sub}")) {
            missing_from_help.push(sub);
        }
    }
    assert!(
        missing_from_help.is_empty(),
        "these subcommands exist but `--help` does not list them: {missing_from_help:?}"
    );
}

// ----- the same contract over manifest CONTENT, not just argument shapes (C49) -------------------

/// A `delulu.toml` is untrusted input in exactly the way a `.delulu` file is: it arrives from a repo
/// someone else wrote, or from a beginner who just ran `touch delulu.toml`. The sweep above covers
/// the shapes a caller gets wrong on the COMMAND LINE and never looked inside a package, so five
/// manifest shapes crashed `delulu build` with a Rust panic — among them an empty file, a file that
/// is not TOML, and one missing `[package]`.
///
/// The diagnostic was never the problem: DL1004 was computed correctly every time. The crash was in
/// a *courtesy note* — C26/D33 added "no `.delulu` modules found under `<dir>`" to stop an empty
/// package reporting success, and composing that message reached for the root package's directory in
/// the one situation where resolution never recorded a root package. **A fix from an earlier phase
/// of this campaign introduced the crash it is now guarded against**, which is the lesson worth
/// keeping: a repair needs its own skip-branch analysis, and "what if there is nothing to name?" is
/// one of them.
///
/// Note which verbs did and did not crash. `lock` and `authority` handled all five shapes with a
/// diagnostic and exit 1; only `build`/`check` panicked. A rule that holds on two paths out of three
/// holds nowhere (C23/D30).
#[test]
fn no_manifest_shape_makes_a_package_command_panic() {
    let dir = std::env::temp_dir().join(format!("delulu-manifest-sweep-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    // Every one of these is a manifest a real person or tool produces.
    let manifests: &[(&str, &str)] = &[
        ("empty", ""),
        ("not-toml", "this is not toml ][{\n"),
        ("no-package-section", "[authority]\neffects = [\"Write\"]\n"),
        ("no-name-key", "[package]\nversion = \"0.1.0\"\n\n[authority]\neffects = []\n"),
        ("no-version-key", "[package]\nname = \"app\"\n\n[authority]\neffects = []\n"),
        // Valid TOML that the manifest reader does not understand: a dependency's authority given as
        // a sub-table instead of the documented inline table.
        (
            "dep-authority-as-subsection",
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[authority]\neffects = []\n\n\
             [dependencies]\ndep = { path = \"../dep\" }\n\n[dependencies.dep.authority]\neffects = []\n",
        ),
        ("only-a-comment", "# nothing else\n"),
        ("bom-only", "\u{feff}"),
    ];

    let mut bad: Vec<String> = Vec::new();
    for (tag, manifest) in manifests {
        // The app DEPENDS on a sibling package, and that detail is load-bearing: with no dependency
        // the resolver still loads the entry module, `modules` is non-empty, and the note that used
        // to panic is never reached. A first version of this test used a standalone package and
        // passed against the unfixed code — witnessing nothing (the same trap C19 set in P5).
        let case = dir.join(tag);
        let pkg = case.join("app");
        let dep = case.join("dep");
        std::fs::create_dir_all(pkg.join("src")).unwrap();
        std::fs::create_dir_all(dep.join("src")).unwrap();
        std::fs::write(
            dep.join("delulu.toml"),
            "[package]\nname = \"dep\"\nversion = \"0.1.0\"\n\n[authority]\neffects = []\n",
        )
        .unwrap();
        std::fs::write(
            dep.join("src").join("main.delulu"),
            "module dep\n\npub fn v(x: Int) -> Int {\n    x + 1\n}\n",
        )
        .unwrap();
        std::fs::write(pkg.join("delulu.toml"), manifest).unwrap();
        std::fs::write(
            pkg.join("src").join("main.delulu"),
            "module app\n\nimport dep\n\nfn main(root: Root) ! {Write} {\n    let c = root.console()\n    c.println(str(v(1)))\n}\n",
        )
        .unwrap();

        for verb in ["build", "check", "lock", "authority", "atlas"] {
            for json in [false, true] {
                let mut args: Vec<String> = vec![verb.to_string(), pkg.display().to_string()];
                if json {
                    args.push("--json".to_string());
                }
                let refs: Vec<&str> = args.iter().map(String::as_str).collect();
                let out = run(&refs);
                let code = out.status.code().unwrap_or(-1);
                if code == 101 || !(0..132).contains(&code) || panicked(&out) {
                    let how = if panicked(&out) { "PANICKED" } else { "died" };
                    bad.push(format!("`{verb}` {how} on the `{tag}` manifest (exit {code})"));
                }
                // A crash is not the only way to fail a user here: succeeding on an unreadable
                // manifest would be worse. None of these may report success.
                if code == 0 {
                    bad.push(format!("`{verb}` REPORTED SUCCESS on the `{tag}` manifest"));
                }
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        bad.is_empty(),
        "an unreadable manifest must produce a diagnostic — never a panic, never success:\n  {}",
        bad.join("\n  ")
    );
}
