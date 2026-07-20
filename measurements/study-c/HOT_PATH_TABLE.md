# The hot-path table — Stage 10 Track A (§2.1), acceptance criterion 1

**Stage 10 phase 10l, spec §2.1 and §11 criterion 1.** This is the published hot-path table
criterion 1 asks for, drawn from the pinned Study-C measurement (`measurements/study-c/results.json`,
release build, 2026-07-20). It is published **as-is, both the better and the worse numbers**, which
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

Lane figures are the **minimum of 5 repeats**, whole-process wall clock, from the pinned Study-C
results. `wasm ÷ C` is the criterion-1 ratio; `interp ÷ C` is shown because the interpreter is the
default engine and runs the whole suite.

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

**The geo-mean ≤ 2.5× C target cannot be met by 1.x, and the honest reason is twofold:**

1. **Coverage.** The optimizing backend runs **1 of 6** compute kernels. A geo-mean "on the
   compute-kernel suite under the optimizing backend" is not computable when the backend runs one
   sixth of the suite. On the single kernel it does run (`fib_recursive_24`) it is **2.0× C** — which
   is *within* 2.5× — but one startup-dominated point is not a suite geo-mean, and reporting it as
   one would be the dishonesty this table exists to avoid.
2. **The startup caveat cuts against us, not for us.** The C lane's figures (5–6 ms) are dominated by
   **process startup**, not compute — its spread (44–77 ms) exceeds its own minimum on every
   benchmark. So the `wasm ÷ C` and `interp ÷ C` ratios **understate** the true compute gap: the C
   denominator is mostly the cost of `fork/exec`, which the C code did not spend computing. The real
   compute gap is wider than the table shows.

The interpreter — the default engine, which runs the whole suite — is **2.0× to 60.5× C**. v1.0 is
**not competitive with C** on these workloads, and Study-C's own write-up says so in those words.

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

Writes `measurements/study-c/REPORT.md` and `results.json`. The figures in this table are the pinned
release measurement; re-running on a different machine will move the wall-clock numbers (and the C
lane especially, being startup-bound), but not the shape of the finding: the optimizing backend runs
a subset, and v1.0 is not competitive with C on this suite.
