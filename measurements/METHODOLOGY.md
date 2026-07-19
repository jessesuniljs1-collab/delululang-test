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

## 2. Studies B and C

See `study-b/METHODOLOGY.md` and `study-c/METHODOLOGY.md` (Stage 9e).

---

## 3. Reproducing

```
cargo run -p delulu-measure -- study-a          # regenerates results.json + REPORT.md
cargo test -p delulu-measure                    # the study's own tests, incl. the fences
```

The corpus generator is deterministic and the toolchain is pinned by `rust-toolchain.toml`. A run on
a different machine should reproduce every result except the wall-clock timings, which are expected
to differ and are labelled as such.
