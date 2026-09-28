---
name: subagent-cost-strategy
description: "Jesse hits session/usage limits; keep subagent cost down — default Sonnet, reserve Opus, prefer inline"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-07-20T15:31:01.822Z
---

Jesse repeatedly hits Claude session/usage limits (e.g. Opus 4.8 review subagent failed mid-run
with "session limit"). He asked me (Fable, the orchestrator / "head chef") to control subagent
effort so he stops hitting limits — WITHOUT downgrading quality.

**Why:** every subagent spawn starts cold and re-derives context I already hold warm, and Opus 4.8
is the most expensive model; both burn the limited session budget fast.

**How to apply (UPDATED 2026-07-05 — Jesse explicitly wants delegation-first now):**
- Jesse's standing instruction: DON'T do heavy coding/repeated tasks inline myself (he believes it
  auto-switches the main thread to Opus billing). Head-chef pattern: I orchestrate, review, fix,
  commit; subagents cook.
- The dominant cost lever I control is the subagent's **model** (Agent tool `model` param). There
  is NO separate effort dial — model choice IS the effort control; told Jesse this.
- **Opus 4.8 subagents** = soundness-critical / complex compiler work. **Sonnet 5 subagents** =
  well-specified, lower-reasoning coding + tests + docs chores.
- Parallel agents MUST have disjoint file sets (no shared cli.rs edits etc.); give each an explicit
  do-not-touch list; agents do NOT git commit — I review diffs, run the suite, commit.
- Subagent prompts must include: repo path D:\nelan\DeluluLang, the cargo PATH fix
  ($env:USERPROFILE\.cargo\bin prepend), spec file refs, current-architecture facts (so no
  re-derivation), scope, constraints, verification steps, report format. Retry-once note for
  Windows transient LNK1104 linker errors.
- Keep each turn bounded; don't cram a risky refactor into a long turn's tail.

Kitchen analogy Jesse uses: I'm the head chef orchestrating; Opus 4.8 = pro chefs; Sonnet 5 =
other chefs/workers. See [[delululang-project]].

**Evolution since (see [[head-chef-handoff]] for the full history):** this delegation-first
guidance was REVERTED 2026-07-19 after a Stage-9 sous-chef on phase 9a ran slower and pricier than
the head chef doing it directly — Jesse pulled it and ruled "head chef cooks directly... unless
Jesse says otherwise." It stayed that way through all of Stage 10's phases 10a–10h.

**RE-AUTHORIZED narrowly 2026-07-20, Stage 10 phase 10i** — "run multile sonnet 5 agents" — and
this time it worked cleanly: two Sonnet-5 agents (`model: "sonnet"` explicit), disjoint file sets
(one wired CLI flags, one did web research + a KAT vector fixture), thorough self-contained prompts
per the pattern below, neither committed, both independently re-verified before landing (re-ran
every test from a clean build; re-fetched two of the research agent's cited source files from
GitHub myself and matched hashes rather than trusting its self-report) — no rework needed on
either. **Confirms the 07-05 pattern was sound; 9a's failure was likely about that specific task or
that specific agent run, not delegation itself.** Still: default to head-chef-direct unless Jesse
asks for agents again — this is not yet a standing default, just a proven option.
