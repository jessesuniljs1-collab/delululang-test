# Agent test campaign, 2026-10-24 — status (head chef)

**Commission (Jesse, 2026-10-24):** "Use haiku 5.5 or sonnet 5.5 as agents building software and writing programs and other
things using delululang with various test conditions like no restrictions full access, limited network and permission
access, only given access, etc. for testing everything currently being build in delululang." Then: "While running agents
for testing DeluluLang, U will be having delulu authority, Only u should be able to change anything that agents can do,
Agent themselves shouldn't able to change access." Then: "save everything agents are doing to storage … I don't want
anything to be lost".

## The binary
`delulu 1.0.0` built from `laptop/wasm-post` `41f1dc6` (master `b40de37` + PR #64), debug, Windows 11. Frozen copy in the
authority dir so rebuilds of the checkout never change what the agents run.

## Authority — the head chef's, not the agents'
- Broker daemon, state in the head chef's authority dir (scratchpad `authority/state`), started in **strict anchored-root
  mode** (`broker start --require-anchored-roots <anchor>`): an unsigned root is refused DL1421 (checked).
- Anchor and holder ed25519 keys, the root certificate (`--fs-read/--fs-write` = the dogfood dir, `--net
  127.0.0.1:GET,HEAD,POST,PUT,PATCH,DELETE`, effects Read, Write, Net, Clock, Rand, Async, Declassify, 12 h) and the Guard
  owner code are held by the head chef only; none is copied to any snapshot.
- Each test condition is a `--multi` lease (8 h) delegated under that root, or (B-secret only) a head-chef-owned
  `--grant` list. Conditions: A-full, A-console; B-console, B-read, B-rw, B-net-get, B-net-api, B-secret, B-wide;
  C-none, C-console, C-clockrand; and `none`. Descriptions: `authority-public/conditions/*.desc` in the mirror.
- **The wrapper `dl`** (`authority-public/dl`) is the agents' only way to run delulu: it refuses every flag that names
  authority (`--grant`, `--lease`, `--break-glass`, `--broker`, `--owner`, `--test-authority`, …), runs `run <condition>`
  with that condition's lease, opens read-only and proposal verbs of `grants`/`guard`/`audit`, and refuses `test` on a
  package that declares an effectful `[test-authority]` ceiling (the agent would be writing its own access).
- **The gate** (`authority-public/agent_gate.py`, a Claude Code PreToolUse hook in the checkout's
  `.claude/settings.local.json`, not committed): the main session passes; a subagent's call is refused when it reaches
  the authority dir or its keys/leases/owner code, runs delulu other than through `dl`, names a grant/lease/owner flag,
  sets a DELULU_ variable, changes accounts or permissions (wsl, runas, icacls, …), touches Claude Code settings or the
  repository, writes outside the dogfood dir, writes a script that would do any of those, or uses Agent/Web/etc.
  Every decision is logged (`gate.log`).
- **Falsified 2026-10-24 00:54 IST** with a Haiku test agent: `dl run A-console` allowed; `dl run … --grant`, `cat
  ../../authority/owner.code` and Read of `owner.code` denied before running — confirmed in the gate's own log, which
  shows the subagent's calls carry `agent_id` and `agent_type` while the main session's carry neither.
- Residual, stated plainly: the gate is pattern-based and the agents run as the same OS account. The boundary against a
  determined same-account process is a second OS account (`docs/DEPLOYMENT.md` Tier 2), which this laptop does not
  have set up. DeluluLang itself also holds: a `--lease` run refuses any local `--grant` (checked).

## Agents (launched 2026-10-24 ~00:57 IST, in the background)
| Agent | Model (alias requested) | Workdir | Conditions | Brief |
|---|---|---|---|---|
| A — the builder | `sonnet` (Sonnet 5.5) | `A-builder` | A-full, A-console, none | `_briefs/COMMON.md` + `_briefs/A.md` |
| B — least authority | `sonnet` (Sonnet 5.5) | `B-least-authority` | B-* , none | `_briefs/COMMON.md` + `_briefs/B.md` |
| C — zero authority | `haiku` (Haiku 5.5) | `C-zero-authority` | none, C-none, C-console, C-clockrand | `_briefs/COMMON.md` + `_briefs/C.md` |
**Models actually run, read from the transcripts' own `model` fields (2026-10-24 01:15 IST):** A `claude-sonnet-5-5`
(252 messages so far), B `claude-sonnet-5-5` (230), C `claude-haiku-5-5` (155), the gate test `claude-haiku-5-5` (13).
Transcripts: `~/.claude/projects/D--nelan-DeluluLang/eb34182b-…/subagents/agent-<id>.jsonl`, copied by the saver (the
session's task `.output` files are empty stubs — the first saves copied those, fixed 01:14 IST).
Each writes `REPORT.md` in its workdir as it goes (the sub-agent Write tool refused a report file: C's report was saved by
the head chef from its final message; B wrote `REPORT.txt`). Their findings are unverified until the head chef re-runs them
against the binary; nothing reaches the repository before that.

## Saving (nothing lost)
`D:\nelan\dogfood-campaign\save.sh`, Task Scheduler `DeluluLang-dogfood-save`, every 5 min for 24 h, outside Claude:
the whole dogfood dir mirrored to `D:\nelan\dogfood-campaign\mirror\2026-10-24\`; the agents' transcripts copied to
`D:\nelan\DeluluLang-agent-transcripts\2026-10-24-dogfood\` (local only — reasoning never goes to the repository); a
filtered snapshot (no binaries, files < 5 MB, no keys/tokens/owner code/secret value, not the banned word) pushed to
branch `autosave/dogfood` of the test repository. Never merge it. Log: `D:\nelan\dogfood-campaign\save.log`.

## If a session or an agent stops
- An agent killed by a usage limit: resume it with SendMessage to its id from the SAME session (never respawn).
- A new session: the agents' work is in the mirror and on `autosave/dogfood`; re-launch an agent with its brief and
  tell it to continue from its own REPORT.md. The leases expire ~09:00 IST 2026-10-24 and the root at 12:48 IST; a
  new session mints new ones (a new anchor, a new root, new leases) — nothing in this campaign depends on these keys.

## Found by the head chef while setting up
- **CERT-SECRET-1 (RW 4.80), fixed on `laptop/cert-secret`:** under strict anchored roots no lease could hold a
  broker-held secret — `grants certify` could not write secrets, audiences, `declassify`, foreign libraries or a
  budget, though the certificate format carries them all. Witnessed red, fixed, mutant red.
- **`toolchain --json` described `net` without methods** a day after D-V2-182 — fixed on the same branch.
