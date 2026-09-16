# The 1.0 release checkpoint

**Companion to** [`CHECKLIST-1.0.md`](CHECKLIST-1.0.md), which is the *gate* — ten criteria, each MET
only because a named test or a committed artifact says so. This page is the *state of the thing*: what
exists, how much of it, what it is verified to do, and what it is not.

**Written 2026-08-02**, after the production-readiness and architecture-stabilization pass
(`PRODUCTION_READINESS_REVIEW.md`, rulings D67–D71). Every number here is either recounted from the
tree by the Survey, which fails a test when it drifts, or taken from a suite run named beside it.

---

## 1. Architecture

**Thirteen crates: nine are the language, four only measure or map this repository.** The split is
declared per crate (`[package.metadata.delulu] surface`) and gated, so it cannot drift into a proxy
the way it once had (D69).

```
delulu-diag ──▶ delulu-syntax ──▶ delulu-check ──▶ delulu-broker
                                        │                │
                                        ├──▶ delulu-atlas│
                                        └──▶ delulu-runtime ──▶ delulu-wasm
                                                    │
                                                    └──▶ delulu-registry
delulu (CLI) ──▶ everything above
delulu-survey ──▶ nothing in this workspace
```

The graph is a DAG and every edge is deliberate. Two shapes are worth naming because both look like
defects and are not:

- **`delulu-broker` depends on `delulu-check` for exactly one item, `Effect`.** Authority is *defined*
  in terms of effects, and one shared definition between the crate that computes rows and the crate
  that grants them is the "one list, both sides" pattern this project adopted after six drift
  findings. Relocating it buys tidiness and risks the drift.
- **`PluginEngine` has a single implementor.** It lives in `delulu-runtime` and is implemented in
  `delulu-wasm`, which depends on the runtime — the trait exists to **invert** that dependency.
  Removing it creates a cycle.

`delulu-survey` deliberately depends on **nothing** in the workspace, so the map still builds when the
compiler does not.

**Extension points:** nine public traits (`AuditSink`, `SignatureVerifier`, `IdSource`, `ClockSource`,
`Custody`, `BoundForeign`, `ForeignBinder`, `PluginEngine`, `TypeNames`). All have real implementors;
`IdSource` and `ClockSource` are in-crate only, because they exist to make time and identity
deterministic under test.

**Size, recounted from the tree:** 212 Rust files, **101,363 lines**; 121 documentation files, 28,190
lines; 146 DeluluLang programs, 2,942 lines.

## 2. The Survey

A map of this repository generated from this repository: **964 nodes, 8,390 edges**, every edge citing
the file and line it was read from. Nothing is inferred from name similarity or proximity.

- **3 discrepancies, 0 errors, 0 warnings** — all notes, all judged (§8).
- `impact <id>` walks the true blast radius; `rdeps` is one hop and says so. Only relations that
  *propagate* compose — a narrative link between two documents is true at one hop and meaningless at
  two, which an earlier version learned by reporting 236 nodes reachable from every node in the tree.
- **`query` answers "may I change this?" before any edge.** Nineteen nodes are marked entrenched from
  `.github/CODEOWNERS`, each citing its line: the constitution, `DELULU_CORE.md`, `STABILITY.md`,
  `/rfcs/`, `SECURITY.md`, `/docs/security/`, the soundness audit and its laundering suite, and the
  conformance machinery.
- Six integrity invariants hold it to its own standard, including that two builds of one tree produce
  the same map.

## 3. The compiler

`delulu-syntax` (lexer, AST, hand-written recursive-descent parser with error recovery) and
`delulu-check` (name resolution, types, effects, authority, reference capabilities) — the soundness
core.

- **145 registered diagnostic codes**, add-only from 1.0, each with an accepting *and* a rejecting
  conformance witness. Coverage is **100%** and hard-gated per commit.
- Effect rows, the authority lattice, `Secret[T]` opacity, reference capabilities and sendability.
  Where the checker cannot decide, **it refuses rather than guesses** — `default_rcap` is exhaustive
  over every type and returns `None` for a type variable, with the reason written in the source.
- The normative grammar lives in `STAGE1_SPECIFICATION.md` §3 plus each later stage's grammar
  additions. `docs/reference/grammar.md` indexes the productions and now names where each is
  *defined*, checked in three directions (D70).

## 4. The runtime

A tree-walking interpreter is the language; a WASM backend covers a **fragment** and says so.

- **Interpreter**: full language, including actors. The depth bound is a published contract
  (`DEFAULT_MAX_DEPTH`, `STACK_BYTES_PER_DEPTH`), and every thread that can run a program reserves a
  stack sized for it — checked by a gate over every thread-creation site in the tree, after an actor
  behavior was found aborting the process above depth 43 (C70/D67).
- **WASM**: Int/Bool/Str arithmetic, control flow, calls, recursion, `match` on a sum, string
  concatenation, console. `while`, `Float`, records, lists, string methods, clock and Python are
  `DL1201`. **Six of nineteen entry-point programs compile.** The fragment is measured, not estimated.
- **Both engines agree on faults**, not merely on success — an earlier version classified any
  `(Err, Err)` as agreement and hid a real divergence.
- Actors: worker-owned, zero `unsafe`, per-sender-pair FIFO, quiescence-based exit. Race freedom is
  static; deadlock and starvation are **not** prevented, and the guide says so.

## 5. The CLI

**32 subcommands**, one dispatcher, one `--json` contract. Exit codes are stable: `0` success, `1`
diagnostics or a runtime fault, `2` usage error.

- **Every `--json` command emits exactly one object**, enforced by a sweep that counts — it caught a
  double-emit in three separate commands.
- **No command silently ignores an argument.** Extra paths are refused, unknown options are refused,
  and a value-taking option given no value is refused rather than falling back to a default. All
  three are swept across the whole surface rather than checked per command. This matters most for
  the machine channel: a person may notice a missing effect, but an agent that mistypes a flag would
  otherwise get exit 0 for work that never happened.
- Asking a tool what it does never makes it do the thing: `--help` is answered by the dispatch, after
  `keygen --help` once generated a key.
- Every dispatched subcommand appears in `--help`, gated — because an undocumented command is a
  command nothing sweeps.
- `delulu doctor` is the whole environment-and-repository check in one verb; `--check` never writes.

## 6. The package ecosystem

- **Manifest** (`delulu.toml`) declares a package's authority *ceiling*; code exceeding it is
  `DL1009`. **Lockfile** (`delulu.lock`) pins version, content hash and authority — and
  `build --locked` now verifies the recorded effects, cap kinds, secrets and scopes, not only the
  hashes. A 15-case tampering matrix went from 15 accepted to 2 (both named and reasoned).
- **The semver-authority law**: any widening of what a package can do to your system requires a major
  bump, even when the API is unchanged. `authority --diff` prints the verdict.
- **Registry** (`delulu-registry`): sparse index, publish API, server-side authority recomputation.
  Local-index-only in this release, by ruling.
- **Plugins**: `.dpx` artifacts carrying DIR, signed and class-gated (`Verified` / `Contained`). A
  present-but-invalid signature refuses *regardless of policy*; unsigned-but-required is a different
  code from badly-signed.

## 7. Testing

| Gate | Windows | Linux |
|---|---|---|
| `cargo test --workspace` | **115 suites / 1,500 passed / 0 failed / 4 ignored** | **115 / 1,506 / 0 / 4** |
| `cargo clippy --workspace --all-targets` (cold, findings only) | **14** | **14** |
| CLI + compiler sweep (21 exit-status cases) | **21/21** | **21/21** |
| Packaged archive, unpacked and run outside the workspace | **8/8** | **8/8** |
| `delulu-conform --coverage` | 100% | 100% |
| `delulu-conform --check-reference` | in sync | in sync |
| `delulu fmt --check examples` | 0 would change | 0 would change |
| `delulu doctor --check` | 12/12 | 12/12 |

The clippy counts are a *baseline watched for movement*, not a target — none of the findings was ever
a `clippy::correctness` lint. They must be measured **cold, in a throwaway target dir, excluding
cargo's per-crate summary lines**; a warm run under-reports and the summary lines are not findings.
Earlier revisions of this table quoted **65/66**, which was that broken measure, not a regression
since. The 6-test suite delta between platforms is named test-by-test in
`docs/design/CROSS_PLATFORM_VERIFICATION.md` §2 — Linux runs 8 the Windows build does not compile,
Windows runs 2 that assert the corresponding refusal.

Beyond the suite: a **core-invariance snapshot** records the exact bytes the toolchain answers with
for all 108 programs the repository ships (360 cases), so work on the tooling cannot quietly move the
language — it caught a one-word diagnostic change that left the entire pre-existing suite green.
Fuzzing covers four parsers; the laundering suite encodes the soundness audit's exploit set; an
allocation-scaling gate asserts a *shape* rather than a wall-clock constant.

## 7.1 It reproduces from nothing

Not asserted — run. A `git clone` into a fresh directory with its own `target/`, inheriting nothing
from the working tree (2026-08-02, at `ed8a769`, on Linux):

| Step | Result |
|---|---|
| clone | 572 tracked files, `Cargo.lock` present |
| `cargo build --workspace` (cold) | **1 m 21 s**, OK |
| `cargo test --workspace` | **113 suites / 1,485 passed / 0 failed** |
| `delulu-conform --coverage` | 100% |
| `delulu-conform --check-reference` | in sync |
| `delulu fmt --check examples` | 0 would change |
| `delulu doctor --check` | 12/12 |

Re-run at `0c98a58` (2026-08-03, Linux): **573 tracked files**, cold build OK, `DL0703` refusal on the
ungranted run, **113 suites / 1,494 passed / 0 failed**, every gate green.

Then the README's front door, replayed **verbatim** in that clone — the exact commands this page's
own `README` tells a newcomer to type, run from a directory that is not the workspace:

```
delulu new hello              → hello/src/main.delulu, delulu.toml, .gitignore
delulu run . --grant console  → hello, world
delulu check hello.delulu     → ok: hello.delulu checked clean
delulu authority hello.delulu → effects: Write · capabilities: Console stdio
delulu run hello.delulu       → error[DL0703]: `console` was not granted
  … --grant console           → Hello, Delulu
```

**The refusal is the line that matters.** A fresh clone holds zero ambient authority, and the failure
without `--grant console` is what makes the success with it mean anything. The hardening campaign's
very first finding was that this page's front door was *false*; this is it being true, from nothing.

## 8. Known limitations — stated, not softened

1. **macOS has never been executed. Not once, in any phase.** There is no Apple hardware and nothing
   was emulated. Every conditional-compilation site was enumerated and is exhaustive for macOS, and
   the one macOS-specific hazard the audit could quantify — `sun_path` is 104 bytes there against 108
   on Linux — is now refused with both figures. **A path that should work and a path that has been
   run are different claims.** *(Superseded 2026-09-14: CI's macOS runner passed the whole suite end
   to end — [`CROSS_PLATFORM_VERIFICATION.md`](../design/CROSS_PLATFORM_VERIFICATION.md) §9. What is
   still unrun is a developer's Mac.)*
2. **Nothing is distributed.** No crates.io entry, no release binary, no public repository. *(2026-09-17:
   the source is now in a public **testing** repository; nothing is released, and the final public
   repository will be a different one.)* The CLI
   could not be published even deliberately: its path dependencies carry no version numbers, so
   `cargo publish` refuses it. Only the CLI is *permitted* to publish; the libraries are refused by a
   gate, because `STABILITY.md` §2 says they are not a stable interface.
3. **Certification is NONE.** No safety standard, no external audit, no third-party review.
4. **No physical device has ever been commanded.** Every demonstration drives the simulator. The one
   hardware adapter is an **operator-supplied subprocess**, not the specification's signed
   Verified-class plugin: signing can be *required* and *pinned*, but the shipped path buys isolation
   and reach, not supply-chain assurance.
5. **The WASM backend is a fragment** (§4). The interpreter is the language.
6. **C55, a published limit**: runtime record field access is O(record width). Measured, U-shaped with
   a minimum near width 20; the real fix is static field indices through the DIR.
7. **The mechanized core proof is now PARTIAL, and its target changed.** As of 2026-08-04 Lean
   4.32.2 machine-checks the **higher-order fragment** with no axioms at all
   (`docs/design/models/lean/DeluluCore.lean`). Everything else — capabilities, the store, secrets,
   attenuation, Progress, Preservation — remains a paper sketch, and **two of those sketches are now
   known to be defective**: the calculus models no primitive that invokes a function argument
   (P17-T1), and Progress is false as stated for scope violations (P17-T2).
8. **Three Survey notes remain, and all three are correct.** Bare `D<n>` citations rely on a
   documented latest-stage default; one `C<n>` token is the C language (found, aptly, inside the
   Survey's own comment explaining why it is not a finding); and five lines quote test counts, which
   cannot be read out of a tree — four of those are dated historical records and the fifth is README,
   verified against the run above.
9. **Deadlock, livelock, starvation and mailbox exhaustion are not prevented.** Race freedom is
   static; liveness is not.
10. **The optimizer described in spec §2.1 is not implemented**, and there is no native backend.
    Both are RFC-gated and published as deferred.

### Added by the P17 proof campaign (2026-08-03→04) — all OPEN

Full accounts with witnesses in [`../design/PROOF_CAMPAIGN.md`](../design/PROOF_CAMPAIGN.md); the
proof-boundary assignment for every claim is in [`../MATHEMATICS.md`](../MATHEMATICS.md).

11. 🟡 **`cargo deny check advisories` reports "advisories ok" — and the two entries that make it say
    so are ignores, so the verdict is only as good as their reasons.** Re-measured 2026-08-07 with a
    freshly installed `cargo-deny`.

    The two **wasmtime** advisories that used to dominate this entry are genuinely gone: the 27 → 47
    upgrade (2026-08-04) closed **RUSTSEC-2026-0096**, the aarch64 Cranelift sandbox escape, along
    with sixteen others. `Cargo.lock` pinned `wasmtime 47.0.3` until 2026-09-14, when two advisories
    published after this re-measurement (RUSTSEC-2026-0268 and RUSTSEC-2026-0269, both in WASI
    functionality DeluluLang never uses) failed the gate on the first CI run; the patch release
    `47.0.4` closed both. This item previously said "four reachable advisories" and was **stale** —
    that is documentation drift, found by re-running the tool rather than re-reading the page.

    What remains is **two pyo3 CVEs in a default build** (`pyo3 0.25.1`; `python` is a default
    feature). They are ignored on **reachability**, verified rather than asserted: RUSTSEC-2026-0176
    is in `nth`/`nth_back` on PyList/PyTuple *iterators*, which this code never creates — it only
    constructs those objects — and RUSTSEC-2026-0177 is `PyCFunction::new_closure`, which appears
    nowhere.

    **That argument is now a test, not a comment** (`governance.rs::the_ignored_pyo3_advisories_are_still_unreachable`).
    It had to become one: the reasons justified themselves with a workspace-wide grep — *"there is no
    `.nth(` anywhere in the workspace"* — and there are now sixteen, none of them near Python. The
    criterion was a **proxy** that drifted independently of the condition it stood for, so a reviewer
    running it would get a false alarm and, the second time, stop believing it. The test checks the
    real condition, scoped to the one file that touches pyo3, and fails if pyo3 is ever used
    elsewhere. The real fix is still the upgrade (pyo3 0.25 → 0.29 removes `Python::with_gil`), and
    it stays deferred deliberately rather than attempted blind, because GIL handling is exactly where
    a hasty migration introduces undefined behaviour.
12. **Secrets are protected against direct observation, not against a program that is trying.**
    `Secret.map` hands its closure the plaintext, gated only on purity; `verify` reads a chosen bit
    back out. `verify` now carries `Declassify` so this is **visible** — but it is not impossible,
    and `verify` still requires no `Cap[Declassify]`. There is **no implicit-flow tracking**; this is
    not noninterference.
13. ✅ **Audit-chain truncation — FIXED.** `ANCHOR.json` now records the head and record count
    outside the log; `verify` compares against it and `AuditLog::open` refuses a log that disagrees
    with its own anchor. **Still open, and pinned as a passing test:** the anchor sits beside the
    log, so an attacker who rewrites both is not caught. That needs an **external witness**; what is
    closed is accidental truncation and naive tampering.
14. ✅ **Wall-clock expiry — FIXED.** `Broker::now` ratchets to the running maximum, so a backwards
    step can no longer resurrect expired authority. `Instant` was not an option — certificate
    validity times are signed absolute epoch-millis — so the reading stays wall-clock-comparable and
    is merely made non-decreasing. It can only withhold authority, never grant it. **Residue:**
    monotonicity is not accuracy, and the ratchet is in-memory per broker.
15. ✅ **`⊑` preorder / `⊓` asymmetry / divergent hashes — FIXED 2026-08-06 (F1, F2, F3).** Neither
    was an escalation — the meet is a genuine greatest lower bound and never widens, proved in Z3
    across all nine dimensions — but `⊑`-equivalent authorities **hashed differently**, which reaches
    the audit chain and certificate signatures. All three had one cause: `⊑` is defined through a
    **non-injective** resolution, and such a relation is a preorder on its domain as a matter of
    mathematics, not a bug in the comparison. The broker now stores one canonical representative per
    equivalence class, so the quotient and the representation coincide. **The three committed
    `#[ignore]`d failing tests are un-ignored and passing**; `order_laws.rs` is 9 passed, 0 ignored.
    **Two things worth carrying forward.** First, "canonicalization" meant *two* collapses, and only
    one was predicted: spellings (`./data` ≡ `data`) **and set redundancy**
    (`{./data, ./data/sub} ≡ {./data}`). The Z3 model could not express the second — it abstracts a
    dimension as a set over an opaque element type — so the counterexample came from the enumerator,
    not the proof. Second, **this is format-affecting and shipped without an RFC**, which
    `PROOF_CAMPAIGN.md` had earlier said it would not do; that is an owner decision, recorded rather
    than presented as compatible. **Residue:** an authority written before the change with a
    non-canonical spelling hashes differently from the same authority written after it.
16. **Type inference is order-dependent and has no principal types.** Swapping two parameters can
    decide whether a program compiles. Fail-closed, so no authority escapes.
17. ✅ **Miri now completes — 2026-08-06, three crates, 192 tests, zero undefined behaviour.**
    Previously it had been started twice and finished neither time, and an unfinished run is not a
    pass. Run **per crate** rather than over the workspace, on Linux:

    | crate | result | wall time |
    |---|---|---|
    | `delulu-diag` | 45 passed, 0 failed | 54 s |
    | `delulu-broker` | **129 passed, 0 failed** | 1583 s |
    | `delulu-atlas` | 18 passed, 0 failed | 342 s |

    **`-Zmiri-disable-isolation` is required, and its absence had been misread as a failure.**
    Without it Miri aborts on `create_dir_all` with *"unsupported operation"* in any crate whose
    tests touch the filesystem. That is a Miri limitation, **not a finding**, and an earlier pass had
    recorded the abort as though it were one.

    The honest limit, as stated on 2026-08-06: the crates Miri *can* run contain **no `unsafe` at
    all**, and the crates that do — `broker_transport.rs`, `foreign.rs`, `foreign_worker.rs` — are
    the ones it cannot execute. So "0 UB" meant the interpreter found nothing wrong in the code least
    likely to contain it. `delulu-broker` is still the crate the security argument rests on, and it
    is interpreted end to end.

    **That limit was stated too strongly, and 2026-08-07 corrected it.** "Exactly the ones it cannot
    execute" was true of the FFI *calls* and false of the code around them. `validate_c_string` in
    `foreign.rs` — a hand-rolled NUL scan with `ptr.add` and `slice::from_raw_parts`, the highest-risk
    function in the tree — takes a raw pointer to **caller-supplied** memory and needs no foreign
    library to exercise. It already had five tests against crafted Rust-owned buffers, and nothing had
    ever interpreted them, purely because the crate was not in the matrix. They now run as a dedicated
    `miri-ffi` job: **6 tests, 0 UB, 2.4 s** — against 26 minutes for a crate containing no `unsafe`.

    The detector was confirmed live rather than assumed. A temporary probe passing a 4-byte buffer
    with no terminator and a bound of 64 was rejected with *"attempting to access 1 byte, but got
    alloc+0x4 which is at or beyond the end of the allocation of size 4 bytes"* — so under this
    project's flags (`-Zmiri-disable-isolation`, Stacked Borrows deliberately ON) Miri still sees an
    out-of-bounds read here. A gate that cannot fail is not a gate, and that applies to the tool as
    much as to the test.

    What stays out of reach, as an explicit assumption rather than a to-do: Miri cannot execute
    `dlopen` or Windows API calls, so the foreign *calls* themselves — and all 37 `unsafe` sites in
    `delulu`'s Windows transport — remain uninterpreted. **The general lesson is that choosing where
    to point a checker is a bigger decision than how to configure it.** A tool aimed at code that
    cannot exhibit the defect reports clean forever, and reads as coverage while doing it.

    **`delulu-syntax` timed out at crate granularity** — the whole `fmt::` module exceeded 30
    minutes with no result line, which is a timeout and not a pass. Per-module batching then closed
    `lexer` (25), `num` (4), `token` (3), `grammar` (3) and `morph` (18), all clean. The two bulk
    tests responsible now shrink under `cfg!(miri)` rather than being skipped: skipping would buy a
    green tick by interpreting none of that code, while shrinking still interprets the same paths on
    a small input.

    **`fmt::` now completes — verified 2026-08-07: 16 passed, 0 failed, 1 ignored, 1357 s.** It took
    three attempts, and the first two failed for the same reason: they changed a quantity without
    measuring one. The first cap counted *parse-clean files kept*, which bounds nothing, because the
    walk still parses every reject-corpus file it meets on the way. The second counted files
    *examined*, which was the right quantity at the wrong value. Only the third was preceded by a
    measurement — ~115 s per corpus file, ~54 s per generated program — which showed the two tests
    together needed ~41 minutes against a 40-minute wall, so the module reported a timeout while
    every test inside it was passing. **A per-test budget does not bound a per-module run, and
    nothing was measuring the sum.** One `time` invocation would have replaced all three attempts. **`-Zmiri-disable-stacked-borrows` was refused** — it would speed this up a lot by
    switching off Miri's pointer-aliasing detector, and a run that reports "0 UB" without being able
    to see aliasing violations is a weaker claim wearing the same words.
18. **CI carries every campaign gate and has never executed.** The repository is not pushed.
    "Prepared" and "green" are different claims.
19. **The compiler could be crashed by deeply nested input — FIXED 2026-08-04 (`DL0210`).** A valid
    module nested 100,000 levels deep overflowed the stack and killed the process (exit 127) instead
    of producing a diagnostic. Nesting is now capped at 128. Recorded here rather than quietly
    closed, for three reasons: the fix **narrows the accepted language** (input that used to compile
    now returns `DL0210`); it required **two** distinct fixes, because bounding parser recursion left
    a second unbounded recursion in `Drop` walking the postfix `Box` chain; and it was found by
    inspecting **disk usage**, not by any test — ten 784 MB crash dumps had sat in `%TEMP%` for a day
    and the fuzzers never generated input that deep. **Residue:** the limit is sized for a 2 MiB
    thread stack, so it is conservative on the 512 MiB main thread.
20. **macOS is now partly type-checked, still never executed.** As of 2026-08-04 the pure-Rust core
    compiles for `x86_64-apple-darwin` (0 errors) — the first macOS evidence from a compiler rather
    than from reading code. `aarch64-apple-darwin` **could not be checked from this host**: `blake3`
    builds a NEON path through `cc-rs` and no Apple/ARM C cross-compiler exists here. That is a host
    limitation, **not a defect**, and blake3's `pure` feature was deliberately not enabled to force
    it green. **Zero macOS executions remain the standing position** — nothing about linking, the
    test suite, the `SUN_PATH_MAX = 104` socket guard, or the C-dependent crates is proven.

    **Re-verified and widened 2026-08-06.** Reproduced from scratch rather than carried forward, and
    extended from four crates to all seven that have no C dependency: **`x86_64-apple-darwin` is
    clean on all seven**. Two corrections, both toward *more* being known: **Apple Silicon is not
    entirely uncheckable here** — `delulu-diag`, `delulu-syntax` and `delulu-survey` type-check clean
    for `aarch64-apple-darwin`; the 2026-08-04 pass happened to sample only crates that depend on
    `blake3`, so a per-crate failure was read as a whole-target one. And every aarch64 failure was
    read individually rather than assumed alike: all four are the same
    `failed to find tool "cc"` in blake3's build script. The **full workspace** still cannot be
    cross-checked for either Apple target, blocked by a *different* third-party build script —
    `libffi-sys` picks its MSVC path from the host and dies on `Could not locate cl.exe`.
    Type-checking is not running; the standing position is unchanged.

## 9. Future roadmap

Ordered by what would most change the language's usefulness, not by ease:

1. **Run it on a Mac.** Everything else about the third platform is an argument.
2. **Mechanize Delulu Core.** The soundness story is currently a test suite plus paper proofs; a
   machine-checked core is what would make the strongest claims safe to state plainly.
3. **Widen the WASM backend** past its fragment, or retire the parity claim to match what it is.
4. **Static field indices through the DIR** (closes C55) and a name→index map in the checker (closes
   C48's residual scan).
5. **Distribution**: version fields on path dependencies, a published CLI, signed release binaries —
   each a decision, not an accident, and each needing its own ruling.
6. **A signed Verified-class hardware adapter**, which is what §5.4 describes and what a real device
   deployment would require.
7. **Per-plugin microVMs**, per-spawn mailbox configuration, supervision for actors — all named,
   RFC-deferred, none pretended.

---

## The verdict on this checkpoint

**The gate's verdict stands: 1.0 shipped, locally, on 2026-07-20.** What this pass adds is not a
higher number but a shorter list of things that were true by accident. A normative runtime rule that
was false on the concurrency path is now true and gated; a stability promise that had no mechanism has
one; a published count that was right by coincidence is now derived from the question it claims to
answer; and an index that led nowhere leads somewhere.

**What would prevent calling it a *public* stable 1.0 is on the list above and has not moved: macOS
has never run it, nothing is distributed, and certification is none.** Those are facts about the
world, not about the code, and the honest thing is to name them in the same voice as the successes.
