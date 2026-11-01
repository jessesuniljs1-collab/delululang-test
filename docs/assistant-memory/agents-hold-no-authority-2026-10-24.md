---
name: agents-hold-no-authority-2026-10-24
description: "Jesse 2026-10-24 — when agents test DeluluLang, the head chef holds the delulu authority; agents must not be able to change their own access"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-23T19:36:52.876Z
---

Jesse, 2026-10-24: "While running agents for testing DeluluLang, U will be having delulu authority, Only u should be able to change anything that agents can do, Agent themselves shouldn't able to change access".

**Why:** the agents play the untrusted program DeluluLang exists to contain; a test where the agent can grant itself access tests nothing, and Jesse wants the authority split real.

**How to apply (the setup that worked, 2026-10-24):** a broker in strict anchored-root mode whose keys, root cert and Guard owner code sit in a head-chef-only dir; one `--multi` lease per test condition (a `--lease` run refuses any local `--grant` — DeluluLang enforces that part itself); a wrapper `dl` that is the agents' only way to run delulu and refuses authority flags; and a Claude Code PreToolUse hook in `.claude/settings.local.json` (`agent_gate.py`) that refuses subagent calls reaching the authority dir, naming grant/lease flags, running delulu directly, setting DELULU_ vars, wsl/runas/icacls, settings or repo writes. Subagent hook input carries `agent_id`/`agent_type`; the main session's does not. **Falsify the gate with a tiny agent before trusting it**, and judge by the gate's own log. Residual stated honestly: same OS account, pattern-based — Tier 2 (second account) is the real boundary. Copies of the scripts: `D:\nelan\dogfood-campaign\mirror\2026-10-24\_head-chef\`. Related: [[save-agent-work-to-storage-2026-10-24]], [[agent-usage-rule-2026-09-17]].
