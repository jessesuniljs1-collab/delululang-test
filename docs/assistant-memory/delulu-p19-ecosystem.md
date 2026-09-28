---
name: delulu-p19-ecosystem
description: "P19 (2026-08-07) — the ecosystem campaign: the extension had NO working language server since P18, a workspace could choose which binary it launched, and Miri was aimed at crates with zero unsafe"
metadata: 
  node_type: memory
  type: project
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-08-06T21:22:19.698Z
---

**Jesse's commission, 2026-08-07: "Tonight we are not building a compiler. Tonight we are building
an ecosystem."** Compiler, CLI, runtime, LSP, extension, formatter, docs as ONE product — and
"primarily for ai, llms, ai agents, robots, physical ai". Commits `eea95a7`, `3cf7f20`, `b0c69a0`,
`6789f13` (local only, never pushed).

**THE WORST FINDING WAS MINE.** `extension.js` registered `delulu.authority`, which a language
client **also registers on the server's behalf** for every entry in `executeCommandProvider.commands`.
`registerCommand` threw *"already exists"* from inside `client.start()` → initialization failed →
queued `didOpen` dropped → server shut down. **The extension shipped with syntax highlighting and
nothing else, in every workspace, since P18** — and the whole suite was green, because `lsp_cli.rs`
talks to the server without being a VS Code client, and `editor_contract.rs` compares source text,
which cannot say what a third-party library does at runtime. **That test asserted the OPPOSITE of the
correct rule** (it scraped the server's `execute_command` dispatch as lens commands and required the
client to register them) — it did not miss the bug, it demanded it. Lens now names
`delulu.showAuthority`; protocol name stays `delulu.authority`.

**Second vulnerability, witnessed end-to-end:** `delulu.serverPath` had VS Code's default `window`
scope, so a repo's own `.vscode/settings.json` could set it — and the extension launches that path
as a process when a `.delulu` file opens. **Unfixed build ran a planted executable 7 s after the
folder opened; fixed build never ran it**; only difference was the `scope: machine-overridable` line.
Windows `CreateProcess` searches the CWD before PATH, so the extension now resolves to an absolute
path itself (`server-resolve.js`). **Never add "auto-detect the binary in the workspace"** — that
convenience IS the attack.

**MIRI WAS AIMED AT THE WRONG CRATES.** unsafe counts: `delulu` 37 (Windows FFI), `delulu-runtime` 13
(`ptr.add`, `from_raw_parts`, `dlopen`), `delulu-diag` 2 — and **ZERO** in atlas/broker/syntax/check,
which were the four in the matrix. `validate_c_string` is PURE over caller-supplied memory, needs no
FFI, had five tests, and nothing had ever interpreted them: **6 tests, 0 UB, 2.4 s** vs 26 minutes for
a crate with no unsafe. Detector confirmed live by a deliberate OOB probe (*"at or beyond the end of
the allocation of size 4 bytes"*). **Choosing where to point a checker is a bigger decision than how
to configure it.**

**`fmt::` now COMPLETES under Miri — 16 passed, 1357 s** (third attempt). First cap counted
parse-clean files *kept* (bounds nothing — the walk still parses every reject-corpus file); second
counted files *examined* at the wrong value; only the third was preceded by `time`. **A per-test
budget does not bound a per-module run.** Budgets 4 files / 8 programs. Predicted 15 min, actual 23 —
per-unit figures from single-test runs miss module setup.

**Advisories are now ZERO** (`cargo deny`: advisories/bans/licenses/sources all ok). CHECKPOINT item
11 still said "four reachable, gate deliberately red" — stale since wasmtime 27→47. `continue-on-error`
removed from CI: a check that passes and cannot fail the build is not a gate. Two pyo3 CVEs stay
ignored on reachability, and **that argument is now a test** — it had justified itself with a
workspace-wide grep ("no `.nth(` anywhere") and there are now sixteen, none near Python. **A proxy
criterion drifts independently of the condition it stands for.**

**Self-matching scans bit twice in one night:** `pgrep -f "miri --sysroot"` matched the watcher's own
command line and deadlocked two background jobs; the pyo3 test matched its own comments. *A scan that
cannot tell a mention from a use will find its own explanation.*

**Also shipped:** LSP `textDocument/formatting` (Format Document did NOTHING before — falsifying it
left all 3 handler tests green and failed only the *advertisement* test); atlas webview with a CSP
that **I first wrote fail-open** (`String.replace` with no match returns the input unchanged);
snippets carrying effect rows; tasks; problem matcher; status bar; icon; `did you mean` (declines to
guess — `package` gets nothing); `delulu help <cmd>`; `error:` prefix convention now a test;
multi-root workspace test (roots was a `Vec` every test sent one entry to).

**Verified:** Windows 122 suites / 1577 tests / 0 failed; sweep 27/27; `cargo install` → PATH →
extension works with DEFAULT settings; `e2e.js` launches real VS Code and requires three POSITIVE
signals (falsified against the broken build). Trojan Source refused (DL0107). LSP survives malformed
framing and pipelining.

**Still open:** macOS never executed; CI never executed; Dockerfile + devcontainer **written, never
built** (Docker daemon not running); no CODE_OF_CONDUCT.md; pyo3 0.25 upgrade deferred.
See [[delulu-p18-uncertainty]], [[delulu-proof-campaign]], [[delulu-hardening-campaign]].
