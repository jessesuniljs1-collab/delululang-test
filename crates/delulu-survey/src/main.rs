//! `delulu-survey` — build, query, and staleness-check the repository map.
//!
//! Exit codes follow the project's convention (`docs/for-agents.md`): `0` success, `1` the check
//! failed, `2` the invocation was wrong.

use delulu_survey::{render, EdgeKind, Severity, Survey};
use std::path::{Path, PathBuf};

const OUT_DIR: &str = "docs/survey";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let verb = args.first().map(String::as_str).unwrap_or("build");
    let root = repo_root();

    match verb {
        "build" => build(&root, false),
        "check" => build(&root, true),
        "findings" => findings(&root),
        "query" | "rdeps" => match args.get(1) {
            Some(id) => query(&root, id, verb == "rdeps"),
            None => usage("query and rdeps need a node id, e.g. `crate:delulu-check`"),
        },
        "--help" | "-h" | "help" => {
            print_help();
        }
        other => usage(&format!("unknown verb `{other}`")),
    }
}

fn print_help() {
    println!(
        "delulu-survey — the map of this repository\n\n\
         USAGE\n  \
           delulu-survey build       regenerate {OUT_DIR}/\n  \
           delulu-survey check       fail if the committed map is out of date\n  \
           delulu-survey findings    print the discrepancy list\n  \
           delulu-survey query <id>  a node, what it points at, and what points at it\n  \
           delulu-survey rdeps <id>  only what points at it — \"what breaks if I change this?\"\n\n\
         IDS\n  \
           crate:delulu-check  mod:crates/delulu-check/src/ty.rs  doc:README.md\n  \
           code:DL0501         ruling:S10-D64                     finding:C69\n"
    );
}

/// Walk up from the working directory to the directory holding the workspace manifest, so the tool
/// works from anywhere inside the tree.
fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if dir.join("Cargo.toml").exists() && dir.join("crates").is_dir() {
            return dir;
        }
        if !dir.pop() {
            return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        }
    }
}

fn build(root: &Path, check_only: bool) {
    let survey = Survey::build(root);
    let out = root.join(OUT_DIR);

    let files = [
        ("survey.json", render::json(&survey)),
        ("SURVEY.md", render::markdown(&survey)),
        ("DISCREPANCIES.md", render::discrepancies(&survey)),
    ];

    if check_only {
        let mut stale = Vec::new();
        for (name, content) in &files {
            let path = out.join(name);
            match std::fs::read_to_string(&path) {
                Ok(on_disk) if on_disk == *content => {}
                Ok(_) => stale.push(format!("{OUT_DIR}/{name} is out of date")),
                Err(_) => stale.push(format!("{OUT_DIR}/{name} is missing")),
            }
        }
        if stale.is_empty() {
            println!("ok: the Survey matches the tree ({} nodes, {} edges)", survey.nodes.len(), survey.edges.len());
            return;
        }
        for s in &stale {
            eprintln!("stale: {s}");
        }
        eprintln!(
            "\nThe repository changed and its map did not. Run `cargo run -p delulu-survey -- build`\n\
             and commit the result with the change that caused it."
        );
        std::process::exit(1);
    }

    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("error: cannot create {}: {e}", out.display());
        std::process::exit(1);
    }
    for (name, content) in &files {
        if let Err(e) = std::fs::write(out.join(name), content) {
            eprintln!("error: cannot write {name}: {e}");
            std::process::exit(1);
        }
    }

    let errors = survey.findings.iter().filter(|f| f.severity == Severity::Error).count();
    let warnings = survey.findings.iter().filter(|f| f.severity == Severity::Warning).count();
    println!(
        "wrote {OUT_DIR}/ — {} nodes, {} edges, {} discrepancies ({errors} error, {warnings} warning)",
        survey.nodes.len(),
        survey.edges.len(),
        survey.findings.len()
    );
}

fn findings(root: &Path) {
    let survey = Survey::build(root);
    for f in &survey.findings {
        let sev = match f.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        };
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
