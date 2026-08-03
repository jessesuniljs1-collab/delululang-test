//! The Survey's **machine channel** — `--json` on every read-only verb.
//!
//! The Survey is the map every maintainer consults before changing anything, and it had only one
//! working channel: `--json` was neither implemented nor rejected, so asking for a structured answer
//! returned formatted text and exit 0. A structured answer is not a machine's feature — a person
//! writing a CI check needs it, and an agent chasing a bad edge reads the prose. Both channels are
//! first-class because **any maintainer may use either**, which is the same no-discrimination rule
//! the language applies to the parties holding its grants (Constitution invariant 24).
//!
//! The failure was quiet in one direction and not the other, which is why it lasted: a reader
//! watching the terminal sees prose where JSON should be, and a pipeline sees a parse error much
//! later, or a wrong answer never.
//!
//! Two rules are enforced here, and they are the same two the main CLI already lives by:
//!
//! 1. **One object per invocation** (`docs/for-agents.md`, campaign finding C2).
//! 2. **An option nobody understood is refused, never ignored** (campaign finding C76 / D73).
//!
//! And one rule that belongs to the Survey alone: **every relation cites the file and line it was
//! read from.** That is the provenance law, and it is not a property of the human rendering — if it
//! does not survive into the machine channel, an agent reading this map has strictly less ability to
//! check it than a human reading the same map.
//!
//! Only READ-ONLY verbs are exercised. `build` rewrites `docs/survey/`, and a test that rewrites the
//! repository it is testing is a test that breaks the next run.

use std::process::{Command, Output};

fn survey(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu-survey"))
        .args(args)
        // The Survey only means anything inside its own source tree, and the test binary's cwd is
        // the crate directory, which is inside it.
        .output()
        .expect("the delulu-survey binary must run")
}

fn one_object(out: &Output, what: &str) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut values = serde_json::Deserializer::from_str(stdout.trim()).into_iter::<serde_json::Value>();
    let first = values
        .next()
        .unwrap_or_else(|| panic!("{what}: nothing on stdout"))
        .unwrap_or_else(|e| panic!("{what}: stdout is not JSON ({e}):\n{stdout}"));
    assert!(
        values.next().is_none(),
        "{what}: more than one JSON value on stdout — the contract is exactly one:\n{stdout}"
    );
    first
}

/// A node that exists, is stable, and is interesting: the module that decides what type-checks.
const HEART: &str = "mod:crates/delulu-check/src/check.rs";

#[test]
fn every_read_only_verb_answers_json_with_one_object_and_says_what_it_is() {
    for (args, verb) in [
        (vec!["query", HEART, "--json"], "query"),
        (vec!["rdeps", HEART, "--json"], "rdeps"),
        (vec!["impact", HEART, "--json"], "impact"),
        (vec!["affected-by", HEART, "--json"], "affected-by"),
        (vec!["findings", "--json"], "findings"),
        (vec!["check", "--json"], "check"),
    ] {
        let out = survey(&args);
        let v = one_object(&out, &format!("{args:?}"));
        assert_eq!(v["tool"], "delulu-survey", "{args:?}: {v}");
        assert_eq!(v["verb"], verb, "{args:?} must name the verb it answered: {v}");
        assert_eq!(v["schema"], 1, "{args:?}: a caller branches on schema before anything else: {v}");
        assert!(v["delulu_version"].is_string(), "{args:?}: {v}");
    }
}

/// The regression witness for the defect this file exists because of: `--json` used to be accepted
/// and ignored, so the answer was prose. Prose is not a JSON object, so parsing is the assertion.
#[test]
fn json_is_honoured_rather_than_silently_ignored() {
    let out = survey(&["query", HEART, "--json"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.trim_start().starts_with('{'),
        "`--json` was accepted and answered with human text — the flag is being ignored:\n{stdout}"
    );
    one_object(&out, "query --json");
}

/// The provenance law, in the machine channel. Every edge and every hop names where it was read.
#[test]
fn every_relation_in_json_cites_the_file_and_line_it_was_read_from() {
    let v = one_object(&survey(&["query", HEART, "--json"]), "query");
    let mut checked = 0usize;
    for key in ["points_at", "pointed_at_by"] {
        for e in v[key].as_array().unwrap_or_else(|| panic!("{key} must be an array: {v}")) {
            let via = &e["via"];
            assert!(via["file"].as_str().is_some_and(|s| !s.is_empty()), "{key} edge with no file: {e}");
            assert!(via["line"].as_u64().is_some_and(|l| l > 0), "{key} edge with no line: {e}");
            assert!(via["kind"].as_str().is_some_and(|s| !s.is_empty()), "{key} edge with no kind: {e}");
            checked += 1;
        }
    }
    assert!(checked > 0, "the fixture node has no edges, so this test proved nothing: {v}");

    let w = one_object(&survey(&["impact", HEART, "--json"]), "impact");
    let hops = w["hops"].as_array().expect("hops must be an array");
    assert!(!hops.is_empty(), "the fixture node reaches nothing, so this test proved nothing");
    for h in hops {
        let via = &h["via"];
        assert!(via["file"].as_str().is_some_and(|s| !s.is_empty()), "hop with no file: {h}");
        assert!(via["line"].as_u64().is_some_and(|l| l > 0), "hop with no line: {h}");
        assert!(h["from"].as_str().is_some_and(|s| !s.is_empty()), "hop with no parent: {h}");
    }
}

/// Entrenchment answers "may I change this?", so it must be present on every node — `null` when the
/// node is ordinary rather than absent. A caller that keys on a missing field cannot distinguish
/// "not entrenched" from "this tool did not tell me", and those two must never look alike.
#[test]
fn entrenchment_is_always_answered_never_merely_omitted() {
    let ordinary = one_object(&survey(&["query", HEART, "--json"]), "query");
    assert!(
        ordinary.get("entrenched").is_some(),
        "an ordinary node must still carry the field, set to null: {ordinary}"
    );
    assert!(ordinary["entrenched"].is_null(), "this node is not entrenched: {ordinary}");

    let locked = one_object(&survey(&["query", "doc:docs/design/CONSTITUTION.md", "--json"]), "query");
    let e = &locked["entrenched"];
    assert!(!e.is_null(), "the constitution IS entrenched — if this is null the fixture moved: {locked}");
    assert!(e["owner"].as_str().is_some_and(|s| s.starts_with('@')), "{e}");
    assert!(e["matched_at"]["file"].as_str().is_some_and(|s| !s.is_empty()), "{e}");
    assert!(e["matched_at"]["line"].as_u64().is_some_and(|l| l > 0), "{e}");
}

/// An option nobody understood is refused, never ignored — and the refusal says which one.
#[test]
fn an_unknown_option_is_refused_and_named() {
    let out = survey(&["query", HEART, "--not-a-real-flag"]);
    assert_eq!(out.status.code(), Some(2), "a usage error exits 2 (STABILITY.md)");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("--not-a-real-flag"), "the refusal must name the option:\n{err}");
    assert!(
        out.stdout.is_empty(),
        "nothing was done, so nothing should have been printed to stdout:\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// `--depth` still works, and its VALUE is not mistaken for a node id now that positionals and flags
/// are separated. This is the exact shape that breaks when a flag parser is bolted on afterwards.
#[test]
fn depth_limits_the_walk_and_its_value_is_not_read_as_a_node() {
    let shallow = one_object(&survey(&["impact", HEART, "--depth", "1", "--json"]), "impact --depth 1");
    assert_eq!(shallow["verb"], "impact");
    assert_eq!(shallow["depth_limit"], 1, "{shallow}");
    assert_eq!(shallow["node"], HEART, "the depth VALUE was read as the node id: {shallow}");

    let deep = one_object(&survey(&["impact", HEART, "--json"]), "impact");
    assert!(
        shallow["reached"].as_u64().unwrap_or(0) < deep["reached"].as_u64().unwrap_or(0),
        "depth 1 must reach fewer nodes than the default depth, or the flag does nothing"
    );
}
