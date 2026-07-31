//! The three channels the Survey publishes on.
//!
//! - `survey.json` — the machine channel, schema `survey/1`, for tools and agents.
//! - `SURVEY.md` — the human map: what the parts are, how they depend on each other, where a
//!   given kind of question is answered.
//! - `DISCREPANCIES.md` — everything the cross-check pass could not reconcile.
//!
//! All three are pure functions of a [`Survey`], and a [`Survey`] is a pure function of the tree.
//! Two runs on the same commit produce byte-identical files on any platform, which is what lets a
//! test assert that the committed copy is current.

use crate::{EdgeKind, NodeKind, Severity, Survey};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// The machine channel: pretty at the top level, one compact object per line inside the arrays.
///
/// Fully pretty-printed this file is 1.8 MB and sixty-three thousand lines, and every regeneration
/// produces an unreadable diff. Fully compact it is one line, and the diff is worse. One record per
/// line keeps it a fraction of the size *and* makes a changed edge show up as a changed line, which
/// is what a reviewer needs when the gate says the map moved.
pub fn json(s: &Survey) -> String {
    let mut o = String::from("{\n");
    let _ = writeln!(o, "  \"schema\": {},", serde_json::to_string(s.schema).unwrap_or_default());
    let facts = serde_json::to_string_pretty(&s.facts).unwrap_or_default().replace('\n', "\n  ");
    let _ = writeln!(o, "  \"facts\": {facts},");
    array(&mut o, "nodes", &s.nodes, true);
    array(&mut o, "edges", &s.edges, true);
    array(&mut o, "findings", &s.findings, false);
    o.push_str("}\n");
    o
}

fn array<T: serde::Serialize>(o: &mut String, name: &str, items: &[T], comma: bool) {
    if items.is_empty() {
        let _ = writeln!(o, "  \"{name}\": []{}", if comma { "," } else { "" });
        return;
    }
    let _ = writeln!(o, "  \"{name}\": [");
    for (i, item) in items.iter().enumerate() {
        let sep = if i + 1 == items.len() { "" } else { "," };
        let _ = writeln!(o, "    {}{sep}", serde_json::to_string(item).unwrap_or_default());
    }
    let _ = writeln!(o, "  ]{}", if comma { "," } else { "" });
}

fn crate_name(id: &str) -> &str {
    id.trim_start_matches("crate:")
}

pub fn markdown(s: &Survey) -> String {
    let mut o = String::new();
    let f = &s.facts;

    o.push_str(
        "# The DeluluLang Survey\n\n\
         **Generated — do not edit by hand.** Regenerate with `cargo run -p delulu-survey -- build`.\n\n\
         This is the map of the *repository*: what the parts are, how they actually depend on one\n\
         another, and where a given kind of question is answered. It is derived only from the\n\
         repository's own text.\n\n\
         > **The provenance law.** Every edge in this map names the file and line it was read from,\n\
         > and every edge whose target is a path was checked to exist. Nothing here is inferred from\n\
         > name similarity, embeddings, or proximity. A relation that cannot be pointed at in the\n\
         > text is not in the map — it is a [discrepancy](DISCREPANCIES.md) instead.\n\n\
         Not to be confused with the **Atlas** (`crates/delulu-atlas`), which maps a checked Delulu\n\
         *program* from compiler facts. The Atlas needs the compiler to work; the Survey reads\n\
         files, so it still opens when the tree does not build.\n\n",
    );

    // --- facts ---------------------------------------------------------------------------------
    o.push_str("## Measured facts\n\n| | |\n|---|---:|\n");
    let _ = writeln!(o, "| Workspace members | {} |", f.crates);
    let _ = writeln!(o, "| … shipped language crates | {} |", f.crates_shipped);
    let _ = writeln!(o, "| … repository tooling (`publish = false`) | {} |", f.crates - f.crates_shipped);
    let _ = writeln!(o, "| Rust files | {} |", f.rust_files);
    let _ = writeln!(o, "| Rust lines | {} |", f.rust_lines);
    let _ = writeln!(o, "| Rust files outside `src/` (test/bench targets) | {} |", f.test_suites);
    let _ = writeln!(o, "| Markdown documents | {} |", f.doc_files);
    let _ = writeln!(o, "| Markdown lines | {} |", f.doc_lines);
    let _ = writeln!(o, "| DeluluLang programs | {} |", f.delulu_programs);
    let _ = writeln!(o, "| Registered diagnostic codes | {} |", f.registered_codes);
    let _ = writeln!(o, "| Recorded rulings | {} |", f.rulings);
    let _ = writeln!(o, "| Recorded campaign findings | {} |", f.findings_recorded);
    let _ = writeln!(o, "| Nodes / edges in this map | {} / {} |", s.nodes.len(), s.edges.len());
    let _ = writeln!(o, "| Open discrepancies | {} |", s.findings.len());
    o.push_str(
        "\nLines are counted as text lines. Test *counts* are not here: the number of passing tests\n\
         is produced by `cargo test`, not by reading files, and the Survey does not restate numbers\n\
         it did not measure.\n\n",
    );

    // --- the crate graph -----------------------------------------------------------------------
    o.push_str("## How the crates depend on each other\n\n");
    o.push_str("Read from the `path = \"../…\"` entries in each `Cargo.toml`.\n\n```mermaid\ngraph TD\n");
    let mut edges: Vec<(&str, &str)> = s
        .edges
        .iter()
        .filter(|e| e.kind == EdgeKind::DependsOn && e.from != "workspace")
        .map(|e| (crate_name(&e.from), crate_name(&e.to)))
        .collect();
    edges.sort_unstable();
    edges.dedup();
    for (from, to) in &edges {
        let _ = writeln!(o, "  {}[\"{from}\"] --> {}[\"{to}\"]", ident(from), ident(to));
    }
    o.push_str("```\n\n");

    // --- per crate -----------------------------------------------------------------------------
    o.push_str("## The crates\n\n");
    let mut deps: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut rdeps: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (from, to) in &edges {
        deps.entry(from).or_default().push(to);
        rdeps.entry(to).or_default().push(from);
    }

    for n in s.nodes.iter().filter(|n| n.kind == NodeKind::Crate && n.id != "workspace") {
        let name = crate_name(&n.id);
        let _ = writeln!(o, "### `{name}`\n");
        if let Some(d) = &n.summary {
            let _ = writeln!(o, "{d}\n");
        }
        let d = deps.get(name).map(|v| join_code(v)).unwrap_or_else(|| "—".into());
        let r = rdeps.get(name).map(|v| join_code(v)).unwrap_or_else(|| "—".into());
        let _ = writeln!(o, "- **Depends on:** {d}");
        let _ = writeln!(o, "- **Depended on by:** {r}  ← change this crate, and these must be re-checked");

        let prefix = format!("crates/{name}/");
        let mods: Vec<&crate::Node> = s
            .nodes
            .iter()
            .filter(|m| m.kind == NodeKind::RustModule)
            .filter(|m| m.path.as_deref().is_some_and(|p| p.starts_with(&prefix)))
            .collect();
        if !mods.is_empty() {
            let total: u32 = mods.iter().filter_map(|m| m.lines).sum();
            let _ = writeln!(o, "- **Modules:** {} files, {total} lines\n", mods.len());
            o.push_str("| Module | Lines | What it is |\n|---|---:|---|\n");
            for m in mods {
                let short = m.path.as_deref().unwrap_or("").trim_start_matches(&prefix);
                let summary = m.summary.as_deref().unwrap_or("").replace('|', "\\|");
                let summary = truncate(&summary, 110);
                let _ = writeln!(o, "| `{short}` | {} | {summary} |", m.lines.unwrap_or(0));
            }
        }
        o.push('\n');
    }

    // --- diagnostic codes ----------------------------------------------------------------------
    o.push_str("## Diagnostic codes\n\n");
    o.push_str(
        "Allocated in `crates/delulu-diag/src/codes.rs` and never reused. Below, each code's\n\
         **range** is mapped to the crates whose source raises it — read from the source, so it is\n\
         where the code is *produced*, not where someone wrote its number in a comment.\n\n",
    );
    let mut by_range: BTreeMap<String, (usize, Vec<&str>)> = BTreeMap::new();
    for n in s.nodes.iter().filter(|n| n.kind == NodeKind::DiagnosticCode) {
        let range = format!("DL{}xx", &n.name[2..4]);
        let e = by_range.entry(range).or_insert((0, Vec::new()));
        e.0 += 1;
        for edge in s.edges.iter().filter(|e| e.to == n.id && e.kind == EdgeKind::RaisesCode) {
            if let Some(c) = edge.file.strip_prefix("crates/").and_then(|r| r.split_once('/')).map(|(c, _)| c) {
                e.1.push(c);
            }
        }
    }
    o.push_str("| Range | Codes | Raised in |\n|---|---:|---|\n");
    for (range, (count, mut crates)) in by_range {
        crates.sort_unstable();
        crates.dedup();
        let _ = writeln!(o, "| `{range}` | {count} | {} |", join_code(&crates));
    }

    // --- where decisions live ------------------------------------------------------------------
    o.push_str("\n## Where the decisions live\n\n");
    o.push_str(
        "Rulings are allocated **one namespace per stage**, so `D21` alone is ambiguous — the Stage-9\n\
         `D21` and the Stage-10 `D21` are different decisions. The Survey namespaces them `S<stage>-D<n>`.\n\n",
    );
    let mut ns: BTreeMap<&str, usize> = BTreeMap::new();
    for n in s.nodes.iter().filter(|n| n.kind == NodeKind::Ruling) {
        if let Some((prefix, _)) = n.name.split_once("-D") {
            *ns.entry(prefix).or_default() += 1;
        }
    }
    o.push_str("| Namespace | Rulings | Allocated in |\n|---|---:|---|\n");
    for (prefix, count) in ns {
        let stage = prefix.trim_start_matches('S');
        let _ = writeln!(o, "| `{prefix}` | {count} | `docs/design/STAGE{stage}_BUILD_ORDER.md` |");
    }

    // --- reading order -------------------------------------------------------------------------
    o.push_str(
        "\n## Where to start\n\n\
         | If you want to… | Read |\n|---|---|\n\
         | understand the promise | `docs/design/CONSTITUTION.md` |\n\
         | use the language | `docs/GETTING_STARTED.md` |\n\
         | drive it from a program | `docs/for-agents.md` |\n\
         | change the type or effect system | `crates/delulu-check/` |\n\
         | change what a grant means | `crates/delulu-broker/` — and read the campaign first |\n\
         | know why something is the way it is | the ruling namespaces above |\n\
         | know what is still wrong | [DISCREPANCIES.md](DISCREPANCIES.md) |\n\n\
         ### Querying this map\n\n```\n\
         cargo run -p delulu-survey -- query crate:delulu-check   # neighbourhood of a node\n\
         cargo run -p delulu-survey -- rdeps crate:delulu-diag    # what breaks if I change it\n\
         cargo run -p delulu-survey -- findings                   # the discrepancy list\n\
         cargo run -p delulu-survey -- check                      # is this map current?\n```\n",
    );
    o
}

pub fn discrepancies(s: &Survey) -> String {
    let mut o = String::new();
    o.push_str(
        "# Survey discrepancies\n\n\
         **Generated — do not edit by hand.** Regenerate with `cargo run -p delulu-survey -- build`.\n\n\
         Each entry is a place where two parts of this repository disagree, or where something points\n\
         at what is not there. Every one names the file and line that produced it. Entries are not\n\
         graded by how bad they look — an `error` is a statement that contradicts the tree, a\n\
         `warning` is true today and drifting, a `note` is for a human to judge.\n\n",
    );

    if s.findings.is_empty() {
        o.push_str("No discrepancies. Every extracted relation was corroborated by a second source.\n");
        return o;
    }

    let mut by_class: BTreeMap<(&Severity, &str), Vec<&crate::Finding>> = BTreeMap::new();
    for f in &s.findings {
        by_class.entry((&f.severity, &f.class)).or_default().push(f);
    }

    o.push_str("| Severity | Class | Count |\n|---|---|---:|\n");
    for ((sev, class), items) in &by_class {
        let _ = writeln!(o, "| {} | `{class}` | {} |", severity_word(sev), items.len());
    }
    o.push('\n');

    for ((sev, class), items) in &by_class {
        let _ = writeln!(o, "## {} — `{class}` ({})\n", severity_word(sev), items.len());
        if let Some(first) = items.first() {
            if !first.remedy.is_empty() {
                let _ = writeln!(o, "**What to do:** {}\n", first.remedy);
            }
        }
        for f in items {
            if f.line > 0 {
                let _ = writeln!(o, "- `{}:{}` — {}", f.file, f.line, f.message);
            } else {
                let _ = writeln!(o, "- `{}` — {}", f.file, f.message);
            }
        }
        o.push('\n');
    }
    o
}

fn severity_word(s: &Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
    }
}

fn join_code(v: &[&str]) -> String {
    let mut v = v.to_vec();
    v.sort_unstable();
    v.dedup();
    v.iter().map(|s| format!("`{s}`")).collect::<Vec<_>>().join(", ")
}

/// Mermaid node ids may not contain `-`.
fn ident(name: &str) -> String {
    name.replace('-', "_")
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    let cut: String = s.chars().take(n).collect();
    match cut.rsplit_once(' ') {
        Some((head, _)) => format!("{head} …"),
        None => format!("{cut} …"),
    }
}
