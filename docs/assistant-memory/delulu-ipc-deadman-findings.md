---
name: delulu-ipc-deadman-findings
description: "IPC-1 + DEADMAN-1 (multi-agent red team of the untested surfaces, 2026-08-08): the broker had no IPC read timeout, so a same-uid stall hung the daemon (denying e-stop) AND blocked the dead-man's probe forever on Unix (fail-open). Both fixed by Connection::set_read_timeout. WASM clean."
metadata: 
  node_type: memory
  type: project
  originSessionId: c99ba5c5-c39b-4a1f-b406-92fbf2c601b0
  modified: 2026-08-08T16:31:11.889Z
---

**2026-08-08 (Opus 4.8 holding authority, 2× Haiku 4.5 attacking). Commit `0d06a4e` (fix) + `487c342`
(evidence). Full record: `docs/security/red-team-surfaces-2026-08-08/`.** Jesse's "use multiple haiku
agents, u hold delulu authority" — I re-verified every agent claim against the tree ([[discovery-over-checklist]]).

Three untested surfaces attacked. **WASM: CLEAN** (deny-by-default Wasmtime linker in `host.rs`, effects
host-mediated, loop refusal DL1201 complete, secrets DL1205, fuel+mem limits; honest incompleteness —
refuses loops/secrets/GC/Python, falls back to the interpreter; no authority escape). Two real findings,
one root cause: **no IPC read timeout**.

- **IPC-1** (found by hand + IPC agent + code-conclusive): the single-connection blocking serve loop
  (`brokerd.rs` serve) called `read_frame` with no server-side read timeout, so a same-uid client that
  connects and stalls (or dribbles a partial frame) hangs the daemon INDEFINITELY — denying all custody
  ops, incl. the operator's e-stop `grants revoke`. (Version check `brokerd.rs:270`, malformed-CBOR
  fail-closed, peer auth DACL/0700 all HOLD — the read bound was the only gap.)
- **DEADMAN-1** (dead-man agent, confirmed; AMPLIFIES IPC-1): the dead-man watchdog's authority probe
  (`device.rs:599`) runs BEFORE the wall-clock heartbeat sweep and used the unbounded `request()`. On
  **Unix** a hung broker makes the probe's `read_frame` block forever → the watchdog stalls → the
  heartbeat park never runs → the dead-man is DISABLED. On **Windows** the connect already bounded it
  (fail-closed park). So a same-uid attacker who triggers IPC-1 defeats the dead-man on Linux. The
  probe maps a broker `Err` → `AuthorityState::Dead` → park (`cli.rs:4654`), so bounding the read makes
  it fail-closed.

**Fix (one mechanism, both sides; stays blocking std I/O — no async, no thread pool):**
`Connection::set_read_timeout` on both transports — Unix delegates to `SO_RCVTIMEO`; Windows polls
`PeekNamedPipe` (non-blocking) until data or the deadline, then `ReadFile`. Serve loop bounds each read
at 5 s and drops a stalled client (fail-closed `continue`). The dead-man probe uses a new
`request_timed` with a 1 s bound → non-answering broker → Err → Dead → PARK. Pinned + FALSIFIED by
`request_timed_fails_closed_on_a_broker_that_accepts_but_never_answers` (cfg unix): 300 ms with the
bound, blocks 2.0003 s without it. Windows 15/0+144/0+66/66, Linux 16/0+144/0+integration green.

**Residual (category 7, MATHEMATICS.md item 12):** the INDEFINITE hang is closed; a same-uid attacker
can still churn/dribble connections or `kill` the daemon (shares the OS user). Full availability against
a co-resident same-uid process needs a separate OS account. The automatic heartbeat park is now
fail-closed independent of broker responsiveness — the safety-critical guarantee.

**2026-09-30 — the dribble half was never closed (FRAME-DRIP-1, D-V2-73, routine run 6).** The 5 s bound was on
each READ: a same-uid client sending one byte every 2 s was never dropped, so the "indefinite hang is closed" line
above did not hold for a dribbler — a `Status` behind one waited its whole 23.7 s life (witnessed on `dcf4fcb`).
The serve loop now owes a client's whole request within 5 s of acceptance (`delulu_runtime::channel::Within`),
and the same bound per message went onto the foreign worker's call and the sandbox host. Lesson: the witness was
a client that STALLED; the dribbling client the comment named was never tried.

**The same day — the probe side too (PROBE-DRIP-1, REQUEST-HANG-1, D-V2-75).** DEADMAN-1's fix bounded the probe's
read per READ: an answer dribbled onto the broker's socket, a byte every 250 ms, kept the 1 s probe waiting 6 s — the
red-team pass on FRAME-DRIP-1 ran an e-stop that printed "revoked" while the arm kept moving. And `request` — every
custody op and operator command — had no bound at all (40 s against a mute acceptor, the acceptor's whole hold). Both
now owe the whole answer within a bound (1 s for the probe, 15 s for `request`). Related:
[[delulu-root-issuance-bypass]] (DISC-1, the prior finding), [[delulu-hw-adapter]].
