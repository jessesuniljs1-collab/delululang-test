---
name: zero-sleep-yields-on-windows
description: "thread::sleep(Duration::ZERO) is Sleep(0) on Windows — it yields a time slice on a busy machine; a test reader that \"slept\" a zero gap per byte made a channel control test Windows-only red (RW 7.23 (b), 2026-10-23)"
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-22T23:29:44.679Z
---

`std::thread::sleep(Duration::ZERO)` returns at once on Unix but is `Sleep(0)` on Windows, which yields the processor to any ready thread. `channel.rs`'s test reader `Drip` slept before every byte, its zero gap included; on a busy runner each byte cost a time slice, ~162 ms a frame against a 100 ms control. CI saw it once; 32 busy loops on the laptop (PowerShell `while ($true) {}` at Normal/AboveNormal priority) turned the frozen test binary red 98/100 and 200/200; fixed (sleep only a nonzero gap): 0/100 at both. PR #53.

**Why:** a five-attempt retry was tried first; measuring each attempt's elapsed time (~162 ms every time, not random) is what showed it was a per-byte cost, not a stall.

**How to apply:** when a timing test fails only on Windows, look for a zero/near-zero sleep, a `yield`, or a spin in the test or the code before blaming the runner; reproduce with busy loops and print each attempt's elapsed time. In a PowerShell starvation script, never name the process list like a parameter (`$hogs` = `$Hogs`): the hogs were left running once. Related: [[delulu-timing-tests]], [[a-latency-may-be-a-deadline]].
