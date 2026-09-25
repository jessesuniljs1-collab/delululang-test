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
   and host configuration, and Landlock already holds T14 there where the kernel has it.

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

## Owner decisions carried from V1, still open
D-NE-3 (snapshot regeneration is a reviewed act — the diff is shown in each phase's log),
D-NE-6, D-NE-7, D-NE-8, D-NE-25, D-NE-27; the Constitution §5.15 wording (RW 7.10a); rustfmt and a
code of conduct; the four pre-public-repository items. Each is asked at the start of the phase that
needs it (`V2_MASTER_PLAN.md` §7).

**Corrected 2026-09-25 — five entries this list carried were already decided.** D-NE-17 was built in
P1-F on the owner's word; D-NE-24, D-NE-26 and **D-NE-31** were ruled by the owner in D-V2-25
(2026-09-18) — D-NE-31's defaults are **1 GiB of memory and 5 minutes of CPU, never unlimited, the
operator may change them**; D-NE-33 was superseded by D-V2-25 and then set by D-V2-26. The list was
written before those rulings and never walked back, so PS-B's own entries went on calling D-NE-31
"the owner's" while the answer sat 280 lines above them. Found while checking the earlier phases.
