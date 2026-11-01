---
name: gate-records-before-the-check
description: "NET-PATH-RECORD (2026-10-21, D-V2-169) — a custody gate that records its decision BEFORE the effect's own check attests an allow that check then refuses; list the later check's refusals and ask what the record says for each"
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-21T16:31:33.804Z
---

Under the daemon, the custody gate (interp/channel/wasm `check_at`) runs BEFORE the effect's own first check (`prim::url_verdict`). The broker judged a `net` use by host only, so a request for a path outside the grant's patterns was recorded `use allow` and then refused DL0904 at the call. The chain said "allowed" for a use that never happened, with no path to tell which. Witnessed on `678dee6`'s binary (seq 9 `use allow`, run refused).

Fixed in D-V2-169: the use carries `url_path` (`prim::request_path`, no query), the broker judges it with `netscope::judge_all` (`Denial::NetPath`, DL0904 in the call's words), and the payload names it beside `op`; `target` stays the host (OCSF reads it as hostname). Mutants M703–M717.

**Why:** a record written by an earlier gate is only as true as the gate's judgment; a later, stricter check silently makes it false.

**How to apply:** for every record a gate writes, list the refusals a LATER check can make and ask what the record says for each. Related: [[artifact-claim-1]] (list what an integrity check excludes), [[skip-branch-verification-rule]].
