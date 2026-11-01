---
name: walk-every-part-of-a-statement
description: "INDEX-TARGET-1 (2026-11-02) — every walk of `xs[e] = v` read only `v`; the checker called a printing fn pure, rcap missed a consume, --within said within; walk what the runtime evaluates"
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-11-01T19:18:10.490Z
---

INDEX-TARGET-1 (D-V2-205, RW 2.29, found 2026-11-02 while writing DL0414 on `laptop/unread-lets`): `xs[e] = v` evaluates `e`
at run time, but all twelve walks of `Stmt::Assign` read only `v`. Since v1.0.0:
- the checker (`check_lvalue` ignored `_idx`): a fn with no row printed through `xs[noisy(c)] = 9`, `authority` said pure fns;
- rcap + its loop pre-scan: `ys[give(s, consume xs)] = 1` then `xs.len()` accepted (data race);
- `--within`, `--grants`, the foreign report, the atlas, the plugin list, the LSP: all blind there.
The capability check held at run time. Fix: `LValue::index_exprs` (innermost first, after the value); every walk uses it.

**Why:** a walk written `Assign { value, .. }` or `Let { value, .. } | Assign { value, .. }` silently skips the target's
expressions — same family as [[walk-catch-all-swallows]] and [[two-engines-agreeing]].

**How to apply:** when writing or reviewing ANY statement walker, list what the interpreter evaluates for that statement
(`interp.rs` `exec_stmt`/`assign`) and walk each part. Search key: `Assign { value, .. }`. DL0414's read marks must count
a read in an index, and the base of `xs[i] = v` is read (records/lists are shared `Rc<RefCell>`).
