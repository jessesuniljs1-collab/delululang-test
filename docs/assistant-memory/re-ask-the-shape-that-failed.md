---
name: re-ask-the-shape-that-failed
description: "2026-10-31: RW 4.53's open measurement came back clean on the current binary, but the red team's F4 shape (a held audit chain, requests queued) still failed — the e-stop answered DL1401 at 15 s; re-run the failing shape, not only the summary number"
metadata:
  node_type: memory
  type: feedback
  modified: 2026-10-31T17:00:00.000Z
---

Before closing a row on a clean re-measurement, re-run the exact shape the original failure had.

**Why:** on-demand routine run 14 took RW 4.53, whose last open item was routine run 16's measurement — a daemon request
beside twelve loops of sandboxed runs. On `0ebbc15` it was clean: 22 ms median, 49 ms max, the chain's contention gone
(D-V2-159, D-V2-160). The row also cited routine run 18's red team F4: an e-stop behind four queued requests while the
chain is held. Re-run with a writer holding the chain's append lock, it still failed: `grants revoke` answered "DL1401 …
did not answer within 15s — start it with `delulu broker start`" and the revocation landed about 25 s later
(ESTOP-QUEUE-1, fixed by D-V2-198: revocations have a line the serial handler reads first). Closing on the clean number
would have closed the row on the wrong question.

**How to apply:** list every shape a row's history names (each red-team finding, each by-hand reproduction) and re-run
each against the current binary before recording the row closed. And a witness of an ORDER asserts the place of every
member it is about — M970 (the quarantine left out of the lane) survived a witness that placed only the e-stop.
