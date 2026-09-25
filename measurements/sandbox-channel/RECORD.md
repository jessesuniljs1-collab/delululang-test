# What one sandbox-channel round trip costs (PS-B-04's gate)

**What this decides.** PS-B-04 was written as "channel batching for epoch-class effects, after PS-A's
measurement says it pays". That measurement had never been taken: no record under `measurements/`
and no line in the V2 log gave a channel cost. This is it, taken in PS-B (2026-09-25).

**The rule, fixed before any number was seen.** Batching PAYS only if the channel adds **more than
50 µs per effect** — past that, a program making 20,000 effects loses over a second to round trips
alone. Below it, batching is not worth what it costs: a new request kind on the channel, and a change
to WHEN an effect is observed (a clock read answered inside a batch is not taken when the program
asked), which would need its own justification even if it were fast.

**Method** (`bench.py`, run against a RELEASE build — debug timings distort both sides):

- Two programs, identical but for one line: a loop of `N` clock reads (`clk.now_ms()`, one `Clock`
  effect each) and the same loop doing arithmetic instead (the baseline).
- Each is run at L0 (`--grant clock`) and sandboxed (`--sandbox --grant clock`), five times, medians.
- **Channel cost per effect** = [(clock_sandbox − base_sandbox) − (clock_L0 − base_L0)] / N. The
  interpreter's loop and every fixed cost (process start, the check, the guest launch) cancel out.
- Two sizes (`N` = 2,000 and 10,000), so a cost that is not linear in the number of effects shows.

## Result — Windows 11, x86-64, this workstation (2026-09-25)

| transport | N = 2,000 | N = 10,000 |
|---|---|---|
| L0: the effect itself | ~0 µs (below the noise) | 0.17 µs |
| **today's Windows guest** (PS-B-03: a per-run AppContainer, inherited pipes, a drain thread) | **47.55 µs** | **49.20 µs** |
| PS-A's named-pipe guest (measured by forcing the plain launch in a local build, then reverted) | 71.0 µs | 57.37 µs |

Two findings, both worth more than the verdict:

1. **PS-B-03 did not slow the channel; it sped it up** by roughly 15–30%. The inherited pipes plus a
   drain thread beat the named pipe they replaced, so the identity cost nothing in round-trip time.
2. **Under PS-A's transport the rule would have said batching pays** (57–71 µs). Under today's it is
   **just under** (47.5–49.2 µs). A result within 2% of its threshold, on one machine, is not a
   verdict, which is why the same measurement runs on every CI operating system below.

## Result — the three CI runners

`.github/workflows/channel-measure.yml` (manual) builds the release binary on each runner and runs
`bench.py`. Transcribed from the run once read — until then this section says so rather than
guessing: **not yet run.**

## Decision

Recorded in `V2_LOG.md` (PS-B-04) once the CI rows exist.
