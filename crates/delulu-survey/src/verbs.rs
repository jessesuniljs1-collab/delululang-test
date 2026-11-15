//! The CLI's verbs, read from the one `match` that dispatches them — and the tests that drive each.
//!
//! Until this module the map knew modules, crates and tests, and nothing between a module and the
//! command a person types. So `impact mod:crates/delulu/src/run_cmd.rs` answered *"the crate, and
//! every one of its 109 test files"*: true, and no help in choosing which test to run. The dispatch
//! is one `match` in one function, so the relation *verb → the module that handles it* can be read
//! from the text with a line to cite, like every other edge here.
//!
//! Two kinds of edge come out of it, and they are **not equally sure**:
//!
//! - [`EdgeKind::DispatchesTo`], verb → module, is **source-derived**: the arm is on the cited line.
//! - [`EdgeKind::Exercises`], test → verb, is **inferred**: the test's text holds the verb as the
//!   first string of an argument list (`["run", …]`). That is how every integration test here calls
//!   the binary, but it is a reading of a string, not of a call — a helper that builds its arguments
//!   elsewhere is missed, and a list that merely begins with the same word is counted. So the edge
//!   carries [`Confidence::Inferred`], and every renderer says so beside it.
//!
//! What is deliberately **not** claimed: which functions a verb reaches after its handler. That
//! needs name resolution, which this crate does not do (it reads text, so that it opens when the
//! compiler does not build). An internal subcommand dispatched through a constant gets a node whose
//! name is the constant — its spelling is not on the line, and the map does not guess it.

use crate::{scan, Builder, Confidence, EdgeKind, NodeKind, Severity};

/// The file and function that hold the dispatch. Named, so a refactor that moves the CLI's entry
/// produces a finding instead of a map that quietly has no verbs.
pub const DISPATCH_FILE: &str = "crates/delulu/src/cli.rs";
const DISPATCH_FN: &str = "fn run_inner(";
const DISPATCH_MATCH: &str = "    match cmd.as_str() {";
/// rustfmt's indentation of an arm of a `match` in a function body. Arms of a `match` nested inside
/// an arm are deeper, and are sub-verbs of that command, not verbs.
const ARM_INDENT: &str = "        ";

/// The node id of a verb.
pub fn verb_id(name: &str) -> String {
    format!("verb:{name}")
}

pub fn extract(files: &[scan::ScannedFile], b: &mut Builder) {
    // A tree without the CLI — a fixture, a partial checkout — has no verbs, and that is not a defect.
    let Some(cli) = files.iter().find(|f| f.rel == DISPATCH_FILE) else { return };
    let lines: Vec<&str> = cli.text.lines().collect();
    let start = lines
        .iter()
        .position(|l| l.starts_with(DISPATCH_FN))
        .and_then(|f| lines[f..].iter().position(|l| *l == DISPATCH_MATCH).map(|m| f + m));
    let Some(start) = start else {
        b.finding(
            Severity::Warning,
            "cli-dispatch-not-found",
            DISPATCH_FILE,
            1,
            format!(
                "the CLI's dispatch (`{}` … `{}`) is not where the map reads it, so the map holds no verb and no \
                 test-to-verb edge",
                DISPATCH_FN.trim_end_matches('('),
                DISPATCH_MATCH.trim()
            ),
            "point `crates/delulu-survey/src/verbs.rs` at the function that now dispatches the commands",
        );
        return;
    };

    let cli_module = crate::rust::path_node_id(DISPATCH_FILE);
    let mut public: Vec<String> = Vec::new();
    let mut gate: Option<&str> = None;
    for (i, raw) in lines.iter().enumerate().skip(start + 1) {
        if *raw == "    }" {
            break;
        }
        let Some(arm) = raw.strip_prefix(ARM_INDENT).filter(|a| !a.starts_with(' ')) else { continue };
        if arm.starts_with("#[cfg(") {
            gate = Some(arm);
            continue;
        }
        let Some((pattern, handler)) = arm.split_once(" => ") else {
            gate = None;
            continue;
        };
        let line = (i + 1) as u32;
        let condition = gate.take();
        // The module a handler lives in: `crate::<module>::<function>(…)` names it; anything else is in this file.
        let module = handler
            .strip_prefix("crate::")
            .and_then(|h| h.split_once("::"))
            .map(|(m, _)| format!("crates/delulu/src/{m}.rs"))
            .filter(|p| files.iter().any(|f| &f.rel == p))
            .map(|p| crate::rust::path_node_id(&p))
            .unwrap_or_else(|| cli_module.clone());

        let mut names: Vec<(String, bool)> = Vec::new();
        if let Some(constant) = pattern.strip_prefix("s if s == ") {
            // Dispatched through a constant: the verb's spelling is not on this line, and is not guessed.
            names.push((constant.trim_start_matches("crate::").to_string(), false));
        } else {
            for alt in pattern.split(" | ") {
                let Some(word) = alt.trim().strip_prefix('"').and_then(|w| w.strip_suffix('"')) else { continue };
                // `--help` and `-V` are options spelled where a verb goes, not verbs.
                if !word.starts_with('-') && !word.is_empty() {
                    names.push((word.to_string(), true));
                }
            }
        }
        for (name, typed) in names {
            let id = verb_id(&name);
            let n = b.node(&id, NodeKind::CliVerb, &name);
            // Verbatim, as every summary here is: the arm itself, and its `cfg` gate when it has one.
            n.summary = Some(match condition {
                Some(c) => format!("{c} {arm}"),
                None => arm.to_string(),
            });
            if !typed {
                n.contents =
                    vec!["internal: dispatched through this constant; its spelling is not read here (DYNAMIC)".to_string()];
            }
            b.edge(EdgeKind::DispatchesTo, &id, &module, DISPATCH_FILE, line);
            if typed {
                public.push(name);
            }
        }
    }

    // Which tests drive which verb: the verb as the first string of an argument list. INFERRED.
    for f in files.iter().filter(|f| f.kind == scan::FileKind::RustTest && f.rel.starts_with("crates/delulu/tests/")) {
        let test = crate::rust::path_node_id(&f.rel);
        for verb in &public {
            let head = format!("[\"{verb}\"");
            if let Some(at) = f.text.lines().position(|l| head_of_a_list(l, &head)) {
                b.edge_with(Confidence::Inferred, EdgeKind::Exercises, &test, &verb_id(verb), &f.rel, (at + 1) as u32);
            }
        }
    }
}

/// Whether `line` opens a list with `head` (`["verb"`) and the string ends there — `["run"` must not
/// match `["runner"`, and the next character decides.
fn head_of_a_list(line: &str, head: &str) -> bool {
    line.match_indices(head).any(|(at, _)| matches!(line[at + head.len()..].chars().next(), Some(',') | Some(']') | None))
}

#[cfg(test)]
mod tests {
    use super::head_of_a_list;

    #[test]
    fn a_verb_is_matched_only_as_a_whole_first_string() {
        assert!(head_of_a_list(r#"    let o = delulu(&["run", "p.delulu"]);"#, r#"["run""#));
        assert!(head_of_a_list(r#"        .args(["run"])"#, r#"["run""#));
        assert!(!head_of_a_list(r#"    let o = delulu(&["runner", "x"]);"#, r#"["run""#));
        assert!(!head_of_a_list(r#"    let o = delulu(&["check", "run"]);"#, r#"["run""#));
    }
}
