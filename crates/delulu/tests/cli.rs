//! End-to-end CLI tests: they run the real `delulu` binary and assert on both the human surface
//! and the agent surface (JSON diagnostics, typed repairs, the authority report).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

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

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

#[test]
fn authority_human_reports_what_the_program_can_do() {
    let o = delulu(&["authority", "examples/demo.delulu"]);
    assert!(o.status.success(), "authority should succeed");
    let out = stdout(&o);
    assert!(out.contains("Authority of `demo`"), "{out}");
    assert!(out.contains("Read, Write"), "effects missing: {out}");
    assert!(out.contains("API_KEY"), "secret missing: {out}");
    assert!(out.contains("apply") && out.contains("fib"), "pure fns missing: {out}");
}

#[test]
fn authority_json_has_the_stable_shape() {
    let o = delulu(&["authority", "examples/demo.delulu", "--json"]);
    let v: Value = serde_json::from_str(&stdout(&o)).expect("authority --json must be valid JSON");
    assert_eq!(v["schema"], 1);
    assert_eq!(v["command"], "authority");
    let effects: Vec<&str> = v["authority"]["effects"].as_array().unwrap().iter().map(|x| x.as_str().unwrap()).collect();
    assert!(effects.contains(&"Read") && effects.contains(&"Write"), "{effects:?}");
    assert_eq!(v["authority"]["secrets"][0], "API_KEY");
    assert_eq!(v["authority"]["foreign_calls"].as_array().unwrap().len(), 0);
}

#[test]
fn check_json_emits_code_span_and_authority_flagged_repair() {
    let o = delulu(&["check", "tests/conformance/reject/DL0501_undeclared_effect.delulu", "--json"]);
    let v: Value = serde_json::from_str(&stdout(&o)).expect("check --json must be valid JSON");
    let diag = &v["diagnostics"][0];
    assert_eq!(diag["code"], "DL0501");
    assert_eq!(diag["explanation_id"], "E-DL0501");
    assert!(diag["spans"][0]["start"]["byte"].is_number());
    let repair = &diag["repairs"][0];
    assert_eq!(repair["id"], "add_effect_to_row");
    assert_eq!(repair["authority_widening"], true);
    // Exit code 1 signals diagnostics (part of the stable contract).
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn check_clean_program_exits_zero() {
    let o = delulu(&["check", "examples/demo.delulu"]);
    assert!(o.status.success());
}

#[test]
fn run_executes_the_demo() {
    let o = delulu(&[
        "run",
        "examples/demo.delulu",
        "--grant",
        "console",
        "--grant",
        "fs.read=./config",
        "--grant",
        "secret:API_KEY=demo-key",
    ]);
    let out = stdout(&o);
    assert!(out.contains("hello, delulu world"), "{out}");
    assert!(out.contains("fib(10) = 55"), "{out}");
    assert!(out.contains("apply = 42"), "{out}");
    assert!(o.status.success());
}

#[test]
fn run_without_grant_faults_dl0703() {
    // Well-typed, but no console granted: the runtime refuses (defense in depth).
    let o = delulu(&["run", "examples/demo.delulu", "--json"]);
    let v: Value = serde_json::from_str(&stdout(&o)).expect("run --json must be valid JSON");
    assert_eq!(v["diagnostics"][0]["code"], "DL0703");
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn explain_prints_a_code_title() {
    let o = delulu(&["explain", "DL0501"]);
    assert!(stdout(&o).contains("DL0501"));
    assert!(o.status.success());
}

#[test]
fn run_with_trace_and_assert_trace_passes_and_emits_records() {
    let o = delulu(&[
        "run",
        "examples/demo.delulu",
        "--grant",
        "console",
        "--grant",
        "fs.read=./config",
        "--grant",
        "secret:API_KEY=k",
        "--trace-effects",
        "--assert-trace",
        "--seed",
        "42",
        "--clock",
        "fixed:1000",
    ]);
    assert!(o.status.success(), "trace ⊆ row must hold on the demo");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("\"effect\":\"Write\""), "trace records on stderr: {err}");
    assert!(err.contains("\"op\":\"read_text\""), "{err}");
    // The demo's secret is requested but never exposed — no Declassify record, and the
    // granted secret value must never appear in the trace.
    assert!(!err.contains("\"effect\":\"Declassify\""), "{err}");
    assert!(!err.contains("\"detail\":\"k\""), "{err}");
}

#[test]
fn build_multimodule_package_within_manifest_passes() {
    let o = delulu(&["build", "examples/greeter"]);
    assert!(o.status.success(), "greeter should build clean: {}", stdout(&o));
}

#[test]
fn authority_on_a_package_crosses_modules() {
    let o = delulu(&["authority", "examples/greeter", "--json"]);
    let v: Value = serde_json::from_str(&stdout(&o)).expect("authority --json must be valid JSON");
    let effects: Vec<&str> = v["authority"]["effects"].as_array().unwrap().iter().map(|x| x.as_str().unwrap()).collect();
    assert_eq!(effects, vec!["Write"]);
    // Two modules; imported helpers proven pure across the boundary.
    let modules: Vec<&str> = v["authority"]["modules"].as_array().unwrap().iter().map(|x| x.as_str().unwrap()).collect();
    assert!(modules.contains(&"greeter") && modules.contains(&"greeter.greetings"), "{modules:?}");
    let pure: Vec<&str> = v["authority"]["pure_functions"].as_array().unwrap().iter().map(|x| x.as_str().unwrap()).collect();
    assert!(pure.iter().any(|p| p.contains("salutation")), "{pure:?}");
}

#[test]
fn build_package_exceeding_manifest_authority_is_dl1009() {
    // A package that performs Write but declares only Read must fail to build (DL1009).
    let dir = std::env::temp_dir().join("delulu_cli_leaky_test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("delulu.toml"),
        "[package]\nname = \"leaky\"\nversion = \"0.1.0\"\n\n[authority]\neffects = [\"Read\"]\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src").join("main.delulu"),
        "module leaky\nfn main(root: Root) ! {Write} { let o = root.console()\n o.println(\"x\") }\n",
    )
    .unwrap();
    let o = delulu(&["build", dir.to_str().unwrap(), "--json"]);
    let v: Value = serde_json::from_str(&stdout(&o)).expect("build --json must be valid JSON");
    let has = v["diagnostics"].as_array().unwrap().iter().any(|d| d["code"] == "DL1009");
    assert!(has, "expected DL1009: {}", stdout(&o));
    assert_eq!(o.status.code(), Some(1));
}

// ----- Stage 4: foreign (phases 4d/4e) ---------------------------------------------------------

/// A minimal program that binds and calls a foreign lib — the shape every foreign test drives.
const FOREIGN_PROGRAM: &str = "module fdemo\n\
    foreign \"c\" lib mathlib { fn cos(x: Float) -> Float }\n\
    fn compute(root: Root) -> Result[Float, ForeignErr] ! {ForeignCall} { let load = root.foreign_load()\n \
    let m: mathlib = root.foreign(load)?\n Ok(m.cos(1.0)) }\n\
    fn main(root: Root) ! {ForeignCall, Write} { let c = root.console()\n \
    match compute(root) { Ok(r) => c.println(str(r)), Err(_) => c.println(\"bind failed\") } }\n";

fn write_foreign_program(dir_name: &str, manifest: Option<&str>) -> PathBuf {
    let dir = std::env::temp_dir().join(dir_name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("main.delulu"), FOREIGN_PROGRAM).unwrap();
    if let Some(m) = manifest {
        std::fs::write(dir.join("delulu.toml"), m).unwrap();
    }
    dir.join("main.delulu")
}

#[test]
fn authority_without_foreign_is_byte_identical_to_stage3() {
    // Criterion 7 (regression-critical): a program with no foreign use has `"foreign_calls": []`
    // and its human report is BYTE-IDENTICAL to the Stage-3 output, pinned here verbatim.
    let o = delulu(&["authority", "examples/demo.delulu"]);
    assert!(o.status.success());
    let expected = "Authority of `demo` — what this program can do to your system:\n\
        \x20 effects:      Read, Write\n\
        \x20 capabilities:\n\
        \x20   - FsRead   (scope granted at runtime)\n\
        \x20   - Console  stdio\n\
        \x20 secrets:      API_KEY\n\
        \x20 pure fns:     apply, fib\n\
        \x20 foreign:      (none — no code outside the guarantee)\n";
    assert_eq!(stdout(&o), expected, "the no-foreign report must not change byte-for-byte");
    let j = delulu(&["authority", "examples/demo.delulu", "--json"]);
    let v: Value = serde_json::from_str(&stdout(&j)).unwrap();
    assert_eq!(v["authority"]["foreign_calls"].as_array().unwrap().len(), 0);
}

#[test]
fn authority_lists_foreign_under_the_outside_the_proof_separator() {
    let file = write_foreign_program("delulu_cli_foreign_authority", None);
    let o = delulu(&["authority", file.to_str().unwrap()]);
    assert!(o.status.success(), "{}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("-- outside the proof (contained at process level) --"), "{out}");
    assert!(out.contains("c mathlib [cos]"), "{out}");

    // The JSON shape (spec §6): abi, lib, symbols, granted_path, used_at.
    let j = delulu(&["authority", file.to_str().unwrap(), "--json"]);
    let v: Value = serde_json::from_str(&stdout(&j)).unwrap();
    let fc = &v["authority"]["foreign_calls"][0];
    assert_eq!(fc["abi"], "c");
    assert_eq!(fc["lib"], "mathlib");
    assert_eq!(fc["symbols"][0], "cos");
    assert!(fc["granted_path"].is_null(), "authority is static; the path is a runtime grant");
    assert!(fc["used_at"][0]["line"].is_number());
}

#[test]
fn run_foreign_without_grant_is_dl1303_at_startup_not_mid_run() {
    // Criterion 4: the refusal happens in the grant flow, before `main` runs — the program's own
    // output never appears.
    let file = write_foreign_program("delulu_cli_foreign_ungranted", None);
    let o = delulu(&["run", file.to_str().unwrap(), "--grant", "console", "--json"]);
    let v: Value = serde_json::from_str(&stdout(&o)).expect("run --json must be valid JSON");
    assert_eq!(v["diagnostics"][0]["code"], "DL1303");
    assert_eq!(o.status.code(), Some(1));
    assert!(!stdout(&o).contains("bind failed"), "main must never have run");
}

#[test]
fn run_foreign_exceeding_manifest_ceiling_is_dl1303() {
    // A manifest that permits ForeignCall but does NOT list `mathlib` under `foreign.c`: the
    // program is reaching for authority it never declared — refused even WITH a grant on the line.
    let manifest = "[package]\nname = \"fdemo\"\nversion = \"0.1.0\"\n\n[authority]\neffects = [\"ForeignCall\", \"Write\"]\nforeign.c = []\n";
    let file = write_foreign_program("delulu_cli_foreign_ceiling", Some(manifest));
    let o = delulu(&[
        "run",
        file.to_str().unwrap(),
        "--grant",
        "console",
        "--grant",
        "foreign.c=mathlib:whatever.dll",
        "--json",
    ]);
    let v: Value = serde_json::from_str(&stdout(&o)).expect("run --json must be valid JSON");
    let d = &v["diagnostics"][0];
    assert_eq!(d["code"], "DL1303");
    assert!(
        d["message"].as_str().unwrap().contains("authority manifest"),
        "the ceiling message names the manifest: {d}"
    );
    assert_eq!(o.status.code(), Some(1));
}

#[cfg(windows)]
#[test]
fn run_foreign_cos_end_to_end_with_foreigncall_traced() {
    // Criterion 1 at the CLI surface (Windows: msvcrt.dll exports `cos`). The value is right and
    // `ForeignCall` appears in the effect trace; `--assert-trace` proves trace ⊆ row.
    let file = write_foreign_program("delulu_cli_foreign_cos", None);
    let o = delulu(&[
        "run",
        file.to_str().unwrap(),
        "--grant",
        "console",
        "--grant",
        "foreign.c=mathlib:msvcrt.dll",
        "--trace-effects",
        "--assert-trace",
    ]);
    assert!(o.status.success(), "stderr: {}", stderr(&o));
    assert!(stdout(&o).contains("0.5403023058681398"), "{}", stdout(&o));
    let err = stderr(&o);
    assert!(err.contains("\"effect\":\"ForeignCall\""), "trace records: {err}");
    assert!(err.contains("\"op\":\"cos\""), "{err}");
}

#[test]
fn explain_dl13xx_carries_the_honesty_caveats_and_never_says_sandbox() {
    // Spec §10 / trap 1: every foreign explain text states reachability-not-behavior, links forward
    // to Stage 5 for containment, and the word "sandbox" is banned for Stage 4 foreign code.
    for code in ["DL1301", "DL1302", "DL1303", "DL1304", "DL1306", "DL1308"] {
        let o = delulu(&["explain", code]);
        assert!(o.status.success(), "explain {code} failed");
        let out = stdout(&o);
        assert!(out.contains("bounds") && out.contains("reachability"), "{code}: {out}");
        assert!(out.contains("not behavior"), "{code}: {out}");
        assert!(out.contains("Stage 5"), "{code} must link forward to Stage 5: {out}");
        assert!(!out.to_lowercase().contains("sandbox"), "`sandbox` is banned: {code}: {out}");
    }
    // DL1301 additionally: the fence never suggests laundering a secret across the FFI.
    let o = delulu(&["explain", "DL1301"]);
    assert!(stdout(&o).contains("never suggests `expose`"), "{}", stdout(&o));
}

#[test]
fn why_help_lists_foreigncall_as_a_core_effect() {
    // The one-line side task: the `why` unknown-effect help must name ForeignCall with the core six.
    let o = delulu(&["why", "NotARealEffect", "examples/demo.delulu"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(stderr(&o).contains("ForeignCall"), "{}", stderr(&o));
}

#[test]
fn why_foreigncall_walks_the_chain_like_any_effect() {
    let file = write_foreign_program("delulu_cli_why_foreign", None);
    let o = delulu(&["why", "ForeignCall", file.to_str().unwrap()]);
    assert!(o.status.success(), "{}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("main") && out.contains("compute") && out.contains("ForeignCall"), "{out}");
}
