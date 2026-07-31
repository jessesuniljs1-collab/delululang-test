//! Reading Markdown.
//!
//! This repository keeps a great deal of load-bearing information in prose: which ruling authorized
//! a change, which finding a fix closed, which file a paragraph is talking about. Those references
//! are real relations and they are checkable, so the Survey checks them — a link to a file that is
//! not there is reported, not drawn.

use crate::paths::{strip_code_spans, PathIndex, Resolution};
use crate::rust::{cited_paths, diagnostic_codes, path_node_id};
use crate::scan::ScannedFile;
use crate::{Builder, EdgeKind, NodeKind, Severity};

/// Documents that *allocate* ruling ids, one namespace each.
fn ruling_namespace(rel: &str) -> Option<String> {
    let name = rel.rsplit('/').next()?;
    let stage = name.strip_suffix("_BUILD_ORDER.md")?.strip_prefix("STAGE")?;
    Some(format!("S{stage}"))
}

pub fn extract(idx: &PathIndex, f: &ScannedFile, b: &mut Builder) {
    let Some((self_id, _)) = f.node_identity() else { return };
    let dir = f.rel.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let namespace = ruling_namespace(&f.rel);

    let mut title: Option<String> = None;
    let mut headings: Vec<String> = Vec::new();
    let mut in_fence = false;

    for (i, line) in f.text.lines().enumerate() {
        let lineno = i as u32 + 1;
        let t = line.trim();

        if t.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }

        if let Some(h) = t.strip_prefix("# ") {
            title.get_or_insert_with(|| h.trim().to_string());
        } else if let Some(h) = t.strip_prefix("## ") {
            headings.push(h.trim().to_string());
        }

        // --- ruling and finding ids ------------------------------------------------------------
        if let Some(ns) = &namespace {
            if let Some((num, heading)) = ruling_definition(t) {
                let id = format!("ruling:{ns}-D{num}");
                b.defined_rulings.insert(format!("{ns}-D{num}"));
                let n = b.node(&id, NodeKind::Ruling, &format!("{ns}-D{num}"));
                n.summary = Some(heading);
                b.edge(EdgeKind::Defines, &self_id, &id, &f.rel, lineno);
            }
        }
        if let Some((num, summary)) = finding_definition(t) {
            let id = format!("finding:C{num}");
            b.defined_findings.insert(format!("C{num}"));
            let n = b.node(&id, NodeKind::Finding, &format!("C{num}"));
            n.summary = Some(summary);
            b.edge(EdgeKind::Defines, &self_id, &id, &f.rel, lineno);
        }
        for num in id_citations(t, 'D') {
            b.raw_ruling_cites.push((f.rel.clone(), lineno, num));
        }
        for num in id_citations(t, 'C') {
            b.raw_finding_cites.push((f.rel.clone(), lineno, num));
        }

        // --- diagnostic codes --------------------------------------------------------------------
        for c in diagnostic_codes(t) {
            let code_id = format!("code:{c}");
            b.node(&code_id, NodeKind::DiagnosticCode, &c);
            b.cited_codes.insert(c);
            b.edge(EdgeKind::DocumentsCode, &self_id, &code_id, &f.rel, lineno);
        }

        // --- links and inline paths ---------------------------------------------------------------
        // A Markdown link never lives inside a code span or a fenced block. This repository writes
        // enough inline Delulu and Rust that skipping those is not an optimization — generic
        // syntax like `fn map[T, U](xs: List[T])` contains the very `](` that opens a link target.
        if !in_fence {
            for target in markdown_links(&strip_code_spans(t)) {
                resolve(idx, b, &self_id, &f.rel, dir, lineno, &target, true);
            }
        }
        for p in cited_paths(t) {
            resolve(idx, b, &self_id, &f.rel, dir, lineno, &p, false);
        }
    }

    headings.dedup();
    let n = b.node(&self_id, NodeKind::Doc, &f.rel);
    n.summary = title;
    n.contents = headings;
}

/// Draw the edge if the target exists; report it if it does not.
///
/// `explicit` separates a written Markdown link — which the author meant as navigation and which is
/// simply broken if it dangles — from a path mentioned in passing, which may be an example or a
/// historical reference. Both are reported; only the first is an error.
#[allow(clippy::too_many_arguments)]
fn resolve(
    idx: &PathIndex,
    b: &mut Builder,
    self_id: &str,
    rel: &str,
    dir: &str,
    lineno: u32,
    target: &str,
    explicit: bool,
) {
    let clean = target.split('#').next().unwrap_or(target);
    if clean.is_empty() {
        return; // a pure `#anchor` is intra-document
    }
    match idx.resolve(clean, dir, None) {
        Resolution::Found(p) => {
            let to = target_node(idx, b, &p);
            b.edge(EdgeKind::LinksTo, self_id, &to, rel, lineno);
        }
        Resolution::Ambiguous(candidates) => b.finding(
            Severity::Note,
            "ambiguous-path-reference",
            rel,
            lineno,
            format!("`{target}` matches {} files: {}", candidates.len(), candidates.join(", ")),
            "write the path from the repository root so it names one file",
        ),
        Resolution::Missing => {
            if explicit {
                b.finding(
                    Severity::Error,
                    "broken-link",
                    rel,
                    lineno,
                    format!("link to `{target}` resolves to nothing in the tree"),
                    "correct the path, or remove the link",
                );
            } else {
                b.finding(
                    Severity::Warning,
                    "prose-cites-missing-path",
                    rel,
                    lineno,
                    format!("prose names `{target}`, which is nowhere in the tree"),
                    "correct the path, or say plainly that it no longer exists",
                );
            }
        }
    }
}

/// The node id for a resolved target, creating the node when the target is a directory.
///
/// Files already have nodes from the walk; directories do not, because nothing walks a directory
/// as a thing in itself. A link to `docs/reference/` is still a real link, so the directory becomes
/// a node rather than the edge becoming a dangling one.
pub fn target_node(idx: &PathIndex, b: &mut Builder, p: &str) -> String {
    if idx.is_dir(p) {
        let id = format!("dir:{p}");
        let name = p.rsplit('/').next().unwrap_or(p).to_string();
        let n = b.node(&id, NodeKind::Directory, &name);
        n.path = Some(p.to_string());
        id
    } else {
        path_node_id(p)
    }
}

/// `[text](target)` targets that are not external URLs.
fn markdown_links(t: &str) -> Vec<String> {
    let mut out = Vec::new();
    let chars: Vec<char> = t.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == ']' && chars.get(i + 1) == Some(&'(') {
            let mut j = i + 2;
            let mut depth = 1;
            let mut target = String::new();
            while j < chars.len() {
                match chars[j] {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                target.push(chars[j]);
                j += 1;
            }
            let target = target.trim().to_string();
            let external = target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
                || target.starts_with('#');
            // A relative link target has no whitespace in it. Anything that does is prose or code
            // that happened to contain `](` — belt and braces alongside the code-span stripping.
            if !external && !target.is_empty() && !target.contains(char::is_whitespace) {
                out.push(target);
            }
            i = j;
        }
        i += 1;
    }
    out
}

/// `**D64 — heading.**` at the start of a line: the shape this repository uses to allocate a ruling.
fn ruling_definition(t: &str) -> Option<(u32, String)> {
    let rest = t.strip_prefix("**D")?;
    let (num, tail) = split_number(rest)?;
    // The em dash is the separator the build orders use; without it this is a citation, not a
    // definition.
    let heading = tail.trim_start().strip_prefix('—')?.trim();
    let heading = heading.trim_end_matches("**").trim();
    Some((num, heading.to_string()))
}

/// `| C69 | description | severity | status |` — a row in the campaign ledger.
fn finding_definition(t: &str) -> Option<(u32, String)> {
    let rest = t.strip_prefix("| C")?;
    let (num, tail) = split_number(rest)?;
    let tail = tail.trim_start().strip_prefix('|')?;
    let desc = tail.split('|').next()?.trim();
    if desc.is_empty() {
        return None;
    }
    Some((num, desc.to_string()))
}

/// The `S9` of an `S9-D21` citation, if the id at `at` carries one.
fn explicit_stage(chars: &[char], at: usize) -> Option<String> {
    if at < 3 || chars[at - 1] != '-' {
        return None;
    }
    let mut k = at - 1;
    while k > 0 && chars[k - 1].is_ascii_digit() {
        k -= 1;
    }
    if k == at - 1 || k == 0 || chars[k - 1] != 'S' {
        return None;
    }
    let digits: String = chars[k..at - 1].iter().collect();
    Some(format!("S{digits}"))
}

fn split_number(s: &str) -> Option<(u32, &str)> {
    let digits: String = s.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    Some((digits.parse().ok()?, &s[digits.len()..]))
}

/// Citations of the form `D64`, `C12`, `D19c` anywhere in a line.
///
/// Deliberately conservative: the token must not be glued to a word on either side, and a trailing
/// sub-letter (`D19c`) is accepted because the build orders use it. `3D` and `CI` do not match.
pub fn id_citations(t: &str, letter: char) -> Vec<String> {
    let chars: Vec<char> = t.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == letter && chars.get(i + 1).is_some_and(char::is_ascii_digit) {
            let before_ok = i == 0 || !(chars[i - 1].is_alphanumeric() || chars[i - 1] == '_');
            let mut j = i + 1;
            while chars.get(j).is_some_and(char::is_ascii_digit) {
                j += 1;
            }
            let num: String = chars[i + 1..j].iter().collect();
            // One optional lowercase sub-letter, then the token must end.
            let after = if chars.get(j).is_some_and(|c| c.is_ascii_lowercase()) { j + 1 } else { j };
            let after_ok = chars.get(after).is_none_or(|c| !c.is_alphanumeric() && *c != '_');
            if before_ok && after_ok && num.len() <= 3 {
                // `S9-D21` names the Stage-9 ruling explicitly — the convention this repository
                // documents in `CHANGELOG.md` and in the Stage-10 ledger heading. Reading it as a
                // bare `D21` would attach it to the wrong stage, which is the exact confusion the
                // prefix was introduced to end.
                match explicit_stage(&chars, i) {
                    Some(stage) => out.push(format!("{stage}-{num}")),
                    None => out.push(num),
                }
            }
            i = j;
            continue;
        }
        i += 1;
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ruling numbers are allocated once per stage. Fusing `D21` from two build orders into one
    /// node would merge two unrelated decisions — a mistake this project has already made once in
    /// prose, which is why the map refuses to make it in data.
    #[test]
    fn a_ruling_belongs_to_the_stage_that_allocated_it() {
        assert_eq!(ruling_namespace("docs/design/STAGE10_BUILD_ORDER.md").as_deref(), Some("S10"));
        assert_eq!(ruling_namespace("docs/design/STAGE9_BUILD_ORDER.md").as_deref(), Some("S9"));
        assert_eq!(ruling_namespace("docs/design/STAGE10_SPECIFICATION.md"), None, "a spec allocates no rulings");
        assert_eq!(ruling_namespace("CHANGELOG.md"), None);
    }

    /// A definition allocates the id; a mention merely cites one. Only the em dash separates them.
    #[test]
    fn a_definition_is_not_the_same_as_a_mention() {
        let (n, heading) = ruling_definition("**D64 — A diagnostic names the types it is about.**").expect("a definition");
        assert_eq!(n, 64);
        assert_eq!(heading, "A diagnostic names the types it is about.");
        assert_eq!(ruling_definition("**D64** closed this"), None, "no em dash, so this cites rather than allocates");
    }

    /// The campaign ledger allocates finding ids as table rows.
    #[test]
    fn a_ledger_row_allocates_a_finding() {
        let (n, desc) = finding_definition("| C69 | The audit chain's single-writer assumption | medium | CLOSED |")
            .expect("a ledger row");
        assert_eq!(n, 69);
        assert!(desc.starts_with("The audit chain"), "the description comes from the row: {desc}");
    }

    /// Conservative on purpose: `3D` is not a ruling and `DL0501` is not `D0501`.
    #[test]
    fn a_citation_must_stand_on_its_own() {
        assert_eq!(id_citations("closed under D64 and D65", 'D'), vec!["64", "65"]);
        assert_eq!(id_citations("ruling D19c applies", 'D'), vec!["19"], "a sub-letter is part of the same ruling");
        assert!(id_citations("a 3D model", 'D').is_empty(), "glued to a digit, it is not a citation");
        assert!(id_citations("DL0501", 'D').is_empty(), "a diagnostic code is not a ruling");
    }

    /// `S9-D21` and `D21` are different rulings, and the prefix exists precisely because this
    /// project once confused them. Reading the prefixed form as a bare number would re-create the
    /// confusion in data.
    #[test]
    fn an_explicitly_staged_citation_keeps_its_stage() {
        assert_eq!(id_citations("the S9-D21 precedent", 'D'), vec!["S9-21"]);
        assert_eq!(id_citations("closed under D21", 'D'), vec!["21"], "a bare number carries no stage");
        assert_eq!(id_citations("see S10-D64", 'D'), vec!["S10-64"]);
    }

    /// A relative link target has no spaces in it. This is the belt to the code-span braces.
    #[test]
    fn prose_that_happens_to_contain_a_bracket_paren_is_not_a_link() {
        assert_eq!(markdown_links("see [the guide](docs/GETTING_STARTED.md)"), vec!["docs/GETTING_STARTED.md"]);
        assert!(markdown_links("fn map[T, U](xs: List[T]) -> U").is_empty(), "generic syntax is not a link");
        assert!(markdown_links("[home](https://example.com)").is_empty(), "external links are not tree edges");
    }
}
