//! The CLI's verbs in the map, and the difference between an edge that was read and one that was guessed.
//!
//! Architecture slice 1 of the owner's commission of 2026-11-14. Before it, the map had no node
//! between a module and a test: `impact mod:crates/delulu/src/run_cmd.rs` reached `main.rs`, the
//! crate, and then every test file the crate has — 111 nodes, the same answer for any module. The
//! dispatch that connects a module to the command a person types is one `match`, so it is read; and
//! the tests that drive a command are found by the one string they must all contain. The second is a
//! guess, and these tests hold the map to saying so wherever it shows the edge.

use delulu_survey::{answers, Confidence, Dir, EdgeKind, NodeKind, Survey};
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn line_of(root: &Path, file: &str, line: u32) -> String {
    let text = std::fs::read_to_string(root.join(file)).unwrap_or_else(|e| panic!("{file}: {e}"));
    text.lines().nth(line as usize - 1).unwrap_or_else(|| panic!("{file} has no line {line}")).to_string()
}

/// Every verb is a node, and every verb's edge to its handler cites the arm it was read from.
#[test]
fn the_dispatch_is_mapped_verb_by_verb_with_the_arm_it_was_read_from() {
    let root = repo_root();
    let s = Survey::build(&root);
    let verbs: Vec<_> = s.nodes.iter().filter(|n| n.kind == NodeKind::CliVerb).collect();
    assert!(verbs.len() >= 30, "the CLI has dozens of commands; the map holds {} verb(s)", verbs.len());
    assert_eq!(s.facts.cli_verbs as usize, verbs.len(), "the fact and the nodes disagree");

    // The command this branch's first slice changed, by name: `run` is handled by `run_cmd.rs`.
    let run: Vec<_> = s.out("verb:run").into_iter().filter(|e| e.kind == EdgeKind::DispatchesTo).collect();
    assert_eq!(run.len(), 1, "`run` has one handler: {run:?}");
    assert_eq!(run[0].to, "mod:crates/delulu/src/run_cmd.rs");
    assert!(line_of(&root, &run[0].file, run[0].line).contains("\"run\" =>"), "the cited line is the arm");

    for v in &verbs {
        let out: Vec<_> = s.out(&v.id).into_iter().filter(|e| e.kind == EdgeKind::DispatchesTo).collect();
        assert_eq!(out.len(), 1, "`{}` must name exactly one handler module", v.id);
        let e = out[0];
        assert_eq!(e.confidence, Confidence::SourceDerived, "a dispatch arm is read, not guessed: {e:?}");
        assert!(s.node(&e.to).is_some_and(|n| n.kind == NodeKind::RustModule), "`{}` dispatches to a module: {e:?}", v.id);
        let cited = line_of(&root, &e.file, e.line);
        assert!(cited.contains(" => "), "`{}` cites a line that is not a match arm: {cited}", v.id);
        assert!(cited.contains(v.name.rsplit("::").next().unwrap()), "`{}` is not on the line it cites: {cited}", v.id);
        // An internal subcommand's spelling is a constant's value, which is not on the line: said, not guessed.
        if v.name.contains("::") {
            assert!(v.contents.iter().any(|c| c.contains("DYNAMIC")), "`{}` must be marked as not read: {:?}", v.id, v.contents);
        }
    }
}

/// An inferred edge says it is inferred: in the data, in the machine answer, and in the words a person reads.
#[test]
fn an_inferred_edge_says_so_everywhere_it_is_shown() {
    let s = Survey::build(&repo_root());
    let inferred: Vec<_> = s.edges.iter().filter(|e| e.confidence == Confidence::Inferred).collect();
    assert!(!inferred.is_empty(), "the test-to-verb edges exist");
    assert!(inferred.iter().all(|e| e.kind == EdgeKind::Exercises), "only `exercises` is a heuristic today");
    assert!(
        s.edges.iter().filter(|e| e.kind == EdgeKind::Exercises).all(|e| e.confidence == Confidence::Inferred),
        "an `exercises` edge that claims to be derived would be the lie this field exists to prevent"
    );
    assert_eq!(s.facts.edges_inferred as usize, inferred.len(), "the map counts its own guesses, exactly");

    // The data: absent means derived, so the edges that always were are byte-for-byte what they were.
    let derived = s.edges.iter().find(|e| e.kind == EdgeKind::DependsOn).expect("a crate dependency");
    assert!(serde_json::to_value(derived).unwrap().get("confidence").is_none());
    assert_eq!(serde_json::to_value(inferred[0]).unwrap()["confidence"], "inferred");

    // The machine answer: always present, either word.
    let q = answers::query_json(&s, "verb:run", false).expect("verb:run is a node");
    assert_eq!(q["points_at"][0]["via"]["confidence"], "source-derived", "{q}");
    let by = q["pointed_at_by"].as_array().expect("pointed_at_by");
    assert!(!by.is_empty() && by.iter().all(|h| h["via"]["confidence"] == "inferred"), "{q}");
    let w = answers::walk_json(&s, "mod:crates/delulu/src/run_cmd.rs", true, 2).expect("the module is a node");
    let hops = w["hops"].as_array().expect("hops");
    assert!(hops.iter().any(|h| h["via"]["kind"] == "exercises" && h["via"]["confidence"] == "inferred"), "{w}");
    assert!(hops.iter().all(|h| h["via"]["confidence"].is_string()), "every hop says how it is known");

    // The words: no renderer can print a guess in a fact's words.
    assert_eq!(answers::edge_word(inferred[0]), "exercises (inferred)");
    assert_eq!(answers::edge_word(derived), "depends-on");
}

/// What the slice is for: a handler's blast radius names the tests that drive its command, two hops
/// away, instead of every test in the crate three hops away.
#[test]
fn impact_of_a_handler_reaches_the_tests_that_drive_its_verb() {
    let s = Survey::build(&repo_root());
    let module = "mod:crates/delulu/src/run_cmd.rs";
    let near: Vec<_> = s.walk(module, Dir::Incoming, 2);
    assert!(near.iter().any(|r| r.id == "verb:run" && r.depth == 1), "the verb is one hop from its handler");
    let driven: std::collections::BTreeSet<&str> =
        s.into_("verb:run").into_iter().filter(|e| e.kind == EdgeKind::Exercises).map(|e| e.from.as_str()).collect();
    let reached: std::collections::BTreeSet<&str> = near.iter().filter(|r| r.id.starts_with("test:")).map(|r| r.id).collect();
    assert_eq!(reached, driven, "within two hops: exactly the tests that drive `run`, no more");
    assert!(driven.contains("test:crates/delulu/tests/json_contract.rs"), "the suite that pins `run`'s report is among them");
    let every = s.nodes.iter().filter(|n| n.kind == NodeKind::TestSuite && n.id.starts_with("test:crates/delulu/tests/")).count();
    assert!(driven.len() < every, "{} of {every} suites drive `run`: a narrower answer than the crate's", driven.len());
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

const CLI: &str = "//! a CLI\n\nfn run_inner(args: &[String]) -> i32 {\n    let cmd = &args[0];\n    let rest = &args[1..];\n    \
                   match cmd.as_str() {\n        \"alpha\" => crate::one::cmd_alpha(rest),\n        \"beta\" => cmd_beta(rest),\n        \
                   #[cfg(target_os = \"linux\")]\n        s if s == crate::one::HIDDEN => crate::one::hidden(rest),\n        \
                   \"--help\" | \"-h\" | \"help\" => {\n            match rest[0].as_str() {\n                \"inner\" => 0,\n                \
                   _ => 1,\n            }\n        }\n        _ => 2,\n    }\n}\n";

/// The map follows the dispatch: rename a command, and the node, its handler edge and the test's edge
/// all move with it; lose the dispatch, and the map says so instead of quietly holding no verbs.
#[test]
fn the_map_follows_the_dispatch_when_a_command_is_renamed_or_the_dispatch_is_lost() {
    let dir = std::env::temp_dir().join(format!("delulu-survey-verbs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    write(&dir, "crates/delulu/Cargo.toml", "[package]\nname = \"delulu\"\nversion = \"0.0.0\"\n");
    write(&dir, "crates/delulu/src/cli.rs", CLI);
    write(&dir, "crates/delulu/src/one.rs", "//! one\npub const HIDDEN: &str = \"__hidden\";\n");
    write(
        &dir,
        "crates/delulu/tests/t.rs",
        "#[test]\nfn t() {\n    let _ = delulu(&[\"alpha\", \"x\"]);\n    let _ = [\"alphabet\"];\n}\n",
    );

    let s = Survey::build(&dir);
    let names: Vec<&str> = s.nodes.iter().filter(|n| n.kind == NodeKind::CliVerb).map(|n| n.name.as_str()).collect();
    assert_eq!(names, ["alpha", "beta", "help", "one::HIDDEN"], "the arms of the dispatch, and not the arms nested in one");
    assert_eq!(s.out("verb:alpha")[0].to, "mod:crates/delulu/src/one.rs", "a handler in another module");
    assert_eq!(s.out("verb:beta")[0].to, "mod:crates/delulu/src/cli.rs", "a handler in the dispatch's own file");
    let hidden = s.node("verb:one::HIDDEN").expect("the internal subcommand");
    assert!(
        hidden.summary.as_deref().is_some_and(|x| x.starts_with("#[cfg(target_os = \"linux\")]")),
        "its gate is kept: {hidden:?}"
    );
    assert!(hidden.contents.iter().any(|c| c.contains("DYNAMIC")), "{hidden:?}");
    let drives = |s: &Survey, verb: &str| s.into_(verb).into_iter().filter(|e| e.kind == EdgeKind::Exercises).count();
    assert_eq!(
        (drives(&s, "verb:alpha"), drives(&s, "verb:beta")),
        (1, 0),
        "`[\"alphabet\"]` is not `alpha`; nothing drives beta"
    );
    assert_eq!(s.into_("verb:alpha")[0].line, 3, "the edge cites the line of the argument list");
    assert!(s.node("verb:inner").is_none(), "an arm of a `match` nested in an arm is a sub-verb, not a verb");
    assert_eq!(s.out("verb:alpha")[0].line, 7, "the handler edge cites the arm's own line");
    assert_eq!(s.facts.edges_inferred, 1, "one guess in this tree, and it is counted");

    // Rename the command. The test still says the old name: its edge must go, not follow by luck.
    write(&dir, "crates/delulu/src/cli.rs", &CLI.replace("\"alpha\" =>", "\"gamma\" =>"));
    let s = Survey::build(&dir);
    assert!(s.node("verb:alpha").is_none() && s.node("verb:gamma").is_some(), "the node follows the name");
    assert_eq!(s.out("verb:gamma")[0].to, "mod:crates/delulu/src/one.rs");
    assert_eq!(drives(&s, "verb:gamma"), 0, "no test drives the new name yet, and the map does not pretend one does");
    assert_eq!(s.facts.edges_inferred, 0);

    // Lose the dispatch (the function renamed). No verbs — and a finding that says why.
    write(&dir, "crates/delulu/src/cli.rs", &CLI.replace("fn run_inner(", "fn dispatch("));
    let s = Survey::build(&dir);
    assert_eq!(s.facts.cli_verbs, 0);
    let said: Vec<_> = s.findings.iter().filter(|f| f.class == "cli-dispatch-not-found").collect();
    assert_eq!(said.len(), 1, "a map with no verbs must say it could not find the dispatch: {:?}", s.findings);
    assert!(!said[0].remedy.is_empty(), "and what to do about it");
    let _ = std::fs::remove_dir_all(&dir);
}
