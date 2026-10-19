# The hot-path table — Stage 10 Track A (§2.1), acceptance criterion 1

**Stage 10 phase 10l, spec §2.1 and §11 criterion 1.** This is the published hot-path table
criterion 1 asks for, drawn from the Study-C measurement of **2026-10-19** (`measurements/study-c/results.json`,
release build, a 4-vCPU Linux cloud VM — `REPORT.md` names the machine). The table it replaces, from the v1.0
baseline of 2026-07-20, is kept below as it was, and that run whole in `runs/2026-07-20/`. It is published
**as-is, both the better and the worse numbers**, which
is exactly what criterion 1 requires and what the constitution's own performance-honesty rule
(§5.11) demands.

> **Criterion 1 (P1):** published hot-path table: geo-mean ≤ 2.5× C on the compute-kernel suite
> **under the optimizing backend**, with per-benchmark numbers, both better and worse, published
> as-is.

## The optimizing backend, precisely

The "optimizing backend" that ships in 1.x is the **Cranelift-optimized Wasmtime tier**: `delulu run
--engine wasm` compiles the `.dwx` with Cranelift at optimization level `Speed` (pinned explicitly
in `delulu_wasm::optimizing_engine`, phase 10l, rather than inherited from wasmtime's default so it
cannot silently drift). That is the whole of the optimizing backend in 1.x. The **DIR-level
optimizer** §2.1 also describes — cross-package inlining, monomorphization, escape analysis — is
**deferred** (build-order D18; rationale below).

## The table

Lane figures are the **minimum of 5 repeats**, whole-process wall clock in milliseconds, from the
2026-10-19 14:26 UTC Study-C results (`delulu 1.0.0` built in release from `79f83d6`; gcc 13.3 `-O2`; each lane
timed only when it printed the interpreter's answer). `wasm ÷ C` is the criterion-1 ratio; `interp ÷ C` is
shown because the interpreter is the default engine and runs the whole suite. The last row is the start-up
control — a program that prints `0` and nothing else — and the bracketed figures are each benchmark less its
lane's start-up.

| Benchmark | Kind | C (gcc -O2) | wasm (opt backend) | interp | **wasm ÷ C** | interp ÷ C |
|---|---|---|---|---|---|---|
| `fib_recursive_24` | micro | 1.50 ms [0.58] | 9.74 ms [2.73] | 96.19 ms [91.49] | **6.5×** | 64.2× |
| `loop_sum_1m` | micro | 0.97 ms [0.05] | 11.14 ms [4.13] | 349.30 ms [344.60] | **11.5×** | 359.0× |
| `string_build_20k` | micro | 1.03 ms [0.11] | 8.31 ms [1.30] | 14.77 ms [10.07] | **8.1×** | 14.3× |
| `wordcount_macro` | macro | 0.99 ms [0.07] | — (DL1201) | 11.34 ms [6.64] | **n/a** | 11.4× |
| `list_map_macro` | macro | 1.27 ms [0.34] | — (DL1201) | 11.24 ms [6.54] | **n/a** | 8.9× |
| `nested_calls_macro` | macro | 1.10 ms [0.18] | 11.11 ms [4.10] | 267.98 ms [263.28] | **10.1×** | 242.5× |
| *start-up control* | control | 0.92 ms | 7.01 ms | 4.70 ms | — | — |

`— (DL1201)` means the WASM backend could not run that program: `wordcount_macro` needs `split`, and
`list_map_macro` a `List` and `map`, both outside the fragment it compiles (RW 3.4). Where both engines
ran, WASM took 1.8× (`string_build_20k`) to 31.3× (`loop_sum_1m`) less time than the interpreter.

**One run, and how much a run moves.** The harness's run twelve minutes earlier, before the start-up control was added, read `wasm ÷ C` 6.7×–11.7× and
`interp ÷ C` 7.5×–311.4× (`V2_LOG.md`, 2026-10-19): between the two runs single ratios moved by up to a quarter
(`loop_sum_1m`'s `interp ÷ C` 290.4× then 359.0×), mostly through the C denominator, which is about a millisecond of
process start-up — the bracketed figures show how little of C's
figure is work (0.05–0.58 ms), and why no ratio is computed from them.

### The 2026-07-20 table — the v1.0 baseline, kept as it was

Release build, whole milliseconds, on the development machine — the owner's laptop, the only machine the
project had before CI (2026-09-14); the record itself names none. There a C process read 5–6 ms, which is
most of why its ratios were smaller.

| Benchmark | Kind | C (gcc -O2) | wasm (opt backend) | interp | **wasm ÷ C** | interp ÷ C |
|---|---|---|---|---|---|---|
| `fib_recursive_24` | micro | 6 ms | 12 ms | 77 ms | **2.0×** | 12.8× |
| `loop_sum_1m` | micro | 6 ms | — (DL1201) | 363 ms | **n/a** | 60.5× |
| `string_build_20k` | micro | 6 ms | — (DL1201) | 20 ms | **n/a** | 3.3× |
| `wordcount_macro` | macro | 6 ms | — (DL1201) | 12 ms | **n/a** | 2.0× |
| `list_map_macro` | macro | 6 ms | — (DL1201) | 17 ms | **n/a** | 2.8× |
| `nested_calls_macro` | macro | 5 ms | — (DL1201) | 281 ms | **n/a** | 56.2× |

`—  (DL1201)` means the WASM backend could not run that program: the 1.x backend compiles a
**subset** of the language, and these five programs use constructs (unbounded loops in `main`,
string building, the macro workloads) it does not yet lower. That is a real, published limitation,
not an omission.

## The honest verdict: criterion 1 is NOT met, and this is the deferral

**The geo-mean ≤ 2.5× C target is not met, and the honest reason is twofold** (the 2026-10-19 figures;
the 2026-07-20 ones in brackets):

1. **Coverage.** The optimizing backend runs **4 of 6** compute kernels [1 of 6]. A geo-mean "on the
   compute-kernel suite under the optimizing backend" is not computable when the backend runs two
   thirds of the suite; over the four it does run the geo-mean is **8.8× C**, and each of them is past
   2.5× — **6.5× to 11.5×** [`fib_recursive_24` alone, 2.0×]. A geo-mean over the kernels a backend
   happens to compile is not the suite's, and reporting it as one would be the dishonesty this table
   exists to avoid.
2. **Every figure is a whole process, and start-up is most of the small ones.** On this machine a C
   process starts and exits in about 1 ms [5–6 ms on the laptop, whose spread of 44–77 ms exceeded its
   minimum], and every DeluluLang figure also holds DeluluLang's own start-up — reading and checking the
   program and, on WASM, compiling it with Cranelift. The start-up control measures it: 0.92 ms for C,
   4.70 ms for the interpreter, 7.01 ms for the WASM engine. Less their floors, the four WASM kernels took
   1.30–4.13 ms and C 0.05–0.58 ms — so the whole-process ratios still **understate** the compute gap,
   and a compute-only ratio is not offered: C's figures above its floor are within the noise of a
   millisecond's start-up (its floor's spread is 0.15 ms).

The interpreter — the default engine, which runs the whole suite — is **8.9× to 359.0× C** on this
machine [2.0× to 60.5×]. It did not get slower: its own figures are within a run's spread of the
2026-07-20 ones (`loop_sum_1m` 349 ms against 363); the C lane's start-up shrank. DeluluLang is **not
competitive with C** on these workloads, and Study-C's own write-up says so in those words.

**This is the D4 outcome, and D4 makes it a passing one:** "not production-ready, deferred, here is
why" is a passing result for phase 10l, and criterion 1 carries this note verbatim. What is deferred,
and why:

- **The DIR-level optimizer** (cross-package inlining, monomorphization, escape analysis). It is the
  piece that would close the gap, and it is a substantial compiler in its own right. Building it
  credibly needs (a) an authority-preservation argument for cross-package inlining — legal in
  principle because rows are declared and checked (§2.1), but it needs the proof machinery, not just
  the assertion — and (b) evidence the three passes actually pay off, which does not exist yet.
  Rushing it into the final phase would trade the honesty this whole stage is built on for a mode
  count. Deferred, on the record.
- **Widening the WASM backend's language coverage** so it runs the rest of the suite. Also a real
  feature (lowering loops-in-`main`, string building, the macro workloads), also deferred rather than
  rushed. Until it lands, the `n/a` rows above stand.

  **Since 2026-10-19 (RW 3.4, D-V2-163) it has partly landed:** `while`, `break`, `continue`, assignment to a `var`,
  `Str.len()` and a memory that grows compile, so `loop_sum_1m`, `string_build_20k` and `nested_calls_macro` run on the
  WASM engine — **4 of the 6 kernels** — each printing what the interpreter prints. `wordcount_macro` (`split`) and
  `list_map_macro` (a `List` and `map`) stay DL1201. **Re-measured 2026-10-19** (the on-demand routine's fourth run):
  Study C in release, all four lanes on one machine — the table above. The same run fixed two things in the harness
  first: it timed in whole milliseconds, which read a C process on Linux as 0 or 1 and dropped a benchmark whose C read
  0 from every ratio without a word; and it timed any lane that exited 0, whatever it printed — a lane is now timed only
  when it printed the interpreter's answer.

## What is NOT deferred, and did ship

- The Cranelift-optimized Wasmtime tier itself (`--engine wasm`), now with its optimization level
  **explicitly pinned** so the tier is a documented, drift-proof artifact rather than an implicit
  default (phase 10l).
- **Semantic parity between the interpreter and the optimizing backend**, proven at scale by the
  Stage-3 two-engine differential fuzz (50k generated programs; a 3000-program pass was re-run in
  10l against the explicitly-pinned engine and agreed on every program). Optimization changes speed,
  never meaning.
- **Authority is engine-independent by construction**: `delulu authority` is computed by the checker
  from the checked module, before any backend runs, and the `.dwx` embeds that answer hash-bound to
  the code. An optimizing backend inlines at codegen, *after* authority is computed — it has nothing
  authority-relevant to change. §2.1's "inlining cannot change authority" holds structurally, not by
  luck.

## Reproducing

```
cargo run -p delulu-measure --release -- study-c
```

Writes `measurements/study-c/REPORT.md` and `results.json` (schema `study-c/2`: microsecond timing, the
date and the machine). Before re-running, keep the run it replaces whole under `runs/<its date>/` — this
table's figures are the 2026-10-19 run's. Re-running on a different machine will move the wall-clock
numbers (and the C lane especially, being startup-bound), but not the shape of the finding: the
optimizing backend runs a subset, and DeluluLang is not competitive with C on this suite.
