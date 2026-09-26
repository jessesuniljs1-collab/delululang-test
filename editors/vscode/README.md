# DeluluLang for VS Code

Language support for DeluluLang — an authority- and effect-typed language where what a program
*may do* is part of its type.

This extension is a **thin client**. It contributes syntax highlighting, snippets and a language
configuration, and otherwise forwards everything to `delulu lsp`, the same LSP 3.17 server every
other editor uses. There are no editor-specific server features, by rule (Constitution §8.4): the
answers you get here are the compiler's answers, so they cannot drift from `delulu check`.

## Requirements

The `delulu` binary must be on your `PATH`. Check with:

```
delulu --version
```

If it lives somewhere else, set **`delulu.serverPath`** to its full path. If neither holds, the
extension says which binary it looked for and how many `PATH` entries it searched, rather than
leaving you with a quietly dead editor.

## What you get

Everything the server provides: diagnostics with the compiler's own codes and spans, code actions
built from typed repairs, hover carrying the effect row, completion, signature help,
definition/references, rename, document and workspace symbols, semantic tokens, and inlay hints
showing *inferred* effect rows on unannotated lambdas.

**Format Document** and `editor.formatOnSave` run the same `format_source` that `delulu fmt` calls —
byte for byte, enforced by a test — so your editor and `delulu fmt --check` in CI can never disagree
about whether a file is formatted. An already-canonical file produces **no** edit at all, and an
unparseable file is left exactly as written (formatting on save fires precisely when the file is
mid-edit and broken).

Range formatting is deliberately **not** offered: the formatter's contract is over a complete parse,
and quietly widening a selection to the whole file would reformat lines you did not choose.

`docs/editors.md` in the repository describes each feature in detail, including the deliberate
limits — a *local* rename is refused rather than guessed.

### Commands

| Command | What it does |
|---|---|
| **DeluluLang: Run this file** | `delulu run` in a terminal |
| **DeluluLang: Run test** | Runs *that* test by name |
| **DeluluLang: Show authority report** | The §10.5 answer to "what can this program do?", as JSON |
| **DeluluLang: Show authority atlas** | The call graph with each function's effect row on it, in a panel beside the file |
| **DeluluLang: Show Guard status (read-only)** | The Guard's mode, rules, pending requests and permits — what `delulu guard status --json` prints for the broker this editor's environment points at. Approving and changing rules stay in the terminal |

The same first three appear as code lenses on `main` and on `test` blocks.

### Tasks and problems

`delulu: check`, `build`, `test` and `fmt` are contributed as tasks (one per workspace folder, so a
multi-root workspace does not silently pick one). `fmt` is bound to `--check`, because a task that
rewrites your files when you press the build key is a surprise. The `$delulu` problem matcher turns
terminal output into clickable Problems entries.

A status bar item shows whether the language server is running — a server that failed to start should
be visible, not merely quiet.

## Settings

| Setting | Default | Scope | Meaning |
|---|---|---|---|
| `delulu.serverPath` | `delulu` | **machine-overridable** | Path to the `delulu` binary (the server is `delulu lsp`) |
| `delulu.trace.server` | `off` | window | Log protocol traffic to the **DeluluLang** output channel. Set to `verbose` before reporting a bug |

**`delulu.serverPath` is machine-scoped on purpose, and that is a security boundary.** At VS Code's
default scope a repository's own `.vscode/settings.json` could set it — and this extension launches
that path as a process the moment a `.delulu` file is opened. Opening a cloned repository would then
run a binary the repository chose, with no click from you. That was reproduced end to end before the
scope was added, and the extension additionally resolves the configured name to an absolute path
itself rather than letting the operating system search the current directory first.

`delulu run` and `delulu test` also refuse to run in a workspace you have not trusted, since they
execute that workspace's code. Analysis still runs there: reading a hostile file is what a language
server is for.

## Building it yourself

```
npm install
npm test          # unit tests — no test framework in the dependency tree
npm run package   # -> delulu-lang.vsix
npm run verify    # refuses a .vsix that would fail to activate
code --install-extension delulu-lang.vsix
```

`npm run verify` exists because a `.vsix` can package cleanly and still throw `Cannot find module` on
activation. It unpacks the built archive and checks that every `require` in every shipped file
resolves. `node e2e.js <path-to-delulu>` goes further: it launches a real VS Code against a real
server and requires the extension to activate, the server process to be alive, and diagnostics to
appear — because "no errors were reported" is what a working extension and a dead one have in common.

## Licence

Apache-2.0, the same licence as the repository this ships from. See `LICENSE` at the repository
root, and `NOTICE` for attribution.
