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
