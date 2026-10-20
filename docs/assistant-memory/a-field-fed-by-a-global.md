---
name: a-field-fed-by-a-global
description: "2026-10-20: RW 7.20 — two confinement tests failed once with a refusal count of 2; the count was a struct's field, but the method answering a request filled it from a window on the process-wide egress log, which two tests beside them wrote; read every call for a static, and reproduce a once-seen red with a frozen test binary run hundreds of times beside its neighbours"
metadata:
  node_type: memory
  type: feedback
  modified: 2026-10-20T08:30:00.000Z
---

When a once-seen red asserts a count, read every call the asserting path makes, transitively, for a `static` — not only
the method that increments the count — and reproduce it with the tests that share its binary.

**Why:** RW 7.20 recorded `denied().1` at 2 in two `channel.rs` tests, "a field of the `HostChannel` each test makes … nothing
global feeds it that a read found". `HostChannel::answer` opened a window on the process-wide egress log (`egress::mark` /
`records_since`) and counted every refusal in it; two network tests in the same binary make refused requests. On-demand
routine run 5 copied the pre-fix test binary out of `target/debug/deps` and ran the module's filter 400 times at
`--test-threads 16`: 13 red, at the laptop's two lines; the fixed binary beside it, 0. The fix reads a thread-local tally
of the answering thread's own requests; the run report's process-wide log is unchanged (an actor's request is on a worker
thread).

**How to apply:** for a flake in a unit test, list the globals its path touches (`static`, `thread_local`, the environment,
the current directory, the temp directory); for each, which tests in the same binary write it. Witness the mechanism
deterministically (here: a sink that has another thread make a refused request inside the window), then the original red
statistically with a frozen binary, and keep the fixed binary's loop as the control.
