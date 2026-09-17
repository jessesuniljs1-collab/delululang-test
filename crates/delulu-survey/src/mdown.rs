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
        if let Some((num, summary)) = finding_definition(t).or_else(|| finding_heading_definition(t)) {
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
        // The path moved into the V1 archive in phase V2-0. A *link* and a *mention* are not the
        // same claim, so they do not get the same verdict: a link is navigation a reader clicks,
        // and one that lands nowhere is broken however well the map understands why. A mention in
        // a historical record is correct as written — it names where the file was when the record
        // was made — so the map follows it to the archived file and merely notes the hop.
        Resolution::Archived { cited, archived } => {
            if explicit {
                b.finding(
                    Severity::Error,
                    "broken-link",
                    rel,
                    lineno,
                    format!(
                        "link to `{target}` resolves to nothing in the tree \
                         (the file moved to `{archived}`)"
                    ),
                    "correct the path, or remove the link",
                );
            } else {
                let to = target_node(idx, b, &archived);
                b.edge(EdgeKind::LinksTo, self_id, &to, rel, lineno);
                b.finding(
                    Severity::Note,
                    "cites-archived-path",
                    rel,
                    lineno,
                    format!(
                        "prose names `{cited}`, which moved to `{archived}` in V2-0; \
                         the citation is historical and left as written"
                    ),
                    "a current document should cite the archived path",
                );
            }
        }
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

/// `### C88 · The effect row could be escaped entirely — CLOSED (D78)` — the shape the campaign's
/// LATER passes use to record a finding.
///
/// # Why both shapes are needed
///
/// `HARDENING_CAMPAIGN.md` opens with a ledger **table** (`## 3. Findings ledger`) covering the
/// original 2026-07-24 campaign. Every pass after it — the 2026-08-02 production-readiness pass, the
/// 2026-08-03 sweeps, and later — records its findings as **sections under its own dated heading**
/// instead, which is the right place for them: back-filling a 2026-08-03 finding into a 2026-07-24
/// ledger would misfile it.
///
/// Reading only the table therefore made the Survey report **eleven genuine findings** (C82–C92,
/// including C84's filesystem escape and C88's effect-row escape) as *"not campaign findings"* —
/// and it silently falsified a claim in `PRODUCTION_READINESS_REVIEW.md` that the note's *"only
/// trigger today is `C99`"*, which was true when written and stopped being true as soon as a pass
/// recorded a finding outside the table. The map fell behind the thing it maps: this project's own
/// design rule 1, pointed at the mapper.
fn finding_heading_definition(t: &str) -> Option<(u32, String)> {
    if !t.starts_with('#') {
        return None;
    }
    let rest = t.trim_start_matches('#').trim_start().strip_prefix('C')?;
    let (num, tail) = split_number(rest)?;
    // The middle dot is the separator these headings use; without it this is prose about a finding,
    // not the heading that records one.
    let title = tail.trim_start().strip_prefix('·')?.trim();
    if title.is_empty() {
        return None;
    }
    Some((num, title.to_string()))
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
    use crate::scan::{FileKind, ScannedFile};

    fn scanned(rel: &str) -> ScannedFile {
        ScannedFile {
            rel: rel.to_string(),
            abs: rel.into(),
            kind: FileKind::Doc,
            lines: 0,
            crate_name: None,
            text: String::new(),
        }
    }

    /// The tree after phase V2-0: one document archived, the live tree around it.
    fn archive_index() -> PathIndex {
        PathIndex::new(&[
            scanned("README.md"),
            scanned("docs/design/STAGE8_BUILD_ORDER.md"),
            scanned("docs/archive/v1/playbooks/STAGE8_PLAYBOOK.md"),
        ])
    }

    fn report(idx: &PathIndex, target: &str, explicit: bool) -> (Builder, Vec<(Severity, String)>) {
        let mut b = Builder::default();
        resolve(
            idx,
            &mut b,
            "doc:docs/design/STAGE8_BUILD_ORDER.md",
            "docs/design/STAGE8_BUILD_ORDER.md",
            "docs/design",
            7,
            target,
            explicit,
        );
        let found = b.findings.iter().map(|f| (f.severity, f.class.clone())).collect();
        (b, found)
    }

    /// A build order names the playbook it was written beside. The playbook moved into the V1
    /// archive in V2-0 and the sentence was left as written, because rewriting a record of what was
    /// true in 2026-07 is how a record stops being one. The map follows it to the archived file and
    /// says so — a **note**, not the warning it would have been.
    #[test]
    fn a_prose_citation_of_an_archived_path_is_a_note_and_still_draws_its_edge() {
        let idx = archive_index();
        let (b, findings) = report(&idx, "docs/playbooks/STAGE8_PLAYBOOK.md", false);
        assert_eq!(findings, vec![(Severity::Note, "cites-archived-path".to_string())]);
        assert!(
            b.findings[0].message.contains("docs/archive/v1/playbooks/STAGE8_PLAYBOOK.md"),
            "the note must say where the file went: {}",
            b.findings[0].message
        );
        let to: Vec<&str> = b.edges.iter().map(|e| e.to.as_str()).collect();
        assert_eq!(to, vec!["doc:docs/archive/v1/playbooks/STAGE8_PLAYBOOK.md"], "the edge is real");
    }

    /// A **link** is navigation. Knowing why it dangles does not make it work for the reader who
    /// clicks it, so an explicit link to a path that moved is still an error — with the new
    /// location in the message, because the fix should not require a search.
    #[test]
    fn an_explicit_link_to_an_archived_path_is_still_an_error() {
        let idx = archive_index();
        let (b, findings) = report(&idx, "docs/playbooks/STAGE8_PLAYBOOK.md", true);
        assert_eq!(findings, vec![(Severity::Error, "broken-link".to_string())]);
        assert!(
            b.findings[0].message.contains("the file moved to `docs/archive/v1/playbooks/STAGE8_PLAYBOOK.md`"),
            "the error must name the new location: {}",
            b.findings[0].message
        );
        assert!(b.edges.is_empty(), "a broken link is not an edge, however well understood");
    }

    /// **The falsification.** Take the archived file out of the index and the identical citation
    /// must go back to `prose-cites-missing-path` at `Warning`. A rule that reported a note either
    /// way would be a rule that cannot fail, and this repository does not keep those.
    #[test]
    fn with_no_archived_file_the_same_citation_is_a_warning_again() {
        let without = PathIndex::new(&[scanned("README.md"), scanned("docs/design/STAGE8_BUILD_ORDER.md")]);
        let (_, findings) = report(&without, "docs/playbooks/STAGE8_PLAYBOOK.md", false);
        assert_eq!(findings, vec![(Severity::Warning, "prose-cites-missing-path".to_string())]);
        // The explicit form, too: still an error, and now with nothing to add about where it went.
        let (b, findings) = report(&without, "docs/playbooks/STAGE8_PLAYBOOK.md", true);
        assert_eq!(findings, vec![(Severity::Error, "broken-link".to_string())]);
        assert!(!b.findings[0].message.contains("moved to"), "there is no archived file to name");
    }

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

    /// A finding recorded as a SECTION is as real as one recorded as a table row.
    ///
    /// The campaign's original ledger is a table; every pass after it records findings under its own
    /// dated heading instead, which is the right place for them. Reading only the table made the
    /// Survey report eleven genuine findings — C82–C92, including C84's filesystem escape and C88's
    /// effect-row escape — as *"not campaign findings"*, and silently falsified a claim in
    /// `PRODUCTION_READINESS_REVIEW.md` that the note's only trigger was `C99`.
    #[test]
    fn a_finding_recorded_as_a_heading_is_defined_like_a_ledger_row() {
        let (n, title) =
            finding_heading_definition("### C88 · The effect row could be escaped entirely — CLOSED (D78)")
                .expect("a `### C<n> · title` heading defines a finding");
        assert_eq!(n, 88);
        assert!(title.starts_with("The effect row"), "the heading text becomes the summary: {title}");
        // The later passes use `##` as well as `###`.
        assert_eq!(
            finding_heading_definition("## C70 · A normative runtime rule was false").map(|(n, _)| n),
            Some(70)
        );
        // And the table row still defines one, because both records are real.
        assert_eq!(finding_definition("| C69 | the audit chain | high | CLOSED |").map(|(n, _)| n), Some(69));
    }

    /// The separator is what distinguishes a RECORD from PROSE about a finding. Without it every
    /// sentence mentioning a `C<n>` under a heading would invent one — exactly what the Survey's
    /// provenance law forbids.
    #[test]
    fn prose_about_a_finding_does_not_define_one() {
        assert!(finding_heading_definition("### C88 was closed by D78").is_none(), "no separator, no record");
        assert!(finding_heading_definition("C88 · not a heading").is_none(), "a heading starts with #");
        assert!(finding_heading_definition("### Closing C88 · a retrospective").is_none(), "the number must lead");
        assert!(finding_heading_definition("### C88 ·").is_none(), "an empty title records nothing");
    }
}
