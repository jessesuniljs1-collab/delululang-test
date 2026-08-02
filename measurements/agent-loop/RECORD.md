# The agent loop: what one toolchain invocation costs before any program grows

*Taken: 2026-08-02, release build. A measurement without a date is a claim without a shelf life —
these numbers describe the tree as it stood on that day, not a permanent property.*

**What this measures, and why nothing else did.** Every performance table this project already
publishes measures the *marginal* cost of size: `scale/RECORD.md` charts `delulu check` against
lines and shapes, `study-c` charts compute against C. None of them measures the **floor** — what a
single invocation costs on a program small enough to be free.

That floor is the number DeluluLang's primary users actually pay. An AI agent editing a file and
re-checking it runs that loop once per iteration, and it pays the floor every time regardless of
how small the edit was. Study C noticed the effect and only from the other side: its own threat-to-
validity section records that wall-clock "includes process startup, and for the C lane it
*dominates*". Nobody had turned that observation on DeluluLang.

**Method.** Whole-process wall clock, release build, warm cache, **best of 40 repeats**, each table
taken in one sitting so its rows are comparable. The decomposition needs no instrumentation — it is
built from controls that each add exactly one layer:

| Control | Isolates |
|---|---|
| `empty` — a Rust binary whose whole source is `fn main() {}` | OS process creation + the Rust runtime. Nothing DeluluLang chose. |
| the same, plus a 512 MiB-stack thread | what `main.rs`'s interpreter worker costs |
| `delulu --version` | a 17.8 MB (Windows) / 22 MB (Linux) image and its static initialisers |
| `delulu check` on a 1-line module | read + morph pragma + lex + parse + check + report |

Absolute numbers are machine-specific; **the decomposition is the result**, not the milliseconds.
Harness kept out of the repo (it builds throwaway binaries): `empty.rs`, `nulstack.rs` and a timing
loop, as described above.

## Windows 11 (native, x86_64-pc-windows-msvc)

| Layer | min ms | median | max | added by this layer |
|---|---:|---:|---:|---:|
| `empty` — `fn main() {}` | **26.87** | 29.32 | 32.97 | — |
| + 512 MiB-stack thread | 27.39 | 29.46 | 42.56 | +0.52 |
| `delulu --version` | 32.04 | 33.28 | 37.60 | +4.65 |
| `delulu --version` (python-less build) | 29.09 | 30.67 | 35.08 | *control, see below* |
| `delulu check` — 1 line | 32.47 | 34.75 | 37.94 | +0.43 |
| `delulu check` — 35 lines (the reference program) | 32.82 | 35.45 | 41.58 | +0.35 |
| `delulu check` — 1,001 lines | 47.40 | 50.53 | 57.20 | +15.36 |
| `delulu authority` — 35 lines | 33.44 | 35.40 | 39.22 | |
| `delulu fmt --check` — 35 lines | 34.91 | 36.53 | 39.86 | |

## Linux (WSL2 Ubuntu 20.04, isolated `CARGO_TARGET_DIR` on ext4)

| Layer | min ms | added by this layer |
|---|---:|---:|
| `empty` — `fn main() {}` | **3.54** | — |
| + 512 MiB-stack thread | 4.21 | +0.67 |
| `delulu --version` | 6.19 | +1.98 |
| `delulu check` — 1 line | 6.72 | +0.53 |
| `delulu check` — 35 lines | 7.63 | +0.91 |
| `delulu check` — 1,001 lines | 29.16 | +21.53 |
| `delulu authority` — 35 lines | 7.26 | |
| `delulu fmt --check` — 35 lines | 9.10 | |

## The finding

**On a 35-line file, 26.87 of 32.82 ms — 82% — is Windows creating a process, before a single byte
of DeluluLang runs.** On Linux the same share is 46%. The compiler's own work on that file is under
1.5 ms on both platforms.

Stated as the question an optimiser would ask: **how large must a program be before compiling it
costs as much as starting the process?**

| | floor | compiling, per 1,000 lines | crossover |
|---|---:|---:|---:|
| Windows | 32.04 ms | ~15.4 ms | **≈ 2,080 lines** |
| Linux | 6.19 ms | ~23.0 ms | **≈ 270 lines** |

For the file sizes an agent actually edits — tens to a few hundred lines — the per-invocation floor
dominates on both platforms and overwhelmingly on Windows. **Making the checker twice as fast would
save 0.4 ms of a 32.8 ms Windows loop: 1.2%.** The lever is not the compiler. It is the number of
processes.

### Two hypotheses this killed, which is why the controls are here

- **"It is the embedded CPython."** The python-less build (`--no-default-features`, a first-class
  gated configuration) starts **1–3 ms** faster across two sittings — 32.04 → 29.09 in the sitting
  above, 30.8 → 29.8 in an earlier one. Real, small, and **not** the reason the floor is 26.87 ms:
  the floor belongs to a binary that contains no DeluluLang at all.
- **"It is the 512 MiB stack."** `main.rs` reserves that much for the interpreter's depth bound to
  be the limit that fires, and its docstring claims "the reservation is virtual … so this costs
  nothing for the programs that never recurse." Measured: **+0.52 ms on Windows, +0.67 ms on
  Linux** — smaller than the run-to-run spread of the control itself (26.87–32.97 ms). The claim
  survives contact with a stopwatch, which is the only reason it is worth repeating.

## What this changed

`delulu check` now takes **several files in one process** (`delulu check a.delulu b.delulu …`).
Nothing was made faster to get this; the loop simply stopped paying the floor once per file:

| 20 files, best of repeats | 20 separate invocations | one invocation | |
|---|---:|---:|---:|
| Windows | 711.6 ms | **48.0 ms** | **14.8×** |
| Linux | 119.4 ms | **14.2 ms** | **8.4×** |

**And it closed a defect, which is the part worth remembering.** Looking at whether one process
could do several files turned up something worse than a missing feature: the argument parser kept
the first non-flag argument and dropped the rest **in silence**. `delulu check a.delulu bad.delulu`
printed `ok: a.delulu checked clean` and exited **0** while `bad.delulu` — never opened — held two
errors. A shell glob did the same. Observed against the unmodified release binary before anything
was changed. Every command that takes one path now refuses a second rather than ignoring it.

## Threats to validity

- **One machine, one sitting per platform.** No cross-machine variance is characterised, and the
  Windows spread is wide (26.9–33.0 ms on the control alone) because process creation there
  competes with whatever else the OS is doing, including its own file-scanning.
- **The two platforms disagree about the compiler's share and the disagreement is not explained
  here.** Checking 1,001 lines costs ~15.4 ms of compute on Windows and ~21.5 ms on Linux — same
  code, same input. At the 35-line size the difference (0.78 vs 1.44 ms) is at the resolution limit
  of both timers. The crossover figures inherit that uncertainty and are given to two significant
  figures for that reason.
- **"Per 1,000 lines" is a two-point slope, not a curve.** `scale/RECORD.md` has the curve; this
  table is about the intercept, and the slope here is only used to locate the crossover.
- **WSL2 is not bare-metal Linux.** Its process creation may differ from a native kernel in either
  direction; it is the Linux this project can execute.
- **macOS is absent because it has never been executed** — no Apple hardware (see
  `docs/design/CROSS_PLATFORM_VERIFICATION.md` §5). Its process-creation cost is unknown to this
  project and is not extrapolated from either column.
- **The batch comparison measures the loop, not the checker.** The 14.8× and 8.4× figures are what
  one process buys over twenty; they say nothing about how fast the compiler is.

## The gate this left behind

A measurement that is not repeated stops being true. But the thing worth gating is **not** wall
clock — a timing gate on a shared machine is a flake, and this project has already had one (the
criterion-10 satellite's debug-timing failure). Nor is it the floor, which belongs to the operating
system.

What can regress silently and would matter is the compiler's **curve**. This project has shipped one
accidentally-quadratic checker path already (C48: a whole-definition `.clone()` on the field-access
path, 632 ms at N=2000), and nothing failed — it was found because someone chose to measure.

`crates/delulu-check/tests/work_scaling.rs` counts **allocations**, which are a deterministic
function of the input and the pinned toolchain, and asserts a **shape** rather than a constant:
doubling the input may not more than double the work. A constant would fail on every harmless
refactor and teach everyone to re-bless it unread.

| shape | healthy ratio | with C48 reintroduced | bound |
|---|---:|---:|---:|
| `records` — N fields, N accesses | 1.88 | **3.89** | 2.6 |
| `wide` — N sibling functions | 1.97 | — | 2.6 |

The C48 column is not an estimate: the `.clone()` was put back in `check.rs`, the gate was
**observed failing** at 3.89, and the change was reverted. A third test asserts the allocation
counter is actually counting, because a gate whose instrument reads zero passes everything.

## What this points at, and did not do

The measurement names the process boundary as the cost, and multi-file `check` removes it for one
common shape. The other shapes are already built or already named:

- **The language server** (`delulu lsp`) is the general answer — the process is paid for once and
  every subsequent check is the sub-millisecond part. That it is *also* the performance answer was
  not the reason it was built, and this measurement is the first thing to say so.
- **`delulu test` and `build`** already do a whole package in one process.
- **A persistent check daemon** is not built, not scoped, and not recommended on this evidence
  alone: it would be a second long-lived process with its own state, and the LSP already occupies
  that role.
