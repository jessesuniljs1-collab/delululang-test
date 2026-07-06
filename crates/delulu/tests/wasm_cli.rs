//! End-to-end tests for `delulu run --engine wasm`: the WASM sandbox floor reached from the
//! terminal. A console `main` compiles to WebAssembly and runs under Wasmtime with the capability
//! checked host-side; its output must match the interpreter (the reference engine).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
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
    // demo.delulu uses str()/string-concat/fs/match/secret — not compilable by the WASM backend,
    // so it stays on the interpreter and `--engine wasm` reports DL1201 rather than misrunning it.
    let o = delulu(&["run", "examples/demo.delulu", "--engine", "wasm", "--grant", "console", "--json"]);
    assert!(stdout(&o).contains("DL1201"), "expected DL1201: {}", stdout(&o));
    assert_eq!(o.status.code(), Some(1));
}
