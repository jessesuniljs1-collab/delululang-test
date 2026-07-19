//! Tests for the coverage tool itself (Stage 9a, house rule 3 — every check has its skip-branch
//! case written before the check is trusted).

use super::*;
use std::sync::atomic::{AtomicU32, Ordering};

// ---------- unit tests: the parsers and probes ----------

#[test]
fn registry_is_non_empty_and_spans_every_category() {
    let reg = Registry::build();
    assert!(!reg.anchors.is_empty(), "registry must never be empty (no vacuous pass)");
    let cats: BTreeSet<Category> = reg.anchors.values().copied().collect();
    for c in Category::all() {
        assert!(cats.contains(&c), "registry is missing category {}", c.as_str());
    }
    // The diagnostic registry is mirrored one-for-one.
    let diag = reg.anchors.values().filter(|c| **c == Category::Diag).count();
    assert_eq!(diag, delulu_diag::REGISTRY.len(), "every diagnostic code is an anchor");
}

#[test]
fn header_anchors_reads_only_the_leading_comment_block() {
    let src = "// anchors: ref.grammar.fn ref.diag.DL0501\n// anchors: ref.cli.check\nmodule m\n// anchors: ref.diag.DL9999\n";
    let a = header_anchors(src);
    assert_eq!(a, vec!["ref.grammar.fn", "ref.diag.DL0501", "ref.cli.check"]);
}

#[test]
fn reject_file_code_matches_the_conformance_runner() {
    assert_eq!(reject_file_code(Path::new("DL0501_undeclared_effect.delulu")).as_deref(), Some("DL0501"));
    assert_eq!(reject_file_code(Path::new("notacode.delulu")), None);
    assert_eq!(reject_file_code(Path::new("DL05_short.delulu")), None);
}

#[test]
fn parse_witnesses_accepts_a_well_formed_record() {
    let text = "# comment\n[[witness]]\nanchor = \"ref.diag.DL0501\"\nfile = \"crates/x/src/y.rs\"\ntest = \"t\"\nkind = \"negative\"\n";
    let recs = parse_witnesses(text).expect("valid");
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].anchor, "ref.diag.DL0501");
    assert!(!recs[0].positive);
}

#[test]
fn parse_witnesses_is_fail_closed_on_bad_metadata() {
    // Missing `kind`.
    let missing = "[[witness]]\nanchor = \"a\"\nfile = \"f\"\ntest = \"t\"\n";
    assert!(parse_witnesses(missing).is_err(), "a missing field must be an error");
    // Unknown kind.
    let badkind = "[[witness]]\nanchor = \"a\"\nfile = \"f\"\ntest = \"t\"\nkind = \"maybe\"\n";
    assert!(parse_witnesses(badkind).is_err(), "an unknown kind must be an error");
    // Garbage line.
    let garbage = "[[witness]]\nthis is not a key value line\n";
    assert!(parse_witnesses(garbage).is_err(), "an unparseable line must be an error");
    // Key before any table.
    let orphan = "anchor = \"a\"\n";
    assert!(parse_witnesses(orphan).is_err(), "a key before [[witness]] must be an error");
}

#[test]
fn test_presence_detects_active_ignored_and_absent() {
    let src = "#[test]\nfn active_one() {}\n\n#[ignore]\n#[test]\nfn ignored_one() {}\n";
    assert_eq!(test_presence(src, "active_one"), TestPresence::Active);
    assert_eq!(test_presence(src, "ignored_one"), TestPresence::Ignored);
    assert_eq!(test_presence(src, "missing_one"), TestPresence::Absent);
}

#[test]
fn test_presence_does_not_match_a_name_prefix() {
    // `foo` must not match `foo_bar` (identifier-boundary check).
    let src = "fn foo_bar() {}\n";
    assert_eq!(test_presence(src, "foo"), TestPresence::Absent);
    assert_eq!(test_presence(src, "foo_bar"), TestPresence::Active);
}

#[test]
fn empty_registry_never_passes() {
    // Skip-branch: a checker with nothing to check must FAIL, never vacuously pass.
    let cov = Coverage {
        registry: Registry { anchors: BTreeMap::new() },
        witnesses: Witnesses::default(),
        gaps: Vec::new(),
        validation_errors: Vec::new(),
    };
    assert!(!cov.pass(), "an empty registry must not pass");
}

// ---------- fixture-based integration tests ----------

static FIXTURE_SEQ: AtomicU32 = AtomicU32::new(0);

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new() -> Fixture {
        let n = FIXTURE_SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!("delulu-conform-{}-{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("tests/conformance/accept")).unwrap();
        std::fs::create_dir_all(root.join("tests/conformance/reject")).unwrap();
        std::fs::create_dir_all(root.join("tests/corpus")).unwrap();
        Fixture { root }
    }
    fn write(&self, rel: &str, content: &str) {
        let p = self.root.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(p, content).unwrap();
    }
    fn run(&self) -> Coverage {
        run_coverage(&self.root)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn synthetic_gap_is_detected() {
    // No witnesses at all: every anchor is a gap, and the run must FAIL.
    let fx = Fixture::new();
    let cov = fx.run();
    assert!(!cov.pass());
    assert_eq!(cov.gaps.len(), cov.total(), "with no witnesses, every anchor is a gap");
    assert!(cov.validation_errors.is_empty(), "no witnesses is not itself an error");
}

#[test]
fn header_and_reject_filename_close_an_anchor() {
    let fx = Fixture::new();
    // Positive via an accept header; negative via a reject header (and the reject filename covers
    // the DL0101 diagnostic anchor's negative side automatically).
    fx.write("tests/conformance/accept/ok.delulu", "// anchors: ref.cli.check\nmodule m\n");
    fx.write("tests/conformance/reject/DL0101_x.delulu", "// anchors: ref.cli.check\nmodule m\n");
    let cov = fx.run();
    assert!(cov.validation_errors.is_empty(), "{:?}", cov.validation_errors);
    let gap_anchors: BTreeSet<&str> = cov.gaps.iter().map(|g| g.anchor.as_str()).collect();
    assert!(!gap_anchors.contains("ref.cli.check"), "ref.cli.check has both witnesses now");
    // The reject filename gave the negative for DL0101; it still lacks a positive → still a gap,
    // but a gap missing ONLY the positive side.
    let d = cov.gaps.iter().find(|g| g.anchor == "ref.diag.DL0101").expect("DL0101 still a gap");
    assert!(d.missing_positive && !d.missing_negative, "reject file supplies the negative side");
}

#[test]
fn unknown_anchor_citation_is_a_validation_error() {
    // Skip-branch: a citation the registry does not know must FAIL, not silently pass.
    let fx = Fixture::new();
    fx.write("tests/conformance/accept/bad.delulu", "// anchors: ref.bogus.nope\nmodule m\n");
    let cov = fx.run();
    assert!(!cov.pass());
    assert!(
        cov.validation_errors.iter().any(|e| e.contains("ref.bogus.nope")),
        "unknown anchor must be reported: {:?}",
        cov.validation_errors
    );
}

#[test]
fn dangling_rust_witness_is_a_validation_error() {
    // Skip-branch: a witness naming a test that does not exist must FAIL.
    let fx = Fixture::new();
    fx.write("crates/fake/src/x.rs", "fn some_other_test() {}\n");
    fx.write(
        "tests/conformance/witnesses.toml",
        "[[witness]]\nanchor = \"ref.cli.check\"\nfile = \"crates/fake/src/x.rs\"\ntest = \"does_not_exist\"\nkind = \"positive\"\n",
    );
    let cov = fx.run();
    assert!(!cov.pass());
    assert!(
        cov.validation_errors.iter().any(|e| e.contains("nonexistent test")),
        "dangling witness must be reported: {:?}",
        cov.validation_errors
    );
}

#[test]
fn ignored_rust_witness_is_a_validation_error() {
    // Skip-branch: an `#[ignore]`d test is an inactive witness and must FAIL.
    let fx = Fixture::new();
    fx.write("crates/fake/src/x.rs", "#[ignore]\n#[test]\nfn sleeping() {}\n");
    fx.write(
        "tests/conformance/witnesses.toml",
        "[[witness]]\nanchor = \"ref.cli.check\"\nfile = \"crates/fake/src/x.rs\"\ntest = \"sleeping\"\nkind = \"positive\"\n",
    );
    let cov = fx.run();
    assert!(!cov.pass());
    assert!(
        cov.validation_errors.iter().any(|e| e.contains("ignore")),
        "ignored witness must be reported: {:?}",
        cov.validation_errors
    );
}

#[test]
fn active_rust_witness_closes_the_polarity() {
    let fx = Fixture::new();
    fx.write("crates/fake/src/x.rs", "#[test]\nfn produces_check() {}\n#[test]\nfn accepts_check() {}\n");
    fx.write(
        "tests/conformance/witnesses.toml",
        "[[witness]]\nanchor = \"ref.cli.check\"\nfile = \"crates/fake/src/x.rs\"\ntest = \"accepts_check\"\nkind = \"positive\"\n\
         [[witness]]\nanchor = \"ref.cli.check\"\nfile = \"crates/fake/src/x.rs\"\ntest = \"produces_check\"\nkind = \"negative\"\n",
    );
    let cov = fx.run();
    assert!(cov.validation_errors.is_empty(), "{:?}", cov.validation_errors);
    let gap_anchors: BTreeSet<&str> = cov.gaps.iter().map(|g| g.anchor.as_str()).collect();
    assert!(!gap_anchors.contains("ref.cli.check"), "both polarities present via Rust witnesses");
}

#[test]
fn malformed_witness_file_fails_the_run() {
    let fx = Fixture::new();
    fx.write("tests/conformance/witnesses.toml", "[[witness]]\nthis line is not parseable\n");
    let cov = fx.run();
    assert!(!cov.pass());
    assert!(!cov.validation_errors.is_empty());
}

// ---------- drift guards for the two consts owned by this crate ----------

#[test]
fn cli_subcommands_match_the_dispatch() {
    // Every declared subcommand appears as a match arm in the CLI dispatch. Renaming/removing a
    // subcommand without updating CLI_SUBCOMMANDS fails here.
    let cli = include_str!("../../delulu/src/cli.rs");
    for s in CLI_SUBCOMMANDS {
        let arm = format!("\"{s}\" =>");
        assert!(cli.contains(&arm), "CLI subcommand `{s}` has no `{arm}` in cli.rs — the list drifted");
    }
    // Skip-branch: the guard must be able to fail (a bogus subcommand is not a dispatch arm).
    assert!(!cli.contains("\"definitely-not-a-subcommand-9a\" =>"));
}

#[test]
fn audit_rules_appear_in_the_soundness_audit() {
    let doc = include_str!("../../../docs/design/SOUNDNESS_AUDIT.md");
    for r in AUDIT_RULES {
        assert!(doc.contains(r), "audit rule `{r}` not found in SOUNDNESS_AUDIT.md");
    }
}

// ---------- the real repository ----------

/// THE RATCHET. Coverage of the committed repo may only ever RISE. This is the gate that makes
/// the coverage law bite before it reaches 100%: a change that drops a witness, deletes a
/// conformance program, or adds an unwitnessed anchor fails here immediately.
///
/// Raise it when you add witnesses. It must never need lowering — a lowered floor in a diff is a
/// coverage regression wearing a disguise, and reviewing this constant is how you catch it.
const COVERED_FLOOR: usize = 263;

/// The committed repo's witnesses are internally valid (no dangling/ignored/unknown citations)
/// and coverage has not regressed.
#[test]
fn coverage_never_regresses() {
    let cov = run_coverage(&repo_root());
    assert!(!cov.registry.anchors.is_empty(), "registry built from the real source");
    assert!(
        cov.validation_errors.is_empty(),
        "the committed witnesses/headers must be clean: {:?}",
        cov.validation_errors
    );
    assert!(
        cov.covered() >= COVERED_FLOOR,
        "coverage REGRESSED: {} covered, floor is {COVERED_FLOOR}. Restore the missing witness \
         rather than lowering the floor.",
        cov.covered()
    );
}

/// RELEASE CRITERION 1: the 1.0 cut requires 100% anchor coverage. `#[ignore]`d until the gaps
/// classified in `STAGE9_BUILD_ORDER.md` D10 are closed — the honest state is 216/233, and this
/// test is what flips the claim from "ratcheting" to "complete".
#[test]
#[ignore = "release criterion 1: coverage is 263/287 — see STAGE9_BUILD_ORDER.md D10 for the classified remainder"]
fn release_requires_full_coverage() {
    let cov = run_coverage(&repo_root());
    assert!(cov.pass(), "expected 100% coverage, {} gap(s) remain", cov.gaps.len());
}

// ---------- the generated reference (Stage 9b) ----------

/// THE ANTI-ROT GATE: the committed reference must equal what the current compiler source
/// produces. A change to the grammar, the primitive table, the diagnostics registry or the rule
/// index that would stale a chapter fails HERE, at the moment it is made — which is the only
/// reliable time to catch it.
#[test]
fn the_generated_reference_is_not_stale() {
    let stale = crate::reference::drift(&repo_root());
    assert!(
        stale.is_empty(),
        "the generated reference is stale — run `cargo run -p delulu-conform -- --reference`:\n  {}",
        stale.join("\n  ")
    );
}

/// Every generated chapter carries the DO-NOT-EDIT banner naming its source of truth, so someone
/// opening one to "just fix a typo" is told where the text actually comes from.
#[test]
fn every_generated_chapter_declares_itself_generated() {
    for (path, content) in crate::reference::generate(&repo_root()) {
        assert!(
            content.starts_with("<!-- GENERATED FILE"),
            "{path} does not open with the generated-file banner"
        );
        assert!(
            content.contains("--check-reference"),
            "{path} does not say how staleness is detected"
        );
    }
}

/// THE SKIP-BRANCH CASE (house rule 3): the drift gate must be able to FAIL. If `drift` returned
/// empty for content that plainly differs, `the_generated_reference_is_not_stale` would be
/// vacuous — a green light wired to nothing.
#[test]
fn the_drift_gate_detects_a_modified_chapter() {
    let fx = Fixture::new();
    // An empty fixture root has no docs/reference/ at all: every chapter is missing, and missing
    // must be reported as drift rather than skipped.
    let stale = crate::reference::drift(&fx.root);
    assert!(!stale.is_empty(), "a repo with no reference at all must report drift");
    assert!(
        stale.iter().all(|s| s.contains("missing")),
        "missing chapters must be named as missing: {stale:?}"
    );

    // Now write one chapter with the wrong content: it must be reported as differing, not as fine.
    let generated = crate::reference::generate(&fx.root);
    let (first, _) = generated.iter().next().expect("at least one chapter");
    fx.write(first, "not what the generator produces\n");
    let stale2 = crate::reference::drift(&fx.root);
    assert!(
        stale2.iter().any(|s| s.starts_with(first) && s.contains("differs")),
        "a modified chapter must be reported as differing: {stale2:?}"
    );
}

/// Rule coverage is DERIVED, and derivation must be strict: a rule whose enforcing code has no
/// producing test is not covered. Proven against the live data rather than asserted in prose.
#[test]
fn a_rule_is_only_covered_when_every_enforcing_code_is() {
    let cov = run_coverage(&repo_root());
    let uncovered_codes: BTreeSet<String> = cov
        .gaps
        .iter()
        .filter(|g| g.category == Category::Diag)
        .map(|g| g.anchor.trim_start_matches("ref.diag.").to_string())
        .collect();
    assert!(!uncovered_codes.is_empty(), "this test needs at least one uncovered code to be meaningful");

    for rule in crate::rules::RULES {
        let leans_on_a_gap = rule.enforced_by.iter().any(|c| uncovered_codes.contains(*c));
        let rule_is_a_gap = cov.gaps.iter().any(|g| g.anchor == rule.anchor);
        assert_eq!(
            leans_on_a_gap, rule_is_a_gap,
            "rule `{}` enforced by {:?}: covered-status must follow its codes exactly",
            rule.anchor, rule.enforced_by
        );
    }
}
