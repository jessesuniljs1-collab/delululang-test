//! Stage 8 close-out (house rule 6): the user guide carries the spec §11 honesty caveats
//! VERBATIM. This meta-test pins the four sentences and asserts each appears character-for-
//! character in BOTH `STAGE8_SPECIFICATION.md` §11 and `STAGE8_SURFACE_GUIDE.md` §7, so the
//! two can never drift.

use std::path::PathBuf;

fn repo(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    std::fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

/// The four spec §11 caveats, verbatim. If the spec's wording changes, change it HERE and
/// in both documents — this test is the tripwire that forces all three to move together.
const CAVEATS: [&str; 4] = [
    "The LSP is analysis-only; it holds no leases and can effect nothing (its process needs no\n  broker connection). Its *availability* is not a security property.",
    "Catalogs are prose: a malicious catalog can mislead a human reader (it cannot alter codes,\n  repairs, or JSON). Catalog plugins are zero-authority and verified-class, which bounds them to\n  exactly this prose surface — stated in `delulu locale add`'s confirmation prompt.",
    "Signing authenticates origin, not behavior (Stage-6 caveat, unchanged). The registry index's\n  authority line is *the publisher's verified-at-publish claim*; consumers re-verify on first\n  build (Stage-2 machinery) — trust-on-first-verify, not trust-on-read.",
    "Test determinism covers clock/rand caps; it does not make foreign code or the OS\n  deterministic.",
];

/// Normalize line-ending + wrap differences: the two documents wrap the same sentences at
/// different column widths, so compare on whitespace-collapsed text.
fn flat(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn the_guide_carries_the_spec_caveats_verbatim() {
    let spec = flat(&repo("docs/design/STAGE8_SPECIFICATION.md"));
    let guide = flat(&repo("docs/design/STAGE8_SURFACE_GUIDE.md"));
    for caveat in CAVEATS {
        let needle = flat(caveat);
        assert!(spec.contains(&needle), "spec §11 must contain the caveat verbatim:\n{caveat}");
        assert!(guide.contains(&needle), "guide §7 must contain the caveat verbatim:\n{caveat}");
    }
}

#[test]
fn the_guide_states_the_no_semantics_invariant() {
    let guide = flat(&repo("docs/design/STAGE8_SURFACE_GUIDE.md"));
    assert!(
        guide.contains("Nothing here changes what programs mean"),
        "the guide must lead with invariant 38"
    );
    // The welcome text is never reproduced in the guide (it is not a machine surface, but
    // the guide is a doc — the law lives in locale.rs, pinned there).
    assert!(!guide.contains("Delulu Gang"), "the guide does not reproduce the welcome text");
}
