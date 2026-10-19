---
name: a-fix-for-one-node-asks-every-node
description: "2026-10-19: RW 4.57 — RW 4.51 ended a plugin's node with its run, and the run's own root, the widest, outlived every run; when a fix asks whether one kind of node outlives its holder, ask it of every node that holder holds, the parent first"
metadata:
  node_type: memory
  type: feedback
  modified: 2026-10-19T00:45:00.000Z
---

When a fix answers "does this node outlive its holder?" for one kind of node, list EVERY node the same holder holds and
ask each — the parent first.

**Why:** RW 4.51 (routine run 16) revoked a run's plugin nodes at its end; device nodes were already revoked. The run's
own root — issued for it alone by `run --grant` under the daemon, and the parent of both — stayed `[live]` after every
run, and after a kill everything below it did too (RW 4.57, measured 2026-10-19 by the on-demand routine's second run;
D-V2-157 closed it: revoked at the run's end, reaped by a broker dead-man when the run dies first).

**How to apply:** `grants list --json` after the scenario (clean end, budget stop, kill) and read the state of every node
whose holder is the run, not only the kind the finding named.
