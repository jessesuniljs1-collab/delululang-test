# DeluluLang — Next Evolution 2026: the master plan

**Written:** 2026-09-17, by Claude Fable 5.1 as head chef, on the owner's commission
([`OWNER_COMMISSION.md`](OWNER_COMMISSION.md), a redacted copy; `EXECUTION_LOG.md` §Housekeeping
says where the original lives and why).
**Status:** **PLAN COMPLETE — AWAITING THE OWNER'S APPROVAL.** No implementation has started.
**Revised the same day by a second, owner-commissioned research pass on sandboxing and VM isolation:
§12 carries its decision and supersedes §6's order; the sandbox documents listed in §12 are
supporting documents of this plan, not a second roadmap.** The only tree changes made by these two
passes are this folder, the accounting lines in `docs/REPOSITORY_STRUCTURE.md` §5, a dispatch-only
CI probe workflow, one owner-rule line in `HANDOFF.md` §11.1, and the regenerated Survey.
**Companions:** [`RESEARCH.md`](RESEARCH.md) (what the field does),
[`VERIFICATION_FINDINGS.md`](VERIFICATION_FINDINGS.md) (what the binary does),
[`IMPLEMENTATION_ROADMAP.md`](IMPLEMENTATION_ROADMAP.md) (the phases),
[`DECISION_LOG.md`](DECISION_LOG.md) (why), [`DOCUMENTATION_AUDIT.md`](DOCUMENTATION_AUDIT.md)
(every document classified), [`EXECUTION_LOG.md`](EXECUTION_LOG.md) (what was actually done).

**Precedence used throughout:** code/tests/current binary > README/HANDOFF > other current docs >
historical docs > assumptions. Every claim about the current state was executed on 2026-09-17.

---

## 1. What DeluluLang is today — verified, not remembered

A statically typed language in which **every function's authority and effects are in its type**,
with a compiler that answers *what can this program do to my system* before it runs. The Rust
workspace is 13 crates and 111,437 lines (Survey, 2026-09-17), CI green on macOS, Linux (x64 and
arm64) and Windows in one run since 2026-09-14 and again on the public runners on 2026-09-17.

What exists and was **executed in this pass** (`VERIFICATION_FINDINGS.md` §2):

- **The core:** effect rows, capabilities as values, `delulu authority` / `why`, typed repairs with
  the widening rule honoured by both `fix` and the language server, deterministic replay, the
  nesting bounds, the JSON refusal envelope, the tree-walking interpreter, the WASM *fragment* that
  refuses rather than falls back, `.dwx` artifacts, actors, `for`/`break`/`continue`, tests under
  a ceiling, packages with computed authority pins and a lockfile.
- **Custody:** the daemon broker — delegate a lease, run under it, a second redemption refused,
  a wider program refused, transitive revocation with the latency bound stated, a verified audit
  chain that recorded even the tester's own malformed token. This is the multi-agent story and it
  works end to end.
- **The map and the health check:** the Survey (1,121 nodes, 10,018 edges, every hop cited) and
  `doctor` with a security-posture section.
- **The evidence culture:** conformance coverage at 100% per commit, the core-invariance snapshot,
  the evidence gate over every markdown file, model-checked custody, a machine-checked fragment of
  the effect calculus, `cargo deny`, Miri where it can reach.

What does **not** exist, verified against the binary (the three `README.md` names, plus what this
pass found): the microVM layer (a probe that always refuses), a standard library beyond four list
and six string methods, a native backend — and, **not previously on the gap list, a program cannot
load a plugin at run time** (`NE-01`: `root.plugin_host()` is a runtime stub that faults under a
grant-refusal code). Nothing is distributed. No physical device has ever been commanded.
Certification is none.

## 2. Where it drifted

The original commission (`docs/design/DeluluLang_PROMPT.md`, 2026-07) named the primary users —
AI agents, LLMs, future AI, physical AI and robots, with humans reviewing — and named three proofs
of the identity: plugins loaded at run time and still bounded, actuators as capabilities, and a
machine-first diagnostics loop. Measured against that, five drifts are real:

1. **Verification consumed the calendar; usability did not move.** Since `v1.0.0` (2026-07-20)
   there have been 202 commits, and almost all of them are adversarial review, proof work,
   documentation honesty and CI. That work was right and it found genuine defects (the escapable
   effect row, the dangling-symlink escape, the guard seal that gated nothing). But in the same two
   months the standard library stayed at four list methods, the plugin load surface stayed a stub,
   and an agent writing its first ordinary program still hits a reference-capability wall with no
   repair (`NE-02`), a guide example that silently fails (`NE-03`), and a grant grammar it cannot
   discover (`NE-10`). The trust is earned; the *reach* is not.

2. **The flagship demo does not run.** The Book, the plugins guide and the example README describe a
   running program loading a plugin it has never seen. The checker types it, `authority` reports it,
   `plugin build/verify/inspect` work — and `delulu run` refuses with a stub message. The Stage 6
   specification's status log says so in one clause; `REMAINING_WORK.md`, which calls itself the
   single gap list, does not. For a language whose identity is "code that arrives after compile
   time still cannot overreach", this is the drift that matters most.

3. **The machine contract is uneven where the identity is machine-first.** `for-agents.md`
   promises one envelope; `why`, `add`, `plugin inspect` and `test` break it (`NE-05`); `explain`
   has no machine channel and swallows `--json` (`NE-06`); a repair says "apply me" and carries no
   edit (`NE-07`); a single defect produces 145 diagnostics (`NE-04`). None of these is a soundness
   hole. All of them are exactly what an agent trips on.

4. **The knowledge is written for a human maintainer, not packaged for an agent.** 156 markdown
   documents, 43,418 lines; `HANDOFF.md` alone is 1,149 lines. The field converged on two small
   things — a skill an agent loads on demand and an MCP server it can call — and DeluluLang has
   neither, while having stronger underlying facts than any tool surveyed (`RESEARCH.md` §5).

5. **Nothing is installable.** Build from source, or an archive produced by hand. The morphs the
   docs say "ship" do not ship (`NE-11`). There is no release automation, no attestations, no
   installer, no upgrade story. A "language of the future" that a normal person cannot install is a
   research artefact.

Not drift, but must be said: the **authority model itself did not drift.** No holder-kind branch
was found anywhere; the no-discrimination principle holds in the code that issues, attenuates and
revokes (Stage-5 criterion 9). The plan protects that.

## 3. What the AI-first goal actually means, restated as engineering

Restating the owner's goal in terms a roadmap can be measured against:

> **DeluluLang is the language an agent writes when a human — or a supervising agent — must be
> able to know, before running it, exactly what it can do; and the toolchain is the set of
> deterministic, machine-shaped instruments that make writing, checking, reviewing, running and
> supervising that code cheaper for an agent than any alternative.**

That yields seven measurable properties, each mapped to the roadmap:

| Property | What it means concretely | Today | Roadmap |
|---|---|---|---|
| **Agent-native language surface** | a first ordinary program compiles first time given the skill; walls come with repairs | walls without repairs (`NE-02`), silent failures (`NE-03`) | P1, P3 |
| **Authority and capability safety** | zero ambient authority, attenuation only, revocation transitive, Guard supervises | **holds** (`NE-16`) | protect; P2 extends it to run-time loading |
| **Machine-readable contracts** | one envelope, stable codes, typed repairs that are applicable when they say they are | uneven (`NE-05…08`) | P1 |
| **Interoperability** | a skill any harness loads; an MCP server any client calls; a manifest that states what the toolchain can do | none | P4 |
| **Deterministic, verifiable execution** | replay, trace ⊆ row, artifacts hash-bound | **holds** | protect |
| **Program and repository understanding** | Atlas for programs, Survey for the repo, both with provenance; a diff → blast radius | Atlas drops requested scopes (`NE-14`); no diff-to-impact | P1, P4 |
| **Safe autonomy and physical authority** | envelopes, dead-man, e-stop, sim-to-hardware gate, signed adapters | simulator only; adapter unsigned | P7 (owner-gated) |
| **Reproducible distribution** | download, verify, run in five minutes on three OSes | none | P5 (owner-gated) |
| **Human review of AI-written code** | `authority`, `why`, exposure join, Trojan-Source refusals; required grants stated | required grants unstated (`NE-10`) | P1 |

## 4. What DeluluLang already does better than anything surveyed

Stated carefully, each verified in this pass or by an existing gate:

- **A compiler-computed authority answer with a proof path**, at function granularity, across
  dependencies and (statically) across plugins. No surveyed language or platform has this;
  ZeroLang has a coarse `World` handle, Deno a process-level flag, Cloudflare OS a platform-level
  capability broker.
- **The custody tree**: delegate → single-use lease → revoke-transitively → audit, exercised end to
  end here, model-checked, and holder-kind-blind by test.
- **Repairs that know whether they widen authority**, and a `fix` that refuses to apply one unless
  named. Every surveyed diagnostics design stops at "suggest".
- **A byte-deterministic machine channel** pinned by a snapshot, and a repository map that cannot
  cite what it cannot point at. The code-intelligence tools surveyed tag inferred edges; the Survey
  refuses them.
- **An honesty discipline enforced by tests**, not by review: evidence claims, install-path
  promises, banned governance phrases, JSON one-object, help/dispatch/completions in sync.

## 5. Major gaps, ranked by how much they block the goal

1. Runtime plugin loading is a stub (`NE-01`) — blocks the identity's own demo and the
   "agent extends a running system" story.
2. Machine-contract unevenness (`NE-04…NE-10`, `NE-13`) — blocks reliable agent loops.
3. Standard library breadth (`REMAINING_WORK.md` 2.1) — blocks ordinary programs.
4. No skill, no MCP, no toolchain manifest, no checked edit — blocks interoperability.
5. No distribution — blocks everyone else.
6. Documentation volume without an agent-sized entry — raises every session's cost.
7. Cheap verification items already on the list (KAT vectors, fuzz targets, Miri shrink, Progress
   restatement) — raise trust at low cost.
8. Owner-gated: physical device, signed adapter, microVM, final public repository.

## 6. What should be implemented — and what should not

**Implement (in roadmap order; full detail in `IMPLEMENTATION_ROADMAP.md`):**

- **P1 — Machine contract truth** (small changes, high value, no semantics change): the
  success-envelope sweep and the five `why` emitters; `explain --json`; the DL0210 cascade;
  repairs-without-edits; `test --json` diagnostics; `test` default target; `authority --grants`
  plus `requested_scopes`; the DL1603 message/explanation/repair; the guide path fix and rule; the
  REPOSITORY_STRUCTURE gate; the `NE-01` row in `REMAINING_WORK.md`.
- **P2 — Plugins for real**: wire `root.plugin_host()` and `load` into the CLI runtime through the
  existing `PluginEngine` seam, following the Stage 6 load sequence exactly (ceiling, holder check
  at the broker, class-specific verification, signature policy, instantiate), with a grant dimension
  the operator controls, unload/revocation, the Windows refusal for Contained execution kept, and a
  shipped, gated `examples/plugin_host/`.
- **P3 — Standard library, additively**: `filter`, `fold`, `find`, `contains`, `sort`, `reverse`,
  `concat`, `is_empty`, `pop`, `slice`, `join` on lists; `to_upper`, `to_lower`, `replace`, `join`
  on strings; a `Map[K, V]` prelude type — each with a prim-table row, interpreter, WASM lowering
  or an honest `DL1201`, two conformance witnesses, R-4 row carriage for the higher-order ones,
  and generator coverage in `delulu-fuzz` (the C88 lesson).
- **P4 — Agent surfaces**: an Agent-Skills-standard skill; `delulu mcp` (stateless, read-only,
  deterministic tool order); `delulu toolchain --json` (commands, grant grammar, effects, prim
  table, limits — generated from the binary's tables); **checked edits**: `delulu edit` with a
  content-hash stale-state guard and one envelope (diagnostics + new hash), then node-addressed
  edits by Atlas id; `survey diff` (git diff → affected nodes); a read-only Guard/broker view for
  the editor.
- **P5 — Distribution** (owner-gated on channel): a tag-triggered release workflow building the
  portable archive for four targets, `SHA256SUMS`, GitHub artifact attestations, `morphs/` and the
  skill in the archive, installer scripts if the owner accepts the posture, a documented upgrade
  path (download the next archive; editions already cover language compatibility).
- **P6 — Documentation consolidation**: execute `DOCUMENTATION_AUDIT.md` — move historical and
  superseded material under `docs/archive/` preserving paths, update every link, regenerate the
  Survey; shrink `HANDOFF.md` to a current briefing; keep release snapshots untouched.
- **P7 — Verification depth (cheap items)**: KAT vectors test, `cargo-fuzz` targets, Miri
  shrinking, Progress restated as progress-or-fault.
- **P8 — Safe autonomy (owner-gated)**: the signed Verified-class adapter (no hardware needed);
  everything else waits on hardware, a KVM host or the final repository.

**Do not implement (with the reason; full records in `DECISION_LOG.md`):**

- A graph-as-program-database (ZeroLang's model). Text stays authoritative; adopt the guards only.
- LLM-evaluated conditions or any non-deterministic construct inside the language (Pel).
- Inferred edges in the Survey or Atlas (the KG skill's `INFERRED`, "semantically related").
- A package-manager ecosystem or `cargo install` from crates.io (already ruled; still right).
- The native backend and the DIR optimizer (D18 stands; no evidence the passes pay off).
- Any change to what Authority or the Guard *mean*: no holder-kind branch, no default flip of
  strict anchored-root mode before a major version, no auto-applied widening, no "helpfully find
  the binary" convenience in the editor.
- Token-count or performance superlatives on any surface.
- Rewriting pushed history, or touching the four owner-gated pre-public items.

## 7. Implications, by concern

- **Security.** P2 activates a code-loading path in the CLI runtime; it must follow the specified
  load order, keep the Windows Contained refusal, reuse `plugin verify`'s verdicts (criterion 9),
  bind the grant to a broker node so revocation cascades, and be red-teamed with the search key
  that found four 2026-08-10 defects: *what else spells the same thing?* (a plugin path, a grant
  scope). P4's MCP server is analysis-only by construction, like the LSP; it never runs, grants or
  loads. P5's installers, if accepted, must verify checksums and never be the only path.
- **Compiler and runtime.** P1's cascade fix touches parser recovery and the checker's treatment of
  recovered nodes — the core-invariance snapshot is regenerated deliberately and the diff reviewed.
  P3 grows the prim table (checker, interpreter, WASM); every addition is additive and witnessed.
  P2 touches `prim.rs`, `interp.rs`, `run_cmd.rs` and the broker's holder check — the Survey's
  `impact` for those modules is the blast radius and is recorded per task.
- **Agents.** P1 and P4 are the agent phases; the measure is the cost of an edit→check→repair loop
  and the number of walls without repairs (target: zero in the guide corpus).
- **Distribution.** P5 changes nothing in the language; it changes who can run it. The archive
  format stays; automation, attestations and shipped morphs/skill are added.
- **Documentation.** Every phase moves its docs with its code (house rule 6). P6 reorganizes the
  tree without deleting history; `REMAINING_WORK.md` gains the `NE-01` row in P1.
- **Testing and verification.** Every new gate is falsified (introduce the defect, watch it fail);
  every fix has a witness against the pre-fix binary; the Survey is regenerated before the suite;
  `doctor` runs after; declared CI gates outside `cargo test` are run explicitly.

## 8. Effort, order, and what can start now

Estimates are engineering judgement, not measurements, and are given as ranges of focused
sessions (S = one session of a few hours of head-chef work with verification included).

| Phase | Effort | Can start after approval | Needs an owner decision first |
|---|---|---|---|
| P1 Machine contract truth | 3–5 S | yes | only the snapshot regeneration review (D-NE-3) |
| P2 Plugins for real | 6–10 S | yes | the grant grammar for loading (D-NE-10) |
| P3 Standard library | 5–8 S | yes | `Map` representation choice is a ruling in the build order, not an owner decision |
| P4 Agent surfaces | 5–8 S | yes | MCP in the CLI binary (D-NE-6); skill folder name (D-NE-5) |
| P5 Distribution | 3–5 S | partly | release channel and installer posture (D-NE-7, D-NE-8) |
| P6 Docs consolidation | 2–3 S | yes | archive folder name (D-NE-11) |
| P7 Verification depth | 3–4 S | yes | none |
| P8 Safe autonomy | 3–5 S for the adapter | the adapter only | hardware, KVM host, final repository |

**Recommended order (superseded by §12.3 after the sandbox pass):** P1 (+PS-0) → PS-A → P2 →
P4a (skill) → P3 → PS-B → P4b–d → PS-C → P6 → P5 → P7 → PS-D → P8. P1 first because every
later phase's agents will use the contract; the L1 sandbox next because it makes every later
"untrusted code" sentence true on the developer's own machine; P2 third because it is the
identity's own demo and its code does not block on PS-A; the skill early because it is cheap and
makes every subsequent agent session cheaper. The sandbox phases (PS-0…PS-D) and their effort are
in `SANDBOX_IMPLEMENTATION_PLAN.md`.

## 9. What requires the owner, listed once

1. Approval of this plan and its order (or a different order).
2. The distribution channel: whether pre-release archives may be attached to the testing
   repository's GitHub Releases at all, given the standing rule that it is not a distribution
   channel — or whether releases wait for the final public repository.
3. Installer posture: `curl | sh`-style scripts (convenient, and the field's norm) versus
   download-and-verify only.
4. The loading grant grammar (`--grant plugin=<path>` or a manifest `[plugins]` ceiling).
5. Constitution §5.15 guarantee 5 wording (entrenched; `REMAINING_WORK.md` 7.10a) — still pending.
6. rustfmt adoption and `CODE_OF_CONDUCT.md` — still pending from `HANDOFF.md` §8.
7. ~~Where the owner's commission file lives~~ — settled by the owner the same afternoon: both
   commission files live in `docs/design`, the first with one banned word redacted in place
   (`DECISION_LOG.md` D-NE-20).
8. The four pre-public-repository decisions (unchanged; nothing here touches them).
9. **From the sandbox pass (§12.2, question 12):** the interpreter-in-the-guest deviation from
   Stage 5 §6, the profile names and the strengthening of the `process` label, `Secret.map` under
   strong profiles, the two sandboxing crates, distributing a built (GPL) kernel image, the first
   network client's TLS dependency and the special-address spelling, resource-limit defaults, and
   whether `--sandbox` ever becomes the default.

## 10. What must remain deferred

The microVM layer (needs a Linux+KVM host and a guest launch path), the native backend and
optimizer (D18), principal types (F4), `Secret[Bool]` for `verify` (RFC), multi-tenancy (separate
OS accounts remain the answer), federation model-checking (a TLA+ model, valuable, not blocking),
a real device (hardware), certification (external), and the final public repository (owner's step).

## 11. How to read the rest

- Want the evidence? `VERIFICATION_FINDINGS.md`.
- Want the field? `RESEARCH.md`.
- Want the tasks, dependencies and acceptance criteria? `IMPLEMENTATION_ROADMAP.md`.
- Want to argue with a choice? `DECISION_LOG.md` — each record has evidence, assumptions,
  alternatives, confidence and what still needs verifying.
- Want to know which document is current, normative, historical or generated?
  `DOCUMENTATION_AUDIT.md`.
- Want to know what this session actually ran? `EXECUTION_LOG.md`.
- Want the execution layer — sandboxing, isolation levels, the microVM — and what it changes
  about all of the above? §12, and the five `SANDBOX_*` documents it names.

---

## 12. The execution layer — what sandboxing changes about this plan (second pass, 2026-09-17)

**Commission:** `docs/design/DeluluLang_Sandbox_VM_Integrated_Next_Evolution_Prompt.md`. **Evidence:**
[`SANDBOX_RESEARCH.md`](SANDBOX_RESEARCH.md) (the field, and a CI probe of what the runners
offer), [`VERIFICATION_FINDINGS.md`](VERIFICATION_FINDINGS.md) §4 (the binary), and the two
sous-chef records under `agent-notes` (verified before use). **Design:**
[`SANDBOX_ARCHITECTURE.md`](SANDBOX_ARCHITECTURE.md), [`SANDBOX_THREAT_MODEL.md`](SANDBOX_THREAT_MODEL.md),
[`SANDBOX_TEST_PLAN.md`](SANDBOX_TEST_PLAN.md), [`SANDBOX_IMPLEMENTATION_PLAN.md`](SANDBOX_IMPLEMENTATION_PLAN.md).
**Status: proposed; nothing built; §12.3's order supersedes §6's and §8's.**

### 12.1 The decision

**Sandboxing is a cross-cutting architecture beneath the runtime, plugins, agents and autonomy,
introduced early** — its truth-and-probe half inside P1, its minimum useful form (L1, the process
jail with the effect channel) as the phase immediately after P1 — **and the microVM is no longer
"deferred": it becomes an engineering phase gated on Linux + KVM**, which the public CI runners now
measurably have, while microVMs on Windows and macOS stay deferred with named triggers. This is
the combination the commission listed as option 4 (an early phase, a later phase, and a
cross-cutting layer), reached from four pieces of evidence rather than assumed:

1. **The field converged on enforcing the boundary outside the guest** (every cloud sandbox, both
   local agent tools, the evaluation frameworks) and on stating what the boundary does not cover.
   DeluluLang already enforces *authority* outside the program's code path (the primitive table,
   the broker); it has no *OS or hardware* boundary for the verified program at all.
2. **The binary today:** `--isolation process` isolates foreign code only; the main program has no
   CPU, memory, wall-clock or disk bound on either engine; `http.get` has no client behind it; a
   hung foreign worker hangs the host; Windows device names and trailing characters slip past
   containment (all verified this pass). A sandbox layer does not fix these — they are runtime
   hardenings, scheduled into P1 — but each is a reason the layer must exist.
3. **The same-OS-user residual (category 7)** is closed only by identity separation or an
   out-of-band key. A launcher that runs the guest under a restricted token, an AppContainer, a
   mapped uid or a VM *is* identity separation, provided by the toolchain instead of the operator.
   Sandboxing is how `DEPLOYMENT.md`'s Tier 2 stops being an instruction and becomes a mechanism.
4. **CI reality, measured:** `ubuntu-latest` has `/dev/kvm` and Landlock ABI 7 but blocks
   unprivileged user namespaces; `macos-latest` has no hypervisor support; `windows-latest` has the
   Windows Hypervisor Platform enabled. So an L1 jail must not depend on user namespaces, an L2
   microVM can be exercised on Linux CI, and the honest Windows and macOS tier is L1.

The load-bearing design choice is that **the guest performs no effects**: it holds opaque handles
and asks the host over one bounded channel; the host runs today's custody, Guard, containment and
audit code and performs the effect. Adding a backend can therefore never re-encode authority, the
guest never learns a broker address, secrets never enter it, and the microVM becomes "the same
guest behind a hypervisor" rather than a second design. The Stage 5 §6 requirement that the guest
run the WASM engine is replaced by running the interpreter (a ruled deviation, not a semantics
change), because a tier that refuses most programs with `DL1201` is a tier in name.

### 12.2 The fifteen questions, answered

| # | Question | Answer |
|---|---|---|
| 1 | Does sandbox/VM change the previous roadmap? | **Yes.** PS-0 joins P1; PS-A (L1) becomes the phase after P1; P2 keeps its code order but its "untrusted plugin on your own machine" claim waits for PS-A; PS-B (limits, the first network client as an egress proxy, identity separation) precedes P4's remaining surfaces; PS-C (microVM) replaces P8's deferral; P5 gains the guest image and the probe; P8's control program runs in a guest. Effort grows by roughly 30–40 sessions. |
| 2 | Should microVM remain deferred? | **No, not as a category.** Its Linux/KVM implementation is PS-C, after L1, because it reuses the channel; its cross-platform forms (Hyperlight on WHP, libkrun or Virtualization.framework) stay deferred with triggers in `SANDBOX_IMPLEMENTATION_PLAN.md` §6. |
| 3 | Should sandboxing move earlier than P8? | **Yes.** PS-0 inside P1 now; PS-A immediately after P1; PS-B and PS-C interleaved with P3 and P4 as ordered in §12.3. |
| 4 | Cross-cutting for P2, P4, P8? | **Yes.** P2: the load sequence runs host-side for guests, Contained exports execute host-side, nothing plugin-specific is needed. P4: the MCP server stays analysis-only and gains a read-only probe tool; the skill teaches the profiles. P8: the adapter and the dead-man watchdog stay host-side (invariant 52 untouched); the control program is contained. |
| 5 | Does real plugin loading depend on sandboxing? | **Technically no** — the Stage 6 load sequence is host-side and can ship first. **For the claim, yes:** "code that arrives after compile time cannot overreach *on your own machine*" is true only with L1 under it. Order: P2's code may proceed in parallel; the sentence is published after PS-A's witnesses are green. |
| 6 | Does agent execution, MCP or tool execution need the sandbox first? | **MCP and the LSP do not** — analysis-only by construction, never launching a guest. **Agent-run programs do:** `delulu run` under a harness should have `--sandbox` before the skill tells agents to run generated code; the skill lands after PS-A or says the profile is not yet available. |
| 7 | Does distribution need to account for backend availability? | **Yes.** The archive ships the guest mode (the same binary); the L2 image is a separate, checksummed, attested artifact whose kernel is GPL (an owner decision); `sandbox probe` and `doctor` report what the installed host can do; no installer may imply L2 where KVM is absent. |
| 8 | Which sandbox features belong in the core language? | **None.** No syntax, no semantics change; the effect row is already the tool list. A `[sandbox]` table in `delulu.toml` is package metadata, bounded by `[authority]`. |
| 9 | Which belong in the runtime? | The `EffectSink` seam (local versus channel), the guest mode, capability handles, resource budgets, the device-name and trailing-character refusals, the worker read deadline, the empty-environment rule for children. |
| 10 | Which belong in the host/launcher? | Policy derivation, host-capability probes, the launchers (process, microvm, external), the channel server and host effect proxy, the egress proxy, audit lifecycle records, the `sandbox` subcommands, the `doctor` section, the `--json` fields. |
| 11 | Which belong in deployment/cloud infrastructure? | L3 launchers (Docker with gVisor, Kata, Kubernetes, cloud sandboxes), jailer uid provisioning, image hosting, the attestation relying party (L4), operator recipes for identity separation where the OS gives an unprivileged launcher none. |
| 12 | Which parts require owner decisions? | D-NE-23 (the interpreter in the guest — a Stage 5 §6 deviation), D-NE-24 (profile names; strengthening the `process` label), D-NE-25 (`Secret.map` under strong profiles), D-NE-26 (the `landlock` and `seccompiler` crates), D-NE-27 (distributing a built kernel), D-NE-28 (the first network client's TLS dependency and the special-address spelling), D-NE-31 (resource-limit defaults), D-NE-33 (whether `--sandbox` ever becomes the default). |
| 13 | Which parts can be implemented immediately after approval? | All of PS-0: the documentation truth, the `run --json` field, the DL1408 repair, `sandbox probe`, the `doctor` section, the four runtime hardenings, the CI experiments; then PS-A's channel protocol and fuzz target, the policy derivation, the Windows launcher (restricted token plus Job Object, no admin needed), the Linux launcher on Landlock 7 + seccomp + rlimits + a user cgroup, the macOS Seatbelt launcher. |
| 14 | Which parts require Linux/KVM/cloud infrastructure? | PS-C entirely (KVM on CI subject to PS-0-08's experiment; Linux to build the image; a jailer uid); PS-D's L3 and L4; anything multi-tenant. |
| 15 | What is the minimum useful secure sandbox before a full microVM? | **L1 as specified:** the guest holds no OS authority and reaches only the channel; Landlock + seccomp + rlimits + a user cgroup on Linux, a restricted token + Job Object (+ AppContainer) on Windows, Seatbelt on macOS; identity separation where the OS permits; hard resource limits; the host-side egress proxy as the only network path; generated boundary tests and mutant launchers green on all three CI runners, with the residuals (kernel bugs, side channels, hosts without the controls) named in `host_guarantees`. |

### 12.3 The revised order

`P1 (+PS-0)` → `PS-A` → `P2` → `P4-01 skill` → `P3` → `PS-B` → `P4-02…07` → `PS-C` → `P6` →
`P5` → `P7` → `PS-D` → `P8`. The owner may swap PS-A and P2; the plan's argument for PS-A first is
that it makes every later "untrusted code" sentence true on the developer's own machine, and that
P2's code does not block on it.

### 12.4 What does not change

Authority and the Guard mean what they meant. No holder-kind branch. No silent fallback: a profile
that cannot be honoured is refused, as `DL1408` already is. The WASM engine keeps its in-process
containment role. Certification stays none. Nothing here is claimed until its witness is green and
its mutant is red.
