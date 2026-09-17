//! Resolving a path someone wrote in prose or a comment to a file that is actually there.
//!
//! People do not write repository-relative paths. They write `tests/cli.rs` from inside
//! `crates/delulu/src/`, meaning `crates/delulu/tests/cli.rs`, and they are right to — the reader
//! knows where they are standing. A checker that only tries the repository root reports all of
//! those as broken and is simply wrong about the repository.
//!
//! So resolution tries the readings a human would, in order of decreasing confidence, and reports
//! only what none of them can find. When several files could be meant, that ambiguity is itself
//! reported rather than resolved by picking one.

use crate::scan::ScannedFile;
use std::collections::BTreeSet;

pub struct PathIndex {
    all: BTreeSet<String>,
    /// Every directory implied by a file path. A link to `docs/reference/` is a perfectly good
    /// link; without this the Survey called all of them broken.
    dirs: BTreeSet<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Resolution {
    Found(String),
    /// More than one file could be meant. The reader cannot tell either.
    Ambiguous(Vec<String>),
    /// The cited path is not in the tree, but the V1 archive holds the file it names.
    ///
    /// Phase V2-0 moved thirty-two documents into [`crate::ARCHIVE_ROOT`], which mirrors the
    /// original paths relative to `docs/`. The records that cite the old paths were left as
    /// written, because a historical record that is rewritten stops recording what it recorded.
    /// This resolution is how the map says both true things at once: the path is not there, and
    /// the file is.
    Archived {
        /// The root-relative path as the text names it, normalized.
        cited: String,
        /// Where that file is now.
        archived: String,
    },
    Missing,
}

impl PathIndex {
    pub fn new(files: &[ScannedFile]) -> PathIndex {
        let mut all: BTreeSet<String> = files.iter().map(|f| f.rel.clone()).collect();
        // The Survey's own output exists — this program writes it — but is deliberately not walked.
        // Without this, every document that links to the map reports a broken link to it.
        all.extend(crate::OUTPUT_FILES.iter().map(|s| s.to_string()));
        let mut dirs = BTreeSet::new();
        for p in &all {
            let mut cur = p.as_str();
            while let Some((parent, _)) = cur.rsplit_once('/') {
                if !dirs.insert(parent.to_string()) {
                    break; // this ancestry is already recorded
                }
                cur = parent;
            }
        }
        PathIndex { all, dirs }
    }

    /// Whether `rel` names a file or a directory in the tree.
    pub fn exists(&self, rel: &str) -> bool {
        let trimmed = rel.trim_end_matches('/');
        self.all.contains(trimmed) || self.dirs.contains(trimmed)
    }

    pub fn is_dir(&self, rel: &str) -> bool {
        let trimmed = rel.trim_end_matches('/');
        !self.all.contains(trimmed) && self.dirs.contains(trimmed)
    }

    /// `ctx_dir` is the directory of the file doing the citing; `crate_dir` is its crate root, if
    /// it has one.
    pub fn resolve(&self, hint: &str, ctx_dir: &str, crate_dir: Option<&str>) -> Resolution {
        // `path.md:210` is a line reference, which this repository writes constantly. The path is
        // the part before the colon.
        let hint = strip_line_suffix(hint.trim_start_matches("./"));
        if hint.is_empty() {
            return Resolution::Missing;
        }

        // 1. As written, from the repository root.
        if let Some(p) = normalize(hint) {
            if self.exists(&p) {
                return Resolution::Found(p);
            }
        }
        // 2. Relative to the citing file.
        if !ctx_dir.is_empty() {
            if let Some(p) = normalize(&format!("{ctx_dir}/{hint}")) {
                if self.exists(&p) {
                    return Resolution::Found(p);
                }
            }
        }
        // 3. Relative to the citing file's crate — how `tests/cli.rs` is meant from `src/`.
        if let Some(cd) = crate_dir {
            if let Some(p) = normalize(&format!("{cd}/{hint}")) {
                if self.exists(&p) {
                    return Resolution::Found(p);
                }
            }
        }
        // 4. A unique file whose path ends this way. Unique is the whole condition: if two files
        //    match, the citation genuinely does not say which, and saying so beats guessing.
        let tail = format!("/{hint}");
        let matches: Vec<String> = self.all.iter().filter(|p| p.ends_with(&tail)).cloned().collect();
        match matches.len() {
            // 5. The V1 archive mirrors `docs/`, so a citation of a path that moved there in V2-0
            //    still names a file. Last, deliberately: every reading of the *present* tree is
            //    tried first, so the archive can never shadow a live file of the same name.
            0 => self.archived(hint),
            1 => Resolution::Found(matches.into_iter().next().unwrap_or_default()),
            _ => Resolution::Ambiguous(matches),
        }
    }

    /// `docs/<x>` read as `docs/archive/v1/<x>`, when that file is really there.
    ///
    /// The existence check is the gate. A citation of a document that never moved and is simply
    /// gone gets no mirror and stays [`Resolution::Missing`] — otherwise this would turn every
    /// genuinely broken citation into a reassuring note, which is the failure mode a checker is
    /// there to prevent.
    fn archived(&self, hint: &str) -> Resolution {
        let Some(cited) = normalize(hint) else { return Resolution::Missing };
        // A path already inside the archive is never mirrored again: `docs/archive/v1/docs/...`
        // names nothing, and a second hop would be the checker inventing a location.
        if cited.starts_with(&format!("{}/", crate::ARCHIVE_ROOT)) || !cited.starts_with("docs/") {
            return Resolution::Missing;
        }
        let candidate = format!("{}/{}", crate::ARCHIVE_ROOT, &cited["docs/".len()..]);
        if self.exists(&candidate) {
            Resolution::Archived { cited, archived: candidate }
        } else {
            Resolution::Missing
        }
    }
}

/// Drop a trailing `:123` or `:123:4` line/column reference.
fn strip_line_suffix(p: &str) -> &str {
    let mut cut = p;
    for _ in 0..2 {
        if let Some((head, tail)) = cut.rsplit_once(':') {
            if !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) {
                cut = head;
                continue;
            }
        }
        break;
    }
    cut
}

/// Collapse `a/../b` and `./b`. Returns `None` if the path escapes the repository root.
pub fn normalize(p: &str) -> Option<String> {
    let mut out: Vec<&str> = Vec::new();
    for seg in p.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                out.pop()?;
            }
            s => out.push(s),
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out.join("/"))
    }
}


/// Blank out `` `code spans` `` so their contents cannot be mistaken for Markdown syntax.
///
/// This repository writes a great deal of Delulu and Rust inline, and generic syntax such as
/// `fn map[T, U, e](xs: List[T])` contains the exact `](` that opens a Markdown link target. Every
/// one of those was reported as a broken link until this existed.
pub fn strip_code_spans(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_span = false;
    for c in line.chars() {
        if c == '`' {
            in_span = !in_span;
            out.push(' ');
        } else if in_span {
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::{FileKind, ScannedFile};

    fn file(rel: &str) -> ScannedFile {
        ScannedFile {
            rel: rel.to_string(),
            abs: rel.into(),
            kind: FileKind::Other,
            lines: 0,
            crate_name: None,
            text: String::new(),
        }
    }

    fn index() -> PathIndex {
        PathIndex::new(&[
            file("crates/delulu/tests/cli.rs"),
            file("crates/delulu-broker/tests/holder_neutrality.rs"),
            file("docs/reference/grammar.md"),
            file("README.md"),
        ])
    }

    /// A directory is a perfectly good link target. Before this, every `docs/reference/` in the
    /// repository was reported as a broken link.
    #[test]
    fn a_directory_is_a_place_that_exists() {
        let idx = index();
        assert!(idx.exists("docs/reference"));
        assert!(idx.exists("docs/reference/"));
        assert!(idx.exists("crates/delulu/tests"));
        assert!(!idx.exists("docs/nowhere"));
    }

    /// `tests/cli.rs` written inside `crates/delulu/src/` means that crate's tests, and a reader
    /// knows it. A checker that only tries the repository root calls it broken and is wrong.
    #[test]
    fn a_path_is_read_the_way_the_person_who_wrote_it_meant_it() {
        let idx = index();
        assert_eq!(
            idx.resolve("tests/cli.rs", "crates/delulu/src", Some("crates/delulu")),
            Resolution::Found("crates/delulu/tests/cli.rs".to_string())
        );
    }

    /// With no crate to anchor it, a unique match anywhere in the tree is still unambiguous.
    #[test]
    fn a_unique_match_resolves_and_a_missing_one_does_not() {
        let idx = index();
        assert_eq!(
            idx.resolve("tests/holder_neutrality.rs", "docs/design", None),
            Resolution::Found("crates/delulu-broker/tests/holder_neutrality.rs".to_string())
        );
        assert_eq!(idx.resolve("tests/does_not_exist.rs", "docs", None), Resolution::Missing);
    }

    /// This repository cites `file:line` constantly; the path is the part before the colon.
    #[test]
    fn a_line_reference_still_names_its_file() {
        let idx = index();
        assert_eq!(idx.resolve("README.md:34", "", None), Resolution::Found("README.md".to_string()));
        assert_eq!(strip_line_suffix("a/b.rs:12:4"), "a/b.rs");
        assert_eq!(strip_line_suffix("a/b.rs"), "a/b.rs");
    }

    /// An index holding one archived document and the live tree around it.
    fn archive_index() -> PathIndex {
        PathIndex::new(&[
            file("README.md"),
            file("docs/design/CONSTITUTION.md"),
            file("docs/archive/v1/playbooks/STAGE8_PLAYBOOK.md"),
        ])
    }

    /// Phase V2-0 moved thirty-two documents into `docs/archive/v1/`, which mirrors `docs/`. The
    /// records that cite the pre-move paths were deliberately left as written, so the resolver has
    /// to be able to say *both* that the path is not there and that the file is.
    #[test]
    fn a_path_that_moved_into_the_archive_still_names_its_file() {
        let idx = archive_index();
        assert_eq!(
            idx.resolve("docs/playbooks/STAGE8_PLAYBOOK.md", "docs/design", None),
            Resolution::Archived {
                cited: "docs/playbooks/STAGE8_PLAYBOOK.md".to_string(),
                archived: "docs/archive/v1/playbooks/STAGE8_PLAYBOOK.md".to_string(),
            }
        );
    }

    /// The mirror is applied once. A second hop would look for `docs/archive/v1/archive/v1/…`,
    /// which names nothing — and an archived record citing an archived path that is genuinely gone
    /// deserves the same answer as anyone else: it is missing.
    #[test]
    fn a_path_already_in_the_archive_is_not_mirrored_again() {
        let idx = archive_index();
        assert_eq!(
            idx.resolve("docs/archive/v1/playbooks/GONE.md", "docs", None),
            Resolution::Missing,
            "the archive does not mirror itself"
        );
        // And a path outside `docs/` is never mirrored at all.
        assert_eq!(idx.resolve("crates/delulu/src/gone.rs", "docs", None), Resolution::Missing);
    }

    /// **The falsification.** The rule is *"the archive holds it"*, not *"the path starts with
    /// `docs/`"*, and the difference is the whole gate: without the existence check every genuinely
    /// broken citation would come back as a reassuring note. Take the archived file out of the
    /// index and the very same citation must go back to being missing.
    #[test]
    fn without_the_archived_file_the_same_citation_is_still_missing() {
        let without = PathIndex::new(&[file("README.md"), file("docs/design/CONSTITUTION.md")]);
        assert_eq!(
            without.resolve("docs/playbooks/STAGE8_PLAYBOOK.md", "docs/design", None),
            Resolution::Missing,
            "nothing is at that path and nothing is in the archive — the map must say so"
        );
        // Sanity: the only difference between the two indexes is the archived file itself.
        assert!(matches!(
            archive_index().resolve("docs/playbooks/STAGE8_PLAYBOOK.md", "docs/design", None),
            Resolution::Archived { .. }
        ));
    }

    /// A live file always wins. The archive is tried *after* every reading of the present tree, so
    /// a document that exists at `docs/<x>` is never shadowed by an older copy of the same name.
    #[test]
    fn the_present_tree_is_read_before_the_archive() {
        let idx = PathIndex::new(&[
            file("docs/playbooks/STAGE8_PLAYBOOK.md"),
            file("docs/archive/v1/playbooks/STAGE8_PLAYBOOK.md"),
        ]);
        assert_eq!(
            idx.resolve("docs/playbooks/STAGE8_PLAYBOOK.md", "", None),
            Resolution::Found("docs/playbooks/STAGE8_PLAYBOOK.md".to_string())
        );
    }

    /// The bug that produced thirty-five false "broken link" reports: Delulu and Rust generic
    /// syntax contains the `](` that opens a Markdown link target.
    #[test]
    fn generic_syntax_inside_a_code_span_is_not_markdown() {
        let line = "functions take row variables: `fn map[T, U, e](xs: List[T]) -> List[U] ! e`.";
        let stripped = strip_code_spans(line);
        assert!(!stripped.contains("]("), "a code span must not leave link syntax behind: {stripped}");
        assert!(stripped.starts_with("functions take row variables:"), "prose outside the span survives");
    }
}
