# DeluluLang for VS Code

Language support for [DeluluLang](https://github.com/delulu-lang/delulu) — an authority- and
effect-typed language where what a program *may do* is part of its type.

This extension is a **thin client**. It contributes syntax highlighting and a language
configuration, and otherwise forwards everything to `delulu lsp`, the same LSP 3.17 server every
other editor uses. There are no editor-specific server features, by rule (Constitution §8.4): the
answers you get here are the compiler's answers, so they cannot drift from `delulu check`.

## Requirements

The `delulu` binary must be on your `PATH`. Check with:

```
delulu --version
```

If it lives somewhere else, set **`delulu.serverPath`** to its full path.

## What you get

Everything the server provides — diagnostics with the compiler's own codes and spans, code actions
built from typed repairs, hover with the effect row, the authority lens on unannotated lambdas,
completion, definition/references, rename, and semantic tokens. `docs/editors.md` in the repository
describes each in detail, including the deliberate limits (a *local* rename is refused rather than
guessed).

Three code lenses appear on `main` and on `test` blocks:

| Lens | What it does |
|---|---|
| **▶ run** | Runs the file via `delulu run` in a terminal |
| **▶ run test** | Runs *that* test by name via `delulu test <file> <name>` |
| **authority: {…}** | Asks the server for the §10.5 authority report and opens it as JSON |

Commands are executed with an argument vector, never a shell command string, so a path containing
a space — or a semicolon — is an argument and never a command.

## Settings

| Setting | Default | Meaning |
|---|---|---|
| `delulu.serverPath` | `delulu` | Path to the `delulu` binary (the server is `delulu lsp`) |

## Licence

Apache-2.0, the same licence as the repository this ships from. See `LICENSE` at the repository
root, and `NOTICE` for attribution.
