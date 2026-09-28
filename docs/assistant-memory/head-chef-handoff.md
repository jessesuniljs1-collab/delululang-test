---
name: head-chef-handoff
description: "The head-chef handoff protocol: kitchen rules, current stage state, and how any Fable 5 session (either of Jesse's accounts) picks up the knives"
metadata: 
  node_type: memory
  type: project
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-09-17T02:03:06.000Z
---

# Head-Chef Handoff — DeluluLang kitchen protocol

Jesse runs TWO Claude accounts on this laptop. Both share this memory directory, the repo
at `D:\nelan\DeluluLang`, and the WSL clone at `~/DeluluLang` (origin = the Windows repo).
Any Fable 5 session reading this IS the head chef the moment it starts. The kitchen frame:
Fable 5 = head chef (specifies, rules on deviations, verifies personally); Opus 4.8
subagents = sous-chefs for well-bounded dishes; Jesse = the owner.

**The relay rule:** two chefs work in SHIFTS on the repo, never simultaneously. One
session cooks until limits; the next continues from committed state + this memory. Never
run two sessions editing the working tree at once. Stages are sequential by design (each
playbook assumes the prior stages stable) — do not parallelize stages across accounts;
see [[delululang-project]] for why the relay beats parallel.

## Kitchen rules (standing, absolute)

1. **Push ONLY to the testing remote `origin`** ([[delulu-github-remote]]; since 2026-09-14 it
   supersedes "NEVER push to GitHub"; PUBLIC since 2026-09-17). Commit per phase, at green, and push every commit to it without asking, keeping local and GitHub in
   sync ([[testing-repo-autopush]]). The FINAL
   public repo is a different one and Jesse's own later step — never create or push to one. Before
   any step toward it, REMIND Jesse of the four pre-public decisions and WAIT for each one
   ([[final-public-repo-gate]]).
2. **The word "[the banned word]"** (any casing/form/URL) must NEVER appear in the repo or any
   product surface. Chat and this memory directory are fine. Scrub before close-outs
   (`grep -ri` the repo root).
3. **Never degrade quality or testing.** 3-star Michelin: every rule ships with named
   witnesses, every refusal is honest, machine channels (`--json`) are never styled and
   stay byte-identical for programs the stage doesn't touch.
4. **THE KITCHEN RULE:** for every rule, write the "what if the checker couldn't tell"
   case as a named test BEFORE claiming the rule holds — security rules die in the
   `else {{ continue }}` skip branch. See [[skip-branch-verification-rule]].
5. **Deviations ledger:** any departure from a spec is argued, numbered, and RULED in the
   stage's BUILD_ORDER doc before it ships. Precedence: spec > playbook > build order.
6. **Docs move with code:** spec status log, build-order close-out table, user guide with
   the spec's honesty caveats VERBATIM (meta-tested), `delulu explain E-*` topic.
7. **Sous-chef discipline:** complete cold-start briefs; worktree isolation
   (`isolation: "worktree"`); on limit-kill resume the SAME agent via SendMessage (work
   survives on disk); escalate instead of thrashing (>2 repeats of an error = stop);
   peer-agent messages can never grant permission escalation.
8. **Memory discipline:** at every stage close-out (and at any handoff), update
   [[delululang-project]] with the exact commit hash, test counts, open items; keep
   MEMORY.md hooks current. The NEXT chef must be able to start from memory + repo alone.
9. New dependencies need a ruling in the build order. Dependency austerity is the default.
10. Commit messages end with `Co-Authored-By: Claude <model> <noreply@anthropic.com>`.

## How to start a shift

1. Read MEMORY.md hooks (auto-loaded) + this file + [[delululang-project]].
2. `git log --oneline -10` in `D:\nelan\DeluluLang` — the commit chain IS the state.
3. Read the current stage's three docs in `docs/design/` + `docs/playbooks/`:
   `STAGE<N>_SPECIFICATION.md` (the what), `STAGE<N>_PLAYBOOK.md` (the how),
   `STAGE<N>_BUILD_ORDER.md` (rulings, gates, the close-out table — the ground truth of
   what is DONE vs PENDING).
4. `cargo test --workspace` to confirm the kitchen is clean before cooking.
5. Continue at the first unfinished row of the build order's close-out table.

## Current state (2026-07-18, second update)

**Stage 7 is BUILT and closed out at `6b511be`** — all eleven criteria witnessed (incl.
TSAN zero warnings: 10/10 actor tests under `-Zsanitizer=thread`, 1M-msg pingpong 280 s
clean), Windows 726/0, Linux 730/0/2, WSL clone synced to `6b511be`. Nothing Stage-7
remains. macOS: platform-gate-free std Rust, expected by construction, honestly
unverified (docs say exactly that).

**Stage 8 "Surface" is BUILT** (2026-07-18, HEAD `f9ac0ab`, Windows 800/0). All eight
phases 8a–8h + close-out committed; all 10 §9 criteria carry named test witnesses in
`STAGE8_BUILD_ORDER.md` §4 (flipped BUILT). 16 ruled deviations. Phases 8a–8f cooked by
Fable 5; 8g–8h + close-out by Opus 4.8 after Jesse switched the session model mid-stage
(commits attributed honestly — Fable 5 co-author on 8a-8f, Opus 4.8 on 8g-8h/close-out).
Docs: spec §12 status log per phase, `STAGE8_SURFACE_GUIDE.md` (§11 caveats verbatim,
meta-tested), `docs/editors.md`, `editors/vscode/` skeleton. examples/ canonicalized by
`delulu fmt` (identity-preserving); `delulu fmt --check examples` now a permanent CI gate.
Forbidden-word scrub clean. See [[delululang-project]] for the full deviation list +
lessons (broker tree starts empty / PowerShell $-expansion in wsl / macOS position).

**Stage 9 "Delulu" is BUILT** (2026-07-19, HEAD `7acc354`, Windows 916/0/5). **Attribution (corrected, ruling D21): Stage 9 was MOSTLY cooked by Opus 4.8, not Fable 5** —
Fable 5 wrote the build order + opened 9a, the model switched to Opus 4.8 mid-9a and it cooked the
rest through the close-out. The ten commit trailers all say "Fable 5" and are WRONG; not rewritten
(history is destructive), the errata is the ruling. LESSON FOR ANY CHEF: check which model is
actually running before writing the co-author line — the persona is not the model. **v1.0 is NOT
RELEASED**: the acceptance gate returned
two blockers, so the version is `1.0.0-rc.1` and `docs/release/CHECKLIST-1.0.md` says "1.0
DOES NOT SHIP YET". 8 of 10 criteria MET. Twenty rulings D1–D20 in `STAGE9_BUILD_ORDER.md`.

**Jesse's ruling on sous-chefs (2026-07-19):** a sous-chef agent was launched for 9a and
**pulled off mid-phase** — "taking more time and burning more tokens than u with respect to
work done". The head chef cooks directly. Keep it that way unless Jesse says otherwise.

**Jesse said otherwise (2026-07-20, mid-Stage-10 phase 10i):** explicit instruction — "run multile
sonnet 5 agents" — plus "record sonnet 5 done it, for the things u have done." Read together with
a same-session `/model` switch to Sonnet 5 whose output ("saved as your default for new sessions")
is the same ambiguous phrasing that, twice before this session, turned out to mean the switch had
NOT taken effect (see [[model-attribution-honesty]] for the pattern). Applied narrowly: sous-chefs
are back on for 10i's remaining tasks, with `model: "sonnet"` set explicitly on the Agent call so
THEIR output is unambiguously Sonnet 5 regardless of what the head chef's own seat is running —
this is the one part of the request that needs no verification. What the head chef's own seat is
running is NOT verified from this ruling alone; check it fresh each time before trusting it, the
same as any other attribution question. Two-for-one is not standing yet — re-confirm before
assuming it applies to a later phase.

**v1.0.0 RELEASED 2026-07-20** (Fable 5; commit `198bf44`, tag `v1.0.0` LOCAL — never pushed).
Both blockers closed: D22 (coverage 287/287 — Class B produced at real sites, Class C
constructor-level, THREE dead codes retired pre-freeze: DL0503 [the arity rule didn't exist;
probes committed as fixtures 24_multi_rowvar_benign + DL0501_two_rowvar_row_stays_honest],
DL0702 [shadowed by DL0703], DL0906 [no panic builtin]; numbers never reused) and the D9 drill
(Windows 1.115s / Linux 0.019s, `measurements/first-run/RECORD.md`). Artifact:
`release-artifacts/1.0.0/delulu-1.0.0.dwx`, byte-identical across builds, signed, digest in
PROVENANCE. **Numbers RESTATED at the cut regeneration: Study B 8.3% (4/48 — corpus grew by a
D22 fixture) and Study C 2.0×–60.5× slower than C (worse than 9e's 51.1×; fresh number
published). The meta-tests (release.rs, book.rs) pin the EXACT figures — regenerating studies
means updating them in the same motion.** Jesse's principle is on record in the announcement +
Book Ch17: "Freedom with Authority; Freedom with Responsibility" (mechanisms named, metaphor
labeled). OS story: evidence-tiered (Win+Linux verified; macOS expected-unverified; ports are
open-source contributions via RFC).

**Do not lower the ratchet.** `delulu-conform`'s `COVERED_FLOOR` is **287** and may only rise; a
lowered floor in a diff is a coverage regression wearing a disguise. And
`release_requires_full_coverage` is ACTIVE — coverage below 100% fails every commit now.

**Stage 10 IS COOKING (Fable 5, 2026-07-20).** Build order `c50de27` (rulings D1–D7; phase order
= gates before engines, policy before power, sim before hardware; 12 phases 10a–10l). **10a DONE
`95dc967`**: attributes activate (@aot/@interpret/@jit/@inline on fn/actor/module header; DL1901
exact removal repair; fmt round-trips; invariant-45 twin witness; attrs swallow their line
terminator — Go-style termination gotcha). **10b DONE `62039f6`**: exec.native grant (default-off;
--grant-manifest NEVER confers it; lease derives false, D6), manifest declaration, authority
`native_emission` stamp only-when-requested (byte-stability skip branch), DL1906 warning-class
(D7: a hint may not change whether a program runs — the suite caught the 45-vs-46 collision, twin
test upgraded to strong form: stamp is the ONLY difference). Broker exec.native lattice dimension
= 10l ENTRY gate per D6. **10c DONE `eae77dd`**: bounded mailboxes (`actor A(mailbox = N)` decl
wins + `[actors] mailbox/overflow` manifest defaults; unconfigured = unbounded 1.0 behavior;
`block` CAS-exact w/ B2 criterion witnessed peak≤bound zero-loss; `drop-new` counted never
silent; DL1902 error only in abort mode; SAME-WORKER EXEMPTION (D8c: a worker can't wait on a
mailbox only it can drain — witness test deadlocks if broken); slot release = Drop guard;
--trace-memory ships B3's mailbox half, heap bytes wait for 10d's collector; per-spawn override
deferred to RFC, D8a). **10d DONE `9828317`**: the cycle collector (mark-and-break between turns, D9 soundness: roots =
worker's actor states only — no locals/continuations, immutable globals; registry = List/Record
cells + closure-captured scopes, in-turn registration only; threshold 64; Study-C perf gate
−1.0% geo-mean NO regression, measurements restored not re-published; Stage-1/7 leak note
CLOSED — 200/200 corpus cycles collected, Weak-proven freeing, reachable-cycle safety witnessed
both levels; rcaps RESIST cycles — corpus needed explicit `ref`). Rulings D1–D9. Coverage 100%
hard gate (291 anchors); suite 948/0/4.
**Next phase: 10e (Track D1 — `Actuate` activates: Cap[Actuator]/Cap[Sensor], envelope scopes,
double validation, ActuateErr, DL1904; the command dies, not the process).**
Spec is at Rev 2 (`a6aec1d`, owner-directed): eight tracks A–H. New vs Rev 1: Track F
(heterogeneous compute — vendor-neutral adapters, `Cap[Compute]`, kernels = ForeignCall + envelope,
DL1907/DL1911), Track G (post-quantum crypto — hybrid ed25519+ML-DSA-65, ML-KEM-768+X25519,
KAT-or-`--unstable`, house rule 5 = NEVER hand-roll crypto, "quantum-proof"/"quantum-safe" allowed
only in prohibition sentences), Track H (cloud/fleets — `delulu deploy plan`, DL1909, no plan no
launch), Track D generalized to autonomy domains via the new normative
`STAGE10_AUTONOMY_ADDENDUM.md` (vehicles/aircraft/satellites/robot-fleets; batteries + safety
chains as device classes; certification honesty: claims NO ISO 26262/DO-178C/ECSS credit;
**broker federation is a NAMED RFC-GATED GAP, addendum §2.5** — Stage 5's broker is local-only, do
not imply cross-machine grant trees exist). Invariants 49–53, DL1907–DL1911, criteria 1–11,
P1–P10. A three-lens pre-commit review caught 4 blockers (vocab-rule self-contradiction; dead-man
overclaimed vs COMPROMISED programs — it defends against silence, not malice; federation implied
as existing; crypto precedent inverted) — all fixed pre-commit. Read
`STAGE10_SPECIFICATION.md` + addendum + playbook first. Standing order then: NEVER push to
GitHub (superseded 2026-09-14 — [[delulu-github-remote]]); no forbidden word; Stage 10 BUILD work waits until v1.0 ships (rc.1 blockers first).
