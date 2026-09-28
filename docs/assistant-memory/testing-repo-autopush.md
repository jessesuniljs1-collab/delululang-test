---
name: testing-repo-autopush
description: "Standing permission (Jesse, 2026-09-17): commit locally AND push every commit to the GitHub TESTING repo (delululang-test) without asking; local and GitHub must stay in sync. Does NOT cover the final public repo"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: c99ba5c5-c39b-4a1f-b406-92fbf2c601b0
  modified: 2026-09-17T02:02:55.103Z
---

**Jesse, 2026-09-17:** *"for push everything and anything to the github test repo, no need to ask my
permission everytime. commit locally and commit to github test repo always, both should be in
sync."*

**Why:** the testing repo (`origin` = `delululang-test`, public; see [[delulu-github-remote]]) exists
for cross-OS CI and cloud editing. A local commit that is not on GitHub is invisible to CI and to the
cloud, and waiting for a "push it" after every commit only slowed him down.

**How to apply:**
- After every verified commit, `git push origin master` immediately, without asking. Then confirm
  sync: `git ls-remote origin` matches every local branch and tag, and the tree is clean. Say in the
  report that it was pushed.
- Keep the usual pre-commit checks: Survey rebuild, the gates, `delulu doctor`, no CRLF, and no
  banned word ([[no-banned-word-mentions]]).
- **Never put a literal GitHub skip token in a commit message** unless skipping CI is intended. It
  silently skips the push run anywhere in the message.
- Never rewrite pushed history, and never force-push. The docs cite commit hashes.
- **This permission covers ONLY the testing repo.** The final public repo stays behind the hard gate
  in [[final-public-repo-gate]]: remind him of the four decisions and wait for each. No other remote
  may be added.
- If a push fails (auth, rejection, divergence), do not force it. Report it and resolve it with him.
