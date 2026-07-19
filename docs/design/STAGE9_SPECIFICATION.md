# DeluluLang — Stage 9 Implementation Specification

**Version:** 1.0 ("Delulu"). **Status:** Committed — this stage *is* the v1.0 release.
**Depends on:** Stage 8 complete. From this stage's start, the language core is change-frozen
except through the RFC process; Stage 9 adds **no language features by definition**.
**Governing documents:** `CONSTITUTION.md` (§9, §10), all prior stage specs.

---

## 0. Scope and goal

**Goal:** turn a finished language into a **released, proven, governable v1.0**: freeze and
document the specification as an executable artifact, publish the measurable proof of the
identity claim, stand up the governance and security machinery the constitution mandates, open
the registry, and cut a reproducible, signed, provenance-attested 1.0.

**In scope (must ship):**
- **Specification freeze**: the language reference, conformance-coverage law, stability
  contract, deprecation policy, RFC process activation.
- **The measurement program** (§3): the published, reproducible evidence for the identity.
- **Governance & security operations** (§4): everything Constitution §9 lists, operationalized.
- **Registry go-live** (§5).
- **Documentation set** (§6): the Book, the reference, the explain corpus.
- Release engineering: reproducible builds, signing, SLSA provenance, the 1.0 cut (§7).
- New diagnostics: DL18xx (stability/deprecation only — by construction).

**Non-goals:** performance work, JIT policy, robotics profile, multi-threaded WASM, GC upgrade —
all Stage 10; any new syntax/semantics (RFC-only, post-1.0).

---

## 1. Invariants (carried + new)

All prior invariants hold. New:

42. **The conformance suite is the specification's executable form.** Every grammar production,
    typing rule, audit rule (R-1…R-7), primitive-table entry, diagnostic code, and CLI contract
    has at least one accepting and one rejecting conformance test, and the coverage map is
    machine-checked (§2.3). A behavior not covered by a test is not stable, and the reference
    says so per item.
43. **Stability contract.** From 1.0: stable = surface grammar, typing/effect/authority rules,
    diagnostic codes and repair ids (add-only), JSON schemas (versioned, additive), DIR major
    version, `delulu:cap`/broker protocol majors, CLI exit codes, the manifest/lockfile formats,
    the catalog key space. Explicitly *not* stable: human prose, performance, trace record
    ordering, anything labeled experimental.
44. **Entrenchment is enforced in process.** RFCs touching Constitution §1, §2, or §5.14's
    honesty limits require the §10 entrenchment analysis; the RFC template has a mandatory
    section for it; the repo's CODEOWNERS gates constitution changes on the project lead.

---

## 2. Specification freeze

### 2.1 The language reference

`docs/reference/` — one chapter per constitution §5 subsection plus grammar, tokens, primitive
table, diagnostics registry, and the audit rules. Each normative statement carries a stable
anchor id (`[ref.effects.subsumption-site]`) that conformance tests cite in their metadata. The
reference is generated-in-part: grammar (from the parser's declarative tables), primitive table,
and diagnostics registry are **extracted from the compiler source** at build time — the docs
cannot drift from the implementation.

### 2.2 Deprecation policy

Deprecations are RFC-gated, produce DL1801 (warning, with migration repair where mechanical, and
a `delulu fmt --migrate` rule), live ≥ 2 minor versions before any removal, and removals are
major-version-only. The `[package] language = "1.x"` manifest key (new, optional, defaults to
the toolchain's version) lets a package pin the language edition; a toolchain refuses a package
declaring a newer edition (DL1802).

### 2.3 Coverage law (invariant 42, mechanized)

`delulu-conform --coverage` maps tests → reference anchors and fails CI if any anchor has zero
tests or any diagnostic code is never produced by the suite. The 1.0 release requires 100%
anchor coverage — this number is in the release announcement, because it *is* the credibility
claim.

---

## 3. The measurement program (the published proof)

All three studies ship as a reproducible repo (`measurements/`, pinned toolchains, fixed seeds,
raw data + scripts + a written methodology and threats-to-validity section). Honesty clauses
(Constitution §9) bind every sentence of the write-up.

### 3.1 Study A — whole-program authority verification at scale

Build/port a corpus of ≥ 25 real-shaped packages (CLI tools, parsers, an HTTP client stack, a
static site generator, agent-tool servers) with real dependency graphs (depth ≥ 4). Deliverables:
`delulu authority` reports for each; the xz-scenario injection test across the corpus (a patch
release adding an effect anywhere in the graph must be caught 100% of the time — this is a
*claim of mechanism*, so 100% is required, not aspirational); time-to-verify numbers. Comparison
column: the same audit performed on equivalent Rust/npm graphs by a human reviewer (time taken,
findings missed) — the honest framing is "mechanical vs. manual," not "we're smarter."

### 3.2 Study B — agent task success and repair loops

Fixed task set (≥ 30 tasks: implement/modify/fix programs against tests), fixed models, N runs
each, same harness: DeluluLang (JSON diagnostics + typed repairs) vs Python and Go baselines.
Metrics: iterations-to-green, wall time, unauthorized-effect attempts (DeluluLang: caught at
compile time — count them; baselines: detectable only by post-hoc trace inspection — count what
the harness can see and say so). Report includes per-tokenizer token counts as a *minor*
appendix with the Constitution §5.11 caveat verbatim (no "lowest tokens" claim).

### 3.3 Study C — performance honesty baseline

Microbenchmarks + 3 macro workloads on both engines vs C (native) and Go: publish the actual
numbers, whatever they are, as the v1.0 baseline for Stage 10's work. The only claim permitted
in prose: measured facts. (The "competitive with C on hot paths" commitment is *assessed* here
and *worked* in Stage 10 — if 1.0 is not yet competitive, the write-up says exactly that.)

---

## 4. Governance and security operations

Operationalizing Constitution §9 — each item lands as a repo artifact + CI gate:

- `SECURITY.md`: private disclosure channel (security@ + PGP key), 90-day coordinated
  disclosure default, severity rubric, the **patch runbook** (rehearsed: §8 criterion 6).
- **Two-person review** enforced by branch protection; **signed commits/tags** (Sigstore
  gitsign); **SLSA L3 provenance** on all release artifacts; **reproducible builds** (§7);
  SBOM (CycloneDX) per release; OpenSSF Scorecard in CI with a floor score gate.
- **AI-contribution policy** (`CONTRIBUTING.md` §AI): disclosure required; named human sponsor
  accountable per AI-authored PR; no unsupervised autonomous PRs; unchanged quality bar;
  AI-submitted code runs only in sandboxed CI (Stage-5 isolation profiles — the project dogfoods
  its own containment for its own contributions).
- **AI-overseer monitoring** as defense-in-depth on incoming PRs (advisory labels, never
  merge authority — Constitution §9: guarantees hold even if every overseer colludes).
- **RFC process** (`rfcs/`): template (motivation, irreducibility analysis for core changes,
  entrenchment section per invariant 44, drawbacks, rejected alternatives), public comment
  period ≥ 14 days, disposition recorded. The constitution's Appendix A grows only via merged
  RFCs from here on.

## 5. Registry go-live

The Stage-8 index format, hosted: static sparse index behind a CDN + a minimal publish API
(token auth via `delulu login`; tokens are scoped, revocable, and never grant yank on others'
packages). Policies shipped in writing: first-come namespaces with a squatting-review process;
**yank ≠ delete** (yanked versions resolve only from existing lockfiles); mandatory signature on
publish; the index's authority line is recomputed server-side from the uploaded artifact
(publishers cannot claim an authority they don't carry — server runs `delulu plugin
verify`/package verification in a Stage-5 microVM profile, dogfooding again). Registry outage
degrades to lockfile/vendored builds (never blocks existing users' CI).

## 6. Documentation set

- **The Delulu Book** (`docs/book/`): the learn-by-building tutorial — Stage-1 demo to actors —
  optimized for the two real human audiences (learners; reviewers of AI code). Chapter 1 ends
  with `delulu authority` on hello-world, because that is the identity in one command.
- **Reference** (§2.1). **`delulu explain` corpus**: 100% of codes have long-form en-US docs at
  1.0 (closing the Stage-8 coverage gap); the top-50 slang set ships per Stage 8.
- `docs/for-agents.md`: the machine-surface index (JSON schemas, exit codes, repair semantics,
  `--no-prompt`/env conventions) — one page, stable anchors, the page agent harnesses pin.

## 7. Release engineering and the 1.0 cut

- **Reproducibility:** two independent builders (different OS images) produce byte-identical
  release binaries and `.dwx` artifacts (pinned Rust toolchain, vendored deps, `SOURCE_DATE_EPOCH`);
  divergence is a release blocker.
- Release artifacts: per-OS binaries, SBOM, SLSA provenance, signatures, the conformance-suite
  tarball (users can verify their toolchain: `delulu-conform --self-check`).
- Versioning from 1.0: semver on the toolchain; the language edition (§2.2) moves only with
  minors that are strictly additive; diagnostics/JSON schemas add-only per invariant 43.

## 8. Acceptance criteria (v1.0 ships when all pass)

1. 100% conformance anchor coverage (§2.3), all suites green on three OSes, both engines,
   embedded + daemon custody, all isolation profiles CI can host.
2. The full audit exploit set (F-1…F-6, R-7) present as permanent rejection/containment tests —
   re-verified in the release pipeline itself.
3. Measurement studies A/B/C published with raw data; Study A's injection catch rate is 100%;
   every prose claim in the release announcement traces to a measured number or is absent.
4. Reproducible-build check passes on two independent builders; SLSA provenance verifies;
   Scorecard ≥ the declared floor.
5. Registry live; publish→add→build round-trip works from a clean machine; server-side authority
   recomputation demonstrably rejects a doctored index line.
6. **Patch runbook rehearsal:** a staged vulnerability (planted in a release-candidate branch)
   goes disclosure→fix→signed emergency release→advisory in under the runbook's target time,
   as a drill, with the timeline published internally.
7. The Book builds and its every code sample is compiled+run in CI (samples are conformance
   tests); `delulu explain` coverage is 100% en-US.
8. Fresh-machine first-run experience validated on all three OSes: picker → welcome (byte-exact)
   → hello-world → `delulu authority` in under five minutes by a tester following the Book only.
9. DL1801/DL1802 behave per §2.2 on a synthetic deprecation fixture.
10. The v1.0 announcement draft passes an honesty review against Constitution §9's clauses —
    checked line-by-line, sign-off recorded.

## 9. Diagnostics (fresh range DL18xx)

| Code | Meaning | Repair |
|---|---|---|
| DL1801 | use of deprecated feature (RFC-linked) | migration repair where mechanical |
| DL1802 | package declares a newer language edition than the toolchain | upgrade toolchain — exact |

## 10. Honesty and threat-model caveats

- v1.0's soundness claims remain design-level + audit-rule + test-enforced; Delulu Core
  mechanization is still open work and the release notes say so.
- The measurement program measures what it measures: fixed task sets and corpora, stated models,
  threats-to-validity included. No generalization beyond the data appears in project prose.
- Governance reduces, never eliminates, supply-chain and contributor risk; the structural
  defense (§5.6 semantics) remains the strongest layer.
- The registry is a trusted service for *distribution*; verification remains client-side and
  local (trust-on-first-verify, Stage-8 caveat carried).

*Stage 9 is the promise, kept and published. Stage 10 is the language at industrial and physical
stakes.*

---

## Implementation status

**Stage 9 is BUILT** (2026-07-19). **v1.0 is NOT RELEASED** — the acceptance gate returned two
blockers. Operational record, rulings D1–D21, and the criteria table: `STAGE9_BUILD_ORDER.md`.

**Attribution (corrected — see ruling D21).** Stage 9 was **mostly cooked by Opus 4.8**, not
Fable 5. Fable 5 wrote the build order (`93afcaf`) and opened phase 9a; the session model switched
to Opus 4.8 partway through 9a, and Opus 4.8 produced the rest of 9a through the close-out. The ten
Stage-9 commit trailers all say `Co-Authored-By: Claude Fable 5`, which is **false** for the
Opus-4.8 span; the trailers are left as-is (history is not rewritten) and this note is the errata.

| Phase | Commit | What landed |
|---|---|---|
| build order | `93afcaf` | Kitchen protocol, gates, rulings D1–D9 |
| 9a | `335a875` | The coverage law, mechanized. Found and fixed a **primitive-table arity fail-open** (`root.console(1,2,3,4,5)` minted a capability and ignored the surplus) and two permissive `map` branches |
| 9b | `629bc3b` | The reference, **generated in part** from compiler source; `--check-reference` a hard CI gate; 54 normative rules with derived coverage |
| 9c | `9edde1e` | `STABILITY.md`, deprecation policy, language editions (DL1801/DL1802); 3 shadowed codes made reachable; `keygen --help` no longer writes a key |
| 9d | `2f08b44` | Study A — **20/20 injections caught**, after catching a **false 100%** in the study's own first run |
| 9e | `0cb09e5` | Studies B and C published as measured; found and fixed a **host crash on deep recursion** (DL0905) |
| 9f | `f118396` | Governance + `DRILL-001`: the runbook rehearsed in 5m28s, and it proved the **875-test suite was blind to a signature bypass** |
| 9g | `9589e5d` | The registry: server-side authority recomputation, scoped tokens, yank≠delete |
| 9h | `20b7245` | Book samples as conformance tests; **95 missing explain bodies** written; `docs/for-agents.md` |
| 9i | `7acc354` | Release engineering; reproducibility witnessed; **the gate said no** (D19) |

### The two blockers

1. **Criterion 1** — conformance coverage is **273/290 (94.1%)**. The 17 open anchors are
   classified in the build order's D10: shadowed codes, producible-but-untested, and codes no
   program can produce by construction.
2. **Criterion 8** — the fresh-machine first-run has not been performed or timed on any OS.

### Verification

Windows **916 passed / 0 failed / 5 ignored**. Linux (WSL Ubuntu) **920 passed / 0 failed /
5 ignored** at `7acc354` — the +4 are platform-gated tests that only run on Linux. macOS
**unverified**: no Apple hardware (D8). The one Stage-9 change worth confirming there is the 512 MB
interpreter thread stack introduced in 9e.

### Honest note on §3

Study B's live-model lane and the mechanical-vs-manual comparison in Study A are **UNRUN**, not
dropped (D4, D14). The Go baselines in Studies B and C are UNRUN — no Go toolchain on the
measurement machine. Each is labelled in its report rather than omitted so the remainder looks
complete.
