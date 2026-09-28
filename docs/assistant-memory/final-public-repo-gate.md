---
name: final-public-repo-gate
description: "HARD GATE (Jesse, 2026-09-17): before creating, adding as a remote, or pushing to the FINAL public DeluluLang repo, remind Jesse of the 4 pre-public decisions and the suggestions, and WAIT for his decision on each. No push until decided"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: c99ba5c5-c39b-4a1f-b406-92fbf2c601b0
  modified: 2026-09-17T02:03:04.557Z
---

**Jesse's instruction, 2026-09-17:** *"before moving to real public repo later remind to make changes
to these and remind me that time before even push happens. Do not push to new real repo (future)
unless my decision on these are given."* He also asked for **no change to the items yet**.

**Why:** the testing repo (`delululang-test`, PUBLIC since 2026-09-17, see [[delulu-github-remote]])
already exposes all four items. The final public repo is the one chance to leave them behind, and
anything pushed there is public for good.

**How to apply:** this triggers on ANY step toward the final/"real" public repo:
- creating it;
- `git remote add` for it;
- a push URL other than `delululang-test`;
- "move to the real repo" or "make the public repo".

At that point STOP. Present the four items below, each with its suggestion, and ask for his decision
on each one. Record the decisions in HANDOFF §1.1, and only then act. Never push before all four are
decided. Pushes to the testing repo are unaffected: since 2026-09-17 every commit goes there without asking
([[testing-repo-autopush]]).

1. **The banned word** ([[no-banned-word-mentions]]) is in the HISTORY.
   - Commit `0a58451` (2026-07-14) put it, with the tool's address, into its commit message and into
     `docs/design/SURFACE_ATLAS_PALETTE_ADDENDUM.md`. `f4ffd01` anonymised the file but not the
     history.
   - HANDOFF.md also spells it, twice, in the rule lines of §1 and §11.1 (added in `c37c0ad`).
   - Suggestion for the testing repo: reword those two lines.
   - Suggestion for the final repo: a FRESH history.
2. **The author e-mail**: all 342 commits (as of 2026-09-17) record Jesse's personal address as author
   and committer.
   - Suggestion for the testing repo: switch new commits to GitHub's no-reply address. Change
     `git config user.email` BEFORE turning on "Block command line pushes that expose my email", or
     pushes get refused.
   - Suggestion for the final repo: a fresh history authored with the no-reply address.
3. **The CODEOWNERS placeholder**: `@PENDING-PUBLIC-project-lead`, on 11 lines. GitHub reports 11
   "Unknown owner" errors; this is harmless without branch protection.
   - Suggestion for the testing repo: leave it.
   - Suggestion for the final repo: replace it with his real handle, AND update
     `governance.rs::pending_public_controls_are_still_marked_as_pending`, which requires the
     placeholder today.
4. **The SECURITY.md `PENDING-PUBLIC` controls**: the `security@` address and PGP key, branch
   protection, signed commits, SLSA L3, the Scorecard floor.
   - The testing repo already has GitHub private vulnerability reporting, secret scanning and push
     protection.
   - Suggestion for the final repo: decide which controls to switch on at launch. SECURITY.md is
     ENTRENCHED, so record any change in `docs/design/ENTRENCHED_CHANGE_RECORD.md`.

**Do NOT rewrite the testing repo's history.** That changes every hash from July onward, and the docs
cite hashes throughout. The gate text lives in HANDOFF §1 (rules table), §1.1 ("Before the final
public repository — a gate") and §11.1, plus REMAINING_WORK 7.5.
