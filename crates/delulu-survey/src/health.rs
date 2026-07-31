//! Repository health — the single place that knows what a healthy map looks like.
//!
//! # Why this module exists
//!
//! The map's structural invariants — every edge cites a line, nothing dangles, the totals agree
//! with the contents — were first written twice: once in `delulu doctor`, once in this crate's own
//! test suite. Both were correct and neither was the source. A fourth invariant added to either
//! would have been invisible to the other, and the divergence would have shown up as `delulu
//! doctor` reporting a healthy map that the suite refused, or the reverse.
//!
//! So they live here, once. [`integrity`] is consumed by both, which means **adding an invariant is
//! a one-line change that the CLI reports and the test enforces on the same commit**.
//!
//! # The boundary this module defends
//!
//! Everything about *the map* is here: building it, deciding whether it is behind the tree,
//! writing it, checking it, counting what it found. Nothing about the *environment* is — whether
//! Python is compiled in or the audit chain verifies belongs to the CLI, which owns those things.
//! `delulu doctor` is the orchestration and the presentation; this is the knowledge.
//!
//! The dependency direction is one-way and load-bearing: the CLI depends on this crate, and this
//! crate depends on **no sibling at all** — not the checker, not the runtime, not the diagnostics.
//! That is what lets the map be read while the compiler is mid-refactor and does not build.
//! `architecture.rs` holds that property to its word rather than trusting this paragraph.

use crate::{Severity, Survey};
use std::path::{Path, PathBuf};

/// What to do about a map that is behind the tree.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Repair {
    /// Rewrite the generated files that differ.
    Regenerate,
    /// Report only. Never writes — safe from a hook, a CI step, or several processes at once.
    ReportOnly,
}

/// One structural property of the map, and whether it holds.
#[derive(Clone, Debug)]
pub struct Integrity {
    /// Stable, human-readable name. Shown by `delulu doctor` and named in test failures.
    pub name: &'static str,
    pub ok: bool,
    /// What was measured, whether or not it held — so a passing check is still informative.
    pub detail: String,
}

/// How many discrepancies of each severity the map found.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub errors: usize,
    pub warnings: usize,
    pub notes: usize,
}

impl Tally {
    /// Only an error means the repository contradicts itself. A warning is drifting and a note is
    /// for a person to judge — neither should fail a build, or the list stops being read.
    pub fn is_healthy(self) -> bool {
        self.errors == 0
    }
}

/// Everything a caller needs to report on the repository, from one pass over the tree.
#[derive(Clone, Debug)]
pub struct RepoHealth {
    pub root: PathBuf,
    pub nodes: usize,
    pub edges: usize,
    /// Generated files that were behind the tree when this ran.
    pub stale: Vec<&'static str>,
    /// Files actually rewritten. Empty under [`Repair::ReportOnly`], and empty when nothing was
    /// stale — a healthy tree is never written to.
    pub regenerated: Vec<&'static str>,
    /// Set when regeneration was attempted and the write failed.
    pub write_error: Option<String>,
    pub integrity: Vec<Integrity>,
    pub tally: Tally,
}

impl RepoHealth {
    /// Every structural invariant holds, the map is current, and nothing contradicts itself.
    pub fn is_healthy(&self) -> bool {
        self.integrity.iter().all(|i| i.ok)
            && self.write_error.is_none()
            && self.tally.is_healthy()
            && (self.stale.is_empty() || !self.regenerated.is_empty())
    }

    /// The invariants that failed, for a caller that only wants the bad news.
    pub fn failures(&self) -> Vec<&Integrity> {
        self.integrity.iter().filter(|i| !i.ok).collect()
    }
}

/// Build the map, decide whether the committed copy is current, optionally rewrite it, and check it.
///
/// One entry point on purpose. A caller that had to remember to build, then compare, then write,
/// then verify would eventually forget a step — and the step most likely to be forgotten is the
/// last one.
pub fn inspect(root: &Path, repair: Repair) -> RepoHealth {
    let survey = Survey::build(root);
    let stale = stale_outputs(root, &survey);

    let (regenerated, write_error) = match (repair, stale.is_empty()) {
        (Repair::Regenerate, false) => match sync_outputs(root, &survey) {
            Ok(written) => (written, None),
            Err(e) => (Vec::new(), Some(e.to_string())),
        },
        _ => (Vec::new(), None),
    };

    RepoHealth {
        root: root.to_path_buf(),
        nodes: survey.nodes.len(),
        edges: survey.edges.len(),
        stale,
        regenerated,
        write_error,
        integrity: integrity(&survey),
        tally: tally(&survey),
    }
}

/// Every structural invariant the map must satisfy.
///
/// **This list is the definition.** `delulu doctor` reports it and
/// `delulu-survey`'s freshness suite asserts every entry holds, so the two cannot drift. To add an
/// invariant, add it here — nothing else needs to change for it to be both reported and enforced.
pub fn integrity(survey: &Survey) -> Vec<Integrity> {
    let mut out = Vec::new();

    // The provenance law itself. An edge nobody can check is a guess with better typography.
    let uncited = survey.edges.iter().filter(|e| e.file.is_empty() || e.line == 0).count();
    out.push(Integrity {
        name: "every edge cites a line",
        ok: uncited == 0,
        detail: if uncited == 0 {
            format!("{} edges, all with a file and line", survey.edges.len())
        } else {
            format!("{uncited} edge(s) carry no citation")
        },
    });

    // An edge to a node that does not exist is a relation the reader cannot follow.
    let dangling: Vec<&String> =
        survey.edges.iter().flat_map(|e| [&e.from, &e.to]).filter(|id| survey.node(id).is_none()).collect();
    out.push(Integrity {
        name: "no edge dangles",
        ok: dangling.is_empty(),
        detail: match dangling.first() {
            None => "every endpoint is a node".to_string(),
            Some(first) => format!("{} dangling endpoint(s), e.g. `{first}`", dangling.len()),
        },
    });

    // A map that miscounts itself has no standing to report that a document miscounts.
    let crates = survey.nodes.iter().filter(|n| n.kind == crate::NodeKind::Crate && n.id != "workspace").count();
    let agrees = survey.facts.crates as usize == crates && survey.facts.crates_shipped <= survey.facts.crates;
    out.push(Integrity {
        name: "totals agree with contents",
        ok: agrees,
        detail: if agrees {
            format!("{} workspace members, {} shipped", survey.facts.crates, survey.facts.crates_shipped)
        } else {
            format!("facts say {} crates; the map holds {crates}", survey.facts.crates)
        },
    });

    // Ids are the public surface: `rdeps crate:delulu-diag` has to be typeable from what is shown.
    let malformed = survey.nodes.iter().filter(|n| !n.id.contains(':') && n.id != "workspace").count();
    out.push(Integrity {
        name: "every node has a usable id",
        ok: malformed == 0,
        detail: if malformed == 0 {
            format!("{} nodes, all addressable as `kind:name`", survey.nodes.len())
        } else {
            format!("{malformed} node(s) carry an id no one could type")
        },
    });

    out
}

/// Count the discrepancies by severity.
pub fn tally(survey: &Survey) -> Tally {
    let mut t = Tally::default();
    for f in &survey.findings {
        match f.severity {
            Severity::Error => t.errors += 1,
            Severity::Warning => t.warnings += 1,
            Severity::Note => t.notes += 1,
        }
    }
    t
}

/// The DeluluLang source tree containing `from`, or `None` if there is not one.
///
/// Identified by what it **contains** — a workspace manifest beside the crate that owns the
/// diagnostic registry and the crate that builds the map — rather than by its name. A directory
/// called `DeluluLang` is not one, and a checkout renamed by whoever cloned it still is.
///
/// This lives here rather than in the CLI because it is knowledge about the repository, and the
/// repository is what this crate is about.
pub fn find_source_tree_from(from: &Path) -> Option<PathBuf> {
    let mut dir = from.to_path_buf();
    loop {
        if dir.join("Cargo.toml").is_file()
            && dir.join(crate::rust::CODE_REGISTRY).is_file()
            && dir.join("crates/delulu-survey/Cargo.toml").is_file()
        {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// [`find_source_tree_from`], starting at the working directory.
pub fn find_source_tree() -> Option<PathBuf> {
    find_source_tree_from(&std::env::current_dir().ok()?)
}

// --- the generated artifacts ---------------------------------------------------------------------

/// The three generated files and the content the tree currently implies.
pub fn outputs(survey: &Survey) -> Vec<(&'static str, String)> {
    vec![
        ("SURVEY.md", crate::render::markdown(survey)),
        ("DISCREPANCIES.md", crate::render::discrepancies(survey)),
        ("survey.json", crate::render::json(survey)),
    ]
}

/// Which generated files are behind the tree. Empty means the committed map is current.
pub fn stale_outputs(root: &Path, survey: &Survey) -> Vec<&'static str> {
    let dir = root.join(crate::OUTPUT_DIR);
    outputs(survey)
        .into_iter()
        .filter(|(name, fresh)| std::fs::read_to_string(dir.join(name)).ok().as_deref() != Some(fresh.as_str()))
        .map(|(name, _)| name)
        .collect()
}

/// Write only the files that differ, each through a temporary file and a rename.
///
/// Two properties, both learned the hard way in this repository. **Only what differs** is written,
/// so a healthy tree is untouched and running this is not a change. And each write is **atomic**,
/// so a concurrent reader — the freshness test, another `delulu doctor`, a parallel suite — sees
/// either the old file or the new one and never half of either. A short-lived process writing a
/// shared artifact is how campaign finding C69 corrupted an audit chain; the shape is the same
/// here and is designed out rather than hoped away.
pub fn sync_outputs(root: &Path, survey: &Survey) -> std::io::Result<Vec<&'static str>> {
    let dir = root.join(crate::OUTPUT_DIR);
    std::fs::create_dir_all(&dir)?;
    let mut written = Vec::new();
    for (name, fresh) in outputs(survey) {
        let path = dir.join(name);
        if std::fs::read_to_string(&path).ok().as_deref() == Some(fresh.as_str()) {
            continue;
        }
        // The temporary name carries the process id so two writers cannot collide on it.
        let tmp = dir.join(format!(".{name}.{}.tmp", std::process::id()));
        std::fs::write(&tmp, &fresh)?;
        // Windows will not rename onto an existing file; removing first is a narrow window, and
        // narrower than writing the destination in place.
        let _ = std::fs::remove_file(&path);
        if let Err(e) = std::fs::rename(&tmp, &path) {
            let _ = std::fs::remove_file(&tmp);
            return Err(e);
        }
        written.push(name);
    }
    Ok(written)
}
