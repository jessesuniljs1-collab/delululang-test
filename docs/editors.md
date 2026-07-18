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
- **Definition / references / rename** for module-level names across open documents
  (locals refuse rename honestly in v0.8 rather than guessing through shadows).
- **Semantic tokens** with dedicated kinds for effects, reference capabilities,
  capability types, and secrets.
- **Code lenses** on `fn main` (`▶ run`, `authority: {…}`) and every `test` block.
- **`delulu.authority`** (workspace/executeCommand) — the §10.5 authority report as
  JSON over the wire; agent harnesses call this instead of shelling out.

## What the server can NOT do — by construction

The LSP is analysis-only: it never runs code, never loads plugins, and holds no broker
connection or lease. A compromised workspace cannot use it as an effector. Its
availability is not a security property (spec §11).

## Per-editor notes

- **VS Code:** the in-repo extension skeleton is `editors/vscode/` (LSP client +
  TextMate grammar). `npm install && code --install-extension` after packaging, or use
  it as the template for a marketplace build.
- **Zed / Helix / Neovim (lspconfig) / Kate / Emacs (eglot):** point the editor's LSP
  config at `delulu lsp` for `*.delulu` — the three-line config above is all of it.
- **JetBrains:** via the native LSP support (2023.2+) or the LSP4IJ plugin; same command.
- **Agent IDEs / harnesses:** spawn `delulu lsp`, speak stdio; call `delulu.authority`
  for the report. `delulu` never prompts on machine channels (invariant 40) — the
  first-run picker cannot appear under `lsp`.
