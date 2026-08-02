//! Entrenchment: `.github/CODEOWNERS` marks which paths require the project lead specifically.
//!
//! The map's primary reader is a machine maintaining this repository, and *"may I change this?"* is
//! the question it has to answer before *"what breaks if I do?"* is even relevant. These tests hold
//! the answer to being a fact read from a file rather than a habit.

use delulu_survey::Survey;

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn survey() -> Survey {
    Survey::build(&repo_root())
}

/// The constitution, the stability contract, `/rfcs/` and the soundness audit are entrenched by
/// Constitution §10. If the map cannot say so, the attribute is decoration.
#[test]
fn the_entrenched_set_is_marked_and_every_marking_cites_its_line() {
    let s = survey();
    for id in [
        "doc:docs/design/CONSTITUTION.md",
        "doc:docs/design/STABILITY.md",
        "doc:docs/design/DELULU_CORE.md",
        "doc:docs/design/SOUNDNESS_AUDIT.md",
        "doc:SECURITY.md",
        "doc:rfcs/0000-template.md",
        "test:crates/delulu-check/tests/laundering.rs",
        "file:tests/conformance/witnesses.toml",
    ] {
        let n = s.node(id).unwrap_or_else(|| panic!("{id} must exist in the map"));
        let e = n
            .entrenched
            .as_ref()
            .unwrap_or_else(|| panic!("{id} is entrenched by CODEOWNERS and the map does not say so"));
        assert_eq!(e.file, ".github/CODEOWNERS", "the citation names the file it was read from");
        assert!(e.line > 0, "the provenance law does not weaken for an attribute: {id} has no line");
        assert!(!e.owner.is_empty(), "a marking without an owner assigns nothing");
    }
}

/// A directory rule reaches what is beneath it — including the node a reader is most likely to name.
#[test]
fn a_directory_rule_reaches_the_directory_itself_and_its_contents() {
    let s = survey();
    let crate_node = s.node("crate:delulu-conform").expect("the conformance crate is in the map");
    assert!(
        crate_node.entrenched.is_some(),
        "`/crates/delulu-conform/` entrenches the crate, not only the files under it — this is the \
         node an agent asks about by name"
    );
    let module = s.node("mod:crates/delulu-conform/src/rules.rs").expect("module in the map");
    assert!(module.entrenched.is_some(), "a file beneath an entrenched directory is entrenched");
}

/// THE SKIP-BRANCH CASE, and the reason the attribute is worth anything: **most of the repository is
/// not entrenched.** An attribute that marked everything would be indistinguishable from one that
/// marked nothing, and the `*` default in CODEOWNERS would do exactly that if it were honoured.
#[test]
fn ordinary_paths_are_not_entrenched_and_the_catch_all_marks_nothing() {
    let s = survey();
    for id in ["doc:README.md", "doc:CHANGELOG.md", "mod:crates/delulu-check/src/check.rs"] {
        let n = s.node(id).unwrap_or_else(|| panic!("{id} must exist"));
        assert!(
            n.entrenched.is_none(),
            "{id} is matched only by the `*` default; honouring that would entrench the whole tree \
             and the word would stop meaning anything"
        );
    }
    let marked = s.nodes.iter().filter(|n| n.entrenched.is_some()).count();
    assert!(marked > 5, "the entrenched set is not empty: {marked}");
    assert!(
        marked < s.nodes.len() / 10,
        "entrenchment must separate a few paths from the rest, not most of them: {marked} of {}",
        s.nodes.len()
    );
}

/// A rule guarding a path that is not there guards nothing, while reading in a diff exactly like a
/// rule that guards something. `.github/CODEOWNERS` warns about this shape in its own header for a
/// handle that does not exist; the same hazard applies to a pattern that no longer resolves — rename
/// an entrenched file and it is silently un-entrenched.
#[test]
fn a_rule_that_matches_nothing_is_reported_as_an_error() {
    let s = survey();
    assert!(
        !s.findings.iter().any(|f| f.class == "codeowners-rule-guards-nothing"),
        "every committed CODEOWNERS rule must resolve: {:?}",
        s.findings.iter().filter(|f| f.class == "codeowners-rule-guards-nothing").collect::<Vec<_>>()
    );
    // And the check is not vacuous: it is wired to a real class the builder can raise, exercised
    // against a tree where the rule cannot resolve (below).
}

/// The check above proves the committed rules are good. This proves the check would NOTICE if they
/// were not — the same discipline every gate in this repository is held to.
#[test]
fn the_guards_nothing_check_fires_on_a_rule_that_resolves_to_no_path() {
    let dir = std::env::temp_dir().join(format!("delulu-codeowners-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join(".github")).unwrap();
    std::fs::create_dir_all(dir.join("crates/only/src")).unwrap();
    std::fs::write(dir.join("crates/only/src/lib.rs"), "//! a crate\n").unwrap();
    std::fs::write(
        dir.join(".github/CODEOWNERS"),
        "*                        @lead\n/docs/design/GONE.md     @lead\n",
    )
    .unwrap();

    let s = Survey::build(&dir);
    let hit: Vec<_> = s.findings.iter().filter(|f| f.class == "codeowners-rule-guards-nothing").collect();
    assert_eq!(hit.len(), 1, "a rule naming a path that is not there must be reported: {:?}", s.findings);
    assert!(hit[0].message.contains("GONE.md"), "the report names the pattern: {}", hit[0].message);
    assert_eq!(hit[0].line, 2, "and the line it is written on");
    let _ = std::fs::remove_dir_all(&dir);
}
