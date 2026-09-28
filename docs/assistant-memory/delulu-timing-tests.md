---
name: delulu-timing-tests
description: "Diagnosing a wall-clock test that fails only on CI: reproduce runner starvation with a pinned high-priority hog, judge against the gap the thread actually left, never lengthen sleeps or switch to the stepped clock; plus the harness pitfalls that bit on 2026-09-14"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: c99ba5c5-c39b-4a1f-b406-92fbf2c601b0
  modified: 2026-09-25T12:52:53.920Z
---

A wall-clock test that fails only on CI is often **measuring the runner**, not the code. Three cases
so far:
- the pingpong speedup on 2-vCPU runners;
- PowerShell start-up inside the adapter's 2000 ms budget;
- on 2026-09-14, the dead-man test `a_beaten_lease_is_never_revoked` ([[delulu-github-remote]],
  run 4).

- 2026-09-25, run `36117814329`: two `hw_adapter_cli` tests on Windows, the job's FIRST Python driver
  starts. The pinned hog reproduced NOTHING (0/8). The mechanism was a COLD interpreter start: the
  installed Python took 4251 ms cold vs 177 ms warm (`site` walking site-packages). Fixed in the test
  (`python -I -S`, one untimed warm start), never in `EXCHANGE_TIMEOUT`; a slow-driver mutant still
  fails. **So starvation is one hypothesis, not the answer: if the hog does not reproduce it, measure
  the other candidate (cold start, first-time file scan) directly.**

Settle it by experiment, and keep the test able to fail.

**Why:** in run 4 an outside diagnosis, pasted by Jesse, blamed a race. It proposed a longer sleep or
the stepped clock. Measurement showed the longer sleep SHRINKS the stall the test survives. The
stepped clock never runs the wall-clock watchdog, so the stepped rewrite passed a mutant that only
the original test caught. Jesse's words were "Verify before doing anything." Check every claim in a
pasted diagnosis against the code AND an experiment before applying any of it.

**How to apply:**
- **Read the mechanism first:** what exact condition fires the assertion? Here it is
  `now − last_beat > heartbeat` on a monotonic clock. The watchdog samples `now` before taking the
  lock, so a stale sample only makes it more lenient, and the tick interval is irrelevant.
- **Reproduce the starvation, not a description of it** ([[delulu-reproduce-the-shape]]):
  - Run a HIGH-priority PowerShell hog (150 ms busy, 40 ms idle) pinned to CPU 0 through
    `ProcessorAffinity`.
  - Run the test exe pinned to CPU 0 at normal priority.
  - With that, the unmodified test failed 5 of 8 runs with CI's exact message. Idle, it passed 25/25.
- **Fix by judging, not by loosening.** Compare the outcome to what the thread actually did: the
  broker's `hb + overdue_us` must be ≤ the time since the last accepted command was SENT. When the
  premise didn't hold, retry on fresh state within a time budget. Fail as NOT MEASURED rather than
  pass on something never observed.
- **Prove it can still fail** ([[delulu-proof-campaign]]: a gate that cannot fail is not a gate). A
  mutant (the watchdog 200 ms early) must fail the new test. Also check which OTHER tests catch the
  mutant. Here none did, and that is why the test could not move to the stepped clock.
- **Harness pitfalls that bit on 2026-09-14:**
  - **Arrays through `powershell -File`:** `powershell -File x.ps1 -Tests 'a','b'` passes ONE joined
    string. `--exact` then matched nothing, 0 tests ran, and the harness printed PASS. Require
    `running 1 test` in the output.
  - **`Copy-Item` keeps the source file's mtime.** After restoring a file over a mutant, cargo may
    think the file is unchanged and silently keep the mutant binary. Touch the file, and prove the
    restore with a run.
  - **No rustfmt here.** This repo deliberately does not run rustfmt: `ci.yml` says it is
    hand-formatted, with no `rustfmt.toml`. Never run `cargo fmt`. CI's lint is
    `cargo clippy --workspace --all-targets -- -D warnings`.
  - **Cost measurements: read the SLOPE between two sizes, not each point (PS-B-04, 2026-09-25).**
    `bench.py` subtracts a baseline program assuming every fixed cost cancels; on the macOS runner
    one did not (~0.1 s extra for the clock program at both sizes), so its N=2000 point said 70 µs
    while the slope said 19.8 µs. Two sizes are there to expose exactly this.
  - **Before changing semantics to buy speed, take the cost apart.** The Windows channel's 48.6 µs
    was four thread wakes per round trip (drain threads); removing hops one at a time in a throwaway
    local build (48.6 → 40.4 → 30.1) located it, and the fix (overlapped duplex pipe + guest
    watchdog, 28.9 µs) needed no batching and no change to when effects are observed.
