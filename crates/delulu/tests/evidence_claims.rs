//! The evidence gate — what this repository may SAY about its own verification is decided by what
//! it CONTAINS (campaign P20, 2026-08-08).
//!
//! # Why this file exists
//!
//! Five statements across three documents denied a result that had been on disk for four days.
//! `docs/QUESTIONS.md` — the page `README.md` sends an evaluator to — said, inside its own block
//! headed *"What is NOT true, and is the honest limit"*: **"Nothing here has been verified by Coq,
//! Lean, Isabelle, or anything else."** `docs/design/models/lean/DeluluCore.lean` had been
//! machine-checking the higher-order fragment since 2026-08-04, with no axioms at all.
//!
//! The correction had already been made — in `README.md`, at commit `7e9c5a3`, **which edited two of
//! the three wrong files in the same commit and corrected the sentence in neither.** The claim lived
//! in five places and exactly one was fixed. Nothing caught the rest, because `docs/QUESTIONS.md`
//! and `docs/MATHEMATICS.md` were referenced by **zero** tests: the only major documents in this
//! repository with no freshness gate of any kind.
//!
//! Two of the five also erred the *understating* way, calling model checking and Z3 "machine-checked
//! evidence" — the exact category conflation `docs/MATHEMATICS.md` §12 forbids, in the documents
//! that exist to tell a reader how strong the evidence is. Understating is still wrong: someone
//! deciding whether to trust this toolchain got a false picture in both directions.
//!
//! # What this gate does differently
//!
//! It does not mirror the claims. It reads `docs/design/models/` — the artifacts themselves — and
//! refuses any shipped document that contradicts what is there. It scans **every** shipped Markdown
//! file rather than a hand-maintained list, because the defect being prevented is precisely a
//! document escaping notice by not being in an array.
//!
//! Sibling gate: `crates/delulu/tests/governance.rs`, over policy documents, which shares both the
//! shape and the reason — *a claim restated by hand in many documents falls out of step with the one
//! that defines it.* Related prose: `docs/MATHEMATICS.md`, `docs/QUESTIONS.md`,
//! `docs/design/PROOF_CAMPAIGN.md`, `docs/design/models/README.md`.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn rel(p: &Path) -> String {
    p.strip_prefix(root()).unwrap_or(p).to_string_lossy().replace('\\', "/")
}

/// Every shipped Markdown document — the prose a reader can actually reach.
///
/// Deliberately **not** a hand-maintained list. Excluded: `target/` and `dist/` (build output and a
/// packaged copy of the tree, which would report every finding twice), `docs/survey/` (generated,
/// and it quotes the documents it maps — the Survey once read its own output and never reached a
/// fixed point), `.claude/` (agent scratch, which has previously held a second checkout at another
/// revision), `.git/` and `node_modules/`.
fn shipped_markdown() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                let r = rel(&p);
                if matches!(
                    r.as_str(),
                    "target" | "dist" | ".git" | "node_modules" | ".claude" | "docs/survey"
                ) || r.ends_with("/node_modules")
                {
                    continue;
                }
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "md") {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    walk(&root(), &mut out);
    out.sort();
    assert!(
        out.len() > 50,
        "the walk found only {} markdown files — it is not reaching the tree, and a scan that \
         reaches nothing passes everything",
        out.len()
    );
    out
}

/// What the tree actually contains, read from disk rather than remembered.
struct Evidence {
    /// `docs/design/models/lean/*.lean` — category 2, machine checked.
    lean: Vec<String>,
    /// `docs/design/models/*.tla` — category 3, model checked.
    tla: Vec<String>,
}

impl Evidence {
    fn read() -> Self {
        let models = root().join("docs/design/models");
        let ext = |dir: PathBuf, want: &str| -> Vec<String> {
            let mut v: Vec<String> = std::fs::read_dir(dir)
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == want))
                .map(|p| rel(&p))
                .collect();
            v.sort();
            v
        };
        Evidence { lean: ext(models.join("lean"), "lean"), tla: ext(models, "tla") }
    }
}

/// Is this occurrence a QUOTATION rather than an assertion?
///
/// **This is the skip branch, and a rule like this dies here.** A document may legitimately quote a
/// false claim in order to record that it was corrected — `CHANGELOG.md` and
/// `docs/design/PROOF_CAMPAIGN.md` both do, and deleting those quotations would erase history this
/// project keeps on purpose. The question is how to tell a quotation from an assertion without
/// guessing.
///
/// Prose markers (`said`, `previously`, `no longer`) were tried first and **failed against this
/// gate's own first draft**: the CHANGELOG entry announcing the fix wrote the phrase inside
/// backticks, and the marker rule read it as an assertion. That is this repository's recurring
/// hazard — *a scan that cannot tell a mention from a use will find its own explanation.*
///
/// So the rule is **structural**: an occurrence is a quotation only if a Markdown code span, a
/// strikethrough, or a quotation mark stands **open** to its left. Anything the rule cannot classify
/// counts as an assertion, because an honesty gate that fails open is not a gate.
///
/// `at` must index the same string that is passed in — see the caller, which searches and slices the
/// *lowercased* line for both, since `to_lowercase` can change byte lengths and the two indices
/// would otherwise disagree on any line containing such a character.
fn is_quoted(line: &str, at: usize) -> bool {
    let before = &line[..at];
    before.matches('`').count() % 2 == 1
        || before.matches("~~").count() % 2 == 1
        || before.matches('"').count() % 2 == 1
        || before.matches('\u{201c}').count() != before.matches('\u{201d}').count()
}

/// Phrases that deny a machine-checked (category 2) result. Contradicted by any `.lean` on disk.
///
/// Kept few and high-signal on purpose. A gate that fires on ambiguous prose becomes noise, and a
/// noisy gate gets deleted or `#[ignore]`d — which is how a check stops checking.
const DENIES_MACHINE_CHECKED: &[&str] = &[
    "no proof assistant is installed",
    "nothing here has been verified by coq",
    "is currently empty",
    "no mechanized proof exists",
];

/// Phrases that deny a model-checked (category 3) result for leases and certificate adoption.
/// Contradicted by `Custody.tla`, which models delegate -> mint -> redeem and adoption directly.
const DENIES_CUSTODY_MODEL_CHECKED: &[&str] =
    &["certificate adoption and federation are not model-checked"];

/// **The load-bearing test.** No shipped document may deny evidence this tree contains.
#[test]
fn no_document_denies_evidence_the_tree_actually_contains() {
    let ev = Evidence::read();
    let mut banned: Vec<(&str, String)> = Vec::new();
    if !ev.lean.is_empty() {
        let why = format!("a Lean development is on disk: {}", ev.lean.join(", "));
        banned.extend(DENIES_MACHINE_CHECKED.iter().map(|p| (*p, why.clone())));
    }
    if ev.tla.iter().any(|p| p.ends_with("Custody.tla")) {
        let why = "docs/design/models/Custody.tla models leases and certificate adoption".to_string();
        banned.extend(DENIES_CUSTODY_MODEL_CHECKED.iter().map(|p| (*p, why.clone())));
    }
    assert!(
        !banned.is_empty(),
        "the evidence inventory came back empty — docs/design/models/ is not being read, and a \
         gate with nothing to compare against passes unconditionally"
    );

    let mut violations = Vec::new();
    for file in shipped_markdown() {
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        for (n, line) in text.lines().enumerate() {
            // Search and slice the SAME string: `to_lowercase` may change byte lengths, so an index
            // taken from the lowercased line cannot be used to slice the original.
            let lower = line.to_lowercase();
            for (phrase, why) in &banned {
                let mut from = 0;
                while let Some(i) = lower[from..].find(phrase) {
                    let at = from + i;
                    if !is_quoted(&lower, at) {
                        violations.push(format!(
                            "{}:{} states \"{phrase}\" as current, but {why}",
                            rel(&file),
                            n + 1
                        ));
                    }
                    from = at + phrase.len();
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "a document denies verification evidence that is on disk. Either the claim is stale and \
         must be corrected, or the artifact was removed and the claim has become true — decide \
         which. Do not silence this:\n  {}",
        violations.join("\n  ")
    );
}

/// **And it fails the other way too, so a claim cannot outlive its artifact.**
///
/// If the Lean development is deleted, every document asserting a machine-checked result becomes
/// false, and this gate must go red rather than stay quiet. The reverse direction is the half that
/// is normally forgotten; this repository has paid for that before (C70/D67 — *"it fails the other
/// way too, so an exemption cannot outlive its fact"*).
#[test]
fn a_machine_checked_claim_cannot_outlive_the_artifact_backing_it() {
    let ev = Evidence::read();
    const CLAIMANTS: &[&str] = &["README.md", "docs/MATHEMATICS.md", "docs/QUESTIONS.md"];
    let mut claims = Vec::new();
    for f in CLAIMANTS {
        let text = std::fs::read_to_string(root().join(f)).unwrap_or_default().to_lowercase();
        if text.contains("machine-checked in lean")
            || text.contains("machine checked in lean")
            || text.contains("machine-checks the")
            || text.contains("machine checked, in lean")
        {
            claims.push(*f);
        }
    }
    if ev.lean.is_empty() {
        assert!(
            claims.is_empty(),
            "no Lean development is on disk, yet these documents still claim one: {claims:?}"
        );
    } else {
        assert!(
            !claims.is_empty(),
            "a Lean development exists ({:?}) and no front-door document mentions it. Evidence \
             this project HAS should not be invisible either — understating was half of the P20 \
             defect",
            ev.lean
        );
    }
}

/// Every formal-model artifact a document names must be a file that exists.
///
/// Catches the drift opposite to the gate above: a claim citing an artifact that was renamed or
/// deleted reads exactly like a claim that is still backed by one.
#[test]
fn every_cited_model_artifact_exists() {
    // The append-only dated ledgers are records of what was true when written. If an artifact is
    // renamed, updating a LIVING document that points a reader at it is correct; forcing a rewrite
    // of a historical CHANGELOG entry is the same error this campaign refused for the cli-sweep
    // counts. So those ledgers are scanned by the denial gate (which has the quotation skip-branch)
    // but exempted here.
    const HISTORICAL: &[&str] = &[
        "CHANGELOG.md",
        "docs/design/PROOF_CAMPAIGN.md",
        "docs/design/HARDENING_CAMPAIGN.md",
        "docs/archive/v1/design/P19_ECOSYSTEM_REVIEW.md",
    ];
    let mut missing = Vec::new();
    for file in shipped_markdown() {
        if HISTORICAL.contains(&rel(&file).as_str()) {
            continue;
        }
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        for (n, line) in text.lines().enumerate() {
            for tok in line.split(|c: char| !(c.is_alphanumeric() || "._/-".contains(c))) {
                // A citation is `Name.lean`, not the bare extension `.lean` — prose describing the
                // gate itself writes the latter, and reading it as a citation is the "a scan that
                // cannot tell a mention from a use finds its own explanation" hazard, self-inflicted.
                let stem = tok.strip_suffix(".lean").or_else(|| tok.strip_suffix(".tla"));
                let Some(stem) = stem else { continue };
                if !stem.chars().next_back().is_some_and(|c| c.is_alphanumeric()) {
                    continue;
                }
                let base = tok.rsplit('/').next().unwrap_or(tok);
                let found = ["docs/design/models", "docs/design/models/lean"]
                    .iter()
                    .any(|d| root().join(d).join(base).exists());
                if !found {
                    missing.push(format!(
                        "{}:{} cites `{tok}`, which is not in docs/design/models/",
                        rel(&file),
                        n + 1
                    ));
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "a document cites a formal-model artifact that does not exist:\n  {}",
        missing.join("\n  ")
    );
}
