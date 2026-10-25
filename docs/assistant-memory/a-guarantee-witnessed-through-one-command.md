---
name: a-guarantee-witnessed-through-one-command
description: "2026-10-25: RUN-PIN-1 — the supply-chain gate (DL1001, DL1009) was witnessed through `build`, said to hold before any code runs, and `run`, `test`, `build --target wasm`, `deploy plan`, `publish`, `add` and `sandbox policy` all skipped it; drive a guarantee's witness through every command that reaches what it guards"
metadata:
  node_type: memory
  type: feedback
  modified: 2026-10-25T10:30:00.000Z
---

When a guarantee says "before X happens", list every command that can make X happen and drive the witness through each,
not through the one the guarantee was built in.

**Why:** the xz scenario's test (`provenance.rs`) drove `delulu build` and the specification, the book and the test's own
doc comment said the pin refuses a dependency "BEFORE any code runs". Only `build_workspace` and `cmd_lock` called
`check_pins`; `run` on every engine, `run --sandbox`, `build --target wasm`, `test`, `deploy plan`, `publish`, `add` and
`sandbox policy` each had a loader that checked types alone. On-demand routine run 6 found it while verifying a repair
(A2) by running the repaired package: the unrepaired one ran too. The fix is one function, `supply_chain_gate`, called on
every route (D-V2-186), and a witness that drives seven routes.

**How to apply:** for a gate, `grep` the function that loads what it guards (here `resolve_workspace(`) and read each
caller for the gate; for each one that runs, ships or plans the thing, add a row to the witness. Verify a repair or a fix
by doing what the user does next with its result, not by re-reading the diagnostic.
