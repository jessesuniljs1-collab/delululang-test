---
name: a-latency-may-be-a-deadline
description: "2026-10-19: MONITOR-CHAIN-1 — a per-poll cost filed as a LOW latency was a deadline: the monitor beats a dead-man between polls, so a long audit chain made its broker revoke every run it watched; grow the input past the nearest deadline before grading a cost"
metadata:
  node_type: memory
  type: feedback
  modified: 2026-10-19T09:30:00.000Z
---

When a loop's cost grows with history, find the nearest deadline the loop must meet and grow the input until the cost
crosses it — before grading the cost as a latency.

**Why:** RW 4.49 recorded "each poll re-reads the whole chain (about 200 ms at 10,000 records)" as LOW. The same loop
beats the monitor's dead-man, so once one read outlasted the period (2 s by default) the broker revoked the monitor's
node and every run under it, with nothing wrong (250,000 records, measured 2026-10-19; D-V2-158 fixed it with an
incremental, verified reader, `audit::Follower`).

**How to apply:** for every watchdog, heartbeat or lease a loop must renew, list what the loop's work grows with (chain
length, tree size, queue depth) and witness it with that input grown — the witness's state is "one iteration takes
twice the period", measured in the test.

**Second instance, same day (MONITOR-FRAME-1, RW 4.69, D-V2-161 — the laptop):** the monitor's per-poll whole-tree `List`
was measured as a latency (371 ms at 30,003 nodes). The nearest limit was not a clock but a constant: a wire frame holds
16 MiB, and past it the daemon dropped the connection, the monitor read "unreachable", stopped, and its dead-man revoked its
fleet (550 nodes of long descriptions, none under it). So also list the SIZE limits a growing answer meets (frame, line,
buffer), and what the client does when the answer cannot be sent.
