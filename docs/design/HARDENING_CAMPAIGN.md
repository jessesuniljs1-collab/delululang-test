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
| C2 | **CLI — `--json` emitted NO object on failure, across essentially every subcommand** (filed as one case; it was the whole surface) | **high** (machine contract) | **CLOSED** — D38 |
| C3 | Stage 1 — bidirectional-override source is accepted silently | **high** (review integrity) | **CLOSED** — D26 |
| C4 | Front door — `README.md` describes a project that no longer exists | **high** (adoption) | **CLOSED** — D25 |
| C5 | `REPOSITORY_STRUCTURE.md` — the repository map has drifted from the repository | medium (accuracy) | **CLOSED** — D25 |
| C6 | **The documented surface is a subset of the real one** — working constructs are untaught | **high** (adoption) | **CLOSED** — D25 |
| C7 | The capability corpus is 8 programs; `tier4-multimodule` has none | medium (evidence) | OPEN |
| C8 | The interpreter's recursion bound is fixed at 10,000 and appears in no user-facing document | medium (usability) | **CLOSED** (documented) — D25 |
| C9 | **There is no LICENSE** — nobody may legally use the project | **high** (adoption) | **CLOSED** — D27 (owner-approved) |
| C10 | Runtime — DL0703 refused without naming the grant that would fix it | medium (usability) | **CLOSED** — D25 |
| C11 | Checker — a user function silently loses to a same-named prelude builtin | **high** (correctness) | **CLOSED** — D25 |
| C12 | Diagnostics — `DL0401` prints type *variables* where the type names are known | medium (usability) | OPEN |
| C13 | **Runtime — a named function used as a value checks clean and faults at runtime** | **high** (correctness) | **CLOSED** — D25 |
| C14 | `DL0907`'s registry text is narrower than the conditions that raise it | low (accuracy) | OPEN |
| C15 | **`delulu fmt` deleted the blank line between two comment paragraphs**, merging them — and the identity law could not see it | medium (fidelity) | **CLOSED** — D41 |
| C16 | Checker — a cyclic type alias (`type A = A`) is silently accepted | low (hygiene) | OPEN — P2 return |
| C17 | Lexer — a float literal that overflows to `inf` is accepted without a warning | low (honesty) | OPEN |
| C18 | **Stage 2 — the semver-authority law and `authority --diff` were blind to secret-scope widening** | **high** (supply chain) | **CLOSED** — D28 |
| C19 | **Stage 2 — the dependency pin (DL1001) and self-declaration (DL1009) do not enforce secrets** | **high** (supply chain) | **CLOSED** — D34 (owner approved 2026-07-25) |
| C20 | **Stage 3 — the two engines disagree on fault codes, and a WASM trap dumps a ~16k-line backtrace** | **high** (parity/usability) | **CLOSED** — D29 |
| C21 | Runtime — the interpreter's `MAX_DEPTH=10000` overflows the host stack below ~20 MiB (small-stack embeddings) | medium (robustness) | OPEN |
| C22 | **Stage 8 — the syntax-morph system is specified normatively and does not exist**; its spec claims to be implemented | **high** (doc contradiction / missing commissioned feature) | **CLOSED** — D35 (built 2026-07-25) |
| C23 | **Every builtin type name and core effect name can be shadowed by a user declaration, and the shadow is silently inert** — including `Root`, `Cap`, `Secret`, `Plugin`, and `Write` | **high** (review integrity / authority legibility) | **CLOSED** — D30 |
| C24 | Stage 4 — a type alias in a foreign signature was refused as "not marshallable" without saying it was an alias or what it aliased | medium (diagnostic quality) | **CLOSED** — D31 |
| C25 | **The authority report holds every fact needed to see that a credential can leave the program, and never says so** | **high** (the commission's credential-exposure requirement) | **CLOSED** — D32 |
| C26 | **A package whose sources are not under `src/` reports `built clean (0 module(s))` and exits 0** — nothing checked, success claimed | **high** (silent success) | **CLOSED** — D33 |
| C27 | Naming a directory where a file belongs surfaced the raw OS error (`Access is denied. (os error 5)` on Windows) | medium (diagnostic confusion) | **CLOSED** — D33 |
| C29 | **A lease token for a REVOKED grant redeemed successfully** — and for a revoked or expired *ancestor* too; the audit log recorded `decision: "allow"` for it | **high** (accountability / fail-open at the custody boundary) | **CLOSED** — D36 |
| C30 | **`audit tail`/`query` displayed a tampered chain as authentic, and a corrupted record VANISHED from the listing** with no gap marker | **high** (accountability — the omission attack) | **CLOSED** — D36 |
| C31 | **The `device` scope dimension had no Guard class** — actuation was gateable only all-or-nothing via `effect:Actuate`, on the one axis that moves hardware | **high** (safety granularity) | **CLOSED** — D37 |
| C32 | **A 10 KB source file emitted 76 MB of diagnostics** — every diagnostic quoted its entire source line, times ~5000 errors | **high** (denial of service against the reader) | **CLOSED** — D38 |
| C33 | `deploy` and `fleet` are working top-level subcommands that `--help` never listed; `deploy` also double-emitted JSON on refusal | medium (discoverability / machine contract) | **CLOSED** — D38 |
| C34 | Stage 6 — a plugin manifest could declare `device`/`foreign_c`/`foreign_python` authority and have it **silently dropped**, advertising a ceiling the plugin can never have | medium (legibility — the C23 class, one level out) | **CLOSED** — D39 |
| C35 | Stage 7 — a `Root` slice **silently loses `computes`** when it crosses an actor boundary; `RootMsg` is a hand-written enumeration that Stage 10 phase 10h did not extend | medium (silent narrowing — fail-closed but undecided) | **CLOSED** — D40 (gated + documented; carrying it is a capability decision for the owner) |
| C36 | `atlas --format mermaid` is a module-level overview that did not say so — a reader could conclude a program has no functions or effects | medium (legibility of the authority graph) | **CLOSED** — D41 |
| C28 | **`type A = B` is ambiguous in the normative grammar** — it matches both the sum and the alias production; the parser silently prefers a single-variant sum | **high** (specification ambiguity) | OPEN — owner-reserved (public specification) |

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

### C3 · Bidirectional-override characters are accepted silently — CLOSED (D26)

A source file containing U+202E RIGHT-TO-LEFT OVERRIDE inside a comment checked **clean, exit 0**.
This is the Trojan Source class (CVE-2021-42574): the bytes a compiler parses and the glyphs a
reviewer sees can be made to disagree, so a change can read as innocuous while doing something
else. Rust, Go, and others made this deny-by-default after 2021.

The exposure here is narrower than in most languages — identifiers are ASCII-only (DL0101), so the
attack cannot hide inside a name — which leaves comments and string literals. That is still the
whole of it: this project's stated purpose includes *humans reviewing AI-written code*, and a file
that renders differently than it parses attacks review directly. A language whose value proposition
is legible authority must not accept source whose legibility can be inverted invisibly. Raised from
medium to **high** on that reasoning: for this language, defeating review is not a side effect, it
is the whole attack.

**Fix (`DL0107`).** The lexer refuses any of the eleven Unicode bidirectional control characters
(the set Rust denies) as raw bytes anywhere in source. Two design choices make it durable:

- **One scan, ahead of tokenizing, over the whole raw source.** The rule lives in exactly one
  place and cannot die in a branch that forgot to check — the project's skip-branch discipline
  applied to a security rule. Every entry point (check, run, fmt, authority) reaches it because
  all of them lex.
- **It scans raw bytes, so the escape survives.** `\u{202e}` is ASCII in source — visible to a
  reviewer — and is left untouched, so a string that genuinely needs the code point can still have
  it, explicitly and legibly. Raw RTL *letters* (Arabic, Hebrew) are never affected, because they
  are not control characters; refusing them would break internationalized data, which would be its
  own discrimination. Both properties are witnessed.

**Witnesses.** In `crates/delulu-syntax/src/lexer.rs`: a raw override is DL0107; **all eleven**
controls fire and each names its own code point in the message (a missing entry is a character the
scan waves through); an escaped `\u{202e}` does **not** fire; RTL letters do **not** fire. Plus the
conformance reject file `tests/conformance/reject/DL0107_bidi_override.delulu`. Verified against the
old code: the reject file checks **clean** on the pre-fix binary and refuses after.

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

### C9 · There is no LICENSE — CLOSED (D27, owner-approved)

The state at the finding: no `LICENSE` file, and `Cargo.toml` declared no `license`. Under default
copyright that means **all rights reserved** — nobody but the copyright holder could use, copy,
modify, or distribute the code. Measured against the campaign's objective (anyone can download,
build, install, and use DeluluLang), it was the single hardest blocker, and no engineering moved
it. It was deliberately **not fixed unilaterally**: choosing a licence is a legal commitment
belonging to the copyright holder alone, so it was presented as a recommendation and held.

**Resolution (owner-approved 2026-07-24).** Jesse chose **Apache-2.0 for the code** and, for
derivatives, the **different-name** rule. The recommendation's two-tool split was adopted whole:

- **`LICENSE`** — the verbatim Apache-2.0 text, copyright Jesse Sunil. Permissive: use, modify,
  distribute, and sell, by anyone, commercially, with an explicit patent grant.
- **`NOTICE`** — the attribution that Apache §4(d) forces every redistribution to carry, naming
  Jesse Sunil as original creator; it may not be removed or altered.
- **`TRADEMARK.md`** — the DeluluLang name is a mark. Truthful reference is always fine; a modified
  or derivative language must ship under a **different name** and must not claim to be the original
  or official DeluluLang. This protects the name **without** restricting the code — the Rust /
  Python / Mozilla separation.
- **`GOVERNANCE.md`** — names Jesse as project lead, ties changes to the RFC process, and binds
  governance to the same honesty clauses as the code.
- **Machine-readable:** `license = "Apache-2.0"` and `authors = ["Jesse Sunil"]` on every one of the
  twelve crates (via `[workspace.package]` inheritance), and `licenses: [Apache-2.0]` added to the
  SBOM's own component — because an SBOM that lists its dependencies' licences and omits its own is
  exactly the kind of gap this project refuses.

No legal language was invented: the Apache text is standard, and the trademark/governance policies
are adapted from established open-source practice (`delulu-licensing-intent` memory records the
mapping). The one genuine ambiguity in the brief — whether a derivative must rename or must keep the
name — was **not guessed**; Jesse resolved it to rename.

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

### C16 · Cyclic type aliases are silently accepted — OPEN (P2 return)

`type A = A`, and `type A = B; type B = A`, check **clean, exit 0**. A serious language rejects a
cyclic type alias (Rust: "cycle detected when expanding type alias") because it names no ground
type — nothing can ever have that type.

**This was chased to a soundness verdict before being filed as hygiene, because the tempting
assumption is that it's harmless.** Three things were verified against the current binary:

- **It does not hang or overflow.** Cycles of length 1–4 terminate in well under a second; the
  alias resolver is genuinely cycle-aware, not merely stack-bounded.
- **It does not truncate deep chains unsoundly.** A **5,000-deep** non-cyclic alias chain
  terminating in `Int` resolves fully — `fn f(x: A0) -> Str { x }` is correctly refused DL0401.
  So the cycle handling is not a step-limit that could silently turn a deep-but-valid type into a
  fresh variable that unifies with anything.
- **It cannot launder an opaque type.** `type Sneaky = Sneaky; fn leak(s: Secret[Str]) -> Sneaky {
  s }` is refused DL0602 — secrets never coerce, even through a cycle.

So the exposure is exactly two hygiene defects, no soundness hole: `type A = A` should be a clear
"cyclic type alias" diagnostic rather than a silent accept, and *using* a cyclic alias currently
emits a confusing `expected T10, found T9` (the same type-variable-leak as C12) instead of naming
the cycle. Deferred to a focused Stage-1 return rather than bolted onto the D26 commit, because it
wants real cycle detection in the resolver with its own diagnostic and witnesses — not a patch.

### C17 · A float literal that overflows to infinity is accepted silently — OPEN

`1.0e400` checks clean and evaluates to `inf`. This matches C, JavaScript, and Rust (which produce
`inf` for an over-range float literal), so it is defensible and is **not** a soundness issue — filed
as an honesty gap, not a defect. Rust emits a warning in this case; DeluluLang, whose posture is to
say what it is doing, arguably should too. Low priority. (Note: `1e400` *without* a decimal point is
a parse error, because a float literal requires the point — `1e400` lexes as `1` then `e400`.)

### C18 · The semver-authority law was blind to secret-scope widening — CLOSED (D28)

Stage 2's headline is a supply chain that *cannot lie*: "any authority widening requires a major
version bump" (Constitution invariant 10), and `authority --diff` is the CI gate that surfaces a
widening on a dependency upgrade. Both were blind to secrets.

**The mechanism, confirmed by reading and then by witness.** Reading a secret — `root.secret("X")`
— adds **no effect and no capability kind**; it only records the name in a `secret_names` set. The
lock entry stored `effects`, `cap_kinds`, `net`, `fs_read`, `fs_write` — and **not** secrets. So
`authority_widened` could not see a secret change, and neither could `authority --diff` (which
diffs the same coarse fields). A dependency that read `TELEMETRY_TOKEN` in v1.0.0 and *also*
`DB_PASSWORD` in v1.0.1 had byte-identical observable authority to both tools: the patch bump was
waved through, and the new secret access was locked in silently. That is precisely the
silent-widening invariant 10 exists to forbid, in the stage built to forbid it.

**Fix (D28).** The lock entry gains a `secrets` field (the computed secret names), and
`authority_widened`, the semver-authority law (DL1003), and `authority --diff` all treat a new
secret name as a widening — like a new effect or host. Scoped deliberately: the `authority_hash`
is **left** over effects+kinds, because folding secrets into it would change every existing hash
and invalidate any committed lockfile (DL1002) — a format break, and backward compatibility is
owner-reserved. A same-version secret change is still caught, because it changes the source and so
the `content_hash` (DL1010). The security property — no silent secret widening across versions — is
fully closed without a format break.

**Witnesses.** Two, both observed to fail against the pre-fix code: a lock-level unit test
(`a_new_secret_is_a_widening_and_needs_a_major_bump` — a patch bump with one new secret and
otherwise identical authority is DL1003) and an end-to-end test that builds two real package
versions differing only by a secret read, asserts their coarse authority is identical (why it was
invisible) and their secret sets differ, and that `authority_widened` now sees it.

### C19 · The pin and self-declaration do not enforce secrets — CLOSED (D34)

Found while closing C18, and it is the **same blindness one layer earlier and more consequential**,
because the pin is the *first* review gate:

- **`check_self_authority` (DL1009)** forces a package to declare its `effects` in its manifest, but
  does **not** require it to declare the secrets it reads. A package can `root.secret("DB_PASSWORD")`
  with an empty `[authority] secrets` and check clean.
- **`check_pins` / `scope_violations` (DL1001)** constrains a dependency's effects, `net`, and `fs`
  against the consumer's pin, but **not** its secrets. A consumer cannot pin "this dependency may
  read TELEMETRY and nothing else." `AuthoritySpec` already *has* a `secrets` field; it is simply
  never checked.

This is consistent with invariant 10 ("scopes"), and the fix (enforce computed `secrets ⊆
manifest.secrets` for DL1009, and `dep.secrets ⊆ pin.secrets` for DL1001) is small and, verified
against the corpus, breaks **nothing in-tree** — no existing package reads a secret at all, which is
also why the gap was never noticed.

**Why it was held, and then made.** Unlike C18, this changes what the *checker accepts*: a package
that read an undeclared secret and checked clean now errors (DL1009), and a pin that does not list a
dependency's secrets now errors (DL1001). That is a **backward-compatibility change to acceptance
behaviour**, which the owner explicitly reserved, so it was presented as a recommendation and held —
the same discipline that governed licensing (C9). **Jesse approved it on 2026-07-25**; implemented as
ruling **D34**.

As built:

- **DL1009** compares the package's computed `secrets` (already available from `package_authority`,
  previously unread) against `[authority] secrets`, pointing at the `secrets` key — or, when the key
  is absent, at the file start, which is where the fix goes.
- **DL1001** adds a `secrets` dimension to `scope_violations`, following the **same convention as its
  siblings**: an empty pin means the consumer did not constrain that dimension, exactly as an empty
  `net` pin does not constrain hosts. Secret names compare exactly — there is no prefix or glob
  relation between them, unlike paths.

Two witnesses plus an accepting case (declaring the secret checks clean, because a ceiling bounds
rather than forbids). Both refusal witnesses were confirmed against the pre-fix behaviour — the
pre-fix code produced **no diagnostic at all** for either program, which is what made this a
supply-chain gap rather than a diagnostic-quality one.

*Implementation note worth keeping:* `check_self_authority` and `check_pins` are **not** part of
`check_workspace` — the library computes facts and the CLI composes the gate (`cli.rs`, in both the
check/build path and the lock path). A test that drives `check_workspace` alone will observe neither
rule, which is exactly how the first drafts of these witnesses passed against broken code.

### C20 · The two engines disagreed on faults, and WASM flooded on recursion — CLOSED (D29)

Stage 3's promise (invariant 15) is that the interpreter and the WASM backend agree: same output,
same exit, same fault. For *success* the 50k-program differential fuzz enforces it. For *faults* it
did not, and the fuzz missed it because it classifies any `(Err, Err)` as agreement without
checking the faults are the same.

Run the same faulting program on both engines and a user saw two different things:

| program | interpreter | WASM engine (before) |
|---|---|---|
| `7 / 0` | `error[DL0902]: division by zero` | `error[DL0904]: WASM trap … <backtrace>` |
| `i64::MAX + 1` | `error[DL0901]: integer overflow` | `error[DL0904]: WASM trap … <backtrace>` |
| deep recursion | `error[DL0905]: recursion depth exceeded` | `error[DL0904]` **+ 16,326 lines** of `<wasm function 8>`, one per frame |

Two defects in one. The **codes diverged** — every deterministic fault collapsed to a generic
`DL0904` on WASM, so an agent (the primary user, and the one most likely to read the code
programmatically) could not tell divide-by-zero from overflow from recursion on the sandbox floor.
And the **backtrace flooded**: a deep recursion emitted one `<wasm function N>` line per frame — over
sixteen thousand lines where the interpreter prints one — drowning a terminal and any log or agent
buffer downstream.

Both had one root cause: the WASM run captured `e.to_string()` on the wasmtime error, which appends
the full guest backtrace *and* discards the structured trap.

**Fix (D29).** `delulu_wasm::clean_trap` downcasts to `wasmtime::Trap` and maps the deterministic
traps to the interpreter's codes — `IntegerDivisionByZero → DL0902`, `IntegerOverflow → DL0901`,
`StackOverflow → DL0905`, `MemoryOutOfBounds → DL0903`, the codegen's overflow `unreachable →
DL0901` — and never includes the backtrace (unknown traps keep only the first line). The recursion
flood collapses from 16,326 lines to one. One honest residual, documented: rem-by-zero and overflow
both trap via `unreachable` and are indistinguishable from the trap alone, so rem-by-zero reports
DL0901 on WASM where the interpreter says DL0902 — a single exotic case, named rather than hidden.

**Witnesses** (`crates/delulu-wasm/tests/fault_parity.rs`), both observed to fail against the pre-fix
code: the two engines report the same code for divide-by-zero and overflow, and a guest stack
overflow is a single-line DL0905 with no backtrace.

### C21 · The interpreter's recursion guard overflows the host stack on small stacks — OPEN

Found while writing the C20 witness. The interpreter caps recursion at `MAX_DEPTH = 10_000` and then
reports DL0905 — but a tree-walking interpreter frame is large in a debug build (measured: 10,000
frames need **more than 16 MiB** of host stack). On the CLI's main thread this is fine, which is why
`delulu run` on deep recursion gives a clean DL0905. But delulu-runtime embedded on a **worker
thread** — Rust's default is ~2 MiB, and the test harness's threads overflowed even at 16 MiB —
hits a hard host stack overflow (`STATUS_STACK_OVERFLOW`, SIGSEGV) *before* the guard fires. That is
the "host crash on deep recursion" class Stage 9 (D15) fixed for the CLI, resurfacing for
small-stack embeddings.

The robust fix is a depth guard that does not depend on host-stack size — either counting logical
frames against a bound chosen for the *smallest* supported stack, or growing the stack deliberately
(`stacker`). Both are runtime-architecture changes deserving their own pass; recorded here rather
than bolted onto D29, which is about engine parity, not the interpreter's stack discipline.

### C22 · The syntax-morph system was specified but absent — CLOSED (D35)

`docs/design/SYNTAX_MORPH_SPEC.md` is a complete, normative specification of **surface-syntax
plugins**: bijective token-level keyword remappings that let a human write DeluluLang with keywords
in their own language (`函数` for `fn`) and let an AI apply a token-minimizing profile, with the
canonical form — and therefore every hash, artifact, and diagnostic envelope — unchanged. It defines
the morph TOML format, the bijectivity law, storage pragmas, and a DL17xx refusal class.

**None of it exists.** The string `morph` does not appear in a single `.rs` file in the workspace
(only `polymorphism`/`monomorphic` match). There is no `delulu morph` verb, no morph loader, no
`--morph` flag, no test. The sibling mechanism it is modelled on — *prose* localization — is fully
built and shipped (`delulu locale add/remove/list`, `--locale`, catalog plugins pinned to
verified-class with a zero-authority ceiling), which is precisely why the gap is easy to miss.

The defect recorded here is not "a planned feature is unbuilt" — that is legitimate. It is the
**contradiction**: the spec's own header said *"Implemented by: Stage 8 tooling"*, while
`STAGE8_SPECIFICATION.md` §6's companion note says to *"plan a §6.5-style `delulu morph` sibling of
`delulu locale` when building"* — future tense, never discharged. A reader of the morph spec alone
would conclude the feature ships. Two documents in the same tree disagreed about whether a feature
exists, and the code sided with the more pessimistic one.

Corrected immediately: the spec header stated its status honestly and pointed at this finding.
**Then built, on Jesse’s instruction (2026-07-25) — ruling D35.**

The build found a hole in the spec’s own law, which is the part worth remembering. §1 required a
morph to be bijective, single-token, and free of alias-vs-alias collisions — and that permits
`let = "fn"`. Such a morph is bijective, its alias is one token, it renders and round-trips
perfectly, and a file written in it uses the word `fn` to mean `let`. For a language whose premise is
that a human or an AI can REVIEW code another AI wrote, a surface that lies to the reviewer is the
same class of attack as the bidi controls D26 refuses. Closed as **DL1711**, and the spec’s §1 now
carries it as rule 1a. Bidi controls inside an alias are refused for the same reason (DL1712).

Two design decisions are load-bearing and were made deliberately:

- **The parser does not resolve the pragma.** Resolving `//! morph: X` means reading a morph file off
  disk, and a parser that acquires filesystem authority from a comment in its own input is ambient
  authority inside the compiler — precisely what this language exists to eliminate. The lookup lives
  in the CLI, which already has that authority; `delulu-syntax` takes an already-loaded `Morph`.
- **Conversion happens in exactly two places.** `lex_with_morph` is the only point where a
  non-canonical surface becomes tokens, and the CLI’s loader normalizes a pragma-bearing file at the
  edge. Everything downstream — checker, DIR, hashes, both engines, every report — sees canonical and
  cannot tell which surface produced it. Verified: the authority report for a Chinese-keyword program
  is byte-identical to its canonical form’s.

Proof it is real rather than plumbed: `fib(10) = 55`, printed by a program whose keywords are
Chinese, run directly through `delulu run`. Round-trip is byte-identical, and the string literal and
every identifier survive untouched. Emoji, Cyrillic, Greek, and mixed-script morphs round-trip in the
property tests — the alias rule is a denylist of *structural* hazards, not an allowlist of scripts,
because the point of the feature is that the surface belongs to whoever reads it.

Scope shipped vs. deferred is enumerated in the spec header rather than blurred: `.dpx` plugin
delivery, per-reader LSP view morphs, the `[style] morph` repo policy key, and morph-aware **package**
builds are NOT built. A package’s `src/` must be canonical — which is what the spec itself
recommends for shared projects.

### C23 · Builtin names were shadowable, and the shadow did nothing — CLOSED (D30)

Found while attacking the Stage-4 marshallability fence, which is an allowlist matched **by name**
(`Int | Float | Bool | Str | Unit | ForeignPtr`). The obvious attack on a name-matched allowlist is
to make something else answer to one of those names, so: `type Int = Secret[Str]`, then a foreign
signature naming `Int`. It **checked clean**.

The exploit does not exist, and that matters as much as the defect. Both engines lower foreign
signatures by NAME through one shared path (`lower_foreign_sig`, shared with the WASM host), and the
call site types `Int` as the builtin, so the two sides agree and no secret ever crosses — verified
directly: `lib.f(7)` still checks clean and passing a real `Secret[Str]` is still refused **DL0602**
("secrets never coerce"). The end-to-end run confirms invariant 20 holds in practice: the only way a
credential reaches C is the authorized `expose(Cap[Declassify])` route.

What was actually broken is **review integrity**, which for this language is not a lesser property:

- All **16** builtin type names were shadowable — `Int Float Bool Str Unit Root List Option Result
  Secret Cap ForeignPtr PyObj Plugin Verified Contained` — and every shadow was *inert*, because
  `lower_type` matches them before it searches user scope. No diagnostic, anywhere.
- All **10** core effect names too (`Read Write Net Clock Rand Declassify ForeignCall Load Async
  Actuate`), because `lower_row` resolves core effects before `user_effects`. **This is the
  authority-bearing case**: an author who wrote `effect Write` believed they had declared something
  private, while every `! {Write}` in the module continued to mean the effect that reaches the
  filesystem and the console.
- A `foreign … lib Int` block, and an actor of a builtin name, were accepted with handle types no
  signature could ever name.

A file containing `type Cap = Int` invited a reader — human or agent — to conclude that `Cap[FsRead]`
denoted something the author defined. In a language whose premise is that a program's authority can
be read off its source, a declaration that silently means nothing is the worst available outcome: it
misleads review without ever failing. This is exactly C11 (a user `fn parse_int` lost silently to the
prelude builtin) in the type and effect namespaces, and it was never closed there.

Refused now at the definition site with DL0302, the same code and the same reasoning as C11 —
refuse rather than pick a winner, because either winner makes one name mean two things depending on
where it is read. Enforced on **all three** `DeclTable` construction paths (`resolve.rs` single
module, `program.rs` package, `deps.rs` dependency graph); the package path was verified separately,
because a rule that holds on two paths out of three holds nowhere.

### C24 · A foreign signature's alias refusal did not mention the alias — CLOSED (D31)

`type Meters = (Int)` in a foreign signature reported "type `Meters` cannot be marshalled … only
Int, Float, Bool, Str, Unit, and ForeignPtr marshal" — true, and read as though the compiler had lost
track of the fact that `Meters` *is* an `Int`.

The fence still refuses it, deliberately and permanently: both engines lower foreign signatures by
type name through one shared path that cannot see a module's aliases, so expanding the alias in the
checker and not in the marshaller is precisely how ABI confusion begins. **Loosening the fence would
have been the wrong fix** — the runtime would have marshalled `FKind::Unit` for `Meters` (the
`unwrap_or` default), silently substituting a value. Instead the refusal now names the target and
carries an Exact repair that writes it. An alias expanding to a function type is additionally
re-classified from DL1301 to **DL1302**, because R-6a is about what the type *means*, and that one
means a re-entry point into verified code.

The alias walk is **bounded at 32 hops**. `type A = (A)` is accepted by the checker (C16), so an
unbounded resolver here would have turned three lines of source into a hung compiler — a denial of
service introduced *by the fix*. Its witness is part of the suite.

### C25 · The report knew a credential could leave and never said so — CLOSED (D32)

The commission requires that DeluluLang **tell** users when code exposes credentials. Tested against
the real thing: a program that reads `root.secret("API_KEY")`, declassifies it, and passes it to
`msvcrt.puts`. It ran, and `SUPERSECRET-abc123` was printed **by the C function**.

Every gate behaved correctly — `Declassify` declared in the row, permitted by the manifest (an
earlier attempt was refused DL0701 for omitting it), and granted explicitly by the human along with
the secret's value. That is the design working: freedom with responsibility, and the authorized route
is the only route.

The defect was in the report a human reads *before* granting. It listed `effects: Declassify,
ForeignCall, Write`, `secrets: API_KEY`, and `foreign: - c msvcrt [puts]` — three separate lines, all
the facts, and never the sentence. The decision to type `--grant declassify` turns precisely on the
join, so the report now states it:

```
  secrets:      API_KEY
  exposure:     API_KEY declassifiable -> foreign code (outside the proof), files/console
                an exposed secret is an ordinary value; the language cannot follow it past `expose`
```

It reports **capability, never behaviour** — `Declassify` in the row means `expose` *can* be called,
not that it is — and it names the safe case honestly too ("declassifiable, but this program has no
egress in its row"). Nothing about what the language permits changed: no new refusal, no change to
any grant relation, no new authority concept. The machine channel is deliberately untouched, because
`--json` already carries `effects`, `secrets`, and `foreign_calls`: an agent could always derive
this conclusion, and only the human could not.

### C26 / C27 · Silent success, and a raw OS error where a sentence belonged — CLOSED (D33)

**C26.** A package whose sources sat beside `delulu.toml` instead of under `src/` printed
`ok: built clean (1 package(s), 0 module(s))` and exited **0**. The toolchain looked in `<root>/src`,
found nothing, checked nothing, and reported success — so the natural layout mistake produced a green
build of an empty program, and a CI gate would have gone green with it. Refused now, on the same
posture the deferred-git-dependency gate already used: a check that could not run must never report
success. The closing line `0 error(s)` was fixed alongside it — it read as success beside a nonzero
exit, and it affected the pre-existing git and advisory refusals too.

**C27.** `delulu run <dir>` reported `error: cannot read <dir>: Access is denied. (os error 5)`. The
mistake is natural — `build` and `plugin build` take directories, so `run`/`check`/`authority` look
like they should — and the message was actively misleading, sending the reader after an ACL that was
never involved. It was also platform-dependent (Linux says `Is a directory`). One fix in the shared
`load` helper covers every file-taking command, names the mistake, and points at `delulu build`.

### C29 · A token for a revoked grant redeemed successfully — CLOSED (D36)

`Broker::redeem` verified the MAC over the whole payload, confirmed the bound node still existed, and
checked the token's own `exp_millis`. It never asked whether the node was **alive**. Four cases, run:

| token for | before | after |
|---|---|---|
| a **revoked** node | `Ok(node)` | `Err(Revoked)` |
| a node whose own TTL passed | `Err(Expired)` | `Err(Expired)` |
| a node under an **expired ancestor**, token minted with no deadline (`exp = i64::MAX`) | `Ok(node)` | `Err(Expired)` |
| a node under a **revoked ancestor** | `Ok(node)` | `Err(Revoked)` |

Only the second was refused, and only incidentally — that token's deadline happened to mirror the
node's TTL. There was no state check at all, so the third case was invisible by construction.

**It was not privilege escalation, and saying otherwise would be wrong.** `validate` re-reads the
effective state on every operation, so a redeemed dead node authorizes nothing. What it did do
matters for a system whose product is accountability:

- the redemption emitted an audit record reading **`decision: "allow"`** for a grant an operator had
  explicitly killed — the log an investigator reads said the redemption was allowed;
- `set_holder_peer` wrote the **redeemer's own text** onto the revoked node, so attacker-supplied
  data landed in custody state after revocation;
- the redeeming party was told it held a node it did not.

Fixed with one call to `effective_state_inherited`, placed after the MAC check (never act on
unauthenticated data) and **before** the nonce is burned or any state is written, so a refused
redemption mutates nothing. Two witnesses, both observed failing against the old code.

### C30 · The audit read surface presented a broken chain as authentic — CLOSED (D36)

`delulu audit verify` recomputes every hash and every `prev_hash` link, across files, and refuses at
the failing seq with DL1405. It works. **Nothing on the read path called it.** `tail` and `query` go
through `read_all_records`, which validates nothing and — worse — `continue`s silently past any line
that fails to parse.

Demonstrated on a four-record chain:

- **Tamper.** Flip one record's `decision` from `allow` to `deny`, leave its hash alone. `verify`
  catches it (DL1405 at seq 2). `tail` printed four records **displaying the forged value**, with no
  warning, on the one surface an operator uses after an incident.
- **Omission.** Corrupt one record into non-JSON. `verify` catches it. `tail` printed **three**
  records — `g_1`, `g_2`, `g_4` — and `g_3` was simply gone. No gap marker, no error, no hint that
  the listing was shorter than the log. An entry can be removed from the record of what happened by
  corrupting one line.

Fixed by using the verifier that already existed: the read path verifies first, then **still shows
the records** — an operator investigating a tampered log is precisely the person who most needs to
read it — behind a warning that says the entries must not be trusted and that any record corrupted
beyond parsing is missing entirely. Exit is nonzero so a script cannot mistake a corrupt read for a
clean one, and `--json` always carries `chain_verified` (with `chain_error` when false) so a machine
consumer never infers integrity from a field's absence.

The audit log remains **observability, not enforcement** — it detects, it does not prevent. The point
of the fix is that the observation must be honest about its own integrity.

### C31 · The device dimension had no Guard class — CLOSED (D37)

`Scopes` carries eight dimensions. The Guard enumerated seven.

`tier_for_mint` walked a fixed `[(GuardClass, &BTreeSet<String>); 7]` array, and `use_axis_class`
ended in `_ => None`. When RFC 0001 F1 added `device`, neither grew, and **neither could fail to
compile** — the array is a literal, and the catch-all swallows any new `Op`. So `Op::Actuate`, which
is active and round-trips per command, was born with no axis of its own.

**Precisely what was and was not broken:** actuation was still gated, through the cross-cutting
`effect:Actuate` rule, at both mint and use. It was never ungated. But `device` was the **only**
authority axis with no per-item granularity: an operator could write `net:api.example.com`,
`secret:DB_PASSWORD`, `fs_write:./out` — and for devices, only "all actuation" or "none". On the one
axis in the system that moves physical hardware, you could not seal a thruster while leaving a status
LED at `warn`.

Fixed by adding `GuardClass::Device` (gating on the device name; the envelope itself is bounded by the
`⊑` lattice, not by policy patterns), extending the mint walk, and mapping `Op::Actuate` to it.
**No default rule was added** — that would change behaviour for existing device holders, and what
tier physical actuation deserves is an operator's decision, not a library's.

The class fix is the durable part: `use_axis_class` is now **exhaustive**, listing the ops that
genuinely have no scope dimension. The next `Op` variant cannot be born ungated in silence — the
build breaks until a person answers "what gates it?"

### C2 / C32 / C33 · The two front doors under load — CLOSED (D38)

Commissioned as a crash hunt: *"test everything till it crashes and breaks, then fix it, then test
again."* The framing correction that came with it matters and is recorded here because it shaped the
work — **there is no discrimination between the surfaces**: a human may drive the CLI and an agent may
drive the compiler, so both surfaces must be equally good for both audiences. A `--json` contract break
is not "an AI problem" and a wall of unreadable stderr is not "a human problem".

**What did not break.** Twenty-six hostile programs — 2000-deep parentheses, 1500-deep blocks,
800-deep generic types, 20 000-term expressions, a 200 KB string literal, a 100 000-character
identifier, 6000 functions, a 3000-field record, a 2000-variant match, unterminated strings and
comments, embedded NUL bytes, an empty file, a BOM-only file — produced **no panic, no hang, no signal
death**. The CLI sweep (every subcommand × malformed argument shapes, both output modes) produced
none either. The front end is genuinely robust; that is worth stating as plainly as the defects.

**C2 — `--json` emitted nothing at all on failure.** `docs/for-agents.md` promises *"Every `--json`
command emits one object."* On a usage or I/O error — a missing argument, an unreadable path, a
malformed flag — the CLI printed a human sentence to stderr and exited nonzero with **zero bytes on
stdout**, on essentially every subcommand. The finding had been filed as one narrow case (a read
failure); it was the entire surface. Any programmatic caller then has an exit code and nothing to
parse.

Fixed in `cli::run`, one wrapper around the whole dispatch, rather than at the ~161 individual
`return 2` sites — for the reason this campaign keeps rediscovering: a rule enforced at every site is
a rule the next site forgets. The fallback envelope carries the documented fields, sets
`summary.errors = 1` so the documented pass test stays correct, and **invents no DL code**, because
the registry is a stable contract and a usage error is not a language diagnostic.

The gate is `crates/delulu/tests/json_contract.rs`, and it tests **exactly one** object rather than at
least one — which is how it immediately caught the opposite defect in `delulu test`, and later in
`deploy`, where a report was already being printed and the fallback added a second.

**C32 — 76 MB of stderr from a 10 KB file.** `x.a.a.a…` 5000 deep is ~5000 unknown-field errors, and
every diagnostic quoted its whole source line — which *is* the 10 KB chain — twice, once as text and
once as an underline. Measured: 76,518,387 bytes, 14.2 seconds. This is D29's backtrace flood wearing
a different costume, and the same reasoning applies: past some volume, output stops being a diagnostic
and becomes a denial of service against whoever must read it, human or agent.

Two bounds, both on the human channel only:

- **Snippet window** (`delulu-diag`): a quoted line is cut to 160 characters around the span, marked
  `...` on whichever side was elided, with the caret arithmetic corrected for the window and every
  index char-based so a multi-byte character is never split. Lines at or under the limit — which is
  every line in the corpus, the examples, and the Book — render byte-identically.
- **Diagnostic cap** (CLI): at most 50 human-rendered diagnostics, then a note stating exactly how
  many were withheld and how to get them all. The `--json` channel is deliberately uncapped: it is a
  contract to report every diagnostic, and a consumer that asked for all of them can page itself.

Result: **76,518,387 → 23,530 bytes** (3,252×), **14.2 s → 0.125 s**.

**C33 — two commands nothing could find.** `deploy` and `fleet` dispatch and work, and `--help` listed
neither. That is why the first CLI sweep missed them, and `deploy` was double-emitting JSON on its
refusal paths — caught only because an older test happened to parse its output. Both are in `--help`
now, and a gate asserts that every dispatched subcommand appears there, because **an undocumented
command is a command nothing sweeps.**

### C34 · A plugin ceiling could advertise authority the model cannot confer — CLOSED (D39)

Stage 6's adversarial pass, hunting P6's pattern deliberately: `Scopes` has eight dimensions and the
places that enumerate them do not always follow.

**What was wrong.** `Grant::to_authority` and `PluginArtifact::ceiling` both hard-code `device`,
`foreign_c`, and `foreign_python` to empty — deliberately and correctly, since a plugin is not a thing
that may command a machine or bind a native library. But *reading* a manifest that declared one of them
simply dropped the declaration. Observed directly: a manifest declaring
`device: ["arm0/elbow:pitch=-5..5,…"]`, `foreign_c: ["libm"]`, and effects `[Read, Actuate,
ForeignCall]` produced a ceiling with **empty device and foreign scopes but all three effects intact**,
and `step1_container_api` returned `Ok`. The artifact loaded clean while advertising a ceiling it did
not have, so a reviewer of that manifest — or of `plugin verify`'s output — was told the plugin could
reach a device it can never reach.

Not exploitable, and that is worth stating precisely rather than inflating: the drop is *toward* less
authority, and `cap_slice` gives an unlisted effect **no host import at all** (its `_ => {}` is
fail-closed and says so), so a declared `Actuate` reaches nothing. The defect is legibility, which for
this language is not a lesser property — it is the same finding as C23's inert declarations, and
SECURITY.md §3.1 now scopes it in explicitly.

Refused now at step 1 with DL1508, naming the dimension. An **empty** list stays legal: it claims
nothing, and refusing it would break manifests that spell their dimensions out for documentation.

**Deliberately NOT refused: an effect with no host import.** A ceiling may still name `Actuate` or
`ForeignCall`. `cap_slice`'s comment marks Net/Declassify/Load/ForeignCall as having "no Contained host
import in v0.6" — forward work, not an oversight — and refusing them today would prejudge it. The
inertness is witnessed instead (`an_effect_with_no_host_import_reaches_nothing`).

### 3.2 What Stage 6 got right, verified rather than assumed

Recorded because a campaign that only lists defects gives a false picture of the code, and because
re-deriving these costs a future session real time:

- **One path to a loaded plugin's authority.** `grant.to_authority()` → `step3_ceiling` (which reuses
  the broker's own `attenuation_check`, all nine dimensions in one conjunction) → `step4_holder` → the
  `PreparedLoad`. The value in `PreparedLoad` is *exactly* the one that passed `⊑`; there is no
  alternative constructor.
- **The class is never inferred or substituted.** `step2_class` refuses any mismatch between declared
  and requested (DL1508); a Verified request is never satisfied by a Contained artifact, and a Verified
  plugin whose DIR is missing or fails replay is DL1504 with **no fallback to Contained**.
- **Signatures: three cases, three outcomes, and no lenient path.** `Invalid` is matched *first and
  unconditionally* — a present-but-invalid signature refuses **even when `require_signed` is false**,
  which is exactly the skip-branch a "not required, so don't check" reading would have opened. Unsigned
  under `require_signed` is a different code (DL1511) from badly-signed (DL1510), and the messages say
  so. `verify_detached` documents the discipline outright: "the couldn't-tell cases each refuse
  honestly (wrong length, bad key, non-verifying) — never silently treated as unsigned."
- **Reload cannot swap authority.** A reload mints a *fresh* node, so an old reference can never be
  re-bound to a plugin with different authority; an unknown node "confers nothing"; a limit kill drops
  the instance *and* revokes its node in the same act; and if any load step fails, the node is revoked
  and nothing is instantiated — no partial load.
- **The `.dpx` reader was already hardened, and the sweep is real.** Every length goes through
  `checked_add` plus a `<= bytes.len()` filter, so nothing allocates on a *declared* size; `read_uleb`
  bounds its shift at 32 bits (no LEB bomb) and uses `get()?` (no panic); the section walk strictly
  advances, so it cannot spin. An existing test already flips every single byte and truncates at every
  offset. Added the one hostile shape that sweep structurally cannot reach — a **crafted** 5-byte ULEB
  declaring 0xFFFF_FFFF, the classic allocation bomb — plus an unterminated ULEB and 64 zero-length
  sections. All refuse from header arithmetic alone.

### C35 · Root authority silently narrows across an actor boundary — CLOSED (D40)

`RootMsg` is a **hand-written enumeration** of the dimensions a `Root` carries when it is sent to an
actor. Diffed against `RootVal` field by field, exactly one is missing: **`computes`**, Stage 10 phase
10h's compute-dispatch grant. Phase 10e's `actuators` and `sensors` — one phase earlier, and
`ComputeEnvelope` is plain data of exactly the same shape as `ActuatorEnvelope` — do cross.

So an actor holding a Root slice loses compute authority, and nothing says so: a program that
dispatches a kernel from `main` fails inside an actor, at run time, with a refusal about the device
rather than an explanation about the boundary. The direction is **fail-closed**, so nothing here is
unsafe. What is wrong is that it was an *omission rather than a decision*, and no test could tell those
apart.

**Not fixed by carrying it, deliberately.** Making `computes` cross would *widen* what an actor may do.
That is a capability decision, not a hardening fix, and this campaign does not get to make it — the
restrictive reading stands until the owner chooses. What is fixed is the part that is unambiguously
wrong: the silence.

- The conversion site now names the omission and points here.
- A gate reads **both struct definitions out of the source** and fails if any `RootVal` dimension
  neither crosses nor appears in an explicit `WITHHELD_FROM_ACTORS` list, with a message that tells a
  maintainer to *decide* rather than to append. It also fails in the other direction, so a stale
  "withheld" claim cannot outlive the fact. Verified by removing `computes` from the list and watching
  it fire.

Rust has no reflection and the two lists live in different files, so a source-scanning test is the only
instrument that closes this class; `delulu-conform` already scans compiler source for the same reason.
This is the third phase running in which the defect was a hand-maintained enumeration that a later
dimension was added past (C31's fixed `[…; 7]` array, C34's dropped plugin dimensions, and now this) —
the pattern is worth naming as such: **every hand-written list of authority dimensions needs a gate, or
it will silently fall behind `Scopes`.**

### 3.3 What Stage 7 got right, verified by execution

The concurrency model held under every attack I could construct, and the skip-branch discipline is
better here than anywhere else in the tree:

- **Sendability refuses when it cannot tell.** `check_boundary_params`' `None` arm is an explicit
  refusal — *"sendability could not be determined … a boundary guarantee is never guessed"* — and
  `default_rcap` is exhaustive over every `Type` with `Type::Var(_) => return None` ("Undetermined:
  never guess (kitchen rule)"), propagating undecidability out of composites rather than defaulting.
  Tested: a `ref List`, a closure parameter, and a generic `T` all refuse (the last as *undecidable*);
  `Cap`, `Secret`, and `Root` are sendable **by decision**, documented as unforgeable immutable handles
  citing invariant 36, not by falling through a catch-all.
- **`consume` is flow-sensitive on every shape.** A branch join, a loop-carried consume, a match arm,
  and a straight-line double consume all produce DL1602 — the loop case with its own message ("by the
  next loop iteration this binding is already dead"). `recover` reaching a non-sendable outer binding
  is DL1605.
- **The rcap deny properties hold.** Writing through `box`, calling a sync method on a `tag`, reading a
  field through a `tag`, and capturing a `ref` in a `val` closure are all refused (DL1603/DL1604).
- **Aliasing an `iso` is safe, and it is worth recording *why*, because it looks like a hole.**
  `let alias = xs` is accepted, and after `c.take(consume xs)` the alias is still *readable*. It is not
  a race: the alias degrades to a read-only view (mutating it is DL1604, sending it is DL1601), and
  `MsgValue` — the wire form of every message — is a fully owned structural type with no `Rc` or
  `RefCell` anywhere, so a send **deep-copies by construction**. The scheduler is genuinely
  multi-threaded with each worker owning its actors' heaps outright and cells never crossing threads.
  The alias therefore reads the sender's own data, which is coherent rather than unsound.

### C15 · `fmt` merged comment paragraphs, and the identity law was blind to it — CLOSED (D41)

Two comment paragraphs separated by a blank line came out as one block. Reproduced, fixed, and the
interesting part is *why no law caught it*.

The formatter has two laws, both compiler-bug class: identity (`parse(fmt(src)) ≡ parse(src)`, with a
projection that includes the full comment sequence) and idempotence. The comment projection is each
comment's `(text, own_line)` **in order** — and merging two paragraphs changes neither the text, nor the
own-line flag, nor the order. Only the *spacing between* comments was lost, which is exactly the part
carrying the author's structure. **A law that watches the pieces and not the gaps between them has a
blind spot precisely this wide**, and that is the transferable lesson: when a projection is chosen to
prove a property, ask what the projection cannot see.

Fixed by tracking the source line each own-line comment ends on and emitting one blank when the next
one starts more than a line later. Runs of blank lines still collapse to one — that is ordinary
canonical formatting, and it is the line between *preserve the author's structure* and *preserve the
author's whitespace*. The skip branch is a **trailing** comment: it belongs to the line above it, so it
must not be treated as a paragraph end, or the formatter starts *inventing* blank lines on top of the
item separation it already emits. Witnessed in both directions, and the identity + idempotence laws
still hold on every corpus program.

### C36 · The mermaid graph did not say what it leaves out — CLOSED (D41)

`delulu atlas --format mermaid` renders **3 nodes and 1 edge** for a program whose graph has **12 nodes
and 23 edges**: packages and modules only. No functions, no effects, no capabilities, no call edges.
`--format dot` renders all of them.

Module level is the right scope for a shareable overview, and it *is* documented — one line in
`SURFACE_ATLAS_PALETTE_ADDENDUM.md`. The problem is where a mermaid diagram ends up: pasted into a
README, an issue, or an agent's context, permanently separated from the command that produced it and
from any documentation about it. A reader then sees two boxes and the honest conclusion available to
them is that the program has no functions and no effects. For a graph whose stated purpose is that a
human or an agent can read unfamiliar code **and its authority**, "this program appears to have no
effects" is the worst wrong conclusion it could invite — and it is the C23/C34 class again: safe,
and misleading.

The artifact now describes itself, in two `%%` mermaid comments naming the scope, what is excluded, and
which format shows the rest. `--help` says it too. **The HTML renderer already got this right** and is
the model: full graph normally, collapsed above a node cap *with a visible notice* — so the pattern
existed in the same file and one renderer had not adopted it.

### 3.4 What Stage 8 got right, verified by execution

- **Locale invariance is real, and now proved mechanically.** The same diagnostic-producing program
  under `--locale en-US` and `--locale delulu-slang` yields **byte-identical `--json`**, while the human
  prose genuinely changes. Invariant 39 holds where it matters.
- **The Atlas queries and refusals work.** All four verbs (`node`/`callers`/`calls`/`why`) answer
  correctly; `digest` is byte-stable across repeated runs; and a program with check errors is refused
  with **DL1780 alongside** the underlying diagnostic, so the reader learns both that the graph was
  refused and why — no partial graph.
- **The LSP is a server and survives being treated as one.** Empty input, non-JSON, an unknown method,
  a truncated frame declaring `Content-Length: 99999`, and a hover on a nonexistent file: no panic, no
  hang, no signal death; every case exits cleanly.

### C28 · `type A = B` is ambiguous in the normative grammar — OPEN, owner-reserved

The Stage-1 grammar says:

```ebnf
type_decl = "type" , IDENT , [ generics ] ,
            ( "{" , field , { "," , field } , [ "," ] , "}"   (* record *)
            | "=" , variant , { "|" , variant }               (* sum    *)
            | "=" , type ) ;                                  (* alias  *)
variant   = IDENT , [ "(" , type , { "," , type } , ")" ] ;
```

`type Meters = Int` matches **both** alternatives: a sum of one field-less variant named `Int`, and
an alias to the type `Int`. The spec does not say which, and the parser decides silently —
`looks_like_variant()` prefers the **sum** reading whenever the right-hand side is an identifier
followed by end-of-statement, `|`, or `(`.

The consequences are observable and were confirmed by running them:

- `type Meters = Int` declares a nominal sum type whose constructor is named `Int`, so
  `fn g() -> Meters { Int }` **checks clean** — the token `Int` in expression position now
  constructs a `Meters`.
- There is no way to write an alias to a bare type name at all. `type Meters = (Int)` — the
  parenthesised form — is the only spelling that reaches the alias production.
- A value that should have been a `Meters` reports `DL0401: argument type mismatch: expected 'T9',
  found 'Int'` — naming an inference variable rather than the type the author wrote, which is finding
  **C12** made materially worse by this ambiguity.

**Choosing the disambiguation rule is a public-specification decision and is therefore reserved to
the owner.** The two coherent resolutions are (a) a bare identifier means an **alias**, matching the
near-universal convention, with single-variant sums requiring an explicit marker; or (b) keep the sum
reading and **refuse the ambiguity**, requiring the author to disambiguate. Both change which
programs are accepted, so neither may be adopted by the kitchen. Recorded here, with the behaviour
documented in `STAGE1_SPECIFICATION.md` so the ambiguity is at least resolved *on paper* against what
the implementation actually does.

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
closed one, RFC 0001's comment period remains open until 2026-08-05 with two recorded process
deviations against it. **Surface-syntax morphs now exist (D35)** but only for single files read
through a `//! morph:` pragma — package sources must be canonical, and plugin-delivered morphs and
per-reader LSP view morphs are not built.
