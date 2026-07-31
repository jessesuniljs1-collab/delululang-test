//! The Survey — a measured map of the DeluluLang **repository**.
//!
//! # What this is, and what it is not
//!
//! `delulu-atlas` maps a checked Delulu *program*, derived only from compiler facts. The Survey
//! maps the *repository that implements the compiler*, derived only from the repository's own
//! text. Atlas is downhill of the type checker; the Survey is downhill of nothing — it reads
//! files. That is deliberate, and it is why the Survey still works on a tree that does not build.
//!
//! # The provenance law
//!
//! > **Every edge names the file and the line it was read from, and every edge whose target is a
//! > path is checked to exist. No edge is inferred from name similarity, embeddings, or
//! > proximity. A relation that cannot be pointed at in the text is not in the graph.**
//!
//! This is the whole difference between a map and a guess. A guessed graph gets more impressive
//! as it gets less true; this one cannot say anything it cannot cite. Where an extraction is
//! genuinely uncertain it becomes a [`Finding`], not a confident edge.
//!
//! # What a lexical reader cannot see
//!
//! The Survey reads text, not syntax trees ([`rust`] documents the specific blind spots). Rather
//! than pretend otherwise, every extracted relation is **cross-checked against an independent
//! source**, and disagreement is reported instead of silently resolved:
//!
//! - a `use delulu_x::` with no matching dependency in that crate's manifest is a finding;
//! - a manifest dependency no source mentions is a finding;
//! - a `mod x;` with no `x.rs`/`x/mod.rs` on disk is a finding;
//! - a cited `DLxxxx` absent from the registry is a finding.
//!
//! So the two extractors check each other. Neither is trusted alone.

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub mod health;
pub mod manifest;
pub mod mdown;
pub mod paths;
pub mod render;
pub mod rust;
pub mod scan;
pub mod verify;

/// The machine channel's schema id. Versioned and additive, in the manner of `atlas/1` and the
/// CLI's JSON envelope: new fields may appear, existing ones do not change meaning.
pub const SCHEMA: &str = "survey/1";

/// Directories the Survey never descends into, with the reason each is excluded.
///
/// `.claude/worktrees` matters more than it looks: a detached agent worktree holds a *complete
/// second copy of the repository* at a different commit. Walking it would double every node and
/// silently mix two revisions of the same file into one map.
pub const EXCLUDED_DIRS: &[(&str, &str)] = &[
    (".git", "version-control internals"),
    (".claude", "agent scratch space, including detached worktrees holding a second copy of the repo"),
    ("target", "build output"),
    ("node_modules", "vendored JavaScript dependencies"),
];

/// The Survey's own output, excluded because it must not map itself.
///
/// This is a correctness requirement, not tidiness. `DISCREPANCIES.md` quotes the paths it reports
/// on, so a Survey that read its own last output would find new relations in it, write a different
/// output, and find different relations again on the next run. It would never reach a fixed point,
/// and the staleness gate — which asks "does regenerating change anything?" — would fail forever
/// while being perfectly correct to do so. Found by running it twice.
pub const OUTPUT_DIR: &str = "docs/survey";

/// The files the Survey writes. Not walked, for the reason above — but **known to exist**, because
/// this program is what writes them.
///
/// Seeding them into the path index rather than stat-ing the directory keeps the map a pure
/// function of the tree: on a fresh clone that has never run `build`, a link to `SURVEY.md` still
/// resolves, so the map does not depend on whether it has been generated yet. Note that
/// `docs/survey/README.md` is *not* here — it is written by hand, so it is walked and its links are
/// checked like any other document's.
pub const OUTPUT_FILES: &[&str] =
    &["docs/survey/SURVEY.md", "docs/survey/DISCREPANCIES.md", "docs/survey/survey.json"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// A Rust workspace member.
    Crate,
    /// A third-party crate this workspace depends on.
    ExternalCrate,
    /// A Rust source file inside a crate's `src/`.
    RustModule,
    /// A Rust integration-test file inside a crate's `tests/`.
    TestSuite,
    /// A Markdown document.
    Doc,
    /// A DeluluLang program (`.delulu`).
    Program,
    /// A registered diagnostic code (`DLxxxx`).
    DiagnosticCode,
    /// A recorded ruling. Ids are namespaced by the stage that allocated them.
    Ruling,
    /// A hardening-campaign finding (`Cxx`).
    Finding,
    /// A `delulu` CLI subcommand.
    CliVerb,
    /// A published measurement study or demo.
    Measurement,
    /// An RFC.
    Rfc,
    /// A directory named as a link target. Directories get nodes because links to them are real
    /// links, and an edge whose endpoint is not a node is a dangling edge.
    Directory,
    /// Anything tracked that is none of the above (JSON fixtures, shell scripts, CI config).
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// crate → crate. Read from a `path = "../x"` dependency in a Cargo manifest.
    DependsOn,
    /// crate → external crate. Read from a Cargo manifest dependency line.
    DependsOnExternal,
    /// crate → rust module. Read from a `mod x;` item.
    DeclaresModule,
    /// rust module → crate. Read from a `delulu_x::` path in the source text.
    Uses,
    /// rust module → diagnostic code. The registry entry that allocates the code.
    DefinesCode,
    /// rust module → diagnostic code. The source raises or handles it.
    RaisesCode,
    /// program → diagnostic code. A conformance expectation names it.
    ExpectsCode,
    /// doc → diagnostic code. Prose that documents it.
    DocumentsCode,
    /// doc → ruling / finding. The document that *allocates* the id.
    Defines,
    /// any → ruling / finding. A citation of an id defined elsewhere.
    Cites,
    /// doc → file. A Markdown link or an inline path reference, verified to exist.
    LinksTo,
    /// rust module → file. A path named in a comment, verified to exist.
    References,
    /// test suite → crate. The crate whose `tests/` directory holds it.
    TestsCrate,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    pub name: String,
    /// Repository-relative, forward-slashed. Absent for ideas that are not files (a code, a ruling).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<u32>,
    /// One line, lifted verbatim from the file — a `//!` header, a manifest `description`, a
    /// registry title. Never generated, never summarized: a description this map invented would be
    /// the one thing on the page that nothing backs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// What is inside this file, read from its own text: public items for Rust, section headings
    /// for Markdown. This is the index that answers "where is X defined?" without a grep.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub contents: Vec<String>,
}

/// An edge, and the exact place in the text that justifies it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Edge {
    pub kind: EdgeKind,
    pub from: String,
    pub to: String,
    /// The file this relation was read from, repository-relative.
    pub file: String,
    /// The 1-based line within `file`. Every edge has one; that is the law.
    pub line: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// The repository contradicts itself, or points somewhere that does not exist.
    Error,
    /// True today but load-bearing and drifting, or a duplication that will diverge.
    Warning,
    /// Worth a human's attention; not necessarily wrong.
    Note,
}

/// A discrepancy the Survey found while cross-checking its own extractions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Finding {
    pub severity: Severity,
    /// Stable kebab-case class, e.g. `broken-link`, `undeclared-dependency`, `stale-count`.
    pub class: String,
    pub file: String,
    pub line: u32,
    pub message: String,
    /// What the reader should do about it. Empty when the Survey genuinely cannot say.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub remedy: String,
}

/// Counts that other documents like to quote and then fail to update.
#[derive(Debug, Clone, Default, Serialize)]
pub struct RepoFacts {
    /// Every workspace member.
    pub crates: u32,
    /// Members that are shipped language surface — total minus `publish = false` tooling. This is
    /// the number a document means when it says "N crates", and keeping the two apart is why
    /// adding the Survey itself does not make every such sentence in the repository wrong.
    pub crates_shipped: u32,
    pub rust_files: u32,
    pub rust_lines: u32,
    pub doc_files: u32,
    pub doc_lines: u32,
    pub delulu_programs: u32,
    pub delulu_lines: u32,
    pub test_suites: u32,
    pub registered_codes: u32,
    pub rulings: u32,
    pub findings_recorded: u32,
    pub files_walked: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Survey {
    pub schema: &'static str,
    pub facts: RepoFacts,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub findings: Vec<Finding>,
}

/// The health API, re-exported so a caller writes `delulu_survey::inspect` rather than reaching
/// into a module path. [`health`] documents the boundary these draw.
pub use health::{
    find_source_tree, find_source_tree_from, inspect, integrity, outputs, stale_outputs, sync_outputs, tally,
    Integrity, Repair, RepoHealth, Tally,
};

impl Survey {
    /// Read `root` and derive the map. Pure with respect to the tree: nothing is written here.
    pub fn build(root: &Path) -> Survey {
        let files = scan::walk(root);
        // Every path question is answered from this index rather than from the filesystem, so the
        // map is a pure function of the tree that was walked — the same input cannot produce two
        // different maps because something changed on disk halfway through.
        let index = paths::PathIndex::new(&files);
        let mut b = Builder::default();

        for f in &files {
            b.file_node(f);
        }
        // The Survey's own outputs get nodes because documents link to them, but they are never
        // read — an edge whose endpoint is absent is a dangling edge, and a map with dangling
        // edges is a map that has stopped checking itself.
        for p in OUTPUT_FILES {
            let kind = if p.ends_with(".md") { NodeKind::Doc } else { NodeKind::Other };
            let name = p.rsplit('/').next().unwrap_or(p);
            let n = b.node(&rust::path_node_id(p), kind, name);
            n.path = Some((*p).to_string());
            n.summary = Some("generated by `delulu-survey build`; never read back".to_string());
        }
        for f in &files {
            match f.kind {
                scan::FileKind::Manifest => manifest::extract(f, &mut b),
                scan::FileKind::RustSource | scan::FileKind::RustTest => rust::extract(&index, f, &mut b),
                scan::FileKind::Doc => mdown::extract(&index, f, &mut b),
                _ => {}
            }
        }

        // Cross-check before finishing: the checks need what each extractor *claimed*, which is
        // still on the builder. This is the pass that turns a lexical guess into either a
        // corroborated edge or a reported disagreement.
        verify::cross_check(&index, &files, &mut b);

        let mut survey = b.finish(&files);
        survey.findings.sort();
        survey
    }

    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.binary_search_by(|n| n.id.as_str().cmp(id)).ok().map(|i| &self.nodes[i])
    }

    /// Every edge leaving `id`.
    pub fn out(&self, id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.from == id).collect()
    }

    /// Every edge arriving at `id` — the question "what breaks if I change this?".
    pub fn into_(&self, id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.to == id).collect()
    }
}

/// Accumulator shared by the extractors. Deduplicates on the way in: the same relation read from
/// the same line twice is one edge, but the same relation read from two different lines is two,
/// because both citations are real and a reader may want either.
#[derive(Default)]
pub struct Builder {
    pub nodes: BTreeMap<String, Node>,
    pub edges: BTreeSet<Edge>,
    pub findings: Vec<Finding>,
    /// Ids the Survey has seen *cited*, for the "is this defined anywhere?" pass.
    pub cited_codes: BTreeSet<String>,
    pub defined_codes: BTreeSet<String>,
    pub defined_rulings: BTreeSet<String>,
    pub defined_findings: BTreeSet<String>,
    /// Ruling citations exactly as written, resolved later.
    ///
    /// They cannot be resolved on sight: this repository allocates ruling numbers in **one
    /// namespace per stage**, so a bare `D21` names a different decision depending on which build
    /// order you are standing in. Guessing here would fuse two unrelated rulings into one node,
    /// which is precisely the failure this map exists to avoid — so the resolution, and the
    /// ambiguity when there is one, is deferred to [`crate::verify`].
    pub raw_ruling_cites: Vec<(String, u32, String)>,
    /// Finding citations as written. Resolved late for a duller reason than rulings: `C99` and
    /// `C11` are the C language, and only the ledger can say which `C<n>` is one of ours.
    pub raw_finding_cites: Vec<(String, u32, String)>,
    /// Crates marked `publish = false` — repository tooling, not shipped language surface.
    pub tooling_crates: BTreeSet<String>,
}

impl Builder {
    pub fn node(&mut self, id: &str, kind: NodeKind, name: &str) -> &mut Node {
        self.nodes.entry(id.to_string()).or_insert_with(|| Node {
            id: id.to_string(),
            kind,
            name: name.to_string(),
            path: None,
            lines: None,
            summary: None,
            contents: Vec::new(),
        })
    }

    pub fn edge(&mut self, kind: EdgeKind, from: &str, to: &str, file: &str, line: u32) {
        self.edges.insert(Edge {
            kind,
            from: from.to_string(),
            to: to.to_string(),
            file: file.to_string(),
            line,
        });
    }

    pub fn finding(&mut self, severity: Severity, class: &str, file: &str, line: u32, message: String, remedy: &str) {
        self.findings.push(Finding {
            severity,
            class: class.to_string(),
            file: file.to_string(),
            line,
            message,
            remedy: remedy.to_string(),
        });
    }

    fn file_node(&mut self, f: &scan::ScannedFile) {
        let Some((id, kind)) = f.node_identity() else { return };
        let name = f.rel.rsplit('/').next().unwrap_or(&f.rel).to_string();
        let n = self.node(&id, kind, &name);
        n.path = Some(f.rel.clone());
        n.lines = Some(f.lines);
    }

    fn finish(self, files: &[scan::ScannedFile]) -> Survey {
        let mut facts = RepoFacts { files_walked: files.len() as u32, ..RepoFacts::default() };
        for f in files {
            match f.kind {
                scan::FileKind::RustSource => {
                    facts.rust_files += 1;
                    facts.rust_lines += f.lines;
                }
                scan::FileKind::RustTest => {
                    facts.rust_files += 1;
                    facts.rust_lines += f.lines;
                    facts.test_suites += 1;
                }
                scan::FileKind::Doc => {
                    facts.doc_files += 1;
                    facts.doc_lines += f.lines;
                }
                scan::FileKind::Program => {
                    facts.delulu_programs += 1;
                    facts.delulu_lines += f.lines;
                }
                _ => {}
            }
        }
        let nodes: Vec<Node> = self.nodes.into_values().collect();
        facts.crates = nodes.iter().filter(|n| n.kind == NodeKind::Crate).count() as u32;
        facts.crates_shipped = facts.crates - self.tooling_crates.len() as u32;
        facts.registered_codes = self.defined_codes.len() as u32;
        facts.rulings = self.defined_rulings.len() as u32;
        facts.findings_recorded = self.defined_findings.len() as u32;

        Survey {
            schema: SCHEMA,
            facts,
            nodes,
            edges: self.edges.into_iter().collect(),
            findings: self.findings,
        }
    }
}
