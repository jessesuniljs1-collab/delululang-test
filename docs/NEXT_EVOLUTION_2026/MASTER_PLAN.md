# DeluluLang — Next Evolution 2026: the master plan

**Written:** 2026-09-17, by Claude Fable 5.1 as head chef, on the owner's commission
([`OWNER_COMMISSION.md`](OWNER_COMMISSION.md), a redacted copy; `EXECUTION_LOG.md` §Housekeeping
says where the original lives and why).
**Status:** **PLAN COMPLETE — AWAITING THE OWNER'S APPROVAL.** No implementation has started.
The only tree changes made by this pass are this folder, one line in
`docs/REPOSITORY_STRUCTURE.md` §5 accounting for it, and the regenerated Survey.
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

**Recommended order:** P1 → P2 → P4a (skill) → P3 → P4b–d → P6 → P5 → P7 → P8. P1 first
because every later phase's agents will use the contract; P2 second because it is the identity's
own demo; the skill early because it is cheap and makes every subsequent agent session cheaper.

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
7. Where the owner's commission file lives (see `EXECUTION_LOG.md` §"Housekeeping"): it quotes the
   banned word, so it cannot be committed as written and cannot stay in the walked tree either.
8. The four pre-public-repository decisions (unchanged; nothing here touches them).

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
