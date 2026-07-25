# DeluluLang — AI-Native Design

**Status:** Design reference (design philosophy + normative pointers into the stage specs; adds no
new mechanism — it *names and connects* what the machine-facing design already guarantees, and
records the commitments future work must keep).
**Governing documents:** `CONSTITUTION.md` (§5.11, §8.4 two-audience rule, §9 honesty, the
no-discrimination principle), all stage specs. **Companions:** `LOCALIZATION_PLUGIN_GUIDE.md`,
`SYNTAX_MORPH_SPEC.md`.

---

## 0. The principle, stated once

> **No discrimination between humans and AI** — agents, LLMs, physical AI, robots, AGI, ASI, future
> evolutions. *If you are made of atoms or of electrons, DeluluLang treats you the same.* Authority
> is the only thing the language cares about; **what you are is never inspected anywhere on any
> authority-decision path** (this is Stage-5 acceptance criterion 9, made a project-wide law here).

DeluluLang is built for a near future in which **AI systems are the majority of programmers**. The
machine-facing surface is therefore a first-class citizen, not an afterthought bolted onto a
human tool. This document collects the properties that make DeluluLang *fast, efficient, and
self-aware for machines* — and marks where future stages must keep that promise.

---

## 1. Awareness — authority is machine-legible, always

The single most AI-native property: **a program's full authority is a computed, queryable fact**, not
a comment or a convention.

- `delulu authority <file|pkg> --json` returns the whole-program effect/capability/secret/foreign/
  plugin report (spec §10.5) — an agent asks "what can this code do to the system?" and gets a
  machine answer *before running anything*.
- `delulu why <Effect> --json` returns the shortest call chain from `main` to the primitive that
  performs the effect — an agent debugging an unexpected authority gets the *proof path*, not a guess.
- The LSP `delulu.authority` command (Stage 8 §3) returns the same report over the editor protocol —
  agentic IDEs call it instead of shelling out.

This is "awareness" in the literal sense: the environment can inspect itself. An agent orchestrating
sub-agents (the Stage-5 holder story) can *verify* each delegated slice mechanically, not trust a
prose promise.

## 2. Efficiency — the feedback loop is built for repair, not reading

Agents work in tight edit→check→repair loops. Every machine surface is shaped for that loop
(Constitution §8.4, the machine-audience half of the two-audience rule):

- **Stable diagnostic codes** (DL0501, …) — an agent keys on the code, never on prose. Codes are
  add-only and versioned (Stage-9 invariant 43): an agent written today keeps working.
- **Typed, machine-applicable repairs** — each diagnostic carries structured repair objects (the
  exact edit), flagged `authority_widening: true|false` and `requires_human: true|false`. An agent
  applies the safe narrowing repair automatically and *escalates* the widening one — the language
  tells it which is which.
- **Locale- and morph-invariant machine envelope** — codes/JSON/spans/DIR/traces are byte-identical
  across every human language and every syntax surface (Stage-8 invariant 39; `SYNTAX_MORPH_SPEC.md`
  §1). Fifty human languages and any number of keyword skins cost the AI side **zero** — nothing new
  to parse, no schema churn. The human side is richly localizable *because* the machine side is
  frozen. **Status, stated precisely:** the *locale* half is built and enforced (`delulu locale`,
  `--locale`); the *morph* half is specified and **not yet implemented**, so its invariance is a
  design commitment the toolchain has not yet had the chance to violate — see finding C22 in
  `docs/design/HARDENING_CAMPAIGN.md`.
- **`--json` / `--no-prompt` / env conventions and cold-start silence** — an agent runs `delulu`
  with no interactive surprise ever (Stage-8 invariant 40: `--json`/`CI`/non-TTY/`DELULU_NO_FIRST_RUN`
  each suppress the picker+welcome). `docs/for-agents.md` (Stage 9 §6) is the one page harnesses pin.
- **Deterministic replay** — `--seed`/`--clock fixed:` make `Cap[Rand]`/`Cap[Clock]` reproducible, so
  an agent's test run is stable and its failures are re-triable.

## 3. Speed — the sandbox is free at the hot path

AI-scale workloads cannot pay a tax per capability check. DeluluLang's architecture is designed so
authority enforcement is *not* on the compute hot path:

- **Capability checks are host-side, not in guest code** (Stage 3): there is nothing for an optimizer
  or a JIT to "optimize away," and pure compute compiles to ordinary WASM/native with no per-op
  authority overhead. The Stage-10 optimizing backend and `@jit` tier inherit this for free
  (invariant 45: every mode passes the same suites — speed never weakens the sandbox).
- **Epoch-class validation is amortized** (Stage 5 §4): read/clock/rand/console re-validate against a
  cached revocation epoch refreshed at most every 50 ms — not a broker round-trip per read.
- **Cross-package inlining is legal because rows are declared** (Stage 10 §2.1): the optimizer can
  inline across boundaries without changing authority, because authority is in the types.
- **Concurrency is data-race-free at compile time** (Stage 7): the actor/reference-capability system
  gives parallelism with *zero* runtime race-checking cost — the guarantee is static.

Honesty (Constitution §5.11): "competitive with C on hot paths, with safety C cannot offer" is the
claim, assessed in Stage-9 Study C and worked in Stage 10 — never "faster than C," never a token
superlative.

## 4. Delulu Authority for machines — the multi-agent story

The holder model (Constitution §5.16, Stage 5) is the same for every kind of holder, and it is
*exactly* what agent orchestration needs:

- An orchestrating LLM holds a grant node; it `delegate`s an attenuated slice (`⊑` its own) to each
  sub-agent as a portable lease token. A sub-agent — whatever it writes or loads or executes —
  **cannot acquire anything outside its node** (Stage-5 invariants 24, 25). Revoke one sub-agent →
  only its subtree dies; revoke the orchestrator → all die.
- **Identical mechanics whether the orchestrator is a human, an LLM, a CI system, or a robot's
  supervisory computer.** No code path inspects which. This is the no-discrimination principle at the
  authority layer.
- Runtime-loaded plugins (Stage 6) let an agent extend a running system with third-party code that
  *still* cannot exceed its grant — the flagship demo, and the safe substrate for agent tool-use.
- Physical stakes (Stage 10 `Actuate`): a robot's control program holds an actuator lease whose scope
  *is* its physical envelope, with a dead-man heartbeat — an agent that hangs or is partitioned
  **loses physical authority by default.** Safety for embodied AI is the same authority mechanism,
  now with a measured latency budget.

## 5. What "make the AI side better" commits future work to

This document is also a standing requirement list. Any stage or RFC touching the machine surface must
preserve, and where possible strengthen:

1. **Zero authority-cost on pure compute** — new capabilities are host-side; never add a per-op guest
   check on the compute path.
2. **Code stability** — diagnostics/JSON/repairs are add-only; an agent written against v1.0 keeps
   working (invariant 43). Breaking the machine contract is a major-version, RFC-gated act.
3. **Machine/human parity of information** — anything a human can learn (a hover, an explain, an
   authority summary) is available as JSON to a machine, from the same source of truth (no
   human-only or machine-only knowledge).
4. **No `what-are-you` branch** — never introduce an authority decision, a rate limit, a default, or
   a prompt that inspects holder kind. Capability is the only currency.
5. **Awareness scales** — as programs, plugins, and actor graphs grow, `authority`/`why`/traces must
   stay complete and queryable (the review surface is the safety surface for AI-written code).
6. **Honesty scales** — every machine-facing claim (perf, token counts, containment) traces to a
   measured number or a stated threat model; superlatives never ship (Constitution §9).

## 6. Where the mechanisms actually live (pointers, not duplication)

| AI-native property | Mechanism | Spec |
|---|---|---|
| Authority is queryable | `delulu authority`/`why` + JSON + LSP command | §10.5; Stage 8 §3 |
| Repair loop | stable codes + typed repairs + `authority_widening`/`requires_human` | Stage 1 §10; all stages |
| Machine surface frozen across locales/morphs | locale + morph invariance | Stage 8 inv. 39; SYNTAX_MORPH_SPEC §1 |
| Cold-start silence | picker/welcome suppression on `--json`/`CI`/non-TTY | Stage 8 inv. 40 |
| Sandbox free at hot path | host-side checks; epoch class; declared-row inlining | Stage 3; Stage 5 §4; Stage 10 §2 |
| Static data-race freedom | actors + reference capabilities | Stage 7 |
| Multi-agent authority | holder model + grant tree + delegation tokens | Stage 5; Constitution §5.16 |
| Runtime extension, still bounded | plugins (Verified/Contained) | Stage 6 |
| Embodied authority + dead-man | `Actuate` + heartbeat leases | Stage 10 §5 |
| No discrimination | no holder-kind branch anywhere | Stage 5 crit. 9 (project-wide) |

*DeluluLang does not make AI a second-class user of a human language, nor humans a second-class user
of a machine language. It gives both the same law — authority — and a different surface over one
truth. That symmetry is the design.*
