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
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(workspace_root())
        .args(args)
        .output()
        .expect("failed to run delulu")
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

#[test]
fn run_engine_wasm_matches_the_interpreter() {
    let w = delulu(&["run", "examples/hello_wasm.delulu", "--engine", "wasm", "--grant", "console"]);
    let i = delulu(&["run", "examples/hello_wasm.delulu", "--grant", "console"]);
    assert!(w.status.success() && i.status.success());
    assert_eq!(stdout(&w), stdout(&i), "WASM and interpreter output must be identical");
}

#[test]
fn run_engine_wasm_ungranted_console_is_refused() {
    // No console grant → root.console() is refused host-side; the run fails.
    let o = delulu(&["run", "examples/hello_wasm.delulu", "--engine", "wasm"]);
    assert!(!o.status.success(), "an ungranted console must be refused on the wasm engine");
}

#[test]
fn run_engine_wasm_on_unsupported_program_is_dl1201() {
    // A `while` loop is outside the WASM fragment, so `--engine wasm` reports DL1201 and the program
    // stays on the interpreter rather than being misrun.
    let path = std::env::temp_dir().join(format!("delulu_while_{}.delulu", std::process::id()));
    std::fs::write(&path, "module m\nfn main(root: Root) ! {Write} { let out = root.console()\n let x = 0\n while x < 0 { }\n out.println(\"done\") }\n").expect("write");
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
