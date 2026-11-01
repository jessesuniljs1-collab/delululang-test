---
name: static-verdict-skips-a-dimension
description: "authority --within / deploy plan judged only the MINT of a capability, not what the program then does with it: secret sends' audiences (D-V2-180) and requests' paths (D-V2-181) answered 'within' though the run refuses — PR #57, 2026-10-23"
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-23T05:58:29.186Z
---

`--within`'s verdict decided each capability-minting call (`Ask::Mint`) and never the uses the runtime judges later:
- `http.get_with_secret` goes only to the grant's audience (P9-04 (c));
- a request's path is judged against a path-scoped `net` entry (P9-05 (a)).

So `--within` and `deploy plan` (same verdict, D-V2-176) said *within* for boundaries under which the run refuses. `--grants` even proposed `secret:NAME=VALUE`, under which every send is refused. The runtime was fail-closed throughout; the static answers were wrong. Fixed: `Ask::Secret { name, to }`, `Ask::SecretSend`, `Ask::Request { url }`, decided by the runtime's own readers (`parse_audience`, `prim::host_of`/`path_of`, `netscope::judge_all`). Requests need the checker's per-node types, so they are set only by verdict callers (a file, a package by unit, `deploy plan`'s `checked_package`).

**Why:** the RW 2.11 row said only "propose the audience in --grants"; building it surfaced the verdict gap. Asking "what else does the runtime judge at use time that the static verdict doesn't?" found the second one, paths, at once.

**How to apply:** when a static tool answers "within/approved", list every check the runtime makes at USE time, not only at the mint, and test each with a boundary that grants the mint but not the use. Run mutants with `--no-fail-fast` (without it cargo stops at the first red target and the second goes unread). A flag whose mutant survives may be dead code: here `typed` filtered requests out of `--grants`, whose walk has none, and it was removed. Related: [[skip-branch-verification-rule]], [[gate-records-before-the-check]].
