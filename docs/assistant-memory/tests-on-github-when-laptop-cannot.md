---
name: tests-on-github-when-laptop-cannot
description: "Jesse 2026-10-19: when the laptop cannot run or finish tests (low memory, too long, other OS), run them on GitHub Actions automatically, see them through to green, review the results"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-19T08:10:14.640Z
---

Jesse, 2026-10-19: *"Whenever tests cannot be run locally on lap or run cannot be completed from lap for any reason,
automatically run them using GitHub Actions runners or any other alternatives instead. Ensure the tests complete
successfully and review the results."*

**Why:** on 2026-10-17 Claude Code stopped the laptop's full suite for RW 6.1 for low system memory before any test ran,
and I stopped to ask him whether to re-run it. He does not want work to wait on that question.

**How to apply:** without asking, push the branch to `origin` (delululang-test only — [[test-repo-only-2026-10-17]]) and
`gh workflow run ci.yml --ref <branch> -f jobs=everything` (its `test` matrix = the full suite on 3 OSes), or
`witness.yml` for one target/OS. Read every job to its end; land on `master` only on green; the record names the run and
says the laptop suite did not run. Targeted tests still run locally. Recorded in `HANDOFF.md` §11.1 and `AGENTS.md`
step 3. Related: [[autosave-task-2026-10-17]].
