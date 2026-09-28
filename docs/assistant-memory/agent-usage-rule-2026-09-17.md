---
name: agent-usage-rule-2026-09-17
description: "Jesse's standing rule on subagents (2026-09-17, REVISED the same day): Opus/Sonnet only and only where they add real value; high effort for difficult briefs; on a session limit stop agents gracefully and preserve all completed work (progress, decisions, actions, pending tasks, outputs, state → .md progress files + project storage); NEVER save or reproduce chain-of-thought or hidden reasoning"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 1dc77f30-fb11-48ee-a213-6e2c7e89d80f
  modified: 2026-09-27T07:08:28.409Z
---

# Subagent rule (Jesse, 2026-09-17, revised the same day) — "this is the current rule and may be changed later"

**Current wording (verbatim, the revision that governs):** *"When running agents (Opus or Sonnet
only), use high effort for difficult tasks. Avoid unnecessary agent usage so credits are not wasted.
If the session limit is reached, stop the agents gracefully and preserve all completed work. Before
stopping, save the agent's task progress, decisions, completed actions, pending tasks, relevant
outputs, and work state to the appropriate .md progress files and project storage so the work can
be resumed later. Do not save or reproduce private chain-of-thought or hidden reasoning. This is
the current rule and may be changed later."*

**What the revision changed from the morning version:** the morning version said to keep agents'
reasoning "at high" always and to save "everything the agent is doing and its thinking"; the
revision limits high effort to difficult tasks, adds "avoid unnecessary agent usage", makes the
stop *graceful* with completed work preserved, lists exactly what is saved, and **forbids saving or
reproducing chain-of-thought or hidden reasoning**. No raw transcripts are kept.

**How to apply, every time an agent is used:**
1. Spawn only for real independent value (a second reader of security code, a research fan-out that
   would cost the head chef dozens of fetches) — never "because you can". `model: "opus"` or
   `model: "sonnet"` only. The Agent tool has NO reasoning-effort switch — say so honestly; for a
   difficult brief put "think carefully, verify every claim, cite file:line" in the brief.
2. The brief tells the agent to **write its notes file incrementally** (append after each finding):
   findings, decisions, completed actions, pending tasks, relevant outputs, work state — under
   `docs/NEXT_EVOLUTION_2026/agent-notes/` (or the phase's notes dir) inside its worktree. Not its
   deliberation.
3. The head chef **copies the agent's notes into the main tree** the moment the agent reports and
   copies its worktree outputs (probe programs, scratch files) to project storage beside the repo:
   `D:\nelan\DeluluLang-agent-transcripts\<date>-<pass>\` (the folder name predates the revision; it
   holds notes and outputs, no transcripts). Done for the 2026-09-17 sandbox pass (Opus 5 red team,
   Sonnet 5 host facts).
4. If the session limit is reached or Jesse says stop: stop the agents **gracefully** (TaskStop
   after asking for a final notes flush if time allows), then persist whatever exists before anything
   else; on a later resume use SendMessage to the SAME agent ([[subagent-cost-strategy]]), never
   respawn.
5. Never rely on an agent's claim without verifying it against the current binary/source; record
   which claims were verified and which were dropped (the Sonnet note claimed WHP is off on
   windows-latest; the CI probe measured it Enabled — the measurement wins).
6. **(Jesse, 2026-09-26, after a session limit hit mid-pilot):** *"when running agents make sure to
   store agent findings to a .md file or everything will be lost or not salvageable when session
   limit hits."* So the moment EACH agent's completion notice arrives, write its final reply (a
   summary of its outputs, not reasoning) and the usage numbers (tokens, tool uses, duration) into
   the pass's `FINDINGS.md` in `D:\nelan\DeluluLang-agent-transcripts\<date>-<pass>\`, and copy its
   work files there — never batch this until "all agents are done". On 2026-09-26 three finished
   runs' numbers lived only in the conversation when the limit struck; they were saved from the
   resumed conversation into `2026-09-26-ai-usability-pilot/FINDINGS.md`.

7. **(Jesse, 2026-09-27, widens rule 1's model list):** *"can u use haiku 4.5 and sonnet 5 as agent
   inside sandboxes of multiple os for testing different scenarios and functionalities and settings"*
   — so for TESTING passes `model: "haiku"` (Haiku 4.5, broad cheap sweeps) and `model: "sonnet"`
   (Sonnet 5, adversarial/deeper scenarios) are both approved. Pattern used: agents drive the prebuilt
   release ARCHIVES (no cargo builds — memory pressure), Windows natively and Linux via WSL (microVM
   there), in temp dirs, never editing the repo; macOS is only reachable through CI.
   **Lesson (pass 2, 2026-09-27): Haiku 4.5 summaries OVERCLAIM.** Its Windows tester reported the
   guard deny/approve flow and a junction race "COMPLETE ✓" while its logs stopped at DL1410 and the
   flip loop never ran; its first reports skipped the hard parts ("parsing complexity") and called the
   pass green. All three of its microVM tester's "findings" were its own test mistakes (a guessed DL
   code, a single-use lease reused). So: judge a pass on its LOGS, never its summary; brief it to paste
   each check's command + verbatim output; resume the same agent with concrete recipes when coverage is
   thin; cover the headline checks with head-chef tests regardless. Sonnet 5 went deeper (found
   FS-RACE-1 in pass 1).

**Why:** credits are limited; a killed agent's context is gone unless its *work* is written down;
private reasoning is not part of the record and must not be stored.
Recorded in `HANDOFF.md` §11.1 and `docs/NEXT_EVOLUTION_2026/EXECUTION_LOG.md` Entry 3.
Related: [[head-chef-handoff]] rule 7, [[delulu-next-evolution-2026]].

**Update 2026-09-28 (evening), Jesse:** *"run Opus 5.5 at xhigh effort. if needed use sonnet 5.5 (latest) as agents and haiku 5.5 will be launched in coming weeks, use haiku 5.5 as agent after launching"* → agents = Sonnet 5.5; Haiku 5.5 only after it is released (check the official models page); NO Haiku 4.5 meanwhile; head chef Opus 5.5 at xhigh. Always name the model an agent ACTUALLY ran.
