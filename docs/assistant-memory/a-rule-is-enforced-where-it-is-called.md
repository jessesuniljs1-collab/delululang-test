---
name: a-rule-is-enforced-where-it-is-called
description: "Third verification pass 2026-11-05 — R-Get (p.get's type and row check) existed as unit-tested functions since July but no program path called them; a ! {Load, Read} program wrote a file through a ! {Write} plugin export (RW 4.95)"
metadata:
  node_type: memory
  type: feedback
  originSessionId: session_01Deh6W9bJdfxAeMARtXU9R9
  modified: 2026-11-05T08:00:00.000Z
---

A security rule written as a function is enforced only where something calls it. Find its callers before you count
it as enforced.

**Why:** R-Get (*"`p.get[F]` succeeds only if `row(export) ⊆ row(F)`"*) arrived in `33f08d1` (2026-07-15) as
`r_get_verified` and `r_get_contained`, unit-tested, in a commit titled *"R-Get at runtime"*. Neither the checker nor
the interpreter ever handed `F` to the runtime, and `plugin_get` (`interp.rs` lines 1902–1919) checks only that the
name exists. The laundering suite leaves F-1 to *"the runtime plugin loader"*, and the CLI tests take every export at
its honest type. So each layer assumed another covered it. The third verification pass wrote a host whose `main` is
`! {Load, Read}`: it took a Verified plugin's `! {Write}` export as `! {}` and wrote a file. `check`, `authority` and a
package's DL1009 ceiling all passed (RW 4.95). The grant still bounded the write.

**How to apply:**
- For every rule a spec states, `grep -rn 'fn_name('` for its callers and name the one a program's run passes through.
- Keep one CLI witness that drives the rule the way a program does. For R-Get: the narrower annotation must be refused
  and must write nothing.
- When a check is *"deferred to run time"*, verify that the runtime receives what the check needs. An annotation that
  is erased before run time cannot be checked there.
