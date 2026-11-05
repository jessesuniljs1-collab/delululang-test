---
name: measure-child-memory-from-c
description: "Hidden-complexity audit second pass 2026-11-05 — ru_maxrss read by a Python parent includes the parent's pre-exec high-water mark; libpython's +4.2 MiB per command read as +0.4 MiB; every RSS figure in the audit was corrected with a C launcher"
metadata:
  node_type: memory
  type: feedback
  originSessionId: session_01Deh6W9bJdfxAeMARtXU9R9
  modified: 2026-11-05T12:30:00.000Z
---

Measure a child process's peak memory with a C fork-and-exec launcher, or from its `VmHWM`, never with `os.wait4`
from a Python parent.

**Why:** on Linux, `exec` carries the forked parent's memory high-water mark into the child's `ru_maxrss`. The
hidden-complexity audit timed `delulu` from a Python harness:
- every small command read 9–15 MiB;
- libpython's cost looked like +0.4 MiB on `--version`, when it is +4.2 MiB on every command.

The verification sub-agent caught it, and the earlier audit's `rss.c` confirmed it: `--version` measures 9.5 / 5.3 MiB
with and without Python.

**How to apply:** use `rss.c` (fork, `execvp`, `wait4`) or read `/proc/<pid>/status` `VmHWM`. Wall-clock timing from
Python is fine.
