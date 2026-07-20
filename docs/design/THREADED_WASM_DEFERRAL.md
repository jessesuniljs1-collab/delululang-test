# Multi-threaded WASM engine — the honest deferral (Track A §2.4)

**Status:** deferred for 1.x, on the record. **Stage 10 phase 10l, spec §2.4 and §11 criterion 3.**
This is the honesty note criterion 3 requires when the multi-threaded-WASM track is deferred rather
than shipped. Spec §2.4 sanctions the wait in advance:

> **§2.4:** Wasmtime threads + shared-everything-GC per its maturity at implementation time; actor
> scheduler ported; the Stage-7 TSAN/parity criteria re-run on this engine. **If the proposal stack
> is not production-ready when this track lands, the track *waits* and says so (mode honesty beats
> mode count).**

> **Criterion 3 (P2), third clause:** multi-threaded WASM passes Stage-7 criteria 1, 6, 7 **(or the
> track is explicitly deferred with its honesty note published).**

The track is deferred. This is that note.

## What is deferred, precisely

A **second, genuinely multi-threaded execution engine** built on the WebAssembly backend — Wasmtime
running the actor scheduler across OS threads, with the actor heaps shared via the wasm
**shared-everything-GC** proposal stack, and the Stage-7 concurrency criteria (TSAN cleanliness,
two-engine parity, fault isolation) re-run on it.

## Why — and why nothing is actually lost

**1. Multi-threaded actor execution already ships, on the default engine.** The interpreter's actor
scheduler is genuinely multi-threaded: worker-owned actors, each pinned to one worker thread at
spawn, each worker owning its actors' heaps outright, cross-worker delivery over `mpsc`
(`crates/delulu-runtime/src/actors.rs`). It is driven by `--actors-threads`, and the Stage-7
concurrency criteria — including a **clean TSAN run** and the two-engine parity suite — were met on
it. So the *capability* "DeluluLang runs actors across real threads" is not pending; it is the
shipped default. The deferral is about adding a **second** multi-threaded engine, not about whether
threads work.

**2. The WASM engine is deliberately single-threaded today, and correct.** The WASM backend's actor
scheduler is **cooperative single-threaded** by design (Stage 7 §6.5,
`crates/delulu-wasm/src/actors.rs`): turn atomicity is free, per-sender-pair FIFO holds, and the
quiescence accounting matches the native scheduler's. It delivers the same *observable* actor
semantics as the interpreter — on one thread. It is a correct, proven execution option; it simply
does not add parallelism.

**3. The proposal stack the port needs was not production-ready at 1.x implementation time.** Porting
the actor scheduler onto a *multi-threaded* WASM engine means running GC-managed actor heaps across
threads — the **shared-everything-GC** proposal, layered on wasm threads. The pinned engine
(`wasmtime = "27"`) is not configured for that stack (this project enables neither the wasm-threads
nor the wasm-GC proposals in its run-path engine — `crates/delulu-wasm/src/host.rs`), and
combining GC objects with threads was still maturing upstream, not a production-ready foundation to
put actor heaps on. §2.4 says, in that situation, the track waits — because a multi-threaded engine
that is not TSAN-clean would be a *downgrade* dressed up as a mode, and the Stage-7 criteria exist
precisely so that "we added threads to the WASM engine" cannot be said until it survives them.

**4. Building it now would trade honesty for a mode count.** A rushed second concurrent engine, not
held to the Stage-7 TSAN/parity bar, is exactly the "mode count over mode honesty" the spec's own
§2.4 wording rejects. The engineering-honest move is to wait for the proposal stack, and say so.

## What re-opens this

The track re-opens when the wasm shared-everything-GC + threads proposals reach a maturity in a
pinned Wasmtime that can host the actor heaps, at which point §2.4's plan runs as written: port the
scheduler, and **re-run the Stage-7 criteria (1, 6, 7) on the new engine** — the same TSAN
cleanliness, two-engine parity, and fault-isolation bar the interpreter already cleared. Until then,
multi-threaded actors run on the interpreter, the WASM engine runs them cooperatively on one thread,
and both are correct.

## What this does not claim

- It does not claim the WASM engine is slower or faster than the interpreter for actor workloads —
  see `measurements/study-c/HOT_PATH_TABLE.md` for what the performance study does and does not show.
- It does not claim wasm threads are unavailable in Wasmtime generally — only that the
  **shared-everything-GC** stack needed to share *actor heaps* across threads was not a
  production-ready foundation for this port at 1.x, and that this project's engine does not enable
  those proposals today.
- It does not weaken any Stage-7 guarantee: those hold on the interpreter, which is where
  multi-threaded actors run.
