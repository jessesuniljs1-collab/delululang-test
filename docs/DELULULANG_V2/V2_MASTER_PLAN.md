# DeluluLang V2 — master plan

**Status:** EXECUTING since 2026-09-17, on the owner's approval of the Next Evolution 2026 plan
(`docs/design/DeluluLang_V2_Execution_Master_Prompt.md`). This file is the maintained form of that
plan. Its evidence — the verified findings `NE-01…NE-22`, the field research, the sandbox research,
architecture, threat model and test plan, and the decision records `D-NE-01…D-NE-33` — is kept
unchanged in `docs/archive/v1/NEXT_EVOLUTION_2026/`. Decisions taken since are `D-V2-nn` in
`V2_DECISION_LOG.md`. Phase state is `V2_PHASE_STATUS.md`; what was actually done is
`V2_EXECUTION_LOG.md`.

**Precedence:** the current tree, the current binary, the Survey and `doctor` outrank every document,
including this one, when they disagree.

---

## 1. The goal

> DeluluLang is the language an agent writes when a human — or a supervising agent — must be able to
> know, before running it, exactly what it can do; and the toolchain is the set of deterministic,
> machine-shaped instruments that make writing, checking, reviewing, running and supervising that
> code cheaper for an agent than any alternative.

Its primary anticipated users are AI agents, LLM-driven agents, local and cloud models, autonomous
software, robotics and physical AI, and AI architectures not yet designed. Humans are first-class
users. **There is one computational-principal model**: different principals receive different
authority by explicit policy, never by kind. What success looks like, in the owner's words: a
developer or an agent says *"build this"*; the AI generates DeluluLang; the toolchain teaches itself to
the AI if necessary; the compiler explains what the program can do, Authority what it may request,
the Guard what was approved, the sandbox what the environment can physically reach, `doctor` what the
host can enforce, the Atlas what the program is made of, the Survey what the repository contains, the
tests what was verified, and the audit what happened — and none of those answers depends on who the
principal is.

## 2. What V2 changes, and what it does not

**Does not change.** The meanings of Authority, the Guard, effects, capabilities, broker custody,
revocation, attenuation and audit. Deterministic, reproducible compilation and execution. The
provenance law of the Survey. The honesty clauses. V2 may harden, fix, expand and add enforcement
layers beneath these; it may not redefine them (`V2_SECURITY_MODEL.md` §2; Constitution §5.16).

**Changes.** V2 adds, in this order of value: a machine contract that is uniformly true; a real
execution-isolation layer under Authority (levels L0–L4, one channel, the guest performs no effects);
run-time plugin loading that actually runs; a standard library an ordinary program can use; the
surfaces agents plug into (a skill, a toolchain manifest, MCP, checked edits, schemas, examples);
resource budgets that become authority; a Linux/KVM microVM tier; a distribution; and a measured —
not asserted — answer to whether the language is AI-friendly.

## 3. The architecture, in one column

```
principal → intent / program → static effects → static authority analysis → operator grant / lease
        → Guard → sandbox policy → sandbox backend → host effect channel → broker / custody
        → real effect → audit
```

Read downward as attenuation: nothing lower can widen what was decided higher. The sandbox may
reduce environmental reach and never creates authority; a backend never reinterprets Authority; every
backend — process jail, microVM, container, gVisor, Kubernetes, remote, cloud, a future confidential
VM — enforces the **same** DeluluLang authority semantics. The roles, the execution modes
(STRICT / AUDIT / BREAK-GLASS), resource authority, the levels and the microVM principle are
specified in `V2_SECURITY_MODEL.md`.

## 4. The phases, in the approved order

| # | Phase | One line | Owner decisions it needs |
|---|---|---|---|
| 0 | **V2-0** | the V2 workspace and the documentation migration into `docs/archive/v1/` | none |
| 1 | **P1** | machine-contract truth: one envelope on every `--json`, `explain --json`, one diagnostic per nesting defect, honest repairs, `test --json` diagnostics, bare `delulu test`, `authority --grants`, the `val`-literal repair, the guide path rule | D-NE-3 (snapshot diffs are shown, not hidden) |
| 2 | **PS-0** | sandbox truth, probes and the cheap hardenings: the `sandbox`/`isolation` object under `--json`, the DL1408 repair, `delulu sandbox probe`, the `doctor` sandbox section, Windows device names and trailing characters refused, the worker read deadline, special-use addresses, the CI experiments | D-NE-28 (address spelling) |
| 3 | **PS-A** | L1: the whole program as a guest holding no OS authority on Linux, Windows and macOS; the effect channel; policy derivation and profiles; the STRICT/AUDIT modes and their transition tests; the machine surface | D-NE-24, D-NE-26, D-NE-31, D-NE-33 (asked at phase start) |
| 4 | **P2** | plugins for real: `root.plugin_host()` and `load` run the Stage 6 load sequence from `delulu run`, under an operator-controlled grant, with revocation and a shipped example | D-NE-10 (the loading grant grammar) |
| 5 | **P4a** | the Agent Skill (`skills/delulu/SKILL.md`, standard format) and `delulu skill` | D-NE-5 (folder name) |
| 6 | **P3** | the standard library, additively: list, string and `Map[K, V]` methods with witnesses and generator coverage | none (build-order rulings) |
| 7 | **PS-B** | resource limits as authority on every engine, the host-side egress proxy as the first network client, identity separation where the OS allows it, BREAK-GLASS as an external operator control | D-NE-28 (TLS dependency), D-NE-31 |
| 8 | **P4b–e** | `toolchain --json`, `schema`, `examples --json`, `delulu mcp`, checked edits, `survey diff`, the Guard view, the Atlas sandbox/resource view, the AI usability benchmark | D-NE-6 (mcp in the CLI) |
| 9 | **PS-C** | L2: the interpreter in a kernel-only guest on Linux/KVM, vsock only, Firecracker under its jailer, criterion 8 un-gated | D-NE-23 (ruled), D-NE-27 (GPL kernel) |
| 10 | **P6** | documentation consolidation: `HANDOFF.md` shrunk to a briefing, the Book and README brought to V2 | none |
| 11 | **P5** | distribution: a tag-triggered release workflow, four targets, checksums, attestations, the skill and morphs in the archive, installers if accepted | D-NE-7 (channel), D-NE-8 (installer posture) |
| 12 | **P7** | verification depth: KAT vectors, `cargo-fuzz` targets, Miri shrinking, Progress restated | the entrenched restatement (owner) |
| 13 | **PS-D** | external launchers (L3) and the attestation seam (L4) | hardware beyond the seam |
| 14 | **P8** | safe autonomy: the signed Verified-class adapter; the rest waits for hardware | owner-gated |

This is an execution dependency graph, not a checklist: if the code reveals a dependency that forces
a different order, the phase stops, records the evidence, explains the dependency, updates this table
and `V2_IMPLEMENTATION_ROADMAP.md`, and continues only when the change is justified. The order is
never changed for convenience. Task-level detail: `V2_IMPLEMENTATION_ROADMAP.md`.

## 5. The gaps V2 closes — each verified against the binary, each with its phase

| Gap | Evidence (archive) | Phase |
|---|---|---|
| A program cannot load a plugin at run time; the flagship demo refuses with a misleading code | NE-01 (`prim.rs:366` stub) | P2 (P1 records it) |
| The `--json` envelope is not uniform; `explain` has no machine channel; a repair with no edits says "apply me"; one defect yields 145 diagnostics | NE-04…NE-08 | P1 |
| `authority` never says which `--grant` flags a program needs; requested scopes are dropped from the Atlas | NE-10, NE-14 | P1 |
| `http.get` has no network client behind it; special-use addresses are ordinary grant values | NE-17, NE-18 | PS-0 (truth), PS-B (the egress proxy) |
| Windows device names and trailing characters pass containment; the foreign worker has no read deadline; the main program has no resource bound | NE-19…NE-22 | PS-0, PS-B |
| `--isolation process` isolates foreign code only, is invisible under `--json`, and `microvm` is a probe that always refuses | NE-16b, RW 4.1 | PS-0, PS-A, PS-C |
| The standard library is four list methods; no `Map` | RW 2.1 | P3 |
| No skill, no MCP, no toolchain manifest, no checked edit, no schema | RESEARCH §0 | P4 |
| Nothing is installable; the morphs the docs say ship do not ship | NE-11, RW 7.5 | P5 |
| AI-friendliness is asserted, not measured | commission §19 | P4e |
| Resource limits are launcher configuration, not authority | commission §13 | PS-B |
| Physical AI: the adapter is an unsigned subprocess | RW 4.7 | P8 |

## 6. How success is measured

Not by commit count. By: time to first compiling program; correction iterations; token and context
cost; undocumented surprises; diagnostic-repair success; agent task success; authority-review effort;
sandbox-policy errors — measured with `delulu-measure` and the V2 usability benchmark
(`V2_AI_NATIVE_DESIGN.md` §4) across five knowledge conditions, from a model that has never seen the
language to one with the full documentation. Every published number comes from a record under
`measurements/` with its method, or it is not published.

## 7. Owner decisions

**Ruled by the commission (2026-09-17):** the plan and the phase order of §4; V2 as the active path
with V1 preserved; `docs/DELULULANG_V2/` as the single active source and `docs/archive/v1/` as the
archive; Opus 5 as the main execution sous-chef; the interpreter in the microVM guest (D-NE-23);
resources as first-class authority dimensions where they can be formalized; STRICT / AUDIT /
BREAK-GLASS as the execution modes, with a program never able to relax its own authority or sandbox;
no holder-kind discrimination anywhere; the phase pause and single continuation.

**Still the owner's, asked at the start of the phase that needs each** (the plan's recommendation is
recorded beside every one in the archive's `DECISION_LOG.md`): the loading grant grammar (D-NE-10,
P2); the skill folder name (D-NE-5, P4a); `mcp` as a CLI subcommand (D-NE-6, P4c); the profile names
and the strengthened `process` label (D-NE-24, PS-A); `Secret.map` under strong profiles (D-NE-25,
PS-A); the `landlock` and `seccompiler` crates (D-NE-26, PS-A); resource-limit defaults (D-NE-31,
PS-A/PS-B); whether `--sandbox` becomes the default (D-NE-33, PS-A — the commission leans to secure
by default where appropriate, with disabling explicit); the first network client's TLS dependency and
the special-address spelling (D-NE-28, PS-0/PS-B); distributing a built GPL kernel (D-NE-27, PS-C);
the release channel and installer posture (D-NE-7/8, P5); the Constitution §5.15 wording, rustfmt and
a code of conduct (unchanged from V1); the four pre-public-repository items (untouched until the owner
decides). A decision that only the owner can take blocks only the task that needs it; the rest of the
phase proceeds and the blocked task is marked *awaiting owner* in `V2_PHASE_STATUS.md`.

## 8. Where the evidence lives

`docs/archive/v1/NEXT_EVOLUTION_2026/`: `VERIFICATION_FINDINGS.md` (NE-01…NE-22, each reproduced on
the binary), `RESEARCH.md` and `SANDBOX_RESEARCH.md` (the field, sources named there and on no
product surface), `SANDBOX_ARCHITECTURE.md`, `SANDBOX_THREAT_MODEL.md` (claims T1–T15 and the
evidence category each owes), `SANDBOX_TEST_PLAN.md`, `SANDBOX_IMPLEMENTATION_PLAN.md`,
`DECISION_LOG.md` (D-NE-01…33), `DOCUMENTATION_AUDIT.md` (the classification behind V2-0's moves),
`EXECUTION_LOG.md` (the two planning passes, every CI run read), and `agent-notes/` (the two
sous-chefs' records). The current gap list of the V1 era stays `docs/REMAINING_WORK.md`; V2 closes
its rows and says what closed each.
