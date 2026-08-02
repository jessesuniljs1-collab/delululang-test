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

Workspace size: **~93,500 lines of Rust** across 12 shipped crates plus one tooling crate, 193
files. These numbers are no longer maintained by hand — `docs/survey/SURVEY.md` recounts them from
the tree on every build, and a stale figure here now fails a test rather than sitting quietly (the
figure this line carried until 2026-08-01 was ~82,000 across 175 files, which had drifted by 14%).
The repository is *not* rustfmt-formatted under default settings and has no `rustfmt.toml`; house
style is wider than rustfmt's default, and `cargo fmt` is therefore **not** a gate. Do not run it.

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
| C7 | The capability corpus is **7** programs (filed as 8 — the count included a `NOTE.md`); `tier4-multimodule` has none | medium (evidence) | **CLOSED** — D55 (tier 4 is now 4 packages / 7 modules, depth 3; 3 programs are RUN, not only checked) |
| C8 | The interpreter's recursion bound is fixed at 10,000 and appears in no user-facing document | medium (usability) | **CLOSED** (documented) — D25 |
| C9 | **There is no LICENSE** — nobody may legally use the project | **high** (adoption) | **CLOSED** — D27 (owner-approved) |
| C10 | Runtime — DL0703 refused without naming the grant that would fix it | medium (usability) | **CLOSED** — D25 |
| C11 | Checker — a user function silently loses to a same-named prelude builtin | **high** (correctness) | **CLOSED** — D25 |
| C12 | Diagnostics — `DL0401` prints a type-table INDEX (`T11`) wherever a record or sum is named — in the checker, the LSP hover, the REPL, **and the published `interface.json`** | **high** (every user-declared type in every message; a machine-readable artifact said `fn(T9) -> Float`) | **CLOSED** — D64. Was wrongly carried as DOES NOT REPRODUCE: the clearing re-test used `Int`/`Str`, the two shapes that cannot fail |
| C13 | **Runtime — a named function used as a value checks clean and faults at runtime** | **high** (correctness) | **CLOSED** — D25 |
| C14 | **`DL0907` was titled "match reached no arm" and is raised for a dozen unrelated conditions** — a reader hitting it for an unbound name was told something false about their program | medium (honesty) | **CLOSED** — D42 |
| C15 | **`delulu fmt` deleted the blank line between two comment paragraphs**, merging them — and the identity law could not see it | medium (fidelity) | **CLOSED** — D41 |
| C16 | Checker — a cyclic type alias (`type A = A`) is silently accepted | low as filed (hygiene) — **the severity was wrong**: P2 tested the declaration, never a USE, and a used cycle crashed the compiler (C54) | **CLOSED** — D47a, together with C54 |
| C17 | Lexer — a float literal that overflows to `inf` is accepted without a warning; **and, not filed, the worse half: one that underflows to `0.0`** | low as filed (honesty) — **the framing was wrong**: the integer column already refused its own version, so this was an asymmetry, not a preference | **CLOSED** — D54 (both refused as DL0104; `parse_float` added on the same shared rule) |
| C18 | **Stage 2 — the semver-authority law and `authority --diff` were blind to secret-scope widening** | **high** (supply chain) | **CLOSED** — D28 |
| C19 | **Stage 2 — the dependency pin (DL1001) and self-declaration (DL1009) do not enforce secrets** | **high** (supply chain) | **CLOSED** — D34 (owner approved 2026-07-25) |
| C20 | **Stage 3 — the two engines disagree on fault codes, and a WASM trap dumps a ~16k-line backtrace** | **high** (parity/usability) | **CLOSED** — D29 |
| C21 | Runtime — the interpreter's `MAX_DEPTH=10000` overflows the host stack below ~20 MiB (small-stack embeddings) | medium (robustness) | **CLOSED** — D51 (the bound is now API: `Interp::with_max_depth`; per-frame cost measured at 80 KiB debug) |
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
| C35 | Stage 7 — a `Root` slice **silently loses `computes`** when it crosses an actor boundary; `RootMsg` is a hand-written enumeration that Stage 10 phase 10h did not extend | medium (silent narrowing — fail-closed but undecided) | **CLOSED** — D40 (gated + documented); the capability question itself CLOSED — D46b (it crosses) |
| C36 | `atlas --format mermaid` is a module-level overview that did not say so — a reader could conclude a program has no functions or effects | medium (legibility of the authority graph) | **CLOSED** — D41 |
| C37 | **The conformance coverage law proved a witness EXISTS, not that it exercises its anchor** — a rejecting witness repointed at an unrelated real test left coverage reporting 100% | **high** (the project's own proof of spec coverage) | **CLOSED** — D42 |
| C38 | **An unsigned artifact and a badly-signed one both reported DL1705** on the detached path, contradicting the project's own ruled deviation 8 | medium (release integrity / machine contract) | **CLOSED** — D42 |
| C39 | **The stepped simulator's clock does not advance on a refused command**, so a dead-man revocation the wall clock produces cannot occur in simulation — and simulation is what `--signoff` approves for hardware | **high** (physical safety rehearsal / DL1905 evidence) | **CLOSED** — D43 |
| C40 | **A term stated twice in an envelope was resolved silently, and the two parsers resolved dimensions in OPPOSITE directions** — appending a tighter bound recorded the tightening and did not apply it | **high** (physical envelope widening) | **CLOSED** — D43 |
| C41 | **Both runtime envelope parsers accepted non-finite bounds** (`angle_deg=-inf..inf`, `kernel_ms=0..inf`) — an envelope that bounds nothing while satisfying every mandatory-term check | **high** (physical envelope vacuity) | **CLOSED** — D43 |
| C42 | The broker accepted any `fail=` string including empty; the runtime accepts three — a device grant no program could ever mint, discovered in the field rather than at delegation | medium (operability of a safety term) | **CLOSED** — D43 |
| C43 | **The cross-parser law was one-directional and example-based** — "runtime-accepts ⇒ broker-agrees" over four good specs — so it could see none of C40/C41/C42 | **high** (a law proving less than it claims) | **CLOSED** — D43 |
| C44 | `module_requests_native`'s `_ => false` covers all three attribute-carrying AST structs today, with nothing stopping a fourth from carrying an unreported `@jit` | low (drift risk, not a live defect) | **CLOSED** — D43 (gated) |
| C45 | An approved `deploy plan` said "within the authority ceiling" and `--json` said `"approved": true`, while comparing one authority dimension of nine | medium (a security verdict read as broader than it is) | **CLOSED** — D43 |
| C47 | **The normative grammar cannot describe the output of the project's own formatter** — `fmt` emits trailing commas in param lists and record literals that §3 does not permit, and requires one on any multi-line list while §3 says it is optional | **high** (an independent implementation built from the spec would reject every formatted file) | **CLOSED** — D44 (grammar corrected); parser relaxation raised as C47b |
| C48 | **Record field lookup cloned the whole type definition per access** — a function reading N fields of an N-field record did N² field-entry deep clones; 632 ms to check one 2000-field record | **high** (quadratic compile time on a realistic shape) | **CLOSED** — D44 (15× faster; residual curve measured and published) |
| C49 | **An empty `delulu.toml` crashed `build`/`check` with a Rust panic** — and the project's own no-panic gate could not see it, because the CLI runs on a worker thread whose panic is mapped to exit 2 ("internal"), not 101 | **high** (crash on the most ordinary beginner mistake; the crash gate was structurally blind) | **CLOSED** — D44 |
| C50 | **`delulu authority` reported `summary.errors: 0` and exit 0 for a package `check` refuses** — the review surface never opened `delulu.toml` at all | **high** (the review surface asserting a package is clean when it is not) | **CLOSED** — D44 |
| C51 | **`delulu authority <dir>` cannot report on any package that has a dependency** — it uses the single-package loader while `build`/`lock`/`authority --diff` resolve the graph, so DL0303 refuses every monorepo member | **high** (the supply-chain question is exactly when the review surface is wanted) | **CLOSED** — D45 |
| C53 | **An unused type alias is never resolved** — `type Meters = Metres` (a typo) checks clean, and the error only appears if and where the alias is used; in a library whose own code never uses it, the diagnostic lands on the consumer | medium (a declaration accepted and silently inert — the C11/C23 family) | **CLOSED** — D47b |
| C54 | **A USED cyclic type alias aborted the compiler with a stack overflow** — `type A = A` plus one use died at `0xC00000FD`; the no-panic sweeps could not see it, because a stack overflow prints no `panicked at` | **high** (hard crash on ordinary input; DoS for anything compiling untrusted code) | **CLOSED** — D47a (reshapes C16) |
| C55 | Runtime record field access is **O(record width) per read** (165→5,071 µs/1k reads at 50→3,200 fields) | — | **NAMED LIMIT** — D47c, re-examined D56: the claim about 5–20 fields was an extrapolation from a table starting at 50. Measured, the curve is **U-shaped** and per-read cost is at its MINIMUM there. Limit stands, reasoning is now data |
| C57 | **The authority SERIALIZATION seam had no gate** — `to_json` (write) is hand-enumerated with nothing to catch a dimension added to `Scopes` and not emitted; the read side already fails closed, the write side did not | medium (a silently under-reporting audit record — the C29/C30 class, not an escalation) | **CLOSED** — D48a (compile-enforced round-trip gate) |
| C56 | **`--trace-effects` buffers the entire trace in RAM** — 100k effects take peak memory from 6.7 MB to 70.1 MB (~633 B/record), unbounded; the audit chain already streams to day files, the trace does not | medium (opt-in flag, but the runs that enable it are the long-lived ones) | **CLOSED** — D49 (bounded when diagnostic; `--assert-trace` never capped) |
| C52 | **A `delulu.lock` could misstate what a dependency does and `build --locked` reported "built clean"** — the recorded `effects`/`cap_kinds`/`secrets`/scope fields, the ones a reviewer reads, were verified against nothing; so were the format version, duplicate entries and a stale recorded version | **high** (the CI gate trusted a review artifact it never checked, while `authority --diff` on the same file reported WIDENING) | **CLOSED** — D45 |
| C28 | **`type A = B` is ambiguous in the normative grammar** — it matches both the sum and the alias production; the parser silently prefers a single-variant sum | **high** (specification ambiguity) | **CLOSED** — D46a (resolved to ALIAS; a variant list is signalled only by `(` or `\|`) |
| C47b | **Should a multi-line bracketed list require its trailing comma?** The parser requires it, most languages do not, and the diagnostic does not teach the fix | medium (front-door usability) | **CLOSED** — D46d (no; all four spellings accepted, in all nine lists) |
| C46 | **Should a refused command prove liveness?** The dead-man now charges a refused attempt the same simulated time the wall clock charges it, but whether a controller whose every setpoint is out of range should KEEP its machine is a safety-policy choice | — | **CLOSED** — D46c (no; the stricter reading, which is what the code already did) |
| C58 | **A `pub fn` whose signature names a type the package does not re-export builds clean alone and fails when consumed** — and the error is reported *inside the dependency's own source*, calling a type "not a type" in a file where it is in scope | medium (diagnostic blames the wrong line in the wrong package — the C53 family) | **CLOSED** — D65 (re-framed to the import that brought the signature in; the private-in-public rule was measured against the corpus and rejected) |
| C59 | **A multi-package program cannot be RUN.** `delulu run` takes one file or a `.dwx`; `build` emits `interface.json` and nothing executable, so `kind = "bin"` is declarable and unexecutable — true of the shipped `examples/greeter/` too | **high** (the largest capability gap the campaign found) | **CLOSED** — D61 (`run <package-dir>`; source-flattened after the workspace check; a cross-module name collision is refused, not guessed) |
| C60 | **The adapter provenance verdict is printed, not recorded** — no durable, queryable evidence of which key signed the driver that drove the machine. Both existing homes were checked and neither fits (the audit chain is a no-op without a sink; the DL1905 sign-off is written by a simulation, before any adapter exists) | medium (accountability at the physical boundary) | **CLOSED** — D66 (written into the broker's existing hash-chained audit log; refusals recorded too) |
| C69 | **The audit chain's single-writer assumption is undocumented and unenforced** — `AuditLog::open` reads the head then appends, which is safe for the long-lived broker and unsafe for any short-lived process that can run concurrently | medium (a second writer silently corrupts the chain: `prev_hash` break plus physically interleaved lines) | **CLOSED** — D66 (no default sink; the assumption is now stated where a writer is created). **Found by causing it** |
| C61 | `let _ = expr` is refused (DL0201) although `_` is a valid **match** pattern; discarding is still possible under any other name, so the restriction prevents nothing | low (friction with no safety benefit) | **CLOSED** — D63 (`_` is unreadable because it lexes as its own token, not because a rule forbids it) |
| C62 | **`.gitattributes` declares `* text=auto eol=lf` and nothing enforced it** — one tracked file (`HARDENING_CAMPAIGN.md`, this document) was stored **CRLF** in its committed blob, created three days after the attribute was adopted and unnoticed for the whole campaign | low (repository hygiene) — but it is rule 2's shape with the gate missing entirely | **CLOSED** — D57 (renormalized, and a test now reads the index) |
| C63 | **The Book credits two-engine parity to a fuzzer that cannot run the second engine, with a number 25× too large** — Chapter 9 claimed "tens of thousands of programs on both engines" and "50,000 random programs"; the generative sweep is **2,000**, all inside the WASM fragment, and `delulu-fuzz` depends only on `delulu-check`/`delulu-runtime`. It also never said the WASM backend is a **fragment** — ~a third of entry-point programs compile, and **none of the Book's own guide chapters do** | **high** (front-door claim; the C4/C6 family crossed with C37) | **CLOSED** — D58 (prose corrected with the error left visible; two gates added) |
| C64 | **A record or list literal bound with `let` cannot be passed to a function** — `let p = P { x: 1 }` then `f(p)` is DL1603, while `f(P { x: 1 })` inlined is fine, and so is the same value arriving from a call's return or a `match` binding. Extract-variable, the most basic refactoring there is, turns a working program into a compile error | **high** (ordinary code refused; hit three times in one session writing the C7 corpus) | **CLOSED** — D62 (lifted at `val` arguments; the caller gives up write access, and an author-written `ref` is never lifted) |
| C70 | **A normative runtime rule was false on the concurrency path.** `ref.rule.runtime.faults-are-diagnostics` names recursion depth and promises "a diagnostic with a code, never a host crash"; the reference marked it **covered**. The same function at the same depth printed its answer from `fn main` and aborted the process from inside an actor behavior — above depth **43** (debug) and **~350** (release) against a documented bound of 10,000. The actor scheduler runs the same interpreter on worker threads that reserved no stack | **high** (a shipped normative guarantee, broken by an ordinary recursive helper called from an actor — no embedder, no hostile input) | **CLOSED** — D67 (one budget in `delulu-runtime`, workers reserve it with their bound sized to match, and a source-scanning gate over every thread site). Design rule 1's **7th** instance; design rule 2 in a new costume — *the right signal, on the wrong thread* |
| C65 | **`delulu authority` could not read a `.dwx`** — the DISTRIBUTION format, whose whole claim is "authority that travels with the code". It fell through to the source loader and died with `stream did not contain valid UTF-8`, while `run` verified the same embedded manifest and printed the effects | **high** (the review surface cannot review what you ship) | **CLOSED** — D59 |
| C66 | **`delulu fmt notes.txt` reported "reformatted 0 file(s)" and exited 0** — nothing done, success claimed, on a path the user named deliberately | medium (silent success — the C26 class) | **CLOSED** — D59 |
| C67 | **The authority report's `pure fns:` list is unbounded** — on a 24,630-line program it is 2,536 names on ONE line of 28,242 characters, burying the six lines a reviewer came for. D38 capped diagnostics for exactly this reason; the review surface was never capped | medium (legibility of the review surface at scale — the C32 class) | **CLOSED** — D60 |
| C68 | **The Book's code-block gate compared COUNTS, never contents** — 0 of 10 blocks were slices of any compiled sample, and Chapter 14 taught `root.foreign[mathlib](...)`, which does not compile at all. The sample backing it held only the `foreign` declaration: the safe half, no call site | **high** (the one chapter for calling C taught a form the compiler rejects — C37's shape in the front door) | **CLOSED** — D60 (block corrected and made a literal slice; correspondence gate added) |

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

### C7 · The capability corpus is seven programs — CLOSED (D55)

`tests/corpus/` is described in `REPOSITORY_STRUCTURE.md` as "coding-capability tiers (simple →
security-expert)". It contained **seven programs**: two simple, two DSA, one application, two
security — and `tier4-multimodule/` held a `NOTE.md` and **no program at all**. For a language
proposing itself for robotics, satellites, SaaS backends, and enterprise systems, this is not
enough evidence to support the proposal, independent of whether the language is capable.

*(The finding said eight. Its own breakdown sums to seven; the eighth item was the `NOTE.md`. The
miscount is corrected here rather than silently — a ledger that quietly fixes its own numbers is
one nobody can audit.)*

**Closed (D55).** Tier 4 is now **four packages, seven modules, dependency depth three, with a
diamond**, and the other tiers gained four programs (a recursive sum type with its algorithms,
row-polymorphic higher-order code, an application over real input with distinguishable failures,
and capability attenuation via `narrow`). Eleven single-file programs and one package graph.

Two things make it evidence rather than volume:

- **The harness learned the difference between a file and a package.** A directory holding a
  `delulu.toml` is built through the same loader `delulu build` uses; feeding its files to
  `check_source` one at a time reported failures that said nothing about the program.
  `accepting_packages_build_clean` carries a count floor, because a corpus law that passes by
  examining nothing is the shape C37 and C49 already cost this project.
- **Three programs are RUN, with their output asserted** (`corpus_cli.rs`). Checking clean and
  working are different claims, and the corpus was only ever making the first one.

**Writing it found four defects, which is the argument for having done it** — see D55 for each:
`parse_float` did not exist (a language with a `Float` type could not read one from input, now
closed under D54), **C58** (a public signature naming an unexported type), **C59** (a multi-package
program cannot be run at all), and **C61** (`let _` is refused).

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

### C12 · `DL0401` printed a table index where the names were known — CLOSED (D64)

Passing a `Cap[Http]` result to a function declared `Result[Str, IoErr]` reports:

```
expected `Result[Str, T0]`, found `Result[Str, T1]`
```

The real answer is `IoErr` versus `NetErr`, and the checker knows both — `T0` and `T1` are
internal identifiers for builtin sums that the type printer does not resolve back to names. The
diagnostic is accurate and useless: it names the shape of the disagreement and hides its content.

**This entry was marked DOES NOT REPRODUCE for a year of campaign time, and that was wrong.** The
reproduction above was re-run verbatim and produced exactly the text above. The clearing re-test had
used the monomorphic and generic shapes (`Int` versus `Str`) — `Type::Int` and `Type::Str` print
themselves, and `Type::Record`/`Type::Sum` are the only variants that store an index, so the only
shapes that could fail were the ones not tested. Rule 2 of this campaign, applied to this campaign.

It was also filed too narrowly. The defect was not a `Cap[Http]` corner: **every user-declared record
and sum** printed as `T11` — `expected T11, found T12` for `Verdict` versus `Status` — and three
further surfaces shared the printer: the **LSP hover**, the **REPL** type echo, and the published
**`interface.json`**, which recorded `"type": "fn(T9) -> Float"` in a file whose purpose is machine
introspection. **Closed by D64**: `Display for Type` is deleted so the compiler demands names at
every site, the no-table case renders `<type #11>` (unmistakable rather than plausible), and
`api_row_hash` — computed from the AST, not from `Type` — was verified byte-identical, so no lockfile
moved.

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

### C17 · A literal that is not the value you wrote — CLOSED (D54)

**As filed:** `1.0e400` checks clean and evaluates to `inf`. This matches C, JavaScript, and Rust,
so it is defensible and is **not** a soundness issue — filed as an honesty gap, not a defect, and
left at low priority for a year of campaign time.

**The framing was wrong, and that is why it sat.** Comparing DeluluLang to C was comparing it to the
wrong thing. The right comparison was to the arm of the same function twelve lines below, which has
always refused an integer literal too large for `Int` — *"there is no automatic promotion, because a
silent widening is a silent change of meaning."* Two literals, one failure mode (the written value is
not the value the program will use), opposite answers. It was an **asymmetry**, not a preference.

**And the half that was not filed is the worse one.** `f64::from_str` also *flushes to zero*:
`1.0e-400` becomes `0.0`. Where `inf` announces itself downstream, a silently-zeroed gain makes a
control law quietly do nothing while every value along the way looks perfectly ordinary. For a
language aimed at machinery that is the more dangerous direction, and nothing in the original finding
saw it.

**Closed (D54).** Both are **DL0104** on the existing code. A literal the author *did* write as zero
is still zero (`0.0e-400` is accepted) and a **subnormal is accepted** — it loses precision but keeps
its magnitude, which is the property the rule is about. Infinity is still reachable by computing it
(`1.0 / 0.0`); it just cannot be spelled as a finite number. The rule lives in one function
(`delulu_syntax::num::float_from_text`) because it has two callers — the lexer and the new
`parse_float` — and two copies of it would have been C40 again.

(Note: `1e400` *without* a decimal point is a parse error, because a float literal requires the
point — `1e400` lexes as `1` then `e400`.)

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

### C37 · The coverage law proved existence, not exercise — CLOSED (D42)

Invariant 42 is the project's own proof that its suite covers its specification, so this pass tested the
**proof** rather than the code. P9's lesson applied directly: ask what the projection cannot see.

Two of the documented claims hold, verified by breaking them:

- A witness pointing at a **nonexistent** test is caught, and the anchor correctly drops to "no
  rejecting test".
- A witness pointing at an **`#[ignore]`d** test is caught the same way.

The third did not. Repointing DL1710's *rejecting* witness at
`a_pragma_is_read_only_from_the_first_line` — a real, active test that has nothing to do with DL1710 and
never produces it — left the gate reporting **`PASS: 100% anchor coverage`**. So the law proved *"a
named, non-ignored test exists for this anchor"*, not *"that test exercises this anchor"*. 100% coverage
did not mean every diagnostic was tested, and a single mis-registration would void it for one code
silently.

A static scanner cannot run a test and observe which codes it emits. What it *can* check is that the
test **names** the code — which every correctly written rejecting witness does. Measured before
deciding: **108 of 109** rejecting witnesses already named their code. The check costs almost nothing and
closes the mis-registration hole.

Two things it deliberately does not do:

- **It applies to rejecting witnesses only.** An accepting witness proves a code does *not* fire on
  valid input; `accepting_programs_check_clean` witnesses eighty of them and would never name one,
  because the whole point is that nothing fires. Requiring a mention there would be requiring the wrong
  thing — measured too: 82 of 144 accepting witnesses name no code, correctly.
- **The one non-conforming rejecting witness is an explicit, reasoned exception, not a weakened rule.**
  DL1907's witness observes the refusal through the DeluluLang program's own
  `ComputeErr::KernelEnvelope` value (with a control case), because that refusal surfaces as a catchable
  error rather than a DL-coded diagnostic. It genuinely exercises the code. Listed with its reason, the
  way `WITHHELD_FROM_ACTORS` is (D40).

### C38 · Unsigned and badly-signed reported the same code — CLOSED (D42)

Seven attacks on the detached signature path. Every bad signature refused — a signature over a
different artifact, truncated to 95 of 96 bytes, a flipped public-key byte, a flipped signature byte —
and `--require-hybrid` against a classical-only signature correctly refused with its own code (DL1908).
No fail-open anywhere.

But an **absent** signature reported **DL1705, "signature verification failed"** — the same code as an
invalid one, and for an unsigned artifact not even true, because nothing was verified.

The distinction is the one that matters most here. "No signature exists" is a *policy* question and is
often benign; "a signature exists and does not verify" is an *attack indicator* — tampered content, or
the wrong key. A caller handed one code for both cannot tell them apart, and `docs/for-agents.md` is
explicit that the code is the contract and the message is not.

**This was not a new principle — the project had already ruled it.** Stage-6 deviation 8 says
badly-signed and unsigned are different faults, its own test asserts that phrase, and the plugin path
implements it with DL1510 vs DL1511. Only the detached path never followed the rule. Fixed by using
**DL1511**, whose meaning already *is* "the artifact carries no signature", and generalizing its registry
entry from plugins to every artifact kind rather than minting a new number.

### C14 · `DL0907` described one condition and is raised for a dozen — CLOSED (D42)

Registry title: *"match reached no arm (checker bug if ever seen)"*. Explain body: only the `match` case.
Actually raised for an unbound name, an assignment to one, a field assignment on a non-record, an index
assignment on a non-list, `?` on a non-Result, a call to an unknown function, an unknown test name, an
actor turn with no actor address, and a foreign value reaching an actor boundary.

So a reader who hit DL0907 for an unbound name — and ran `delulu explain DL0907`, which is exactly what
the diagnostic invites — was told *"A `match` reached no arm at runtime"*, which is false about their
program. The code is really the **class** "an internal invariant the checker should have guaranteed was
violated"; the message text names the specific condition. Both entries now say that, the `match` case is
kept as the canonical example, and the generated reference was regenerated.

### 3.5 What Stage 9 got right, verified by execution

- **The SBOM is accurate, and its scope is declared unusually well.** 17 direct third-party dependencies
  declared by workspace crates, 17 listed, **zero drift in either direction**, and every version matches
  what the lockfile resolves for that direct declaration. Its own note explains that transitive
  dependencies live in the committed lockfile and *why* that omission is stated rather than left to be
  discovered — "an SBOM that omits something linked into the binary is worse than none, because it will
  be trusted". The D19 fix held.

  Worth recording how nearly this was mis-reported: a first pass compared SBOM versions against a
  name→version map built from the lockfile, and `wasm-encoder` and `getrandom` each appear at three
  versions there. The SBOM names the version bound by the *direct* declaration (`wasm-encoder = "0.221"`
  → 0.221.3), which is correct; the others are transitive resolutions for other dependents. The tool was
  right and the analysis was wrong.
- **Every bad signature refuses**, on all six shapes tried, and hybrid policy refusals are a distinct
  code from verification failures.

### C39 · A simulation that cannot run out of time — CLOSED (D43)

The stepped clock (`--sim-step`, ruling D20) exists so a device demonstration replays identically
regardless of build speed: simulated time advances one step per **device interaction** rather than by
the wall. The interpreter, meanwhile, checks a command against the capability value's own envelope and
returns early when it refuses — 10e's law, the command dies and never the process — so a refused
command never reaches the broker.

Put together, those two correct decisions produced this: **a program whose every command is refused
froze simulated time and held its device for unbounded simulated duration.** Witness, same program and
same grant, six out-of-envelope commands, `heartbeat_ms=1`:

| clock | result |
|---|---|
| wall (`--broker-profile sim`) | `lease revoked (missed-heartbeat), beat overdue by 657 µs` |
| stepped (`--sim-step 1000`) | six refusals, **no revocation** — 1000× the heartbeat, six times over |

The divergence pointed the wrong way, and the reason it matters is the sign-off chain: **DL1905 refuses
hardware unless a simulation of those exact artifact bytes was approved.** So the one environment that
authorizes hardware could not exhibit a revocation that hardware would produce — and the fault class it
could not rehearse is a controller whose every setpoint is out of range, which is precisely the
malfunction a dead-man exists to take a machine away from. A units bug is the ordinary cause.

**The fix does not touch the dead-man.** `due()` still decides when a lease dies, the wall-clock watchdog
is unchanged, and a refused command still does not *beat* a lease — only an interaction that reached the
broker ever did, and that is still true. What changed is that a refused attempt now costs the simulated
time a real controller would have spent (`DeviceBroker::note_refused_attempt`), which is what the wall
clock provides for free. If that sweep is what killed the lease, the lease is the reported fact: "you no
longer hold this device" outranks "your setpoint was out of range", matching the ordering
`DeviceBroker::command` already documented for the accepted path.

Witnessed against the old code with the exact pre-fix payload (six `REFUSED`, no revocation), and the
control — same all-refused program, heartbeat long enough to cover the run, device retained — passes
against both old and new code, so it is not vacuous.

**And the fix turned out to close a second defect on the WALL clock, which is the more broadly
important of the two.** The ordering change means the lease is consulted before the envelope on the
refusal path — so a program that has *already lost* its device to the watchdog and then sends an
out-of-envelope command is told it lost the device. Before, it was told its setpoint was out of range.
Witnessed on the wall clock with no `--sim-step` at all: a burn longer than the heartbeat between two
out-of-envelope commands gives `after: REVOKED` now and gave `after: REFUSED` before, while the run
summary said the lease was revoked either way.

That is a real accountability defect in production, not a simulator artifact, and it defeats the
distinction Stage 10 deliberately built: `Envelope` and `LeaseRevoked` are separate variants *because*
"you clamp a bad setpoint and retry, and you STOP when you no longer hold the machine." A controller
told `Envelope` clamps and retries against a machine it does not hold.

Recorded honestly: this consequence was **not** predicted when the fix was designed. The first attempt
to demonstrate it (six rapid refusals on the wall clock) showed no change at all, because all six
commands completed before the watchdog's first tick — the observable difference needs real time to pass
between commands. The prediction was wrong before it was right, and the witness is what settled it.

### C40 · A term stated twice, resolved two different ways — CLOSED (D43)

`authority.rs` already wrote the rule down: `Scopes::device` is a map keyed by device because "two
envelopes for the same device would be an ambiguity the enforcement path would have to resolve, and
resolving it silently is how a widening gets in." That reasoning was applied **per device** and never
**per term inside one envelope** — and the two parsers for the one grant grammar resolved a repeated
dimension in *opposite* directions:

- `delulu_broker::device_scope::parse` → `BTreeMap::insert` → **last wins**
- `ActuatorEnvelope::parse` → `Vec::push` + first-match in `envelope_check` → **first wins**

So `angle_deg=-30..95,angle_deg=-1..1` meant `[-1, 1]` to the authority that is recorded, delegated,
attenuated and audited, and `[-30, 95]` to the code that moves the machine. **The dangerous edit is the
safe-looking one**: appending a tighter bound — the obvious way to tighten an envelope, and the obvious
thing an agent does when asked to reduce a limit — recorded the tightening and did not apply it. Witness
with a control: a command of 12° was accepted under `-30..95,-1..1` and refused under `-1..1` alone.

Both parsers now refuse a repeated term, dimensions and fixed terms alike. Nothing legitimate states a
bound twice, and an ambiguity about a physical bound is refused rather than resolved, because whichever
way it is resolved, half the readers are wrong.

Also verified and NOT a defect: two separate `--grant actuator=` flags for the same device are **met**,
giving the tighter envelope in either order.

### C41 · An envelope that bounds nothing — CLOSED (D43)

`"inf".parse::<f64>()` and `"NaN".parse::<f64>()` both succeed. The broker has refused non-finite bounds
since D12e and documents the refusal as load-bearing — it is what makes `DeviceScope`'s `impl Eq` sound.
**Neither runtime parser had the check**, and the machine is moved from the runtime side:

- `angle_deg=-inf..inf` — commanded 12° successfully; the envelope admits every finite value.
- `kernel_ms=0..inf` — and here the gap is sharpest, because that term is mandatory with this stated
  reason: *"a kernel with no time budget can occupy the device forever."* `0..inf` satisfies the
  requirement while being exactly the unbounded budget the requirement exists to prevent.

`NaN..NaN` was already harmless (every comparison against NaN is false, so it refused everything) —
fail-closed by accident, not by design. Both parsers now refuse any non-finite bound.

### C42 · A fail-state no program could mint — CLOSED (D43)

`fail` was free text on the broker side and a closed three-variant enum on the runtime side. So
`fail=hodl`, `fail=`, `fail=safe_park` and `fail=Hold` all produced a valid *grant* that the runtime
would refuse to turn into a capability. Fail-closed, but at the wrong time and place: delegation happens
in advance — in a certificate, a fleet plan, a CI config — and the refusal arrived when a robot tried to
move. The canonical list now lives in `device_scope::FAIL_STATES`, in the lower crate (`delulu-runtime`
depends on `delulu-broker`, not the reverse), so there is **one** list rather than two that can drift.

### C43 · A cross-parser law that could not see any of this — CLOSED (D43)

The existing pin read: *"Every envelope the runtime can parse must render to a string the broker parses
back to the SAME authority"* — one-directional, and quantified over four hand-picked well-formed specs.
C40, C41 and C42 all lived underneath it. This is the C37 defect in a different subsystem: **a law that
proves less than it claims**, and the specific gap is the same shape — it checked agreement where both
sides said yes, and never checked that they said yes and no in the same places.

Replaced with a bidirectional law over a corpus that includes the hostile shapes: for every spec,
`runtime_ok == broker_ok`, plus the corpus's own expected verdict, plus full field agreement wherever
both accept. Each of C40/C41/C42 was observed failing it, in the correct direction —
`runtime: accepted / broker: refused` for the non-finite and duplicate cases, and
`runtime: refused / broker: accepted` for the fail-state.

### C44 · The hint scan's catch-all — CLOSED (D43, gated)

`module_requests_native` decides both whether the authority report carries a `native-emission` line and
whether DL1906 warns. It walks `Item::Fn` and `Item::Actor` plus module attributes and ends in
`_ => false`. That is **correct today** — those three are the only AST structs with an `attrs` field, so
no other item *can* carry `@jit` — but a catch-all cannot fail to compile, so a future
`TestDecl { attrs }` (`@ignore`, `@slow` are the obvious candidates) would carry a hint that neither
surface mentions. This is the fifth instance of the recurring pattern; the gate reads `ast.rs` for
structs declaring `pub attrs:` and fails in both directions, telling the maintainer to **decide** rather
than to append. Verified non-vacuous by deleting the `Item::Actor` arm and watching it fail.

### C45 · An approval that did not carry its own scope — CLOSED (D43)

`deploy plan` compares the environment profile's **effect** ceiling and nothing else. `deploy.rs`'s header
has always said so — "Recorded as a gap, not implied as covered" — but the verdict a reader acts on said
`approved — N service(s) within <env>'s authority ceiling`, and `--json` said `"approved": true` with no
scope at all. Nine authority dimensions exist; this compares one. A deployment gate is read by people and
agents deciding whether to launch, far away from the source file that qualifies it — the same reasoning
C36 applied to a mermaid graph pasted into a README. The verdict now says `EFFECT ceiling` and both
surfaces carry `compared` / `not_compared` explicitly, the machine surface with the same facts as the
human one.

**Verified separately and not a defect: DL1909 cannot be bypassed by a bad profile.** An empty file,
unparseable garbage, a mis-spelled `[authorities]` section, a singular `effect =` key, and a
wrong-typed `effects = "Clock"` all yield a ceiling of *no effects at all* — every service refused. An
unreadable profile is a plain exit-2 error, never an approval.

### 3.6 What Stage 10 got right, verified by execution

- **DL1909 cannot be bypassed by a bad environment profile.** Five hostile profiles — an empty file,
  unparseable garbage, a mis-spelled `[authorities]` section, a singular `effect =` key, and a
  wrong-typed `effects = "Clock"` — every one yields a ceiling of *no effects at all*, refusing every
  service. An unreadable profile is a plain exit-2 error, never an approval. Fail-closed in every
  direction tried.
- **The `@jit` leash cannot be slipped.** Without the grant: DL1906 warns, the hint is ignored, the
  program runs interpreted. The authority report carries a dedicated `native-emission: requested`
  line in the human surface and `native_emission.via` in `--json`. With the grant, nothing native
  exists to run. And **a lease can never confer it** — `exec_native: false` is hard-coded on the lease
  path, so the leash holds across the federation boundary, not just locally.
- **The device-scope lattice is the best-reasoned module in Stage 10.** The whitelist direction is
  counter-intuitive (more dimensions is *wider*, because a dimension the envelope never bounded is a
  command shape it refuses) and it is stated next to the code and tested from both sides. `meet` is
  never wider than either input, drops non-overlapping dimensions rather than widening them, is
  symmetric, and preserves `ttl ≥ heartbeat`. Inverted ranges, zero heartbeats, `ttl < heartbeat`,
  missing mandatory terms and unknown devices are all refused.
- **Multiple `--grant actuator=` flags for one device are MET, not last-wins.** Tried in both orders;
  the tighter envelope wins either way. The per-device ambiguity `authority.rs` warns about is
  genuinely handled — it was the per-*term* case inside one string that was not (C40).
- **Certificate authority parsing refuses on "cannot tell".** An unknown authority key, an unknown
  effect name, and an unknown scope dimension each refuse the certificate **whole**, with the reason in
  the code: *"an authority dimension a verifier cannot see is one it cannot enforce, so ignoring it
  would silently WIDEN the grant."* This is the recurring pattern answered correctly on the read side.
- **A long forged certificate chain is not a denial of service.** Verification is sequential and dies
  at the first hop that fails, and the anchor check is at hop 0 — so a million-certificate chain costs
  one signature verification before refusal. An attacker who prepends a genuine anchored certificate
  gets exactly two.
- **The dead-man's wall clock was correct throughout.** It revokes on its own tick, names the cause and
  the fail-state, reports `LeaseRevoked` distinctly from `Envelope`, and the control case (a healthy
  loop) keeps its device. Exit status stays 0 — the command dies, never the process.

Noted, not a defect: **single adoption is keyed per certificate**, so a subordinate broker may adopt
chains from two different roots, each separately bounded, revocable and audited. The docstring's phrase
"only ONCE per broker lifetime" describes the scope of the *memory*, not a one-chain-per-broker limit.
Whether multi-root adoption should be permitted is federation policy, and so is not settled here.

### 3.7 P12 — scale, and what size exposed

Every earlier phase attacked one program or one hostile input. This one attacked SIZE, on the premise
that a language nobody can use at 30k lines is not a production language. Corpora were generated, not
hand-written, and varied by SHAPE as well as line count — wide (10,000 sibling functions), deep (5,000
nested calls), one 30,000-statement function, 2,000 types, 4,000-field records, 1,000-arm matches,
50-deep dependency chains and 200-package diamonds.

**The headline is that the compiler scales and the tooling around it mostly does too.** Release, warm,
on a 40,046-line / 478 KB file: `check` 173 ms, `atlas` 290–426 ms in every format, `fmt` on 30,009
lines 1,292 ms and its output still checks. Peak memory never exceeded 52 MB anywhere in the corpus.
Five thousand real errors in one file render in 155 ms / 14 KB because D38's 50-diagnostic cap holds
exactly as designed, and `--json` stays deliberately uncapped at 5,000 diagnostics in one object.
Lockfiles for a 50-deep chain are byte-identical across repeated writes. `atlas --format digest` is
byte-stable across runs at scale.

### C47 · The grammar could not describe the formatter's output — CLOSED (D44)

Found while generating the corpus: a 100-field record written the ordinary multi-line way would not
parse. The rule turned out to be one rule, not one bug — **every comma-separated bracketed list
requires a trailing comma when it spans lines**, `match` arms excepted — and the normative grammar
(§3.0) disagreed with the implementation in *both* directions at once:

| construct | §3 says | multi-line, no trailing `,` | multi-line, trailing `,` |
|---|---|---|---|
| record type body | `[ "," ]` — optional | **DL0201** | ok |
| record literal | no trailing `,` listed | **DL0201** | **ok** |
| fn params | no trailing `,` listed | **DL0201** | **ok** |
| call args | no trailing `,` listed | **DL0201** | **ok** |
| list literal | no trailing `,` listed | **DL0201** | **ok** |
| match arms | `[ "," ]` — optional | ok | ok |

So the grammar permitted a form the parser refuses, *and* refused a form the parser accepts. The
second half is the serious one, because **`delulu fmt` emits exactly what the grammar forbids**:
formatting a wide record and a wide parameter list produces trailing commas in both. An independent
implementation written from §3 alone would have rejected every formatted file containing a wide list —
in a language whose stated ambition is other implementations.

Fixed on the specification side, which is where the defect was: §3.0 now states the newline rule
normatively (it is not derivable from an EBNF with no `NEWLINE` terminal), and `params`, `list_lit`
and the record-literal production carry the `[ "," ]` the parser has always accepted. **Not fixed:
whether the parser SHOULD require the comma.** It is stricter than most languages, the diagnostic a
person meets is `expected }` with the caret after the last element while `}` sits on the next line,
and relaxing it changes what compiles → C47b, owner-reserved.

### C48 · A quadratic field lookup, hidden behind a `.clone()` — CLOSED (D44)

`records_2000` took **516 ms** while `wide_10000` — five times the bytes, seven times the lines — took
173 ms. Isolating the two conflated variables settled it in one table (release, warm, ms):

| N | N-field type, 1 access | 2-field type, N accesses | N locals, N-term sum | **N fields, N accesses** |
|---|---|---|---|---|
| 250 | 18 | 14 | 18 | 18 |
| 500 | 16 | 12 | 14 | **57** |
| 1000 | 17 | 16 | 15 | **134** |
| 2000 | 26 | 19 | 19 | **632** |

Declaration alone is flat. Accesses alone are flat. An N-term `+` chain with no records is flat. Only
the *product* explodes — the signature of per-access work proportional to field count.

The cause was not the `find()` scan. `field_type` did `self.table.type_def(id).clone()` on **every
field access**, deep-copying all N field entries each time, so a function reading N fields of an
N-field record performed N² field clones. The clone existed only to release the borrow on
`self.table` before `lower_type` takes `&mut self`; a scoped block that clones the one field's type
expression and the generics does the same job. **632 ms → ~42 ms at N=2000, a 15× improvement**, and
4× the input now costs ~2.9× the time instead of 11×.

**The residual is real and is published rather than implied away.** The `find()` scan remains, so the
cost is still O(fields × accesses) with a small constant. Attributed by measurement, not assumed — at
N=4000 the declaration, access and expression shapes all grow linearly while only the product grows
at 2.7× per doubling:

| N | decl | access | chain | both |
|---|---|---|---|---|
| 2000 | 27 | 20 | 19 | 42 |
| 4000 | 59 | 29 | 32 | **113** |

A name→index map would make it O(1) and is the obvious next step; generated code from a protocol or
database schema is where thousands of fields actually occur. Not done here: the cliff that made the
shape unusable is gone, and adding a cache at the end of a long phase without room to verify it is how
a fix becomes a defect.

### C49 · An empty `delulu.toml` crashed the build — and the crash gate could not see it — CLOSED (D44)

`delulu build` on a package whose manifest was empty, not TOML, missing `[package]`, or missing `name`
panicked: `index out of bounds: the len is 0 but the index is 0`. Five of twelve manifest shapes
crashed, on `build` and `check`, while **`lock` and `authority` diagnosed every one of them correctly
with exit 1** — a rule holding on two paths out of four (C23/D30's pattern again).

**The origin is worth stating plainly: a fix from an earlier phase of this campaign introduced it.**
C26/D33 added the note *"no `.delulu` modules found under `<dir>`"* so that an empty package could not
report success. Composing that message reaches for the root package's directory — and an unreadable
manifest fails resolution *before* a root package is recorded, leaving `packages` empty and
`modules` empty, so the note fires and the index panics. The diagnostic was never wrong: DL1004 was
computed correctly every time. The tool crashed while being helpful about something else. A repair
needs its own skip-branch analysis, and "what if there is nothing to name?" is one.

`Workspace::root_pkg()` now returns `Option`, so the next caller cannot reintroduce the crash without
the compiler making them consider the empty case.

**The larger finding is why this survived P0's breadth sweep, P1's front door, and D38's dedicated
crash hunt — all of which swept for crashes.** `main.rs` runs the whole CLI on a spawned thread with a
512 MiB stack (so `MAX_DEPTH` fires before the native stack does), and when that worker panics `main`
joins it and returns **2**, deliberately and documented: *"exit codes are part of the stable contract:
0 ok / 1 diagnostics / 2 internal."* That is the honest code and it must not change. But exit 2 is
also what an ordinary usage error returns, and `json_contract.rs`'s no-panic sweep keyed on
`code == 101` — so **the gate that exists to catch crashes was blind to every crash in the path where
all the work happens.** Both sweeps now detect the panic message itself.

⚠ **Method note.** The first version of the manifest witness PASSED against the unfixed code, because
its fixture had no dependency: with nothing to resolve the entry module still loads, `modules` is
non-empty, and the panicking note is never reached. It witnessed nothing until the fixture gained a
sibling dependency — the same trap C19 set in P5. Separately, two "no panic here" readings during the
hunt were artifacts of `head -3`: the panic line sat below the diagnostic. **Do not conclude absence
from truncated output.**

### C50 · The review surface never opened the manifest — CLOSED (D44)

`delulu authority <dir>` printed a confident report, `diagnostics: []`, `summary: {errors: 0}` and
exit 0 for a package whose `delulu.toml` was empty — the same package `check` refuses with DL1004.
The report was **byte-identical** to the report for a well-formed manifest, so a reader could not
distinguish "I read your manifest" from "your manifest is unreadable and I ignored it". Both surfaces
were silent, human and `--json` alike, so `summary.errors: 0` was a false claim in a
machine-readable field on the one command whose entire product is *"what this program can do to your
system"*.

The cause: `load_package` walks `src/` and never opens the manifest, and `authority_package` consulted
only the program's diagnostics. A present manifest is now parsed and its diagnostics unioned in.
**Absent stays legal** — `authority` accepts a plain directory of modules (C26/D33 ruled on flat
layouts) — but present-and-unreadable is refused, which is the same shape as Stage-6 deviation 8's
present-but-invalid signature: a checker that shrugs at a claim it cannot check is the "when it cannot
tell, it says yes" failure.

Noted, not fixed, because it is a different question: `authority` does not evaluate the manifest
CEILING either. A package whose code performs `Write` while its manifest declares `effects = []` gets
a clean report and exit 0 from `authority`, and DL1009 from `check`. That is defensible — `authority`
answers "what can this do", `check` answers "is this package well-formed" — but the review surface
never mentioning that the package violates its own declaration is worth an owner's attention.

### C51 · `authority` cannot review a package that has dependencies — CLOSED (D45)

Found in P12 and closed in P13; the full account, including the regression the obvious fix
would have caused, is under §3.8 below.

### 3.8 P13 — every input is written by an adversary

P12 proved the toolchain survives size. This phase assumed the input is hostile. Two findings closed,
both on the supply-chain surface, and one large negative result.

**The fuzz sweep found nothing, which is worth stating as a result.** 561 invocations across four
parsers — `.delulu` source, `delulu.toml`, `delulu.lock` and morph TOML — driven by truncation at
twelve offsets plus byte flips, deletions, inflations, injections (NUL, BOM, `1e400`, 200-deep bracket
runs, oversized integers) and self-duplication, each run through `check`/`fmt`/`atlas`/`build`/`lock`/
`authority`/`morph` as applicable. **No panics, no hangs.** Crashes were detected by the panic
MESSAGE, not by exit code — the C49 lesson, without which this sweep would have been as blind as the
one it replaced.

The findings came instead from asking a different question of the same inputs: not *does it crash*
but **does it NOTICE**.

### 3.9 P14 — runtime cost, and a crash the crash-gates could not see

### C54 · A used cyclic type alias aborted the compiler — CLOSED (D47a), and it reshapes C16

`lower_type` expands an alias by recursing into its target, so a cycle is unbounded recursion. Five
shapes all died the same way as soon as the alias was USED:

```
$ delulu check cyc.delulu          # type A = A ; fn f(x: A) -> Int { 1 }
thread 'delulu-main' has overflowed its stack
exit code: -1073741571             # 0xC00000FD = STATUS_STACK_OVERFLOW
```

`type A = A`, `type A = B; type B = A`, `type A = List[A]`, `type A = iso A`, `type A = fn(A) -> Int`.
A hard crash from three lines of ordinary source — and for anything that compiles code it did not
write (an editor, a CI runner, a registry) a denial of service.

**C16 was right about what it measured and wrong about the class.** P2 recorded cyclic aliases as
hygiene on the evidence that 5000-deep terminating chains resolve and that a secret cannot launder
through a cycle. Both still hold. What it never tested was a cycle that is USED — and the declaration
alone is harmless precisely because nothing lowers it. C16 is closed here with its severity corrected
rather than left standing as "low".

⚠ **And note what the crash was invisible to.** A stack overflow aborts without printing `panicked at`,
so the no-panic sweeps — which match exactly that message, as D44c made them — could not see it. That
is the **third** gate in this campaign blind to the failure it exists to catch: D42a's coverage law
proved existence rather than exercise, D44c's sweep keyed on exit 101 while the CLI maps a worker panic
to exit 2, and now a sweep that matches a panic message against a failure that produces none. The
durable lesson is not about any one gate: **ask what signal a gate keys on, then ask what failure
produces a different signal.**

The fix runs before anything lowers a type, reports **DL0304** at each participating declaration with
the chain named (`A = B = C = A`), and has `lower_type` refuse to expand a cyclic alias — so the crash
is structurally impossible, not merely diagnosed. Only alias→alias edges are walked, which is what
keeps a recursive `type Node { next: Option[Node] }` and a recursive `type Tree = Leaf | Branch(Tree)`
legal: those are nominal and are never expanded. Both are tested.

DL0304 was **generalized, not duplicated** — "a cycle in the declaration graph (imports, or type
aliases)" — because the reason is identical in both graphs: resolution has to terminate. Same
discipline as DL1511 in D42b.

### C53 · An unused alias target was never resolved — CLOSED (D47b)

`type Meters = Metres` checked clean; DL0301 arrived only at a use site, and in a library whose own
code never uses the alias, on a consumer who did not make the mistake. Verified **pre-existing**, not
caused by D46a: `type X = (Nonexistent)` and `type Y = List[Nonexistent]` were accepted too. The same
pass now lowers each alias target at its declaration, reusing the real resolver rather than
duplicating its notion of which names exist — the alternative was a second list of builtin type names,
which is the drift shape this campaign has closed four times.

### C55 · Runtime record field access is O(record width) — NAMED LIMIT (D47c)

The interpreter was checked for C48's clone-per-access shape and does **not** have it: `Interp::field`
clones only the value it finds. It does scan linearly. Measured with total field reads held constant
at ~200,000 and only the width varying:

| fields | µs per 1k reads |
|---|---|
| 50 | 165 |
| 200 | 300 |
| 800 | 1,570 |
| 3,200 | 5,071 |

Linear in width. **Published rather than fixed**, with the reasoning on the record: for the widths real
programs use, a linear scan over a short `Vec` is the *faster* representation, and removing the cost
means either a per-instance map (paying memory on every narrow record) or static field indices through
the DIR. It is linear, not quadratic, and the constant is small — the opposite of C48, which was
accidentally quadratic *and* allocated per access. Full table in `measurements/scale/RECORD.md`.

### C51 · The review surface could not review a real package — CLOSED (D45)

`delulu authority <dir>` ran the single-package loader while `build`, `check`, `lock` and
`authority --diff` all resolve the dependency graph. So:

```
$ delulu build     app   →  ok: `app` built clean (2 package(s), 2 module(s))
$ delulu authority app   →  error[DL0303]: unknown module `lib` imported by `app`
```

Every package with a dependency was refused by the one command whose purpose is answering "what can
this do to my system" — and in a monorepo that is nearly every package. The supply-chain question is
*exactly* the case it could not handle.

**The naive fix would have traded this for a worse bug, and nearly did.** Routing every
`authority <dir>` through `resolve_workspace` looked obvious — until measurement showed
`resolve_workspace` requires a manifest and reports DL1004 without one, which would have refused a
plain directory of modules: legal since C26/D33 and deliberately preserved by C50. A manifest is what
makes a directory a package, so its presence now selects the loader. Both cases are tested.

Verified the way the phase brief demanded, before shipping: a no-dependency package's report is
**byte-identical** before and after, on both the human and `--json` surfaces, for a simple package and
for one exercising multiple modules, secrets and `Net`. One deliberate improvement rides along — a
library package with no `fn main` used to be reported as `Authority of \`package\``, a placeholder;
the root package's name is available once the graph is resolved, so it is used.

### C52 · A lockfile could lie about a dependency, and the CI gate believed it — CLOSED (D45)

`verify_locked` recomputed `content_hash` and `authority_hash` from reality and compared them to the
stored hashes. It never looked at `effects`, `cap_kinds`, `secrets` or the scope lists — **the fields
a human opens a lockfile to read.** A lockfile could therefore claim a dependency has no effects and
no capabilities while that dependency genuinely calls `h.get`, and `build --locked` printed
**"built clean"**.

What made it undeniable is the disagreement between two commands reading the same file:

| command | verdict on the forged lockfile |
|---|---|
| `delulu authority --diff <lock> <dir>` | `lib: + effects Net` … `verdict: WIDENING` |
| `delulu build <dir> --locked` | `ok: built clean` |

The interactive review command caught what the automated gate did not — backwards, because CI is
where nobody is looking.

A semantic attack matrix on the lockfile (fifteen things an attacker or a bad merge actually does)
went from **15 accepted to 2**, with the untampered control still building throughout:

| tampering | before | after |
|---|---|---|
| dep effects narrowed (hide `Net`) | accepted | **DL1002** |
| dep `cap_kinds` emptied | accepted | **DL1002** |
| dep effects widened | accepted | **DL1002** |
| dep secrets widened | accepted | **DL1002** |
| dep net/fs scopes widened to wildcards | accepted | **DL1002** |
| recorded version ≠ package version | accepted | **DL1002** |
| duplicate entry with wider authority | accepted | **DL1011** |
| lock format `version = 999` | accepted | **DL1011** |
| `authority_hash` forged | DL1002 | DL1002 |
| `content_hash` forged / hashes emptied | DL1010 | DL1010 |
| dep entry deleted | DL1011 | DL1011 |
| dependency SOURCE tampered | DL1010 | DL1010 |

Three of the new refusals restate rules this project had already made elsewhere. An unreadable lock
format pins **nothing** rather than being read as version 1 — the same rule as an unverifiable
signature algorithm (DL1908), and it reuses DL1011 because "nothing is pinned" is exactly what DL1011
already names (`Lockfile::parse` documents the same posture for a garbled file). A duplicated entry is
an ambiguity **refused rather than resolved** — the C40 rule, third application. And the field
comparison is written as a **destructuring** `let LockEntry { .. }`, so a field added to the type
later cannot compile until someone decides whether it belongs in the check: the C31/C34/C35/C44
pattern answered structurally rather than by another hand-maintained list.

**Two residuals, named rather than quietly left:**

- *A lock entry for a package that is not in the graph is still accepted.* The verification iterates
  the resolved packages and looks each up, so extra entries are never examined. They affect nothing
  that is verified, and refusing them could break a legitimate superset lockfile, so this is recorded
  rather than changed.
- *`accepted_by` can be edited freely.* **Verified as conferring nothing**: it is written by the
  `--accept-authority` flow and is never read to make a decision, so a forged value is a misleading
  label and not an escalation. It is also not re-derivable from source, so nothing can validate it —
  which is precisely why it is excluded from the destructured comparison, by name and with a reason.

### What P13 verified and did not change

- **`accepted_by` is a record, not a gate** (above) — grep-verified across the tree, not assumed.
- **Hash-protected tampering was already caught** on every shape tried: forged `authority_hash`
  (DL1002), forged or emptied `content_hash` (DL1010), a deleted entry (DL1011), and a dependency's
  source edited under a valid lockfile (DL1010). D28's and P3's work holds.
- **The manifest pin still bounds a dependency independently of the lockfile** — C52 is a
  review-integrity defect, not an authority escalation, and is described that way throughout.

⚠ **Method note, and it nearly produced a false report of catastrophe.** The first run of the lockfile
attack used plain `delulu build` and reported **all fifteen tamperings accepted**. `build` does not
consult the lockfile; `--locked` is the verb that verifies it. The sweep was measuring a command that
was never claiming to check. Before reporting a supply-chain surface as unprotected, confirm the
command under test is the one that makes the guarantee.

### C46 · Should a refused command prove liveness? — OPEN, owner-reserved

C39 made the stepped clock charge a refused attempt the same simulated time the wall clock charges it,
which is a fidelity fix and nothing more: both clocks now agree, and neither treats a refusal as a beat.

The question underneath is not answered and is not the chef's to answer. The dead-man's documented remit
is *"silence, not malice"*, and a program issuing refused commands is not silent — under both clocks it
keeps its lease as long as it keeps interacting fast enough. But a controller whose every setpoint is out
of envelope is malfunctioning, and taking a machine away from a malfunctioning controller is what this
mechanism is for. Both readings are defensible, which is exactly why it must not be settled by an
implementation detail. Whichever way it goes, it changes when a machine stops moving. **Present, don't
decide.**

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
| P15 | Authority + Guard cross-stage audit | the capstone: consistency across all ten stages at once — **discharged in `AUTHORITY_GUARD_CAPSTONE.md`** |
| P16 | Cross-platform re-verification and close-out | Windows, Linux, and an honest statement about macOS |

Phase state is tracked in the working session, not here; this table is the shape, and §3 is the
durable record.

**P15's deliverable is a separate document** — `docs/design/AUTHORITY_GUARD_CAPSTONE.md` — because it
is the one artifact a reader should be able to open without reading this whole ledger first. It
discharges the commission's 18-item authority checklist and 14-item Guard surface checklist item by
item, each with the evidence and where the evidence lives, and it ends with the limits the audit did
**not** settle stated in the same voice as the successes. Three of the fourteen Guard surfaces do not
exist in v1.x, and it says which and why rather than marking them covered.

## 5. Standing limits this campaign does not remove

Carried forward and restated so that no reader of this document alone concludes otherwise:
**macOS has never been executed**, no driver for any real device ships in-tree, certification is
**none** under every regime, `ForeignCall` remains an enumerated hole in the proof rather than a
closed one, RFC 0001's comment period remains open until 2026-08-05 with two recorded process
deviations against it. **Surface-syntax morphs now exist (D35)** but only for single files read
through a `//! morph:` pragma — package sources must be canonical, and plugin-delivered morphs and
per-reader LSP view morphs are not built.

### C63 · The Book credited the wrong harness, at 25× the real number — CLOSED (D58)

Found by answering a direct question from Jesse: *"both cli and compiler are working and have all the
features of DeluluLang right?"* The answer is no, and the interesting part is not the gap — the gap is
deliberate and the Stage-3 spec states it plainly — but that **the Book described it as something
else.**

**What is actually true, and is fine.** The interpreter is the reference engine and runs the whole
language. The WASM backend compiles a **fragment**, and outside it a program is refused as **DL1201**
and falls back to the interpreter rather than miscompiling. `STAGE3_SPECIFICATION.md` says "the
**pure-Int/Bool fragment**" and "constructs outside the fragment are `CompileError` (DL1201-class)";
`crates/delulu-wasm/tests/conformance_parity.rs` says "byte-identical observable output for every
program **inside its fragment**" and tests the boundary itself. Fail-closed at the edge is what makes
a partial backend safe to have. Measured over corpus + examples: **6 of 19 entry-point programs
compile to WASM.** `.len`, `.trim`, `.split`, `.narrow`, `.fs_write`, embedded Python and actor state
outside the `Int`/actor-reference subset are all DL1201 — so **none of the Book's own guide chapters
build to a `.dwx`.**

**What the Book said.** *"this 'two-engine parity' is enforced by a differential fuzzer running tens of
thousands of programs on both engines with zero divergence. When two independent implementations agree
on 50,000 random programs…"* Three things wrong:

1. The generative two-engine sweep runs **2,000** programs, not 50,000 — a 25× overstatement of a
   headline number.
2. The crate actually *named* the differential fuzz harness, `delulu-fuzz`, depends on
   `delulu-check` and `delulu-runtime` and **cannot run the WASM backend at all**. Its dependency
   list is the proof.
3. "Two independent implementations" invites the reader to think the second one runs the language.
   It runs about a third of the corpus and none of the Book's teaching examples, and the chapter
   never said so.

**And the evidence it misattributed is better than the claim it was attached to.** What `delulu-fuzz`
proves is that for every accepted program the observed runtime effects are a **subset of the effect
row the checker computed for `main`** — the executable Effect-Soundness theorem, invariant 12. That is
the one bug this language exists to prevent, and the old paragraph spent it on backend parity.

**Fixed (D58).** Chapter 9 now states the fragment, quotes 2,000, names DL1201 as the boundary, and
carries the correction visibly rather than swapping the number silently. Two gates:
`book.rs::the_books_engine_parity_number_matches_the_harness` reads the loop bound **out of the parity
test** and requires the prose to match (and refuses the two stale phrasings by name), and
`the_differential_fuzz_crate_still_does_not_run_the_wasm_backend` asserts the dependency list the
prose now relies on. Observed failing against the old text.

### C64 · A named record literal could not be passed to a function — CLOSED (D62)

Found by writing the tier-3 and tier-4 programs Jesse asked to be run end to end, and hit **three times
in one session** while writing perfectly ordinary code — worked around each time by restructuring, which
is how a defect this ordinary stays invisible.

```delulu
type P { x: Int }
fn f(p: P) -> Int { p.x }

let p = P { x: 1 }
f(p)            // WAS DL1603: cannot store `ref` (aliases as `ref`) where `val` is required
f(P { x: 1 })   // the SAME value, inlined: accepted
```

Refused: a `let`-bound **record literal** and a `let`-bound **list literal**. Accepted: the identical
value from a **call's return**, from a **`match` binding**, a `let`-bound **variant** literal, a
`let`-bound `Int`, field access on the let-bound record, and the literal **inlined**. Four other
spellings of the same program compiled, so the restriction was not protecting an invariant — it was an
artifact of one binding form. The diagnostic also said the value *"aliases as `ref`"* where there was one
binding and one use.

**Mechanism.** A record or list literal evaluates to `K::Fresh { lift_val, lift_iso }`; `Stmt::Let`
carries that onto the binding as `fresh_lift` — whose doc comment says it is `Some` *"while the binding
still holds a fresh, never-escaped literal"* — and the lattice already permits the lift
(`subcap(Iso, Val)`). `fresh_lift` was consulted in exactly one place: `check_return_position`.

**Closed (D62)** by consulting it at `val` arguments too, with two guards that are the actual ruling:
the lift **costs the caller its write access** (a `val` is immutable *and* sendable, so the callee may
keep it), and an **author-written `ref` is never lifted**. That second guard was missing from the first
implementation and an existing regression test — the one pinning the `val`-parameter laundering channel
— failed. The test was right; the fix was wrong. Worth recording: a loosening of a soundness-bearing
rule checked only by the author's own new tests is a loosening nobody has checked.

### C65 · The review surface could not review the shipped artifact — CLOSED (D59)

Found by answering Jesse's question about whether the features work on "both CLI and compiler". A
`.dwx` is what you *ship*, and Chapter 9's claim for it is "authority that travels with the code":

```
$ delulu build frag.delulu --target wasm -o frag.dwx
ok: wrote `frag.dwx` (1163 bytes) with authority embedded as `delulu:authority`
$ delulu run frag.dwx --grant console
running `frag.dwx` — authority verified; declared effects: Write
$ delulu authority frag.dwx
error: cannot read `frag.dwx`: stream did not contain valid UTF-8
```

The runner reads the embedded manifest and announces it. The command whose entire job is "everything
this program can do" fell through to the source loader and died on an encoding error. **The manifest
was always there; nothing asked it.** `check`, `why` and `atlas` did the same.

**Fixed (D59).** `authority <file>.dwx` reports the embedded manifest through the **same**
`read_and_verify` the runner uses — so the review surface cannot vouch for bytes the runtime would
refuse, and a witness proves a one-byte tamper yields **DL1202 from both**. The report is labelled a
COMPILED ARTIFACT and says what it therefore cannot tell you (no per-function purity, no `why` chain —
those need source). `check`/`why`/`atlas` now name what the file is and point at the two commands that
can read it, and detection is by content (`\0asm`) rather than extension, so a renamed artifact gets
the same answer.

### C66 · `fmt` reported success on a file it ignored — CLOSED (D59)

`delulu fmt notes.txt` printed "reformatted 0 file(s), 0 already canonical, 0 refused" and exited
**0**. `collect_delulu_files` keeps only `*.delulu`, so an explicitly named file of any other kind was
silently dropped — nothing done, success claimed, which is C26 exactly. A *directory* filtering to
nothing is different and still fine: filtering is the entire point of walking one.

### C67 · The review surface is unreadable at scale — CLOSED (D60)

Found by running the 24,630-line generated program Jesse asked for. The authority report proved
**2,536 of 2,539 functions pure** — correct, and exactly the language's central claim — and printed
all 2,536 names on **one line of 28,242 characters**, burying the six lines a reviewer opened it for.

D38 already ruled this shape for diagnostics: *bounded for the human, complete for the machine.* The
review surface never got the same treatment. `--json` was already complete (`authority.pure_functions`
carries every name), so the fix is only to the human channel: 40 names and the count. **28,536 bytes →
912.** The count was always the load-bearing part anyway — "2,536 of 2,539 cannot touch the world" is
the finding; the roll-call is not.

### C68 · The Book's gate counted its examples instead of compiling them — CLOSED (D60)

`every_book_code_block_has_a_checked_sample` asserts that the NUMBER of ` ```delulu ` blocks equals the
number of files in `docs/book/samples/`. It never compared a block to a file. Measured: **0 of 10
blocks were slices of any sample.** They were paraphrases, and paraphrases are not compiled.

**The live consequence.** Chapter 14 — *"Real adoption means calling C and Python"* — showed:

```
let m = root.foreign[mathlib](root.foreign_load())?
```

which does not compile: `Root` has no field `foreign` (DL0405), and `mathlib` is a type, not a value.
There is no method type-argument syntax in the grammar, so the lib type cannot be written at the call
at all — it is **inferred from an annotated parameter**, which the chapter never showed. And
`08_foreign.delulu`, the sample "backing" it, contained only the `foreign` declaration: the safe half,
with no call site. So the single chapter a developer reads to call C taught a form the compiler
rejects, and the gate built to prevent that was counting.

This is C37's shape — existence proved, correspondence not — in the front door rather than in a test.

**Fixed (D60).** The chapter's block is now a **literal slice** of `08_foreign.delulu`, which compiles
on every run and which was additionally **executed against the real Windows CRT** (`ucrtbase.dll`:
`cos(0.0)=1.0`, `sqrt(144.0)=12.0`). A new gate checks *correspondence* for blocks declared backed,
and refuses the broken form inside any code block by name. The gate is scoped to what has been
converted so far and says so, because the honest state is partial — a gate claiming more than it checks
is the thing being fixed.

### C59 · A multi-package program could not be run — CLOSED (D61)

`delulu run` took one `.delulu` file or a `.dwx`. Everything else about packages worked — resolution,
per-module visibility, authority ceilings, pins, the authority report, `why` across a package boundary
— and none of it could be *executed*. `kind = "bin"` was a manifest field the toolchain could not
honour, and the plainest evidence was `examples/greeter/`: a two-module binary package shipped in this
tree since Stage 2 that had never once been run.

**Closed by flattening, after the real check.** `check_workspace` remains authoritative; only a
program that passes it is flattened into one module for the interpreter.

**The instructive part is the implementation that was wrong.** Merging the module ASTs is the obvious
approach and it produced a *wrong answer* rather than an error: each module is parsed separately, so
their `NodeId`s both start at zero and overlap, and the checker's node-keyed side tables — `node_types`,
and through it the reference-capability analysis — then read one module's entry for another module's
expression. The first attempt reported `cannot store 'box' where 'val' is required` about the tier-4
corpus program that the workspace had just accepted. It was caught by disbelieving the diagnostic and
hand-flattening the same seven modules into one file, which checked clean. Concatenating source and
parsing once gives one numbering; `package_run.rs` pins it as a regression by name.

**The remaining edge is deliberate.** Two modules may legally declare the same top-level name, and
flattened they are one scope. That is refused — with the colliding name, and with the sentence *"the
program is not wrong"* — because `check`, `build` and `authority` all handle such a program and only
the runner cannot. Silently running whichever definition merged last would be the exact failure this
campaign exists to find.

## 6. Close-out (2026-07-26, ruling D50)

The campaign ran sixteen phases over three days: a baseline and breadth sweep, the front door, one
adversarial pass per stage for all ten stages, scale, fuzzing, performance, the Authority + Guard
capstone, and this. **69 findings have been filed. 68 are closed and 1 is a published limit
(C55).** Nothing is open, and nothing is carried as "does not reproduce" — the one entry that was
(C12) reproduced on re-test and is closed under D64.

*(Those five numbers were counted from the table above by script, not estimated. The first draft of
this paragraph said "57 filed, 48 closed, 3 limits, 4 open, 2 not reproducing" — written from memory
before C56 was closed, and wrong on every count. Correcting it is noted rather than quietly fixed,
because a close-out that miscounts its own findings is the exact defect this campaign spent sixteen
phases on.)*

**The count grew after the close-out, and that is the honest shape of it.** At close-out the total
was 58 filed / 54 closed / 2 open. Three of the four items open at that point (C58, C59, C61) were
found by doing the work C7 asked for — writing the multi-package corpus — and the fourth (C60) by answering
Jesse's question about whether the D52 adapter gate was right, which it was not. C62 was found by
*staging the edits to this very file* and noticing the diff was twenty times larger than the change.
A campaign whose finding count only ever falls is a campaign that stopped looking.

### What shipped

| Phase | Ruling | Headline |
|---|---|---|
| P0–P1 | D24, D25 | The front door was false — a v1.0-tagged tree said "Stage 1, under construction". `apply(double, 21)` type-checked and faulted at runtime, hidden because the Book's sample gate checked and never ran. |
| P2 | D26 | Trojan Source refused (DL0107) — the security property a review-focused language must have. |
| P3 | D28 | The semver-authority law was blind to secret-scope widening. |
| P4 | D29 | The two engines did not fault alike, and the fuzz harness counted `(Err, Err)` as agreement. |
| P5 | D30–D34 | All 16 builtin type names and 10 core effect names were shadowable, every shadow silently inert. The report knew a credential could leave and never said so. |
| P6 | D36, D37 | A lease token for a **revoked** grant redeemed `Ok`; `audit tail` verified nothing; the Guard enumerated 7 of 8 dimensions. |
| Crash hunt | D38 | `--json` emitted nothing on failure across ~every subcommand; a 10 KB file produced 76 MB of stderr. |
| P7–P8 | D39, D40 | A plugin ceiling could advertise authority the model cannot confer; a `Root` slice silently lost `computes` at an actor boundary. |
| P9–P10 | D41, D42 | `fmt` merged comment paragraphs; **the coverage law proved a witness existed, not that it exercised its anchor.** |
| P11 | D43 | **A simulation could not run out of time** — and the same fix stopped a controller being told its setpoint was wrong when it had lost the machine. |
| P12 | D44 | A quadratic field lookup behind a `.clone()`; **an empty `delulu.toml` crashed the build, and the crash gate could not see it.** |
| P13 | D45 | **A lockfile could lie about a dependency and `build --locked` said "built clean"** — while `authority --diff` on the same file said WIDENING. |
| — | D46 | The four owner-reserved questions decided under Jesse's explicit authority. |
| P14 | D47 | **A used cyclic type alias aborted the compiler** with a stack overflow. |
| P15 | D48 | The Authority + Guard capstone: 18 + 14 items discharged (`AUTHORITY_GUARD_CAPSTONE.md`). |
| P16 | D49, D50 | The trace buffer bounded; both platforms re-verified; the front door re-tested from a clean clone. |
| — | D51, D52 | The depth bound made API with its stack cost measured; the hardware driver's provenance checked before spawn. |
| — | D53–D56 | **A signature that verified under an ATTACKER'S key satisfied D52's strongest flag**; both numeric columns now refuse a literal that is not the value written; tier 4 built and the corpus RUN; C55 re-measured at the widths its claim was about. |

### Still open, each checked rather than assumed

Every item that stood here when this section was first written — **C7, C17, C21** — is now closed
(D55, D54, D51). They are named rather than deleted, because a close-out that silently loses an item
it once listed is the kind of drift this campaign existed to stop. What replaced them are four
findings that did not exist then, three of which were found by *writing the corpus C7 asked for*:

- **C59 — CLOSED by D61.** `delulu run <package-dir>` executes a multi-package program: the graph is
  checked authoritatively, then flattened on SOURCE (an AST merge overlaps `NodeId`s and corrupts the
  checker's node-keyed tables — that was written, caught, and is now a named regression test). The
  four-package corpus tier and the shipped `examples/greeter/` both run. **What remains is the
  fail-closed edge of that fix, not a leftover:** two modules declaring the same top-level name are
  refused by the runner rather than resolved by merge order, and the refusal says the program is
  correct. Per-module resolution inside `Interp` would lift it.
- **C58 — CLOSED by D65.** Re-framed onto the import that brought the signature in, with where the
  type is declared and both fixes; a misspelling is left untouched, because it has no true advice to
  add. The tempting fix — Rust's private-in-public rule — was measured against the shipped corpus and
  **rejected**: three tier-4 modules legitimately name a type behind a plain `import`, and the rule
  would have outlawed the diamond the tier exists to demonstrate.
- **C60 — CLOSED by D66.** Written into the broker's existing hash-chained audit log as
  `adapter.provenance`, carrying the signer and the pinned key. The earlier reading — "neither home
  fits" — was half wrong: the chain is complete and already has a reader and a default location;
  nothing was writing the adapter decision *into* it. **Refusals are recorded too**, before the
  refusal is acted on, and a named sink that cannot be written refuses the run.
- **C69 — CLOSED by D66, and found by causing it.** The first version of D66 defaulted to the shared
  `~/.delulu/audit`. A hash chain has one writer (`open` reads the head, then appends), the broker
  satisfies that and a short-lived `delulu run` does not — so the parallel suite produced a chain
  that failed `delulu audit verify`, with two physically interleaved half-lines. Caught by
  `every_listed_subcommand_honors_a_valid_invocation`, a gate written for something else. Recorded
  because the assumption is real and undocumented, and because a fix that damages the artifact it
  was written to create is worth remembering.
- **C64 — CLOSED by D62.** The lift happens at `val` arguments, the caller gives up its write
  access (witnessed: a write after the lift is refused), and an author-written `ref` is never lifted —
  that last clause exists because the first implementation lacked it and **an existing regression test
  caught it**, which is the part worth remembering.
- **C61 — CLOSED by D63.** `let _` / `var _` bind the ordinary name `_`, which cannot be read
  because a bare `_` lexes as its own token and never as an identifier — write-only by construction
  rather than by a rule someone could forget to enforce.

### Published limit

- **C55 — runtime record field access is O(record width).** Linear rather than quadratic, and the
  real fix is static field indices through the DIR. Still the only finding carried as a limit rather
  than closed or open — but the *reason* has changed. Its claim was that "for the widths real
  programs use — five to twenty fields — a linear scan is the faster representation", and the table
  under it began at fifty: the claim about the range that matters was an extrapolation past the
  smallest measured point. Measured at the narrow end (D56), the curve is **U-shaped** — 444 µs/1k
  reads at width 2, a **minimum of 188 at width 20**, 415 at width 200. In the band the claim was
  about, the field scan is not the cost; interpreter overhead is. The limit stands and its reasoning
  is now data.

### Does not reproduce

- **C12 — CLOSED by D64, after this section wrongly said it did not reproduce.** It reproduced
  verbatim. The clearing re-test used `Int`/`Str`, which print themselves; the failing shapes —
  records and sums, the only ones stored as an index — were never re-tested. The caution in the old
  wording ("saying closed would claim knowledge this ledger does not have") was the right instinct
  pointed at the wrong risk: the danger was not over-claiming a fix, it was under-testing a clear.
- **C16 — a cyclic type alias is silently accepted.** Superseded by C54: the declaration is accepted,
  but *using* one crashed the compiler. Closed under D47a with its severity corrected.

### The two rules this campaign actually produced

Both are in D48d, and they are the part worth carrying into whatever comes next.

1. **A hand-maintained list of authority-bearing things falls behind the type that defines it, and
   nothing notices.** Six instances: C31, C34, C35, C44, C52, C57. The answer is never "remember to
   update the list" — it is a compiler-enforced pattern (a struct destructuring that will not compile)
   or a source-scanning gate. Where a dependency edge allows it, better still is one list referenced
   by both sides.
2. **A gate is blind to the failure it exists to catch. Ask what SIGNAL a gate keys on, then ask what
   failure produces a different signal.** Four instances: a coverage law that proved a witness existed
   rather than that it exercised its anchor (D42a); a no-panic sweep keyed on exit 101 while the CLI
   deliberately maps a worker-thread panic to exit 2 (D44c); a sweep matching `panicked at` against a
   stack overflow, which prints no such text (D47a); and a cross-parser law that checked agreement only
   where both sides said *yes* (D43e).

A third pattern is worth naming even though it produced no ruling: **three of this campaign's findings
were introduced by earlier fixes in this same campaign** (C26/D33's courtesy note caused C49's panic,
and D46a's grammar change surfaced C53). A repair needs its own skip-branch analysis. "What if there is
nothing to name?" is one of them.

### Method notes that cost real time

Recorded because the next person will otherwise pay for them again:

- **A witness that passes against the old code witnesses nothing.** Two tests in this campaign passed
  before their fix existed — one because its fixture never reached the defective path. Confirm the
  failure *first*.
- **Never conclude absence from truncated output.** Two "no panic here" readings were `head -3`
  artifacts; the panic was below the fold.
- **Confirm the command under test is the one making the guarantee.** A lockfile sweep run against
  plain `build` reported all fifteen tamperings accepted — a false catastrophe. `--locked` is the verb
  that verifies.
- **A patch script must assert its replacement applied.** A heredoc turned `\n` into a real newline,
  the replacement silently matched nothing, and the script printed success anyway.

## C70 · A normative runtime rule was false on the concurrency path — CLOSED (D67)

Found during the production-readiness review, by asking a question the campaign had never asked of
this subsystem: *`main.rs` reserves a big stack so the depth guard fires — which other threads run
interpreter code, and what do they reserve?* The answer was: the actor scheduler's workers, and
nothing.

```
$ delulu run main_deep.delulu  --grant console     # down(1000) called from fn main
1000                                                exit 0

$ delulu run actor_deep.delulu --grant console     # the SAME down(1000), inside a behavior
thread 'delulu-actor-0' has overflowed its stack    exit 0xC00000FD
```

Same function, same depth, same process, same binary. `ref.rule.runtime.faults-are-diagnostics`
promises *"a diagnostic with a code, never a host crash"* and names recursion depth explicitly; the
reference marked it **covered**. Windows brackets: abort above depth **43** (debug) and between
**300** and **400** (release), against a documented bound of **10,000**.

**Three things made it survivable for two stages, and all three are already-named patterns:**

1. The rule *"a thread that runs a DeluluLang program reserves a stack sized for the depth bound"*
   was a private constant in `main.rs` — **design rule 1**, seventh instance.
2. Its witness recurses in `fn main`, the one thread where the rule already held — **design rule 2**,
   in a new costume: not a gate keyed on the wrong signal, but a gate keyed on the right signal *on
   the wrong thread*.
3. **A stack overflow prints no `panicked at`** (D47a), so no no-panic sweep could see it.

**This also reframes C21, which is the reason it is worth reading twice.** C21 was filed as a
*library-embedding* residual — a hypothetical embedder on a small stack — and D51 closed it by making
the bound a contract (`with_max_depth`, `STACK_BYTES_PER_DEPTH`). D51 built exactly the right
mechanism. **Nothing in the tree was calling it, and the caller that needed it most was not an
embedder at all — it was DeluluLang's own actor runtime, reachable from the shipped CLI with an
ordinary program.** A contract with no caller is a contract nobody is keeping.

Closed by D67: one definition of the budget in `delulu-runtime`, actor workers reserving it with
their bound sized to match, and a source-scanning gate that fails unless every thread-creation site
in the tree is either sized or classified. D67 also closed a quieter contradiction it exposed — the
reservation (512 MiB) and the published per-depth budget (80 KiB × 10,000 = 800 MiB) had disagreed
since Stage 9, in the direction where the advice to embedders was safer than what the toolchain gave
itself.

### What this campaign does not claim

**macOS has never been executed** — not once, in any phase; there is no hardware and nothing was
emulated. **No physical device has ever been commanded**; every demonstration drives the simulator.
**Certification is NONE.** The hardware adapter is an operator-supplied subprocess with **no signature
check**. Post-quantum cryptography is gated behind `--unstable` because the adopted implementations are
unaudited by their own authors. The full list, in the same voice, is §3 of
`AUTHORITY_GUARD_CAPSTONE.md`.

