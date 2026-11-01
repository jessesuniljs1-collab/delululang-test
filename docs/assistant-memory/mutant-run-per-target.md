---
name: mutant-run-per-target
description: "When killing mutants, run each cargo test target separately — a name filter applies to every target and silently runs nothing"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-17T11:28:23.302Z
---

`cargo test -p X --lib NAME --test T` applies the positional NAME filter to EVERY target, so the
integration test `T` runs zero tests and a mutant looks alive (2026-09-26, P4-06: six "survivors"
were all killed once each target ran on its own).

**Why:** a mutant harness that reports "survived" when it ran nothing is a gate that cannot fail —
the same shape as [[delulu-proof-campaign]]'s rule.

**How to apply:** in a mutant loop, invoke `--lib FILTER` and `--test T` as separate cargo commands,
and treat an all-survive result as a harness bug to check before believing it.

**And rebuild after restoring** (2026-10-17): `git checkout` of the mutated files leaves
`target/debug/delulu` as the MUTANT build; the next run of a fixed test went red on the mutant's own
leak and looked like a regression until `grep -c <mutant marker> target/debug/delulu.exe` found it.
Put a unique marker string in every mutant, rebuild after restoring, and grep the binary for the marker
before trusting any later run. (`delulu` is a bin crate: its unit tests run with `--bin delulu`, not `--lib`.)

**And rebuild exactly as the mutant built** (2026-10-17, P9-05 (a)): `cargo test -p A -p B --lib --no-run` names a
crate's test binary differently from `cargo test -p A --lib`, so a rebuild with another package set left the M2 mutant's
binary on disk. Scan `target/debug/deps/*.exe` for the markers after the rebuild; rebuild each package as its mutant did.

**The control command must rebuild every artifact a mutant command built** (2026-10-22, M791): per-mutant `test` commands ran `-p delulu --test …` (rebuilding `delulu.exe` with the mutant) while the spec's top-level control ran only `-p delulu-runtime --lib`, so the CONTROL went green and `target/debug/delulu.exe` still held M791 — a by-hand run then "failed" on a correct change. Put the widest command in the top-level `test`, or `cargo build -p delulu` after the run.

**Again 2026-10-23:** a release binary built on a branch BEFORE master (with the type-size bound, #56) was merged into it made a self-attack report seven shapes still exponential; rebuilt from the merged tree, all seven were refused in ~0.1 s. Before trusting a timing, confirm the binary holds the code under test (rebuild after every merge, or grep the source for the change and rebuild).
