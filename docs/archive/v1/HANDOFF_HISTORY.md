# HANDOFF.md — the historical sections

**Moved here on 2026-09-27, in V2 phase P6,** when `HANDOFF.md` was rewritten as a current briefing.
Everything below is the text as it stood that day, **verbatim and under its original section numbers**,
so a citation of "`HANDOFF.md` §N" for any of these sections resolves here. Like everything in this
archive it is not maintained, rewritten or deleted: each section describes the day it was last written.
For what is true now, `HANDOFF.md`'s last section names the successor of each.

---

## The old header (the current-state paragraph, the update trail, the 2026-08 campaign notes)

# HANDOFF — DeluluLang

**For:** whoever picks this repository up next, human or AI.
**Written:** 2026-08-07, at the end of the P19 ecosystem campaign.
**Current state (2026-09-27):** V2 phases V2-0, P1, PS-0, PS-A, P2, P4a, P3 and PS-B are complete, each with its CI run read green; PS-B ran — its TLS dependency landed 2026-09-20 (D-V2-30) and **PS-B-02, the network client, on 2026-09-25**: `http.get` now fetches over verified HTTPS through one host-side client that serves L0 and sandboxed guests alike. PS-B-01 (every run budgeted, D-V2-25) followed the same day (`936500f`; its Windows failure was a cold Python start in a TEST driver, measured and fixed), and **PS-B-05, a budget as the tenth authority dimension** (`grants delegate --budget`, inherited, never widened, a lease run held to it; Z3 26 obligations with CI-enforced mutants; D-V2-33). PS-B-05 also found and closed `run --sandbox` silently dropping most of `run`'s flags (`--lease` among them). **PS-B-03 on Windows** followed: a `--sandbox` guest runs as a per-run AppContainer with no capabilities (T14 measured with a control; D-V2-34), and macOS's guest profile refuses the state directory. **PS-B-06, BREAK-GLASS**, then: an operator may require the sandbox on a host, and a signed, single-use, one-program ticket is the only way past it (D-V2-35). **PS-B-03b** then gave a Linux guest a subordinate uid where the host allows user namespaces (measured first on a GitHub runner, both ways; D-V2-37), and **PS-B-04** was decided by measurement on all three CI runners: no batching, the Windows transport rebuilt instead (48.6 → 28.9 µs per effect; D-V2-36). **PS-B closed 2026-09-26** (`7a94684`, CI `36173241488` green on every job). **P4b–e is in progress** (D-NE-6 taken as D-V2-38): `delulu toolchain --json`, `delulu schema` (closed JSON Schemas + validator, every emitter validated), `delulu examples`, the read-only `delulu mcp` server and checked edits (`delulu edit`: refused on a stale hash, by byte range or by Atlas id, re-checked, the authority an edit adds named) and `delulu-survey diff <rev>` (a change's blast radius, entrenched files named; also the MCP `survey_diff` tool) the Guard read-only in the editor with declared-vs-performed rows on hover, `delulu atlas chain` (the V2 chain, snapshotted on the corpus) and the AI usability benchmark (P4-08: harness, controls, replayable record, a Sonnet 5 pilot published as it came out — it found `main` accepting signatures the runtime could not run, now refused) are built — **P4b–e CLOSED 2026-09-26** (`11a7af1`, CI `36261139018` green on every job). **PS-C (the Linux/KVM microVM) is in progress** (D-V2-39; prerequisites in `V2_PS_C_PREREQUISITES.md`): `run --isolation microvm` runs a program at level 2 on Linux x86_64 with KVM — the interpreter as PID 1 of its own 6.18 kernel under Firecracker v1.17.0, no network device, no filesystem device, one vsock channel, the image built from source by `scripts/microvm/build-image.sh` and checked on the copy that boots; criterion 8 is met, restated for the no-NIC guest, with the lifecycle family, all falsified by mutants — on this machine's WSL2 KVM and on a GitHub KVM runner (`ab1bfdc`, CI `36266702402` green on every job, two clean image builds identical). Run as root, the VMM runs under Firecracker's jailer with a reaper in place of the death signal (PS-C-03b, D-V2-40); the L2 red team is recorded and gated (PS-C-06, `V2_PS_C_RED_TEAM.md`). Open: PS-C-05 (a distributed image — the owner's, D-NE-27); then P6. **Resume from `docs/DELULULANG_V2/V2_PHASE_STATUS.md`, then the newest entry in `docs/DELULULANG_V2/V2_LOG.md`.** §3 has the V2 ledger.

**Last updated:** 2026-09-25 — PS-B-02 (the network client) and a check of every earlier V2 phase, which found five things (V2 log, PS-B-02 entry) — among them that the portable download would have shipped with no network client, and that the resource-limit defaults everyone was waiting on had been ruled a week earlier. Before that, 2026-09-17 — **DeluluLang V2 is executing.** The owner approved the 2026 evolution plan the same evening (`docs/design/DeluluLang_V2_Execution_Master_Prompt.md`); the active source of truth is now `docs/DELULULANG_V2/` (start at `V2_README.md`; phase state in `V2_PHASE_STATUS.md`), and the two planning passes that produced the plan were archived under `docs/archive/v1/NEXT_EVOLUTION_2026/` together with sixteen other historical documents (`docs/DELULULANG_V2/V2_DOC_MOVE_MANIFEST.md`). Earlier that day the owner made the testing remote public (§1.1). Before that, 2026-09-14: its first push, and CI green on all three operating systems in one run. Earlier: 2026-08-23 (`docs/REMAINING_WORK.md`), 2026-08-10 (containment + deployment hardening).
**Repository:** `D:\nelan\DeluluLang` — a Rust workspace, 13 crates, **111,437 lines of Rust**
(measured by the Survey on 2026-09-14, not remembered) — and on GitHub, **publicly since 2026-09-17**
(privately from 2026-09-14): `origin` → `https://github.com/jessesuniljs1-collab/delululang-test.git` (§1.1).
**State:** clean tree; `master`, `rc/1.0.0-drill` and the `v1.0.0` tag pushed to `origin`;
**209 commits past `v1.0.0`** at this update (the V2-0 phase commit).

> **If you are starting today, read this first.** Two campaigns have run since this document was
> written, and the second changed what you should assume:
>
> - **2026-08-09** — production-readiness sweep; six defects fixed (see `PRODUCTION_READINESS_2026-08-09.md`).
> - **2026-08-10** — containment + deployment hardening (`PRODUCTION_READINESS_2026-08-10.md`).
>   **Eleven defects fixed**, two of them high: a **dangling symlink escaped filesystem containment**
>   (workspace-deliverable via git), and a **guard seal written the natural way gated nothing while
>   the CLI said `ok`**. Also: strict anchored-root mode was **unusable** until this campaign and now
>   works end to end; `delulu doctor` grew a `security posture` section; and
>   [`docs/DEPLOYMENT.md`](../../../docs/DEPLOYMENT.md) now says what a deployment actually protects.
>
> **Current status: PRODUCTION READY WITH DOCUMENTED DEPLOYMENT REQUIREMENTS** — Windows and Linux,
> in the Tier-2 deployment of `DEPLOYMENT.md`. **macOS is now verified by CI (green on its Apple Silicon
> runner, 2026-09-14), but is not covered by this verdict: the Tier-2 cross-account boundary was tested
> with a real second UID on Linux only.**

Read §1 and §2 before touching anything. The rest is reference.

---

## §1.1's run rows — the first six CI runs

| | |
| --- | --- |
| **The first run** | Run `34830053479` (2026-09-14), read with `gh`: **3 passed** — clippy, the editor build and the formal models, each on a runner for the first time — **1 skipped, 11 failed**, and every failure accounted for. Two gates had never been able to pass. **All six Miri jobs** ran a bare `cargo miri` under the stable toolchain `rust-toolchain.toml` pins (Miri is nightly-only; the job now says `+nightly`). **`cargo deny`** found two advisories published after its last clean run (2026-08-07) against wasmtime 47.0.3 — RUSTSEC-2026-0268 and RUSTSEC-2026-0269, both in WASI functionality DeluluLang never uses (no `wasmtime-wasi`, no WASI calls), both closed by the patch release **47.0.4**. Two artifacts had been recorded from this machine's disk rather than from git: the **core-invariance snapshot** counted carriage returns that 58 CRLF working-tree files had and the committed LF files do not, and the **Survey** mapped a gitignored `.vsix`. **arm64** ran all 124 test binaries and failed only on those two. **macOS** stopped building `libffi-sys`'s bundled libffi, whose aarch64 assembly current Apple clang rejects, before any DeluluLang code ran; it now links macOS's own libffi, untested until the next run. Full record: `docs/design/CROSS_PLATFORM_VERIFICATION.md` §9. |
| **The second run** | Run `34836508713`: **macOS built the whole workspace and passed 1,654 of 1,655 tests — the first DeluluLang code ever to run on a Mac.** arm64, clippy, `cargo deny`, the editor, the formal models, and Miri on `delulu-atlas`, `delulu-diag` and the FFI decoder passed. The rest failed where a test met the runner (a speedup criterion on 2-vCPU machines; a PowerShell driver too slow to start inside the adapter's deliberate 2000 ms budget), where it found a real defect (a socket-path refusal that never left the detached daemon; a fresh state directory the broker could not start in), or where it ran out of time (Miri on three crates with no `unsafe`, now nightly/manual as `miri-slow`). All fixed in the next commit; the record is in `docs/design/CROSS_PLATFORM_VERIFICATION.md` §9. |
| **The third run** | Run `34841317790`, on `28e10e6`: **green everywhere but one Windows test.** macOS end to end — 1,657 tests, the CLI sweep 27/27, the fuzz campaign's 50,000 programs with no trace escaping its row — and the same on Linux x86-64 and arm64, plus clippy, `cargo deny`, the editor, the formal models and Miri on atlas, diag and the FFI decoder. Windows passed 1,646 of 1,647: `adapter::tests::a_garbled_reading_is_an_error_never_a_none_and_never_a_number` failed on the runner (it passes locally, and passed there in run 2): its PowerShell fake driver started too slowly for the adapter's 2000 ms budget. The drivers are Python now — run 4 confirmed it. |
| **The fourth run** | Run `34844151767`, on `ef9cb49`: **Windows green end to end, its first complete pass on CI** — 125 test binaries, 1,647 tests passed and 0 failed; conformance 330 of 330; the reference in sync; the CLI sweep; the fuzz campaign with no trace escaping its row — and Linux x86-64 and arm64 green again, with clippy, `cargo deny`, the editor, the formal models and Miri on atlas, diag and the FFI decoder. **macOS failed one test.** `device::tests::a_beaten_lease_is_never_revoked` beats a 120 ms lease every 20 ms, and a command found it revoked for a missed heartbeat: the runner had left the test's thread unscheduled for more than 100 ms, so the lease really had gone unbeaten past its heartbeat and the watchdog was right. Reproduced here by starving the thread — the unmodified test failed 5 of 8 starved runs with CI's message word for word, and passed 25 of 25 idle. The test now judges each revocation against the gap its thread actually left and restarts the drive when the gap explains it; a watchdog made to fire 200 ms early still fails it, and passes every other device test. Run 5 passed with it on macOS, Linux x86-64 and arm64. Record: `docs/design/CROSS_PLATFORM_VERIFICATION.md` §9. |
| **The fifth run** | Run `34849980129`, on `010c36c` (2026-09-14): **the first run with no failures — every job green, all three operating systems in one run.** macOS, Linux x86-64 and Linux arm64 each passed 125 test binaries, 1,657 tests, 0 failed; Windows 125 binaries, 1,647 tests, 0 failed (it compiles ten platform-gated tests fewer). On the three x86 test runners every later step passed too — `fmt --check`, the Python-less build, conformance 330 of 330, the reference in sync, the CLI sweep 27 of 27, and the fuzz campaign with no trace escaping its row. clippy, `cargo deny`, the editor, the formal models and Miri on atlas, diag and the FFI decoder passed; `heavy-gates` and `miri-slow` were skipped, by design. The dead-man test run 4 failed passed on every platform, and `docs/REMAINING_WORK.md` 7.2 is closed. |
| **The sixth run** | Run `35147900141` (2026-09-17), started by hand on `51aab51` with `everything`, because that push's commit message quoted GitHub's skip token and so started no run. **The first run on the public runners:** every push job green — 1,657 tests on macOS, Linux x64 and arm64, 1,647 on Windows, 0 failed; the sweep, the fuzz campaign, conformance and the reference as before — and faster (Windows 10.4 min, against 15–28). **`heavy-gates` passed its first run ever.** The ping-pong speedup criterion, asserted on CI for the first time, passed. `miri-slow` then ran out its 240-minute budget on all three crates, with no undefined behaviour in what it reached (broker 124 of 150 tests, syntax 104 of 131, check 30 of 237), so the run as a whole ended *cancelled*; `docs/REMAINING_WORK.md` 5.6 names the tests in flight. |

---

## 3. What has been built — the ledger

**Stages 1–10 are BUILT.** v1.0.0 was tagged locally 2026-07-20 (`198bf44`). Since then the project
has been in continuous adversarial review rather than feature work.

| Stage | What it added |
| --- | --- |
| 1 | Lexer, AST, error-recovering parser, the core type + effect checker, the tree-walking interpreter |
| 2 | Packages, `delulu.toml`, path dependencies, the lockfile, the **semver-authority law** (authority may never widen silently across versions — `DL1003`) |
| 3 | The WASM backend (a *fragment*, not the whole language) under an embedded deny-by-default Wasmtime host |
| 4 | Foreign function interface, embedded Python, cross-engine parity |
| 5 | **Custody**: the broker, the `⊑` attenuation lattice, the grant tree, revocation epochs, the hash-chained audit log, lease tokens — and **the Guard** |
| 6 | **Plugins** (`.dpx`), two classes, signing, verification — the artifact and its checks. The in-language load surface WAS a runtime stub (`prim.rs`, `DL0703`); **V2 phase P2 built it on 2026-09-20**: `root.plugin_host()` under `--grant plugin=<path>`, `load` running the whole Stage-6 sequence, `p.get`/`p.unload`, a `[plugins] allow` hash ceiling in the manifest, and revocation killing a retained callable per call (R-6c). A Verified plugin executes its **DIR**, which the container format itself calls canonical (the `delulu:wasm` section is a cache). Still refused: `Declassify`/`ForeignCall` in a plugin grant, and `Contained` on Windows |
| 7 | **Actors** — message passing, per-sender-pair FIFO, bounded mailboxes, quiescence |
| 8 | **The language server** (`delulu lsp`, LSP 3.17) and the editor surface |
| 9 | The registry, publishing, deployment planning, the measurement program |
| 10 | "Industrial": fleets, devices, federation, hardware adapters, autonomy |

**Post-1.0 campaigns** (each has its own document — see §5):

- **P16 — hardening.** Found the worst defect in the project's history: *the effect row was escapable*
  — `delulu authority` reported "provably pure" for a program that printed at run time.
- **P17 — proof.** Every subsystem placed in exactly one of seven evidence categories. Lean 4 entered
  the picture (`C88` mechanized, no axioms); the broker was model-checked in TLA+; the nine-dimension
  authority order was proved in Z3.
- **P18 — eliminating uncertainty.** `⊑` was shown to be a *preorder*, not a partial order, because
  `path::resolve` is not injective — mathematics, not a bug. Fixed by canonicalizing at the custody
  boundary. Miri completed for the first time.
- **P19 — ecosystem** (2026-08-07). See §8 for what it found.
- **P20 — zero-trust red team & evidence honesty** (2026-08-08). Fixed five documents that *denied* a
  machine-checked proof that exists (Lean re-runs in 35 s); added the **evidence gate**
  (`crates/delulu/tests/evidence_claims.rs`) so prose cannot outlive fact. Red team: filesystem
  containment escapes through a **hardlink** (P20-R1 — documented boundary, not workspace-deliverable,
  pinned by a test); a deeply nested **type** was a checker DoS and, deeper, a parser crash
  (P20-R3 → **`DL0211`** caps type nesting at 128); an adversarial multi-agent authority test
  (Sonnet 5 + Haiku 4.5) confirmed Authority, effect rows and secret-flow hold under attack.
- **Tier 1 — bounded iteration** (2026-08-08, **owner-directed**). Activated four reserved keywords:
  **`for x in xs { … }`**, **`break`**, **`continue`** — the last two also make `while` breakable.
  New diagnostics `DL0411` (non-list iterable) and `DL0412` (break/continue outside a loop). Built
  through every layer with the same effect-transparency and reference-capability soundness as `while`;
  the WASM backend refuses it (`DL1201` interpreter fallback). The other reserved words stay reserved
  with written reasons (`async`/`await` rejected by Constitution decision 12; `trait`/`impl`/`where`
  undesigned; `ref`/`box`/`trn` soundness-critical; `pure` redundant with `!{}`).
- **P21 — cross-account boundary** (2026-08-08). Tested the "separate OS account" recommendation with
  a real second UID: it **holds** on a POSIX filesystem and is **absent on 9p**, where `chmod` is a
  silent no-op. Both `keygen` and the broker now refuse to write a secret onto such a filesystem.
- **Production-readiness sweep** (2026-08-09, `PRODUCTION_READINESS_2026-08-09.md`). Six defects,
  each witnessed against pre-fix code — including a key rotation that a restart undid, a revoked
  federation certificate that a restart resurrected, and two unbounded parser recursions. All four
  recursive-descent nesting classes are now bounded (`DL0210`/`DL0211`/`DL0212`/`DL0213`).
- **Containment + deployment hardening** (2026-08-10, `PRODUCTION_READINESS_2026-08-10.md`). **Eleven
  defects.** Two high: a **dangling symlink escaped filesystem containment** (`canonicalize` fails
  identically for "absent name" and "broken link", so the link's name was re-appended as a plain
  component and the write followed it out of the grant — workspace-deliverable via git, unlike the
  hardlink boundary), and a **guard seal written the natural, relative way gated nothing while the CLI
  answered `ok`**. Also: a dependency's authority pin was escapable by spelling; strict root mode
  failed *open* on a corrupt policy; a valid program aborted the host during value teardown; and
  strict anchored-root mode — the DISC-1 mitigation — turned out to be **unusable** until three
  defects in its own path were fixed. `delulu doctor` gained a `security posture` section, and
  [`docs/DEPLOYMENT.md`](../../../docs/DEPLOYMENT.md) now states what a deployment actually protects.

  **The through-line worth carrying forward:** four of them were the same defect — a security decision
  made on an **unnormalized or unresolved representation**, walked past by a different spelling of the
  same thing. It became the search key, and it is what found the Guard one *after* the campaign's own
  final phase had closed. Ask it of any string compared to decide a security outcome: **what else
  spells the same thing?**

### DeluluLang V2 (2026-09-17 onwards) — the phase ledger

Executed in the owner's order from `docs/DELULULANG_V2/` (the one active source of truth). A phase is
complete only when its commit is pushed and its CI run has been read green. Details, including what
each phase deliberately did NOT build and why, are in `docs/DELULULANG_V2/V2_LOG.md`.

| Phase | Commit | CI | What it made true |
| --- | --- | --- | --- |
| V2-0 workspace + documentation migration | `e48f9c3` | `35258166713` | the V2 folder; 32 historical files archived under `docs/archive/v1/`; the Survey's archive-mirror rule |
| P1 machine-contract truth | `d8dbc24` | `35299533343` | the `--json` envelope, unknown flags refused, `test --test-authority` (D-NE-17) |
| PS-0 sandbox truth, probes, cheap hardenings | `32ba712` | `35376528795` | `run --report-out`, `sandbox probe`, doctor's sandbox section, hostile Windows spellings refused, the foreign-call deadline, `net.special=` |
| PS-A the L1 process sandbox | `31643fe` | `35492572666` | `run --sandbox`: a guest holding no authority, the host performing every effect, an OS jail on all three systems |
| P2 real plugin loading | `5f52eb8` | `35505790785` | NE-01 closed: a running program loads a `.dpx` |
| P4a the Agent Skill | `2ad6e1d` | `35520103322` | `skills/delulu/SKILL.md` and `delulu skill`, derived from `--help` so they cannot drift |
| P3 the standard library | `3ab0cc9` | `35522886721` | `List` 15, `Str` 10, `Map[K, V]` 8; coverage 353/353 |
| PS-B (in progress) | `d0ae0f9`, `d59201a`, `936500f`, then PS-B-05 | `35524134404`, `36114298041`, `36117814329` (Windows re-run green; the first attempt's failure was a cold test-driver start, settled by experiment) | the TLS dependency measured before use; **the network client** — verified HTTPS, host-side, resolve-once-and-pin, special-use refused unless `net.special=`; **every run budgeted** — 1 GiB / 5 min unless `--limits` says otherwise, stopped and reported on breach; **the budget as an authority dimension** — delegated, inherited, never widened, a lease run held to it, proved in Z3 (26 obligations) |

Still to come, in order: the rest of PS-B, P4b–e, PS-C (the Linux/KVM microVM), P6, P5, P7, PS-D, P8.

---

## 5. Which document to read, and what each is for

### Start here

| File | Read it when |
| --- | --- |
| **`README.md`** | You want the project in one page, honestly — including what it does *not* have. |
| **`docs/GETTING_STARTED.md`** | You want to write DeluluLang. Install → first program → real programs → your editor. |
| **`docs/for-agents.md`** | **You are an AI agent driving the toolchain.** Exit codes, `--json` envelopes, how to batch `check`, and what to ask the LSP instead of shelling out. |
| **`docs/QUESTIONS.md`** | Someone asks "can this be broken?" It answers the hard questions with evidence, and enumerates the known leaks rather than implying there are none. |
| **`docs/DEPLOYMENT.md`** | **You are about to run code you did not write.** What a deployment actually protects, the three tiers (single-user legacy / strict anchored roots with an offline anchor / separate OS account), the exact commands, how to verify each with `delulu doctor`, per-platform status, an explicit list of what is NOT protected, and why strict mode is not yet the default. |
| **`docs/REMAINING_WORK.md`** | **You are deciding what to build next, or wondering whether a feature exists.** Every gap between what a document in this repository describes and what the code does, each row re-verified against the current binary. Also carries §1: the places where the *documents* were stale and the code had moved ahead. |
| **`docs/DELULULANG_V2/V2_README.md`** | **You are working on DeluluLang V2 — the active path since 2026-09-17.** The one active source of truth: the master plan, the roadmap with every phase, the phase status, the execution and decision logs, the security model and the AI-native design. The V1 planning passes behind it are archived under `docs/archive/v1/NEXT_EVOLUTION_2026/`. |
| **`docs/survey/SURVEY.md`** | You are about to change the compiler and want the blast radius. |

### Reference

| File | For |
| --- | --- |
| `docs/REPOSITORY_STRUCTURE.md` | What every directory and significant file is, annotated with *why* it is shaped that way |
| `docs/MATHEMATICS.md` | The formal claims and, for each, which of the seven evidence categories it sits in. **A claim with no category is a claim to be deleted or demoted.** |
| `docs/editors.md` | The language server, per-editor setup, and the editor surface's security history |
| `docs/reference/` | `cli.md`, `diagnostics.md`, `grammar.md`, `tokens.md`, `primitives.md`, the 16 `semantics-5-*.md` chapters, `audit-rules.md`, `coverage.md` |

**If you came to work on the compiler specifically**, read these four in this order:

| File | For |
| --- | --- |
| `docs/design/STAGE1_SPECIFICATION.md` **§9** | *Compiler architecture* — why Rust, and the crate-by-crate pipeline |
| `docs/design/STAGE1_SPECIFICATION.md` **§3** (+ each later stage's additions) | The **normative grammar**. `docs/reference/grammar.md` indexes the productions and names where each is *defined* |
| `docs/release/CHECKPOINT-1.0.md` **§3** | "The compiler" in one page: the soundness core, the code registry, and the refuse-rather-than-guess rule |
| `docs/design/SOUNDNESS_AUDIT.md` | Where the soundness argument **is and is not** complete — findings F-1…F-6 are rejection tests in `crates/delulu-check/tests/laundering.rs` |

Then ask the Survey for the blast radius before you touch anything:
`cargo run -p delulu-survey -- impact mod:crates/delulu-check/src/check.rs`.
| `docs/book/THE_DELULULANG_BOOK.md` | The tutorial. Its samples are conformance-tested — a sample that stops compiling fails the build. |
| `docs/lang/` | Localized human prose (`en-US`, `hi-IN`, `ja-JP`, `de-DE`, `fr-FR`, `es-ES`, `ar-SA`, plus `delulu-slang`). **Codes and JSON never localize.** |

### Design and specification (`docs/design/`)

| File | For |
| --- | --- |
| `CONSTITUTION.md` | The project's own rules. §8.4 forbids editor-specific server features, and that is load-bearing. |
| `LANGUAGE_SPECIFICATION.md`, and `STAGE1_SPECIFICATION.md` through `STAGE10_SPECIFICATION.md` | What each stage is contractually required to do |
| `STAGE*_BUILD_ORDER.md` | The rulings (`D<n>`) that authorized each change. **A bare `D<n>` means the latest stage's numbering.** |
| `AUTHORITY_GUARD_CAPSTONE.md` | Authority and the Guard, together, as one argument |
| `STAGE5_GUARD_ADDENDUM.md` | The Guard's design, including what was adopted from and rejected of `dcg` |
| `STAGE6_PLUGINS_GUIDE.md`, `STAGE7_ACTORS_GUIDE.md` | Plugins and actors, for users |
| `SOUNDNESS_AUDIT.md` | Where the type system's soundness argument is and is not complete |
| `CROSS_PLATFORM_VERIFICATION.md` | **Which platforms have actually been executed**, and which are merely prepared |
| `HARDENING_CAMPAIGN.md` | P16 findings (`C<n>`) — the "what is known to be wrong" list |
| `PROOF_CAMPAIGN.md` | P17/P18 — evidence categories, proof boundaries, and the Miri story |
| `P19_ECOSYSTEM_REVIEW.md` | **The most recent review.** Seven personas, every claim tied to something executed |
| `STABILITY.md`, `VERSION_COEVOLUTION.md` | What may change and when |

### Release and governance

`docs/release/CHECKPOINT-1.0.md` is the **single best status page**: architecture, testing, and
twenty known limitations named without softening. `SUPPORT_MATRIX.md`, `SBOM-1.0.json`,
`PROVENANCE-1.0.json`, `CHECKLIST-1.0.md` sit beside it. Governance lives in `GOVERNANCE.md`,
`CONTRIBUTING.md`, `SECURITY.md`, `TRADEMARK.md`, `INSTALL.md`, `CHANGELOG.md` at the root.

---

## 6. The features, explained

### Delulu Authority

The whole system, and the thing to be most careful with.

- **Effects** — a fixed core set of **ten**: `Read`, `Write`, `Net`, `Async`, `Clock`, `Rand`,
  `Load`, `Actuate`, `ForeignCall`, `Declassify` — carried in a function's row and inferred
  transitively. A module may declare its own with `effect Name`; those are `Effect::User(String)`,
  which is the *variant that carries a user effect*, **not** an eleventh core effect. This line said
  "eleven" and counted `User` as one until 2026-08-23; `CORE_EFFECT_NAMES` and
  `Effect::core_from_name` both hold exactly ten.
- **Capability scopes** — an effect is not enough. A `Cap[FsRead]` is scoped to *paths*; `Cap[Net]` to
  *hosts*; devices to named dimensions with numeric envelopes. `Scopes` carries seven set-valued
  dimensions (`fs_read`, `fs_write`, `net`, `secrets`, `declassify`, `foreign_c`, `foreign_python`)
  plus `device`, which is a **map** from device path to envelope rather than a set — one device has
  exactly one envelope per grant, because two would be an ambiguity the enforcement path must resolve,
  and resolving it silently is how a widening gets in.
- **`child ⊑ parent` is one conjunction over ten dimensions** — the effect set, those seven scopes,
  `device`, and (since PS-B-05) `budget`. A conjunction, so a widening in *any single* dimension fails
  the whole check. Proved in Z3 (26 obligations, with three mutants CI requires to fail).
- **`⊑` (attenuation)** — "this authority is contained in that one". A child grant may only ever be
  narrower than its parent. **It is a preorder on the representation and a partial order on the
  quotient**, which is why scopes are stored canonically (P18, findings F1–F3). Canonical form is an
  **antichain**: `{./data, ./data/sub}` collapses to `{./data}`.
- **The broker** (`delulu-broker`) holds the grant tree, revocation epochs, lease tokens, and a
  hash-chained audit log with an external anchor. It is transport-free by design; the daemon and wire
  live in the `delulu` crate.
- **Federation** — brokers can delegate across machines with signed certificates. Two real
  vulnerabilities were found and closed here: certificate replay could undo a revocation, and a child
  with `ttl:None` could outlive its parent's expired uplink lease.

`delulu authority` computes it. `delulu why <Effect>` explains it at function granularity.
`delulu authority --diff` compares two lockfile states and reports widening.

### Delulu Guard

The **policy layer above authority**: how a principal supervises agents that hold authority.

- **Three tiers per rule** — `warn`, `guarded` (a person can approve it at run time), `sealed` (not
  runtime-approvable at all).
- **Permits, not bearer codes.** Approval mints a broker-held permit scoped to a grant, with a TTL and
  a use count. It is *not* a code that whoever runs a CLI verb can claim.
- **Owner codes** — admin verbs require one, printed once and never written down.
- **E-stop** — the emergency stop, and the dead-man timer for devices.

`delulu guard status | policy | request | pending | approve | deny | permits`.

The Guard's idea came from [`dcg`](https://github.com/Dicklesworthstone/destructive_command_guard) by
Jeffrey Emanuel — its *policy* ideas, not its implementation. `dcg` guards untyped shell strings and
needs heavy pattern analysis; DeluluLang's effects are typed and enforced structurally, so only the
supervision model transferred. The credit is recorded in `STAGE5_GUARD_ADDENDUM.md` and stays.

### Plugins

`.dpx` archives, in **two declared classes** — declared, never inferred:

- **`Plugin[Verified]`** ships DIR (the typed IR) and is **re-checked at load** by replaying the
  compiler's own `resolve` + `check_module`. Its exports carry re-verified effect rows and it runs on
  the host engine.
- **`Plugin[Contained]`** ships opaque WASM and is **confined at the module boundary** — imports are
  the only way out.

A `.dpx` claiming Verified whose DIR fails any check is refused (`DL1504`) and **never falls back to
Contained**. `delulu plugin build | inspect | verify`.

**What is NOT built: loading one from a running program.** `root.plugin_host()` is a stub —
`prim.rs:366` faults `DL0703` "plugin hosting is not available in the Stage-1 runtime" — and no
`--grant` spelling enables it (`plugin`, `plugin=…`, `plugin.load`, `load`, `plugins` are each
"unknown grant"). Such a program still **checks clean** and `delulu authority` still reports the
plugin and its load site, so nothing but `run` tells you. The load sequence's own mechanics are
witnessed by the Stage-6e library tests and by `plugin verify`, whose verdicts are identical to a
real load; what is missing is the wiring from a program to them. `REMAINING_WORK.md` row 4.11, V2
phase **P2** (`docs/DELULULANG_V2/V2_IMPLEMENTATION_ROADMAP.md`).

### Actors

Message passing, per-sender-pair FIFO (falling out of `mpsc` rather than asserted), bounded mailboxes,
deterministic quiescence. `actors.rs` contains **zero `unsafe`** — the topology earns it: actors never
migrate, `Value` is `Rc`-based and deliberately not `Send`, and messages cross only as an owned
`Send`-by-construction representation. **Deadlock, livelock and starvation are not prevented**; race
freedom is not liveness.

### The language server and editor

One server, `delulu lsp`, LSP 3.17 over stdio, **analysis only** — it never runs code, never loads a
plugin, and holds no broker lease. Diagnostics, hover with the effect row, completion, signature help,
definition, references, rename, document and workspace symbols, semantic tokens, inlay hints showing
*inferred* rows, quick fixes from typed repairs, and formatting that calls the same `format_source`
the CLI does (byte-identical, enforced by a test).

The VS Code extension adds an authority atlas webview, snippets carrying effect rows, tasks, a problem
matcher, a status bar, and an icon. **`delulu.serverPath` is machine-scoped on purpose** — see §8.

### Other surfaces

- **Registry** — sparse index, publish API, **server-side authority recomputation** (the server does
  not trust the client's claim).
- **Morphs** — surface keyword skins, human languages or AI-compact profiles. The *program* is
  unchanged; codes and JSON never pass through a morph.
- **Devices and fleets** — simulated by default (`--broker-profile sim`, deterministic under
  `--seed`), with a real hardware adapter behind an operator-supplied subprocess.
- **Measurement** — `measurements/` is a 15-study reproducible evidence program, not a benchmark suite.

---

## 8. Problems — what is open, and why

> **The complete inventory is [`docs/REMAINING_WORK.md`](../../../docs/REMAINING_WORK.md)** (2026-08-23) —
> sixty items across the language, backends, containment, proof, tooling and platform, each
> verified against the binary and each with what closing it takes. This section stays because it is
> the *briefing* version: the things you must know before touching anything. That file is the list
> you plan from. It also carries a §1 this section cannot: **eleven** places where the *documents*
> disagreed with the code — ten corrected, one (Constitution §5.15) left for the project lead
> because the file is entrenched.

### Blocked on the owner (not on engineering)

| | |
| --- | --- |
| **rustfmt** | *(new, 2026-08-07)* There is no `rustfmt.toml`; rustfmt's default style disagrees with this hand-written codebase **3,890** times (2,766 even at `max_width=120`). The choices are: reformat the whole tree (now ~111,000 lines) in one unreviewable commit, keep a permanently red CI step, or say the project has not adopted rustfmt. I removed the step and wrote the reason into `ci.yml`, because a permanently red job teaches people that red is normal. **Adopting rustfmt rewrites every file, and this project's comments carry much of its value — it is the owner's call.** Formatting is currently unenforced. |
| **CODE_OF_CONDUCT.md** | Absent. A policy commitment, not a cleanup task. |
| **The 2026 evolution plan → DeluluLang V2** | *(updated 2026-09-17, evening)* Approved by the owner and executing (`docs/DELULULANG_V2/`). No longer blocked as a whole; the decisions still reserved to the owner are listed in `docs/DELULULANG_V2/V2_MASTER_PLAN.md` §7 and are asked at the start of the phase that needs each. |

**Settled, and no longer blocking — the licence.** `LICENSE` (Apache-2.0), `NOTICE`, `TRADEMARK.md`
and `GOVERNANCE.md` shipped at commit `42702e2` under ruling **D27**, closing hardening finding C9;
before that, default copyright meant nobody could legally use DeluluLang at all. This row sat in the
table above as an open owner blocker long after the decision had landed, which is why it is called
out here rather than silently deleted. **Changing** the licence stays owner-reserved. Deciding it is
done.

### Cannot be done on this machine

| | Why |
| --- | --- |
| ~~**macOS has never been executed.**~~ **Closed 2026-09-14: green on CI.** | The project has no Apple hardware of its own. Re-measured per crate 2026-08-10 with `cargo check --target aarch64-apple-darwin`: **`delulu-diag`, `delulu-syntax`, `delulu-measure` and `delulu-survey` type-check**; every remaining member stops inside a **third-party C build script** (`blake3`, `zstd-sys`, `libffi-sys`) for want of an Apple cross-toolchain, *before* the compiler reaches DeluluLang code. So **no DeluluLang source was shown to fail — and most was not shown to compile either.** Both halves are the claim. Exactly one line in the tree branches on macOS (`broker_transport.rs`, `SUN_PATH_MAX` 104 vs 108); the rest is `cfg(unix)`, which Linux exercises. A matrix entry naming `macos-latest` is a plan, not a result. **`cargo test --workspace` on a Mac was expected to close this, and on CI's macOS runner it did:** the first run stopped building `libffi-sys` (now linked to macOS's own libffi), the second passed 1,654 of 1,655, and the third was green end to end (§1.1). What CI cannot close: a developer's Mac. |
| ~~**CI has never executed.**~~ **Closed 2026-09-14: run 5 was green on every job — all three operating systems in one run.** | Until 2026-09-14 the repository was never pushed, so CI never ran. The first push, to the private testing remote (§1.1), activated `ci.yml`, and the first run went red. This row used to say *"every command in it has been run by hand"*; the Miri command, as written, could never run in this tree (§1.1, *The first run*), so that was not true of it. All five runs are transcribed in `docs/design/CROSS_PLATFORM_VERIFICATION.md` §9: the first four found what they found (§1.1 has each), and the fifth had no failures. *"Written"*, *"triggered"* and *"green"* are three different claims, and the third took five runs to earn. |
| **The Dockerfile has never been built.** | `Dockerfile` and `.devcontainer/devcontainer.json` were written 2026-08-07. Docker CLI 29.5.2 is installed; **the daemon was not running**. Dockerfiles fail for boring reasons that are invisible by reading. Treat the first `docker build` as an experiment. |

### Known technical limits, deliberately not softened

- **No principal types.** Inference is order-dependent; swapping two parameters can change what is
  inferred. (Finding F4.)
- **`Secret.verify` declassifies without requiring `Cap[Declassify]`** — visible in the authority
  report, but not impossible. There is **no implicit-flow tracking**; this is not noninterference.
  (Finding IF-1.)
- **The audit anchor is not tamper-proof.** It catches truncation and naive tampering. An attacker who
  rewrites the log *and* the anchor is not caught; that needs an external witness.
- **The clock ratchet gives monotonicity, not accuracy.** A rewind can no longer resurrect expired
  authority, but it still distorts measured intervals.
- **The WASM backend is a fragment.** The interpreter is the language.
- **The optimizer in spec §2.1 is not implemented**, and there is no native backend.
- **The microVM isolation layer is not built** — and it is a *specified defence layer*, not a
  performance tier. Constitution §5.14 names four layers; `crates/delulu/src/microvm.rs` is 58 lines
  and `probe()` returns `Err` on **every** path, including a fully-provisioned Linux+KVM host. The
  read-only rootfs, virtio-fs scope mounts, egress proxy and vsock broker proxy are unwritten, so
  `--isolation microvm` refuses with `DL1408` everywhere. Nothing weaker ever launches under the
  name — the right refusal — but §5.15 guarantee 5 rests today on the **WASM half only**.
- **The standard library is four list methods** — `len`, `get`, `push`, `map`. No `filter`, `fold`,
  `sort`, `contains`; no `Map`/`Dict`/`Set` among the 16 prelude types; 15 prelude builtins.
  `check.rs::is_higher_order_method` still names `List.filter`, which answers `DL0405`. Nobody ruled
  on this; it is how far the prelude got.
- **CLI-string localization has zero registered strings.** `delulu_diag::catalog::CLI_STRINGS` is an
  empty array, so no CLI prose is localizable in any locale, and every `[cli.*]` key the
  localization guide documents would get `DL1704` and fall back. Diagnostics *do* localize (8 codes
  in the shipped `delulu-slang` catalog, out of 154).
- **Type inference is exponential on a small class of programs** — reproduced 2026-08-23 at
  **10.2 s from 29 lines** (`type Pair[L, R]`, depth 22, doubling per level). No fuel bound, no
  `--max-type-size`, no timeout, and `delulu check` is the agent hot loop. A bound is
  language-visible, so it is an RFC.
- **Deadlock, livelock, starvation and mailbox exhaustion are not prevented.**
- **Multi-tenancy is not provided.** Separate OS accounts are required.
- **`pyo3` stays at 0.25** with two CVEs, ignored on *reachability* — and that argument is now a test
  (`governance.rs::the_ignored_pyo3_advisories_are_still_unreachable`). The upgrade to 0.29 removes
  `Python::with_gil` and is deferred **deliberately**, because GIL handling is exactly where a hasty
  migration introduces undefined behaviour.
- **Miri cannot reach the FFI.** It cannot execute `dlopen` or Windows API calls, so 37 `unsafe` sites
  in `delulu`'s Windows transport stay uninterpreted. This is an explicit assumption, not a to-do.
- **Certification is NONE.** No safety standard, no external audit, no third-party review.
- **Nothing is distributed.** No registry entry, no release binary, and no final public repository —
  the public testing repository of §1.1 is not a release.
- **No physical device has ever been commanded.** Every demonstration drives the simulator.
- **The RFC 0001 governance debt** — a core authority change shipped without its comment period. Open,
  recorded, and never to be restated as compliance.

### What P19 found on 2026-08-07 (read `P19_ECOSYSTEM_REVIEW.md`)

Three defects a green test suite could not see:

1. **The VS Code extension had no working language server**, since P18, in every workspace. It
   registered `delulu.authority`, which the language client *also* registers on the server's behalf;
   `registerCommand` threw inside `client.start()` and killed initialization. The test that should
   have caught it **demanded the bug**, by modelling lens commands and protocol commands as one list.
2. **A cloned repository could choose which binary the extension launched.** `delulu.serverPath` had
   VS Code's default `window` scope, writable by a workspace's own `.vscode/settings.json`. Witnessed:
   the unfixed build ran a planted executable **7 seconds** after the folder opened; the fixed build
   never ran it. **Never add "helpfully find the binary in the workspace" — that convenience is the
   attack.**
3. **Miri was pointed at four crates containing zero `unsafe`**, skipping the two holding 50 of the 54
   sites. Choosing where to point a checker is a bigger decision than how to configure it.

Plus: the CI `lints` job could never have passed; two acceptance-criteria gates (`fmt_laws_100k_gate`,
`criterion3_latency_150ms_on_10kloc_release`) had never been run by anything; and `CHECKPOINT-1.0.md`
still claimed four reachable advisories after the wasmtime 27→47 upgrade had closed them.

### Operational traps that cost real time here

- **Editing the tree while the suite runs** makes the Survey stale and fails three `doctor_cli` tests.
  It looks like a product defect and is not. Regenerate, *then* measure.
- **`cmd | head` gives you `head`'s exit code.** This has turned a refusal into an apparent success
  more than once. Redirect to a file and read `$?`.
- **`pgrep -f "foo"` matches the watching shell's own command line**, so a wait-loop can deadlock
  against itself. Two background jobs died this way in one night.
- **A scan that cannot tell a mention from a use will find its own explanation** — a test searching for
  `pyo3` flagged its own comments.
- **Windows locks a running executable**, so `cargo install` fails with `os error 5` while VS Code is
  running a `delulu.exe` from `target/`. Kill the editor first.
- **PowerShell 5.1 `*>` writes UTF-16LE**; `iconv` before grepping.
- **WSL `nohup setsid` detached jobs do not survive.** Use the harness's background mechanism.

---

## 9. Current numbers

### Measured 2026-08-10, re-verified 2026-08-23 on an untouched tree (current)

The 2026-08-23 run was taken after the documentation pass that produced `REMAINING_WORK.md` and
returned **exactly** the figures below — 124 binaries, 1,645 passed, 0 failed, 4 ignored, cargo's
own exit code 0. That is the point of quoting it: a pass that touched thirteen files and one code
comment moved nothing.

| | Windows | Linux |
| --- | --- | --- |
| test binaries | 124 | 124 |
| tests passing | **1,645** | **1,654** |
| failures | 0 | 0 |
| clippy findings | clean | **0** |

Also verified by execution on this pass: every shipped example checks clean on both platforms · the
**LSP answers a real `initialize` / `didOpen` / `hover` / `shutdown` sequence over stdio** with genuine
`DL` diagnostics (verified by speaking the protocol to the binary, not by trusting the suite — P19's
lesson was that the suite was green while the extension had no server) · VS Code extension **17/17**
and the `.vsix` verifies · `delulu doctor` passes every check · Survey 0 errors / 0 warnings.

**Quote cargo's own exit code, never a pipeline's.** `cargo test … | tail` reports the *pipe's* status,
so a failing suite reads as exit 0 — that is how a red core-invariance gate survived a whole campaign
being described as green (finding CORE-SNAPSHOT-1).

### Measured 2026-08-07, on an untouched tree (kept — the delta below is explained against it)

| | Windows | Linux |
| --- | --- | --- |
| test suites | 122 | 122 |
| tests passing | **1,581** | **1,587** |
| failures | 0 | 0 |

The difference is exactly **6**, and it is a **set** difference rather than a gap. (On the 2026-08-10 run the delta is **9**; the mechanism is the same — platform-gated tests — and the enumeration below is the last one taken test-by-test.) Linux runs 8 tests
Windows does not — 2 Unix-socket transport tests, 5 wasmtime live-engine contained-execution tests,
and 1 verified-plugin-on-wasm test — while Windows runs the 2 refusal counterparts
(`windows_refuses_contained_execution_rather_than_risk_a_fastfail` and
`a_verified_plugin_on_wasm_inherits_the_windows_enforcement_refusal`). 8 − 2 = 6.

Both figures above were taken on the **same tree**, with nothing edited during either run. That
mattered: an earlier attempt compared a Windows number taken *before* three tests were added against
a Linux number taken after, which looked like three tests silently not running on Windows and was
nothing of the kind. **Two measurements are only comparable if they were taken of the same thing.**

Also verified by execution: `cli-sweep.sh` **27/27** · `cargo deny` advisories/bans/licenses/sources
all **ok** · `cargo clippy … -D warnings` **exit 0** · `cargo install` → PATH → the extension works
with **default settings** · Miri `delulu-runtime` FFI decoding 6 tests / 0 UB / 2.4 s and
`delulu-syntax` `fmt::` 16 tests / 0 UB / 1357 s · Trojan Source refused (`DL0107`) · the LSP survives
malformed framing six ways and answers seven pipelined requests · all six `--json` surfaces emit valid
JSON, including on failure.

---

## 12. Performance — measured, never promised

The project's stability contract says performance is **measured, never promised**, and the
constitution's performance-honesty rule (§5.11) requires publishing the worse numbers alongside the
better ones. Everything here comes from `measurements/`, where each record names the machine, the
method and the raw figures so you can disagree with the method rather than only the conclusion.

### The finding that matters most for daily use: process startup dominates

On a 35-line file, **26.87 ms of 32.82 ms — 82% — is Windows creating a process**, before a single
byte is compiled. The compiler's own work on that file is under 1.5 ms.

| Layer (min ms) | Windows 11 native | Linux (WSL2) |
| --- | ---: | ---: |
| `empty` — a Rust `fn main() {}` | **26.87** | **3.54** |
| `delulu --version` | 32.04 | 6.19 |
| `delulu check` — 1 line | 32.47 | 6.72 |
| `delulu check` — 35 lines | 32.82 | 7.63 |
| `delulu check` — 1,001 lines | 47.40 | 29.16 |
| `delulu authority` — 35 lines | 33.44 | 7.26 |
| `delulu fmt --check` — 35 lines | 34.91 | 9.10 |

**The practical consequence, and it is large: batch your `check` calls.**

| 20 files, best of repeats | 20 separate invocations | one invocation | speed-up |
| --- | ---: | ---: | ---: |
| Windows | 711.6 ms | **48.0 ms** | **14.8×** |
| Linux | 119.4 ms | **14.2 ms** | **8.4×** |

Nothing was made faster to achieve that — the loop simply stopped paying the process floor once per
file. Every *other* command takes exactly one path and refuses a second rather than silently using
the first. For a long edit→check loop, `delulu lsp` pays the process cost once and every check after
it is the sub-millisecond part.

### Checking scales linearly in program size

A **30,000-line program checks in ~150 ms**. Every shape measured is linear or better in input size.
One published exception: `records_N`, an N-field record with a function reading all N fields, cost
**516 ms at N = 2000** — recorded rather than hidden, and the reason ruling D44 exists.

Dependency resolution: verifying **25 packages across 5 graphs took 98 ms** in total (19 ms mean per
graph).

### Execution speed, against C — and criterion 1 is NOT met

From the pinned Study-C run (release build, minimum of 5 repeats, whole-process wall clock):

| Benchmark | C (gcc -O2) | wasm (opt backend) | interpreter | wasm ÷ C | interp ÷ C |
| --- | ---: | ---: | ---: | ---: | ---: |
| `fib_recursive_24` | 6 ms | 12 ms | 77 ms | **2.0×** | 12.8× |
| `loop_sum_1m` | 6 ms | — (`DL1201`) | 363 ms | n/a | 60.5× |
| `string_build_20k` | 6 ms | — (`DL1201`) | 20 ms | n/a | 3.3× |
| `wordcount_macro` | 6 ms | — (`DL1201`) | 12 ms | n/a | 2.0× |
| `list_map_macro` | 6 ms | — (`DL1201`) | 17 ms | n/a | 2.8× |
| `nested_calls_macro` | 5 ms | — (`DL1201`) | 281 ms | n/a | 56.2× |

**Stage 10 criterion 1 asked for a geometric mean ≤ 2.5× C under the optimizing backend, and it is
not met.** `— (DL1201)` means the WASM backend **could not run that program at all**: the 1.x backend
compiles a *subset* of the language and does not lower unbounded loops in `main`, string building, or
the macro workloads. Five of six benchmarks fall outside it. That is published as a real limitation,
not omitted, and the DIR-level optimizer §2.1 describes (cross-package inlining, monomorphization,
escape analysis) is **deferred** under ruling D18.

The honest summary: **the interpreter is the language**, it is between 2× and 60× C depending
entirely on the workload, and the optimizing backend is a narrow fast path rather than a general one.

---

## 13. What has actually been done on each operating system

The distinction this section turns on is the one the whole project turns on: **executed** and
**prepared** are different words.

| | Windows 11 (x86_64-msvc) | Linux (WSL2 Ubuntu) | macOS |
| --- | --- | --- | --- |
| Full test suite | ✅ **124 binaries / 1,645 tests / 0 failed**; on CI's Windows runner **125 / 1,647 / 0** (runs 4 and 5, 2026-09-14) | ✅ **124 binaries / 1,654 tests / 0 failed**; on CI **125 / 1,657 / 0**, x86-64 and arm64 (run 5, 2026-09-14) | ✅ **125 binaries / 1,657 tests / 0 failed** on CI's macOS runner (runs 3 and 5, 2026-09-14) |
| CLI sweep (`cli-sweep.sh`) | ✅ 27/27, and 27/27 on CI (runs 4 and 5) | ✅ run in earlier passes; 27/27 on CI (runs 3 and 5) | ✅ 27/27 on CI (runs 3 and 5) |
| Compiler + interpreter | ✅ | ✅ | ✅ via CI's suite (run 2) |
| WASM engine | ✅ | ✅ (plus 5 live-engine tests Windows refuses by design) | ✅ via CI's suite (run 2) |
| Language server | ✅ | ✅ (via the suite's `lsp_cli.rs`) | ✅ via CI's `lsp_cli.rs` (run 2) |
| VS Code extension, end to end | ✅ **real editor, real server** | ⚠️ **not run** | ❌ |
| Miri | — | ✅ broker/diag/atlas/syntax + the FFI decoder | — |
| `cargo deny`, clippy `-D warnings` | ✅ | — | — |

### Windows — the primary development platform

Everything above was developed and verified here. The extension was installed into a real VS Code, in
an isolated profile, and exercised: activation, server startup, diagnostics, formatting, the atlas
webview, snippets, tasks, and the two security witnesses (a planted binary that ran against the
unfixed build and did not against the fixed one).

Windows-specific behaviour that is real and tested: the `broker_transport` named-pipe path with SID
checks, and `limits::tests::windows_refuses_contained_execution_rather_than_risk_a_fastfail` — Windows
**refuses** contained plugin execution rather than risk a fast-fail, and a test pins that refusal.

### Linux — verified for everything except the editor client

The full suite, the sweep, and all Miri work run here. Linux additionally runs 8 tests Windows does
not: 2 Unix-socket transport tests, 5 wasmtime live-engine contained-execution tests, and 1
verified-plugin-on-wasm test.

**The one gap: `editors/vscode/e2e.js` has never been run on Linux.** The extension is
platform-independent JavaScript and `server-resolve.js` is unit-tested for POSIX lookup, but *"the
unit tests cover the POSIX branch"* and *"the extension works on Linux"* are different claims and this
document does not blur them. Running it needs a display; the command is
`node e2e.js <path-to-delulu>`.

### macOS — green on CI, and only on CI

**First executed 2026-09-14, on CI's macOS runner** (the project has no Apple hardware of its own). The
first run stopped building a dependency; the second built the whole workspace and passed 1,654 of
1,655 tests; the third was green end to end (`docs/design/CROSS_PLATFORM_VERIFICATION.md` §9). In the
fourth, one dead-man test failed there because the runner left its thread unscheduled for over 100 ms
— a test measuring the runner rather than the code; it now judges each revocation against the gap
its thread left (§1.1); run 5 passed with it.
Before that, the standing rested on:

1. The runtime's Unix half is `#[cfg(unix)]`, and *that same code* passes the full suite on Linux —
   which exercises the socket transport, the `0700` directory guard, and the interpreter.
2. Static reading of the macOS-relevant `cfg` branches.
3. **Actual compilation evidence since 2026-08-04**, which is more than reading: 7 crates
   **compile clean** for `x86_64-apple-darwin`, and 3 type-check for `aarch64-apple-darwin`. The full
   workspace cannot be checked from this host because `libffi-sys` picks its MSVC path from the
   Windows host.

Runs 2 and 3 proved more than that reading could: the default build, CPython embedding included,
links and passes on macOS, and in run 3 so did the CLI sweep and the fuzz campaign. What is still
**not** proven: anything outside CI's runner — a developer's Mac, and the VS Code extension on one.

macOS is *green on CI, and only on CI*. The testing remote (§1.1) triggers the `macos-latest`
job on every push to `master`, so every push re-checks it; what no push can check is a Mac outside that
runner.

### The three surfaces, per platform

- **CLI** — Windows and Linux both verified by `cli-sweep.sh`, which asserts an exact exit code for
  each of 27 cases; on CI, 27/27 on macOS and Linux (run 3) and on Windows (run 4).
- **Compiler** — Windows and Linux both run the full suite including the conformance corpus and the
  core-invariance snapshot (the exact bytes the toolchain answers with, for all 108 shipped programs,
  so tooling work cannot quietly move the language). macOS: the same suite runs green on CI (run 3).
- **VS Code extension** — verified end to end on **Windows only**. The `.vsix` is platform-independent
  and its unit tests cover POSIX path resolution, but no editor has been launched against a server on
  Linux or macOS.

### 13.1 Which DeluluLang features reach the editor, and which do not

Verified against `crates/delulu/src/lsp.rs` and the extension manifest, not from memory.

| Feature | In VS Code | How, or why not |
| --- | --- | --- |
| **Authority** | ✅ four ways | the `authority: {…}` code lens on `main`; **Show authority report** (the §10.5 JSON, answered by the server over `workspace/executeCommand`); **Show authority atlas** (the call graph with each function's effect row on it); and hover, which carries the transitively computed authority |
| **Effects** | ✅ | in hover, in **inlay hints showing the *inferred* row** on unannotated lambdas, and as a dedicated semantic-token kind |
| **Capabilities** | ✅ | semantic tokens have their own kinds for capability types, reference capabilities and secrets |
| **Diagnostics + typed repairs** | ✅ | the compiler's own codes, spans and repairs. An authority-widening repair is ⚠-titled and **never** marked preferred, and carries `data.authority_widening` so an agent can refuse it by policy |
| **Formatting** | ✅ | the same `format_source` the CLI runs, byte-identical by test |
| **Tests** | ✅ | `▶ run test` runs *that* test by name |
| **The Guard** | ❌ **nothing at all** | CLI only: `delulu guard status / policy / request / pending / approve / deny / permits` |
| **Broker, grants, leases, audit chain** | ❌ | CLI only: `delulu grants`, `delulu audit`, `delulu broker` |
| **Plugins, devices, fleets, registry** | ❌ | CLI only |

**The Guard's absence is structural, not an oversight.** `lsp.rs` states it in its own header: the
module *never constructs an `Interp`, a broker*, or a lease. The server is analysis-only, which is
what lets it read a hostile file safely — a compromised workspace cannot use it as an effector, and
its availability is explicitly not a security property (spec §11). The Guard supervises things at
**run** time; it has nothing to say about a file you are editing.

**What is genuinely missing rather than deliberately absent:** a **read-only** Guard/broker status
view in the editor — current policy tiers, pending approval requests, live permits. That would not
require the server to hold any authority (it could shell out to `delulu guard status --json`), it
would be useful to anyone supervising an agent, and **it does not exist**. Named here so the gap is a
decision rather than an assumption.

---

## 14. CLI vs compiler vs editor — what each user actually gets

**There is a real compiler. There is no separate compiler *binary*.** Those are different statements
and conflating them misleads in both directions.

**The compiler is a specified component** (`STAGE1_SPECIFICATION.md` §9 "Compiler architecture",
`docs/release/CHECKPOINT-1.0.md` §3 "The compiler"):

| Stage of the pipeline | Crate | What it does |
| --- | --- | --- |
| lex → parse | `delulu-syntax` | tokens, AST, a hand-written recursive-descent parser **with error recovery** — it resyncs and keeps finding faults rather than stopping at the first |
| resolve → typecheck → effect/authority check | `delulu-check` | names, types, effect rows, the authority lattice, `Secret[T]` opacity, reference capabilities and sendability. The docs call it **"the soundness core"** |
| diagnostics | `delulu-diag` | spans, the code registry, the JSON envelope, typed repairs, the human renderer |
| execute | `delulu-runtime` | the tree-walking interpreter — **the interpreter is the language** |
| compile to WASM | `delulu-wasm` | a *subset* backend under a deny-by-default Wasmtime host |

**154 registered diagnostic codes** (the Survey's measured count today; `CHECKPOINT-1.0.md` says 145
and is correct *as a 1.0 snapshot* — release documents are deliberately exempt from the
freshness scan). Codes are **add-only** from 1.0, each with an accepting *and* a rejecting conformance
witness; coverage is 100% and hard-gated per commit. The grammar is normative
(`STAGE1_SPECIFICATION.md` §3 plus each stage's additions, indexed by `docs/reference/grammar.md`).
Where the checker cannot decide, **it refuses rather than guesses**.

### When did the compiler last change?

Two different questions, and the second is the one that matters.

- **Last touched:** 2026-08-07 (today). Three commits edited compiler crates — but only to fix lints
  (a `zip` replacing a hand-rolled index in `deps.rs`, a `const { assert! }` in `codes.rs`, an
  `#[allow]` in `ast.rs`, a `while let` in `parser.rs`) and to shrink two **test-only** `cfg!(miri)`
  budgets in `fmt.rs`.
- **Last change to what the compiler DECIDES:** 2026-08-04, commit `47393b9` — the `DL0210`
  deep-nesting guard, so a valid but pathologically nested module is *refused* instead of overflowing
  the stack. Before that, the P16/P17 soundness fixes of 2026-08-03 (the escapable effect row, and
  `Secret.verify` being typed pure).

**And that is checked, not asserted.** `tests/core-invariance/SNAPSHOT.txt` records the exact bytes
the toolchain answers with for all **109** programs the repository ships, across 362 invocations. It
was last modified on **2026-08-04**, and `the_core_still_answers_exactly_as_recorded` passes against
today's binary — so today's edits provably moved nothing.

> This is the owner's **core-regression rule**: *tooling is for future developers, the core is the
> product; a green suite is not proof.* After any tooling-only change, that snapshot is the evidence
> that the language did not move. Regenerate it deliberately, never incidentally:
> `DELULU_BLESS=1 cargo test -p delulu --test core_invariance`.

What does *not* exist is a separate driver you invoke yourself. There is one binary, `delulu`, and
the compiler is reached through `delulu check` (analyse), `delulu build` (resolve deps, verify pins,
optionally emit `.dwx`) and `delulu run` (check, then execute). There is no `deluluc`, and nothing to
install alongside. **That is why the editor's answers cannot drift from the CLI's** — the language
server calls the same `delulu-check`, so a diagnostic in your editor is the diagnostic
`delulu check --json` prints, not a re-implementation of it.

There are exactly **three doors**:

| Door | What it is | Who uses it |
| --- | --- | --- |
| `delulu <subcommand>` | 32 subcommands. **The whole product.** | people and scripts |
| `delulu lsp` | LSP 3.17 over stdio. **Analysis only.** | every editor, and agent harnesses |
| the VS Code extension | a thin client over `delulu lsp`, plus terminal commands that shell out to the CLI | VS Code users |

### The dividing line, and it is not arbitrary

**The editor gets everything that reads. The CLI gets everything that acts.**

`delulu lsp` never constructs an interpreter, never loads a plugin, and holds no broker connection or
lease — stated structurally in `lsp.rs`'s own header, not merely intended. That is what lets it open a
hostile file safely: a compromised workspace cannot use the language server as an effector, and its
availability is explicitly *not* a security property (spec §11).

So anything that **holds authority or executes code** is CLI-only, by construction:

| Available in the editor | CLI only |
| --- | --- |
| diagnostics, typed repairs, quick fixes | `delulu run` / `test` — *reachable from the editor, but only by shelling out to the CLI in a terminal, and gated on workspace trust* |
| hover, completion, signature help | `delulu guard` — policy tiers, permits, approvals, e-stop |
| definition, references, rename | `delulu broker` / `grants` / `audit` — the grant tree, leases, revocation, the audit chain |
| document + workspace symbols | `delulu plugin` — build, inspect, verify `.dpx` |
| semantic tokens, inlay hints (inferred rows) | `delulu fleet`, device profiles, hardware adapters |
| **authority**: the lens, the report, the atlas | `delulu publish` / `deploy` / `add` / `login` — the registry |
| formatting | `delulu secrets`, `keygen`, `sign`, `verify-sig` |
| | `delulu repl`, `morph`, `locale`, `doctor`, `explain`, `completions` |

### What that means in practice, per audience

**A developer in VS Code** sees the authority system continuously and passively: the effect row in
hover, the inferred row as an inlay hint, `authority: {Write}` as a lens on `main`, and the atlas one
click away. They never need to run a command to know what their program can do. But to *supervise*
anything — approve a guarded operation, inspect a grant tree, revoke a lease — they go to a terminal.

**Someone using only the CLI** loses nothing. Every capability exists there; the editor is a
convenience layer over a subset. `delulu authority --json` and `delulu atlas --format json` give the
same answers the editor renders.

**An AI agent or harness** should use `delulu lsp` and speak the protocol rather than shelling out per
question: it pays the process cost once, and on Windows that cost is 82% of a small `check`. Over the
wire it gets diagnostics identical to `delulu check --json`, typed repairs carrying
`data.authority_widening` so it can refuse them **by policy rather than by parsing prose**, and the
authority report via `workspace/executeCommand` → `delulu.authority`. When it needs to *act* — run,
grant, revoke — it drops to the CLI, which is exactly where the Guard can supervise it.

**A robot or physical-AI deployment** uses the CLI only. Devices, envelopes, dead-man timers, sign-off
records and e-stop have no editor surface at all and are not meant to.

### The one gap worth naming

A **read-only** Guard/broker status view in the editor — current policy tiers, pending approval
requests, live permits — would not require the server to hold any authority (it could shell out to
`delulu guard status --json`), would be genuinely useful to anyone supervising an agent, and **does
not exist**. Recorded here so its absence is a decision rather than an assumption.
