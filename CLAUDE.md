@AGENTS.md

# Claude Code — what applies to you beyond AGENTS.md

**Roles.** The owner calls the main Claude Code session the **head chef** and its sub-agents
**sous-chefs**. The head chef writes each agent's brief, verifies every result against the binary, and
commits. The owner's standing delegation (2026-09-17 → now): *"whatever is good for DeluluLang, the
production-ready language of the future"* — keep working phase by phase without asking, except on the
owner-reserved items in AGENTS.md and at stop points he sets.

## Memory: there is no auto memory in a cloud session

Auto memory is machine-local: the owner's laptop keeps it at
`C:\Users\jesse\.claude\projects\D--nelan-DeluluLang\memory\`, and a cloud session does not have it.
**`HANDOFF.md` §11 is that memory, transcribed** — read it in full at the start of a session. When you
learn something durable (an owner instruction, a trap, a finding that must never be re-softened), write
it into `HANDOFF.md` §11 and note it in `docs/CLOUD_SYNC_LOG.md`, so it reaches the laptop's memory when
the repositories are synced.

## Survey and doctor, always

Start every session with `cargo run -p delulu-survey -- check` and `cargo run -p delulu -- doctor --check`;
ask the Survey (`impact`, `affected-by`, `query`) before any change; after the last edit run
`survey build`, `check`, `findings` and `doctor --check`; record the results in the pull request and in
`docs/CLOUD_SYNC_LOG.md`. The table in AGENTS.md says exactly when (owner, 2026-09-28: *"run and check
everything using survey and doctor"*).

## In a cloud session (`CLAUDE_CODE_REMOTE=true`)

- Ubuntu 24.04, x86-64, 4 vCPU, 16 GB RAM. `rust-toolchain.toml` pins Rust **1.96.1**; the first
  `cargo` call installs it with clippy and rustfmt (crates.io and `static.rust-lang.org` are on the
  Trusted network list). Python 3, GCC/Clang, Docker and `gh` are pre-installed.
- `git push` reaches only this session's branch → push, open a pull request into `master`, record the
  change in `docs/CLOUD_SYNC_LOG.md` in the same pull request. `master` moves only by the owner's merge.
- CI: `gh run list -R jessesuniljs1-collab/delululang-test`, `gh run view <id> --log`, a job's log via
  `gh api repos/jessesuniljs1-collab/delululang-test/actions/jobs/<job-id>/logs`. Manual runs:
  `gh workflow run ci.yml --ref <branch> -f jobs=everything|heavy|heavy-gates|miri-slow`; also
  `release.yml` (a dry run unless a `v*` tag — never push a tag), `channel-measure.yml`,
  `host-capability-probe.yml`.
- A background job can be stopped if the VM runs short of memory: run the full suite alone, `-j 4`.

## Commits

End every commit message with the model that is ACTUALLY running (check before writing it) and the
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
