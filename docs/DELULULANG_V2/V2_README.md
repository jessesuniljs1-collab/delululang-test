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
| [`V2_LOG.md`](V2_LOG.md) | the running log: one short block per phase | you want what happened, briefly |
| [`V2_PHASE_STATUS.md`](V2_PHASE_STATUS.md) | one row per phase: state, commit, CI run, date | you want to know where V2 is right now |
| [`V2_EXECUTION_LOG.md`](V2_EXECUTION_LOG.md) | **frozen at P1** (D-V2-22): the per-phase record of V2-0 and P1 — files, commands, Survey, doctor, CI | you want the first two phases in full; everything later is in `V2_LOG.md` |
| [`V2_DECISION_LOG.md`](V2_DECISION_LOG.md) | records `D-V2-nn`: evidence, alternatives, why, status (RULED by the owner / TAKEN by the head chef / PROPOSED) | you want to argue with a choice |
| [`V2_SECURITY_MODEL.md`](V2_SECURITY_MODEL.md) | the authority + sandbox model: the enforcement stack, the roles, execution modes, resource authority, isolation levels, the microVM principle, what is and is not claimed | you touch Authority, the Guard, the broker or a sandbox |
| [`V2_AI_NATIVE_DESIGN.md`](V2_AI_NATIVE_DESIGN.md) | the AI-native goal as engineering: one principal model, the zero-shot learning loop, the surfaces, the usability benchmark, AI-audits-AI | you build an agent surface or measure usability |
| [`V2_DOC_MOVE_MANIFEST.md`](V2_DOC_MOVE_MANIFEST.md) | every file moved into `docs/archive/v1/` in phase V2-0: original path, new path, why, class, links changed, whether it stays authoritative anywhere | you are looking for a document that used to be somewhere else |
| [`V2_AGENT_LOG.md`](V2_AGENT_LOG.md) | **frozen at P1** (D-V2-22): what each sous-chef agent was asked, did, found and failed, up to P1; later agent passes are recorded in `V2_LOG.md` | you want to know what an early agent did |
| [`V2_PS_C_PREREQUISITES.md`](V2_PS_C_PREREQUISITES.md) | what the microVM phase needed before it could start — KVM, the VMM, the kernel and image, the host — and how each was met | you touch the microVM |
| [`V2_PS_C_RED_TEAM.md`](V2_PS_C_RED_TEAM.md) | the microVM's red-team record (PS-C-06): the hostile guests, what each tried, what held | you want to know what attacks the microVM was tested against |
| [`V2_P8_DESIGN.md`](V2_P8_DESIGN.md) | P8's design: the control program in a guest, the Verified-class adapter, a reference transport, the out-of-band monitor | you start a P8 slice |
| [`V2_OPENSHELL_STUDY.md`](V2_OPENSHELL_STUDY.md) | the study of NVIDIA OpenShell (2026-09-28): the two designs side by side, what DeluluLang takes and why, what it does not, and the design of phases PS-E and P9 | you start a PS-E or P9 slice, or compare DeluluLang's sandbox with another |

## How V2 proceeds

**Where V2 is now:** `V2_PHASE_STATUS.md` — as of 2026-09-28 evening, every phase through P7 and
PS-D is complete; **PS-E** (the boundary, confirmed — from the study of NVIDIA OpenShell) is next, then
P8 (designed) and P9 (designed). **From
2026-09-28 to 2026-10-16 the work runs in Claude Code cloud sessions**, through pull requests, with every
change recorded in `docs/CLOUD_SYNC_LOG.md`, and a scheduled routine runs the loop in
`docs/CLOUD_ROUTINE.md` under the owner's delegation (`HANDOFF.md` §0).

One phase at a time, in the approved order (`V2_MASTER_PLAN.md` §4). At the start of a phase: read
its objectives, inspect the affected code, run the Survey and `doctor`, establish the baseline. The
head chef — the main Claude Code session: Fable 5.1 for V2-0, Opus 5 from P1 to 2026-09-20, Opus 5.5
from PS-B-02 (2026-09-25) on, as each commit's `Co-Authored-By` line records — does
or delegates the implementation, each agent with an exact brief, and verifies everything an agent
produces against the real binary; testing passes use Haiku 4.5 and Sonnet 5 agents (Sonnet 5.5 from 2026-09-28) on several
operating systems (owner, 2026-09-27). At the end of a phase:
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
