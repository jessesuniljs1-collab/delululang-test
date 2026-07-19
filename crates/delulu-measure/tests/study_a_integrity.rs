//! Study A's integrity gates (Stage 9d, release criterion 3).
//!
//! These do not re-run the campaign — the binary does that. They check the properties that make a
//! published number believable: that the scoring cannot be fooled, that a broken mutation is not
//! counted as a catch, that a campaign which catches everything is rejected as uninformative, and
//! that the committed results actually say what the report says.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn results() -> serde_json::Value {
    let p = repo_root().join("measurements/study-a/results.json");
    let raw = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("Study A results are committed at {}: {e}", p.display()));
    serde_json::from_str(&raw).expect("results.json is valid JSON")
}

/// RELEASE CRITERION 3: Study A's injection catch rate is 100% over a non-empty set, every
/// mutation compiled, and the negative controls passed.
#[test]
fn criterion3_the_committed_results_show_a_sound_100_percent() {
    let r = results();
    let inj = &r["injection"];
    let sites = inj["sites"].as_u64().expect("sites");
    assert!(sites >= 20, "expected the full campaign, found {sites} sites");
    assert_eq!(inj["caught"].as_u64(), Some(sites), "every site must be caught");
    assert_eq!(inj["catch_rate_pct"].as_f64(), Some(100.0));
    assert_eq!(
        inj["invalid_mutations"].as_array().map(|a| a.len()),
        Some(0),
        "a mutation that did not compile invalidates the run"
    );
    assert_eq!(
        r["negative_controls"]["all_clean"].as_bool(),
        Some(true),
        "the negative controls must build clean, or the catches prove nothing"
    );
    assert_eq!(inj["mechanism_holds"].as_bool(), Some(true));
}

/// The corpus meets the spec's floors, as committed — not merely as generated.
#[test]
fn the_committed_corpus_meets_the_spec_floors() {
    let r = results();
    assert!(r["corpus"]["packages"].as_u64().unwrap_or(0) >= 25, "spec §3.1: >= 25 packages");
    assert!(r["corpus"]["min_depth"].as_u64().unwrap_or(0) >= 4, "spec §3.1: depth >= 4");
}

/// Every injection was refused by an AUTHORITY diagnostic, not by a syntax or type error. This is
/// the check that would have caught the false 100% the first version of this study produced.
#[test]
fn every_refusal_is_an_authority_diagnostic_not_a_compile_error() {
    let r = results();
    // The codes that mean "the authority claim did not hold". A refusal carrying only parse/type
    // codes would be the build failing for an unrelated reason.
    let authority = ["DL1001", "DL1002", "DL1003", "DL1009", "DL1010", "DL0501"];
    let compile_only = ["DL0201", "DL0202", "DL0203", "DL0401", "DL0403", "DL0405"];

    for i in r["injections"].as_array().expect("injections") {
        let codes: Vec<&str> =
            i["codes"].as_array().unwrap().iter().filter_map(|c| c.as_str()).collect();
        let victim = i["victim"].as_str().unwrap_or("?");
        assert!(
            codes.iter().any(|c| authority.contains(c)),
            "injection into `{victim}` was refused without any authority diagnostic: {codes:?}"
        );
        assert!(
            !codes.iter().any(|c| compile_only.contains(c)),
            "injection into `{victim}` produced a COMPILE error {codes:?} — the mutation is broken \
             code and its catch was not earned by the authority mechanism"
        );
        assert_eq!(
            i["injection_valid"].as_bool(),
            Some(true),
            "injection into `{victim}` did not compile on its own"
        );
    }
}

/// THE SKIP-BRANCH CASE (house rule 3): the scoring must be able to report failure. Asserted
/// against the scoring logic directly, because a metric that cannot express "missed" is a metric
/// that always says "caught".
#[test]
fn the_scoring_can_express_failure() {
    use delulu_measure::study_a::{Baseline, Control, Injection, Results};

    let inj = |caught: bool, valid: bool| Injection {
        chain: "c".into(),
        archetype: "a".into(),
        victim: "v".into(),
        depth: 1,
        caught,
        injection_valid: valid,
        codes: vec![],
        millis: 0,
    };
    let clean_control = || Control { chain: "c".into(), built_clean: true, codes: vec![] };
    let baseline = || Baseline {
        chain: "c".into(),
        archetype: "a".into(),
        packages: 5,
        depth: 4,
        lock_millis: 0,
        verify_millis: 0,
        clean: true,
    };

    // An empty campaign is 0%, never a vacuous 100%.
    let empty =
        Results { baselines: vec![baseline()], injections: vec![], controls: vec![clean_control()] };
    assert_eq!(empty.catch_rate(), 0.0, "an empty experiment proves nothing");
    assert!(!empty.mechanism_holds());

    // A genuine miss is reported as a miss.
    let missed = Results {
        baselines: vec![baseline()],
        injections: vec![inj(true, true), inj(false, true)],
        controls: vec![clean_control()],
    };
    assert_eq!(missed.catch_rate(), 50.0);
    assert!(!missed.mechanism_holds(), "a miss must break the mechanism claim");
    assert_eq!(missed.missed().len(), 1);

    // A mutation that did not compile is not a catch, however the build behaved.
    let broken = Results {
        baselines: vec![baseline()],
        injections: vec![inj(true, false)],
        controls: vec![clean_control()],
    };
    assert_eq!(broken.caught(), 0, "an invalid mutation is never credited as a catch");
    assert!(!broken.mechanism_holds(), "an invalid mutation invalidates the run");

    // A campaign that catches everything INCLUDING the control has measured nothing.
    let overzealous = Results {
        baselines: vec![baseline()],
        injections: vec![inj(true, true)],
        controls: vec![Control { chain: "c".into(), built_clean: false, codes: vec![] }],
    };
    assert!(
        !overzealous.mechanism_holds(),
        "if even the no-op control is refused, a 100% catch rate is meaningless"
    );

    // And the honest case still passes, so the gates above are not simply always-false.
    let good = Results {
        baselines: vec![baseline()],
        injections: vec![inj(true, true), inj(true, true)],
        controls: vec![clean_control()],
    };
    assert!(good.mechanism_holds(), "a sound campaign must still be able to pass");
}

/// The methodology's threats to validity are present and specific. A measurement program whose
/// limits section quietly disappears is one nobody can check.
#[test]
fn the_methodology_states_its_threats_to_validity() {
    let m = std::fs::read_to_string(repo_root().join("measurements/METHODOLOGY.md"))
        .expect("METHODOLOGY.md is committed");
    assert!(m.contains("Threats to validity"), "the threats section must exist");
    for required in [
        "synthetic",           // the corpus is not real software
        "not catching malice", // the guarantee's actual boundary
        "UNRUN",               // the human comparison was not run
        "one machine",         // timing caveat
    ] {
        assert!(m.contains(required), "METHODOLOGY.md must state the `{required}` limit plainly");
    }
}
