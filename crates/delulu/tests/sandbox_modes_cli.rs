//! PS-A-10: the transition matrix of `V2_SECURITY_MODEL.md` §5, as tests against the real binary.
//!
//! The matrix is not a list of features; it is a list of ways a boundary can be moved. Each row here
//! is one of them, and each asks the same two questions: does the intended transition WORK, and does
//! the unintended one REFUSE rather than happen quietly? A transition that happens silently is worse
//! than one that fails, because a silent one is indistinguishable from a boundary that held.
//!
//! What is deliberately absent is a row too: there is no language surface for a program to change
//! its own mode or level, and one test asserts that absence instead of assuming it.

use std::process::{Command, Output};

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        // Never the developer's own chain: parallel test processes sharing `~/.delulu` corrupted it
        // once already, which is how the append lock in `guest.rs` came to exist.
        .env("DELULU_STATE_DIR", isolated_state())
        .output()
        .expect("the delulu binary runs")
}

fn tmp(name: &str) -> std::path::PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let d = std::env::temp_dir().join(format!("delulu-mode-{name}-{}-{t}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("out")).expect("a temp directory");
    d
}

/// A program that asks for one write into `dir/out`, so a run that performed effects is visible as a
/// FILE rather than as a claim on standard output.
fn writer(dir: &std::path::Path) -> (std::path::PathBuf, String) {
    let scope = dir.join("out").display().to_string().replace('\\', "/");
    let src = dir.join("p.delulu");
    std::fs::write(
        &src,
        format!(
            "module g\n\nfn main(root: Root) ! {{Write}} {{\n    \
             let w = root.fs_write(\"{scope}\")\n    \
             let _ = w.write_text(\"made.txt\", \"performed\")\n}}\n"
        ),
    )
    .unwrap();
    (src, scope)
}

fn out(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

// ---------------------------------------------------------------------------------------------
// ON to OFF, and OFF to ON
// ---------------------------------------------------------------------------------------------

/// Both directions work when they are SAID. The point of the pair is that each is a separate,
/// deliberate sentence, and the run report says which one was written.
#[test]
fn each_direction_works_when_it_is_asked_for_once() {
    let dir = tmp("both");
    let (src, scope) = writer(&dir);
    let rep = dir.join("on.json");

    let on = delulu(&[
        "run",
        src.to_str().unwrap(),
        "--sandbox",
        "--grant",
        &format!("fs.write={scope}"),
        "--report-out",
        rep.to_str().unwrap(),
    ]);
    assert_eq!(on.status.code(), Some(0), "{}", out(&on));
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&rep).expect("the run report")).unwrap();
    assert_eq!(report["sandbox"]["level"], 1, "the on run must report a confined level: {report}");

    let _ = std::fs::remove_file(dir.join("out").join("made.txt"));
    let off = delulu(&["run", src.to_str().unwrap(), "--sandbox=off", "--grant", &format!("fs.write={scope}")]);
    assert_eq!(off.status.code(), Some(0), "{}", out(&off));
    assert!(dir.join("out").join("made.txt").exists(), "the off run still performs the effect");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The transition that must NOT happen quietly. `--sandbox --sandbox=off` used to be resolved by
/// argument order, so a wrapper's default and a caller's appended argument decided the boundary
/// between them and neither of them knew. Now the command line is refused and nothing runs — in BOTH
/// orders, because a rule that only holds for one spelling is the P22 shape all over again.
#[test]
fn a_command_line_that_asks_for_both_is_refused_in_either_order() {
    let dir = tmp("contradict");
    let (src, scope) = writer(&dir);
    let grant = format!("fs.write={scope}");
    for args in [
        vec!["run", src.to_str().unwrap(), "--sandbox", "--sandbox=off", "--grant", &grant],
        vec!["run", src.to_str().unwrap(), "--sandbox=off", "--sandbox", "--grant", &grant],
    ] {
        let o = delulu(&args);
        let text = out(&o);
        assert_eq!(o.status.code(), Some(2), "{text}");
        assert!(text.contains("different answers"), "the refusal must say what is contradictory: {text}");
        assert!(
            !dir.join("out").join("made.txt").exists(),
            "a refused command line must perform nothing: {text}"
        );
    }
    // Falsification: the SAME answer twice is not a contradiction, so this must still run. Without
    // it the test above would pass for a rule that merely forbade repeating the flag.
    let twice = delulu(&["run", src.to_str().unwrap(), "--sandbox=off", "--sandbox=off", "--grant", &grant]);
    assert_eq!(twice.status.code(), Some(0), "{}", out(&twice));
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------------------------
// strict and audit, in both directions
// ---------------------------------------------------------------------------------------------

/// AUDIT performs nothing and says what a run would need; STRICT then performs it. The same program,
/// the same grant, the same command but one word: the difference is visible as a file on disk.
#[test]
fn audit_performs_nothing_and_strict_performs_it() {
    let dir = tmp("audit");
    let (src, scope) = writer(&dir);
    let grant = format!("fs.write={scope}");
    let made = dir.join("out").join("made.txt");

    let audit = delulu(&["run", src.to_str().unwrap(), "--sandbox", "--mode", "audit", "--grant", &grant]);
    assert_eq!(audit.status.code(), Some(0), "{}", out(&audit));
    assert!(!made.exists(), "an audit run performed an effect: {}", out(&audit));
    let report: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&audit.stdout)).expect("the audit report is JSON");
    assert_eq!(report["sandbox"]["mode"], "audit", "{report}");
    assert_eq!(report["outcome"]["ran"], false, "{report}");
    assert!(
        report["audit"]["required_grants"].as_array().is_some_and(|g| !g.is_empty()),
        "an audit that names no required authority has told its reader nothing: {report}"
    );

    // And back: naming `strict` explicitly must mean exactly the default, and perform the run.
    let strict = delulu(&["run", src.to_str().unwrap(), "--sandbox", "--mode", "strict", "--grant", &grant]);
    assert_eq!(strict.status.code(), Some(0), "{}", out(&strict));
    assert!(made.exists(), "the strict run did not perform the effect: {}", out(&strict));
    let _ = std::fs::remove_dir_all(&dir);
}

/// An unknown mode is a REFUSAL, never a fallback to the default. A misspelling that silently became
/// `strict` would be harmless; one that silently became `audit` would make a run that performed
/// nothing look exactly like a run that succeeded.
#[test]
fn an_unknown_mode_refuses_rather_than_falling_back() {
    let dir = tmp("badmode");
    let (src, scope) = writer(&dir);
    let o = delulu(&[
        "run",
        src.to_str().unwrap(),
        "--sandbox",
        "--mode",
        "audit-ish",
        "--grant",
        &format!("fs.write={scope}"),
    ]);
    assert_eq!(o.status.code(), Some(2), "{}", out(&o));
    assert!(out(&o).contains("is not a mode this command knows"), "{}", out(&o));
    assert!(!dir.join("out").join("made.txt").exists(), "nothing ran");
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------------------------
// A transition that could not be applied at all
// ---------------------------------------------------------------------------------------------

/// The flags that only mean something inside a sandbox are refused outside one. This is the failure
/// class the whole phase exists to prevent, in its cheapest form: a flag nobody reads looks exactly
/// like a flag that was applied, and `--mode audit` without `--sandbox` used to PERFORM THE RUN while
/// the caller believed nothing would happen.
#[test]
fn a_sandbox_flag_without_a_sandbox_is_refused_not_ignored() {
    let dir = tmp("ignored");
    let (src, scope) = writer(&dir);
    let grant = format!("fs.write={scope}");
    let made = dir.join("out").join("made.txt");
    for flag in [
        vec!["--mode", "audit"],
        vec!["--mode", "strict"],
        vec!["--sandbox-profile", "hostile-agent"],
        vec!["--limits", "mem=64000000"],
    ] {
        let _ = std::fs::remove_file(&made);
        let mut args = vec!["run", src.to_str().unwrap(), "--grant", &grant];
        args.extend(flag.iter().copied());
        let o = delulu(&args);
        let text = out(&o);
        assert_eq!(o.status.code(), Some(2), "`{flag:?}` was accepted outside a sandbox: {text}");
        assert!(text.contains("describes a sandboxed run"), "{text}");
        assert!(!made.exists(), "`{flag:?}` was ignored and the run happened anyway: {text}");

        // The same flags, with `--sandbox=off` said out loud, are still refused: `off` is not a
        // sandbox either, and a profile for a run that has none is a claim about nothing.
        let mut args = vec!["run", src.to_str().unwrap(), "--sandbox=off", "--grant", &grant];
        args.extend(flag.iter().copied());
        let o = delulu(&args);
        assert_eq!(o.status.code(), Some(2), "`{flag:?}` was accepted with `--sandbox=off`: {}", out(&o));
    }
    // Falsification: every one of those flags is accepted WITH a sandbox, so the refusals above are
    // about the missing sandbox and not about the flags being unknown.
    let ok = delulu(&[
        "run",
        src.to_str().unwrap(),
        "--sandbox",
        "--sandbox-profile",
        "hostile-agent",
        "--limits",
        "mem=64000000",
        "--mode",
        "audit",
        "--grant",
        &grant,
    ]);
    assert_eq!(ok.status.code(), Some(0), "{}", out(&ok));
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------------------------
// A program cannot move its own boundary
// ---------------------------------------------------------------------------------------------

/// There is no transition DURING execution, because there is no surface for one. A program reaching
/// for a name that would change its own mode or level is refused by the CHECKER, before anything
/// runs, and the refusal is the ordinary "no such method" — which is the strongest form this rule can
/// take: not a guard that could be walked past, but nothing to walk past.
#[test]
fn a_program_cannot_relax_its_own_sandbox_because_there_is_nothing_to_call() {
    let dir = tmp("selfrelax");
    for method in ["sandbox_off", "set_mode", "break_glass", "escape"] {
        let src = dir.join(format!("{method}.delulu"));
        std::fs::write(&src, format!("module g\n\nfn main(root: Root) {{\n    let _ = root.{method}()\n}}\n"))
            .unwrap();
        let o = delulu(&["check", src.to_str().unwrap()]);
        assert_ne!(o.status.code(), Some(0), "`root.{method}()` checked clean: {}", out(&o));
        // And under a sandbox it is refused too, having run nothing.
        let r = delulu(&["run", src.to_str().unwrap(), "--sandbox"]);
        assert_ne!(r.status.code(), Some(0), "`root.{method}()` ran under a sandbox: {}", out(&r));
    }
    // Falsification: a method that DOES exist checks clean, so the refusals above are about these
    // names being absent rather than about calls on `root` failing in general.
    let real = dir.join("real.delulu");
    std::fs::write(&real, "module g\n\nfn main(root: Root) ! {Write} {\n    let _ = root.fs_write(\"out\")\n}\n")
        .unwrap();
    let o = delulu(&["check", real.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    let _ = std::fs::remove_dir_all(&dir);
}

/// After a delegation the guest is no better off. A capability inside a sandbox is an opaque handle
/// minted by the host, so a program cannot reach a scope the operator did not grant — the refusal
/// comes from the host's own check, exactly as it would for an unsandboxed run.
#[test]
fn a_scope_that_was_never_granted_is_still_refused_inside_the_sandbox() {
    let dir = tmp("delegate");
    let scope = dir.join("out").display().to_string().replace('\\', "/");
    let sibling = dir.join("elsewhere").display().to_string().replace('\\', "/");
    std::fs::create_dir_all(dir.join("elsewhere")).unwrap();
    let src = dir.join("p.delulu");
    std::fs::write(
        &src,
        format!(
            "module g\n\nfn main(root: Root) ! {{Write}} {{\n    \
             let w = root.fs_write(\"{sibling}\")\n    \
             let _ = w.write_text(\"leaked.txt\", \"out of scope\")\n}}\n"
        ),
    )
    .unwrap();
    let o = delulu(&["run", src.to_str().unwrap(), "--sandbox", "--grant", &format!("fs.write={scope}")]);
    assert_ne!(o.status.code(), Some(0), "a scope that was never granted was reached: {}", out(&o));
    assert!(
        !dir.join("elsewhere").join("leaked.txt").exists(),
        "a file appeared outside the granted scope: {}",
        out(&o)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// A state directory of this test binary's own, so a run never touches the developer's real one.
fn isolated_state() -> std::path::PathBuf {
    static DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let d = std::env::temp_dir().join(format!("delulu-modestate-{}-{t}", std::process::id()));
        let _ = std::fs::create_dir_all(d.join("audit"));
        d
    })
    .clone()
}
