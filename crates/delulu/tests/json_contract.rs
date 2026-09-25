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
    "add", "atlas", "audit", "authority", "build", "check", "completions", "deploy", "edit", "examples", "explain",
    "fix", "fleet", "fmt", "grants", "guard", "keygen", "locale", "lock", "login", "morph", "new",
    "plugin", "publish", "run", "sandbox", "schema", "secrets", "sign", "skill", "test", "toolchain", "verify-sig",
    "why",
];

/// Every run-once subcommand the dispatcher accepts must appear in [`SUBCOMMANDS`] AND in `--help`.
///
/// `deploy` and `fleet` were both working top-level commands that `--help` never mentioned, which is
/// how they escaped the first sweep — and `deploy` was double-emitting JSON on its refusal paths, found
/// only because an older test happened to parse its output. An undocumented command is a command
/// nothing sweeps.
const NOT_SWEPT: &[&str] = &[
    "lsp",    // a JSON-RPC server: blocks on stdin by design
    "mcp",    // the MCP server (P4-03): JSON-RPC on stdin, like `lsp`; `tests/mcp_cli.rs` drives it
    "repl",   // interactive
    "broker", // can start a daemon; not run-once
    // `doctor` WRITES: without `--check` it regenerates `docs/survey/`. A blind argument sweep would
    // rewrite the repository from inside the test suite, which is the one thing a test must not do.
    // It is covered instead by `doctor_reporting_a_problem_still_emits_exactly_one_object`, which
    // uses the read-only `--check` form.
    "doctor",
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

    // The other direction, which this gate did not check for its whole life. Above proves every
    // NAMED command is documented; it cannot notice a command the dispatcher accepts that nobody
    // ever wrote down. `doctor` was exactly that — dispatched, in `--help`, in neither list, and
    // therefore swept by nothing. Its `--json` path had been emitting TWO objects on any run that
    // reported a problem, and no gate could see it.
    //
    // This is the campaign's recurring shape (C31/C34/C35/C44/C52): a hand-maintained list falls
    // behind the thing that defines it. The definition here is the dispatch `match`, so read it.
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("cli.rs"),
    )
    .expect("cli.rs must be readable — it is the definition this gate reads");

    let body = src
        .split_once("fn run_inner")
        .and_then(|(_, rest)| rest.split_once("match cmd.as_str() {"))
        .map(|(_, rest)| rest)
        .expect("the top-level dispatch must be `match cmd.as_str()` inside `fn run_inner`");

    let mut dispatched: Vec<String> = Vec::new();
    for line in body.lines() {
        let t = line.trim();
        // The catch-all ends the dispatch; everything after belongs to other matches (subverbs).
        if t.starts_with("other =>") {
            break;
        }
        let Some((pats, _)) = t.split_once("=>") else { continue };
        if !pats.trim_start().starts_with('"') {
            continue;
        }
        for pat in pats.split('|') {
            let name = pat.trim().trim_matches('"');
            // `--help`/`-h`/`--version`/`-V`/`help` are flags on the dispatch, not subcommands.
            if name.is_empty() || name.starts_with('-') || name == "help" {
                continue;
            }
            dispatched.push(name.to_string());
        }
    }

    assert!(
        dispatched.len() > 20,
        "the dispatch scan found only {} arms — the parse broke, and a gate that reads nothing \
         passes everything",
        dispatched.len()
    );

    let unswept: Vec<&String> = dispatched
        .iter()
        .filter(|d| !SUBCOMMANDS.contains(&d.as_str()) && !NOT_SWEPT.contains(&d.as_str()))
        .collect();
    assert!(
        unswept.is_empty(),
        "the dispatcher accepts these, but neither SUBCOMMANDS nor NOT_SWEPT names them, so no \
         sweep in this file can reach them: {unswept:?}\n\
         Add each to SUBCOMMANDS, or to NOT_SWEPT with the reason it cannot be swept."
    );
}

/// A command that **ran correctly and reported a problem** is a different signal from a command that
/// refused its arguments — and the sweep above can only produce the second. Every `FAILING_SHAPES`
/// entry is bad input, which a command rejects during argument parsing, before it prints anything of
/// its own. So the shape that broke here was structurally unreachable: `doctor` parses fine, does its
/// work, prints its own envelope, and exits 1 because a check failed.
///
/// `doctor` printed that envelope with a bare `println!` and never called `note_json_emitted()`, so
/// the nonzero exit made `cli::run` add its fallback envelope on top — **two objects for any caller
/// that asked a machine question and got a problem back**. Which is the case an agent hits most.
///
/// The fixture uses no repository state: with no home directory there is nowhere to keep state, and
/// doctor reports that as a problem. The `assert_ne!` on the exit code is deliberate — if the fixture
/// ever stops producing a problem this test would pass while proving nothing.
#[test]
fn doctor_reporting_a_problem_still_emits_exactly_one_object() {
    let out = Command::new(env!("CARGO_BIN_EXE_delulu"))
        // `--check` is the READ-ONLY form: plain `doctor` would regenerate `docs/survey/`.
        .args(["doctor", "--check", "--json"])
        .env_remove("DELULU_HOME")
        .env_remove("HOME")
        .env_remove("USERPROFILE")
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("the delulu binary must run");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let code = out.status.code().unwrap_or(-1);

    assert!(!panicked(&out), "doctor panicked:\n{}", String::from_utf8_lossy(&out.stderr));
    assert_ne!(
        code, 0,
        "the fixture must actually produce a problem, or this test proves nothing:\n{stdout}"
    );
    assert_eq!(
        count_json_values(&stdout),
        1,
        "doctor exited {code} and put more than one JSON object on stdout:\n{stdout}"
    );
}

// ----- the SUCCESS envelope (NE-05) --------------------------------------------------------------
//
// Everything above tests the FAILURE envelope. `docs/for-agents.md` [agents.json-envelope] promises
// the same five fields on success — *"Every `--json` command emits one object: {command, schema,
// delulu_version, diagnostics, summary}"* — and that half was never gated. Verification finding
// NE-05 drove eight commands by hand and found `why` printing a bare `{effect, path, performs}`,
// `plugin inspect` printing `command` alone, `test` carrying no `diagnostics`, `atlas` printing its
// own `atlas/1` document, and `--version --json` printing text. A caller could not read
// `summary.errors` — the documented definition of "this passed" — from any of them.
//
// The sweep below is written the way the failure sweep is: one table of real invocations, plus a
// completeness check against `SUBCOMMANDS` so that a command added later cannot be born unswept.

/// A success invocation: the argv (after the subcommand), and where to run it. `Cwd::Pkg` runs
/// inside the fixture package, `Cwd::Repo` inside the repository (for paths under `examples/`).
#[derive(Clone, Copy, PartialEq)]
enum Cwd {
    Pkg,
    Repo,
}

/// Commands this sweep cannot drive to a `--json` success, each with the reason it cannot.
/// A name belongs here only when the command cannot *reach* a success in a fixture — never
/// because its envelope is inconvenient to fix.
const NO_SUCCESS_SWEEP: &[(&str, &str)] = &[
    ("fleet", "needs a fleet manifest and an artifact to roll out"),
    ("completions", "emits a shell script; it refuses `--json` by design and says so"),
    // Named in NOT_SWEPT above for the same reasons; repeated here so this table is readable on
    // its own rather than by subtraction.
    ("lsp", "a JSON-RPC server: it blocks on stdin by design and is not run-once"),
    ("repl", "interactive: it reads a human line at a time and never exits on its own"),
    ("broker", "can start a daemon; not run-once"),
    ("doctor", "without `--check` it WRITES `docs/survey/`; covered by its own test above"),
];

fn fixture() -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-success-envelope-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();

    // A dependency to `add --path`, and the package that adds it.
    let dep = d.join("dep");
    std::fs::create_dir_all(dep.join("src")).unwrap();
    std::fs::write(
        dep.join("delulu.toml"),
        "[package]\nname = \"dep\"\nversion = \"0.1.0\"\n\n[authority]\neffects = []\n",
    )
    .unwrap();
    std::fs::write(
        dep.join("src").join("dep.delulu"),
        "module dep\n\npub fn v(x: Int) -> Int {\n    x + 1\n}\n",
    )
    .unwrap();

    let pkg = d.join("pkg");
    std::fs::create_dir_all(pkg.join("src")).unwrap();
    std::fs::write(
        pkg.join("delulu.toml"),
        "[package]\nname = \"pkg\"\nversion = \"0.1.0\"\n\n[authority]\neffects = [\"Write\"]\n",
    )
    .unwrap();
    std::fs::write(
        pkg.join("src").join("main.delulu"),
        "module pkg\n\nfn greet(name: Str) -> Str {\n    \"hi, \" + name\n}\n\n\
         fn main(root: Root) ! {Write} {\n    let out = root.console()\n    out.println(greet(\"world\"))\n}\n\n\
         test \"greet builds the message\" {\n    assert_eq(greet(\"x\"), \"hi, x\")\n}\n",
    )
    .unwrap();
    pkg
}

/// Every `--json` success must carry the five documented fields, typed.
fn assert_envelope(label: &str, stdout: &str, want_command: &str) -> Vec<String> {
    let mut bad = Vec::new();
    let v: serde_json::Value = match serde_json::from_str(stdout.trim()) {
        Ok(v) => v,
        Err(e) => {
            bad.push(format!("{label}: stdout is not one JSON value ({e}): {stdout}"));
            return bad;
        }
    };
    if v["command"] != serde_json::Value::String(want_command.to_string()) {
        bad.push(format!("{label}: command = {} , want \"{want_command}\"", v["command"]));
    }
    if v["schema"] != 1 {
        bad.push(format!("{label}: schema = {}, want 1", v["schema"]));
    }
    if !v["delulu_version"].is_string() {
        bad.push(format!("{label}: delulu_version missing or not a string"));
    }
    if !v["diagnostics"].is_array() {
        bad.push(format!("{label}: diagnostics missing or not an array"));
    }
    if !v["summary"].is_object() {
        bad.push(format!("{label}: summary missing or not an object"));
    }
    // `summary.errors == 0` is the documented definition of "this passed", so on an exit of 0 it
    // must be 0 — and it is checked here rather than trusted, because a verdict string and an exit
    // code that disagree is the defect on record in `docs/security/DRILL-001.md`.
    if v["summary"]["errors"] != 0 {
        bad.push(format!(
            "{label}: exited 0 but summary.errors = {} — a verdict that disagrees with the exit code",
            v["summary"]["errors"]
        ));
    }
    bad
}

#[test]
fn every_json_success_emits_the_documented_envelope() {
    let pkg = fixture();
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let home = pkg.join("home");
    std::fs::create_dir_all(&home).unwrap();

    let run_in = |cwd: &std::path::Path, args: &[&str]| -> Output {
        Command::new(env!("CARGO_BIN_EXE_delulu"))
            .args(args)
            .current_dir(cwd)
            .env("DELULU_HOME", &home)
            .env("DELULU_STATE_DIR", &home)
            .env("DELULU_NO_FIRST_RUN", "1")
            .env("DELULU_NO_COLOR", "1")
            .output()
            .expect("the delulu binary must run")
    };

    // A plugin artifact for the three `plugin` verbs and for `why <file.dpx>`, and a `.dwx`
    // artifact for the `authority <artifact>` path — both built through the CLI, so the fixture
    // cannot drift from what the tool actually produces.
    let dpx = pkg.join("shout.dpx");
    let built = run_in(&repo, &["plugin", "build", "examples/plugin_shout", "-o", &dpx.display().to_string()]);
    assert_eq!(built.status.code(), Some(0), "fixture plugin: {}", String::from_utf8_lossy(&built.stderr));
    let dwx = pkg.join("pkg.dwx");
    let wasm = run_in(
        &pkg,
        &["build", "src/main.delulu", "--target", "wasm", "-o", &dwx.display().to_string()],
    );
    assert_eq!(wasm.status.code(), Some(0), "fixture .dwx: {}", String::from_utf8_lossy(&wasm.stderr));
    let locked = run_in(&pkg, &["lock", "."]);
    assert_eq!(locked.status.code(), Some(0), "fixture lockfile: {}", String::from_utf8_lossy(&locked.stderr));
    // The default signing key, so `sign`/`verify-sig` have one; the swept `keygen` case makes a
    // second, named one (a `keygen` that would overwrite a key refuses, correctly).
    let kg = run_in(&pkg, &["keygen"]);
    assert_eq!(kg.status.code(), Some(0), "fixture key: {}", String::from_utf8_lossy(&kg.stderr));
    std::fs::write(pkg.join("env.toml"), "[authority]
effects = [\"Write\"]
").unwrap();

    // (subcommand, argv, cwd, the `command` field the envelope must carry)
    let dpx_s = dpx.display().to_string();
    let dwx_s = dwx.display().to_string();
    // P4-04: `edit` names the hash of the bytes it was computed against, so the case reads it.
    let main_hash = blake3::hash(&std::fs::read(pkg.join("src").join("main.delulu")).unwrap()).to_hex().to_string();
    let cases: Vec<(&str, Vec<&str>, Cwd, &str)> = vec![
        ("check", vec!["check", ".", "--json"], Cwd::Pkg, "check"),
        ("authority", vec!["authority", ".", "--json"], Cwd::Pkg, "authority"),
        ("authority-artifact", vec!["authority", &dwx_s, "--json"], Cwd::Pkg, "authority"),
        ("authority-diff", vec!["authority", "--diff", "delulu.lock", ".", "--json"], Cwd::Pkg, "authority"),
        ("build", vec!["build", ".", "--json"], Cwd::Pkg, "build"),
        ("lock", vec!["lock", ".", "--json"], Cwd::Pkg, "lock"),
        ("fmt", vec!["fmt", "src", "--check", "--json"], Cwd::Pkg, "fmt"),
        ("test", vec!["test", ".", "--json"], Cwd::Pkg, "test"),
        ("fix", vec!["fix", "src/main.delulu", "--dry-run", "--json"], Cwd::Pkg, "fix"),
        ("new", vec!["new", "scaffolded", "--json"], Cwd::Pkg, "new"),
        ("add", vec!["add", "--path", "../dep", "--json"], Cwd::Pkg, "add"),
        ("plugin-build", vec!["plugin", "build", "examples/plugin_shout", "-o", &dpx_s, "--json"], Cwd::Repo, "plugin"),
        ("plugin-verify", vec!["plugin", "verify", &dpx_s, "--json"], Cwd::Pkg, "plugin"),
        ("plugin-inspect", vec!["plugin", "inspect", &dpx_s, "--json"], Cwd::Pkg, "plugin"),
        ("why", vec!["why", "Write", "src/main.delulu", "--json"], Cwd::Pkg, "why"),
        ("why-absent", vec!["why", "Net", "src/main.delulu", "--json"], Cwd::Pkg, "why"),
        ("why-dpx", vec!["why", "Read", &dpx_s, "--json"], Cwd::Pkg, "why"),
        ("explain", vec!["explain", "DL0501", "--json"], Cwd::Pkg, "explain"),
        ("explain-topic", vec!["explain", "E-PLUGIN", "--json"], Cwd::Pkg, "explain"),
        ("explain-unallocated", vec!["explain", "DL0503", "--json"], Cwd::Pkg, "explain"),
        // P4a: the Agent Skill. It takes no path and no argument, so any working directory serves.
        ("skill", vec!["skill", "--json"], Cwd::Pkg, "skill"),
        // P4-02: the toolchain as data. No argument, any working directory.
        ("toolchain", vec!["toolchain", "--json"], Cwd::Pkg, "toolchain"),
        // P4-09: the schemas, listed and one printed.
        ("schema", vec!["schema", "--json"], Cwd::Pkg, "schema"),
        // P4-10: the shipped examples, each with its authority report.
        ("examples", vec!["examples", "--json"], Cwd::Pkg, "examples"),
        // P4-04: a checked edit — no edits, dry run, so the fixture is left as it was.
        ("edit", vec!["edit", "src/main.delulu", "--expect-hash", &main_hash, "--edits", "[]", "--dry-run", "--json"], Cwd::Pkg, "edit"),
        ("schema-one", vec!["schema", "diagnostic", "--json"], Cwd::Pkg, "schema"),
        ("atlas", vec!["atlas", "src/main.delulu", "--json"], Cwd::Pkg, "atlas"),
        ("atlas-query", vec!["atlas", "node", "main", ".", "--json"], Cwd::Pkg, "atlas"),
        ("locale", vec!["locale", "list", "--json"], Cwd::Pkg, "locale"),
        ("morph", vec!["morph", "list", "--json"], Cwd::Pkg, "morph"),
        ("secrets", vec!["secrets", "list", "--json"], Cwd::Pkg, "secrets"),
        ("audit", vec!["audit", "tail", "--json"], Cwd::Pkg, "audit"),
        ("keygen", vec!["keygen", "--name", "swept", "--json"], Cwd::Pkg, "keygen"),
        ("sign", vec!["sign", "src/main.delulu", "--json"], Cwd::Pkg, "sign"),
        ("verify-sig", vec!["verify-sig", "src/main.delulu", "--json"], Cwd::Pkg, "verify-sig"),
        ("login", vec!["login", "--registry", "http://127.0.0.1:1", "--token", "t", "--json"], Cwd::Pkg, "login"),
        ("publish", vec!["publish", ".", "--dry-run", "--json"], Cwd::Pkg, "publish"),
        ("deploy", vec!["deploy", "plan", "--service", "s=.", "--env", "env.toml", "--json"], Cwd::Pkg, "deploy"),
        ("version", vec!["--version", "--json"], Cwd::Pkg, "version"),
        ("sandbox", vec!["sandbox", "probe", "--json"], Cwd::Pkg, "sandbox"),
    ];

    let mut bad: Vec<String> = Vec::new();
    for (label, args, cwd, want) in &cases {
        let dir: &std::path::Path = if *cwd == Cwd::Pkg { &pkg } else { &repo };
        let out = run_in(dir, args);
        let code = out.status.code().unwrap_or(-1);
        let so = String::from_utf8_lossy(&out.stdout).to_string();
        if code != 0 {
            // A case that stopped succeeding proves nothing about the success envelope, so it is a
            // failure of the sweep rather than a silent skip (the C19/P5 trap).
            bad.push(format!(
                "{label}: `delulu {}` exited {code}, so the success envelope was never exercised:\n{}\n{so}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr)
            ));
            continue;
        }
        if count_json_values(&so) != 1 {
            bad.push(format!("{label}: stdout is not exactly one JSON value:\n{so}"));
            continue;
        }
        bad.extend(assert_envelope(label, &so, want));
    }
    let _ = std::fs::remove_dir_all(pkg.parent().unwrap());
    assert!(
        bad.is_empty(),
        "`--json` promises the same five fields on success as on failure:\n  {}",
        bad.join("\n  ")
    );
}

/// NE-06: `explain` accepted `--json` and ignored it. The option parser exempted the flag from its
/// own "an option nobody understood is refused, never ignored" rule and then nothing read it, so
/// `delulu explain DL0501 --json` printed prose and exited 0 — and `explain` sat inside
/// `SUBCOMMANDS` the whole time, passing, because the sweep above only ever drove it to failure.
///
/// Both halves are tested here, because fixing the first could easily break the second: the flag
/// must now be honoured, and every OTHER flag must still be refused with exit 2.
#[test]
fn explain_answers_on_the_machine_channel_for_all_three_registries() {
    // (argv code, the `kind` the answer must carry, a word the body must contain)
    let cases: &[(&str, &str, &str)] = &[
        // An allocated diagnostic code.
        ("DL0501", "code", "row"),
        // A named topic (`E-…`), reachable with or without the prefix.
        ("E-PLUGIN", "topic", "plugin"),
        // A code the registry deliberately never allocated, with its recorded disposition.
        ("DL0503", "unallocated", "retired"),
    ];
    for (code, kind, needle) in cases {
        let out = run(&["explain", code, "--json"]);
        assert_eq!(out.status.code(), Some(0), "`explain {code} --json` must answer");
        let s = String::from_utf8_lossy(&out.stdout).to_string();
        assert_eq!(count_json_values(&s), 1, "exactly one object for `{code}`:\n{s}");
        let v: serde_json::Value = serde_json::from_str(s.trim()).unwrap();
        assert_eq!(v["command"], "explain");
        assert_eq!(v["schema"], 1);
        assert_eq!(v["summary"]["errors"], 0);
        let e = &v["explain"];
        assert!(e.is_object(), "the answer rides in an `explain` object: {v}");
        assert!(e["code"].is_string(), "`code` identifies the answer: {e}");
        assert!(e["title"].is_string(), "`title` is the one-line answer: {e}");
        assert_eq!(e["kind"], *kind, "which registry answered, for `{code}`: {e}");
        // `body` and `disposition` are ALWAYS present — `null` where they do not apply. A missing
        // field cannot be told apart from "this tool did not answer" (`docs/for-agents.md`).
        assert!(e.get("body").is_some(), "`body` is present even when null: {e}");
        assert!(e.get("disposition").is_some(), "`disposition` is present even when null: {e}");
        let blob = format!("{} {}", e["title"], e["body"]).to_lowercase();
        assert!(blob.contains(needle), "the body is the real explanation for `{code}`: {e}");
        // The human channel is unchanged: prose, no JSON.
        let human = run(&["explain", code]);
        assert_eq!(human.status.code(), Some(0));
        let h = String::from_utf8_lossy(&human.stdout).to_string();
        assert_eq!(count_json_values(&h), usize::MAX, "the human channel stays prose:\n{h}");
    }
    // An allocated code carries `disposition: null` — it was allocated, so there is nothing to
    // dispose of — while an unallocated one names the disposition. The two must not look alike.
    let allocated: serde_json::Value =
        serde_json::from_slice(&run(&["explain", "DL0501", "--json"]).stdout).unwrap();
    assert!(allocated["explain"]["disposition"].is_null());
    let retired: serde_json::Value =
        serde_json::from_slice(&run(&["explain", "DL0503", "--json"]).stdout).unwrap();
    assert_eq!(retired["explain"]["disposition"], "retired");
}

#[test]
fn explain_still_refuses_an_option_it_does_not_know() {
    // The rule `--json` was smuggled past. Honouring one flag must not turn `explain` into a
    // command that ignores flags, and a refusal is exit 2 — usage, not a diagnostic.
    for bad in ["--bogus", "--jsonn", "-j", "--json=1"] {
        let out = run(&["explain", "DL0501", bad]);
        assert_eq!(
            out.status.code(),
            Some(2),
            "`explain DL0501 {bad}` must be refused, not ignored:\n{}",
            String::from_utf8_lossy(&out.stdout)
        );
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains("does not know the option"), "and must say so: {err}");
    }
}

/// The other direction of DRILL-001. The sweep above proves a zero exit reports zero errors; this
/// proves the converse for the paths that **ran, produced their own report, and failed**. Those are
/// the ones the argument-shape sweep can never reach, because it only ever produces refusals during
/// argument parsing — and they are where a verdict and an exit code can quietly disagree.
#[test]
fn a_report_shaped_failure_never_claims_zero_errors() {
    let d = std::env::temp_dir().join(format!("delulu-verdict-exit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let pkg = d.join("pkg");
    let dep = d.join("dep");
    std::fs::create_dir_all(pkg.join("src")).unwrap();
    std::fs::create_dir_all(dep.join("src")).unwrap();
    let home = d.join("home");
    std::fs::create_dir_all(&home).unwrap();

    // A dependency that performs an effect: `add --path` must refuse it without --accept-authority.
    std::fs::write(
        dep.join("delulu.toml"),
        "[package]\nname = \"dep\"\nversion = \"0.1.0\"\n\n[authority]\neffects = [\"Write\"]\n",
    )
    .unwrap();
    std::fs::write(
        dep.join("src").join("dep.delulu"),
        "module dep\n\npub fn shout(root: Root) ! {Write} {\n    let out = root.console()\n    out.println(\"x\")\n}\n",
    )
    .unwrap();
    std::fs::write(
        pkg.join("delulu.toml"),
        "[package]\nname = \"pkg\"\nversion = \"0.1.0\"\n\n[authority]\neffects = [\"Write\"]\n",
    )
    .unwrap();
    // Deliberately unformatted (double blank line, trailing spaces) so `fmt --check` fails.
    std::fs::write(
        pkg.join("src").join("main.delulu"),
        "module pkg\n\n\n\nfn main(root: Root) ! {Write} {\n    let out = root.console()   \n    out.println(\"hi\")\n}\n",
    )
    .unwrap();
    // A test that fails at runtime, and an environment profile the package exceeds.
    std::fs::write(
        pkg.join("failing.delulu"),
        "module failing\n\ntest \"two is three\" {\n    assert_eq(str(2), str(3))\n}\n",
    )
    .unwrap();
    std::fs::write(pkg.join("pure-env.toml"), "[authority]\neffects = []\n").unwrap();

    let run_in = |args: &[&str]| -> Output {
        Command::new(env!("CARGO_BIN_EXE_delulu"))
            .args(args)
            .current_dir(&pkg)
            .env("DELULU_HOME", &home)
            .env("DELULU_STATE_DIR", &home)
            .env("DELULU_NO_FIRST_RUN", "1")
            .env("DELULU_NO_COLOR", "1")
            .output()
            .expect("the delulu binary must run")
    };

    let cases: Vec<(&str, Vec<&str>)> = vec![
        ("fmt-check-dirty", vec!["fmt", "src", "--check", "--json"]),
        ("add-refused", vec!["add", "--path", "../dep", "--json"]),
        ("verify-sig-unsigned", vec!["verify-sig", "src/main.delulu", "--json"]),
        ("deploy-exceeds-ceiling", vec!["deploy", "plan", "--service", "s=.", "--env", "pure-env.toml", "--json"]),
        ("test-failing", vec!["test", "failing.delulu", "--json"]),
    ];

    let mut bad: Vec<String> = Vec::new();
    for (label, args) in &cases {
        let out = run_in(args);
        let code = out.status.code().unwrap_or(-1);
        let so = String::from_utf8_lossy(&out.stdout).to_string();
        if code == 0 {
            bad.push(format!(
                "{label}: `delulu {}` exited 0, so this case witnesses nothing:\n{so}",
                args.join(" ")
            ));
            continue;
        }
        if count_json_values(&so) != 1 {
            bad.push(format!("{label}: exited {code} without exactly one JSON object:\n{so}"));
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(so.trim()).expect("one object");
        if v["summary"]["errors"] == 0 {
            bad.push(format!(
                "{label}: exited {code} while reporting summary.errors == 0 — a verdict and an exit \
                 code that disagree (DRILL-001):\n{so}"
            ));
        }
    }
    let _ = std::fs::remove_dir_all(&d);
    assert!(bad.is_empty(), "verdict and exit code must agree:\n  {}", bad.join("\n  "));
}

/// The completeness half. The sweep above is a hand-written table, and a hand-written list falls
/// behind the thing that defines it (C31/C34/C35/C44/C52 — the campaign's most repeated shape). So:
/// every name in `SUBCOMMANDS` is either swept for its success envelope or named in
/// `NO_SUCCESS_SWEEP` **with a reason**.
#[test]
fn every_subcommand_is_either_success_swept_or_excused_in_writing() {
    let src = std::fs::read_to_string(std::path::Path::new(file!()))
        .or_else(|_| {
            std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("json_contract.rs"),
            )
        })
        .expect("this test file must be readable — it is the table this gate reads");
    let table = src
        .split_once("let cases: Vec<(&str, Vec<&str>, Cwd, &str)> = vec![")
        .map(|(_, rest)| rest.split_once("\n    ];").map(|(t, _)| t).unwrap_or(rest))
        .expect("the success table must be present");
    assert!(table.len() > 500, "the table scan broke, and a gate that reads nothing passes everything");

    // The broker verbs are swept by their own test, which needs a running daemon (P1-F2).
    let broker_sweep = src
        .split_once("fn broker_verbs_emit_the_documented_envelope()")
        .map(|(_, rest)| rest)
        .expect("the broker success sweep must be present");
    let mut missing: Vec<&str> = Vec::new();
    for sub in SUBCOMMANDS {
        let swept = table.contains(&format!("vec![\"{sub}\""))
            || broker_sweep.contains(&format!("step(&[\"{sub}\""))
            // `run` reports through `--report-out` (D-V2-21), swept by its own test.
            || (*sub == "run" && src.contains("fn run_report_is_the_documented_envelope()"));
        let excused = NO_SUCCESS_SWEEP.iter().any(|(n, _)| n == sub);
        if !swept && !excused {
            missing.push(sub);
        }
    }
    assert!(
        missing.is_empty(),
        "these subcommands have no success-envelope case and no recorded reason: {missing:?}\n\
         Add an invocation to the table, or an entry to NO_SUCCESS_SWEEP saying why it cannot be driven."
    );
    for (_, why) in NO_SUCCESS_SWEEP {
        assert!(why.len() > 20, "an exclusion needs a real reason, not a word");
    }
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

/// Stops the sweep's broker on drop, never panicking (a Drop panic masks the real assertion).
struct SweepDaemon {
    state: std::path::PathBuf,
}
impl Drop for SweepDaemon {
    fn drop(&mut self) {
        let _ = Command::new(env!("CARGO_BIN_EXE_delulu"))
            .current_dir(std::env::temp_dir())
            .env("DELULU_STATE_DIR", &self.state)
            .args(["broker", "stop"])
            .output();
    }
}

/// The verbs a broker command lists in its own unknown-subcommand refusal, e.g.
/// `(list | tree | … | pubkey)`. Read from the binary, so a verb added to the dispatcher and its
/// message is a verb this sweep must drive — the list is not kept by hand and cannot fall behind.
fn advertised_verbs(command: &str) -> Vec<String> {
    let out = run(&[command, "no-such-verb"]);
    assert_eq!(out.status.code(), Some(2), "`{command} no-such-verb` must be refused");
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    let list = err
        .rsplit_once('(')
        .and_then(|(_, t)| t.split_once(')'))
        .map(|(l, _)| l.to_string())
        .unwrap_or_else(|| panic!("the refusal must list the verbs: {err}"));
    let verbs: Vec<String> = list.split('|').map(|v| v.trim().to_string()).filter(|v| !v.is_empty()).collect();
    assert!(verbs.len() >= 5, "the verb list scan broke, and a sweep of nothing passes: {err}");
    verbs
}

/// P1-F2: every `grants` and `guard` verb prints its `--json` success inside the standard envelope.
/// A real broker is started in a temporary state directory (invariant 27: these verbs have no
/// local fallback), and each verb is driven with `--json`; the offline federation verbs
/// (`pubkey`, `certify`, `receipt`) run beside it, as they do in the field.
#[test]
fn broker_verbs_emit_the_documented_envelope() {
    // Short names: the broker's socket lives in the state dir, and macOS caps a socket path at 103
    // bytes (`federation_cli.rs`).
    let base = std::env::temp_dir().join(format!("dsw_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cwd = base.join("w");
    let state = base.join("s");
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    let go = |args: &[&str]| -> Output {
        Command::new(env!("CARGO_BIN_EXE_delulu"))
            .args(args)
            .current_dir(&cwd)
            .env("DELULU_STATE_DIR", &state)
            .env("DELULU_HOME", &state)
            .env("DELULU_NO_FIRST_RUN", "1")
            .env("DELULU_NO_COLOR", "1")
            .output()
            .expect("the delulu binary must run")
    };

    let started = go(&["broker", "start"]);
    assert!(started.status.success(), "broker start: {}", String::from_utf8_lossy(&started.stderr));
    let daemon = SweepDaemon { state: state.clone() };
    let start_text =
        format!("{}{}", String::from_utf8_lossy(&started.stdout), String::from_utf8_lossy(&started.stderr));
    let owner = {
        let i = start_text.find("gow1_").expect("broker start prints the guard owner code");
        let tail = &start_text[i..];
        let end = tail.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(tail.len());
        tail[..end].to_string()
    };

    let mut bad: Vec<String> = Vec::new();
    let mut driven: Vec<String> = Vec::new();
    // Drive one verb with `--json`: it must exit 0 and print exactly one envelope, returned.
    let mut step = |args: &[&str], want: &str| -> serde_json::Value {
        let mut argv: Vec<&str> = args.to_vec();
        argv.push("--json");
        let out = go(&argv);
        let so = String::from_utf8_lossy(&out.stdout).to_string();
        // The owner code is a credential: it never reaches a failure message.
        let label = argv.join(" ").replace(owner.as_str(), "<owner>");
        driven.push(format!("{} {}", args[0], args[1]));
        if out.status.code() != Some(0) {
            bad.push(format!(
                "`delulu {label}` exited {:?}:\n{}\n{so}",
                out.status.code(),
                String::from_utf8_lossy(&out.stderr)
            ));
            return serde_json::Value::Null;
        }
        if count_json_values(&so) != 1 {
            bad.push(format!("`delulu {label}`: stdout is not exactly one JSON value:\n{so}"));
            return serde_json::Value::Null;
        }
        bad.extend(assert_envelope(&label, &so, want));
        serde_json::from_str(so.trim()).unwrap_or(serde_json::Value::Null)
    };

    // ----- grants: the local tree -------------------------------------------------------------
    let d = step(&["grants", "delegate", "--effects", "Declassify", "--declassify", "S", "--owner", &owner], "grants");
    let node = d["node"].as_str().unwrap_or("g_missing").to_string();
    assert!(d["token"].is_string(), "the delegate report keeps its keys at the top level: {d}");
    let l = step(&["grants", "list"], "grants");
    assert!(l["nodes"].is_array() && l["subcommand"] == "list", "additive: {l}");
    step(&["grants", "tree"], "grants");
    step(&["grants", "inspect", &node], "grants");

    // ----- guard ------------------------------------------------------------------------------
    let s = step(&["guard", "status"], "guard");
    assert_eq!(s["bypass"], false, "the status report keeps its keys at the top level: {s}");
    step(&["guard", "policy", "show"], "guard");
    step(&["guard", "policy", "set", "net:*", "guarded", "--owner", &owner], "guard");
    step(&["guard", "policy", "unset", "net:*", "--owner", &owner], "guard");
    step(&["guard", "bypass", "on", "--owner", &owner], "guard");
    step(&["guard", "bypass", "off", "--owner", &owner], "guard");
    let r = step(&["guard", "request", &node, "--use", "declassify:*", "--why", "sweep"], "guard");
    let req = r["id"].as_str().unwrap_or("r_missing").to_string();
    step(&["guard", "pending"], "guard");
    let a = step(&["guard", "approve", &req, "--owner", &owner, "--comment", "sweep"], "guard");
    let permit = a["permit"].as_str().unwrap_or("p_missing").to_string();
    step(&["guard", "permits"], "guard");
    step(&["guard", "permits", "revoke", &permit, "--owner", &owner], "guard");
    let r2 = step(&["guard", "request", &node, "--use", "declassify:*", "--why", "sweep again"], "guard");
    let req2 = r2["id"].as_str().unwrap_or("r_missing").to_string();
    step(&["guard", "deny", &req2, "--owner", &owner, "--comment", "sweep"], "guard");

    // ----- grants: federation (ground side offline, vehicle side on this broker) ----------------
    let gk = cwd.join("ground.key").display().to_string();
    let vk = cwd.join("vehicle.key").display().to_string();
    let gp = step(&["grants", "pubkey", "--key", &gk], "grants pubkey");
    let ground = gp["pubkey"].as_str().unwrap_or("").to_string();
    let vp = step(&["grants", "pubkey", "--key", &vk], "grants pubkey");
    let vehicle = vp["pubkey"].as_str().unwrap_or("").to_string();
    step(
        &[
            "grants", "certify", "--subject", &vehicle, "--effects", "Write", "--ttl", "1h", "--uplink-ttl", "1h",
            "--key", &gk, "--out", "c.dlcert",
        ],
        "grants certify",
    );
    let ad = step(&["grants", "adopt", "c.dlcert", "--anchor", &ground], "grants adopt");
    let fp = ad["fingerprint"].as_str().unwrap_or("").to_string();
    step(&["grants", "receipt", "--for", &fp, "--ttl", "1h", "--key", &gk, "--out", "r.dlrcpt"], "grants receipt");
    step(&["grants", "renew", "r.dlrcpt", "--anchor", &ground], "grants renew");

    // Last, because it kills the node the steps above used.
    step(&["grants", "revoke", &node], "grants");

    // Every verb each command advertises was driven.
    for command in ["grants", "guard"] {
        for verb in advertised_verbs(command) {
            let key = format!("{command} {verb}");
            if !driven.contains(&key) {
                bad.push(format!("`{key}` is advertised but this sweep never drove it with --json"));
            }
        }
    }
    drop(daemon);
    let _ = std::fs::remove_dir_all(&base);
    assert!(bad.is_empty(), "broker verbs must print the documented envelope:\n  {}", bad.join("\n  "));
}

/// PS-0-02 (D-V2-21): `run --json --report-out <path>` — the runtime writes one envelope to the
/// file, with the `sandbox` object and the outcome, whether the program ran or was refused. The
/// program's stdout stays its own, and a program printing a counterfeit `sandbox` object changes
/// nothing in the report.
#[test]
fn run_report_is_the_documented_envelope() {
    let d = std::env::temp_dir().join(format!("delulu-run-report-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("out")).unwrap();
    std::fs::create_dir_all(d.join("rep")).unwrap();
    // The counterfeit: the program prints what a confused consumer might take for the report.
    std::fs::write(
        d.join("fake.delulu"),
        "module fake\n\nfn main(root: Root) ! {Write} {\n    let out = root.console()\n    \
         out.println(\"{\\\"sandbox\\\":{\\\"backend\\\":\\\"microvm\\\",\\\"level\\\":4,\\\"break_glass\\\":true}}\")\n}\n",
    )
    .unwrap();
    std::fs::write(d.join("bad.delulu"), "module bad\n\nfn main(root: Root) ! {Write} {\n    let out = root.console()\n    out.println(nope)\n}\n")
        .unwrap();
    let go = |args: &[&str]| -> Output {
        Command::new(env!("CARGO_BIN_EXE_delulu"))
            .args(args)
            .current_dir(&d)
            .env("DELULU_NO_FIRST_RUN", "1")
            .env("DELULU_NO_COLOR", "1")
            .output()
            .expect("the delulu binary must run")
    };
    let expected_sandbox = serde_json::json!({
        "backend": "inproc", "level": 0, "requested": "none", "granted": "none",
        "host_guarantees": [], "mode": "strict", "break_glass": false,
        // PS-B-01: an ordinary run is budgeted now (D-V2-25), so `limits` is the budget it is held to
        // rather than `null`. Compared field by field below, not by its prose `enforced_by` line.
        "limits": { "memory_bytes": 1u64 << 30, "cpu_seconds": 300, "wall_seconds": null },
    });
    let read = |p: &str| -> serde_json::Value {
        let text = std::fs::read_to_string(d.join(p)).unwrap_or_else(|e| panic!("no report at {p}: {e}"));
        assert_eq!(count_json_values(&text), 1, "one envelope: {text}");
        serde_json::from_str(text.trim()).unwrap()
    };
    // The report's `sandbox` object with its one prose field checked and set aside: `enforced_by`
    // must say what enforces the budget, and everything else must match exactly.
    let sandbox_of = |v: &serde_json::Value| -> serde_json::Value {
        let mut s = v["sandbox"].clone();
        let how = s["limits"]["enforced_by"].take();
        assert!(how.as_str().is_some_and(|h| !h.is_empty()), "the budget says what enforces it: {v}");
        s["limits"].as_object_mut().expect("limits is an object").remove("enforced_by");
        s
    };

    // It ran: the program's own bytes on stdout, the report in the file.
    let o = go(&["run", "fake.delulu", "--grant", "console", "--json", "--report-out", "rep/ran.json"]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(String::from_utf8_lossy(&o.stdout).contains("\"level\":4"), "stdout is the program's");
    let text = std::fs::read_to_string(d.join("rep/ran.json")).unwrap();
    let bad = assert_envelope("run --report-out", &text, "run");
    assert!(bad.is_empty(), "{bad:?}");
    let v = read("rep/ran.json");
    assert_eq!(sandbox_of(&v), expected_sandbox, "the counterfeit changed nothing: {v}");
    assert_eq!(v["outcome"], serde_json::json!({ "ran": true, "exit": 0 }), "{v}");

    // Refused before it started: the report still exists, and says so.
    let o = go(&["run", "bad.delulu", "--grant", "console", "--json", "--report-out", "rep/refused.json"]);
    assert_eq!(o.status.code(), Some(1));
    let v = read("rep/refused.json");
    assert_eq!(sandbox_of(&v), expected_sandbox, "{v}");
    assert_eq!(v["outcome"], serde_json::json!({ "ran": false, "exit": 1 }), "{v}");
    assert_eq!(v["summary"]["errors"], 1, "a failed run never reports zero errors: {v}");

    // A bare file name means the current directory, the commonest spelling of all. The head chef's
    // PS-0 verification found it refused as "cannot be resolved", because its parent is empty.
    let o = go(&["run", "fake.delulu", "--grant", "console", "--json", "--report-out", "bare.json"]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(sandbox_of(&read("bare.json")), expected_sandbox);
    let o = go(&["run", "fake.delulu", "--grant", "console", "--trace-effects", "--trace-out", "bare-trace.txt"]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(d.join("bare-trace.txt").is_file(), "the bare-named trace was written");
    // ... and a bare name is still refused when the current directory itself is writable.
    let o = go(&["run", "fake.delulu", "--grant", "console", "--grant", "fs.write=.", "--report-out", "bare2.json"]);
    assert_eq!(o.status.code(), Some(2), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(!d.join("bare2.json").exists(), "nothing written");

    // Inside a writable grant — by any spelling — the report (and the trace) are refused, and
    // nothing runs.
    for (flag, path) in [
        ("--report-out", "out/r.json"),
        ("--report-out", "rep/../out/r.json"),
        ("--trace-out", "out/t.txt"),
    ] {
        let mut args = vec!["run", "fake.delulu", "--grant", "console", "--grant", "fs.write=./out", flag, path];
        if flag == "--trace-out" {
            args.push("--trace-effects");
        }
        let o = go(&args);
        assert_eq!(o.status.code(), Some(2), "`{}`: {}", args.join(" "), String::from_utf8_lossy(&o.stderr));
        assert!(String::from_utf8_lossy(&o.stderr).contains("D-V2-21"), "{}", String::from_utf8_lossy(&o.stderr));
        assert!(!String::from_utf8_lossy(&o.stdout).contains("sandbox"), "nothing ran");
        assert!(std::fs::read_dir(d.join("out")).unwrap().next().is_none(), "nothing written in the scope");
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// PS-0-02: the runtime opens its report without following a symbolic link at the final component.
#[cfg(unix)]
#[test]
fn run_report_does_not_follow_a_planted_link() {
    let d = std::env::temp_dir().join(format!("delulu-run-report-link-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("p.delulu"), "module p\n\nfn main() {\n}\n").unwrap();
    std::fs::write(d.join("victim.txt"), "keep").unwrap();
    std::os::unix::fs::symlink(d.join("victim.txt"), d.join("r.json")).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(["run", "p.delulu", "--json", "--report-out", "r.json"])
        .current_dir(&d)
        .env("DELULU_NO_FIRST_RUN", "1")
        .output()
        .unwrap();
    assert_ne!(o.status.code(), Some(0), "a planted link is refused");
    assert_eq!(std::fs::read_to_string(d.join("victim.txt")).unwrap(), "keep", "the link's target is untouched");
    let _ = std::fs::remove_dir_all(&d);
}
