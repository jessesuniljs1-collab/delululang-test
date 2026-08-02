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

/// **A code the registry does not allocate gets an answer, not a dead end.**
///
/// Sixteen `DLxxxx` were named across this repository — in specifications, in build orders, in the
/// registry's own comments — that `REGISTRY` does not allocate. Asking the toolchain about any of
/// them returned `unknown code`, which is exactly what a typo returns. For the population this
/// language is built for, "I cannot tell you" and "that was withdrawn, here is why" are not the
/// same answer, and only one of them means the reader made a mistake.
#[test]
fn explain_answers_for_a_code_the_registry_does_not_allocate() {
    // One per explainable disposition, each a real code from a real place in the tree.
    let cases = [
        ("DL0503", "retired"),                   // withdrawn: the rule it named does not exist
        ("DL1404", "never-allocated"),           // the range skips it on purpose
        ("DL0210", "reserved"),                  // held open for the next parse diagnostic
        ("DL1012", "specified-not-implemented"), // a spec names it; nothing emits it
    ];
    for (code, disposition) in cases {
        let o = delulu(&["explain", code]);
        assert_eq!(o.status.code(), Some(0), "`explain {code}` must answer:\n{}", stderr(&o));
        let out = stdout(&o);
        assert!(out.contains(disposition), "`{code}` is {disposition}:\n{out}");
        // And it says WHY, not merely that it cannot be emitted.
        assert!(out.len() > 200, "`{code}` needs a reason, not a label:\n{out}");
    }

    // A number nobody has ever recorded is still an honest dead end.
    //
    // Built rather than written, so no literal code appears here. This file is scanned like every
    // other, and spelling it out would make it a *citation* — the Survey would then report it as a
    // code named in the tree that nothing explains, which is precisely the finding this work
    // closed. The first draft did exactly that and was caught by it.
    let never_allocated = format!("DL{}", 7777);
    let unknown = delulu(&["explain", &never_allocated]);
    assert_eq!(unknown.status.code(), Some(1), "a code with no record must still fail");

    // The sentinel stays unrecognised on purpose: `cli_contract` uses it as its negative case and
    // the registry guard asserts its absence. Explaining it would defeat both.
    let sentinel = delulu(&["explain", "DL9999"]);
    assert_eq!(sentinel.status.code(), Some(1), "the sentinel must keep being refused");
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

/// Criterion 1 at the CLI surface, shared by the three per-OS entry points below. `lib` is the
/// concrete C math library the human grants for the logical `mathlib`; the program, grant shape,
/// value (`cos(1.0) == 0.5403023058681398`), and trace assertions (`ForeignCall` + `op: cos`,
/// `--assert-trace` proving trace ⊆ row) are IDENTICAL on every OS. This body is not `cfg`-gated,
/// so it type-checks on every host regardless of which wrapper is active — only the one-line
/// library name per OS lives behind a `cfg`.
fn foreign_cos_end_to_end_asserting(lib: &str) {
    let file = write_foreign_program("delulu_cli_foreign_cos", None);
    let grant = format!("foreign.c=mathlib:{lib}");
    let o = delulu(&[
        "run",
        file.to_str().unwrap(),
        "--grant",
        "console",
        "--grant",
        grant.as_str(),
        "--trace-effects",
        "--assert-trace",
    ]);
    assert!(o.status.success(), "stderr: {}", stderr(&o));
    assert!(stdout(&o).contains("0.5403023058681398"), "{}", stdout(&o));
    let err = stderr(&o);
    assert!(err.contains("\"effect\":\"ForeignCall\""), "trace records: {err}");
    assert!(err.contains("\"op\":\"cos\""), "{err}");
}

#[cfg(windows)]
#[test]
fn run_foreign_cos_end_to_end_with_foreigncall_traced() {
    // Windows: `msvcrt.dll` exports `cos` and resolves by bare name via the OS loader.
    foreign_cos_end_to_end_asserting("msvcrt.dll");
}

#[cfg(target_os = "macos")]
#[test]
fn run_foreign_cos_end_to_end_with_foreigncall_traced() {
    // macOS mirror: `libm.dylib` resolves via the dyld shared cache on modern macOS (Big Sur+) even
    // though no such file exists on disk — dlopen serves it from the cache. Same value, same trace.
    foreign_cos_end_to_end_asserting("libm.dylib");
}

#[cfg(target_os = "linux")]
#[test]
fn run_foreign_cos_end_to_end_with_foreigncall_traced() {
    // Linux mirror: `libm.so.6` is the glibc math-library soname, resolved by bare name via ld.so.
    foreign_cos_end_to_end_asserting("libm.so.6");
}

#[test]
fn explain_dl13xx_carries_the_honesty_caveats_and_never_says_sandbox() {
    // Spec §10 / trap 1: every foreign explain text states reachability-not-behavior, links forward
    // to Stage 5 for containment, and the word "sandbox" is banned for Stage 4 foreign code.
    for code in ["DL1301", "DL1302", "DL1303", "DL1304", "DL1305", "DL1306", "DL1307", "DL1308"] {
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
    // DL1305 additionally: the spec §5.3 allowlist honesty note — the allowlist gates the interface,
    // not transitive imports; embedded Python has full process authority.
    let o = delulu(&["explain", "DL1305"]);
    let out = stdout(&o);
    assert!(out.contains("gates the interface"), "DL1305 must carry the §5.3 honesty note: {out}");
    assert!(out.contains("full process authority"), "{out}");
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

// ----- Stage 4: embedded CPython (phase 4f) ----------------------------------------------------

/// A program that reaches for embedded Python by importing `os` — the shape the Python CLI tests
/// drive. `attempt` reports whether the import was allowed or denied.
const PY_IMPORT_OS: &str = "module pyos\n\
    fn attempt(py: Cap[Python]) -> Str ! {ForeignCall} {\n\
      match py.import(\"os\") { Ok(_) => \"imported\", Err(e) => \"denied:\" + e.kind } }\n\
    fn main(root: Root) ! {ForeignCall, Write} { let c = root.console()\n\
      let load = root.foreign_load()\n\
      match root.python(load) { Ok(py) => c.println(attempt(py)), Err(_) => c.println(\"unavailable\") } }\n";

fn write_python_program(dir_name: &str, src: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(dir_name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("main.delulu"), src).unwrap();
    dir.join("main.delulu")
}

#[test]
fn run_numpy_mean_demo_prints_the_mean() {
    // Criterion 2 at the CLI surface: the committed example imports NumPy, builds a list, calls
    // `mean`, converts back, and prints it. Skip-with-notice when the interpreter / NumPy is absent
    // (CI portability); on a host that has both it RUNS and prints `mean = 2.5`.
    let o = delulu(&[
        "run",
        "examples/numpy_mean.delulu",
        "--grant",
        "console",
        "--grant",
        "foreign.python=numpy",
        "--grant",
        "foreign.python=numpy.*",
    ]);
    let out = stdout(&o);
    if out.contains("unavailable") || out.contains("No module") {
        eprintln!("SKIP (portability): embedded Python or NumPy unavailable on this host");
        return;
    }
    assert!(o.status.success(), "stderr: {}", stderr(&o));
    assert!(out.contains("mean = 2.5"), "criterion 2 demo output: {out}");
}

#[test]
fn run_import_os_off_allowlist_is_dl1305_and_the_denied_attempt_is_traced() {
    // Criterion 5 at the CLI surface: `py.import("os")` under an allowlist of ["numpy"] is refused
    // as a runtime PyErr (DL1305), and the denied attempt is a ForeignCall record in the trace.
    let file = write_python_program("delulu_cli_py_import_os", PY_IMPORT_OS);
    let o = delulu(&[
        "run",
        file.to_str().unwrap(),
        "--grant",
        "console",
        "--grant",
        "foreign.python=numpy",
        "--trace-effects",
        "--assert-trace",
    ]);
    let out = stdout(&o);
    if out.contains("unavailable") {
        eprintln!("SKIP (portability): no embedded CPython on this host");
        return;
    }
    assert!(o.status.success(), "stderr: {}", stderr(&o));
    assert!(out.contains("denied:ImportNotAllowed"), "off-allowlist import must be denied: {out}");
    let err = stderr(&o);
    assert!(err.contains("\"op\":\"py.import\""), "the denied import must be traced: {err}");
    assert!(err.contains("\"effect\":\"ForeignCall\""), "{err}");
    assert!(err.contains("\"detail\":\"os\""), "the trace must name the refused module: {err}");
}

#[test]
fn run_python_without_grant_is_dl1303_at_startup() {
    // Deny-by-default: a program that reaches `root.python` with no `foreign.python` grant is refused
    // at the grant flow (DL1303), before `main` runs.
    let file = write_python_program("delulu_cli_py_ungranted", PY_IMPORT_OS);
    let o = delulu(&["run", file.to_str().unwrap(), "--grant", "console", "--json"]);
    let v: Value = serde_json::from_str(&stdout(&o)).expect("run --json must be valid JSON");
    assert_eq!(v["diagnostics"][0]["code"], "DL1303");
    assert_eq!(o.status.code(), Some(1));
    assert!(!stdout(&o).contains("denied"), "main must never have run");
}

#[test]
fn authority_lists_python_under_the_outside_the_proof_separator() {
    let o = delulu(&["authority", "examples/numpy_mean.delulu"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("-- outside the proof (contained at process level) --"), "{out}");
    assert!(out.contains("python"), "{out}");
    assert!(out.contains("imports seen: [numpy]"), "{out}");

    // The JSON shape (spec §6): abi:"python", allowlist, imports_seen, used_at.
    let j = delulu(&["authority", "examples/numpy_mean.delulu", "--json"]);
    let v: Value = serde_json::from_str(&stdout(&j)).unwrap();
    let entry = v["authority"]["foreign_calls"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["abi"] == "python")
        .expect("a python foreign_calls entry");
    assert_eq!(entry["imports_seen"][0], "numpy");
    assert!(entry["used_at"][0]["line"].is_number());
}

// ===== Stage 5 chunk 5 (phases 5i + 5j): isolation profiles + the explain honesty text ==========

/// A trivial pure program in a temp dir (no grants needed).
fn write_pure_program(dir_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(dir_name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("main.delulu"), "module m\nfn main(root: Root) {}\n").unwrap();
    dir.join("main.delulu")
}

/// Playbook trap 3 / spec §4.2: `delulu explain E-REVOKE` states the latency bound VERBATIM —
/// synchronous = before the next use; epoch = within one interval (≤ 50 ms default) — and never
/// claims "immediate" (the word appears only inside the §10 quote denying the claim).
#[test]
fn explain_e_revoke_states_the_4_2_bound_verbatim() {
    let o = delulu(&["explain", "E-REVOKE"]);
    assert!(o.status.success(), "explain E-REVOKE failed: {}", stderr(&o));
    let out = stdout(&o);
    assert!(
        out.contains(
            "Revocation takes effect: synchronous class — before the next use; epoch class — \
             within one epoch interval (≤ 50 ms default)."
        ),
        "the §4.2 bound must appear verbatim: {out}"
    );
    assert!(out.contains("No stronger claim is made anywhere."), "{out}");
    assert!(out.contains("\"immediate\" is never claimed"), "the §10 caveat word-for-word: {out}");
}

/// The DL14xx custody codes explain themselves with the spec §10 caveats word-for-word
/// (playbook 5j: "Copy spec §10 caveats into docs and explain-text word-for-word").
#[test]
fn explain_dl14xx_carries_the_spec_10_caveats() {
    let o = delulu(&["explain", "DL1408"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("--isolation process"), "the fallback command is shown: {out}");
    assert!(
        out.contains("Foreign workers bound blast radius, not foreign behavior"),
        "§10 caveat verbatim: {out}"
    );
    assert!(out.contains("Linux-first"), "{out}");

    let o = delulu(&["explain", "E-DL1403"]);
    assert!(o.status.success());
    assert!(stdout(&o).contains("\"immediate\" is never claimed"), "{}", stdout(&o));

    let o = delulu(&["explain", "DL1405"]);
    assert!(stdout(&o).contains("detects tampering after the fact; it does not prevent it"), "{}", stdout(&o));
}

/// Phase 5i (spec §6): `--isolation microvm` is refused with DL1408 — nothing runs, the fallback
/// is named and labeled weaker, never silently substituted (trap 8). Truthful on every platform
/// v0.5 supports: on non-Linux it is a platform refusal, on Linux without a provisioned guest
/// launch it is a prerequisite/pending refusal — both DL1408 with the same documented fallback.
/// (The positive criterion-8 test is `tests/microvm_criterion8.rs`, gated on `cfg(delulu_kvm)`.)
#[test]
fn run_isolation_microvm_is_dl1408_with_labeled_weaker_fallback() {
    let file = write_pure_program("delulu_cli_iso_microvm");
    let o = delulu(&["run", file.to_str().unwrap(), "--isolation", "microvm", "--json"]);
    assert_eq!(o.status.code(), Some(1), "microvm must refuse, not run: {}", stderr(&o));
    let v: Value = serde_json::from_str(&stdout(&o)).expect("run --json must be valid JSON");
    assert_eq!(v["diagnostics"][0]["code"], "DL1408");
    let msg = v["diagnostics"][0]["message"].as_str().unwrap();
    assert!(msg.contains("--isolation process"), "the documented fallback command: {msg}");
    assert!(msg.contains("weaker"), "the fallback is labeled explicitly weaker: {msg}");
}

/// Phase 5i: `--isolation process` runs and labels itself honestly — foreign code in workers, the
/// verified program in-process, never sold as microVM-equivalent (trap 8).
#[test]
fn run_isolation_process_is_labeled_honestly() {
    let file = write_pure_program("delulu_cli_iso_process");
    let o = delulu(&["run", file.to_str().unwrap(), "--isolation", "process"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let err = stderr(&o);
    assert!(err.contains("isolation: process"), "the run is labeled: {err}");
    assert!(err.contains("weaker than microvm"), "labeled weaker, honestly: {err}");
    // And the default run stays byte-identical (criterion 11): no isolation label without the flag.
    let o = delulu(&["run", file.to_str().unwrap()]);
    assert!(o.status.success());
    assert!(!stderr(&o).contains("isolation:"), "no label without --isolation: {}", stderr(&o));
}

/// Phase 5i: `delulu authority --isolation microvm` reports the profile honestly (unavailable
/// here), and an unknown profile is a usage error.
#[test]
fn authority_isolation_label_is_honest_and_unknown_is_refused() {
    let o = delulu(&["authority", "examples/demo.delulu", "--isolation", "microvm"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("isolation:"), "{out}");
    assert!(out.contains("unavailable here"), "an unattainable microvm is never affirmed: {out}");

    let file = write_pure_program("delulu_cli_iso_unknown");
    let o = delulu(&["run", file.to_str().unwrap(), "--isolation", "container"]);
    assert_eq!(o.status.code(), Some(2), "unknown profile is a usage error");
    assert!(stderr(&o).contains("none | process | microvm"), "{}", stderr(&o));
}

// ----- P5 (HARDENING_CAMPAIGN C25/C26/C27) -----------------------------------------------------

#[test]
fn the_authority_report_says_out_loud_that_a_credential_can_leave() {
    // C25. Every fact was already in this report — `Declassify` on the effects line, the secret
    // name on the secrets line, the lib on the foreign line — and the reader had to join three
    // lines to see "this program can hand API_KEY to code outside the proof". This report exists to
    // be read by whoever decides whether to type `--grant declassify`, and the decision turns
    // precisely on that join, so the report states it.
    let dir = std::env::temp_dir().join("delulu_cli_exposure_line");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let src = "module fdemo\n\
        foreign \"c\" lib msvcrt { fn puts(s: Str) -> Int }\n\
        fn compute(root: Root) -> Result[Int, ForeignErr] ! {ForeignCall, Declassify} { let load = root.foreign_load()\n \
        let m: msvcrt = root.foreign(load)?\n let k = root.secret(\"API_KEY\")\n let d = root.declassify()\n \
        Ok(m.puts(k.expose(d))) }\n\
        fn main(root: Root) ! {ForeignCall, Write, Declassify} { let c = root.console()\n \
        match compute(root) { Ok(_) => c.println(\"ok\"), Err(_) => c.println(\"bind failed\") } }\n";
    let file = dir.join("main.delulu");
    std::fs::write(&file, src).unwrap();
    let o = delulu(&["authority", file.to_str().unwrap()]);
    let out = stdout(&o);
    assert!(out.contains("exposure:"), "the report must carry an exposure line:\n{out}");
    assert!(out.contains("API_KEY declassifiable"), "it must name the secret:\n{out}");
    assert!(
        out.contains("foreign code (outside the proof)"),
        "it must name foreign reach as the egress:\n{out}"
    );
}

#[test]
fn a_program_that_cannot_declassify_has_no_exposure_line() {
    // The other half: the line is gated on `Declassify` being in the row, so every report for a
    // program that cannot call `expose` is unchanged — including the byte-identical Stage-3 report
    // pinned in `authority_without_foreign_is_byte_identical_to_stage3`, which lists a secret.
    let o = delulu(&["authority", "examples/demo.delulu"]);
    assert!(!stdout(&o).contains("exposure:"), "{}", stdout(&o));
}

#[test]
fn a_package_with_no_modules_does_not_build_clean() {
    // C26. A flat layout (sources beside `delulu.toml` instead of under `src/`) produced
    // `built clean (1 package(s), 0 module(s))` and exit 0: the toolchain checked nothing and
    // reported success. Same posture as a deferred git dependency — a check that could not run must
    // never report success.
    let dir = std::env::temp_dir().join("delulu_cli_zero_modules");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("delulu.toml"), "[package]\nname = \"empty\"\nversion = \"0.1.0\"\n").unwrap();
    // A source file in the WRONG place — the natural mistake this refusal exists to catch.
    std::fs::write(dir.join("main.delulu"), "module empty\nfn main(root: Root) { }\n").unwrap();
    let o = delulu(&["build", dir.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(1), "a zero-module build must fail:\n{}", stderr(&o));
    let err = stderr(&o);
    assert!(err.contains("no `.delulu` modules found"), "{err}");
    assert!(err.contains("src/"), "the note must say where sources belong: {err}");
    assert!(!stdout(&o).contains("built clean"), "{}", stdout(&o));
    // And it must not close with `0 error(s)`, which read as success beside a nonzero exit.
    assert!(!err.contains("0 error(s)"), "{err}");
}

#[test]
fn naming_a_directory_where_a_file_belongs_says_so() {
    // C27. Reading a directory as a file surfaced the raw OS error — on Windows
    // `Access is denied. (os error 5)`, which reads as a permissions problem and sends the reader
    // hunting for an ACL that was never involved; on Linux `Is a directory`. Different misleading
    // text per platform, for one mistake.
    //
    // D61 changed the premise, not the property: `run` now DOES take a package directory, so the old
    // advice ("try `delulu build`") became wrong and the two directories that still cannot be run
    // need their own answers. What C27 asserts is unchanged and is the last two lines of each case —
    // exit 2, and never a raw OS error.

    // 1. A manifest with no sources. `build` refuses this too (C26/D33), and `run` uses the same
    //    words rather than inventing a second vocabulary for one condition.
    let dir = std::env::temp_dir().join("delulu_cli_dir_empty_pkg");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("delulu.toml"), "[package]\nname = \"p\"\nversion = \"0.1.0\"\n").unwrap();
    let o = delulu(&["run", dir.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(2), "a usage mistake is exit 2");
    let err = stderr(&o);
    assert!(err.contains("no `.delulu` modules found"), "it must say what is missing: {err}");
    assert!(err.contains("src"), "and where sources belong: {err}");
    assert!(!err.contains("os error"), "no raw OS error may leak into this message: {err}");

    // 2. A directory that is not a package at all.
    let bare = std::env::temp_dir().join("delulu_cli_dir_not_pkg");
    let _ = std::fs::remove_dir_all(&bare);
    std::fs::create_dir_all(&bare).unwrap();
    let o = delulu(&["run", bare.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(2), "a usage mistake is exit 2");
    let err = stderr(&o);
    assert!(err.contains("not a DeluluLang package"), "it must say why: {err}");
    assert!(err.contains("delulu.toml"), "and name what is absent: {err}");
    assert!(!err.contains("os error"), "no raw OS error may leak into this message: {err}");
}

// ----- Surface morphs (Stage 8 §6.5; HARDENING_CAMPAIGN C22) ------------------------------------

/// Write a morph file into a temp dir and point `DELULU_MORPH_PATH` at it.
fn with_morph_dir(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (f, body) in files {
        std::fs::write(dir.join(f), body).unwrap();
    }
    dir
}

fn delulu_with_morphs(dir: &std::path::Path, args: &[&str]) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        .env("DELULU_MORPH_PATH", dir)
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("delulu must run")
}

const ZH: &str = "[meta]\nmorph=\"zh\"\nname=\"zh\"\nversion=\"1.0.0\"\nkind=\"human\"\n\
    [keywords]\nfn=\"函数\"\nlet=\"令\"\nif=\"如果\"\nelse=\"否则\"\nmodule=\"模块\"\n";

#[test]
fn a_program_written_in_a_morph_runs_and_prints_the_same_thing() {
    // The whole feature in one test. A DeluluLang program whose keywords are Chinese is the SAME
    // program: it runs, and it prints what its canonical form prints.
    let dir = with_morph_dir("delulu_morph_run", &[("zh.toml", ZH)]);
    let canonical = "module demo\nfn main(root: Root) ! {Write} { let o = root.console()\n o.println(\"hi\") }\n";
    let cf = dir.join("canon.delulu");
    std::fs::write(&cf, canonical).unwrap();

    // Render to the morph, then run the rendered file directly.
    let r = delulu_with_morphs(&dir, &["morph", "render", cf.to_str().unwrap(), "--to", "zh"]);
    assert!(r.status.success(), "render failed: {}", stderr(&r));
    let morphed = stdout(&r);
    assert!(morphed.starts_with("//! morph: zh\n"), "must carry a pragma:\n{morphed}");
    assert!(morphed.contains("函数 main"), "keywords must be morphed:\n{morphed}");
    assert!(morphed.contains("\"hi\""), "a string literal is never morphed:\n{morphed}");
    assert!(morphed.contains("root.console()"), "identifiers are never morphed:\n{morphed}");

    let mf = dir.join("zh.delulu");
    std::fs::write(&mf, &morphed).unwrap();
    let o = delulu_with_morphs(&dir, &["run", mf.to_str().unwrap(), "--grant", "console"]);
    assert!(o.status.success(), "the morphed program must run: {}", stderr(&o));
    assert_eq!(stdout(&o).trim(), "hi");
}

#[test]
fn a_morph_round_trip_is_byte_identical_and_the_authority_report_does_not_move() {
    // The canonical-form law, end to end: converting there and back returns the original bytes, and
    // the machine envelope (here, the authority report) cannot tell which surface was used.
    let dir = with_morph_dir("delulu_morph_roundtrip", &[("zh.toml", ZH)]);
    let canonical = "module demo\nfn main(root: Root) ! {Write} { let o = root.console()\n o.println(\"x\") }\n";
    let cf = dir.join("canon.delulu");
    std::fs::write(&cf, canonical).unwrap();

    let r = delulu_with_morphs(&dir, &["morph", "render", cf.to_str().unwrap(), "--to", "zh"]);
    let mf = dir.join("zh.delulu");
    std::fs::write(&mf, stdout(&r)).unwrap();
    let back = delulu_with_morphs(&dir, &["morph", "render", mf.to_str().unwrap(), "--to-canonical"]);
    assert_eq!(stdout(&back), canonical, "round-trip must be byte-identical");

    let a1 = delulu_with_morphs(&dir, &["authority", cf.to_str().unwrap()]);
    let a2 = delulu_with_morphs(&dir, &["authority", mf.to_str().unwrap()]);
    assert_eq!(
        stdout(&a1),
        stdout(&a2),
        "the authority report must be morph-invariant — the surface is not part of the program"
    );
}

#[test]
fn a_morph_whose_alias_is_another_keyword_is_refused() {
    // DL1711 — the review attack. This morph is bijective and its alias is a single token, and a
    // file written in it uses the word `fn` to mean `let`: it would render and round-trip perfectly
    // while misleading every human who read it. Bijectivity alone does not forbid this.
    let dir = with_morph_dir(
        "delulu_morph_liar",
        &[("liar.toml", "[meta]\nmorph=\"liar\"\nkind=\"custom\"\n[keywords]\nlet=\"fn\"\n")],
    );
    let o = delulu_with_morphs(&dir, &["morph", "check", dir.join("liar.toml").to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(1));
    let err = stderr(&o);
    assert!(err.contains("DL1711"), "{err}");
    assert!(err.contains("misled"), "{err}");
}

#[test]
fn a_morph_alias_may_not_carry_a_bidi_control_or_split_into_two_tokens() {
    // DL1712, and the D26 discipline extended to morphs: an alias that renders differently than it
    // lexes defeats the purpose of having a canonical form at all.
    let dir = with_morph_dir(
        "delulu_morph_bad_alias",
        &[
            ("bidi.toml", "[meta]\nmorph=\"bidi\"\nkind=\"custom\"\n[keywords]\nfn=\"a\u{202e}b\"\n"),
            ("split.toml", "[meta]\nmorph=\"split\"\nkind=\"custom\"\n[keywords]\nfn=\"a b\"\n"),
        ],
    );
    for f in ["bidi.toml", "split.toml"] {
        let o = delulu_with_morphs(&dir, &["morph", "check", dir.join(f).to_str().unwrap()]);
        assert_eq!(o.status.code(), Some(1), "{f}");
        assert!(stderr(&o).contains("DL1712"), "{f}: {}", stderr(&o));
    }
}

#[test]
fn two_keywords_sharing_an_alias_is_refused() {
    // DL1710 — without bijectivity, rendering back to canonical would have to guess.
    let dir = with_morph_dir(
        "delulu_morph_dup",
        &[("dup.toml", "[meta]\nmorph=\"dup\"\nkind=\"custom\"\n[keywords]\nfn=\"x\"\nlet=\"x\"\n")],
    );
    let o = delulu_with_morphs(&dir, &["morph", "check", dir.join("dup.toml").to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stderr(&o).contains("DL1710"), "{}", stderr(&o));
}

#[test]
fn renaming_something_that_is_not_a_keyword_is_refused() {
    // DL1713 — including the contextual keywords. `foreign` is lexed as an identifier, so renaming
    // it would be renaming an identifier, which the canonical-form law forbids.
    let dir = with_morph_dir(
        "delulu_morph_notkw",
        &[("nk.toml", "[meta]\nmorph=\"nk\"\nkind=\"custom\"\n[keywords]\nforeign=\"外部\"\n")],
    );
    let o = delulu_with_morphs(&dir, &["morph", "check", dir.join("nk.toml").to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stderr(&o).contains("DL1713"), "{}", stderr(&o));
}

#[test]
fn a_file_naming_an_uninstalled_morph_is_refused_not_guessed() {
    // DL1714. Reading a morphed file as though it were canonical would produce a wall of unrelated
    // syntax errors; silently succeeding on whatever happened to lex would be worse.
    let dir = with_morph_dir("delulu_morph_missing", &[]);
    let f = dir.join("ghost.delulu");
    std::fs::write(&f, "//! morph: nope\nmodule m\nfn main(root: Root) { }\n").unwrap();
    let o = delulu_with_morphs(&dir, &["check", f.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(1));
    let err = stderr(&o);
    assert!(err.contains("DL1714"), "{err}");
    assert!(err.contains("looked in"), "the message must say where it searched: {err}");
}

#[test]
fn a_morph_id_may_not_escape_the_search_path() {
    // A morph id becomes a filename. Refusing path-shaped ids is one line, and a toolchain that
    // reads arbitrary paths on request is not one this language should ship.
    let dir = with_morph_dir("delulu_morph_traversal", &[]);
    let o = delulu_with_morphs(&dir, &["morph", "info", "../../etc/passwd"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stderr(&o).contains("not a valid morph id"), "{}", stderr(&o));
}
