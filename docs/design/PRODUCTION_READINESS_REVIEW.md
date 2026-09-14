# Production-readiness review

**Commissioned 2026-08-02.** *"A production readiness and architecture stabilization phase, not
feature chasing … Do not assume previous conclusions are correct. Challenge them independently."*

**Companion:** `HARDENING_CAMPAIGN.md` (the findings ledger), `STAGE10_BUILD_ORDER.md` (the rulings),
`AUTHORITY_GUARD_CAPSTONE.md` (the authority/Guard discharge), `CROSS_PLATFORM_VERIFICATION.md`.

This document is the disposition register. Every outstanding recommendation this repository carries —
from the campaign, from previous reviews, from the Survey's own notes, and from this review — appears
below with a verdict: **implement**, **improve**, **reject**, or **postpone**. A rejection carries its
technical reason, because a rejection without one is just an omission that learned to write.

---

## 0. Outcome

All five phases are closed. Five commits: `87eca8e` (D67), `47e339e` (D68), `910e9f5` (D69),
`ed8a769` (D70), and this one (D71). The register in §4 carries each item's disposition.

**The pass found one defect in shipped behaviour** — a normative runtime rule that was false on the
concurrency path (C70) — and a number of things that were true only by accident: a stability promise
with no mechanism, a crate count derived from a proxy, an index that led nowhere. It also produced
four corrections to its *own* earlier conclusions, which are in §3 rather than quietly dropped.

The release state, evidence and remaining limitations are in
[`docs/release/CHECKPOINT-1.0.md`](../release/CHECKPOINT-1.0.md).

## 1. Method, and why it differed

Previous passes audited the code against the specs. This one audited **the claims against the tree**,
and started from a different question: *not "is this rule enforced?" but "on how many paths does this
rule have to hold, and does the witness exercise all of them?"*

That question is what found C70, and C70 is why the method is written down here rather than assumed.

Three rules were applied to this review's own output:

1. **A finding is not real until it is reproduced.** Every claim below that says something is broken
   was run, not read.
2. **A finding that dissolves under checking is recorded as dissolved**, not quietly dropped. Three
   did (§3).
3. **The reviewer's own search is a gate, and gates go blind.** One finding in this review was wrong
   because a `grep` used the reviewer's vocabulary instead of the document's (§3.3).

---

## 2. What this review found that was new

### 2.1 C70 — a normative rule was false on the concurrency path (**closed**, D67)

`ref.rule.runtime.faults-are-diagnostics` names recursion depth explicitly and promises *"a diagnostic
with a code, never a host crash."* The reference marked it **covered**. It was false inside an actor.

```
down(1000) from fn main            → prints 1000, exit 0
down(1000) inside a behavior       → 0xC00000FD, no diagnostic
```

Windows brackets: abort above depth **43** (debug), between **300** and **400** (release), against a
documented bound of **10,000**. Forty-three frames is an ordinary recursive tree walk, and this needs
no embedder, no hostile input and no unusual configuration — only `spawn`.

Full account in `HARDENING_CAMPAIGN.md` C70 and ruling D67. The three reasons it survived two stages
are all previously-named patterns, which is the uncomfortable part: the safety rule lived at one site,
its witness exercised the one thread where it already held, and a stack overflow prints no
`panicked at` so every no-panic sweep was structurally blind.

**The rest of the rule was checked and holds.** Integer overflow inside a behavior is `DL0901` and the
actor dies cleanly, so the defect was specific to the one fault class that depends on the *host stack*
rather than on interpreter logic. Scope confirmed by test, not by argument.

### 2.2 A documented promise with no mechanism — crate publishability

`STABILITY.md` §2 states: *"Internal crate APIs. The Rust crates are an implementation detail; the
stable interface is the language, the CLI, and the machine schemas."* §6 is an explicit
**promise → mechanism** table, and this promise is not in it.

Measured at the time of the review: of thirteen crates, **one** set `publish = false`. The other
twelve defaulted to publishable at `version = "1.0.0"` — including `delulu-check`, which exposes
seventeen modules wholesale, and `delulu-runtime`, with 307 public items. `cargo publish -p
delulu-check` would have minted a 1.0.0 semver contract over internals the stability document
explicitly disclaims.

The mechanism already existed in the tree and had been applied to exactly the crate whose *name* made
it obvious. **Closed by D69** — but not before separating the two questions the field was answering
(§3.5), because the naive fix would have broken a published count.

### 2.3 Documentation that outlived its facts

Three statements that were true when written and are false now. Each verified against the source, not
against another document:

- `STAGE2_SPECIFICATION.md` — *"`delulu authority <dir>` still uses the single-package path (does not
  resolve cross-package imports)"*. Closed by D45a; `authority_package` selects `authority_workspace`
  on a manifest and resolves the graph.
- `STAGE6_BUILD_ORDER.md` deviation 3 — defers multi-module plugin packages because they *"would need
  a whole-program replay path (`check_program`)"*. `check_program` exists. The **status** is still
  accurate (the refusal is still in the code, deliberately); only its stated blocker is gone.
- `.github/workflows/ci.yml` — *"Flip to a hard gate at the 1.0 cut."* 1.0 shipped. The flip happened,
  but in the **test suite** (`release_requires_full_coverage`), not in the workflow, so the comment
  describes an unkept promise that was in fact kept somewhere better.

### 2.4 macOS: what the static audit can and cannot say

Every conditional-compilation site was enumerated and classified. **All are exhaustive for macOS**:
`cfg(unix)`/`cfg(not(unix))` and `cfg(windows)`/`cfg(not(windows))` pairs cover it, and the two
Linux-only mechanisms are handled deliberately — `PR_SET_PDEATHSIG` is `cfg(target_os = "linux")` with
a comment naming the macOS substitute (`WorkerGuard`'s kill-on-drop) and the possible future hardening
(kqueue `EVFILT_PROC`), and the microVM profile refuses with `DL1408` on the not-Linux arm.

**This is evidence of care, not evidence of working.** The one risk the audit *can* name concretely is
below (§4, macOS-1): the Unix domain socket path limit is **104 bytes on macOS** against 108 on Linux,
and macOS temp directories are long. Arithmetic on the longest state dir the test suite builds leaves
roughly fourteen bytes of margin. That is a real, unverified risk with a number attached, and it is
the first thing to check on the day a Mac exists.

---

### 2.5 The documentation is more complete than the brief assumed, and its one hole was a broken path

Asked to complete the specification, grammar, semantics, effects, capabilities, ownership,
diagnostics, package format, plugin architecture and Survey architecture, the audit's answer is that
**all of them already exist** — the package format in `STAGE2_SPECIFICATION.md`, plugins in
`STAGE6_PLUGINS_GUIDE.md`, ownership in `semantics-5-9`, capabilities in `semantics-5-4`, the Survey
in `docs/survey/README.md`, and sixteen generated semantics chapters behind a hard `--check-reference`
gate. Producing more prose over that would have been motion rather than work, and is recorded here as
a deliberate non-action.

**One thing was genuinely broken.** The normative EBNF is not in one document — it is
`STAGE1_SPECIFICATION.md` §3 plus each later stage's *grammar additions* section, 149 production
lines across eight files. That is defensible as history. What is not defensible is that
`docs/reference/grammar.md`, the chapter a reader opens looking for the grammar, was a bare index of
production **names taken from `parser.rs`** — and six of the twenty-seven are spelled differently in
the normative text. `ref.grammar.args` was a citable anchor, with both witnesses, and no
specification defined anything called `args`.

Closed by D70 rather than by rewriting the grammar, deliberately: **authoring a fresh consolidated
EBNF by hand risks shipping one that is wrong, which is C47's exact lesson** — a normative grammar
that could not describe `delulu fmt`'s own output. Relocating a correct grammar is safe; re-deriving
one from a 2,000-line recursive-descent parser is not, and would need its own phase and its own
differential evidence.

### 2.6 A tracked metric was not measuring what it named

The clippy baseline — quoted in five documents and treated as load-bearing across many phases
("clippy 65/0, the exact pre-federation baseline") — was produced by
`grep -cE '^warning:|^error:'`. That counter has **two independent faults**, and neither is subtle
once the number is asked to be reproducible:

1. It also matches cargo's per-crate **summary** lines (``warning: `delulu-wasm` (lib) generated 1
   warning``) — 13 to 18 of them, depending on how many crates and targets are linted. Those are
   totals, not findings.
2. A **warm** `cargo clippy` does not re-emit warnings for units it did not re-lint. The same tree
   measured **26 and then 42 within the hour**, which is what exposed this at all.

So a figure held constant across phases as evidence that "~4,000 added lines added zero warnings"
was a number whose value depended on cache state and whose units were wrong. Nothing unsafe followed
from it — **no finding was ever a `clippy::correctness` lint** — but a metric nobody can reproduce
cannot support the claim it was being used to support.

Measured properly (cold, isolated target dir, summary lines excluded) on one machine, at `0c98a58`
and at the working tree: **Windows 34 → 14, Linux 35 → 15.** Both platforms fell by exactly 20 and
the one-warning Linux surplus survived, which is evidence the old measure was *consistently* wrong
rather than randomly wrong. The dated historical figures are **left as recorded** with an annotation;
rewriting them would hide the mistake instead of fixing it.

This is design rule 2 in a new costume — *ask what signal a gate keys on, then what failure produces
a different signal.* Here the gate keyed on a line prefix, and two different things produce it.

## 3. Corrections — where earlier conclusions, including this review's, were wrong

### 3.1 C21 was filed under the wrong heading for its whole life

C21 was recorded as a **library-embedding** residual: a hypothetical embedder on a small stack. D51
closed it correctly by making the depth bound a contract (`with_max_depth`, `STACK_BYTES_PER_DEPTH`)
and the final ledger carried it as *"only a small-stack library EMBEDDING is uncovered."*

That framing was wrong, and it is why the real defect sat behind it. **D51 built exactly the right
mechanism and nothing in the tree called it.** The caller that needed it most was not an embedder at
all — it was DeluluLang's own actor runtime, reachable from the shipped CLI by an ordinary program.
A contract with no caller is a contract nobody is keeping.

### 3.2 CODEOWNERS: the previous disposition is reversed

Phase 5 recorded the entrenchment marker as *"belongs as a node attribute, not a verb … and it is not
built."* The reasoning for rejecting an `owners` **verb** was and remains correct — every rule names
the same placeholder, so the verb is a constant function. But the conclusion was applied to the wrong
thing: rejecting the verb was used to shelve the **attribute** too.

The attribute is a different claim. `.github/CODEOWNERS` names eight paths that require the project
lead specifically — the constitution, `DELULU_CORE.md`, `STABILITY.md`, `/rfcs/`, `SECURITY.md`,
`/docs/security/`, the soundness audit and its laundering suite, and the conformance machinery. That
is a *fact in the tree*, citable to a file and line, and it is precisely the signal an agent needs
before editing. The Survey exists primarily for AI systems maintaining DeluluLang; *"you may not
casually change this"* is among the most valuable things it could carry, and it carries nothing.

**Disposition: implement** (§4).

### 3.3 This review's own first reading of `STABILITY.md` was wrong

The first pass concluded that the stability contract *"says nothing about the Rust API surface,"* based
on a `grep` for `rust api`, `public api`, `crates.io`, `semver` and `publish`. The document says
**"Internal crate APIs"** and **"The Rust crates"** — the reviewer's vocabulary, not the document's.

The finding survived, in a sharper and smaller form (§2.2: the promise exists and has no mechanism),
but the process point is the one worth keeping: **a search is a gate, and the rule is to ask what
signal it keys on and what a real hit would look like if it used different words.** That is D47a's
lesson, applied to a reviewer instead of a test.

### 3.4 Two claims checked and found already honest

Recorded because a review that only reports problems is not a review:

- **CI's `|| true` on the coverage step is not a hole.** It looks like a disabled gate; the real gate
  is `release_requires_full_coverage` in the suite, which is stronger (per-commit, not per-push). Only
  the comment is stale.
- **The `macos-latest` CI matrix is not an overclaim.** `CROSS_PLATFORM_VERIFICATION.md` §"Nothing in
  this repository may describe DeluluLang as supported on three platforms" already states that the
  matrix names macOS and **has never executed**. The project got there first. *(2026-09-14: it has
  executed since, on a private testing remote, and macOS went green on it —
  `CROSS_PLATFORM_VERIFICATION.md` §9.)*

### 3.5 One Cargo field is already answering two different questions

Found while designing the fix for §2.2, and it changes that fix. `publish = false` was not inert:
`delulu-survey` read it to populate `tooling_crates` and derived `crates_shipped = 13 − 1 = 12` —
the number README quoted at the time, and a test gated. (Since D69 the Survey reads
`[package.metadata.delulu] surface` instead, and measures **9 shipped language crates plus 4
tooling**, which is what README quotes now. The `12` below is the pre-fix state, not a current count.)

So the field carries two questions at once:

- **Is this part of the language product?** — the Survey's reading, and the input to a published count.
- **May this be uploaded to crates.io?** — Cargo's meaning, and what §2.2 needs.

They agreed only because exactly one crate answered "no" to both. The library crates need **opposite**
answers — they *are* the language product, and `STABILITY.md` §2 says they are not a stable interface
— so naively adding `publish = false` to them would have driven the shipped-crate count to **1** and
failed its own gate.

That is a duplicate-concept defect, and it had to be untangled *before* either promise could be
enforced: one signal for product surface, one for publishability. **Closed by D69**, and the
untangling paid immediately — separating them showed the count had been *right by coincidence*. It
was derived as "thirteen minus the crates that say `publish = false`" and **labelled** "shipped
language crates", which was true only while the sole unpublishable crate was also the sole piece of
tooling. Asked directly, the tree says **nine** language crates and four that measure or map this
repository. A number computed from a proxy is a measurement waiting to be wrong, and this one was
wrong by three before anything moved.

---

## 4. The register

**Implement.**

| # | Item | Source | Why |
|---|---|---|---|
| 1 | Actor workers reserve an interpreter-sized stack; bound sized to match; thread-site gate | this review (C70) | **DONE — D67.** A normative rule was false on a shipped path |
| — | *Items 3, 4, 5 (in part) and 7 below shipped together as **D68**; see the ruling for what each closed.* | | |
| 2 | `publish = false` on every crate but the CLI, plus a gate over the partition | this review | Gives `STABILITY.md` §2 the mechanism its own §6 table demands |
| 3 | CODEOWNERS entrenchment as a Survey **node attribute** | reverses Phase 5 | The map's primary audience is agents; "do not casually change this" is a citable fact it lacks |
| 4 | Give Stage 6/7/8 decisions stage-qualified ruling ids, additively | Survey note ×3 | Those stages record real decisions as "Deviation *n*" — but **three stages each have a Deviation 3**, so the note is right that they cannot be cited. An index naming each existing deviation as `S6-D1`… makes them citable without renaming anything |
| 5 | Re-verify and correct every quoted test count | Survey note ×1 | The Survey deliberately will not guess; a reviewer can measure. The front door is where staleness costs most |
| 6 | Correct the three statements in §2.3 | this review | **DONE — D70.** All three corrected in place rather than deleted, so the record still reads as a record |
| 10 | Make the reference's grammar index reach the grammar | this review (§2.5) | **DONE — D70.** Six of twenty-seven anchors named productions no specification defined; a three-way gate now holds the correspondence |
| 7 | Name the macOS socket-path limit in a diagnostic rather than surfacing a raw OS error | this review (macOS-1) | Testable on Linux today; turns an obscure failure into a named one on the day a Mac exists. **That day came on 2026-09-14 (CI run 2):** the named refusal fired, but inside the detached broker daemon, so it reached only `broker.log` and the user saw a five-second timeout; `broker start` now runs the same check before spawning (`28e10e6`) |
| 8 | Decompose `cli.rs` along the seams already established | this review | **PARTLY DONE — D69.** `delulu run` (1,116 lines) extracted after measuring the coupling: 24 of 143 items reached, sixteen of them its own. `cli.rs` 8,856 → 7,740. **Shared helpers deliberately stayed** — moving them would assert a false owner. The remaining clusters (broker-facing commands ≈1,050 lines; authority reporting ≈800) are characterised and mechanical |
| 9 | Separate "not shipped language surface" from "not publishable" before enforcing either | this review (§3.5) | **DONE — D69**, and it corrected a published count that had been right by coincidence |

**Reject, with reasons.**

| Item | Reason |
|---|---|
| A Survey `why <id>` verb | Confirmed by inspection: a strict subset of what `query` already returns. Building it would add a second, narrower answer to a question already answered |
| A Survey `owners <id>` verb | Every CODEOWNERS rule names the same placeholder until public launch, so the verb is a constant function. The *entrenchment* half is real and is item 3 — the verb is not |
| Suppress the `c-token-not-a-campaign-finding` note | **This review proposed it, then read the code and withdrew it.** The check is already deliberate: it counts an unresolved `C<n>`, never errors, and its own guidance reads *"no action if these are C-language references."* Its only trigger today is `C99`, which appears exactly once in the tree — inside the Survey's **own source comment explaining why `C99` is not a finding**. An allowlist would be strictly worse: **`C11` and `C17` *are* real campaign findings**, so a number-keyed list is wrong, and a context-keyed one is guessing, which is the thing the provenance law exists to forbid |
| Move `Effect` out of `delulu-check` to drop the broker→check edge | `delulu-broker` uses exactly one item from `delulu-check`: `Effect`. That looks like accidental coupling and is the opposite. Authority is *defined* in terms of effects; one shared definition between the crate that computes rows and the crate that grants them is the "one list referenced by both sides" pattern this project adopted after six drift findings. Relocating it buys build-graph tidiness and risks the exact drift the pattern prevents |
| Raise `DEFAULT_MAX_DEPTH` now that workers have real stacks | The bound is a published contract and 10,000 is not the constraint anyone hits. Changing it would move an observable limit for no demonstrated need |
| Split `cli.rs` all the way down | Measured, then stopped. `cmd_run` was a real seam — 24 of 143 items reached, sixteen of them its own. The scattered helpers are **shared** with `grants`/`guard`, and relocating shared code into one command's module asserts an ownership that is not true. A file-size target is not an architecture; the coupling measurement is what says where to stop |
| Collapse `PluginEngine` because it has one implementor | A single-implementor trait is usually an unnecessary abstraction. Not here: it lives in `delulu-runtime` and is implemented in `delulu-wasm`, which depends on the runtime. The trait exists to **invert that dependency**, and removing it would create a cycle. Verified against the dependency graph, not assumed |

**Postpone / accept as a standing limit.**

| Item | Status |
|---|---|
| C55 — record field access is O(record width) at runtime | **Accepted limit**, published with its curve. Measured U-shaped with a minimum at width 20; a short `Vec` scan is genuinely faster for ordinary records. The real fix is static field indices through the DIR — a change to the IR, not a patch |
| macOS execution | **Unblocked 2026-09-14, by CI rather than hardware:** the private testing remote's `macos-latest` runner passed the whole suite end to end in run 3 (`CROSS_PLATFORM_VERIFICATION.md` §9). Still unrun: a developer's Mac. *(Was: blocked, not deferred — there was no Apple hardware, and every macOS cell said "never run" rather than "untested" or "pending", because those invite a reader to assume someone tried.)* |
| The optimizer (spec §2.1) and a native backend | **Honestly deferred**, RFC-gated, with published notes. Neither is claimed to exist |
| Multi-threaded WASM engine | **Deferred with its honesty note** (`THREADED_WASM_DEFERRAL.md`) — the sanctioned passing outcome, not an omission |
