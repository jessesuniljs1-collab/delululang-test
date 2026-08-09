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
| E | Security discovery (hardware adapter) | ✅ found + fixed a reply-framing gap (fail-closed) |
| F | Miri (small batches) | ✅ 0 UB on the interpretable unsafe; FFI out of reach (stated) |
| G | macOS honest assessment | ⚠️ designed-for (cfg audit); UNVERIFIED (no Mac; C toolchain blocks cross-check) |
| H | Documentation consistency + final full-suite | ✅ 124 test binaries green, both platforms |

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

## Phase E — security discovery: the hardware-adapter subprocess protocol (finding + fix)

Applied the discovery mandate to the D23 hardware adapter (`crates/delulu-runtime/src/adapter.rs`),
an operator-supplied subprocess speaking a line protocol over stdio with no signature check — the
newest and least-attacked real-world surface. Its tests already cover garbled output, "anything that
is not OK is not success", timeout, death, and a missing program. The unattacked assumption was
**one reply line per command**, which nothing enforced.

**Finding (fixed, witnessed):** the reader thread feeds an unbounded channel and each exchange reads
one line, so an untrusted adapter that emitted an EXTRA line per command left it buffered — and the
next command read that stale line as its reply: an off-by-one reply desync, silently, with no
poisoning. That is precisely the half-open hazard the module's rule 3 exists to forbid ("a reply
must never be attributed to the wrong command"), reached via an extra line rather than a late one —
rule 3 had only closed the timeout path. Witnessed against the unfixed code with a two-line-per-command
adapter: `first = Ok, second = Ok, poisoned = false` (the second "success" was the first command's
leftover line).

**Fix:** before sending each request, any line already waiting in the channel is unsolicited output
the previous exchange did not consume — the framing is in doubt, so the adapter is poisoned and the
command fails closed. Adapter tests **10/0 on Windows and Linux** (the new witness plus the nine that
already passed), runtime lib 162/0, hw-adapter / actuate / dead-man integration green, clippy clean.

**Honest severity:** this is **not** a containment break. The envelope is still validated host-side
against the grant *before* any byte reaches the adapter (`crates/delulu-runtime/src/device.rs`,
confirmed), so every dispatched command stays within the granted envelope whatever the adapter
replies. The defect was reply *attribution* under a misbehaving/untrusted adapter; the fix makes it
fail closed, matching the module's own rule 3. This is a robustness hardening, not a vulnerability
disclosure — recorded honestly as such.

## Phase F — Miri, in small batches (no code change; negative result)

Miri interprets Rust MIR and cannot execute real FFI or syscalls, so it validates the runtime's
Rust-level `unsafe` but not the FFI/syscall `unsafe`. That split is the whole story of this phase,
and it is stated rather than hidden. Run on the WSL nightly toolchain with
`-Zmiri-disable-isolation`, one tiny batch at a time.

**What Miri validated — 0 UB:**
- The **secret-zeroing** `unsafe` in `crates/delulu-runtime/src/value.rs` — `as_bytes_mut()` +
  `ptr::write_volatile(b, 0)` inside the secret's `Drop` — exercised by the three `secret_*` tests
  (`daemon_secret_handle_holds_no_bytes`, `secret_never_reaches_stdout…`, `trace_never_leaks…`):
  **0 UB**. Miri would have caught an out-of-bounds write, an aliasing violation, or a
  use-after-free in that pointer loop; none is present.
- The runtime's determinism and filesystem-containment paths (`prim::` — RNG seed mapping, fixed
  clock, hardlink/symlink containment): 8 tests, **0 UB**. 11 runtime tests total under Miri, clean.

**What Miri cannot reach, said plainly:** the bulk of the workspace's `unsafe` is FFI/syscall —
`crates/delulu/src/broker_transport.rs` (Windows named pipe / Unix domain socket, ~30 sites) and
`crates/delulu-runtime/src/foreign.rs` (embedded CPython via pyo3, ~11 sites). Miri cannot execute
those — there is no real pipe and no real interpreter under Miri — so it neither passes nor fails
them; it simply does not run them. They are defended instead by their own tests, the
process-isolation contracts (`foreign_worker.rs`), and the OS boundary — not by Miri, and this log
does not pretend otherwise.

## Phase G — macOS: an honest assessment (static-only; no Mac on this bench)

There is no Mac here, so macOS is **not run**, and this phase claims nothing it did not check. What it
*could* check — the platform `cfg` paths and a cross-target type-check — shows macOS is carefully
designed-for, and pins the exact reason it stays unverified.

**The macOS-specific code is deliberate, not accidental.** Every Linux-specific path has a documented
macOS story:
- `microvm` isolation is Linux-only (`crates/delulu/src/main.rs`); every other platform — macOS
  included — refuses `--isolation microvm` with **DL1408** rather than faking a weaker isolation as
  equivalent.
- `PR_SET_PDEATHSIG` (kill-the-worker-if-the-host-dies) is Linux-only; `crates/delulu/src/foreign_worker.rs`
  explicitly notes that a bare `cfg(unix)` there **would break the macOS build** (libc omits it on
  Apple/BSD), so it is `cfg(target_os = "linux")` with a portable fallback (the `WorkerGuard`'s
  explicit kill on drop), and `libc` is scoped as a Linux-only dependency (`crates/delulu/Cargo.toml`).
  A kqueue `EVFILT_PROC` watch is named as the macOS hardening if a macOS lane ever goes live.
- The Unix-socket path limit is set per-OS (`crates/delulu/src/broker_transport.rs`: 104 on macOS,
  108 elsewhere), and `cli.rs` already carries macOS-specific test branches.

No cfg bug was found; nothing needed changing.

**Cross-target type-check.** `rustup target add aarch64-apple-darwin` (the macOS Rust std installs
fine) then `cargo check --target aarch64-apple-darwin -p delulu` compiles the pure-Rust dependency
graph for Apple silicon and stops at a **C dependency's build script**: `cc-rs: failed to find tool
"cc"`. That is the honest blocker — `blake3` (and `libffi` via pyo3) build C, which needs a
darwin-targeting C compiler this Windows bench does not have; a real Mac ships one (Xcode
command-line tools). The stop is a missing C toolchain, not a fault in the DeluluLang Rust.

**Honest status:** macOS is **designed-for** (correct, documented `cfg` handling; the Rust
dependencies cross-compile) but **UNVERIFIED** — never built, never run, no CI run on hardware.
`docs/design/CROSS_PLATFORM_VERIFICATION.md` already marks every macOS gate "never run", and that
stays true. "Works on macOS" is not a claim this pass can make; "written for macOS, and blocked only
by the absence of a Mac and its C toolchain" is.

## Phase H — documentation consistency + final full-suite (finale)

**Documentation consistency:** the Survey reports **0 errors / 0 warnings** — only three
tolerated-by-design notes remain (Lean `C<n>` tokens, the bare-`D<n>` stage convention, and the
README test-count that can only come from a suite run). No doc drift was introduced across the night;
each phase regenerated the Survey and updated this log, so "update all the .md files" holds by
construction rather than by a last-minute sweep.

**Final full-workspace suite:** `cargo test --workspace` re-run from clean on **both** platforms
after all the night's changes — **124 test binaries `ok` on Windows and 124 on Linux, zero
failures**. The two things most worth re-checking both held: the DL1421 conformance witness (Phase A)
and the hardware-adapter fail-closed fix (Phase E).

## Closing summary — phases A–H

An eight-phase overnight production-readiness sweep, verified on Windows and Linux (macOS
designed-for but unverified — no Mac on this bench). Every phase committed with its evidence; the
Survey stayed at 0/0 throughout and `doctor_cli` was 7/0 every time.

| Phase | Outcome | Commit |
|---|---|---|
| A — full-workspace baseline | fixed 2 conformance gates (DL1421 witness + stale reference) | `e5ae9ec` |
| B — LSP server + VS Code extension | verified live; the historical collision cannot recur | `d66e600` |
| C — CLI + compiler dogfood | clean on both platforms; authority enforced; broken → diagnosed | `46be87e` |
| D — deployability / install | portable archive builds, unpacks + runs; checksums intact | `9af4611` |
| E — security discovery (hw adapter) | **found + fixed** a reply-framing desync (fail-closed) | `ffca9bd` |
| F — Miri (small batches) | 0 UB on the interpretable unsafe; FFI out of reach (stated) | `6a55380` |
| G — macOS honest assessment | designed-for; unverified (C toolchain blocks the cross-check) | `8f9e6c1` |
| H — doc-consistency + final full-suite | 124 test binaries green on Windows AND Linux | this commit |

**The one real defect** was the hardware-adapter reply-framing gap (Phase E): an untrusted adapter
emitting an extra line per command desynced reply attribution off-by-one, silently; it now fails
closed, witnessed against the old code. **Not a containment break** — the envelope guarantee was
confirmed intact.

**Honest residuals carried forward** (the standing shape of the system, not regressions): macOS is
unverified; the same-OS-user threat model still needs a separate account for full isolation (the
DISC-1 / IPC-1 category-7 residuals, empirically bounded in P21); the from-source default-features
install embeds CPython; and the FFI/syscall `unsafe` is defended by tests + isolation contracts + the
OS boundary rather than by Miri. None is new tonight; all are documented where they live.

**Verdict:** DeluluLang builds, checks, runs, packages, deploys, and installs cleanly on Windows and
Linux; its language server and CLI work; its one newly-found defect is fixed; its `unsafe` is either
Miri-clean or honestly out of reach; and its macOS story is designed-for and truthfully labelled
unverified. The final full-workspace suite is green on both platforms.

## Phase I — continued discovery (bonus): broker `rotate-key` persistence (finding + fix)

The A–H sweep above is a complete production-readiness pass. Per the never-ending discovery mandate,
this is one round beyond it — and it found a real security defect.

**Finding — ROTATE-1 (fixed, witnessed).** `delulu broker rotate-key` rotated the lease-MAC key only
in the daemon's MEMORY; it never rewrote `broker.key` on disk. Witnessed against the current binary:
the on-disk key hash was **byte-identical before and after** a rotate. Because `serve_inner` reloads
the key from `broker.key` at startup, a daemon **restart** reloaded the old key and re-validated every
lease token the rotation was supposed to invalidate — while the CLI told the operator they were "now
invalid". For a security rotation (revoking outstanding delegations after a suspected key compromise),
the effect lasted only until the next restart, and daemons restart routinely.

**Fix.** The daemon now generates the new key and **persists it to `broker.key` (0600) before applying
it in memory** — a rotation that cannot be made durable is not applied at all (fail-closed;
`Broker::rotate_key_to` is the in-memory half, the daemon owns the disk half). Witnessed fixed: the
on-disk key now **changes** after a rotate on Windows and Linux. Regression guard:
`rotate_key_is_persisted_so_a_restart_cannot_resurrect_old_tokens`; delulu-broker 144/0 on both
platforms; broker / grants / dead-man CLI green; clippy clean.

Found by continuing to ask what assumption had not been attacked — here, that a documented,
unit-tested *in-memory* behavior was also durable across a restart. It was not.

## Phase J — continued discovery (bonus): foreign-worker isolation (NEGATIVE result — the surface holds)

Red-teamed the process-isolation of foreign (C/Python) workers (`crates/delulu/src/foreign_worker.rs`)
against the current binary. It holds and is honestly scoped — no finding.

- **Non-disclosure of the broker is honestly labelled, not over-claimed.** The worker is spawned with
  no broker address in argv, `DELULU_STATE_DIR` overridden to an isolated broker-less directory, and
  std-handle inheritance cleared — but the module doc states plainly (criterion 7b) that this is
  *"non-disclosure, not kernel enforcement — a same-user process is out of scope for hard blocks"* per
  the §10 threat model. So "malicious same-uid foreign code could find the real broker socket by its
  path" is the DOCUMENTED same-uid limit, not a gap. Test: `worker_command_does_not_disclose_the_broker`.
- **Crash-isolation holds (the headline guarantee).** A worker that dies mid-call — a segfault, a hard
  crash — surfaces as `ForeignErr::WorkerDied` (any write/read failure on the channel maps to it) and
  **the host keeps running**. Verified: foreign-worker integration 3/0, foreign-FFI 9/0.
- **A worker HANG is inherent, not a defect.** The host waits on the worker's response with no read
  timeout — but a fixed timeout would be wrong here: a foreign call is arbitrary user computation, so a
  legitimate long call is indistinguishable from a hang, and an in-process foreign call hangs the host
  thread identically. Isolation claims CRASH-resilience, not HANG-resilience, and cannot bound
  unbounded computation with a fixed deadline. This differs from IPC-1/DEADMAN-1, where broker ops and
  heartbeats DO have a sensible bound and the timeout was the right fix.
- **Kill-on-host-death** is a Job Object (Windows) / `PR_SET_PDEATHSIG` (Linux), consistent with the
  Phase G cfg audit.

Recorded as a NEGATIVE result: the isolation is real, its headline guarantee holds, and its limits
(same-uid, unbounded-computation hang) are documented rather than hidden.
