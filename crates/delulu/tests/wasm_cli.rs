//! End-to-end tests for `delulu run --engine wasm`: the WASM sandbox floor reached from the
//! terminal. A console `main` compiles to WebAssembly and runs under Wasmtime with the capability
//! checked host-side; its output must match the interpreter (the reference engine).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// A unique temp path for a `.dwx` built by a test (parallel tests must not collide).
fn temp_dwx(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("delulu_test_{}_{}.dwx", std::process::id(), tag))
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu")).current_dir(workspace_root()).args(args).output().expect("failed to run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

#[test]
fn run_engine_wasm_prints_console_output() {
    let o = delulu(&["run", "examples/hello_wasm.delulu", "--engine", "wasm", "--grant", "console"]);
    assert!(o.status.success(), "wasm run should succeed: {}", String::from_utf8_lossy(&o.stderr));
    assert!(stdout(&o).contains("hello from the wasm engine"), "{}", stdout(&o));
}

/// The book's guide chapters that the WASM engine runs since 2026-10-20 (RW 3.4), end to end through the command line:
/// `01` (records, `Float`, sums with payloads), `02` (a list begun empty, typed by the checker — what `compile_checked` is
/// for), `03` (`parse_int`), `04` (a function named as a value, `.trim`) — each the interpreter's output, byte for byte.
#[test]
fn the_guide_chapters_the_wasm_engine_runs_match_the_interpreter() {
    for chapter in [
        "examples/guide/01_types.delulu",
        "examples/guide/02_option_and_lists.delulu",
        "examples/guide/03_errors.delulu",
        "examples/guide/04_effects.delulu",
    ] {
        let w = delulu(&["run", chapter, "--engine", "wasm", "--grant", "console"]);
        let i = delulu(&["run", chapter, "--grant", "console"]);
        assert!(w.status.success(), "{chapter} on the WASM engine: {}", String::from_utf8_lossy(&w.stderr));
        assert!(i.status.success(), "{chapter} on the interpreter: {}", String::from_utf8_lossy(&i.stderr));
        assert!(!stdout(&i).is_empty(), "{chapter} prints");
        assert_eq!(stdout(&w), stdout(&i), "{chapter}: WASM and interpreter output must be identical");
    }
}

#[test]
fn run_engine_wasm_matches_the_interpreter() {
    let w = delulu(&["run", "examples/hello_wasm.delulu", "--engine", "wasm", "--grant", "console"]);
    let i = delulu(&["run", "examples/hello_wasm.delulu", "--grant", "console"]);
    assert!(w.status.success() && i.status.success());
    assert_eq!(stdout(&w), stdout(&i), "WASM and interpreter output must be identical");
}

/// WASM-OUTPUT-1: the CLI printed the WASM engine's captured output only when the run succeeded, so a program that
/// printed and then faulted showed nothing on stdout, where the interpreter shows every line it printed. The fault's
/// code is the same on both engines.
#[test]
fn what_a_program_printed_before_a_fault_reaches_stdout_on_both_engines() {
    let path = std::env::temp_dir().join(format!("delulu_printfault_{}.delulu", std::process::id()));
    std::fs::write(
        &path,
        "module m\nfn main(root: Root) ! {Write} {\n  let out = root.console()\n  out.println(\"first\")\n  let z = 0\n  \
         out.println(\"second\")\n  out.println(str(10 / z))\n  out.println(\"never\")\n}\n",
    )
    .expect("write");
    let p = path.to_string_lossy().to_string();
    let w = delulu(&["run", &p, "--engine", "wasm", "--grant", "console"]);
    let i = delulu(&["run", &p, "--grant", "console"]);
    let _ = std::fs::remove_file(&path);
    assert!(!w.status.success() && !i.status.success(), "both fault");
    assert_eq!(stdout(&i), "first\nsecond\n", "the interpreter");
    assert_eq!(stdout(&w), stdout(&i), "the WASM engine prints what the interpreter printed before the fault");
    let (we, ie) = (String::from_utf8_lossy(&w.stderr), String::from_utf8_lossy(&i.stderr));
    assert!(we.contains("DL0902") && ie.contains("DL0902"), "a division by zero on both:\n{we}\n{ie}");
}

#[test]
fn run_engine_wasm_ungranted_console_is_refused() {
    // No console grant → root.console() is refused host-side; the run fails.
    let o = delulu(&["run", "examples/hello_wasm.delulu", "--engine", "wasm"]);
    assert!(!o.status.success(), "an ungranted console must be refused on the wasm engine");
}

#[test]
fn run_engine_wasm_on_unsupported_program_is_dl1201() {
    // A list of lists is outside the WASM fragment, so `--engine wasm` reports DL1201 and the program stays on the
    // interpreter rather than being misrun. (A `while` loop was the example until RW 3.4 compiled it, then a `for` over
    // a list until the WASM `List` compiled that.)
    let path = std::env::temp_dir().join(format!("delulu_for_{}.delulu", std::process::id()));
    std::fs::write(&path, "module m\nfn main(root: Root) ! {Write} { let out = root.console()\n var t = 0\n for x in [[1], [2]] { t = t + x.len() }\n out.println(\"done\") }\n").expect("write");
    let p = path.to_string_lossy().to_string();
    let o = delulu(&["run", &p, "--engine", "wasm", "--grant", "console", "--json"]);
    assert!(stdout(&o).contains("DL1201"), "expected DL1201: {}", stdout(&o));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn run_engine_wasm_on_demo_reports_dl1205_secret() {
    // With generics/fs/Result/match all compiling now, demo.delulu gets all the way to its
    // `root.secret("API_KEY")` line — which is DL1205 (secrets never enter a guest), keeping the
    // program on the interpreter. (Progress marker: everything BEFORE the secret compiles.)
    let o = delulu(&["run", "examples/demo.delulu", "--engine", "wasm", "--grant", "console", "--json"]);
    assert!(stdout(&o).contains("DL1205"), "expected DL1205: {}", stdout(&o));
    assert_eq!(o.status.code(), Some(1));
}

// ----- Phase 3g: the .dwx authority-carrying artifact ---------------------------------------

#[test]
fn build_dwx_then_run_matches_the_interpreter() {
    // build --target wasm produces a self-describing artifact; running it must be byte-identical to
    // running the source on the interpreter (the reference engine).
    let dwx = temp_dwx("roundtrip");
    let dwx_s = dwx.to_string_lossy().to_string();
    let b = delulu(&["build", "examples/hello_wasm.delulu", "--target", "wasm", "-o", &dwx_s]);
    assert!(b.status.success(), "build should succeed: {}", String::from_utf8_lossy(&b.stderr));
    assert!(dwx.exists(), "the .dwx artifact must be written");

    let run = delulu(&["run", &dwx_s, "--grant", "console"]);
    assert!(run.status.success(), "running the .dwx should succeed: {}", String::from_utf8_lossy(&run.stderr));

    let interp = delulu(&["run", "examples/hello_wasm.delulu", "--grant", "console"]);
    assert_eq!(stdout(&run), stdout(&interp), "artifact output must match the interpreter");
    let _ = std::fs::remove_file(&dwx);
}

#[test]
fn dwx_run_without_console_grant_is_refused() {
    let dwx = temp_dwx("nogrant");
    let dwx_s = dwx.to_string_lossy().to_string();
    assert!(delulu(&["build", "examples/hello_wasm.delulu", "--target", "wasm", "-o", &dwx_s]).status.success());
    let o = delulu(&["run", &dwx_s]);
    assert!(!o.status.success(), "an ungranted console must be refused when running the artifact");
    let _ = std::fs::remove_file(&dwx);
}

#[test]
fn tampered_dwx_is_rejected_dl1202() {
    // Alter a byte of the artifact after it's built: re-verification catches the code/manifest
    // mismatch before the program ever runs.
    let dwx = temp_dwx("tampered");
    let dwx_s = dwx.to_string_lossy().to_string();
    assert!(delulu(&["build", "examples/hello_wasm.delulu", "--target", "wasm", "-o", &dwx_s]).status.success());

    let mut bytes = std::fs::read(&dwx).expect("read dwx");
    bytes[20] ^= 0xff; // a code-region byte (well before the trailing authority section)
    std::fs::write(&dwx, &bytes).expect("write tampered dwx");

    let o = delulu(&["run", &dwx_s, "--grant", "console", "--json"]);
    assert!(stdout(&o).contains("DL1202"), "tampered artifact must be DL1202: {}", stdout(&o));
    assert_eq!(o.status.code(), Some(1));
    let _ = std::fs::remove_file(&dwx);
}

#[test]
fn run_engine_wasm_on_a_secret_program_is_dl1205() {
    // A program that mints/exposes a secret must not compile to WASM — the CLI reports DL1205 and
    // runs nothing, so secret bytes never reach a guest. (Omitting --engine wasm runs it on the interp.)
    let path = std::env::temp_dir().join(format!("delulu_secret_{}.delulu", std::process::id()));
    std::fs::write(
        &path,
        "module m\nfn main(root: Root) ! {Declassify} { let key = root.secret(\"TOKEN\")\n let d = root.declassify()\n let _r = key.expose(d) }\n",
    )
    .expect("write secret program");
    let p = path.to_string_lossy().to_string();
    let o = delulu(&["run", &p, "--engine", "wasm", "--json"]);
    assert!(stdout(&o).contains("DL1205"), "expected DL1205: {}", stdout(&o));
    assert_eq!(o.status.code(), Some(1));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn non_artifact_dwx_is_rejected_dl1202() {
    // A .dwx that isn't a Delulu artifact at all (random bytes) is refused, not run.
    let dwx = temp_dwx("garbage");
    std::fs::write(&dwx, b"this is not a wasm module").expect("write garbage");
    let dwx_s = dwx.to_string_lossy().to_string();
    let o = delulu(&["run", &dwx_s, "--json"]);
    assert!(stdout(&o).contains("DL1202"), "a non-artifact .dwx must be DL1202: {}", stdout(&o));
    assert_eq!(o.status.code(), Some(1));
    let _ = std::fs::remove_file(&dwx);
}
