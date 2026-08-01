//! `delulu-survey` — build, query, and staleness-check the repository map.
//!
//! Exit codes follow the project's convention (`docs/for-agents.md`): `0` success, `1` the check
//! failed, `2` the invocation was wrong.

use delulu_survey::{EdgeKind, Repair, Survey, OUTPUT_DIR};
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let verb = args.first().map(String::as_str).unwrap_or("build");
    if matches!(verb, "--help" | "-h" | "help") {
        print_help();
        return;
    }
    let Some(root) = repo_root() else {
        eprintln!(
            "error: not inside a DeluluLang source tree — there is no repository here to map.\n\
             The Survey describes THIS repository and means nothing outside it."
        );
        std::process::exit(2);
    };

    match verb {
        "build" => build(&root, false),
        "check" => build(&root, true),
        "findings" => findings(&root),
        "query" | "rdeps" => match args.get(1) {
            Some(id) => query(&root, id, verb == "rdeps"),
            None => usage("query and rdeps need a node id, e.g. `crate:delulu-check`"),
        },
        other => usage(&format!("unknown verb `{other}`")),
    }
}

fn print_help() {
    println!(
        "delulu-survey — the map of this repository\n\n\
         USAGE\n  \
           delulu-survey build       regenerate {OUTPUT_DIR}/\n  \
           delulu-survey check       fail if the committed map is out of date\n  \
           delulu-survey findings    print the discrepancy list\n  \
           delulu-survey query <id>  a node, what it points at, and what points at it\n  \
           delulu-survey rdeps <id>  only what points at it — \"what breaks if I change this?\"\n\n\
         IDS\n  \
           crate:delulu-check  mod:crates/delulu-check/src/ty.rs  doc:README.md\n  \
           code:DL0501         ruling:S10-D64                     finding:C69\n"
    );
}

/// The source tree this tool is standing in.
///
/// Delegates to [`delulu_survey::find_source_tree`] rather than walking up on its own criteria.
/// It used to accept any directory holding a `Cargo.toml` beside a `crates/` folder, which is a
/// *different* and weaker test than the one `delulu doctor` applies — so the two commands could
/// disagree about whether they were in a DeluluLang checkout at all. One definition, one answer.
fn repo_root() -> Option<PathBuf> {
    delulu_survey::find_source_tree()
}

fn build(root: &Path, check_only: bool) {
    // The same entry point `delulu doctor` uses, so the two commands cannot drift on what "behind
    // the tree" means or on how the files get written.
    let h = delulu_survey::inspect(root, if check_only { Repair::ReportOnly } else { Repair::Regenerate });

    if check_only {
        if h.stale.is_empty() {
            println!("ok: the Survey matches the tree ({} nodes, {} edges)", h.nodes, h.edges);
            return;
        }
        for s in &h.stale {
            eprintln!("stale: {OUTPUT_DIR}/{s} is out of date or missing");
        }
        eprintln!(
            "\nThe repository changed and its map did not. Run `cargo run -p delulu-survey -- build`\n\
             and commit the result with the change that caused it."
        );
        std::process::exit(1);
    }

    if let Some(e) = &h.write_error {
        eprintln!("error: cannot write {OUTPUT_DIR}: {e}");
        std::process::exit(1);
    }

    let t = h.tally;
    println!(
        "wrote {OUTPUT_DIR}/ — {} nodes, {} edges, {} discrepancies ({} error, {} warning)",
        h.nodes,
        h.edges,
        t.errors + t.warnings + t.notes,
        t.errors,
        t.warnings
    );
}

fn findings(root: &Path) {
    let survey = Survey::build(root);
    for f in &survey.findings {
        let sev = f.severity.word();
        if f.line > 0 {
            println!("{sev}: {}:{} [{}] {}", f.file, f.line, f.class, f.message);
        } else {
            println!("{sev}: {} [{}] {}", f.file, f.class, f.message);
        }
    }
    println!("\n{} discrepancies", survey.findings.len());
}

fn query(root: &Path, id: &str, rdeps_only: bool) {
    let survey = Survey::build(root);
    let Some(node) = survey.node(id) else {
        eprintln!("error: no node `{id}`");
        let near: Vec<&str> =
            survey.nodes.iter().filter(|n| n.id.contains(id) || n.name.contains(id)).map(|n| n.id.as_str()).take(10).collect();
        if !near.is_empty() {
            eprintln!("did you mean:\n  {}", near.join("\n  "));
        }
        std::process::exit(1);
    };

    println!("{} [{:?}]", node.id, node.kind);
    if let Some(p) = &node.path {
        println!("  path    {p}{}", node.lines.map(|l| format!("  ({l} lines)")).unwrap_or_default());
    }
    if let Some(s) = &node.summary {
        println!("  is      {s}");
    }
    if !node.contents.is_empty() {
        println!("  holds   {}", node.contents.join(", "));
    }

    let incoming = survey.into_(id);
    if !rdeps_only {
        let outgoing = survey.out(id);
        println!("\npoints at ({}):", outgoing.len());
        for e in group(&outgoing, false) {
            println!("  {e}");
        }
    }
    println!("\npointed at by ({}):", incoming.len());
    for e in group(&incoming, true) {
        println!("  {e}");
    }
}

/// One line per edge, always carrying the citation — the map never asserts a relation without
/// telling the reader where to go and confirm it.
fn group(edges: &[&delulu_survey::Edge], incoming: bool) -> Vec<String> {
    let mut out: Vec<String> = edges
        .iter()
        .map(|e| {
            let other = if incoming { &e.from } else { &e.to };
            format!("{:<20} {:<48} @ {}:{}", kind_word(e.kind), other, e.file, e.line)
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

fn kind_word(k: EdgeKind) -> &'static str {
    match k {
        EdgeKind::DependsOn => "depends-on",
        EdgeKind::DependsOnExternal => "depends-on-ext",
        EdgeKind::DeclaresModule => "declares",
        EdgeKind::Uses => "uses",
        EdgeKind::DefinesCode => "defines-code",
        EdgeKind::RaisesCode => "raises",
        EdgeKind::ExpectsCode => "expects",
        EdgeKind::DocumentsCode => "documents",
        EdgeKind::Defines => "defines",
        EdgeKind::Cites => "cites",
        EdgeKind::LinksTo => "links-to",
        EdgeKind::References => "references",
        EdgeKind::TestsCrate => "tests",
    }
}

fn usage(msg: &str) -> ! {
    eprintln!("error: {msg}\n");
    print_help();
    std::process::exit(2);
}
