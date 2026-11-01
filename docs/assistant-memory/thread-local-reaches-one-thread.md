---
name: thread-local-reaches-one-thread
description: "ACTOR-DETERMINISM-1 (2026-10-27, D-V2-192): --seed and --clock fixed: were thread-locals set on main's thread, so actors (worker threads) ignored both; list every thread that runs program code before trusting a thread_local! setting"
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-10-27T17:10:31.213Z
---

`prim.rs` holds the generator (`RNG`) and the fixed clock (`FIXED_CLOCK`) in `thread_local!`; `run` sets them on the
main thread. Actor turns run on worker threads, so under `--seed 7` an actor drew differently every run (even with
`--actors-threads 1`) and read the wall clock under `--clock fixed:1000`. Every `--seed` test drove `main`; the agent
campaign's actor programs drew nothing at random, so A11 (line order) looked like the whole story.

Fixed in #79: each actor owns a `RandStream` (state + key + spawn count), keyed by its spawner via `next_spawn_key`,
swapped in per turn (`swap_rand_stream`); workers adopt the fixed clock at start. Interleaving across actors stays
unordered (Stage 7: per-sender FIFO only) — documented, not "fixed".

**Why:** a determinism knob that silently skips threads breaks spec §6.2 replay without any test going red.
**How to apply:** for any `thread_local!` setting, enumerate the threads that run program code (actor workers first)
and test the setting from each. HANDOFF §11.5 carries the same lesson. Related: [[a-field-fed-by-a-global]],
[[release-program-progress]].
