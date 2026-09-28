# AGENTS.md — instructions for every AI agent working in this repository

Claude Code reads this file through `CLAUDE.md` (which imports it); Codex, Cursor and other agents read
it directly. It is the short form. **`HANDOFF.md` is the full briefing and the authority** — its §1
(standing rules), §0 (the cloud period) and §11 (the project's memory, written down) outrank anything
you remember from elsewhere.

## Read first, in this order

0. **Run the Survey and `doctor`** (`cargo run -p delulu-survey -- check`,
   `cargo run -p delulu -- doctor --check`) — the section below says when and why.
1. `HANDOFF.md` — §0, §1, then §11.
2. `docs/DELULULANG_V2/V2_PHASE_STATUS.md`, then the NEWEST entry of `docs/DELULULANG_V2/V2_LOG.md`.
   Decisions taken under the owner's delegation are `V2_DECISION_LOG.md` (`D-V2-nn`).
3. `docs/CLOUD_SYNC_LOG.md` — what earlier cloud sessions changed.
4. `docs/REMAINING_WORK.md` — everything open, each row checked against the binary.

DeluluLang is a statically typed, authority-and-effect-typed language (a Rust workspace of 13 crates):
**a program can do nothing except what it was explicitly handed.** Effects are in the type, capabilities
are values, authority is computed from the code. A sandboxed program runs as a guest that holds no
authority; the host performs every effect under grants, a lease and the Guard.

## Hard rules — never break these

- **Push only to `origin`** (`github.com/jessesuniljs1-collab/delululang-test`, a public TESTING
  repository). Never force-push, never rewrite pushed history (documents cite commit hashes). **Never
  create, add or push to the project's final public repository** — first put the four decisions in
  `HANDOFF.md` §1.1 in front of the owner and wait for each.
- **Never put CI's skip token in a commit message** — it silently skips the push run.
- **A word the owner banned** (named in `HANDOFF.md` §1's rule table) appears nowhere — no file, commit
  or product surface. Do not copy or restate the two existing rule lines that spell it.
- **Owner-reserved — change only with the owner's word:** `docs/design/CONSTITUTION.md`,
  `DELULU_CORE.md`, `STABILITY.md`, `SOUNDNESS_AUDIT.md`, `SECURITY.md`, `rfcs/`, `docs/security/`,
  `crates/delulu-check/tests/laundering.rs`, `crates/delulu-conform/`, `tests/conformance/witnesses.toml`
  (CODEOWNERS "entrenched"); the licence; the final public repository; RW 4.19 (D23); D-NE-27 (never
  commit or ship a built GPL kernel image); D-NE-7 (publication — nothing is released without the
  owner's `v*` tag). Before editing anything under `docs/design/`, check
  `cargo run -p delulu-survey -- query doc:<path>` for ENTRENCHED.
- **Never delete a `.md` file.** Move superseded documents to `docs/archive/`, never remove them.
- **No new `DL` diagnostic codes** without the owner — refuse in words through existing paths.
- **Harden, never redefine, Authority and the Guard.** Closing a hole is welcome; changing what they
  mean is the owner's.
- **Never claim what did not run.** "Prepared", "activated" and "green" are different words. A CI run
  counts only once its result is READ and recorded. Never fabricate evidence; record failed runs.
- **Decisions you take under the owner's delegation** are written as
  `D-V2-nn — … — TAKEN (head chef, <date>, under the owner's delegation)`, never as his rulings.
- **Stop points the owner set are binding:** as of 2026-09-28, *do not start PS-D-02* until he says so.

## Run and check everything with the Survey and `doctor` (owner, 2026-09-28)

The Survey (`delulu-survey`) is the repository's map of itself — every edge cites a `file:line`; `doctor`
is the one command that answers "is this healthy?" (environment, security posture, and the map's
freshness and integrity). **Use both at every one of these points, and write their results down:**

| When | Run | It must say |
|---|---|---|
| **Start of every session** | `cargo run -p delulu-survey -- check` and `cargo run -p delulu -- doctor --check` | `ok: the Survey matches the tree`; `ok: N check(s) passed`. Anything else is the first thing to fix or record. |
| **Before any change** | `cargo run -p delulu-survey -- impact <id>` (what breaks), `affected-by <id>` (what it rests on), `query <id>` (both directions, and ENTRENCHED) | read what it reaches before editing. IDs: `crate:delulu-check`, `mod:crates/delulu/src/guest.rs`, `doc:README.md`, `code:DL0501`, `ruling:S10-D64` |
| **Reviewing a branch** | `cargo run -p delulu-survey -- diff <base>` | every changed path and the nodes it reaches — also the file list for `docs/CLOUD_SYNC_LOG.md` |
| **After the LAST edit** | `cargo run -p delulu-survey -- build`, then `-- check`, then `-- findings`, then `cargo run -p delulu -- doctor --check` | the map fresh; **0 errors** in `findings` (a warning is for a human to judge); all doctor checks pass |
| **Before every commit and pull request** | `survey check` and `doctor --check` again | their results go into the pull request and the sync-log entry |
| **After syncing the laptop** | the same, on Windows and in WSL | |

`doctor` exits 0 when healthy, 1 when a problem remains, 2 on a bad invocation; `--check` never writes;
`--json` gives one envelope. Never quote its check count as a fact — it varies by machine. A stale map
fails `doctor_cli` and the Survey's freshness test: regenerate it after the last edit, never before.

## The cloud period (2026-09-28 → 2026-10-16) — `HANDOFF.md` §0

- A cloud session can `git push` **only to its own working branch**. Commit and push there, open a
  pull request into `master` (CI runs on pull requests), and say in it what was verified. The owner
  merges. Never force-push.
- **Record every change in `docs/CLOUD_SYNC_LOG.md`** — commits, and every file and folder added,
  modified, deleted or renamed (`git diff --name-status <base>..HEAD`) — in the same pull request, so
  the local repository on the owner's laptop can be synced later. Use the template in that file.
- **Heavy runs belong to GitHub** (owner, 2026-09-28: "run everything on github"): the multi-OS
  matrix, Miri, `heavy-gates`, the release dry run. The cloud VM is Linux only (Ubuntu 24.04, x86-64,
  4 vCPU, 16 GB) — no Windows, no macOS, no KVM; CI covers those. Run one heavy job at a time, with
  `-j 4`.

## How every change is made

1. **Survey and `doctor` first** (the table above): `survey check` and `doctor --check` at the start,
   then `survey impact <id>` for the blast radius of what you are about to change.
2. **Witness a defect failing before you fix it**, and **falsify every new test** (reintroduce the
   defect, watch it go red). A gate that cannot fail is not a gate.
3. `cargo clippy --workspace --all-targets -- -D warnings` (zero warnings) and the tests —
   `cargo test --workspace --no-fail-fast -j 4`. **Freeze the tree while a suite runs.** Read cargo's
   own exit code, never a pipeline's.
4. **Regenerate the Survey LAST** — `cargo run -p delulu-survey -- build`, `-- check`, `-- findings`
   (0 errors) — then `cargo run -p delulu -- doctor --check` (all checks must pass). A stale map fails
   `doctor_cli` and the freshness test.
5. **Record:** a `V2_LOG.md` entry (what, why, evidence), `CHANGELOG.md` for anything a user sees, a
   `D-V2-nn` for a decision, `V2_PHASE_STATUS.md`, `REMAINING_WORK.md`, `docs/CLOUD_SYNC_LOG.md`, and
   `HANDOFF.md` §11 for any durable lesson. Every new markdown file needs a row in
   `docs/REPOSITORY_STRUCTURE.md` §5 (a test enforces it).
6. Commit, push, **read the CI run**, record it.

## Running sub-agents — lessons already paid for

- Use an agent only where it adds real, independent value; the owner watches usage. Testing passes
  may use Haiku 4.5 and Sonnet 5 (owner, 2026-09-27); implementation agents are Opus or Sonnet.
- **Judge an agent by its logs, never its summary.** Haiku overclaimed; one "microVM TTL defect" was
  the agent's own test mistake; one "dead delegate" was really a stale secret store. Re-run every claim
  against the CURRENT binary before it is used or recorded.
- **Read an agent's "non-blocking notes".** Two of them were real defects (JSON-EXIT-1,
  SANDBOX-CPU-LATE-1).
- **The brief forbids writing into the repository**; agents work in a scratch directory outside the
  checkout. One agent wrote into the repo root anyway — check `git status` after every agent.
- **On a usage-limit kill, resume the SAME agent** (its ID, by message) — never respawn; its work is on
  disk. On a session limit, stop agents gracefully and save progress, decisions, pending tasks and
  outputs to the phase's `.md` files.
- **Never save or reproduce an agent's chain-of-thought** — findings and decisions only.
- Run agents on **different operating systems**: Windows found the 8.3-name alias, Linux the alias
  race, and each missed the other's.

## Traps that cost real time

- A security decision made on an **unnormalized spelling** (relative path, `..`, 8.3 short name,
  junction, case, dangling link) is walked past by another spelling of the same thing. Ask: what else
  spells this?
- A lock that is **taken over after a timeout** is a race, not a recovery (AUDIT-LOCK-TAKEOVER-1): a
  slow holder is not a dead one. Use the OS lock.
- **Measure an OS limit, don't trust its name**: Windows' job time limit fires late (SANDBOX-CPU-LATE-1).
- **Miri is ~100× slower:** shrink a test under `cfg!(miri)` only where its size is not the witness;
  ignore under Miri a test whose witness is the wall clock.
- A wall-clock test failing only on CI may be measuring the RUNNER: reproduce by starving it, never by
  lengthening sleeps.
- macOS: unix-socket paths are limited to 104 bytes; a socket answers `EOPNOTSUPP`, not `ENXIO`; its
  clock is microsecond-grained (add a counter to scratch names). Windows: 8.3 names and junctions.
