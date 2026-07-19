//! The Book's samples are conformance tests (Stage 9h — release criterion 7).
//!
//! A tutorial whose examples do not compile teaches people something false and wastes their
//! afternoon proving it. Every sample the Book ships is a real file, checked by the real binary,
//! on every CI run.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(root())
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn samples() -> Vec<PathBuf> {
    let dir = root().join("docs/book/samples");
    let mut v: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("the Book's samples live at {}: {e}", dir.display()))
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("delulu"))
        .collect();
    v.sort();
    v
}

/// CRITERION 7: every code sample the Book ships compiles.
#[test]
fn criterion7_every_book_sample_checks_clean() {
    let files = samples();
    assert!(files.len() >= 8, "expected the Book's full sample set, found {}", files.len());
    for f in &files {
        let o = delulu(&["check", &f.to_string_lossy()]);
        assert!(
            o.status.success(),
            "the Book sample `{}` does not compile — a tutorial whose examples fail teaches \
             something false:\n{}",
            f.file_name().unwrap().to_string_lossy(),
            text(&o)
        );
    }
}

/// Chapter 1 ends with `delulu authority` on hello-world, "because that is the identity in one
/// command" (spec §6). So the hello sample must actually produce an authority report — and the
/// report must be the honest one for a program that only writes.
#[test]
fn chapter1_hello_has_a_real_authority_report() {
    let hello = root().join("docs/book/samples/01_hello.delulu");
    let o = delulu(&["authority", &hello.to_string_lossy(), "--json"]);
    assert!(o.status.success(), "authority on hello must succeed: {}", text(&o));

    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&o.stdout)).expect("valid JSON");
    let effects: Vec<&str> =
        v["authority"]["effects"].as_array().unwrap().iter().filter_map(|e| e.as_str()).collect();
    assert_eq!(effects, vec!["Write"], "hello-world writes and does nothing else: {effects:?}");
    assert!(
        v["authority"]["secrets"].as_array().unwrap().is_empty(),
        "hello-world holds no secrets"
    );
}

/// A sample that claims to be pure must BE pure. This is the sample most likely to rot into a lie,
/// because it is the one making the strongest claim.
#[test]
fn the_pure_sample_is_actually_pure() {
    let pure = root().join("docs/book/samples/02_pure_function.delulu");
    let o = delulu(&["authority", &pure.to_string_lossy(), "--json"]);
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&o.stdout)).expect("valid JSON");
    assert!(
        v["authority"]["effects"].as_array().unwrap().is_empty(),
        "the Book says this function cannot read, reach the network, or tell time — the authority \
         report must agree: {}",
        v["authority"]
    );
}

/// Every ```delulu block in the Book has a corresponding sample file. Without this, a new example
/// could be added to the prose and never checked by anything.
#[test]
fn every_book_code_block_has_a_checked_sample() {
    let book = std::fs::read_to_string(root().join("docs/book/THE_DELULULANG_BOOK.md"))
        .expect("the Book is committed");
    let blocks = book.matches("```delulu").count();
    let files = samples().len();
    assert_eq!(
        blocks, files,
        "the Book has {blocks} `delulu` code blocks but {files} checked samples — every example \
         must be backed by a file the compiler actually reads"
    );
}

/// The Book keeps its honesty chapter. This is the section most likely to be trimmed for length,
/// and the one whose absence would change what the project is.
#[test]
fn the_book_keeps_its_honesty_chapter() {
    let book = std::fs::read_to_string(root().join("docs/book/THE_DELULULANG_BOOK.md")).unwrap();
    assert!(
        book.contains("What DeluluLang Refuses to Claim"),
        "the honesty chapter must not be dropped"
    );
}

/// `docs/for-agents.md` is the page machine harnesses pin, so its anchors are a contract. It must
/// also carry the warnings a harness author is most likely to overstate downstream.
#[test]
fn the_agent_page_keeps_its_anchors_and_its_warnings() {
    let p = std::fs::read_to_string(root().join("docs/for-agents.md"))
        .expect("docs/for-agents.md is committed");
    for anchor in [
        "[agents.exit-codes]",
        "[agents.json-envelope]",
        "[agents.diagnostics]",
        "[agents.repairs]",
        "[agents.authority]",
        "[agents.limits]",
    ] {
        assert!(p.contains(anchor), "the stable anchor `{anchor}` must exist");
    }
    // The two things a harness must not get wrong.
    assert!(
        p.contains("DO NOT APPLY AUTOMATICALLY"),
        "the authority-widening warning must be unmissable"
    );
    assert!(
        p.contains("8.5%"),
        "the honest repair-coverage number must be stated, not implied to be universal"
    );
    assert!(
        p.contains("not competitive with C"),
        "the performance position must be stated plainly on the page agents read"
    );
}
