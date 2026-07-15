//! Phase A2 — the Atlas core, through the real `delulu` binary (Surface addendum §2).
//!
//! Witnesses acceptance criteria 1 (determinism), 2 (`atlas/1` JSON), 3 (digest discipline),
//! 4 (query verbs + explicit budget truncation), 5 (authority parity), and 6 (refusal honesty:
//! DL1780 on check errors, no partial graph). The self-contained-HTML / custody halves land in A3.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

const DEMO: &str = "examples/demo.delulu";
const REJECT: &str = "tests/conformance/reject/DL0501_undeclared_effect.delulu";
const ESC: char = '\u{1b}';

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(workspace_root())
        .env_remove("NO_COLOR")
        .env_remove("DELULU_COLOR")
        .args(args)
        .output()
        .expect("failed to run delulu")
}
fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

// ----- criterion 1: determinism -------------------------------------------------------------------

#[test]
fn output_is_byte_identical_across_runs_every_format() {
    for fmt in ["tree", "digest", "json"] {
        let a = stdout(&delulu(&["atlas", DEMO, "--format", fmt]));
        let b = stdout(&delulu(&["atlas", DEMO, "--format", fmt]));
        assert_eq!(a, b, "format {fmt} must be byte-identical across runs");
        assert!(!a.is_empty(), "format {fmt} produced output");
    }
}

// ----- criterion 2: atlas/1 JSON ------------------------------------------------------------------

#[test]
fn atlas_json_is_a_versioned_envelope_with_stable_ids() {
    let o = delulu(&["atlas", DEMO, "--format", "json"]);
    let v: Value = serde_json::from_str(&stdout(&o)).expect("valid JSON");
    assert_eq!(v["atlas"], "atlas/1", "versioned envelope");
    assert_eq!(v["root"], "demo");
    assert!(v["nodes"].as_array().unwrap().iter().any(|n| n["id"] == "fn:demo/demo.main"));
    assert_eq!(v["custody"], Value::Null, "no overlay without --custody");
    // Round-trips through serde (re-serialize the parsed value equals the text's value).
    let reparsed: Value = serde_json::from_str(&serde_json::to_string(&v).unwrap()).unwrap();
    assert_eq!(reparsed, v);
}

#[test]
fn json_carries_zero_ansi_even_under_color_always() {
    let o = delulu(&["atlas", DEMO, "--format", "json", "--color", "always"]);
    assert!(!stdout(&o).contains(ESC), "the machine channel is never colored");
}

// ----- criterion 3: digest discipline -------------------------------------------------------------

#[test]
fn digest_has_budget_ordering_authority_gods_caveats_and_footer() {
    let d = stdout(&delulu(&["atlas", DEMO, "--format", "digest"]));
    assert!(d.contains("## Querying further"), "the fixed footer is present");
    assert!(d.contains("delulu atlas node <name-or-id>"), "footer teaches the verbs verbatim");
    assert!(d.contains("## God nodes"), "god nodes present");
    assert!(d.contains("Authority (mirrors `delulu authority`)"), "authority table present");
    assert!(
        d.contains("static map of checked facts, not a runtime trace"),
        "the verbatim caveat ships"
    );
    assert!(d.contains("chars/4 heuristic"), "the budget is stated as a heuristic");
    // Default budget respected (chars/4 ≤ 2000).
    assert!(d.chars().count() / 4 <= 2000, "digest respects the default 2000-token budget");
}

#[test]
fn digest_is_never_colored_even_under_color_always() {
    let d = stdout(&delulu(&["atlas", DEMO, "--format", "digest", "--color", "always"]));
    // The digest is a machine/LLM channel (Markdown) — no SGR.
    assert!(!d.contains(ESC));
}

// ----- criterion 4: query verbs -------------------------------------------------------------------

#[test]
fn query_verbs_answer_without_the_whole_graph() {
    let callers = stdout(&delulu(&["atlas", "callers", "greet", DEMO]));
    assert!(callers.contains("main"), "callers of greet include main: {callers}");
    let calls = stdout(&delulu(&["atlas", "calls", "main", DEMO]));
    assert!(calls.contains("greet"), "main calls greet: {calls}");

    let path = stdout(&delulu(&["atlas", "path", "main", "Write", DEMO]));
    assert!(path.contains("--performs-->") || path.contains("--calls-->"), "typed hops: {path}");

    let why = stdout(&delulu(&["atlas", "why", "Write", DEMO]));
    assert!(why.contains("greet") || why.contains("main"), "why names a performer: {why}");

    let node = stdout(&delulu(&["atlas", "node", "fib", DEMO]));
    assert!(node.contains("pure:      true"), "node shows purity: {node}");
}

#[test]
fn query_json_is_structured_and_uncolored() {
    let o = delulu(&["atlas", "node", "main", DEMO, "--json", "--color", "always"]);
    let s = stdout(&o);
    assert!(!s.contains(ESC), "query --json is a machine channel");
    let v: Value = serde_json::from_str(&s).expect("valid JSON");
    assert_eq!(v["verb"], "node");
    assert_eq!(v["node"]["id"], "fn:demo/demo.main");
    assert!(v["edges"]["calls"].as_array().unwrap().iter().any(|c| c["name"] == "greet"));
}

#[test]
fn budget_truncation_is_explicit_never_silent() {
    let capped = stdout(&delulu(&["atlas", "node", "main", DEMO, "--budget", "1"]));
    assert!(capped.contains("truncated at budget"), "truncation is explicit: {capped}");
}

/// A3.2 (head-chef live-verification gap): the DIGEST honors `--budget` too — §2.2 caps ANY
/// textual output. A multi-module fixture big enough to truncate; the floor (title + explicit
/// notice + the "Querying further" footer) always survives.
#[test]
fn digest_honors_budget_through_the_binary() {
    // Generate a package with enough modules that the unbudgeted digest is large.
    let dir = std::env::temp_dir().join(format!("delulu_digest_budget_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("delulu.toml"),
        "[package]\nname = \"big\"\nversion = \"0.1.0\"\nkind = \"bin\"\n\n\
         [authority]\neffects  = [\"Write\"]\nfs.read  = []\nfs.write = []\nnet      = []\nsecrets  = []\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src").join("main.delulu"),
        "module big\nfn main(root: Root) ! {Write} { let out = root.console()\n out.println(\"hi\") }\n",
    )
    .unwrap();
    for i in 0..40 {
        std::fs::write(
            dir.join("src").join(format!("m{i:02}.delulu")),
            format!("module big.m{i:02}\npub fn f{i:02}(n: Int) -> Int {{ n + {i} }}\n"),
        )
        .unwrap();
    }
    let target = dir.to_str().unwrap();

    let unbudgeted = stdout(&delulu(&["atlas", target, "--format", "digest"]));
    assert!(unbudgeted.len() > 2000, "the fixture digest is large enough to truncate: {} chars", unbudgeted.len());

    let budgeted = stdout(&delulu(&["atlas", target, "--format", "digest", "--budget", "100"]));
    // (a) the truncation sentence is present — explicit, never silent.
    assert!(budgeted.contains("truncated at budget"), "explicit truncation: {budgeted}");
    // (b) the footer (the remedy pointer) is still present.
    assert!(budgeted.contains("## Querying further"), "footer survives truncation");
    assert!(budgeted.contains("delulu atlas node <name-or-id>"), "the verbs are still taught");
    // (c) dramatically smaller than unbudgeted.
    assert!(
        budgeted.len() < unbudgeted.len() / 2,
        "dramatically smaller: {} vs {} chars",
        budgeted.len(),
        unbudgeted.len()
    );
    // (d) deterministic: byte-identical rerun.
    let again = stdout(&delulu(&["atlas", target, "--format", "digest", "--budget", "100"]));
    assert_eq!(budgeted, again, "byte-identical across runs");

    let _ = std::fs::remove_dir_all(&dir);
}

// ----- criterion 5: authority parity --------------------------------------------------------------

#[test]
fn atlas_authority_matches_delulu_authority_exactly() {
    let atlas: Value = serde_json::from_str(&stdout(&delulu(&["atlas", DEMO, "--json"]))).unwrap();
    let authority: Value =
        serde_json::from_str(&stdout(&delulu(&["authority", DEMO, "--json"]))).unwrap();
    // The atlas embeds the authority report's compiler-computed fields verbatim; `delulu authority`
    // additionally stamps run-mode decorations (custody/foreign_isolation) that the atlas omits.
    for key in ["effects", "capabilities", "secrets", "foreign_calls", "pure_functions"] {
        assert_eq!(
            atlas["authority"][key], authority["authority"][key],
            "authority parity on `{key}`"
        );
    }
    // And the graph's performs edges reach exactly those effects.
    let effects: std::collections::BTreeSet<String> = authority["authority"]["effects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let performs: std::collections::BTreeSet<String> = atlas["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "performs")
        .map(|e| e["to"].as_str().unwrap().trim_start_matches("effect:").to_string())
        .collect();
    assert_eq!(performs, effects, "performs edges agree with authority effects");
}

// ----- criterion 6: refusal honesty ---------------------------------------------------------------

#[test]
fn check_errors_refuse_with_dl1780_and_no_partial_graph() {
    let o = delulu(&["atlas", REJECT]);
    assert_eq!(o.status.code(), Some(1), "the atlas refuses (exit 1)");
    let e = stderr(&o);
    assert!(e.contains("DL0501"), "the check errors are printed first: {e}");
    assert!(e.contains("DL1780"), "then DL1780 refuses: {e}");
    assert!(e.contains("fix the 1 error(s) above first"), "it names the count");
    // No partial graph anywhere.
    assert!(!stdout(&o).contains("atlas/1"), "no atlas envelope on refusal");
    assert!(!e.contains("atlas/1"));
}

#[test]
fn refusal_in_json_mode_is_a_diagnostics_envelope_not_a_graph() {
    let o = delulu(&["atlas", REJECT, "--json"]);
    assert_eq!(o.status.code(), Some(1));
    let v: Value = serde_json::from_str(&stdout(&o)).expect("a diagnostics envelope is valid JSON");
    let codes: Vec<&str> =
        v["diagnostics"].as_array().unwrap().iter().map(|d| d["code"].as_str().unwrap()).collect();
    assert!(codes.contains(&"DL1780"), "DL1780 in the envelope: {codes:?}");
    assert!(v.get("atlas").is_none(), "no atlas schema on refusal");
}

// ----- A3: renderers + colored tree ---------------------------------------------------------------

#[test]
fn tree_is_colored_only_when_asked_and_plain_when_piped() {
    // Piped (non-TTY): plain — the deterministic byte-identical form (criterion 1).
    let plain = stdout(&delulu(&["atlas", DEMO]));
    assert!(!plain.contains(ESC), "piped tree is colorless");
    // --color always: painted via the Palette roles.
    let colored = stdout(&delulu(&["atlas", DEMO, "--color", "always"]));
    assert!(colored.contains(ESC), "the tree is colored under --color always");
}

#[test]
fn dot_and_mermaid_render_through_the_binary() {
    let dot = stdout(&delulu(&["atlas", DEMO, "--format", "dot"]));
    assert!(dot.starts_with("digraph atlas {"), "{dot}");
    assert!(dot.contains("performs"), "typed edges in dot");
    let mermaid = stdout(&delulu(&["atlas", DEMO, "--format", "mermaid"]));
    assert!(mermaid.starts_with("graph LR"), "{mermaid}");
    assert!(mermaid.contains("package demo"), "module-level mermaid");
}

#[test]
fn html_is_self_contained_through_the_binary() {
    let html = stdout(&delulu(&["atlas", DEMO, "--format", "html"]));
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("const ATLAS="), "data embedded inline");
    // Criterion 7: zero external URLs.
    assert!(!html.contains("http://"), "no http URLs in the artifact");
    assert!(!html.contains("https://"), "no https URLs in the artifact");
}

// ----- A3.1: the foreign boundary in the graph ----------------------------------------------------

#[test]
fn foreign_boundary_appears_in_atlas_json_with_nodes_and_edges() {
    // The head-chef acceptance gate: a real foreign program's atlas carries a `foreign:py:` node
    // and a `foreign` edge — the boundary is IN the graph, not only in the digest's authority text.
    const NUMPY: &str = "examples/numpy_mean.delulu";
    let o = delulu(&["atlas", NUMPY, "--json"]);
    assert!(o.status.success(), "atlas on the numpy example: {}", stderr(&o));
    let s = stdout(&o);
    let v: Value = serde_json::from_str(&s).expect("valid atlas/1 JSON");
    // The Python module node.
    let numpy = v["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == "foreign:py:numpy")
        .expect("a foreign:py:numpy node in the graph");
    assert_eq!(numpy["kind"], "foreign");
    assert_eq!(numpy["name"], "numpy");
    // The function→foreign edge, from the ForeignCall-rowed function that imports it.
    assert!(
        v["edges"].as_array().unwrap().iter().any(|e| e["kind"] == "foreign"
            && e["from"] == "fn:numpy_mean/numpy_mean.mean_of"
            && e["to"] == "foreign:py:numpy"),
        "mean_of --foreign--> numpy edge present"
    );
    // Query verbs route through the boundary.
    let path = stdout(&delulu(&["atlas", "path", "mean_of", "numpy", NUMPY]));
    assert!(path.contains("--foreign-->"), "typed foreign hop: {path}");
    // Determinism holds with foreign nodes in place (criterion 1 re-verified).
    let again = stdout(&delulu(&["atlas", NUMPY, "--json"]));
    assert_eq!(s, again, "byte-identical across runs with the foreign boundary populated");
}

// ----- explain surface ----------------------------------------------------------------------------

#[test]
fn explain_e_atlas_describes_the_model() {
    let s = stdout(&delulu(&["explain", "E-ATLAS"]));
    assert!(s.contains("Atlas"), "{s}");
    assert!(s.contains("atlas/1"), "the schema is named");
    assert!(s.contains("not a runtime trace"), "the honesty caveat ships");
    assert!(s.contains("Querying further"), "the digest footer is named");
}
