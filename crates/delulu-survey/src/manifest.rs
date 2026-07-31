//! Cargo manifests — the ground truth for "which crate depends on which".
//!
//! Read line-wise rather than with a TOML parser, so every dependency edge can cite the exact line
//! that declares it. The cost is that this understands manifest *shape*, not TOML semantics; the
//! bracket-depth tracking below is what keeps a multi-line `features = [...]` array from being
//! misread as a list of dependencies.

use crate::scan::ScannedFile;
use crate::{Builder, EdgeKind, NodeKind};

/// Sections whose keys are dependency names.
fn is_dependency_section(header: &str) -> bool {
    let h = header.trim_start_matches('[').trim_end_matches(']');
    h == "dependencies"
        || h == "dev-dependencies"
        || h == "build-dependencies"
        // `[target.'cfg(windows)'.dependencies]` and friends.
        || (h.starts_with("target.") && h.rsplit('.').next().is_some_and(|s| s.ends_with("dependencies")))
}

pub fn extract(f: &ScannedFile, b: &mut Builder) {
    if f.rel == "Cargo.toml" {
        workspace_members(f, b);
        return;
    }
    let Some(crate_dir) = f.rel.strip_suffix("/Cargo.toml") else { return };

    let mut name = String::new();
    let mut description = String::new();
    let mut section = String::new();
    let mut depth: i32 = 0;

    // First pass: the package name, because every edge below needs it as a source id.
    for line in f.text.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("name = ") {
            name = v.trim().trim_matches('"').to_string();
            break;
        }
    }
    if name.is_empty() {
        b.finding(
            crate::Severity::Error,
            "manifest-without-name",
            &f.rel,
            1,
            "manifest declares no `name`, so nothing can be attributed to this crate".to_string(),
            "add `name = \"...\"` under `[package]`",
        );
        return;
    }

    let crate_id = format!("crate:{name}");
    for (i, line) in f.text.lines().enumerate() {
        let lineno = i as u32 + 1;
        let t = line.trim();

        if depth == 0 && t.starts_with('[') && t.ends_with(']') {
            section = t.to_string();
            continue;
        }
        if depth > 0 {
            depth += bracket_delta(t);
            continue;
        }
        if section == "[package]" {
            if let Some(v) = t.strip_prefix("description = ") {
                description = v.trim().trim_matches('"').to_string();
            }
            if t.starts_with("publish") && t.contains("false") {
                b.tooling_crates.insert(name.clone());
            }
            continue;
        }
        if !is_dependency_section(&section) {
            continue;
        }

        if let Some(dep) = dependency_key(t) {
            // A `path = "../x"` dependency is a sibling crate; anything else is third-party. The
            // distinction is exactly where the supply-chain boundary sits.
            if let Some(target) = path_dependency(t) {
                b.node(&format!("crate:{target}"), NodeKind::Crate, &target);
                b.edge(EdgeKind::DependsOn, &crate_id, &format!("crate:{target}"), &f.rel, lineno);
            } else {
                b.node(&format!("ext:{dep}"), NodeKind::ExternalCrate, &dep);
                b.edge(EdgeKind::DependsOnExternal, &crate_id, &format!("ext:{dep}"), &f.rel, lineno);
            }
        }
        depth += bracket_delta(t);
    }

    let n = b.node(&crate_id, NodeKind::Crate, &name);
    n.path = Some(crate_dir.to_string());
    if !description.is_empty() {
        n.summary = Some(description);
    }
}

/// `members = [ "crates/x", ... ]` from the workspace root.
fn workspace_members(f: &ScannedFile, b: &mut Builder) {
    let mut in_members = false;
    for (i, line) in f.text.lines().enumerate() {
        let t = line.trim();
        if t.starts_with("members") && t.contains('[') {
            in_members = true;
            continue;
        }
        if in_members {
            if t.starts_with(']') {
                in_members = false;
                continue;
            }
            // A member entry is a quoted string and nothing else. Requiring the quote is what keeps
            // a `#` comment inside the list from being read as three more crates — which is exactly
            // what happened the first time this ran, and the comment in question was the one
            // introducing the Survey.
            let member = t.trim_end_matches(',');
            if !member.starts_with('"') {
                continue;
            }
            let member = member.trim_matches('"');
            if let Some(name) = member.rsplit('/').next().filter(|s| !s.is_empty()) {
                let id = format!("crate:{name}");
                b.node(&id, NodeKind::Crate, name);
                b.edge(EdgeKind::DependsOn, "workspace", &id, &f.rel, i as u32 + 1);
            }
        }
    }
    // Deliberately NOT a `Crate` node: the workspace is not a crate, and counting it as one would
    // make `facts.crates` say thirteen where the tree holds twelve. A map whose own totals are off
    // by one has no standing to report that another document's totals are off by one.
    b.node("workspace", NodeKind::Other, "workspace").summary =
        Some("the Cargo workspace root; its `members` list is the authoritative crate set".to_string());
}

/// The dependency name on a `name = ...` or `name.workspace = ...` line, if this line declares one.
fn dependency_key(t: &str) -> Option<String> {
    if t.is_empty() || t.starts_with('#') {
        return None;
    }
    let (key, _) = t.split_once('=')?;
    let key = key.trim();
    // `serde.workspace = true` declares `serde`.
    let key = key.split('.').next()?.trim();
    if key.is_empty() || !key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return None;
    }
    Some(key.to_string())
}

/// The sibling crate named by a `path = "../x"` value, if present.
fn path_dependency(t: &str) -> Option<String> {
    let after = t.split_once("path = \"")?.1;
    let value = after.split('"').next()?;
    value.rsplit('/').next().filter(|s| !s.is_empty()).map(str::to_string)
}

/// Net bracket/brace change on a line, ignoring anything inside a string or after a `#` comment.
/// This is what tells a continuation line from a new key.
fn bracket_delta(t: &str) -> i32 {
    let mut d = 0;
    let mut in_str = false;
    for c in t.chars() {
        match c {
            '"' => in_str = !in_str,
            '{' | '[' if !in_str => d += 1,
            '}' | ']' if !in_str => d -= 1,
            '#' if !in_str => break,
            _ => {}
        }
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::{FileKind, ScannedFile};

    fn manifest(text: &str) -> ScannedFile {
        ScannedFile {
            rel: "Cargo.toml".to_string(),
            abs: "Cargo.toml".into(),
            kind: FileKind::Manifest,
            lines: text.lines().count() as u32,
            crate_name: None,
            text: text.to_string(),
        }
    }

    /// The self-inflicted one: a `#` comment inside `members` was read as three more crates, and
    /// the comment in question was the one introducing the Survey. The map reported sixteen
    /// workspace members where the tree holds thirteen.
    #[test]
    fn a_comment_in_the_members_list_is_not_a_crate() {
        let mut b = Builder::default();
        workspace_members(
            &manifest(
                "[workspace]\nmembers = [\n    \"crates/delulu-diag\",\n    # Repository tooling, not language surface\n    # listed last because nothing depends on it\n    \"crates/delulu-survey\",\n]\n",
            ),
            &mut b,
        );
        let found: Vec<&str> = b
            .edges
            .iter()
            .filter(|e| e.from == "workspace")
            .map(|e| e.to.trim_start_matches("crate:"))
            .collect();
        assert_eq!(found, vec!["delulu-diag", "delulu-survey"], "a comment became a crate");
    }

    /// A sibling dependency is the one with a `path`; everything else is third-party, and the
    /// difference is exactly where the supply-chain boundary sits.
    #[test]
    fn a_path_dependency_is_a_sibling_and_the_rest_are_not() {
        assert_eq!(path_dependency("delulu-diag = { path = \"../delulu-diag\" }").as_deref(), Some("delulu-diag"));
        assert_eq!(path_dependency("serde = { workspace = true }"), None);
        assert_eq!(dependency_key("serde.workspace = true").as_deref(), Some("serde"));
        assert_eq!(dependency_key("# a comment"), None);
    }

    /// A multi-line `features = [...]` array must not be read as a list of dependencies.
    #[test]
    fn a_continuation_line_is_not_a_new_dependency() {
        assert_eq!(bracket_delta("windows-sys = { version = \"0.61\", features = ["), 2);
        assert_eq!(bracket_delta("    \"Win32_Foundation\","), 0);
        assert_eq!(bracket_delta("] }"), -2);
        assert_eq!(bracket_delta("serde = { workspace = true }"), 0);
    }
}
