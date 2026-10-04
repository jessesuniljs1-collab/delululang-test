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

## Owner decisions carried from V1, still open
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
