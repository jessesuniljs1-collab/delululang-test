# Stage 9 Playbook — "Delulu" (the v1.0 release: freeze, prove, govern, ship)

**Companion to:** `docs/design/STAGE9_SPECIFICATION.md` (normative). This file is *how to build it*.
**Depends on:** Stage 8 complete. From this stage's start the language core is **change-frozen except
through the RFC process**; Stage 9 adds **no language features by definition** (DL18xx is
stability/deprecation only).

> **The one-sentence goal:** turn a finished language into a **released, proven, governable v1.0** —
> freeze the spec as an *executable* artifact (the conformance suite), publish the measurable proof
> of the identity claim, stand up governance + security operations, open the registry, ship the docs
> set, and cut a reproducible, signed, provenance-attested 1.0.

This is the **least code-heavy, most process-and-rigor-heavy** stage. Its deliverables are corpora,
CI gates, published studies, written policies, and a reproducible release — not new compiler
features. The implementing model's job here is *discipline and completeness*, not cleverness.

---

## 0. Orientation — what "done" means differently here

Stage 9 is done when v1.0 **ships with its proof attached**. The three pillars:

1. **The conformance suite IS the specification's executable form (invariant 42).** Every grammar
   production, typing rule, audit rule (R-1…R-7), primitive-table entry, diagnostic code, and CLI
   contract has ≥1 accepting and ≥1 rejecting test, and the coverage map is machine-checked. **A
   behavior not covered by a test is not stable, and the reference says so per item.** 100% anchor
   coverage is the release's credibility claim — it goes in the announcement.
2. **The measurement program is the published proof (Studies A/B/C).** Reproducible, pinned, raw data
   + methodology + threats-to-validity. Honesty clauses (Constitution §9) bind every sentence.
3. **Governance + release engineering make it real** — stability contract, RFC process, security
   runbook (rehearsed), reproducible signed builds with SLSA provenance, registry go-live.

Read before writing: spec §2 (freeze + coverage law), §3 (the three studies), §4 (governance/security
ops), §5 (registry), §6 (docs set), §7 (release engineering), §1 invariants 42–44.

---

## 1. Where the work lands

Mostly **not** in the compiler crates. Stage 9 produces:
- **`docs/reference/`** — the language reference, *generated-in-part* (grammar from the parser's
  declarative tables, primitive table + diagnostics registry extracted from compiler source at build
  time — so docs cannot drift). One chapter per Constitution §5 subsection + grammar/tokens/prim-table/
  diagnostics/audit-rules; every normative statement carries a stable anchor id.
- **`delulu-conform --coverage`** — a mode of the conformance harness that maps tests → reference
  anchors and fails CI on any zero-coverage anchor or never-produced diagnostic code.
- **`measurements/`** — a reproducible repo (pinned toolchains, fixed seeds, raw data + scripts +
  written methodology) for Studies A/B/C.
- **Governance artifacts:** `SECURITY.md`, `CONTRIBUTING.md` (§AI policy), `rfcs/` (template + process),
  CI gates (two-person review, signed commits/tags via Sigstore gitsign, SLSA L3 provenance,
  reproducible builds, SBOM, OpenSSF Scorecard floor).
- **Registry service** (Stage-8 index format, hosted) + release engineering (two independent builders,
  byte-identical artifacts).
- **`docs/book/`** (the Book — see Phase 9-Book note), the `delulu explain` corpus (100% en-US),
  `docs/for-agents.md`.

---

## 2. Phase plan (each phase: build/write → CI-gate green → commit)

### Phase 9a — the coverage law (invariant 42, mechanized) — do this FIRST
Add reference-anchor ids to normative statements; add test metadata citing anchors; build `delulu-
conform --coverage` to map tests → anchors and **fail CI on any zero-coverage anchor or any diagnostic
code the suite never produces.** Doing this first *reveals the actual gaps* (which tests are missing)
before you write prose claiming completeness.
*Gate:* CI fails until 100% anchor coverage; the number is published in the release announcement.

### Phase 9b — the language reference (`docs/reference/`), generated-in-part
One chapter per Constitution §5 subsection + grammar/tokens/primitive-table/diagnostics/audit-rules.
Extract grammar (parser tables), primitive table, and diagnostics registry **from compiler source at
build time** — the docs cannot drift from the implementation (that drift is the classic reference-rot
failure; mechanize it away). Each normative statement gets a stable anchor id that conformance tests
cite. Wire the reference build into CI.
*Gate:* the reference builds; every anchor cited by a test exists; extracted sections match source.

### Phase 9c — stability contract + deprecation policy + editions
Write the stability contract (invariant 43): stable = surface grammar, typing/effect/authority rules,
diagnostic codes + repair ids (add-only), JSON schemas (versioned, additive), DIR major, `delulu:cap`/
broker protocol majors, CLI exit codes, manifest/lockfile formats, catalog key space. Explicitly *not*
stable: human prose, performance, trace ordering, anything experimental. Deprecations are RFC-gated →
DL1801 (warning + `fmt --migrate` rule where mechanical), live ≥2 minors, removals major-only. The
`[package] language = "1.x"` edition key (DL1802 when a package declares a newer edition than the
toolchain).
*Test (criterion 9):* DL1801/DL1802 behave per §2.2 on a synthetic deprecation fixture.

### Phase 9d — Study A (whole-program authority verification at scale)
A corpus of ≥25 real-shaped packages (CLI tools, parsers, an HTTP client stack, a static-site
generator, agent-tool servers) with real dependency graphs (depth ≥4). Deliverables: `delulu
authority` per package; **the xz-scenario injection test across the corpus — a patch release adding an
effect anywhere must be caught 100% of the time** (this is a claim of *mechanism*, so 100% is
required, not aspirational); time-to-verify numbers; a "mechanical vs. manual" comparison column
(human review of equivalent Rust/npm graphs — honest framing, not "we're smarter").
*Gate (criterion 3, part):* injection catch rate = 100%; raw data published.

### Phase 9e — Studies B + C (agent success; performance honesty baseline)
**Study B:** ≥30 fixed tasks, fixed models, N runs, same harness — DeluluLang (JSON diagnostics +
typed repairs) vs Python/Go baselines. Metrics: iterations-to-green, wall time, unauthorized-effect
attempts (DeluluLang catches at compile time — *count them*; baselines only post-hoc — count what the
harness can see and *say so*). Token counts are a **minor appendix** with the §5.11 caveat verbatim
(no "lowest tokens" claim). **Study C:** microbenchmarks + 3 macro workloads on both engines vs C
(native) and Go — **publish the actual numbers whatever they are** as the v1.0 baseline for Stage 10;
the only permitted prose claim is measured facts. If 1.0 is not yet competitive-with-C, the write-up
says exactly that.
*Gate:* studies reproduce from clean checkout under pinned toolchains/seeds; threats-to-validity
sections present.

### Phase 9f — governance + security operations
`SECURITY.md` (private disclosure security@ + PGP, 90-day coordinated default, severity rubric, the
**patch runbook**). Branch protection (two-person review), signed commits/tags (Sigstore gitsign),
SLSA L3 provenance, reproducible builds, SBOM (CycloneDX), OpenSSF Scorecard floor gate.
`CONTRIBUTING.md` §AI policy (disclosure required; named human sponsor per AI-PR; no unsupervised
autonomous PRs; unchanged quality bar; **AI-submitted code runs only in sandboxed CI using the
project's own Stage-5 isolation profiles — the project dogfoods its own containment**). AI-overseer
monitoring as advisory-only defense-in-depth (never merge authority — guarantees hold even if every
overseer colludes).
**SUPERSEDED 2026-08-03** and left as recorded: the policy is now **kind-blind** — every rule keys on
the change rather than on its author's kind, all contributed code runs sandboxed, and a human sponsor
is a preference, not a requirement. See `CONTRIBUTING.md` §4; the old form contradicted Constitution
invariant 24. The `rfcs/` template (motivation, irreducibility analysis for core changes,
entrenchment section per invariant 44, drawbacks, rejected alternatives) + ≥14-day comment period;
CODEOWNERS gates Constitution changes on the project lead.
*Test (criterion 6):* **the patch runbook rehearsal** — a staged vulnerability planted in an RC branch
goes disclosure→fix→signed emergency release→advisory under the runbook's target time, as a drill,
timeline published.

### Phase 9g — registry go-live
The Stage-8 index format, hosted: static sparse index behind a CDN + a minimal publish API (`delulu
login` scoped/revocable tokens; never grant yank on others' packages). Written policies: first-come
namespaces + squatting review; **yank ≠ delete** (yanked versions resolve only from existing
lockfiles); mandatory signature on publish; **the index authority line is recomputed server-side from
the uploaded artifact** (publishers cannot claim an authority they don't carry — the server runs
`delulu plugin verify`/package verification in a Stage-5 microVM profile, dogfooding again). Registry
outage degrades to lockfile/vendored builds (never blocks existing users' CI).
*Test (criterion 5):* publish→add→build round-trip from a clean machine; server-side authority
recomputation demonstrably rejects a doctored index line.

### Phase 9h — the docs set (Book, explain corpus, for-agents)
**The Book** (`docs/book/`) — the learn-by-building tutorial, Stage-1 demo to actors, for the two
human audiences (learners; reviewers of AI code); **Chapter 1 ends with `delulu authority` on
hello-world, because that is the identity in one command.** Every code sample is a conformance test
(compiled + run in CI — criterion 7). The **`delulu explain` corpus**: 100% of codes have long-form
en-US docs at 1.0 (closing the Stage-8 gap); top-50 slang set per Stage 8. `docs/for-agents.md`: the
machine-surface index (JSON schemas, exit codes, repair semantics, `--no-prompt`/env conventions) —
one page, stable anchors, the page agent harnesses pin.
*Note:* The full standalone Book (`THE_DELULULANG_BOOK.md`) is being drafted separately in this
planning pass; Stage 9's `docs/book/` is its CI-tested, sample-verified home. Reconcile the two when
building — the standalone draft is the content source; the CI harness is the Stage-9 deliverable.

### Phase 9i — release engineering + the 1.0 cut
**Reproducibility:** two independent builders (different OS images) produce byte-identical release
binaries and `.dwx` artifacts (pinned Rust toolchain, vendored deps, `SOURCE_DATE_EPOCH`); divergence
= release blocker. Release artifacts: per-OS binaries, SBOM, SLSA provenance, signatures, the
conformance-suite tarball (`delulu-conform --self-check`). Semver on the toolchain from 1.0; the
language edition moves only with strictly-additive minors; diagnostics/JSON add-only (invariant 43).
*Test (criterion 4, 8, 10):* reproducible-build check passes on two builders; SLSA verifies; Scorecard
≥ floor; fresh-machine first-run (picker→welcome→hello-world→`delulu authority`) under five minutes on
three OSes following the Book only; the announcement draft passes a **line-by-line honesty review**
against Constitution §9, sign-off recorded.

---

## 3. The traps

1. **Coverage law first, prose second.** Writing "100% covered" before `--coverage` proves it is how
   references lie. Build the gate, let it fail, close the gaps, *then* claim.
2. **The reference is generated-in-part or it will rot.** Grammar/prim-table/diagnostics extracted from
   source at build time. Hand-maintained reference tables drift within one release; mechanize.
3. **100% is required where it's a mechanism claim.** Study A's injection catch rate is 100% because
   whole-program authority verification is a *mechanism*, not a heuristic — anything less means the
   mechanism has a hole to fix, not a number to soften.
4. **Publish the numbers whatever they are (Study C).** If 1.0 is not competitive-with-C yet, say so;
   that honest baseline is *exactly* what Stage 10 works against. No "lowest tokens", no "faster than
   C" (Constitution honesty clauses — the honesty review, criterion 10, checks this line by line).
5. **The patch runbook must be rehearsed, not just written.** Criterion 6 is a *drill* with a
   published timeline — a security process nobody has executed is not a security process.
6. **Server-side authority recomputation is non-negotiable** for the registry: a publisher cannot
   claim an authority they don't carry. The server re-derives it in a microVM (dogfooding Stage 5).
7. **No language features.** Stage 9 adds none by definition. If a study or the Book "needs" a feature,
   that is an RFC for post-1.0, not a Stage-9 change.
8. **AI contributions run only in sandboxed CI** using the project's own isolation profiles — the
   project must dogfood its own containment for its own contributions (`CONTRIBUTING.md` §AI).

---

## 4. Definition of done (map to spec §8 acceptance criteria)

Ship v1.0 when all 10 criteria pass: 100% anchor coverage on 3 OSes × both engines × both custody
modes × all CI-hostable isolation profiles (criterion 1); the full audit exploit set (F-1…F-6, R-7)
as permanent tests re-verified in the release pipeline (criterion 2); Studies A/B/C published with raw
data and Study A's 100% injection catch (criterion 3); reproducible build on two builders + SLSA +
Scorecard (criterion 4); registry round-trip + doctored-line rejection (criterion 5); the rehearsed
patch runbook (criterion 6); the Book's samples all CI-run + 100% explain coverage (criterion 7);
five-minute fresh-machine first-run on three OSes (criterion 8); DL1801/DL1802 deprecation behavior
(criterion 9); the line-by-line honesty review sign-off (criterion 10). Add `## Implementation status`
to `STAGE9_SPECIFICATION.md`.

*Stage 9 is the promise, kept and published. Stage 10 is the language at industrial and physical
stakes.*
