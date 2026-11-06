---
name: rerun-a-flake-the-way-it-failed
description: "RW 7.25 (2026-11-06): my six 'confirming' runs ran the flaky test ALONE (witness filter) while both reds came in push runs of the WHOLE monitor_cli target side by side — a different experiment; witness.yml now has repeat + stress; GitHub Windows runners have Defender real-time OFF"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-11-06T02:39:51.552Z
---

Re-run a flaky test under the conditions of its red, many times, and count every run — never the one test alone.

**Why:** Jesse asked "Why isn't it solved yet" (2026-11-06) about RW 7.25. Both reds (macOS 2026-10-29, Windows
2026-11-05) came in push runs, where `cargo test --workspace` runs the whole `monitor_cli` target with its tests side by
side (threads = CPUs), each starting daemons and monitors. My six runs of 2026-11-05 used `witness.yml` with a filter —
the one test alone — and I reported them as if they said something. The failure rate is about 2 in ~650 OS-runs, so
even the right experiment needs hundreds of runs.

**How to apply:**
- `witness.yml` takes `repeat` (runs of the selection in one job, each counted; the job fails if any did) and `stress`
  (busy loops, one CPU each). A whole `monitor_cli` run takes ~23 s on Windows, ~26 s on macOS, ~15 s on the laptop.
- Measure the runner before blaming it: the repeat run prints CPUs and, on Windows, `Get-MpComputerStatus` —
  GitHub's Windows runners have `RealTimeProtectionEnabled: False` (the laptop's is True).
- Put both sides' timing on one clock (RW 7.25's `Moment`: monotonic + unix ms) so a single red is decisive.
Related: [[rw-7-25-deadman-reader-stall]], [[delulu-timing-tests]], [[a-witness-needs-a-twin]].
