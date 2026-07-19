# Stage 9 Build Order — "Delulu" (the v1.0 release)

**Status:** COOKING (opened 2026-07-19).
**Governing documents (precedence):** `STAGE9_SPECIFICATION.md` (normative) >
`docs/playbooks/STAGE9_PLAYBOOK.md` (method) > this build order (operational rulings).
**Depends on:** Stages 1–8 BUILT (Stage 8 closed at `aac2f33`, Windows 800/0, Linux 807/0/4).
From this stage's start the language core is change-frozen except through the RFC process;
Stage 9 adds **no language features** (DL18xx is stability/deprecation only).

This document is the operational ground truth for Stage 9: the phase gates, the house rules,
every ruled deviation, and the close-out table. A deviation not ruled here does not ship.

---

## 1. Kitchen protocol for this stage

Stage 9 is cooked by per-phase implementing agents ("sous-chefs") under a single verifying
lead ("head chef"). Division of labor, absolute:

- **Sous-chef** (one at a time, strictly sequential): implements exactly one phase from a
  complete written brief. Cooks in the main working tree. **Never commits, never pushes,
  never edits this build order.** Runs the tests it writes; reports honestly, including
  failures and open questions.
- **Head chef**: writes the briefs, rules on every deviation, personally verifies each phase
  (full workspace suite, skip-branch review per house rule 3, restricted-word scrub, diff
  read), and is the only one who commits. Commit at green, per phase.
- Escalation: a sous-chef that hits the same error twice, or needs a decision not in its
  brief, stops and reports rather than improvising.

## 2. House rules (carried from Stages 5–8, absolute)

1. **Never push to any remote.** Local commits only, per phase, at green.
2. **Restricted-word scrub** before every commit (`git grep -i` for the word ruled out of all
   product surfaces in Stage 5's close-out; the scrub list lives outside the repo).
3. **The skip-branch rule:** for every enforcement rule, the "what if the checker couldn't
   tell" case is a named test written BEFORE the rule is claimed to hold. Security rules die
   in the `else { continue }` branch.
4. **Machine channels are law:** `--json` output, exit codes, DIR, schemas stay byte-identical
   for programs Stage 9 doesn't touch. Stage 9 being docs-and-process-heavy makes this easy
   to honor and easy to forget — CI's byte-identity gates stay on.
5. **Deviations ledger (§3):** any departure from spec/playbook is argued, numbered, and ruled
   here before it ships.
6. **Docs move with code.** Spec §12-style status log, this close-out table, user-facing
   guides with honesty caveats verbatim (meta-tested where prior stages did so).
7. **Dependency austerity.** Any new external dependency needs a numbered ruling. Default: no.
8. **Honesty clauses bind prose.** Every claim in any Stage 9 write-up (studies, announcement,
   reference) traces to a measured number or a named test, or it is absent (Constitution §9).

## 3. Ruled deviations

Numbered; each states the spec's demand, the conflict, the ruling, and what would reopen it.

**D1 — Sequential sous-chefs cook in the main tree (worktree waiver).**
Standing sous-chef discipline prescribes worktree isolation. That rule exists to prevent
concurrent mutation of one tree. Stage 9 phases are strictly sequential and dependent; a
worktree per phase costs a full cold cargo build (~10 min) and duplicate disk per phase with
zero concurrency benefit. RULED: sous-chefs cook sequentially in the main tree; the head
chef's verify-before-commit is the safety net. Reopens if any two agents ever run concurrently
— then worktrees are mandatory again.

**D2 — Public-hosting gates land as policy + config, marked PENDING-PUBLIC.**
Spec §4 mandates branch protection, two-person review, signed commits/tags (Sigstore gitsign),
SLSA L3 provenance, OpenSSF Scorecard floor — all of which require a public hosted repository
and its CI identity infrastructure. The standing order for this repository is: no remote push.
RULED: each such item ships as (a) written policy in-repo, (b) committed CI/workflow
configuration that activates on public hosting, and (c) an explicit `PENDING-PUBLIC` marker in
the release checklist. Nothing is claimed live that is not. Cook/verifier separation is
enforced procedurally this stage (sous-chef ≠ committer); organizational two-person review
activates with public hosting. Reopens the moment the repository goes public.

**D3 — Registry go-live is a locally-hosted go-live.**
Spec §5 says "static sparse index behind a CDN + a minimal publish API," hosted. The testable
substance — index format, publish semantics, scoped/revocable tokens, yank ≠ delete, mandatory
signature, **server-side authority recomputation**, outage degradation, clean-machine
round-trip, doctored-line rejection — is fully exercisable against a locally-run registry
service (localhost HTTP over the Stage-8 index format). RULED: Stage 9 ships the registry as a
runnable local service with every policy implemented and witnessed; putting it behind a CDN is
a deployment act at public launch, not a code artifact. Criterion 5 is witnessed against the
local service. The server-side verification runs under the strongest Stage-5 isolation profile
the host OS can run (microVM where available; the ruled Windows fallback otherwise, stated).

**D4 — Study B ships two lanes; only the reproducible lane's numbers are published as results.**
Spec §3.2 fixes external AI models as subjects. The test/CI environment cannot call external
model APIs (network-forbidden tests, cost, and non-reproducibility of remote model versions).
RULED: Study B ships (a) the complete harness, (b) a **deterministic scripted-agent lane**
(fixed seeds, replayable transcripts) that exercises the full harness end-to-end and produces
the published comparison data between the DeluluLang loop and the Python/Go baseline loops,
and (c) a **live-model lane**, documented and runnable when API keys are supplied, labeled
UNRUN in the write-up until someone runs it. Every published sentence names its lane. The
identity claim measured in lane (b) is about the *diagnostic/repair loop mechanics*, and the
write-up says exactly that. Reopens when live-model runs are funded and executed.

**D5 — Signing and provenance dogfood the project's own machinery.**
Spec §4/§7 name Sigstore gitsign and SLSA L3. Both require OIDC identity and public
transparency infrastructure. RULED: release artifacts are signed with the project's own
ed25519 detached-signature machinery (Stage 8 — dogfood), verified by `delulu verify-sig`; a
provenance statement in SLSA/in-toto format is generated locally with the builder identity
stated honestly as the local runner (sub-L3 and marked as such); Sigstore/SLSA-L3 attestation
configs ship per D2 as PENDING-PUBLIC. Reopens with public CI.

**D6 — Two independent builders, local form.**
Spec §7 requires two independent builders on different OS images producing byte-identical
artifacts. Available hardware: one Windows machine + WSL. RULED: (a) Linux release binaries
are built twice in two fresh, independent clones under pinned toolchain + `SOURCE_DATE_EPOCH`
and must be byte-identical; (b) platform-independent artifacts (`.dwx`, the conformance
tarball) are built on **both Windows and Linux** and must be byte-identical **cross-OS** — a
strictly stronger check than same-OS twice; (c) per-OS native binaries get the two-clone check
on the OS that can host it. Divergence anywhere is a release blocker, as specced. macOS: D8.

**D7 — Security contact is PENDING-PUBLIC; the runbook and drill are real now.**
Spec §4 requires `security@` + PGP. No project domain or mail infrastructure exists
pre-launch. RULED: `SECURITY.md` ships the complete disclosure process, severity rubric, and
patch runbook with the contact channel marked `PENDING-PUBLIC` (assigned at public launch, PGP
key generated with it). The runbook **rehearsal (criterion 6) runs for real, locally**: staged
vulnerability in an RC branch, disclosure→fix→signed emergency release→advisory, timed against
the runbook's target, timeline recorded in-repo.

**D8 — macOS position carried.** No Apple hardware. Same ruling as Stages 5–8: state the
per-criterion macOS expectation honestly (platform-gate-free by construction where true),
verify on Windows + Linux, never claim a green that didn't run. "Three OSes" criteria (1, 8)
are witnessed on two OSes with the macOS gap stated in the close-out table.

**D9 — "Fresh machine" first-run, local form (criterion 8).**
No second physical machine per OS. RULED: fresh-machine = pristine environment on existing
hardware — new user profile / cleared `DELULU_*` env + empty config dir on Windows; a fresh
WSL user home on Linux — following the Book only, timed. Stated in the close-out table.

**D10 — Coverage reaches 100% at the release gate, not at phase 9a; the remainder is classified,
not hidden.**
The playbook orders the coverage law FIRST precisely so the real gaps are revealed before any
prose claims completeness. Built and run, it reports **216 / 233 anchors (92.7%)** with zero
validation errors: grammar 26/26, primitives 53/53, audit rules 7/7, CLI 24/24, diagnostics
106/123. Closing the last 17 is not one task — each has a different cause. RULED: phase 9a ships
the mechanized law, the ratchet (`coverage_never_regresses`, floor 216, may only rise), and the
classification below; **release criterion 1 is what forces 100%**, and `release_requires_full_
coverage` is the `#[ignore]`d test that flips. CI reports coverage without failing the build until
the 1.0 cut, when the step becomes a hard gate. The 17 open anchors, all "diagnostic code never
produced by the suite":

> **9c update:** Class A is **3 of 4 resolved.** `DL0203`, `DL0408` and `DL0601` now fire at their
> own sites and carry conformance reject programs. `DL0503` remains: a signature with two row
> variables is currently caught by `DL0501` (the effect it lets through) rather than by the
> arity-of-row-variables rule itself, and separating them means changing where row variables are
> collected — a change to inference, which is exactly what Stage 9 must not do. RULED: `DL0503`
> stays open and is listed in the reference as not-stable; the disposition (emit it, or retire the
> code) belongs to the first post-1.0 RFC that touches row inference. Coverage 263 → 270 of 289.

- **Class A — shadowed codes (4): `DL0203`, `DL0408`, `DL0503`, `DL0601`.** The *rule* is enforced,
  but under a more general code, so the specific one is unreachable. Verified by probe: `if 42 {}`
  is refused as `DL0401` carrying the message "`if` condition must be Bool" while `DL0408` exists
  for exactly that; forging a capability is refused as `DL0301`/`DL0401`, never `DL0601`. **No
  soundness hole — every rule holds.** This is a *registry* defect, and it matters because 9c
  freezes diagnostic codes as stable (invariant 43): freezing an unreachable code either strands
  agents keying off it, or makes a later fix that starts emitting it a breaking change. RULED:
  pre-1.0 is the only moment this is free to fix; **phase 9c owns emitting the specific code at
  each shadowed site**, as a refinement of an error path that changes no accepting program.
- **Class B — producible, untested (6): `DL0701`, `DL0702`, `DL0905`, `DL1204`, `DL1304`,
  `DL1307`.** Real behavior with a real emission site and no test yet. Ordinary work; each needs a
  fixture (a manifest-exceeding package, a refused grant, deep recursion, a future-versioned
  artifact, a missing foreign symbol, a Python-less build). RULED: closed as their owning phases
  build the fixtures anyway (9c, 9g, 9i).
- **Class C — unproducible by construction (7): `DL0906`, `DL1101`, `DL1102`, `DL1206`, `DL1610`,
  `DL1701`, `DL1702`.** No program can produce these: they are the compiler-bug class (`DL1101`
  trace ⊄ row, `DL1206` engine-parity self-check, `DL1610` race-checker, `DL1702` formatter-law),
  the fuzz harness's own code (`DL1102`), a tooling-configuration error (`DL1701`), and `DL0906`
  "explicit panic" — for which **no `panic` builtin exists in the language**. RULED: a code that
  cannot fire is not thereby exempt; the honest witness is a *constructor-level* test (the fault
  is built and classified, its explain body renders) rather than a producing program. Phase 9b
  records per-code in the reference which of these is which — invariant 42's own words: "a
  behavior not covered by a test is not stable, and the reference says so per item." `DL0906`
  additionally gets a disposition in 9c: emit it or retire it, but do not freeze a code for a
  feature that does not exist.

**D11 — CLI argument handling is frozen as-is in Stage 9; two warts are recorded, not silently
changed.** The coverage sweep found that `--help` is not honored per-subcommand: `delulu keygen
--help` **generates a key**, `delulu test --help` runs the tests, `delulu repl --help` starts the
REPL. An unrecognized flag causing a side effect is a genuine DX/safety wart. RULED: not fixed
inside 9a (it is a CLI-surface change made mid-phase, and house rule 4 protects machine channels);
**phase 9c owns the disposition**, because 9c freezes the CLI contract and a wart frozen at 1.0 is
a wart forever. Recorded here so it cannot be lost. (`keygen` is otherwise correct: it refuses to
overwrite an existing private key — witnessed.)

**D12 — The §5 normative statements live in code, and rule coverage is derived, not asserted.**
Spec §2.1 wants each normative statement to carry a stable anchor that conformance tests cite. A
hand-written chapter drifts from the compiler within a release, so the 54 rule statements live in
`delulu_conform::rules::RULES` and the §5 chapters are *generated* from them. Each rule names the
diagnostics that enforce it, and a drift guard proves every cited code is actually registered — a
rule citing a code that does not exist fails the build rather than reading as enforcement. RULED:
a rule is covered **only when every one of its enforcing codes is covered in both directions**.
That strictness is deliberate and it bites: 7 of 54 rules are uncovered *because* they lean on a
code from D10's remainder (`ref.rule.authority.no-forgery` is open exactly because `DL0601` is
shadowed). Rounding those up to "covered" would have made the reference lie about the very thing
it exists to report. Registering rule anchors moved the totals from 233 to 287 anchors; the ratchet
floor rose 216 → 263 accordingly.

*(Ledger grows as phases surface new conflicts; nothing ships un-ruled.)*

## 4. Phase plan and gates

Order is 9a → 9i as in the playbook; each phase = brief → cook → verify → commit.

| Phase | Deliverable | Gate (verified by head chef before commit) |
|---|---|---|
| 9a | **DONE** — Coverage law: anchor registry (extracted from compiler source), test→anchor metadata, `delulu-conform --coverage`, the ratchet | `--coverage` fails on any zero-coverage anchor, never-produced code, dangling/ignored witness, or unknown citation; **216/233 (92.7%), 0 validation errors**; ratchet floor committed; CI wired; remainder classified in D10 |
| 9b | **DONE** — `docs/reference/` generated-in-part: 24 chapters (16 §5 semantics + tokens/grammar/primitives/diagnostics/audit-rules/CLI/coverage/index), every normative statement anchored | `--check-reference` is a HARD CI gate; drift test + its skip-branch case green; token index fenced against `TokenKind`; every rule's enforcing code proven registered; **287 anchors, 263 covered (91.6%)** |
| 9c | **DONE** — `STABILITY.md` (invariant 43), deprecation policy + registry (empty at 1.0, mechanism complete), `[package] language` edition, DL1801/DL1802; D10 class A 3/4 fixed; D11 `--help` fixed | Criterion 9 witnessed (`stability_cli.rs`, 8 tests incl. older-edition and unpinned skip branches); both codes registered with explain bodies; `keygen --help` no longer writes a key; **289 anchors, 270 covered (93.4%)** |
| 9d | Study A: `measurements/` corpus ≥25 packages, depth ≥4; authority reports; injection campaign | Injection catch = 100% (mechanism claim); raw data + methodology + threats-to-validity in-repo; reproducible from clean checkout |
| 9e | Study B (two lanes per D4) + Study C (micro + 3 macro, both engines, vs C and Go) | Reproduce from clean checkout, pinned seeds/toolchains; numbers published as measured; §5.11 caveat verbatim in token appendix |
| 9f | SECURITY.md, CONTRIBUTING.md §AI, rfcs/ process, CODEOWNERS, CI gate configs (D2) | Criterion 6 drill executed and timed, timeline recorded; skip-branch tests for every enforcement gate that has a checker |
| 9g | Registry local go-live (D3): publish API, tokens, yank, server-side authority recomputation | Criterion 5: clean-machine publish→add→build round-trip; doctored index line demonstrably rejected (skip-branch: unverifiable artifact → refuse, not accept) |
| 9h | Book (`docs/book/`) with every sample CI-compiled+run; `delulu explain` 100% en-US; `docs/for-agents.md` | Criterion 7 witnessed (samples are conformance tests); explain coverage meta-test at 100% |
| 9i | Release engineering: reproducible builds (D6), signatures (D5), SBOM, provenance, conformance tarball + `--self-check`, v1.0.0, announcement | Criteria 4 (local form), 8 (D9), 10 (line-by-line honesty review recorded in-repo) |

## 5. Diagnostics budget

DL1801 (deprecated feature, RFC-linked, migration repair where mechanical), DL1802 (package
declares newer language edition — exact repair). **No other new codes** without a ruling here;
Stage 9 adds no language features by definition.

## 6. Close-out table (spec §8 — v1.0 ships when all pass)

| # | Criterion | Status | Witness |
|---|---|---|---|
| 1 | 100% anchor coverage; suites green on OSes × engines × custody × profiles | PENDING | — |
| 2 | Audit exploit set (F-1…F-6, R-7) permanent + re-verified in release pipeline | PENDING | — |
| 3 | Studies A/B/C published, raw data; Study A injection catch 100% | PENDING | — |
| 4 | Reproducible builds (D6 local form); provenance verifies (D5); Scorecard per D2 | PENDING | — |
| 5 | Registry live (D3 local form); round-trip; doctored-line rejection | PENDING | — |
| 6 | Patch runbook rehearsed under target time, timeline recorded | PENDING | — |
| 7 | Book samples 100% CI-run; explain coverage 100% en-US | PENDING | — |
| 8 | First-run < 5 min following Book only (D9 form, per-OS) | PENDING | — |
| 9 | DL1801/DL1802 per §2.2 on synthetic fixture | PENDING | — |
| 10 | Announcement passes line-by-line honesty review, sign-off recorded | PENDING | — |

---

*The build order is the head chef's ledger. If it isn't written here, it didn't happen.*
