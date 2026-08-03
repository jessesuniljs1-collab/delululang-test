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

/// The contribution policy exists and carries the clauses that matter — especially the ones that
/// constrain this project's own way of working.
///
/// The clauses changed on 2026-08-03 by ruling of the project lead: **anyone may maintain and
/// develop DeluluLang — human, AI, or any other kind of party** — and every rule now keys on the
/// CHANGE rather than on who or what authored it. The substance was not weakened; each kind-blind
/// clause is stricter, because a rule applied to one kind of author leaves the rest unexamined.
#[test]
fn the_contribution_policy_binds_this_project_too() {
    let c = read("CONTRIBUTING.md");
    assert!(c.contains("names its author"), "authorship must be attributed — for everyone");
    assert!(c.contains("sponsor"), "a named sponsor must be accountable");
    assert!(
        c.contains("not the\n   author") || c.contains("not the author") || c.contains("sponsor is not"),
        "the sponsor must be someone other than the author — that is the independence the rule buys"
    );
    assert!(
        c.contains("say-so of its own author"),
        "no change may merge on the say-so of its own author"
    );
    assert!(
        c.contains("Stage-5 isolation profiles"),
        "ALL contributed code runs under the project's own isolation"
    );
    assert!(
        c.contains("never merge one"),
        "unattended automation must never hold merge authority — Constitution §9"
    );
}

/// **No governance document may reintroduce a kind-of-party trust hierarchy.**
///
/// Constitution invariant 24 rejects such hierarchies as *"discriminatory and fragile"*, and no
/// decision path in the Guard reads the kind of a grant holder. For a long time the governance
/// documents did the opposite anyway — the policy was restated in **six** places and five of them
/// said *"a named **human** sponsor per **AI-authored** PR"*, so the constitution contradicted its
/// own invariant. Fixing one file would have left the other five to drift back.
///
/// This is the campaign's recurring shape once more: a claim restated by hand in many documents
/// falls out of step with the one that defines it. So the gate reads **all** of them, and fails on
/// the phrasings that encode the hierarchy rather than trusting any single file to stay correct.
#[test]
fn no_governance_document_makes_a_rule_depend_on_the_kind_of_party() {
    // Phrases that only exist to make a REQUIREMENT apply to one kind of author.
    const BANNED: &[&str] = &[
        "named human sponsor",
        "human sponsor accountable",
        "No unsupervised autonomous PRs",
        "AI-submitted code runs only",
        "A human decides it is submitted",
    ];
    // Where the policy is stated or summarised. A new restatement belongs in this list.
    const RESTATED_IN: &[&str] = &[
        "CONTRIBUTING.md",
        "GOVERNANCE.md",
        "docs/design/CONSTITUTION.md",
        "docs/design/LANGUAGE_SPECIFICATION.md",
    ];

    let mut violations: Vec<String> = Vec::new();
    for file in RESTATED_IN {
        let text = read(file);
        for phrase in BANNED {
            // A document may QUOTE the old wording to record that it changed; what it may not do is
            // state it as current. The historical form always names the date it stopped being true.
            for (n, line) in text.lines().enumerate() {
                if line.contains(phrase) && !line.contains("2026-08-03") && !line.contains("previously") && !line.contains("used to") && !line.contains("Until") {
                    violations.push(format!("{file}:{} states `{phrase}` as current", n + 1));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "a governance rule may not depend on the KIND of the party it applies to \
         (Constitution invariant 24):\n  {}",
        violations.join("\n  ")
    );

    // And the positive direction: the welcome must actually be stated, not merely implied by the
    // absence of a prohibition.
    for file in ["CONTRIBUTING.md", "GOVERNANCE.md", "docs/design/CONSTITUTION.md"] {
        let text = read(file);
        assert!(
            text.contains("human, AI, or any other kind of party"),
            "{file} must state plainly who may contribute and maintain"
        );
    }
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

/// `.gitattributes` (ruling D19d) declares `* text=auto eol=lf`, on the stated evidence that every
/// tracked text file was already stored LF — "472/472 LF, zero CRLF". That was true when written.
///
/// **Nothing checked it afterwards, and it stopped being true.** By 2026-07-26 exactly one tracked
/// file was stored with CRLF in its committed blob: `docs/design/HARDENING_CAMPAIGN.md`, created
/// three days AFTER the attribute was adopted and rewritten by tooling on nearly every phase of the
/// campaign. It went unnoticed for the whole campaign because a `.gitattributes` line is a
/// declaration, not a gate, and the file's own §6 lists "a gate is blind to the failure it exists to
/// catch" as one of the two rules the campaign produced. Here there was no gate at all.
///
/// So this is the gate. It reads the INDEX — the bytes about to be committed — and not the working
/// tree: `eol=lf` normalizes on the way in, so a working copy may legitimately hold CRLF on a
/// machine with `core.autocrlf=true` while the repository stays clean. What must never contain a CR
/// is what gets stored. After a fresh checkout the index matches `HEAD`, so on CI this is exactly a
/// statement about the committed tree.
#[test]
fn no_tracked_text_file_is_stored_with_crlf() {
    // `-I` skips anything git considers binary, which is what keeps the signed release artifacts
    // (`*.dwx`, `*.sig`, pinned `-text` in .gitattributes) out of this — their bytes are a
    // signature's subject and nothing may normalize them.
    let out = std::process::Command::new("git")
        .current_dir(root())
        .args(["grep", "--files-with-matches", "--cached", "-I", "\r"])
        .output();
    let Ok(out) = out else {
        // A source tarball has no git. The invariant is about the repository, so there is nothing
        // to check here and nothing to fail — but say so rather than passing silently.
        eprintln!("note: `git` unavailable, line-ending invariant not checked");
        return;
    };
    let offenders: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    assert!(
        offenders.is_empty(),
        "`.gitattributes` declares `* text=auto eol=lf`, and these tracked files are stored with \
         CRLF in the committed blob: {offenders:#?}\n\nRenormalize them (`git add --renormalize \
         <path>`) — a declared invariant that only holds by luck is the shape ruling D19d meant to \
         end."
    );
}

/// **Every crate declares what it is, and only the CLI may be published.**
///
/// `STABILITY.md` §2 says plainly that *"the Rust crates are an implementation detail; the stable
/// interface is the language, the CLI, and the machine schemas."* That promise had no mechanism:
/// twelve of thirteen crates defaulted to publishable, at `version = "1.0.0"`, so a single
/// `cargo publish -p delulu-check` would have minted a semver contract over seventeen public
/// modules the stability document explicitly disclaims. `STABILITY.md`'s own §6 is a
/// promise-to-mechanism table; this test is the row that was missing.
///
/// The CLI is the exception because it is the only crate that could ever *be* a distributed
/// artifact — **not** because it is distributed. Nothing here is: there is no crates.io entry, and
/// the CLI's own path dependencies carry no version numbers, so `cargo publish` would refuse it
/// regardless. This gate prevents an accident; it does not preserve an install path that exists.
///
/// **`surface` and `publish` are deliberately two keys.** They answer different questions — *is this
/// part of the language product* versus *may this go to crates.io* — and the Survey's "shipped
/// crates" count rode on `publish` as a proxy for years because a single crate happened to answer
/// no to both. A count derived from a proxy is a measurement waiting to be wrong.
#[test]
fn every_crate_declares_its_surface_and_only_the_cli_publishes() {
    let crates_dir = root().join("crates");
    let mut checked = 0;
    let mut publishable: Vec<String> = Vec::new();
    let mut undeclared: Vec<String> = Vec::new();
    let mut language = 0;
    let mut tooling = 0;

    for entry in std::fs::read_dir(&crates_dir).expect("crates/ is readable").flatten() {
        let manifest = entry.path().join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let text = std::fs::read_to_string(&manifest).expect("manifest is readable");
        checked += 1;

        let unpublishable = text.lines().any(|l| l.trim().starts_with("publish") && l.contains("false"));
        if !unpublishable && name != "delulu" {
            publishable.push(name.clone());
        }
        if unpublishable && name == "delulu" {
            panic!("the CLI must stay publishable: `cargo install delulu` is how anyone gets it");
        }

        match text.lines().find_map(|l| l.trim().strip_prefix("surface = ").map(|v| v.trim().trim_matches('"').to_string())) {
            Some(s) if s == "language" => language += 1,
            Some(s) if s == "tooling" => tooling += 1,
            Some(other) => panic!("{name} declares an unknown surface `{other}` — it is \"language\" or \"tooling\""),
            None => undeclared.push(name),
        }
    }

    assert!(checked >= 13, "the crate sweep found too few manifests to be right: {checked}");
    assert!(
        publishable.is_empty(),
        "these crates would be uploaded to crates.io at 1.0.0, minting a semver contract over \
         internals `STABILITY.md` §2 disclaims: {publishable:?}\n\
         Add `publish = false`. If a crate is genuinely meant to be a library other people depend \
         on, that is a change to the stability contract and needs a ruling, not a manifest edit."
    );
    assert!(
        undeclared.is_empty(),
        "these crates declare no `[package.metadata.delulu] surface`: {undeclared:?}\n\
         Every crate is either the language product or repository tooling, and the Survey's shipped \
         count is derived from the answer. Leaving it out is how that count came to ride on \
         `publish` instead."
    );
    assert!(language > 0 && tooling > 0, "both surfaces exist: {language} language, {tooling} tooling");
}
