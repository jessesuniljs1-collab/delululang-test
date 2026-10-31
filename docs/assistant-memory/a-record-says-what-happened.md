---
name: a-record-says-what-happened
description: "2026-10-31: RW 4.79 — every custody gate asked the custody before the program's own narrower capability, so the chain recorded `use allow` for writes and requests that never happened; judge what needs nothing from the custody first, and TELL the custody, so a monitor still counts the refusal"
metadata:
  node_type: memory
  type: feedback
  modified: 2026-10-31T08:00:00.000Z
---

When a decision is made by two judges, the one that needs nothing from the other judges first — and the record says
what happened, not what the first judge asked would have allowed.

**Why:** a capability may hold less than its node. The interpreter, a sandboxed guest's host and the WASM host each asked
the custody first, against the node's grant, so under the daemon a write or a request the capability then refused left
`use allow` in the audit chain (19 across seven calls and three routes, read red on `658daf1`), and a Guard rule was asked
about a read that could never happen. On-demand routine run 10 (D-V2-195): `prim::capability_refusal` judges first, the
custody is not asked, and it is told — one `use deny`, `refused_by: capability` — because recording nothing would have
hidden a probing program from a monitor's `denies=N/MS`.

**How to apply:** for any gate that asks a custody, list what else will refuse the call after it. A refusal that needs
the custody's answer (where a name resolved, a command's magnitudes) follows it as an overriding `deny`; one that needs
nothing from it goes first. Then read the chain of a refused call on every engine and route — `audit query --json`.
