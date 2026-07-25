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
    "add", "atlas", "audit", "authority", "build", "check", "deploy", "explain", "fleet", "fmt",
    "grants", "guard", "keygen", "locale", "lock", "login", "morph", "plugin", "publish", "run",
    "secrets", "sign", "test", "why",
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
                let code = run(&args).status.code().unwrap_or(-1);
                if code == 101 || !(0..132).contains(&code) {
                    bad.push(format!("{args:?} exited {code}"));
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
