//! `.github/CODEOWNERS` — which paths are **entrenched**.
//!
//! The map already answers *where is this* and *what breaks if I change it*. It could not answer
//! *may I change it at all*, and for this repository that is a real question with a real answer
//! sitting in a file: `CODEOWNERS` names a handful of paths that require the **project lead
//! specifically, not any maintainer** — the constitution, the stability contract, `/rfcs/`, the
//! soundness audit and its laundering suite, the security policy, and the conformance machinery
//! that keeps the docs honest. Constitution §10 calls these entrenched; invariant 44 requires an
//! entrenchment analysis before any of them moves.
//!
//! **This is an attribute, not a verb, and the distinction was the whole finding.** An `owners <id>`
//! verb was proposed once and correctly rejected: every rule here names the same deliberate
//! placeholder until public launch, so the verb would be a constant function. That rejection was
//! then wrongly applied to *this* — but "who reviews it" and "is it entrenched" are different
//! claims, and only the second one is interesting while the first is a placeholder. Reversed in the
//! production-readiness review; see `docs/archive/v1/design/PRODUCTION_READINESS_REVIEW.md` §3.2.
//!
//! Provenance is unchanged by any of this: a marking names the CODEOWNERS line it was read from, and
//! the owner string is carried **verbatim, never interpreted**. The Survey has no opinion about who
//! `@PENDING-PUBLIC-project-lead` is.

use crate::scan::ScannedFile;
use crate::{Builder, Severity};

/// Where the rules live. One location, because a CODEOWNERS in a place GitHub does not read is a
/// policy nobody enforces.
pub const CODEOWNERS_PATH: &str = ".github/CODEOWNERS";

/// One CODEOWNERS rule, resolved.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct Entrenchment {
    /// The owner the rule names, verbatim. Never parsed into an identity.
    pub owner: String,
    /// The pattern that matched, as written in the file.
    pub pattern: String,
    /// The file this was read from, and the 1-based line. The provenance law does not weaken for
    /// an attribute any more than it does for an edge.
    pub file: String,
    pub line: u32,
}

/// A rule line: a pattern, whitespace, then one or more owners. Comments and blanks are skipped.
///
/// The `*` default is skipped too, and that is a decision rather than an omission: a rule that
/// matches every path marks nothing, because "entrenched" only means something when it separates
/// some paths from the rest. Marking all 900-odd nodes would make the attribute unreadable and
/// untrue at once.
fn rule(line: &str) -> Option<(&str, String)> {
    let t = line.split('#').next()?.trim();
    if t.is_empty() {
        return None;
    }
    let mut parts = t.split_whitespace();
    let pattern = parts.next()?;
    if pattern == "*" {
        return None;
    }
    let owners: Vec<&str> = parts.collect();
    if owners.is_empty() {
        return None; // a pattern with no owner assigns nothing
    }
    Some((pattern, owners.join(" ")))
}

/// Does `path` fall under `pattern`?
///
/// The subset of the CODEOWNERS syntax this repository actually uses: a leading `/` anchors at the
/// repository root, and a trailing `/` means a directory and everything beneath it. Anything richer
/// is deliberately **not** guessed at — see [`unsupported`].
///
/// A directory rule also covers the directory *itself*, which is not pedantry: a crate node's path
/// is `crates/delulu-conform` with no trailing slash, so `/crates/delulu-conform/` left the very
/// node a reader is most likely to ask about unmarked while marking all five of its modules.
fn covers(pattern: &str, path: &str) -> bool {
    let p = pattern.strip_prefix('/').unwrap_or(pattern);
    match p.strip_suffix('/') {
        Some(dir) => path == dir || path.starts_with(&format!("{dir}/")),
        None => path == p,
    }
}

/// Patterns this reader would have to guess about. A wildcard in the middle of a path has real
/// glob semantics, and a map that quietly matched it *approximately* would be asserting an
/// entrenchment nobody wrote — the exact failure the provenance law exists to prevent. So an
/// unsupported pattern is reported rather than interpreted.
fn unsupported(pattern: &str) -> bool {
    let p = pattern.strip_prefix('/').unwrap_or(pattern);
    p.contains('*') || p.contains('?') || p.contains('[')
}

pub fn extract(files: &[ScannedFile], b: &mut Builder) {
    let Some(f) = files.iter().find(|f| f.rel == CODEOWNERS_PATH) else { return };

    for (i, line) in f.text.lines().enumerate() {
        let lineno = i as u32 + 1;
        let Some((pattern, owner)) = rule(line) else { continue };

        if unsupported(pattern) {
            b.finding(
                Severity::Note,
                "codeowners-pattern-not-resolved",
                &f.rel,
                lineno,
                format!(
                    "`{pattern}` uses glob syntax this map does not interpret, so the paths it \
                     entrenches are not marked"
                ),
                "no action if the rule is correct; the Survey reports rather than guesses at a match",
            );
            continue;
        }

        // Mark every node whose path the rule covers, plus the directory node itself when there is
        // one. A reader asking about `rfcs/0001-broker-federation.md` deserves the same answer as a
        // reader asking about `/rfcs/`.
        let dir_id = pattern.strip_prefix('/').unwrap_or(pattern).strip_suffix('/').map(|d| format!("dir:{d}"));
        let mut matched = 0usize;
        let ids: Vec<String> = b
            .nodes
            .values()
            .filter(|n| {
                n.path.as_deref().is_some_and(|p| covers(pattern, p))
                    || dir_id.as_deref().is_some_and(|d| d == n.id)
            })
            .map(|n| n.id.clone())
            .collect();
        for id in ids {
            if let Some(n) = b.nodes.get_mut(&id) {
                n.entrenched = Some(Entrenchment {
                    owner: owner.clone(),
                    pattern: pattern.to_string(),
                    file: f.rel.clone(),
                    line: lineno,
                });
                matched += 1;
            }
        }

        // THE SKIP-BRANCH CASE. A rule guarding a path that is not there guards nothing while
        // reading, in a diff, exactly like a rule that guards something — which is the same failure
        // mode this file's own header warns about for a handle that does not exist. Renaming an
        // entrenched file without updating the rule silently un-entrenches it.
        if matched == 0 {
            b.finding(
                Severity::Error,
                "codeowners-rule-guards-nothing",
                &f.rel,
                lineno,
                format!("`{pattern}` matches no path in this repository, so the rule protects nothing"),
                "fix the pattern or remove the rule — a rule that matches nothing reviews nothing \
                 while looking like it reviews something",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rule_line_is_a_pattern_and_its_owners() {
        assert_eq!(rule("/rfcs/    @lead"), Some(("/rfcs/", "@lead".to_string())));
        assert_eq!(rule("/a.md @one @two"), Some(("/a.md", "@one @two".to_string())));
    }

    #[test]
    fn comments_blanks_and_the_catch_all_assign_nothing() {
        assert_eq!(rule("# a comment"), None);
        assert_eq!(rule("   "), None);
        assert_eq!(rule("*    @lead"), None, "a rule matching everything separates nothing");
        assert_eq!(rule("/orphan.md"), None, "a pattern with no owner assigns nothing");
        assert_eq!(rule("/a.md @lead # trailing"), Some(("/a.md", "@lead".to_string())));
    }

    #[test]
    fn a_directory_rule_covers_what_is_beneath_it_and_a_file_rule_does_not_over_reach() {
        assert!(covers("/rfcs/", "rfcs/0001-broker-federation.md"));
        assert!(covers("/rfcs/", "rfcs/README.md"));
        assert!(!covers("/rfcs/", "rfcsold/x.md"), "a prefix is not a directory");
        assert!(
            covers("/crates/delulu-conform/", "crates/delulu-conform"),
            "a directory rule covers the directory itself — a crate node's path carries no trailing slash"
        );
        assert!(covers("/SECURITY.md", "SECURITY.md"));
        assert!(!covers("/SECURITY.md", "docs/SECURITY.md"));
        assert!(!covers("/docs/design/CONSTITUTION.md", "docs/design/CONSTITUTION_NOTES.md"));
    }

    #[test]
    fn glob_syntax_is_reported_rather_than_approximated() {
        assert!(unsupported("/docs/**/*.md"));
        assert!(unsupported("*.rs"));
        assert!(!unsupported("/rfcs/"));
        assert!(!unsupported("/SECURITY.md"));
    }
}
