# V2 decision log

Append-only. Records are `D-V2-nn`. Each carries **evidence**, **alternatives** where any were
weighed, **why**, and a status: **RULED** (the owner decided), **TAKEN** (within the head chef's
remit, reversible, recorded), **PROPOSED** (needs the owner). A reversed decision gets a new record
naming the old one. The V1-era records `D-NE-01…D-NE-33` are in
`docs/archive/v1/NEXT_EVOLUTION_2026/DECISION_LOG.md`; where a `D-NE` record is ruled or superseded
here, this log says so and the archive is not edited.

---

## D-V2-01 — The Next Evolution 2026 plan is approved; execution starts as DeluluLang V2 — RULED (owner, 2026-09-17)
- **Evidence:** `docs/design/DeluluLang_V2_Execution_Master_Prompt.md` ("approval is given"; the
  phase list in its §8; V1 preserved, V2 the active path).
- **Why:** the owner's decision. It approves the plan's content and the order of `V2_MASTER_PLAN.md`
  §4 — which is the archive's `MASTER_PLAN.md` §12.3 with P1 and PS-0 listed as consecutive phases
  (D-V2-07) and the P4 sub-phases named a–e.
- **Supersedes:** the "PROPOSED" status of D-NE-1, D-NE-19 and D-NE-21 (the plan, the order, the
  sandbox as an early cross-cutting layer). D-NE-23 (the interpreter in the microVM guest) is RULED by
  the commission's §16, subject to verification against the code before PS-C.

## D-V2-02 — `docs/DELULULANG_V2/` is the single active source; the V1 archive is `docs/archive/v1/`, mirroring paths — RULED (owner) / TAKEN (mechanics)
- **Evidence:** commission §2, §3, §26: one active V2 source of truth; old, superseded and historical
  documentation moved (never copied, never deleted) into `docs/archive/v1/` preserving the original
  relative structure; the planning folder becomes historical once the migration is complete.
- **Alternatives:** `docs/archive/` without a version segment (D-NE-11); per-campaign folders; keeping
  `docs/NEXT_EVOLUTION_2026/` in place with a banner.
- **Why:** the owner named the folder. Mirroring paths makes every move reversible by `git mv` alone
  and gives old citations one decodable rule. The planning folder is archived in the same phase that
  creates the V2 files carrying its active content, so there is never a second live plan.
- **Supersedes:** D-NE-11.

## D-V2-03 — The Survey resolves a citation of a pre-archive path to its archived mirror, as a note — TAKEN
- **Evidence:** the Survey walks every markdown file and reports a prose citation of a missing path as
  a warning and a broken link as an error. After V2-0, historical and verbatim documents (the
  ledgers, the build orders, the owner's commission texts, the archived planning records themselves)
  cite paths that no longer exist there, and the move manifest must record original paths by design.
- **Alternatives:** rewrite every citation in every file (edits historical statements, and makes the
  manifest's "original path" column report itself as a defect); accept permanent warnings; a
  scanner rule.
- **Why:** the rule keeps historical text as written, keeps the map honest (the edge is drawn to the
  file that exists, and the note names the move), and keeps warnings meaningful. An explicit link
  stays an error, because a link must work for a human reader; those are rewritten. The rule is
  deterministic, provenance-based and unit-tested with a falsification.
- **Confidence:** high. **Needs:** nothing further; a future archive root joins the same constant.

## D-V2-04 — What moved and what stayed in V2-0 — TAKEN
- **Evidence:** the archive's `DOCUMENTATION_AUDIT.md` (every file classified, inbound references
  counted); the two tests that name moved paths; the Survey's extraction of rulings and findings.
- **Why:** the 14 files the audit proposed plus the 16 of the planning folder moved (32; the rows
  and reasons are `V2_DOC_MOVE_MANIFEST.md`). Kept in place, each for a stated reason: the build
  orders and `HARDENING_CAMPAIGN.md` (the map extracts rulings and findings from them; 258 bare
  citations rely on the documented default), `PROOF_CAMPAIGN.md` (16 inbound references; a maintained
  evidence record), `CROSS_PLATFORM_VERIFICATION.md` (the live CI ledger), the owner's commission
  texts and `DeluluLang_PROMPT.md` (verbatim, where the owner placed them), release snapshots,
  entrenched paths, `docs/security/`, the measurement records, `docs/lang/` (pending content).
  `HANDOFF.md` is not moved; it is shrunk to a briefing in P6 and gains V2 pointers now.
- **Confidence:** high on the mechanism; the owner may name further files to archive at any time.

## D-V2-05 — Opus 5 is the main execution sous-chef; the head chef architects and verifies — RULED (owner)
- **Evidence:** commission §5, §6, §30; the owner's agent rule of 2026-09-17 (revised the same day).
- **Why:** the owner's decision. Application: one strong Opus 5 agent at a time; a second agent only
  for a real independent reason; every brief written to the file with objective, files, constraints,
  security and verification requirements and expected outputs; agents ask rather than decide; nothing
  an agent claims is used before the head chef verifies it; on a limit the agent is stopped safely and
  its work preserved; no chain-of-thought is stored. The sous-chef's records live in project storage
  beside the repository and are summarized in `V2_AGENT_LOG.md`.

## D-V2-06 — The phase pause: one controlled continuation, never a loop — RULED (owner)
- **Evidence:** commission §7, §31: after every phase STOP, wait about sixty seconds for owner input;
  a reply stops automatic progression; silence continues to the next approved phase; exactly one
  controlled phase-to-phase continuation; no uncontrolled background loop.
- **Why:** the owner's decision; it also matches the standing rule that an autonomous wake-up is a
  one-shot chain, never a recurring cron. Application: the wait is a single timed background step
  started after the phase's result is recorded and pushed; a message from the owner arriving during
  it is processed first; the phase-end logging and verification are never skipped to continue.

## D-V2-07 — P1 and PS-0 run as two consecutive phases — TAKEN
- **Evidence:** the archive's roadmap places PS-0 inside P1; the commission's §8 lists P1 and PS-0 as
  separate entries in that order.
- **Why:** two phases keep each end-of-phase gate small and let the sandbox-truth work start from a
  green P1 baseline. Nothing is reordered: P1 then PS-0, as both documents say.

## D-V2-08 — Resources become authority dimensions where they can be formalized — RULED (direction, owner) / TAKEN (method)
- **Evidence:** commission §13 and §34.D; NE-22 (no bound on the main program).
- **Why:** the owner's direction. Method: a budget joins the ⊑ order only with a containment
  relation and a meet that are added to the Z3 model and the exhaustive enumeration and documented in
  `docs/MATHEMATICS.md`; a budget that cannot yet be formalized stays an explicit launcher control
  labelled as such. Defaults are the owner's (D-NE-31, asked at PS-A/PS-B).

## D-V2-09 — Execution modes STRICT / AUDIT / BREAK-GLASS; a program can never relax its own authority or sandbox — RULED (principle, owner) / TAKEN (design)
- **Evidence:** commission §14; `DL1408`'s existing no-silent-downgrade rule; `DEPLOYMENT.md` §6's
  reasoning on defaults.
- **Why:** the owner's principle. Design in `V2_SECURITY_MODEL.md` §5; the transition matrix is a
  test obligation of PS-A (STRICT/AUDIT) and PS-B (BREAK-GLASS); revocation, kill and restart are
  preferred over hot weakening; language semantics, execution enforcement and operator controls are
  kept distinct in every document and every diagnostic.

## D-V2-10 — No holder-kind discrimination anywhere; no "AI mode" — RULED (owner; restates Constitution §5.16)
- **Evidence:** commission §12, §17, §21, §22, §34.G.
- **Why:** the owner's rule and the constitution's. A V2 surface that would branch on the kind of
  principal is refused at review; policy, identity, authority, grant, sandbox, attestation and audit
  are the only levers.

## D-V2-11 — AI-native usability is measured by a V2 benchmark (five conditions, seven measures) — RULED (owner) / TAKEN (design)
- **Evidence:** commission §19, §34.A; Study B's existing repair-loop measurement.
- **Why:** the owner's instruction; the design is `V2_AI_NATIVE_DESIGN.md` §4; scheduled as P4e
  after the surfaces it measures exist. Its first result is published whatever it says.

## D-V2-12 — `SandboxPolicy` is a first-class formal object — TAKEN (design; carries D-NE-22)
- **Evidence:** commission §34.C; the archive's architecture §3 and §8.
- **Why:** a policy that is explainable, hashable, auditable and diffable is what lets every backend
  enforce one meaning and lets an auditor compare two runs. PS-A-06 builds it; its hash enters the
  `sandbox-launch` audit record.

## D-V2-13 — The sandbox default posture is decided at PS-A with evidence — PROPOSED (owner; narrows D-NE-33)
- **Evidence:** commission §14 ("secure sandboxing should be the normal/default posture where
  appropriate; disabling must be explicit"); D-NE-33's argument that a default which breaks the
  primary workflow teaches people to disable it; `DEPLOYMENT.md` §6.
- **Recommendation:** default-on for non-interactive runs (`--json`, `--no-prompt`, a harness) and
  for any run under a lease; opt-in for an interactive developer run in 1.x; a package may require a
  level; the flip to always-on is a major-version act. Asked at PS-A's start with the measurement of
  L1's launch and round-trip cost beside it.

## D-V2-14 — `docs/design/AI_NATIVE_DESIGN.md` stays normative; `V2_AI_NATIVE_DESIGN.md` extends it — TAKEN
- **Why:** the V1 document records commitments the binary honours (the JSON contract, locale
  invariance, determinism); the V2 file adds the zero-shot loop, the benchmark and the new surfaces.
  Two files with one boundary beat one file that mixes a record with a plan.

## D-V2-15 — The three owner commission texts stay in `docs/design/`, verbatim — TAKEN (restates D-NE-20)
- **Why:** the owner placed them there; the V2 execution prompt joins the first two as the third
  commission text. They cite the pre-archive planning paths in fenced blocks, which the Survey now
  reports as notes under D-V2-03 rather than as defects.

## D-V2-16 — `fix` remains the repair command; `delulu repair` is not added in V2-0 — TAKEN (reversible)
- **Evidence:** the commission's §18 lists `delulu repair` among the surfaces to build toward;
  `delulu fix` exists with the widening rule and a gated command list (help, dispatch and completions
  bound to one list).
- **Why:** a second name for one command is a decision for P4 (an alias widens the surface the gate
  binds), not a gap to close in a documentation phase. Recorded so the omission is visible.

## D-V2-17 — P1-11 (`[run-authority]`) and the `test --test-authority` flag are deferred to P2's ruling on operator-side grant sources — TAKEN (reversible)
- **Evidence:** both are new sources of authority outside the program (a manifest table for
  `delulu run`; a command-line ceiling for `delulu test`); P2 must rule the loading grant's
  spelling (D-NE-10, the owner's), which is the same question.
- **Why:** one ruling for every operator-side grant source keeps the ceiling law (a source can
  never exceed the manifest's `[authority]`) stated once and tested once. P1 documents the rule
  as it stands today (an effectful single-file test needs a package `[test-authority]`) and
  builds nothing new. D-NE-17 stays the owner's.

## D-V2-18 — Non-human users first; human surfaces are conveniences — RULED (owner, 2026-09-18)
- **Evidence:** the owner's message of 2026-09-18: DeluluLang will have a majority of non-human
  users — AI, LLMs, agents, robots, and future forms (AGI, ASI, physical AI); focus there; not every
  feature is implementable on every surface, and that does not matter.
- **What it means, concretely:** the machine channel (the CLI's `--json`, the LSP for harnesses,
  the MCP server, the skill, the toolchain manifest, the sandbox, run-time plugin loading) is the
  product; the VS Code extension and every human render are conveniences over the same compiler.
  Every phase's acceptance is stated for the machine channel first; the editor gets no phase of its
  own and the read-only Guard view (P4-07) moves to the end of P4; human-facing prose consolidation
  stays P6; the AI usability benchmark (P4-08) is the measure of success. The surfaces that differ
  by host (sandbox levels per OS; contained plugin execution refused on Windows) stay honest
  through `sandbox probe` and `doctor`, never smoothed over.
- **Order:** unchanged — the approved order already serves this (P1 machine contract, PS-0/PS-A a
  safe place to run agent-written code, P2 the agent extends a running system, P4a the skill).
  Offered to the owner, undecided: pulling P4b (`toolchain --json`, `schema`, `examples --json`)
  directly after P1, because they are what an untrained model learns the language from.

## D-V2-19 — The DL0301 → DL0404 cascade is fixed by poison propagation, as follow-up P1-F1, not inside P1 — TAKEN (reversible)
- **Evidence:** Entry P1, DECISIONS item 3: `unknown name f` (DL0301) is followed by "value of type
  `'t0` is not callable" (DL0404), naming a type the source never writes;
  `examples/greeter/src/main.delulu` reached as loose files yields the pair twice over. The naive
  fix, suppressing DL0404 whenever the callee's type is an inference variable, would let
  `fn apply[F](f: F, x: Int) { f(x) }` compile. That program is refused with a correct DL0404 today,
  so the naive fix would change the accepted language.
- **Alternatives:** leave the cascade (rejected: it is the NE-04 shape on the machine channel, where
  an agent receives every diagnostic); the naive suppression (rejected: a language change); **poison
  propagation**, where an inference variable created for a name that failed to resolve is marked as
  born from an error and DL0404 is suppressed only when the callee's type is such a variable
  (chosen).
- **Why not inside P1:** it is a type-checker change in `delulu-check` (`check.rs`, blast radius 145
  in the Survey) that P1's brief did not name. It deserves its own brief, witnesses and snapshot
  review, and P1's verification is closed.
- **Acceptance (P1-F1):** the cascade program yields DL0301 alone; `fn apply[F](f: F, x: Int) { f(x) }`
  still yields DL0404; every conformance case keeps its outcome; the snapshot moves only diagnostic
  counts in cases that contain the cascade; a mutant that suppresses on every inference variable
  makes the generic witness fail.

## D-V2-20 — The sous-chef's two judgements in P1 are confirmed — TAKEN
- **`required_grants` does not list `exec.native`.** Head chef, on the P1 binary:
  `authority tests/conformance/accept/25_attributes_hints.delulu --json` reports
  `required_grants: ["console"]` while `native_emission` reads `{requested: true, via: "@jit"}`.
  Invariant 45 (an attribute is a hint and changes nothing observable) holds, and the list stays
  true: the program runs interpreted without that grant (DL1906). The request remains visible where
  it already lived.
- **`drop-val-annotation` removes the whole type annotation.** Head chef, on the NE-02 shape
  (`let rows: val List[List[Int]] = [a]` with `a: ref List[Int]`): the repair's one edit removes
  `: val List[List[Int]]`, and the program then checks clean. Removing only the keyword
  (`let rows: List[List[Int]] = [a]`) still yields DL1603, "where `val` is required (binding)". The
  repair is `safe`, not `exact`, so `delulu fix` lists it as a suggestion and does not apply it on
  its own, which is the rule for every `safe` repair.

## D-V2-21 — The `sandbox` object travels in a run report the runtime writes to a file, never on the program's standard output — TAKEN (reversible; re-specifies PS-0-02)
- **Evidence:** under `--json`, `delulu run` keeps the program's own bytes as its standard output
  (`crates/delulu/tests/json_contract.rs`, `NO_SUCCESS_SWEEP`). The effect trace already takes the
  file route (`--trace-effects --trace-out <path>`, or standard error without a path). PS-0-02 as
  first written put the `sandbox` object on the standard output of `run --json`.
- **Alternatives:** the object on standard output (rejected: it interleaves with the program's
  bytes, and **the program writes to standard output too, so it could print a counterfeit `sandbox`
  object** claiming a confinement it does not have); standard error (rejected for the same reason:
  the program writes there as well); **a report file named by a flag and written only by the
  runtime** (chosen).
- **The ruling:** (1) `delulu run --json --report-out <path>` writes one envelope
  (`command: "run"`) to `<path>` carrying the `sandbox` object, the execution mode, the budgets and
  the run's outcome, both when the program ran and when the run was refused before it started, so
  the report exists in every outcome. (2) The program's standard output and standard error are
  untouched; without `--report-out` nothing changes. (3) The report is the runtime's alone: a
  `<path>` inside a scope the program is granted to write is refused before the program starts,
  because a program must never be able to write, or redirect, the report of its own confinement.
  PS-0 checks whether `--trace-out` needs the same rule. (4) Wherever a V2 document says the
  `sandbox` object, the mode or the budgets appear "in `run --json`", it means this report. (5) The
  spelling `--report-out` (unused today; it mirrors `--trace-out`) is PS-0's to confirm; PS-0's
  success sweep drives `run` through it, and `run` leaves `NO_SUCCESS_SWEEP`.
- **Why:** non-human users first (D-V2-18). An agent has to learn what confined a program from a
  channel the program cannot forge.

## D-V2-22 — One small log — RULED (owner, 2026-09-18)
- The owner: the logs cost more tokens than the work. From now on one short block per phase in
  `V2_LOG.md`; `V2_EXECUTION_LOG.md` and `V2_AGENT_LOG.md` are frozen at P1 (kept, not deleted);
  decision records are at most three lines. Evidence goes to project storage as raw command output.

## Owner decisions carried from V1, still open
D-NE-3 (snapshot regeneration is a reviewed act — the diff is shown in each phase's log), D-NE-5,
D-NE-6, D-NE-7, D-NE-8, D-NE-10, D-NE-17, D-NE-24, D-NE-25, D-NE-26, D-NE-27, D-NE-28, D-NE-31,
D-NE-33 (narrowed by D-V2-13); the Constitution §5.15 wording (RW 7.10a); rustfmt and a code of
conduct; the four pre-public-repository items. Each is asked at the start of the phase that needs it
(`V2_MASTER_PLAN.md` §7).
