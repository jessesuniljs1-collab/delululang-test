//! Cross-checking — the pass that decides whether an extracted relation is trustworthy.
//!
//! Nothing here re-reads the tree looking for new relations. Every check takes something one
//! extractor *claimed* and holds it against an independent source: source text against manifests,
//! `mod` declarations against the filesystem, cited ids against the registries that allocate them.
//! Where the two agree, the edge stands. Where they disagree, the disagreement is the finding —
//! the Survey never picks a winner and never quietly drops the loser.

use crate::paths::PathIndex;
use crate::scan::{FileKind, ScannedFile};
use crate::{Builder, EdgeKind, NodeKind, Severity};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{DefaultHasher, Hash, Hasher};

pub fn cross_check(idx: &PathIndex, files: &[ScannedFile], b: &mut Builder) {
    code_registry_exists(idx, b);
    dependencies_agree_with_source(b);
    cited_codes_are_registered(b);
    resolve_ruling_citations(b);
    resolve_finding_citations(b);
    every_crate_is_a_member(files, b);
    modules_are_reachable(files, b);
    identical_files(files, b);
    build_orders_allocate_rulings(files, b);
    quoted_counts_match_the_tree(files, b);
}

/// A document that describes the present is a document whose numbers can go stale.
///
/// Historical records are exempt, and the distinction is the whole difficulty. A build order
/// saying "the suite stood at 1098 at close-out" is *correct forever*; the front door saying "12
/// crates" is a claim about now. Checking the first kind would be a bug in the checker, so only
/// documents that speak in the present tense are checked, and the list is explicit rather than
/// inferred.
fn describes_the_present(rel: &str) -> bool {
    if rel.contains("_BUILD_ORDER.md")
        || rel == "CHANGELOG.md"
        || rel.starts_with("docs/release/")
        // A dated audit quotes the numbers it found *in order to correct them*. Checking those
        // against today's tree would report the record of a fix as the very drift it recorded.
        || rel == "docs/survey/AUDIT.md"
        // The V1 archive is where phase V2-0 put the documents that are records rather than
        // descriptions — superseded drafts, dated campaign passes, process records, executed
        // planning material. Every one of them is *about a moment*: "the suite stood at 1645"
        // was true when it was written and is still a true record of that day. Checking those
        // counts against today's tree would report correct history as drift, and the only way
        // to silence it would be to edit the record — which is the one thing an archive exists
        // to prevent. One prefix, so nothing has to be added here as the archive grows.
        || rel.starts_with("docs/archive/")
    {
        return false; // records of what was true at a moment, and still are records of it
    }
    matches!(rel, "README.md" | "CONTRIBUTING.md" | "SECURITY.md" | "GOVERNANCE.md")
        || rel.starts_with("docs/")
}

/// Facts a document might quote, and what the tree actually says.
///
/// The fourth element is a guard: a word that must also appear on the line before the phrase is
/// believed. "175 files" means Rust files in "12 crates, 175 files" and means nothing at all in a
/// sentence about conformance fixtures, so the bare unit is only trusted in company.
type Fact = (&'static [&'static str], u64, &'static str, Option<&'static str>);

fn measured(files: &[ScannedFile], b: &Builder) -> Vec<Fact> {
    let rust_files = files.iter().filter(|f| matches!(f.kind, FileKind::RustSource | FileKind::RustTest)).count();
    let rust_lines: u32 =
        files.iter().filter(|f| matches!(f.kind, FileKind::RustSource | FileKind::RustTest)).map(|f| f.lines).sum();
    let crates = b.nodes.values().filter(|n| n.kind == NodeKind::Crate && n.id != "workspace").count();
    let shipped = crates - b.tooling_crates.len();

    vec![
        (&["crates"], shipped as u64, "shipped language crates", None),
        (&["workspace members"], crates as u64, "workspace members", None),
        (&["lines of Rust", "lines of rust"], rust_lines as u64, "lines of Rust", None),
        (&["Rust files"], rust_files as u64, "Rust files", None),
        (&["files"], rust_files as u64, "Rust files", Some("crates")),
        (
            &["registered codes", "diagnostic codes"],
            b.defined_codes.len() as u64,
            "registered diagnostic codes",
            None,
        ),
    ]
}

fn quoted_counts_match_the_tree(files: &[ScannedFile], b: &mut Builder) {
    let facts = measured(files, b);
    let mut hits: Vec<(String, u32, String)> = Vec::new();
    let mut unverifiable: Vec<(String, u32)> = Vec::new();

    for f in files {
        if f.kind != FileKind::Doc || !describes_the_present(&f.rel) {
            continue;
        }
        let lines: Vec<&str> = f.text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let lineno = i as u32 + 1;
            // A number can be separated from its unit by a **line wrap** — `193` ending one line
            // and `files.` beginning the next. This check used to read one line at a time, so a
            // text formatter's line break hid a stale count from it for months, while the
            // surrounding prose promised that "a stale figure here now fails a test" (C81). Reading
            // the following line closes that; `num_start` keeps a number that belongs to the next
            // line from being counted twice, since that line gets its own turn.
            let joined = match lines.get(i + 1) {
                Some(next) => format!("{line} {}", next.trim_start()),
                None => (*line).to_string(),
            };
            // This check's own advice is "update the number, or say plainly that it is a snapshot of
            // a past moment" — and until now there was no way to do the second. A document that
            // records how a figure CHANGED has to be able to quote the old one. The marker is an
            // explicit ISO date on the line: deliberate, greppable, and not something a stale figure
            // acquires by accident.
            let dated_history = joined
                .match_indices("20")
                .any(|(k, _)| {
                    let b = joined.as_bytes();
                    k + 10 <= b.len()
                        && b[k + 2].is_ascii_digit()
                        && b[k + 3].is_ascii_digit()
                        && b[k + 4] == b'-'
                        && b[k + 7] == b'-'
                });
            if dated_history {
                continue;
            }
            for (claimed, num_start, unit_at, approx) in numbers_with_units(&joined) {
                if num_start >= line.len() {
                    continue; // the number lives on the next line; it is checked there
                }
                for (phrases, actual, label, guard) in &facts {
                    if !phrases.iter().any(|p| joined[unit_at..].starts_with(p)) {
                        continue;
                    }
                    if guard.is_some_and(|g| !joined.contains(g)) {
                        continue;
                    }
                    // An approximate claim is allowed to be approximate; a precise one is not.
                    let ok = if approx {
                        let slack = (*actual as f64 * 0.1).max(1.0);
                        (claimed as f64 - *actual as f64).abs() <= slack
                    } else {
                        claimed == *actual
                    };
                    if !ok {
                        hits.push((
                            f.rel.clone(),
                            lineno,
                            format!("says {claimed} {label}; the tree has {actual}"),
                        ));
                    }
                }
                // Test counts come from `cargo test`, not from reading files. The Survey will not
                // check a number it cannot measure — but it will say where those numbers are, so
                // that after a suite run a human knows exactly which lines to revisit.
                // Only phrasings that assert a *current* total. A spec saying "≥1 accepting test
                // per rule" is a rule, not a count, and listing it here would bury the handful of
                // lines that genuinely go stale after every suite run.
                if ["test suites", "tests passing", "suites passing", "tests pass,", "tests green"]
                    .iter()
                    .any(|p| joined[unit_at..].starts_with(p))
                {
                    unverifiable.push((f.rel.clone(), lineno));
                }
            }
        }
    }

    for (file, line, message) in hits {
        b.finding(
            Severity::Warning,
            "stale-count",
            &file,
            line,
            message,
            "update the number, or say plainly that it is a snapshot of a past moment",
        );
    }
    if !unverifiable.is_empty() {
        let where_ = unverifiable.iter().map(|(f, l)| format!("{f}:{l}")).collect::<Vec<_>>().join(", ");
        b.finding(
            Severity::Note,
            "test-count-quoted-but-unverifiable",
            "README.md",
            0,
            format!(
                "these lines quote a test count, which is produced by `cargo test` and cannot be read \
                 out of the tree — verify them against a suite run: {where_}"
            ),
            "re-run the suite and update these by hand; the Survey deliberately does not guess",
        );
    }
}

/// `(value, byte offset just past the number and any spaces, approximate?)` for each number in the
/// line. `~82,000` and "about 82,000" are approximate; `12` is not.
/// Returns `(value, number_start, unit_start, is_approximate)`. `number_start` is carried so a
/// caller joining two lines can tell which line the number itself came from.
fn numbers_with_units(line: &str) -> Vec<(u64, usize, usize, bool)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        // Don't read the `4` of `DL0401` or the `10` of `Stage-10` as a quantity.
        let prev = line[..i].chars().next_back();
        if prev.is_some_and(|c| c.is_alphanumeric() || c == '-' || c == '.' || c == '_') {
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b',') {
                i += 1;
            }
            continue;
        }
        let start = i;
        while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b',') {
            i += 1;
        }
        let raw = line[start..i].trim_end_matches(',');
        let Ok(value) = raw.replace(',', "").parse::<u64>() else { continue };

        let mut j = i;
        while j < bytes.len() && bytes[j] == b' ' {
            j += 1;
        }
        let head = &line[..start];
        let approx = head.ends_with('~')
            || head.ends_with("about ")
            || head.ends_with("roughly ")
            || head.ends_with("approximately ")
            || head.ends_with("over ")
            || head.ends_with("more than ")
            || head.ends_with("nearly ");
        out.push((value, start, j, approx));
    }
    out
}

/// The one hardcoded path in the extractor, checked so the assumption cannot fail silently.
fn code_registry_exists(idx: &PathIndex, b: &mut Builder) {
    if !idx.exists(crate::rust::CODE_REGISTRY) {
        b.finding(
            Severity::Error,
            "code-registry-moved",
            crate::rust::CODE_REGISTRY,
            1,
            "the diagnostic code registry is not at the path the Survey reads it from, so no code \
             is recorded as *defined* and every citation looks unregistered"
                .to_string(),
            "update `CODE_REGISTRY` in crates/delulu-survey/src/rust.rs to the new location",
        );
    }
}

/// A `use delulu_x::` must be backed by a manifest dependency, and vice versa.
///
/// These are two genuinely independent statements of the same fact — one by the author of the code,
/// one by the author of the manifest — which is what makes their disagreement worth reporting.
fn dependencies_agree_with_source(b: &mut Builder) {
    let mut declared: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut used: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut first_use: BTreeMap<(String, String), (String, u32)> = BTreeMap::new();

    for e in &b.edges {
        match e.kind {
            EdgeKind::DependsOn if e.from != "workspace" => {
                declared.entry(e.from.clone()).or_default().insert(e.to.clone());
            }
            EdgeKind::Uses => {
                // Attribute the use to the crate that owns the file.
                if let Some(owner) = owning_crate(&e.file) {
                    used.entry(format!("crate:{owner}")).or_default().insert(e.to.clone());
                    first_use
                        .entry((format!("crate:{owner}"), e.to.clone()))
                        .or_insert((e.file.clone(), e.line));
                }
            }
            _ => {}
        }
    }

    for (krate, uses) in &used {
        let empty = BTreeSet::new();
        let deps = declared.get(krate).unwrap_or(&empty);
        for target in uses {
            if target == krate || deps.contains(target) {
                continue;
            }
            let (file, line) = first_use.get(&(krate.clone(), target.clone())).cloned().unwrap_or_default();
            b.finding(
                Severity::Error,
                "undeclared-dependency",
                &file,
                line,
                format!(
                    "{} uses `{}` but its manifest does not depend on it",
                    krate.trim_start_matches("crate:"),
                    target.trim_start_matches("crate:")
                ),
                "add the dependency, or drop the use",
            );
        }
    }

    for (krate, deps) in &declared {
        let empty = BTreeSet::new();
        let uses = used.get(krate).unwrap_or(&empty);
        for target in deps {
            if uses.contains(target) {
                continue;
            }
            b.finding(
                Severity::Note,
                "dependency-never-used-in-source",
                &format!("{}/Cargo.toml", krate.trim_start_matches("crate:")),
                0,
                format!(
                    "{} depends on `{}`, which no source file names",
                    krate.trim_start_matches("crate:"),
                    target.trim_start_matches("crate:")
                ),
                "confirm it is reached through a re-export or a feature, or remove it",
            );
        }
    }
}

fn owning_crate(file: &str) -> Option<String> {
    file.strip_prefix("crates/").and_then(|r| r.split_once('/')).map(|(c, _)| c.to_string())
}

/// Codes named somewhere in the repository that the registry does not allocate **and that nothing
/// explains**.
///
/// This check used to report every such code, because nothing distinguished "retired on purpose"
/// from "typo" except a sentence in a comment next to it. That record now exists — the
/// `UNALLOCATED` table beside the registry gives each one a disposition and a reason, `delulu
/// explain` answers from it, and this check subtracts it.
///
/// **What is left is the part that was always worth reporting.** A code named in the tree that is
/// in neither table is either a typo or a decision nobody wrote down, and both are worth a
/// person's attention. Adding a row to `UNALLOCATED` is how you answer it; there is deliberately
/// no way to silence it without saying something.
///
/// Still a note rather than an error: the remedy is to record a decision, and a check that fails
/// the build over an unrecorded one would be graded harsher than the facts warrant.
fn cited_codes_are_registered(b: &mut Builder) {
    let unregistered: Vec<String> = b
        .cited_codes
        .difference(&b.defined_codes)
        .filter(|c| !b.dispositioned_codes.contains(*c))
        .cloned()
        .collect();
    if unregistered.is_empty() {
        return;
    }
    let mut sites: Vec<String> = Vec::new();
    for code in &unregistered {
        let site = b
            .edges
            .iter()
            .find(|e| e.to == format!("code:{code}") && e.kind != EdgeKind::DefinesCode)
            .map(|e| format!("{} ({}:{})", code, e.file, e.line));
        sites.push(site.unwrap_or_else(|| code.clone()));
    }
    b.finding(
        Severity::Note,
        "code-cited-but-not-allocated",
        crate::rust::CODE_REGISTRY,
        0,
        format!(
            "{} code(s) are named in the tree, are not allocated by the registry, and have no \
             recorded disposition — so nothing says whether each is a retired code or a typo: {}",
            unregistered.len(),
            sites.join("; ")
        ),
        "add a row to `UNALLOCATED` in the registry saying which it is, or fix the citation",
    );
}

/// A build order that records no citable decisions.
///
/// Stages 9 and 10 allocate numbered rulings; stages 6, 7 and 8 do not, so a decision taken during
/// those stages cannot be cited by anything. This is measured, not judged: the check reports the
/// asymmetry and leaves what to do about it to a human.
fn build_orders_allocate_rulings(files: &[ScannedFile], b: &mut Builder) {
    let with_rulings: BTreeSet<String> =
        b.defined_rulings.iter().filter_map(|id| id.split_once("-D").map(|(ns, _)| ns.to_string())).collect();

    for f in files {
        let Some(name) = f.rel.rsplit('/').next() else { continue };
        let Some(stage) = name.strip_suffix("_BUILD_ORDER.md").and_then(|n| n.strip_prefix("STAGE")) else {
            continue;
        };
        if !with_rulings.contains(&format!("S{stage}")) {
            b.finding(
                Severity::Note,
                "build-order-without-citable-rulings",
                &f.rel,
                1,
                format!(
                    "this build order allocates no numbered rulings, so a Stage-{stage} decision cannot be \
                     cited the way a Stage-9 or Stage-10 one can"
                ),
                "no action if the stage genuinely took no recorded decisions; otherwise number them",
            );
        }
    }
}

/// Resolve `D<n>` to a namespaced ruling, and report the cases where it cannot be done.
fn resolve_ruling_citations(b: &mut Builder) {
    // number -> the namespaces that allocate it
    let mut by_number: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for id in &b.defined_rulings {
        if let Some((ns, num)) = id.split_once("-D") {
            by_number.entry(num.to_string()).or_default().push(ns.to_string());
        }
    }

    let cites = std::mem::take(&mut b.raw_ruling_cites);
    let mut by_default: BTreeMap<String, usize> = BTreeMap::new();
    let mut undefined: BTreeMap<String, usize> = BTreeMap::new();

    for (file, line, raw) in cites {
        // `S9-21` carries its stage; `21` does not.
        let (explicit, num) = match raw.split_once('-') {
            Some((stage, n)) => (Some(stage.to_string()), n.to_string()),
            None => (None, raw),
        };

        let own = file
            .rsplit('/')
            .next()
            .and_then(|n| n.strip_suffix("_BUILD_ORDER.md"))
            .and_then(|n| n.strip_prefix("STAGE"))
            .map(|s| format!("S{s}"));

        let Some(namespaces) = by_number.get(&num) else {
            *undefined.entry(format!("D{num}")).or_default() += 1;
            continue;
        };

        // In order of how much the text actually says:
        //   1. an explicit `S9-D21`;
        //   2. a build order citing its own numbers;
        //   3. a number only one stage ever allocated;
        //   4. the convention this repository documents — bare `D<n>` means the latest stage
        //      (`CHANGELOG.md`, and the Stage-10 ledger heading: "Stage-9 rulings are cited as
        //      S9-D<n>"). Followed rather than second-guessed, and counted so the reliance on it
        //      is visible.
        let (ns, defaulted) = match &explicit {
            Some(s) if namespaces.contains(s) => (s.clone(), false),
            _ => match &own {
                Some(s) if namespaces.contains(s) => (s.clone(), false),
                _ if namespaces.len() == 1 => (namespaces[0].clone(), false),
                _ => (latest_stage(namespaces), true),
            },
        };
        if defaulted {
            *by_default.entry(format!("D{num}")).or_default() += 1;
        }
        let from = crate::rust::path_node_id(&file);
        b.edge(EdgeKind::Cites, &from, &format!("ruling:{ns}-D{num}"), &file, line);
    }

    // One note for the whole pattern, not one per number. The convention is real and documented;
    // what is worth a reader's attention is how much weight it carries silently.
    let total: usize = by_default.values().sum();
    if total > 0 {
        let numbers: Vec<String> = by_default.keys().cloned().collect();
        b.finding(
            Severity::Note,
            "ruling-cited-without-its-stage",
            "docs/design",
            0,
            format!(
                "{total} citation(s) of {} name a ruling number that more than one stage allocates, and \
                 rely on the documented default that a bare `D<n>` means the latest stage \
                 (`CHANGELOG.md`, `STAGE10_BUILD_ORDER.md` §2). Numbers affected: {}",
                if numbers.len() == 1 { "a ruling" } else { "rulings" },
                numbers.join(", ")
            ),
            "write the stage when you mean an earlier one — `S9-D21` — as the Stage-10 ledger already does",
        );
    }
    for (id, count) in undefined {
        b.finding(
            Severity::Note,
            "citation-of-unallocated-ruling",
            "docs/design",
            0,
            format!("`{id}` is cited {count} time(s) but no build order allocates it"),
            "check the number, or record the ruling where it belongs",
        );
    }
}

/// The highest-numbered stage among the namespaces that allocate a number — "the latest stage",
/// compared numerically so `S10` beats `S9` rather than losing to it alphabetically.
fn latest_stage(namespaces: &[String]) -> String {
    namespaces
        .iter()
        .max_by_key(|s| s.trim_start_matches('S').parse::<u32>().unwrap_or(0))
        .cloned()
        .unwrap_or_default()
}

fn resolve_finding_citations(b: &mut Builder) {
    let cites = std::mem::take(&mut b.raw_finding_cites);
    let mut unknown: BTreeMap<String, usize> = BTreeMap::new();
    for (file, line, num) in cites {
        let id = format!("C{num}");
        if b.defined_findings.contains(&id) {
            let from = crate::rust::path_node_id(&file);
            b.edge(EdgeKind::Cites, &from, &format!("finding:{id}"), &file, line);
        } else {
            // `C99`, `C11` and the like are the C language, not this project's ledger. Counted, not
            // drawn, and never reported as an error: the Survey will not invent a finding to
            // explain a token it failed to understand.
            *unknown.entry(id).or_default() += 1;
        }
    }
    let listed: Vec<String> = unknown.keys().cloned().collect();
    if !listed.is_empty() {
        b.finding(
            Severity::Note,
            "c-token-not-a-campaign-finding",
            "docs",
            0,
            format!(
                "these `C<n>` tokens are not campaign findings and were not linked: {}",
                listed.join(", ")
            ),
            "no action if these are C-language references; otherwise the ledger is missing a row",
        );
    }
}

fn every_crate_is_a_member(files: &[ScannedFile], b: &mut Builder) {
    let members: BTreeSet<String> = b
        .edges
        .iter()
        .filter(|e| e.from == "workspace")
        .map(|e| e.to.trim_start_matches("crate:").to_string())
        .collect();

    for f in files {
        if f.kind != FileKind::Manifest || f.rel == "Cargo.toml" {
            continue;
        }
        let Some(rest) = f.rel.strip_prefix("crates/") else { continue };
        let Some((name, _)) = rest.split_once('/') else { continue };
        if !members.contains(name) {
            b.finding(
                Severity::Error,
                "crate-not-in-workspace",
                &f.rel,
                1,
                format!("`crates/{name}` has a manifest but is not listed in the workspace `members`"),
                "add it to `members` in the root Cargo.toml, or delete the directory",
            );
        }
    }
}

/// A `.rs` file under `src/` that no `mod` declaration reaches is not compiled by anything.
fn modules_are_reachable(files: &[ScannedFile], b: &mut Builder) {
    let declared: BTreeSet<String> =
        b.edges.iter().filter(|e| e.kind == EdgeKind::DeclaresModule).map(|e| e.to.clone()).collect();

    for f in files {
        if f.kind != FileKind::RustSource {
            continue;
        }
        let stem = f.rel.rsplit('/').next().unwrap_or("").rsplit_once('.').map(|(s, _)| s).unwrap_or("");
        if stem == "lib" || stem == "main" {
            continue;
        }
        let id = format!("mod:{}", f.rel);
        if !declared.contains(&id) {
            b.finding(
                Severity::Warning,
                "unreachable-module",
                &f.rel,
                1,
                "no `mod` declaration reaches this file, so nothing compiles it".to_string(),
                "declare it, or delete it — a file that cannot be reached cannot be trusted to work",
            );
        }
    }
}

/// Byte-identical files at two paths: one of them will be edited and the other will not.
fn identical_files(files: &[ScannedFile], b: &mut Builder) {
    let mut by_hash: BTreeMap<u64, Vec<&str>> = BTreeMap::new();
    for f in files {
        // Short files collide for boring reasons (an empty file, a one-line stub).
        if f.text.len() < 400 {
            continue;
        }
        // A measurement record's run evidence is written once and never edited, and its copies are
        // the facts it records: the same task shown under every condition, a program checked twice
        // without a change, a final program that is its last snapshot. "One copy will drift" is a
        // premise about files people edit, and it does not hold here (V2 P4-08).
        if f.rel.starts_with("measurements/") && f.rel.contains("/runs/") {
            continue;
        }
        let mut h = DefaultHasher::new();
        f.text.hash(&mut h);
        by_hash.entry(h.finish()).or_default().push(&f.rel);
    }
    for (_, paths) in by_hash {
        if paths.len() > 1 {
            b.finding(
                Severity::Warning,
                "duplicated-file",
                paths[0],
                1,
                format!("byte-identical to {}", paths[1..].join(", ")),
                "make one the source and have the other point at it; two copies drift the first time one is edited",
            );
        }
    }
}

/// Node counts by kind, for the human render.
pub fn counts_by_kind(nodes: &[crate::Node]) -> BTreeMap<&'static str, usize> {
    let mut m = BTreeMap::new();
    for n in nodes {
        *m.entry(crate::scan::id_prefix(n.kind)).or_default() += 1;
    }
    m
}

/// Whether a node kind is a file on disk.
pub fn is_file_kind(k: NodeKind) -> bool {
    matches!(k, NodeKind::RustModule | NodeKind::TestSuite | NodeKind::Doc | NodeKind::Program | NodeKind::Other)
}
