# Stage 9 Build Order — "Delulu" (the v1.0 release)

**Status: BUILT** (2026-07-19). All nine phases 9a–9i committed. **v1.0 is NOT released** — the
acceptance gate returned two blockers (criteria 1 and 8); the version is `1.0.0-rc.1`. See §6.
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

**D13 — Study A's first result was a FALSE 100%; both fences are recorded, not quietly patched.**
The initial injection emitted code that did not parse. Every build failed, the campaign scored
20/20, and the number was worthless: the refusals were syntax errors the authority mechanism had
not earned (`DL0202`/`DL0401` appeared at all 20 sites — the tell). RULED: the study carries two
permanent fences, and `mechanism_holds` is false unless both pass. (a) **The validity fence** —
every mutated package must COMPILE on its own before its result is recorded; an injection that does
not compile is an experiment defect, never a catch. (b) **The negative control** — the identical
pipeline run with no mutation at all, once per chain, must build CLEAN; without it a toolchain that
refused everything would score perfectly while measuring nothing. Both the failure and the fences
are written into `measurements/METHODOLOGY.md` §1.4 rather than erased, because a measurement
program that hides its own near-miss has no standing to publish anything. The corpus also had to
change: libraries now receive `Root` and pass it down (a real-world anti-pattern the authority
report exposes), because in an object-capability language a dependency genuinely *cannot* gain a
new effect kind unless it was handed the authority to do so — which is the design working, and is
stated as a threat to validity rather than sold as a stronger result than it is.

**D14 — The mechanical-vs-manual comparison ships UNRUN, with a structural argument instead.**
Spec §3.1 asks for a column comparing the same audit performed manually on equivalent Rust/npm
graphs. Doing it properly needs multiple reviewers, blinding, and a task set this stage does not
have; publishing an uncontrolled anecdote as a comparison would be exactly the claim Constitution
§9 forbids. RULED: no human-trial numbers are reported. The methodology states the qualitative
difference as a structural argument (the check is a total function of the lockfile, runs on every
build, costs ~20 ms per graph, and does not tire) and marks the measured column UNRUN. Reopens if a
trial is funded.

**D15 — Study C found a host crash, and it was fixed rather than benchmarked around.**
`fib(24)` killed the process with a raw stack-overflow abort: no diagnostic, no usable exit code.
The interpreter's `MAX_DEPTH` guard existed but was **unreachable** — a tree-walker spends several
large native frames per DeluluLang call, and the default main-thread stack ran out first. This made
`ref.rule.runtime.faults-are-diagnostics` — one of this stage's own published normative rules —
**false**. RULED: fixed in-phase, because a runtime fault that crashes the host is the worst
failure mode a language has and shipping 1.0 with it would make the reference a liar. The CLI now
runs on a thread with a stack large enough for the depth bound to be the limit that actually fires;
deep recursion reports `DL0905`, witnessed by `unbounded_recursion_is_dl0905_not_a_host_crash` plus
its skip-branch twin (recursion *within* the bound must still work — a "fix" that refused all
recursion would pass the first test and destroy the language). Closes a D10 class-B gap; coverage
270 → 272.

**D16 — Study C refuses to measure a debug build.**
The first run measured the debug binary and produced authoritative-looking figures describing a
binary nobody runs — and the debug build's oversized frames were also what made `fib(24)` crash at
depth 22. RULED: the study errors out unless a release build is present. A performance baseline
from an unoptimised binary is worse than no baseline, because it will be quoted.

**D17 — Study B publishes 8.5% and a zero, with the decomposition.**
Only 4 of 47 real defects offer a machine-applicable repair, and **none** of those four can be
driven to green by a loop with no model: two offer only `add_effect_to_row` (authority-widening,
which the loop refuses **by policy** — widening authority to silence a diagnostic removes the
objection rather than fixing the program), and two offer a warning-level repair that cannot clear
the error beside it. RULED: published as measured, with the decomposition, because a bare zero
reads as a broken harness and the truth is more interesting than that — the typed-repair channel is
real and correctly conservative, and its *coverage* does not yet match what the phrase "typed
repairs" invites a reader to assume. The release announcement may not imply otherwise.

**D18 — The drill found a real hole in the TEST SUITE, and the finding outranks the timing.**
DRILL-001 staged a signature-verification bypass: `verify-sig` returned exit **0** for an artifact
with **no signature at all**. The runbook executed cleanly in 5m28s — and that is the less
interesting half. The important half: **the full 875-test suite passed with the bypass in the
tree.** Every existing signing test asserted on the rendered *verdict* (`unsigned`/`valid`/
`invalid`) and none on the **exit code** for the missing-signature path — the one channel every
shell script and CI job actually gates on. Root cause is the familiar shape: the suite tested the
paths where verification *does something* and skipped the branch where it has nothing to check,
which is the same class as the Stage-6 `DL0803` fail-open and the Stage-9a arity gate. RULED:
`an_unsigned_artifact_fails_verification_by_exit_code` lands permanently (asserting the exit code
*and* that the JSON envelope agrees with it), and the drill's **open** recommendation — audit
`plugin verify`, `audit verify`, `build --locked` for the same "verdict asserted, exit code not"
gap — stays recorded as NOT YET DONE rather than being quietly closed. A drill that finds a hole
and reports only its stopwatch has wasted the hole.

**D19 — The 1.0 cut is BLOCKED by its own gate, and the version says so.**
Stage 9's stated purpose is to ship v1.0. Every piece of the machinery is built and the acceptance
gate has been *run* — and it comes back with **two criteria NOT MET**: criterion 1 (conformance
coverage is 273/290, not 100%) and criterion 8 (the fresh-machine first-run has never been
performed and timed, on any OS). RULED: the version is **`1.0.0-rc.1`**, not `1.0.0`, and
`docs/release/CHECKLIST-1.0.md` states **"1.0 DOES NOT SHIP YET"** in those words with both
blockers named. Stamping 1.0.0 on a build its own checklist refuses would make every other honesty
claim in this project worthless — the announcement's whole argument is *"here is the number,
including when it is unflattering"*, and the first thing a reader would check is whether the release
met its own bar. A gate that cannot say no is not a gate. Both blockers are bounded and listed;
neither is unknown.

**D20 — The announcement's honesty review is mechanized in BOTH directions.**
`criterion10_the_announcement_makes_no_unsupported_claim` fails on a forbidden claim ("faster than
C", "lowest tokens", "provably secure", …) **and** on the *absence* of the inconvenient measured
results — the "not competitive with C" position, the 8.5% repair coverage, and the pointer to
`measurements/`. RULED both directions deliberately: the likelier failure mode for a release
announcement is not a loud lie, it is quiet omission, and a check that only bans bad sentences would
pass a document that simply left performance out.

**D21 — ATTRIBUTION ERRATUM: Stage 9 was mostly cooked by Opus 4.8; the commit trailers say Fable 5 and are WRONG.**
The honest record, corrected 2026-07-19 after Jesse flagged it: Fable 5 wrote the build order
(`93afcaf`) and opened phase 9a. The session model was then switched to **Opus 4.8** partway
through 9a (a `/model` command mid-phase), and Opus 4.8 produced the **rest of 9a, its commit
`335a875`, and all of 9b through 9i and the close-out** (`629bc3b`…`2c59af6`). The model was
switched back to Fable 5 only for the post-hoc verification pass. **Every one of those ten commits
nonetheless carries `Co-Authored-By: Claude Fable 5`, which is false for the Opus-4.8 span.**

Stage 8 handled the identical mid-stage switch correctly (8g–8h attributed to Opus 4.8); Stage 9
did not, because the head-chef persona kept signing "Fable 5" without checking which model was
actually running. That is a real honesty defect in the co-author trail — the one kind of dishonesty
this project treats as unforgivable, recorded here rather than quietly left in place.

RULED: the correction lives in this ledger, the spec's status section, and the memory files. The
commit trailers themselves are **not** rewritten: `git rebase`/history-rewrite is destructive and
was not authorized, and an errata that everyone can read beats a silently altered history that
hides that the mistake was ever made. Honest attribution going forward: **Stage 9 = Fable 5 (build
order + opening of 9a) + Opus 4.8 (bulk: 9a-tail → close-out).**

**D22 — The release gate closes criterion 1: the D10 remainder ends at 287/287, and three codes
that could never fire are RETIRED rather than frozen unreachable.**
Executed 2026-07-20 at the owner's order to finish the blockers and release. The 17 open anchors
closed in three honest ways, plus one D10 correction the work itself surfaced:

- **Class B, produced from the real emission sites (4):** `DL0701` (a manifest-exceeding main row,
  produced by `Manifest::check_main_row` — `a_main_row_exceeding_the_manifest_is_refused_with_dl0701`,
  with the in-ceiling converse asserted); `DL1204` (a future-versioned `.dwx` built with the real
  section machinery, refused by `read_and_verify` — with the current-version converse proving the
  refusal is the version gate, not a construction accident); `DL1304` (the bind-time fail-fast
  test: a declared symbol the library does not export fails the BIND, nothing foreign runs);
  `DL0905` had already closed in 9i (D15's fix carried its witness).
- **Class C, constructor-level per D10's own ruling (7):** `DL1101`, `DL1102`, `DL1206`, `DL1610`,
  `DL1701`, `DL1702` — and `DL1307`, which D10 had classed as producible but is NOT in the default
  matrix: the `python` feature is ON in every normal build, so the feature-off shim that emits it
  cannot be compiled into the suite binary. Each witness proves the fault constructs at its
  classification, the explain body meets the length bar and names the compiler-bug class where
  that is the class, and the JSON envelope carries the code (`unproducible_witnesses.rs`).
- **Retired pre-freeze (3): `DL0503`, `DL0702`, `DL0906`.** Codes are add-only from 1.0; a code
  that cannot fire either strands agents keying off it forever or makes the fix that starts
  emitting it a breaking change — pre-1.0 was the only free moment, exactly as D10 said of Class A.
  - `DL0503`: the probe on record shows the "at most one row variable per signature" rule **does
    not exist in the checker** — a benign two-row-variable signature checks clean
    (`24_multi_rowvar_benign.delulu`), and the laundering attempt is refused by row honesty
    (`DL0501_two_rowvar_row_stays_honest.delulu`): no soundness hole, but D10's Class-A framing
    ("the rule is enforced under a more general code") was wrong for this one — the *dangerous
    shapes* are fenced (DL0306/DL0501), the *arity rule* is fiction. The rule anchor is rewritten
    to what is true and enforced: `ref.rule.effects.row-bindings-never-merge` (DL0504). This
    supersedes D10's 9c note deferring the disposition to a post-1.0 RFC — post-1.0, retirement
    would be a breaking change, which made that deferral self-defeating.
  - `DL0702`: never emitted anywhere — the "reconcile grants and refuse at startup" flow was never
    wired, and what actually exists is better: deny-by-default at *derivation* (`DL0703`, both
    engines, witnessed), under which a program is never refused for a capability it never
    exercises. D10 misclassified it as Class B.
  - `DL0906`: no `panic` builtin exists in the language (§5.8); 9c's assigned disposition was
    never executed, and it executes here: retire. If a panic construct ever lands by RFC, it gets
    a new code.
- The registry keeps a tombstone comment at each retired number; **numbers are never reused.**
- **The ratchet rose 273 → 287, `release_requires_full_coverage` flipped from `#[ignore]` to a
  hard per-commit gate, permanently** — coverage below 100% is now a build failure, not a report.

RULED: this is the honest 100% — no anchor was widened, no witness is a mention, and the two
fixtures that proved the DL0503 decision are committed as conformance programs so the probe
outlives the ruling.

*(Ledger grows as phases surface new conflicts; nothing ships un-ruled.)*

## 4. Phase plan and gates

Order is 9a → 9i as in the playbook; each phase = brief → cook → verify → commit.

| Phase | Deliverable | Gate (verified by head chef before commit) |
|---|---|---|
| 9a | **DONE** — Coverage law: anchor registry (extracted from compiler source), test→anchor metadata, `delulu-conform --coverage`, the ratchet | `--coverage` fails on any zero-coverage anchor, never-produced code, dangling/ignored witness, or unknown citation; **216/233 (92.7%), 0 validation errors**; ratchet floor committed; CI wired; remainder classified in D10 |
| 9b | **DONE** — `docs/reference/` generated-in-part: 24 chapters (16 §5 semantics + tokens/grammar/primitives/diagnostics/audit-rules/CLI/coverage/index), every normative statement anchored | `--check-reference` is a HARD CI gate; drift test + its skip-branch case green; token index fenced against `TokenKind`; every rule's enforcing code proven registered; **287 anchors, 263 covered (91.6%)** |
| 9c | **DONE** — `STABILITY.md` (invariant 43), deprecation policy + registry (empty at 1.0, mechanism complete), `[package] language` edition, DL1801/DL1802; D10 class A 3/4 fixed; D11 `--help` fixed | Criterion 9 witnessed (`stability_cli.rs`, 8 tests incl. older-edition and unpinned skip branches); both codes registered with explain bodies; `keygen --help` no longer writes a key; **289 anchors, 270 covered (93.4%)** |
| 9d | **DONE** — Study A: deterministic 25-package corpus (5 archetypes × depth 4), 20-site injection campaign, validity fence + negative controls | **20/20 caught (100%)**, all 20 mutations compile, all 5 controls clean; refusals are authority codes (DL1001/DL1010 at every site) not compile errors; raw data + corpus + METHODOLOGY with threats-to-validity committed; 9 integrity tests incl. proof the scoring can express failure |
| 9e | **DONE** — Study B (repair loops over the 47-program reject corpus) + Study C (6 benchmarks × 4 lanes, release-only) | B: **8.5% repair availability, 0 reach green mechanically** — published with the decomposition. C: **2.0×–51.1× slower than C**, "not competitive" stated in those words. §5.11 caveat verbatim; token counts deliberately not published. Found and fixed the DL0905 host-crash (D15) |
| 9f | **DONE** — `SECURITY.md` (rubric + runbook), `CONTRIBUTING.md` §AI, `rfcs/` process + template, `CODEOWNERS`, drill record | **Criterion 6 witnessed: DRILL-001 executed end-to-end in 5m28s** against a staged signature bypass. The drill's real finding (D18): the **entire 875-test suite passed with the bypass planted**. Regression test added; 6 governance tests pin the artifacts, incl. that PENDING-PUBLIC items are still marked |
| 9g | **DONE** — `delulu-registry` (sparse index, publish API, scoped/revocable tokens, yank≠delete, mandatory signature, **server-side authority recomputation**), `delulu login` | **Criterion 5 witnessed**: clean-machine publish→resolve→offline-build round-trip; doctored claim refused in-process AND over HTTP; the skip branch (**cannot recompute → REFUSE**) tested; outage degrades to the cached lockfile; 18 tests, zero new deps (hand-rolled HTTP) |
| 9h | **DONE** — 8 Book samples as real checked files, 95 missing explain bodies written, `docs/for-agents.md` | **Criterion 7 witnessed both halves**: every sample compiles (CI-gated, fmt-checked) and `criterion7_every_code_has_a_long_form_explanation` pins **100%** — measured at **95 of 150 title-only** when the phase opened. Agent page carries the honest 8.5% repair number and the "not competitive with C" position |
| 9i | **DONE** — pinned toolchain, reproducibility check, SBOM, provenance, announcement + checklist, **version `1.0.0-rc.1`** | Criterion 4 **MET (local form)**: artifacts byte-identical across builds (SHA-256 equal), signatures verify, tamper refused. Criterion 10 **MET**: honesty review mechanized in both directions. **Criteria 1 and 8 NOT MET — the gate says 1.0 does not ship (D19)** |

## 5. Diagnostics budget

DL1801 (deprecated feature, RFC-linked, migration repair where mechanical), DL1802 (package
declares newer language edition — exact repair). **No other new codes** without a ruling here;
Stage 9 adds no language features by definition.

## 6. Close-out table (spec §8 — v1.0 ships when all pass)

| # | Criterion | Status | Witness |
|---|---|---|---|
| 1 | 100% anchor coverage; suites green on OSes × engines × custody × profiles | **NOT MET** | **273/290 (94.1%)** — `delulu-conform --coverage`; ratchet `coverage_never_regresses` (floor 273); remainder classified in D10. Suites green Windows + Linux, both engines, embedded + daemon |
| 2 | Audit exploit set (F-1…F-6, R-7) permanent + re-verified in release pipeline | **MET** | `delulu-check/tests/laundering.rs`; audit rules **7/7 covered** in `docs/reference/audit-rules.md`; re-run by the full suite on every phase gate |
| 3 | Studies A/B/C published, raw data; Study A injection catch 100% | **MET** | `measurements/` — A: **20/20**, all mutations valid, 5/5 negative controls clean (`study_a_integrity.rs`, 9 tests). B: 8.5% / 0-to-green, published with decomposition. C: 2.0×–51.1× slower than C, published as measured |
| 4 | Reproducible builds (D6 local form); provenance verifies (D5); Scorecard per D2 | **MET (local form)** | `criterion4_a_dwx_artifact_is_byte_identical_across_builds` (SHA-256 equal across builds); sign/verify/tamper witnessed; SLSA L3 + Scorecard `PENDING-PUBLIC` |
| 5 | Registry live (D3 local form); round-trip; doctored-line rejection | **MET (local form)** | `delulu-registry` 18 tests — `criterion5_publish_add_build_round_trip_from_a_clean_machine`, `criterion5_a_doctored_index_line_never_reaches_a_client`, `when_the_server_cannot_recompute_the_publish_is_refused`, outage degradation |
| 6 | Patch runbook rehearsed under target time, timeline recorded | **MET** | `docs/security/DRILL-001.md` — **5m28s** end to end vs a 14-day target; found a real hole in the suite (D18); `criterion6_the_patch_runbook_has_been_rehearsed` |
| 7 | Book samples 100% CI-run; explain coverage 100% en-US | **MET** | `criterion7_every_book_sample_checks_clean` (8/8, fmt-gated in CI) + `criterion7_every_code_has_a_long_form_explanation` (**100%**, from 95/150 title-only at phase open) |
| 8 | First-run < 5 min following Book only (D9 form, per-OS) | **NOT MET** | Not performed or timed on any OS. macOS additionally unverifiable here (D8) |
| 9 | DL1801/DL1802 per §2.2 on synthetic fixture | **MET** | `crates/delulu/tests/stability_cli.rs` — 8 tests incl. the older-edition and unpinned-package skip branches |
| 10 | Announcement passes line-by-line honesty review, sign-off recorded | **MET** | `criterion10_the_announcement_makes_no_unsupported_claim` (both directions, D20); sign-off in `docs/release/CHECKLIST-1.0.md` |

### Verdict

**8 of 10 met. Stage 9 is BUILT; v1.0 is NOT RELEASED.**

The stage's deliverables — the coverage law, the generated reference, the stability contract, the
three studies, governance, the registry, the docs set, and the release machinery — are all built,
tested, and committed. The acceptance gate for the *release* was then run against them, and it
returned **two blockers** (criteria 1 and 8). The version is `1.0.0-rc.1` and
`docs/release/CHECKLIST-1.0.md` says **"1.0 DOES NOT SHIP YET"** in those words. See D19: a gate
that cannot say no is not a gate.

Both blockers are bounded. Criterion 1 is 17 classified anchors. Criterion 8 is a walkthrough
nobody has sat down and timed.

### Cross-platform verification

| Platform | Result | How |
|---|---|---|
| **Windows 11** | **916 passed / 0 failed / 5 ignored** | Native, every phase gated before commit |
| **Linux (WSL Ubuntu)** | **920 passed / 0 failed / 5 ignored** | Full workspace suite at `7acc354`; the +4 are platform-gated tests that only run on Linux |
| **macOS** | **UNVERIFIED** | No Apple hardware (D8) |

The macOS position, precisely: Stage 9 is platform-gate-free standard Rust and is expected to work
by construction. The **one** change worth confirming on Apple hardware is the 512 MB interpreter
thread stack introduced in 9e — a virtual reservation, fine on 64-bit macOS in principle, but no
machine here has run it. Stated rather than assumed, as in every prior stage.

---

*The build order is the head chef's ledger. If it isn't written here, it didn't happen.*
