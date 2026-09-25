//! P4-10: `delulu examples [--json]` — the shipped example programs, each with its authority and the
//! line that runs it.
//!
//! The zero-shot loop's "generate a program" step (`V2_AI_NATIVE_DESIGN.md` §2) goes better from a
//! program that is known to be right than from a description of one. These are the programs
//! `examples/` ships and `tests/examples_run.rs` gates — embedded, so they read outside the checkout
//! as the Agent Skill does — and each comes with the authority report `delulu authority` prints for it
//! (computed now, by the same function, not stored) and the `delulu run` line its required grants
//! spell. The list is bound to the directory by a test: an example added there and not here fails.
//!
//! Package examples (`greeter`, `plugin_shout`) are several files and a manifest; they are named with
//! the command that checks them rather than embedded.

use serde_json::{json, Value};

/// Every single-file example under `examples/`, in the order a reader should meet them.
pub(crate) const EMBEDDED: &[(&str, &str)] = &[
    ("examples/guide/01_types.delulu", include_str!("../../../examples/guide/01_types.delulu")),
    ("examples/guide/02_option_and_lists.delulu", include_str!("../../../examples/guide/02_option_and_lists.delulu")),
    ("examples/guide/03_errors.delulu", include_str!("../../../examples/guide/03_errors.delulu")),
    ("examples/guide/04_effects.delulu", include_str!("../../../examples/guide/04_effects.delulu")),
    ("examples/guide/05_capabilities.delulu", include_str!("../../../examples/guide/05_capabilities.delulu")),
    ("examples/guide/06_actors.delulu", include_str!("../../../examples/guide/06_actors.delulu")),
    ("examples/demo.delulu", include_str!("../../../examples/demo.delulu")),
    ("examples/hello_wasm.delulu", include_str!("../../../examples/hello_wasm.delulu")),
    ("examples/numpy_mean.delulu", include_str!("../../../examples/numpy_mean.delulu")),
];

/// The package examples, by directory, with the command that checks each.
pub(crate) const PACKAGES: &[(&str, &str)] = &[
    ("examples/greeter", "delulu check examples/greeter --json"),
    ("examples/plugin_shout", "delulu plugin build examples/plugin_shout --json"),
];

/// The file's first comment block, as one line: what the example is for, in its own words. A file may
/// open with its `module` line or a blank line before the comment; those are skipped.
fn summary(src: &str) -> String {
    let lines: Vec<&str> = src
        .lines()
        .skip_while(|l| {
            let t = l.trim_start();
            t.is_empty() || t.starts_with("module ")
        })
        .take_while(|l| l.trim_start().starts_with("//"))
        .map(|l| l.trim_start().trim_start_matches('/').trim_start_matches('!').trim())
        .filter(|l| !l.is_empty() && !l.starts_with("anchors:"))
        .collect();
    let joined = lines.join(" ");
    // The first sentence is the summary; the rest is the file's to tell.
    match joined.find(". ") {
        Some(i) => joined[..=i].to_string(),
        None => joined,
    }
}

/// The `delulu run` line for a file whose report names `required` grants. An UPPERCASE word in a grant
/// is a placeholder the reader fills in, as `authority --grants` prints it.
fn run_line(path: &str, required: &[String]) -> String {
    let mut line = format!("delulu run {path} --no-prompt");
    for g in required {
        if g.contains(' ') || g.contains('*') {
            line.push_str(&format!(" --grant \"{g}\""));
        } else {
            line.push_str(&format!(" --grant {g}"));
        }
    }
    line
}

pub(crate) fn describe() -> Value {
    let programs: Vec<Value> = EMBEDDED
        .iter()
        .map(|(path, src)| match crate::cli::authority_of_text(path, src) {
            Ok(report) => {
                let required: Vec<String> = report["required_grants"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|g| g.as_str().map(str::to_string)).collect())
                    .unwrap_or_default();
                json!({
                    "path": path,
                    "summary": summary(src),
                    "checks": true,
                    "run": run_line(path, &required),
                    "authority": report,
                    "source": src,
                })
            }
            // Never expected — the examples gate checks every one — but a broken example is reported
            // as broken, never silently dropped from the list.
            Err(n) => json!({ "path": path, "summary": summary(src), "checks": false, "errors": n, "source": src }),
        })
        .collect();
    let packages: Vec<Value> =
        PACKAGES.iter().map(|(p, cmd)| json!({ "path": p, "check": cmd })).collect();
    json!({ "programs": programs, "packages": packages })
}

pub fn cmd_examples(rest: &[String]) -> i32 {
    if let Some(bad) = rest.iter().find(|a| a.starts_with('-') && a.as_str() != "--json") {
        eprintln!("error: `examples` does not know the option `{bad}`");
        eprintln!("  nothing was done — an option nobody understood is refused, never ignored");
        return 2;
    }
    if let Some(extra) = rest.iter().find(|a| !a.starts_with('-')) {
        eprintln!("error: `examples` takes no arguments (got `{extra}`)");
        return 2;
    }
    let doc = describe();
    if rest.iter().any(|a| a == "--json") {
        crate::cli::print_success_envelope("examples", json!({ "examples": doc }));
        return 0;
    }
    println!("the shipped examples — each checks, and each runs with the line under it:\n");
    for p in doc["programs"].as_array().into_iter().flatten() {
        println!("  {}", p["path"].as_str().unwrap_or(""));
        println!("    {}", p["summary"].as_str().unwrap_or(""));
        match p["run"].as_str() {
            Some(run) => println!("    $ {run}\n"),
            None => println!("    DOES NOT CHECK ({} error(s))\n", p["errors"]),
        }
    }
    println!("  packages:");
    for p in doc["packages"].as_array().into_iter().flatten() {
        println!("    {}  —  $ {}", p["path"].as_str().unwrap_or(""), p["check"].as_str().unwrap_or(""));
    }
    println!("\n`delulu examples --json` carries each program's source and full authority report.");
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The embedded list IS the directory: every single-file example under `examples/` and
    /// `examples/guide/` is embedded, and nothing embedded is missing from disk.
    #[test]
    fn the_embedded_list_is_exactly_the_examples_directory() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut on_disk: Vec<String> = Vec::new();
        for dir in ["examples", "examples/guide"] {
            for e in std::fs::read_dir(root.join(dir)).unwrap().flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.ends_with(".delulu") && e.path().is_file() {
                    on_disk.push(format!("{dir}/{name}"));
                }
            }
        }
        on_disk.sort();
        let mut embedded: Vec<String> = EMBEDDED.iter().map(|(p, _)| p.to_string()).collect();
        embedded.sort();
        assert_eq!(embedded, on_disk, "`examples.rs` and `examples/` disagree");
        for (p, _) in PACKAGES {
            assert!(root.join(p).join("delulu.toml").is_file(), "`{p}` is a package");
        }
    }

    /// Every embedded example checks, has a summary, and has a run line naming each grant its report
    /// requires.
    #[test]
    fn every_example_checks_and_its_run_line_names_its_grants() {
        let d = describe();
        for p in d["programs"].as_array().unwrap() {
            let path = p["path"].as_str().unwrap();
            assert_eq!(p["checks"], true, "{path} does not check");
            assert!(!p["summary"].as_str().unwrap().is_empty(), "{path} has no opening comment");
            let run = p["run"].as_str().unwrap();
            for g in p["authority"]["required_grants"].as_array().unwrap() {
                assert!(run.contains(g.as_str().unwrap()), "{path}: `{run}` lacks `{g}`");
            }
        }
    }
}
