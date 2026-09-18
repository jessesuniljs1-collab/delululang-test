//! The CLI subcommand contracts (Stage 9a, invariant 42 — the coverage law).
//!
//! Every `ref.cli.<subcommand>` anchor needs one ACCEPTING and one REJECTING witness. Most
//! subcommands already have them in the end-to-end suites (`cli.rs`, `provenance.rs`,
//! `signing_cli.rs`, `plugin_cli.rs`, `test_runner_cli.rs`, `lsp_cli.rs`, `broker_cli.rs`,
//! `grants_cli.rs`, `guard_e2e.rs`) and `tests/conformance/witnesses.toml` cites those directly.
//! This file covers the remainder, table-driven, through the real binary.
//!
//! The rejecting half is the skip-branch case (house rule 3): an invalid invocation must REFUSE
//! with a nonzero exit and an honest message — never exit 0, never panic, never silently do
//! something else. A CLI that shrugs at bad input is the same fail-open failure as a checker that
//! shrugs at an unmodeled type.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// Run the real binary with an isolated `DELULU_HOME` so no test touches the user's state.
fn delulu(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(workspace_root())
        .env("DELULU_HOME", home)
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("failed to run delulu")
}

fn scratch(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-cli-contract-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

const DEMO: &str = "examples/demo.delulu";
const REJECT: &str = "tests/conformance/reject/DL0501_undeclared_effect.delulu";

/// (subcommand, accepting argv, rejecting argv). The subcommand is the first element of both.
fn contracts() -> Vec<(&'static str, Vec<&'static str>, Vec<&'static str>)> {
    vec![
        ("check", vec!["check", DEMO], vec!["check"]),
        ("fmt", vec!["fmt", "--check", "examples"], vec!["fmt", "--check", "no-such-path-9a"]),
        ("authority", vec!["authority", DEMO], vec!["authority", "no-such-file-9a.delulu"]),
        ("why", vec!["why", "Write", "examples/greeter"], vec!["why", "NotAnEffect9a", "examples/greeter"]),
        ("atlas", vec!["atlas", DEMO], vec!["atlas", REJECT]),
        ("explain", vec!["explain", "DL0501"], vec!["explain", "DL9999"]),
        ("audit", vec!["audit", "tail", "1"], vec!["audit", "no-such-verb-9a"]),
        ("secrets", vec!["secrets", "list"], vec!["secrets", "no-such-verb-9a"]),
        ("locale", vec!["locale", "list"], vec!["locale", "no-such-verb-9a"]),
    ]
    // `doctor` is deliberately NOT here. This file covers subcommands that lack witnesses
    // elsewhere, and `doctor_cli.rs` carries seven — including the two that `witnesses.toml`
    // cites for `ref.cli.doctor`. Listing it here as well would make every stale map fail an
    // interface test whose message points nowhere useful.
}

/// The ACCEPTING side: every listed subcommand honors a valid invocation (exit 0).
#[test]
fn every_listed_subcommand_honors_a_valid_invocation() {
    let home = scratch("accept");
    for (sub, ok, _) in contracts() {
        let o = delulu(&home, &ok);
        assert!(
            o.status.success(),
            "`delulu {}` must succeed (ref.cli.{sub}), got {:?}:\n{}",
            ok.join(" "),
            o.status.code(),
            text(&o)
        );
    }
}

/// The REJECTING side (the skip-branch case): every listed subcommand refuses an invalid
/// invocation with a nonzero exit and an honest message — never a silent success, never a panic.
#[test]
fn every_listed_subcommand_refuses_an_invalid_invocation() {
    let home = scratch("reject");
    for (sub, _, bad) in contracts() {
        let o = delulu(&home, &bad);
        let out = text(&o);
        assert!(
            !o.status.success(),
            "`delulu {}` must REFUSE (ref.cli.{sub}) — a CLI that shrugs at bad input is fail-open:\n{out}",
            bad.join(" ")
        );
        assert!(
            !out.contains("panicked at"),
            "`delulu {}` panicked instead of refusing honestly (ref.cli.{sub}):\n{out}",
            bad.join(" ")
        );
        assert!(!out.trim().is_empty(), "`delulu {}` refused silently (ref.cli.{sub})", bad.join(" "));
    }
}

/// Subcommands whose accepting side needs real artifacts (covered by the end-to-end suites) still
/// owe a rejecting witness: invoked with no arguments at all, each must refuse honestly.
#[test]
fn argument_taking_subcommands_refuse_an_empty_invocation() {
    let home = scratch("empty-args");
    for sub in ["sign", "verify-sig", "publish", "add", "build", "run", "why", "explain"] {
        let o = delulu(&home, &[sub]);
        let out = text(&o);
        assert!(!o.status.success(), "`delulu {sub}` with no arguments must refuse (ref.cli.{sub}):\n{out}");
        assert!(!out.contains("panicked at"), "`delulu {sub}` panicked instead of refusing:\n{out}");
        assert!(
            out.contains("needs") || out.contains("error"),
            "`delulu {sub}` must say what it needs (ref.cli.{sub}):\n{out}"
        );
    }
}

/// An unknown subcommand is refused by the dispatch itself — the fence around the whole CLI
/// surface. Without this, "every subcommand refuses bad input" could hold vacuously for a
/// subcommand that does not exist at all.
#[test]
fn an_unknown_subcommand_is_refused_by_the_dispatch() {
    let home = scratch("unknown");
    let o = delulu(&home, &["definitely-not-a-subcommand-9a"]);
    assert!(!o.status.success(), "an unknown subcommand must refuse");
    assert!(text(&o).contains("unknown command"), "the refusal must name the problem: {}", text(&o));
}

/// `keygen` writes a key pair into an empty home (accepting), and REFUSES to overwrite an existing
/// private key (rejecting). Silently overwriting a signing key would destroy the only copy of an
/// identity — the destructive skip branch.
#[test]
fn keygen_writes_a_key_then_refuses_to_overwrite_it() {
    let home = scratch("keygen");
    let first = delulu(&home, &["keygen"]);
    assert!(first.status.success(), "keygen into an empty home must succeed: {}", text(&first));
    let key = home.join("keys").join("id_ed25519");
    let before = std::fs::read(&key).expect("keygen wrote a private key");

    let second = delulu(&home, &["keygen"]);
    assert!(!second.status.success(), "a second keygen must refuse: {}", text(&second));
    assert!(text(&second).contains("already exists"), "the refusal names the reason: {}", text(&second));
    let after = std::fs::read(&key).expect("the key still exists");
    assert_eq!(before, after, "keygen must NEVER overwrite an existing private key");
}

/// Runtime arithmetic faults are reported as diagnostics, not as a host crash: an `Int` overflow
/// is DL0901. The producing witness for that code (Stage 9a — it had none).
#[test]
fn integer_overflow_at_runtime_is_dl0901() {
    let home = scratch("overflow");
    let src = home.join("overflow.delulu");
    std::fs::write(
        &src,
        "module m\nfn main(root: Root) {\n  let big = 9223372036854775807\n  let x = big + big\n}\n",
    )
    .unwrap();
    let o = delulu(&home, &["run", src.to_str().unwrap()]);
    let out = text(&o);
    assert!(!o.status.success(), "an overflowing program must not exit 0: {out}");
    assert!(out.contains("DL0901"), "integer overflow must be DL0901: {out}");
    assert!(!out.contains("panicked at"), "the fault must be a diagnostic, not a host panic: {out}");
}

/// Unbounded recursion is DL0905 — a diagnostic, not a host crash.
///
/// Found by Study C (Stage 9e): `fib(24)` killed the process with a raw stack-overflow abort. The
/// interpreter's own `MAX_DEPTH` guard existed but was unreachable, because a tree-walker spends
/// several large native frames per DeluluLang call and the default main-thread stack ran out
/// first. A crash with no diagnostic and no usable exit code is the worst failure mode there is,
/// and it made `ref.rule.runtime.faults-are-diagnostics` false.
#[test]
fn unbounded_recursion_is_dl0905_not_a_host_crash() {
    let home = scratch("recursion");
    let src = home.join("deep.delulu");
    std::fs::write(
        &src,
        "module m\n\nfn down(n: Int) -> Int {\n  if n < 1 { 0 } else { down(n - 1) + 1 }\n}\n\n\
         fn main(root: Root) ! {Write} {\n  let out = root.console()\n  out.println(str(down(60000)))\n}\n",
    )
    .unwrap();
    let o = delulu(&home, &["run", src.to_str().unwrap(), "--grant", "console"]);
    let out = text(&o);
    assert!(out.contains("DL0905"), "deep recursion must report DL0905, got: {out}");
    assert_eq!(
        o.status.code(),
        Some(1),
        "a runtime fault exits 1; a stack-overflow abort produces no usable code at all"
    );
    assert!(!out.contains("has overflowed its stack"), "the host must not crash: {out}");
}

/// THE SKIP-BRANCH CASE: recursion *within* the bound must still work. A fix that made DL0905 fire
/// by refusing all recursion would pass the test above and destroy the language.
#[test]
fn recursion_within_the_bound_still_runs() {
    let home = scratch("recursion-ok");
    let src = home.join("fib.delulu");
    std::fs::write(
        &src,
        "module m\n\nfn fib(n: Int) -> Int {\n  if n < 2 { n } else { fib(n - 1) + fib(n - 2) }\n}\n\n\
         fn main(root: Root) ! {Write} {\n  let out = root.console()\n  out.println(str(fib(24)))\n}\n",
    )
    .unwrap();
    let o = delulu(&home, &["run", src.to_str().unwrap(), "--grant", "console"]);
    let out = text(&o);
    assert!(o.status.success(), "fib(24) must run cleanly: {out}");
    assert!(out.contains("46368"), "fib(24) = 46368, got: {out}");
}

/// A DeluluLang program with an actor whose behavior recurses. The bare source used by both actor
/// tests below; `DEPTH` is substituted.
const ACTOR_RECURSION: &str = "module m\n\n\
     fn down(n: Int) -> Int {\n  if n <= 0 { 0 } else { down(n - 1) + 1 }\n}\n\n\
     actor Deep {\n  var seen: Int\n  new() { self.seen = 0 }\n  \
     be go(n: Int, out: Cap[Console]) ! {Write} {\n    self.seen = down(n)\n    \
     out.println(str(self.seen))\n  }\n}\n\n\
     fn main(root: Root) ! {Async, Write} {\n  let d = spawn Deep()\n  \
     d.go(DEPTH, root.console())\n}\n";

/// **`ref.rule.runtime.faults-are-diagnostics` must hold on the CONCURRENCY path too, and it did
/// not.** `main.rs` reserves a large stack so the interpreter's own bound is what fires; that
/// reservation belongs to the `delulu-main` thread, and the actor scheduler spawned its workers
/// with the OS default and no bound of their own. The same function at the same depth printed its
/// answer from `main` and killed the process from inside a behavior — measured at depth **43**
/// (debug) and **~350** (release) against a documented bound of 10,000 (ruling D67).
///
/// The existing witness above could not see it: it recurses in `main`, which is the one thread
/// where the rule was already true. **A gate keyed on the right signal, on the wrong thread.**
///
/// This asserts on the EXIT STATUS, not on a message, because a stack overflow prints no
/// `panicked at` — the whole reason the no-panic sweeps were blind to this class (ruling D47a).
#[test]
fn recursion_inside_an_actor_is_not_a_host_crash() {
    let home = scratch("actor-recursion");
    let src = home.join("deep_actor.delulu");
    std::fs::write(&src, ACTOR_RECURSION.replace("DEPTH", "1000")).unwrap();
    let o = delulu(&home, &["run", src.to_str().unwrap(), "--grant", "console"]);
    let out = text(&o);
    assert!(
        o.status.success(),
        "a 1,000-deep recursion inside an actor is far inside the bound and must simply run; \
         exit {:?} means the worker's stack died: {out}",
        o.status.code()
    );
    assert!(out.contains("1000"), "the behavior must compute its answer: {out}");
    assert!(!out.contains("has overflowed its stack"), "the host must not crash: {out}");
}

/// THE OTHER HALF: the bound must still BITE on a worker. A fix that only enlarged the stack would
/// pass the test above while pushing the crash out to a deeper recursion instead of removing it.
#[test]
fn recursion_past_the_bound_inside_an_actor_is_dl0905() {
    let home = scratch("actor-recursion-bound");
    let src = home.join("deep_actor.delulu");
    std::fs::write(&src, ACTOR_RECURSION.replace("DEPTH", "60000")).unwrap();
    let o = delulu(&home, &["run", src.to_str().unwrap(), "--grant", "console"]);
    let out = text(&o);
    assert!(out.contains("DL0905"), "the guard must fire on a worker thread too, got: {out}");
    assert!(!out.contains("has overflowed its stack"), "the host must not crash: {out}");
    assert!(
        o.status.code().is_some(),
        "an aborted process yields no exit code at all; a diagnosed fault always does"
    );
}

/// `login` stores a scoped registry token — and never echoes it. A credential printed to a
/// terminal ends up in a scrollback buffer, a screen recording, and a CI log.
#[test]
fn login_stores_a_token_without_ever_echoing_it() {
    let home = scratch("login");
    const SECRET: &str = "dlt_this_value_must_never_be_printed";

    let o = delulu(&home, &["login", "--registry", "http://127.0.0.1:9", "--token", SECRET]);
    let out = text(&o);
    assert!(o.status.success(), "login must succeed: {out}");
    assert!(
        !out.contains(SECRET),
        "the token must NEVER appear in output — it would land in scrollback and CI logs:\n{out}"
    );

    let creds = std::fs::read_to_string(home.join("credentials.jsonl")).expect("credentials stored");
    assert!(creds.contains(SECRET), "the token must actually be stored: {creds}");

    // Storing a second token for the same registry supersedes the first rather than accumulating
    // stale credentials that might be picked up later.
    let o2 = delulu(&home, &["login", "--registry", "http://127.0.0.1:9", "--token", "dlt_newer"]);
    assert!(o2.status.success());
    let creds2 = std::fs::read_to_string(home.join("credentials.jsonl")).unwrap();
    assert!(!creds2.contains(SECRET), "the superseded token must be gone: {creds2}");
    assert!(creds2.contains("dlt_newer"));
}

/// `login` without a token is refused — it cannot guess, and storing an empty credential would
/// fail confusingly later.
#[test]
fn login_without_a_token_is_refused() {
    let home = scratch("login-bare");
    let o = delulu(&home, &["login", "--registry", "http://127.0.0.1:9"]);
    assert!(!o.status.success(), "login needs a token");
    assert!(text(&o).contains("--token"), "the refusal must say what is missing: {}", text(&o));
}

/// `repl` accepts piped input and reports errors on bad input rather than dying. The REPL's
/// contract is that it *reports*, not that it exits nonzero — asserted honestly as such.
#[test]
fn repl_evaluates_piped_input_and_reports_errors() {
    use std::io::Write;
    use std::process::Stdio;

    let home = scratch("repl");
    let run = |input: &str| -> String {
        let mut child = Command::new(env!("CARGO_BIN_EXE_delulu"))
            .current_dir(workspace_root())
            .env("DELULU_HOME", &home)
            .env("DELULU_NO_FIRST_RUN", "1")
            .arg("repl")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn repl");
        child.stdin.as_mut().unwrap().write_all(input.as_bytes()).unwrap();
        let o = child.wait_with_output().expect("repl exits");
        format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
    };

    let ok = run("1 + 1\n:q\n");
    assert!(ok.contains('2'), "the REPL evaluates a valid expression: {ok}");

    let bad = run("this is not delulu ~~~\n:q\n");
    assert!(bad.contains("error"), "the REPL reports bad input rather than ignoring it: {bad}");
    assert!(!bad.contains("panicked at"), "the REPL must never panic on bad input: {bad}");
}

/// The effect list a user is shown must be the list the checker accepts.
///
/// It was not. Two error messages hand-wrote "Read, Write, Net, Clock, Rand, Declassify,
/// ForeignCall" while `CORE_EFFECT_NAMES` held ten — so a reader who mistyped an effect was told a
/// set that omitted `Load`, `Async` and `Actuate`, and could not discover them from the very error
/// that exists to teach them. The behaviour was always right; only the teaching was wrong, which is
/// the kind of defect no test catches unless it compares the message to the source.
#[test]
fn the_effect_list_shown_to_a_user_is_the_one_the_checker_accepts() {
    let home = scratch("effects");
    let o = delulu(&home, &["why", "NotAnEffect9a", "examples/greeter"]);
    let msg = text(&o);
    for name in delulu_check::check::CORE_EFFECT_NAMES {
        assert!(
            msg.contains(name),
            "`{name}` is a core effect the checker accepts, but the error listing the core effects \
             does not mention it:\n{msg}"
        );
    }
}

/// **No subcommand may silently ignore an option.**
///
/// Every command gets a flag that cannot exist. A command that exits **0** accepted an argument it
/// never understood, which means it reported success for work it did not do.
///
/// This is the twin of the positional rule (`refuse_extra_positionals`, D-phase 4) and it was
/// missing for the whole of 1.0. A sweep found **12 of 22 subcommands** ignoring flags outright —
/// `check`, `authority`, `why`, `atlas`, `explain`, `run`, `build`, `lock`, `test`, `secrets`,
/// `locale`, `morph`. `delulu check app.delulu --strict` printed `checked clean` and exited 0.
///
/// A person might catch that. **An agent assembling a command from a half-remembered flag name gets
/// a green light for work that never happened**, and this language's stated primary users are
/// agents. `audit` reading the wrong store (C75) was the same defect where the consequence was
/// worst; this is the class (C76, ruling D73).
#[test]
fn no_subcommand_silently_ignores_an_unknown_option() {
    let home = scratch("unknown-flags");
    let src = home.join("p.delulu");
    std::fs::write(&src, "module m\n\nfn main(root: Root) {\n}\n").unwrap();
    let p = src.to_str().unwrap();
    let pkg = home.join("pkg");
    let _ = delulu(&home, &["new", pkg.to_str().unwrap()]);
    let pkg = pkg.to_str().unwrap();

    // (subcommand, a minimally valid invocation) — the impossible flag is appended to each.
    let cases: Vec<(&str, Vec<&str>)> = vec![
        ("check", vec!["check", p]),
        ("fmt", vec!["fmt", "--check", p]),
        ("authority", vec!["authority", p]),
        ("why", vec!["why", "Write", p]),
        ("atlas", vec!["atlas", p]),
        ("explain", vec!["explain", "DL0703"]),
        ("run", vec!["run", p]),
        ("build", vec!["build", pkg]),
        ("lock", vec!["lock", pkg]),
        ("test", vec!["test", p]),
        ("fix", vec!["fix", p]),
        ("doctor", vec!["doctor", "--check"]),
        ("audit", vec!["audit", "tail", "1"]),
        ("secrets", vec!["secrets", "list"]),
        ("locale", vec!["locale", "list"]),
        ("morph", vec!["morph", "list"]),
        ("completions", vec!["completions", "bash"]),
    ];

    let mut ignored = Vec::new();
    for (name, argv) in cases {
        let mut args = argv.clone();
        args.push("--totally-not-a-real-flag-9z");
        let o = delulu(&home, &args);
        if o.status.success() {
            ignored.push(name);
        }
    }
    assert!(
        ignored.is_empty(),
        "these subcommands exited 0 with an option that cannot exist, meaning they dropped an \
         argument the caller typed and reported success anyway: {ignored:?}\n\
         Refuse it — `refuse_unknown_flags` for anything using `parse_opts`, `refuse_unlisted_flags` \
         for a command that scans its own argv."
    );
}

/// **A flag that needs a value and is given none must refuse, not fall back to the default.**
///
/// The same defect as the sweep above, one position over. Every value-taking arm read
/// `if i + 1 < rest.len() { take it }` with no `else`, so a flag in final position vanished and the
/// command ran on its default. Seven did it. The one that matters most:
///
/// ```text
/// delulu run app.delulu --grant console --isolation      → ran with NO isolation, exit 0
/// ```
///
/// A security-relevant setting, dropped in silence, reported as success (C76, ruling D73).
#[test]
fn a_value_taking_flag_with_no_value_is_refused() {
    let home = scratch("missing-values");
    let src = home.join("p.delulu");
    std::fs::write(
        &src,
        "module m\n\nfn main(root: Root) ! {Write} {\n    let out = root.console()\n    out.println(\"ran\")\n}\n",
    )
    .unwrap();
    let p = src.to_str().unwrap();

    let cases: Vec<Vec<&str>> = vec![
        vec!["run", p, "--grant"],
        vec!["run", p, "--grant", "console", "--engine"],
        vec!["run", p, "--grant", "console", "--isolation"],
        vec!["run", p, "--grant", "console", "--seed"],
        vec!["run", p, "--grant", "console", "--lease"],
        vec!["authority", p, "--diff"],
        vec!["atlas", p, "--format"],
        vec!["atlas", p, "--out"],
    ];

    let mut dropped = Vec::new();
    for argv in &cases {
        let o = delulu(&home, argv);
        if o.status.success() {
            dropped.push(argv.join(" "));
        }
    }
    assert!(
        dropped.is_empty(),
        "these ran successfully with a value-taking option that had no value, which means the \
         option was discarded and the command used its default instead: {dropped:?}"
    );
}

/// Every error the CLI prints starts with `error:`.
///
/// This is not cosmetics. The prefix is how a person scanning a terminal finds the line that
/// matters, and how anything parsing stderr — a CI log scraper, an agent harness, a problem matcher
/// in an editor — separates a failure from progress chatter. The extension's own problem matcher
/// keys on it.
///
/// It had drifted: 147 sites used the prefix and eleven did not, so `delulu fmt <missing>` said
/// *"no such file or directory: …"* while `delulu check <missing>` said *"error: cannot read …"*
/// for the identical condition. Nothing failed, because nothing compared them — which is what makes
/// a convention held only by habit worth converting into a test.
#[test]
fn every_user_facing_error_line_is_prefixed_so_it_can_be_found() {
    let src = std::fs::read_to_string(workspace_root().join("crates/delulu/src/cli.rs"))
        .expect("cli.rs is readable");

    // Prefixes that mark a line's role. `error:`/`warning:`/`note:`/`hint:` are the diagnostic
    // vocabulary; the rest are usage or a rendered diagnostic that carries its own.
    const KNOWN: [&str; 8] =
        ["error", "warning", "note", "hint", "usage", "ok", "delulu", "  "];

    // The check is deliberately scoped to lines that READ as failures, not to every `eprintln!`.
    // stderr legitimately carries progress here too — `wrote …`, `fingerprint: …`, `expires: …` —
    // and demanding a role prefix on those would be a rule about noise rather than about
    // findability. The first version of this test did exactly that, flagged eleven innocent
    // progress lines alongside three real misses, and would have been "fixed" by prefixing
    // everything, which destroys the signal the prefix carries.
    const FAILURE_SHAPED: [&str; 12] = [
        "cannot",
        "could not",
        "unable",
        "no such",
        "not found",
        "unknown",
        "failed",
        "invalid",
        "missing",
        "refus",
        "violated",
        "no command",
    ];

    let mut unprefixed = Vec::new();
    for (i, line) in src.lines().enumerate() {
        let t = line.trim_start();
        if t.starts_with("//") {
            continue;
        }
        let Some(rest) = t.strip_prefix("eprintln!(\"") else { continue };
        // A bare interpolation (`eprintln!("{msg}")`) forwards a string built elsewhere, which
        // carries its own prefix; judging it here would be guessing at a value this test cannot see.
        if rest.starts_with('{') || rest.starts_with('"') {
            continue;
        }
        let lower = rest.to_lowercase();
        if !FAILURE_SHAPED.iter().any(|w| lower.starts_with(w)) {
            continue;
        }
        if !KNOWN.iter().any(|p| rest.starts_with(p)) {
            unprefixed.push(format!("cli.rs:{}: {}", i + 1, t.chars().take(90).collect::<String>()));
        }
    }

    assert!(
        unprefixed.is_empty(),
        "these stderr lines carry no role prefix, so neither a person nor a log parser can tell \
         them from ordinary output — start them with `error:` (or `warning:`/`note:`/`hint:`):\n  {}",
        unprefixed.join("\n  ")
    );
}

/// The options a piece of help text names: every `--word` (not inside a word) and `-o`, which is
/// the short spelling of `--out`.
fn flags_named_in(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let b = text.as_bytes();
    for (i, _) in text.match_indices('-') {
        if i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'-') {
            continue;
        }
        let tail = &text[i..];
        let name = if let Some(rest) = tail.strip_prefix("--") {
            let body: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-').collect();
            if body.is_empty() || !body.starts_with(|c: char| c.is_ascii_lowercase()) {
                continue;
            }
            format!("--{body}")
        } else if tail.starts_with("-o") && !tail[2..].starts_with(|c: char| c.is_ascii_alphanumeric()) {
            "--out".to_string()
        } else {
            continue;
        };
        if !out.contains(&name) {
            out.push(name);
        }
    }
    out
}

/// P1-F3: a flag the shared option parser knows is refused — exit 2, the unknown-option message —
/// by every command whose `--help` does not document it. `check x.delulu --grants` and
/// `check x --diff foo` exited 0 having ignored the flag, because a flag the parser recognised was
/// never "unknown", and commands with their own parsers never looked at such a flag at all.
///
/// Nothing here is a hand-kept list: the shared flags are read from `parse_opts` itself, the
/// commands from `SUBCOMMANDS`, and what each command accepts from its own `--help`.
#[test]
fn every_command_refuses_a_shared_flag_its_help_does_not_document() {
    let src = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("cli.rs"))
        .expect("the CLI source is readable");
    let parser = src
        .split_once("pub(crate) fn parse_opts(")
        .and_then(|(_, t)| t.split_once("\npub(crate) fn refuse_extra_positionals"))
        .map(|(p, _)| p)
        .expect("parse_opts must be present");
    // (flag, takes a value)
    let mut shared: Vec<(String, bool)> = Vec::new();
    for lit in parser.split('"').skip(1).step_by(2) {
        let name = if lit == "-o" { "--out".to_string() } else { lit.to_string() };
        if !(name.starts_with("--") && name.len() > 2 && name[2..].chars().all(|c| c.is_ascii_lowercase() || c == '-')) {
            continue;
        }
        let takes_value = parser.contains(&format!("missing_values.push(\"{name}\""));
        if !shared.iter().any(|(n, _)| *n == name) {
            shared.push((name, takes_value));
        }
    }
    assert!(shared.len() >= 35, "the shared-flag scan broke, and a sweep of nothing passes: {shared:?}");
    let commands: Vec<String> = src
        .split_once("pub(crate) const SUBCOMMANDS: &[&str] = &[")
        .and_then(|(_, t)| t.split_once("];"))
        .map(|(l, _)| l.split('"').skip(1).step_by(2).map(str::to_string).collect())
        .expect("SUBCOMMANDS must be present");
    assert!(commands.len() >= 30, "the command scan broke: {commands:?}");

    let home = scratch("f3-sweep");
    let bad = std::sync::Mutex::new(Vec::<String>::new());
    let swept = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|s| {
        for cmd in &commands {
            let (home, shared, bad, swept) = (&home, &shared, &bad, &swept);
            s.spawn(move || {
                let help = delulu(home, &[cmd, "--help"]);
                assert_eq!(help.status.code(), Some(0), "`{cmd} --help` must answer");
                let documented = flags_named_in(&String::from_utf8_lossy(&help.stdout));
                for (flag, takes_value) in shared {
                    if documented.contains(flag) || (cmd == "completions" && flag == "--json") {
                        continue;
                    }
                    let mut argv: Vec<&str> = vec![cmd, flag];
                    if *takes_value {
                        argv.push("v");
                    }
                    let o = Command::new(env!("CARGO_BIN_EXE_delulu"))
                        .current_dir(home)
                        .env("DELULU_HOME", home)
                        .env("DELULU_STATE_DIR", home)
                        .env("DELULU_NO_FIRST_RUN", "1")
                        .args(&argv)
                        .stdin(std::process::Stdio::null())
                        .output()
                        .expect("failed to run delulu");
                    swept.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let err = String::from_utf8_lossy(&o.stderr);
                    if o.status.code() != Some(2) || !err.contains("does not know") || !err.contains(flag.as_str()) {
                        bad.lock().unwrap().push(format!(
                            "`delulu {}` exited {:?} without refusing {flag}: {}",
                            argv.join(" "),
                            o.status.code(),
                            err.lines().next().unwrap_or("")
                        ));
                    }
                }
            });
        }
    });
    let bad = bad.into_inner().unwrap();
    let swept = swept.into_inner();
    let _ = std::fs::remove_dir_all(&home);
    assert!(swept > 500, "only {swept} refusals were exercised — the sweep lost its reach");
    assert!(bad.is_empty(), "{} undocumented shared flag(s) were not refused:\n  {}", bad.len(), bad.join("\n  "));
}

/// The two invocations P1 recorded, with a real file, as the brief names them.
#[test]
fn check_refuses_grants_and_diff_but_keeps_its_documented_flags() {
    let home = scratch("f3-check");
    for extra in [&["--grants"][..], &["--diff", "foo"][..]] {
        let mut argv = vec!["check", DEMO];
        argv.extend_from_slice(extra);
        let o = delulu(&home, &argv);
        assert_eq!(o.status.code(), Some(2), "`{}` must be refused: {}", argv.join(" "), text(&o));
        assert!(text(&o).contains("does not know this option"), "{}", text(&o));
    }
    // What `check --help` documents still works.
    let o = delulu(&home, &["check", DEMO, "--json"]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    let _ = std::fs::remove_dir_all(&home);
}
