# DeluluLang V2 — the active source of truth

**Status: EXECUTING.** The owner approved the 2026 evolution plan on 2026-09-17 and named the work
**DeluluLang V2** (`docs/design/DeluluLang_V2_Execution_Master_Prompt.md`). V1 — everything up to
tag `v1.0.0` and the post-1.0 hardening campaigns — is preserved and stays recoverable; nothing of it
is rewritten. V2 is the active forward path, and **this folder is its one active source of truth**.

The plan V2 executes was written in two planning passes on 2026-09-17 and is kept, with all its
evidence, in the V1 archive: `docs/archive/v1/NEXT_EVOLUTION_2026/`. The active, maintained form of
it is here. When a V2 decision changes, these files change; the archive does not.

| File | What it is | Read it when |
|---|---|---|
| [`V2_MASTER_PLAN.md`](V2_MASTER_PLAN.md) | the goal, the architecture, the phases, the gaps V2 closes, what does not change, the owner's decisions | you want the whole picture |
| [`V2_IMPLEMENTATION_ROADMAP.md`](V2_IMPLEMENTATION_ROADMAP.md) | every phase with its tasks, verification, dependencies, and the end-of-phase protocol | you are about to do the work |
| [`V2_PHASE_STATUS.md`](V2_PHASE_STATUS.md) | one row per phase: state, commit, CI run, date | you want to know where V2 is right now |
| [`V2_EXECUTION_LOG.md`](V2_EXECUTION_LOG.md) | per phase: what was implemented, files, commands and results, Survey, doctor, CI, git, agents, problems, decisions | you want to know what actually happened |
| [`V2_DECISION_LOG.md`](V2_DECISION_LOG.md) | records `D-V2-nn`: evidence, alternatives, why, status (RULED by the owner / TAKEN by the head chef / PROPOSED) | you want to argue with a choice |
| [`V2_SECURITY_MODEL.md`](V2_SECURITY_MODEL.md) | the authority + sandbox model: the enforcement stack, the roles, execution modes, resource authority, isolation levels, the microVM principle, what is and is not claimed | you touch Authority, the Guard, the broker or a sandbox |
| [`V2_AI_NATIVE_DESIGN.md`](V2_AI_NATIVE_DESIGN.md) | the AI-native goal as engineering: one principal model, the zero-shot learning loop, the surfaces, the usability benchmark, AI-audits-AI | you build an agent surface or measure usability |
| [`V2_DOC_MOVE_MANIFEST.md`](V2_DOC_MOVE_MANIFEST.md) | every file moved into `docs/archive/v1/` in phase V2-0: original path, new path, why, class, links changed, whether it stays authoritative anywhere | you are looking for a document that used to be somewhere else |
| [`V2_AGENT_LOG.md`](V2_AGENT_LOG.md) | what each sous-chef agent was asked, did, found and failed | you want to know what an agent did |

## How V2 proceeds

One phase at a time, in the approved order (`V2_MASTER_PLAN.md` §4). At the start of a phase: read
its objectives, inspect the affected code, run the Survey and `doctor`, establish the baseline. Most
implementation is delegated to one Opus 5 sous-chef with an exact brief; the head chef (Claude
Fable 5.1) verifies everything an agent produces against the real binary. At the end of a phase:
tests, the Survey regenerated as the last edit, `doctor`, the relevant gates, the V2 logs, commit,
push to the testing remote only, the CI result read and recorded — then **STOP**, wait about sixty
seconds for the owner, and continue to the next approved phase if nothing arrives. A newer owner
instruction always beats the plan. The full protocol and the verification gate are in
`V2_IMPLEMENTATION_ROADMAP.md` §0.

## Rules this folder observes

- **The core is stable.** Authority, the Guard, effects, capabilities, custody, revocation,
  attenuation and audit keep their meaning; V2 hardens, fixes, expands and adds enforcement
  layers, never redefines (`V2_SECURITY_MODEL.md` §2).
- **One principal model.** Nothing in the language or the toolchain branches on whether the
  holder is a human, an AI, an agent, a robot or something not yet named.
- **Nothing is claimed that was not executed.** Every feature carries one label: implemented,
  measured, designed, partially implemented, unverified, or deferred. The labels are never collapsed.
- **Historical documents are not maintained.** They live in `docs/archive/v1/` and are neither
  rewritten nor deleted. Current documents are updated only when genuinely necessary; the V2 files
  here are updated always.
- **The Survey stays repository truth** (generated, provenance-based, regenerated after every
  change); `doctor` answers what the host can enforce; the Atlas answers what a program means.
- Push only to the testing remote; never rewrite pushed history; never a literal CI skip token in a
  commit message; the final public repository is the owner's step and is gated on his decisions.
