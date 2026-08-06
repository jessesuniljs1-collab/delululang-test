// End-to-end: launch a REAL VS Code, activate the REAL extension, against the REAL server.
//
// Why this exists, in one sentence: the extension shipped with a language server that never
// started, and every test in the repository passed.
//
// The bug was that `extension.js` registered `delulu.authority`, which the language client also
// registers on the server's behalf during initialization. `registerCommand` threw
// "command already exists" from inside `client.start()`, initialization failed, the queued
// `didOpen` was dropped, and the server was shut down. Users got syntax highlighting and nothing
// else. Nothing caught it because of a gap in the *shape* of the test suite, not its size:
//
//   * `lsp_cli.rs` starts the server and speaks the protocol to it — but it is not a VS Code
//     client, so it never runs the client library's feature registration.
//   * `editor_contract.rs` compares command lists as source text — but reading two files cannot
//     tell you what a third-party library does at runtime.
//
// Both were green. Neither had ever started a client against a server. This does.
//
// Usage: `node e2e.js <path-to-delulu-binary>` (needs the `code` CLI and a display).
// It is not part of `npm test`, which must stay fast and headless.

const { spawnSync, spawnSync: sh } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");

const serverPath = process.argv[2];
if (!serverPath || !fs.existsSync(serverPath)) {
  console.error("usage: node e2e.js <path-to-delulu-binary>");
  process.exit(2);
}

const vsix = path.resolve(__dirname, "delulu-lang.vsix");
if (!fs.existsSync(vsix)) {
  console.error(`no package at ${vsix} — run \`npm run package\` first`);
  process.exit(2);
}

const root = fs.mkdtempSync(path.join(os.tmpdir(), "delulu-e2e-"));
const exts = path.join(root, "exts");
const udd = path.join(root, "udd");
const ws = path.join(root, "ws");
fs.mkdirSync(exts, { recursive: true });
fs.mkdirSync(path.join(udd, "User"), { recursive: true });
fs.mkdirSync(ws, { recursive: true });

// A file with a real diagnostic in it, so "the server answered" is observable and not just
// "the server did not crash".
fs.writeFileSync(path.join(ws, "hello.delulu"), "fn main() -> Int {\n    0\n}\n");
fs.writeFileSync(
  path.join(udd, "User", "settings.json"),
  JSON.stringify(
    {
      // USER settings, not workspace settings — `delulu.serverPath` is machine-scoped precisely so
      // a workspace cannot set it. Setting it here also checks that the scope did not break
      // legitimate configuration, which is the other half of that fix.
      "delulu.serverPath": serverPath,
      // Verbose tracing turns this test from "nothing went wrong" into "these things happened".
      // The first draft asserted on the ABSENCE of errors and passed a broken extension for the
      // same reason the bug shipped: silence is not evidence. With the trace on, the output channel
      // contains the actual protocol traffic, so the test can require that the server was asked to
      // initialize AND that it published diagnostics back.
      "delulu.trace.server": "verbose",
    },
    null,
    2
  )
);

const code = (args, timeout) =>
  spawnSync("code", args, { encoding: "utf8", shell: true, timeout });

console.log("installing the packaged extension into an isolated profile…");
const install = code(
  ["--extensions-dir", exts, "--user-data-dir", udd, "--install-extension", vsix, "--force"],
  180000
);
if (!/successfully installed/i.test(install.stdout || "")) {
  console.error("install failed:\n" + install.stdout + install.stderr);
  process.exit(1);
}

console.log("opening a workspace…");
code(
  [
    "--extensions-dir", exts,
    "--user-data-dir", udd,
    "--disable-workspace-trust",
    "--new-window",
    ws,
    path.join(ws, "hello.delulu"),
  ],
  60000
);

// Activation, server spawn, initialize, didOpen, publishDiagnostics.
const waitMs = 20000;
console.log(`waiting ${waitMs / 1000}s for activation and the initialize round-trip…`);
Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, waitMs);

const findLogs = (dir, out = []) => {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) {
      findLogs(p, out);
    } else if (/DeluluLang\.log$/i.test(e.name)) {
      out.push(p);
    }
  }
  return out;
};

const logsRoot = path.join(udd, "logs");
const logs = fs.existsSync(logsRoot) ? findLogs(logsRoot) : [];
let body = logs.map((f) => `--- ${path.basename(f)} ---\n${fs.readFileSync(f, "utf8")}`).join("\n");

// Did the extension activate at all? The extension host says so plainly, and asking it directly
// beats inferring activation from a side effect.
const hostLogs = [];
const findHostLogs = (dir) => {
  if (!fs.existsSync(dir)) return;
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) findHostLogs(p);
    else if (e.name === "exthost.log") hostLogs.push(fs.readFileSync(p, "utf8"));
  }
};
findHostLogs(logsRoot);
const activated = hostLogs.some((h) => /_doActivateExtension delulu-lang\.delulu-lang/.test(h));

// Is the server actually alive right now? Checked BEFORE the cleanup below, because a server that
// starts and dies is exactly the failure this test exists to catch.
const ps = spawnSync(
  "powershell",
  [
    "-NoProfile",
    "-Command",
    "Get-CimInstance Win32_Process -Filter \"Name='delulu.exe'\" | Select-Object -ExpandProperty CommandLine",
  ],
  { encoding: "utf8", timeout: 30000 }
);
const serverAlive = /\blsp\b/.test(ps.stdout || "");

// Leave the machine as we found it.
spawnSync(
  "powershell",
  [
    "-NoProfile",
    "-Command",
    `Get-CimInstance Win32_Process -Filter "Name='Code.exe'" | Where-Object { $_.CommandLine -like '*${path.basename(root)}*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }`,
  ],
  { encoding: "utf8", timeout: 30000 }
);

console.log(body || "(no DeluluLang output channel was created)");

const problems = [];

// Three POSITIVE requirements. Each one is a thing that must have happened, not a thing that must
// be absent — an absence is what a dead extension and a healthy one have in common.
if (!activated) {
  problems.push("the extension host never activated delulu-lang.delulu-lang");
}
if (!serverAlive) {
  problems.push(
    "no `delulu … lsp` process was running — the server never started, or started and exited"
  );
}
if (!/textDocument\/publishDiagnostics/.test(body)) {
  problems.push(
    "the trace shows no `textDocument/publishDiagnostics` — the server never analysed the open " +
      "file, so the editor would show no problems no matter what the code said"
  );
}
// A provider that is implemented but not ADVERTISED is dead code: no client will ever send the
// request. `documentFormattingProvider` was missing for the life of the project, so Format Document
// was greyed out while a law-verified formatter sat behind the CLI. Checking the wire here means the
// claim is made against what the server actually said, not against what the source says it says.
if (!/"documentFormattingProvider"\s*:\s*true/.test(body)) {
  problems.push(
    "the server did not advertise `documentFormattingProvider` — Format Document and " +
      "editor.formatOnSave would do nothing on .delulu files"
  );
}

// …and then the absence check, which is still worth having: the output channel is where the client
// reports its own failures, and anything at Error level there means a degraded editor.
for (const line of body.split(/\r?\n/)) {
  if (/\[Error[^\]]*\]/.test(line)) {
    problems.push(line.trim());
  }
}

if (problems.length) {
  console.error("\nFAIL — the extension did not come up cleanly:");
  for (const p of problems) {
    console.error("  • " + p);
  }
  process.exit(1);
}
console.log("\nOK — extension activated and the language server started with no errors reported.");
