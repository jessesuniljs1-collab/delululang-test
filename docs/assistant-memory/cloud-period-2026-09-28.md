---
name: cloud-period-2026-09-28
description: "Jesse's laptop is away 2026-09-28 → 2026-10-16; DeluluLang work runs in Claude Code cloud sessions via PRs, every change recorded in docs/CLOUD_SYNC_LOG.md for a later sync"
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-09-28T05:57:50.018Z
---

From **2026-09-28 to 2026-10-16** Jesse uses the laptop for other work; DeluluLang is developed from
**Claude Code cloud sessions** (claude.ai/code / app / `claude --cloud`) on the GitHub testing repo,
which is already connected. The laptop checkout `D:\nelan\DeluluLang` is synced afterwards.

- **Baseline:** the commit that added `docs/CLOUD_SYNC_LOG.md` (laptop clean, nothing unpushed).
- **Every cloud session appends an entry to `docs/CLOUD_SYNC_LOG.md`** (Jesse: "keep a record of files
  and folders changed, to be sync with local repo later") — commits + `git diff --name-status`,
  what was verified, what to redo on the laptop, what the laptop memory must learn.
- **Cloud facts (official docs, read 2026-09-28):** Ubuntu 24.04 x86-64, 4 vCPU / 16 GB / 30 GB;
  Rust, Python, GCC/Clang, Docker, `gh` preinstalled; crates.io + static.rust-lang.org allowed
  (Trusted); `git push` ONLY to the session's own branch → PR into master, Jesse merges; auto memory
  is machine-local (NOT in cloud) → `HANDOFF.md` §11 and `docs/assistant-memory/` (a sanitized
  snapshot of this directory) are the cloud's memory; CLAUDE.md must `@AGENTS.md` for both to load.
- **Same day:** "run everything on github" (heavy runs on CI, not the laptop — it ran out of RAM and
  Claude Code reaped the suite and a Miri run); "stop before PS-D-02" (its design draft is in
  `HANDOFF.md` §0).
- **Back on the laptop:** follow `docs/CLOUD_SYNC_LOG.md` *Syncing the laptop afterwards* — ff-only
  pull, CRLF check, Survey + doctor, full suite Win + WSL, each entry's redo items, then merge
  `docs/assistant-memory/` and HANDOFF §11 changes back into this directory.

**Why:** Jesse's instruction, 2026-09-28. **How to apply:** in the cloud read CLAUDE.md → AGENTS.md →
HANDOFF §0/§1/§11 → docs/assistant-memory/MEMORY.md; on the laptop after the 16th, sync first.
Related: [[delulu-v2-execution]], [[testing-repo-autopush]], [[final-public-repo-gate]],
[[agent-usage-rule-2026-09-17]].
