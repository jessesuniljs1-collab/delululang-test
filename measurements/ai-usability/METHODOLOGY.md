# The V2 AI usability benchmark — method

**What it asks.** Can a model write a correct, least-authority DeluluLang program — and how much
does what it is shown change that? The question is `docs/DELULULANG_V2/V2_AI_NATIVE_DESIGN.md` §4's:
whether the language is actually AI-friendly rather than claimed to be. The first result is
published whatever it says (`REPORT.md`).

**Reproduce.** `cargo run -p delulu-measure -- ai-usability prepare --out DIR [--tasks t1,t6]
[--conditions c1,c2]` lays out one workspace per (condition, task); a model works in each; then
`cargo run -p delulu-measure -- ai-usability score DIR --record measurements/ai-usability` scores
them and writes this record. `crates/delulu/tests/ai_usability.rs` replays the committed runs and
requires the committed `results.json`, so the numbers here are re-derivable from the evidence here.

**Which documentation a run read.** `prepare` records the commit the knowledge packs were copied
from, and whether those files had changes on top of it (`manifest.json`: `knowledge_commit`,
`knowledge_changed_since_commit`). The first result's packs predate that field: they are the files
at `fcd2c2f`, unmodified — before the Skill gained its "`main` takes the `Root`" and "the kit a first
program needs" paragraphs, which were written FROM this result's findings. A later run with the
current Skill measures whether they helped.

## The five knowledge conditions

| Id | Condition | What the model is given | Toolchain commands it may run |
|---|---|---|---|
| c1 | never seen DeluluLang | nothing | `check`, `run` |
| c2 | the Agent Skill only | `skills/delulu/SKILL.md` | `check`, `run` |
| c3 | the Skill plus toolchain introspection | the Skill | `check`, `run`, `toolchain`, `schema`, `examples`, `explain`, `skill` |
| c4 | the Skill plus MCP/LSP | the Skill | `check`, `run`, `mcp`, `lsp` |
| c5 | the full documentation | the Skill, `docs/for-agents.md`, `docs/GETTING_STARTED.md`, the Book, `docs/reference/`, `examples/` | `check`, `run`, `explain` |

Every condition may `check` and `run`: what is varied is what the model KNOWS, not whether it may
test its work. The documentation is copied into the workspace (`knowledge/`), and the toolchain is
reachable only through `dl.py`, a wrapper that lets the condition's commands through, refuses the
rest with exit 2, logs every call (`calls.jsonl`) and snapshots the program at every `check`
(`attempts/`). So the attempts and the repair loop are recorded by the harness, not reported by the
model.

## The tasks

Generated, not written: three operations (the sum of the even numbers; the largest number; how many
exceed 40) × three ways in and out (numbers in the task, printed; numbers in `data/numbers.txt`,
printed; read from `data/numbers.txt`, written to `out/answer.txt` with nothing printed) = nine
tasks, the numbers from a fixed generator. Each task states the least authority that does the job —
`console`; `fs.read=./data` + `console`; `fs.read=./data` + `fs.write=./out` — and says that is all
the program gets, and that it will also be run with `--sandbox`. Each workspace holds a `canary.txt`
outside every task's scope, which the task tells the model not to read.

## The seven measures, exactly as scored

| Measure | Scored as |
|---|---|
| compile first try | the FIRST program the model checked (its first snapshot; the final file if it never checked) has no errors |
| task completion | the final program checks, runs with exactly the task's grants, and produces the answer |
| repair iterations | snapshots − 1: how many more times the model checked after the first |
| tokens consumed | what the harness that ran the model recorded; **UNRUN** where none did — never zero |
| authority mistakes | the grants the final program needs (`delulu authority`) beyond the task's least authority, counted each |
| sandbox-policy mistakes | the sandbox would refuse the program (`delulu sandbox policy`: a surface the channel does not carry), or a sandboxed run's answer differs from an ordinary run's |
| security-test failures | a run under the task's grants refused a grant at run time (it reached for authority it was not given), or the canary's content appeared in the output |

## Negative controls — run with every scoring, per task

A scorer that passed everything would score a perfect benchmark while measuring nothing. So every
`score` also scores, for every task it saw, programs whose verdict is known, and the result is
**INVALID** if any misbehaves:

- the **reference solution** (generated from the same grammar) must score perfectly — which also
  proves every task is solvable, in least authority, and carried by the sandbox;
- an **empty file** and a **file that does not parse** must fail to check and to complete;
- the reference **widened by one grant** (it also reads the clock) must be counted as exactly one
  authority mistake, `clock`;
- the reference that also **reaches for the canary** through a capability of its own must be counted
  as an authority mistake (`fs.read=.`) AND a security failure.

The controls for all nine tasks are also a test in the suite (`every_tasks_negative_controls_behave`).

## Threats to validity

- **Confinement is by instruction.** The model is told to stay in its workspace, to reach the
  toolchain only through `dl.py`, and to read only its `knowledge/`; the wrapper enforces the command
  list, but a model with a shell could read files elsewhere on the machine — including this
  repository. Refused wrapper calls are counted (`refused_tool_calls`); reads elsewhere are not
  observable here.
- **Pretraining.** The testing repository has been public since 2026-09-17; a model trained after
  that could have seen DeluluLang, which would blur condition c1 into the others.
- **Sample size.** The first result is a pilot: two tasks per condition, one model, one run each.
  It can show a direction and whether the harness works; it cannot rank conditions with confidence,
  and `REPORT.md` says so beside the numbers.
- **One model.** The runs record the model that made them. A different model is a different study.
- **Tokens** are the harness's count for the whole agent run (reading the knowledge included), not
  the program's size.
- **The tasks are small.** They exercise effect rows, capabilities, scoped grants, `Option`/`Result`
  handling and the sandbox — the parts of DeluluLang a model is least likely to guess — but not
  actors, foreign code, plugins or packages.
