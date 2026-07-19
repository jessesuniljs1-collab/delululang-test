# The DeluluLang measurement program — methodology

**Governs:** Study A (this stage), Studies B and C. **Binding:** Constitution §9's honesty clauses
apply to every sentence in this directory. **Reproduce:** `cargo run -p delulu-measure -- study-a`.

---

## 0. The rules this program follows

1. **No number appears in prose that is not in `results.json`.** The reports are generated from the
   raw data, not written alongside it.
2. **An empty experiment is a failure, not a success.** Zero injections is a 0% catch rate, not a
   vacuous 100%. This is enforced in code, not remembered by a person.
3. **Every experiment carries a negative control.** A test that cannot fail proves nothing, and a
   campaign that refuses everything would score perfectly while measuring nothing.
4. **Threats to validity are part of the deliverable.** They are in this file, linked from every
   report, not in an appendix nobody opens.
5. **Where a lane cannot run, it is labelled UNRUN** rather than being dropped so the remainder
   looks complete.

---

## 1. Study A — whole-program authority verification at scale

### 1.1 The claim under test

*A patch release that adds an effect anywhere in a dependency graph is caught, every time.*

This is a claim about a **mechanism**, not a heuristic. The required catch rate is therefore 100%,
and anything less is a hole to find rather than a number to soften. That asymmetry is deliberate:
a 97% heuristic is useful; a 97% mechanism is broken.

### 1.2 The corpus

Generated deterministically by `delulu_measure::corpus` — 5 chains × 5 packages = **25 packages**,
each chain **depth 4** from application to leaf. The five chains are shaped like the archetypes the
spec names: a CLI tool, a parser, an HTTP client stack, a static site generator, an agent-tool
server.

Generation is deterministic with no randomness anywhere, so the corpus is byte-identical on every
machine and every run. It is generated rather than hand-written specifically so that no one can
shape individual packages, one at a time, into the ones the mechanism happens to catch.

Every library begins **pure**. The experiment is "an effect appears where none was before", so a
library that already declared one would have nothing to detect. This is asserted in
`corpus::tests::every_library_starts_pure`, not assumed.

### 1.3 The injection

For each library in each chain (20 sites), a variant is produced where that library:

1. **gains an effect** — it derives a console from the `Root` it already receives, and writes;
2. **declares that effect honestly** in its own manifest; and
3. **ships as a PATCH bump** (0.1.0 → 0.1.1).

Steps 2 and 3 are what make this the real scenario rather than a strawman. A package that lies
about its own authority is caught by its own authority check (`DL1009`) — easy, and not what anyone
fears. The dangerous case is a dependency that is **completely truthful** about its new authority
and simply expects nobody to read the diff, shipped as the kind of patch release a maintainer
applies without thinking. The consumer's lockfile pinned the old authority; the build must refuse.

### 1.4 The two fences that make the result meaningful

Both exist because the first version of this study produced a **false 100%**, and the failure is
recorded here rather than quietly fixed:

**The validity fence.** The initial injection emitted code that did not parse. Every build failed,
the campaign scored 20/20, and the number was worthless — the refusals were syntax errors the
authority mechanism had not earned. Now every mutated package is checked to **compile on its own**
before its result is recorded, and an injection that does not compile is an experiment defect that
invalidates the run (`injection_valid` in the raw data).

**The negative control.** The identical pipeline — fresh copy, lock, rebuild — is run once per
chain with **no mutation at all**. Every control must build clean. Without it, a toolchain that
refused everything would score a perfect catch rate. `mechanism_holds` is false unless all controls
pass.

### 1.5 What the result was

20 of 20 injections caught, all 20 mutations valid, all 5 controls clean. Refusals carried
`DL1001` (dependency authority exceeds its pin) and `DL1010` (content hash mismatch) at every site,
with `DL0501`, `DL1002` and `DL1009` at a subset. Full data in `study-a/results.json`.

### 1.6 Threats to validity — Study A

These are real limits, stated plainly. None of them is walked back elsewhere in the project's prose.

- **The corpus is synthetic.** Twenty-five generated packages shaped like real software are not real
  software. They are small, layered uniformly, and each has exactly one dependency per level. Real
  graphs are wider, messier, and contain code that does something. The study measures the mechanism,
  not the ecosystem.
- **One mutation shape.** Every injection adds `Write` via a console derived from `Root`. A wider
  study would vary the effect kind, the position within the file, and whether the change is
  syntactically obvious. The mechanism is structural (it compares declared authority against a pin),
  so shape is not expected to matter — but "not expected to" is not "measured", and this study did
  not measure it.
- **The libraries hold `Root`.** The corpus passes `Root` down the chain, which is a real-world
  pattern and also an anti-pattern — a library holding `Root` can derive any capability it likes.
  That is precisely why the injection is possible at all. A corpus that passed narrow capabilities
  instead would make this *particular* attack impossible by construction, which is a stronger
  property of the design and a weaker test of the verification mechanism. Both facts are true and
  neither is hidden.
- **Catching an authority change is not catching malice.** A dependency that was always granted
  `Net` and starts using it for something else is **not** caught by this mechanism, and nothing here
  claims otherwise. The guarantee is about the *authority boundary*, not about intent.
- **Timing numbers are from one machine, one run.** They are reported as measured. Performance is
  measured, never promised (see `STABILITY.md` §2).
- **The mechanical-vs-manual comparison is not a controlled trial.** See §1.7.

### 1.7 Mechanical vs. manual — the honest framing

The spec asks for a comparison column against the same audit performed by a human reviewer on
equivalent Rust/npm graphs. The honest framing is **mechanical vs. manual**, not "we are smarter".

What can be said without a trial: the DeluluLang check is a *total function of the lockfile and the
package sources* — it examines every package in the graph on every build, at a measured cost of
about 20 ms per graph, and it does not get tired at 2 a.m. or on the fortieth dependency. A human
reviewing an equivalent npm graph is doing something qualitatively different: sampling, in a
diff-shaped view, under time pressure, with no mechanism that forces the authority question to be
asked at all.

**No human-trial numbers are reported here**, because none were run. Running one properly needs
multiple reviewers, blinding, and a task set none of which this stage has. Publishing an
uncontrolled anecdote as a comparison column would be exactly the kind of claim Constitution §9
forbids. The column is therefore marked UNRUN, and the qualitative difference above is stated as a
structural argument rather than as a measurement.

---

## 2. Study B — agent task success and repair loops

### 2.1 What is actually measured

**What the toolchain hands a machine.** Not which language is better; not how clever a model is.
The lanes compare how much *structured* information each toolchain gives an automated repair loop.
A DeluluLang diagnostic can carry a typed repair — id, confidence, authority-widening flag, and
byte-range edits a program can splice without understanding the language. A Python traceback
carries prose for a human.

### 2.2 The task set

The **conformance reject corpus** — 47 programs with genuine defects, already used to hold the
compiler to its diagnostics. Real defects rather than synthesised ones, and not selected for this
study, which removes the obvious way to flatter the result.

### 2.3 The loop

Deterministic, with **no model in it at all**: run the checker, apply any repair the toolchain
declares machine-applicable, re-check, repeat to a bound of 8 iterations.

One rule is absolute: **a repair flagged `authority_widening` is never applied automatically**,
however exact it is. `add_effect_to_row` silences a diagnostic by granting the program more
authority. A loop that takes that repair has not fixed the program; it has removed the objection.

### 2.4 The results, including the unflattering ones

- **4 of 47 (8.5%)** defects offered a machine-applicable repair.
- **0 of those 4** reached a clean program mechanically — 2 offered only the authority-widening
  repair (correctly refused), and 2 offered a warning-level repair that does not clear the error
  beside it.
- The Python lane: **0 of 6** defect shapes offered anything machine-applicable. One (`os.listdir`)
  produced no error at all, which is the finding rather than a gap in the harness.

The honest summary: the typed-repair channel is real and correctly conservative, and it currently
cannot drive any real defect to green without a model. "Typed repairs" invites the reader to
imagine universality; the measurement does not support that.

### 2.5 Threats to validity — Study B

- **The scripted lane is not an agent.** It measures loop *mechanics*, not task success by a real
  model. The live-model lane that would measure the latter is **UNRUN** (build order D4): no API
  keys in CI, and a remote model version is not reproducible.
- **The Python comparison is narrow** — 6 defect shapes against 47, chosen to mirror the DeluluLang
  ones. It is a reference point, not a controlled comparison. No claim is made about Python beyond
  the mechanical observation that a traceback carries no applicable edit.
- **`os.listdir` succeeding is not a Python defect.** Python has no effect declarations, so there is
  nothing to violate. It illustrates what "unauthorized-effect attempts" can and cannot mean across
  the two systems, and the harness counts only what a toolchain surfaces.
- **The Go baseline is UNRUN** — no Go toolchain on the measurement machine.
- **Repair coverage is a moving target.** 8.5% is today's number, not a property of the design.

---

## 3. Study C — the performance honesty baseline

### 3.1 The only permitted claim

Measured facts. Constitution §5.11 rejects *"faster than C"* as false; the committed claim is
*competitive with C on hot paths*, which this study **assesses** and Stage 10 **works**.

### 3.2 Method

Six benchmarks (three micro, three macro) across four lanes: the DeluluLang interpreter, the
DeluluLang WASM backend, C at `gcc -O2`, and CPython. All lanes compute the same result by the same
algorithm. Each runs 5 times; the **minimum** is reported with the spread beside it.

**Release only.** The study refuses to run against a debug build. The first run did measure debug,
and the numbers were both wrong and dangerous — authoritative-looking figures describing a binary
nobody runs.

### 3.3 The result

**2.0× to 51.1× slower than C**, depending on the benchmark. v1.0 is **not** competitive with C on
these workloads, and the report says so in those words. The constitution's claim is about hot paths
under a tiered backend with a JIT; v1.0 ships a tree-walker and a straightforward WASM backend, and
neither is that. Publishing the gap now is what will make Stage 10's numbers mean something.

### 3.4 Threats to validity — Study C

- **Wall-clock includes process startup**, and for the C lane it *dominates*: its spread exceeds its
  minimum on several benchmarks, so those figures are mostly process creation. The consequence
  points the uncomfortable way — the ratios **understate** the true compute gap, because the C
  denominator is inflated by time C did not spend computing. The 2.0×–2.9× rows are the least
  trustworthy for this reason, not the most impressive.
- **The WASM lane ran only one benchmark.** Five were refused as unsupported (`DL1201` — `var`/
  `while` constructs). That is an honest limitation of the backend at 1.0, reported rather than
  hidden by dropping the lane.
- **Six benchmarks are not a benchmark suite.** They are small, they fit in cache, and they were
  written for this study. No claim generalises beyond them.
- **One machine, one OS, one run of five repeats.** No cross-machine or cross-OS variance is
  characterised.
- **CPython is not a tuned baseline** — no PyPy, no JIT, no C extensions.

### 3.5 What Study C found that was not a number

Running `fib(24)` crashed the process with a raw stack-overflow abort — no diagnostic, no usable
exit code. The interpreter's `MAX_DEPTH` guard existed but was unreachable: a tree-walker spends
several large native frames per DeluluLang call, and the default main-thread stack ran out long
first. That made `ref.rule.runtime.faults-are-diagnostics` **false**.

Fixed in the same phase: the CLI now runs on a thread with a stack large enough for the depth bound
to be the limit that actually fires, and deep recursion reports `DL0905` as it always claimed to.
The bug is recorded here because a measurement program that finds a defect and mentions only its
timings is not doing its job.

---

## 3. Reproducing

```
cargo run -p delulu-measure -- study-a          # regenerates results.json + REPORT.md
cargo test -p delulu-measure                    # the study's own tests, incl. the fences
```

The corpus generator is deterministic and the toolchain is pinned by `rust-toolchain.toml`. A run on
a different machine should reproduce every result except the wall-clock timings, which are expected
to differ and are labelled as such.
