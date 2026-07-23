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
| C4 | Front door — `README.md` describes a project that no longer exists | **high** (adoption) | **CLOSED** — D25 |
| C5 | `REPOSITORY_STRUCTURE.md` — the repository map has drifted from the repository | medium (accuracy) | **CLOSED** — D25 |
| C6 | **The documented surface is a subset of the real one** — working constructs are untaught | **high** (adoption) | **CLOSED** — D25 |
| C7 | The capability corpus is 8 programs; `tier4-multimodule` has none | medium (evidence) | OPEN |
| C8 | The interpreter's recursion bound is fixed at 10,000 and appears in no user-facing document | medium (usability) | **CLOSED** (documented) — D25 |
| C9 | **There is no LICENSE** — nobody may legally use the project | **high** (adoption) | OPEN — owner decision |
| C10 | Runtime — DL0703 refused without naming the grant that would fix it | medium (usability) | **CLOSED** — D25 |
| C11 | Checker — a user function silently loses to a same-named prelude builtin | **high** (correctness) | **CLOSED** — D25 |
| C12 | Diagnostics — `DL0401` prints type *variables* where the type names are known | medium (usability) | OPEN |
| C13 | **Runtime — a named function used as a value checks clean and faults at runtime** | **high** (correctness) | **CLOSED** — D25 |
| C14 | `DL0907`'s registry text is narrower than the conditions that raise it | low (accuracy) | OPEN |
| C15 | `delulu fmt` deletes the blank line between two comment paragraphs before an item | medium (fidelity) | OPEN — P9 |

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

### C6 · The documented surface is a strict subset of the real one — OPEN

**This is P0's headline finding, and it reframes the campaign.** The breadth sweep wrote one
genuine program per domain — beginner, DSA, compiler, LLM client, robotics control loop, OS-style
file walker, actor pool, SaaS backend — using only what the Book, the samples, the examples, and
the reference teach. Most did not compile. Every failure looked like a language limitation. **Not
one of them was.** In each case the language supported the construct perfectly well under a syntax
or a rule that no user-facing document mentions:

| What a reader writes from the docs | What the language actually wants | Where the real form appears |
|---|---|---|
| `type Tree { Leaf, Node(Tree, Int, Tree) }` | `type Tree = Leaf \| Node(Tree, Int, Tree)` | conformance corpus only |
| `let v = xs.get(i)` then use `v` | `match xs.get(i) { Some(v) => …, None => … }` — `List.get` is total and returns `Option[T]` | conformance corpus only |
| `be dispatch(w: tag Worker) { w.job(1) }` | `be dispatch(w: tag Worker) ! {Async} { … }` — sending from a behavior is an effect and must be declared | nowhere |
| `net: Cap[Net]` | `Cap[Http]` — `Net` is the *effect*; `Http` is the *resource kind* | nowhere |

Rewritten against the real surface, the same programs check clean, and the DSA program runs. So
the capability is there and the wall is the documentation.

That distinction matters for how this is fixed. Sum types are the language's central
data-modelling construct — `match` on a sum is how every error in the language is handled — and
they appear in **no** Book chapter, **no** sample, **no** example, and **no** reference page. They
exist only in `tests/conformance/`, which is not a place a user reads. The effect/resource split
(`Net` vs `Http`) is a genuinely subtle and *deliberate* piece of the design — constitution
invariant 6, "kind is static, scope is runtime" — and a reader who guesses wrong gets DL0307 with
no pointer to the rule they violated.

The lesson generalizes past this list: **the project tested its documentation for accuracy and
never for sufficiency.** Every documented claim is true. A developer cannot get from them to a
working program.

### C7 · The capability corpus is eight programs — OPEN

`tests/corpus/` is described in `REPOSITORY_STRUCTURE.md` as "coding-capability tiers (simple →
security-expert)". It contains **eight programs**: two simple, two DSA, one application, two
security — and `tier4-multimodule/` holds a `NOTE.md` and **no program at all**. For a language
proposing itself for robotics, satellites, SaaS backends, and enterprise systems, this is not
enough evidence to support the proposal, independent of whether the language is capable.

### C8 · The recursion bound is fixed at 10,000 and is undocumented — OPEN

`MAX_DEPTH = 10_000` in `crates/delulu-runtime/src/interp.rs`. Exceeding it is **DL0905**, an
honest named diagnostic with exit 1 and no host crash — the behaviour is correct and was
deliberately fixed once already (Stage 9, D15, after a real host crash). Two things remain open:
the bound is **not configurable**, and it is named in no user-facing document — only in build-order
records, which are internal process history. A recursive descent over a 10,000-element structure is
an ordinary thing to write; the author should be able to find the limit before meeting it.

## 3.1 What held up under the sweep

Recorded with the same weight as the defects, because a campaign that only lists failures
misrepresents the system.

- **Scaling is linear, with no O(n²) anywhere in `check`.** Measured on generated programs with
  real dispatch chains and interdependent functions: 1,220 lines 0.20 s · 6,020 0.37 s · 12,020
  0.60 s · 30,020 1.40 s · 60,020 2.75 s · **120,020 lines 5.75 s**, peak RSS 676 MB. That is a
  **debug** binary; release was not measured, so these are upper bounds. The memory figure is worth
  watching for constrained CI, and is the only scale number here that suggests future work.
- **Arithmetic is checked, not wrapping.** Integer overflow on `+`, `-`, and `*` is **DL0901**;
  division and modulo by zero are **DL0902**. Every case aborts with exit 1 and a named code. This
  is a stronger default than C or than Rust in release mode.
- **Deep recursion refuses honestly** — DL0905, exit 1, never a raw stack overflow (see C8).
- **String indexing is character-based, not byte-based.** `"héllo".len()` is 5, `"🐦".len()` is 1,
  `"日本語".slice(0,1)` is `"日"`. No slice can land mid-codepoint or produce invalid UTF-8 — an
  entire class of crash is absent by construction.
- **The parser is robust under abuse.** 20,000 top-level functions in 0.84 s; 3,000 nested blocks
  in 3.1 s with no stack overflow; 2,000 nested generic types in 0.73 s; a 1 MB string literal in
  0.27 s; a 100,000-token expression in 0.93 s. Non-ASCII identifiers are refused by DL0101, which
  closes the confusable-identifier attack **by construction** rather than by a lint.
- **Zero ambient authority is real and immediate.** Every sweep program that touched the console
  failed at runtime with DL0703 until `--grant console` was passed. The central claim of the
  language is enforced on the first program anyone writes.

One inconsistency worth naming rather than filing as a defect: **out-of-range access is handled two
different ways.** `List.get` is total and returns `Option[T]`, making failure visible and forcing
the caller to decide. `Str.slice` silently clamps — `"abc".slice(0,99)` is `"abc"`, `"abc".slice(2,1)`
is `""`, `"abc".slice(-1,2)` is `"ab"`. Both are defined, neither is undefined behaviour, and the
clamping form is what most languages do. But a negative index reaching `slice` produces a
plausible-looking answer instead of a signal, in a language that chose the opposite convention one
method earlier. Settling this is a design decision, not a bug fix, and it is recorded here for
that decision rather than being changed unilaterally.

### C9 · There is no LICENSE — OPEN, and it is the owner's decision

No `LICENSE` file exists, and `Cargo.toml` declares no `license` field. Under default copyright
that means **all rights reserved**: nobody but the copyright holder may use, copy, modify, or
distribute this code. Measured against the campaign's own objective — that anyone should be able
to download, build, install, and use DeluluLang in production — this is the single hardest
blocker, and no amount of engineering moves it.

It is recorded and deliberately **not fixed**. Choosing a licence is a legal commitment with real
consequences (patent grants, copyleft reach, contributor terms) and it belongs to the copyright
holder alone. The README now states the situation plainly instead of leaving a reader to discover
it. Related and smaller: `Cargo.toml` names a `repository` URL that is not published, because this
project is never pushed by owner policy; the README no longer implies a download exists.

### C11 · A user function silently loses to a same-named prelude builtin — CLOSED (D25)

Found by writing a guide, not by reading code. A program declaring `fn parse_int(s: Str) ->
Result[Int, ParseErr]` **checked clean in isolation**. Every *call* to it silently resolved to the
prelude builtin `parse_int`, which returns `Option[Int]`, so the only symptom was a type error at
a **call site**, naming `Option` — a type the author never wrote — for a function they had
declared as returning `Result`. Nothing anywhere named the collision.

That is the worst shape a diagnostic can take: correct, distant, and about the wrong thing. It
took a dozen bisection steps to find in a codebase already well understood; a newcomer would
simply conclude the language was broken.

Builtins are intercepted at the call site before user scope, so the fix refuses the declaration
(`DL0302`, an existing code — no new code, so the machine surface and the stability contract are
untouched) and says why: *"calls resolve to the builtin before user scope, so this definition
would never be called — rename it."* Refusing is the right resolution rather than letting the
user's definition win, which would make a call mean different things in different modules.

**A latent host panic came out with it.** `check_fn` looked its signature up with
`.expect("fn in table")`. Refusing to register a name therefore turned a bad program into a
process crash — the invariant "every `Item::Fn` is in the table" was held by nothing but
`resolve.rs` never skipping registration, and the very first skip found it. It now returns
gracefully, because resolve has already reported the reason. A host panic is never an acceptable
answer to a bad program (Stage 9, D15, made the same point about DL0905).

**Witnesses.** Three, in `crates/delulu-check/src/lib.rs`. The definitional refusal; a loop
asserting every name in `PRELUDE_BUILTINS` is refused *without panicking*; and a drift guard
asserting `PRELUDE_BUILTINS` equals the set of names `check_builtin_call` actually intercepts —
because a name in one list and not the other is either un-refusable or un-callable, which is the
defect this pair exists to prevent.

### C13 · A named function used as a value checks clean and faults at runtime — CLOSED (D25)

`apply(double, 21)` — the canonical row-polymorphism example, and the shape of every higher-order
call — **type-checks correctly and then dies at runtime** with `DL0907: unbound name 'double'`.
A *lambda* in the same position always worked; only a named top-level function was missing from
`eval_var`, which resolved local bindings and capitalized nullary variants and then gave up.

This is a checker/runtime divergence, the most serious class in this campaign so far: the type
system accepted a program the interpreter could not execute. Row polymorphism exists specifically
to make higher-order code expressible, and `SOUNDNESS_AUDIT.md` §C examines function values as
values and concludes the channel is closed — the analysis is right about the *types* and the
runtime simply did not implement the case.

**Why nobody noticed, which is the more important finding.** `docs/book/samples/04_row_polymorphism.delulu`
has exactly this shape. It is covered by `criterion7_every_book_sample_checks_clean`, which
*checks* every sample on every CI run — and never runs one. The sample also has no `fn main`, so
it could not have been run without being rewritten. A gate that only checks proves the program is
well-typed and says nothing about whether it works.

**Fix.** A named function now evaluates to a closure over the globals — precisely the environment
`call_fn` builds for a direct call — so calling it through a value and calling it by name are the
same computation.

**Gate.** A new `crates/delulu/tests/examples_run.rs` adds the missing half: every shipped example
checks clean, **and** every one with a `fn main` is run, asserting it never fails with `DL0907` —
the code the runtime raises when the checker let something through. The assertion is deliberately
narrow: other runtime outcomes (an ungranted capability, a missing file) are legitimate and the
test says nothing about them. Verified by removing the fix and observing the gate fail by name,
along with all three unit witnesses, then restoring it.

### C12 · `DL0401` prints type variables where the names are known — OPEN

Passing a `Cap[Http]` result to a function declared `Result[Str, IoErr]` reports:

```
expected `Result[Str, T0]`, found `Result[Str, T1]`
```

The real answer is `IoErr` versus `NetErr`, and the checker knows both — `T0` and `T1` are
internal identifiers for builtin sums that the type printer does not resolve back to names. The
diagnostic is accurate and useless: it names the shape of the disagreement and hides its content.

### C15 · The formatter merges comment paragraphs — OPEN (deferred to P9)

`delulu fmt` deletes the blank line separating two comment blocks that precede an item:

```delulu
// A file-level note about the whole module.
// It is its own paragraph.

// A note about THIS function specifically.
fn f() -> Int { 1 }
```

becomes a single five-line block with no separation between the module-level note and the
function-level one. "One canonical style, zero options" is a good decision and this is not an
argument against it — but canonicalization should not destroy authored structure, and every
comparable formatter (rustfmt, gofmt, prettier, black) preserves blank lines between comment
paragraphs. Found while writing `examples/guide/`, where it forced comments to be restructured
around the tool rather than for the reader.

Deferred to P9 (Stage 8, Surface) rather than fixed here: changing canonical output means
re-formatting the shipped corpus and re-establishing `fmt`'s laws, which deserves its own pass.

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
