@AGENTS.md

# Claude Code — what applies to you beyond AGENTS.md

**Roles.** The owner calls the main Claude Code session the **head chef** and its sub-agents
**sous-chefs**. The head chef writes each agent's brief, verifies every result against the binary, and
commits. The owner's standing delegation (2026-09-17 → now): *"whatever is good for DeluluLang, the
production-ready language of the future"* — keep working phase by phase without asking, except at stop
points he sets. **Since 2026-10-17 the owner-reserved items are delegated too**, up to and including the public release
(AGENTS.md, *The owner's delegation*; his commission `docs/design/DeluluLang_Autonomous_Development_Prompt.md`).

## Memory: there is no auto memory in a cloud session

Auto memory is machine-local: the owner's laptop keeps it at
`C:\Users\jesse\.claude\projects\D--nelan-DeluluLang\memory\`, and a cloud session does not have it.
**`HANDOFF.md` §11 is that memory, transcribed** — read it in full at the start of a session. When you
learn something durable (an owner instruction, a trap, a finding that must never be re-softened), write
it into `HANDOFF.md` §11 and note it in `docs/CLOUD_SYNC_LOG.md`, so it reaches the laptop's memory when
the repositories are synced.

## GitHub first, then the Survey and doctor

**Since the 2.0.0 launch (D-V2-153) the repository is `github.com/jessesuniljs1-collab/DeluluLang` and its branch is
`main`:** read every `delululang-test` below as `DeluluLang`, and every `master` as `main`. `delululang-test` is the private
archive of the development history before 2.0.0 — never pushed to again; a commit hash or run id in a record dated
before 2.0.0 refers to it.

Every session — laptop or cloud — starts with `git fetch origin` and reads what came in (`HEAD..origin/main`, the
`origin/claude/*` branches, their CI) before any edit: the on-demand routine pushes while the laptop is away (owner,
2026-10-17; `AGENTS.md` *Read first* 0).

## Survey and doctor, always

Start every session with `cargo run -p delulu-survey -- check` and `cargo run -p delulu -- doctor --check`;
ask the Survey (`impact`, `affected-by`, `query`) before any change; after the last edit run
`survey build`, `check`, `findings` and `doctor --check`; record the results in the pull request and in
`docs/CLOUD_SYNC_LOG.md`. The table in AGENTS.md says exactly when (owner, 2026-09-28: *"run and check
everything using survey and doctor"*).

## In a cloud session (`CLAUDE_CODE_REMOTE=true`)

- Ubuntu 24.04, x86-64, 4 vCPU, 16 GB RAM. `rust-toolchain.toml` pins Rust **1.96.1**; the first
  `cargo` call installs it with clippy and rustfmt (crates.io and `static.rust-lang.org` are on the
  Trusted network list). Python 3, GCC/Clang, Docker, Node 22 and `xvfb-run` are pre-installed. **`gh`:**
  the official docs list it as pre-installed and authenticated through the GitHub proxy, but the first
  routine run found none — check with `command -v gh`, and read CI with the GitHub MCP tools either way
  (`docs/CLOUD_ROUTINE.md` step 3). The VM can fetch GitHub release assets only from this repository.
  Run `cargo fetch --locked` once before the suite: `egress_features` needs every platform's crates.
- **The owner, 2026-10-17:** *"Not just routine or scheduled task runs, I will be using claude code cloud sessions also occasionally using test GitHub repo … I don't want delululang development and testing and addition of new features to be stopped just because of my lap not available and I am busy with other projects."* So an **interactive cloud session** with no narrower request from him continues
  DeluluLang exactly as a routine run does: `docs/CLOUD_ROUTINE.md` from step 1, its work order step 4 (the release
  program, then features, tests and fixes), every verified slice pushed and recorded. Development never waits for the laptop.
- **A routine run** follows `docs/CLOUD_ROUTINE.md` and pushes to `master` — since 2026-10-17 the on-demand routine
  `trig_01HFG6usfzmDAPQigiDu9DLq` (created paused; the owner switches it on when he cannot use the laptop; Opus 5.5 at
  xhigh through `.claude/settings.json`; it does not launch — D-V2-146). **An interactive
  cloud session** can push only its own branch → push, open a pull request into `master`, and record the
  change in `docs/CLOUD_SYNC_LOG.md` in the same pull request. The next session — laptop, routine or cloud — reads that
  branch's CI and merges it (`docs/CLOUD_ROUTINE.md` step 1, every branch), so no session's work is stranded.
- CI, where `gh` exists (the laptop): `gh run list -R jessesuniljs1-collab/delululang-test`, `gh run view <id> --log`, a job's log via
  `gh api repos/jessesuniljs1-collab/delululang-test/actions/jobs/<job-id>/logs`. Manual runs:
  `gh workflow run ci.yml --ref <branch> -f jobs=everything|heavy|heavy-gates|miri-slow`; also
  `release.yml` (a dry run unless a `v*` tag — never push a tag), `channel-measure.yml`,
  `host-capability-probe.yml`; and **`witness.yml`** — one test target on one runner at any ref
  (`-f os=macos-latest -f target=<test file> -f filter=<name>`), for a macOS- or Windows-only witness
  read red on a branch before its fix reaches `master` (`docs/CLOUD_ROUTINE.md` step 5). macOS
  and Windows code is linted in the VM first with `scripts/check-other-os.sh`.
- A background job can be stopped if the VM runs short of memory: run the full suite alone, `-j 4`.

## Commits

End every commit message with the model that is ACTUALLY running (check before writing it — after a
safety-classifier fallback notice, Opus 5.5 has become Opus 4.8 for the rest of the session) and the
session link:

```
Co-Authored-By: Claude <Model name> <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/<session id>
```

In a cloud session the link is `echo "https://claude.ai/code/${CLAUDE_CODE_REMOTE_SESSION_ID/#cse_/session_}"`.
Write the message to a file and use `git commit -F` (heredocs have mangled backslashes and non-ASCII).

## Sub-agents

The rules are in AGENTS.md. Two Claude-specific ones: an agent killed by a usage limit is resumed with
`SendMessage` to its ID, never respawned; and agent notes are verified by the head chef before a word
of them reaches the repository.
