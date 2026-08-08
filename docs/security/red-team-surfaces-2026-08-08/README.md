# Red-team — the three untested surfaces (2026-08-08)

Three AI agents (Haiku 4.5) attacked the broker IPC daemon, the device dead-man / e-stop, and the WASM
host, while the head chef (Opus 4.8) held the delulu authority and **re-verified every claim
independently against the current tree** — agent output is evidence, not verdict (the P20 custody
campaign had a confident "HOLDS" and a "BROKE" that were both wrong). Raw agent notes and the
adjudication are in `agent-notes/`. Two real findings, one clean surface; both findings fixed in commit
`0d06a4e`.

## Verdicts

| Surface | Agent | Head-chef verdict |
|---|---|---|
| Broker IPC daemon | Haiku 4.5 | **IPC-1 CONFIRMED** (also found independently by the head chef, and code-conclusive). No server-side read timeout on the single-connection blocking serve loop → a same-uid client that connects and stalls hangs the daemon, denying all custody incl. e-stop. Agent's "invariants 2–5 hold" spot-checked TRUE (the version check at `brokerd.rs:270` rejects `!= "broker/1"` before dispatch; malformed CBOR → ciborium `Err` → `continue`; peer auth DACL/0700; pid not load-bearing). **Fixed.** |
| Device dead-man / e-stop | Haiku 4.5 | **DEADMAN-1 CONFIRMED, and it amplifies IPC-1.** The watchdog's authority probe (`device.rs:599`) runs before the wall-clock heartbeat sweep and used the unbounded `request()`. On **Unix** a hung broker makes the probe's `read_frame` block forever → the watchdog stalls → the heartbeat park never runs → the dead-man is disabled. On **Windows** the connect already bounded it (fail-closed park). The agent's 65% confidence and platform split were right. The dead-man's other invariants (wall-clock liveness, stepped determinism, envelope, command-path fail-closed) hold. **Fixed.** |
| WASM host | Haiku 4.5 | **CLEAN (verified).** Deny-by-default Wasmtime linker (`host.rs`), effects host-mediated (existing traps tests), loop refusal DL1201 complete (the for/break/continue arm was written this session on top of the existing `while` refusal), secrets DL1205, fuel+mem limits. Honest incompleteness — it refuses loops/secrets/GC/Python and falls back to the interpreter (the reference engine). No authority escape. No fix needed. |

## The two findings share one root cause: no IPC read timeout

- **IPC-1** — server side: the serve loop's `read_frame` had no read timeout, so a stalled client hung
  the whole daemon (all custody ops, including the operator's e-stop revoke, denied until it left).
- **DEADMAN-1** — client side: the dead-man watchdog's authority probe used the unbounded `request()`,
  so on Unix a hung broker blocked the probe forever, stalling the watchdog and disabling the heartbeat
  dead-man. A same-uid attacker who hangs the broker (IPC-1) thereby also defeats the dead-man on Linux.

## The fix (commit `0d06a4e`) — one mechanism, both sides

`Connection::set_read_timeout` on both transports (Unix `SO_RCVTIMEO`; Windows a `PeekNamedPipe` poll —
still blocking std I/O, no async runtime, no thread pool, so the "blocking single-thread" broker design
holds). The serve loop bounds each read at 5 s and drops a stalled client (fail-closed `continue`); the
dead-man probe uses `request_timed` with a 1 s bound so a non-answering broker → `Err` →
`AuthorityState::Dead` → the device **parks**. Both platforms now fail closed at the same bound.

Pinned by `request_timed_fails_closed_on_a_broker_that_accepts_but_never_answers` (Unix): it returns
`Err` in 300 ms against a hung acceptor; **falsified** by removing the bound, which makes it block
2.0003 s and bust the `< 1 s` assertion — the exact fail-open. Verified Windows (15/0 daemon, 144/0
broker, 66/66 delulu) and Linux (16/0 daemon, 144/0 broker, broker/dead-man/actuate integration green).

## Honest residual (category 7, same-uid domain)

The server bound closes the *indefinite* hang from connect-and-stall. A same-uid attacker can still
churn or dribble connections — but each now completes/aborts quickly, and a same-uid attacker can always
just `kill` the daemon anyway. The sharp part (one client hanging the broker, or the dead-man, forever)
is closed. The dead-man's e-stop still depends on broker availability for the *deliberate* revoke path;
the *automatic* heartbeat park is now independent of broker responsiveness on both platforms, which is
the safety-critical guarantee.
