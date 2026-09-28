---
name: mutant-run-per-target
description: "When killing mutants, run each cargo test target separately — a name filter applies to every target and silently runs nothing"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-09-26T00:03:10.482Z
---

`cargo test -p X --lib NAME --test T` applies the positional NAME filter to EVERY target, so the
integration test `T` runs zero tests and a mutant looks alive (2026-09-26, P4-06: six "survivors"
were all killed once each target ran on its own).

**Why:** a mutant harness that reports "survived" when it ran nothing is a gate that cannot fail —
the same shape as [[delulu-proof-campaign]]'s rule.

**How to apply:** in a mutant loop, invoke `--lib FILTER` and `--test T` as separate cargo commands,
and treat an all-survive result as a harness bug to check before believing it.
