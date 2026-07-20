# Dead-man revoke-to-fail-state latency (Stage 10 phase 10f, spec §5.2)

**What this measures.** Spec §5.2 requires the revoke-to-fail-state latency to be *measured and
published per adapter*, not asserted. This record publishes it for the in-tree reference
simulator (`--broker-profile sim`) — the only adapter that exists. A hardware adapter publishes
its own numbers or it does not ship; that is what "per adapter" means.

**The scenario.** A program commands `arm0/elbow` once, then goes and computes `fib(27)` — far
longer than its granted `heartbeat_ms=25` — and commands again. Nothing in the program asks about
the lease. The watchdog thread revokes it unaided, engages the declared `safe-park` fail-state,
and the program's second command comes back `LeaseRevoked`. This is the CLI path, not a harness:
the same binary, flags and code a user runs.

## The three terms, and why they are reported separately

A single "latency" number here would hide where the time goes, so the budget is published in
parts:

| Term | What it is | Who controls it |
|---|---|---|
| `overdue` | How late the beat already was when the watchdog *noticed*. Bounded by the watchdog's tick interval. | The runtime's tick: `clamp(heartbeat_ms / 4, 1 ms, 25 ms)` — 6 ms at `heartbeat_ms=25` |
| `engage` | Detection → the adapter's fail-state actually applied. | The adapter |
| `heartbeat_ms` | How long silence is tolerated before any of the above starts. | **The human, at grant time** |

The term that dominates in any real deployment is the third, and it is not ours: an operator who
grants `heartbeat_ms=200` has chosen to accept up to ~200 ms of unnoticed silence. The runtime's
contribution is the other two.

## Results (2026-07-20, Windows 11, `heartbeat_ms=25`, n=20 runs)

| Term | min | p50 | p95 | max |
|---|---|---|---|---|
| `overdue` (detection delay past the deadline) | 244 µs | 631 µs | 6.22 ms | **6.33 ms** |
| `engage` (sim adapter fail-state applied) | 2 µs | 3 µs | 16 µs | **17 µs** |

**Worst observed, last-beat to fail-state engaged: 25 ms + 6.33 ms + 17 µs ≈ 31.3 ms** — inside
spec §5.2's `heartbeat_ms + adapter latency` budget, with the tick as the named third term.

Two of the twenty runs landed at ~6.2 ms of `overdue` and the other eighteen under 1 ms. That
spread is the tick beating against the OS scheduler, not noise in the measurement: a 6 ms sleep
request on Windows is honored at whatever granularity the platform's timer is currently running
at, so detection lands on a tick boundary that may be one full interval late. **The number to
plan against is the max, not the median** — which is why the table publishes both and the budget
above uses the max.

**Not measured here: Linux and macOS.** This record covers the platform the phase was built on.
Stating that is the point; extrapolating a scheduler-sensitive number from one OS to another
would be inventing evidence. Criterion 4's demonstration (phase 10g) is where the cross-platform
picture belongs.

## What this number is not

It is not a safety case, and it is not a real-time guarantee. Nothing above changes spec §5.3's
honest boundary: DeluluLang commands the policy/command layer at 1–100 Hz, and the mechanisms
that must hold when *everything* software fails — interlocks, e-stop chains, firmware limits —
live below the adapter and must not depend on DeluluLang existing (invariant 52). A dead-man
lease shortens the window in which a wedged program can keep commanding a machine. It does not
close it, and no adapter's numbers can.

## Reproducing

```
delulu run arm.delulu \
  --grant console \
  --grant "actuator=arm0/elbow:angle_deg=-30..95,velocity_dps=0..40,heartbeat_ms=25,ttl_ms=600000,fail=safe-park" \
  --broker-profile sim --trace-effects
```

The `lease.revoked` record carries `overdue`, and `failstate.engaged` carries `engage`. The
regression witnesses for the same behavior — including the control case, where the identical
program with a generous heartbeat keeps its device — are in
`crates/delulu/tests/dead_man_cli.rs`.
