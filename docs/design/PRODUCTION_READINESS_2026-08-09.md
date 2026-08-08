# Production-readiness verification — overnight pass, 2026-08-09

An autonomous, phase-by-phase pass (Opus 4.8) to make DeluluLang deployable and free of known
security defects, verified on Windows and Linux, with macOS honestly marked as un-runnable on this
bench. Each phase runs its gates, records evidence here, and commits. Standing rules: use and
regenerate the Survey every phase; never push to GitHub; harden-never-redefine Authority/Guard.

Platforms this bench can execute: **Windows 11** (native) and **Linux** (WSL2 Ubuntu-20.04, a real
second UID). **macOS cannot be executed here** — those rows are static review only, never claimed as
run.

| Phase | Scope | Result |
|---|---|---|
| A | Full-workspace suite baseline (Win + Linux) | ✅ green after fixing 2 conformance gates |
| B | LSP language server + VS Code extension | ✅ verified live; no changes needed |
| C | CLI + compiler end-to-end dogfood | ✅ clean on Win + Linux; no defects |
| D | Deployability / install from clean | ✅ portable archive builds, unpacks + runs (Linux) |
| E | Security discovery (untested surfaces) | pending |
| F | Miri (small batches) | pending |
| G | macOS honest assessment | pending |
| H | Documentation consistency + final full-suite | pending |

## Phase A — full-workspace suite baseline (commit `e5ae9ec`)

`cargo test --workspace` had not been run since the `DL1421` diagnostic shipped; it exposed two
`delulu-conform` failures the targeted suites had missed (everything else — hundreds of tests — was
green on both platforms):

- **`release_requires_full_coverage`** — `DL1421` (DISC-1 strict root issuance) shipped without a
  conformance witness → 327/328 (99.7%). It is a broker-only diagnostic no `.delulu` program can
  produce, so both polarity witnesses were added to `tests/conformance/witnesses.toml`. Coverage is
  now **328/328 (100.0%)** on Windows and Linux.
- **`the_generated_reference_is_not_stale`** — the generated `docs/reference/` chapters had drifted;
  regenerated (`delulu-conform --reference`).

Gates after fix: `delulu-conform` 28/0 (Win + Linux), `doctor_cli` 7/0, Survey 0 error / 0 warning.

## Phase B — LSP server + VS Code extension (no code change; verification only)

The DeluluLang language server (`delulu lsp`, `crates/delulu/src/lsp.rs`) and the VS Code client
(`editors/vscode/`) have regressed twice historically (a command-name collision that shut the server
down; a shell-injection in the run lens). Both are fixed in the current tree; this phase re-verified
against the **current build** rather than trusting the record.

**Server — driven live over real stdio JSON-RPC (independent of the Rust test harness):**
- `initialize` advertises `hoverProvider`, incremental `textDocumentSync`, and
  `executeCommandProvider = [delulu.authority]`.
- A **valid** document yields **0** diagnostics (no false positives); a **broken** document (a
  `match` missing its `Empty` arm) yields exactly **DL0407 "non-exhaustive match: missing Empty"** —
  real semantic analysis, not just lexing.
- `textDocument/hover` returns the typed signature and authority (`fn area: fn(Float, Float) ->
  Float`, `authority: pure`); `workspace/executeCommand delulu.authority` returns a full authority
  report.

**Extension — its own node tests + package verification (Windows, node v22):**
- `npm test` green (10/10): the CSP-insertion guard and the `server-resolve` security cases — a bare
  name is found by walking PATH, a **relative path is refused** (never resolved against the CWD), a
  missing absolute path errors naming the setting.
- `verify-package` OK — `delulu-lang.vsix` builds (97.8 KB), `./dist/extension.js` resolves with no
  missing modules, and all 4 contributed commands are both declared and registered.

The historical break — the extension registering `delulu.authority` and colliding with the client's
own registration — cannot recur: the editor command is the distinct `delulu.showAuthority`, and the
server advertises `delulu.authority`; the two names were confirmed distinct **live**. Cross-platform:
`lsp_cli.rs` (a real client driving the server) is 34/0/1-ignored on Windows and Linux; the live
drive above was on Linux; the extension is platform-agnostic JavaScript.

## Phase C — CLI + compiler end-to-end dogfood (no code change; verification only)

Dogfooded the compiler and CLI against the current binary on **both** Windows and Linux, with
identical results:

- `delulu new hello_dl` scaffolds a `bin` project (`src/main.delulu`, `delulu.toml`, `.gitignore`)
  and prints an onboarding note that teaches the authority model rather than hiding it.
- `delulu check .` → "checked clean (authority within manifest and pins)"; `delulu check <file>`
  and every `examples/guide/*.delulu` check clean (6/6).
- `delulu run src/main.delulu --grant console` → `hello, world`; `delulu build .` → "built clean".
- `delulu run --grant console examples/guide/01_types.delulu` → `area = 12.0 / size = 3 /
  point = 1.5, 2.5` — byte-identical on Windows and Linux.
- The authority model is enforced end to end: `run` without `--grant` stops with **DL0703**
  (`console was not granted`), pointing at the exact call site — a feature working, not a defect.
- A non-exhaustive `match` is **diagnosed as DL0407, never a panic**; the no-argument verbs give
  consistent, helpful errors (`check` / `run` / `build` each name the file-or-directory they need).

No defects found: the compiler produces correct output and correct exit codes on both platforms.

## Phase D — deployability / install from clean (no code change; verification only)

The shipped artifact is the **portable, Python-less** archive that `scripts/package-toolchain.sh`
builds (`cargo build --release -p delulu --no-default-features`); `cargo install delulu` from
crates.io is intentionally impossible (every internal crate is `publish = false`, per `STABILITY.md`).
Verified on Linux by behaving like a downloader — no repository, no Rust toolchain on the machine:

- **Build:** the release build finished in **3m36s** and produced
  `delulu-1.0.0-x86_64-unknown-linux-gnu.tar.gz` (10 MB, 21 files).
- **Unpack + run:** the extracted `bin/delulu` reports `delulu 1.0.0` and checks
  `examples/hello_wasm.delulu` clean, with nothing from this repository present.
- **Honest Python-less degradation:** `examples/numpy_mean.delulu` (which imports NumPy) prints
  `embedded Python is unavailable` and exits 0 — the DL1307 path surfaced as a **value, not a crash**,
  exactly as a machine with no interpreter behaves.
- **Integrity:** `sha256sum -c SHA256SUMS` passes (**20/20 OK, 0 FAILED**); the archive ships its own
  `INSTALL.txt`, `LICENSE`, `NOTICE`, `TRADEMARK.md`, `README.md`, `CHANGELOG.md`, `SECURITY.md`.
- **Lockfile:** `Cargo.lock` is committed and `cargo tree --locked` resolves the workspace against it
  with no update needed.

Honest limits: only the **x86_64-unknown-linux-gnu** archive was built on this bench (the packager is
per-host; the macOS and Windows archives were not produced — see Phase G). The from-source
default-features path (`cargo install --path crates/delulu`, which the VS Code extension suggests)
embeds CPython and so needs a Python present at build time; it was not exercised tonight — the
Python-less archive is the path that was.
