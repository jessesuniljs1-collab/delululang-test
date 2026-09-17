# V2 AI-native design — the goal as engineering, and how it is measured

**Status:** the active design for V2's agent-facing work. It extends `docs/design/AI_NATIVE_DESIGN.md`
(the V1 machine-facing commitments, still normative) and does not replace it. Labels as in
`V2_SECURITY_MODEL.md`: implemented / measured / designed / partially implemented / unverified /
deferred.

---

## 1. One principal model

DeluluLang's primary anticipated users are AI agents, LLM-driven agents, local and cloud models,
autonomous software, robotics and physical AI, future AGI-like systems and architectures not yet
predicted. Humans remain first-class users. **The language model makes no distinction between human,
AI, agent, robot, AGI, ASI, physical AI or future AI.** There is one computational-principal model;
different principals receive different authority according to explicit policy — a grant, a lease, a
Guard tier, a sandbox profile — never according to kind. No "AI mode" or "robot mode" is invented
without a real technical reason, and none is known. [implemented in the authority model and tested
(Stage-5 criterion 9); binding on every V2 surface]

## 2. The zero-shot loop — a model that has never seen the language

The most important V2 usability objective: **a model never trained on DeluluLang can learn enough
from the installed toolchain to write correct programs.** Pretraining is not assumed. The loop the
toolchain must support, and the surface that serves each step:

```
unknown model
  → discover the language              delulu skill · delulu toolchain --json · delulu --help
  → read the compact machine contract  delulu schema · docs/for-agents.md (pinned)
  → generate a program                 delulu examples --json (shipped, checked programs with their authority)
  → check                              delulu check --json (many files, one process)
  → structured diagnostics             codes, byte spans, typed repairs, explanation ids
  → repair                             delulu fix --json (never a widening one unless named) · delulu explain --json
  → authority analysis                 delulu authority --json (+ required grants, requested scopes)
  → sandbox analysis                   delulu sandbox policy --json · run --json's sandbox object
  → test                               delulu test --json (diagnostics carried)
  → security analysis                  delulu atlas --json · the audit chain · doctor --json
  → run                                delulu run --json --no-prompt under a grant and a profile
```

| Surface | Today | Phase |
|---|---|---|
| `delulu --help`, completions, the command list — one source, gated | implemented | — |
| `check --json`, `authority --json`, `run --json`, `test --json`, `fix --json`, `atlas --format json`, `doctor --json` | implemented, envelope uneven (NE-05); `test` hides diagnostics (NE-08) | P1 |
| `explain --json` | missing (`--json` accepted and ignored, NE-06) | P1-03 |
| `authority --grants` / `required_grants`, `requested_scopes` | missing (NE-10, NE-14) | P1-08 |
| `delulu skill` — prints the shipped Agent Skill; `skills/delulu/SKILL.md` in the standard format | missing | P4a |
| `delulu toolchain --json` — version, commands and flags, grant grammar, the ten effects, the primitive table, limits, engines and fragments, **generated from the binary's own tables** | missing | P4-02 |
| `delulu schema [envelope\|diagnostic\|repair\|authority\|atlas\|sandbox\|policy] --json` — the JSON shapes as data, generated from the emitters | missing | P4-09 |
| `delulu examples --json` — the shipped, gated example programs with their authority report and the grant line that runs each | missing | P4-10 |
| `delulu repair` | `fix` is the repair command; a second name is a decision, not a gap (D-V2-16) | P4 |
| `delulu sandbox probe/policy --json`, the `sandbox` object | missing | PS-0, PS-A |
| `delulu mcp` (read-only, stateless) and the LSP | LSP implemented; MCP missing | P4c |

## 3. AI-friendly syntax — measured, not defended

DeluluLang is not simplified by making it look like Python. AI-friendly means: regular syntax, low
ambiguity, explicit semantics, a stable grammar, strong diagnostics, machine-readable errors,
structured repairs, discoverability, one canonical representation, good examples, predictable rules.
Syntax morphs are used where they genuinely help and the canonical form stays the single semantic
representation. **The current syntax is something to measure, not merely defend**: time to first
successful program; number of correction loops; token and context cost; undocumented surprises;
diagnostic-repair success; agent task success; authority-review effort; sandbox-policy mistakes.
`delulu-measure` (studies A, B, C) is the instrument; Study B already measures agent repair loops and
reports that 8.3% of real defects offer a machine-applicable repair. [partially measured]

## 4. The V2 AI usability benchmark

A reproducible benchmark, built on `delulu-measure` and recorded as a record `ai-usability` under
`measurements/`, in the house style (machine, method, threats, negative controls), run across
**five knowledge conditions**:

1. never seen DeluluLang (the model's pretraining only);
2. only the compact Agent Skill;
3. the Skill plus toolchain introspection (`toolchain --json`, `schema`, `examples --json`, `explain --json`);
4. the Skill plus MCP/LSP;
5. the full documentation.

and reporting **seven measures** per condition: compile-first-try rate, successful task completion,
repair iterations, tokens consumed, authority mistakes, sandbox-policy mistakes, security-test
failures. Tasks are generated from a task grammar, not hand-written; every run is replayable; models
and versions are recorded in the measurement record (a measurement may name what it measured — the
owner's rule about research sources concerns product surfaces). The benchmark tells us whether the
language is actually AI-friendly rather than claimed to be, and its first result is published
whatever it says. [designed; P4e, after P4a–d exist]

## 5. AI security audits AI

The V2 workflow around generated code:

```
AI writes code → AI explains code → AI queries the Atlas → AI runs the Survey (in the source tree)
→ AI checks authority → AI checks the sandbox policy → AI generates adversarial tests
→ AI attempts attacks → AI reviews audit data → AI proposes remediation
→ a human security expert reviews the important boundaries → execution
```

A model's "this code looks secure" is not evidence. The auditor consumes machine-grounded facts:
compiler output, the Authority report, Guard state, sandbox state, the Atlas, diagnostics, tests,
fuzz results, the audit chain, CI, `doctor`, deployment facts. Every one of those exists today except
sandbox state (PS-0/PS-A) and a read-only Guard view for tools (P4-07). The MCP server and the LSP are
analysis-only by construction and never run, grant or load. [partially implemented]

## 6. Future algorithms — a stable core, extensible outer layers

Nothing is designed around today's algorithms. **Stable core:** types, effects, authority,
capabilities, deterministic semantics, audit, the sandbox model, resource control. **Extensible
outer layers:** plugins, the standard library, foreign interfaces, device adapters, runtime backends,
sandbox providers, remote execution, agent protocols, new accelerators, future hardware. No assumption
about what "AI" means is hard-coded anywhere.

## 7. Natural language stays outside the language core

Humans describe goals naturally; an AI compiles those goals into DeluluLang. No opaque,
LLM-evaluated semantics enter DeluluLang itself: the compiler, runtime and security properties stay
deterministic and reproducible (D-NE-14; deterministic replay, `--assert-trace` and the
core-invariance snapshot depend on it).

## 8. Physical AI

The same principal, identity, authority, capability, sandbox and resource-budget model; the actuator
path and its current state are in `V2_SECURITY_MODEL.md` §9.

## 9. What is measured today, and what is only claimed

Measured: the process-startup floor and the batching gain (`measurements/agent-loop/`); repair-loop
coverage (Study B); performance against C (Study C; not competitive, said in those words). Claimed
and not yet measured: that an untrained model can learn the language from the toolchain; that the
skill and the manifest reduce correction loops; that the compact morph saves tokens with any given
tokenizer (the project claims no number). Each of those becomes a row of the benchmark or stays a
claim with the word *unverified* beside it.
