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
| `cargo test --workspace` | **113 suites / 1,488 passed / 0 failed / 4 ignored** | **113 / 1,494 / 0 / 4** |
| `cargo clippy --workspace --all-targets` (cold, findings only) | **14** | **15** |
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
   run are different claims.**
2. **Nothing is distributed.** No crates.io entry, no release binary, no public repository. The CLI
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
7. **The mechanized core proof (Delulu Core) does not exist.** Theorems have paper-proof sketches;
   mechanization is committed future work and is never claimed as present.
8. **Three Survey notes remain, and all three are correct.** Bare `D<n>` citations rely on a
   documented latest-stage default; one `C<n>` token is the C language (found, aptly, inside the
   Survey's own comment explaining why it is not a finding); and five lines quote test counts, which
   cannot be read out of a tree — four of those are dated historical records and the fifth is README,
   verified against the run above.
9. **Deadlock, livelock, starvation and mailbox exhaustion are not prevented.** Race freedom is
   static; liveness is not.
10. **The optimizer described in spec §2.1 is not implemented**, and there is no native backend.
    Both are RFC-gated and published as deferred.

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
