# DeluluLang 1.0

**Status: RELEASE (2026-07-20).** Passes the line-by-line honesty review against Constitution §9
(`criterion10_the_announcement_makes_no_unsupported_claim`). Every number below is in
`measurements/` or in a named test. If a sentence here cannot cite one, it does not ship.
The release candidate's own gate blocked this release once (criteria 1 and 8, recorded in
`docs/design/STAGE9_BUILD_ORDER.md` D19); both blockers were closed by work, not by wording
(D22 and `measurements/first-run/RECORD.md`), and the gate was re-run before this line changed.

## The principle

The creator's statement, from the project's first discussions: **freedom with authority; freedom
with responsibility.** Everyone — human, AI, company, robot — has the same standing and the same
freedom to build (Constitution §5.16: no code path inspects who you are); what bounds that freedom
is only what you were granted, and the granting side — the broker, the Guard, the audit chain —
carries the responsibility of keeping freedom from becoming harm. The compiler educates (95+
long-form explanations), warns (stable diagnostics with typed repairs), supports (the tooling),
and holds the line (grants it will not widen without a human). The metaphor is a well-run free
society; the mechanisms are what this document can actually cite.

---

## What it is

A language where **a function can only do what its type says it can do**.

Not by convention. Not by lint. By type. A function whose row is empty cannot read a file, reach the
network, or tell the time — and if you add one of those, every caller's type changes too, all the
way up to `main`.

That gives you one command:

```
delulu authority ./my-program
```

which prints **everything the program can do**, computed from the code. Not what it declares. Not
what its README says. What the compiler can prove.

## Why it exists

Because reviewing code you did not write — increasingly, code no human wrote — is now the bottleneck,
and reading a diff is a terrible way to answer "what can this thing actually do?"

The measurable version of that claim is **Study A**: a 25-package corpus with dependency graphs four
deep, and an effect injected at **every** library position — a patch release that gains an effect,
declares it honestly, and ships as a patch bump, which is what a real supply-chain attack looks
like. Not a package lying about itself; one being truthful about a change nobody reads.

**Caught: 20 of 20.** The required rate was 100%, because this is a claim of *mechanism*, not a
heuristic — a mechanism at 97% has a hole to find, not a number to round.

The full method, and the two fences that make the number mean anything, are in
`measurements/METHODOLOGY.md`. Including the part where **the first version of that study produced
a false 100%** and we caught it: the injected code did not compile, so every build failed for
reasons that had nothing to do with authority. That failure is written up rather than erased.

## What 1.0 includes

- **The language**: effect rows, object-capability authority, secrets that cannot be printed or
  compared, results instead of exceptions, actors with reference capabilities.
- **The toolchain**: `check`, `run`, `build`, `test`, `fmt` (one style, two verified laws), an LSP,
  `atlas`, and `authority`.
- **Plugins** that arrive at runtime and still cannot overreach.
- **A registry** where the index's authority summary is **recomputed server-side from the
  artifact** — a publisher cannot claim an authority they do not carry.
- **A language reference** generated from the compiler source, so it cannot drift.
- **Governance**: a stability contract, an RFC process, a security policy with a runbook that has
  been rehearsed rather than merely written.

## What we are NOT claiming

This section is longer than most projects'. That is deliberate.

### Performance: v1.0 is **not competitive with C**

Measured, published, and stated in those words. Across six benchmarks, the interpreter runs
**2.0× to 60.5× slower than C** (`measurements/study-c/REPORT.md`, regenerated at the release
cut — the earlier run measured 2.0×–51.1×, and the fresh, worse number is the one published). The
constitution's commitment is
about hot paths under a tiered backend with a JIT; 1.0 ships a tree-walking interpreter and a
straightforward WASM backend, and neither is that.

We are publishing the gap *before* the work rather than after, because that is the only way the next
release's numbers will mean anything.

### Typed repairs: the channel is real, the coverage is **8.3%**

Diagnostics can carry machine-applicable repairs — typed, with byte-range edits a tool can apply
without understanding the language. Measured across 48 real defects: **4 offered one**, and a repair
loop with no model in it reached a clean program on **none** of them
(`measurements/study-b/REPORT.md`).

Two of the four offered only a repair that *widens authority*, which automated tooling refuses by
policy — silencing a diagnostic by granting more authority removes the objection rather than fixing
the program. The other two fixed a warning while the error stood.

So: the mechanism works and is correctly conservative. Its coverage does not yet match what the
phrase "typed repairs" invites you to imagine.

### Conformance coverage: 287 of 287 — and how it got there is part of the number

Every reference anchor — **287 of 287** — carries both an accepting and a rejecting test, and
`release_requires_full_coverage` now fails any commit that drops below 100%. Read the how before
you weigh the number (`docs/design/STAGE9_BUILD_ORDER.md` D22): most gaps closed with tests that
*produce* their diagnostic from the real emission site; the compiler-bug-class codes (which no
program can produce by construction) closed with constructor-level witnesses, exactly as ruled;
and **three codes were retired** rather than frozen — codes that could never fire, including one
whose stated rule turned out not to exist in the checker (the probe that established there was no
soundness hole ships as two conformance fixtures). A 100% reached partly by removing dead codes
is a fact you get to weigh, which is why it is written here and not in a footnote.

### The guarantee has a boundary

- **Authority, not intent.** A dependency that was always granted `Net` and starts using it
  differently is not caught. The mechanism watches the authority boundary.
- **Foreign code is outside the proof.** A `ForeignCall` is a hole, and the authority report
  enumerates holes rather than hiding them.
- **Soundness is design-level, audit-rule, and test-enforced.** The Delulu Core mechanization is
  open work.
- **Kind is static; scope is runtime.** The type proves what *kind* of thing a function can do; the
  capability decides which file or host. We do not claim static path-level proof.

### Things we found by looking

Two defects surfaced during release preparation, both fixed, both recorded:

- **A runtime fault that crashed the host.** Deep recursion aborted the process instead of reporting
  `DL0905` — the depth guard existed but the native stack ran out first. Found by the performance
  study, which is not what a performance study is for.
- **A signature check that passed an unsigned artifact.** `verify-sig` returned exit 0 for an
  artifact with no signature. **The entire 875-test suite passed with that bypass in the tree**,
  because every signing test asserted the verdict string and none asserted the exit code. Found by
  the security drill (`docs/security/DRILL-001.md`).

We report these because a project that only publishes what makes it look good has trained you to
discount everything it publishes.

## Where it runs

| Tier | Platform | What backs the claim |
|---|---|---|
| Verified | **Windows 11** | full suite green at the release gate (926 passed / 0 failed); every stage gated here before commit |
| Verified | **Linux** | full suite green at the release gate (930 passed / 0 failed; the +4 are Linux-only platform tests incl. worker isolation) |
| Expected, unverified | **macOS** | platform-gate-free std Rust by construction — and no Apple hardware has ever run this suite, so we say "expected", not "verified" |

The microVM isolation profile is Linux-first (KVM) and says so wherever it appears. Other
operating systems are not targeted by 1.0 — and DeluluLang is **fully open source**, so a port is
a contribution, not a petition: anyone — a developer, a company, an AI — can bring a platform
through the ordinary RFC and review process, under the same authority rules as every other
contributor. The fresh-machine first run is measured, not asserted: the Book's Chapter-1 journey
completed in **1.115 s** on a pristine Windows profile and **0.019 s** on a pristine Linux home
(`measurements/first-run/RECORD.md` — including what the clock does and does not include).

## Getting started

```
delulu                       # the first run picks your language and says hello
delulu check hello.delulu
delulu authority hello.delulu
```

Chapter 1 of the Book ends with `delulu authority` on hello-world, because that is the identity in
one command. Every sample in the Book is compiled on every CI run.

- The Book: `docs/book/`
- The reference: `docs/reference/` (generated from the compiler)
- For machines: `docs/for-agents.md`
- The measurements: `measurements/`

## Verifying this release

```
delulu verify-sig delulu-1.0.0.dwx     # detached ed25519 signature
```

Artifacts are reproducible: two independent builds of the same source produce byte-identical
output. Provenance is an in-toto statement that describes the builder **honestly** — a local runner,
sub-SLSA-L3, and it says so, because a provenance statement claiming an identity it does not have is
a supply-chain lie with a schema around it.

## Thanks

To everyone who asked the awkward question early enough that it was still cheap to answer.
