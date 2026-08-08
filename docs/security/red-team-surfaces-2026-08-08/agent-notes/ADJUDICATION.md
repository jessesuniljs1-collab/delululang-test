# Head-chef adjudication of the 3 Haiku red-team agents (Opus 4.8, holding authority)

Every claim re-verified independently against the current tree. Two confirmed findings, one clean surface.

## IPC daemon → IPC-1 CONFIRMED (agent + my own read + code-conclusive)
No server-side read timeout on the single-connection blocking serve loop (`brokerd.rs` ~690,
`broker_ipc.rs:298`, `broker_transport.rs` read impls). A same-uid client that connects and stalls (or
sends a partial frame) hangs the loop indefinitely → denies all custody ops incl. e-stop. Agent's
"invariants 2–5 hold" spot-checked TRUE (version check at `brokerd.rs:270` rejects != "broker/1" before
dispatch; malformed CBOR → ciborium Err → `continue`; peer auth DACL/0700; pid not load-bearing).

## Dead-man → DEADMAN-1 CONFIRMED, and it AMPLIFIES IPC-1 (Unix fail-OPEN)
The dead-man watchdog (`device.rs:584`) runs the authority probe (line 599, an IPC round-trip) BEFORE
the wall-clock heartbeat sweep (line 616) in the same loop. The probe (`cli.rs:4633`) uses
`brokerd::request` (no client read timeout) and maps a broker Err → `AuthorityState::Dead` (line 4654,
fail-closed → park). So:
- **Windows fail-CLOSED**: a hung broker makes the probe's connect time out (~1 s, `broker_transport.rs`
  connect retry window) → Err → Dead → park. Safe.
- **Unix fail-OPEN**: `UnixStream::connect` succeeds into the backlog, write buffers, then `read_frame`
  BLOCKS FOREVER (no client read timeout) since the hung broker never accept()s it → the watchdog
  stalls in the probe → the heartbeat sweep never runs → the dead-man is DISABLED for every device.
A same-uid attacker triggers IPC-1 (hang the broker) → on Linux the dead-man's heartbeat park is
defeated. SAFETY-critical fail-open on Unix. Confidence HIGH after verifying the loop order + the probe
using `request` (no timeout) + Err→Dead.

Agent's other invariants (dead-man liveness on wall clock, stepped determinism, envelope, fail-closed on
the COMMAND path) hold — the fail-open is specifically the watchdog PROBE blocking on a hung broker.

## WASM → CLEAN (verified)
Deny-by-default Wasmtime linker (`host.rs`), effects host-mediated (existing traps tests), loop refusal
DL1201 complete (I wrote the for/break/continue arm this session), secrets DL1205, fuel+mem limits.
Honest incompleteness (refuses loops/secrets/GC/Python, falls back to the interpreter). No escape.

## THE FIX (both findings, one root cause: no IPC read timeout)
1. `Connection::set_read_timeout(Option<Duration>)` on both transports.
   - Unix: delegate to `UnixStream::set_read_timeout`.
   - Windows: store a deadline; in `read()`, poll `PeekNamedPipe` (non-blocking) until data or deadline,
     then `ReadFile`; on deadline → `ErrorKind::TimedOut`. No async, no thread pool — stays in the
     stated blocking-std-I/O design.
2. IPC-1: serve loop sets a server read timeout (~5 s) on the accepted connection; a stalled client is
   dropped and the loop `continue`s (fail-closed). Residual: a same-uid attacker can still churn/dribble
   connections (same-uid domain; they can also kill the daemon) — the INDEFINITE hang is what's closed.
3. DEADMAN-1: the probe uses a request variant with a ~1 s client read timeout → a hung/slow broker →
   Err → `AuthorityState::Dead` → PARK. This makes both platforms fail-closed at ~1 s (consistent) and
   the watchdog can NEVER be stalled by broker unavailability. This is the safety-critical half.
4. Regression tests + falsification for each. Honest residual documented (category 7 same-uid churn).
