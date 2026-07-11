//! Criterion 9 (holder-neutrality), the grep-checkable form (playbook trap 2 / spec criterion 9).
//!
//! The `delulu-broker` crate must contain NO `match`/`if` on `holder.kind` on any authority-decision
//! path. `holder` is descriptive `{kind, desc, peer}` metadata that is *stored and displayed, never
//! switched on*. This test reads the crate's own `src/*.rs` at test time and asserts that every
//! access of a `.kind` field is either a comment or an explicitly allowlisted display/serialization
//! line (marked with `KIND_IS_DATA`). A stray `if holder.kind == "human"` fails the build.
//!
//! (The companion *behavioral* form — identical serialized outcomes across holder kinds — lives in
//! `tree::tests::criterion_9_holder_kind_does_not_affect_the_outcome`.)

use std::fs;
use std::path::Path;

/// The one comment marker that whitelists a legitimate display/serialization access of `.kind`.
const ALLOW_MARKER: &str = "KIND_IS_DATA";

#[test]
fn no_holder_kind_branching_outside_allowlisted_display() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let needle = format!("{}{}", ".", "kind"); // avoid a literal `.kind` in this file
    let mut offending: Vec<String> = Vec::new();

    for entry in fs::read_dir(&src_dir).expect("read src dir") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read source file");
        for (i, line) in text.lines().enumerate() {
            if !line.contains(&needle) {
                continue;
            }
            let trimmed = line.trim_start();
            let is_comment = trimmed.starts_with("//");
            let is_allowlisted = line.contains(ALLOW_MARKER);
            if !is_comment && !is_allowlisted {
                offending.push(format!(
                    "{}:{}: {}",
                    path.file_name().unwrap().to_string_lossy(),
                    i + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        offending.is_empty(),
        "found `.kind` access(es) that are neither comments nor `{ALLOW_MARKER}`-marked display \
         code — holder-neutrality (criterion 9) forbids branching on holder kind:\n{}",
        offending.join("\n")
    );
}

/// Sanity: the marker discipline is actually exercised — at least one source line legitimately reads
/// a `.kind` for display and carries the marker. (Guards against the whitelist silently going stale
/// if the render code is refactored away, which would make the test vacuous.)
#[test]
fn display_access_is_marked_and_present() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let needle = format!("{}{}", ".", "kind");
    let mut marked_access = false;
    for entry in fs::read_dir(&src_dir).expect("read src dir") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read source file");
        for line in text.lines() {
            if line.contains(&needle) && line.contains(ALLOW_MARKER) {
                marked_access = true;
            }
        }
    }
    assert!(marked_access, "expected at least one KIND_IS_DATA-marked `.kind` display access");
}
