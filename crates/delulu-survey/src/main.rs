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

    let f = flags(&args);
    // A node id never begins with `-`, so the flags cannot swallow one and a flag cannot be
    // mistaken for one.
    let pos: Vec<&str> = args[1..].iter().map(String::as_str).filter(|a| !a.starts_with('-')).collect();
    // `--depth` takes a value, and that value is a positional-looking number that belongs to it.
    let pos: Vec<&str> = if args.iter().any(|a| a == "--depth") {
        pos.into_iter().filter(|a| a.parse::<u32>().is_err()).collect()
    } else {
        pos
    };

    match verb {
        "build" => build(&root, false, f.json),
        "check" => build(&root, true, f.json),
        "findings" => findings(&root, f.json),
        "query" | "rdeps" => match pos.first() {
            Some(id) => query(&root, id, verb == "rdeps", f.json),
            None => usage("query and rdeps need a node id, e.g. `crate:delulu-check`"),
        },
        "impact" | "affected-by" => match pos.first() {
            Some(id) => walk(&root, id, verb == "impact", f.depth, f.json),
            None => usage(&format!("{verb} needs a node id, e.g. `crate:delulu-check`")),
        },
        "path" => match (pos.first(), pos.get(1)) {
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
           code:DL0501         ruling:S10-D64                     finding:C69\n\n\
         MACHINE OUTPUT\n  \
           Add `--json` to any verb above for ONE object on stdout, carrying `tool`, `verb` and\n  \
           `schema` so a caller can branch before reading anything else. Every edge and every hop\n  \
           keeps its `via: {{kind, file, line}}` citation — the map never asserts a relation to a\n  \
           machine that it would not point a human at. `query --json` always carries `entrenched`,\n  \
           null when the node is ordinary, so \"may I change this?\" is never merely unanswered.\n  \
           The JSON walk is UNCAPPED; only the human render is truncated.\n  \
           An option this tool does not know is refused, never ignored.\n"
    );
}

/// Everything the flags carry, parsed once.
struct Flags {
    json: bool,
    depth: u32,
}

/// Parse the flags, and **refuse any option nobody understood**.
///
/// This tool used to accept `--json` and print human text anyway: the flag was neither implemented
/// nor rejected, so a caller that asked for machine output got prose and an exit code of 0. That is
/// the defect the main CLI closed in campaign finding C76 — *"an option nobody understood is refused,
/// never ignored"* — and the rule did not reach this binary because it is a separate program written
/// before the lesson. A tool whose audience is increasingly machines is the worst place to keep it:
/// a human notices prose where JSON should be, a pipeline does not.
fn flags(args: &[String]) -> Flags {
    let mut f = Flags { json: false, depth: delulu_survey::MAX_WALK_DEPTH };
    let mut unknown: Vec<&str> = Vec::new();
    let mut i = 1; // args[0] is the verb
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--json" => f.json = true,
            "--depth" => match args.get(i + 1).and_then(|s| s.parse::<u32>().ok()) {
                Some(n) => {
                    f.depth = n.clamp(1, delulu_survey::MAX_WALK_DEPTH);
                    i += 1;
                }
                None => usage("--depth needs a positive number"),
            },
            _ if a.starts_with('-') => unknown.push(a),
            _ => {} // a positional: a node id
        }
        i += 1;
    }
    if !unknown.is_empty() {
        eprintln!(
            "error: delulu-survey does not know {}: {}\n  \
             nothing was done — an option nobody understood is refused, never ignored\n\
             note: `delulu-survey --help` lists what this tool accepts",
            if unknown.len() == 1 { "this option" } else { "these options" },
            unknown.join(", ")
        );
        std::process::exit(2);
    }
    f
}

/// The envelope every `--json` answer is wrapped in, so a caller can branch on `verb` and `schema`
/// before it knows anything else. Mirrors the CLI's contract in `docs/for-agents.md`: **one object,
/// on stdout, never coloured, never localized.**
fn envelope(verb: &str, payload: serde_json::Value) -> String {
    serde_json::to_string_pretty(&delulu_survey::answers::envelope(verb, payload)).unwrap_or_else(|_| "{}".into())
}

/// Everything reachable, grouped by distance, every hop cited.
///
/// Grouping by depth rather than printing each full chain is what keeps the answer readable
/// without weakening it: each line names the node it came from, and that node is listed one group
/// above, so the whole chain is recoverable by reading upward. `path` prints one chain in full when
/// that is the question.
fn walk(root: &Path, id: &str, reverse: bool, depth: u32, json: bool) {
    let survey = Survey::build(root);
    if survey.node(id).is_none() {
        not_found(&survey, id);
    }
    let dir = if reverse { delulu_survey::Dir::Incoming } else { delulu_survey::Dir::Outgoing };
    let reached = survey.walk(id, dir, depth);

    let question = if reverse { "what breaks if this changes" } else { "what this rests on" };

    if json {
        // The shared machine answer (`answers::walk_json`), uncapped; `delulu mcp` returns the same.
        let v = delulu_survey::answers::walk_json(&survey, id, reverse, depth).expect("the node exists: checked above");
        println!("{}", serde_json::to_string_pretty(&v).unwrap_or_else(|_| "{}".into()));
        return;
    }

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

fn build(root: &Path, check_only: bool, json: bool) {
    // The same entry point `delulu doctor` uses, so the two commands cannot drift on what "behind
    // the tree" means or on how the files get written.
    let h = delulu_survey::inspect(root, if check_only { Repair::ReportOnly } else { Repair::Regenerate });
    let t = h.tally;

    if check_only {
        if json {
            println!(
                "{}",
                envelope(
                    "check",
                    serde_json::json!({
                        "fresh": h.stale.is_empty(),
                        "stale": h.stale,
                        "nodes": h.nodes,
                        "edges": h.edges,
                    })
                )
            );
            if !h.stale.is_empty() {
                std::process::exit(1);
            }
            return;
        }
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
        if json {
            println!("{}", envelope("build", serde_json::json!({ "wrote": false, "error": e.to_string() })));
        } else {
            eprintln!("error: cannot write {OUTPUT_DIR}: {e}");
        }
        std::process::exit(1);
    }

    if json {
        println!(
            "{}",
            envelope(
                "build",
                serde_json::json!({
                    "wrote": true,
                    "output_dir": OUTPUT_DIR,
                    "nodes": h.nodes,
                    "edges": h.edges,
                    "discrepancies": {
                        "error": t.errors, "warning": t.warnings, "note": t.notes,
                        "total": t.errors + t.warnings + t.notes,
                    },
                })
            )
        );
        return;
    }

    println!(
        "wrote {OUTPUT_DIR}/ — {} nodes, {} edges, {} discrepancies ({} error, {} warning)",
        h.nodes,
        h.edges,
        t.errors + t.warnings + t.notes,
        t.errors,
        t.warnings
    );
}

fn findings(root: &Path, json: bool) {
    let survey = Survey::build(root);
    if json {
        let items: Vec<serde_json::Value> = survey
            .findings
            .iter()
            .map(|f| {
                serde_json::json!({
                    "severity": f.severity.word(),
                    "file": f.file,
                    // 0 means "this finding is about the file, not a line in it" — said explicitly
                    // rather than encoded as a line number that does not exist.
                    "line": if f.line > 0 { serde_json::json!(f.line) } else { serde_json::Value::Null },
                    "class": f.class,
                    "message": f.message,
                })
            })
            .collect();
        println!("{}", envelope("findings", serde_json::json!({ "count": items.len(), "findings": items })));
        return;
    }
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

fn query(root: &Path, id: &str, rdeps_only: bool, json: bool) {
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

    if json {
        // The shared machine answer (`answers::query_json`); `delulu mcp` returns the same.
        let v = delulu_survey::answers::query_json(&survey, id, rdeps_only).expect("the node exists: checked above");
        println!("{}", serde_json::to_string_pretty(&v).unwrap_or_else(|_| "{}".into()));
        return;
    }

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
    delulu_survey::answers::kind_word(k)
}

fn usage(msg: &str) -> ! {
    eprintln!("error: {msg}\n");
    print_help();
    std::process::exit(2);
}
