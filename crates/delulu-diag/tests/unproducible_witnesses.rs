//! Constructor-level witnesses for the unproducible-by-construction codes (Stage 9, ruling D10;
//! executed at the 1.0 release gate, ruling D22).
//!
//! No program can produce these codes: they are the compiler-bug class (DL1101 trace ⊄ row,
//! DL1206 engine-parity self-check, DL1610 race-checker, DL1702 formatter-law), the fuzz
//! harness's own code (DL1102), a tooling-configuration error (DL1701), and — for the default
//! test matrix, where the `python` feature is ON — DL1307 (python runtime unavailable), whose
//! feature-off shim cannot be compiled into a default build. D10's ruling, verbatim in spirit:
//! "a code that cannot fire is not thereby exempt; the honest witness is a *constructor-level*
//! test (the fault is built and classified, its explain body renders) rather than a producing
//! program." These are those witnesses. Each proves, per code:
//!   1. the code is registered and titled;
//!   2. the diagnostic CONSTRUCTS at error severity (the classification a real emission would
//!      carry) — so if the registry ever dropped the code, the emission site's contract breaks
//!      here first;
//!   3. the long-form explanation exists, meets the Stage-9 length bar, and is honest about the
//!      compiler-bug class where that is the class;
//!   4. the diagnostic renders on the machine channel (the JSON envelope carries the code and
//!      an explanation id) — an agent that receives one of these has everything it needs.

use delulu_diag::{
    code_explain, code_title, envelope, is_registered, Diagnostic, Severity, SourceMap,
    MIN_EXPLAIN_BODY,
};

/// The witness body shared by every unproducible code.
fn witness(code: &'static str, expect_in_explain: &str) {
    assert!(is_registered(code), "{code} must be registered");
    assert!(code_title(code).is_some(), "{code} must carry a title");

    let d = Diagnostic::error(code, format!("constructor-level witness for {code}"));
    assert_eq!(d.code, code);
    assert!(matches!(d.severity, Severity::Error), "{code} classifies as an error");

    let explain = code_explain(code).unwrap_or_else(|| panic!("{code} needs an explain body"));
    assert!(
        explain.len() >= MIN_EXPLAIN_BODY,
        "{code} explain body meets the Stage-9 length bar ({} < {MIN_EXPLAIN_BODY})",
        explain.len()
    );
    assert!(
        explain.to_lowercase().contains(&expect_in_explain.to_lowercase()),
        "{code}'s explanation must say `{expect_in_explain}` — the honesty the class demands:\n{explain}"
    );

    let map = SourceMap::new();
    let env = envelope("check", &[d], None, &map);
    assert_eq!(env["diagnostics"][0]["code"], code);
    assert!(
        env["diagnostics"][0]["explanation_id"].as_str().is_some(),
        "{code} carries an explanation id on the machine channel"
    );
}

/// DL1101 — a runtime effect outside the static row. The soundness claim checked against
/// reality; producing it requires the type system to have already failed.
#[test]
fn dl1101_trace_outside_row_constructs_classifies_and_explains() {
    witness("DL1101", "compiler-bug");
}

/// DL1102 — a repair that did not produce an accepting program. The fuzz harness's own code;
/// producing it requires a broken repair, which criterion-2 fuzzing exists to prevent.
#[test]
fn dl1102_failed_repair_constructs_classifies_and_explains() {
    witness("DL1102", "repair");
}

/// DL1206 — the two engines disagreeing on one program. Producing it requires a parity bug the
/// differential suite exists to prevent.
#[test]
fn dl1206_engine_parity_failure_constructs_classifies_and_explains() {
    witness("DL1206", "compiler-bug");
}

/// DL1307 — python runtime unavailable. Unproducible in the DEFAULT matrix (the `python`
/// feature is on, so the feature-off shim that emits it is not compiled in); the python-less
/// build's behavior is documented at the shim (`python.rs`) and exercised by construction here.
#[test]
fn dl1307_python_unavailable_constructs_classifies_and_explains() {
    witness("DL1307", "python");
}

/// DL1610 — the debug race-checker detector. The rcap system makes the race impossible
/// statically; the detector exists as defense-in-depth, and firing is compiler-bug class.
#[test]
fn dl1610_race_checker_violation_constructs_classifies_and_explains() {
    witness("DL1610", "compiler-bug");
}

/// DL1701 — LSP/workspace configuration error: tooling-environment class, no program produces it.
#[test]
fn dl1701_lsp_configuration_error_constructs_classifies_and_explains() {
    witness("DL1701", "workspace");
}

/// DL1702 — the formatter breaking its own identity/idempotence laws. `delulu fmt` verifies both
/// laws inline before writing a byte, so producing this requires a formatter bug.
#[test]
fn dl1702_formatter_law_violation_constructs_classifies_and_explains() {
    witness("DL1702", "compiler bug");
}
