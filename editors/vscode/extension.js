// The DeluluLang VS Code client — a thin shell around `delulu lsp` (Stage 8, spec §3).
// One server, every surface: this extension adds NOTHING the server doesn't provide,
// by rule (Constitution §8.4 — no editor-specific server features, ever).
const vscode = require("vscode");
const { LanguageClient } = require("vscode-languageclient/node");

let client;

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
      const t = vscode.window.createTerminal("delulu run");
      t.show();
      t.sendText(`${serverPath} run ${vscode.Uri.parse(uri).fsPath}`);
    }),
    vscode.commands.registerCommand("delulu.test.run", (uri) => {
      const t = vscode.window.createTerminal("delulu test");
      t.show();
      t.sendText(`${serverPath} test ${vscode.Uri.parse(uri).fsPath}`);
    })
  );
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
