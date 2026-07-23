# The Hardening Campaign — testing DeluluLang to failure

**Status:** OPEN. Started 2026-07-24 on `c823037` (v1.0.0 + Stage 10 closed + rulings D1–D23).
**Governing documents (precedence):** `CONSTITUTION.md` > `SOUNDNESS_AUDIT.md` > the stage
specifications > `STAGE10_BUILD_ORDER.md` (rulings) > this document (campaign method and ledger).

This is a **multi-week program**, not a phase. Its premise, in the commissioner's words: *even NASA
tests new rocket engines until they fail.* Stages 1–10 were each built to their specification and
each closed against acceptance criteria written by the same people who wrote the code. That
produces a system that passes its own exam. It does not produce evidence about what happens when
someone who is *trying to break it* shows up — or, more mundanely, when someone who has never read
the specification sits down and writes the program they came to write.

The campaign's object is to move DeluluLang from "released and internally consistent" to
**"a language a stranger can download, build, install, and run production systems on."**

## 1. Method

1. **A finding is not real until a witness fails against the old code.** Write the test, revert the
   fix, *observe* the exact expected failures, restore. Findings that were only reasoned about are
   labelled as such and never counted as closed.
2. **Breadth before depth.** Fire the engine and see what melts, then go stage by stage. The
   per-stage passes are steered by observed failures, not by re-reading specifications top to
   bottom — a specification review finds what its author already thought of.
3. **Fix the class, not the instance.** Every defect gets the question "how many more of these are
   there, and what makes the next one impossible?" A fix that only closes the reported case is
   half a fix.
4. **Honesty over green.** A named gap outranks a passing badge. Anything that cannot be fixed in
   this campaign is written down here in the terms a user would need to make their own decision.
5. **Docs move with code, in the same commit.** A behaviour change whose documentation lands later
   is a behaviour change that shipped undocumented.

### 1.1 What "push it until it breaks" means operationally

For each area: enumerate the invariant, write the program that a working developer would actually
write, run it, and then write the program an adversary would write. Record both. The categories
under test are deliberately the full range a real language meets — beginner programs, DSA,
compilers, AI/LLM clients, robotics control loops, networking, OS-style tools, concurrency and
distribution, SaaS and enterprise backends, monorepos, 10k–30k+ line codebases, fuzz corpora,
malicious input, performance and memory pressure, and the authority and Guard models themselves.

## 2. Baseline (2026-07-24, `c823037`, Windows x86_64, rustc 1.96.1)

Every number below was produced by one scripted replay of the CI gates, sequentially and
uncontended. This is the line every later claim is measured against.

| Gate | Result |
|---|---|
| `cargo test --workspace` | **92 suites, 1179 passed, 0 failed, 4 ignored** — exit 0 |
| `delulu fmt --check examples` | exit 0 |
| `delulu fmt --check docs/book/samples` | exit 0 |
| `cargo check -p delulu --no-default-features` (Python-less) | exit 0 |
| `delulu-conform --coverage` | exit 0 (100%) |
| `delulu-conform --check-reference` | exit 0 |
| `cargo clippy --workspace --all-targets` | exit 0, **65 warnings** (tracked baseline, not a gate) |

Workspace size: **~82,000 lines of Rust** across 12 crates, 175 files. The repository is *not*
rustfmt-formatted under default settings and has no `rustfmt.toml`; house style is wider than
rustfmt's default, and `cargo fmt` is therefore **not** a gate. Do not run it.

## 3. Findings ledger

Findings are numbered `C<n>` in this campaign's namespace. Fixes that change behaviour also get a
`D<n>` ruling in `STAGE10_BUILD_ORDER.md`, which remains the single ledger of record for
deviations.

| # | Area | Severity | State |
|---|---|---|---|
| C1 | Stage 1 — two unbounded parser loops | **high** (availability) | **CLOSED** — D24 |
| C2 | CLI — `--json` emits no object on a read failure | medium (machine contract) | OPEN |
| C3 | Stage 1 — bidirectional-override source is accepted silently | medium (review integrity) | OPEN |
| C4 | Front door — `README.md` describes a project that no longer exists | **high** (adoption) | OPEN |
| C5 | `REPOSITORY_STRUCTURE.md` — the repository map has drifted from the repository | medium (accuracy) | OPEN |

### C1 · Two unbounded loops in the Stage-1 parser — CLOSED (ruling D24)

Both were reached from programs a person would plausibly type. Both survived Stage 1 through
Stage 10, the v1.0.0 release, and 23 rulings.

**C1a — `match` arm with an unbraced assignment.** Six lines:

```delulu
module m
fn f(flag: Bool) {
    var n = 0
    match flag {
        true => n = 1
        false => n = 2
    }
}
```

An arm body is an *expression*; assignment is a *statement*. `parse_expr` therefore stopped dead at
`=`, and nothing else in the arm loop consumed it: `expect` reports its diagnostic and deliberately
does **not** advance. The loop's guard condition was unchanged, so it ran again — pushing one more
`Arm` per iteration. Measured on the reference machine: **CPU pegged, resident memory 650 MB → 1.16
GB in three seconds**, roughly 380 MB/s, until the machine ran out. `delulu check` is a static
analysis with no execution and no authority; that a six-line file turns it into a memory-exhaustion
DoS matters most for the population the language is aimed at, which submits code it did not write.

**C1b — `import` after the first item.** Three lines:

```delulu
module m
fn f() {}
import b
```

`parse_item` returns `None` without consuming for any token that does not start an item, and the
resynchronizer `recover_item` deliberately stops **at** `import` — so the position never moved.
This one allocates nothing: it is a silent spin that pegs one core forever, with no output and
nothing to notice.

**Why this is a class, not two bugs.** The parser's author knew this hazard exactly and defended
against it *four separate times* with the same hand-written idiom:

```rust
let before = self.pos;
… parse one element …
if self.pos == before {
    // No progress — force one to avoid an infinite loop.
    self.bump();
}
```

It appears in the foreign-block loop, the actor-body loop, and the statement-block loop, and
`recover_item` bumps for the same reason. The two loops that lacked it are the two that hung. This
is the project's own `skip-branch` lesson in its purest form: the rule was known, written down, and
applied — and the defect lived in the one place it was not.

**Fix.** The guard now exists in exactly one place, `Parser::parse_until`, and every
brace-delimited list goes through it. It is no longer possible to write a new list loop and forget
the guard, because there is no longer a hand-written list loop to copy. Progress is guaranteed
structurally: if a step consumes nothing, the loop consumes one token on its behalf, so iteration
count is bounded by token count.

Two user-facing diagnostics were sharpened at the same time, because both defects were reached by
*plausible* code and the generic message sent the reader to the wrong line:

- an assignment in an arm now reports **"a match arm's body is an expression — assignment is a
  statement"** and carries an exact repair (`brace-match-arm-assignment`) that wraps it in a block;
- a misplaced import now reports **"`import` must appear before the first item, directly under the
  `module` header"** instead of "expected an item", which is true but useless.

**Witnesses.** Three, in `crates/delulu-syntax/src/parser.rs`. Two pin the fixed behaviour. The
third is the one that matters:
`every_delimited_list_loop_goes_through_the_progress_guard` scans the parser source and asserts
that exactly one such loop exists — the one inside `parse_until`. A reintroduced hand-rolled loop
makes it **fail**, not hang, which is the difference between CI telling you something and CI
telling you nothing. It earned its keep immediately: on first run it caught an occurrence in a
comment that a manual `grep` had missed.

**Verification against the old code.** Both hangs were reproduced and *observed* before the fix —
`rc=124` at an 8-second timeout, with CPU and resident-memory growth sampled during the run for
C1a — and both terminate with the correct diagnostic after it.

**What this finding does not show.** The parser is otherwise robust, and that deserves saying
plainly. Under the same corpus: 20,000 top-level functions checked in 0.84 s; 3,000 nested blocks
in 3.1 s with no stack overflow; 2,000 nested generic types in 0.73 s; a 1 MB string literal in
0.27 s; a 100,000-token single expression in 0.93 s. Non-ASCII identifiers are refused by DL0101,
which closes the confusable-identifier attack by construction. The two hangs were real and are
fixed; they were not symptoms of a fragile front end.

### C2 · `--json` emits no object when the input cannot be read — OPEN

`docs/for-agents.md` states "Every `--json` command emits one object" and calls the JSON envelope
"the contract". When the input file cannot be read — missing, a directory, or not valid UTF-8 —
every such site instead prints a plain-text line to **stderr** and exits **2**, with no object on
stdout, whether or not `--json` was passed. Roughly ten call sites in `crates/delulu/src/cli.rs`
behave this way, so this is a uniform design choice rather than an oversight.

The practical impact is bounded — stdout is *empty* rather than malformed, and the documentation
tells agents to read the exit code first — but the promise as written is not kept, and an agent
that follows the documented contract must special-case a condition the contract says cannot occur.

Two sub-questions to settle when this is fixed, not before:

1. whether the envelope should be emitted (making the promise true) or the promise narrowed
   (making the documentation true) — the first is additive and is what the docs already claim;
2. whether **exit 2** is right for a file that exists but is not UTF-8. Exit 2 is documented as
   "your invocation was wrong; the program was never examined". The second clause is true; the
   first is arguable. Exit 2 for a *missing* file is clearly right, and consistency has value.

### C3 · Bidirectional-override characters are accepted silently — OPEN

A source file containing U+202E RIGHT-TO-LEFT OVERRIDE inside a comment checks **clean, exit 0**.
This is the Trojan Source class (CVE-2021-42574): the bytes a compiler parses and the glyphs a
reviewer sees can be made to disagree, so a change can read as innocuous while doing something
else. Rust, Go, and others made this deny-by-default after 2021.

The exposure here is narrower than in most languages — identifiers are ASCII-only (DL0101), so the
attack cannot hide inside a name — which leaves comments and string literals. That is still the
whole of it: this project's stated purpose includes *humans reviewing AI-written code*, and a file
that renders differently than it parses attacks review directly. A language whose value proposition
is legible authority should not accept source whose legibility can be inverted invisibly.

### C4 · The front door describes a project that no longer exists — OPEN

`README.md`, on a tree tagged **v1.0.0** with Stage 10 closed and 23 rulings, says:

> Stage 1 ("Skeleton") — under construction. The design is complete and committed

and instructs the reader to `rustup default stable`, against a `rust-toolchain.toml` that pins
**1.96.1** precisely so that builds are reproducible. There is no install section, no first-program
walkthrough, no packaging or deployment guidance, and `docs/reference/cli.md` is a generated
25-row coverage table rather than a usage reference — it names every subcommand and documents the
behaviour of none.

This is recorded as a **high** severity finding rather than a documentation chore. The project's
honesty discipline is real and unusually rigorous, but it was aimed inward at specifications and
outward at capability claims, and it skipped the one page every new user reads first. Correctness
in the core does not survive contact with users if the front door is false.

### C5 · The repository map has drifted from the repository — OPEN

`docs/REPOSITORY_STRUCTURE.md` opens with "**this document is the map of the repository**". It
currently maps a different one.

**Claimed, and absent:**

- `stdlib/std/{core,fs,net,io}.delulu` — "the DeluluLang standard library, in DeluluLang". **There
  is no standard library.** No `.delulu` file matching it exists anywhere in the tree. This is the
  consequential one: a reader deciding whether to adopt the language reads that line as "there is
  a standard library", and the answer is no.
- `tests/laundering/` — "SOUNDNESS_AUDIT.md F-1…F-6, R-7 — permanent rejection tests". The
  directory does not exist. **The tests do**, at `crates/delulu-check/tests/laundering.rs`, so the
  soundness obligations of §E are met and only the path is wrong. Stated explicitly because the
  alarming reading — that the audit's rejection tests were never written — is false.

**Present, and unmapped:** `rfcs/`, `release-artifacts/`, `SECURITY.md`, `CONTRIBUTING.md`,
`docs/release/`, `docs/for-agents.md`, `docs/editors.md`, `measurements/METHODOLOGY.md`,
`.github/`.

The map also predates the crates added after Stage 1: `delulu-wasm`, `delulu-fuzz`,
`delulu-registry`, `delulu-conform`, and `delulu-measure` do not appear in the tree it draws.

## 4. Phase plan

| # | Phase | Covers |
|---|---|---|
| P0 | Baseline, gate harness, breadth sweep | §2 above; the sweep that produced C1–C4 |
| P1 | Front door | C4: download, build, install, configure, CLI, first program, packaging, deployment |
| P2–P11 | Stage 1 → Stage 10 adversarial passes | one phase per stage, steered by sweep results |
| P12 | Scale | 10k–30k+ line programs, monorepos, compile-time and memory behaviour |
| P13 | Fuzz, malicious input, security | hostile programs, packages, plugins, adapters, certificates |
| P14 | Performance and memory pressure | against `measurements/METHODOLOGY.md` |
| P15 | Authority + Guard cross-stage audit | the capstone: consistency across all ten stages at once |
| P16 | Cross-platform re-verification and close-out | Windows, Linux, and an honest statement about macOS |

Phase state is tracked in the working session, not here; this table is the shape, and §3 is the
durable record.

## 5. Standing limits this campaign does not remove

Carried forward and restated so that no reader of this document alone concludes otherwise:
**macOS has never been executed**, no driver for any real device ships in-tree, certification is
**none** under every regime, `ForeignCall` remains an enumerated hole in the proof rather than a
closed one, and RFC 0001's comment period remains open until 2026-08-05 with two recorded process
deviations against it.
