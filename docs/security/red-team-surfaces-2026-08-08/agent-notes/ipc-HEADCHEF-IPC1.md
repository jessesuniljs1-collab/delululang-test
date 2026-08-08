# IPC-1 — head chef's independent finding (before the Haiku agent report)

**Found by reading the code (Opus 4.8, holding authority), 2026-08-08. To be cross-checked against the
IPC agent's NOTES.md when it reports.**

## The finding: no server-side read timeout → single-client DoS of the whole daemon

The broker serve loop is single-connection and blocking (`brokerd.rs` ~690):
```
loop {
    let mut conn = listener.accept()?;         // one connection at a time
    let req = read_frame(&mut conn)?;          // BLOCKS here with no timeout
    let (resp, stop) = handle(...);
    write_frame(&mut conn, &resp);
    drop(conn);
}
```
`read_frame` (`broker_ipc.rs:298`) does `r.read_exact(&mut len_bytes)` then `r.read_exact(&mut buf)`.
`read_exact` blocks until the bytes arrive. **Neither transport sets a server-side read timeout:**
- Unix (`broker_transport.rs:444`): `Connection::read` = `self.inner.read(buf)` — no `set_read_timeout`.
- Windows (`broker_transport.rs:266`): `ReadFile` in blocking mode — no timeout.
The only timeout anywhere is the CLIENT-side connect retry (`broker_transport.rs:325`, 1000 ms for
ERROR_PIPE_BUSY). Nothing bounds the SERVER read.

**Attack:** a same-uid process connects to the pipe/socket and sends nothing (or a partial 1–3 byte
length prefix, or dribbles 1 byte at a slowloris pace). The serve loop blocks on `read_exact` and
**never accept()s another connection** — every legitimate custody op (grants/revoke/guard/secrets/audit)
is denied until the malicious client disconnects. Trivial to launch.

**Why it matters beyond ordinary same-uid DoS:** the operator's **e-stop is `grants revoke` THROUGH
this daemon**. A rogue same-uid agent that hangs the broker denies the operator's deliberate e-stop of a
running actuator — the safety fallback then degrades to the slower wall-clock dead-man. A compromised or
autonomous AI agent (Jesse's explicit threat) sharing the OS user can do this. So this is availability +
a safety-relevant degradation, not just nuisance DoS.

Severity: **medium-high** (same-uid trust domain, but denies e-stop; the e-stop dependency is the
aggravating factor). Confidence: **high** (conclusive from the code; no timeout exists on either read).

## Fix design (implement AFTER the agents finish, to not race their builds)

Bound the per-connection read with a deadline so a stalled/dribbling client is dropped and the loop
continues:
1. **Add a server read timeout on the accepted connection**, set right after `listener.accept()`.
   - Unix: `UnixStream::set_read_timeout(Some(dur))` — `read_exact` then returns `WouldBlock`/`TimedOut`.
   - Windows: give `Connection` a `read_deadline`; in `Connection::read`, poll `PeekNamedPipe`
     (non-blocking, reports available bytes) in a short loop until data is available OR the deadline
     passes, then `ReadFile`; on deadline with no data return `ErrorKind::TimedOut`. (No overlapped I/O
     needed.)
2. **Enforce a TOTAL per-connection deadline in the serve-loop frame read** (stops slowloris dribble,
   where each individual read gets one byte before the per-read timeout): the serve loop computes
   `deadline = now + N s` at accept and the frame read fails if the whole request isn't in by then.
3. On timeout: `eprintln!` + `continue` (drop the connection, keep serving) — fail-closed, daemon lives.
4. Pick a generous local-IPC budget (a legit client sends its frame immediately after connecting): a
   few seconds per read + a small total deadline. Do NOT make it so tight that a legitimately slow
   machine drops valid requests.

**Regression test:** a client that connects and sends only a partial length prefix is dropped within the
deadline, AND a legitimate `Status` request immediately afterward is served (proving the loop recovered).
Falsify by removing the timeout and showing the test hangs / the legit request is starved.

**Honest residual:** this bounds a STALLED client. It does not stop a same-uid attacker from simply
reconnecting in a tight loop (connection churn) — but each connection now completes/aborts quickly, so
the daemon keeps serving between them; a same-uid attacker that wants to fully saturate can always spin,
which is the same-uid trust-domain residual (they can also just `kill` the daemon). The fix removes the
*single-connection indefinite hang*, which is the sharp part.
