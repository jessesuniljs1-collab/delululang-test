//! Governance and security artifacts (Stage 9f, spec §4 — release criterion 6).
//!
//! Policy documents rot faster than code, because nothing executes them. These tests are the
//! closest thing to executing them: they check that the artifacts exist, that the commitments the
//! rest of the project leans on are actually written down, and — most importantly — that the
//! things marked `PENDING-PUBLIC` are still marked, rather than having quietly become claims.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel))
        .unwrap_or_else(|e| panic!("{rel} must exist and be readable: {e}"))
}

/// The governance artifacts Constitution §9 requires all exist.
#[test]
fn the_governance_artifacts_exist() {
    for f in [
        "SECURITY.md",
        "CONTRIBUTING.md",
        "docs/design/STABILITY.md",
        "docs/security/DRILL-001.md",
        "rfcs/README.md",
        "rfcs/0000-template.md",
        ".github/CODEOWNERS",
    ] {
        assert!(root().join(f).exists(), "{f} is required by Constitution §9 / spec §4");
    }
}

/// The licensing artifacts (D27) are load-bearing legal files: without them the project reverts to
/// default copyright and nobody may use it. They must never silently vanish, and the machine-readable
/// declaration must agree with the files — a `Cargo.toml` that claims MIT over an Apache LICENSE is a
/// supply-chain lie the same way a wrong SBOM is.
#[test]
fn the_licensing_is_present_and_consistent() {
    for f in ["LICENSE", "NOTICE", "TRADEMARK.md", "GOVERNANCE.md"] {
        assert!(root().join(f).exists(), "{f} is required — its absence reverts the project to default copyright (D27)");
    }
    let license = read("LICENSE");
    assert!(license.contains("Apache License") && license.contains("Version 2.0"), "LICENSE must be Apache-2.0");
    assert!(license.contains("Jesse Sunil"), "the copyright holder must be named in LICENSE");
    // The workspace manifest's machine-readable licence must match the file on disk.
    assert!(
        read("Cargo.toml").contains("license = \"Apache-2.0\""),
        "Cargo.toml must declare the same licence the LICENSE file grants"
    );
    // NOTICE carries the attribution Apache §4(d) propagates; the trademark policy is the rename rule.
    assert!(read("NOTICE").contains("Jesse Sunil"), "NOTICE must record the original creator");
    assert!(
        read("TRADEMARK.md").contains("different name"),
        "the trademark policy must state the derivative-renaming rule (D27, reading A)"
    );
}

/// SECURITY.md carries the substance, not just a heading: a disclosure window, a severity rubric
/// with every level, and the runbook's phases.
#[test]
fn the_security_policy_is_substantive() {
    let s = read("SECURITY.md");
    assert!(s.contains("90 days"), "the coordinated-disclosure default must be stated");
    for sev in ["Critical", "High", "Medium", "Low"] {
        assert!(s.contains(sev), "the severity rubric must define `{sev}`");
    }
    for phase in ["Receipt", "Containment", "Fix", "Release", "Advisory", "Retrospective"] {
        assert!(s.contains(phase), "the patch runbook must include the `{phase}` phase");
    }
    // The rubric must say what is NOT a vulnerability — a policy that only lists what counts wastes
    // reporters' time on the boundaries it never drew.
    assert!(s.contains("not a vulnerability"), "the out-of-scope section must be present");
}

/// CRITERION 6: the runbook has been REHEARSED, and the drill record contains a real timeline and a
/// retrospective. A runbook nobody has executed is a document, not a process.
#[test]
fn criterion6_the_patch_runbook_has_been_rehearsed() {
    let s = read("SECURITY.md");
    assert!(
        s.contains("rehearsed") && s.contains("DRILL-001"),
        "SECURITY.md must link the drill that rehearsed it"
    );

    let d = read("docs/security/DRILL-001.md");
    assert!(d.contains("Timeline"), "the drill must record a timeline");
    assert!(d.contains("staged"), "the drill must state that the vulnerability was staged");
    assert!(
        d.contains("Retrospective") && d.contains("why the tests did not catch"),
        "the drill must answer the only question that matters: why the tests missed it"
    );
    // The drill's finding was that the whole suite passed with a bypass planted. If that sentence
    // ever disappears, the record has been sanitised.
    assert!(
        d.contains("entire 875-test suite passed"),
        "the drill must keep stating that the full suite was blind to the staged vulnerability"
    );
    assert!(
        d.contains("not yet taken"),
        "the drill must keep its open actions open rather than quietly closing them"
    );
}

/// The AI-contribution policy exists and carries the clauses that matter — especially the ones that
/// constrain this project's own way of working.
#[test]
fn the_ai_contribution_policy_binds_this_project_too() {
    let c = read("CONTRIBUTING.md");
    assert!(c.contains("Disclosure is required"), "AI authorship must be disclosed");
    assert!(c.contains("sponsor"), "a named human sponsor must be accountable");
    assert!(c.contains("No unsupervised autonomous PRs"), "autonomy limit must be stated");
    assert!(c.contains("sandboxed CI"), "AI-submitted code runs under the project's own isolation");
    assert!(
        c.contains("advisory only"),
        "overseer monitoring must never hold merge authority — Constitution §9"
    );
}

/// The RFC template keeps the two analyses the constitution requires: irreducibility for core
/// changes, entrenchment for §1/§2/§5.14. These are the sections most likely to be dropped for
/// being inconvenient, which is exactly why they are pinned.
#[test]
fn the_rfc_template_keeps_its_hard_sections() {
    let t = read("rfcs/0000-template.md");
    assert!(t.contains("Irreducibility analysis"), "core changes need the irreducibility test");
    assert!(t.contains("Entrenchment analysis"), "invariant 44 requires the entrenchment section");
    assert!(t.contains("Rejected alternatives"), "rejected alternatives must be recorded");
    assert!(t.contains("skip branch"), "every RFC must answer what happens when the checker cannot tell");
    assert!(t.contains("14 days"), "the comment period must be stated");
}

/// PENDING-PUBLIC items must still be MARKED. This is the test most likely to fail usefully: the
/// natural drift is for a placeholder to quietly become a claim as launch approaches, and a
/// security control that is documented as live but is not is worse than one that is absent.
#[test]
fn pending_public_controls_are_still_marked_as_pending() {
    let s = read("SECURITY.md");
    assert!(
        s.contains("PENDING-PUBLIC"),
        "controls that need public hosting must still be marked PENDING-PUBLIC, not silently \
         upgraded to claims — build-order D2"
    );
    let owners = read(".github/CODEOWNERS");
    assert!(
        owners.contains("PENDING-PUBLIC"),
        "CODEOWNERS must not name a plausible-looking account that does not exist"
    );
    // And the reverse: anything claimed LIVE must be something that actually runs here.
    for live in ["Release artifact signatures", "Reproducible builds", "SBOM"] {
        assert!(s.contains(live), "the controls table must list `{live}`");
    }
}
