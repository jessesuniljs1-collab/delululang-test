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

## Result — the three CI runners (run `36167360827`, `7910497`, read 2026-09-25)

`.github/workflows/channel-measure.yml` (manual) builds the release binary on each runner and runs
`bench.py`. Transcribed from the run's log:

| runner | N = 2,000 | N = 10,000 | slope between the two sizes |
|---|---|---|---|
| Linux 6.17 (Azure), x86-64 | 23.85 µs | 20.30 µs | **19.5 µs** |
| macOS 26 (Darwin 25.6), arm64 | 70.55 µs | 29.05 µs | **19.8 µs** |
| Windows Server 2025, x86-64 | 68.20 µs | 67.13 µs | **66.9 µs** |

Reading it honestly takes the slope as well as the two points. The method assumes every fixed cost
cancels between the clock program and the baseline; on macOS one did not — about 0.1 s more for the
clock program at both sizes — so its N = 2,000 figure is inflated and the slope (19.8 µs) is the
channel. Linux and Windows are linear: their points and slopes agree.

**By the rule fixed first, batching does not pay on Linux or macOS and does pay on Windows.**

## Decision — fix the transport that failed the rule, do not batch (D-V2-36)

Batching would change WHEN an effect is observed, on every platform, to recover a cost only one
platform has. So the Windows cost was taken apart first. PS-B-03's transport read each pipe through a
drain thread and a queue (an anonymous pipe has no read deadline), so one round trip woke four
threads where Linux wakes two. Measured by removing the hops in a local build, one at a time, then
reverted:

| Windows workstation, release build | per effect (slope) |
|---|---|
| PS-B-03's transport (both hops) | 48.6 µs |
| the guest's hop removed | 40.4 µs |
| both hops removed | 30.1 µs |

And one thing that was NOT the cost: each frame used to be written in two calls (length, then body).
Merging them into one changed nothing measurable here; it stays, because it cannot cost anything and
removes a possible second wake per frame on every platform.

What was built: the host reads the server end of ONE duplex pipe, opened overlapped, directly on its
own thread with a real deadline (issue, wait at most the deadline, cancel); the guest reads its end
directly, and a watchdog ends the guest if a read outlives the deadline. Deadlines are kept on both
sides; nothing about when an effect happens changed.

| Windows workstation, release build | N = 2,000 | N = 10,000 | slope |
|---|---|---|---|
| **the new transport** | **31.0 µs** | **29.15 µs** | **28.9 µs** |

Below the 50 µs line with room, on the machine that was just under it.

## After the change — the three CI runners (run `36171534543`, `2fb0895`, read 2026-09-25)

| runner | N = 2,000 | N = 10,000 | slope | before (slope) |
|---|---|---|---|---|
| Linux 6.17 (Azure), x86-64 | 15.15 µs | 13.61 µs | **13.2 µs** | 19.5 µs |
| macOS 26, arm64 | 37.0 µs | 22.23 µs | **18.7 µs** | 19.8 µs |
| Windows Server 2025, x86-64 | 43.15 µs | 43.23 µs | **43.0 µs** | 66.9 µs |

**Every runner is now under the rule**, Windows included, with the transport change and nothing
semantic. One result was not predicted: Linux fell by a third. Its guest on this workflow's runner is
the plain one (the workflow does not lift Ubuntu's user-namespace restriction), whose only change was
the one-write frame — which measured as nothing on the Windows workstation. A reader woken once per
frame instead of twice is the likely reason; one run on a shared runner is not proof of it, and it is
recorded as observed. macOS's N = 2,000 point again carries the fixed cost the slope removes.

**PS-B-04 is closed on this evidence (D-V2-36).** The script and the manual workflow stay: if a runner
measures above the line again, the question is one button away.
