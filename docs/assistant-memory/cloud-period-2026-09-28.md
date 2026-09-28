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

- **Run and check everything using the Survey and doctor** (Jesse, 2026-09-28): `survey check` + `doctor --check` at every session start; `survey impact/affected-by/query` before a change; `survey build/check/findings` + `doctor --check` after the last edit and before every commit/PR, results in the PR and the sync-log entry (table in AGENTS.md; HANDOFF §0 rule 6).
- **Baseline = the newest `Cloud handoff` commit** (`git log -1 --grep='^Cloud handoff'`): `d41e558`, then `bc9192c`, then `937aea8`, `7e67f97` (routine + delegation), then `7bd018c`, then `5bb39bc` (Cloud handoff (6), the mandate), then **`Cloud handoff (7)` = THE BASELINE** (2026-09-28 evening, pushed from the laptop after it fast-forwarded to the cloud's `dd543e5`: the OpenShell study). **ROUTINE `trig_01GG41tCGxZXyt7bZUid8Vuu`** (https://claude.ai/code/routines/trig_01GG41tCGxZXyt7bZUid8Vuu): Opus 5.5, cron `7 */5 * * *` UTC (IST 05:37/10:37/15:37/20:37/01:37), env `env_012Mxeiw6zvfyxPy9ZEEFf3p`, tools Bash/Read/Write/Edit/Glob/Grep/Agent/WebFetch/WebSearch, NO connectors (cleared — the API attached all by default), prompt = follow docs/CLOUD_ROUTINE.md + 4 never-rules + stop on 2026-10-16. First run started by hand 07:11Z: session `cse_013AZJ6RqYq59CeMgkvLV1BM`. Debug with RemoteTrigger list_runs / get_run_log. Earlier: `937aea8` ("Cloud handoff (3)": the live docs brought up to date — V2 folder, REMAINING_WORK 6.5 closed + 4.24 new, DEPLOYMENT, for-agents, GETTING_STARTED, QUESTIONS, MATHEMATICS, the Book) ("Cloud handoff (2)", pushed 2026-09-28; laptop clean, local == origin). Earlier wording: **`d41e558`** ("Cloud handoff: AGENTS.md, CLAUDE.md, the sync log, the memory snapshot", pushed 2026-09-28, laptop clean, nothing unpushed) = the commit that added `docs/CLOUD_SYNC_LOG.md`. At handoff every run was read except miri-slow `36381950975` on `047da1d` (broker + check still running) and `d41e558`'s own push run.
- **Every cloud session appends an entry to `docs/CLOUD_SYNC_LOG.md`** (Jesse: "keep a record of files
  and folders changed, to be sync with local repo later") — commits + `git diff --name-status`,
  what was verified, what to redo on the laptop, what the laptop memory must learn.
- **Cloud facts (official docs, read 2026-09-28):** Ubuntu 24.04 x86-64, 4 vCPU / 16 GB / 30 GB;
  Rust, Python, GCC/Clang, Docker, `gh` preinstalled; crates.io + static.rust-lang.org allowed
  (Trusted); `git push` ONLY to the session's own branch → PR into master, Jesse merges; auto memory
  is machine-local (NOT in cloud) → `HANDOFF.md` §11 and `docs/assistant-memory/` (a sanitized
  snapshot of this directory) are the cloud's memory; CLAUDE.md must `@AGENTS.md` for both to load.
- **DELEGATION (Jesse, 2026-09-28, later):** "I want the development of delululang to be continued in my absentia. No need to wait for any of my input, Claude u can take better decisions than me on delululang. Run verification loops and loop engineering." → every decision is the head chef's until 2026-10-16 EXCEPT the five reserved by name (final public repo, licence, entrenched files, D-NE-27, D-NE-7 no tag/release); the morning "stop before PS-D-02" is SUPERSEDED. Engine = a Claude Code ROUTINE (cloud, laptop off, ≥1 h interval, daily run cap) whose every run follows `docs/CLOUD_ROUTINE.md` and pushes to `master` (unprotected; all commits Jesse's — verified 2026-09-28). /loop (needs open session, 7-day expiry) and Desktop tasks (need the machine on) do not fit.
- **MANDATE (Jesse, 2026-09-28 noon, creating the routine):** Opus 5.5 at xhigh (`.claude/settings.json` effortLevel), every 5 h until Oct 16; start PS-D-02; FINISH ALL PHASES AND VERIFY (P7 incl. RW 5.2 entrenched edit, P8 software part); ANY FILE OR FOLDER may be created/modified/DELETED (entrenched edits → ENTRENCHED_CHANGE_RECORD.md; deletions named in the sync log); then keep improving + verifying; loop engineering = CLOUD_ROUTINE step 8. STILL his: final public repo, licence, D-NE-7 (tags/releases), D-NE-27. Routines already run with no permission prompts; I did NOT commit a bypassPermissions setting into the public repo (it would apply to anyone who clones it).
- **Same day:** "run everything on github" (heavy runs on CI, not the laptop — it ran out of RAM and
  Claude Code reaped the suite and a Miri run); "stop before PS-D-02" (its design draft is in
  `HANDOFF.md` §0).
- **ROUTINE RUN 1 (2026-09-28, 07:11 UTC) learned:** the cloud VM has NO `gh` and the proxy refuses the
  signed log URLs → read CI with the GitHub MCP tools (`actions_list`, `actions_get`, `get_job_logs`
  `failed_only`), a Haiku sous-chef to pull lines out of a long log; run `cargo fetch --locked` before
  the suite (`egress_features` runs `cargo metadata --offline`); Survey test nodes are `test:<path>`.
  `actors_pingpong` went red on Windows twice at 1.31x — starving the VM reproduced it; D-V2-47 gave it a
  control and CI now prints its verdict. PS-D-02 built (D-V2-48): `--require-attestation HEX`,
  `delulu sandbox attest`; `delulu` reads nothing after a bare `--`. Also: PS-D and P7 COMPLETE;
  ADAPTER-SPELL-1 and ATTEST-FIFO-1 found and fixed; DELULU_CORE v0.3 (D-V2-49, entrenched — owner to
  review); P8 designed (D-V2-51); RW 7.4 closed (VS Code via apt from packages.microsoft.com); Docker
  Hub refuses the VM's pulls → `container.yml` builds on GitHub; the MCP log tool returns only a job's
  last 5,000 lines.
- **Runs 2-4 (10:11, 10:12, 10:44 UTC) did nothing:** each ended in seconds on the account's five-hour
  usage limit (`rate_limit: rejected (five_hour)`) — run 1 had spent it. Jesse paused the routine at
  14:30 UTC. Routine runs share his usage; a run that starts on an exhausted window is wasted.
- **2026-09-28 evening, on the laptop:** synced (ff `5bb39bc..dd543e5`, no CRLF, Survey ok 1,462 nodes,
  findings 0 errors, doctor 29/29 on Windows; full suite NOT run locally — CI covered every OS); then
  **NVIDIA OpenShell studied** on Jesse's commission → [[openshell-study-2026-09-28]]; PS-E is next,
  then P8 (+P8-04), then P9; the routine resumed with its connectors cleared again.
- **Models, 2026-09-28 evening (official models page):** Sonnet 5.5 (`claude-sonnet-5-5`) RELEASED; Haiku 5.5 announced, NOT released (Haiku 4.5 still newest; retirement not sooner than 2026-10-15). Agents (Jesse's ruling): Sonnet 5.5; Haiku 5.5 once released; no Haiku 4.5 meanwhile. Opus 5.5 defaults to `medium` in Claude Code — repo `effortLevel: xhigh` raises it. A cyber-flagged request re-runs Opus 5.5 → Opus 4.8 and the session STAYS there: the trailer must name it. Cloud VM: `gh` listed as pre-installed in docs but run 1 found none — check; release assets only from the attached repo.
- **Back on the laptop:** follow `docs/CLOUD_SYNC_LOG.md` *Syncing the laptop afterwards* — ff-only
  pull, CRLF check, Survey + doctor, full suite Win + WSL, each entry's redo items, then merge
  `docs/assistant-memory/` and HANDOFF §11 changes back into this directory.

**Why:** Jesse's instruction, 2026-09-28. **How to apply:** in the cloud read CLAUDE.md → AGENTS.md →
HANDOFF §0/§1/§11 → docs/assistant-memory/MEMORY.md; on the laptop after the 16th, sync first.
Related: [[delulu-v2-execution]], [[testing-repo-autopush]], [[final-public-repo-gate]],
[[agent-usage-rule-2026-09-17]].
