---
name: shared-type-walks
description: "A substitution makes a type a graph; any checker walk that resolves vars as it goes (occurs, unify, apply_type, opaque_for) makes it an exponential tree again — RW 2.3, D-V2-179, PR #56 (2026-10-23)"
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-23T05:25:32.801Z
---

`let aN = Pair { l: aN-1, r: aN-1 }` is N small types through type variables and 2^(N+1)-1 parts written out. 22 lines took `delulu check` 5.8 s (release), doubling per line. Four separate walks treated it as its tree, each found only after fixing the one before: the DIR side table (`node_types` via `apply_type`), the occurs check, `unify_type` (`a == b` of two chains: 261 s), and `opaque_for` (re-applied at every level). Fix (D-V2-179): explored-set occurs check, a pair cache of bound variables in unify (sound: bindings only grow, failures end the call with `?`), `opaque_for` given the type applied once; `apply_type` budgeted at 100,000 parts per type and 2,000,000 per module for types over 1,000 parts; past either, DL0211 (reused, fail-closed), named in node order by a counted replay of the settle.

**Why:** RW 2.3 had been deferred as "an RFC" because a bound is language-visible; delegation made it the head chef's call. A per-binding size check was considered and rejected: a type can be checked while its leaves are unbound, and the leaves bound to large types later, so the sizes multiply.

**How to apply:** when timing a checker fix, time a twin (`a == b`), a field read and a long reference list of the shape, not only the shape. Measure ordinary-program overhead against current master by alternating binaries, best of 5. Here it was about 3%, found by elimination; the first antivirus scan of a freshly copied binary adds ~1 s to its first run on this laptop. A mutant can survive because the change was not needed: the field re-visit was cut off by `visiting`, and its claimed second was really the per-part re-apply. Related: [[a-latency-may-be-a-deadline]], [[pinned-refusal-names-its-reason]].
