//! Reading Rust source as text.
//!
//! # What this cannot see, stated plainly
//!
//! This is a lexical reader, not `syn`. It will miss:
//!
//! - items produced by a macro (a `macro_rules!` that emits `pub fn` yields no export here);
//! - `#[cfg(...)]`-gated modules, which are reported as unconditional — the Survey shows the union
//!   of every configuration, never one build's view;
//! - a `use` written through an alias or a re-export chain, which lands on the crate that owns the
//!   path as written, not the crate that ultimately defines the item.
//!
//! None of these produce a *wrong* edge; they produce a *missing* one, or a coarser one. That
//! asymmetry is deliberate. Every edge this file emits is cross-checked in [`crate::verify`]
//! against the crate's manifest and against the filesystem, so an edge that survives has been
//! corroborated by a second, independent source.

use crate::paths::{PathIndex, Resolution};
use crate::scan::{split_code_and_comments, ScannedFile};
use crate::{Builder, EdgeKind, NodeKind, Severity};

/// The one file that *allocates* diagnostic codes. Everything else merely cites them.
///
/// Hardcoded, and therefore checked: [`crate::verify`] reports it if this path stops existing, so
/// the assumption fails loudly rather than silently emptying the code registry.
pub const CODE_REGISTRY: &str = "crates/delulu-diag/src/codes.rs";

/// Path prefixes worth resolving when they appear in a comment. A bare word is never treated as a
/// path — only something already shaped like a repository location.
const PATH_ROOTS: &[&str] =
    &["crates/", "docs/", "tests/", "examples/", "measurements/", "morphs/", "rfcs/", "editors/", "release-artifacts/"];

pub fn extract(idx: &PathIndex, f: &ScannedFile, b: &mut Builder) {
    let Some((self_id, _)) = f.node_identity() else { return };
    let lines = split_code_and_comments(&f.text);
    let dir = f.rel.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let crate_dir = f.crate_name.as_ref().map(|c| format!("crates/{c}"));

    // The crate this file belongs to, and the file's role within it.
    if let Some(cname) = &f.crate_name {
        let crate_id = format!("crate:{cname}");
        b.node(&crate_id, NodeKind::Crate, cname);
        let stem = file_stem(&f.rel);
        if stem == "lib" || stem == "main" {
            b.edge(EdgeKind::DeclaresModule, &crate_id, &self_id, &f.rel, 1);
        }
        if f.kind == crate::scan::FileKind::RustTest {
            b.edge(EdgeKind::TestsCrate, &self_id, &crate_id, &f.rel, 1);
        }
    }

    let mut in_registry = false;
    let mut in_unallocated = false;
    let mut doc_header: Option<String> = None;
    let mut contents: Vec<String> = Vec::new();

    for (i, (code, comment)) in lines.iter().enumerate() {
        let lineno = i as u32 + 1;
        let t = code.trim();

        // `//!` on the first lines is the module's own one-line description.
        if doc_header.is_none() {
            if let Some(rest) = comment.trim().strip_prefix("//!") {
                let rest = rest.trim();
                if !rest.is_empty() {
                    doc_header = Some(rest.to_string());
                }
            }
        }

        // --- module declarations -------------------------------------------------------------
        if let Some(name) = module_declaration(t) {
            match resolve_module(idx, &f.rel, &name) {
                Some(target) => {
                    b.edge(EdgeKind::DeclaresModule, &self_id, &format!("mod:{target}"), &f.rel, lineno);
                }
                None => b.finding(
                    Severity::Error,
                    "module-without-file",
                    &f.rel,
                    lineno,
                    format!("`mod {name};` declares a module with no `{name}.rs` or `{name}/mod.rs` beside it"),
                    "add the file, or remove the declaration",
                ),
            }
        }

        // --- sibling-crate usage -------------------------------------------------------------
        for (dep, col) in crate_paths(t) {
            b.node(&format!("crate:{dep}"), NodeKind::Crate, &dep);
            b.edge(EdgeKind::Uses, &self_id, &format!("crate:{dep}"), &f.rel, lineno);
            let _ = col;
        }

        // --- diagnostic codes ----------------------------------------------------------------
        if f.rel == CODE_REGISTRY {
            if t.starts_with("registry!") {
                in_registry = true;
            } else if in_registry && t == "}" {
                in_registry = false;
            }
            // The table beside the registry that records WHY a code is absent from it. Read
            // lexically, like everything else here — the Survey depends on no crate in this
            // workspace, so it cannot simply import `delulu_diag::UNALLOCATED`.
            if t.starts_with("pub const UNALLOCATED") {
                in_unallocated = true;
            } else if in_unallocated && t == "];" {
                in_unallocated = false;
            }
        }
        for c in diagnostic_codes(&format!("{code} {comment}")) {
            let code_id = format!("code:{c}");
            if in_unallocated && t.starts_with(&format!("code: \"{c}\"")) {
                // A code with a recorded disposition. Still a mention, so it still gets a node and
                // an edge; what changes is that it is no longer an UNEXPLAINED absence.
                b.dispositioned_codes.insert(c.clone());
            }
            // Only the registry block *defines*; every other mention cites.
            if in_registry && t.starts_with(&format!("\"{c}\"")) {
                let title = t.split_once("=> ").map(|(_, s)| s.trim().trim_end_matches(',').trim_matches('"'));
                b.defined_codes.insert(c.clone());
                let n = b.node(&code_id, NodeKind::DiagnosticCode, &c);
                if let Some(title) = title {
                    n.summary = Some(title.to_string());
                }
                b.edge(EdgeKind::DefinesCode, &self_id, &code_id, &f.rel, lineno);
            } else {
                b.node(&code_id, NodeKind::DiagnosticCode, &c);
                b.cited_codes.insert(c.clone());
                b.edge(EdgeKind::RaisesCode, &self_id, &code_id, &f.rel, lineno);
            }
        }

        // --- public items --------------------------------------------------------------------
        if let Some(item) = public_item(t) {
            contents.push(item);
        }

        // --- rulings and findings cited in comments --------------------------------------------
        // Code in this repository routinely names the decision that authorized it. That is a real
        // edge from an implementation to its justification, and it is the one an author most wants
        // when asking "why is this like this?".
        for num in crate::mdown::id_citations(comment, 'D') {
            b.raw_ruling_cites.push((f.rel.clone(), lineno, num));
        }
        for num in crate::mdown::id_citations(comment, 'C') {
            b.raw_finding_cites.push((f.rel.clone(), lineno, num));
        }

        // --- paths named in comments ----------------------------------------------------------
        for p in cited_paths(comment) {
            match idx.resolve(&p, dir, crate_dir.as_deref()) {
                Resolution::Found(target) => {
                    let to = crate::mdown::target_node(idx, b, &target);
                    b.edge(EdgeKind::References, &self_id, &to, &f.rel, lineno);
                }
                Resolution::Ambiguous(_) => {}
                // Phase V2-0 moved thirty-two documents into the V1 archive, which mirrors `docs/`.
                // A comment that names the pre-move path still names a real file, so the reference
                // is drawn and the hop is noted — not reported as a missing path it is not.
                Resolution::Archived { cited, archived } => {
                    let to = crate::mdown::target_node(idx, b, &archived);
                    b.edge(EdgeKind::References, &self_id, &to, &f.rel, lineno);
                    b.finding(
                        Severity::Note,
                        "comment-cites-archived-path",
                        &f.rel,
                        lineno,
                        format!(
                            "prose names `{cited}`, which moved to `{archived}` in V2-0; \
                             the citation is historical and left as written"
                        ),
                        "a current document should cite the archived path",
                    );
                }
                Resolution::Missing => b.finding(
                    Severity::Warning,
                    "comment-cites-missing-path",
                    &f.rel,
                    lineno,
                    format!("a comment points at `{p}`, which is nowhere in the tree"),
                    "correct the path, or drop the reference",
                ),
            }
        }
    }

    contents.sort();
    contents.dedup();
    let n = b.node(&self_id, NodeKind::RustModule, &f.rel);
    n.summary = doc_header;
    n.contents = contents;
}

fn file_stem(rel: &str) -> &str {
    rel.rsplit('/').next().unwrap_or(rel).rsplit_once('.').map(|(s, _)| s).unwrap_or("")
}

/// `mod x;` / `pub mod x;` / `pub(crate) mod x;` — but never `mod x {`, which is inline and has no
/// file of its own.
fn module_declaration(t: &str) -> Option<String> {
    // A declaration ends in `;`. `mod x {` is inline: it has no file, so it is no file-to-file
    // relation.
    let rest = t.strip_suffix(';')?;
    let i = rest.find("mod ")?;
    let (vis, name) = (rest[..i].trim(), rest[i + 4..].trim());

    // Only a visibility modifier may stand before `mod`. Without this check, any line ending in a
    // semicolon that happened to contain the word would be read as a module — and with a check
    // that only understood bare `pub`, every `pub(crate) mod` was missed instead, which showed up
    // downstream as perfectly reachable files reported as unreachable.
    if !(vis.is_empty() || vis == "pub" || (vis.starts_with("pub(") && vis.ends_with(')'))) {
        return None;
    }
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    Some(name.to_string())
}

/// Where Rust would look for `mod name;` declared in `from`.
fn resolve_module(idx: &PathIndex, from: &str, name: &str) -> Option<String> {
    let dir = from.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let stem = file_stem(from);
    // A module file that is not a crate/dir root owns a subdirectory named after itself.
    let base =
        if stem == "lib" || stem == "main" || stem == "mod" { dir.to_string() } else { format!("{dir}/{stem}") };

    [format!("{base}/{name}.rs"), format!("{base}/{name}/mod.rs")].into_iter().find(|c| idx.exists(c))
}

/// Sibling crates named by a `delulu_x::` path in code. Returns the *crate* name (hyphenated).
fn crate_paths(t: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    let bytes: Vec<char> = t.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(&['d', 'e', 'l', 'u', 'l', 'u', '_']) {
            // Not a match if it is part of a longer identifier (`my_delulu_check`).
            let prev_is_ident = i > 0 && (bytes[i - 1].is_alphanumeric() || bytes[i - 1] == '_');
            if !prev_is_ident {
                let mut j = i;
                while j < bytes.len() && (bytes[j].is_alphanumeric() || bytes[j] == '_') {
                    j += 1;
                }
                let ident: String = bytes[i..j].iter().collect();
                if bytes.get(j) == Some(&':') && bytes.get(j + 1) == Some(&':') {
                    out.push((ident.replace('_', "-"), i));
                }
                i = j;
                continue;
            }
        }
        i += 1;
    }
    out.sort();
    out.dedup();
    out
}

/// `DL` followed by exactly four digits, with the neighbouring characters checked so a five-digit
/// run is not read as a four-digit code with a stray digit after it.
///
/// Written without a literal example on purpose: this doc comment is itself scanned, and an
/// example code here would be indexed as a real citation from a file that raises nothing.
pub fn diagnostic_codes(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 6 <= chars.len() {
        if chars[i] == 'D' && chars[i + 1] == 'L' && chars[i + 2..i + 6].iter().all(|c| c.is_ascii_digit()) {
            let before_ok = i == 0 || !(chars[i - 1].is_alphanumeric() || chars[i - 1] == '_');
            let after_ok = chars.get(i + 6).is_none_or(|c| !c.is_ascii_digit());
            // A RANGE, not a citation. Prose that allocates a block writes it as `DLxxxx-DLyyyy`,
            // and neither endpoint claims those two codes exist: a sentence reserving a block for
            // a later stage was being read as citing both ends, which then showed up as codes the
            // registry had failed to allocate. Both ends are skipped. A trailing word like
            // `DLxxxx-style` is untouched, because the dash there is not followed by another code.
            //
            // No literal code appears in this comment, for the reason `diagnostic_codes` gives
            // below: this file is scanned like any other, so an example here would be indexed as a
            // real citation from a module that raises nothing.
            if before_ok && after_ok && !in_range(&chars, i) {
                out.push(chars[i..i + 6].iter().collect::<String>());
                i += 6;
                continue;
            }
        }
        i += 1;
    }
    out.sort();
    out.dedup();
    out
}

/// Whether the code beginning at `i` is one end of a `DLxxxx–DLyyyy` range.
///
/// Hyphen, en dash and em dash all appear in this repository's prose, so all three count.
fn in_range(chars: &[char], i: usize) -> bool {
    let dash = |c: char| c == '-' || c == '\u{2013}' || c == '\u{2014}';
    let code_at = |j: usize| {
        chars.get(j) == Some(&'D')
            && chars.get(j + 1) == Some(&'L')
            && chars.get(j + 2..j + 6).is_some_and(|w| w.iter().all(char::is_ascii_digit))
    };
    let opens = chars.get(i + 6).copied().is_some_and(dash) && code_at(i + 7);
    let closes = i >= 7 && dash(chars[i - 1]) && code_at(i - 7);
    opens || closes
}

/// A public item's name, for the file index.
fn public_item(t: &str) -> Option<String> {
    let rest = t.strip_prefix("pub ")?;
    // `pub(crate)` is not public API; skip it rather than overstate the surface.
    let rest = rest.trim_start();
    for kw in ["fn ", "struct ", "enum ", "trait ", "const ", "type ", "static ", "macro_rules! "] {
        if let Some(after) = rest.strip_prefix(kw) {
            let name: String =
                after.trim_start().chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if name.is_empty() {
                return None;
            }
            let kind = kw.trim().trim_end_matches('!');
            return Some(format!("{kind} {name}"));
        }
    }
    // `pub async fn` / `pub unsafe fn` / `pub extern "C" fn`
    if let Some(i) = rest.find("fn ") {
        if rest[..i].split_whitespace().all(|w| matches!(w, "async" | "unsafe" | "extern" | "\"C\"")) {
            let name: String =
                rest[i + 3..].trim_start().chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() {
                return Some(format!("fn {name}"));
            }
        }
    }
    None
}

/// Repository paths mentioned in a comment.
pub fn cited_paths(comment: &str) -> Vec<String> {
    let mut out = Vec::new();
    for root in PATH_ROOTS {
        let mut from = 0;
        while let Some(i) = comment[from..].find(root) {
            let start = from + i;
            let before_ok = start == 0
                || !comment[..start].ends_with(|c: char| c.is_alphanumeric() || c == '/' || c == '.' || c == '-');
            let raw: String = comment[start..]
                .chars()
                .take_while(|c| !c.is_whitespace() && !matches!(c, '`' | ',' | ')' | '(' | '"' | ';' | '\'' | '<' | '>'))
                .collect();
            let cleaned = raw.trim_end_matches(['.', ':']).to_string();
            // Only real file references — a bare directory prefix or a glob is not a citation.
            if before_ok && cleaned.len() > root.len() && cleaned.contains('.') && !cleaned.contains('*') {
                out.push(cleaned);
            }
            from = start + root.len();
        }
    }
    out.sort();
    out.dedup();
    out
}

/// The node id a repository path maps to, matching [`crate::scan::ScannedFile::node_identity`].
pub fn path_node_id(p: &str) -> String {
    let prefix = if p.ends_with(".md") {
        "doc"
    } else if p.ends_with(".delulu") {
        "prog"
    } else if p.ends_with(".rs") {
        if p.contains("/src/") {
            "mod"
        } else {
            "test"
        }
    } else {
        "file"
    };
    format!("{prefix}:{p}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `DL` plus four digits — and a longer digit run is a different thing, not a code with a
    /// stray digit stuck to it.
    #[test]
    fn a_longer_digit_run_is_not_a_code() {
        assert_eq!(diagnostic_codes("raises DL0501 here"), vec!["DL0501"]);
        assert!(diagnostic_codes("DL12345").is_empty(), "five digits is not a four-digit code");
        assert!(diagnostic_codes("xDL0501").is_empty(), "glued to a word, it is not a citation");
        assert_eq!(diagnostic_codes("DL0501/DL0306"), vec!["DL0306", "DL0501"]);
    }

    /// An inline `mod x { … }` has no file of its own, so it is not a file-to-file relation.
    #[test]
    fn only_a_module_with_a_file_is_a_declaration() {
        assert_eq!(module_declaration("mod build;").as_deref(), Some("build"));
        assert_eq!(module_declaration("pub mod render;").as_deref(), Some("render"));
        assert_eq!(module_declaration("pub(crate) mod query;").as_deref(), Some("query"));
        assert_eq!(module_declaration("mod tests {"), None, "inline modules have no file");
        assert_eq!(module_declaration("#[cfg(test)]"), None);
    }

    /// The file index answers "where is X defined?" — so it records what is public, and only that.
    #[test]
    fn the_index_records_the_public_surface_and_not_more() {
        assert_eq!(public_item("pub fn check_source(id: u32) -> Checked {").as_deref(), Some("fn check_source"));
        assert_eq!(public_item("pub struct Authority {").as_deref(), Some("struct Authority"));
        assert_eq!(public_item("pub async fn serve() {").as_deref(), Some("fn serve"));
        assert_eq!(public_item("fn private_helper() {"), None);
        assert_eq!(public_item("pub(crate) fn internal() {"), None, "pub(crate) is not public surface");
    }

    /// A path is only a citation when it is shaped like one — a bare directory prefix is not.
    #[test]
    fn a_comment_path_is_recognised_only_when_it_names_a_file() {
        assert_eq!(cited_paths("// see crates/delulu-check/src/ty.rs for the rule"), vec!["crates/delulu-check/src/ty.rs"]);
        assert!(cited_paths("// everything under crates/ is a crate").is_empty(), "a bare prefix is not a file");
        assert!(cited_paths("// see docs/design/*.md").is_empty(), "a glob names no single file");
    }

    fn source(text: &str) -> ScannedFile {
        ScannedFile {
            rel: "crates/delulu-survey/src/codeowners.rs".to_string(),
            abs: "crates/delulu-survey/src/codeowners.rs".into(),
            kind: crate::scan::FileKind::RustSource,
            lines: 1,
            crate_name: Some("delulu-survey".to_string()),
            text: text.to_string(),
        }
    }

    fn doc(rel: &str) -> ScannedFile {
        ScannedFile {
            rel: rel.to_string(),
            abs: rel.into(),
            kind: crate::scan::FileKind::Doc,
            lines: 0,
            crate_name: None,
            text: String::new(),
        }
    }

    /// The V2-0 archive-mirror rule reaches **comments**, not only prose.
    ///
    /// Code in this repository names the decision that authorized it, and some of those records
    /// moved into the V1 archive. A comment naming the pre-move path still names a real file, so
    /// the reference is drawn to where the file is and the hop is a note — the alternative is a
    /// `comment-cites-missing-path` warning about a document that is right there.
    #[test]
    fn a_comment_citing_an_archived_path_is_a_note_with_its_reference_drawn() {
        let src = source("//! reversed in the review; see docs/design/PRODUCTION_READINESS_REVIEW.md §3.2\n");
        let idx = PathIndex::new(&[src.clone(), doc("docs/archive/v1/design/PRODUCTION_READINESS_REVIEW.md")]);
        let mut b = Builder::default();
        extract(&idx, &src, &mut b);

        let notes: Vec<&str> =
            b.findings.iter().filter(|f| f.severity == Severity::Note).map(|f| f.class.as_str()).collect();
        assert_eq!(notes, vec!["comment-cites-archived-path"]);
        assert!(
            b.edges.iter().any(|e| e.kind == EdgeKind::References
                && e.to == "doc:docs/archive/v1/design/PRODUCTION_READINESS_REVIEW.md"),
            "the reference points at the archived file: {:?}",
            b.edges.iter().map(|e| e.to.as_str()).collect::<Vec<_>>()
        );
    }

    /// **The falsification.** Take the archived file away and the identical comment is a warning
    /// again. Without this the rule would be "any `docs/` path is fine", which is not a rule.
    #[test]
    fn with_no_archived_file_the_same_comment_is_a_warning_again() {
        let src = source("//! reversed in the review; see docs/design/PRODUCTION_READINESS_REVIEW.md §3.2\n");
        let idx = PathIndex::new(std::slice::from_ref(&src));
        let mut b = Builder::default();
        extract(&idx, &src, &mut b);

        let classes: Vec<&str> = b.findings.iter().map(|f| f.class.as_str()).collect();
        assert_eq!(classes, vec!["comment-cites-missing-path"]);
        assert_eq!(b.findings[0].severity, Severity::Warning);
    }
}
