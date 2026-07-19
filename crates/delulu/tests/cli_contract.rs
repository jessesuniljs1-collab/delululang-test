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
