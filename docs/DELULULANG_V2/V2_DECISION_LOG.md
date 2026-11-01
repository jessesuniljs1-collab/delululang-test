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

## D-V2-23 — Three owner rulings — RULED (owner, 2026-09-18)
- D-NE-17: build `delulu test --test-authority <row>` (done in P1-F; it may only narrow the enclosing
  package's ceiling, on every route). P1-11 `[run-authority]`: NOT built, because a manifest would grant itself authority.
  D-NE-28: special-use addresses need a separate explicit grant spelling (PS-0-09).

## D-V2-24 — PS-0's five questions — TAKEN (head chef; (a) RESOLVED by D-V2-26 on 2026-09-20)
- (a) a DL code for the special-address refusal needs the entrenched `witnesses.toml`: **the owner ruled no (D-V2-26)** — it keeps sharing the existing network-refusal code; exit 2 stands.
  (b) POSIX name normalization deferred (RW 4.17). (c) 60 s foreign-call deadline, env can only shorten: kept.
  (d) `--report-out` without `--json`: kept. (e) UNC allowed in operator `fs.*` grants only: kept (RW 4.17).

## D-V2-25 — PS-A’s four owner rulings — RULED (owner, 2026-09-18)
- D-NE-24 profiles `dev`, `contained`, `hostile-agent`. D-NE-26 the `landlock` and `seccompiler` crates approved.
  D-NE-31 default budgets 1 GiB memory and 5 min CPU (never unlimited; the operator may change them).
  D-NE-33 **sandbox ON by default everywhere** (supersedes D-NE-33’s off-by-default and D-V2-13). PS-A must say how a host without L1 behaves: refuse, never silently downgrade.

## D-V2-26 — PS-A's three owner rulings — RULED (owner, 2026-09-20)
- **(a) No new sandbox DL codes in PS-A.** The existing diagnostic contract is sufficient for this
  phase; sandbox refusals keep reusing DL1401/DL1408 and plain `error:` + exit 2. Dedicated codes are
  reconsidered once the machine contract is stable. `witnesses.toml` is NOT touched. This also
  **RESOLVES D-V2-24(a)**: no dedicated code for the special-use-address refusal either — reassess in
  the mature network-policy phase, and only if a machine consumer has a demonstrated need to
  distinguish that condition.
- **(b) The sandbox stays OPT-IN for now**, and D-V2-25's "ON by default everywhere" is a
  *destination*, not this phase's behaviour. The owner's progression: **PS-A = opt-in while the
  enforcement channel is incomplete; PS-B/PS-C = expand the channel and enforcement coverage; then
  default-on, once the actual supported execution surface can be enforced.** An unsupported operation
  is still never silently downgraded — it refuses.
- **Why:** flipping the default while the channel cannot carry actors, foreign C, Python, plugins,
  devices or secrets would refuse most of the examples corpus, and the alternative — running those
  unconfined with a warning — is the silent-downgrade shape D-V2-25 itself forbids, only louder.
- **Standing instruction:** these three are not to be asked again unless new evidence shows the
  architecture has materially changed.

## D-V2-27 — D-NE-10, the run-time loading grant — RULED (owner, 2026-09-20)
- **Both spellings** (the archive's option (c)): `--grant plugin=<path-or-dir>` for the five-minute
  case, AND a `[plugins]` section in `delulu.toml` naming permitted artifacts **by hash**. Every path
  in either form goes through the containment resolver, so `..`, a symlink or a case difference cannot
  widen the grant — the P22 shape.
- **Why:** hash-pinning in a manifest is the supply-chain-honest form and matches the lockfile's habit;
  the flag is what an operator types while developing, and a single-file `delulu run x.delulu` has no
  manifest to edit. Path-only pinning would leave an artifact swap at that path invisible until it ran.
- Opens P2. `DL0703` must name the flag when a program needs `Load` and was not granted it.

## P2 build order — the run-time load path (P2-01, head chef, 2026-09-20)

Reviewed before code, as the roadmap requires. What follows is the shape P2 builds, and — as much as
the shape — the places where it must refuse.

**What already exists.** Stage 6 built nearly all of the load SEQUENCE and none of its entry point.
`delulu-runtime/src/plugin.rs` holds `step1_container_api`, `step2_class`, `step3_ceiling`,
`step4_holder`, `load_prepare`, `step5_verified`, the `PluginEngine` trait (`read_artifact`,
`validate_imports`), `cap_slice`, `r_get_verified`, `r_get_contained`, `HandleTable`,
`kill_on_limit`, `unload`, and the refusal vocabulary. `delulu-wasm` implements the engine.
`delulu plugin verify` drives it from the CLI.

**What does not exist**, and is NE-01: nothing calls any of it from a running program.
`prim.rs:453` is the whole of it —

    "plugin_host" => Err(Fault::at("DL0703", "plugin hosting is not available in the Stage-1 runtime", span)),

— so `root.plugin_host()` type-checks (it is in `prim_table.rs`) and then refuses at run time. The
checker, the authority report and the Atlas all already model plugins; the runtime does not load one.

### The path P2 builds

1. **The grant** (P2-02, D-V2-27). Two spellings, both ruled: `--grant plugin=<path-or-dir>` and a
   `[plugins]` section in `delulu.toml` naming artifacts **by hash**. Every path goes through the
   containment resolver before it is stored, because a grant is a path spelling too, and `..`, a
   symlink or a case difference must not widen it — the shape that produced SYMLINK-DANGLE-1 and
   GUARD-SPELL-1 one layer out. `DL0703` names the flag when a program needs `Load` without it.
2. **`root.plugin_host()`** (P2-03) mints a `Cap[PluginHost]` whose scope carries the granted roots and
   the hash ceiling — never the program's own idea of either.
3. **`load(host, path, grant)`** runs steps 1–7 through the `delulu-wasm` engine. The path is resolved
   against the host capability's scope by the same primitive-table containment every `fs.*` operation
   uses; a path outside it is refused before a byte is read.
4. **The holder check** (P2-04) creates a child grant node under the program's node, in embedded and
   daemon custody alike. `unload` and `grants revoke` kill it; a revoked export faults with
   `PluginErr::Revoked` on its next call.
5. **Limits** (P2-05) come from `Grant.limits` on the live engine. The Windows `Contained` refusal is
   retained exactly as it is: it is a real limitation, and softening it here would be the silent
   downgrade this project keeps refusing.
6. **Evidence** (P2-07): the loaded node id in the audit record and in `--trace-effects`.

### The rules this path must not break

- **`verify ≡ load`.** `plugin verify` and a real load must reach the same verdict on the same bytes,
  or `verify` is advice rather than a check. The sequence functions are shared, not reimplemented.
- **The hash is checked on the bytes that were OPENED**, not on the path. A `.dpx` swapped between
  `verify` and `load` is the TOCTOU case, and the artifact already carries its blake3 content binding
  — so the check is on the buffer in hand. (P2-08 witnesses it.)
- **A grant can never exceed the artifact's ceiling** (`step3_ceiling`), and the manifest's `[plugins]`
  hash list is a CEILING, not a source of authority: it says which artifacts may load, never what they
  may do. This mirrors `foreign.c`, where the manifest names *what* and the operator supplies *which*.
- **A plugin cannot load a plugin** (R-7). The child's grant derives no `Load`.
- **For a sandboxed program the load runs HOST-side.** A guest cannot load code: the channel carries no
  plugin request kind, and `unsupported_surface` already refuses such a program with exit 2. That
  refusal stays until the channel carries it, which is not P2.
- **Windows `Contained` stays refused**, and the run says so.

### Where this could go wrong, and what will catch it

The highest-risk item in the whole plan is a new code-loading path, and the mitigations are reuse
rather than novelty: the containment resolver for paths, the existing sequence functions for the
decision, and the existing refusal codes. P2-08 red-teams the path — path spelling, the swapped
artifact, a grant wider than the ceiling, `Contained` on Windows, a plugin loading a plugin — with each
test witnessed failing on the unpatched code, not merely passing on the patched one.

## D-V2-28 — D-NE-5, the Agent Skill — TAKEN (head chef, 2026-09-20, under the owner's delegation)
- The owner delegated the remaining phase-start decisions on 2026-09-20: *"don't need to ask me
  questions. u can take better decisions."* Entrenched files, the final public repository and licensing
  stay his regardless.
- **Folder `skills/delulu/`, `name: delulu`.** The format requires folder == `name:`, and `delulu` is
  what an agent types.
- **Validated by an IN-TREE test, not the reference Node validator.** The format's required fields are
  four single-line scalars; importing an npm package into CI to read them is supply-chain surface for
  nothing. And the in-tree gate can check what an external validator cannot: that every `delulu <verb>`
  the skill teaches is a verb the binary's own `--help` documents, and that every `[agents.*]` anchor it
  cites exists in `for-agents.md`. A skill naming a command the tool lacks sends an agent into a loop it
  cannot escape, because the instructions it was given are its authority.
- **`delulu skill` prints the committed file**, embedded with `include_str!`, so it reads outside a
  checkout — and a test asserts the two are byte-identical. Two copies of agent instructions is two
  things to go stale, and this project has watched that happen twice.

## D-V2-29 — P3-01, the standard-library method set — TAKEN (head chef, 2026-09-20, under the owner's delegation)

The roadmap's P3-01 asks for a ruling *before* code: the method set, `Map`'s key discipline, and the
WASM policy per method. Additive work, so minor-version, not an RFC (`REMAINING_WORK.md` 2.1).

### The measured starting point

`List` has four methods (`len`, `get`, `push`, `map`), `Str` has six, and there is no `Map`. **And none
of the ten lower to WASM** — the backend has no `List` in its `Ty` at all:

    $ delulu build listwasm.delulu --target wasm
    error[DL1201]: WASM codegen does not support this expression form

for a program whose only method call is `xs.len()`. So the WASM question is not "which new methods
lower" but "does this phase change a backend that already refuses all ten".

### The ruling

**WASM: none of them lower, and that is not a regression.** `Str`, `List` and `Map` methods stay
interpreter-only in 1.x. `Map` has no `Ty` representation and would need a heap layout, a key
canonicalization and an ordering in the guest — three designs, each a place for a security decision on
an unnormalized representation. The backend's existing `DL1201` already says exactly the right thing
("run it on the interpreter"). **`REMAINING_WORK.md` 2.1's sentence "Each method needs … a WASM
lowering" is corrected by this ruling**: it stated a rule the four existing methods already break, and a
rule that the code does not follow is not a rule.

**`List[T]` gains eleven.** `filter(fn(T) -> Bool) -> List[T]`, `fold(A, fn(A, T) -> A) -> A`,
`find(fn(T) -> Bool) -> Option[T]`, `contains(T) -> Bool`, `sort() -> List[T]`, `reverse() -> List[T]`,
`concat(List[T]) -> List[T]`, `is_empty() -> Bool`, `pop() -> Option[T]`, `slice(Int, Int) -> List[T]`,
`join(Str) -> Str` (on `List[Str]` only).

**`Str` gains four**: `to_upper()`, `to_lower()`, `replace(Str, Str)`, `chars() -> List[Str]`.

**`Map[K, V]` is new**: `Map()` (a free prelude builtin, following the `Ok`/`Err`/`Some`/`None`
precedent — there is no literal syntax and no static-method syntax to hang `Map.new()` on),
`get(K) -> Option[V]`, `insert(K, V) -> Unit`, `remove(K) -> Option[V]`, `len() -> Int`,
`keys() -> List[K]`, `values() -> List[V]`, `is_empty() -> Bool`, `contains_key(K) -> Bool`.

### The six decisions inside that list, each with the reason it went this way

1. **`join` lives on `List[Str]`, not on `Str`.** The roadmap's P3-03 line lists `join` under `Str`
   (Python's `sep.join(xs)`). Deviating deliberately: the receiver is the collection being folded, and
   two spellings of one operation is exactly the drift this project keeps finding. Recorded here rather
   than quietly implemented.

2. **`sort()` is refused on `List[Float]`.** There is no total order on `Float`: NaN compares false
   against everything, so every comparison sort places it by accident of the algorithm. A sort that
   silently puts NaN somewhere is a decision made on a representation that does not admit the decision.
   `Int`, `Str` and `Bool` sort; `Float` is refused with the reason named in the diagnostic; any other
   element type is refused because it has no order at all. Stable sort, so equal elements keep their
   input order and the answer is reproducible.

3. **`contains` on an opaque element type is `DL0605`** — the SAME code `==` already emits for
   Secret/Cap/Root, with the same "use `Secret.verify` for secrets" hint. `contains` is `==` in a loop,
   so if the two disagreed one of them would be wrong; sharing the code is what keeps them from
   disagreeing. Note what the alternative would have been: `Value::eq` answers `false` for two
   `Secret`s, so an admitted `xs.contains(k)` over secrets would have returned a confident, wrong
   `false` — an equality answer derived from secret data, which is the shape of finding IF-1. No new
   code, per D-V2-26.

4. **`Map` keys are `Str`, `Int` or `Bool` in 1.x, iteration is ascending by key.** `Float` keys are
   refused for the reason in (2) plus `-0.0 == 0.0`; structural keys (lists, records) are refused
   because their canonical form is a design, not a detail. Deterministic iteration is ascending key
   order, so `keys()`, `values()` and any future `for` agree with each other and across runs — the
   hash-order nondeterminism that makes other languages' map output untestable never exists here.
   `is_opaque` gains its `Map` arm in the same commit: without it, `Map[Str, Secret[Str]]` would compare
   structurally, which is the recursion `List`/`Option`/`Result` already have and the reason to add it
   by pattern rather than by instance.

5. **`pop` and `insert`/`remove` mutate, so they take `push`'s reference-capability path.** `push` is
   already special-cased in `rcap_check` so a write through a `val` (deeply immutable) reference is
   refused. Every new mutating method joins that list *in the same edit*, and a test asserts the list
   and the mutating set are the same set — a mutator that forgot to register would be a silent hole in
   `val`.

6. **`to_upper`/`to_lower` are full Unicode and are documented as NOT a security normalization.** They
   are `str::to_uppercase`/`to_lowercase`, so they are locale-independent and round-tripping is not
   guaranteed (`ß` → `SS`). The reference says plainly: never case-fold to compare a path, a host name
   or a capability — this project's own P22 campaign found four defects of exactly that shape. And
   `replace("", to)` returns the receiver unchanged: unlike `split("")`, which has a natural reading
   (the characters), an empty replacement pattern has none, and inserting between every character is a
   surprise, not a semantic.

### The one soundness change this phase forces

`fold`'s callback is argument **1**, not argument 0. The R-4 gate (the builtin-callback law, the C88
fix) reads `arg_tys.first()`. Left alone, `xs.fold(0, effectful_fn)` would have dropped the callback's
row — the exact escape F-3/F-4 calls a total soundness failure, reopened by a method with a different
argument order. So `is_higher_order_method` becomes `higher_order_callback_arg`, returning the
callback's POSITION, and the fail-closed branch moves with it. A test pins that every higher-order
entry's recorded position is the position `method_sig` type-checks as a function, so a future method
cannot register with the wrong index.

### `chars()` and `split("")`

`split("")` already returns the characters. `chars()` is the named spelling and is *defined* as equal to
it; a test asserts the two agree on the same input, including on multi-byte characters, so the second
path cannot drift from the first.

## D-V2-30 — PS-B-02, the TLS dependency — **RULED BY THE OWNER** (Jesse, 2026-09-20), resolving D-NE-28

**The question, and why it was the owner's.** PS-B-02 builds the first network client this project has
ever had: `http.get` has answered `NetErr::Refused` unconditionally since Stage 1 (finding NE-17). The
DESIGN was never in doubt and was not asked — one host-side implementation serving L0 and sandboxed
guests alike, hostname allowlist, resolve once and pin, special-use ranges refused unless granted by
their own spelling, SNI/Host agreement, redirects re-checked, bounded response, no resolver in the
guest. What was the owner's was the DEPENDENCY, and D-NE-28 said so in as many words: "the TLS
dependency is the largest this project would take and needs the owner and a `cargo deny` pass."

**The ruling.** `reqwest` over `rustls`, with a minimal explicitly-named feature set; the full HTTP
client with real HTTPS. Not HTTP-only, and TLS is never implemented here — house rule 5 ("cryptography
is NEVER hand-rolled") already forbade the second option and the owner closed the first.

**The feature set, and what each refusal costs.** `default-features = false`, then exactly:

| Feature | Why |
|---|---|
| `blocking` | the interpreter is synchronous; DeluluLang has no async |
| `rustls-tls-native-roots` | the PLATFORM trust store, not a compiled-in CA bundle |

The trust-store choice is the one worth arguing. A compiled-in Mozilla bundle (`webpki-roots`) gives
the same anchors on every platform, which this project normally prefers for reproducibility. It was
rejected anyway: an operator who distrusts a CA does it in the OS store, and a language whose whole
claim is that authority is explicit and operator-controlled must not ignore the operator's own trust
decisions. The cost is honest and must be REPORTED rather than worked around — a host with an empty
store cannot make an HTTPS request, and PS-B-02 owes a `doctor` line carrying the root count so that
failure is legible instead of mysterious. Falling back to a bundled bundle when the store is empty
would be exactly the silent widening this project refuses everywhere else.

Refused, each for a reason rather than for size:

- **`gzip`/`brotli`/`zstd`/`deflate`** — a decompressor **defeats the response size bound**. A bounded
  number of bytes on the wire is an unbounded number of bytes in the program, so the only way to keep
  the bound is to never negotiate an encoding we would have to expand. Same shape as ADAPTER-LINE-1.
- **`cookies`** — a cookie jar is cross-request state the program never granted: ambient authority
  with a specification.
- **`http2`** — HTTP/1.1 is enough for `http.get`, and a second protocol is a second parser.
- **`charset`** — `http.get` answers `Result[Str, NetErr]` and a `Str` is UTF-8. Transcoding from a
  server-declared charset is a conversion decided by the far end, which is the wrong party to decide it.
- **`json`** — the program parses its own bodies.
- **reqwest's own redirect following** is turned off in code (`Policy::none()`), not configured, because
  every hop must go back through the FULL check — allowlist, special-use, re-resolve, re-pin — and a
  policy that only counts hops does none of that.

**The measured cost (the `cargo deny` pass the ruling required).** Taken before and after the manifest
change, on the same lockfile, recorded in full at `measurements/dependency-egress/RECORD.md`:

| | before | after | delta |
|---|---|---|---|
| distinct crates in the graph | 222 | **303** | **+81 (+36%)** |
| distinct licenses | 14 | 15 | +1 (`BSL-1.0`, from `ryu`'s dual `Apache-2.0 OR BSL-1.0`) |
| unlicensed crates | 0 | 0 | 0 |
| `cargo deny check` | advisories ok, bans ok, licenses ok, sources ok | **the same four ok** | none |

Nothing was removed. The +81 is honestly reported rather than minimized: it is the largest single
dependency increase in the project's history, the owner took it knowingly, and three parts of it
deserve naming. `ring` carries assembly and C and is the cryptographic core under `rustls-webpki`.
`wasm-bindgen`, `js-sys` and `web-sys` arrive because reqwest supports `wasm32` targets; they are
target-gated and compile on no platform this project builds, but they ARE in the graph and the
lockfile, and a dependency in the lockfile is a dependency. The ICU stack (`icu_*`, `zerovec`, `yoke`,
`tinystr` — nineteen of the eighty-one) arrives through `idna` for URL parsing, which is the part of this tree that does
IDNA normalization; **that is a normalization on a string used to make a security decision**, so
PS-B-02 owes it the treatment this project's own recurring search key demands and must not assume the
host it checks is the host `reqwest` connects to. Pinning the resolved address is what makes that
answerable rather than a matter of trust.

**Feature accounting is a gate, not a comment.** The manifest names every feature and why every other
one is absent; PS-B-02 owes a test that reads the manifest and fails if a refused feature is ever
enabled, because a comment explaining that decompression is off does not keep decompression off.

## D-V2-31 — PS-B-02, the egress client's shape and defaults — TAKEN (head chef, 2026-09-25, under the owner's delegation)

The owner ruled the dependency (D-V2-30) and the design points he listed; everything below is how
those points were made true, and each item was decided rather than defaulted into. Reversible; the
owner's standing instruction of 2026-09-25 ("do whatever good for delululang") delegates it.

1. **Refuse, never normalize, the URL.** `egress::parse_target` accepts only a spelling no URL parser
   would rewrite: `https://` exactly, printable ASCII, no backslash, no userinfo, a lower-case LDH host
   or a canonical address literal, a canonical port. The HTTP client normalizes host names itself
   (IDNA via `url`, the ICU stack D-V2-30 recorded) and a WHATWG parser reads `127.1`, `0x7f.1` and
   `2130706433` as `127.0.0.1`; refusing those spellings makes the host checked the host dialled,
   instead of a string that some later parser agrees with. An international host is written in its
   `xn--` form, which is what it is on the wire anyway.
2. **Every candidate address is classified, and ONE special-use candidate refuses the request.** A name
   that resolves partly into a private range is a name someone pointed there, and a client that tries
   addresses in turn would eventually dial it.
3. **The pin is a construction, not a comparison.** The client gets the classified addresses as an
   override for the checked host AND a resolver that refuses every lookup, so a host the pin does not
   cover cannot be resolved at all — tested with `localhost`, the one name every system resolver would
   have answered. Proxy variables are ignored (`no_proxy`): a proxy resolves the name itself.
4. **`Refused` stays opaque; `Other` carries a fixed phrase.** Every POLICY decision reaches the program
   as `NetErr::Refused`, so special-use and no-address are indistinguishable to it (a resolver oracle
   built from error messages is still a resolver). Transport failures are `Other` with a phrase chosen
   here — never a server's words or an address. The machine-readable reason (`egress::Reason::code`)
   goes to the operator: stderr, the run report's `egress` object, and a guest's `sandbox.denied`
   (which also reaches the hash-chained audit record through PS-A-08's `channel-violation`).
5. **The defaults.** 8 MiB response, 5 redirects, 30 s for the whole request across every hop. These
   are the egress ceilings until PS-B-01 folds per-run limits in; they are constants in `egress.rs`,
   named in the operator's explanations, and never unlimited.
6. **Only 2xx delivers, and only UTF-8.** A 4xx/5xx or an unfollowed 3xx is `Other("HTTP status N")`
   rather than an error page handed over as data; a body that is not UTF-8 is `Other` rather than a
   lossy string, because `http.get` answers a `Str` and `charset` was refused (D-V2-30).
7. **`net.special=` is its own dimension now.** It was checked at grant parse time and then merged into
   `net`, which was harmless only while nothing connected. `Grants.net_special` -> `RootVal.net_special`
   -> `CapScope::Net { special }`, carried across actor boundaries. A LEASED run gets none (the
   broker's node has no such dimension): it cannot reach a special-use range at all, fail closed, open.
8. **The special-use tables grew to the IANA special-purpose registries**, and the two translation
   prefixes are judged by what they carry: NAT64 `64:ff9b::/96` and 6to4 `2002::/16` are special
   exactly when their embedded IPv4 address is. Grant time and connect time share the tables
   (`netclass::addr_class`), and a test asserts they agree.
9. **The sandbox carries `Http`.** The refusal "cannot carry this program yet: it uses Http" was honest
   while the host had no client and would be dishonest now: `get` needs no new request kind, and what
   crosses back is a `Str` or a `NetErr`.
10. **The CLI forwards `net`, and the download names it.** The dependency's own commit left the CLI
    crate taking the runtime with `default-features = false` and forwarding only `python`, so a network
    client reached the binary only when another workspace member happened to enable the feature —
    and the portable release build (`--no-default-features`, to leave Python out) would have shipped
    none. Fixed in this phase: `default = ["python", "net"]`, and the packaging script builds
    `--no-default-features --features net`. `tests/distribution.rs` pins both.
11. **Two direct dependencies and two test-only ones, none new to the tree's normal graph.** `rustls`
    and `rustls-native-certs` are named directly (already present through reqwest): the TLS
    configuration is built here and handed over whole, `doctor` counts the platform roots with the
    same loader, and a TLS failure is recognised by type. `rcgen` and `rustls` are dev-dependencies for
    a loopback TLS server with a certificate made at test time, so no private key is ever committed —
    measured in `measurements/dependency-egress/RECORD.md`.

## D-V2-32 — PS-B-01, how the main program's budgets are enforced — TAKEN (head chef, 2026-09-25, under the owner's delegation)

The owner ruled the numbers (D-V2-25, answering D-NE-31: 1 GiB, 5 minutes, never unlimited, the
operator may change them). This is how they are made true on an ordinary run, where before there was
no bound at all (NE-22).

1. **A host watchdog that samples, not an OS ceiling.** Every OS ceiling fails somewhere this project
   ships: `RLIMIT_AS` kills the WASM engine at start-up (Wasmtime reserves address space it never
   uses — `jail.rs` carries the scar), `RLIMIT_DATA` is refused on macOS (EINVAL, experiment run
   35480762820), and an allocation refused at a ceiling ends in Rust's allocation-failure abort, which
   leaves no report. A sampler measures the PROCESS, so it holds the interpreter, the WASM engine and
   every actor thread with one mechanism, and it stops the run in a way that can still write the
   report. Its cost is stated, not hidden: 25 ms resolution, so a run can overshoot by what it
   allocates in one interval (`limits.enforced_by` says so).
2. **Peak memory, not current.** Peak commit on Windows (the measure a Job Object's memory ceiling
   uses for sandboxed guests, so the two budgets mean the same thing), peak resident set elsewhere. A
   spike between two samples is still seen at the next one; a current-usage sampler would miss it.
3. **No default wall clock.** D-V2-25 ruled memory and processor time; a program waiting on its input
   uses neither. `wall=` exists for the operator who wants one.
4. **At L0 the operator may raise a budget as well as lower it** ("may change them"); zero is refused
   in every dimension, because it would mean either "stop at once" or "unlimited". Under `--sandbox`
   the profile's rule stands: `--limits` only narrows.
5. **A stop is exit 1 with `outcome.stopped_by`** — the dimension, the budget, the measurement — and
   no DL code: every code needs witnesses in the entrenched `witnesses.toml`, and D-V2-26 already
   settled that sandbox limit kills use plain `error:` plus the exit status. The message never offers
   raising the budget as a repair (`limits.rs`'s attribution rule).
6. **The race between a stop and a normal ending has one owner.** Whichever of the watchdog and the
   finishing run claims the exit first writes the report and chooses the code; the other writes
   nothing.

## D-V2-33 — PS-B-05, a budget as an authority dimension — TAKEN (head chef, 2026-09-25, under the owner's delegation)

D-V2-08 is the owner's direction: a budget joins `⊑` only with a containment relation and a meet
that are proved in the Z3 model, enumerated, and documented. This is how.

1. **Memory and processor time join; wall time does not.** Each of the two is a chain under `≤`,
   so the pair is a product of chains and its laws are the textbook ones. A wall budget is a launcher
   control the operator sets per run (PS-B-01): a program waiting on its input consumes nothing a
   delegator hands down.
2. **Absent is the top.** Every node written before PS-B-05 has no budget, and must keep meaning what
   it meant, so "no budget" is the largest element, not the smallest. Consequently an absent budget
   under a present one is a WIDENING and is refused (the device rate's asymmetry).
3. **A delegation that names no budget inherits its parent's**, filled in at the tree's one
   attenuation chokepoint before `⊑` is asked. Without it every "delegate this slice" under a budgeted
   parent would be refused, since absence is the top. Inheritance cannot widen: it copies the parent.
4. **The wire fails toward the bottom.** An unreadable budget string becomes `BudgetScope::SMALLEST`
   (one byte, one second), never "none": dropping it, as the device dimension drops an unreadable
   envelope, would read it as unbounded.
5. **A lease run is held to its node's budget, which is also its default.** `--limits` may name less
   per dimension, never more (refused before `main`, not clipped, so the operator learns what the run
   got). A node with no budget leaves the operator's budget exactly as PS-B-01 made it.
6. **A `--broker daemon` run's root records the budget the run is held to**, so everything the
   program delegates from it inherits or narrows that budget.
7. **Serialized only when present.** An unbudgeted authority's canonical JSON, wire bytes and
   `grants --json` output are byte-identical to before, so no certificate fingerprint, audit hash or
   snapshot moved.
8. **The model was corrected while extended.** Its full conjunction had carried seven set dimensions
   while claiming "all nine"; it carries the code's eight now, plus the device and the budget. The
   four product laws hold for any product, so dropping the budget from the model left them all
   discharged (measured); one more obligation, the asymmetry at the level of the whole order, is what
   fails then. Three model mutants run in CI and must each end with obligations NOT discharged.

## D-V2-34 — PS-B-03, identity separation for a sandbox guest — TAKEN (head chef, 2026-09-25, under the owner's delegation)

1. **Windows: a per-run AppContainer with NO capabilities.** Created for the run, deleted after it
   (stale profiles of dead runs are pruned by their creator's process id). No capability means no
   network of any kind, so the guest loses the network the Job Object never took from it, and none
   of the operator's files, because they are not granted to that identity. Measured before building
   (a throwaway spike) and after (T14, with a control), and falsified by a launch without the
   attribute.
2. **The guest runs from a RUNTIME COPY** — the executable and the non-system modules this process
   has loaded (`python313.dll` on the default build) — in `%LOCALAPPDATA%\DeluluLang\guest-runtime\<key>`,
   whose one extra grant is read-and-execute for `ALL APPLICATION PACKAGES`. Chosen over granting the
   operator's own install directories, which would widen what every AppContainer on the machine can
   read into the operator's Python install. The copy is code any DeluluLang ships; pruned after a day
   unused, never while in use.
3. **The channel moves to inherited pipes** on that path (`pipe_channel.rs`): an AppContainer's named
   pipes live in its own namespace. The read deadline the named pipe gave the channel is kept by a
   drain thread and a queue.
4. **`LOCALAPPDATA` is passed**, the one variable beyond the loader's: Windows refuses to start an
   AppContainer from an environment without it (error 203, measured) and rewrites it to the
   container's folder, which the T14 test checks.
5. **Absent is loud, never silent.** A host that cannot give the identity runs the guest as PS-A did
   and says why on standard error; the identity appears in `host_guarantees`, `posture.identity` and
   the removal of the `identity_separation` limitation only when the launch applied it.
6. **macOS: T14 by the profile, not an identity.** macOS gives an unprivileged launcher no second
   identity. The Seatbelt profile, which must allow reads, now denies the state directory after the
   broad allow. **Linux: deferred to PS-B-03b (RW 4.21)** — a subordinate uid needs a setuid helper
   and host configuration, and Landlock already holds T14 there where the kernel has it. *(Built in
   PS-B-03b the same day — D-V2-37. Point 3's drain thread was later replaced — D-V2-36.)*

## D-V2-35 — PS-B-06, BREAK-GLASS — TAKEN (head chef, 2026-09-25, under the owner's delegation; D-V2-09 is the owner's principle)

1. **Break-glass needs a restriction to break, so the restriction ships with it.** An operator may
   REQUIRE the sandbox on a host (`delulu sandbox require`). Opt-in: the default stays what D-V2-26
   set, and this is D-V2-25's destination offered now to operators who want it. Under the policy
   `run` without `--sandbox`, `test` and `repl` refuse before reading a line; `run --sandbox` is what
   the policy asks for and works unchanged. `test` and `repl` have no sandboxed form yet and take no
   ticket, so they are simply refused there (named in RW 4.22).
2. **A ticket breaks exactly one thing, once, for one program.** It names the program by the blake3 of
   its bytes and the relaxation (`sandbox`, or `policy-off` to remove the policy), lives at most a day
   (refused if signed to live longer), and is SPENT by an atomic create before anything it permits
   happens. A wrong use (another program, expired, tampered) is refused without spending it.
   Authority, custody, the Guard, grants and budgets are untouched: a break-glass run is an ordinary
   strict L0 run with its own grants. Language semantics are never what it changes.
3. **The credential is an Ed25519 key the operator holds off the host**; only the public half is
   pinned. The signer is checked, not trusted: a body naming the pinned key but signed by another is
   refused. `doctor` notes a pinned key whose private half `delulu keygen` left on the host.
4. **Loud and fully audited.** A banner on standard error (even under `--json`), `break_glass` and
   `break_glass_ticket` in the run report, a `break-glass` record in the audit chain for every use and
   every refused ticket, and `doctor` / `sandbox status` lines. A use that cannot be recorded does
   not happen.
5. **Fail closed on the policy itself.** A policy file that cannot be read, or that says "not
   required", is treated as REQUIRED WITH NO KEY. A policy is never replaced in place, because adding a
   key widens who may break glass; it comes off only by a `policy-off` ticket. A `--no-break-glass`
   policy has no key and so cannot be undone through `delulu` — said when it is written.
6. **Named limit (category 7, RW 4.4):** a process running as the operator can delete the policy file.
   The policy binds what `delulu` runs, and a sandboxed guest, which cannot reach the state directory
   (T14); a same-user shell is what a separate OS account is for.

## D-V2-36 — PS-B-04, channel batching: NOT built; the Windows transport fixed instead — TAKEN (head chef, 2026-09-25, under the owner's delegation)

1. **The rule was fixed before any number** (`measurements/sandbox-channel/RECORD.md`): batching pays
   only above 50 µs of channel cost per effect. Measured on the three CI runners (run `36167360827`),
   by slope: Linux 19.5 µs, macOS 19.8 µs, Windows Server 2025 66.9 µs. Batching fails the rule on two
   platforms and passes it on one.
2. **Batching is not built.** It would change WHEN an epoch-class effect is observed — a clock read
   answered from a snapshot is not taken when the program asked — on every platform, to recover a
   cost one platform has. A semantic change needs a reason on its own; a slow transport is not one.
3. **The Windows transport is fixed instead**, because the cost was taken apart and found there: each
   read crossed a drain thread and a queue, so one round trip woke four threads where Linux wakes two
   (48.6 → 40.4 → 30.1 µs per effect as each hop was removed in a local build). The host now reads the
   overlapped server end of one duplex pipe directly with a real deadline; the guest reads directly
   and a watchdog ends it if a read outlives the deadline. Deadlines kept on both sides; writes gained
   one. Workstation: 28.9 µs per effect (from 48.6); the Windows CI runner 43.0 µs (from 66.9, run
   `36171534543`) — every runner under the line.
4. **Each frame is one write** on both channels (sandbox and broker): no measurable change here, but it
   cannot cost anything and removes a possible second wake per frame.
5. **Re-open condition:** if a runner measures above the line again with this transport, the record
   says so and PS-B-04 is reconsidered on that evidence — the script stays committed and the workflow
   manual, so the question is one button away.

## D-V2-37 — PS-B-03b, a Linux guest as a subordinate uid — TAKEN (head chef, 2026-09-25, under the owner's delegation)

1. **Measured before built** (`host-capability-probe`, `linux-subordinate-uid`, runs `36167365278` and
   `36167600016`): on Ubuntu 24.04's default, AppArmor leaves an unprivileged user namespace without
   the capabilities to set its ids, so even mapping the runner's own uid failed; on the same runner
   with that one sysctl lifted, a child mapped to a subordinate uid was refused the operator's `0600`
   file that the control read. So the mechanism works where the distribution allows it (Debian and
   Fedora by default), and the build must fall back where it does not.
2. **The guest is born in a new user namespace as uid 1 / gid 1**, mapped by the setuid helpers
   `newuidmap`/`newgidmap` (by absolute path, never `PATH`) to one id chosen per run from the ranges
   `/etc/subuid` and `/etc/subgid` give the operator; a range containing the operator's own uid is
   refused. Not uid 0 inside: with no mapping for 0 the namespace has no root, and `exec` clears the
   capabilities. It drops every supplementary group and every capability before the jail's own steps
   run — the order matters, because changing ids clears the parent-death signal the jail sets.
3. **No runtime copy.** The binary is executed through an open descriptor (`/proc/self/fd/N`), so the
   stranger never walks the operator's directories; the channel is an inherited socket pair, so it
   needs no path either. A guest that cannot LOAD as the stranger (a library under a directory only the
   operator may walk) is caught by a ready byte it sends before anything else, and the launch falls
   back rather than failing the run.
4. **It claims less than Windows' container, and the report says exactly what.** A subordinate uid is
   refused what only the operator's account may touch; it still reads what every account may, writes
   where every account may and opens sockets — those are Landlock's and seccomp's to refuse, and they
   still apply inside the namespace. `posture.identity` = "a subordinate uid in its own user namespace",
   `filesystem_reads` = "only what every account on the host may read" unless Landlock narrows it.
5. **Absent is loud.** A host that forbids it runs the guest as the operator, says why on a run's
   standard error, and `doctor` repeats the launcher's own reason. CI proves both: the x86-64 job lifts
   the restriction and REQUIRES the identity (`DELULU_REQUIRE_SUBORDINATE_UID`), arm64 keeps Ubuntu's
   default and exercises the fallback.
6. **Residual, named:** the ids are the operator's to hand out and other tools (rootless containers)
   use the same ranges, so a guest could share an id with a container's process; the per-run choice
   makes that unlikely, not impossible. The broker, the CLI and unsandboxed runs are still the
   operator (RW 4.4, category 7).

## D-V2-38 — P4b–e opening: D-NE-6 (`delulu mcp`) and the introspection surfaces — TAKEN (head chef, 2026-09-26, under the owner's delegation)

1. **D-NE-6 as proposed:** `delulu mcp` is a CLI subcommand, not a separate binary; stateless; a
   deterministic `tools/list`; hand-written JSON-RPC over stdio as the LSP is (no SDK dependency);
   **read-only by construction** — no tool runs a program, grants anything, or loads code, every tool
   is annotated read-only, and a test holds the tool list against that. Why: the door rule — the
   server that reads must never be an effector — is what makes it safe to point at a hostile
   workspace, and the LSP proves the hand-written protocol layer is affordable. It is built in P4-03.
2. **Introspection is read from the binary's own tables, never restated.** `toolchain --json` (P4-02)
   reads the command list, the usage text, the grant parser's forms, the primitive table, the budgets,
   the sandbox levels and the diagnostic registry; two new tables were lifted out of code to make that
   possible (`broker::GRANT_FORMS`, bound to `Grants::add` by a test that reads its match arms, and
   `codes::TOPICS`, bound to `topic_explain` the same way), and one out of the probe
   (`sandbox::LEVELS`). The first run of the grant-form test caught a wrong example (an actuator grant
   without its mandatory dead-man).
3. **`delulu schema` publishes CLOSED JSON Schemas** (P4-09): every object lists every field its
   emitter writes and forbids the rest, except two that say they are open (a command's payload beside
   the envelope; the Atlas's embedded custody view). The validator is in the binary (`schema
   validate`), a subset of JSON Schema 2020-12, so an agent checks an output with the same code the
   tests use and no dependency is added. The payload key is `document`, because the envelope already
   owns `schema`.
4. **`delulu examples` embeds the nine single-file examples** (P4-10) and derives each one's authority
   report through the SAME function `delulu authority` uses (`source_authority_report`, extracted for
   this) and its run line from that report's required grants. The package examples are named with the
   command that checks them. A run line is proven sufficient by running it.

## D-V2-39 — PS-C opening: the microVM's pins, its image, and what stays the owner's — TAKEN (head chef, 2026-09-27, under the owner's delegation)

The prerequisites record is `V2_PS_C_PREREQUISITES.md`; every one was present, so the phase proceeded.

1. **D-NE-23, built as ruled.** The guest runs the interpreter as PID 1 of its own kernel, with one
   device — vsock — and no network device and no filesystem device: the launcher never makes the
   `/network-interfaces` or `/drives` API calls. Firecracker first. Cloud Hypervisor, the second VMM
   the ruling names, is not built in PS-C and stays open.
2. **The pins moved forward from PS-0-08's experiment.** Firecracker **v1.17.0** (release archive
   sha256 `06094a1108ae9e82aa4c23a775aa92758f53f1175d422270d9d6162cb9ade558` x86_64,
   `e351ebe4f7a16b5873bbd51005d2e6767103cff4d5ebc829df2d3f95a93e2256` aarch64), because Firecracker's
   own kernel policy ended 6.1-guest support on 2026-09-02; the guest kernel is **6.18 LTS**, supported
   by v1.17 to at least 2028-06-01: `linux-6.18.54.tar.xz`, sha256
   `9df30b02dd8102bbd0be52556288ef6889ddbe7f1ddb96fbf847d0becf3eacac`, which matches kernel.org's
   published `sha256sums.asc` fetched over HTTPS. The signature on that file was checked the same day,
   once `gpg` could be given the key without `dirmngr`: **"Good signature from Kernel.org checksum
   autosigner"**, key `B8868C80BA62A1FFFAF5FDA9632D3A06589DA6B1`, fetched over HTTPS from kernel.org's own
   `pgpkeys` repository (not web-of-trust certified — the usual caveat, said rather than implied).
3. **The kernel is ours, not the vendor's.** Built from source by `scripts/microvm/kernel.config` on top
   of `tinyconfig` — vsock and nothing else, no IP stack at all — and the build refuses a kernel in which
   any configured line did not survive `olddefconfig` (it refused one on its first run: Landlock had been
   dropped by a missing dependency). Firecracker's CI kernel, used for the first probe, is not used.
4. **D-NE-27 stays the owner's.** A built kernel is written to `target/` (ignored by git) and never
   committed, uploaded or attached; the CI job builds its image and keeps it inside the job. PS-C-05's
   distributed artifact waits for the owner.
5. **The hashes live in the image's manifest, checked on the copy that boots — not compiled into the
   CLI** (a deviation from the plan's "a manifest the CLI carries"). The guest binary is built from the
   same source as the host, so a hash compiled into the host would be a hash of itself; and kernel
   bytes differ by C compiler, so a compiled-in kernel hash would hold for one toolchain only. The
   launcher copies each file into the VM's private directory while hashing it, refuses a mismatch
   before boot (T13), and boots the copy — so the bytes checked are the bytes booted. Pinning a
   DISTRIBUTED image into a release is PS-C-05's, with the owner. sha256, through the `sha2` crate
   already in the tree.
6. **The jailer is not applied yet.** It needs root (a per-VM uid, a chroot, cgroups). Until PS-C-03b
   builds that path the VMM runs as the operator, under Firecracker's own seccomp filters and the
   launcher's ceilings, and every L2 run reports `identity_separation` as a limitation.
7. **The ceilings.** Memory: the VM's RAM is the guest's ceiling (it has no more), and the VMM's data is
   capped at that plus 256 MiB; below 128 MiB a run is refused, because the guest cannot unpack its own
   initramfs. Processor time: `RLIMIT_CPU` on the VMM, whose threads include the vCPU. Wall clock: twice
   the processor time, never under a minute. One vCPU.
8. **The console, bounded.** The guest's standard error reaches the operator through the serial
   console, relayed up to 64 KiB and drained after that; the kernel's own messages are silenced
   (`loglevel=0`). Firecracker advises no serial console in production because a guest can flood it;
   without one a failing guest would say nothing at all, so the bound is the mitigation instead.
9. **`--isolation microvm` IS a sandboxed run, at level 2.** Same guest, same channel, same policy,
   audit records and run report; `--sandbox --isolation microvm` means the same. `--isolation none` or
   `process` beside `--sandbox`, and `--sandbox=off` with `--isolation microvm`, are refused as naming
   two boundaries at once. A host that cannot give L2 refuses with DL1408, as it always did — never a
   run at L1 under an L2 label.

## D-V2-40 — PS-C-03b, the jailer, and PS-C-06's red team — TAKEN (head chef, 2026-09-27, under the owner's delegation)

1. **Root runs the VMM under Firecracker's jailer, or not at all.** Run as root, `--isolation microvm`
   starts the VMM through `jailer`: a uid of its own, in a chroot holding only what it needs, entered
   by `pivot_root` in a new mount namespace. Root WITHOUT a jailer is refused (DL1408, naming the
   jailer), because the alternative is a VMM running as root — a VMM escape would then BE root. Run as
   an ordinary user, the VMM runs as that user (the jailer needs root) and every report names
   `identity_separation` as a limitation, as D-V2-39 said.
2. **A uid per VM**, from a block no ordinary account is given (900000–965535), reserved by creating
   `/srv/delulu-jailer/uid-<n>` exclusively with the host's pid in it — the create is what keeps two
   hosts starting at once apart — and skipping any uid a process runs as or `/etc/passwd` names. A
   reservation is released only by the host that holds it.
3. **The jails live under `/srv/delulu-jailer`** (beside Firecracker's own `/srv/jailer`), overridable
   with `DELULU_JAIL_BASE`, and never on a filesystem mounted `nodev` — refused with that reason. The
   first choice was `/run`, and the first jailed launch failed inside the chroot with KVM's
   "permission denied": `/run` is `nodev`, so the `/dev/kvm` node the jailer makes there cannot be
   opened.
4. **A reaper replaces the death signal.** The jailer's `setuid` clears `PR_SET_PDEATHSIG`, so a jailed
   VMM would outlive a host killed mid-run. Each jailed VM gets a reaper (`__vm_reaper`, internal): a
   process blocked on a pipe only its host writes to, which, when the pipe closes, kills the VMM —
   identified by pid AND jail id, never by pid alone — and removes the jail. Measured: without it, a
   jailed VMM whose host and reaper were both killed was still running fifteen seconds later, bounded
   only by its processor-time ceiling.
5. **Not in PS-C: cgroups and a network namespace for the VMM.** The jailer can place the VMM in a
   cgroup and an empty network namespace; neither is used yet. The ceilings stay `rlimit`s (which
   survive the jailer's `setuid`), and the VMM's own network reach is bounded by Firecracker's seccomp
   filter rather than by a namespace. Named here so they are not mistaken for applied.
6. **The red team's instruments are gates.** `scripts/microvm/redteam/probe.sh` boots the image's OWN
   kernel with a native probe as init and fails unless it finds one vsock device and nothing else to
   reach; `redteam/hostile.sh` runs a program against five hostile guests and fails unless each is
   refused cleanly with nothing left behind. Both run on the KVM CI job.

## D-V2-41 — P6, how `HANDOFF.md` becomes a briefing — TAKEN (head chef, 2026-09-27, under the owner's delegation)

1. **Extract, do not delete or rename.** `HANDOFF.md` stays at the root; the sections the roadmap
   names as history move, **verbatim and under their original numbers**, to
   `docs/archive/v1/HANDOFF_HISTORY.md`, and `HANDOFF.md` ends with a map from each moved section to its
   successor. A citation of "`HANDOFF.md` §8" anywhere in the tree still finds its text. The two rule
   lines that spell the banned word stay exactly where they were and are not copied into the archive —
   the script that made the move asserts both.
2. **One deviation from the roadmap: all of §11 stays, not only §11.1.** §11.2–§11.6 are the project's
   working rules, environment facts, findings that must never be re-softened and operational traps —
   rules in force, which the archive's "not maintained" label would demote. What moved: the old header
   and its update trail, §1.1's six CI-run rows, §3, §5, §6, §8, §9, §12, §13, §14.
3. **Explicit links in the moved text are rewritten to reach their targets from the archive; prose
   citations are left as written** — the V2-0 manifest's rule, for the same reason.

## D-V2-42 — P5, the distribution: D-NE-8 taken in part, D-NE-7 left with the owner — TAKEN (head chef, 2026-09-27, under the owner's delegation)

1. **A hand-written release workflow around `scripts/package-toolchain.sh`, not `cargo-dist`** (D-NE-8's
   first half, as proposed). The script already encodes what the archive must be — the Python-less
   build with the network client, what a recipient is owed, checksums from the staged tree — and a
   generator would re-decide those and add a tool the dependency rule would need a ruling for.
2. **No installer scripts.** A `curl | sh` installer is the field's norm and a security posture; which
   posture DeluluLang takes is the owner's (D-NE-8's second half stays open). Until he chooses, the
   documented manual path is the install — and it is TESTED: `INSTALL.md` §1's `install-gate` block is
   executed as written by `scripts/check-install.sh` against a real archive on every target.
3. **Publication stays the owner's (D-NE-7), and so does anything public and permanent.** A run of the
   release workflow that was not started by a `v*` tag — the manual dry run and the weekly run — builds,
   checks and installs the archive, then discards it: no workflow artifact (a public repository's
   artifacts are downloadable, and the owner has said this repository is not a distribution channel)
   and no provenance attestation (the public attestation log is permanent). A `v*` tag — pushing one is
   itself the owner's act — keeps and attests the archives; a GitHub release is created only when the
   repository variable `RELEASES` is also `on`, and only as a DRAFT a person publishes. `RELEASES` is
   off. Consequence, recorded rather than hidden: P5-01's verification "`gh attestation verify`
   recorded" waits for the owner's first tag.
4. **A tag must have its `## [X.Y.Z]` section in `CHANGELOG.md`** (P5-04), checked before anything is
   built for it; a test holds the current version to the same rule.
5. **The build names itself**: `--version --json` carries `target` and `commit`; the commit is `null`
   when the builder did not say, never read from the environment by guesswork, so two builds told the
   same thing stay byte-identical (the microVM guest's reproducibility check relies on that).

## D-V2-43 — what the 2026-09-27 agent pass changed in the architecture — TAKEN (head chef, 2026-09-27, under the owner's delegation; FLAGGED for the owner's review)

1. **A Guard path rule covers the path and everything beneath it** (GUARD-SCOPE-1). Exact equality made
   a seal on a directory seal nothing in it. The addendum's §2.3 promised the broker's own matcher
   vocabulary; for a path that vocabulary is containment (`fs.read=DIR` grants the subtree), so a rule now
   covers exactly what a grant of the same path would. At mint and request time a path scope is judged by
   overlap in either direction (`/srv/app` includes a sealed `/srv/app/secret`). **Globs are refused**
   when a rule is set — the directory is the spelling that means "all of this", and a pattern that is
   stored and can never match is the failure this entry closes. **Case is folded** on Windows and macOS
   and for any drive-letter path: the lattice compares case-sensitively because for a GRANT that only
   narrows, but a Guard rule REFUSES, and the same choice would fail open. Flagged because the Guard's
   addendum is a design document the owner reviewed; this is its promise kept, not a change of intent.
2. **The audit chain is multi-writer, under one lock** (AUDIT-WRITERS-1). Sandboxed runs, `reconcile`
   and a run's own records already wrote it beside the daemon; the log now owns the lock and catches up
   under it, rather than every writer being asked to be careful. A `seq` is still a writer's own number
   (the broker restarts its count per daemon life); the chain's integrity is its hashes and anchor.
3. **A checked path is opened, not looked up again** (FS-RACE-1). Containment is decided on the resolved
   path and the SAME path is opened with no link followed (`beneath.rs`). Two OS-specific APIs
   (`openat`/`O_NOFOLLOW`, `NtCreateFile` relative opens) through crates already in the tree.
4. **A guest runs under the broker when the run does** (REMAINING_WORK 4.20): the host authorizes each
   channel request through the run's custody, with the interpreter's op mapping, before performing it.

## D-V2-44 — the Guard decides on the resolved path, resolved at the edges — TAKEN (head chef, 2026-09-27, under the owner's delegation; FLAGGED for the owner's review)

1. **A file effect is decided on, and opened at, ONE resolved path** (GUARD-ALIAS-1). The runtime pins
   the path (links followed, checked inside the grant on disk); the broker and the Guard decide on the
   pin; the effect opens exactly the pin with no link followed. A decision on a spelling the filesystem
   does not honour was GUARD-SPELL-1's class, and this closes it for every link, 8.3 name and case.
2. **Ruling 2 stands: the broker never touches the disk.** The resolution happens where a filesystem is:
   the runtime for a use, the CLI (and the daemon's policy-file seed) for an operator's grant, rule or
   request, which are stored resolved. Consequence, stated because it changes what a stored lease means:
   a lease or rule names the directory as it resolved WHEN IT WAS WRITTEN — repointing a symlinked path
   afterwards does not move the authority with it. Leases and rules written before this change keep
   their old spelling and can now fail to match (fail-closed); re-delegate or re-set them.
3. **`Secret.verify` of broker-held secrets is a declassification of ONE BIT about BOTH secrets**
   (VERIFY-FABRICATED-1): computed broker-side, the Guard asked for each secret, audited — and a custody
   without the bytes refuses rather than answers.
4. **A sandboxed stop is named only from evidence** (SANDBOX-STOP-1): the guest's allocator status, a
   microVM guest's console line, the signal, the OS's own accounting, the host's watchdog — never an exit
   code that does not say. The process exit is the report's.

## D-V2-45 — P7: which parsers are fuzzed, and how a Miri test is shrunk — TAKEN (head chef, 2026-09-27, under the owner's delegation)

1. **"The four parsers" are the parsers of bytes an attacker controls:** program source (through the whole
   checker), a package's manifests (`delulu.toml`, a plugin manifest, a lockfile), a plugin's compiled DIR,
   and a lease token — plus the `--grant` parser, which RW 5.4 names. The plan never listed four; this is
   the rule that decides them. The `.dpx` container parser lives in the wasm crate and would bring wasmtime
   into the fuzz build; the DIR it carries is fuzzed instead.
2. **A fuzz property is one function in its crate** (the channel's rule, PS-A-02), replayed by the ordinary
   suite over a deterministic mutation corpus, so a coverage-guided run and a per-commit run cannot test
   different things.
3. **A test is shrunk for Miri only where its size is not the witness.** Miri looks for undefined behaviour;
   generator counts and exhaustive sweeps shrink under `cfg!(miri)`, and every size that IS a witness (the
   100,000-level nesting input) keeps running at full size natively on every push.

## D-V2-46 — PS-D-01: what an external launcher is told, and what the report may say about it — TAKEN (head chef, 2026-09-28, under the owner's delegation)

1. **The launcher is a command, not a shell line.** `external:CMD` is split on whitespace and run
   directly; there is no shell, so no quoting, expansion or injection surface in DeluluLang. Anything that
   needs one belongs in the operator's script.
2. **The channel is the launcher's standard input and output**, and the guest is `delulu __guest
   --stdio-pipes`, which moves the program's own standard output to standard error before anything runs,
   so nothing the program prints can be read as a frame. The launcher is told the guest's words
   (`DELULU_GUEST_ARGS`) and the limits the run asked for (`DELULU_LIMIT_MEMORY_BYTES`, `…_CPU_SECONDS`,
   `…_WALL_SECONDS`); enforcing them is the launcher's.
3. **The report claims only what was measured.** Level 3, backend `external`, `fully_enforced: false`, no
   host guarantee. The layers the guest applied to itself count (RW 4.23 — the host checks the words), so
   on Linux `granted` can be `contained` while every host guarantee stays empty. The host's wall-clock
   watchdog still ends the launcher, but it is not claimed: ending a container's client does not end the
   container.
4. **The launcher's program is recorded — in the report and the `sandbox-launch` audit record — and its
   arguments never are**, because a launcher line is where a registry token or an `ssh` identity goes.
5. **The Docker + gVisor recipe is documented, not shipped**, and says it is not tested by this project.
   Shipping one would make its wall a DeluluLang claim; attesting one is PS-D-02's seam.
6. `--sandbox-backend` without `--sandbox`, with `--isolation microvm`, or in a mode that refuses
   sandbox options (PS-A-10) is refused with exit 2 before anything runs.

## D-V2-47 — criterion 1's speed-up is judged against a control, on a machine that is free — TAKEN (head chef, 2026-09-28, under the owner's delegation; the first cloud routine run)

`actors_pingpong`'s criterion 1 (1.5x at 4 workers against 1) went red on Windows on `937aea8`, a commit
that changed only documents (1.31x, best of two). Witnessed in the 4-vCPU cloud VM by starving it, never
by loosening the bar: idle, 1.89–2.18x; one CPU busy 1.82x, two 1.54x, three **1.20x — red**, the shape
of the Windows failure, with the same runtime binary. The test was measuring how many of the runner's
four CPUs were free.

1. **A control brackets every attempt**: four equal CPU-bound units, run one after another and then at
   once on four threads, each phase the best of three — the machine's parallel speed-up for perfectly
   parallel work, right now (3.2–4.1 on the idle VM, 1.8–2.4 with three CPUs busy). It is the `hw < 4`
   rule the test already had, measured instead of read from the hardware's name.
2. **An attempt counts only where the control gave at least 3.0x on both sides of it.** Below that a
   parallel runtime and a serial one overlap (the runtime gives about half the control's figure), so the
   ratio is printed as NOT MEASURED with its numbers — exactly as on a machine with fewer than four
   threads. Up to three attempts.
3. **The bar is the criterion's share of what the machine gave:** 1.5 of 4, 37.5%, so exactly 1.5x on
   four free threads (capped there) and proportionally less on a machine that gave less — never under
   1.125x, which a runtime that runs its actors one at a time (1.00x) does not reach.
4. **Falsified:** a runtime that starts one worker whatever it is asked for is red on the idle VM (1.00x
   against a bar of 1.21–1.41x, three attempts); starving three CPUs now reports NOT MEASURED instead of a
   defect.
5. **The residual, named:** a passing log does not show whether a run measured or skipped (cargo shows a
   passing test's output only with `--nocapture`), so how often CI's runners are "busy" by this rule is
   not yet known; a red now always carries the control's figure, which says whether the runtime or the
   runner was slow. *(Answered the same day: CI now prints the verdict — §6.)*
6. **What CI's runners are, measured (`0a7b881`, run `36399295189`, 2026-09-28):** the Linux x86-64 and
   Windows runners' controls were **2.28–2.31x and 1.99–2.12x with nothing else running** — four
   hardware threads with about two cores' worth of parallel throughput (SMT), not a busy machine; the
   runtime reached 2.09–2.16x and 1.74–1.81x there, so both are NOT MEASURED. The macOS runner offers
   three hardware threads (the `hw < 4` rule). **The arm64 runner's control was 4.11x and the criterion
   was MEASURED there: 2.93x against the full 1.5x bar** (2.80x on `ed74683`). So the criterion is
   asserted on every push, on arm64, and no longer turns red on an x86 runner that cannot show it —
   which is what `937aea8`'s and `5bb39bc`'s Windows reds (1.31x) were: that hardware, not a regression.
   A floor low enough to measure on the SMT runners would also pass a serial runtime (1.0x), so it
   stays at 3.0; the verdict's words now name SMT beside "busy".

## D-V2-48 — PS-D-02: the attestation seam verifies a statement, not a platform — TAKEN (head chef, 2026-09-28, under the owner's delegation; the first cloud routine run)

Built from the design in `HANDOFF.md` §0, with these choices:

1. **What is checked, and nothing more:** an ed25519 signature over `delulu-attestation-v1\n` and the
   statement's canonical JSON (keys in byte order, no whitespace; control characters refused, so only `\"`
   and `\\` are ever escaped — Python's `json.dumps(sort_keys=True, separators=(",", ":"),
   ensure_ascii=False)` writes the same bytes, checked); the signer is the key pinned on the command line
   (`--require-attestation HEX`); the statement carries this run's nonce (32 bytes of OS randomness).
   The signature field is the detached form every other signature here uses (public key ‖ signature).
2. **Checked before the program is sent.** The host waits up to 10 s (`CONNECT_DEADLINE`, the time a
   guest has to come up) for the document, refuses at once when the launcher exits first, and on any
   refusal ends the guest having told it nothing: the run exits 1 in words, and `sandbox-attestation`
   (deny, with the reason) and `sandbox-death` are in the audit chain. No run report is written for a
   refused run, as for a launch that failed.
3. **The claims are the attester's.** `sandbox.attestation = {key, attester, guarantees, verified}` sits
   beside `host_guarantees`, never merged; the level stays 3 and `fully_enforced` false. A dry run
   (`--mode audit`) reports the requirement with `verified: false`.
4. **Bounded, closed documents:** at most 64 KiB, 32 claims of at most 256 printable characters, an
   unknown field anywhere refused — a claim silently not carried reads like one never made, and a claim
   is shown on a terminal and kept in a report.
5. **L3 only.** With `--sandbox` alone (L1) or `--isolation microvm` (L2) the flag is refused, exit 2: the
   host measures those boundaries itself, and nothing an attester says would be checked against them.
   Without `--sandbox` it is refused like the other sandbox options (PS-A-10).
6. **The reference attester is a verb, `delulu sandbox attest`, beside `sandbox ticket`** — not the
   design's new top-level `attest launch`: it is the same kind of act as minting a break-glass ticket, and
   a verb under `sandbox` needs no new command in the dispatcher, the completions, the MCP door rule or
   the Skill. `--key` names a seed FILE, as `sandbox ticket`'s does. It refuses `--json` (its standard
   output is the channel), removes the two variables from its command's environment, and on Unix
   becomes the command (`exec`), so the host's watchdog reaches it.
7. **`delulu` reads nothing after a bare `--`** — not `--help`/`-h`, `--json`, `--color`, `--locale`, nor
   the undocumented-flag refusal. Found while building 6: `docker run -h HOST` after `--` would have
   printed `delulu sandbox`'s help. No command used `--` before.
8. **The document is read from a regular file only** (ATTEST-FIFO-1): judged by `symlink_metadata`,
   opened `O_NOFOLLOW | O_NONBLOCK` on Unix, judged again on the handle. The read precedes every
   watchdog, so a pipe there would hang the host for ever — witnessed before the fix.
9. **Not built, deliberately:** a lease-level "only attested" constraint (it changes the authority model —
   RFC territory); level 4 `attested` (it waits for a hardware attester whose quote the host itself
   checks); key rotation or a list of pinned keys (one key per run; an operator's script picks it).

## D-V2-49 — Delulu Core v0.3: faults and the higher-order primitive enter the calculus — TAKEN (head chef, 2026-09-28, under the owner's delegation; FLAGGED for the owner's review — an entrenched file)

1. **Progress becomes progress-or-fault (RW 5.2, P17-T2).** A configuration may be `fault(c)`;
   `E-Refuse` takes a present capability whose scope does not cover its arguments there, and
   `E-Fault` propagates it through every evaluation context — the calculus has no handlers.
   Theorem 1: a well-typed configuration is a value, steps, or faults.
2. **`E-Refuse` emits the attempted label**, because the implementation's trace records the attempt
   (witnessed: a `Read` record for the refused `../outside.txt`, then `DL0904`). Emitting nothing
   would also satisfy Theorem 3; the calculus follows the implementation, so `--assert-trace` and
   §8's traceability agree.
3. **The higher-order primitive enters §1–§7 (RW 5.3, P17-T1):** `hop(e, ē)`, typed by `T-HOp` with the
   callback's latent row joined (R-4), reduced by `E-HOp` to an unfolding in which the callback occurs
   only applied. It is the Lean fragment's `TypedGood.ho`, generalized from a literal closure to any
   term of arrow type.
4. **Theorem 3 is stated for every finite prefix** of a run — ending in a value, in a fault, or not
   ending — rather than only for runs that reach a value.
5. **No RFC:** the language does not move; its formal model is corrected toward the implementation
   (`ENTRENCHED_CHANGE_RECORD.md` says why, and how to revert). Nothing new is machine-checked; RW 5.1
   stays open, now aimed at the right calculus.

## D-V2-50 — ADAPTER-SPELL-1: a driver's name is resolved once, by DeluluLang — TAKEN (head chef, 2026-09-28, under the owner's delegation)

1. **One resolution, used twice.** `--adapter-cmd`'s first word is resolved once
   (`cli::resolve_driver`): a word with a path separator is made absolute from the working directory; a
   bare word is looked up on `PATH` by DeluluLang, in its absolute directories only (an empty or
   relative entry is the working directory again, another spelling) — the first executable regular
   file, `.exe` appended on Windows where the word has no extension. The absolute path found is the
   file verified AND the file started. Links are not resolved: a multi-call binary (`sh` → `busybox`)
   dispatches on the name it is started as, and the read that verifies follows the same link the start
   does.
2. **Nothing resolves → refused, never guessed.** The bare word is not handed to the operating
   system's own search, which on Windows looks in its own directories before `PATH` and could find a
   file the check never saw. The refusal keeps the words the old spawn failure had ("adapter could not
   be started").
3. **An interpreter-hosted driver** now resolves to the interpreter it starts, which is what is checked
   unless `--adapter-artifact` names the driver's own bytes; the unsigned refusal and warning say so.
   `--adapter-artifact` remains the operator's statement that the command executes those bytes.
4. **The residual, named:** the file can be replaced between the check and the start by anyone who can
   write its directory — the deployment rule for grants applies to drivers. Closing it needs the
   Verified-class adapter (a `.dpx` whose bytes are loaded once, verified, and executed from memory):
   P8.

## D-V2-51 — P8's shape: the control program in a guest first, then the adapter's logic as a Verified `.dpx` — TAKEN (head chef, 2026-09-28, under the owner's delegation)

1. **P8 is built in three slices** (`V2_P8_DESIGN.md`): P8-01 carries `Actuator` and `Sensor` over the
   existing sandbox channel, performed by the host's `DeviceBroker`; P8-02 makes the driver's logic a
   Verified-class `.dpx`, pure (no capability), whose frames the host writes; P8-03 a reference transport
   to the simulator in its own process. Each is witnessed against the simulator; a real device stays
   environment-blocked.
2. **The logic and the transport are split.** The part of a driver that touches the machine stays the
   host's; the part a vendor writes becomes code the host can prove, sign-check and run from the bytes it
   verified — which is what ends D-V2-50's check-then-start residual, rather than a tighter race.
3. **No new diagnostic codes are planned:** DL1510, DL1511 and DL1905 already say what can go wrong.
4. **Out of P8:** certification, federation model-checking, and any lease-level constraint on what kind
   of adapter a node may use (RFC territory).

## D-V2-52 — NVIDIA OpenShell studied: what is taken, on what terms, and in what order — TAKEN (head chef, 2026-09-28, under the owner's delegation; the owner commissioned the study)

The owner, 2026-09-28 evening, having paused the routine: NVIDIA and its partners launched OpenShell
(the open runtime of NVIDIA's Open Agent Safety Platform) the same day, *"and I want this to be studied
and incorporated not just as copy but as real engineering for the sandbox currently we are working
on"*. Studied on the laptop (Opus 5.5): the 57 pages of its documentation (v0.1.2 and dev) as
Markdown, its seven architecture diagrams read from their SVG sources, its repository at `36b0386`
(`architecture/`, RFCs 0001/0002/0005/0012, the isolation-interface, binary-identity, prover and
supervisor-network crates), the announcement and the solutions page. The record, the side-by-side and
every slice's design: `V2_OPENSHELL_STUDY.md`.

1. **The difference that decides every choice:** OpenShell governs programs it cannot read, so it
   mediates an opaque binary's syscalls and keeps a separate policy in step with it; DeluluLang governs
   programs it compiles — authority computed from the text, a guest that performs no effects, `⊑`
   proved once. So DeluluLang can *deny* where OpenShell must *intercept*, derive its policy rather
   than write it, and ask a boundary question of a program rather than of a policy.
2. **Taken, each re-engineered as a slice with a witness and a mutant:** confirmation of the boundary
   by construction, bound to a per-run generation (PS-E-01); host loss ends the guest on every backend
   (PS-E-02); the guest's kernel surface narrowed (PS-E-03, six hypotheses, none a finding until
   witnessed); the external launcher resolved, hashed and pinnable (PS-E-04); OpenShell as a tested L3
   backend and DeluluLang as the author of its policies (PS-E-05); the audit exported as OCSF and still
   verifiable (PS-E-06); an out-of-band monitor with revoke authority only (P8-04); a program checked
   against a boundary with four results and a source-located counterexample (P9-01); categorical risk
   findings on a change of authority (P9-02); proposals to the operator with approval bounded by `⊑`
   (P9-03); secrets bound to endpoints, checked statically and enforced at send (P9-04); network
   authority scoped to methods and paths as a `⊑` dimension (P9-05).
3. **Not taken, with reasons** (`V2_OPENSHELL_STUDY.md` §5): deny rules inside allow rules;
   hot-widening a running sandbox; binary identity for the guest; `audit` as a default enforcement; an
   HTTP proposal endpoint inside the guest; content-inspecting middleware (still category 7); a fleet
   control plane; L7 protocols beyond REST for now.
4. **Terms.** OpenShell is Apache-2.0. Nothing of it — code, schema files, text, diagrams — is copied
   into the repository and none of its crates becomes a dependency: the licence and `NOTICE` are the
   owner's, and importing Apache-2.0 material would change what `NOTICE` must say. Interop emits
   documents in OpenShell's published policy format — DeluluLang's own output — and uses the name only
   to identify that format. A CI workflow that exercises OpenShell downloads a pinned,
   checksum-verified release at run time and ships nothing.
5. **Order:** PS-E next (P8-01 puts control programs in guests, and the boundary they stand on is
   confirmed first), then P8 with P8-04, then P9. `V2_MASTER_PLAN.md` §4, `V2_PHASE_STATUS.md`,
   `V2_IMPLEMENTATION_ROADMAP.md` and `docs/CLOUD_ROUTINE.md` step 4 say so.

## D-V2-53 — PS-E: the boundary, confirmed — TAKEN (head chef, 2026-09-28, under the owner's delegation)

1. **Six slices** (`V2_OPENSHELL_STUDY.md` §4.1–§4.6): E-01 `Launched → Confirmed → Running`, the
   program sendable only from `Confirmed`; five properties (filesystem confinement, egress
   confinement, privilege floor, host loss ends the guest, resource ceiling), each established,
   absent or unknown, with its mechanism and evidence; a generation for every run. E-02 host loss:
   a macOS watcher (`kqueue NOTE_EXIT`, `getppid` fallback), a death signal or Job Object for the
   external launcher. E-03 the kernel surface (H1–H6). E-04 the launcher pinned. E-05 OpenShell interop.
   E-06 OCSF export.
2. **Profiles gain a required set.** `hostile-agent` requires all five properties; `contained`
   requires filesystem, egress and resource; `dev` none. A missing required property refuses before the
   program is sent, under DL1408's rule. Consequence accepted: `hostile-agent` on a Linux host that
   forbids user namespaces refuses instead of running the guest as the operator's own user, and names
   the ways out (the microVM, an external launcher, the host setting).
3. **Hypotheses stay hypotheses** until an escaped-guest witness is red on the current binary; a
   refuted one is recorded with its evidence and changes nothing.
4. **New diagnostic codes** are allowed under the owner's delegation, each with its own record here.
5. **PS-E-05's soundness rule:** an exported policy never allows more than the program's authority and
   grants; a grant with no equivalent refuses the export by name; a dimension OpenShell does not model
   is listed as unrepresented. Its workflow is manual until it has run green, as `container.yml` was.

## D-V2-54 — P8 gains P8-04, an out-of-band monitor — TAKEN (head chef, 2026-09-28, under the owner's delegation)

NVIDIA's Sentry (announced 2026-09-28) watches agents from a DPU the agent's host cannot touch and
quarantines in milliseconds. The hardware is out of reach; the shape is not. P8-04: a monitor is a
separate principal (its own process, a separate OS identity where the host allows one) holding exactly
**revoke** over one run's grant node, reading only what the host recorded (the audit, or PS-E-06's
stream), applying declarative rules, and quarantining by revoking — the device fail-state and e-stop
engage (implemented), the guest ends. It can grant nothing and perform nothing. Designed in
`V2_P8_DESIGN.md` and `V2_OPENSHELL_STUDY.md` §4.7; built after P8-03.

## D-V2-55 — P9: authority at the boundary — TAKEN (head chef, 2026-09-28, under the owner's delegation)

1. **Five slices** (`V2_OPENSHELL_STUDY.md` §4.8–§4.12): P9-01 `delulu authority --within BOUNDARY`,
   four results as a closed set (within 0, exceeds 1 with a source-located counterexample,
   unsupported/inconclusive 3; 2 a bad invocation), coverage named; P9-02 `grants diff` findings;
   P9-03 proposals, operator review, auto-approval only as the operator's standing `⊑`-bounded
   delegation with no findings, and a re-run rather than a live widening; P9-04 secrets bound to
   endpoints (a request with headers, a secret's audience in its grant, the host resolving bytes at
   send only inside the audience, every redirect re-checked, and the checker proving a secret reaches
   only a header position); P9-05 methods and restricted path globs as a `⊑` dimension in the Z3 model.
2. **The invariant is untouched:** a program cannot relax its own authority. A proposal is a request
   to an external principal; auto-approval is that principal's own delegation.
3. P9-04 and P9-05 change the language and `⊑`: each carries its primitive-table version, conformance
   anchors, Z3 obligations with mutants, and `MATHEMATICS.md` entry, as PS-B-05 did.

## D-V2-56 — PS-E-01's first step: the channel opens with a generation and the program follows the confirmation — TAKEN (head chef, 2026-09-28, under the owner's delegation)

1. **The channel is `delulu-sandbox-channel/3`.** The host's first frame (`Open`) carries the version and
   the run's generation and no program; the guest's first request is its confinement report echoing the
   generation; the program (`Program`) is sent only after the host accepts it. A host and a guest of
   different versions refuse each other in words — so an external launcher's image must carry the same
   `delulu` as the host, as `DEPLOYMENT.md` already told operators.
2. **The order is a type** (`crates/delulu/src/boundary.rs`): `Confirmed` has one constructor,
   `Opened::confirm`, and the program frame one writer, `Confirmed::send_program`. A first request that is
   not the confinement report is refused unanswered.
3. **Every run has a generation**, fresh from the OS, at every level — in the launch and death records and
   the report. An attested run signs over it: one nonce per run, not two.
4. **A guest that does not confirm is a failed run (exit 1), not yet a refusal code.** Its report says
   `ran: false` — the program was never sent. The refusal under DL1408's rule belongs with the required
   sets (the next step), where there is a property to name.
5. **Split, so each step is witnessed on its own:** this step (the order and the generation); next, the
   five properties, the profiles' required sets and the refusal (D-V2-53 §2), then E-02 … E-06.

## D-V2-57 — PS-E-01's properties are reported on every OS before any profile requires them — TAKEN (head chef, 2026-09-28, under the owner's delegation)

1. **Every sandboxed run reports five properties** (`sandbox.properties`), each established (by what),
   absent (why) or unknown (why), answered from the posture the same report carries — one source, so the
   properties, the posture and `host_guarantees` cannot disagree. An external launcher's are unknown.
2. **No refusal yet.** D-V2-53 §2's required sets, read against the jail code, would refuse every macOS
   run under the default profile (reads open; no memory ceiling). The sets are decided on the reported
   answers of all three operating systems as CI measures them, and the decision is recorded then — the
   choices being a platform-honest set (what each OS can establish, the gap named), or the refusal with
   the microVM and an external launcher as the ways out.
3. **The privilege floor counts a separate identity** — a per-run AppContainer, a subordinate uid, the
   jailer's uid for the VMM — as well as `no_new_privs` and Seatbelt's deny-by-default, as §4.1 lists.

## D-V2-58 — the red-team pass on `/3`: a failed channel ends its guest, `ran` means sent, a guest's words are data — TAKEN (head chef, 2026-09-28, under the owner's delegation)

1. **A conversation that failed ends the guest**; a guest that said goodbye has 10 s to exit, then is
   ended. The host never waits on a guest's goodwill (GUEST-WAIT-1).
2. **`outcome.ran` means the program was sent** — written whole to a confirmed guest. The death record
   carries `confirmed`, `sent` and `ended_by_host`. A program larger than the channel's frame is refused
   before launch, exit 2 (RAN-SENT-1).
3. **A guest's words are data**: escaped and bounded (`channel::shown`) wherever they can reach the
   terminal, the report or the chain (GUEST-TEXT-1).
4. **An external launcher's pipes are bounded both ways**: a write keeps the channel's deadline; unread
   output waits in a bounded queue (PIPE-WRITE-1, PIPE-FLOOD-1).
5. **F7 is the next step's decision**: an external guest's self-report must stop counting as a host
   guarantee (RW 4.31), taken with the profiles' required sets.

## D-V2-59 — `hostile-agent` requires all five boundary properties; `contained` waits for its gaps — TAKEN (head chef, 2026-09-28, under the owner's delegation; FLAGGED for the owner's review — it narrows D-V2-53 §2)

1. **`hostile-agent` requires all five properties**, checked in `Opened::confirm` after the guest's report
   and before the program is sent; a missing one refuses with DL1408, exit 2, naming it and the ways out.
2. **`contained` and `dev` require none for now.** D-V2-53 §2's `contained` set, measured against CI, would
   refuse every macOS run (reads open, no memory ceiling, no death signal) and every unattested external
   launcher by default. It is taken when those gaps can be closed (PS-E-02; a read-deny Seatbelt profile; a
   macOS memory ceiling; RW 4.31 for L3), each measured.
3. **The privilege floor is `no_new_privs`, a restricted token, a separate identity or a VM** — as §4.1
   lists — so `hostile-agent` runs on a Linux host without user namespaces (arm64 CI). §4.1's consequence
   ("refuses instead") would need identity separation as its own requirement; not taken, and flagged.

## D-V2-60 — PS-E-02 on macOS: a watcher OUTSIDE the guest ends it with its host — TAKEN (head chef, 2026-09-29, under the owner's delegation)

1. **The watcher is a process, not a thread in the guest** — a departure from `V2_OPENSHELL_STUDY.md`
   §4.2's design. A watcher inside the guest is the guest's own word (RW 4.31's lesson): a guest that
   escapes the interpreter could stop it. Outside, the guest cannot reach it — its Seatbelt profile denies
   every signal. It is this binary (`__host_watch <pid>`, the environment cleared), started by the host
   right after the guest, as the microVM's reaper is.
2. **It waits on the host's pipe and the guest's exit** (`kqueue`: `EVFILT_READ` on a pipe only the host
   holds; `EVFILT_PROC`/`NOTE_EXIT` on the guest, registered while the host still holds the guest
   unreaped, so the pid is the guest's). The host gone first ends the guest with SIGKILL; the guest gone
   first ends the watcher. Not `kqueue` on the host's pid, and not `getppid() == 1`: the pipe also
   closes when the host FINISHES, so dropping the `Jail` ends a guest the host is done with — the Windows
   job's kill-on-close semantics, on macOS.
3. **Claimed only once armed.** "killed with the host" (and `host_loss_ends_guest: established`) is
   reported only after the watcher writes that both waits are registered, within the connect deadline; a
   watcher that did not arm is ended before its pipe closes, and the run says what is missing.
4. **The external launcher gets the same watcher on macOS**, and nothing is claimed for it: the level stays
   3, and what the launcher started is the launcher's. **On Windows (the same run) the launcher joins a job
   whose only limit is kill-on-close** — created suspended, assigned, resumed — held beside the jailed
   guest's measured job, so a level-3 run gains no processor-time watchdog and no stop named from the
   launcher's accounting: nothing new is enforced on the launcher but its end. Red: `37828ab`, `witness.yml` run `36527876892` — "the external launcher outlived its host by more than 3.0 s"; green: `1a63829`: `36528140459` (`sandbox_confirm_cli`, 7 passed — the launcher gone 20 ms after its host was killed), `36528149629` (`sandbox_external_cli`), `36528152057` (the jail unit tests), `36528154367` (`sandbox_run_cli`) — all success.
5. **CI-only defects are witnessed off `master`** (`witness.yml`): red on a branch, green on the same
   branch, then `master` fast-forwarded — the history keeps the red witness and `master` never goes red.
   Red: `36526271163` (the guest, "outlived its host by more than 3.15 s"), `36526351005` (the launcher);
   green: `36526707055` (`sandbox_confirm_cli`, 12 passed — the guest gone 3.6 ms after its host's SIGKILL, the launcher's witness green too), `36526709263` (`guest_cli`, 5 passed — the jail's report names "killed with the host"), `36526711462` (the boundary unit tests, 3 passed), all on `2d9d2a0`.

## D-V2-61 — PS-E-03's first step: the escaped guest; H1, H2, H3 confirmed and closed, H7 found — TAKEN (head chef, 2026-09-29, under the owner's delegation)

1. **The escaped guest is a test-only child, not a mode of the shipping binary.** `jail::escaped_tests`
   re-runs the test binary as a child that applies exactly the guest's own lock-down (`confine_filesystem`,
   `lock_down_self`) and then makes raw system calls, against a FREE control that must succeed at each one.
   A guest mode in `delulu` itself would be a switch in every shipped guest; the child is the same code
   with no interpreter in the way, which is the escaped guest's position. An attempt the child does not
   report fails the test — the harness first dropped `CLONE3` unmeasured.
2. **H2 (GUEST-SOCKET-1): the filter refuses `socket` and `socketpair`.** The guest's channel is connected
   before it locks down, and every effect is the host's, so it needs no other socket. It reports the new
   known word "no sockets but the channel" (`channel::SELF_APPLIED`), and the posture's `network: only the
   channel` now needs THAT word (or macOS's, a VM's, an AppContainer): Landlock's "no TCP bind or connect"
   alone had been answering "only the channel" while UDP, netlink and Unix sockets were open — an
   over-claim in every Linux report until now.
3. **H1 (GUEST-SYSCALL-1): the calls the filter did not name are refused** — `memfd_create`, `io_uring_*`,
   `userfaultfd`, `pidfd_*`, the new mount API, `kexec_file_load` — and `clone` with any namespace flag
   (one rule per flag); `clone3`, whose flags no filter can read, is answered ENOSYS by a second filter, so
   the C library makes threads with `clone`. No new word is claimed for these: "no namespace or module
   tricks" already said it, and is now true of `clone` too.
4. **H3 (GUEST-PROC-1): Landlock grants `/proc/self`, not `/proc`.** Traced first: after it locks itself
   down a guest running a program opens no file at all, so nothing global under `/proc` is granted.
   **H7 (GUEST-DEV-1), found building H3: `/dev` is narrowed to null, zero, full, random and urandom** — the
   whole of `/dev` let an escaped same-user guest open the operator's terminal for reading.
5. **Open:** H4 (the host non-dumpable during a run — depth now that H3 is closed), H5 (the Landlock ABI as a
   `hostile-agent` requirement), H6 (macOS `mach-lookup`, Windows named objects — the same harness on those
   runners).

## D-V2-62 — PS-E-03 H4: the host is non-dumpable while it serves a sandboxed run — TAKEN (head chef, 2026-09-29, under the owner's delegation)

1. **On Linux the host sets `PR_SET_DUMPABLE = 0` once its guest is launched**, for the rest of its life: its
   `/proc` entries become root's, so no process of the same user may read its environment or memory or
   attach to it. Witnessed red as a non-root user (a same-user process read the serving host's `environ`,
   sentinel and all — HOST-DUMPABLE-1); root may read any process, so the witness says "unmeasurable" as
   root and CI's non-root runners carry it.
2. **After the launch, not before:** a child forked from a non-dumpable process keeps the flag until its
   `exec`, and the Linux identity path writes that child's `/proc` uid map in exactly that window.
3. **Never undone**, and no core dump: the host exits when the run ends, and a dump would spill the same
   bytes. macOS and Windows are H6's (the same question on those systems, through the same harness).

## D-V2-63 — PS-E-03 H8: an escaped guest cannot type into the operator's terminal — TAKEN (head chef, 2026-09-29, under the owner's delegation)

1. **A new hypothesis, H8, found and confirmed the same run.** A guest shares its host's session, so the
   operator's terminal is its controlling terminal and its standard error. The escaped guest pushed a
   keystroke into that terminal's input with `TIOCSTI` after its full lock-down — for the operator's shell to
   read and RUN once the run is over (`dev.tty.legacy_tiocsti` is 1 in this VM, as on many hosts): a
   sandbox escape through the terminal, the class of CVE-2017-5226 (GUEST-TIOCSTI-1).
2. **The filter refuses `ioctl` with `TIOCSTI` or `TIOCLINUX`** (a console selection paste), comparing the low
   32 bits of the command — the kernel truncates it to them, and a 64-bit comparison let the same command
   through with its high bits set (the harness's `TIOCSTI_HIGH`; mutant M8).
3. **Not taken here:** giving the guest no controlling terminal (`setsid`) and relaying its standard error
   through the host instead of the terminal itself (RW 4.32 names the relay). The filter closes the escape;
   those would narrow what a guest's own output can do to a terminal — escape sequences — and are open.

## D-V2-64 — PS-E-03 H9: an escaped guest signals only itself — TAKEN (head chef, 2026-09-29, under the owner's delegation)

1. **H9, found and confirmed the same run:** after its lock-down an escaped guest could still signal another
   process of the same user, its whole process group (its host and the terminal's foreground job among
   them), and queue signals — `kill(-1, SIGKILL)` would have ended every process the operator has
   (GUEST-SIGNAL-1; witnessed with signal 0, which delivers nothing).
2. **The filter refuses `kill`, `tgkill`, `rt_sigqueueinfo` and `rt_tgsigqueueinfo` unless they name the
   guest's own process** — its pid, fixed when the filter is made, compared on 32 bits as `pid_t` is — and
   `tkill` outright. `abort` (a signal to itself, through `tgkill`) keeps working: the witness's control.
3. **Landlock's signal scoping (ABI 6) is not what closes it:** CI's Ubuntu 24.04 kernels predate it, and the
   filter holds on every kernel the guest runs on. Where ABI 6 exists it would be depth.

## D-V2-65 — PS-E-03 H10: an escaped guest changes no other process — TAKEN (head chef, 2026-09-29, under the owner's delegation)

1. **H10, found and confirmed the same run:** beside signals, the calls that act on another process of the
   same user by pid — its resource limits (`prlimit64`), priority (`setpriority`), CPUs
   (`sched_setaffinity`), scheduling class (`sched_setscheduler`), I/O priority (`ioprio_set`) — all
   succeeded on the operator's process after the guest's lock-down (GUEST-PROCESS-1; witnessed by setting
   each to the value it already had, so nothing was changed).
2. **The setters are refused outright** — with `sched_setparam`, `sched_setattr` and `process_madvise` — since
   a guest sets none of them even on itself; **`prlimit64` is refused unless it names the guest** (0 or its
   own pid), because the C library reads its own limits through it (the witness's control, `PRLIMIT_SELF`).
3. The class — "an escaped guest acts on another process of its user" — is now covered for signals (H9) and
   these; `ptrace`, `process_vm_*` and `pidfd_*` were already refused.

## D-V2-66 — PS-E-03 H5: writes are "denied" only where truncation is refused too — TAKEN (head chef, 2026-09-29, under the owner's delegation)

1. **H5 confirmed, as an over-claim in the report.** On a kernel whose Landlock predates ABI 3 (5.13–6.1)
   the guest reports "no file writes but truncation" — an escaped guest may still `truncate` any file its
   user can write — and the posture's `has` matched "no file writes" INSIDE that word: the report said
   `filesystem_writes: denied`, `filesystem_confinement` established, and `hostile-agent` ran
   (LANDLOCK-TRUNCATE-1; witnessed red in the property test before the fix).
2. **Writes are "denied" only on the exact word.** Below ABI 3 the report now says writes are not confined,
   names the limitation, and `hostile-agent` refuses — H5's "the ABI as a requirement", through the required
   set. `has`'s substring match stays for the rows whose longer words rely on it (the microVM's), and only the
   one word that another word extends is matched exactly.
3. **Kernels ≥ 6.2 are unchanged** (CI's 6.8 among them); the effective ABI is still named by the word, not
   by a number.

## D-V2-67 — Wasmtime 47 → 48.0.3, the 48 LTS line, for RUSTSEC-2026-0315 and -0316 — TAKEN (head chef, 2026-09-29, under the owner's delegation)

1. **The supply-chain gate went red on a records-only commit** (`9fc4d86`, push run `36537537718`): two
   advisories published after its last clean run, both against wasmtime 47.0.4 — RUSTSEC-2026-0315 (`call_ref`
   and an exception `catch` can drop fuel accounting, so a module's fuel is amplified exponentially) and
   RUSTSEC-2026-0316 (component-model record lifting can allocate past the host-call fuel limit). Witnessed in
   the VM with cargo-deny 0.20.2: `cargo deny --all-features check advisories` exit 1 on 47.0.4 naming both,
   exit 0 on 48.0.3; bans, licences and sources ok.
2. **Reachability, read rather than assumed.** 0316 is not reachable here: `harden_wasm_features` turns the
   component model off. 0315 is reachable in principle: a plugin's fuel is a GRANTED limit
   (`delulu-wasm/src/limits.rs`, `consume_fuel(true)`), and neither function references nor exceptions are
   refused, so a hand-made `.dpx` could spend more than its grant (the wall-clock watchdog still bounds it).
   So the answer is the upgrade, never an `ignore`.
3. **48, not 49.** Both 48.0.3 and 49.0.1 carry the fix. Wasmtime 48 is a long-term-support line — the
   advisory's other patched range, 36.x, is the previous one — so it is patched for longest; it needs Rust
   1.95 and this workspace pins 1.96.1; it is one major step, not two. No source line changed: the workspace
   compiles as it was, clippy is clean and the full suite passes on it.
4. **Next — RW 4.33:** narrow what a plugin may use to what DeluluLang emits (function references, GC,
   exceptions refused at validation), so the next fuel-accounting defect in those proposals is unreachable
   too — "upgrade AND narrowing", the rule `deny.toml`'s own history states. **Done the same run: D-V2-68.**

## D-V2-68 — RW 4.33: both WebAssembly engines refuse the 3.0 proposals DeluluLang never emits — TAKEN (head chef, 2026-09-29, under the owner's delegation)

1. **Witnessed, and wider than the advisory.** On `63a375e` (wasmtime 48.0.3) the program engine accepted
   function references, exceptions, GC, tail calls and multi-memory — each on by wasmtime's default — and
   the plugin store RAN a Contained module that calls through `call_ref` (`Ok(Some(7))`): the path
   RUSTSEC-2026-0315 used to spend more than a plugin's granted fuel was open until the upgrade, and the
   next defect in those proposals would have found it open again.
2. **Refused at validation, on both engines** (`harden_wasm_features`, which both call): GC, function
   references, exceptions, stack switching, tail calls, multi-memory, custom page sizes, wide arithmetic
   and shared-everything threads — beside SIMD, relaxed SIMD, threads, memory64 and the component model,
   refused since P17-F. The ones wasmtime leaves off today are named too, so a future default cannot switch
   them on. The legacy exceptions form is not named: wasmtime keeps that switch deprecated, for its own
   spec tests, and Cranelift cannot compile it (an engine asked for it is refused).
3. **What stays: the WebAssembly 2.0 set, and extended constants.** A Contained `.dpx` carries a module an
   ordinary compiler built, and a default wasm32 toolchain emits bulk memory, reference types, multi-value,
   sign extension, saturating conversions and mutable globals — so those keep loading, and a test holds
   that side as firmly as the refusals. `codegen.rs` emits none of the refused proposals (its whole
   instruction list is MVP), so no DeluluLang program changes: the two-engine differential and the parity
   suites pass unchanged.
4. **A narrowing, never a widening:** it can only turn a module that would have loaded into one that is
   refused before it runs (the plugin store answers "module failed to compile", naming the proposal).

## D-V2-69 — PS-E-04: the external launcher is resolved once, hashed with BLAKE3, pinnable, and on Linux started as the file hashed — TAKEN (head chef, 2026-09-29, under the owner's delegation)

1. **Witnessed first (LAUNCHER-SPELL-1).** On `cfc5b01`, `external:lnch` with `.` ahead of the operator's
   directory on `PATH` ran a `lnch` planted in the working directory, while the report named `lnch`: the
   operating system's search honours a relative entry. ADAPTER-SPELL-1's shape (D-V2-50), for launchers.
2. **One resolver.** The launcher's word is resolved by `cli::resolve_driver` — the code that closed
   ADAPTER-SPELL-1, shared, not copied: a word with a separator is a path from the working directory; a
   bare name is looked up in `PATH`'s absolute directories only, `.exe` appended on Windows, and nothing is
   left to the operating system's own search (on Windows the standard library's also looked in the
   application's own directory and the system directories before `PATH`; an operator who relied on that
   names the path now).
3. **BLAKE3, of the bytes opened.** The file is opened once — non-blocking, and judged by the descriptor's
   own `fstat`, not the name `resolve_driver` looked at — and hashed with BLAKE3, the repository's file hash
   (a `.dpx`'s content bindings, the lockfile, the audit chain, a program's hash in its launch record; the
   microVM's image manifest follows kernel.org's SHA-256).
4. **Two flat, additive report fields, not the design's `launcher: {path, digest}`.** `sandbox.launcher`
   has been a string since PS-D-01, and schema 1 promises it; changing its type would break every reader of
   it. So `launcher` stays the word as named, and `launcher_path` and `launcher_blake3` join it, named for
   their algorithm as `microvm_image.kernel_sha256` is — in the report and in the `sandbox-launch` record.
5. **`--launcher-digest HEX` refuses before anything starts, exit 1, in words** — naming the path and both
   digests — **and records a `sandbox-launcher` deny** in the chain: a launcher that changed under a pin is
   news to someone. Exit 1 and no DL code, as its sibling pin's refusal (`--require-attestation`, PS-D-02)
   is: the command line was well formed; the world did not match it. A malformed digest, or the flag
   without an external launcher or without `--sandbox`, is exit 2 like every bad invocation.
6. **Linux starts the descriptor (`fexecve`)** as the last `pre_exec` step, with argv and envp built
   before the fork (the environment is built from the command — `environ` in a `pre_exec` step is not yet
   the command's, witnessed: the launcher started with none of the host's words). A `#!` script's
   descriptor is kept open across the start, because its interpreter reads it back through `/dev/fd/N`;
   the launcher then holds a read-only descriptor of its own script, and a script's `$0` is that name.
7. **macOS and Windows start the resolved path by name.** macOS has no `fexecve`; Windows could hold the
   file open deny-write, but no witness for that exists yet, so it is not built and not claimed. The window
   between the hash and the start stays open there to someone who can write the launcher's directory, and
   `DEPLOYMENT.md` says so. Nowhere does the digest stop an in-place change by someone who may write the
   file itself; the pin refuses the next run.
8. **Not in this slice: the attestation statement binding the launcher's digest.** Binding means an
   attester vouching for a digest it measured itself; a statement that echoes the host's own digest back
   says nothing. RW 4.28 keeps it, with that question.

## D-V2-70 — TERMINAL-TEXT-1: a program's strings are shown escaped wherever `delulu` prints them for a person — TAKEN (head chef, 2026-09-29, under the owner's delegation)

1. **Witnessed on `3489b57`, found while scoping RW 4.32.** A PURE program — no effect in its row, no
   grant — wrote escape sequences to the operator's terminal: `assert_eq("\u{1b}]0;PWNED\u{7}\u{1b}[2K\rsandbox:
   forged", "x")` set the window title, erased the line and printed a forged host line, in a plain run (the
   renderer) and under `--sandbox` (the guest's own fault line). The same bytes reached the terminal through
   a refusal quoting the program's path (DL0703, twice — in the message and in the `--grant` it suggests),
   through a test's name and failure in `delulu test`, and through a quoted source line. A terminal that
   honours OSC 52 would have let the program set the clipboard. "A program can do nothing except what it was
   explicitly handed" — and it had been handed no console.
2. **Escaped at the points text is printed for a person, not where values enter messages.** Values reach
   messages in more places than a list keeps up with (the runtime alone formats a value into a fault at nine
   sites, and every refusal quotes a path), and a list misses the next one; the printers are few: the diagnostic renderer (message,
   labels, file names, quoted lines), the guest's fault line, `delulu test`'s lines, the REPL's fault line.
   `delulu_diag::terminal_safe` shows every control character but a line break or a tab — C0 and C1, and
   the characters that reorder or split a line (bidi controls, U+2028/2029) — as `\u{…}`; `terminal_line`
   escapes the line break too, for a line that must stay one; a quoted source line shows each as `?`, so
   the caret keeps its column. Text with nothing to escape is borrowed untouched, so no ordinary output
   changes.
3. **JSON is unchanged.** Its own escaping is exact (`\u001b`), and an agent reading `--json` gets the
   program's bytes as they were; a test holds that.
4. **Named residual:** the renderer keeps line breaks in a diagnostic's message — one checker message lays
   itself out on three lines (`delulu-check/src/plugin.rs`, a plugin signature mismatch) — so a path or a
   value with a line break can still begin a new line inside a diagnostic it caused. No control sequence
   reaches the terminal that way; a line that looks like another can. RW 4.34 keeps it. And a guest that
   has ESCAPED its interpreter still writes raw bytes to its standard error, which is the operator's
   terminal — RW 4.32's relay, still open.

## D-V2-71 — RW 4.32: the host relays a guest's standard error — escaped, marked `guest:` or `launcher:`, cut at 1 MiB — TAKEN (head chef, 2026-09-29, under the owner's delegation)

1. **Why.** After TERMINAL-TEXT-1 (D-V2-70) a guest's own lines were escaped, but its standard error was
   still the operator's terminal, inherited, and so was an external launcher's. A guest that escaped its
   interpreter writes raw bytes there. Witnessed on `6cd68c8` with a launcher standing in for one: control
   sequences raw, a forged `sandbox:` line unmarked, 3 MiB passed whole.
2. **The host reads it and prints it a line at a time**, each through `terminal_line`, prefixed with whose
   it is: `guest:` at L1, where the host started the guest itself; `launcher:` at L3, where the stream is the
   launcher's and carries the guest's inside it, and the host cannot tell the two apart. A prefix rather
   than a colour: it survives a log file, and it cannot be stripped by the text it marks.
3. **Bounded at 1 MiB** — more than the microVM console's 64 KiB, because at L3 the stream is also the
   launcher's own tooling, which may say more than a guest — and drained past it (a full pipe would stall
   the writer, and a stalled guest is a hung run), said once. A line longer than 8 KiB is cut there.
4. **The host waits up to two seconds, once the guest is gone, for the last line** — a guest's fault is its
   last word, and a host that exits at once can lose it. The bound is there because at L3 something the
   launcher started may hold the stream open indefinitely.
5. **Not in this slice: the microVM's console relay**, which still writes the guest's bytes raw (capped).
   Its change is witnessed only on the KVM job; RW 4.32 keeps it.

## D-V2-72 — RW 4.34: a message's continuation lines are indented, not escaped — TAKEN (head chef, 2026-09-29, under the owner's delegation)

A program's string with a line break began a line of its own at column 0 inside a diagnostic it caused —
witnessed on `41ac869`: an assertion's value put a forged `sandbox: the guest is confined` line and a forged
`error[DL0000]` line there. Escaping the line break (as a one-line context does) would flatten the one checker
message that lays itself out on three lines, and any multi-line error text a message quotes; so the renderer
keeps a message's line breaks and indents every line after the first by two spaces. No line a message carries
can then begin where the host's own lines begin. A slight change in layout: the plugin signature mismatch's
`manifest:`/`code:` lines are indented four spaces instead of two. JSON unchanged.

## D-V2-73 — RW 4.32, FRAME-DRIP-1: a peer DeluluLang does not trust owes each frame WHOLE within a bound — TAKEN (head chef, 2026-09-30, under the owner's delegation)

Every channel that reads such a peer bounded each READ and none bounded the frame, so a peer sending one byte
just inside the read deadline held a frame — and its reader — open for as long as it liked. Witnessed on
`dcf4fcb` twice: on the broker daemon, whose loop serves one connection at a time and whose comment promised a
dribbling client would be dropped after 5 s (a `Status` behind a client sending one byte every 2 s waited
23.7 s, its whole life — the operator's e-stop revoke waits the same way); and on a foreign worker's call (foreign
code answering one byte every 300 ms, each inside the 1.5 s call deadline, held the host past 20 s). IPC-1's fix of
2026-08-08 had bounded each read and recorded the indefinite hang closed; a dribble was never closed.

Decided: one reader, `delulu_runtime::channel::Within`, and a bound at each place:
1. **The host reading a guest** (`HostChannel::serve`, and the confinement report in `boundary.rs`): a frame
   begun is owed whole within `FRAME_DEADLINE`, 60 s, **from its first byte** — a guest may still be quiet
   between frames for as long as the channel's own read deadline allows (a guest computing).
2. **The broker daemon**: a client's whole request within 5 s **of its connection's acceptance** — the bound the
   loop always claimed ("a legit client sends its whole frame immediately after connecting").
3. **A foreign call**: the worker's whole reply within the call's deadline (60 s, or shorter by
   `DELULU_FOREIGN_CALL_DEADLINE_MS`) **from the call**; the bind handshake's within its 10 s.

The check is made around each read, so a frame is abandoned no later than its bound plus one read deadline — a
read in flight is not cut short, because the transports share no way to shorten one; the bound is finite and
stated rather than exact. No new code: a guest's slow frame ends the run in its own words ("the guest did not send
one whole frame within 60s"), told apart from a silent guest; a worker's is DL1409's "no reply from the worker
within N ms" (the worker is killed); the broker's is a dropped connection in its log. **Not changed:** the broker
still serves one connection at a time, so clients queued one behind another each take their 5 s — a same-user
process that can do that can also stop the broker (category 7, as IPC-1 recorded); the operator's `request` still
waits without a bound for a broker that accepted and never answered (the dead-man probe's `request_timed` does not)
— **both corrected by D-V2-75**: `request` was unbounded (REQUEST-HANG-1), and the probe's bound was per read
(PROBE-DRIP-1, found by the red-team pass on this decision).

## D-V2-74 — PS-E-04 on Windows: the launcher is held open, sharing reads only, from the hash until the run ends — TAKEN (head chef, 2026-09-30, under the owner's delegation)

D-V2-69 left Windows starting the resolved launcher by name, "not built and not claimed" until a witness existed.
**Witnessed on `34cfc38`** (`witness.yml` `36652740569`, Windows): the host hashed the launcher through the
standard library's `File::open`, which shares reading, writing AND deletion, then started it by path — B, renamed
over the pinned A the moment Windows said the host held A open, was what started: "the guest never confirmed its boundary, so it was not sent the program: it closed the channel first", exit 1 (B is `hostname.exe`, not a guest). Decided: `Launcher::resolve`
opens the file sharing reads only (`FILE_SHARE_READ`) and the `Launcher` — the handle with it — lives until the
run's report is written, so from the hash to the end of the run nobody can write the file, or rename or delete it
(a rename over it is a deletion): the path started names the bytes hashed, and on Windows the pin now also stops an
in-place change for the run's length. Starting it is a read, still shared. **Consequence:** a launcher some other
process already holds open for writing cannot be hashed — the run fails in words ("cannot be read to be hashed")
rather than hash a file in the middle of being written. macOS has no equivalent (no `fexecve`, no share modes): its
window stays open and `DEPLOYMENT.md` says so.

## D-V2-75 — REQUEST-HANG-1 and PROBE-DRIP-1: every round trip to the broker daemon owes its whole answer within a bound — TAKEN (head chef, 2026-09-30, under the owner's delegation)

Invariant 27 asks that an unreachable broker fail every effectful op "fast (bounded, never a hang)". Two ways it did
not, each witnessed on `9bf09b9`: **REQUEST-HANG-1** — `brokerd::request`, which every daemon-mode custody op
(`BrokerClientCustody`) and every operator command uses, the e-stop's `grants revoke` among them, read the answer
with no bound at all: against a broker that accepted and never answered it waited the acceptor's whole 40 s hold.
**PROBE-DRIP-1** (the red-team pass on FRAME-DRIP-1's F1, Sonnet 5.5, re-run by the head chef) — the dead-man
probe's `request_timed` bounded each READ: an answer dribbled a byte every 250 ms kept it waiting 6 s against its 1 s
bound, and in the pass's end-to-end run an e-stop printed "revoked" while the arm, its probe fed a dribbled answer by
a process on the broker's socket, kept moving. D-V2-73 had said the probe "does not" wait without a bound: true only
of a silent broker.

Decided: `request` is `request_timed` with `REQUEST_DEADLINE`, 15 s — generous, since the daemon answers in
milliseconds and a client ahead of this one holds its one-connection loop for at most twice its 5 s bound; and
`request_timed` reads the whole answer through `Within::from_now`, so the bound is on the answer, not on each read
(abandoned no later than the bound plus one read deadline). A broker that does not answer is an error in the
operator's words — "the broker accepted the request but did not answer within 15s" — which every caller already
turns into DL1401 or its refusal; no new code. **Consequence:** a broker whose queue holds three silent clients
(5 s each) now fails an operator's command after 15 s instead of answering after them — fail closed, and the
command can be asked again; the dead-man probe parks the devices meanwhile, as it always did.

## D-V2-76 — REGISTRY-BOUNDS-1: `delulu-registry serve` bounds every connection, and one slow client delays no other — TAKEN (head chef, 2026-09-30, under the owner's delegation)

The registry's port is reachable by whoever can reach it, and reading it needs no token. It served one connection at
a time, read its request and header lines with no deadline and no cap, and allocated the body at whatever
`Content-Length` said before reading a byte. Witnessed on `76aa676` (`tests/serve_bounds.rs`): a client behind one idle
connection got no answer in 3.2 s; `Content-Length: 18446744073709551615` panicked the server (the next connect
refused); a 1 MiB request line with no end was read on and never answered. Decided: **`Limits`**, with defaults — 10 s
for any one read, 60 s for the whole request from its acceptance (`Within::from_now`, FRAME-DRIP-1's reader), 8 KiB for
a request or header line, 64 headers, a 16 MiB body (the sandbox channel's frame bound), 64 connections at once (one
past it is closed at once) — and `serve_with` so a test can shrink them. **Each connection is read on a thread of its
own; the requests are still handled one at a time under one lock**, because a publish reads and rewrites a package's
index. A panic stays on its connection's thread. Past a bound the client is answered in HTTP's own words — 413, 431,
400 — with the registry's existing code, DL1706; no new code.

## D-V2-77 — AUDIT-FIFO-1: an audit day log is read only if it is a regular file; `delulu-broker` gains `libc` on Unix — TAKEN (head chef, 2026-09-30, under the owner's delegation)

The audit chain's readers took each `YYYYMMDD.jsonl` by name with `read_to_string`: a FIFO named like a day log hung
`verify`, `tail`, `query` and the log's own open — so `broker start` — waiting for a writer that never came (the
red-team pass on FRAME-DRIP-1's F6; re-run by the head chef: `audit verify` and `audit tail` held until killed at 12 s;
witnessed in-crate on `2734ba9`, `verify` still waiting at 5 s). Decided: one reader, `read_day`, opens non-blocking and
refuses what it OPENED unless it is a regular file — ATTEST-FIFO-1's rule — in words ("is not a regular file, and an audit
day log must be one"), and the readers that treat an unreadable day log as empty (the log's open) do not treat this one
so: the log refuses to open rather than start a chain over a file it cannot read. `O_NONBLOCK` needs `libc`, which
`delulu-broker` did not depend on (ruling 1 keeps its list short); it is already in the tree through `getrandom`, so the
Unix-only line adds no crate. Under Miri the flag is left out: Miri's `open` takes a short list of flags and never makes a
FIFO, so there the open is the blocking one it replaced. `secrets.rs`'s store is read the same way and was not tried.

## D-V2-78 — PS-E-06: the audit chain exported as OCSF 1.8.0, each event carrying its record, so the export still verifies — TAKEN (head chef, 2026-09-30, under the owner's delegation)

`V2_OPENSHELL_STUDY.md` §4.6 proposed `delulu audit export --format ocsf` with `seq`, `hash` and `prev_hash` under
`unmapped.delulu`. Built (routine run 7), with three departures, each for a reason found while building it:

1. **The whole record rides in each event, not three of its fields.** `seq`, `hash` and `prev_hash` show a removed
   line (a broken link) but not an edited one: a hash can be recomputed only from every hashed field. So each event
   carries its record exactly as the chain holds it (`AuditRecord::to_value`), and every OCSF field is a pure function
   of that record and two labels stated once for the export (the product's version, the device's name).
   `delulu audit verify --ocsf FILE [--expect-start HASH]` recomputes each event and requires the line to be exactly
   it — an edited class, time, severity or message fails — and re-verifies the chain over the records with the same
   code as a reconciliation bundle (`Bundle::verify`). What it cannot prove it says: a removed LAST event leaves every
   link intact (P17-C1), so the head is printed for comparison with the source's `audit verify`; and the labels are
   the exporter's word, checked only to be the same on every line.
2. **A capability use is a Base Event (0), not File System Activity (1001) or HTTP Activity (4002).** The study
   mapped effects to those classes; the chain does not record a use's effect — `record_op("use", …)` carries the node,
   the argument and the decision, and the op is dropped (`validate.rs`). A path and a host are both strings, and
   choosing a class from the argument's spelling would be the export inventing a fact. The mapping by action, each class
   checked against the published 1.8.0 files (`class_uid` = category × 1000 + the class's own uid): any `deny`, a
   `break-glass`, `guard_bypass_on` and `guard_bypassed_use` → Detection Finding 2004 (High for the three special
   uses, Medium for a refusal); `issue`, `delegate`, `attenuate`, `redeem`, `renew`, `adopt` → User Access Management
   3005 Assign Privileges, `revoke` and `guard_permit_revoke` → Revoke Privileges (the privileges are the authority's
   effects, or "every effect held by" the node when the record carries none); `sandbox-launch` and `sandbox-death`
   (either decision — a failed program is not a detection) → Process Activity 1007, the guest named by its run's
   generation so the two pair; everything else → Base Event, activity Other, the record's action as its name. **The
   use's effect in the record is the next step** (its own slice: it changes the hashed shape of new records).
3. **The schema is checked at run time, never copied.** `scripts/ocsf-validate.py` reads OCSF's own class, object,
   profile and dictionary files — from a checkout, or file by file from the tag on raw.githubusercontent.com — and
   checks each event: the class's uid and category, `type_uid`, every required attribute with inheritance, no undefined
   attribute (recursively), each object's required attributes and `at_least_one`/`just_one` constraints, types and
   enums. Nothing of OCSF is in the repository (as for OpenShell, D-V2-52).

Also: a chain that does not verify is not exported (exit 1, the DL1405 shown); `--since SEQ` exports from that seq on,
and verifies with `--expect-start` = the previous export's head; the device label defaults to the machine's name
(`gethostname`, `COMPUTERNAME`), and `--device-name` names it. No new `DL` code — a broken export is DL1405, as a
broken chain is. The audit holds no secret bytes and no query string (a network use records the host,
`interp.rs::custody_op_for`), and the export adds nothing a record does not hold.

## D-V2-79 — AUDIT-TEXT-1: what a program or an agent wrote is shown escaped where a person investigates or approves — TAKEN (head chef, 2026-09-30, under the owner's delegation)

TERMINAL-TEXT-1 (D-V2-70) escaped a program's strings on the surfaces that print them LIVE. Two surfaces print them
LATER, from storage, and stayed raw. **`delulu audit tail` and `audit query`**: a use's record names its argument — a
path the program chose inside its grant — and `render_audit_record` printed it as stored. Witnessed on `b50bb36` through
a real leased run: a program wrote `./out/ESC]0;PWNED BEL ESC[2J ESC[31mFORGED.txt` inside its `./out` grant (an
ordinary, allowed use), and the investigator's `audit tail` set the terminal's title, cleared the screen and turned the
text red; a file name can hold a line break, so the witness forged a whole `seq 9 … revoke allow` record on that screen.
**`delulu guard pending`**: the owner's decision surface printed each request's `why` — the requesting agent's own text —
raw; the witness's `why` erased the real request's line (`use=[fs_write:*]`) and printed a request for `fs_read:./data`
in its place. Decided: every field of both listings is escaped onto one line with `terminal_line` (the record's action,
decision, actor and target; the request's id, status, node, uses and why). The chain, the queue and `--json` keep the
bytes exactly — the escaping is only where a person reads. No new code. **Applied the same run to `grants tree`, `list`
and `inspect`** (RW 4.42, witnessed on `634e9f0`): the broker's tree renderer escapes each field of its node's one line,
the CLI its list line and inspect's fields.

## D-V2-80 — PS-E-06's remainder: a use's audit record names its effect, and the export maps it — TAKEN (head chef, 2026-09-30, under the owner's delegation)

D-V2-78 left a capability use a Base Event: `Broker::check_use` recorded the node, the argument and the decision, and
dropped the op — so a path read, a path written and a host asked were the same record. Decided: every use record
`check_use` writes — `use` (allowed or refused, and for a node that does not exist), `guard_permit_use`,
`guard_bypassed_use`, `guard_warn`, `guard_block` — carries `{"op": <the op's wire name>}` in the record's optional
payload slot, the field a revocation's bound already rides in under a self-describing key (spec §11, chunk-5 deviation 3).
**Not a new field:** a new key in the hashed body would make every record written from now on unverifiable by an older
`delulu`; the payload slot is already hashed and already optional, so old and new records verify under either binary, and
a record written before this names no effect. The export then maps an allowed use by the effect its record NAMES:
`FsRead` → File System Activity (1001) Read, `FsWrite` → 1001 Update (the file object carries the path the use was decided
on, its name, type Unknown), `Net` → HTTP Activity (4002) Get with the host as `dst_endpoint.hostname` — `Get` because the
language's one network primitive is `http.get` (`custody_op_for` maps nothing else to `Net`); a method the language gains
must reach the record before the export names it. A refused use stays a Detection Finding, and its finding's `types` now
name the effect too. Other effects (`Actuate`, `ForeignBind`, `Declassify`) and every record without an effect stay Base
Events. Only synchronous-class uses are recorded (invariant 26), so an allowed read that no Guard gates leaves no record
and no event — the export shows what the chain holds.

## D-V2-81 — SCOPE-HIDDEN-1: a minting site the source does not show leaves its kind's placeholder beside the literals — TAKEN (head chef, 2026-09-30, under the owner's delegation)

`required_grants` promised (NE-10) that where a scope is visible in the source the flag is spelled with it and where it
is not the flag carries its placeholder — and `authority`'s scope walk printed a placeholder only when it had found NO
literal of a kind. Found building PS-E-05 (whose export needed to know whether a program's literal scopes are the whole
of what it asks for): a program with `root.fs_read("./data")` and a helper `fn helper(r: Root)` calling
`r.fs_read("./secret")` was reported as needing `fs.read=./data` alone; so was one beside `root.fs_read(pick(2))`.
Decided: the walk marks a KIND hidden when a minting site's scope is computed, or when its receiver is not the bare
name `root` (only `Root` has the minting methods, per the primitive table, so such a receiver is a `Root` under another
name); `required_grants` then adds the kind's placeholder beside the literals. **The literal at a renamed receiver is
not attributed** — the walk runs on the parsed module, not the typed one, and a guessed literal would tell an operator
to grant a specific scope on inference; a placeholder tells them a scope exists that they must decide. The placeholder
is added only for a kind the checker's report names (kinds are static and sound, P16), so it never invents a
capability. `requested_scopes` keeps its meaning: the literals at `root`'s own sites. Witness
`cli::a_scope_the_source_does_not_show_leaves_its_placeholder_beside_the_literals` (a renamed receiver, a computed path
and a computed host beside literals, and an all-literal control); mutants M58–M61 red. No program in the repository's
corpus changes (0 of 251 have a mixed kind).

## D-V2-82 — PS-E-05 (a): `sandbox policy --format openshell` emits the wall from the program's authority, never wider than it and the grants — TAKEN (head chef, 2026-09-30, under the owner's delegation)

The study (§4.5 (a), D-V2-53) designed the export; building it settled nine things.
**(1) The grant's scope, not the source's literal, is what is emitted** — for a capability KIND the authority report
names (kinds are static and sound, P16); a kind it does not name leaves the grant `omitted`. Emitting the literals the
source shows would be narrower, but SCOPE-HIDDEN-1 (D-V2-81, found here) showed the literals are not always the whole
ask, and a wall narrower than DeluluLang's own grant breaks a program that DeluluLang would run. **(2) Exact refusal
set:** `net.special` (OpenShell never authorizes loopback, link-local or unspecified destinations; a private one is not
mapped yet), `actuator`/`sensor`/`compute` (no device model), `plugin`/`foreign.*`/`exec.native` (code whose files this
export does not map yet), `fs.write=/` (OpenShell refuses it), a wildcard under a top-level domain (OpenShell refuses it,
and `**` would widen it) — exit 1, no document. **(3) Unrepresented, listed:** console, clock, rand, declassify, a
secret (by name, never its value — P9-04 is endpoint-bound secrets), the budget. **(4) Narrowed, listed:** port 443
only (DeluluLang fetches `https://` on any port a URL names). **(5) One method, written out:** `GET` on `/` and on
`/**` (a whole-segment `**` needs one segment, so both are named); the `read-only` preset also allows `HEAD` and
`OPTIONS`. **(6) Hosts:** DeluluLang's `*.x.y` matches one label or more and never the apex (`prim::host_matches`) —
OpenShell's `**.x.y`; and a host (or suffix) DeluluLang would never match — not in `egress::parse_target`'s canonical
form, `EXAMPLE.com`, `example.com.` — grants nothing, so it is omitted, never lowercased for a wall that matches
case-insensitively (EXPORT-CASE-1, found before `master` moved); an IPv6 host refuses (not mapped yet). **(7) Paths are the sandbox's:** absolute; a relative one is joined to `--workdir` (this machine's
working directory names a path on the wrong machine) and refused without it; `..` is REFUSED, never collapsed, because
the kernel resolves it after following a link; the runtime's own read-only paths (`/usr`, `/lib`, `/etc` — three of
OpenShell's baseline) and the program's file are the only paths no grant names, and both are reported as such;
`include_workdir: false`; `landlock.compatibility: hard_requirement`. OpenShell's baseline additions for a policy with a
network rule (`/tmp` read-write among them) are named in a note, not suppressed — suppressing them waits for (b)'s real
run to show what `delulu` needs. **(8) Identity:** `process` names `sandbox` (OpenShell's own unprivileged identity) or
`--run-as` numeric ids; root in any spelling is refused. **(9) One source:** the YAML document is written FROM the JSON
policy the envelope carries, every string double-quoted, so the two cannot differ (a mutant adding a method to the text
alone passed the unit gate while they were two code paths). The binary is `--binary`, default `/usr/local/bin/delulu`
(the repository's `Dockerfile`). **(10) Scope of the flags:** `--format`, `--grant`, `--workdir`, `--binary` and `--run-as`
belong to `sandbox policy` alone and are refused on every other verb (documenting them for `sandbox` let the dispatcher
pass them to `status` and `probe`, which ignored them — found by the suite); a Windows spelling of a path (a backslash, a
drive letter) is refused by name, since it names a path on the machine running `delulu`, never one in the sandbox. **The falsifier the study named** — OpenShell's own prover answering `within_boundary`
against a boundary written by hand and `exceeds_boundary` against one missing a granted host — is
`scripts/openshell-prove.sh` in `.github/workflows/openshell.yml`, which also requires each widening of the export (a
preset, a writable `/tmp`, another binary) to be caught and an unknown field to be an error. **Not built here:** (b), the
guest inside an OpenShell sandbox at L3, and the study's runtime witnesses (a denied `curl`, a granted/ungranted pair).

## D-V2-83 — PS-E-05 (b): a guest runs under an outer wall's syscall filter only when its launcher declares that wall, and says so — TAKEN (head chef, 2026-09-30, under the owner's delegation)

**The finding it answers (routine run 8, `openshell.yml`'s runtime job):** inside an NVIDIA OpenShell sandbox the guest's
own Landlock layer took hold, and its `seccomp` filter was refused — `Error calling seccomp: Operation not permitted`.
OpenShell's sandbox installs a seccomp user-notification listener and a final filter on every process it runs, and that
filter answers `seccomp` with EPERM. So the guest failed closed and the host never sent the program: PS-E-01's rule held
inside someone else's wall, and the study's (b) — the guest inside OpenShell at L3 — could not run at all.

**Options weighed.** (i) Keep failing closed, and run only `delulu run` inside OpenShell (what (a) already does): the
host, its grants, secrets and audit chain would then live inside the sandbox too, which is (a), not (b). (ii) Install the
guest's filter another way (`prctl(PR_SET_SECCOMP)`), walking round the outer wall's denial: it would only ever narrow
the guest (a stacked filter cannot loosen one in force), but it defeats a denial another system made on purpose, and
stops working the day that system closes it — rejected. (iii) Accept an outer filter AUTOMATICALLY when the guest's own
is refused: silent in the one place a sandbox must not be — a host running inside a container that forbids nested
filters would start L1 guests without their filter, while L1's report claims properties that filter is part of —
rejected. (iv) **Taken:** the LAUNCHER declares the outer wall, and the guest checks what it can.

**The rule.** (1) An external launcher's guest may be started as `delulu __guest --stdio-pipes --outer-syscall-filter`.
The declaration is the launcher's, like the wall: a guest the host starts (`--stdio`, a channel directory, the microVM's
vsock) refuses it, and after `--stdio-pipes` the guest takes that word once and nothing else (a word it would not act on
is refused, not ignored — the lesson of run 8's `--grant`). (2) Declared, the guest still tries its own filter first; it
never skips one it can install. (3) Only if that is refused with **EPERM** — a filter's answer: `no_new_privs` was set just
before, so the kernel's own permission check passes, and without it the kernel answers EACCES — **and** the kernel reports
a filter in force on the guest (`/proc/self/status`: `Seccomp: 2`, and `Seccomp_filters` ≥ 1 where printed), the guest
runs under that filter; any other answer (ENOSYS, EACCES, EINVAL, a kill) fails closed, declared or not. (4) It says so
twice: on the operator's screen ("the guest's own syscall filter was refused by a filter already in force on it — the
outer wall its launcher declared stands in for it; the guest's own filter is NOT installed") and in its confinement report,
where the checked word **"an outer syscall filter, not its own"** replaces its filter's four ("no new programs", "no
debugger", "no namespace or module tricks", "no sockets but the channel"). Its Landlock words are unchanged — that layer
still applies. (5) The word establishes nothing: it matches no posture row, so it moves no property; at L3 every property is
`unknown` anyway and the word is kept as `guest_reported` and in the death record's `guest_words`. (6) The host refuses the
word from a guest it started itself (`boundary.rs`, before the program is sent): its own filter is part of the boundary an
L1 or L2 host measures and claims. (7) On a guest that applies no filter of its own (macOS, Windows) the declaration means
nothing and is refused.

**Witnesses.** `crates/delulu/tests/sandbox_outer_filter_cli.rs`: the outer wall simulated as it looks from inside — the
test installs on `delulu run` a filter answering `seccomp` with an errno, which the launcher and the guest inherit. Red on
`e936ea5` (the declared guest failed closed with OpenShell's exact words; the declaration was not refused); green after:
undeclared → fails closed, nothing sent; declared → runs, the word in place of the four, level 3, every property `unknown`,
and an ungranted effect still DL0703; ENOSYS, EACCES, EINVAL → fail closed though declared; declared with no outer wall →
its own four words; the declaration refused off an external guest and with any other word. Unit: the host refuses the
word from a host-started guest and confirms it from an external one; the word moves no posture row and no property;
`filter_in_force` over eleven spellings. **Mutants:** M75 (any errno), M76 (undeclared), M78 (the parser always yes), M79
(declared skips an installable filter), M80 (the host's check removed), M81 (extra words accepted), M82 (the declaration
taken off an external guest), M83 (the word a posture needle) red. **M77 survives, and why:** bypassing the in-force check
changes nothing any kernel we can run produces — EPERM from `seccomp` after `no_new_privs` comes only from a filter in
force — so the check guards an LSM or a future kernel, and its parser is pinned by the unit test (M78). Recorded, not
hidden. **Read inside OpenShell** by `scripts/openshell-runtime.sh` step (5): (5a) undeclared fails closed, (5b) declared
runs the program at level 3 with the word — the run that reads it is in `V2_LOG.md`.

**Amended — routine run 10, 2026-10-04 (head chef, under the owner's delegation): the kernel answers rule (3), not
`/proc`.** Routine run 9 ended with `f446bfa` on its harness branch alone, and its reading inside a real OpenShell sandbox
(`openshell.yml` `36738997626`) was **red**: (5a) held, but (5b)'s declared guest failed closed too — "`/proc/self/status`
shows no filter in force, so none stands in for it". The sandbox's effective policy (the same run's step (4)) names no
`/proc`, and OpenShell adds none, so under its Landlock the guest could not READ its status, and the unreadable file
(`unwrap_or_default`) read as "no filter". The rule held — it failed closed — but (b) could never run where it was built
to. **Changed:** rule (3)'s in-force check is the kernel's own answer, `prctl(PR_GET_SECCOMP) == 2`, which needs no path:
where no filter is in force nothing can answer that call in the kernel's place, so a 2 is never invented; an error is
no. Everything else in the rule stands. **Witnessed:** the test's simulated wall was the filter alone, so `/proc` stayed
readable and the test could not see OpenShell's failure; it now runs every expectation under two walls — the filter, and
the filter with a Landlock layer under which nothing in `/proc` can be read (checked first: `cat /proc/self/status`
refused inside it, a file elsewhere read). Under the second, `f446bfa`'s code went red with OpenShell's exact words; green
after. The parser's unit test became `a_filter_in_force_is_the_kernels_own_answer`: `PR_GET_SECCOMP` agrees with this
process's `/proc/self/status` wherever the suite runs. **Mutants:** M84 (the check always no) and M86 (strict mode taken
for a filter) red under both walls; M85 (the check always yes — run 9's surviving M77) red now, on the unit test; M87
(the wall's Landlock layer dropped) and M88 (`/proc` not skipped) red on the hidden-`/proc` witness's own sanity check.

## D-V2-84 — The nightly red four times: every platform's crates fetched before a suite; Wasmtime 48.0.5 for RUSTSEC-2026-0325 to -0327 — TAKEN (head chef, 2026-10-04, under the owner's delegation)

1. **What was red.** Every nightly since routine run 8 left (`e936ea5`, its push run `36716667440` green): 2026-10-01
   `36844480022` — `supply-chain` alone; 2026-10-02 `36988889261`, 10-03 `37110949841`, 10-04 `37191610614` — `supply-chain`
   and all four test suites (Linux x64, Linux arm64, macOS, Windows). No push came in those days, so `master`'s own runs
   stayed green while the next push would have gone red.
2. **The suites: a test that passed only while the cache did.** `egress_features` asks `cargo metadata --offline
   --locked` for the RESOLVED graph, which reads every platform's crates; a build downloads only the runner's. A warm
   `Swatinem/rust-cache` held them — its own save step resolves the whole graph — so the test passed for as long as the
   cache came back. **Rust 1.99.0 went stable on 2026-10-01**: `dtolnay/rust-toolchain@stable` installs it beside the
   pinned 1.96.1, both are in rust-cache's key, every nightly from 2026-10-02 started with "No cache found", and
   `cargo metadata` failed (`failed to download core-foundation v0.10.1 … --offline was specified`). A red job saves no
   cache, so it could never recover by itself. **Witnessed in the VM:** a fresh `CARGO_HOME` with only what `cargo test -p
   delulu-runtime` downloads — the same failure (`failed to download addr2line v0.26.1`); after `cargo fetch --locked` in
   that home, 2 passed. **Taken:** the `test` and `arm64` jobs run `cargo fetch --locked` before the suite — the
   precondition the VM already writes down (routine run 1), now on the runners too. Not taken: pinning the action to
   1.96.1 to keep the cache warm — it would hide the cold path again, and the next cold cache (eviction after seven
   days, a lockfile change) would find the same test; the fetch makes a cold runner correct.
3. **The supply chain: three Wasmtime advisories and a yanked crate.** RUSTSEC-2026-0325 (mis-typed tag imports —
   exceptions), -0326 (GC rooting across `try_call` — GC and exceptions), -0327 (the component model's async-lifted
   callbacks) against wasmtime 48.0.3; and `yoke-derive 0.8.3` yanked. **Reachability, read:** none reaches DeluluLang —
   `harden_wasm_features` turns the component model, GC and exceptions off (`delulu-wasm/src/host.rs:66-78`, D-V2-67 and
   D-V2-68's narrowing), so the upgrade is the gate's, not a live hole; never an `ignore` all the same. **Taken:**
   wasmtime 48.0.3 → **48.0.5** (the 48 long-term-support line's newest; the advisories' fix is 48.0.4 — 49.0.2 is the other
   range, one major step more), and `yoke-derive` 0.8.3 → 0.8.4. Lockfile only: no manifest or source line changed.
   Witnessed with cargo-deny 0.20.2: `cargo deny --all-features check advisories` exit 1 naming all four, then exit 0;
   `check` (advisories, bans, licences, sources) ok.
4. **Verified:** clippy clean; the full suite alone 2,094 passed, 0 failed, 15 ignored (156 binaries), cargo exit 0;
   the two-engine differential by hand (wasmtime changed). The push run is the cold-cache witness on the runners — the
   cache key is still new, so it starts cold.

## D-V2-85 — PS-E-05 (b): the guest is reached over `ssh` through OpenShell's gateway; `sandbox exec` cannot carry a conversation — TAKEN (head chef, 2026-10-04, under the owner's delegation)

1. **Measured, after a hypothesis was refuted.** With the in-force check fixed (D-V2-83's amendment), the declared guest
   ran under OpenShell's filter and its confinement report never reached the host (`openshell.yml` `37233187076`). The
   first guess — the CLI's line-buffered standard output — was refuted by timing each byte (`37233707240`): output crosses
   `openshell sandbox exec` as it is written, newline or not. The input does not: **the command starts only once its
   standard input ends** — the first line of a command fed `x`, 3 s, `y`, 3 s came at 6.0 s, with `--tty` too
   (`37234148291`). v0.1.2's `exec` reads its input to the end before it runs anything: a command, not a conversation.
2. **Taken:** the external launcher reaches the guest over `ssh`, through the entry `openshell sandbox ssh-config NAME`
   prints (`ProxyCommand openshell ssh-proxy …`): the same sandbox, its policy and its filter, and the input streams (a
   byte at 0.1 s, the next at 3.0 s). `ssh -T`: no terminal, so the channel's binary frames pass unchanged. The guest's
   words are DeluluLang's own (`$DELULU_GUEST_ARGS`), then the launcher's declaration (`--outer-syscall-filter`).
3. **Not taken:** a line-ended framing of the external channel (a newline after every frame) — built for a cause that
   was not there; it would change the wire and fix nothing here. Nor a network channel from the sandbox to the host — a
   network rule the guest's sandbox does not need. Nothing of OpenShell is changed or copied: the CLI's own command is
   the transport.
4. **Read green inside a real OpenShell sandbox** (`37234907676`, `fc676d1`): (5a) undeclared — the guest's `seccomp`
   refused, it failed closed, the host never sent the program (exit 1); (5b) declared — exit 0, the program's granted
   read performed by the host and its line printed, the report level 3 `external` with the guest's words ["no file
   writes", "reads only from the system paths", "no TCP bind or connect", "an outer syscall filter, not its own"], none
   of its own filter's four, every property `unknown`, no host guarantee. The recipe is in `docs/DEPLOYMENT.md`.

## D-V2-86 — RW 4.35, REPLY-HOLD-1: the broker daemon's reply is owed within a bound, as its request is — TAKEN (head chef, 2026-10-04, under the owner's delegation)

1. **The defect** (the red-team pass on FRAME-DRIP-1, F2; re-run by the head chef on `68b8888`): the serve loop serves one
   connection at a time and wrote each reply with no bound, so a same-user client that asked for an answer larger than
   the transport holds and never read it held the loop — every custody operation, the operator's e-stop revoke among
   them, waited behind it. `GuardRequest` needs no credential and takes any `why`, so `GuardPending`'s answer can be made
   as large as a frame. **Witnessed red in the VM (Linux):** a 4 MiB `why`; a client that asks for `GuardPending` and
   never reads, and one that reads 32 KiB every half second (each write progresses; the whole would take a minute) —
   `broker status` behind either got no answer within its 15 s bound.
2. **Taken:** the whole reply is owed within 5 s of the request being read (`SERVE_WRITE_WITHIN`, the request's own
   `SERVE_READ_TIMEOUT`), each write within 1 s (`SERVE_WRITE_EACH`); past either, the connection is dropped in words in
   the broker's log. `channel::Within` — D-V2-73's whole-frame bound for reads — bounds a whole WRITE too (its error
   `FrameNotTaken`, "did not take one whole frame within …"), and offers the transport at most 64 KiB at once
   (`WRITE_CHUNK`): **on Linux a socket's `SO_SNDTIMEO` applies to each buffer a write waits for, not to the call**, so one
   4 MiB `write` to a peer that kept freeing a little room stayed in the kernel for a minute, out of reach of any check
   between writes (found by the slow reader: the first fix passed the silent client and failed it). Unix: `SO_SNDTIMEO`
   on the accepted socket. A legitimate client reads its answer at once, so neither bound is felt.
   **Windows stays open, and why, from its runner** (`witness.yml` `37236818679`, a first Windows design): putting the
   connected server handle in `PIPE_NOWAIT` for a bounded write failed (`SetNamedPipeHandleState`: win32 error 231), and
   even with the reply abandoned the loop stayed held — the connection's `Drop` (and `flush`) call `FlushFileBuffers`,
   which on a pipe waits until the client has READ everything, there so that `DisconnectNamedPipe` discards no reply. So
   that design was withdrawn whole; Windows' transport is exactly as before, and the two witnesses are `cfg(unix)`. The
   Windows half needs the write AND the flush bounded — a writer thread cancelled with `CancelSynchronousIo`, or an
   overlapped server pipe — witnessed on the Windows runner first (RW 4.35 stays open for it).
   **Windows closed the same run (`4d4d033`), with a design that switches no mode and starts no thread:** the pipe's own
   accounting (`NtQueryInformationFile`, `FilePipeLocalInformation`: `WriteQuotaAvailable` against `OutboundQuota`) says how
   much the outbound buffer can take and whether it has drained. With a bound, `write` waits for room — polling, as `read`
   polls `PeekNamedPipe` — and writes no more than fits (buffered at once, so `WriteFile` never blocks); `flush` waits for
   the buffer to drain (then `FlushFileBuffers` returns at once); each for at most the bound; and `Drop` flushes only a
   reply its reader has taken. The witnesses run on every OS again: `witness.yml` at `4d4d033` — Windows `37238378527` and macOS `37238381140` green (Windows: 140 unit tests, the two witnesses among them, its log naming both drops — "the reader took nothing" and "did not take one whole frame within 5s" — and `broker_cli` 3, `estop_cli` 5, `guard_cli` 2, `guard_e2e` 3, `secret_verify_cli` 1).
3. **Not taken (yet):** caps on a request's field lengths and on `List`/`GuardPending` answers (the row's other remedy) —
   they would shrink the largest answer, not bound a peer that reads nothing; the bound is what closes the hold. RW 4.40
   (silent connections queue behind one another) is a different shape and stays open.
4. **Witnesses:** `brokerd::tests::a_client_that_never_reads_its_answer_cannot_hold_the_serve_loop` and
   `…_reads_its_answer_slowly_…`, red before, green after on Linux, green on macOS; **mutants** M89 (the whole-reply bound removed) and
   M91 (the reply offered in one write) red on the slow reader, M90 (no per-write deadline) red on the silent client.
   Read on the other runners before `master` moved: macOS `37236820414` green (both witnesses); Windows `37236818679` red — the first Windows design, withdrawn (above).

## D-V2-87 — PS-E-01: at level 3 a required property is met only by a claim of the pinned attester that names it; the state stays `unknown` — TAKEN (head chef, 2026-10-05, under the owner's delegation)

1. **The gap** (`V2_OPENSHELL_STUDY.md` §4.1, "Open: … attesters' claims as properties"): §4.1 says an unattested
   external launcher is `unknown` throughout "and satisfies a requirement only through an attester's claim that names
   it", and since D-V2-59 `hostile-agent`'s refusal (DL1408) has told the operator "Ways out: … an external launcher
   whose attester vouches for it". No attestation could be that way out: `boundary::properties` answered every level-3
   property `unknown` whatever the pinned key had signed. **Witnessed red on `2cb3f87`** (`sandbox_attest_cli`, the new
   witness): `--sandbox-profile hostile-agent`, `external:` the reference attester in front of the guest,
   `--require-attestation` its key, the statement verified and claiming all five properties by name — refused DL1408,
   every property "an external launcher's wall: DeluluLang measured none of it", the refusal naming the way out it had
   just refused.
2. **Taken:**
   - **A claim names a property** when its text before a `:` — or the whole text, with no `:` — is, trimmed, exactly one
     of the five names a run reports (`boundary::PROPERTIES`); the rest, trimmed, is how the attester says it holds
     (`attest::named_property`). Exact and case-sensitive: `Egress_Confinement`, `egress-confinement`,
     `egress_confinements`, `no egress_confinement` are free text and vouch for nothing — a near-miss fails closed, as an
     unnamed property does. Several claims naming one property are kept in the order signed (`; `); a bare name says
     "the attester did not say how".
   - **The statement does not change** (`delulu-attestation-v1`): naming is a reading of the `guarantees` it already
     signs, so every document an attester signed before still verifies, and only a profile that requires a property
     (`hostile-agent`) reads it. The reference attester needs nothing new (`--guarantee PROPERTY:HOW`); its help says so.
   - **The report:** the property's `state` stays `unknown` — DeluluLang measured none of an external wall — and the
     claim is kept beside it, `properties.<name>.attested = {attester, by}`; a property the attester did not name says
     so in its `why` (the attester, and the claim that would name it). Not a fourth state: schema 1 keeps `state`'s three
     values (`STABILITY.md`: fields are added, never changed in meaning), so a consumer that reads only `state` reads the
     conservative answer. The schema's `property` gains the optional `attested` (`attested_property`).
   - **What meets a requirement** (`boundary::met`): `established`, or an `attested` claim — which `properties` writes at
     level 3 only. A run the host measured takes no attester's word (and `--require-attestation` is refused below L3
     before anything runs). The refusal is computed once, in `Opened::confirm`, from the same answer the report carries.
   - **DL1408's words:** the refusal says what meets a property ("established by this host or, at level 3, vouched for by
     name by the attester the run pinned") and how to vouch (`--require-attestation KEY`, a claim `PROPERTY: how` for
     each); `delulu explain DL1408` says the same.
3. **Not taken:** a fourth state (above); a structured statement field (`properties: {name: how}`) — a new format
   version for what a convention over signed text already says, and a second place for the same claim; case-insensitive
   or fuzzy matching (a security decision on an unnormalized spelling is the trap §11.4 names — here the exact spelling is
   the only one, and every other fails closed). **Still open in E-01:** `contained`'s required set (macOS's gaps); in
   E-04, a launcher digest the attester measured itself, bound into the statement — that one does need a new version.
4. **Witnesses:** `sandbox_attest_cli::an_attester_that_vouches_for_each_property_by_name_meets_hostile_agent_…` (all
   five vouched: served, the program's effect happens, each property `unknown` with the claim beside it, the report
   valid against `delulu schema sandbox`; each of the five left out, and four near-misses, refused DL1408 naming the
   missing property, "did not vouch for", the program never sent); `boundary::tests::at_level_3_only_an_attesters_named_
   claims_meet_a_required_property` (the typestate: all five at L3 confirmed, four refused, all five at a measured run
   refused); `boundary::property_tests::an_attesters_named_claim_is_kept_beside_the_unknown_at_level_3_and_nowhere_else`;
   `attest::tests::a_claim_names_a_property_only_by_its_exact_name`. **Mutants** M92 (`met` ignores `attested` — the
   defect), M93 (a measured run takes the attester's claim — first written as a no-op the code after it overwrote, and
   rewritten), M94 (case-insensitive names), M95 (a name found inside the text), M96 (a claim answers every property),
   M97 (the state moved to `established`): all red. Read on the other runners before `master` moved: macOS ``37247548752` green (153 unit tests, `sandbox_attest_cli` 8, `sandbox_confirm_cli` 13)`,
   Windows ``37247550461` green (142 unit tests — the `cfg(unix)` typestate test not among them — `sandbox_attest_cli` 5, `sandbox_confirm_cli` 8)`, Linux arm64 ``37247552077` green (172 unit tests, `sandbox_attest_cli` 8, `sandbox_confirm_cli` 14)`.

## D-V2-88 — RW 4.37, ANSWER-HOLD-1: the host's answer to a guest is owed whole within the frame deadline, and every guest socket has a write deadline — TAKEN (head chef, 2026-10-05, under the owner's delegation)

1. **The defect** (the red-team pass on FRAME-DRIP-1, F5, recorded as code reading only): `guest.rs`'s `open_channel` gave
   a Linux guest's socket, a microVM's and a guest's reached by name (macOS's) a read deadline and no write deadline, and
   `HostChannel::serve` wrote each answer with no bound. A guest that asks for a large answer — a file's text, up to the
   channel's 16 MiB frame, far more than a socket buffers — and then reads nothing, or a little at a time, holds its host
   for as long as it likes, and no default limit ends it: a guest that is not reading spends no processor time, and the
   wall-clock limit is off by default. An escaped interpreter is exactly such a guest. **Witnessed red on `5bd76aa`**
   (`guest::tests::a_guest_that_stops_reading_an_answer_cannot_hold_its_host`, the host's end configured by the same helper
   `open_channel` uses): the host wrote until the guest hung up, 20 s later. Named **ANSWER-HOLD-1** — REPLY-HOLD-1's shape
   (D-V2-86) on the sandbox's own channel.
2. **Taken:**
   - **Each write is bounded by the transport:** `open_channel` configures every guest socket through one helper —
     `bounded` (a Linux guest's and a microVM's `UnixStream`) and `bounded_by_name` (`broker_transport::Connection`: a Unix
     socket's `SO_SNDTIMEO`, a Windows pipe's write that waits for room within the bound, RW 4.35) — with the channel's
     deadline on every read AND every write. Windows' contained guest's pipe (`HostPipe`) already waited for each write
     within the deadline.
   - **The whole answer is bounded by `serve`:** written through `channel::Within::from_now(…, frame_deadline)` — 64 KiB per
     write, and the frame deadline (60 s) between them — because a guest that reads a little at a time lets every single
     write progress. A write the transport gave up on is said the same way (`FrameNotTaken`).
   - **In words:** "the guest did not take the host's answer whole within 60s, and the channel's deadline ended the run" —
     never "said nothing", which is a silent guest's.
3. **Not taken:** a separate, shorter answer deadline (the frame deadline is the channel's one number for "a frame owed
   whole", for either direction); bounding the program's own frame (`Confirmed::send_program`) beyond the transport's write
   deadline, which now bounds it against a guest that never reads (a guest that sips its own program is not a hold anyone
   but itself pays for — it never runs).
4. **Witnesses:** the guest test above — four cases: a socket pair and a by-name listener (the macOS and Windows path),
   each with a guest that never reads and one that takes 4 KiB every 100 ms of an 8 MiB answer — each ended within
   about a second of the 800 ms bound; `channel::tests::a_write_the_transport_gave_up_on_is_an_answer_not_taken`; the
   words test. **Mutants:** M98 (no write deadline on the socket pair) and M99 (none by name) red on the never-reading
   guest; M100 (`serve` writes unbounded) red on the sipping one; M101 (a transport's give-up not said as the answer not
   taken) red on the unit test — it survived the guest test, because with equal deadlines `Within`'s own check comes
   first, so the path got its own witness; M102 (the words) red. Read on the runners before `master` moved: CI
   `37249221745` dispatched at `608c5c4` — every job read green but the two long `miri-slow` jobs (check, broker), cancelled once the rest were read: `test` on Linux, macOS and Windows, arm64, `microvm` (the KVM runner — a microVM guest's channel goes through `bounded` now), `microvm-reproducible`, `heavy-gates`, `miri-slow` (syntax), lints and the rest; and `witness.yml` naming the witness green on macOS `37250364770` (all four cases, 5.8 s) and Windows `37250366783` (the two by-name cases, over a named pipe, 1.8 s).

## D-V2-89 — PS-E-04: an attestation can bind the launcher its attester measured — a v2 statement, refused for any other launcher — TAKEN (head chef, 2026-10-05, under the owner's delegation)

1. **The gap** (`V2_OPENSHELL_STUDY.md` §4.4: "an attestation statement binds the launcher's digest with the nonce (the
   statement's version bumps)"; RW 4.28's open item, "the attestation statement binding a digest the ATTESTER measured"):
   since PS-E-04 the host hashes the launcher it starts (`launcher_blake3`), and since PS-D-02 an attester signs a
   statement for the run — and nothing joined the two: a statement could not say which launcher its attester vouched
   for, so one written for the operator's image held for whatever file the launcher's word resolved to. **Witnessed
   absent on `396653b`:** the reference attester refused `--measure-launcher` (and a statement carrying any such field was
   refused as not a document).
2. **Taken:**
   - **`Statement.launcher_blake3`** (optional): the BLAKE3 of the launcher file the attester MEASURED itself, 64
     lowercase hex characters (any other spelling is refused by both sides). **The host gives the launcher no digest** —
     an attester that echoed the host's number back would bind nothing (§4.4's condition).
   - **A format for each kind of statement:** with the field, the document is `delulu-attestation-v2` and so is the first
     line of what is signed; without it, `-v1`, byte for byte as before (the field is skipped, so every v1 signature still
     verifies). A document whose `format` is not its statement's kind is refused (`WrongFormat`); dropping or changing the
     binding after signing fails the signature, so a v2 statement cannot be passed off as v1.
   - **The check:** after the signature, the pinned key and the nonce, a bound statement must name the launcher the host
     started (`Refusal::WrongLauncher`, "not the launcher it measured", naming both digests) — before the program is sent,
     recorded as every refusal is (`sandbox-attestation` deny, `sandbox-death`).
   - **The report:** `sandbox.attestation.launcher_blake3` (optional in the schema), equal to `sandbox.launcher_blake3`;
     the stderr line says the statement is "for the launcher it measured".
   - **The reference attester:** `delulu sandbox attest … --measure-launcher FILE` hashes `FILE` and binds it.
3. **Not taken:** the host requiring a bound statement (a flag beside `--require-attestation`) — `--launcher-digest`
   already lets an operator pin a digest it knows; this checks one the attester vouches for, and an operator who wants
   both passes both. A digest algorithm other than BLAKE3 (the host's `launcher_blake3` is BLAKE3, D-V2-69). **Still open
   in E-04:** macOS starts the resolved path (no `fexecve`).
4. **Witnesses:** `sandbox_attest_cli::an_attester_that_measured_the_launcher_binds_it_and_a_statement_for_another_file_is_
   refused` (the launcher started, measured: served, the report's two digests equal and valid against the schema; another
   file measured: refused naming the started digest, the program never sent);
   `attest::tests::a_statement_that_binds_a_launcher_is_v2_and_holds_only_for_that_launcher` (v2's canonical bytes; another
   launcher or none refused; each format for its own kind; a changed binding, and one dropped to pass as v1, fail the
   signature; four bad spellings refused on both sides). **Mutants** M103 (no comparison), M104 (a bound statement signed
   under v1's line), M105 (the format check gone), M106 (the attester measures nothing), M107 (an upper-case digest
   accepted): all red. Read on the runners before `master` moved: `witness.yml` at `3c74e97` — macOS `37251102472`, Windows `37251104259`, Linux arm64 `37251106129`, all green, `sandbox_attest_cli`'s five attestation tests and the binary's attestation unit tests named in each log.

## D-V2-90 — PS-E-01: a macOS guest's memory ceiling is the host's sampler of its peak footprint — TAKEN (head chef, 2026-10-05, under the owner's delegation)

1. **The gap** (`V2_OPENSHELL_STUDY.md` §4.1, "the `contained` set above would refuse every macOS run (reads open, no memory
   ceiling)"; `jail.rs` since PS-A2): macOS refuses `setrlimit(RLIMIT_DATA)` (EINVAL, run `35480762820`) and has no job object
   or cgroup, so a macOS guest had no memory ceiling at all — `resource_ceiling` absent on every macOS run, and one of the two
   gaps `contained`'s required set waited on. **Witnessed red on a macOS runner** (`witness.yml` `37267847333` at `d9d4803`): a
   guest allocating without end under `mem=64 MiB` was stopped by `wall=` at 8 s, its report saying `memory: not confined`.
2. **Taken:**
   - **The host is the ceiling.** `jail::peak_footprint(pid)` reads the guest's `ri_lifetime_max_phys_footprint`
     (`proc_pid_rusage`, `RUSAGE_INFO_V4` — the kernel's own memory accounting, compressed pages included). The process
     guest's watchdog (`guest.rs`, the one that already ends it at `wall=` and, on Windows, at `cpu=`) reads it every 5 ms
     (`Watchdog::MEMORY_TICK`) and ends the guest with SIGKILL at its budget. The PEAK, so a burst freed between two readings
     is still seen at the next. It runs outside the guest: an escaped guest cannot stop it (its Seatbelt profile denies every
     signal and all process information beyond itself), and the pid cannot have been reused (the watchdog is joined before
     the guest is reaped).
   - **Claimed in its own words:** `memory ceiling (the host's sampler)` — never spelled as the kernel's ceilings on Linux
     and Windows; the posture's `memory: capped` and `resource_ceiling` follow from it. Claimed only where the kernel
     answered the first reading, and never for an external launcher (L3), whose wall is the operator's.
   - **The stop is named:** `outcome.stopped_by = {dimension: memory, budget_bytes, observed_bytes, source: "the host's
     memory sampler, reading the guest's peak footprint every 5 ms"}`, exit 1, and the operator's sentence says this host
     ended it (the kernel's stops still say the operating system refused it more).
   - **The interval is the ceiling's resolution, measured:** at the ordinary run's 25 ms the first green read observed
     247 MB against a 64 MiB budget (`37268383827`); 5 ms costs a `proc_pid_rusage` call — microseconds — 200 times a
     second, and the read after it observed 68.8 MB and 92.8 MB on two reads (`37269100267`, `37269929073`).
3. **Not taken:** a kernel ceiling by a private interface (`memorystatus_control`'s per-process limit — undocumented, and a
   privileged call); `RLIMIT_AS` (it caps reservations, and the WebAssembly engine reserves by the gigabyte — the Linux scar
   in `jail.rs`); a guest-side cap (the guest's own word, which an escaped guest is not bound by). **Not changed:** what
   `contained` requires (the next step; macOS's reads remain its one absent property).
4. **Witnesses:** `sandbox_run_cli::a_guest_past_its_memory_budget_is_stopped_on_every_os_and_one_under_it_runs` (every OS:
   the control under the budget runs; past it, `stopped_by: memory`, exit 1, the claim and `memory: capped` in the report;
   `observed_bytes` printed); `jail::macos_tests::the_host_reads_a_processes_peak_footprint_and_it_follows_what_is_touched`
   (the reading answers, moves by what is touched, and never falls); `guest_cli` (the claim on macOS, in the host's words);
   `the_report_names_what_is_not_confined_as_well_as_what_is` (memory enforced on every platform again); the boundary unit
   test (macOS's words with the sampler establish `resource_ceiling`; without it, absent). **Mutant M108** — the reader
   answering a constant, the ceiling claimed and never firing — red on a macOS runner (`37268731314`). Read green before
   `master` moved: macOS, the whole `delulu` package, `37269100267`; Windows `37269926751`; Linux arm64 `37269105326`.

## D-V2-91 — RUNDIR-PERM-1: a run's own directory is made its user's alone by the host, and never adopted — TAKEN (head chef, 2026-10-05, under the owner's delegation)

1. **The finding** (read in the D-V2-90 witness's own log on a macOS runner): every macOS sandboxed run printed the guest's
   `warning: broker state directory … is not owner-only (mode 0755); this filesystem does not enforce POSIX permissions …
   Other local users may read delulu secrets stored here` — on the operator's screen, every run. The host made the run's
   directory (`delulu-guest-…` under the temporary directory: the guest's channel socket, a macOS guest's Seatbelt profile,
   an attester's document) with the process's default mode and left the guest's transport to narrow it to 0700; a macOS
   guest's profile refuses it every write but its socket, so the `chmod` failed and the warning blamed the filesystem. A
   false alarm trains an operator to read past the real one. **Witnessed red on a macOS runner** (`37268778120`).
2. **Taken:** the host makes the directory itself, 0700 on Unix (`guest::make_run_dir`, a `DirBuilder` with the mode), and
   only if nothing is there yet: a directory already under the run's name — someone else's — is refused, never adopted
   (`create_dir_all` adopted it). The same for `sandbox probe`'s directory.
3. **Not taken:** quieting the transport's warning for guests (the warning is right wherever it fires for a real state
   directory); a guest-side `chmod` allowance in the Seatbelt profile (a write the guest does not need).
4. **Witnesses:** `sandbox_run_cli::a_sandboxed_runs_own_directory_is_its_users_alone_and_no_false_alarm_reaches_the_operator`
   (red on macOS before, green after); `guest::tests::a_runs_own_directory_is_made_owner_only_and_never_adopted` (Unix: 0700;
   a second make refused). **Mutants** M109 (mode 0755) and M110 (an existing directory adopted) red in the VM.

## D-V2-92 — PS-E-01: `contained` requires egress, resource and host-loss confinement of a boundary this host measured; an unknown wall is reported, not refused — TAKEN (head chef, 2026-10-05, under the owner's delegation)

1. **The gap** (`V2_OPENSHELL_STUDY.md` §4.1: "`contained` (the default) requires filesystem, egress and resource"; D-V2-59
   deferred it — "`contained` requires none until macOS's gaps close"; RW 4.25 "a missing layer never refuses"): the default
   profile required nothing, so a run whose MEASURED boundary lacked a memory ceiling, a death signal or egress confinement
   was sent its program all the same. **Witnessed red on `4471d6f`:** `boundary::tests::contained_refuses_…` — a boundary
   this host measured without a memory ceiling was confirmed and sent the program.
2. **Taken:**
   - **`contained` requires three:** `egress_confinement`, `resource_ceiling`, `host_loss_ends_guest` — the three every
     operating system now gives a guest it starts (Linux: the guest's socket filter, `RLIMIT_DATA`/`RLIMIT_CPU`,
     `PR_SET_PDEATHSIG`; Windows: the AppContainer, the job's ceilings and kill-on-close; macOS: Seatbelt, the host's
     sampler (D-V2-90) and `RLIMIT_CPU`, the watcher (D-V2-60); the microVM: no network device, the VM's memory and the
     VMM's ceilings, its death signal).
   - **Not required by `contained`:** `filesystem_confinement` — macOS's reads are open (the study's set named it; requiring
     it would refuse every macOS run) — and `privilege_floor`.
   - **The rule for `unknown`** (`Profile::requires_proof`): `contained` refuses only what this host MEASURED as `absent`.
     An external launcher's wall (L3) is `unknown` throughout — the wall the operator chose for the run, reported as
     unmeasured — and `contained` does not refuse it; `hostile-agent` alone demands proof of an unknown (an attester's
     named claim, D-V2-87). The study's "satisfies a requirement only through an attester's claim" is kept for the profile
     that requires proof; a default that refused every unattested launcher would refuse PS-D-01's whole purpose.
   - **The refusal** is DL1408's, before the program is sent: "the `contained` profile requires a boundary this host
     measured to have egress_confinement, resource_ceiling, host_loss_ends_guest — and this one lacks: … Ways out:
     `--isolation microvm`, an external launcher, or — a person's choice, never made for you — `--sandbox-profile dev`".
   - **A consequence named now:** a Windows host that cannot make the guest's AppContainer falls back to the operator's
     identity under the job — no egress confinement — and `contained` now refuses there instead of running a guest that could
     reach the network as the operator; a macOS host whose watcher or sampler cannot start refuses likewise.
3. **Witnesses:** `boundary::tests::contained_refuses_a_measured_boundary_that_lacks_one_of_its_three_and_never_an_unknown_one`
   (all three with filesystem and privilege absent: confirmed and sent; no memory ceiling, no death signal, a TCP-only
   network rule: each refused naming what it lacks and the way out, the program never sent; an external launcher's unknown
   wall: confirmed). The channel's own unit tests moved to `dev`. **Mutants** M112 (`contained` refuses an unknown), M113
   (`hostile-agent` takes an unknown on trust), M114–M116 (each of the three dropped from the set): all red. Every OS's
   default run read green with the set in force — CI dispatched at `0560898`, `37271099512`: every job green — `test` on Linux, macOS and Windows (the whole workspace each), arm64, `microvm` (the KVM runner: a microVM run under `contained`), `microvm-reproducible`, `heavy-gates`, lints, formal, fuzz, Miri (diag, atlas, ffi), supply-chain, editor; the three long `miri-slow` jobs cancelled once the rest were read. Each OS's properties: Linux x64, arm64 and Windows all five established; macOS four, `filesystem_confinement` absent (reads not confined), `resource_ceiling` established.

## D-V2-93 — PS-E is complete, with three residuals named and kept open as rows — TAKEN (head chef, 2026-10-05, under the owner's delegation)

1. **Where PS-E stands** (`V2_OPENSHELL_STUDY.md` §4.1–§4.6): **E-01** complete (the typestate and the generation, D-V2-56;
   the five properties, D-V2-57; `hostile-agent`'s five, D-V2-59; attesters' named claims, D-V2-87; macOS's memory
   ceiling, D-V2-90; `contained`'s three, D-V2-92); **E-02** complete (D-V2-60, every backend); **E-03** complete on Linux —
   H1–H5 and H7–H10, each witnessed red and closed (D-V2-61 to D-V2-66); **E-04** complete on Linux and Windows (D-V2-69,
   D-V2-74) with the attestation binding (D-V2-89); **E-05** complete (D-V2-82, D-V2-83, D-V2-85); **E-06** complete
   (D-V2-78, D-V2-80).
2. **Taken: PS-E closes**, so P8 (P8-01 first: the control program in a guest, on E-01's confirmation) is next, with three
   residuals that stay OPEN where they are recorded — none is restated as closed:
   - **H6** (RW 4.27) — macOS's Seatbelt `mach-lookup` and Windows' reachable named objects under the escaped-guest
     harness. Twice the start of that work in a routine run was stopped by a safety classifier before anything ran
     (`HANDOFF.md` §11.7); it is for a session that can run it — perhaps the owner's.
   - **macOS's launcher window** (RW 4.28) — macOS has no `fexecve`, so between the hash and the start a rename over the
     path is not refused there. Not built: starting a private COPY of the hashed bytes would break every launcher that finds
     its siblings by its own path (a Homebrew binary's `@executable_path`, a script's `$0`).
   - **macOS's reads** (`filesystem_confinement` absent on macOS; `MATHEMATICS.md` "NOT CLAIMED, except the state
     directory") — every allow-list of readable roots aborted the guest (PS-A2); so `contained` does not require it and
     `hostile-agent` refuses macOS's jailed guest.
3. **Why now:** each residual waits on something no routine run can supply this period (a session the classifier lets run;
   a kernel interface macOS lacks; a profile of readable roots macOS's loader accepts), and P8 and P9 — the owner's next
   phases — wait behind them. The phase's CI: `37271099512` at `0560898` and `master`'s push run (V2_PHASE_STATUS).

## D-V2-94 — RW 4.40, SILENT-QUEUE-1: the broker daemon accepts and reads on threads of its own; one handler takes whole requests — TAKEN (head chef, 2026-10-05, under the owner's delegation)

1. **The finding** (the red-team pass on FRAME-DRIP-1, F4; D-V2-73 recorded it "not changed", RW 4.40 "open — recorded"):
   the daemon's serve loop read one connection at a time, each owed its request within 5 s, so N clients that connected
   and said nothing delayed every client behind them by about 5·N s — the e-stop's revoke among them (the red team's twelve
   held `broker status` 61.5 s; since D-V2-75 a command behind them fails in words after 15 s and the dead-man probe parks
   devices, so the failure was safe and the service was not). **Witnessed red in the VM on `8cbd1e0`:** six silent
   connections held a `Status` 30.6 s (`brokerd::tests::silent_connections_do_not_queue_the_clients_behind_them`).
2. **Taken:** an accept thread takes each connection and a reader thread per connection reads its one request (still owed
   whole within 5 s — FRAME-DRIP-1's bound — with its write bounds set); only a WHOLE request is handed, with its
   connection, to the one handler, which alone touches custody and answers in the order requests arrive whole. At most 64
   connections read at once; one past that is dropped at once, so a flood costs threads it cannot grow without bound. A
   stopping daemon sets a flag and connects to itself, so its accept thread ends. The Windows `Listener` is `Send` — its one
   pointer is a security descriptor it owns, with no thread affinity.
3. **Not taken:** a shorter first-byte deadline alone (it divides the delay, never removes it: twelve silent connections
   would still hold a client 12 s); handling requests concurrently (custody and the audit chain are written by one
   handler on purpose — AUDIT-WRITERS-1's lesson).
4. **Witness** red before (30.6 s), green after (well under its 3 s bound). **Mutant** M118 — one reader at a time — red
   (4.8 s). Read on the runners before `master` moved: `witness.yml` at `7c55af9` — Windows `37273275501` (160 passed), macOS `37273278082` (175 passed), Linux arm64 `37273280678` (193 passed): the daemon's unit tests and six daemon-facing targets (`broker_cli`, `estop_cli`, `guard_cli`, `guard_e2e`, `sandbox_guard_e2e`, `secret_verify_cli`), the witness named in each.
5. **Amended the same run (the head chef's re-read of `7c55af9`, already on `master`):** a reader was counted only while
   it READ, not while it waited to hand its whole request to a busy handler, so a flood of complete requests behind an
   answer still being written grew a thread per connection — witnessed red on `101cb1e`, 300 requests grew the process
   from 4 to 240 threads (`brokerd::tests::a_flood_of_whole_requests_behind_a_busy_handler_grows_no_thread_past_the_bound`,
   Linux). A reader now counts until its request is handed over (`77d436e`); past the 64 a connection is dropped at once.
   Mutant M119 (released before the handover) red, 240 threads. Read on the runners: `witness.yml` at `77d436e` — Windows `37275973144` (158 passed), macOS `37275975885` (173 passed), Linux arm64 `37275979053` (187 passed, the flood witness among them), the daemon's unit tests, `broker_cli` and `estop_cli` (and `guard_cli`, `guard_e2e` on Windows and macOS).

## D-V2-95 — P8-01: a control program runs in a guest; the host performs its device operations by the interpreter's own body, and its devices start when the program is sent — TAKEN (head chef, 2026-10-05, under the owner's delegation)

1. **What was missing** (read against the code by routine run 12, `V2_P8_DESIGN.md` "build order"): minting already
   crossed the channel — `root.actuator`/`root.sensor` are root methods the host answers with a handle — but a guest's
   `command` and `read` reached a host channel that knew no device, and `guest::CARRIED` refused the program first
   ("`--sandbox` cannot carry this program yet: it uses Actuator. Nothing ran.", exit 2 — mutant M122 restores exactly
   that and turns all seven new witnesses red).
2. **One body, two callers.** The interpreter's `call_actuator`/`call_sensor` bodies — the envelope check on the command
   value, C39's refused-attempt sweep, `DeviceBroker::command`/`read`, the `Envelope`/`LeaseRevoked`/`NoDevice` values —
   moved into `device.rs` (`actuate`, `sense`, `command_check`, `refusal_detail`). The interpreter calls them and turns
   each refusal into its DL1904 trace record; `HostChannel::with_devices` calls them after its custody gate (so an
   actuator command still round-trips to the grant tree, the e-stop's way in) and records each refusal in the report's
   `denied`, in the same words. A host-held (handle) actuator or sensor in a guest's interpreter goes to its effect sink.
   Behaviour-preserving for the ordinary run: the 46 tests of `actuate_cli`, `dead_man_cli`, `device_delegation_cli`,
   `estop_cli` and `hw_adapter_cli`, and the runtime's 243 unit tests, unchanged and green.
3. **Shared, not copied.** `run_cmd.rs`'s device code became four functions the sandboxed path calls too:
   `device_terms` (the profile, the DL1905 sign-off gate, the clock), `watch_devices` (each actuator's own grant node
   and the e-stop probe, under broker custody), `plan_devices` (under `hw:`, the driver resolved once, its provenance
   checked and recorded, the process started — all before a guest exists) and `close_devices` (the watchdog stopped,
   every lost device said on stderr, the devices' nodes revoked).
4. **The broker starts when the guest is sent its program** (`guest::GuestDevices`) — *since D-V2-110 (2026-10-11), at the guest's
   first request after it, so the guest's own check of the program is not counted either* — not before the launch: the leases'
   dead-man counts from the program's first instant, so a guest's launch — a microVM's boot above all — is not charged
   against its first heartbeat. Everything that can refuse (DL1905, the driver's signature, its start) still refuses
   before any guest exists. A guest that never beats after that loses its device on the host's watchdog: the beat rides
   DEVICE activity only, so a guest cannot keep a machine by talking to its host (witnessed: 3,000 console requests
   did not beat the lease; mutant M124, a beat on every request, red).
5. **Applied under `--sandbox`:** `--broker-profile`, `--approved`, `--adapter-cmd`, `--adapter-artifact`,
   `--require-signed-adapter`, `--adapter-signer`, `--adapter-record`. **Not yet:** `--sim-step` (a sandboxed device
   run's dead-man is the wall clock's) and `--signoff` (a sign-off is written by the ordinary run's simulation) — both
   refused by the allowlist, in words, as every unapplied flag is. `Compute` (accelerators) stays uncarried.
6. **Witnesses** (`crates/delulu/tests/sandbox_devices_cli.rs`, seven): the same program says the same thing in a guest
   as unsandboxed, its refusal a DL1904 on the report's record; a guest's out-of-envelope command never reaches the
   driver process (its own log); a guest that stops beating is told `LeaseRevoked (missed-heartbeat)` with its
   fail-state; a guest wedged for ever, ended by its wall ceiling, loses the arm all the same within a bound of the
   heartbeat (the host's journal: how late the beat was); a guest that only chats is not beating; `grants revoke` of the
   device's node stops a guest's arm mid-motion (`operator-revoke`, the supervisor survives); the DL1905 gate starts no
   driver and no guest. **Mutants:** M122 (master's refusal) red on all seven; M123b (both envelope checks removed) red
   — 999 reached the driver; M123 (the capability's own check removed) SURVIVES on the broker's own envelope check, the
   second wall `device.rs` documents — recorded, not a gap; M124 (a beat on every request) red; M125 (the broker on a
   stepped clock) red; M126 (the host channel given no devices) red.
7. **Found by the suite, and hardened in both paths:** once the sandboxed run applied the device flags,
   `sandbox_modes_cli` saw `--approved` beside a non-hardware run accepted there and doing nothing — as the ORDINARY
   run had always done, in silence. `device_terms` now refuses `--approved`, `--adapter-cmd`, `--adapter-artifact`,
   `--adapter-signer`, `--adapter-record` and `--require-signed-adapter` without `--broker-profile hw:` (exit 2,
   naming each, "Nothing ran") — an operator who passed one believes a sign-off or a driver is in force. Witness
   `a_hardware_flag_without_a_hardware_profile_is_refused_sandboxed_or_not`; mutant M127 red.
8. **Verified:** clippy clean; `check-other-os.sh` clean for Windows and macOS (twice — the second after the last code edit); read on the runners before `master` moved — at `76ba9ca` Windows `37298032325` (73 passed) and Linux arm64 `37298039715` (74 passed) green, macOS `37298035733` red on the e-stop witness alone (the test's broker socket path, 113 bytes, past macOS's 104 — the test's scratch name, fixed in `45e65eb`); at `45e65eb` macOS `37299710143` (47 passed), Windows `37299713468` (46 passed), Linux arm64 `37299716481` (47 passed): `sandbox_devices_cli`'s eight witnesses named on each, with `sandbox_modes_cli`, `hw_adapter_cli` and `dead_man_cli` (and at `76ba9ca` `estop_cli`, `actuate_cli`, `atlas_chain`, `sandbox_run_cli`); the full suite alone 2,127 passed, 0 failed, 15 ignored (158 binaries), cargo exit 0 — its second run: the first (2,124 passed, 2 failed) lost the Survey's freshness test to a test file edited while it ran, and found `sandbox_modes_cli`'s unapplied `--approved` (item 7).
9. **Next (P8-01's rest, then P8-02):** a device run at L2 read on the KVM runner (`microvm_cli`); `--sim-step` under the
   sandbox (the stepped clock is the host's, so it can be offered); the device events in the sandbox report. Then P8-02,
   the Verified-class adapter as a `.dpx`.

10. **Amended the same run — P8-01's rest.** *At level 2:* `microvm_cli.rs`'s KVM-gated witness runs the same control
    program under `--isolation microvm` — commanded, refused out of envelope, the joint read back, level 2, the refusal in
    `denied` — with a 150 ms heartbeat and a command as its first act: green on the KVM runner (CI dispatched at
    `07f6cd9`, `37301435891`, its `microvm` job read — 12 passed — then the run cancelled). **Mutant M128** (the broker
    started when the plan is made, before the launch) SURVIVES every L1 witness — a process guest launches inside every
    heartbeat they use — and is RED on the KVM runner (`666cb53`, `37301599674`): the VM's boot revoked the lease before
    the guest's first command ("REVOKED … (missed-heartbeat)"); reverted in `fee5353`. Item 4's design is witnessed only at
    L2, and says so. *The simulator's parity:* `--sim-step` and `--signoff` are applied under the sandbox now (`5b89f9f`) —
    the stepped clock is the host broker's, and a sign-off is written only when the guest was sent its program and exited 0
    (withheld in words otherwise). Witnesses: a sandboxed simulation loses its lease at the same refused attempt as the
    ordinary run (seven attempts at 1,000 simulated ms outlast a 5,000 ms heartbeat the wall clock never reaches); a clean
    sandboxed simulation signs off and its sign-off opens DL1905 for a sandboxed hardware run, a faulted one signs nothing.
    M129 (the flag refused, as before), M130 (the stepped clock dropped), M131 (no sign-off), M132 (a sign-off despite a
    fault): each red. **Verified:** clippy clean; `check-other-os.sh` clean for Windows and macOS after the last code edit; read on the runners at `5b89f9f` before `master` moved — macOS `37303092995`, Windows `37303096425`, Linux arm64 `37303099029`: `sandbox_devices_cli` 10 passed and `sandbox_modes_cli` 11 passed on each, both new witnesses named; the full suite alone through `scripts/suite.sh`, 2,129 passed, 0 failed, 16 ignored (158 binaries — the new KVM-gated test among the ignored), cargo exit 0, the tree unmoved. **Push runs:** `45e65eb` (carrying `76ba9ca`) — CI `37300708685` success, 14 jobs, none failed (arm64 ping-pong MEASURED 2.93x against a control of 4.12x; the other three NOT MEASURED), `ocsf` `37300708671` success; `6796c31` (records) — CI `37301082817` success, 14 jobs (arm64 MEASURED 2.90x/4.11x).

11. **P8-01 complete (the same run).** A sandboxed run has no trace, so its report gains a top-level `devices` — the host
    watchdog's journal, each `lease.revoked` and `failstate.engaged` with its device, `at_ms` and reason — beside `egress`,
    the other effects the host performed for the guest; the watchdog is stopped first so the journal is whole, and
    `delulu schema run-report` declares it (`device_event`; the schema stays closed). `bc8bd91`. The stops-beating witness
    reads the journal and validates the report against the schema; M133 (no journal) and M134 (the schema not told) red.
    **Verified:** clippy clean; `check-other-os.sh` clean for Windows and macOS after the last code edit; read on the runners at `bc8bd91` before `master` moved — macOS `37305292797` (28 passed: `sandbox_devices_cli` 10, `json_contract` 14, `schema_cli` 4), Windows `37305296588` and Linux arm64 `37305299628` green on the same three targets; the full suite alone through `scripts/suite.sh`, 2,129 passed, 0 failed, 16 ignored (158 binaries), cargo exit 0, the tree unmoved. **Push runs:** `fee5353` — CI `37302336639` success, 14 jobs, `microvm` among them (the L2 witness green on `master` too); `5b89f9f` — CI `37304139892` success, 14 jobs (arm64 MEASURED 2.97x), `ocsf` `37304139836` success; `521d688` — CI `37304345347` success, 14 jobs (arm64 MEASURED 2.92x). With it every item of `V2_P8_DESIGN.md`'s P8-01 is built and
    read — L1 and L2, witnesses 1–5 and their falsifiers — and **P8-02 is next.**

## D-V2-96 — P8-02: a Verified driver is four pure exports R-Get-checked before it exists, its logic on a thread of its own under one deadline — TAKEN (head chef, 2026-10-05, under the owner's delegation)

1. **Four exports, not an `encode`/`decode` pair** (a departure from `V2_P8_DESIGN.md`'s step 4): `encode_command`,
   `decode_command`, `encode_read`, `decode_read`, so a reply's meaning never depends on the plugin guessing which
   question it answers — an `OK` that accepts a command and a `NODEV` that answers a read are different types, not one
   value read two ways. `decode_command` returns `Option[Str]` (the device's refusal, if any) because the language has no
   unit literal to write `Ok(())` with.
2. **The interface is R-Get**, exactly as `p.get` applies it: each export must exist (DL1502 when missing) with exactly the
   interface's parameter and return types and a row inside the EMPTY row (DL1504) — so a driver that names an effect, or
   asks for a capability, is not a driver, whatever its manifest says. No new code.
3. **One thread owns the DIR**, with a 16 MiB stack and the depth bound `max_depth_for_stack` gives it (the stack/depth
   pair is the invariant, D51); **a fresh interpreter per call**, so nothing one call computes reaches the next — the
   plugin is a function of its arguments.
4. **One deadline for the whole exchange** (FRAME-DRIP-1's lesson): the two logic calls and the transport share one
   `EXCHANGE_TIMEOUT`; the transport is handed only the time left, and an answer after it is a timeout.
5. **A frame is one line** (no `\n` or `\r`, non-empty, at most 64 KiB): a transport frames by line, so a frame holding
   a line break would put a second request on the wire that no grant check saw. Checked before the transport is called.
6. **What poisons:** a fault in the plugin's code (its own bug, a spent budget, the depth bound), a value outside its
   type, a frame that is not one line, a non-finite reading (the language's arithmetic can compute one), any transport
   failure — and a transport's `Refused`, which is not a transport's to say. A device's refusal, read by the plugin, does
   not poison: a device saying no is working.
7. **Witnesses and falsifiers:** `verified_adapter.rs`'s eight tests; M136–M144 red but M143, which survives on a second
   wall (the decode's share of the same expired deadline), and M143b — both walls removed — red.

## D-V2-97 — P8-02: a Verified driver is pinned, re-proved and interpreted from one read of its bytes; its transport is the host's — TAKEN (head chef, 2026-10-05, under the owner's delegation)

1. **One read, one artifact.** `--adapter-dpx FILE` is read once; the signature embedded in it (over the manifest and the
   DIR) must verify under `--adapter-signer` — which is REQUIRED for a Verified driver, because an unpinned signature on
   an artifact an operator can replace proves only that somebody signed something (DL1510 for another key or a signature
   that does not verify, DL1511 for none) — and the DIR that is interpreted is the one the signature covered and the
   checker replayed. Nothing is started by name, so D-V2-50's check-then-start window does not exist for it. This is what
   RW 4.7 asked for: specification §5.4's signed-plugin form of the hardware adapter.
2. **The transport is the host's, and only the host's.** `--adapter-transport CMD` names the process that carries the
   driver's frames to the device and its replies back (`adapter::LineTransport`, P8-03's shape, sharing `ProcessAdapter`'s
   bounded line reader). The plugin computes frames and can write nothing: it holds no capability and its exports are
   pure. A transport is resolved once, as a driver is.
3. **The interface before the transport.** `verified_adapter::check_interface` runs before the transport is looked up, so
   a plugin that is not a driver causes no process to be resolved or started — witnessed deterministically with a
   transport that does not exist.
4. **A flag that would be dropped is refused** (run 13's lesson again, `verified_driver_flags`): `--adapter-dpx` beside
   `--adapter-cmd` (each names the driver) or `--adapter-artifact` (which names a process driver's signed bytes), either
   new flag without the other, `--adapter-dpx` without its signer, and either on a run that is not `--broker-profile hw:`
   — exit 2, naming the problem, "Nothing ran".
5. **The sandbox carries both flags**, so a sandboxed control program drives a device through the host's Verified driver
   (P8-01's shape: the guest holds a handle and the host performs every operation).
6. **No new codes.** DL1510, DL1511, DL1507, DL1504 and DL1905 cover every refusal, as `V2_P8_DESIGN.md` said they would.
7. **Witnesses and falsifiers:** `hw_dpx_cli.rs` (five), `adapter.rs`'s three transport tests, `verified_adapter.rs`'s
   eight (D-V2-96); mutants M145–M153 each red.

## D-V2-98 — P8-03: the reference simulator is a module with two callers, and a device process of its own; its bounds are the bench's, not the grant's — TAKEN (head chef, 2026-10-05, under the owner's delegation)

1. **One simulator, two callers.** `delulu_runtime::sim` holds the state, the park pose, the synthetic signal and the
   line protocol; `DeviceBroker` drives it in-process under `Profile::Sim`, and `delulu device sim` drives the same code
   from a pipe. Two implementations of a simulator would eventually disagree, and the parity witness could then only
   measure which one was newer.
2. **A documented verb, not an internal one.** `V2_P8_DESIGN.md` asks for a transport "the tests and a lab can both
   use", so the verb is in `--help`, in `SUBCOMMANDS`, in the completion script and in both `json_contract` sweeps. Its
   `--json` form describes the bench and serves nothing, which is also the only form a blind argument sweep can drive.
3. **The bench's bounds are the DEVICE's limits.** `--actuator` here is not a grant: it is what the machine can do. A
   narrower bench than the grant is how a hard stop is simulated — a real machine's limit switches — and the two
   refusals stay distinguishable, because the host's envelope refusal never reaches the device.
4. **The dead-man terms are ignored by the bench.** The grant form carries `heartbeat_ms`, `ttl_ms` and `fail`; a device
   process is given them (the form is one parser) and acts on none of them. A lease belongs to the host that granted it,
   and a device enforcing its own would be a second dead-man nobody declared.
5. **A device's answer is one bounded, escaped line.** TERMINAL-TEXT-1 and AUDIT-TEXT-1 from the other side: this
   process's words reach an operator's terminal and a host's diagnostics, and a second line would be read as the answer
   to the next frame. A request past 64 KiB is answered with nothing, as `adapter.rs` bounds a reply.
6. **Nothing physical.** It is a simulator, it models no physics, and `--json` says `"simulated": true`. A real device
   stays environment-blocked (D23).
7. **Witnesses and falsifiers:** `sim_device_cli.rs` (three, parity among them), `sim.rs`'s six; M154–M158 each red.

## D-V2-99 — RW 7.17: an unanswered probe gets no cause of its own; the journal's detail is the record, and now a gate — TAKEN (head chef, 2026-10-05, under the owner's delegation)

RW 7.17 left a question beside its unreproduced flake: does a probe the broker never answered deserve its own
`RevokeCause`, beside `operator-revoke`? **No**, and the reading that settles it:

1. **A program must behave identically in all three cases** — a revoked node, an answer that is not one, and no answer —
   and it does: it loses the device, the declared fail-state engages, it stops. A fourth cause would be a distinction
   with no action behind it, in the vocabulary a program, a report and the audit all share.
2. **The difference IS already recorded**, where a person investigating reads: the probe's own words
   (`cli.rs`'s `mint_device_nodes` — "grant node `g_...` is revoked (by audit seq N)", "... answered ... which is not an
   answer", "the broker did not answer for `g_...`: ...") travel verbatim into the `lease.revoked` detail in the
   broker's journal, which `close_devices` prints on standard error and the sandbox report carries (P8-01).
3. **That was a comment until now.** `device.rs`'s
   `the_journal_tells_the_three_dead_probes_apart_even_though_the_cause_cannot` asserts each of the three reasons
   verbatim in the journal while asserting that the cause still collapses — mutants M159 (the probe's reason dropped)
   and M160 (the detail reduced to the cause's name) are red, and two older witnesses go red with them.
4. **What stays open in RW 7.17** is the half that needs a recurrence: the arm64 flake was found, not root-caused. The
   next occurrence is now readable from the journal — which is what the row asked for.

## D-V2-100 — P8-04's reading surface: an envelope's refusal is a `deny` in the chain, recorded by the broker at the run's word, citing the use it overrides; a monitor's cursor is a record's hash — TAKEN (head chef, 2026-10-09, under the owner's delegation)

Routine run 15 measured P8-04 step 1's claim ("everything the monitor reads exists") and it was false (AUDIT-REFUSAL-1,
`V2_LOG.md` 2026-10-09). Decided:

1. **The envelope stays where it is decided.** The device broker in the run judges a command's magnitudes, as
   `mint_device_nodes`' comment insists (copying the envelope into the broker's decision would make a second authority of
   record that could drift from the first). The broker is told AFTERWARDS and records; it does not re-decide.
2. **The record is a `use` with decision `deny`** — the action an investigator and an OCSF export already read — on the
   node that used the arm (the same actor as the `allow` it overrides, the run's node: a lease client acts only as its own
   node, invariant 25), at the device, with `refused_by`, `code` (DL1904), `reason` (bounded) and `overrides_seq`. The pair
   `allow` then `deny` is the truth: the authority allowed the use, the envelope refused the command.
3. **Best-effort, never silent.** The command is already refused and the device already safe when the record is sent; a
   record that cannot be written does not fail the run, and is said once on standard error. A same-uid process could forge
   such a record — as it could any (category 7) — and a forged one can only ADD refusals.
4. **A monitor's cursor is a record's hash, never a seq** (AUDIT-SEQ-1: two writers number the chain independently, and
   `--since SEQ` loses records). Until RW 4.44 gives the chain one numbering, P8-04 resumes after the last hash it read.
5. **Not in this decision:** a dead lease's revocation (missed heartbeat, TTL) is still journaled only on the run's
   standard error; it is the next reading gap for the monitor's rules, recorded in `V2_P8_DESIGN.md`.

## D-V2-101 — P8-04 step 3 (b): `delulu monitor watch` — a monitor node between the operator and the runs, which quarantines a run by revoking it and says why in the revocation's own record — TAKEN (head chef, 2026-10-09, under the owner's delegation)

The question run 14 left (`V2_P8_DESIGN.md` P8-04 step 3) is answered with option (b), as recommended there; (a) stays
refused and (c) stays the owner's.

1. **The monitor holds one node and acts only as it.** The operator mints `g_M` (`grants delegate`) and delegates each
   run's lease under it (`grants delegate --parent g_M`); the monitor names `g_M` as the caller of every quarantine. The
   broker's own rule — a caller revokes only itself or a descendant — is what confines it: not a sibling, not its parent
   (witnessed: a monitor on a sibling node judged every refusal of the run and quarantined nothing). **The residual, named:**
   `g_M` HOLDS the runs' authority, because a parent bounds its child (R-7); what keeps the monitor from using it is that
   this program performs nothing but a revoke, and the same-uid boundary (§11.4, category 7) — a same-uid process could
   name any node as caller, as it could already. Option (c), a revoke-only principal, is what would make "exactly revoke"
   true, and it changes who may revoke: the owner's.
2. **A quarantine stops one run, not the fleet.** Its target is the child of `g_M` on the path to the record's node — the
   run's own node — so the run's devices park (their nodes are descendants: the declared fail-state engages) and the
   program's next use of anything it held faults DL1403 naming the monitor's audit seq: the guest is ended, as the study's
   design asked. A sibling run under the same monitor is untouched.
3. **It says what it saw, in the chain.** `ReqBody::Quarantine { caller, target, why }` is `Revoke` with the monitor's
   account carried in the revocation's own record (`why`: the rule, the count, the window, the evidence's seqs and hashes;
   bounded at 1,024 bytes by `delulu_broker::bounded_text`). It decides nothing new: the same self-or-descendant rule.
4. **It reads only what the host recorded**, from the chain's head at its start, resuming after the last record's HASH
   (D-V2-100: a seq is not a cursor while RW 4.44 is open). A record the monitor last read that is no longer in the chain
   is said, and reading continues from the head — never re-judging records already judged.
5. **Two rules, declarative:** `envelope` (any `use` `deny` with `refused_by: envelope`) and `denies=N/MS` (N `deny`
   records of one run within MS ms, by the records' own timestamps). Only `deny` records count — an allowed use never
   does (mutant M172 red). Break-glass use and special-use reach (`V2_OPENSHELL_STUDY.md` §4.7's other two) wait until
   their records are measured the way step 1's were.
6. **The surface:** `delulu monitor watch --node g_ID --rule … [--poll MS] [--for MS] [--state-dir DIR] [--json]`; a new
   subcommand's five gates in the same commit (the help, the dispatcher and `SUBCOMMANDS`; `json_contract`'s failing sweep
   and its daemon-backed success sweep; `mcp.rs`'s `EFFECTORS` — it revokes). It ends when `g_M` is no longer live, when
   `--for` elapses, or (exit 1) when it cannot reach its broker or a quarantine is refused.
7. **Not built here — step 5, the monitor's own death:** a run whose monitor goes quiet is not yet treated as one whose
   monitor fired. Each run's own dead-man (10f) remains the real-time guarantee; the monitor is a second, slower line.

## D-V2-102 — AUDIT-SEQ-1: the audit chain's seq is settled by the log under its append lock, and the daemon's count takes the chain's next seq as a floor — TAKEN (head chef, 2026-10-09, under the owner's delegation)

1. **One numbering authority: the lock.** `AuditLog::append` assigns `max(entry.seq, highest seq in the chain + 1)` under
   the append lock it already takes (AUDIT-WRITERS-1). Every writer — the daemon, a sandboxed run's host, `audit
   reconcile`, an adapter's provenance record — therefore writes a seq the chain has never held, in increasing order.
2. **The daemon keeps its references true.** It stamps a seq into nodes (`revoked_by_seq`) and answers (`audit_seq`)
   before the record is written, so it takes `AuditSink::next_seq` as a floor before each seq; a record is renumbered at
   the lock only in a true race between that floor and the append, never on a restart or after a record another writer
   appended. **Corrected the same run (red-team F2, RW 4.48):** that race is not rare under contention — 72 parallel
   runs beside the daemon left 2 references naming another writer's record. The numbering holds; the references need the
   lock held from the floor to the write. **Closed by routine run 16 (D-V2-105):** the daemon holds it per request.
3. **Not chosen:** routing every host record through the daemon (a second path for the same record, and still racy for
   two hosts with no daemon); renumbering only at the lock (uniqueness, but the daemon's references would name other
   records — mutant M176 shows the difference).
4. **Old chains** keep their repeated seqs — rewriting a hash chain is what an audit log must never do — and continue after
   their highest seq. `DEPLOYMENT.md`'s caution on `--since` now says exactly that.

## D-V2-103 — ACTOR-CUSTODY-1: an actor's worker performs its effects under the run's custody and brokers — TAKEN (head chef, 2026-10-10, under the owner's delegation)

1. **One authority, every thread.** Each actor worker builds its own custody client for the RUN's own node, on its own
   thread (a client caches its node's epoch and is not shared between threads), and shares the run's device and compute
   brokers. So under daemon custody a revocation reaches an actor's next use (an epoch-class use within one epoch, a
   synchronous one at once), each synchronous use is recorded, and in every mode an actor's commands meet the one lease,
   rate bound and dead-man per device, and its dispatches the bound adapter.
2. **An actor's uses are the run's node's uses.** The chain names the run's node; which actor performed a use is in the
   effect trace (`actor`, `member`), not in the chain. Hardening, not a new meaning: a capability handed to an actor is the
   authority it was in `main`.
3. **Not chosen:** refusing actors that hold an actuator under daemon custody (RW 4.47's stop-gap) — it leaves an actor's
   other effects ungoverned by the daemon and refuses a sound design, a control loop in an actor; a grant node per actor or
   per worker — a finer e-stop, but a new shape of authority (what an actor's node means), which is the owner's to decide.
4. **Unchanged:** an actor gets no plugin engine and no foreign bindings, so its `load` and `root.foreign(load)` still fail
   closed. A plugin export's nested interpreter has the same shape as the worker had (RW 4.50) — its own decision.

## D-V2-104 — PLUGIN-CUSTODY-1: a plugin export runs under the host's custody and the run's device broker; the daemon answers a plugin's liveness — TAKEN (head chef, 2026-10-10, under the owner's delegation)

1. **The nested interpreter shares the host's custody**, for the call's duration (an `Rc`; the host is suspended while the
   export runs), and the run's device broker. A plugin's use of a capability its host handed it is checked and recorded as
   the host's own use of that capability is, and it is the run's node's use — as D-V2-103 decided for an actor's.
2. **`BrokerClientCustody::liveness`** asks the daemon on every call (one `NodeState` round-trip): live, revoked (with the
   revoking seq for DL0801), or `Unknown` — dead — for anything else, the broker unreachable included. Until now no plugin
   could be called under the daemon at all (PLUGIN-DAEMON-1). **D-V2-27 item 4** ("in embedded and daemon custody alike")
   was true of the node's creation and never of a call under the daemon; it is the owner's ruling and is left as written —
   this entry is its correction.
3. **Unchanged:** `load` refuses grants carrying `Declassify` or `ForeignCall` — custody alone decides them, and the host's
   custody answers for the run's node, which may hold more than the plugin's grant. Compute is not passed (no grant can
   carry `ForeignCall`, so a plugin cannot dispatch).
4. **Not chosen:** a custody client for the PLUGIN's own node — a finer record and an `unload` that stops a call midway, but
   it would make the grant's scopes bind beside the capability's, a change in what a grant means: the owner's.

## D-V2-105 — RW 4.48: the daemon holds the audit chain's append lock from a request's first seq to its last record; AUDIT-DAY-1: a record is filed under the later of its day and the chain's — TAKEN (head chef, 2026-10-10, under the owner's delegation)

1. **A held lock is a token.** `AuditLog::take_lock` returns the chain's `AppendLock`; `next_seq_under` and `append_under` take
   it and refuse one for another chain. Every other writer keeps the one-call `append`, which takes and releases the lock.
2. **The daemon holds it per request.** Its sink takes the lock at the request's first seq or record and keeps it to the
   last, so a seq stamped into a node or an answer is the seq its record is written under. The serve loop releases it after
   `handle`, BEFORE the reply is written, and once after the startup record — so the chain is held only while the daemon
   computes, never while a client reads. D-V2-102 item 2's race is closed, not narrowed.
3. **A record's day file follows the chain (AUDIT-DAY-1).** Filed under the later of its own stamp's day and the chain's
   current day; the record keeps its own stamp. Not chosen: re-stamping a late record (its stamp is evidence of when the
   writer acted) or ordering the chain by stamp (two writers' stamps are not an order; the chain's links are).
4. **Not chosen for RW 4.48:** renumbering the daemon's references after the fact (every reference site would need the
   written seq back — a node, an answer, DL1403's text, `overrides_seq`) or routing every host record through the daemon
   (D-V2-102 item 3's reasons stand).

## D-V2-106 — P8-04 step 5: the monitor's own death — a dead-man for a node, held by the broker; `quarantine` the default — TAKEN (head chef, 2026-10-10, under the owner's delegation)

1. **Option (c) of `V2_P8_DESIGN.md` P8-04 step 5, built.** The broker holds a dead-man FOR A NODE (`delulu-broker`'s
   `deadman.rs`): `ArmDeadman { node, period_ms }`, `Beat { node, key }` and `DisarmDeadman { node, key }` over the daemon's
   IPC. If no beat arrives within the period of the arming or of the last beat, the daemon revokes the node AS ITSELF — its
   subtree with it — and the revocation's record says why (`why: "dead-man: … no beat within N ms …"`). **Why this hardens
   rather than redefines Authority:** any node may already revoke itself, so arming a future self-revocation adds no authority;
   it can only remove some, never upward or sideways (the step-3 rule is untouched). Not chosen: (a) a TTL the monitor renews
   (a holder extending its own deadline is a widening in time, and the tree has no local renewal); (b) each run probing its
   monitor (couples every run to a monitor it cannot see).
2. **A beat is a proof of possession.** Arming returns a fresh 128-bit key, held by the monitor in memory and by the daemon
   only as its BLAKE3 hash — never on disk, never in the chain. A beat or a disarm with another key is refused, and so is a
   second arming over a live one, so no other process can keep a dead monitor's dead-man quiet, replace it with its own, or
   switch it off. On Linux the monitor makes itself non-dumpable once it holds the key (HOST-DUMPABLE-1's step), so a process
   of the same user, not root, cannot read the key out of its memory. The residual is root, and a same-user process where the
   kernel gives no such flag (macOS, Windows) — category 7, named, not claimed.
3. **Time is the monotonic clock, and late is dead.** Deadlines are `Instant`s; a wall-clock step neither fires a dead-man
   early nor holds one off. The daemon's loop waits for its next request no longer than the nearest deadline, and fires every
   dead-man past due BEFORE it handles the next request, whichever woke it — so a stream of requests cannot hold one off, and a
   beat (or a disarm) that arrives after its deadline fires it rather than rescuing it. A dead-man whose node was revoked by
   someone else meanwhile records nothing.
4. **`quarantine` is the default for every monitor; `continue` is the operator's opt-out.** The design made it the default
   "for a run that holds a device"; a monitor cannot know what the runs it will watch will hold, so it is the default for all
   — a watchdog gone quiet is treated as one that fired. The period is `--death-after MS` (100 ms to 1 h; at least two polls),
   by default `max(2000 ms, 8 × poll)`; the monitor beats twice per poll — before reading the chain and after — so a slow read of
   a long chain costs one period, not two. A run's own dead-man (10f) stays the real-time line; this is the slower one.
5. **How a watch ends decides what the dead-man does.** Ended as asked (`--for`) while its node lives: the monitor disarms,
   recorded (`deadman-disarm`) — from that record the runs below are unwatched by the operator's choice. Its node revoked by
   someone else (the operator's way to end a watch): nothing to disarm. Ended on an error (the chain unreadable, a beat refused,
   the broker unreachable): the dead-man stays armed — the monitor has stopped watching, and that is a death. A monitor that
   finds its node revoked by its OWN dead-man (it was stopped, or too slow) reads that from the chain, exits 1 and says so,
   rather than ending as if it had been asked to. A monitor whose node is no longer in the tree (the broker restarted) ends with
   an error instead of watching nothing (an oddity of routine run 15's red-team pass).
6. **No new code.** A dead-man's protocol misuses — nothing armed, a second arming, a wrong key, a period out of bounds — are
   DL1401 ("broker-protocol failure"); a beat to a node its dead-man has revoked is DL1403, naming the seq.

## D-V2-107 — CLOSURE-SCOPE-1: a closure carries its home module and resolves its names there; Verified callbacks stay — TAKEN (head chef, 2026-10-10, under the owner's delegation)

1. **Lexical scope across the plugin boundary.** A closure holds its home — its module's functions and globals — and its body
   resolves names there, whoever calls it (RW 4.55). The calling interpreter's custody, brokers and effect sink stay: only the
   NAMES move, so a host callback inside an export answers to the host's custody as before (PLUGIN-CUSTODY-1), now running the
   host's code.
2. **Not chosen: refusing function-typed parameters in Verified exports** (Contained's R-6a rule, DL0803). R-4 admits callbacks
   to Verified plugins because their code is re-proved at load and the callback's row is real; refusing them would remove a
   designed feature to hide an interpreter defect, and would leave the reverse direction — a closure a plugin returns — still
   wrong.
3. **A closure without a home** (rebuilt from an actor message; the cycle collector's probe) resolves in the interpreter that
   calls it — an actor's worker runs the program's own module, so nothing changes for actors.

## D-V2-108 — RW 4.56: a plugin's node gates each use its export makes, not only the call — TAKEN (head chef, 2026-10-10, under the owner's delegation)

1. **The gate.** A plugin export's nested interpreter holds the host's custody behind a check of the plugin's node at every
   use (`PluginUseCustody`). D-V2-104 item 4 stands — the export's uses are checked against, and recorded on, the run's node,
   because the capabilities it uses are the ones the host handed it — and gains what it lacked: the operator's revoke of the node
   `grants list` shows as the plugin's reaches the export's next use, as the CLI's "before the next use" had promised.
2. **What a refused use is.** An actuator command is told `LeaseRevoked` (the value an e-stop gives, 10e's law); any other effect
   faults DL0801 (a call through a revoked plugin reference) — the same code the export's next CALL would have met.
3. **Not chosen:** checking the export's uses against the plugin's own node (F-02) — it would refuse every capability the host
   hands a plugin with an empty grant, the shape callbacks and handed actuators rely on; and changing the CLI's message to "the
   next call" — weaker, and an e-stop's handle must stop a loop.

## D-V2-109 — RW 4.58: a daemon operation whose audit record was lost never answers as if it had been recorded — TAKEN (head chef, 2026-10-10, under the owner's delegation)

1. **A revocation is never refused for want of its record** — refusing an e-stop would be the unsafe direction — **but it says
   so:** `Response::Revoked.unrecorded` carries the reason; `grants revoke` warns (`warning[DL1401]`) and exits 1, a monitor's
   quarantine reports it and exits 1, a fired dead-man logs it.
2. **A mint whose record was lost is refused and withdrawn.** Authority the chain cannot account for is what invariant 26
   refuses for a synchronous-class use; a new node is the same case. It is removed from the tree outright — a leaf just minted,
   with no second record (a revocation record would meet the same held lock). Not chosen: revoking it (two more lock waits, past
   a client's 15 s deadline — the first form of this fix, which its own witness timed out).
3. **Observability, not enforcement still holds** (invariant 26, trap 6) in the sense it was written: no DECISION reads the
   chain. What changes is the ANSWER — a record that was not written is never claimed.

## D-V2-110 — RW 4.59: a sandboxed guest's devices start at its first request after its program, not at the send — TAKEN (head chef, 2026-10-11, under the owner's delegation)

1. **When a run's devices start is "just before the program runs", on every path.** An ordinary run starts its device broker
   after its own check, just before `main` (`run_cmd.rs`). D-V2-95 item 4 started a guest's at the SEND, which kept a microVM's
   boot out of the first heartbeat but left the guest's own check of the program in it: a program whose check outlasted the
   heartbeat lost its arm before its first command in a guest and not unsandboxed — the parity witness's claim, broken by the
   program's size alone (witnessed red at `b0a520c` on Linux, Windows and macOS). Now
   `HostChannel::with_devices_at_first_request` starts them at the first request the host answers after the program is sent
   — the first instant the host KNOWS the program is running.
2. **A guest gains nothing by asking late.** Until its first request it holds no capability at all (every capability is minted
   by the host on request), so a guest that delays its first request delays nothing but itself; the leases, the dead-man, the
   TTL and the e-stop's watch all start together, as in the ordinary run. The difference that remains, stated: a guest's pure
   computation before its first request is not counted against its first heartbeat, where an ordinary run's is. No device
   moves in that time on either path.
3. **A guest that never asks never starts its devices, and its device nodes are revoked all the same**
   (`revoke_device_nodes` on that path): the nodes are the run's, minted before the guest existed. This also covers a guest
   that was never sent its program, which left its device nodes live before (code reading; no witness of that path).
4. **Not chosen:** a longer first window (a number with no meaning but "long enough here"); a new `Started` frame from the
   guest (a protocol version for what the first request already says); starting at the first device COMMAND (a guest could
   mint its actuator, compute, and hold it unbeaten until it chose — the first request of any kind is the earlier,
   stricter point). And never a longer heartbeat in the witnesses to make them green.
5. **Corrected the same run, from its red-team pass (F3, HIGH):** moving WHEN the device broker starts moved the e-stop's watch
   with it — the watchdog first looked one tick after the late start, so an operator's revoke of the device node made while
   the guest computed let that tick's commands reach the arm (8–13 of 300). The broker now runs its first e-stop check
   synchronously as it is built (`device::estop_sweep`), before it can be commanded, on both paths. And (D1, RW 4.60) a guest
   that never asks leaves its devices unstarted and its report silent about them; the lease's TTL, like its heartbeat, counts
   from the late start — by design, since nothing is held until it is asked for.

## D-V2-111 — RW 4.58's rest: a lost device-refusal or adoption record is never answered as success; one wait for the chain per request — TAKEN (head chef, 2026-10-11, under the owner's delegation)

1. **`DeviceRefused` answers DL1401 when its record was lost** — the command stays refused (the envelope refused it before the
   broker was asked), and the client's existing path says once on stderr that the chain lacks the refusal. `Recorded { seq }` is a
   claim about the chain; it is never made for a seq that names nothing (D-V2-109's rule).
2. **An adoption is a mint across a broker boundary**, so a lost `adopt` (or its node's `issue`) record refuses it DL1401 and
   withdraws the node (`unrecorded_mint`), as D-V2-109 does for `Issue`, `Attenuate` and `Delegate`. **The withdrawal takes the
   adoption's bookkeeping with it** (`withdraw_unrecorded` clears the certificate's `adopted` entry and the chain's fingerprints):
   a lost record must not burn a credential — the same certificate adopts once the chain is free.
3. **One wait for the chain per request** (`ChainLock::NotFree`): a request that found the append lock not free within its wait
   does not wait for it again; the serve loop forgets it when it releases the request's lock. Each seq and each record used to
   ask afresh, so an adoption waited 20 s on a held chain (past its client's 15 s deadline, answering `Adopted` to nobody) and an
   e-stop's revoke 10 s. After the first lost record the later waits changed no answer — only how late it came — and a request
   that found the chain not free writes none of its records, so the chain holds no half of it.
4. **Not chosen:** refusing an arm or disarm of the dead-man for a lost record (an arm adds safety; refusing a disarm keeps a
   dead-man a departing monitor cannot beat — each needs its own answer, left open in RW 4.58); retrying the lock later in the
   request (the lost record is already lost).

## D-V2-112 — RW 4.46: an egress refusal is a `deny` record citing the `use allow` it overrides — TAKEN (head chef, 2026-10-11, under the owner's delegation)

1. **D-V2-100's shape, for the network.** The broker answers whether a node may use a HOST and records `use allow`; the egress
   client then decides what the broker cannot see — what the name resolved to, the URL's userinfo, where a redirect points —
   and its refusal reached only the run's report and stderr. Now `Custody::note_egress_refusal` is called where the
   interpreter (`interp.rs`) and a guest's host (`channel.rs`) perform a fetch the gate allowed, once per refusal the egress
   client logged for that URL (`egress::refusals_since`), and the broker records `use deny` on the run's node at the host with
   `refused_by: egress`, the egress client's machine-readable reason, and `overrides_seq` — the `use allow` it overrides
   (`EgressRefused`, `Broker::record_egress_refusal`). A lost record is DL1401, never `Recorded` (D-V2-111).
2. **A refusal, not a failure.** Only what the egress client REFUSED (its reason's `answer()` is `Refused`: special-use,
   userinfo, malformed, not-allowlisted, no-address, too-many-redirects, host-mismatch, no-client) becomes a `deny`. A timeout,
   a TLS or connection error or an HTTP status is a use that was allowed and failed — a `deny` would say the authority refused
   it, which it did not.
3. **The witness needs no resolver seam.** A URL's userinfo is refused by the egress client before any resolution or
   connection, on every operating system, after the gate allowed its host — so `monitor_cli`'s witnesses run hermetically on
   every runner. A special-use refusal takes the same path; it was read by hand in the VM through a one-run `/etc/hosts` entry
   (embedded and sandboxed). The scheme refusal (`http://`) stays unrecorded: no use was authorized for it (HTTP-SCHEME-1).
4. **What this unblocks:** P8-04's `special-use` rule — a monitor can now count a run's refused fetches.
5. **Corrected the same run, from its red-team pass (F1/F2):** the first form read a fetch's refusal from the run's egress
   LOG, which keeps its first 64 records — so from the 65th request of a run on, refusals left an `allow` and no `deny`, and a
   `special-use` monitor was walked past by spending 64 requests first (no quarantine, the run's node left live). The refusal
   now comes from the call's own outcome: `get_for_program` keeps the calling thread's last refusal whatever the log's bound,
   and the gate takes it after the effect (`egress::fetch_refusal`). **Never read a decision's evidence from a store that
   stops keeping.**

## D-V2-113 — P8-04 step 4: the monitor's `special-use` rule; every single-record rule decided by one function — TAKEN (head chef, 2026-10-11, under the owner's delegation)

1. **`special-use` fires on any special-use reach** in the watched subtree: a `use` `deny` with `refused_by: egress` and
   `reason: special-use` — the record D-V2-112 made the chain hold — quarantines the run at the FIRST one, as `envelope` does at
   the first envelope refusal (`V2_OPENSHELL_STUDY.md` §4.7's "any special-use reach"). A run under a monitor that tries to
   reach loopback, a private range, link-local or a cloud metadata address — the shape of a server-side request forgery — is
   stopped by a second, out-of-band line, beside the egress client's own refusal. Only that reason: the egress client's other
   refusals (a URL's userinfo, a redirect off the grant) count toward `denies=N/MS`, not here.
2. **One function decides every rule that one record fires** (`Rule::fires_on`, unit-tested), and the watch loop has no
   per-rule arm left to drift from it; `denies=N/MS` counts records instead. Read only from what the HOST recorded
   (`authority`'s `refused_by` and `reason`), never from words a guest chose.
3. **Its witness is split, and says so:** the rule's matching is unit-tested; the single-record path it shares with `envelope`
   is witnessed end to end by a new `monitor_cli` test — no integration test had ever seen `envelope` FIRE (mutant M233, the
   rule never firing, passed the whole target; the tests that used it assert absences); the special-use case itself needs a
   name resolving into a special-use range and was read by hand in the VM through a one-run `/etc/hosts` entry.
4. **Left of step 4:** `break-glass` — its record (`audit_required`) carries no node, so a monitor cannot attribute it to a run.

## D-V2-114 — RW 4.61, the guard bypass: a `guard bypass on` whose record was lost does not happen — TAKEN (head chef, 2026-10-11, under the owner's delegation)

1. **A security DOWNGRADE the chain cannot account for does not happen.** `guard bypass on` whose `guard_bypass_on` record could not
   be written (another writer held the chain past the lock's wait) is undone in the same request — the guard back in force,
   `withdraw_unrecorded_bypass`, no second record (the chain that refused the first would refuse it too) — and refused DL1401. No
   client saw the bypass in between: the daemon serves one request at a time.
2. **An UPGRADE stays in force and says so:** `guard bypass off` whose record was lost leaves the bypass off and answers DL1401 —
   "the guard bypass IS off, but the chain lacks this record". D-V2-109's revocation rule: never refuse the safe direction for want
   of its record, never claim the record.
3. **Left of RW 4.61:** the guard's policy edits (a seal unset is a downgrade, a seal set an upgrade), approvals and permits (a
   downgrade each), requests, the dead-man's arm and `Redeem` — each its own answer by the same rule.

## D-V2-115 — RW 4.61's rest: every daemon operation whose audit record was lost — a loosening undone, a step whose answer would be a claim withdrawn, a tightening in force and said — TAKEN (head chef, 2026-10-12, under the owner's delegation)

1. **The class, enumerated from the writer's side.** RW 4.61 named eight handlers that never read the daemon's `failed`
   flag (after D-V2-114's bypass): `GuardPolicySet`/`Unset`, `GuardRequest`/`Approve`/`Deny`, `GuardPermitRevoke`,
   `ArmDeadman`, `Redeem`. Every `record_op` site in `delulu-broker` was then read against the handler that reaches it:
   four more wrote a record and answered as if it were written — `DisarmDeadman`, `Renew`, `SecretMap`, `RotateKey`. All
   twelve now answer by D-V2-109/111/114's rule, three ways.
2. **A LOOSENING the chain cannot account for is undone in the same request and refused DL1401** (requires a human): a
   policy edit that lowers a rule's tier or removes a rule (`GuardPolicy::loosened_by` — a rule ADDED never loosens, since
   every gate takes the strongest tier of the rules that match it; the policy is put back in memory and never persisted); an
   approval (its permit withdrawn, its request pending again); a dead-man's disarm (re-armed exactly as it stood — from a
   disarm on the runs below outlive their monitor, and the chain must say when that began; so a monitor whose clean end
   cannot record its disarm leaves its runs to be revoked at the period, the safe direction); a redemption (a single-use
   token unspent, the node's holder as it was — it redeems once the chain is free); a renewal (the adopted node's deadline
   back where it was: a receipt widens the one revocation bound that survives a partition).
3. **A step whose answer would itself be a claim about the chain is withdrawn and refused DL1401:** a guard request (an
   approval of it would approve something the chain never saw asked — the approval's record names the permit, not the
   subset or the `why`; its agent may ask again); a dead-man's arming (the red team's F5b: `armed at seq N` named no record,
   and a firing would have cited that seq as its arming — refused at a monitor's start, before its watch begins, so nothing
   that was in force is lost; this departs from RW 4.61's plan line "the dead-man's arm says so", for that reason); a
   `Secret.map` (a synchronous-class use, refused as an expose and a verify are — invariant 26 — and its derivative, named by
   a counter and so guessable, removed from the store).
4. **A TIGHTENING stays in force and says so** (DL1401, no human needed — D-V2-109's revocation rule: never refuse the safe
   direction for want of its record, never claim the record): a policy edit that adds or raises a rule, or changes nothing
   (persisted); a denial; a permit's revocation; a key rotation (it only kills tokens).
5. **The undo is explicit, not remembered.** Each loosening returns what it replaced (`Broker::redeem_undoable`,
   `renew_undoable`, `disarm_deadman_undoable`; the daemon snapshots the policy before an edit), and only the daemon puts it
   back, in the request that made the change — so no client saw the change in between (the daemon serves one request at a
   time). An in-process broker's answers are unchanged.
6. **Not changed:** a beat's firing — the daemon fires every due dead-man before it handles the next request, so the beat's
   own firing path is reachable only in a race no witness can hold open; `fire_deadmen` already says when a firing's record
   is lost (§11.5, M222: no check a witness cannot fail). The daemon's startup records (its start, `root-policy-mode`).
7. **Witnessed** — seven daemon tests under a held chain, each red on the code before (`a_guard_policy_edit_…`,
   `a_guard_approval_or_request_…`, `a_dead_mans_arming_or_disarming_…`, `a_redemption_…`, `a_renewal_…`, `a_secret_map_…`,
   `a_key_rotation_…`); mutants M241–M257 red.

## D-V2-116 — P8-04's `break-glass` rule: a break-glass run's use is recorded on its own node, and a monitor fires on it; P8 complete as far as software reaches — TAKEN (head chef, 2026-10-12, under the owner's delegation)

1. **The record a rule can judge.** `breakglass::gate` records the glass broken (`break-glass` `allow`, the ticket's
   fingerprint as its target) before the run reads a line of its program — and before a lease run has a node, so the
   record names none, and a monitor judges only records that name a node in its subtree: a run that broke the glass under a
   monitor was invisible to it. Now a lease run, once its lease is redeemed and before its program runs, asks the daemon to
   record its use on its own node — `break-glass-run` `allow`, actor the run's node, target the same fingerprint, `relax` in
   the payload (`ReqBody::BreakGlassRun`, `Broker::record_break_glass_run`). Two records, not one moved: the gate's stays
   where it is, the moment the ticket is spent and the evidence of an attempt whatever follows; the second binds that use to
   the node. A distinct action, so a count of `break-glass` uses stays one per ticket.
2. **A use that cannot be recorded does not happen** (PS-B-06's rule, kept): the daemon answers a lost record DL1401 (D-V2-115's
   form) and the run refuses itself, exit 2, before a line runs. The node must be in the tree; nothing is recorded about one
   that is not.
3. **`--rule break-glass`** fires on that record — the one rule an `allow` fires. The watch loop used to skip every record that
   was not a `deny`; it now hands each record to the rules, the refusal rules (`envelope`, `special-use`) read the decision
   themselves, and `denies=N/MS` counts refusals only. A ticket the operator signed is still a reason to quarantine under a
   monitor whose operator asked for this rule: that subtree is one where nothing runs outside the sandbox unwatched.
4. **P8-04 step 7's last witness**, over the wire: a `Quarantine` of a node outside the caller's subtree — across to a
   sibling root, or upward from a child — is `NotRevocable` (DL0904) and both stay live (M265 red).
5. **P8 complete, as far as software reaches** (taken here, recorded with its push run's read): P8-01 to P8-04, every step of
   `V2_P8_DESIGN.md`'s build orders built and witnessed against the simulator and `delulu device sim`. Named and kept open
   where they are recorded: a real device (environment-blocked — no hardware in a cloud VM), the monitor's residuals (RW 4.49),
   a daemon request's waits on the chain beside busy hosts (RW 4.53, with the red team's F4 and F6), a killed run's nodes
   under the daemon (RW 4.57), a guest that never asks (RW 4.60) — and option (c), a revoke-only principal in the tree, which
   would make "a monitor holds exactly revoke" exact and is the owner's (it changes who may revoke). **Next: P9.**
6. **Witnessed** — `monitor_cli`'s `the_break_glass_rule_quarantines_the_run_under_a_ticket_and_no_other` (a plain run under the
   same monitor is read and left alone; the evidence is the `break-glass-run` record on the run's node, naming the gate's
   ticket); the daemon's `a_break_glass_run_whose_record_was_lost_is_refused`; `a_quarantine_outside_the_callers_subtree_is_refused`;
   `monitor_cmd`'s two unit tests. Mutants M258–M265 red: no binding record, the loop's `deny` filter restored, the rule firing on
   any `allow`, on the gate's action, the daemon claiming a lost record, `denies` counting `allow`s, a record without its node,
   the tree's reach check gone.

## D-V2-117 — RW 4.60: a guest's devices that never started are journaled `device.not-started` — TAKEN (head chef, 2026-10-12, under the owner's delegation)

1. **Say it in the report, in the shape it already has.** A sandboxed guest that never asks its host for anything after its
   program never starts its devices (D-V2-110 — by design: until it asks it holds no capability). Its report now journals each
   planned device as `{"op": "device.not-started", "device": NAME, "at_ms": 0, "detail": …}` in `devices` — the run-report
   schema's `device_event`, so no field changes shape; `at_ms` is 0 because no broker started to count from, and the detail
   says so. A guest that asks starts its devices and its journal is the watchdog's, as before.
2. **Not taken: minting the device node at the first request too** (RW 4.60's other suggestion). The node is the run's — an
   e-stop aimed at it reaches the run — and it is revoked when the run ends (`a_guest_that_never_asks_leaves_no_device_node_live`);
   minting it later would move every check that starts with it (§11.5's lesson from F3), for no authority gained.
3. **Witnessed** — `sandbox_devices_cli`'s `a_report_says_when_a_guests_devices_never_started` (red before: no `devices`
   entry; the control, a guest that asks, has no `device.not-started`); M266 (no names) and M268 (journaled even when started)
   red; M267, a second "none once they started" check, survived because the report's branch never reaches it — removed.

## D-V2-118 — P9-01: `delulu authority FILE --within BOUNDARY` — each request decided by the runtime's own mint; a scope the source does not show is never *within* unless the boundary bounds its whole kind — TAKEN (head chef, 2026-10-12, under the owner's delegation)

1. **The question, as the study put it** (`V2_OPENSHELL_STUDY.md` §4.8): given a boundary in the `--grant` grammar, does
   everything a program asks for fall inside it? Four answers, a closed enum — `within` (exit 0), `exceeds`, `unsupported`,
   `inconclusive` (exit 1; a bad invocation is 2, as everywhere). `exceeds` names a counterexample: the first request in SOURCE
   order the boundary does not cover, with the call that makes it (file, line, column — and in the human answer the line itself,
   escaped). `--json` carries `authority.within {result, boundary, counterexample, findings, coverage, requests}`: every request
   that is not within is a finding, and the coverage names each of the grammar's fifteen dimensions (`not-requested` where the
   program asks for none). One exit code for the three non-`within` answers keeps the CLI's 0/1/2 contract; the answer is named.
2. **The program's side is the list `--grants` prints.** `required_grants` is now `requirements(report, walk)` structured — each
   line with what it asks the runtime for (`Ask`) and its call (`ScopeWalk` keeps the first call of each literal, of each kind's
   first hidden site, of each kind's first mint, and — where nothing mints `Declassify` — of a secret's first `verify`/`expose`).
   One list, so the flags an operator types and the verdict an agent reads cannot disagree about what is asked for; the report is
   byte-identical (`core_invariance`, `atlas_chain`).
3. **Each request is decided by the runtime's own mint, not by the broker's lattice — a departure from `V2_P9_DESIGN.md` step 2,
   taken because the lattice is coarser than a grant set.** The boundary is parsed by `Grants::add` (the parser `run --grant`
   uses) and every literal request is put to `prim::call_root_method` against the root it builds — the decision `run --grant
   BOUNDARY` makes at that call, its refusal quoted in its own words. The broker's `Authority` folds `console` into `Write`
   (`authority_spec_from_grants`) and has no `net.special`, sensor, compute or plugin dimension, so a verdict read through
   `attenuation_check` calls a printing program within an `fs.write`-only boundary that `run` refuses — pinned by
   `a_printing_program_exceeds_a_boundary_that_grants_only_a_write`. Where the two speak of the same thing (paths, hosts,
   secret names), `the_verdict_agrees_with_the_lattice_where_both_speak` holds them to one answer over 400 generated pairs
   (with a control: both answers must occur). A special-use host is within only under `net.special=` (D-NE-28). A foreign
   library is decided by its logical name, a Python import by the runtime's allowlist matcher (a manifest's pattern by exact
   string, the broker's rule).
4. **A scope the source does not show** (a hidden site — SCOPE-HIDDEN-1, SECRET-HIDDEN-1 — or a kind with no minting call in the
   file) is `exceeds` against a boundary that grants none of its kind, `within` only against one that bounds the whole kind (on
   Unix, `fs.read=/` or `fs.write=/`; no other dimension has a widest form), and otherwise `inconclusive`, named. The study
   defined *inconclusive* as "a resource limit reached"; this widens it to "no static answer", as `V2_P9_DESIGN.md` proposed. A
   plugin load is `inconclusive` against any `plugin=` root: its artifact is checked at its load, against the roots and the
   manifest's hash list, not by this reading.
5. **`unsupported`** is empty on today's language — each of the ten capability kinds a report lists has a grant form, and
   `Declassify`, `ForeignLoad` and `Python` have theirs — and stays for a kind added later: `requirements` names an unknown kind
   instead of dropping it in its skip branch, and `every_capability_kind_has_a_grant_form_and_an_unknown_one_is_unsupported`
   holds an exhaustive match over `ResourceKind` (a new kind does not compile until it is placed).
6. **The surface:** `--within GRANT` (repeatable) and `--within-file FILE` (one grant a line; `#` comments and blank lines
   skipped — and an empty file is the empty boundary, which no flag can spell). Refused by name, exit 2, beside `--diff`,
   beside `--grants` (two questions), on a package directory and on a `.dwx` (not checked against a boundary yet), with a bad
   grant, and on every other command (the help documents it for `authority` alone). Not a new subcommand: `authority` stays
   `READ_ONLY` in `mcp.rs`'s door rule — a supervising agent may ask it of a sub-agent's program before running it.
7. **Witnessed** — `within_cli`: every corpus program within its own required grants (operator data filled in: a secret's
   value, a library's path, an actuator's envelope) and `exceeds` each one-grant narrowing, the counterexample the dropped grant
   and its line the call that asks for it (19 programs, 27 narrowings, with a control); `10_actuate_errors` exceeds with no
   device and is inconclusive with one; a computed secret name exceeds with no secret and is inconclusive with the right one; a
   boundary file says what the flags say; six refusals; both answers validate against the authority schema. `within.rs`'s seven
   unit tests. Mutants M269–M280: a dimension decided within regardless, the counterexample's call taken from the wrong kind, a
   hidden site answered within, the special-use rule dropped, SECRET-HIDDEN-1's placeholder dropped, a helper's literal secret
   hidden, an unknown kind dropped, the outcome's precedence inverted, the `verify`/`expose` site dropped, the `--grants` and
   package refusals dropped, the source order reversed.
8. **Not taken here:** a package directory and a `.dwx` (each needs its own requirement list: a package's merged modules, an
   artifact's embedded manifest with no spans); P9-02's `grants diff`, which reads this verdict between two authorities.

## D-V2-119 — P9-02, first slice: `delulu grants diff OLD NEW` between two grant files — what a change ADDS, by category, decided as `--within` decides — TAKEN (head chef, 2026-10-12, under the owner's delegation)

1. **The verb** (`V2_OPENSHELL_STUDY.md` §4.9): `delulu grants diff OLD NEW [--json]`, OLD and NEW each a file of grants, one a
   line (the `--within-file` format: blank lines and `#` comments skipped). Offline — no broker is asked. Every finding has a
   category from a closed set and no severity grade: `effect_expansion` (a dimension of the grant grammar OLD grants none of),
   `scope_expansion` (a wider scope in a dimension OLD grants), `special_use_reach` (a `net.special=` host OLD does not name),
   `device_reach_expansion` (a device OLD does not name, or an envelope wider than OLD's). Only what NEW ADDS is a finding; a
   line NEW shares with OLD, or narrows, is none.
2. **Decided as P9-01 decides** (D-V2-118): each NEW line is put to the runtime's own mint against the root OLD builds — a path
   by on-disk containment, a host by the mint's exact match, a flag by the flag — so a diff and a `--within` answer cannot disagree
   about what OLD covers. The mint reads a device's NAME only, so an actuator's envelope is decided by the broker's device lattice
   (`device_scope::all_within`: interval containment, rate, heartbeat and lease), and a compute device's by its bounds (memory,
   queue depth, kernel time, power), its adapter, its formats, its kernel names and its attestation waiver (a waiver OLD did not
   make is reach added — DL1911). A dimension is "effect" in the grant grammar's sense: `console` is its own, though the broker
   folds it into `Write` — D-V2-118's reason.
3. **Exit 1 when the change adds anything, 0 when it adds nothing, 2 on a bad invocation** — unlike `authority --diff`, which
   reports a widening and exits 0: this is the check a CI gate, and P9-03's auto-approval, read without parsing the report.
4. **The door rule is unchanged:** `grants` stays an EFFECTOR in `mcp.rs` (it mints and revokes), so this read-only verb is not
   reachable from an MCP tool. That is the safe direction; a per-verb door rule would be the change that exposes it, not taken.
5. **Not taken here:** lease nodes (`g_…` is refused by name — a node's authority is the broker's, and `budget_expansion` lives
   there); `secret_reach_expansion`, which waits for P9-04's endpoint-bound secrets; a foreign library's or a kernel's changed
   binary PATH — the broker compares names, and a path swap is not a `⊑` widening, though it is worth a finding of its own later.
6. **Witnessed** — `within_cli`'s `grants_diff_names_what_a_change_adds_and_nothing_it_does_not`, red first (no verb, exit 2):
   every grant form's own example, read from `toolchain --json` (so a form added later is in the witness the day it exists), is
   one finding of its own dimension and category against nothing and none against itself; a subdirectory and an unchanged host
   add nothing while another directory, another host and a wider envelope each add one; the reverse change is no narrowing and
   says so; a compute device's larger memory, extra kernel and waived attestation each add device reach, a smaller ceiling
   nothing; the human answer; a lease node refused as a node. `json_contract` drives it in its success table and its broker-verbs
   sweep (every advertised `grants` verb must be driven). Mutants M281–M289 red: the device lattice replaced by names, every
   expansion called an effect expansion, a path decided by presence, the waiver, the memory bound and the kernel names unchecked,
   the exit code, the node refusal and the host decision dropped.

## D-V2-120 — P9-02 between two lease nodes: `delulu grants diff g_OLD g_NEW`, decided by the lattice, budget included — TAKEN (head chef, 2026-10-12, under the owner's delegation)

1. **The second form of the verb** (`V2_OPENSHELL_STUDY.md` §4.9 names "grant sets or lease nodes"): two `g_…` ids are read from
   the broker that holds them (`Inspect`) and compared; a file beside a node is refused (exit 2) — they are two domains, a node
   carrying a budget and no `console` flag, a grant file the reverse. D-V2-119's exit codes and categories hold, and one is
   added: `budget_expansion` (a budget OLD's node does not bound; an absent budget is the top of the dimension — PS-B-05).
2. **Here the lattice decides** — the opposite of D-V2-118/119, for the same reason: a node's authority IS the broker's
   `Authority`, so `attenuation_check` is the enforcement's own answer. Each item of NEW is asked as a one-item authority (an
   effect by membership; a path, host, name or library through its dimension; a device by its envelope), so every finding names
   one item rather than the whole intersection `attenuation_check` returns.
3. **The probe carries OLD's budget.** A one-item probe built with no budget has the TOP of that dimension, and `⊑` fails it
   against any budgeted OLD — every item of NEW would read as an expansion. The budget is decided once, by `budget_scope::within`.
   Mutant M290 (a probe without it) is red.
4. **Witnessed** — `within_cli`'s `grants_diff_between_two_nodes_names_effect_scope_device_and_budget_reach`, on a real daemon
   with four delegated nodes: seven findings one way (two effects, three scopes, a device, a budget) and a subdirectory adding
   nothing; one the other way (OLD's `./data` is wider than NEW's `./data/sub`); a wider envelope on a device both grant; an
   unbudgeted node beside a budgeted one; none against itself; a file beside a node refused. Mutants M290–M295 red (M291 is the
   form before it: node mode refused), the control green.
5. **P9-02 is built** but for `secret_reach_expansion`, which waits for P9-04's endpoint-bound secrets (no secret has a
   destination to widen until then).

## D-V2-121 — P9-03's proposals are computed from the refused call's span; step 1: a refused run's report records the refusal at its call — TAKEN (head chef, 2026-10-12, under the owner's delegation)

1. **Measured first** (`V2_P9_DESIGN.md`, P9-03): the study's premise that a refusal "already carries a structured repair" is
   false for DL0703 — the grant is prose (`repairs: []`) — and `run --report-out` recorded no diagnostic for a refused run.
2. **The decision:** a proposal is never built from a grant line the runtime or the program words. It is the requirement P9-01's
   `requirements` give for the refused call's SPAN — the static authority of the call site, as §4.10 asks. So the refusal needs
   to travel with its span, not with a repair: no new repair kind, no prose parsed.
3. **Step 1, built (`c2dd5f0`):** the ordinary run's report carries the fault that ended the run as the envelope's own
   diagnostic object (code, message, the call's span) in the `diagnostics` field it always had and left empty — additive, the
   run-report schema unchanged. A clean run's report carries none. A sandboxed run's report is written by its host and is the
   next step, with `grants propose --from REPORT` after it.
4. **Witnessed** — `within_cli`'s `a_refused_runs_report_records_the_refusal_at_the_call_the_verdict_names`, red first: the
   report's span is the very `at` `authority --within` names for `fs.read=./config` (48:18). Mutants M296 (the report ignores
   the fault) and M297 (the fault recorded before its span) red, the control green.

## D-V2-122 — P9-03 step 1b: a sandboxed run's report records the HOST's refusal at the call the guest's request named — TAKEN (head chef, 2026-10-13, under the owner's delegation)

1. **Measured first:** `run FILE --sandbox --grant console --report-out R` on `examples/guide/05_capabilities.delulu` wrote
   `diagnostics: []` (`outcome: {exit: 1, ran: true}`, the refusal in `sandbox.denied` as prose and on the guest's standard
   error), where the ordinary run's report names DL0703 at 48:18 since D-V2-121; the greeter package the same.
2. **Whose words.** A sandboxed run's report is the HOST's (D-V2-21), and the host refuses each capability request itself —
   so the refusal it records is its own: its code and its words (bounded and escaped as a `denied` entry is). The CALL is the
   guest's word: `RootMethod`/`CapMethod` carry the span the guest's interpreter names. The host places it only on character
   boundaries of the one program it sent (a span inside a character panicked the renderer — a guest could have ended its own
   host's report), and a span naming another file is omitted by the renderer itself (C83). A lying guest can point only at
   another place in the same program, whose static requirement is still the program's own — and a proposal built from it is
   still approved under the approver's `⊑` (P9-03's later steps).
3. **Which refusal.** The host keeps the refusal of the guest's LAST capability request and clears it when a later one is
   answered `Ok`. A DeluluLang program ends at a fault, so the refusal still held when the guest says goodbye with a failing
   exit is the one that ended the run; it is recorded only then. A guest that faulted on its own (an assertion) ended for a
   reason the host never saw — its words stay its own (RW 4.31's rule), and the report records no diagnostic for it; a guest
   whose channel failed was ended by something else. An egress or a device refusal is a value the program goes on with, and
   is answered `Ok`.
4. **The name.** The report names the program's file as the ordinary run does — the file as typed, or a package's flattened
   text (`run_cmd::flattened_name`, now the one place that spells it) — so the two reports name one call alike.
5. **Witnessed** — `within_cli`'s `a_sandboxed_refused_runs_report_records_the_hosts_refusal_at_the_ordinary_runs_call`, red
   first: for `05_capabilities` and for the greeter package, the sandboxed report's code, message, file, start and end are the
   ordinary report's; a clean sandboxed run's report carries none. A channel unit test (held at its call, cleared by an `Ok`,
   kept by a goodbye, a channel error held too) and a guest unit test (hostile spans — inside a character, past the program,
   backwards, another file — recorded unplaced; nothing for a clean exit or a failed channel). Mutants M298–M306 red, the
   control green. Read on Windows, macOS and arm64 before `master` moved.

## D-V2-123 — P9-03 step 2: `grants propose REPORT` proposes the refused call's own static requirement, read through every minting call — TAKEN (head chef, 2026-10-13, under the owner's delegation)

1. **What it proposes.** The `--grant` line(s) of the requirement of the capability-minting call at exactly the report's
   refusal span, as `authority --grants` names them (`within::propose` filters P9-01's requirement list by that call's kind
   and literals). Never a line the runtime or the program worded: the refusal's message is not parsed, its repairs are not
   read. One call's requirement and nothing wider — a call that asks for two hosts at once proposes both, because that is
   that call's requirement (the report records no grant set — `sandbox.granted` is the sandbox level — so a proposal is not
   a difference from what the run held; `grants diff BOUNDARY PROPOSAL` is where an approver asks what it adds).
2. **Every call, not each grant's first** (measured while step 1b was built): the requirement list keeps each grant's
   FIRST call (`push` deduplicates by grant), so a refusal at a second call of the same grant matched no requirement's
   `at`. The Survey walk now keeps every minting call (`ScopeWalk.sites` — kind, literals, hidden), and the report's
   fields are unchanged by it.
3. **When it proposes nothing** (exit 1, a `reason`): no refusal in the report; a code a grant does not answer (only
   DL0703 — every DL0703 is raised at a mint, `prim::call_root_method`); a refusal the report does not place; a package's
   flattened text (single files, as `authority --within` — both open together); a program that does not check; a call that
   computes its scope (no static narrowest grant — the operator chooses; the shape, `fs.read=PATH`, is named); a mint whose
   grant names what only the operator holds (a library's or an artifact's path); and a program that changed since the run.
   The report is input: its offsets are checked to lie on character boundaries of the program as it is now before anything
   slices it (mutant M312, without the check, panicked the binary).
4. **Read-only, and under `grants`.** It reads two files, asks no broker, writes nothing. `grants` stays an EFFECTOR in
   `mcp.rs`'s door rule (the command is classified, not the verb); the json contract drives the verb in its success table
   and in the broker-verbs sweep, whose advertised-verbs check refused the verb until it was driven. The report is a
   positional, not `--from`: a flag documented for a command reaches every verb of it (routine run 8).
5. **Witnessed** — `within_cli`'s `grants_propose_reads_the_refused_calls_own_requirement_from_the_report` (red first: no
   verb): the guide's refusal from an ordinary and a sandboxed report (`fs.read=./config`, the `at` `authority --within`
   names); a refusal at a second call of a grant; nothing for a computed scope, a changed program, offsets past the end, a
   clean run, DL1403, and exit 2 for a file that is not a run report. And, generated, `grants_propose_names_the_dropped_
   grants_call_for_every_kind`: a program minting eight requirements across seven kinds, run with all but one, eight times —
   each proposal is exactly the dropped call's requirement. `schema_cli`'s `a_refused_runs_report_matches_the_run_report_
   schema` (ordinary and sandboxed). Mutants M307–M316 red, the control green.
6. **Next (P9-03 step 3):** the store — a proposal kind beside the Guard's queue (a proposal asks for a child NODE, a Guard
   request for permits on a node's uses), approval minting through `attenuate` under the approver's own node, effective on
   the next run; auto-approval within a standing boundary is `grants diff BOUNDARY PROPOSAL` adding nothing (D-V2-119).

## D-V2-124 — `authority DIR --within` and `grants propose` read a package; a `.dwx` stays refused — TAKEN (head chef, 2026-10-13, under the owner's delegation)

1. **`authority DIR --within BOUNDARY`** decides the requirement list `authority DIR --grants` names — the same report and the
   same walk over every module (`emit_authority`, shared by the manifest and the loose-directory paths) — through P9-01's
   verdict; the counterexample's call is named in its module's own file, relative to the package, as `authority DIR` names
   a module. Run 20 refused a package in words until it could be answered; it can.
2. **`grants propose` for a package run's report** finds the refused call in the flattened text the run executed — the
   run's diagnostic names it `DIR (flattened for execution)` (`run_cmd::flattened_name`) and its offsets index that text —
   rebuilt by the run's own loader (`run_cmd::flattened_package`), and proposes the call's requirement read there. The
   package's own requirement list (its manifest's scopes included) was built first and removed: a proposal filters by the
   call's kind and literals, so a manifest's scopes — lines no call writes — never change it, and no witness could tell the
   two apart (M222's rule: code no witness can fail is deleted).
3. **A `.dwx` stays refused** for `--within`: it carries a manifest of effects and scopes, not the calls a counterexample
   names; `authority FILE.dwx` answers from that manifest, and a boundary over it is a question for later.
4. **Witnessed** — `within_cli`'s `a_package_is_checked_against_a_boundary_and_its_refusal_is_proposed_for`, red first (exit
   2): the greeter within `console`, exceeding nothing with the call in `src/main.delulu`; a proposal of `console` from an
   ordinary and a sandboxed run's report, at the place the run's diagnostic names. P9-01's generated corpus walks every
   package under `examples/`. Mutants M318 (the refusal back), M319 (the package path ignores `--within`), M320 (a package's
   report read as a file) and M321 (the walk over the first module only) red, the control green.

## D-V2-125 — P9-03 step 3: a proposal is a request filed with the broker against the refused run's node; its approval is a delegation under the approver's own node of that node's authority joined with the ask — TAKEN (head chef, 2026-10-13, under the owner's delegation)

1. **What is filed.** `grants propose REPORT --node g_ID` files step 2's proposal — the refused call's own requirement
   (D-V2-123) — with the broker as the request of `g_ID`, the node the refused run held (a `run --lease` run prints it).
   Each line is converted as a lease's grant is (`authority_spec_from_grants`); a line with no form in a node — a sensor or a
   compute device (a plugin and a computed scope propose nothing already) — is proposed but NOT filed, said in `not_filed`,
   exit 1, because an approval could not carry it. The broker refuses (DL0904, no new code) a proposal that asks for nothing,
   for what the node already holds, for a budget, or for a second envelope on a device the node has one for; and one from a
   node that is not live (DL1403, DL1402; DL0904 for an unknown one). The record `proposal` carries what is asked and the
   proposer's account — program, call, place, code — as data, each field bounded (512 bytes), never decided on. A proposal
   still pending for the same node and the same ask is answered with its id (`deduped`).
2. **Approval.** `grants approve pp_ID --parent g_ID [--ttl D] [--owner CODE]` delegates, under the APPROVER's node, the
   proposer's authority joined with the ask (effects and scopes unioned, the node's budget kept, paths re-canonicalized —
   `proposal::join`), through the Guard's mint gate (the owner code for guarded authority, as `grants delegate`) and
   `attenuate`'s `⊑`. So **a proposer approving under its own node is refused DL0802** — its widening is not `⊑` it — and an
   approver cannot grant past what it holds (DL0802). It is refused while the proposer's node is revoked or expired: a
   quarantined run's ask cannot bring its authority back. **The proposer's node is never changed**: the approval is a NEW node
   and a single-redemption lease token for the re-run (V2 §5). Records: the delegation's own `delegate`, then
   `proposal_approve` (the approver, the proposal, the new node). The join, not the ask alone, because a re-run needs what its
   node already allowed: a lease of the ask alone is refused at the first call the old node admitted.
3. **What an approver reads.** `grants proposals` lists each proposal's ask, the proposer's account, and — while it is
   pending and its node live — **what an approval would mint now** (`mints`, the join), so the approver reads the whole new
   node and not only the ask: a proposal names its node, and an approval carries that node's authority with it.
4. **Who may approve — no gate beyond the tree's.** An approval is a delegation, and a delegation is bounded by the node it
   is made under: a node id is a capability (any client naming a live node can delegate under it, as `grants delegate
   --parent` always could), so an approval grants nothing its approver could not already delegate, and a proposal that names
   someone else's node gains its proposer nothing it could not take by naming that node itself. The Guard's owner code is
   asked exactly where `grants delegate` asks it.
5. **Rejection.** `grants reject pp_ID --reason ".."` — a reason is required, kept (bounded at 2 KiB) for the proposer in
   `grants proposals --json`, recorded `proposal_reject`, a `deny` naming the proposer's node, as a Guard denial is. Not
   owner-gated: a rejection grants nothing (the same-user threat model, spec §10). A rejected proposal asked again is a new one.
6. **Lost records** (RW 4.61's rule, D-V2-115): a filing whose record was lost is withdrawn and refused DL1401; an approval's
   node is withdrawn and the proposal pending again, refused; a rejection stays in force and says the chain lacks its record.
   Daemon memory only, like the tree and the Guard's queue. OCSF: `proposal` and `proposal_approve` export as Base events,
   `proposal_reject` as a Finding (a deny) — the mapping unchanged.
7. **Witnessed** — `within_cli`'s `a_proposal_is_filed_approved_under_the_approvers_node_and_rejected_with_a_reason_the_agent_
   reads`, red first (`--node` a usage error, exit 2, after the lease run was refused as expected): a lease of the console only,
   refused at the guide's `./config` read; the proposal filed against its node, answered again while pending; filing changes no
   node; the proposer's own approval DL0802; the operator's under R a new node `{Read, Write}` under R; the proposer's node as
   it was; the re-run past the read to the next refusal, the `./out` write; R's own lack of a write DL0802; a rejection whose
   reason the agent reads back; a proposal whose node is revoked before approval mints nothing; a revoked node's ask not filed;
   a sensor's proposed and NOT filed. `brokerd`'s `a_proposal_or_its_approval_whose_record_was_lost_does_not_happen`; seven
   broker unit tests (`proposal.rs`), the Guard's mint gate among them; `json_contract`'s broker-verbs sweep drives `propose
   --node`, `proposals`, `approve` (under another tree's node) and `reject`. **Mutants M322–M336 red, the control green** —
   M331 (the lost filing kept) first SURVIVED: the witness filed the same ask again after the held chain, and the dedup
   answered the leftover's id; it now reads the list before anything else is filed, and M331 is red.
8. **Next (P9-03 step 4):** auto-approval within the operator's standing boundary (`V2_P9_DESIGN.md` item 4 — a proposal is
   auto-approvable exactly when `grants diff BOUNDARY` over what it would mint adds nothing, D-V2-119), then P9-04.

## D-V2-126 — P9-03 step 4: auto-approval is the operator's standing delegation — a node — which approves the proposals whose mint it bounds, never guarded authority — TAKEN (head chef, 2026-10-13, under the owner's delegation)

1. **The boundary is a node.** `grants auto-approve --parent g_B [--ttl D]` offers every pending proposal to the broker for
   approval under B, exactly as `grants approve` would: the tree's own `⊑` decides, against B, what each would MINT (the
   proposer's authority joined with its ask, D-V2-125), and the Guard's mint gate applies. The study's "a proposal is
   auto-approvable when it is `⊑` BOUNDARY and P9-02 finds nothing" is this one check: in the node domain, `grants diff
   B MINT` finds nothing exactly when the mint is `⊑` B (budget aside: an approval inherits B's budget where the mint names
   none — `attenuate`'s own rule). Why a node and not a grant FILE: what an approval mints is a node — a file would have to
   be translated into the node domain, where `console` is the `Write` effect, so a file granting `fs.write` would approve a
   console nobody wrote; and a node is the operator's delegation in the tree it governs, with its own TTL, its own audit
   record and its own revocation — **revoking B revokes everything it auto-approved** (each approval is B's child).
2. **What is left pending, and why.** Each proposal the broker refuses stays pending, listed with its code and message and
   what `grants diff` finds its mint adds beyond B (`within::diff_nodes`) — so the operator reads, per proposal, the
   categories that kept it from the standing delegation. The broker decides; the findings explain.
3. **Never guarded authority.** `auto-approve` takes no owner code — `--owner` is refused (exit 2, saying why) and
   `DELULU_GUARD_OWNER`, which `grants delegate` and `grants approve` read, is not read: guarded authority (declassify,
   native code by default) is approved by a person, one proposal at a time (`grants approve pp_ID --owner CODE`). A proposal
   within B that carries guarded authority stays pending with the Guard's DL1410.
4. **Exit codes.** 0 when nothing is left pending (nothing to approve is not a failure), 1 when anything is, 2 on a bad
   invocation. One pass approves what it can; it is a command, run when the operator chooses (or by a scheduler the
   operator owns) — no daemon-side automation, so nothing approves while nobody is running it.
5. **Witnessed** — `within_cli`'s `auto_approval_approves_under_the_standing_node_what_it_bounds_and_never_guarded_authority`,
   red first (no verb, exit 2): three agents' proposals for the guide's `./config` read against B (the read with the console,
   and `Declassify S`): the one within B approved under B (`{Read, Write}`); one whose node holds `Net` left pending, DL0802,
   with `effect_expansion Net` and `scope_expansion net=example.com`; one within B but guarded left pending, DL1410, though
   `DELULU_GUARD_OWNER` held the code; `--owner` refused; a second pass approves nothing; nothing pending is exit 0; revoking
   B revokes the approved node. `json_contract`'s broker-verbs sweep drives it. Mutants M337–M341 red, the control green.
6. **P9-03 complete** with this step (`V2_P9_DESIGN.md` items 1–4 and hypothesis 5 witnessed). **Next: P9-04**, secrets bound
   to endpoints (RW 2.11) — a language change (a request with headers), read against the code before it is built.

## D-V2-127 — DEADMAN-BUSY-1: a dead-man counts its holder's silence, never the daemon's — a beat is judged by when it arrived, and a monitor waiting on the daemon beats from a thread of its own — TAKEN (head chef, 2026-10-13, under the owner's delegation)

1. **The finding.** `11c9d94`'s push run read `monitor_cli` red on Ubuntu (`38353462220`, job `115150876593`): two healthy
   monitors revoked by their own dead-men (2,000 ms, the default floor) — in one, nothing at all reached the chain between the
   arming (seq 5) and the firing (seq 6), not even the test program's lease redemption: the daemon served nothing for over two
   seconds. Not reproduced in the VM (6 of 6 green, three with every CPU busy), and nothing this run had changed reaches the
   monitor or the serve loop. **The class, witnessed deterministically on the code as it was:** the daemon is serial, and it
   fired dead-men "before the next request is handled" (D-V2-106) — so while it was held on one request (here a request
   waiting on an audit chain another writer holds, RW 4.53's wait), beats a healthy holder SENT in time waited in its queue,
   and the dead-man fired as soon as the hold ended: `brokerd`'s `a_beat_sent_in_time_keeps_its_dead_man_though_a_held_
   request_delays_its_answer` — 25 beats every 100 ms against a 1,000 ms dead-man, the daemon held 2 s — RED, every beat
   answered DL1403. And a monitor waits on the daemon itself: its `List` queues behind the held request, and the beat it sends
   once the answer comes is late however healthy it is — `monitor_cli`'s `a_monitor_waiting_on_a_held_daemon_is_not_revoked_by_
   its_own_dead_man` — RED 2 of 2 with the daemon's half fixed. A false quarantine is the safe direction, but a dead-man that
   reads the daemon's own unavailability as its holder's death quarantines every run of a healthy monitor whenever the daemon
   is slow (RW 4.53 measured holds of seconds beside busy sandbox hosts).
2. **The daemon's half.** Each whole request is stamped with the instant it ARRIVED — read on a thread of its own (RW 4.40),
   which a held handler does not delay. Dead-men are fired as of the handled request's arrival (as of now on a wake with
   nothing waiting); a beat — and a disarm — is judged by its arrival; a beat's next period starts when it is answered, since a
   holder that waits for each answer cannot beat before it. D-V2-106's rules stand: a beat that ARRIVED after the deadline
   rescues nothing, and a stream of requests cannot hold an unbeaten dead-man off — it fires at the first request that arrived
   after its deadline (`a_busy_daemon_still_fires_an_unbeaten_dead_man` still green). The cost, bounded: a backlog delays a dead
   holder's firing by the requests queued ahead of the first late one (at most `MAX_READERS`, 64). Arrival is stamped after a
   request is read whole and before it is queued, so two readers can race by microseconds — the window a misfire needs went
   from seconds to that.
3. **The monitor's half.** While its watch is blocked inside a request to the daemon — and only then — a thread of the
   monitor's own beats for it every quarter period, each beat on its own connection. Only then: a watch wedged anywhere else
   goes silent, and its dead-man fires, as a dead, killed or stopped process's does (`a_stopped_monitor_is_a_dead_one_and_says_
   so_when_it_resumes` still green). The thread ends before the watch disarms.
4. **Witnessed:** both witnesses red first (above), green after — `monitor_cli` 20 passed, the daemon's dead-man tests 9 passed,
   each three times more with all four CPUs busy. Mutants: M342 (fire as of now), M343 and M344 (a beat judged by when it was
   handled), M345 (the beater never beats) red, the control green; **M346 (the beater beats whatever the watch is doing)
   SURVIVES** — no witness reaches a watch wedged outside the daemon (`--death-after` is at least two polls, so a poll cannot
   stand in for a wedge), recorded as RW 4.63, open.
5. **Not changed:** the default period (`max(8 × poll, 2 s)`), the bounds, the records. A whole HOST stalled past the period
   still fires the dead-man — nothing in software can tell that from a dead monitor, and the safe direction is the designed
   one (P8-04: "a watchdog gone quiet is treated as one that fired").
6. **The arming's half** (found by `76f83a2`'s push run, the same run): `an_idle_daemon_fires_an_unbeaten_dead_man_without_any_
   request` saw a 300 ms dead-man fire 201 ms after the arming's ANSWER on Ubuntu. Measured between the daemon's own records
   instead — the arming's and the firing's — with the arming written under a 200 ms hold of the chain: it fired 99 ms after its
   arming record, red 2 of 2. The deadline was decided before the record was written, and the write can wait on the chain, so
   the wait ate the holder's first period, though a holder cannot beat before it holds the key. The first period now starts
   when the arming is answered, its record written (`start_deadman_period`, never earlier than the deadline it replaces):
   fired 301–302 ms after the arming, 3 of 3. The test measures from the records, under the hold. Mutants M347 (the restart
   removed) and M348 (it moves nothing) red, the control green. `eb7b8f8`.

## D-V2-128 — P9-04 step (a): a request with headers is `Cap[Http].get_with(url, headers: Map[Str, Str])`, its headers judged before anything is resolved and sent only to the origin the program named — TAKEN (head chef, 2026-10-14, under the owner's delegation)

**Context.** P9-04 (secrets bound to endpoints, `V2_OPENSHELL_STUDY.md` §4.11, RW 2.11) was read against the code by routine run
22: `Cap[Http]` had one method, `get(url)`, and the egress client's request no header — there was no request a secret could ride
on. Its proposed build order begins with **(a) the request with headers, no secret in it**. Witnessed absent first on the
current binary: a program calling `h.get_with(url, headers)` was refused `DL0405` ("type `Cap[Http]` has no method
`get_with`"); and a `Map` could not cross the sandbox channel at all (`WireValue::from_value` refused it as "a value of a kind
that does not cross"), so no guest could have handed its host a header map.

**Decision.**
1. **The surface.** `Cap[Http].get_with(url: Str, headers: Map[Str, Str]) -> Result[Str, NetErr] ! {Net}` — arity 2, a
   `PRIM_TABLE` row, `PRIM_TABLE_VERSION` 5 → 6 (additive; bumped by the constant's own rule), the conformance anchor
   `ref.prim.http.get_with` with the table's two witnesses and the egress client's two (an entrenched edit,
   `ENTRENCHED_CHANGE_RECORD.md`). **The HTTP method stays in the NAME** (`get`, `get_with`): what a program can do to a host is
   then readable from its calls alone, which P9-05's method dimension can decide statically. The URL stays argument one, so
   every place that decides a fetch on its URL — the custody gate, the refusal note, `fetch_allowed` — reads it where it reads
   `get`'s. **Step (b)'s shape is NOT decided here**: a `Secret[Str]` cannot sit in a `Map[Str, Str]`, so (b) adds its own
   method (for example `get_with_secret(url, headers, name, secret)`) rather than changing this one's arity; recorded so (a)'s
   programs never change.
2. **The headers' rules** (`egress::check_headers`), each a REFUSAL, never a normalization — a header written one way and read
   another by the server is a request nobody checked: a name is an RFC 9110 token (so `:authority`, a space, a colon, a line
   break and non-ASCII are refused); a value is printable ASCII, space and tab only — no CR, LF or NUL (request splitting) — with
   no whitespace at either end (servers strip it); two names that differ only in case are refused (a header name has no case);
   at most 32 headers, a name ≤ 128 bytes, a value ≤ 8 KiB, all ≤ 16 KiB. **Names the client owns are refused**
   (`RESERVED_HEADERS`): `Host` — a program's would carry the request to ANOTHER SITE served from the checked address, the
   net allowlist walked past by a header; `Content-Length`, `Transfer-Encoding`, `TE`, `Trailer`, `Expect` (framing);
   `Connection`, `Keep-Alive`, `Upgrade` (the connection); `Proxy-Authorization`, `Proxy-Connection` (there is no proxy);
   `Accept-Encoding` (no decoder is compiled in, so a coded body would be bytes the program cannot read and the "bytes counted
   are the bytes delivered" claim would need a caveat). `User-Agent` and `Accept` are the program's to set: a request header
   REPLACES the client's default (witnessed over real TLS — one line of each).
3. **What the program and the operator learn.** A refused header answers the opaque `Err(Refused)` like every policy refusal;
   the operator gets the machine-readable reason `header` with the rule in words (stderr, `egress.records[].reason`, a guest's
   `sandbox.denied`) — the rule, never the value. Nothing is resolved or connected for a refused header.
4. **The origin rule.** The headers go only to the ORIGIN the URL names — host and port, the scheme being `https` always (both
   in `parse_target`'s canonical spelling, so equality is safe): a redirect to another host, or to another port of the same
   host, drops them; a hop back to the named origin carries them again. What a program addressed to one server is never handed
   to the next — the rule step (c) applies to a secret's audience, applied first to everything a program says.
5. **The record.** Each hop records the NAMES of the headers it carried (`egress.records[].hops[].headers`, lower-case, absent
   when none — so a plain `get`'s report is unchanged); a value is recorded nowhere — not in the record, not in the trace
   (whose `detail` for `get_with` is the URL only: a value may be a credential the program declassified), not in the audit
   chain. The `run-report` schema gains the optional field.
6. **Custody.** `get_with` is `Op::Net` on the URL's host in `custody_op_for`, exactly as `get`: under the daemon its use is
   `use allow` on the run's node, and an egress refusal (a refused header included) is the `deny` citing it (RW 4.46). A
   `get_with` the gate did not know would be a request no revocation could stop and no chain could see.
7. **The channel.** `WireValue::Map` (entries in the map's ascending order): a key is a `Bool`, `Int` or `Str`; a key sent
   twice is refused rather than resolved by whichever came last; a capability inside a map the host returns is reminted like
   any other (the catch-all would have sent the encoder's placeholder handle 0 — another capability). `/3` is not bumped: an
   older guest cannot check a `get_with` program, so it fails closed at its own check.

**Witnessed** (each red first or falsified): the program refused DL0405 on the old binary; `egress`'s
`a_programs_headers_are_checked_before_anything_is_resolved` (each case refused by its OWN rule — 19 spellings and every
reserved name in two cases — with nothing resolved, nothing sent), `headers_that_follow_the_rules_are_sent_lower_cased_in_the_
order_given` (and the bounds exactly), `the_headers_go_to_the_origin_the_program_named_and_to_no_other` (six hops), and over
real TLS `a_programs_headers_arrive_over_tls_and_the_host_header_stays_the_checked_one`; the channel's three; `egress_cli`'s two
(L0 and a guest: the request leaves as TLS with the report naming its headers and matching `run-report`; a `Host` header
connects nowhere and says `header`); `monitor_cli`'s two (the daemon's custody records the `allow` and the `deny`, plainly and
in a guest). Mutants M349–M367 (below in `V2_LOG.md`).

**Not done here** (P9-04's next slices): (b) a `Secret[Str]` in a header position, resolved by the host at send time, and (c) the
audience in the grant line and the refusal at a destination outside it — **as one slice**: a secret sent to any allowlisted host
would be a declassification without `Cap[Declassify]` (today `expose`, behind it, is the only way a secret's bytes leave), so (b)
alone would widen what a program holding a secret and `net` can do; (d) the broker's audience dimension and
`secret_reach_expansion`; (e) `authority`'s per-secret destinations.

## D-V2-129 — RW 4.62: a trace record is a dispatch, and a refused operation is followed by its refusal — `<method>.refused`, the dispatch's effect, the code and its words — whoever refused it — TAKEN (head chef, 2026-10-14, under the owner's delegation)

**Context.** `--trace-effects` records an operation at its DISPATCH (spec §6.1; `DELULU_CORE.md` v0.3's E-Refuse: the
trace records the ATTEMPT). RW 4.62 (routine run 21): a custody denial that is a fault added nothing after it, so the trace
listed a `println` that never printed. Witnessed first on the current binary (routine run 23): a leased write the Guard holds
was refused DL1410, nothing was written, and the trace's only record was `write_text`.

**Decision.**
1. **The class, from the mechanism:** any FAULT between a recorded dispatch and the operation's end — the custody gate (a
   revoked lease, a guarded or sealed use), the broker's refusal of a daemon `expose` or `verify`, and the effect's own refusal
   (a path outside its capability, a host outside the allowlist) — is followed by a refusal record. RW 4.62 named the first;
   reading the dispatch found the other two, which leave the same unpaired record.
2. **The record:** `op` is `<method>.refused`; `effect` and `cap_kind` are the dispatch's, so the trace stays inside the row
   (`--assert-trace` and `delulu test`'s `effects_traced` read only effects, deduplicated — neither moves); `span` is the
   call's; `detail` is `CODE: message`, and the code ALONE where a secret was involved (its dispatch is opaque, and redaction
   outranks explanation). An actuator's refusals keep their own records (`command.revoked`, `command.refused`): they are
   values the program is told, not faults.
3. **Both engines:** the WASM engine traces `read_text` at dispatch too and now writes the same refusal record (its escape
   refusal and its custody refusal) — the two engines' traces stay byte-identical, witnessed. **Residual, RW 4.64:** the WASM
   engine checks custody BEFORE it traces console, clock and rand and refuses a hostile path before it traces `read_text`, so
   those refusals are in the interpreter's trace and not in the WASM engine's (predating this change); and it writes no
   refusal after `rand.int`'s `hi <= lo` fault, which the interpreter now records (this change's own divergence).
4. **Not changed:** a record's meaning (a dispatch), the trace's shape (one more `op` spelling), redaction, the trace bound.

**Witnessed:** `guard_e2e`'s `a_use_the_custody_refuses_is_followed_by_its_refusal_in_the_trace` (a guarded write, a guarded
`expose` — `detail` exactly `DL1410` — and a guarded read under both engines, identical) and `cli`'s
`an_operation_its_effect_refuses_is_followed_by_its_refusal_in_the_trace` (DL0904, both engines); mutants M368–M373 red, the
control green. `7c653e0`. **RW 4.64, its residual, closed the same run** (`b3e17fc`): the WASM engine traces console, clock
and rand at dispatch, then the custody's refusal; a hostile path after its `read_text` record; and writes the refusal after
`rand.int`'s `hi <= lo` — each witnessed against the interpreter's trace on one program (`refusal_trace_parity`), red first.

## D-V2-130 — A speed claim is bounded against a control measured on the same machine in the same window: the DIR verify witness asserts verify ≤ 5 × one check of the same source, never an absolute 5 ms — TAKEN (head chef, 2026-10-14, under the owner's delegation)

**Context.** `5fa3261`'s push run (CI `38586917971`) was red on Ubuntu alone: `dir::tests::verify_is_comfortably_fast_for_per_load_
use` read 5.7 ms per verify against an absolute 5 ms bound, for a commit that changed only the lockfile (rustls — `delulu-check`
does not use it); the same test was green on `fa2d34e`. Reproduced in the VM by starving it (AGENTS.md's trap; D-V2-47): 2.2 ms
quiet, then 6.8–7.4 ms with every CPU busy — red 3 of 3. The bound measured the machine.

**Decision.** Each of seven rounds times 30 verifies and 30 checks of the same source (the work a verify repeats — resolve and
check — plus a parse), interleaved; each side's best round is kept; the ratio must stay under 5 (measured 2.1 quiet, 2.0–2.6
with every CPU busy: a verify deserializes, re-checks and compares, about twice one check), with an absolute 250 ms ceiling no
machine excuses. The test prints all three numbers, so every green read is also a measurement. **Not a weakening:** a blowup
the old bound let through on a fast machine (a 2× slower verify at 0.75 ms) is caught now on any machine; mutant M374 (a
verify that re-checks ten times) is red, the control green; with every CPU busy the test is green 5 of 5. **The class,
enumerated:** it was the repository's only speed claim asserted in absolute time — the four other absolute time assertions
(`guest.rs`, `budget_cli`, `sandbox_devices_cli`, `microvm_cli`) are 8–60 s ceilings on something that must END, not speeds.

## D-V2-131 — P9-04 steps (b) and (c), as one slice: a secret leaves in a request only as one header's value, through `Cap[Http].get_with_secret(url, headers, header, secret)`, and only to the AUDIENCE its grant names (`secret:NAME@HOST[:PORT][,…]=VALUE`) — judged before anything is resolved and again on every hop that would carry it — TAKEN (head chef, 2026-10-14, under the owner's delegation)

**Context.** P9-04 step (a) (D-V2-128) gave `http` a request with headers; its build order put (b), a `Secret[Str]` in a header
position, and (c), the audience in the grant and the refusal outside it, next — **as one slice**, because (b) alone would send a
secret's bytes to every host a program's `net` grant reaches: a declassification without `Declassify`. **Witnessed absent first**
on the binary at `48ef512`: a program calling `h.get_with_secret(url, hs, "Authorization", tok)` was refused DL0405 (no such
method), and the grant `--grant secret:API_TOKEN@127.0.0.1:9=…` was read as a secret NAMED `API_TOKEN@127.0.0.1:9` — the
program's `root.secret("API_TOKEN")` faulted DL0703 (the audience form did not exist; an `@` was part of the name).

**Decision.**
1. **The surface.** `Cap[Http].get_with_secret(url: Str, headers: Map[Str, Str], header: Str, secret: Secret[Str]) ->
   Result[Str, NetErr] ! {Net}` — arity 4, a `PRIM_TABLE` row (84), `PRIM_TABLE_VERSION` 6 → 7 (additive, bumped by the
   constant's own rule), the anchor `ref.prim.http.get_with_secret` with the table's two witnesses and the egress client's two
   (the period's third entrenched edit, `ENTRENCHED_CHANGE_RECORD.md`). Its effect is `Net`, never `Declassify`: the program
   never reads the bytes — the host resolves them at send time. The secret stands in exactly one position; the header map stays
   `Map[Str, Str]` (a `Secret[Str]` cannot sit in it, DL0401), and a secret is not a header's NAME (DL0602) — so a secret can
   ride in no header where its audience would not be judged. The URL stays argument one, as `get`'s and `get_with`'s.
2. **The grant line.** `secret:NAME@HOST[:PORT][,HOST[:PORT]…]=VALUE` (and `=env:VAR`). The line is split at its FIRST `=`
   (the value may hold `=` and `@`) and the head at its FIRST `@` (so a name holds no `@`; a member with a second `@` is
   refused by its own parse as userinfo, never read as another host). Each member is parsed by the egress client's own URL
   parser (`egress::parse_audience` hands `https://MEMBER` to `parse_target`), so an audience and a request are judged in ONE
   spelling: lower-case LDH or a canonical address literal, the port 443 unless named, every rewritable spelling refused rather
   than normalized; a path, query or fragment is refused (a path in an audience waits for P9-05's path scopes — refused, not
   ignored), a wildcard is refused (an audience names hosts exactly), duplicates are folded. **No audience is the bottom:** a
   secret granted without `@` is sent nowhere (refused `audience`), so every existing grant keeps exactly its old meaning for
   `expose` and `verify` and gains no reach. A later grant of the same name replaces the earlier, audience and all — as it
   replaces the value. **Changed meaning, recorded:** a secret NAME holding `@` was accepted before (never documented, never
   tested); it now reads as NAME@AUDIENCE — a program asking for the old name faults DL0703 (closed, in words).
3. **Where the bytes may go — two walls.** The egress client (`request_with_secret`) holds the secret's header to every rule a
   program's header is (`check_headers`, over the whole set: not `Host`, no second name differing only in case — both ways
   round — no byte that would begin another header, and a reason that never quotes the bytes); then the URL's origin (host and
   port, parsed) must be a member of the audience, or the request is refused BEFORE anything is resolved — reason `audience`,
   the program told the opaque `Refused`. On each hop the secret rides only where the program's headers do (the origin it
   named — a redirect to another host or port drops it, D-V2-128's rule) AND that hop's origin is in the audience. The second
   condition is implied by the first wall today and is kept as a second wall, so that loosening either rule alone cannot send a
   secret where its grant never pointed: mutant M383 (the per-hop audience check removed) survives on the first wall by design;
   M383b (both removed) is red.
4. **What is recorded.** Each hop that carried the secret names it — `secret: "API_TOKEN"` (the grant name) beside the header
   NAMES — in the egress record, the run report (the `run-report` schema's optional hop `secret`) and nowhere its bytes; the
   trace's detail for the call is `«opaque»` (an argument is a secret — the existing redaction rule). `SecretHeader`'s own
   `Debug` omits the bytes, and `PinnedRequest`'s derived `Debug` — which would have printed every header's value, a secret's
   included, had anything ever formatted it (nothing did) — is replaced by one that prints the names (mutant M400 red).
   **Honest limit:** the bytes are copied into the request's buffers (as `expose` copies them) and those copies are not zeroed;
   `SecretVal`'s zeroization bounds exposure in this process, it does not end it (its own comment, P17-F).
5. **Where the audience lives.** `Grants.secret_audiences` and `RootVal.secret_audiences` (by name); a minted `SecretVal`
   carries `SecretAudience { name, to }`; `Secret.map` carries it to its result (a value derived from the bytes goes where they
   may go, and nowhere else); an actor boundary carries both the root's and a secret's (`RootMsg.secret_audiences`,
   `MsgValue::SecretLocal(bytes, audience)` — owned, so the message stays `Send`).
6. **Where it is not yet.** **The daemon** (`--broker daemon`, `--lease`): a run's secret is a broker HANDLE, its bytes the
   broker's, and the broker's lattice has no audience dimension — P9-04 step (d). `get_with_secret` of a handle is refused
   DL0904 in words that name the way out that works (embedded custody, the secret granted with its audience — taken by the
   witness), and **no custody op is asked for it** (HTTP-SCHEME-1's rule: a `use allow` for a request never made would be a
   record of nothing — witnessed: the chain holds the run's `issue` and no `use`). A lease's grants carry no audience
   (`grants_from_lease`). **A guest** holds no secret (DL1401 at `root.secret`, unchanged — nothing is sent from one).
   **`--engine wasm`** has no `http` at all (DL1201, unchanged). **`authority`** still lists `secret:NAME=VALUE` for a
   secret a program sends — the per-secret destinations are step (e).
7. **`grants diff` between two grant files** decides a secret's REACH now: every origin NEW's audience names for a secret OLD
   grants must be in OLD's — else `secret_reach_expansion` (the category reserved since D-V2-119), naming the origin; an
   audience where OLD had none is reach (none is the bottom); a narrower or absent one adds nothing. Between two lease NODES
   it waits for (d), with the broker's dimension.

**Witnessed:** the egress client's four (`a_secret_goes_only_to_its_audience_on_the_origins_hops`,
`a_secret_outside_its_audience_is_refused_before_anything_is_resolved` — another host, another port, a sub-domain, an address
the name resolves to, no audience; nothing resolved, nothing sent — `a_secrets_header_is_held_to_every_header_rule`,
`an_audience_is_spelled_as_the_request_is_compared`), the grant line's (`a_secrets_audience_is_parsed_from_its_grant_line_and_
nothing_else_is`), the actor boundary's, the checker's (`a_secret_rides_only_in_get_with_secrets_secret_position`), and three
at the command line — `egress_cli`'s `a_secret_is_sent_to_its_audience_and_nowhere_else` (over TLS on loopback, through
`Secret.map`; the report schema-valid; the trace read; no byte of the secret in report, trace, stdout or stderr; another port
and no audience connect nowhere; a guest sends nothing), `broker_cli`'s `a_broker_held_secret_is_not_sent_and_no_use_is_
recorded`, `within_cli`'s `grants_diff_names_what_a_change_adds_and_nothing_it_does_not` (secret reach both ways round).

**Falsified:** mutants M380–M400 (`scripts/mutants.py`) red but one: the origin judged against the audience before resolution removed (M380), the empty audience let through (M381), the per-hop origin rule ignored for the secret (M382), both walls removed at once (M383b), the hop's `secret` name dropped (M384), `audience` answered as something other than `Refused` (M385), a path let into an audience (M386), the secret's header unchecked (M387), the grant line split at its last `@` (M388), the audience not stored (M389), `Secret.map` dropping it (M390), the mint dropping it (M391), a broker handle sent (M392), a custody op asked for one (M393), the actor boundary dropping a secret's (M394) or a root's (M395), `grants diff` ignoring it (M396) or calling it `scope_expansion` (M397), the checker taking a `Str` for the secret (M398), the schema's hop `secret` mistyped (M399), `PinnedRequest`'s `Debug` printing values (M400) — and **M383, the per-hop audience check alone, survives on the first wall by design** (D-V2-131 item 3); the control green

## D-V2-132 — MAP-HANDLE-1: `Secret.map` of a broker-held secret is refused in words, never run over bytes this process does not hold — TAKEN (head chef, 2026-10-14, under the owner's delegation)

**Context.** Found by routine run 24 while reading the broker's secret path for P9-04 (d): the interpreter's `Secret.map` handed
its function `s.reveal()` — and a daemon or leased run's secret is a broker HANDLE, whose `reveal()` is the empty string
(invariant 23: no bytes before `expose`). **Witnessed on `48ef512`'s binary:** `s.map(fn(x: Str) -> Str { x + "!" }).expose(d)`
printed `hunter2!` with embedded custody and `!` under `--broker daemon` — a fabricated value, VERIFY-FABRICATED-1's shape, two
custody modes answering one program differently (HTTP-SCHEME-1's lesson) and not even failing closed. Worse, the result was a
LOCAL secret, so its `expose` ran with no broker gate (the node's `Declassify` effect and `secrets` scope) and wrote no `expose`
record. The broker's own `secret_map` (whitelisted `trim`, `upper`, `lower`) exists, and the language never
reaches it: `V2_P9_DESIGN.md`'s "`map` (a whitelisted transform — broker-side under the daemon)" described an operation no program
can call. **The class, enumerated** (every `reveal()` in the tree): `Secret.map` was the only call that could reach a handle —
`expose`'s is routed through custody first, an actor message keeps a handle a handle, and `get_with_secret` (D-V2-131) refuses a
handle before revealing anything.

**Decision.** `Secret.map` of a broker-held secret is refused DL0904 before its function runs, in words that name the two ways
out — `expose` under `Declassify` (through the broker, gated and recorded) and transform the result, or embedded custody — and the
first is taken by the witness. **Not chosen:** compiling a closure into the broker's whitelist (the function is arbitrary pure
code; three named operations cannot express it, and recognizing `fn(x) { x.trim() }` would be a second language for one
receiver); running the closure in the broker (a second interpreter holding the bytes). **Left open:** whether the language should
reach the broker's whitelist at all (a `Secret` method per whitelisted op), recorded with P9-04 (d).

**Witnessed:** `broker_cli`'s `a_broker_held_secret_is_never_mapped_over_bytes_it_does_not_hold` — the control (embedded:
`hunter2!`), the refusal under the daemon (exit 1, nothing printed, DL0904), and the way out (`expose` then transform under the
daemon: `hunter2!`). **Falsified:** mutant M401 (the refusal disabled) red, the control green.

## D-V2-133 — P9-04 step (d), its first half: the broker's `secret_to` dimension — each secret's audience a set of `NAME@host:port` pairs, ordered by inclusion, a pair living only beside its name; carried by every node, every delegation and `grants diff` between nodes — TAKEN (head chef, 2026-10-15, under the owner's delegation)

**Context.** D-V2-131 gave a secret an audience in the grant line (`secret:NAME@HOST[:PORT]=…`) and the egress client the rule
that sends it only there. The broker knew secret NAMES only (`Scopes.secrets`, the wire's `AuthoritySpec.secrets`), so the
audience died at every node boundary: `run --broker daemon --grant secret:K@host=…` issued a node holding `K` with no reach
(witnessed: `grants list --json` showed no audience), `grants delegate` could not say where a delegated secret may go, and
`grants diff` between two nodes could not see a secret's reach at all (`secret_reach_expansion` waited since D-V2-119).
`V2_P9_DESIGN.md`'s reading of step (d) (routine run 24) proposed the dimension and the broker op that sends; this decision is
the dimension, the op is the second half.

**Decision.**
1. **`Scopes.secret_to: BTreeSet<String>`** — the eleventh dimension of `⊑`. Each member is `NAME@host:port` in the egress
   client's canonical spelling (`parse_audience` then `Audience`'s `Display`: the port always named, an IPv6 host bracketed); `⊑`
   is inclusion, `⊓` intersection — one more exact-set dimension. A secret with no pair is the bottom: sent nowhere.
2. **A pair lives only beside its name.** `Scopes::canonicalized` — every node's ingest (`issue`, `attenuate`, `attenuate_core`)
   — drops a pair whose name `secrets` does not hold (it grants reach to nothing, and keeping it would make two authorities that
   grant the same thing unequal under `⊑`); `is_canonical` checks it. The broker parses no URL: a spelling no origin has is
   compared by exact string and matches no destination — reach to nothing either way, the fail-closed direction.
3. **Serialized only when non-empty** (`to_json`, the wire's `#[serde(default, skip_serializing_if = "Vec::is_empty")]`) — the
   rule `device` and `budget` follow, so every authority without an audience keeps its canonical JSON, its chain hash and its
   certificate bytes. A certificate's `secret_to` is read (`authority_from_json`); a build that predates it refuses the
   certificate whole, as for any dimension it cannot see.
4. **Where it enters.** `authority_spec_from_grants` carries a grant line's audience into the node (the issue-then-run path, a
   proposal's ask); `grants delegate --secret NAME --secret-to NAME@HOST[:PORT]` names one, each member read by the egress
   client's own parser and sent canonical — a pair for a secret the delegation does not name, a path, a wildcard or an
   upper-case host is refused in words (exit 2) before the broker sees it, never normalized; a proposal's join keeps both sides'
   pairs; `grants_from_lease` gives a leased run's grants the node's audiences (each pair re-read by the egress parser; one
   that does not parse is dropped — that origin is then outside the audience).
5. **`grants diff g_OLD g_NEW`** reports a pair NEW holds and OLD does not as `secret_reach_expansion` (grant `NAME@host:port`),
   asked of the lattice like every other item; a pair for a secret OLD does not hold at all is that secret's
   `scope_expansion`, counted once (the grant-file diff's rule, D-V2-131).
6. **One conversion per direction.** `grants_diff_nodes` built a node's authority from a struct literal ending
   `..Default::default()`, and `broker_client::authority_to_spec` was a second copy of `brokerd::authority_spec_of`: a new
   dimension is dropped SILENTLY by the first and must be found by hand in the second. The first now calls
   `NodeInfo::authority_spec`, the second `authority_spec_of`.
7. **The proof.** `authority_algebra.py` carries the dimension in the full conjunction (nine set-like dimensions, eleven in
   all) and a new §2c for the one rule the other exact-set dimensions lack: canonicalization is well-formed, never widens, is
   idempotent, keeps a well-formed authority and is monotone; the meet keeps well-formedness and is `⊑` both; and at the level of
   the whole order, a pair the parent lacks is never `⊑` (the obligation that notices the dimension missing — the four product
   laws do not). 34 obligations; three more mutants CI requires to fail (`secret-to-dropped`, `secret-to-meet-union`,
   `secret-to-keeps-orphans`), six in all.

**Not yet — the second half of step (d).** A daemon or leased run still cannot SEND a secret: its secret is a broker handle and
`get_with_secret` refuses one (DL0904, D-V2-131) — the bytes must come from the broker at send time through a recorded op
naming the destination (`SendSecret`, guarded like `Declassify` by default — `V2_P9_DESIGN.md` step (d) item 3). The node now
holds what that op will judge.

**Not chosen:** a structured pair type on the wire (two crates parse one string, as for `device` — the string is the contract);
refusing a non-canonical pair at the broker (it has no URL parser to decide canonicity with; exact-string comparison already
fails closed); normalizing an upper-case host at `--secret-to` (the egress client refuses it for a URL, so the audience does the
same — one rule, one spelling).

**Witnessed red first** on `41ce7c1`'s binary: `broker_cli`'s daemon run — its node's `secret_to` absent (`Null` where
`["API_TOKEN@127.0.0.1:PORT"]`); `within_cli`'s `a_nodes_secret_carries_its_audience_and_a_wider_one_is_secret_reach` —
`--secret-to` an unknown argument. **Falsified:** mutants M402–M421 (`scripts/mutants.py`), every one red: `⊑` without the dimension (M402), the meet as the union (M403), canonicalization keeping an orphan (M404) or `is_canonical` blind to one (M405), the JSON (M406), the compact rendering (M407) or the certificate parser (M408) dropping it, a pair with an empty name read as named (M409), a proposal's join dropping the ask's pairs (M410), the grant line's audience not carried into the node (M411), the wire → authority (M412), the node → wire (M413), the node → spec (M414) or the authority → wire (M415) conversion dropping it, `grants diff` blind to a pair (M416) or counting a pair of a secret OLD lacks twice (M417), `--secret-to` taking a name the delegation does not (M418) or a spelling the egress parser did not canonicalize (M419), the lease helper keeping an orphan (M420), and a lease's run given no audience (M421); the control green. The model's three mutants (§2c and the full conjunction) each end with obligations NOT discharged.

## D-V2-134 — P9-04 step (d), its second half: the broker SENDS a secret — a recorded, Guard-gated `send` that hands a broker-held secret's bytes to the run's egress client for ONE origin its node's `secret_to` names, asked only after everything that can be judged without the bytes — TAKEN (head chef, 2026-10-15, under the owner's delegation)

**Context.** After D-V2-133 a node's secret carries its audience, but a daemon or leased run's secret is a broker HANDLE with
no bytes in the program's process, so `get_with_secret` of one was refused (DL0904, D-V2-131). Under the daemon the program
process performs the request itself (the interpreter's egress client), so a send needs the bytes there at send time — and the
broker, which holds them, must be the one that decides.

**Decision.**
1. **`Broker::send_secret_guarded(node, name, origin, store, span)`** and the wire's `SendSecret { node, name, origin, span }` →
   `SecretBytes { bytes }`. The node must be live; its `secrets` must cover the name's SOURCE (a broker-side derivative is sent
   on its source's pair) and its `secret_to` must hold `SOURCE@origin`; the store must hold the bytes. **No `Declassify`
   effect** — the language sends a secret without one (D-V2-131); the audience is the authority.
2. **The Guard** decides as for a declassification, because the bytes leave the broker — class `declassify`, guarded by
   default, so a DELEGATED node's send is DL1410 until its owner approves; a root node (the principal's own, `run --broker
   daemon`'s) is ungated, as for `expose`. Its axis token is the PAIR, `SOURCE@origin`: a permit names the destination
   (`guard request --use declassify:API_TOKEN@api.example.com:443`), so a permit to expose the secret is not one to send it,
   and one to send it to one origin is not one to send it to another. `declassify:*` still covers every send. No new Guard
   class (the Guard is hardened, not redefined). *(**Corrected the same run by D-V2-137**, after the red team's F1: a rule
   or permit on the secret's NAME covers every send of it — a seal on a name had sealed nothing of its sends — and a pair
   pattern that cannot match a send's token is refused; the pair still names one destination.)*
3. **The record.** A guard event when the Guard is asked (as `verify_guarded`), then one `send` record — allow or deny —
   naming the pair and the calling span, never a byte. The lost-record rule (D-V2-109/111): a `send` whose record could not be
   written hands over nothing (DL1401). An `allow` `send` is written BEFORE the bytes leave the daemon.
4. **The order in the run** (`Interp::send_broker_secret`): the custody's `Net` check of the host first (a request carrying a
   secret is a use of its host like any other — the `use allow` RW 4.46's refusal record cites); then the effect's own
   refusals in their own words (a scheme that is not `https`, a host outside the capability); then
   `egress::prejudge_secret` — everything the egress client judges before resolution, asked BEFORE the bytes exist: every
   header (the secret's with a stand-in value), the URL's strict parse, its origin in the handle's audience. A refusal there
   asks the broker nothing (no `send` record) and is recorded and answered as the egress client's own (`egress::refuse_unsent`).
   Only then is the broker asked, for that ONE origin; the request is made with a secret that holds the bytes and whose audience
   is that origin alone; the egress client then judges the secret's real value and every hop as before. The bytes reach the
   egress client and no program value.
5. **A handle carries its audience** (`SecretVal::handle_bound`, from `root.secret_audiences` at the mint — a leased run's from
   its node's `secret_to`, D-V2-133) and keeps it across an actor boundary (`MsgValue::SecretHandle`'s audience), as a local
   secret does.
6. **Criterion 6 restated as it holds** (`STAGE5_SPECIFICATION.md`): no secret byte is in the program process before an
   `expose` or a recorded `send`, and a `send`'s bytes are handed to the egress client, never to a program value.

**Two walls and one implied rule, recorded as such.** The broker's `secrets` check is implied by its `secret_to` check (a
pair lives only beside its name, D-V2-133): M428 removes the first alone and survives by design; M428b removes both and is red.
Narrowing the sent secret's audience to the one origin (M432) is implied by the egress client's per-hop origin rule — a
redirect to another origin drops every header the program named, the secret's among them (D-V2-128, M382's witness) — so it
survives too; it is kept so that what the egress client holds says exactly what the broker authorized. `prim`'s refusal of a
handle that reaches the request table unanswered is unreachable today (the interpreter routes every handle first; a guest
holds none; the WASM engine has no `http`) and is kept as the refusal in words of an internal wiring error.

**The OCSF mapping, built the same run** (`b8da460`): an allowed `send` exports as HTTP Activity (4002), Get, its
`dst_endpoint` the origin's host and port (the port after the LAST colon, so an IPv6 host stays whole), the secret named only
in the message and the embedded record; a refused one stays a Detection Finding. Witnessed: the seeded chain's `send/allow`
exported as class 0 before; validated against OCSF 1.8.0 (15 events, 0 problems); M440 (no port) and M441 (the pair as the
hostname) red. **Not yet:** step (e) — `authority`'s per-secret destinations. Whether the language should reach the broker's whitelisted
`secret_map` stays open.

**Not chosen:** a `send` class of its own in the Guard (a vocabulary change the owner's — the pair as `declassify`'s token
already makes a permit name the destination); requiring `Declassify` on the node (the language promises a send without it);
asking the broker before judging the request (bytes would cross for a request that could never carry them); the broker
performing the request itself (a second egress client, holding the bytes, that the operator's grants do not configure).

**Witnessed red first** on `a97a953`'s source (the slice's tests against the code before it): the daemon run and the leased
run both refused DL0904. **Falsified:** mutants M422–M438 (`scripts/mutants.py`): the `secret_to` check removed (M422), the pair
built from a derived name rather than its source (M423), the Guard asked on the name (M424 — an expose permit then bought a
send) or not at all (M425), the deny record (M426) or the allow record (M427) dropped, the node's liveness not asked (M429),
the lost-record rule off (M430), the pre-judgement replaced by the URL's parse alone (M431), a handle minted without its
audience (M436), the pre-judgement's audience check (M437) or header check (M438) removed, a handle's audience dropped at an
actor boundary (M435) — each red; **M428** (the `secrets` check alone) and **M432** (the one-origin narrowing) survive by
design, implied walls (item above); **M428b** (both broker checks) red; the controls green.

## D-V2-135 — P9-04 step (e): `authority --json` names where a program sends each secret — `secret_destinations`, the secret and the destination as far as the source shows them, `null` where it does not, never a guess — TAKEN (head chef, 2026-10-15, under the owner's delegation)

**Context.** After D-V2-131/133/134 a secret goes only where its audience says, and the operator writes that audience
(`secret:NAME@HOST[:PORT]=…`, `grants delegate --secret-to`). Nothing read it off the program: `authority` named the secret
and the hosts, never which secret goes to which host. `V2_P9_DESIGN.md` item 5: the walk must follow a secret into a request's
argument and the request's URL literal; a computed URL or a secret reached through a value is reported as such.

**Decision.**
1. **`within::secret_destinations`**, stamped on `authority`'s report (one file and a package) as `secret_destinations` —
   present only when the program sends a secret, so every other report is byte-identical (criterion 11, the `plugins` rule) —
   one `{"secret", "to", "at"}` per `http.get_with_secret` call, in source order.
2. **`secret`**: the grant name when the argument is a literal `.secret("NAME")` (only `Root` has the method, so the receiver's
   name does not matter — `requested_scopes`' rule), a name a `let` or an assignment bound to one, or a `Secret.map` of either
   (a value derived from the bytes goes where they may, D-V2-131). **`to`**: the URL literal's origin through the egress
   client's own `parse_target` — `host:port`, the spelling an audience must name. **`null`** for either when the source does
   not show it: a computed URL, a secret that arrives as a parameter, a field or an element, a name a `for`, a lambda
   parameter or a match arm rebinds (all shadow, conservatively), a block's `let` once its block ends.
3. **Not chosen:** proposing the audience in `--grants`' lines (`secret:NAME@host:port=…`) — it changes a contract several
   verbs read (`--within`, `grants propose`, the corpus); a later slice can derive it from this field. A human line in the text
   report — the JSON first, as for `requested_scopes`.

**Witnessed red first** on `3f03ab0`'s source: no `secret_destinations` in the report. **Falsified:** M442 (a `let` not
recorded), M443 (`map` not followed), M444 (a `for` not shadowing), M445 (a block's names leaking past it), M446 (an assignment
not rebinding), M447 (the origin without its port), M448 (the key on a report that sends nothing) — each red; **M449** (a
per-function reset of the names) survived because a body is a block whose names already end with it — the reset was removed,
not kept as a wall no witness can reach.

## D-V2-136 — NET-WILDCARD-1: one host matcher — the broker's network use check reads a `net` pattern exactly as the runtime does — TAKEN (head chef, 2026-10-15, under the owner's delegation)

**Context.** Found reading P9-05 against the code: with `--grant net=*.example.com` (or `net.special=`), an embedded run's
request to `api.example.com` was allowed, and the same run under `--broker daemon` was refused DL0904 — the broker's `Op::Net`
check compared the host with the node's `net` set by exact string (`scopes.net.contains(host)`), while the runtime's allowlist
and egress client read `*.SUFFIX` as a pattern (`prim::host_matches`, C85). Fail-closed, never a hole; but the daemon refused
what the grant grants, so a program that ran embedded failed under custody.

**Decision.** The matcher moves into `delulu-broker` (`validate::host_matches`, C85's dot boundary, the apex not matched),
and the broker's use check and `prim::host_matches` both call it — one matcher, where there were three and two had disagreed
before (C85). `⊑` between `net` sets stays exact-string: a pattern does not imply a host beneath it (conservative, sound, and
widening it later is additive). The guard's `net` class still matches its patterns by exact string (unchanged).

**Witnessed** both ways on `0f73f09`'s binary; `broker_cli`'s `a_pattern_net_grant_is_honored_under_the_daemon_as_embedded`
red on the old source (DL0904), green now; `validate`'s unit test (subdomains allowed; the apex, `evilexample.com`, a sibling's
subdomain, an unnamed host refused). **Falsified:** M450 (exact string again), M451 (no dot boundary), M452 (`*` without its
dot) red, the control green.

## D-V2-137 — The red team's F1: a Guard rule or permit on a secret's NAME covers every send of it; a `declassify` pattern no use can match is refused — TAKEN (head chef, 2026-10-15, under the owner's delegation)

**Context.** D-V2-134 gated a send on the pair `NAME@host:port`, and Guard rules match their token by exact string. The run's
red-team pass (one Sonnet 5.5 sous-chef against a frozen binary of `0f73f09`) found: with `declassify:API_TOKEN` SEALED, a
delegation of its declassification was refused DL1413, yet a pair permit (`declassify:API_TOKEN@127.0.0.1:PORT`) let the secret
be sent; with `declassify:*` unset, the send needed no permit at all while `guard status` said the name was sealed; and
`guard policy set` accepted `API_TOKEN@*`, `API_TOKEN@127.0.0.1`, `API_TOKEN@127.0.0.1:*` and `API_TOKEN*`, each matching
nothing. Re-witnessed in the VM (a broker unit test: the name's permit did not cover a send).

**Decision.** In the `declassify` class a rule or permit on the NAME covers a send's pair token (`guard::pattern_matches`): a
seal on a secret refuses every send of it, and a permit to declassify it — more than one to send it to its audience — covers
its sends. A pair names one destination exactly. `dead_pattern_reason` refuses a `declassify` pattern that no use's token can
match: a glob other than `*`, or a pair without a name, a host and an exact port. This corrects D-V2-134 item 2's "a permit to
expose the secret is not one to send it". **Falsified:** M453 (no name match) and M454 (dead patterns accepted) red.

## D-V2-138 — P9-05 step (a): a `net` grant may name a path pattern, `HOST/PATTERN`, read one way at the grant, the first check, every egress hop and — by its host — the broker's use check; where a pattern decides, a path a server could read two ways is refused — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Context.** RW 2.12: a `net` grant reached every path of its host. Run 25 read P9-05 against the code (`V2_P9_DESIGN.md`):
the egress client already parses each hop, the broker sees only the host, the grammar had no room for a path; the build
order put (a) first — the grammar and the runtime. The owner's commission of 2026-10-17 names P9-05 as the next work.

**Decision.**
1. **Grammar.** `net=HOST/PATTERN` and `net.special=HOST/PATTERN`. A pattern's segments are literals (letters, digits,
   `-._~!$&'()+,=:@`), `*` (exactly one non-empty segment) or `**` (the rest, none included — only last). Refused at the
   grant: an empty, `.` or `..` segment, a trailing `/`, `%`, `;`, `?`, `#`, a `*` inside a literal, more than 64 segments
   or 1,024 bytes. `net=HOST` is unchanged: every path.
2. **The special-use rule reads the HOST part.** `net=127.0.0.1/x/**` is refused as `net=127.0.0.1` is, with the
   `net.special=` repair — a check on the whole value would be walked past by any path appended to it (HANDOFF §11.4: what
   else spells this?).
3. **One reading**, `delulu_broker::netscope` (split, pattern, ambiguity, match, `judge_all`), beside `host_matches`
   (NET-WILDCARD-1's lesson). Its callers: the grant parser; the runtime's first check (`prim::url_verdict` — DL0904
   saying whether the host or the path is outside); the broker-secret pre-check (`interp::send_broker_secret`, so the broker
   is asked for no bytes, and records no `send`, for a path outside the scope); the egress client on every hop (a redirect
   out of the path is `not-allowlisted`, an ambiguous spelling `malformed`); and the broker's `Net` use check, which compares
   the HOST part — the op carries the host — so `--broker daemon` and embedded custody read the grant alike.
4. **Where a pattern decides, the request path must be plain**: a `.`/`..` segment, an empty segment before the last, a
   `;`, or an escape of `/`, `\`, `.`, `%`, `;` or a control character is refused, never matched — each is a second spelling
   a URL parser or a server reads as another path (`..;/`, `%2F`, `%252F`). A host-only grant reaches every path, so its
   requests are not judged so. The query is not the path.
5. **Minting is exact, as for hosts.** `root.http(["api.example.com/v1/**"])` needs that grant exactly, so what a program
   may reach stays readable from its source and `authority`. Narrowing by containment is step (b)'s order (`⊑`, the Z3
   model, mutants).
6. **The OpenShell export refuses a path-scoped grant in words** — its host alone would widen it to every path. A mapping
   waits until OpenShell's prover can check it in CI.
7. **Not yet:** the broker's op and record carry no path (step (b)); `grants diff` and `authority` compare path entries
   as strings (step (c)); methods wait for the first non-GET primitive (step (d)); a secret's audience still refuses a path.

**Falsified** (each restored from a copy and every binary rebuilt and scanned for the mutant's marker): M1 the hop lets a
path outside pass, M2 no ambiguity check, M3 the special-use rule reads the whole entry, M4 a pattern unvalidated, M5 the
first check reads hosts only, M7 the broker-secret pre-check reads hosts only, M8 the export maps the host — all red; the
broker's host-part comparison (M6) witnessed red against the exact one before it was written.

## D-V2-139 — P9-05 step (b): a `net` set is ordered by containment — an entry within another of the same host string whose path pattern contains it — with every entry's meet as the meet and an antichain as the canonical form, in the broker's `⊑`, the runtime's mint and `grants diff` alike; proved sound in Z3 and exact by enumeration; the chain keeps naming the host — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Context.** D-V2-138 left the `net` order exact-string — the broker's `⊑`, the runtime's mint — so a delegation could not
narrow a host to its paths, a program could not ask for less than it was granted, and `grants diff` reported a narrower
path as an addition. Run 25's build order put the order, with the Z3 model and mutants, in step (b); it also proposed
carrying the path in the broker's `Net` op and record.

**Decision.**
1. **The order** (`netscope::entry_within`): the same host string (a `*.SUFFIX` entry still contains no host under it here —
   conservative, as before); a parent with no path reaches every path; a child with none is within only such a parent; two
   patterns by `pattern_within` — a literal under an equal literal or `*`, `*` under `*`, everything remaining under a
   final `**`. A set is within another when every entry is within one of its entries: a preorder.
2. **The meet** (`pattern_meet`): segment by segment the narrower side, two different literals meeting in nothing, and
   where one side's `**` begins, the other side's rest. A set's meet is every non-empty entry meet.
3. **The canonical form** is an antichain — no entry within another, and of two mutually within the smaller spelling —
   P17-F3's rule for this dimension (`Scopes::canonicalized`, `is_canonical`), so ⊑-equivalent authorities are one value
   and hash alike. Host-only sets are unchanged by it.
4. **One order everywhere**: `attenuation_check` and `intersect` (DL0802's repair is the meet); the runtime's mint (a
   program may ask for a narrower path than granted, never a wider one; its special-use subset by the same order);
   `grants diff` (by the mint, and its `net.special` line by the same order); and every parser of a `net` entry calls
   `netscope::check_entry` — `--grant`, `grants delegate --net`, `grants certify --net` (refused in words, exit 2).
5. **Proved.** The Z3 model's §2d: the order reflexive, transitive and SOUND against what the patterns match, the meet
   exact both ways, within both sides and the greatest such — 7 obligations, 41 in all, and three mutants CI requires to
   fail (`net-star-under-literal`, `net-rest-under-fixed`, `net-meet-star-wins`). In Rust, exhaustively: the order and the
   meet EXACT (complete as well as sound) for every pattern of up to three segments against every path of up to five;
   the canonical form over every subset of a seven-entry universe. The agreement test (`within.rs`: the `--within`
   verdict and the lattice give one answer) now generates path-scoped entries — and went red when only the broker's
   side had the new order.
6. **Not taken: the path in the broker's `Net` op and record.** Every consumer reads a `use` record's `target` as the
   host — the OCSF export maps it to `dst_endpoint.hostname` (`ocsf.rs`, "never a path"), the egress refusal's `deny`
   record and `audit query` likewise — so a path there would make an exported event's hostname carry a path; a separate
   field changes the record's canonical shape and is its own slice (RW 2.12 keeps it open). The path is judged by the
   egress client on every hop under every custody (D-V2-138) and named in the run report (each hop's URL).
7. **Next:** step (c)'s static half — `authority` naming each request's path as the source shows it; (d) methods, with the
   first non-GET primitive.

**Falsified:** N1 `*` within a literal, N2 `*` wins the meet, N3 the broker's exact subset, N4 the exact mint, N5 and N5b
no canonical form, N6 `grants diff`'s special-use line exact, N7 `grants delegate --net` unvalidated — all red; every
binary rebuilt and scanned clean.

## D-V2-140 — P9 closes with P9-05's path dimension: enforced, ordered and proved; `authority`'s static path report and the method dimension are recorded open, each with its reason — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Context.** P9-05 was designed as four slices (`V2_P9_DESIGN.md`): (a) path scopes in the grammar and the runtime, (b) the
order with the Z3 model, (c) `grants diff`/`authority` path reach and the static walk, (d) methods with the first non-GET
primitive. (a) and (b) are built (D-V2-138, D-V2-139); `grants diff`'s path reach came with (b), since it decides by the
runtime's mint.

**Decision.**
1. **P9 is complete as far as the request primitives that exist reach** — every request is a `GET`, and the path
   dimension of a `net` grant is enforced at every check under every custody, ordered by containment everywhere a grant is
   compared, and proved. The phase row says so, as P8's says "as far as software reaches".
2. **(c)'s static half stays open (RW 2.12)**: `authority --json` naming the path each request's URL literal shows. The
   AST walk `secret_destinations` uses cannot tell an `Http` capability's `get` from a `Map`'s without type information
   it does not carry; a report that guesses which call is a request is the kind of claim this project does not make. It
   needs the checker's types at each call, which is its own slice.
3. **(d), methods, waits for the first non-GET primitive (RW 2.11)**, as run 25's reading said: a method scope for a
   method no program can call is a gate that cannot fail. A request with a body and a method is a language addition —
   the checker's table, both engines, the guest channel, the egress client, the trace, the report and its schema, the
   OCSF mapping, conformance anchors — and is ranked as the first engineering priority after the public release.
4. **The path in the chain's record** stays open (RW 2.12; D-V2-139 item 6).

## D-V2-141 — Constitution §5.15 guarantee 5 and §5.14's third layer are narrowed to what is built: a violation is contained by the layer a program runs under, and the microVM is that layer only on Linux x86-64 with KVM — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Context.** RW 7.10a; the owner's commission asks for exactly this review. The entrenched file's change is recorded in
`ENTRENCHED_CHANGE_RECORD.md` (2026-10-17), with why it is not an RFC (it strengthens §5.14's honesty limits; the
2026-08-08 precedent).

**Decision.** Guarantee 5 now names the layer for each way a program runs — the WASM/WASI floor on the WebAssembly engine,
the process sandbox for a guest (L1, three operating systems), the microVM on Linux x86-64 with KVM (L2) — and says that
an ordinary in-process run (L0) has no operating-system boundary, its runtime re-checks (guarantee 6) being the layer
beneath the type system. §5.14's third layer adds that it is built on Linux x86-64 with KVM only, refused (DL1408)
elsewhere, and that a layer counts only where it runs. Nothing else in the Constitution changes.

## D-V2-142 — The review the owner's commission asks for: D-V2-49, D-V2-59 and D-V2-110 to D-V2-137 are kept — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Context.** The commission: *"Review delegated decisions D-V2-49, D-V2-59 and D-V2-110 through D-V2-137. Preserve sound
decisions; revise any that no longer serve the project's goals."* Each entry was read again — the two flagged when taken in
full, the rest from their decision items — against two questions: does it still hold in the code (CI on `master` green
through `8cae6a1`), and does it still serve a public release.

| Decision | Kept, because |
|---|---|
| D-V2-49 (`DELULU_CORE.md` v0.3) | It moves the formal model toward the implementation (progress-or-fault, the attempted label, the higher-order primitive) and changes no language rule; RW 5.1 (mechanization) stays open, aimed at the right calculus. |
| D-V2-59 (`hostile-agent` requires all five properties; `contained`, `dev` none for now) | Still the honest split: macOS's reads remain a PS-E residual, so requiring all five of `contained` (the default) would refuse every macOS run. The documentation must say `contained` is best-effort and `hostile-agent` is the profile for untrusted code — checked in the README refresh before the release. |
| D-V2-110, 117 (devices start at the first request; never-started devices journaled) | Parity between a guest and an ordinary run, and a report that says what did not happen. |
| D-V2-111, 114, 115 (a lost record is never answered as success; a loosening is undone) | The fail-closed rule (invariants 26/27) applied to every handler the writer reaches. |
| D-V2-112, 113, 116 (egress refusals in the chain; the monitor's `special-use` and `break-glass` rules) | The out-of-band monitor judges what the chain holds; P8 complete as far as software reaches. |
| D-V2-118 to 126 (P9-01 to P9-03: `--within`, `grants diff`, proposals, auto-approval) | Each decides through the runtime's mint or the tree's own `⊑` — now with paths (D-V2-139) — and auto-approval never approves guarded authority and is revoked with its node. |
| D-V2-127, 130 (DEADMAN-BUSY-1; a speed claim against a same-machine control) | Measured fixes for the runner-starvation trap; the control keeps the test able to fail. |
| D-V2-128, 129, 131 to 137 (P9-04, the trace's refusals, MAP-HANDLE-1, NET-WILDCARD-1, the red team's F1) | A secret goes only where its audience names, by the broker's recorded and guarded `send`; one matcher per question; `secret_destinations`' static walk matches only the four-argument `get_with_secret`, so it guesses nothing (D-V2-140's concern does not reach it). |

None is revised. One follow-up is carried into the release checks: D-V2-59's documentation note.

## D-V2-143 — The licence stays Apache-2.0 (D27); every archive carries the notices and the bill of materials of everything its binary links, generated from the build's own graph, and refuses to package without them; the GPL microVM guest image is never distributed — the user builds it from pinned sources — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Context.** The commission asks to resolve "the undecided licence". It is not undecided: the owner chose Apache-2.0 on
2026-07-24 (D27 — `LICENSE`, `NOTICE`, `TRADEMARK.md`, `GOVERNANCE.md`); the head chef misreported it that morning
(V2_LOG, 2026-10-17). The commission also asks for MIT, Apache-2.0 and both to be weighed, third-party obligations met,
and the GPL kernel in the microVM image (D-NE-27, PS-C-05) decided.

**Decision.**
1. **Apache-2.0 stays**, weighed against the owner's own intent of 2026-07-24 (HANDOFF §11.1: free to use and sell by
   anyone; the creator's attribution riding with every version and build; the name protected; a patent grant for a
   language aimed at robots). Apache-2.0's §4(d) makes a NOTICE travel with every redistribution — the legal form of that
   attribution — and its §3 grants patents. **MIT alone** has no patent grant and no NOTICE to carry. **MIT OR
   Apache-2.0** (the Rust convention) would let a redistributor take the MIT branch and drop the NOTICE — against the
   owner's stated intent. Every linked crate's licence is compatible (`deny.toml`'s allow-list, checked on every push).
2. **Third-party notices in every archive** (`scripts/third-party-notices.py` → `THIRD-PARTY-NOTICES.txt`): exactly the
   portable build's linked graph for the archive's target (`cargo tree -p delulu --no-default-features --features net -e
   normal --target T` — no build-time tool, no Python-capable build's crates); each crate's own licence and NOTICE files,
   a REUSE `LICENSES/` directory's (the chosen licence's only), the licences of the C code `libffi-sys` and `ittapi-sys`
   bundle, and the Rust standard library's copyright notice; where a crate ships no licence file, the canonical text of
   its declared licence from the Rust toolchain's own copy, with its copyright holders as its `Cargo.toml` declares them;
   `ittapi` from its upstream project's text. **A crate whose licence cannot be shown stops the packaging (exit 1)** —
   witnessed: the first run refused `ittapi` and `ittapi-sys` until their REUSE texts were read. Two crates are
   `GPL-2.0-only OR BSD-3-Clause` (Intel's profiling API, through wasmtime's default `profiling` feature) and are used under
   BSD-3-Clause; no GPL text is shipped. Measured: Windows 257 linked packages and 150 distinct texts; Linux glibc and musl 257 and
   156; macOS arm64 253 and 150.
3. **A bill of materials per archive** (`SBOM.cdx.json`, CycloneDX 1.5, the same graph): 248 components on Windows, each
   with its package URL, declared licence and `Cargo.lock`'s sha256. `docs/release/SBOM-1.0.json` was 1.0's, direct
   dependencies only; SECURITY.md §6's "SBOM per release" is true per archive from this release on.
4. **The legal files are required.** `package-toolchain.sh` copied `LICENSE`, `NOTICE`, `TRADEMARK.md`, `README.md`,
   `CHANGELOG.md` and `SECURITY.md` with `test -f && cp` — a missing `LICENSE` was skipped without a word; now it stops the
   packaging. `distribution.rs` pins both (witnessed red against the old script).
5. **The microVM guest image is never distributed by this project.** It holds a GPL-2.0 Linux kernel (`vmlinux`).
   Distributing it obliges the complete corresponding source for every copy (GPL-2.0 §3) — shipped beside it, or offered
   in writing for three years — and the kernel's notices with it: an obligation the project has no infrastructure to honour
   reliably, for an artifact anyone with KVM can build. So the user builds it: `scripts/microvm/build-image.sh` fetches the
   pinned, sha256-checked kernel source (D-V2-39) and builds it reproducibly (CI's `microvm-reproducible` job). No
   workflow uploads it (only `release-build.yml` uploads, and only the toolchain archives). D-NE-27 is resolved as *never
   distributed by DeluluLang*; PS-C-05 is closed as *won't ship, by decision*. This is the conservative path that needs no
   legal judgement.
6. **One legal uncertainty, named rather than resolved.** DeluluLang has been written largely by an AI under the owner's
   direction; the copyright status of AI-assisted work differs between jurisdictions and is unsettled. `NOTICE` names Jesse
   Sunil as the original creator and copyright holder, by his instruction; the licence's grant to users does not depend on
   the question. Qualified legal advice is the owner's to seek if he wants certainty; nothing here waits for it.

## D-V2-144 — SECURITY.md's pending controls, decided (first part): the release's build and attestation move into a reusable workflow, for SLSA Build Level 3, claimed only once verified; OpenSSF Scorecard measures the public repository — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Context.** SECURITY.md §6 marks five controls `PENDING-PUBLIC`. GitHub's documentation (read 2026-10-17, *Using
artifact attestations and reusable workflows to achieve SLSA v1 Build Level 3*): artifact attestations from hosted
runners give Build Level 2; Level 3 needs the build and its attestation in a reusable workflow, verified with
`gh attestation verify --signer-workflow`.

**Decision (this part).**
1. **`release-build.yml`**, a reusable workflow (`workflow_call`), holds the release's matrix build, the archive checks,
   the attestation (a `v*` tag's run only, as before) and the install gate; `release.yml` keeps its triggers, its dry
   run and its draft release, calls it, and grants it `id-token: write` and `attestations: write`. **Level 3 is claimed
   only after the first release's archives verify with `--signer-workflow
   jessesuniljs1-collab/DeluluLang/.github/workflows/release-build.yml`**; until then SECURITY.md says Level 2.
2. **`scorecard.yml`** runs OpenSSF Scorecard on the public repository only (`if: github.repository ==
   'jessesuniljs1-collab/DeluluLang'`), publishing the results and uploading them to code scanning;
   `ossf/scorecard-action` pinned by commit (`2d1146689b8c…`, v2.4.4 — v2.5.0 was five days old). No floor is claimed
   until scores are read.
3. **The rest** — the disclosure channel, review, signed commits, branch protection, the CODEOWNERS handle — is decided
   with the public repository's creation (the second part of this decision).

## D-V2-145 — rustfmt is adopted: `rustfmt.toml` with the two options nearest the hand style, the tree formatted once, the check in CI — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Context.** RW 7.9 and `ci.yml`'s own comment kept rustfmt out as an owner decision: it rewrites every file, and "this
project's comments carry much of its value". The commission delegates it. A public language project is contributed to by
people who run `cargo fmt` without thinking; an unformatted tree makes every first pull request a formatting argument.

**Decision.**
1. **Adopted**, with `rustfmt.toml`: `max_width = 130`, `use_small_heuristics = "Max"` — measured on the tree as it was
   (2026-10-17): the default style disagreed 7,065 times, `max_width = 130` alone 4,461, both together 2,371. The rest of
   the style is rustfmt's default (edition 2021).
2. **Formatted once, in one commit**, before the public repository's history begins — so the public history has no
   formatting churn in it. 283 files. **No comment's text changed** (the removed and added comment lines were compared
   over the whole diff: rustfmt does not wrap comments; the only differences were three UTF-8 byte-order marks it
   removed); and two test files' long-standing double-encoded characters (`â€”`, `Â§`) were corrected beside it.
3. **Enforced**: `cargo fmt --all -- --check` in CI's `lints` job, `CONTRIBUTING.md` §6 says `cargo fmt --all`.
4. **What it costs**: the `file:line` citations in dated records point at the formatting before this commit; they were
   already snapshots of their date, and the Survey re-derives every live citation.
5. **What it found.** Two gates read Rust *layout*, and the reformat moved it. `json_contract`'s sweep looked for the
   literal `vec!["device"` and rustfmt broke that call across lines — a swept subcommand reported unswept; it now matches
   with whitespace removed. `cli_contract`'s stderr-prefix gate read `eprintln!("…` only on one line — so it had never
   seen any message long enough to be split; rustfmt joined one, the gate found its missing prefix, and taught to read the
   next line too it found a second: **three user-facing error lines without `error:`** (`unknown command …` twice,
   `refused — the source produced no errors …`), prefixed now. Witnessed: the strengthened gate red on the unfixed source.

## D-V2-146 — The on-demand cloud routine: created paused, the owner starts it; a routine run never launches — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Context.** The commission's order has *the scheduled task* as a step. The head chef had meant to leave the first
routine (`trig_01GG41tCGxZXyt7bZUid8Vuu`) disabled. On 2026-10-17 the owner asked instead for *"a new claude code cloud
routine like last time, I will be running it occasionally whenever I can't use my laptop. Let it be paused at start …
Enable Push and Email notifications … run every 5hr and 5 mins"*, modeled on the first routine's settings as he had
modified them.

**Decision.**
1. **Created**: `trig_01HFG6usfzmDAPQigiDu9DLq`, *DeluluLang head chef — on demand (start by hand)* — the first routine's
   settings (Opus 5.5; `delululang-test`; the Default environment `env_012Mxeiw6zvfyxPy9ZEEFf3p`; tools Bash, Read, Write,
   Edit, Glob, Grep, Agent, WebFetch, WebSearch; the Claude-Docs connector; autofix on pull requests) with **push and
   e-mail notifications on** and **`enabled: false`** (read back from the API at creation). Schedule `5 */5 * * *` UTC:
   cron cannot express a 5 h 5 min interval (the routines' minimum is one hour, the expression is cron), so it fires every
   five hours at five past — and the 20:05 → 00:05 gap is four hours. The first routine stays disabled.
   **Amended 2026-10-21, the owner's choice:** he saw the four-hour gap (*"not following 5 hr and 5 min gap as I told"*) and
   chose every six hours: `5 */6 * * *` UTC (05:35, 11:35, 17:35, 23:35 IST), every gap six hours, so never under 5 h 5 min.
   Set through the routine API, read back: `cron_expression` `5 */6 * * *`, `enabled: false`.
2. **Its prompt** keeps the owner's own lines from the first routine verbatim (CI first, finish what a limit cut off, read
   the last commits, record everything, the Survey and doctor, commit as `jessesuniljs1-collab`) and the hard limits
   (only `origin`; no tag or release; no licence change; no kernel image), adds the commission's (never invent a contact,
   key, result or compliance claim), the no-reply commit e-mail and `cargo fmt`, and reads a **Run now** text as the
   owner's direction for that run (the docs, read 2026-10-17: such text arrives in a `routine-fire-payload` block and is
   inert unless the prompt opts in; this routine has no API trigger, so only the owner can send it).
3. **A routine run does not launch.** The commission delegates the release, but its order puts the WSL and VS Code
   validation before it, and both need the laptop (KVM under WSL; VS Code on Windows); the launch needs the owner's `gh`
   login, and the cloud's git proxy reaches only `origin`. So a run does what a Linux VM can verify and names the rest as
   the laptop's in *Open / next* (`docs/CLOUD_ROUTINE.md` step 4, item 3).
4. **Every session starts by syncing with GitHub** (the owner's words, `AGENTS.md` *Read first* 0, `HANDOFF.md` §11.1).

## D-V2-147 — RW 4.19: a driver says `READY` before the program runs; the wait is the operator's flag — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Context.** RW 4.19 (D23's protocol, reserved to the owner until the commission of 2026-10-17): the adapter's 2000 ms
`EXCHANGE_TIMEOUT` began when the first command was written, while the driver had been starting only since just before
`main`. A driver that takes longer to open its port or home an axis failed its first command and poisoned the adapter.
It reached CI twice through the test drivers themselves (PowerShell's start-up; a cold Python's, 4251 ms).

**Decision.**
1. **A driver prints `READY` on its own line before anything else**, once it can accept commands. The host waits for that
   line before `main` runs (`adapter.rs` `await_ready`, for `ProcessAdapter` and `LineTransport` alike); silence, an early
   exit, or any other first line is `AdapterError::NotReady`, the child is stopped, and the run is refused before one
   command is sent. `EXCHANGE_TIMEOUT` is untouched: start-up simply no longer counts against an exchange.
2. **The wait is `--adapter-startup-ms N`**: default 10000 (`STARTUP_DEFAULT`), 1 to 120000 (`STARTUP_MAX`); a value
   outside that, or not a number, is refused before anything runs, and the flag is refused on a run with no driver
   (beside `--adapter-cmd` in the hardware-only list). The RW row proposed declaring it at grant time like
   `heartbeat_ms`; it is a flag instead, because how long a driver takes to start is a fact about the machine the
   operator runs it on, not about what the program may do — and the flag sits next to `--adapter-cmd`, where that
   machine's driver is named.
3. **`delulu device sim` says `READY` too** (`sim::serve`), so the reference device is a correct transport.
4. **Breaking for driver authors**, hence 2.0: a 1.x driver adds one line. CHANGELOG and `DEPLOYMENT.md` say so.

**Evidence.** Unit tests `a_slow_starting_driver_is_waited_for_and_its_first_command_answered` and
`a_driver_that_does_not_say_ready_is_refused_before_any_command` (`adapter.rs`); the CLI test
`a_driver_that_never_says_ready_refuses_the_run_before_main_and_the_wait_is_the_operators` (`hw_adapter_cli.rs`: a
silent driver refused with exit 1 naming the 300 ms it waited; four bad values refused with exit 2; the flag refused under
`sim`); `sim.rs`'s two serve tests now expect `READY` first. Mutants: in the commit message.

## D-V2-148 — RW 4.5: strict anchored-root mode stays opt-in in 2.0; the flip waits for a later major with its migration — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Context.** RW 4.5: legacy root issuance is the default, so a same-user process can mint root authority unless the
operator starts the broker with `--require-anchored-roots` (DISC-1's mitigation). The commission delegates the default.
`ROOT_ISSUANCE_TRUST_BOUNDARY.md` §10 analysed the flip: it breaks `grants delegate` without `--parent` and `run
--grant` against a broker — every first run and quick-start — and its own recommendation is *"keep strict mode opt-in
now; gather evidence; plan a default flip for a major version with the migration above … Do not sell it as airtight
without the deployment-side key custody, which is the real boundary."*

**Decision.** **Opt-in in 2.0.** 2.0 is a major version, but none of §10's migration exists yet (the
`--allow-unsigned-roots` escape, the auto-root re-pointed at adopting a pre-issued root, the quick-starts rewritten), no
user evidence has been gathered, and on the default deployment — one OS account — the anchor key sits where the same
agent can read it, so a default flip would break every first run while protecting little. The flip is planned for a
later major, with §10's migration, once deployments report how they hold the anchor key. `DEPLOYMENT.md` and `SECURITY.md`
keep telling a production deployment to turn it on. RW 4.5 stays open with this as its plan.

## D-V2-149 — The release is 2.0.0 — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Context.** The workspace says 1.0.0 (tagged 2026-07-20). Since then, changes a 1.x user meets as breaks: a hardware
driver must say `READY` first (D-V2-147); a secret's name can no longer hold `@`; a compiled program or a Contained plugin
may no longer use the WebAssembly 3.0 features the engine turned on by default; on Windows a foreign library no longer
falls back to the application's or the system's directories; an ordinary `run` is held to 1 GiB of memory and five
minutes of processor time unless `--limits` sets others (PS-B-01, D-NE-31); and the broker daemon's wire refuses bytes
after a frame's value (CHANGELOG, the *Unreleased* sections).

**Decision.** **2.0.0**, by Semantic Versioning: each item above can stop a 1.x program, driver or deployment that worked.
The bump and the CHANGELOG's `## [2.0.0]` heading (the *Unreleased* sections consolidated under it) are made at the
launch, in the commit the tag names, so the version never claims a release that did not happen.

## D-V2-150 — P8-04 option (c), a revoke-only principal in the grant tree, goes to an RFC after the release — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Context.** P8-04's monitor is a node minted between the operator and the run (D-V2-101 onward); option (c) — a
principal that can revoke and do nothing else, placed in the tree — was reserved to the owner because it changes who may
hold authority. The commission delegates it.

**Decision.** **Deferred to a public RFC after the release**, not built for 2.0. It changes the holder model —
what a node in the grant tree is — which is Authority's meaning, and `AGENTS.md` lets the head chef harden Authority but
not redefine it without the process; the RFC process's own debt (D21, a core authority change shipped without its comment
period) is the reason not to repeat that shortcut on the first public release. What 2.0 ships is unchanged and complete
as far as software reaches (D-V2-116): the monitor revokes its own subtree and nothing else.

## D-V2-151 — The code of conduct: the Contributor Covenant 2.1, reports through the private reporting form — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Context.** A public project needs a code of conduct; `HANDOFF.md` carried it as an owner decision. The commission's
limit applies: never invent a contact.

**Decision.** `CODE_OF_CONDUCT.md` is the **Contributor Covenant 2.1, verbatim** (taken on 2026-10-17 from its official
source, `EthicalSource/contributor_covenant`, `content/version/2/1/code_of_conduct.md`, its front matter removed), with
its one placeholder, the contact method, filled by a channel that exists: the repository's private reporting form (the
**Security** tab, **Report a vulnerability**, a title beginning "Conduct:"), which reaches the maintainer alone, and
GitHub's own **Report content**. No e-mail address is invented. The public repository's private reporting must be on
for the channel to work — a launch item.

## D-V2-152 — The two rule lines that spelled the banned word are reworded; the tree no longer contains it — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Context.** The owner's ruling: a third-party tool's name appears nowhere in the repository. Two lines of `HANDOFF.md` (the §1
rule table and §11.1) stated the rule by spelling the name, and §1.1's pre-public item 1 said to reword them before the
public repository. The public history starts fresh (§1.1), so only the tree matters there.

**Decision.** Both lines now name it as *graph* and *ify* written as one word — enough for an agent to obey the rule, without
the word itself; `AGENTS.md` gives the check (`grep -ci 'graph''ify'`). `git grep -i` over the tracked tree finds nothing
(2026-10-17). The old commits that spell it stay in this development history, which is never rewritten (`HANDOFF.md` §1).

## D-V2-153 — The public repository: its controls now, and development moves there at the launch — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Context.** `HANDOFF.md` §1.1 names `jessesuniljs1-collab/DeluluLang` (public, empty, default branch `main`, created
2026-07-03) as the final repository; the commission delegates its setup, and the launch cannot start before it is decided
where development continues afterwards — the push rule, the routine, cloud sessions and the start-up hook all name a
repository.

**Decision.**
1. **Controls, set and read back on 2026-10-17** (the owner's `gh` login, scopes `repo`/`workflow`): private vulnerability
   reporting **on** (it was off — the code of conduct's channel, D-V2-151, needs it); secret scanning and push protection
   **on** (already); Dependabot alerts **on**. Branch protection of `main` — no force-push, no deletion — is set as soon as
   `main` exists, at the snapshot push.
2. **After the launch, development lives in the public repository**, on `main`, from the 2.0.0 fresh-history snapshot:
   issues and pull requests from users arrive there, CI runs there on public runners, and the routine and cloud sessions
   work where users can see it. A two-repository model (develop here, export snapshots there) would strand every outside
   contribution. **`delululang-test` becomes the private development archive** once the release is verified: its history
   is kept whole and never rewritten (the commission's limit), and every commit hash or run id cited in a record dated
   before 2.0.0 refers to it — the snapshot's commit message and `HANDOFF.md` say so.
3. **The cutover happens in the launch commit itself**, so the public tree states its own rules: `AGENTS.md`'s push rule
   (origin = `DeluluLang`, branch `main`), `CLAUDE.md`, `docs/CLOUD_ROUTINE.md` (`master` → `main`), the `SessionStart`
   hook (`origin/master` → `origin/main`); then, outside the tree, the laptop's `origin` and the routine's repository (by
   the API, read back). Until that commit, nothing changes: this repository stays `origin`.

## D-V2-154 — `SECURITY.md`'s controls decided and CODEOWNERS named — TAKEN (head chef, 2026-10-17, under the owner's delegation)

**Decision.** (1) **Reporting:** GitHub's private reporting form on the public repository is the only channel; no `security@`
address and no PGP key, because none would be monitored — the commission forbids inventing either. (2) **Two-person review:
not adopted** — one maintainer; `main` refuses force-push and deletion, CI gates every change, CODEOWNERS names the
maintainer. (3) **Signed commits and tags: not adopted** — the release's build attestations vouch for what users install.
(4) **SLSA:** Build Level 2 per release (GitHub artifact attestations from `release-build.yml`); **L3 claimed only after**
`gh attestation verify --signer-workflow …/release-build.yml` passes on a published release. (5) **Scorecard:** runs, no
floor claimed. (6) **CODEOWNERS:** `@jessesuniljs1-collab` replaces the placeholder. `governance.rs`'s
`security_controls_are_decided_and_no_claim_runs_ahead_of_its_proof` replaces `pending_public_controls_are_still_marked_as_pending`
and holds the new line; watched red on the old text. Entrenched record: this date. RW 7.8 closed.

## D-V2-156 — A Guard path rule compares in NFC on macOS, whose filesystem ignores Unicode normalization (RW 4.17) — TAKEN (head chef, 2026-10-18, under the owner's delegation)

*(Numbered 156: D-V2-155, the support matrix at 2.0, lives on `launch/2.0.0` until the launch.)*

**Measured first** (on-demand routine run 1, `witness.yml` on `macos-latest`): APFS opens `café` written precomposed
(NFC) and decomposed (NFD) as one file, and `realpath` already resolves an EXISTING file's two spellings to one — so the
Guard, deciding on the resolved spelling since GUARD-ALIAS-1, was right for every file that exists. A name that does not
exist yet keeps the spelling it was written in (`canonical_existing` re-appends it), and the Guard compared it as
written: a seal on `/srv/out/café.txt` (NFC) set before the file exists did not cover a write to
`/srv/out/cafe\u{301}.txt` (NFD) — the same name to the filesystem. Witnessed red on the runner (`80a6243`).

**Decision.** (1) **Normalize, not refuse**, and only in the Guard's comparison: on macOS both sides of a path rule are
compared in NFC after the case fold the same function already does there (`guard::path_within`). Refusing every
non-NFC path would refuse a decomposed name a macOS tool wrote legitimately; normalizing at the comparison also covers
rules stored before this change, in either spelling. (2) **Nowhere else:** on ext4 and NTFS the two spellings are two
files, so a permit on one must not reach the other — folding there would widen a permit; and the grant lattice keeps
comparing as written, where a mismatch can only refuse. (3) **The normalizer:** `icu_normalizer` 2.3, already in the
tree (`url` → `idna` → `idna_adapter`; Unicode-3.0, allowed by `deny.toml`), taken by `delulu-broker` on macOS only —
not a new crate, and the broker stays transport-free (its Ruling 1). RW 4.17's other half — a UNC path in an operator's
grant on Windows — stays open.

## D-V2-157 — A run's own root node ends with the run: revoked at its end, reaped by a dead-man when it is killed (RW 4.57) — TAKEN (head chef, 2026-10-19, under the owner's delegation)

**Measured first** (on-demand routine run 2: a scratch daemon, then three witnesses in `plugin_load_cli.rs`, red on the
old code). Under the daemon, `run --grant …` issues a root node for itself alone (issue-then-run, `STAGE5_SPECIFICATION.md`
§3.2). After a run that ended CLEANLY its plugin's node was revoked (RW 4.51) and its own root stayed `[live]`; after a
run stopped by its budget or killed, both stayed `[live]` — 30 s after a `kill -9`, still `[live]`. The daemon's tree
lives in its memory, so every run left root authority nobody held, for the daemon's whole life, and `grants list` showed
it to the operator as live. RW 4.57 (the red team's F-01, routine run 17) had named the killed half from code reading;
the clean-end half was new.

**Decision.** (1) **A run's own root is held for exactly as long as the run** (`broker_client::RunNode`, at both places a
root is issued for a run: `run_cmd.rs` and the sandboxed host in `guest.rs`). At the run's end it is revoked AS ITSELF —
after `close_devices`, so the device nodes' order is unchanged — and transitively, so anything still live below it goes
with it. (2) **A run that never reaches its end is reaped by the broker's dead-man** (D-V2-106), armed on the root as it is
issued and beaten by a thread of the run's own every fifth of the period: when the beats stop, the daemon revokes the root
and its subtree as itself, its record naming the silence. A budget stop ends the process without returning (`process::exit`),
so it is reaped this way too. (3) **The period is 10 s**, generous on purpose — it reaps a dead run's authority, and a
healthy run whose beats a loaded machine delayed past it would lose its node mid-run; `DELULU_RUN_DEADMAN_MS` may only
SHORTEN it (tests; an operator who wants a killed run reaped sooner), as `DELULU_FOREIGN_CALL_DEADLINE_MS` may only
shorten its bound. (4) **A refused arming refuses the run** — its node revoked at once, the daemon's code and words
reported: a run whose authority could not be reaped does not start. (5) **Not a lease run's own node:** it is the delegator's
(a `--multi` lease is redeemed by several runs), so no one run revokes or dead-mans it. **What a lease run MINTS under it
is dead-manned as it is minted** (the same run, `6f98ae5`): its custody client arms a dead-man on each node it mints — a
lease run's are its devices', since a lease carries no plugin authority — and the run's beater keeps each alive; a node
that cannot be armed is revoked and not handed out. Killed, the run's minted nodes are reaped and the lease lives on
(witnessed, plain and sandboxed). *(This point first said the lease half was left open; it was built later the same run.)* (6) **Hardening, not redefinition:** a node may always revoke itself, and arming a future
self-revocation adds no authority (D-V2-106's argument); the holder model (§3.3) already says a node is its holder's —
the tree now says when the holder is gone. (7) **A run that is suspended past the period is treated as dead** — Ctrl-Z,
`SIGSTOP`, a debugger's halt: its beats stop, the daemon reaps its node, and once it resumes its next use is refused (fail
closed), the record naming the silence. Not measured: a host that sleeps past the period — whether the daemon's monotonic
clock counts the sleep is the platform's; where it does, a run's node may be reaped on waking in the same way. The cost, stated: each daemon run's chain gains two records (its root's
`deadman-arm` and its end's `revoke`) and its broker one beat every 2 s per run.

## D-V2-158 — A monitor reads only what was appended to the chain since its last read, and acts only on a record that names the one before it and whose hash is its own (MONITOR-CHAIN-1, RW 4.49) — TAKEN (head chef, 2026-10-19, under the owner's delegation)

**Measured first** (on-demand routine run 3, `V2_LOG.md`): `monitor watch` read the whole chain on every poll (`audit::query`,
about 3 µs a record in a release build) and beats its dead-man between polls, so a healthy monitor over a long chain was
revoked by its own broker — and every run under it — once one read outlasted the period: 250,000 records and the default
2 s did it by hand, with no run doing anything. RW 4.49 had recorded the cost as a LOW latency.

**Decision.** (1) **A watch's poll costs what was appended since the last, never the chain's history**:
`delulu_broker::audit::Follower` reads the day file it stopped in from the byte it stopped at, and every later day file
(AUDIT-DAY-1 keeps every record at or after the newest file). (2) **Only whole lines**: a line without its line break may
still be growing, and is left for the next read; the end a follower starts from never counts one. (3) **A monitor acts only
on what the chain vouches for**: a record must name the previous record's hash and its own must recompute — the link check
`verify` makes. A line that does not is UNVERIFIED: named on the monitor's standard error with its seq, never judged, and
the follower goes on from its stored hash. The monitor does not stop for it: a stopped monitor's dead-man would quarantine
every run under it, which would make any writer able to damage one line the owner of that switch. (4) **A rewritten chain**
(the day file cut short or gone) is said and skipped to the end, as the old cursor-by-hash did when its record vanished.
(5) **What it does not claim:** a same-user writer who computes the hash appends a record that verifies (category 7, the
same-uid boundary, §11.4); `audit verify` with the anchor, and an external witness of the head, stay the chain's whole
check. (6) **Hardening, not redefinition:** the monitor's rules, its node, its dead-man and what a quarantine revokes are
unchanged; it reads less, and acts on less.

## D-V2-159 — A writer catches up with the chain by reading only what was appended since it last wrote, verified, and falls back to the whole recovery for anything else (RW 4.53) — TAKEN (head chef, 2026-10-19, under the owner's delegation)

**Measured first** (on-demand routine run 3, `V2_LOG.md`): beside another writer on a 100,000-record chain, a recorded daemon
request took median 3,817 ms — every catch-up re-read and re-counted the whole chain under the append lock — and 14 ms alone.

**Decision.** (1) `AuditLog::catch_up` reads the rest of the day file this handle wrote and every later one, each record
checked as `verify` checks a link, and accepts the reading only when the anchor then agrees with its head and count — the
same anchor `catch_up`'s fast path already trusted with the tail file's length. (2) **Everything else falls back to the whole
recovery**, unchanged: a cut file, a broken link, an unanchored line, a missing file are refused (or recovered) exactly as
before; the incremental read never accepts what the recovery refused. (3) **Not claimed:** a record behind the handle's
point that was damaged after it was written is no longer re-read by a writer's catch-up — it never was by its fast path, and
`audit verify` (with the anchor and an external witness of the head) is the chain's check. (4) `AuditLog::open` is unchanged:
it still recovers from every file (RW 4.53 keeps it).

## D-V2-160 — The audit anchor notes where the chain stood, and an opening whose files still measure so reads only the newest record (RW 4.53) — TAKEN (head chef, 2026-10-19, under the owner's delegation)

**Measured first** (on-demand routine run 3, `V2_LOG.md`): every `AuditLog::open` read every day file twice — 180–303 ms at
100,000 records in a release build, paid by every sandboxed run's host and every command that records.

**Decision.** (1) `ANCHOR.json` keeps `head` and `records` and gains `day`, `day_len`, `day_records`, `older_bytes` and
`max_seq` — fixed in size, written with every append as before. (2) An opening takes the anchor's count and seq only when the
newest day file is the one it names and of its length, the older files' bytes are unchanged, and the newest file's last
record is the anchor's head; otherwise the whole recovery runs, unchanged — what it refused (a cut file, a deleted or added
line in any file, a replaced last record, an anchor that disagrees) is still refused at the opening. (3) **What the opening no
longer re-reads:** an older record edited in place to the same length. The opening never recomputed hashes, so such an edit
that kept the line parseable was never its to find; one that broke the line's JSON was, by the count, and is now `verify`'s
alone. Named, not hidden. (4) An anchor without the note — every log written before — is read the whole way once and
re-affirmed with it; an older binary reads `head` and `records` and ignores the rest. (5) Hardening the cost, not the meaning:
the chain, its records, `verify` and the anchor's two original fields are unchanged.

## D-V2-161 — A monitor asks the daemon about the nodes its records name, never for the whole tree; an answer too large for one frame is refused in words (RW 4.69) — TAKEN (head chef, 2026-10-19, under the owner's delegation)

**Measured first.** The on-demand routine's third run timed the cost (RW 4.69: at 30,003 nodes a recorded request's max went
from 17 to 371 ms beside a monitor polling every 25 ms). The laptop then found the cliff behind it (2026-10-19, Windows,
debug build): every wire frame is capped at 16 MiB (`broker_ipc::MAX_FRAME`); the tree keeps every node it ever held with its
holder's description, words of any length; and past the cap the daemon dropped the connection ("response write failed").
The monitor read that as `broker unreachable`, stopped, and its dead-man revoked its node and the run under it — 550 nodes of
30,000-byte descriptions, none under the monitor, did it.

**Decision.** (1) A new additive `broker/1` request, `RunsUnder { root, nodes }`, answered `Runs { state, runs }`: the root's
effective state, and for each asked node under the root the run it belongs to — the child of the root on its path, or the root
itself (`Broker::run_under`, a walk up parent pointers: its cost is the depth of what is asked, never the size of the tree). At
most `MAX_RUNS_ASKED` (4,096) names a request; more are refused DL1401, and the monitor asks in parts. (2) The monitor asks its
node's state at the top of each poll (a `RunsUnder` naming nothing), and the runs of the actors its newly read records name
AFTER reading them — so a node minted between an earlier question and the reading of its records is in the tree when asked
after. The old order (list, then read) skipped such a node's records for good; that is closed by construction and not
witnessed (the window was a few milliseconds, and no test holds it open). (3) An answer whose encoding is larger than one
frame is answered DL1401 in words and the connection kept for it, never dropped: `write_frame` writes nothing of an oversize
frame and says so with `broker_ipc::FrameTooLarge`. (4) Not changed: `grants list`, `grants tree` and `atlas`'s custody overlay
still read the whole tree, and past one frame they now say why they cannot (RW 4.70, open); `List`, its answer, and the
monitor's rules, dead-man, judging and report are unchanged. Hardening the monitor's reach, not what it means.

## D-V2-162 — The whole-tree reads come in pages, and the tree is drawn in the state the broker acts on (RW 4.70) — TAKEN (head chef, 2026-10-19, under the owner's delegation)

**Measured first** (the laptop, 2026-10-19): past one 16 MiB frame `grants list`, `grants tree` and `atlas`'s custody overlay
could not read the tree (RW 4.70, opened beside RW 4.69); `Broker::tree` found each node's children by scanning every node —
0.13 s at 2,000 nodes, 0.54 s at 4,000, 2.6 s at 8,000, 10.9 s at 16,000 (debug build), the serial daemon serving nothing
else meanwhile; and the tree drew each node in its OWN state, so a child of an expired grant read `[live]` where `list`,
`inspect` and every check said expired (TREE-STATE-1, witnessed with the real binary).

**Decision.** (1) A new additive `broker/1` request, `ListPage { after }`, answered `ListedPage { nodes, next }`: the nodes
after `after` in id order, as `List` gives them, at most `LIST_PAGE_BYTES` (4 MiB) of encoded entries a page and at least one;
`next` is the page's last id, `None` on the last page. One entry larger than a frame is refused in words, as any answer that
size is (D-V2-161). (2) `brokerd::list_all` reads every page; `grants list`, `grants tree` and the atlas overlay use it. A
paged listing is not a snapshot, and says so: a node made or revoked while it is read may or may not appear; a node whose
parent the pages do not hold is drawn as a root. (3) `grants tree` is drawn by the CLI from the pages (`cli::render_tree`), in
the daemon's text format — a test holds the two to one text byte for byte. (4) The daemon's `Tree` stays for older clients:
its children from one index, drawn from a stack rather than a call per level (a node's depth has no bound at its creation),
each node in its inherited state. (5) `List` and its answer are unchanged. Display only: no decision reads either text.

## D-V2-163 — The WASM fragment takes loops, a growing memory and `Str.len()`; an inlined body sees the names of where it was written (RW 3.4, WASM-HYGIENE-1) — TAKEN (head chef, 2026-10-19, under the owner's delegation)

**Measured first** (the laptop, 2026-10-19): five of the six Study-C kernels were DL1201, `loop_sum_1m`, `string_build_20k`
and `nested_calls_macro` on `while` alone (and `Str.len()`); the WASM heap was a fixed 1 MiB, never reused, so a program
whose strings added up past it trapped (`DL0903`) where the interpreter ran it — witnessed with a recursion that compiled
already; and the backend's inliner looked a name up through every frame below it, so a lambda passed to a generic function
read the generic's own variable (WASM-HYGIENE-1: 103 on WASM, 4 on the interpreter).

**Decision.** (1) `while c { body }` lowers to `block { loop { br_if 1 (!c); body; br 0 } }`; `break` and `continue` branch to
the enclosing loop's block and loop, counted by a label depth that `Cx::emit` keeps for every `block`, `loop`, `if` and
`end`. (2) Assignment to a local: the variable's slot, the value compiled at its type; the checker has already held it to a
`var`. An assignment from inside an inlined function value to a variable of the frame that defined it is refused (DL1201):
the interpreter's closures share their frame, and the backend does not reproduce that sharing for writes — it is not
guessed. An inlined body's `break`/`continue` reach no loop of its caller's. (3) `for` and every `List` stay DL1201. (4)
Every guest allocation goes through one helper, `__alloc`, which grows the memory by the pages an allocation needs, and
traps when it cannot; the host's allocator grows it by the same rule. The run's memory budget holds the growth, as it holds
the interpreter's. (5) `Str.len()` counts Unicode scalar values, as the interpreter does (`__str_len`: the bytes that do not
continue a character; a guest string is UTF-8). (6) An inlined body sees the frames of where it was written and no others:
a generic function's body none of its caller's, a lambda's the frames visible at its definition (`Callable` records them;
`Cx::hidden` hides the rest from every lookup). (7) The interpreter stays the reference engine and `--engine wasm` stays
opt-in. The Study-C table is not re-measured by this decision (`HOT_PATH_TABLE.md` says so).

## D-V2-164 — The WASM fragment takes `List`: shared headers, a snapshot `for`, `split` and the index fault in the interpreter's own words (RW 3.4) — TAKEN (head chef, 2026-10-20, under the owner's delegation)

**Measured first** (the laptop, 2026-10-20): the last two Study-C kernels, `wordcount_macro` (`Str.split`, `len`) and
`list_map_macro` (a list literal, `map`, `len`), were DL1201 on `List` alone. D-V2-163 (3) had left `for` and every `List`
outside. Two defects were found and witnessed red while building this:
- **WASM-IFTY-1.** `if`'s block type was guessed from the then-branch's syntax, and a variable was guessed `Int`. An `if`
  choosing between two `Str` variables compiled to a module wasmtime refused at run time ("invalid WASM module", DL0904),
  and a `match` arm binding a `Str` was refused.
- **WASM-INTERN-1.** String literals inside a lambda were never interned. `apply(fn(s: Str) -> Str { s + "!" }, "a")`
  panicked `compile_module` at `expect("interned")`.

**Decision.**
1. **The representation.** A `List[T]` of a scalar (`Int`, `Bool`, `Float`, `Str`, a sum type) is an i32 pointer to a
   header `[len][cap][data]`, with `cap` 8-byte slots at `data`. `let ys = xs` names the same header, so a `push` through
   either is seen by both, as on the interpreter (`Rc<RefCell<Vec>>`). A list of lists stays DL1201.
2. **Growth.** `__list_slot` doubles a full list's slots into a fresh allocation and leaves the old ones untouched; the
   bump heap never frees.
3. **`for`.** It reads the length and the slots pointer once, so it walks the list as it was when the loop began. That
   is the interpreter's snapshot, made exact by (2) and by `pop` staying outside: `pop` would let a later `push` reuse a
   slot inside a running `for`'s snapshot. `pop`, `filter`, `find`, `fold`, `sort`, `reverse` and `concat` stay DL1201
   until each is built.
4. **`push`.** It computes its value before the list grows, as the interpreter does: `xs.push(xs.len())` pushes the old
   length.
5. **`get` and indexing.** `get` answers `None` outside the list, a negative index included. An index outside it faults
   DL0903 in the interpreter's own words, through `delulu:list.index_fault`, whose record `trap_or_refusal` reports.
6. **`Str.split`.** It runs host-side by the interpreter's rule (`delulu:list.str_split`): the same Rust `split`, and
   `chars` for an empty separator. A pure helper needs no grant and records no effect.
7. **Imports and helpers.** The two `delulu:list` imports and the two helpers are emitted only in a module that uses a
   list, after the actors' imports and after the user functions, so every other module's bytes are unchanged.
8. **Reference capabilities.** A reference-capability prefix (`ref List[Int]`) is looked through: it is the checker's
   discipline, and the value is represented the same either way.
9. **`run_int_fn`.** It runs under the full host with nothing granted, so a pure function that uses a list can run.
10. **Block types (WASM-IFTY-1).** An `if`'s and a `match`'s block type is set from what the first branch compiled to, and
    the context-free guess is deleted.
11. **Interning (WASM-INTERN-1).** String collection descends into lambdas, lists, records, fields and indexes, and so
    does the import scan.

**Evidence.** `list_parity.rs` (9 tests); `loop_parity.rs`'s random programs now build, push, index, `get`, walk and map
lists (176 of 400, asserted over 120); mutants M533–M540 red, control green; the release differential in `V2_LOG.md`.
**Not re-measured:** Study C's table keeps its 2026-10-19 rows; all six kernels compile and print the interpreter's
answers, and a run over all six on one machine is next.

## D-V2-165 — An `iso` move carries its graph whole, cycles and shared cells included, and the debug race checker asks only for references from outside it (RW 2.16, MSG-CYCLE-1) — TAKEN (head chef, 2026-10-20, under the owner's delegation)

**Found** (the laptop, 2026-10-20, while making `==` and `str` safe on a value that holds itself, RW 2.13–2.14): `recover
{ var k = Node { v: 7, next: None }; k.next = Some(k); k }` checks clean as an `iso Node` — spec §3's `recover` lifts
whatever the block built — and moving it to an actor overflowed the stack and killed the process. `value_to_msg` walked
the graph as a tree. Two more things followed from the same shape:
- `--debug-rcaps` reported that graph, and the legal `recover { let a = [1]; [a, a] }`, as a compiler bug (DL1610). Its
  walk wanted every node below the top held exactly once, following spec §6.2's *"debug builds verify refcount==1
  across the moved graph"* — a proxy for the guarantee, not the guarantee. The same walk never looked inside a variant
  or a map, so an alias from outside reached through `Some(xs)` passed unseen.
- A graph holding one list twice arrived holding two copies, so a `push` through one was not seen through the other, as
  it is on the sending side.

**Decision.**
1. **The move carries the graph.** Each `List`, `Map` and `Record` cell is numbered where it is first met, before what
   it holds. A cell met again crosses as `MsgValue::Back(n)`. The receiver makes each cell empty, records it, then
   rebuilds what it holds into it. A moved graph that holds itself or holds a cell twice arrives as the same graph.
2. **The debug race checker asks for the guarantee.** A mutable node's strong count may be no more than the references
   the moved graph itself holds to it (the top node: those, plus the binding's and the evaluator's two). One more is a
   reference from outside, which the static proof said there was not: DL1610, as before. The walk goes through variants
   (each payload once) and maps. Spec §6.2's sentence is amended to say so.
3. **The sandbox channel refuses a value that holds itself.** Its wire is a tree (canonical CBOR), and it already refuses
   rather than approximates what cannot cross honestly: `Untransferable("a value that holds itself")`, reported as
   DL1401 like the channel's other refusals. A cell held twice without a cycle crosses as plain data, as before.
4. **No new code.** DL1610 keeps its meaning, a uniqueness violation; it simply stops firing on legal programs.

**Rejected.** Refusing a cyclic graph at the actor boundary. The program is legal, the move is sound (no reference
outside the region survives it), and a runtime refusal of a checked program is the checker-bug class this decision
removes. Narrowing `recover` to tree-shaped results was rejected for the same reason: it would redefine what `iso` means.

**Evidence.**
- `actors_cli.rs` `a_graph_that_holds_itself_moves_to_an_actor_whole` (plain and `--debug-rcaps`): the process died
  before; DL1610 under the flag.
- `actors.rs` `move_graph_tests` (shared cell and cycle survive the move, by pointer identity).
- `actors_trace.rs`:
  - `an_aliased_nested_node_is_a_dl1610_class_violation` rewritten to hold the list OUTSIDE the graph — it held it twice
    inside, which `recover` admits;
  - `a_graph_holding_a_node_twice_or_holding_itself_passes_the_uniqueness_walk`;
  - `an_alias_reached_through_a_variant_or_a_map_is_a_dl1610_class_violation`.
- `channel.rs` `a_value_that_holds_itself_is_refused_and_one_held_twice_crosses`.
- Mutants M583–M590 red, control green.

## D-V2-166 — A function has no structural equality; an opaque type inside a record field is seen (RW 2.15, 2.17) — TAKEN (head chef, 2026-10-20, under the owner's delegation)

**Found** (the laptop, 2026-10-20, asking after EQ-RECORD-1 whether the runtime answers `==` for every type the checker
admits):
- **EQ-FN-1.** `let g = fn(x: Int) -> Int { x }; g == g` checked clean and answered `false`. The checker treated a
  function type as comparable, and `Value::eq` has no answer for two closures.
- **OPAQUE-FIELD-1.** The walk over a record's (or sum's) written field types, `type_expr_opaque`, named no `Map`, no
  `Plugin` and no actor. So a record holding `Map[Str, Secret[Str]]`, a `Plugin[Verified]` or an actor reference was
  admitted to `==` (and to `str`), though the same type outside a record is refused (DL0605, DL0604). No secret byte
  was exposed: `Value::eq` answers `false` for two secrets and `display` prints «secret». But the rule R-5 states, that
  an opaque value has no equality wherever it sits, did not hold.

**Decision.**
1. **A function has no structural equality.** `==`, `!=`, `assert_eq` and `List.contains` refuse a type that holds a
   function — bare, in a list, an option, a map or a record field — with DL0605, in its own words ("a function has no
   structural equality"; label: compare what the functions compute). Identity comparison of closures is not a promise
   the language makes; `str` of a function stays `<fn>`. No new code: DL0605 is "opaque type has no structural
   equality", and a function is opaque to equality. Its `explain` text says so.
2. **One walk, two questions.** `opaque_for(t, fns)` is the walk; `is_opaque` (str, DL0604) asks without functions and
   `lacks_equality` (the three equality sites) asks with them. The field walk names what the type walk names: `Map`
   (both sides), `Plugin`, and an actor's name.

**Evidence.**
- `delulu-check` `equality_on_a_function_is_dl0605_in_its_own_words` (five shapes, and `str(g)` still checks) and
  `an_opaque_type_inside_a_record_field_is_seen` (a secret map, an actor, a plugin, `str` of the secret map, and a plain
  record that still compares and prints). Both were red on the old `check.rs`.
- Mutants M591–M598, red, control green.

## D-V2-167 — A file capability is anchored where its directory resolved when it was minted or narrowed; no use resolves the root again (RW 4.72) — TAKEN (head chef, 2026-10-20, under the owner's delegation)

**Found** (the red-team pass of on-demand routine run 5, a Sonnet 5.5 sous-chef, F2 and F3): a file capability's root
was resolved again at every use, on both engines. A directory of the scope swapped for a link after a `narrow` — or after
a mint on a sub-directory of the grant, or on the grant itself — carried the capability with it: reads and writes reached
outside the grant, with no refusal. FS-RACE-1 (`beneath.rs`) refuses a link that appears on the path below a root; the
root was the one part re-resolved, and `beneath.rs` named the granted directory's own path as a residual.

**Decision.**
1. **A capability's directory is decided once, at its mint or its narrowing.** `CapScope::Fs` (and the WASM host's file
   handles) carry `anchor`: the root's resolution then. `prim::mint_anchor` makes the grant decision on that same
   resolution, so the decision and the anchor cannot come apart; a narrowing's anchor is its subtree's resolution, inside
   its parent's.
2. **No use resolves the root again.** The scope gate (`resolve_in_anchor`), the pin the custody and the Guard decide on
   (`pin_fs_at`) and the path opened (`pinned_in`, then `beneath`'s link-refusing walk) all compare against the anchor. A
   use through a directory that has become a link since the mint is DL0904, in `resolve_in_scope`'s words — no new code.
3. **The residual "the grant's own path" is closed for swaps after the mint.** A grant that is already a link when a
   capability is minted is followed then, as before: the operator's grant names a place, and the mint fixes where it is.
4. **What does not change:** the trace and the custody name the root as spelled; HARDLINK-1 stays a residual.

**Evidence.** `prim.rs` `a_capability_directory_swapped_for_a_link_after_its_mint_reaches_nothing_outside` and
`delulu-wasm` `a_narrowed_directory_swapped_for_a_link_reads_nothing_outside`, both red on the old code; mutants M643–M647
(`V2_LOG.md`, 2026-10-20).

## D-V2-168 — P9-05 (c)'s static half: `authority --json` names each request by the checker's type at its call; a package's report waits for per-module types (RW 2.12) — TAKEN (head chef, 2026-10-21, under the owner's delegation)

**Context.** D-V2-140 §2 kept `authority`'s static path report open because the AST walk could not tell an `Http`
capability's `get` from a `Map`'s without type information, and a report that guesses which call is a request is not one
this project makes. The checker's per-node types (`CheckResult::node_types`) are now used by the WASM backend (RW 3.4), and
they answer exactly that question.

**Decision.**
1. **Built for a program checked as one file.** `authority --json` gains `requests`: each `get`, `get_with` and
   `get_with_secret` whose receiver the checker typed `Cap[Http]`, with the URL literal and its host and path by the egress
   client's own parse. A computed URL, or one that client refuses (not `https://`), is `null` there. The key is present only
   when the program makes a request, so every other report is byte-identical (criterion 11); the closed schema admits it.
   The walk is `secret_destinations`' own, given the types, so the two static halves cannot read the source two ways.
2. **A package's report waits.** `check_program` keeps no per-node types, and node ids are per module, so a package's
   modules would need their own maps. That is a slice of its own (RW 2.12), not a guess in the meantime.
   **Built the same evening:** `Program::node_types` keeps one map per module, keyed by its unit name as both package checks
   (`check_program`, `check_workspace`) key everything else. The report pairs each module with its own map by that key,
   never by a declared name a workspace need not keep unique. So a package's, a dependency's and a loose directory's
   requests are named (`within_cli.rs`; M700–M702 red).
3. **Methods (d) still wait for the first non-GET primitive; the path in the chain's record stays open** (D-V2-140 §3, §4).
4. **The core-invariance snapshot moved by one case**, `examples/guide/05_capabilities.delulu :: authority --json`, which
   gains its one request (`client.get(url)` inside `fetch`, the URL a parameter: `null`). Re-recorded as a reviewed act
   (D-NE-3), its diff in `V2_LOG.md`.

**Evidence.**
- `within_cli.rs` `authority_names_each_request_by_the_checkers_type_and_never_by_a_methods_name`: a literal, a parameter
  and an `http://` URL are named, a `List`'s `get` is not, and the schema validates.
- Mutants M696 (decided by the name), M697 (never stamped), M698 (the path from the host) and M699 (the schema without the
  key), each red.

## D-V2-169 — P9-05: the path in the chain's record — a network use carries its request path to its custody, the broker judges it against the node's grant, and the use's record names it in its payload (`url_path`), `target` staying the host — TAKEN (head chef, 2026-10-21, under the owner's delegation)

**Context.** D-V2-139 §6 left the path out of the broker's `Net` op and record: every reader of a `use` record reads
`target` as the host (the OCSF export's `dst_endpoint.hostname`, an egress refusal's record, `audit query`), and a separate
field would change the record's canonical shape. Reading the code for this slice found what the omission cost. Under the
daemon, the custody gate runs before the effect's first check, and the broker compared the host alone. So a request to a
granted host, for a path outside every pattern the grant names for it, was recorded as the host's `use allow` and then
refused at the call (DL0904). Witnessed with the pre-change build (`launch/2.0.0` `678dee6`, Windows), under a daemon,
granted `net.special=*.example.invalid/v1/**`: a request for `/v2/items` was refused DL0904 at the call, and the chain's
seq 9 says `use allow` for its host. With no path in the record, nothing showed which path that `allow` was for.

**Decision.**
1. **The path rides the payload, not a new record field.** A use's payload already holds self-describing keys (`op` since
   PS-E-06, D-V2-80; a refusal's `refused_by` and `reason`). `url_path` joins `op` there, so the record's canonical shape is
   unchanged. A record written before this verifies as it did; `target` stays the host for every reader; and an older
   build verifies a newer chain, since the payload is hashed as a value whatever its keys.
2. **The path is the request's, read once.** `prim::request_path`: the URL's path as the first check reads it
   (`prim::path_of`), `/` when it names none (the two match alike: no segments). The query and the fragment are never
   carried, because either may hold a token. Both engines and a sandboxed guest's host send it (`custody_path_for`,
   `custody_allows_at`), through a defaulted `Custody::check_at`. Embedded custody records nothing and its `prim` check
   judges the path, so the default's `check` is right there. The plugin wrapper and the daemon client override it. The
   wire's `Check` gains `url_path`, absent from the frame when there is none, so every other `Check` is the frame it was.
3. **The broker judges the path it is told** (`validate`), by the reading the runtime's first check and the egress client
   share (`netscope::judge_all`). A path outside every pattern is `Denial::NetPath` and an ambiguous spelling is refused,
   both DL0904, the code the call already gave, worded as the runtime words them, so a daemon run is told what an embedded
   run is. No new code. This hardens the check and redefines nothing: it is D-V2-138's grant, enforced in one more place,
   so the chain never records `allow` for a path the grant does not reach. A use that names no path (an older client) is
   judged on its host, as before. The path is the client's word, as the host always was; the broker bounds the client's
   own record, and the egress client still judges every hop.
4. **Readers.** `audit query` shows `path=` after `target=`, escaped as the target is; its JSON carries the payload whole.
   The OCSF export maps the path to `http_request.url` (`hostname` and `path`); `dst_endpoint` stays the host. An egress
   refusal's record names no path of its own: it overrides the `use allow` that named it (`overrides_seq`).
5. **Left open:** methods (d), with the first non-GET primitive (RW 2.11). A `send`'s record (P9-04 (d)) names its origin
   only: the broker hands a secret per origin, not per path.

**Evidence.**
- `validate.rs` `a_net_use_naming_its_path_is_judged_on_it_and_recorded_with_it` covers these cases. `/v1/items` and `/v1`
  under `api.example.com/v1/**` are allowed. `/v10/items`, `/v2/items` and `/v1/../v2/items` are refused. Any path on a
  host-only entry is allowed. Each record has `target` the host and `url_path` the path. A use naming no path is judged on
  its host, and an FsWrite's path is ignored.
- `broker_cli.rs` `a_path_scoped_net_grant_is_honored_under_the_daemon_as_embedded`: under the daemon the interpreter's
  three runs, the WASM engine's two and a sandboxed guest's two record `url_path` with their decision. The query
  `?page=/v2` is never recorded, and `audit query` prints `path=`.
- `audit_ocsf_cli.rs`: the path is exported as `http_request.url`; a use naming none claims none.
- `delulu-runtime` `a_network_use_carries_its_request_path_and_never_its_query`,
  `a_plugin_exports_custody_forwards_the_request_path`.
- Mutants M703–M717, each red, the restored tree green. The broker judges no path (M703), records none (M704), records one
  for another op (M705). The client (M706), the interpreter (M707), the WASM host's call (M708) and its gate (M710), the
  sandbox channel (M709) and the daemon (M716) each drop it. The path keeps its query (M711) or names no `/` (M712). The
  plugin wrapper drops it (M713). The export (M714) and `audit query` (M715) omit it. The broker's words differ from the
  call's (M717).

## D-V2-170 — RW 3.4: a WASM `Map` lives in the guest's memory, and the host orders its keys by the interpreter's own `MapKey` — TAKEN (head chef, 2026-10-21, under the owner's delegation)

**Context.** The WASM backend had no `Map`: `delulu build --target wasm` refused any program calling `Map()` (DL1201, "a
call to `Map`", witnessed on the pre-change build). A map's semantics are the interpreter's `BTreeMap<MapKey, Value>`:
iteration ascending by key, `keys()` and `values()` in one order, an inserted key that is already held keeping its place,
values shared (`Rc`). Two ways were open. A host-side map, its values copied across, would break sharing: a record put in a
map and written through its own name afterwards would not change the copy. A guest-side map ordered by guest code would
need a second string order kept beside the interpreter's (the two-matchers lesson, NET-WILDCARD-1).

**Decision.**
1. **The map lives in the guest's memory**: a header like a list's (`[len][cap][data]`), its data 16-byte pairs (the key's
   slot, then the value's), ascending by key. A value is the guest's own slot, so a record or a sum it holds is shared as a
   list's elements are, and `let n = m` names one map.
2. **The host finds and places keys by the interpreter's own `MapKey` order** (`delulu:map`: `new`, `find`, `put`,
   `remove_at`, `keys`, `values`). The guest reads and writes the value slots itself. So the two engines cannot order keys
   two ways: a key built at run time, a non-ASCII one and an upper-case one order as the interpreter orders them.
3. **`str` and `==` of a map** go through the host's value interface (`reify.rs`, descriptor `m<K><V>`), rebuilt as the
   interpreter's `Value::Map`, so `display` and `eq` are the interpreter's own.
4. **The memory is read as untrusted** (a `.dwx` can be written by hand). Every read is bounds-checked. A header whose
   length passes its capacity is refused, and the pairs are read before anything is reserved for them (by inspection: M725,
   reserving first, survives — an operating system grants an untouched reservation, so the test cannot see the order). A
   failure is DL0903
   in words, and the import traps so the guest never runs past it. A map whose pairs are out of order is answered wrongly,
   never unsafely.
5. **`Map()`'s types** come from the checker's type for the call, or from a declared type where one is written
   (`let book: ref Map[Str, ref Acct] = Map()`), since the checker's types alone do not name a record or a sum here.
6. **Left DL1201:** a map inside a list, an option or a record field (`Scalar` holds no map).

**Evidence.** `delulu-wasm/tests/map_parity.rs`, four tests:
- every method on each key kind (`Str`, `Int`, `Bool`) and value kind (`Int`, `Bool`, `Float`), sharing, `str`, `==` and
  `!=`;
- a map grown to 300 keys inserted out of order, half removed, and `Str` keys by their bytes (`Zeta` before `ze`, `éclair`
  after `zeta`, a built `"z" + "eta"` replacing the literal's pair);
- a map of records sharing its values, and a map of sums;
- four hand-written modules refused DL0903 (a header outside the memory, a length past its capacity, pairs past the memory,
  2^32 − 1 pairs claimed).
- Mutants M718–M733: fifteen red, the restored tree green. M725 survives, for the reason in item 4. The red ones: the host's
  order reversed (M718); a new pair written over the next (M719); grown data without the old pairs (M720); a removal that
  never closes its gap (M721); `keys` answering the values (M722); a `Bool` key read inverted (M723); a `Str` key compared by
  its pointer (M724); a length past its capacity trusted (M726); a missing key answered as the first pair (M727); `Map()`
  given no declared type (M728); the descriptor naming a map a list (M729); a value read from the key's slot (M730);
  `contains_key` missing the first pair (M731); `reify` reading values from the keys (M732); the module never carrying
  `delulu:map` (M733).

## D-V2-171 — RW 3.4: the ten `List` methods the WASM engine lacked — eight in the guest, `sort` and `join` host-side — TAKEN (head chef, 2026-10-21, under the owner's delegation)

**Context.** The WASM `List` compiled five of its fifteen methods (`len`, `is_empty`, `get`, `push`, `map`). The other ten
answered DL1201 (witnessed on the pre-change build: "the List method `.pop`", "`.filter`", "`.reverse`").

**Decision.**
1. **Eight in the guest**, each over a snapshot of the list's length and slots, as the interpreter's methods walk one:
   - `pop` takes the last slot into a `Some` and shortens the list.
   - `reverse`, `concat` and `slice` make a new list of the same slots, so a record in two lists is one record. `slice`
     clamps as the interpreter does: below 0 is 0, past the end is the end, `hi < lo` is empty.
   - `contains` compares as `==` does: an `Int`, a `Bool` or a `Float` by its instruction (a `NaN` never equal), a `Str`
     by `__str_eq`. A record or sum element stays DL1201, as it would need the host's `eq`. **Amended 2026-10-22:** it
     compiles through the host's `eq` (`reify.rs`), with the element's descriptor, as `==` of two of them does. A module
     whose list `contains` takes a record or a sum carries the value interface. `contains_on_records_and_sums_compares_as_
     equality_does` covers it: a record equal by its contents, another record. M753 (the arm missing, the pre-change
     refusal), M754 (the interface never carried) and M755 (compared by address) are red.
   - `filter`, `find` and `fold` inline their function value as `map` does. `find` stops at its first match. `fold`
     computes its initial value after the snapshot, as the interpreter does.
2. **Two host-side** (`delulu:list.sort`, `delulu:list.join`), each run over the guest's slots:
   - `sort` is stable, by the interpreter's order (an `Int` or a `Bool` by its number, a `Str` by its bytes), and moves
     the slots whole. Its strings are compared in the guest's memory where they lie, never copied.
   - `join` checks every length and bounds the total before allocating, then copies each piece straight into one string
     cell. Built in the guest by `+`, it would copy the growing string once per piece, and the bump heap never frees.
3. **A module carries the list helpers** for a method on any receiver the checker typed a `List`, not only for `split`,
   `push` and `map` by name (`contains` and `slice` are `Str` methods too). A list's `contains` brings string equality as
   `==` does.
4. **The host reads the memory as untrusted**, as for maps (D-V2-170 item 4): a length past its capacity, slots outside
   the memory and a string outside it are refused DL0903 in words, then a trap.
5. **A walk that runs code is not moved by `pop`.** `pop` had waited for exactly this (`list_parity.rs` said so): a `pop`
   and then a `push` reuse the popped slot. A walk of the list's own slots then reads the pushed value where the
   interpreter, walking a copy, reads the popped one. The existing test caught it when `pop` first compiled here. In a
   module that pops a list anywhere (`Imports::pops_lists`), every walk that runs code per element (`for`, `map`,
   `filter`, `find`, `fold`) walks a private copy of the slots taken when it begins. A module that never pops pays
   nothing, and the walks without code (`reverse`, `concat`, `slice`, `contains`, `join`, `sort`) cannot be interleaved
   with a `pop`. Witnessed with the copy off, one program popping and pushing inside each walk:

   | Walk | Interpreter | WASM, no copy |
   |---|---|---|
   | `for` | `1234` | `1239` |
   | `map` | `[2, 4, 6, 8]` | `[2, 4, 6, 100]` |
   | `filter` | `[3, 50]` | `[3, 70]` |
   | `fold` | `76` | `1006` |
   | `find` | `Some(1000)` | `None` |

**Evidence.** `delulu-wasm/tests/list_methods_parity.rs`, six tests:
- the seven first-order methods on `Int`, `Str`, `Bool` and `Float` lists, every clamp included;
- `filter`, `find` (its printing predicate examines four elements and stops) and `fold`, with `Int`, `Str` accumulators;
  a predicate that pushes onto the list it filters walks the list as it began (6 kept, 12 after);
- records shared through `reverse`, `concat`, `slice`, `filter` and `pop`;
- five hand-written modules refused DL0903, a string cell whose length runs past the memory among them;
- a module whose only lists are a map's `keys()` and `values()`;
- `a_walk_is_not_moved_by_a_pop_and_a_push_in_its_own_code` (item 5), red under M751 (no copy) and M752 (the module never
  read for `pop`).
Each program also built to a `.dwx` and run by hand with the interpreter's output. `list_parity.rs`'s list of what stays
outside drops `pop`, and `conformance_parity.rs`'s drops `fold`, each replaced by a list of lists.
- Mutants M734–M752, each red, the restored tree green. M748 (a string cell's end unchecked) first survived. The test's
  string cell lay outside the memory, so its length read failed first. A cell inside the memory whose length runs past
  the end was added; under the mutant, `sort` slices past the memory and panics the host. The others:
  - `pop` (M734), `reverse` (M735), `concat` (M736), `slice`'s three clamps (M737–M739) and `contains` (M740) each lose
    their meaning.
  - `filter` keeps everything (M741), `find` walks past its match (M742), and `fold` forgets each step (M743).
  - `sort` orders strings by address (M744) and `Bool`s inverted (M745). `join` writes no separators (M746).
  - A length past its capacity is trusted (M747).
  - The module is read for three names only (M749), and a list's `contains` brings no string equality (M750).

## D-V2-172 — A `match` must cover every case at every depth; the WASM engine tests inside each constructor — TAKEN (head chef, 2026-10-22, under the owner's delegation)

**Found:** MATCH-NEST-1, MATCH-EXHAUST-1 and WASM-IMPORTS-1 (RW 3.22, 2.20, 3.23; `V2_LOG.md`, 2026-10-22). The checker
proved only that the arms named every constructor at the top. The interpreter's DL0907 text, the reference
(`ref.rule.errors.match-is-exhaustive`, "covered") and `GETTING_STARTED.md` all promise more. The WASM backend relied on
the same top level and ignored what a pattern held inside a constructor.

**Decided:**
1. **DL0407 covers every depth.** A match is refused when some value of the scrutinee's type matches no arm. That
   includes a constructor or a literal inside a constructor, and literal arms on an `Int`, `Str` or `Bool` with no
   catch-all. The message names the first such value (`missing Some(Blue)`; `_` stands for any value of a type with no
   finite set of constructors). It is the existing code, so no new DL code. A program refused now would have faulted
   at run time on that value; none in the repository is.
2. **The search is bounded, and a match it cannot finish is refused, not passed.** The bound is 10,000,000 cells read
   per match (`EXHAUSTIVE_BUDGET`). The refusal says the checker could not prove the match within its limit; one `_` arm
   satisfies it. Passing an unproved match would return the runtime to a fault the checker's text says cannot happen.
   The first version did exactly that, on a match that was not exhaustive.
3. **The WASM engine tests inside constructors as the interpreter does.** Arms are tried in order, each pattern tested
   and bound. A module that holds such a match imports `delulu:match.no_arm`, which records the interpreter's DL0907
   and is unreachable for a checked program. A `.dwx` built with it needs a host that provides `delulu:match`, so an
   older host refuses it at instantiation, never running it wrongly. Every other module's bytes are unchanged.
4. **The import section is written whenever it holds an import**, not when a hand-kept list of interfaces says so.

**Evidence:**
- Red on `30885b1`, green after: `match_parity.rs` (4); `map_parity.rs`
  `a_module_whose_only_imports_are_a_maps_loads`; the three `delulu-check` tests `a_match_*`; the two DL0407
  conformance reject cases.
- Mutants M756–M776 red.
- 252 repository programs re-checked with identical output.

**Residual:** the bound is per match. A file built to hold many matches that each spend it costs roughly a third of a
second per match in a debug build, the same denial-of-service class as RW 2.3. A literal arm on a `Float` cannot be
written (the parser has no `Float` pattern).

## D-V2-173 — RW 3.4: a slot holds a pointer to any list, map, option or result, so they nest on the WASM engine — TAKEN (head chef, 2026-10-22, under the owner's delegation)

**Decided:**
1. A `List`, `Map`, `Option` or `Result` may sit wherever a value goes on the WASM engine: a list's element, a map's
   value, a variant's payload, a record's field. Its 8-byte slot holds a pointer to its cell, as it already held a
   record's.
2. Lists, maps and records are shared as the interpreter shares them. Options and results stay immutable cells.
3. The codegen's `Scalar` names such a type through `Scalar::Boxed(&'static Ty)`, so that `Scalar` and `Ty` stay `Copy`
   (every function of the backend passes them by value). Each distinct type is interned once for the process (`boxed`):
   a leak of a few bytes per type any compile has met, and none per value.
4. What stays outside, refused with DL1201: a list or a record field holding a function or a capability. Neither has a
   cell the host's value interface can rebuild.

**Evidence:**
- `nested_parity.rs` (3): lists of lists, options, maps and records nested, shared, compared, matched, and faulted;
  refused by the old codegen.
- The five pins that had listed these shapes as refused now list what stays outside. None recorded a hazard (read
  first, per POP-WALK-1).
- The whole `delulu-wasm` suite passes.

## D-V2-174 — RW 3.4: a WASM program holds a secret as a handle the host keeps; a request sends it from the host — TAKEN (head chef, 2026-10-22, under the owner's delegation)

**The question:** Phase 3n refused a WASM program that so much as minted a secret (DL1205): minting, it said, would place
the secret's bytes in the guest's memory. `STAGE3_SPECIFICATION.md` §4.4, the normative design, says otherwise: "`Secret[T]`
values in WASM are externrefs; the bytes live only in the host secret table", and DL1205 is for "guest closures over
secret *contents*". Phase 3n's refusal of the mint was a stricter stand-in for that table, never built. It kept every
program that holds a secret off the WASM engine, `examples/demo.delulu` among them, which mints one and does nothing else.

**Decided:**
1. `root.secret(name)` compiles. The host mints the secret by the interpreter's own rule (`call_root_method`) against the
   run's secret grants, keeps it in its capability table (`CapKind::Secret`), and gives the guest the entry's index. Under
   embedded custody the table holds the value; under `--broker daemon` it holds the broker's handle, as the interpreter's
   root does (Phase 5g). An index in place of §4.4's externref: the host's other capabilities are indices already.
2. `Cap[Http].get_with_secret(url, headers, header, secret)` compiles. The host performs it in the interpreter's order:
   the trace record at dispatch with detail `«opaque»`, the custody on the URL's host, then the secret judged and, if the
   broker holds it, its bytes asked for the one origin. That judgement is one function for both engines,
   `interp::broker_secret_send`, taken out of the interpreter's `send_broker_secret`. Then comes the interpreter's own
   `get_with_secret` and its egress client.
3. `Secret.verify(other)` compiles, compared by `interp::secret_verify` (taken out of the interpreter's `verify_secret`):
   two secrets the process holds are compared there, two the broker holds are compared by the broker. It is a
   `Declassify` effect that the interpreter performs without a custody op, so the host asks a `.dwx`'s declared-effects
   ceiling for it directly (ARTIFACT-CLAIM-1, `declared_allows`).
4. What would hand a secret's bytes to the program, or run the program's code on them, stays DL1205: `Secret.expose`,
   `root.declassify()` (whose one use is `expose`, the checker's R-2; refused where it begins, because it would
   otherwise reach only a DL1201) and `Secret.map(f)`. §4.4's host-side pure-op whitelist for `map` is not built here.
   A secret in a list, a record or an option is DL1201, as a capability there is.
5. DL1205 keeps its meaning: secret contents cannot enter the WASM guest. What changed is that holding a secret no longer
   needs its contents there. Its `explain` text now says so.

**Evidence:**
- `host::secret_handle_tests::a_minted_and_sent_secret_never_enters_the_guests_memory`. After a run that minted a secret
  and sent it past its audience to the egress client, no byte of it is in the guest's memory, and the module was built
  without it. The control: the scan finds the program's own strings. Phase 3n called §9.8's memory scan vacuous "when the
  bytes never enter"; it now reads a memory for which the host has handled the secret. (The request ends at the egress
  client's transport: `no-client` when the crate is tested alone, which builds the runtime without `net`, and `connect`
  in a workspace build, where `net` is on and nothing listens on port 9. The CLI tests below see it leave.)
- The conformance witness for `ref.diag.DL1205` named the replaced pin until PR #48's first run (`39825874508`) refused
  it as dangling. It now names `a_secret_compiles_as_a_handle_and_what_would_read_it_stays_dl1205`, an entrenched edit
  recorded in `ENTRENCHED_CHANGE_RECORD.md` (2026-10-22).
- `egress_cli.rs` `a_wasm_program_sends_its_secret_from_the_host_as_the_interpreter_does` runs the same program on the
  interpreter, on `--engine wasm` and as a built `.dwx`. In the audience, the request leaves as TLS and the report's hop
  names the secret. Outside it, nothing connects and the operator is told `audience`. The three traces are identical.
  No byte of the secret is in the artifact, trace, report or output.
- `broker_cli.rs` `a_wasm_runs_broker_held_secret_is_sent_through_the_broker_as_the_interpreters_is`. Under the daemon,
  the WASM run's audit chain matches the interpreter's record for record, with one `send allow` for the pair.
- `wasm_cli.rs`:
  - `demo_runs_on_wasm_as_on_the_interpreter_and_its_secret_stays_on_the_host`: `demo` prints what the interpreter
    prints, from the source and from its `.dwx`. Ungranted, both engines refuse in the interpreter's words.
  - `a_secret_is_verified_on_wasm_as_on_the_interpreter_and_only_where_the_artifact_declares_it`: the same answers and
    the same trace from all three. A `.dwx` whose manifest lost `Declassify` is DL1202 at its first `verify`, before it
    prints.
- `lib.rs` `a_secret_compiles_as_a_handle_and_what_would_read_it_stays_dl1205`, and Phase 3n's
  `secret_handling_program_is_refused_dl1205` unchanged and green.
  - The pin it replaces, `minting_a_secret_alone_is_refused_dl1205`, was read first (POP-WALK-1). Its reason was that the
    bytes would be placed in guest memory, and the memory test checks exactly that.
  - That test went red on the first version of this change, at `root.declassify()`, which is why `declassify` is refused
    by name.
- **A regression found by re-measuring, not by a test run.** Chapter five of the guide holds `password_matches(held:
  Secret[Str], …)`, which `main` never calls. While a `Secret[Str]` parameter had no WASM type, the backend skipped that
  function. Once the parameter compiled, the chapter stopped at its `verify` (DL1201), and the corpus counter fell to
  12. `verify` is item 3. `wasm_cli` `the_guide_chapters_the_wasm_engine_runs_match_the_interpreter` covers the chapter.
- Mutants, each red at the assertion named:
  - the host writes the secret's bytes into guest memory at the mint (the memory scan);
  - the send's trace detail is the URL (`«opaque»`);
  - the WASM run loses the audiences (nothing connected);
  - under the daemon it is given values, not handles (the broker was asked once);
  - the custody check before the send is skipped (the chain is the interpreter's);
  - `declassify` is no longer refused (DL1205);
  - `verify` skips the ceiling (DL1202);
  - `verify`'s trace detail is dropped (the trace is the interpreter's).
- The corpus re-measured file by file: **15 of 17** build as programs (13 files, plus the two packages, whose entry files
  alone stay DL0301). `numpy_mean` and the plugin host remain, DL1201.

## D-V2-175 — RW 5.1: Theorems 1–3 and No-forgery machine-checked for the monomorphic fragment; Delulu Core v0.4 states `verify` and `narrow` — TAKEN (head chef, 2026-10-22, under the owner's delegation)

**The question:** RW 5.1, "the headline evidence gap": `DELULU_CORE.md` §1–§7 had no mechanized proof beyond the
higher-order fragment (P17-9), with no capabilities, store, secrets, faults, Progress or Preservation.

**Decided:**
1. `docs/design/models/lean/DeluluCoreSafety.lean` (903 lines, core Lean 4.32.2, no Lake, no mathlib) states §1–§4 for
   the **monomorphic, closed-row fragment** and proves:
   - `progress`, Theorem 1 (progress-or-fault);
   - `preservation`, Theorem 2 (types and rows);
   - `effect_soundness`, Theorem 3, for every finite prefix of a run;
   - `safety`, every reachable configuration typed and not stuck;
   - `attenuation_monotone`, every token a run's store holds descending from a root token: its kind, a scope `⊑` its
     scope (audit R-7);
   - `no_forgery`, the Corollary: every reachable token in the store and so descended.

   The fragment has functions carrying rows, `let`, products, sums, capability tokens, `op` with `E-Op` and `E-Refuse`,
   `narrow` (a fresh token at the end of the store, or a fault), `hop`, `wrap`, `verify` and `expose`. Its evaluation is
   call-by-value, by frames, with faults propagating (`E-Fault`). Scopes are abstract, with a preorder and the one law
   `narrow(s, v) ⊑ s`.
2. What it assumes, stated in the file and in `DELULU_CORE.md` §9:
   - `SigOk`: each primitive's result on a permitted, well-typed argument is well typed with an empty row.
   - `hop` is its simplest instance, applying the callback once.
   - `verify`'s label is a parameter, so v0.3's rule and v0.4's are both proved.

   What it leaves out: type variables and open rows (§5).
3. CI's `formal` job checks the file with the pinned Lean, and fails on any `sorry` in either file's output. Lean reports
   a `sorry` only as a warning: a planted `theorem planted : False := by sorry` checked "clean" before this gate. Three
   mutants must each fail to check: `T-Op` without its label, `T-HOp` without the callback's latent row, and the law
   `narrow(s, v) ⊑ s` weakened to `s ⊑ s`, its arity kept (a first version changed the arity and failed for a syntax
   error, not the proof).
4. `DELULU_CORE.md` becomes v0.4, an entrenched edit recorded in `ENTRENCHED_CHANGE_RECORD.md`:
   - the honesty clause says what is now machine-checked;
   - `T-Verify` carries `Declassify`, as the checker has typed it since P17-IF1;
   - `E-Expose` and `E-Verify` are written in §4, and attenuation is a term (`narrow`, `T-Narrow`, `E-Narrow`,
     `E-NarrowRefuse`), which §4 had relied on without stating;
   - Fact B and Theorem 3's capability-witness clause name `verify`'s label as the one no capability witnesses (RW 2.6);
   - §9 gains a box for the development.

**Evidence:**
- `lean DeluluCoreSafety.lean` checks. `#print axioms` lists `propext`, `Classical.choice` and `Quot.sound` for the
  theorems, and no `sorryAx`. (`DeluluCore.lean` rests on none, and is unchanged.)
- The three mutants fail to check. The planted `sorry` is caught by the gate. The CI steps were simulated locally with the
  same `sed` patterns.
- The implementation's `verify`, on the current binary: a pure function calling it is `DL0501` ("performs effect
  `Declassify` not declared in its row"), and `--trace-effects` records it as `Declassify`.
- `delulu-survey impact doc:docs/design/DELULU_CORE.md`: 0 nodes reached.
- What mechanizing found: §3's `T-Verify` stale against P17-IF1, and `E-Expose` never written. Both are stated now, and
  stating them shows Theorem 3's capability-witness clause was never true of `verify` (RW 2.6, unchanged and now named in
  the calculus).

## D-V2-176 — RW 6.6: `deploy plan` compares every request's scope, decided by `authority --within`; a package with dependencies plans whole — TAKEN (head chef, 2026-10-22, under the owner's delegation)

**The question:** `delulu deploy plan` compared one dimension of the authority answer spec §9.2 asks for, the effect row.
An environment profile could name `fs.read`, `fs.write`, `net`, `secrets`, `foreign.c` and `foreign.python`, and none of
them bounded a service. A plan was approved whatever paths, hosts or secrets its services asked for, if their effects fit.
And reading the code found DEPLOY-PKG-1: `package_authority_value`, shared by `deploy plan`, `publish` and `add`, kept the
single-package loader after C51 moved `authority` off it. So every package with a dependency was refused as one that "does
not check cleanly". Witnessed on the four-package `station` (`deploy plan`) and on a library depending on another (`add`).

**Decided:**
1. A service is checked as `delulu authority <dir>` checks it (`cli::checked_package`): the workspace loader when the
   directory has a `delulu.toml`. `deploy plan`, `publish` and `add` all go through it.
2. The profile is a boundary in the `--grant` grammar:
   - `fs.read`, `fs.write`, `net` and `foreign.python` lines go through `Grants::add`, the parser `run --grant` uses, so a
     special-use host under `net` is refused as no boundary (D-NE-28) with exit 2;
   - secrets and `foreign.c` libraries are matched by name;
   - `console`, `clock`, `rand` and `declassify` are granted where the profile's effects grant `Write`, `Clock`, `Rand`
     and `Declassify`.

   (`Grants::accept_manifest` was not used: it takes secrets only when they are set in the environment, and no
   `foreign.c`.)
3. Each service's requirements are decided by `within::verdict`, the verdict `authority --within` gives: the runtime's own
   mint for each path, host and secret. A service fits only if its effects are within the ceiling and its verdict is
   *within*. Exceeds, unsupported and inconclusive (a scope the source computes) all refuse the whole plan with DL1909,
   naming each request with its call (`file:line`) and the runtime's words. No new code.
4. The approval carries its scope (C45): `compared` names effects, capability scopes and foreign holes, and
   `profile_cannot_grant` names `net.special`, plugin roots and devices. The human verdict says "authority ceiling".
5. A breaking change for a profile that named effects only: a service that reads files, makes requests or holds secrets
   now needs the profile to name them. 2.0 is unreleased, so it goes under 2.0.0's Breaking.

**Evidence:**
- `deploy_cli.rs`:
  - `a_package_with_dependencies_plans_as_authority_checks_it` (`station`, approved);
  - `a_request_outside_the_profiles_scopes_is_refused_at_its_call` (`fs.read=./data` at `src/station.delulu:16`, exceeds);
  - `a_scope_the_source_computes_is_refused_as_inconclusive`;
  - `a_secret_is_bounded_by_name_and_a_special_use_host_is_no_boundary`.
- `add_path_cli.rs` `a_dependency_with_dependencies_of_its_own_is_computed_whole`. The old loader's exact words,
  "does not check clean, so its authority cannot be computed", came back under the mutant.
- Four mutants red:
  - the single-package loader (two `deploy_cli` tests, and the `add` test);
  - the verdict ignored (three);
  - the profile's `fs.read` dropped (two);
  - its secrets dropped (one).
- `deploy_plan_adversarial.rs`: five approving fixtures name the host their Net service calls (`net = ["example.com"]`).
  The scope contract test now pins `compared` (three) and `profile_cannot_grant` (three). The refusal tests are unchanged.

## The owner's word on the repository, 2026-10-17 (night) — it outranks D-V2-153 §3

**The owner:** *"Only push to and use test repo until I tell u to change. Do not change repo, unless told by me. Always push only to my repo"*. **So:** development stays on `delululang-test`; D-V2-153's move to `DeluluLang` `main` at the
launch, the snapshot push, the tag `v2.0.0`, the switch of `origin`, the routine's repository and the archive's
visibility all wait for his word. D-V2-153's controls on `DeluluLang` (switched on and read back) stand; nothing has been
pushed there. `launch/2.0.0` on `delululang-test` holds the prepared release and its gate (`CHECKLIST-2.0.md`); a
session keeps it current by merging `master` into it, never the reverse.

## Owner decisions carried from V1 — where each stands (rewritten 2026-10-17)

Since the commission of 2026-10-17 none of these waits for the owner; each is the head chef's, with its record.
D-NE-3 (snapshot regeneration is a reviewed act) stands as a process rule. D-NE-6: D-V2-38. D-NE-7, publication: the
launch, from the laptop (D-V2-146). D-NE-8's installer posture: its workflow half D-V2-42; the rest at the launch.
D-NE-25 (`Secret.map` under strong profiles): its proposal stands until built. D-NE-27, the GPL microVM image: D-V2-144,
never distributed. The Constitution §5.15 wording: D-V2-141. rustfmt: D-V2-145. The code of conduct: D-V2-151. RW 4.19:
D-V2-147. RW 4.5: D-V2-148. P8-04 (c): D-V2-150. The version: D-V2-149. The four pre-public-repository items: at the launch.

*(The list as it stood before 2026-10-17:)*
D-NE-3 (snapshot regeneration is a reviewed act — the diff is shown in each phase's log),
D-NE-6 (decided under delegation as D-V2-38), D-NE-7 (the workflow is built and publishes nothing without it, D-V2-42), D-NE-8's installer posture (its workflow half taken in D-V2-42), D-NE-25, D-NE-27; the Constitution §5.15 wording (RW 7.10a); rustfmt and a
code of conduct; the four pre-public-repository items. Each is asked at the start of the phase that
needs it (`V2_MASTER_PLAN.md` §7).

**Corrected 2026-09-25 — five entries this list carried were already decided.** D-NE-17 was built in
P1-F on the owner's word; D-NE-24, D-NE-26 and **D-NE-31** were ruled by the owner in D-V2-25
(2026-09-18) — D-NE-31's defaults are **1 GiB of memory and 5 minutes of CPU, never unlimited, the
operator may change them**; D-NE-33 was superseded by D-V2-25 and then set by D-V2-26. The list was
written before those rulings and never walked back, so PS-B's own entries went on calling D-NE-31
"the owner's" while the answer sat 280 lines above them. Found while checking the earlier phases.

## D-V2-177 — RW 3.4: a WASM slot holds a capability or a secret as its handle — TAKEN (head chef, 2026-10-23, under the owner's delegation)

**The question:** on the WASM engine a capability or a secret could sit in a variable but not in a list, an option, a
record field or a map's value; each was refused with DL1201 ("a list of Cap") and the program ran on the interpreter.

**Decided:** a slot holds a handle as a variable does: its `i32` index into the host's table (`Scalar::Handle`, stored
and loaded as one word). The handles are the ones a variable may hold: `Cap[Console]`, `Clock`, `Rand`, `FsRead`,
`FsWrite`, `Http`, `ForeignLoad` and `Secret[Str]`. Not `Root`, which enters only at `main`. `checked_ty` maps the
same types, so a `Map()` or a `None` whose type only the checker names compiles too.

**Why it is safe:** the guest never reads through a handle, and could already pass any `i32` to any import. The host
checks each index against its table and the kind the import needs (`CapKind`, D-V2-174). The checker gives none of these
types a `str` or an `==` (DL0604, DL0605), so the host never rebuilds one from memory (`type_desc` refuses it).
`expose`, `root.declassify()` and `Secret.map` stay DL1205, however the secret was reached.

**Evidence:** `crates/delulu-wasm/tests/handle_parity.rs`:
- `a_capability_in_a_list_an_option_a_record_and_a_map_acts_as_on_the_interpreter`: a list walked and indexed, an
  option, a record field, an annotated map and one the checker typed, each printing through the handle it holds, held to
  the interpreter's output;
- `a_secret_in_a_list_or_an_option_is_a_handle_and_exposing_it_stays_dl1205`.

Both fail on `master`'s code (DL1201, "a list of Cap"). The `checked_ty` arms dropped: both fail (Console, Secret).
`list_parity.rs` and `record_parity.rs` had pinned a list, and a record field, of `Cap[Console]` as refused: outside the
fragment, not unsafe. Those cases moved here. The pins now hold the root, the handle no slot holds: a list of it, a
record field of it. Letting `Root` into a slot turns both red.

## D-V2-178 — RW 2.10: a record's fields are found at their declared place, by a map checked against the name — TAKEN (head chef, 2026-10-23, under the owner's delegation)

**The question:** the interpreter found a record's field by scanning its fields (finding C55, published as a limit in
2026-07), and since RECORD-ORDER-1 (2026-10-20) it put a literal's fields in declared order with a sort whose every
comparison scanned the declaration for both fields' places: quadratic in the width. RW 2.10 named the fix as static field
indices through the DIR.

**Decided:** the interpreter holds each record type's fields as a map from name to declared place (`Home.records`,
built once per module). A literal looks up each field's place once and is sorted by it (`sort_by_cached_key`, the same
order as before). A field read or write in a record wider than 16 fields (`SCAN_WIDTH`) asks the map for the place and
uses it only when the field held there has the name asked for; otherwise, and for a record of 16 fields or fewer, it
scans as before.

**Why not the DIR:** a static index would change the AST and the DIR's serialized form (byte-reproducible, read by
plugins), and the interpreter would have to trust that every record value it reads was laid out by this module's
declaration. A record can come from a plugin's module, an actor's message or another module's closure; checking the name
at the place makes a wrong place cost a scan, never a wrong field. Below 17 fields the scan stays: D56 measured it at
or below the interpreter's own cost there, and two hash lookups are not cheaper.

**Evidence** (release, the laptop, best of three; `measurements/scale/RECORD.md`): reading the last of 1,600 fields
200,000 times 0.473 s → 0.128 s, a write 0.468 s → 0.110 s, 250 literals of 1,600 fields written out of order 9.79 s →
0.078 s; at 4 fields every figure unchanged. `crates/delulu-runtime/tests/record_width.rs`, four tests: two timing
tests judged against a control measured beside them, both red on the old interpreter (3,662x and 21.4x against bounds of
100 and 3; 17x and 1.00 now), and two that hold the layout's answers. Mutants red: the name check removed, a scan at every
width, no ordering of a literal. The checker's `C48` residual scan was measured rather than changed: 6,400 fields check
in 0.18 s, linear.

## D-V2-179 — RW 2.3: the checker follows a shared type once, and refuses one too large to write out (DL0211) — TAKEN (head chef, 2026-10-23, under the owner's delegation)

**The question:** `let a1 = Pair { l: a0, r: a0 }` … `aN` checked in time doubling per line (22 lines 5.8 s, release;
30 lines about 25 minutes by that rate). RW 2.3 had deferred a bound to an RFC because a bound is language-visible: it
refuses programs that check today. The type is N small types through its variables, 2^(N+1) - 1 parts written out.
Three kinds of work treated it as its tree: walks that followed bindings (the occurs check, unification, `==`'s
equality walk, which wrote the type out again at every part it visited), and writing it out (`apply_type`: the DIR's
side table of every node's type, a field read, a `match`, a binary operator). The first kind can be made to follow each
binding once. The second cannot: the side table is a tree by its format, and the DIR serializes it.

**Decided:**
1. The occurs check follows each bound variable once (a set of variables explored), and unification unifies each pair
   of variables once per call (a set of pairs met). Neither changes what is decided. The variable being bound is
   unbound, so it is never met as a bound variable on the way. A pair met again was unified (bindings only grow, so it
   still is) or failed, and a failure ends the call with `?`. A pair cannot meet itself inside its own binding, which
   the occurs check rules out. The equality walk (`opaque_for`) is given the type written out once and walks it as it
   stands.
2. `apply_type` writes a type out within 100,000 parts (`TYPE_PARTS_LIMIT`). Types past 1,000 parts
   (`TYPE_PARTS_FREE`) also count toward 2,000,000 for the module (`TYPE_PARTS_TOTAL`). A type past either is
   returned with only its top resolved, and the context is marked. From then on a type is written out only within
   1,000 parts, so the rest of the check costs what an ordinary program's does. The checker refuses a marked module
   with **DL0211** at the expression being checked when it was marked, or at the node being settled. The settle runs in
   the table's order; when it refuses, it is run again in node order, counting parts without writing them, from the
   total reached before it, so every run names the same node. It does not run the reference-capability pass over types
   settled only to their top.
   The refusal is fail-closed: a module whose types were cut is never accepted.
3. DL0211 ("type nests too deeply", the parser's code for a written type nested past 128 levels) is the code, not a
   new one. Its explanation already covers a type the checker cannot finish with; it now names an inferred one too.

**Why these numbers:** an ordinary program's types are tens of parts. A type of 65,535 parts (15 lines of pairs) still
checks. The total bounds many references to one type just under the limit: 3,000 references to a 65,535-part type took
3.4 s with a total of 20,000,000, and 0.45 s with the 2,000,000 chosen. After the first refusal the work grows with the
program's length: 20,000 references to a 32,767-part type, 1.4 s.

**What it costs an ordinary program** (release, best of five, alternating with `master` `118cf51`): 3,000 functions using
generic records, closures, `map`, `fold`, `Map`, `Option` and `match` check in 0.391 s against 0.379 s; 10,000 in 1.141 s
against 1.105 s, about 3%. Measured by elimination: the sets of the occurs check and unification cost nothing that shows;
collecting parts through `Option` cost about 2% (1.167 s against 1.145 s), so they go into vectors sized up front;
sorting the side table for a deterministic name cost about 0.4% (1.145 s against 1.141 s), and the settle now sorts only
when it refuses. What is left is the budgeted walk.

**Evidence:** `crates/delulu-check/tests/type_size.rs`, each program checked on a thread within a minute:
- depths 16, 30 and 60 are refused once, at `a16`, and depth 15 is clean;
- a field read at depth 40 is refused;
- two chains compared with `==` check clean at depth 15 and are refused at 40, and with `1` against `"x"` at their
  roots they are still a mismatch (DL0401) at 12 and at 40;
- 200 references to a 32,767-part type are refused by the total, and 40 to an 8,191-part type are clean.

Mutants: no limit and no total, 4 of 4 red; no pair cache, red; an occurs check that re-follows, 3 of 4 red; no DL0211,
4 of 4 red; the equality walk writing out each argument again, red; the replay naming no node, red. One change I tried survived: the equality walk
visits each argument again through its record's fields, but `visiting` cuts that off one level down. It was reverted,
and the claim that it cost a second was struck. Release: every refused depth checks in about 0.1 s; `a22 == b22` went
from 261 s (with the bound but without the walk fixes) to 0.1 s.

## D-V2-180 — RW 2.11: a sent secret is asked for, and decided, with its audience — TAKEN (head chef, 2026-10-23, under the owner's delegation)

**The question:** P9-04 built secret sends: `http.get_with_secret` goes only to an origin the secret's grant names in
its audience (`secret:NAME@HOST[:PORT]=VALUE`). It left one step, proposing the audience in `--grants`' lines. Building
that step showed two gaps.
- `--grants` asked for `secret:API_TOKEN=VALUE`. A run granted that way holds the secret and refuses every send of it,
  so the tool's own advice gave a program that could not do what it was written to do.
- `--within`, and `deploy plan`, which uses its verdict (D-V2-176), answered *within* for a boundary granting the secret
  with no audience, or with one naming another origin. A run refuses those sends, so the request was outside the
  boundary and the verdict did not judge it. The runtime stayed fail-closed and nothing was sent.

**Decided:**
- A secret's requirement carries the origins of its sends as the source shows them (`within::secret_sends`, the walk
  behind `secret_destinations`). Its line names them: `secret:NAME@HOST:PORT[,…]=VALUE`, with `HOST` for a computed URL.
  A send of a secret the source does not name (a parameter, a field) is its own requirement:
  `secret:NAME@HOST:PORT=VALUE`.
- `--within` decides the mint by name, as before, then each send against the boundary's audience for its secret, compared
  as the egress client compares (`parse_audience`, host and port):
  - *exceeds* when the boundary grants the secret with no audience, or one that does not name the origin;
  - *inconclusive* for a computed URL under an audience;
  - for a secret the source does not name: *within* only when every secret the boundary grants may reach the origin,
    *exceeds* when none may, *inconclusive* between.
- A `deploy plan` profile's `secrets` entry may name its audience as a grant does (`NAME@HOST[:PORT]`). The run's own
  parser reads it, so a malformed audience is refused as no boundary.

**Language-visible:** `authority --grants` prints a different line for a program that sends a secret. `--within` and
`deploy plan` now refuse boundaries they used to approve: ones under which the program's sends would be refused when it
runs.

**Evidence:**
- `within_cli` `a_sent_secret_is_asked_for_and_decided_with_its_audience` covers three shapes: a literal URL (five
  boundaries, the port included), a computed URL, and a secret passed to a helper (within, inconclusive, exceeds).
- `deploy_cli` `a_secret_a_service_sends_is_bounded_by_its_audience`: name only refused, named audience approved,
  malformed audience no boundary.
- Both fail on `master`'s code. Mutants red: the line without the audience, the sends not decided, the port ignored, an
  unnamed secret's send always within.

## D-V2-181 — `--within` and `deploy plan` decide each request by its path — TAKEN (head chef, 2026-10-23, under the owner's delegation)

**The question:** P9-05 (a) judges a request at its call against its capability's `net` entries, the path too where an
entry names a pattern. `--within` decided only the mint. A program minting `example.com/api/**` and requesting
`https://example.com/admin` was *within* a boundary of `net=example.com/api/**`, and `deploy plan` approved it, though the
run refuses that request whatever the program minted. The same skipped dimension as D-V2-180, found by asking the
question again for paths.

**Decided:** each request the checker types (`Cap[Http]` receiver; `request_destinations`' walk, `within::request_sites`)
is a requirement, `Ask::Request { url }`. It is decided against the boundary's `net` entries exactly as the call decides
it: `prim::host_of`, `prim::path_of`, `netscope::judge_all`.
- *exceeds* when the URL is one the egress client will not read (plain `http://`, userinfo, a malformed host), in the
  client's own words; when the path is outside every entry for the host; when the host is in none; or when the spelling
  is ambiguous;
- for a computed URL, *within* when the boundary scopes no host by path (the mint decides the host), else *inconclusive*.

A finding names the narrowest line that takes the request (`net=example.com/admin`) at the request's call. Requests are
set only where a verdict is made (`authority --within` for a file and for a package, each module by its own types, and
`deploy plan`'s `checked_package`). `--grants` is unchanged: the line to type for a request is its mint's.

**Evidence:** `within_cli` `a_request_is_decided_by_its_path_against_the_boundary` (a file: outside, inside, computed, a
whole host; a package whose request is in another module) and `deploy_cli`
`a_request_outside_the_profiles_path_scope_is_refused`. On the previous code the first failed and the second's plan was
approved. Mutants red: a request outside the path *within*, a package's requests not wired, a computed URL always
*within*, the unreadable-URL arm removed. A `typed` flag kept requests out of `--grants`' lines; its mutant survived,
because `--grants`' walk has no requests, so it was taken out.

**Found on the way:** `deploy_plan_adversarial`'s fixture service requested `http://example.com`. The run refuses that
request at its call ("only `https://` is fetched"), and the plan had been approved. The first version of this decision
refused it with a misleading reason ("names `http`": `prim::host_of` strips only `https://`). The fixture exists to
perform `Net`, and now requests `https://example.com`; its five tests assert what they did.


## D-V2-182 — P9-05 (d), first slice: a `net` entry names its methods — `HOST[:METHODS][/PATTERN]`, one spelling for each set, none named = GET alone; ordered by subset and met by intersection everywhere the entry is read — TAKEN (head chef, 2026-10-23, under the owner's delegation)

**Context.** RW 2.12: a `net` grant could not limit the methods a program may use on a host. That was moot while
`http` could only GET, and it stops being moot with the first primitive that makes another request: added to today's
grammar, such a primitive would let every `net=HOST` grant ever written POST, PUT or DELETE there. That would widen
what existing grants mean, which the semver-authority law forbids. So the dimension comes first, with no primitive
yet that needs it (D-V2-55, `V2_OPENSHELL_STUDY.md` §4.12).

**Decision.**
1. **The grammar.** `net=HOST[:METHODS][/PATTERN]`, METHODS names from `GET`, `HEAD`, `POST`, `PUT`, `PATCH`,
   `DELETE`, separated by `,`. Each set has one spelling: each name once, in that order, and never `GET` alone. An
   entry naming no methods allows **GET alone**, the one request a program has made, so no grant reaches more than it
   did. After an IPv6 literal the methods follow its `]`. The design's `:GET,POST/repos/**` spelling is kept.
2. **A suffix that is not methods stays in the host** (`netscope::read`). `h.com:8443` reads as before, a host
   string no request names, so the broker's host check matches it no more than it did. Every grant parser refuses it
   through `check_entry`, in words: "a `net` entry names no port".
3. **The order and the meet.** `entry_within`: the same host string, the child's methods a subset of the parent's,
   then the paths as D-V2-139. `entry_meet`: the methods intersected (none in common: no meet), then the paths. The
   canonical form (an antichain) is unchanged; a single spelling per set keeps equivalent sets identical.
4. **The judge.** `judge(entry, method, host, path)`: the host, then the path, then the method.
   `Verdict::OtherMethod` ranks after `OutsidePath`, since "this path is granted, not by this method" is the more
   telling refusal. Every caller asks with `GET` until a primitive makes another request: the runtime's first check
   (DL0904, worded about the method), every egress hop, the broker's use check (a new `Denial::NetMethod`, DL0904,
   no new code) and `--within`. Under `--within`, a computed URL is *within* only when every boundary entry takes
   GET on every path. The OpenShell export refuses a method scope, as it refuses a path scope: the host alone would
   grant every method there.
5. **Hardened on the way.** An entry naming no host takes no request, not even one whose URL names no host
   (`host_matches("", "")` was true; such a URL is refused earlier, so this was latent).
6. **Not yet:** the primitive, the method in the broker's op and record, OCSF's `http_request.http_method`, a
   redirect's method (a 303 turns a POST into a GET), and the static half (`authority --json` `requests`' method,
   `--grants` proposing it).

**Evidence.** `netscope` tests: the grammar (11 refused spellings, each in its words; a malformed suffix read as a
host), the judge, and the order and the meet EXACT with methods over a bounded domain. The canonical-form universe
gained two method entries, and the `--within` agreement test generates three. `egress_cli`
`a_method_scoped_grant_takes_only_its_methods` runs at L0 and from a guest. It failed on the previous code ("host
… is not in the granted allowlist": the methods were read as part of the host). `sandbox_openshell_cli` failed on
the previous code too (exit 0, a document printed). Mutants N1 to N6 were red in the unit tests: methods out of the
order, out of the judge, unparsed, a union meet, a malformed suffix dropped, the default allowing every method.
N1, N2, N3 and N5 were also red end to end. Z3 §2e: 9 obligations proved (50 in all). Its mutants
`net-method-dropped` and `net-method-meet-union` each leave obligations undischarged, and CI requires both.

## D-V2-183 — P9-05 (d), second slice: `Cap[Http].post(url, headers, body)` and `post_with_secret`, the first requests that are not a GET — judged by its method at every check, its method carried to the broker and its record, a redirect's method by its status — TAKEN (head chef, 2026-10-23, under the owner's delegation)

**Context.** RW 2.11 gave an agent a way to call an authenticated API, but only by GET, and most APIs take their writes
by POST. D-V2-182 made the method dimension first, so that a POST could not widen any grant written before it: an
entry naming no methods allows GET alone.

**Decision.**
1. **The primitive.** `post(url: Str, headers: Map[Str, Str], body: Str) -> Result[Str, NetErr] ! {Net}`. The URL stays
   argument one, as `get_with`'s, so every check that reads a request's URL reads it there. The headers are held to
   `get_with`'s rules, and the body is at most 1 MiB (`MAX_REQUEST_BODY`, refused before anything is resolved).
   `PRIM_TABLE_VERSION` goes from 7 to 8 (additive). Conformance anchor `ref.prim.http.post` with four witnesses
   (`ENTRENCHED_CHANGE_RECORD.md`).
   **And `post_with_secret(url, headers, header, secret, body)`**: a POST carrying one `Secret[Str]` in one header,
   held to `get_with_secret`'s rules. It is never declassified and is sent only to its grant's audience; under the
   daemon the broker is asked for the bytes for the one origin (`broker_secret_send`, which now judges the request's
   own method first, so a capability without POST refuses it before a byte is asked). Anchor
   `ref.prim.http.post_with_secret`, four witnesses.
2. **Judged by its method everywhere a GET is judged by its path:**
   - the runtime's first check (DL0904: "a `POST` to … is outside the methods the grant names for it");
   - every egress hop, by that hop's own method;
   - the broker's use check under a daemon or lease: the custody's `check_at` carries the method, and the IPC's
     `Check` gains an optional `method`, absent for a GET, so a GET's frame and record are as before;
   - `--within`, by the request's method.

   A program asks for `root.http(["HOST:POST"])`, and `--grants` names that line.
3. **The record.** A use that is not a GET names its `method` beside `url_path`, allowed or refused. OCSF maps it to
   `http_request.http_method`, which was always `GET` before. A run report's egress hop names its `method` when it
   is not a GET.
4. **Redirects.** After a request that is not a GET, a 301, 302 or 303 is followed as a GET with no body, as browsers
   do, and judged as a GET, so a grant naming POST alone refuses it. A 307 or 308 keeps the method and the body, but
   only to the origin the program named. The body is the program's data, as its headers are, so a redirect that would
   carry it elsewhere is refused (`Reason::BodyElsewhere`).
5. **One implementation for L0 and every guest:** a sandboxed guest's `post` is performed by the host, in the same
   `prim` arm.
6. **Not yet:** `post` on the WASM engine (its custody check passes no method, and the compiler does not emit
   `post`); and PUT, PATCH, DELETE. `deploy plan` decides with
   `--within`'s verdict (D-V2-176), so it judges a POST by its method too. No test of `deploy plan` with a POST was
   written in this slice.

**Evidence.** Unit tests on the egress client (`a_post_sends_its_body_by_its_method_and_a_redirect_keeps_or_drops_them_by_its_status`:
the body sent, refused under GET alone before anything is resolved, 301, 302 and 303 as GET without a body, 307 and 308
kept, kept only to the same origin, the size limit, a GET's record unchanged). Over real TLS
(`a_post_arrives_over_tls_with_its_body`): the request line `POST /items`, its `Content-Length`, its body whole. The
broker (`a_net_use_is_judged_by_its_method_and_a_post_is_recorded_as_one`) and OCSF (`audit_ocsf_cli`) were also
tested. End to end at L0 and from a guest: `egress_cli` `a_post_leaves_under_a_grant_that_names_post_and_no_other`
(TLS out under `:POST`; DL0904 in words about the method under a host-only grant, nothing connects; the report's hop
names POST). `within_cli` `a_post_is_decided_by_its_method_against_the_boundary` covers the mint and a request decided
by its method. Mutants: see V2_LOG.

## D-V2-184 — A generic's own type and row parameters are rigid inside its body (GENERIC-RIGID-1); a laundering witness must parse — TAKEN (head chef, 2026-10-24, under the owner's delegation)

**Context.** The owner commissioned agents to build and run DeluluLang programs under conditions the head chef holds
(2026-10-24). The zero-authority agent (Haiku 5.5) found the effect row escapable through a generic. The checker gave a
function's generics fresh inference variables for its own body (`make_genv`, "fresh inference variables for this
checking pass"), so the body could decide them: `T := Int`, `e := {}`. A caller then instantiated them again. So
`coerce[A, B](x: A) -> B { x }` laundered an effectful closure into a pure type, and a pure function printed. The head
chef reproduced it against the current binary: `check` exit 0 with "`main` declares effect `Write` it never performs",
and the run printed under a console grant. It is the P16 class, the effect row escaping, through a door the laundering
suite never tried.

**Decision.**
1. **Rigid variables.** `InferCtx` marks a variable rigid (`fresh_rigid_type`, `fresh_rigid_row_var`). Unification never
   binds a rigid variable. A flexible variable meeting one is bound to it. A rigid type meeting any other type, or a
   different rigid one, is a mismatch. A rigid row tail is never closed. A flexible tail meeting it is bound to the rigid
   tail plus the effects it lacks, and only when it holds no effect the rigid side does not.
2. **Where.** `check_fn` gives the body rigid generics, and so does `check_actor` for its members' bodies. Call sites
   (`instantiate_fn`) and `spawn` still instantiate flexible ones: the caller picks, and the body holds for every pick.
3. **Messages** name the parameter the author wrote (`expected T, found Int`; `fn() -> Unit !{e}`). `TypeNames` gains
   default `type_var_name` and `row_var_name`, and the checker supplies the rigid generics' names.
4. **The laundering suite** (`crates/delulu-check/tests/laundering.rs`, entrenched) gains five witnesses. Its
   `rejects_with` refuses a witness that does not lex or parse (any DL01xx or DL02xx). F-3, F-3b and F-4 had written two
   statements on one line (DL0209) since they were written, and passed through the parser's recovery. Each is repaired,
   and each is still refused with its own code. Recorded in `ENTRENCHED_CHANGE_RECORD.md`.

**Why this hardens, and redefines nothing.** A generic signature promises its result for every `T` and `e` a caller
picks, and every call site already instantiated them so. Only the body was checked against one choice of its own. No
program the rule always meant to accept is refused: `delulu-check` and the conformance suite pass unchanged, and so
does the stdlib's `Promise[T, e]`. A program that relied on the hole is refused DL0401 (no new code).

**Evidence.** `laundering` 17 passed (12 before). Mutants red: the type guard dropped (three witnesses), the closed-row
guard dropped, actor generics flexible. Agent C's programs (`p2_09`…`p2_13`, `p3_01`…`p3_03`) are refused DL0401.
Suites on the laptop: `delulu-check` (all targets), `delulu-conform` and `delulu-wasm` with 0 failures; `delulu`'s `core_invariance` 53, `cli` 19, `examples_run` 1 and `cli_contract` 6 passed. The full suite on three OSes is the pull request's CI run.

## D-V2-185 — `--within` decides a file use at its call, where the source writes its path (B9) — TAKEN (head chef, 2026-10-25, under the owner's delegation)

**Finding (B9, the agent test campaign of 2026-10-24, the least-authority agent, Sonnet 5.5; confirmed by the head chef on
the current binary).** `rd.read_text("../forbidden/secret.txt")` on a capability minted for `./allowed` was *within* a
boundary of `console` and `fs.read=./allowed`, and `run` with exactly those grants refused it (DL0904). `--within` judged
each file capability at its mint only; the use's path, checked by `prim::resolve_in_anchor` at the call, was never read.
It is the class D-V2-180 and D-V2-181 closed for secret sends and request paths: a verdict that skips a use-time check.

**Decision.** A use whose path the source writes (`read_text`, `list_dir` and `narrow` on `Cap[FsRead]`, `write_text`
and `append_text` on `Cap[FsWrite]`, by the checker's type at the call) is decided at its call:
- **The source shows the capability's mint** (`root.fs_read("DIR")` through a `let`, with `narrow`s of literals; any
  rebinding forgets it, SEND-REBIND-1's walk): the runtime mints and narrows it and asks `resolve_in_anchor`. A use it
  refuses is *exceeds*, refused whatever the boundary grants, as a URL the egress client will not read is (D-V2-181).
  The finding is labelled with the capability's mint line, since no grant lets a capability's use leave its directory.
- **The source does not show the capability** (a parameter, a field): a relative path that never climbs above where it
  starts is *within*; a spelling every call refuses (D-NE-29) is *exceeds*; any other path is *inconclusive*.
- **A computed path** is decided by its capability's mint, as a computed URL is (D-V2-181). **The limit, recorded:** such
  a use can still be refused at its call (DL0904) under a *within* verdict. It asks for nothing beyond its capability,
  and the verdict does not judge a name the source does not show.

`--grants` is unchanged. Witness: `within_cli` `a_file_use_is_decided_by_its_path_against_its_capability`, holding each
verdict on a written path to what `run` does with the same grants. It was red before the fix, and four mutants turn it
red: a refused use answered within, every path taken as staying below, narrowings skipped, and hostile spellings unchecked.

## D-V2-186 — The supply-chain gate (DL1009, DL1001) holds on every route that runs, ships or plans a package; reports keep answering (RUN-PIN-1) — TAKEN (head chef, 2026-10-25, under the owner's delegation)

**Finding (RUN-PIN-1, on-demand routine run 6, found while verifying the agent campaign's A2 on the binary).** A
dependency pinned `effects = []` that writes, or one that carries no pin, is DL1001 under `delulu check`, `build` and
`lock`. `delulu run app` ran it, on the interpreter, in the sandbox and on the WASM engine; `build --target wasm` wrote a
`.dwx` that ran it; `test` ran the package's tests; `deploy plan` planned it and `sandbox policy` previewed its run. Only
`build_workspace` and `cmd_lock` called `check_self_authority` and `check_pins`; every other loader checked types alone.
The specification, the book and `provenance.rs` all say the gate refuses "before any code runs", and its witness drove
`build` alone.

**Decision.** One function, `delulu_check::deps::supply_chain_gate` — every package within its own manifest (DL1009) and
every dependency within its consumer's pin (DL1001) — and every route that **runs** a package's code (`run` on every
engine, `run --sandbox`, `test`), **ships** it (`build`, `build --target wasm`, `lock`, `publish`), **plans or previews**
it (`deploy plan`, `sandbox policy`) or **accepts** it as a dependency (`add`, which computes the authority it pins) calls
it and refuses on its errors. The routes that only **report** — `authority`, `why`, the Atlas — keep answering for such a
package: they run nothing, and an operator needs to read what a package does to decide its pin. No new code: the refusal is
DL1001 or DL1009, as `build` says it.

**Why.** A pin is the consumer's ceiling on a dependency, and the commonest command, `run`, is the one an author types
first; a gate that `build` keeps and `run` skips protects only those who build first. The reports are left alone because
refusing them would hide the answer the pin is decided from.

Witness: `provenance` `no_route_runs_or_ships_a_package_whose_dependency_breaks_its_pin` (seven routes, a broken pin and
a missing one, and a control that goes through all seven); red on the old code; M806–M810 red.

**What it moved in three existing tests (the full suite, 2,544 passed and these 3 failed).** Two `deploy_cli` fixtures
were packages `build` refuses: each read a secret its own manifest never declares (DL1009), and `deploy plan` now refuses
to plan them. They declare the secret now, and every assertion about the profile stands. `sandbox_run_cli`'s package
whose manifest declares no effects was refused DL0701 by `run`'s manifest gate; the supply-chain gate now refuses it
first, with DL1009, as `build` does, and the test asserts that code. A single file beside a manifest has no package
check, and is still refused DL0701.

## D-V2-187 — A special-use host in a delegation is named where it is written, and the node is still made; the `net.special` dimension on a node is the next slice (B1) — TAKEN (head chef, 2026-10-25, under the owner's delegation)

**Finding (B1, the agent test campaign of 2026-10-24, the least-authority agent, Sonnet 5.5; re-run by on-demand routine
run 6).** `grants delegate --net 127.0.0.1` was accepted without a word, and every run under the lease was then refused at
the connect (`special-use`). A broker node has no `net.special` dimension (`grants_from_lease` gives a leased run an empty
`net_special`, PS-B-02), so no lease reaches a special-use address, whatever its `net` says.

**Decision.** `grants delegate` and `grants certify` — the two places a node's `net` is written — print a note for an entry
whose host is a special-use address or name (`netclass::special_use_class` of `netscope::host_of`, so a path or a method
list is no way round it): the node cannot carry `net.special`, a run under it is refused at that connect, and `--grant
net.special=HOST` on the run reaches it. The node is still made. **Refusing it was weighed and rejected:** such a node
drives every decision before the connect — the custody, the Guard's permits per destination, the audit — and
`broker_cli` and `federation_cli` use exactly that to witness a leased send with nothing leaving the machine; a refusal
would remove a working grant to report a missing one. **The real fix is the dimension**, sized here for the next slice:
`Scopes::net_special`, a set of `net` entries ordered by `netscope`'s own order and met by `set_meet`, well-formed only
within `net` (as `secret_to` lives only beside its secret), emitted in JSON only when non-empty (no existing hash, chain
or certificate moves); `--net-special` on `delegate` and `certify`; the certificate, `AuthoritySpec`, `NodeInfo`,
`authority_spec_from_grants` and `grants_from_lease`; the Z3 model's dimension; and the proposal flow, whose filing says
a special-use reach is never filed while a `net.special=HOST` line passes its check (code reading, unwitnessed). With the
dimension, a plain `--net` special-use host can be refused as `--grant net=` refuses one (D-NE-28).

Witness: `broker_cli` `a_special_use_host_is_named_where_a_delegation_is_written_not_first_at_a_connect` (four
spellings, both verbs, a control); M811–M814 red.

## D-V2-188 — A node carries a special-use reach: `net.special` joins the authority order as its twelfth dimension (RW 4.86, B1's fix) — TAKEN (head chef, 2026-10-25, under the owner's delegation)

**Why.** A run is granted a special-use host only by `--grant net.special=HOST` (D-NE-28), and a node had no such
dimension, so no lease could hand that reach on: an agent under a delegation could not reach a loopback test service or
a private-range host its operator meant it to (B1; D-V2-187 named the gap where the grant is written, the same run).

**Decision.** `Scopes::net_special`, a set of `net` entries in `net`'s grammar:
- **Order and meet** are `net`'s own (`netscope::set_within`, `set_meet`): a child reaches a special-use host only where its
  parent does, for the same host, methods and path or fewer.
- **Within `net`**: an entry is kept only within some `net` entry — the runtime mints a capability by `net` first and asks
  this set after — so `Scopes::canonicalized` drops an orphan, as it drops a `secret_to` pair beside no secret.
- **Written only when non-empty** (`"net.special"` in the canonical JSON), so every existing node's hash, every chain record
  and every certificate is byte-identical; a peer that predates the field reads "no reach" (`serde(default)`), the narrow
  direction.
- **Spelled** `--net-special HOST` on `grants delegate` and `grants certify`, which puts the host in `net` too, as
  `--grant net.special=` puts it in both for a run. Plain `--net` with a special-use host still makes the node, with the
  note of D-V2-187 (now naming `--net-special`): the tests that use such a node as a never-connecting stand-in keep their
  meaning.
- **Carried** by the certificate, the wire (`AuthoritySpec`, `NodeInfo`), the daemon's conversions, a `--broker daemon`
  run's root (`authority_spec_from_grants`), a lease's run (`grants_from_lease`), proposals' join and `grants diff` (a
  reach NEW holds and OLD lacks is a scope expansion of its own).
- **Proved** in the Z3 model: §2f's seven obligations and the full conjunction's tenth set dimension with its own teeth
  obligation (58 obligations); three new mutants, all caught.

**Not witnessed by a mutant (code reading):** `authority_spec_of`'s field (it serves plugin attenuation and proposals'
`asks`, neither of which carries a reach in any test) and `authority_spec_from_grants`'s (a `--broker daemon` run's root;
its own egress already reads the run's grants).

Witnesses: `broker_cli` `a_lease_reaches_a_special_use_host_only_when_its_node_names_one` (a loopback listener connected
under `--net-special`, untouched under `--net`; DL0802 for a child claiming the reach; `grants diff`); the broker's
`a_special_use_reach_lives_within_net_and_attenuates_like_it` and the certificate's every-dimension round trip. M818–M824
red.

## D-V2-189 — `Async` is not authority: a lease bounds what a run reaches, and its budget bounds what it computes (A17) — TAKEN (head chef, 2026-10-26, under the owner's delegation)

**Finding (A17, the agent test campaign of 2026-10-24, the builder, Sonnet 5.5, low/medium).** A lease whose conditions
read *"Write only: the console"* ran an actor program, because `spawn` and behaviour sends (the effect `Async`) are not
gated by a lease. The agent's own check: an actor "bomb" under that lease (each behaviour spawns two more actors, to a
depth of 40), run with `--limits mem=268435456,cpu=20,wall=30`, was stopped after 2 s at 258.1 MiB by the memory budget,
exit 1, and its report says `stopped_by: memory`. The ledger asked for a decision.

**Decision: kept as it is, and said where an operator will read it.** `Async` names nothing outside the run: no file,
host, device, secret or foreign library. No capability carries it, and no lease dimension could. Spawning an actor is
computation inside the run, as a loop is, and computation is bounded by the run's budget (`--limits mem,cpu,wall`), which
covers every actor the run starts. A lease that says "the console only" reaches the console only, actors or not. Gating
`Async` by a lease would make it authority over nothing. It would also be a new dimension of the authority order, which
is redefining Authority rather than hardening it.
- `delulu authority` already lists `Async` among a program's effects (checked on the builder's `actors/primes.delulu`:
  *effects: Async, Write*), so an operator who wants no actors sees it before running the program.
- `docs/for-agents.md` (*The effect kinds*) now says all of this.
- Nothing in the runtime changes.

## D-V2-190 — An empty `rand.int` range is DL0908, an argument outside what the operation accepts, not DL0904 (A15) — TAKEN (head chef, 2026-10-27, under the owner's delegation)

**Finding (A15, the agent test campaign of 2026-10-24, the builder, Sonnet 5.5, low).** `r.int(lo, hi)` draws from `lo` up
to but not including `hi`, and nothing said so: 6,000 draws of `r.int(1, 6)` never produced 6. `r.int(5, 5)` aborted the
run as `error[DL0904]: rand.int requires lo < hi`. `delulu explain DL0904` says *"An operation reached outside its
capability's SCOPE — a path outside the granted prefix, a host not on the granted list"*. A bad argument is not that.
Checked again on 2026-10-27 at `d2752cf`: both engines faulted DL0904 with that text. Both also recorded
`int.refused` with it in the effect trace. One test drew from an empty range: `refusal_trace_parity` (RW 4.64), which
pinned DL0904 as the code the two engines' traces must agree on. It checked that the engines agree, not that the code
was right.

**Decision: a new code, DL0908, "an argument outside what the operation accepts".** It is a runtime fault, beside
DL0901–DL0905.
- **Why not keep DL0904.** It is the capability-scope code, the one an operator, a monitor or an agent reads as an
  authority event. `STABILITY.md` §1 makes a code's identity add-only. Filing a bad argument under it was a misfiling
  that blurs that identity, not part of it. 2.0.0 is a major release, and moving a fault to its proper code is
  allowed there in any case.
- **Why not an existing code.** DL0901–DL0903 name overflow, division and indexing. DL0907 is the compiler-bug class,
  and an empty range is the program's own value, not a broken static guarantee.
- **Why not draw something anyway.** An empty range names no value. Returning `lo`, or swapping the bounds, would turn
  a bug into data that flows onward, which is what DL0901 refuses for arithmetic.

**The fault itself.** Both engines report the same text, from one function (`delulu_runtime::prim::rand_int_range_fault`):
*"rand.int(lo, hi) draws from lo up to but not including hi, so it needs lo < hi; it was given lo = 5, hi = 5"*. The
effect trace keeps its `int.refused` record, now naming DL0908. By RW 4.62 that record means *"it did not happen, and
why"*, whether the custody, the broker or the effect itself stopped the operation, and its detail carries the code.

**Documented:**
- the half-open range in `delulu explain DL0908`, `docs/for-agents.md` *Determinism*, and the book's determinism
  paragraph;
- `ref.rule.runtime.faults-are-diagnostics` (`crates/delulu-conform/src/rules.rs`) gains DL0908 in its statement and its
  `enforced_by`.

**Entrenched files:** `crates/delulu-conform/src/rules.rs` and `tests/conformance/witnesses.toml` (two rows, for the
anchor `ref.diag.DL0908`). Both are recorded in `ENTRENCHED_CHANGE_RECORD.md`.

**Evidence:**
- `refusal_trace_parity`'s `an_operation_its_effect_refuses_is_traced_the_same_by_both_engines` now expects DL0908. It
  went red on the change (*left: "DL0908", right: "DL0904"*), and it still asserts the two traces are byte-identical and
  end in `int.refused`.
- `fault_parity`'s `an_empty_rand_range_is_dl0908_on_both_engines` covers `(5, 5)`, `(6, 5)`, `(0, Int::MIN)` and
  `(Int::MAX, Int::MIN)`. It was red before the change (`WASM trap: DL0904: rand.int requires lo < hi`) and green after.
- `a_one_wide_rand_range_draws_its_one_value_on_both_engines`: `r.int(5, 6)` is 5 under three seeds, the engines alike.
- Mutants, each red with the control green:
  - M855: the interpreter keeps DL0904.
  - M856: the WASM host keeps DL0904.
  - M857: the interpreter's guard is `hi < lo`.
  - M858: the WASM host's guard is `hi < lo`.

  M857 and M858 panic the host with *"attempt to calculate the remainder with a divisor of zero"*, so the guard is
  load-bearing.

## D-V2-191 — `--seed` is mixed (splitmix64) before it starts the generator, so every seed draws its own sequence; every seeded sequence changes in 2.0.0 — TAKEN (head chef, 2026-10-27, under the owner's delegation)

**Finding (seen on the way in A21's change, 2026-10-25; V2_LOG *Seen on the way, not changed*).** Both engines started
their xorshift64 generator at `(seed == 0 ? 0x9E3779B9 : seed) | 1`. Witnessed on the interpreter at `ac2920e`:
- **Seed pairs collided.** `--seed 2` and `--seed 3` drew the same sequence, as did 4 and 5, and every even seed and
  the odd one after it. Seeds 0, `2654435768` and `2654435769` drew one sequence between them. Half of all seeds were
  duplicates, so a sweep over seeds 0..N ran each case about twice.
- **A small seed drew tiny numbers first.** Its state has few bits set, and xorshift needs steps to spread them. Seed 1's
  first `r.float()` was `5.9e-11`, and every seed from 1 to 64 drew a first float below 0.01 (their mean `1.9e-9`).

`different_seeds_diverge` compared seeds 1 and 2, which never collided (1 | 1 = 1, 2 | 1 = 3).

**Decision: mix the seed with splitmix64 before it becomes the state, on both engines and in every path.** This covers
`--seed`, `delulu test`'s per-test seeds, the sandboxed guest's, and the unseeded wall-clock default.
- splitmix64 is a bijection on `u64`, so distinct seeds start in distinct states.
- **The one exception.** xorshift stays at zero forever, and exactly one seed mixes to zero:
  `0x61C8864680B583EB`, which is `2^64 - 0x9E3779B97F4A7C15`. That seed starts at the constant instead, where
  `0xCD36DA00563C5EF6` also starts. 2^64 seeds into 2^64 − 1 nonzero states must share at least one state, and this is
  the only shared pair. Both seeds were found by inverting splitmix64.
- One function, `delulu_runtime::prim::rng_state_from_seed`, is called by the interpreter and by the WASM host, so the
  engines cannot start differently.

**Why now: it changes every seeded sequence.** A run recorded under `--seed N` with 1.x draws different numbers under
2.0.0. Nothing in this repository stores a seed for replay. The device simulator reports its seed, and no test or
`.delulu` program pins a seeded value; the tests compare the two engines, or a run with itself. `STABILITY.md` promises
nothing about seeded sequences. 2.0.0 is a major release, and after it this change would have to wait for 3.0. The
generator itself (xorshift64), and the mapping of a draw to `r.int` and `r.float`, are unchanged.

**Evidence:**
- `prim.rs` `determinism_tests`, each red before the change (the first three):
  - `neighbouring_seeds_draw_different_sequences`: seeds 0–63, `0x9E3779B8`, `0x9E3779B9`, `u64::MAX - 1` and
    `u64::MAX` draw distinct sequences. Before: *"seeds 2 and 3 draw the same sequence"*.
  - `a_small_seeds_first_draw_is_spread_over_the_range`: the first draws of seeds 1–64 average between 0.3 and 0.7,
    with at most 4 below 0.01. Before: a mean of `1.9e-9`.
  - `the_seed_that_mixes_to_zero_still_draws`.
- `delulu-wasm` `neighbouring_seeds_draw_differently_on_both_engines`: seeds 0–7 are distinct, and alike on both
  engines. Before, red with *"seeds 2 and 3 draw the same sequence"*.
- Mutants, each red with the control green:
  - M859, no zero-state guard: the zero-state test fails.
  - M860, the WASM host keeps `| 1`: the two engines part under seed 0.

## D-V2-192 — `--seed` and `--clock fixed:` reach every actor: each actor draws from a stream of its own, keyed by its spawner; the interleaving of actors stays unordered (ACTOR-DETERMINISM-1, A11) — TAKEN (head chef, 2026-10-27, under the owner's delegation)

**Finding (the laptop, 2026-10-27, while looking at A11).** The generator and the fixed clock are thread-locals in
`prim.rs`, and `run` sets them on the main thread, where `main` runs. An actor's turns run on a worker thread, which
never saw either setting. Witnessed with `--seed 7`:
- `main` drew 220748 on every run, but an actor holding the same `Cap[Rand]` drew 395444, 186614 and 634774 on three
  runs with `--actors-threads 1`, and other values with 4.
- Under `--clock fixed:1000`, `main` read 1000 and an actor read the wall clock (`1793119167378`).

Spec §6.2 says *"`--seed <u64>` fixes `Cap[Rand]`; `--clock fixed:<ms>` fixes `Cap[Clock]`"*, with no exception for
actors, and `docs/for-agents.md` says `--seed N --clock fixed:MS` *"fixes randomness and time"*. A11 of the agent test
campaign (three runs of an 8-worker prime counter under `--seed 7` printed their lines in three orders) had looked like
an ordering question. Its programs drew nothing at random, so this defect stayed out of sight.

**Decision.**
- **The clock.** The actor system reads the run's fixed clock when it starts, and every worker adopts it.
- **The generator.** Each actor has a `Cap[Rand]` stream of its own (`prim::RandStream`), carried in its cell and
  installed on the worker for each of its turns (`swap_rand_stream`). A stream starts from a key that its spawner
  derives when spawning (`next_spawn_key`): the spawner's own key and how many actors it has spawned, mixed by
  splitmix64. `main`'s key is the run's seed (`set_rand_seed`), or the wall clock when there is none. An actor's turns
  run one at a time, so its draws depend only on its key and its own messages, not on which worker runs it or how many
  workers there are.
- Drawing a spawn key draws nothing from anyone's `Cap[Rand]`, so a program's own sequence of draws is unchanged by
  spawning.
- **What stays unordered.** The interleaving of different actors' turns. Stage 7 promises ordering per sender only
  (*"no global order"*), and that is kept. Even one worker does not promise a single order, because `main` keeps
  sending while the worker runs. `docs/for-agents.md` and the actors guide now say so, which closes A11.

**Evidence.**
- `actors_cli` `seed_and_fixed_clock_reach_every_actor_on_any_number_of_workers` drives the binary. Two actors each
  spawn a child, one of them takes two turns, and a third reads the clock. It checks:
  - the sorted lines are identical across two runs on one worker and one on four, and differ under another seed;
  - an actor reads `clock 1000`;
  - the actor's second turn draws on from its first.

  It was red before the change (*"an actor reads the fixed clock: … clock 1793119228350"*), passed 15 of 15 runs after
  it, and the program's sorted output hashed alike on 1, 4 and 8 workers.
- Mutants, each red with the control green:
  - M861: a worker never adopts the fixed clock.
  - M862: a behaviour's turn runs on the worker's own generator.
  - M863: a spawn key ignores its spawner's key.
  - M864: `--seed` never sets `main`'s key.
  - M865: a turn's stream is not saved for the next.

## D-V2-193 — `IoErr::Other(m)` carries the kind of failure in the same words on every OS, never the operating system's own message (B11) — TAKEN (head chef, 2026-10-29, under the owner's delegation)

**Question.** A file operation the OS refuses for a reason that has no variant of its own (`NotFound`, `Denied`) is
`Other(m)`. What is `m`?

**What it was.** Both engines put the OS's own message in `m`. The agent test campaign's least-authority agent (B11,
2026-10-24) read a 300-character name on Windows and its program received *"The filename, directory name, or volume
label syntax is incorrect. (os error 123)"*. The same program receives other words on Linux and macOS, and on Windows
the words are in the machine's language. So:
- a program that prints or branches on `m` behaves differently by machine;
- the text tells the program which OS and which language its host runs, which nothing granted it;
- it was the one refusal text a program received raw. Every other refusal is opaque (`Refused`) or a named variant.
  For the host's own messages the project already refuses to show raw OS text (C27, D33: `unreadable_paths`,
  `sandbox_run_cli`).

**Decision.**
- `m` is the error's kind in Rust's words: `std::io::ErrorKind`'s `Display`, such as `invalid filename`,
  `is a directory` or `uncategorized error`.
- `prim::io_other_text` is the one place it is made, and the interpreter and both WASM host sites (read, write) call
  it.
- The variants, and when each is chosen, are unchanged. `m` was never specified, by the Stage 1 specification or by
  the reference, so no promise changes.
- A kind is less than the OS said: the error number and its sentence are gone. The program never had a use for them that did
  not tie it to one machine.

**Evidence.**
- `fault_parity::an_os_refusal_reaches_the_program_in_the_same_words_everywhere` reads and writes a 300-character name
  under a granted directory, on both engines. It was red before the change, receiving the OS's sentence, and expects
  `invalid filename` twice.
- Mutants M894 (the interpreter), M895 (the WASM read) and M896 (the WASM write) bring back `e.to_string()`, and each
  is killed.
- The pull request's run checks the same words on Linux and macOS, where a name past 255 bytes is `ENAMETOOLONG`.

## D-V2-194 — DEADMAN-DEAF-1: a dead-man counts its holder's silence only while the daemon could hear it — every dead-man instant is read from a hearing clock that stands still while the daemon was not running — TAKEN (head chef, 2026-10-29, under the owner's delegation)

**Question.** D-V2-127 decided that a dead-man counts its holder's silence, never the daemon's, and stamped each
request's arrival on a reader thread of its own. What does a dead-man count while the whole daemon is not running?

**What it was.** The silence, in wall time. The reader threads are the daemon's, so a daemon that was stopped (`SIGSTOP`,
a debugger), swapped out, or not scheduled on a busy host stamped the beats that had waited in its socket when it ran
again, after their deadline. The serve loop, woken by its own timeout, could also judge before any reader thread had run.
A stop of three periods revoked a healthy monitor that had beaten 73 times, and the run under it, and told the monitor it
"sent no beat" (`monitor_cli` `a_stopped_daemon_is_not_its_monitors_silence`, red 3 of 3). It was found while
root-causing a once-seen macOS red (RW 7.25) whose message held no evidence.

**Decision.**
- The daemon keeps a hearing clock: monotonic time that stands still while the daemon could not hear. Every instant a
  dead-man is judged by reads it: the arming, the first period's start, a beat's and a disarm's arrival, the serve loop's
  wait for its next request, and the as-of its firing is judged by.
- While a dead-man is armed a thread of the daemon looks at the time every 10 ms (`HEARING_TICK`). A gap between two
  looks past 50 ms (`HEARING_SLACK`) is deaf time, and the clock leaves it out. Until the thread looks again, the time
  since its last look past the slack is left out too, so a judgment made the moment the daemon resumes (before the
  thread has run) already leaves the pause out. The clock is continuous and never runs faster than the wall.
- The clock listens from just before an arming is decided, and sleeps once no dead-man is armed, so an idle daemon still
  never wakes. Starting to listen counts nothing before it as deaf.
- `broker.log` names every gap of 100 ms (`deadman::MIN_PERIOD`) or more: *"this daemon did not run for N ms"*. That
  line is the evidence that tells a daemon's stop from a holder's silence in a red run.

**Not changed.**
- D-V2-106's rules: a beat that arrived after its deadline rescues nothing, and a stream of requests cannot hold an
  unbeaten dead-man off. Both now hold in hearing time.
- D-V2-127's rules: a beat and a disarm are judged by their arrival, and a beat's next period starts when it is answered.
- A holder that is itself stopped or dead is revoked one period of hearing after its last beat
  (`a_stopped_monitor_is_a_dead_one_and_says_so_when_it_resumes`, unchanged, and the new witness's killed monitor).
- `broker.log`'s late-answer lines keep the wall clock, since they measure what a client waited.

**What it gives up, bounded.**
- While the daemon cannot hear, no dead-man fires. Nothing under its custody proceeds either, because every use under
  the daemon's custody asks the daemon.
- A truly dead holder's firing is delayed by the time the daemon was stopped past the slack, never more.
- On a host that starves the whole daemon, the clock runs slow by what the hearing thread waited past its slack.
- Not covered: one reader thread starved while the rest of the daemon runs. That is D-V2-127's window of microseconds
  between a request read whole and its stamp, unchanged.

**Evidence.**
- The witness above: green with the fix, its daemon's log naming the stop, and its killed monitor still revoked about
  500 ms later.
- `brokerd` `hearing_tests` (4 tests, one over 200 generated histories) and
  `an_arming_is_answered_with_the_hearing_clock_listening`.
- Mutants M908 to M915 are each red, M914 by the handler test written for it. The runners: macOS `40958776212`, Windows
  `40958779882`, with every `delulu` test target.

## D-V2-195 — RW 4.79: a use the program's own capability refuses is judged before its custody is asked, and the chain records it refused (`refused_by: capability`), never allowed — TAKEN (head chef, 2026-10-31, under the owner's delegation)

**Question.** A capability may hold less than its node: a directory narrowed under the node's, one host of the node's, a
path or a method fewer. Every gate that asks a custody (the interpreter's, a sandboxed guest's host's, the WASM host's)
asked it first, against the node's grant, and the effect then judged the capability. What should the chain say about a
call the capability refuses, and who answers it first?

**What it was.** Measured on `658daf1` by on-demand routine run 10, under `--broker daemon`: each of seven such calls —
`write_text`, `append_text`, `get`, `get_with`, `post`, `get_with_secret`, `post_with_secret` — left a `use allow` in the
chain on every route that ran it, then faulted DL0904: nineteen allowed uses that never happened (three routes for five calls, two for the two that carry a broker-held secret, which a guest's channel cannot). A write neither the
capability nor the node allows left the custody's own `deny`. Under a lease with a Guard rule on reads, a read beside its
capability's directory was asked of the Guard: a `guarded` rule refused it DL1410 and told the agent to request a permit
for a read its capability can never make, and a `warn` rule printed its warning and recorded `guard_warn` `allow`.

**Options.**
- (a) Judge the capability first and record nothing (the row's first sketch). Rejected: the write neither allows is
  recorded `deny` today, and an out-of-band monitor's `denies=N/MS` rule (P8-04, D-V2-100) counts it — a program probing
  `../../` would leave the chain.
- (b) Keep the custody first and append an overriding `deny` after its `allow`, as RW 4.46 (egress) and P8-04 (an
  envelope) do. Rejected: those refusals are made after the custody by a party that needs what it answered (where the
  name resolved, the command's magnitudes); a capability's verdict needs nothing from the custody, so the `allow` would
  record a decision that was never needed, and a Guard would still be asked about a use that cannot happen.
- (c) **Chosen:** the capability first, on every gate, and the custody TOLD its refusal.

**Decision.**
- `prim::capability_refusal` is the effect's own rule — `resolve_in_anchor` for a file, `url_verdict` for a request — in
  the effect's own words, judged before the custody and performing nothing. The gate refuses the call with it and does
  not ask the custody; a guest's handle carries no scope and is judged by the host that holds the capability. A URL that
  is not `https://` is still answered `Refused`, a value, with no use asked (HTTP-SCHEME-1).
- The custody is told: `Custody::note_capability_refusal` (a no-op by default; embedded custody keeps no chain). Under
  the daemon the broker records one `use` `deny` at the argument the use would have been asked about, naming
  `refused_by: capability`, `code: DL0904`, a machine-readable `reason` (`outside-scope`, `hostile-path`, `other-host`,
  `outside-path`, `other-method`, `ambiguous-path`) and, for a network use, its `url_path` and a method other than GET, as
  the broker's own record of a use names them. Never the request's words: a URL's query is in no record (D-V2-169).
- Recorded exactly where a use of its class is: every synchronous-class use (a write, a request), and an epoch-class use
  (a read) only where a Guard rule makes it round-trip. An ungated read its capability refuses is recorded by no one, as
  an ungated read is.
- The wire: `CapabilityRefused`, an additive `broker/1` request. A record lost to a held chain lock is answered DL1401,
  never `Recorded` (RW 4.61's rule), and the client says once on standard error that the chain lacks it; an op the broker
  does not know is refused in words, nothing recorded.
- The WASM host asks it between ARTIFACT-CLAIM-1's declared-effects ceiling and the custody, so an artifact's undeclared
  effect is refused DL1202 first, as before.
- The effect keeps its own check after the custody. A link swapped in after the capability's judgment is refused at the
  open (FS-RACE-1, RW 4.72) — `a_narrowed_directory_swapped_for_a_link_reads_nothing_outside` still swaps it.

**Not changed.** What runs and what is refused: every call refused before is refused now, and none refused now ran before.
The custody's own judgment of a use the capability allows, the Guard's, and their records. The trace (the dispatch, then
`<op>.refused`, RW 4.62), byte for byte on both engines. Authority and the Guard mean what they meant: this hardens the
chain's account of them.

**What moves, for the 2.0.0 *Breaking* judgment.** A call its capability refuses is refused DL0904 under every custody.
Under the daemon, a Guard rule's DL1410–DL1413, or a revoked or expired node's DL0801/DL1402/DL1403, could answer such a
call first; now the capability's DL0904 does, and the custody's code answers the next call the capability allows. The
chain's record of such a call is `deny` with `refused_by: capability` (was `allow`, `guard_warn` or `guard_block`).
STABILITY.md §1 promises a refused program's code for the typing, effect and authority rules of `docs/reference`, which
name no order between a capability and its custody; the move is named in CHANGELOG all the same, for the merge into
`launch/2.0.0` to judge.

**Evidence.** `broker_cli` `a_use_its_capability_refuses_is_recorded_refused_and_never_allowed` (eight calls and an
ungated read, on the interpreter, the WASM engine and a sandboxed guest) and `guard_e2e`
`a_guarded_read_its_capability_refuses_is_refused_by_the_capability_never_the_guard` (both tiers, both engines), each read
red on `658daf1` first; the broker's and the daemon's unit tests; mutants M946–M960 (V2_LOG, on-demand routine run 10).

## D-V2-196 — RW 4.67: while a secret's bytes are in the process for a send, the `delulu` binary's allocator zeroes every block it frees, so a sent secret leaves no copy behind — TAKEN (head chef, 2026-10-31, under the owner's delegation)

**Context.** Under the daemon a secret's bytes enter the program process only to be sent: the broker hands them to the
egress client for one origin (D-V2-134). Routine run 25's red team (F4) found one copy of the plaintext in the run's heap
after the send, until exit. On-demand routine run 12 read it again on `9553ab7` by scanning the run's memory through
`/proc` while it waited after the send: one copy, `…on: hunter2-…` — the serialized request's `authorization` header — in
the arena of reqwest's runtime thread, on the interpreter and on the WASM engine alike. The copies are made by code that
is not ours (the custody's answer as it is read and parsed, `reveal`'s copy, each header the egress client builds,
reqwest's `HeaderValue`, hyper's head buffer) and each buffer is freed as it stands. No call site can clear a buffer it
never sees; zeroing our own copies alone would have left hyper's.

**Decision.**
1. **A window, and the allocator's rule.** `delulu_runtime::scrub::Window` counts, process-wide, the sends whose bytes
   are in this process. While one is open, the `delulu` binary's global allocator (`ceiling::CeilingAware`) zeroes every
   block before it frees it — on every thread, because the request is made on reqwest's — and moves a block that grows
   by `alloc`, a copy and that same `dealloc`, never by the system's `realloc`, which may free the old block as it stands.
   Outside a window the cost is one atomic load per free.
2. **Two windows cover a send.** One while the custody is asked for the bytes (`interp::broker_secret_send`, both
   engines: the answer is read and decoded into buffers freed as it goes); one around every request that carries a
   secret (`prim`'s request arm, under any custody), which closes only after the request's client has been dropped and
   its runtime thread joined. Between the two the bytes live in one `SecretVal`, which zeroes its own buffer as it is
   dropped (P17-F).
3. **What it is not.** The rule is the `delulu` binary's: a program embedding `delulu-runtime` under another allocator
   gets no scrubbing. It clears what is freed, not what is held — the embedded custody holds every secret's bytes in the
   process for the whole run by design, and there the window clears only the send's own copies. A sandboxed guest is
   never handed a secret (DL1401), so its host's send is not a case.
4. **Not taken:** zeroing only our own copies (hyper's head buffer is out of reach — it is the copy run 25 found); a
   zero-on-every-free allocator (a memset on every free of every program for a residue that exists only during a send);
   the broker making the request itself (the serial daemon would wait on the network — RW 4.53 — and D-V2-197 says what
   it would not buy).

**Evidence.** `broker_cli` `a_sent_secret_leaves_no_copy_of_its_bytes_in_the_run_once_its_request_is_done`, over real TLS
(a server presenting a certificate only `SSL_CERT_FILE` vouches for) under `--broker daemon`, on the interpreter (scanned
while the program waits on standard input, which allocates nothing) and the WASM engine (scanned in a held second
request): red on `9553ab7` — `[([], Ok(10), Ok(1)), (["--engine", "wasm"], Ok(13), Ok(1))]`, one copy each beside the
control's count — and green with the fix; two controls in it (the server saw the secret arrive; the scan finds the
program's own path). Four concurrent copies of the test binary green. By hand with a 46- to 8,186-byte secret: the
unscrubbed binary left up to six partial copies, the fixed one none. Mutants: see V2_LOG (on-demand routine run 12).

## D-V2-197 — RW 4.66: a raw `SendSecret` from a caller of the broker's own user stays answered — the socket admits exactly whom the state directory admits, and the directory holds every secret in plaintext — DECIDED, a residual named in `docs/DEPLOYMENT.md` §5 — TAKEN (head chef, 2026-10-31, under the owner's delegation)

**Context.** Routine run 25's red team (F3): a raw `SendSecret` naming a node returns the bytes to any caller of the
socket, though the node holds no `Declassify` and `Expose` would refuse it; the broker cannot tell the run's egress
client from another process of the operator's user. The row proposed binding a send to the run that redeemed the node
(the peer credential of the connection), or handing the egress client a one-use sealed token instead of the bytes.

**What was measured** (on-demand routine run 12, Linux): the socket lives in the state directory, which the broker
makes `0700` (`broker_transport.rs`), beside `broker.key` (`0600`) and `secrets.json` (`0600`), whose JSON holds each
secret's bytes in plaintext (`{"API_TOKEN": "hunter2-…"}`, read back with `cat` as the same user). On Windows the pipe
admits only the broker's own user (an owner-only DACL, then the client's SID compared), the user whose directory it is.
So every caller that can send `SendSecret` can read every secret, and the broker's key, without asking anything.

**Decision.** No change to the broker's answer; RW 4.66 is closed as a decided residual of the same-user boundary,
which Tiers 0 and 1 do not contain and Tier 2 does (`docs/DEPLOYMENT.md` §5 now says so in these words).
- **The peer binding is not taken.** A caller that can reach the socket can read the lease token the run was handed (it
  is on the run's command line, `--lease TOKEN`) and redeem it itself, and so be the peer the binding names; it can read `secrets.json` in any case. A check that protects
  nothing the caller cannot take another way would read as a boundary that is not there.
- **The sealed token is not taken.** Something in the program process must put the bytes on the wire; only the broker
  making the request itself would keep them out of it — a redesign of the serial daemon around network waits (RW 4.53)
  that would still leave the same caller its `cat`.
- **The one same-user process that cannot read the directory cannot reach the socket either:** a sandboxed guest opens
  no new socket (seccomp, GUEST-SOCKET-1, D-V2-61), is refused the state directory and every network but its channel on macOS
  (Seatbelt, T14), and on Windows runs as a per-run AppContainer with no network and none of the operator's files
  (PS-B-03, T14). What separates a program's account from the custody is a
  second account (Tier 2): it reaches neither the socket nor the directory.
- **Revisit** if the secret store is ever sealed against its own user (a key the user's processes cannot read): then
  the socket would be the weaker door, and the binding worth building.

## D-V2-198 — ESTOP-QUEUE-1 (RW 4.53): a revocation — the operator's e-stop, a monitor's quarantine — is handled before every request waiting at the broker daemon, and dead-men are judged as of the earliest arrival still waiting — TAKEN (head chef, 2026-10-31, under the owner's delegation)

**Context.** The daemon is serial: one handler touches custody, in the order whole requests are handed to it (RW 4.40).
A request that records takes the audit chain's append lock and waits for it up to 5 s while another writer — a sandboxed
run's host, a CLI command — holds it (RW 4.48; one wait per request since RW 4.58's rest). Routine run 18's red team (F4)
saw an e-stop behind four queued requests answer `did not answer within 15s` while the revoke landed at ~25 s; RW 4.53
recorded it and the chain-length half was fixed by D-V2-159 and D-V2-160. On-demand routine run 14 re-measured on
`0ebbc15`: beside 12 sandbox loops the revoke took 22 ms median, 49 ms max (control 29 / 50 ms) — the contention is gone —
but with the chain HELD, F4's shape still held: `grants revoke` behind four queued mints answered DL1401 at 15.18 s with
"start it with `delulu broker start`", and the node was revoked about 25 s after the pull. Devices park themselves after
1 s of broker silence (DEADMAN-1), so the machine fails safe; the operator was told the e-stop failed, wrongly, and why.

**Decision.**
1. **A revocation goes first.** `ReqBody::is_revocation` (`Revoke`, `Quarantine`). A reader thread hands a revocation to
   the serve loop on its own bounded line and then a wake on the ordinary line (`try_send`: a full line needs none — the
   handler is not waiting, and reads the revocation line before its next request). The handler stays serial and alone in
   touching custody; it reads the revocation line first. A revocation now waits for the request being handled and no
   other: on the same held chain the e-stop answered in 9.81 s, `ok: revoked`, its record said lost (RW 4.58).
2. **Overtaking only narrows.** A request a revocation overtakes is decided after it, against less authority: a mint or
   delegation under the revoked node is refused instead of minted and revoked; a use under it is denied; a refusal record
   needs only that its node exists (`record_*_refusal`), so none is lost; a `Shutdown` that arrived first no longer
   swallows the e-stop.
3. **Dead-men are judged as of the EARLIEST arrival still waiting** — the requests already handed over sit in a bounded
   `waiting` line, in order — never as of a revocation's own arrival when it goes first: a beat that arrived in time and
   waits behind a held request keeps its dead-man (DEADMAN-BUSY-1, D-V2-127, kept).
4. **A broker that accepted a request and did not answer it in time is said to be running.** Every DL1401 site
   (`grants`, `guard`, a lease run, the daemon custody) and `broker status`, `stop`, `rotate-key` point at `broker.log`,
   which names each late answer (RW 7.17), instead of "start it" / "not running" / "nothing to stop". No new code; exit
   codes and `broker status --json` unchanged.
5. **Not taken:** a second handler for revocations (custody has one writer by design); not waiting for a revocation's
   record on a held chain (RW 4.58 keeps the record whenever the chain frees within its wait — and a device parks after
   1 s of silence anyway); remembering a held chain across requests so later ones fail at once (it would refuse
   synchronous operations a short contention would have recorded).

**Residual.** On a held chain each request, a revocation included, still waits its own 5 s, so several revocations at
once are answered one after another; a continuous stream of revocations is served before everything else and holds
dead-men to the earliest waiting arrival while it lasts — a stream only a process of the broker's own user can send
(RW 4.4). Behind more than 64 waiting requests a reader's hand-over still waits for room, as before.

**Evidence.** `brokerd` unit tests: `an_e_stop_and_a_quarantine_are_handled_before_the_requests_queued_ahead_of_them`
(the chain held while four mints queue, then a revoke and a quarantine; the chain's own seqs say the order — red on
`0ebbc15`: mints at 4–7, the revoke at 8, the quarantine at 9),
`a_revocation_taken_ahead_of_a_waiting_beat_does_not_fire_that_beats_dead_man`,
`a_silent_broker_is_said_to_be_running_and_a_missing_one_to_be_started`. Mutants M969 (revocations
in the ordinary line: the old behaviour), M970 (the quarantine left out), M971 (dead-men judged by the revocation's own
arrival), M972 (no wake: an idle daemon left eight tests' revocations unanswered), M973/M973a/M973b (the words): all red,
control green. By hand (`f4.py`, F4's shape): 15.18 s DL1401 before, 9.81 s revoked after. The code is `294ddfa`.

## D-V2-199 — LATE-LET-1 (RW 2.23): a `let` binds where no closure made before it can see, on both engines — TAKEN (head chef, 2026-11-01, under the owner's delegation)

**Context.** The checker resolves each name a closure (a lambda, or on the WASM engine a function value it inlines) reads
where the closure is written, and a `let` is not recursive: `let f = fn(n: Int) -> Int { f(n - 1) }` is DL0301, *"unknown
name `f`"*. Both engines read a closure's names when it runs, in the scopes it was made over:
- the interpreter's closure holds its maker's `Env` (an `Rc` of the scope chain), and a `let` defined its name in the
  block's scope, rebinding in place a name already there;
- the WASM engine inlines a function value where it is called and lets its body read the frames `0..scopes_at` it was
  defined over, by index (WASM-HYGIENE-1), and a `let` inserted into the innermost of them.
So a `let` made after a closure, in a scope the closure was made over, was what the closure read. Found 2026-11-01 by the
laptop while reading the interpreter's scopes for RW 3.24, during the weekly verification sweep. Each shape below
checked clean:
- `let g = fn() -> Int { 0 }; let h = fn() -> Int { g() }; let g = fn() -> Int ! {Write} { out.println(…); 1 }; h()`
  in a function with an empty row: `delulu authority` said *"(none — provably pure)"*, and `run --grant console`
  printed on both engines. The same held from an inner block through a `var`, and past a parameter of the same name.
- With a `Secret.map` callback in `h`'s place, the interpreter printed `plaintext: hunter2` from a program with no
  `Declassify`. This is C88's shape: R-4 and R-2 reopened (`SOUNDNESS_AUDIT.md`). The WASM engine refuses
  `Secret.map` (DL1205).
- `let x = 1; let f = fn() -> Int { x }; let x = "two"; f() + 40` gave `two40` on the interpreter, and the WASM engine
  refused the `+` as DL1201.
The capability check held throughout (no `--grant console`, DL0703). The engines agreed with each other in every case, so
no parity test and no run of the 50,000-program differential could see it.

**Decision.**
1. **The interpreter.** A scope a closure is made over is *held*: `Scope::hold` marks it and every scope above it,
   stopping at one already held (whose ancestors were held with it), when `Expr::Lambda` makes a closure. A `let` whose
   block scope is held after its value is evaluated binds in a new child scope, and the block's later statements run in
   that scope (`exec_stmt` takes the block's `&mut Env`). The value is evaluated first because a `let` is not recursive:
   a closure made in its own value reads the name the `let` hides (`let y = [fn() -> Int { y }]`).
2. **The WASM engine.** `Cx::held` records how many scope and callable frames the function values defined in the body
   see, at most (`Cx::callable` raises it). `Cx::bind_after`, called by each of a `let`'s three forms before it binds,
   opens a new frame pair when a function value sees the innermost one. `compile_block_inner` truncates both stacks to
   the frames it found, and restores `held`, because a function value defined in a block ends with it.
3. **The meaning is the checker's**, which is lexical scope. A program that made a closure, then a `let` of a name the
   closure reads, then called it, now reads the earlier binding on both engines. A closure still shares a `var` with its
   maker, by reference (`var count = 0; let inc = fn() -> Int { count = count + 1 … }` counts 1, 2 and leaves 2, as
   before).
4. **Not taken:** copying the scope at the closure's making, which would break that sharing of `var`s on both engines; a
   new scope for every `let`, which would bring back the allocation A14 removed from every block; refusing a `let` that
   shadows a name a closure reads, which would change the language and refuse ordinary shadowing.
5. `SOUNDNESS_AUDIT.md` §D gains a box saying its claim was negated a second time, through the runtime's name resolution
   rather than the checker (entrenched, `ENTRENCHED_CHANGE_RECORD.md`).

**Residual.** The holding is conservative: once a closure is made in a scope, the first `let` after it there, and in each
scope above it, opens one scope more (one allocation), though the closure may never read that name. The WASM engine's
extra frames exist only while compiling. A `var` that a closure shares is reached by assignment as before, which is the
meaning the checker gives it.

**Evidence.** `late_let_cli` (3 tests, the real binary on the interpreter: the effects, the secret, the values), red before
(`LEAK: a swapped callee`, `LEAK: through a var`, `LEAK: past a parameter`; `plaintext: hunter2`; `two40` and a list
added to an `Int`). `loop_parity` `a_function_value_never_reads_a_name_bound_after_it` holds both engines to `41 two 23
321001`; red before: the WASM engine refused the program (DL1201), and the interpreter gave `two40 two 33 900001`.
Mutants: M974 (no hold at a lambda), M975 (a held scope bound into), M976 (`hold` marks its own scope alone), M977 (held
asked before the value), M978 (`bind_after` does nothing), M979 (a function value holds nothing) and M980 (a block closes
one frame) were all red, and the control green. M976 survived the first form of the witness, whose later value was itself
a lambda made in the outer scope and held it directly; the witness now makes that value in another function. The
suites: `delulu-runtime` and `delulu-wasm` whole; `examples_run`, `core_invariance`, `conformance`, `for_loop_cli`,
`actors_cli`, `actors_cycles_cli`, `deep_values_cli`, `wasm_cli`, `book`, `stdlib_p3`; `delulu-check`'s `laundering`.

## D-V2-200 — LOCAL-CALLEE-1 (RW 2.24): a call by a name a local binds goes to what the local holds — TAKEN (head chef, 2026-11-01, under the owner's delegation)

**Context.** The checker resolves a call's name to the innermost binding: a local before a module function, and a module
function before a prelude builtin, which DL0302 keeps apart. The interpreter's `eval_call` has two paths:
- the A14 fast path, taken for a module function when *"no local closure has the name"*;
- the general path, which calls a local only `if let Some(Value::Closure(..))`, and otherwise falls through to `load`,
  the builtins and the module's functions.
A plugin's export is a `Value::PluginFn`, not a closure. So a local holding an export, named like a module function,
called that function on both paths. Found 2026-11-01 by the laptop beside LATE-LET-1, reading `eval_call`:
- `let shout: fn(Str) -> Str ! {} = p.get("shout")?; shout("hello")` printed the module's `hello?` for the export's
  `hello!`.
- With an export `relay: fn(Cap[Console], Str) -> Str`, typed pure in the local's annotation, and a module function
  `relay` that prints, `delulu why Write` said *"program cannot perform `Write`"*, `authority` listed `Load, Read`, and
  the run printed. This is C88's shape, R-4.
- It needed `--grant plugin=`, the console capability held, and the WASM engine runs no plugins.

**Decision.** A local binding of the name decides the call, whatever it holds. The fast path is taken only when no local
binds the name (`env.get(name).is_none()`). The general path calls a closure, calls a plugin's export
(`call_plugin_export`, which re-checks its liveness as before), and refuses anything else as DL0907, *"value … is not
callable"*, the same words its last arm used. The module's functions, `load` and the builtins are reached only when no
local binds the name. No new code.

**Not taken:** keeping the closure-only check and adding `PluginFn` beside it. That would leave the next callable kind
to fall through the same way, which is the skip-branch shape (HANDOFF §11.4, C88).

**Evidence.** `plugin_load_cli` `a_local_holding_an_export_is_called_never_a_module_function_of_its_name`: the effect half
first (`why Write` says the program cannot write, and the run must print nothing), then the value half (`hello!`). Red
before: `hello?`. Mutants M981 (the fast path asks for a closure only) and M982 (the general path does) each left
`LEAK: the module's relay`, and the control was green. `delulu-runtime` (351), `plugin_load_cli` (25), `plugin_cli`,
`late_let_cli`, `examples_run`, `core_invariance`, `conformance`, `actors_cli`, `stdlib_p3`, `wasm_cli`, `for_loop_cli`
and `loop_parity` passed.

## D-V2-201 — CTOR-SHADOW-1 (RW 2.25): a name a constructor of the module and a function share is the constructor, on both engines — TAKEN (head chef, 2026-11-01, under the owner's delegation)

**Context.** A function may share its name with a constructor of the module's own sum type. DL0302 refuses only a
function named like a prelude builtin. The checker resolves such a name to the constructor, as a call (`let s: Shape =
Circle(3)` checks, and `let n: Int = Circle(3)` is DL0401) and as a value. The interpreter's order differed:
- for a call, `eval_call`'s two paths asked the module's functions before its capitalized-name fallback;
- for a value, `eval_var` asked the capitalized-name fallback before the functions.

Found 2026-11-01 by the laptop, reading the interpreter's name lookups against the checker's after LATE-LET-1 and
LOCAL-CALLEE-1. On the release binary of `cccb001`, with no special grant:
- `type Holder = Wrap(Cap[Console]) | Empty` with `fn Wrap(c) -> Holder ! {Write}` that prints, called from a pure
  `hold`: `authority` said *"(none — provably pure)"*, `why Write` said *"program cannot perform `Write`"*, and the run
  printed. This is C88's shape, R-4.
- `Circle(3)` was the function's `Int`, and a `match` on it as a `Shape` faulted.
- `apply(Double, 5)`, with no constructor `Double`, faulted *"value `Double` is not callable"* (DL0907).
- The WASM engine built the constructor for a call with an expected type (`Wrap(c)` as `hold`'s result), but called
  the function for one without (`let h = Wrap(c)`), and printed. It refused `let d = Dot` (DL1201: its `let` bound the
  bare name to the function value).
- And the WASM engine compiled the prelude's builtins (`str`, `int`, `float`, `parse_int`, `parse_float`, `Map`) before
  it asked for a local function value, so a local `str` was never called there (WASM-BUILTIN-SHADOW-1, RW 2.27).
- v1.0.0's interpreter had the same order.

**Decision.**
1. **The interpreter keeps the module's constructors** (`Home::ctors`, from its sum types' variants). It builds a
   constructor before it asks the functions: in `eval_var`, in the A14 fast call path (a function that shares a
   constructor's name is never taken there), and in the general call path. The capitalized-name fallback, which
   serves a prelude's constructors (`None`, `NotFound`), now comes after the functions in `eval_var`, as it already did
   for a call. A prelude constructor cannot be a function (DL0302), so the fallback never meets one.
2. **The WASM engine never binds a constructor as a function value** (`EnumEnv::is_ctor`, asked over every sum type,
   since `enum_of_ctor` answers nothing for a name two types share). **Its call asks in the checker's order:** a local
   function value first, then the builtins, then a constructor of the module (built even where a generic or a function
   shares its name, and refused in words where two of the module's types share it, never falling through to the
   function), then the generics and the functions.
3. **Not taken:** refusing a function named like one of the module's constructors (DL0302 widened). That would refuse
   programs that check today, which STABILITY.md §1 forbids within a major and asks a decision of its own for 2.0. The
   runtime now gives the meaning the checker already gives, so no program changes verdict.

**Evidence.** `late_let_cli`'s `a_call_the_checker_resolved_to_a_constructor_never_runs_a_function_of_its_name` (red
before: the `LEAK` line). `loop_parity`'s
`a_constructor_is_built_where_a_function_shares_its_name_and_a_function_is_passed_where_none_does` holds both engines to
`3 -1 10`; red before: the WASM engine refused the program, and the interpreter faulted. Mutants M983 (the general path
asks the functions first), M984 (the fast path takes a function that shares a constructor's name), M985 (a value asks
the functions first), M986 (a capitalized value is a constructor before a function), M987 (the WASM `let` binds a
constructor as a function value), M988 (the WASM call asks a local after the builtins) and M989 (the WASM call takes a
function that shares a constructor's name) were all red, and the control green. The CLI test runs both engines, and
`loop_parity`'s `a_local_named_like_a_builtin_is_called_on_both_engines` holds WASM-BUILTIN-SHADOW-1. `delulu-runtime` (351), `delulu-wasm` (217),
`late_let_cli`, `plugin_load_cli`, `examples_run`, `core_invariance`, `conformance`, `actors_cli`, `stdlib_p3`,
`wasm_cli`, `book`, `corpus_cli` and `deep_values_cli` passed.

## D-V2-202 — the interpreter's maps from a program's names hash without a key (RW 3.24) — TAKEN (head chef, 2026-11-01, under the owner's delegation)

**Context.** RW 3.24's profile (callgrind, `a4da001`, on-demand routine run 16): a call by name to one of the module's
functions asked `Home::funcs` and `Home::ctors`, a `HashMap` and a `HashSet` with the standard library's SipHash-1-3 under
a per-process random key, and spent about 120 of a call's 1,800 instructions there in `nested_calls_macro` (73 million over 600,000 calls). The same hasher
served a record type's field places (`Home::records`) and a scope's index past sixteen bindings (`Vars::index`).

**Decision.** Those four maps hash with `NameHasher` (`crates/delulu-runtime/src/value.rs`): the multiply-rotate hash of
rustc's own name tables, a word at a time, with no key, the same on every run. It is twenty lines and no dependency.

**Why it is safe.** SipHash's key resists an adversary who chooses keys so that they collide. The keys of these four maps
are names the program itself declares: its functions, its sum types' constructors, its record types' fields, its own
`let`s. They are fixed when it is checked, and no input the program reads at run time becomes one. A program that
collided its own names would slow only its own run, which its processor budget bounds: 5 minutes unless `--limits` sets
another, sampled every 25 ms on every engine (PS-B-01, D-V2-25). Its author could as well write a slow loop. A plugin's
nested interpreter holds the plugin's names, and a plugin that collided them would make its own steps slower: its fuel
counts steps, not time, so that is a slower plugin, and the run's processor budget still bounds the run. None of these
maps is iterated to produce anything (read 2026-11-01: each is only looked up), so a fixed order is not observable.
Maps whose keys can come from outside the program — the broker's, the egress client's, the actors' — keep SipHash.

**Alternatives.** Keep SipHash: the measured cost. A crate (`rustc-hash`, `ahash`): a dependency for twenty lines.
Indices resolved before the run: the larger change RW 3.24 still names, which would retire these lookups altogether.

**Evidence.** With run 16's other cuts (`V2_LOG.md`, RW 3.24), `nested_calls_macro` −15.9% in instructions and −19% in time;
the full suite green. The hasher's own share was measured with the scope sizing and the call order together: those three
took `nested_calls_macro` from 1,449 to 1,371 million instructions.

## D-V2-203 — RW 2.26: a pattern binding spelled like a constructor of the matched type is refused (DL0302) — TAKEN (head chef, 2026-11-01, under the owner's delegation)

**Context.** The checker admits lowercase constructors (`type T = foo | bar`), and since D-V2-201 both engines build
them as values. In a pattern the parser reads a lowercase name as a fresh binding and a capitalized one as a constructor
(`parse_pattern_inner`). So `match t { foo => 1, bar => 2 }` checked clean and answered 1 for `bar`, on both engines:
the first arm bound every `T` and the second never ran. That is the language's defined meaning, not an engine defect,
but every reader takes those arms for the constructors. The call form `foo()` already matched the constructor on both
engines (probed on the binary, 2026-11-01).

**Decision.** In a `match`, a binding spelled like a constructor of the type matched at that position is refused,
**DL0302**, at every depth: at the top, inside `Some(…)`, and inside another constructor's fields. The message names the
constructor and the type. The repair `match-the-constructor` (a suggestion, with edits) writes the call form, `foo()`,
or `cell(_)` with one `_` per field. A binding spelled like another type's constructor, and every other binding, check
as before.

**Why DL0302, not a new code.** DL0302's rule is that a second definition of one name is refused rather than shadowing
the first. A binding spelled like the matched type's own constructor would shadow it in the arm, and would take every
value. Its explanation now says so. The precedent is D-V2-179, which reused DL0211. No conformance entry or entrenched
file changes. A rejecting and an accepting program carry `ref.diag.DL0302` in their headers, and coverage stays 358 of
358.

**Why a refusal, not a warning.** Every program holding such a binding computes something other than what it reads as,
silently: its later arms never run. A warning can be scrolled past, and the fix is mechanical. Rust refuses the same
shape by default (E0170, `bindings_with_variant_name`). The refusal is also the other half of an existing one: a
capitalized name that is no constructor of the type is refused, DL0405.

**The alternatives.**
- (b) Refuse lowercase constructors at their declaration: it refuses every such type, its correct uses included, and
  the conformance witness of D-V2-201 holds lowercase constructors as values.
- (c) Read such a name in a pattern as the constructor, as Rust resolves it: it changes what programs that check today
  mean, without a word.
- A warning, option (a) as first written: above.

**Breaking.** A program that checked under 1.x is refused under 2.0.0 when it holds such a binding. Its 2.0.0 entry
goes into the *Breaking* paragraph when `launch/2.0.0` next takes `master`.

**Evidence.**
- Tests: `delulu-check`'s `a_binding_spelled_like_a_constructor_of_the_matched_type_is_refused`. Its four shapes each
  checked clean before: a nullary constructor, one with a field, one inside `Some`, and a generic sum type.
  `a_binding_that_names_no_constructor_of_the_matched_type_checks` covers what stays. Also the conformance pair,
  `loop_parity`'s call-form parity (`1 2 -1 9` on both engines), and `fix_cli`'s repair corpus.
- Mutants, each red:
  - M1004: the binding is never asked about. Unit tests and conformance went red.
  - M1005: only a nullary constructor counts.
  - M1006: the repair writes `()` whatever the fields.
  - M1007: a generic sum type is not asked.
- The core-invariance snapshot is blessed; it gained only the two new programs.

## D-V2-204 — a `match` arm no value reaches is reported, DL0413, a new warning — TAKEN (head chef, 2026-11-01, under the owner's delegation)

**Context.** Arms are tried in order and the first that matches wins, on both engines. So an arm whose every value an
earlier arm already matches never runs. Five such arms checked clean on the binary (`check` exit 0), and the run took
the earlier arms. Examples are `x => …` above `Red => …`, `Red` or `0` twice, `Some(_)` above `Some(Red)`, and a `_`
after `true` and `false`. RW 2.26 (D-V2-203) closed the commonest way to write one by accident, a binding spelled like a
constructor. The rest are an arm in the wrong order, or one left behind by an edit.

**Decision.**
- The checker asks each arm whether some value reaches it past the arms above. This is Maranget's usefulness, the
  question the exhaustiveness search (MATCH-EXHAUST-1) already asks of a row that holds any value, generalized to any
  row.
- An arm no value reaches is reported with **DL0413**, *unreachable match arm*, a **warning**, at the arm's pattern.
- It is asked only when every pattern of the `match` passed, with its own copy of the exhaustiveness search's limit
  (`EXHAUSTIVE_BUDGET`). It reports an arm only when it has proved it unreachable. A search that spends its limit
  stops asking, and says nothing more about that `match`.

**Why a warning.** The program does what its arms say; nothing escapes or faults. rustc, OCaml and GHC each warn on
the same shape. A refusal would make a dead arm in generated code, or one kept while editing, a build failure, for no
safety gain.

**Why a new code.** No existing code says it. DL0407 is the opposite claim (an arm missing). DL0502, the one typing
warning, is about effects. The owner's delegation of 2026-10-17 covers a new `DL` code where one is needed. Under
`STABILITY.md` §5 a new diagnostic code is a minor, additive change.

**What it touches.**
- `codes.rs`: the title and the explanation.
- `witnesses.toml`, entrenched (`ENTRENCHED_CHANGE_RECORD.md`), gains two rows:
  - the producing one is `an_arm_no_value_reaches_warns_dl0413`. A warning cannot be a `.delulu` reject file, which
    must be refused.
  - the accepting one is `an_arm_some_value_reaches_is_not_reported`.
- The generated reference: 359 of 359 anchors, 156 codes.
- The documents that quote the count of codes (156 now).

**Cost.** Each arm is asked against the arms above it, so a `match` of n arms reads about n²/2 cells, within the limit.
On a debug build, 6,000 literal arms check in 0.27 s against 0.06 s before (the limit is reached near arm 4,500), and
1,000 in 0.04 s. No program the repository ships has a dead arm: `core_invariance` is unchanged by the check.

**Evidence.**
- The two tests above, each red before.
- Mutants, each red:
  - M1008: the arms are never asked, the defect itself.
  - M1009: any literal covers any literal.
  - M1010: a constructor every arm covers is read as a value left.
  - M1011: a row's fields are read as any value.
- One mutant was equivalent: reading the constructors some arms name as all of them. For a finite set of constructors,
  trying each in turn, the missing ones included, gives the same answer as the default rows, at more cost. The
  `all(...)` that selects the default rows is a matter of cost, not correctness.
- Coverage is 100% (359/359), and the reference is in sync.
- These targets passed: `delulu-check` (262), `delulu-diag` (57), `conformance`, `core_invariance`, `corpus_cli`,
  `book`, `examples_run`, `json_contract`, `locale_cli`, `locale_plugin_cli` and `cli`.

## D-V2-205 — INDEX-TARGET-1 (RW 2.29): what an assignment's index holds is checked, asked for and judged as any other expression — TAKEN (head chef, 2026-11-02, under the owner's delegation)

**Context.** `xs[e] = v` evaluates `e` when it runs: the interpreter's `assign` evaluates `v`, then walks the target and
evaluates each index, the innermost first. Every walk of `Stmt::Assign` in the repository read only `v`. Found 2026-11-02
by the laptop while writing DL0414 (a binding never read, `laptop/unread-lets`), whose first draft named `i` in `xs[i] = 9`
unread, because the checker never looked inside the index. Each shape below was witnessed on `master` `9b3be30`:
- **The checker** (`check_lvalue`) never checked `e`. An effect there joined no row: `fn quiet(c: Cap[Console]) -> Int {
  var xs = [1, 2, 3]; xs[noisy(c)] = 9; xs[0] }` checked clean, `delulu authority` listed `quiet` under *pure fns*, and
  `run --grant console` printed. §D's claim of `SOUNDNESS_AUDIT.md` negated a fourth time, by the checker this time: step 2
  assumes the checker visits every call the runtime makes. A name there was never resolved (`xs[no_such_name] = 9`
  checked clean and faulted DL0907 when it ran), a `Str` index passed, and a base that is no list got a fresh type.
- **The reference-capability pass** never walked `e`, nor did its loop pre-scan: `ys[give(s, consume xs)] = 1` followed by
  `xs.len()` was accepted — the unique list already sent to an actor, read after. Bound one line up, the same call is
  DL1602.
- **`authority`, `--grants` and `--within`** never saw a mint, a file use, a request or a send there.
  `xs[wipe(root.fs_write("./ix4-out"))] = 9` was *"--grant console"* alone, and `--within console` answered *within*.
  A write of `../escape.txt` through a capability for `./out` was *exceeds* (DL0904) one line up and *within* in an index.
  The walk listing what a loop assigns missed a secret bound again in an index, so a send on the loop's second turn kept
  its first secret's attribution.
- **The foreign report and the atlas** never saw `py.import("socket")` in an index: *"imports seen: (none seen
  statically)"*, no `foreign:py:socket` edge, *"--grant foreign.python=PATTERN"*, and DL0502 told the author the
  declared `ForeignCall` was never performed. Through a call in the index, `foreign:` said *"(none — no code outside the
  guarantee)"* for a program that embeds Python.
- **A plugin `load(…)`** in an index was never listed among the program's plugins.
- **The LSP** hovered nothing in an index; the two WASM walks that collect strings and test a module skipped it too, which
  matters to nothing yet because the WASM engine refuses an index target (`compile_assign`).
The capability check held throughout: a mint the grants did not cover still refused when it ran.

**Decision.**
1. `LValue::index_exprs` (`delulu-syntax`) names a target's indices in the order they run. Every walk of `Stmt::Assign`
   walks them after the value: the checker, the reference-capability pass and its loop pre-scan, `--within`'s two walks,
   the walk behind `authority`'s requirements, the plugin-load walk, the Python and compute walk, the atlas's foreign
   reach, the LSP's expression walk, and the two WASM walks.
2. **The checker checks an index as it checks `xs[i]` read**: an `Int`, its row the statement's, its names resolved, and a
   base that is no list DL0405, *"cannot index a value of type …"* (a poisoned base says nothing more). No new code.
3. **Programs refused now that checked before** are each a program whose run faulted, lied or raced: an undeclared effect
   in an index (DL0501), an unknown name (DL0301, a DL0907 fault when it ran), a non-`Int` index (DL0401, DL0907 when it
   ran), a use after a `consume` in an index (DL1602), an index into a value that is no list (DL0405; the
   reference-capability pass already refused most as DL1604). None is in the programs the repository ships.
4. **Not taken:** refusing a call or a `consume` inside an index, which would change the language to excuse the walks.

**Residual.** The constructor walk that lists the fields `self.f = …` assigns (`collect_self_field_assigns`) reads no
expression of a `let` or an assignment, the index among them, so a field assigned only inside a block in an index is
reported unassigned (DL0405): a refusal, the safe direction, and unchanged.

**Evidence.** `index_target_cli` (7 tests, the real binary: the row, the index's own checks, the consume with its loop,
the mint, the file use against its control one line up, the foreign import across `check`, `authority`, `--grants` and
the atlas, the plugin), `within_cli` `a_secret_bound_again_inside_an_index_in_a_loop_is_not_attributed_to_its_first_value`,
and `lsp_cli` `hover_shows_the_type_of_a_name_in_an_index_target`; conformance reject programs
`DL0501_effect_in_an_index_target` and `DL1602_consume_in_an_index_target`. Each was red on `9b3be30`.
Mutants, each red on its own assertion: M1018 (the index's row dropped), M1019 (no DL0405 for a base that is no list), M1020 (the index never checked), M1021 (the reference-capability walk skips the index), M1022 (its loop pre-scan does), M1023 (`--within`'s send walk does), M1024 (its assigned-names walk does), M1025 (the requirements walk does), M1026 (the Python and compute walk does), M1027 (the plugin-load walk does), M1028 (the atlas's foreign reach does), M1029 (the LSP's walk does) and M1030 (`index_exprs` names nothing). The suites: `delulu-syntax`'s library (the formatter's corpus law over the two new programs), `delulu-check`, `delulu-atlas` and `delulu-wasm` whole; `delulu`'s `conformance`, `core_invariance` (the two new programs added, no existing answer changed), `index_target_cli`, `within_cli`, `lsp_cli`, `plugin_load_cli`, `atlas_cli`, `atlas_e2e`, `cli`, `examples_run` and `late_let_cli`.
