//! The conformance harness (Stage-9 coverage law, scoped to Stage 1).
//!
//! It drives the real compiler over a corpus of `.delulu` programs:
//! - `tests/conformance/accept/**` and `tests/corpus/**` must check clean;
//! - `tests/conformance/reject/**` must produce the diagnostic named by the file
//!   (`DL0402_*.delulu` must yield `DL0402`);
//! - every Stage-1-producible diagnostic code has at least one rejecting case (coverage).
//!
//! Programs are the specification's executable form: an accepting program that regresses, or a
//! rejecting program whose code changes, breaks this test.

use std::path::{Path, PathBuf};

use delulu_check::check_source;

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = <root>/crates/delulu
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(delulu_files(&p));
            } else if p.extension().and_then(|s| s.to_str()) == Some("delulu") {
                out.push(p);
            }
        }
    }
    out
}

fn error_codes(src: &str) -> Vec<String> {
    check_source(0, src)
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .map(|d| d.code.to_string())
        .collect()
}

fn expected_code(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    let code = name.split('_').next()?;
    if code.starts_with("DL") && code.len() == 6 {
        Some(code.to_string())
    } else {
        None
    }
}

#[test]
fn accepting_programs_check_clean() {
    let root = workspace_root();
    let mut checked = 0;
    for dir in ["tests/conformance/accept", "tests/corpus"] {
        for file in delulu_files(&root.join(dir)) {
            let src = std::fs::read_to_string(&file).unwrap();
            let codes = error_codes(&src);
            assert!(codes.is_empty(), "{} should check clean, got {:?}", file.display(), codes);
            checked += 1;
        }
    }
    assert!(checked >= 12, "expected a substantial accept corpus, found {checked}");
}

#[test]
fn rejecting_programs_produce_their_named_code() {
    let root = workspace_root();
    let files = delulu_files(&root.join("tests/conformance/reject"));
    assert!(!files.is_empty(), "no reject programs found");
    for file in &files {
        let want = expected_code(file)
            .unwrap_or_else(|| panic!("reject file {} must be named DLxxxx_*.delulu", file.display()));
        let src = std::fs::read_to_string(file).unwrap();
        let codes = error_codes(&src);
        assert!(
            codes.iter().any(|c| *c == want),
            "{} should produce {}, got {:?}",
            file.display(),
            want,
            codes
        );
    }
}

/// The coverage law (Stage 1 scope): every diagnostic code a Stage-1 program can trigger has at
/// least one rejecting conformance case. Lexer/parser-position codes and runtime DL09xx codes are
/// covered by crate unit tests and the CLI integration test respectively; this list is the set a
/// static `.delulu` reject file can exercise.
#[test]
fn every_stage1_static_code_has_a_reject_case() {
    let root = workspace_root();
    let covered: std::collections::HashSet<String> = delulu_files(&root.join("tests/conformance/reject"))
        .iter()
        .filter_map(|f| expected_code(f))
        .collect();

    let required = [
        "DL0106", "DL0107", "DL0204", "DL0206", "DL0301", "DL0302", "DL0305", "DL0306", "DL0307", "DL0401",
        "DL0402", "DL0403", "DL0405", "DL0407", "DL0409", "DL0410", "DL0501", "DL0504", "DL0602",
        "DL0603", "DL0604", "DL0605",
    ];
    let missing: Vec<&str> = required.iter().copied().filter(|c| !covered.contains(*c)).collect();
    assert!(missing.is_empty(), "coverage law: these codes have no reject case: {missing:?}");
}
