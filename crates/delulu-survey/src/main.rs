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
        "impact" | "affected-by" => match args.get(1) {
            Some(id) => walk(&root, id, verb == "impact", depth_flag(&args)),
            None => usage(&format!("{verb} needs a node id, e.g. `crate:delulu-check`")),
        },
        "path" => match (args.get(1), args.get(2)) {
            (Some(a), Some(b)) => path(&root, a, b),
            _ => usage("path needs two node ids, e.g. `path crate:delulu crate:delulu-broker`"),
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
           delulu-survey rdeps <id>  what points at it — ONE HOP\n\n\
         TRANSITIVE — the questions you actually have before changing code\n  \
           delulu-survey impact <id>       everything that breaks if this changes\n  \
           delulu-survey affected-by <id>  everything this rests on\n  \
           delulu-survey path <a> <b>      how one reaches the other, hop by hop\n\n  \
         Add `--depth N` to impact/affected-by. Every hop names the file and line it was read\n  \
         from, so a chain can be walked back and disagreed with, exactly like a single edge.\n  \
         `rdeps` answers one hop and says so: `mod:crates/delulu-check/src/check.rs` — the module\n  \
         that decides what type-checks — has ONE structural edge arriving at it, and changing\n  \
         it reaches the whole workspace. Reach for `impact` when the question is blast radius.\n\n\
         IDS\n  \
           crate:delulu-check  mod:crates/delulu-check/src/ty.rs  doc:README.md\n  \
           code:DL0501         ruling:S10-D64                     finding:C69\n"
    );
}

/// `--depth N`, bounded by the walk's own guard.
fn depth_flag(args: &[String]) -> u32 {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--depth" {
            if let Some(n) = args.get(i + 1).and_then(|s| s.parse::<u32>().ok()) {
                return n.clamp(1, delulu_survey::MAX_WALK_DEPTH);
            }
            usage("--depth needs a positive number");
        }
        i += 1;
    }
    delulu_survey::MAX_WALK_DEPTH
}

/// Everything reachable, grouped by distance, every hop cited.
///
/// Grouping by depth rather than printing each full chain is what keeps the answer readable
/// without weakening it: each line names the node it came from, and that node is listed one group
/// above, so the whole chain is recoverable by reading upward. `path` prints one chain in full when
/// that is the question.
fn walk(root: &Path, id: &str, reverse: bool, depth: u32) {
    let survey = Survey::build(root);
    if survey.node(id).is_none() {
        not_found(&survey, id);
    }
    let dir = if reverse { delulu_survey::Dir::Incoming } else { delulu_survey::Dir::Outgoing };
    let reached = survey.walk(id, dir, depth);

    let question = if reverse { "what breaks if this changes" } else { "what this rests on" };
    println!("{id}\n  {question} — {} node(s) reached", reached.len());
    if reached.is_empty() {
        println!("\nnothing. This node is a leaf in that direction.");
        return;
    }

    // Grouped by the node each hop came from, rather than one flat line carrying two ids. An
    // arrow between them would be ambiguous in the reverse direction — the walk goes one way and
    // the edge points the other — and an ambiguous rendering of a provenance chain is worse than a
    // verbose one.
    //
    // The render is bounded for the same reason diagnostics are (campaign finding C32): a
    // saturating answer stops informing anyone. What is withheld is stated, never dropped.
    const MAX_PER_PARENT: usize = 25;
    let max_depth = reached.iter().map(|r| r.depth).max().unwrap_or(0);
    for d in 1..=max_depth {
        let wave: Vec<_> = reached.iter().filter(|r| r.depth == d).collect();
        println!("\n  depth {d} ({})", wave.len());

        let mut parents: Vec<&str> = wave.iter().map(|r| r.from).collect();
        parents.sort();
        parents.dedup();
        for p in parents {
            let kids: Vec<_> = wave.iter().filter(|r| r.from == p).collect();
            println!("    from {p}");
            for r in kids.iter().take(MAX_PER_PARENT) {
                println!("      {:<52} {:<15} @ {}:{}", r.id, kind_word(r.via.kind), r.via.file, r.via.line);
            }
            if let Some(hidden) = kids.len().checked_sub(MAX_PER_PARENT).filter(|n| *n > 0) {
                println!("      … {hidden} more from this node (of {})", kids.len());
            }
        }
    }
}

/// One chain, in full, every hop carrying the line it was read from.
fn path(root: &Path, a: &str, b: &str) {
    let survey = Survey::build(root);
    for id in [a, b] {
        if survey.node(id).is_none() {
            not_found(&survey, id);
        }
    }

    // Directed first, because a dependency direction is the one with an architectural meaning. If
    // there is none, the reverse is reported as the reverse rather than quietly presented as if it
    // were the answer to the question that was asked.
    let (chain, from, to, reversed) = match survey.shortest_path(a, b) {
        Some(c) => (c, a, b, false),
        None => match survey.shortest_path(b, a) {
            Some(c) => (c, b, a, true),
            None => {
                println!("no path between `{a}` and `{b}`, in either direction.");
                println!("\nThe map only draws relations it read from a file. Two nodes with no");
                println!("chain between them are genuinely unconnected in this repository's text.");
                std::process::exit(1);
            }
        },
    };

    if reversed {
        println!("no path from `{a}` to `{b}` — but `{b}` reaches `{a}`:");
    }
    println!("{from} -> {to}  ({} hop(s))", chain.len());
    for e in &chain {
        println!("  {:<44} --{}--> {:<44} @ {}:{}", e.from, kind_word(e.kind), e.to, e.file, e.line);
    }
}

fn not_found(survey: &Survey, id: &str) -> ! {
    eprintln!("error: no node `{id}`");
    let near: Vec<&str> = survey
        .nodes
        .iter()
        .filter(|n| n.id.contains(id) || n.name.contains(id))
        .map(|n| n.id.as_str())
        .take(10)
        .collect();
    if !near.is_empty() {
        eprintln!("did you mean:\n  {}", near.join("\n  "));
    }
    std::process::exit(1);
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
    // Printed before the edges, because "may I change this?" is answered before "what breaks if I
    // do?" is worth asking. Constitution §10 / invariant 44.
    if let Some(e) = &node.entrenched {
        println!(
            "  ENTRENCHED — changing this needs {} specifically, not any maintainer\n\
             \x20         matched by `{}` at {}:{}\n\
             \x20         Constitution §10 requires an entrenchment analysis (invariant 44) before it moves",
            e.owner, e.pattern, e.file, e.line
        );
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
