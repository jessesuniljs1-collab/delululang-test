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
        p.contains("8.3%"),
        "the honest repair-coverage number must be stated, not implied to be universal"
    );
    assert!(
        p.contains("not competitive with C"),
        "the performance position must be stated plainly on the page agents read"
    );
}

/// The Book's Chapter 9 quotes a number for the two-engine parity fuzzer, and a number in prose is
/// exactly the kind of claim that rots (campaign finding C63, ruling D58).
///
/// **What it said before D58:** parity was "enforced by a differential fuzzer running tens of
/// thousands of programs on both engines", and "two independent implementations agree on 50,000
/// random programs". Neither was true. The generative two-engine harness runs **2,000** programs, all
/// inside the WASM fragment; and `delulu-fuzz` — the crate actually named the differential fuzz
/// harness — depends on `delulu-check` and `delulu-runtime` and *cannot run the WASM backend at all*.
/// What it proves is effect soundness, which is a better claim credited to the wrong property.
///
/// So this reads the loop bound out of the parity test itself and requires the Book to agree with it.
/// Change the harness and this fails until the prose is changed too.
#[test]
fn the_books_engine_parity_number_matches_the_harness() {
    let harness = std::fs::read_to_string(
        root().join("crates/delulu-wasm/tests/conformance_parity.rs"),
    )
    .expect("the parity harness must exist — the Book's claim rests on it");

    // `for k in 0..2000u64 {` — the generative two-engine sweep.
    let bound = harness
        .split("for k in 0..")
        .nth(1)
        .and_then(|rest| rest.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|n| n.parse::<u32>().ok())
        .expect("could not read the parity sweep's loop bound from the harness");

    let book = std::fs::read_to_string(root().join("docs/book/THE_DELULULANG_BOOK.md")).unwrap();

    // Written with a thousands separator in prose, as the Book does elsewhere.
    let pretty = format!("{},{:03}", bound / 1000, bound % 1000);
    assert!(
        book.contains(&pretty),
        "the parity harness runs {pretty} programs and Chapter 9 does not say so — a number in prose \
         that no test reads is the shape C63 was"
    );

    // And the two overstatements must never come back.
    for stale in ["50,000 random programs", "tens of thousands of programs on both engines"] {
        assert!(
            !book.contains(stale),
            "Chapter 9 claims {stale:?} again — the real sweep is {pretty} programs, inside the fragment"
        );
    }

    // The fragment itself must stay disclosed: the honest half of the correction is that a reader
    // is told the WASM backend is a SUBSET, not a second full implementation.
    assert!(
        book.contains("DL1201"),
        "Chapter 9 must name the refusal code that marks the fragment boundary"
    );
}

/// The other half of C63: `delulu-fuzz` must not silently become a WASM harness (which would make
/// the corrected prose wrong in the other direction), and must not stop being one either without
/// somebody noticing this assertion.
#[test]
fn the_differential_fuzz_crate_still_does_not_run_the_wasm_backend() {
    let manifest = std::fs::read_to_string(root().join("crates/delulu-fuzz/Cargo.toml")).unwrap();
    assert!(
        !manifest.contains("delulu-wasm"),
        "delulu-fuzz now depends on delulu-wasm — Chapter 9 says it cannot run the WASM backend, so \
         either the dependency is wrong or the Book is out of date"
    );
}

/// Normalize a code block or sample for comparison: drop blank lines, comments and trailing
/// whitespace, so a slice matches its source regardless of surrounding prose or comment edits.
fn normalize_code(s: &str) -> String {
    s.lines()
        .map(|l| l.trim_end())
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn book_delulu_blocks() -> Vec<String> {
    let book = std::fs::read_to_string(root().join("docs/book/THE_DELULULANG_BOOK.md")).unwrap();
    let mut out = Vec::new();
    let mut rest = book.as_str();
    while let Some(i) = rest.find("```delulu\n") {
        rest = &rest[i + "```delulu\n".len()..];
        if let Some(j) = rest.find("```") {
            out.push(rest[..j].to_string());
            rest = &rest[j..];
        }
    }
    out
}

/// C68. `every_book_code_block_has_a_checked_sample` compares the NUMBER of `delulu` code blocks to
/// the number of sample files. It never compared their contents — so a block could show anything at
/// all and the gate stayed green as long as the counts matched. Measured when this was written:
/// **0 of 10** blocks were slices of any compiled sample. They were paraphrases.
///
/// It had a live consequence. Chapter 14 — "Real adoption means calling C and Python" — showed
/// `root.foreign[mathlib](root.foreign_load())?`, which does not compile: `Root` has no field
/// `foreign` and `mathlib` is a type, not a value. The one chapter a developer reads to call C taught
/// a form the compiler rejects, and the sample file backing it contained only the `foreign`
/// declaration — the safe half, with no call site at all. Existence checked, correspondence not:
/// C37's shape, in the front door.
///
/// This is the correspondence check. It is deliberately scoped to the blocks that have been made
/// literal slices so far rather than asserted over all ten, because the honest state is partial —
/// see the `BACKED` list, which is meant to grow. A gate that claims more than it checks is the thing
/// being fixed here, so this one says exactly what it covers.
#[test]
fn book_blocks_that_claim_to_be_compiled_really_are() {
    /// Blocks required to be a literal slice of a compiled sample, by a distinctive first line.
    /// Add to this as blocks are converted from paraphrase to excerpt.
    const BACKED: &[(&str, &str)] = &[("foreign \"c\" lib mathlib {", "08_foreign.delulu")];

    let blocks = book_delulu_blocks();
    assert!(blocks.len() >= 10, "the Book lost code blocks: found {}", blocks.len());

    for (marker, sample) in BACKED {
        let block = blocks
            .iter()
            .find(|b| b.contains(marker))
            .unwrap_or_else(|| panic!("no Book block contains {marker:?} any more"));
        let src = std::fs::read_to_string(root().join("docs/book/samples").join(sample))
            .unwrap_or_else(|e| panic!("{sample} must exist: {e}"));
        assert!(
            normalize_code(&src).contains(&normalize_code(block)),
            "the Book block starting {marker:?} is NOT a slice of {sample}, so nothing compiles what \
             the reader copies. Book block:\n{block}\n--- sample:\n{src}"
        );
    }

    // The specific regression that motivated all of this must never come back — checked in the CODE
    // BLOCKS only. The prose quotes the broken form on purpose, to explain what was wrong with it;
    // a gate that cannot tell an example from a description of a former example would forbid the
    // project from documenting its own corrections.
    for b in &blocks {
        assert!(
            !b.contains("root.foreign["),
            "a Book code block shows `root.foreign[...]` again — that syntax does not exist; the lib \
             type is inferred from an annotated parameter:\n{b}"
        );
    }
}
