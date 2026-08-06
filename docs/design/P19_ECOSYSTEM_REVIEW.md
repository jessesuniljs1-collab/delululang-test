# P19 — the ecosystem campaign, and an independent production review

**2026-08-07.** Commissioned as *"tonight we are not building a compiler, tonight we are building an
ecosystem"* — compiler, CLI, runtime, package manager, broker, language server, VS Code extension,
formatter, linter, diagnostics, docs, examples, templates, build system, testing and benchmarks
treated as **one product**, for developers and for AI agents alike.

This document is the review at the end of it. It is written as seven readings of the same tree,
because a compiler engineer and a first-time developer do not find the same things.

---

## 0. The headline, before the personas

Three defects were found that a green test suite could not see, and **the first one was mine**.

### The extension had no working language server, and every test passed

`extension.js` registered `delulu.authority`. A language client **also** registers a VS Code command
for every entry in the server's `executeCommandProvider.commands`, as part of initialization. So the
extension collided with its own client: `registerCommand` threw *"command 'delulu.authority' already
exists"* from inside `client.start()`, initialization failed, the queued `didOpen` was dropped, and
the server was shut down. Users got syntax highlighting and **nothing else** — no diagnostics, no
hover, no completion — in every workspace, since the commit that was supposed to fix the editor.

Nothing caught it because of the **shape** of the suite, not its size:

- `lsp_cli.rs` (1,294 lines, 29 tests) starts the server and speaks LSP to it — but it is not a VS
  Code client, so it never runs the client library's feature registration.
- `editor_contract.rs` compares the command lists as source text — and reading two files cannot say
  what a third-party library does at runtime.

Worse, that second test asserted the **opposite** of the correct rule. It scraped the server's
`execute_command` dispatch as though those were lens commands and required the client to register
them. It did not merely miss the bug; it demanded it.

*The fix:* the lens now names `delulu.showAuthority` (the editor's command), distinct from
`delulu.authority` (the server's protocol command). The two lists are modelled separately, with
opposite obligations. `editors/vscode/e2e.js` launches a **real VS Code against a real server** and
requires three POSITIVE signals — activation, a live `delulu … lsp` process, and
`publishDiagnostics` in the trace. Its first draft asserted only the *absence* of errors and passed
the broken build, which is the same mistake that let the bug ship: **silence is not evidence.**

### A cloned repository could choose which binary the extension launched

`delulu.serverPath` had VS Code's default `window` configuration scope, which a workspace's own
`.vscode/settings.json` can write — and the extension launches that path as a process the moment a
`.delulu` file is opened. Opening a cloned repository therefore ran a binary the repository chose,
with no click from the user.

Reproduced end-to-end in an isolated VS Code profile (own `--user-data-dir` and `--extensions-dir`,
workspace trust **granted**, so that trust was not what refused it):

| build | scope | result |
| --- | --- | --- |
| unfixed | *(default `window`)* | planted executable ran **7 seconds** after the folder opened |
| fixed | `machine-overridable` | never ran (30 s) |

The only difference between the two packages was one `scope` line.

Related, and closed at the same time: on Windows `CreateProcess` resolves a bare name against the
**current directory** before `PATH`. A planted `delulu.exe` was *not* executed by VS Code today, but
that is the host's choice of working directory rather than a guarantee this extension made, so
`server-resolve.js` now walks `PATH` itself and refuses relative paths outright.

> **Do not add "helpfully find the binary in the workspace".** It is the obvious next convenience and
> it *is* the attack.

### Miri was aimed at four crates containing no `unsafe`

Counting `unsafe` in first-party sources:

| crate | sites | was in the Miri matrix |
| --- | --- | --- |
| `delulu` (Windows FFI: SIDs, handle inheritance) | 37 | no |
| `delulu-runtime` (`ptr.add`, `from_raw_parts`, `dlopen`) | 13 | no |
| `delulu-diag` | 2 | yes |
| `delulu-atlas`, `delulu-broker`, `delulu-syntax`, `delulu-check` | **0** | yes |

*"192 tests, 0 UB"* was true, and much weaker than it sounded.

Most of the gap is a real boundary that stays open — Miri cannot execute `dlopen` or Windows API
calls. But `validate_c_string`, a hand-rolled NUL scan with pointer arithmetic and
`slice::from_raw_parts` and the highest-risk function in the tree, is a **pure function over
caller-supplied memory**. It already had five tests against crafted Rust-owned buffers, and nothing
had ever interpreted them. They take **2.4 seconds** — against 26 minutes for a crate with no
`unsafe` in it.

The detector was confirmed live rather than assumed: a probe reading past a 4-byte allocation was
caught as *"at or beyond the end of the allocation of size 4 bytes"*. A gate that cannot fail is not
a gate, and that applies to the tool as much as to the test.

> **Choosing where to point a checker is a bigger decision than how to configure it.** A tool aimed
> at code that cannot exhibit the defect reports clean forever, and reads as coverage while doing it.

---

## 1. Seven readings

### A compiler engineer

- **Error recovery is real.** Three effect-row violations plus a type error are reported in one pass;
  the parser resyncs past a malformed signature on line 3 and still finds the one on line 7.
- **Pathological input does not crash.** A 240 KB single expression (60,000 operands) checks clean.
  Deeply nested input is refused as `DL0210` rather than overflowing the stack (fixed 2026-08-04).
- **Diagnostics carry what a fix needs**: code, span, a typed repair, and an `explain` pointer. A
  repair that would widen authority is `⚠`-titled and never marked preferred.
- **`delulu fmt` and the editor cannot disagree** — the LSP formatter calls the same
  `format_source`, and a test requires byte-identical output.
- Open: no principal types (inference is order-dependent), the optimizer in spec §2.1 is not
  implemented, and the WASM backend is a fragment.

### A security engineer

- Two live editor vulnerabilities found and closed **with witnesses observed to fail against the old
  code** (above).
- **Trojan Source is refused** (`DL0107`) with a precise span and a message naming both the attack
  and the legitimate `\u{…}` escape — even inside a comment. Identifiers are ASCII **by construction**
  and normatively specified, which is the defense rather than a side effect.
- `cargo deny`: advisories, bans, licences and sources all **ok**. The two remaining pyo3 CVEs are
  ignored on *reachability*, and that argument is now a **test**, not a comment — it had justified
  itself with a workspace-wide grep that has since drifted from sixteen counterexamples, none of them
  near Python.
- The LSP survives malformed framing (garbage headers, negative and absurd `Content-Length`,
  malformed JSON, null bodies) with a clean exit and no panic, and answers seven pipelined requests
  without deadlocking.
- Open, unchanged and unsoftened: `verify` declassifies without `Cap[Declassify]`; the audit anchor
  is not proof against an attacker who rewrites it; the clock ratchet gives monotonicity, not
  accuracy; no implicit-flow tracking.

### A DevOps engineer

- **CI has never executed.** The repository is not pushed. Prepared and green are different claims.
- **Two acceptance criteria had never run.** `fmt_laws_100k_gate` (criterion 4) and
  `criterion3_latency_150ms_on_10kloc_release` (criterion 3) were `#[ignore]`d for speed and no
  workflow passed `--ignored`, so the documented way to run them was a sentence addressed to whoever
  remembered. Both pass (132 s and 1.22 s) and now run nightly.
- `continue-on-error` removed from the advisory step: it was honest while four advisories were
  genuinely reachable, and became decoration the moment the check started passing.
- **Containers: written, never built.** `Dockerfile` and `.devcontainer/devcontainer.json` exist;
  the Docker daemon was not running, so `docker build` was never executed. A Dockerfile that has
  never been built is a plan.

### A VS Code extension maintainer

- Packaging is verified rather than assumed: `verify-package.js` unpacks the built `.vsix` and
  refuses one whose requires do not resolve. It caught a real regression tonight (test files
  shipping) and one of its own bugs (`node:test` reported as a missing package, because
  `builtinModules` does not list prefix-only builtins).
- `npm test` is 15 tests with **no test framework in the dependency tree** — every dependency an
  extension carries ships to every user.
- Shipped: formatting, atlas webview, snippets carrying effect rows, tasks via `ProcessExecution`,
  a `$delulu` problem matcher **verified against real compiler output**, a status bar, an icon
  generated from a committed script rather than an unregenerable blob.
- The webview's Content-Security-Policy was **fail-open when first written** — `String.replace`
  with no match returns the input unchanged — and now returns `undefined` so the caller refuses.

### A systems programmer

- 54 `unsafe` sites, 50 of them FFI. The pointer-decoding logic is now interpreted by Miri; the
  foreign calls remain out of reach and are recorded as an explicit assumption, not a to-do.
- `fmt::` completes under Miri at last: 16 passed, 0 failed, **1357 s**. Three attempts, and only
  the third was preceded by a measurement.

### A first-time developer

- `delulu new` produces a project that already checks, tests and runs, and its next-steps message
  explains **why** `--grant console` is not boilerplate. That is the best-written thing in the tool.
- The whole install path works with no configuration: `cargo install --path crates/delulu` →
  `~/.cargo/bin/delulu.exe` → the extension resolves it from `PATH` with **default settings**.
- A missing server now says which binary it looked for and how many `PATH` entries it searched,
  with buttons that work — the "Installation Guide" button previously opened a 404, because this
  project publishes to no public repository and the README says so.
- A typo gets an answer, not a wall: `delulu chekc` → *"did you mean `check`?"*, and `delulu package`
  correctly gets **no** guess, because a confident wrong suggestion is followed.

### A large enterprise adopting DeluluLang

- Licensing, NOTICE, TRADEMARK, SECURITY.md, CONTRIBUTING, SBOM and a support matrix are present and
  tested for consistency. **No CODE_OF_CONDUCT.md** — flagged, not written, because it is a policy
  commitment for the owner to make.
- **Nothing is distributed.** No registry entry, no release binary, no public repository.
- **Certification is NONE.** No safety standard, no external audit, no third-party review.
- **macOS has never been executed. Not once, in any phase.**

---

## 2. What I would still not ship without

1. macOS execution — currently type-checked only, never run.
2. One CI run on a real runner.
3. One `docker build`.
4. The pyo3 0.25 → 0.29 upgrade, done carefully (GIL handling is where a hasty migration introduces
   undefined behaviour).

Everything above that line is either verified by execution on Windows and Linux, or named here as
not verified. Nothing in this document is a claim I have not run.

---

## 3. The numbers, and how they were arrived at

| | Windows | Linux |
| --- | --- | --- |
| suites | 122 | 122 |
| tests passing | **1,580** | **1,586** |
| failed | 0 | 0 |

The 6-test difference is the documented platform delta, and it is a **set** difference rather than a
count: Linux runs 2 Unix-socket transport tests, 5 wasmtime live-engine contained-execution tests and
1 verified-plugin-on-wasm test; Windows runs the 2 refusal counterparts
(`windows_refuses_contained_execution_rather_than_risk_a_fastfail` and
`a_verified_plugin_on_wasm_inherits_the_windows_enforcement_refusal`). 8 − 2 = 6.

That number took two attempts to state correctly, which is worth recording. The first comparison
showed a **9**-test gap and appeared to mean three tests silently were not running on Windows. They
were: the Windows figure was taken **before** three tests were added later the same night, so a stale
measurement was being compared against a current one. A second cause of noise was `comm` on two files
sorted under different locales, which reports differences that are not there.

**Two measurements are only comparable if they were taken of the same thing.** Both numbers above
come from runs on the identical tree, with nothing edited during either — the discipline that had to
be relearned tonight after editing files mid-run invalidated an earlier Linux run (`doctor_cli` and
the survey-freshness test failed, both correctly, because the map genuinely was stale at the moment
they ran).

Other verification performed for this review, each by execution:

- `scripts/cli-sweep.sh` — **27/27**, run with both an absolute and a relative binary path.
- `cargo deny check advisories bans licenses sources` — all **ok**.
- `cargo install --path crates/delulu` → `~/.cargo/bin/delulu.exe` → the extension resolves it from
  `PATH` with **default settings**, confirmed by a live `delulu … lsp` process and an empty error
  channel.
- `editors/vscode/e2e.js` against a real VS Code — green, and **falsified** against the broken build.
- `npm test` — 15 tests, no test framework in the dependency tree.
- Miri: `delulu-runtime` FFI decoding **6 tests / 0 UB / 2.4 s**; `delulu-syntax` `fmt::`
  **16 tests / 0 UB / 1357 s**.
- Adversarial: Trojan Source refused (`DL0107`); malformed LSP framing exits cleanly six ways; seven
  pipelined requests all answered; a 240 KB single expression checks clean; all six `--json` surfaces
  emit valid JSON, including on failure.
