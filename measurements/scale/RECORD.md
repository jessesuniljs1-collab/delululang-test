# Compile-time and memory at scale (hardening campaign P12, ruling D44)

*Taken: 2026-07-26. A measurement without a date is a claim without a shelf life — these numbers describe the tree as it stood on that day, not a permanent property.*

**What this measures.** How `delulu check` behaves as programs get large, by SHAPE and not only by
line count — because "30k lines" is not one workload. Published because ruling D44 corrected an
O(fields × accesses) cost in the checker and the *residual* curve is real: a limit that is measured
and stated is a limit; an unmeasured one is a cliff someone finds in production.

**Method.** Corpora are generated, never hand-written, so each shape varies one thing. Times are wall
clock for the whole process (spawn, parse, check, exit), peak memory is `PeakWorkingSet64` sampled
while the child is alive. Release build, warm cache, best of repeated runs; Windows 11, the machine
described in `METHODOLOGY.md`. Absolute numbers are machine-specific — **the shape of the curve is the
result**, not the milliseconds.

Reproduce with the generator and harness kept out of the repo (they write hundreds of megabytes):
they are `gen.py` / `isolate.py` / `measure.ps1` as described in the D44 ruling.

## By shape

| shape | size | lines | bytes | ms | peak MB |
|---|---|---|---|---|---|
| wide — N sibling functions | 200 | 624 | 9,905 | 16 | — |
| | 1,000 | 3,024 | 47,429 | 30 | — |
| | 3,000 | 9,024 | 143,243 | 48 | — |
| | **10,000** | **30,024** | **478,519** | **173** | — |
| deep — call chain N deep | 5,000 | 15,005 | 227,873 | 98 | — |
| fatfn — one function, N statements | 10,000 | 10,008 | 257,899 | 64 | — |
| | **30,000** | **30,008** | **817,899** | **142** | — |
| types — N type declarations | 2,000 | 8,024 | 183,118 | 50 | — |
| match — one N-arm match | 1,000 | 1,010 | 27,823 | 50 | — |

Every one of these is linear or better in input size. A 30,000-line program checks in ~150 ms.

## The shape that was not linear, and what fixed it

`records_N` — an N-field record with a function reading all N fields — cost **516 ms at N=2000**,
against 173 ms for a file five times its size. Isolating the two conflated variables located it:

| N | N-field type, 1 access | 2-field type, N accesses | N locals, N-term sum | N fields, N accesses |
|---|---|---|---|---|
| 250 | 18 | 14 | 18 | 18 |
| 500 | 16 | 12 | 14 | **57** |
| 1,000 | 17 | 16 | 15 | **134** |
| 2,000 | 26 | 19 | 19 | **632** |

Declaration alone flat, accesses alone flat, expression depth flat — only the product exploding. The
cause was a whole-definition `.clone()` on the per-access path (N² field-entry deep copies), not the
name scan. After D44b:

| N | before (ms) | after (ms) | peak MB |
|---|---|---|---|
| 250 | 18 | 13 | 7.7 |
| 500 | 57 | 14 | 9.3 |
| 1,000 | 134 | 20 | 12.0 |
| 2,000 | 632 | **40** | 17.8 |
| 4,000 | — | 113 | 29.3 |

**15× at N=2000**, and 4× the input now costs ~2.9× the time instead of 11×.

**The residual is honest.** The `find()` scan remains, so the cost is still O(fields × accesses) with
a small constant — visible above as 2000→4000 costing 2.7× rather than 2×. Attributed by measurement,
not assumption: at N=4000 the declaration (59 ms), access (29 ms) and expression (32 ms) shapes are
all linear, and only their product (113 ms) is not. A name→index map would make it O(1) and is the
identified next step; thousands of fields occur in real generated code (protocol and schema bindings).

## Memory

Peak working set never exceeded **52 MB** anywhere in the corpus, including the 40,046-line file and
the 4,000-field record. Memory is not a constraint at these sizes.

## Surfaces around the compiler

Measured on the 40,046-line / 478 KB file unless noted:

| surface | result |
|---|---|
| `atlas --format tree` | 426 ms, 249,610 bytes |
| `atlas --format digest` | 423 ms, 1,636 bytes — **byte-stable across runs** |
| `atlas --format json` | 381 ms, 4,397,509 bytes |
| `atlas --format dot` | 331 ms, 1,198,481 bytes |
| `atlas --format mermaid` | 290 ms, 246 bytes (module-level by design — D41) |
| `fmt` on 30,009 lines | 1,292 ms, and the output still checks clean |
| 5,000 real errors, human | 155 ms, 14,165 bytes, 50 shown + a note naming the 4,950 withheld |
| 5,000 real errors, `--json` | 3,093,531 bytes, one object, all 5,000, `summary.errors: 5000` |

The last two are D38's diagnostic cap doing exactly what it was built for, at a scale it had not been
measured against: bounded for the human, complete for the machine.

## Monorepos

| shape | packages | check | build | lock | lockfile |
|---|---|---|---|---|---|
| chain (dependency depth = N) | 20 | 352 ms | 40 ms | 24 ms | byte-identical across writes (10,225 B) |
| chain | 50 | 528 ms | 79 ms | 53 ms | byte-identical (25,585 B) |
| diamond (all → pkg0) | 50 | 29 ms | 13 ms | 14 ms | byte-identical (1,027 B) |
| diamond | 200 | 30 ms | 13 ms | 14 ms | byte-identical (1,028 B) |

Depth costs linearly and breadth costs nothing for a leaf that does not depend on it — both correct.
**Lockfile determinism holds at every size**, which is what the semver-authority law rests on.

`delulu authority` is absent from that table because, when it was taken, it **failed** on every package
with a dependency (DL0303) while `build` on the same package succeeded — campaign finding C51, since
**closed** in P13 (ruling D45a): a `delulu.toml`'s presence now selects the resolving loader, so
`authority` answers for a monorepo member. The row is left out rather than back-filled because these
numbers were measured in one sitting and a table should say when it was taken, not be quietly patched.

## Runtime: record field access is O(record width) per read (hardening P14, ruling D47c)

The C48 fix was to the CHECKER's field lookup. The interpreter was checked for the same shape and does
**not** have it — `Interp::field` clones only the value it finds, never the whole definition. It does
scan the field list linearly, so a read costs time proportional to the record's WIDTH.

Measured with total field reads held CONSTANT (~200,000) and only the record's width varying, so a
flat row would mean width is free and a rising one is the scan itself. Release build, warm; the
`check` column is subtracted from `run` because `delulu run` checks first.

| fields | passes | field reads | check ms | run ms | µs per 1k reads |
|---|---|---|---|---|---|
| 50 | 4,000 | 200,000 | 44 | 77 | **165** |
| 200 | 1,000 | 200,000 | 45 | 105 | **300** |
| 800 | 250 | 200,000 | 59 | 373 | **1,570** |
| 3,200 | 62 | 198,400 | 173 | 1,179 | **5,071** |

A 64× wider record costs ~31× more per read: linear in width, as a scan implies.

**This is published as a named limit rather than fixed, and the reasoning is on the record.** For the
widths real programs use — five to twenty fields — a linear scan over a short `Vec` is not merely
acceptable, it is the faster representation: no hashing, no indirection, and the whole record sits in
cache. The cost only becomes visible on records wide enough to be unusual, and removing it means
either a per-instance map (paying memory on *every* record, including the narrow ones that dominate)
or resolving field indices statically through the DIR side tables. The second is the right fix and it
is a real change, not a tidy-up; it is not being attempted at the tail of a phase.

### The narrow end, which the paragraph above was guessing about (ruling D56, 2026-07-26)

*Read the previous paragraph again and then the table above it: the claim is about **five to twenty**
fields and the smallest row measured is **fifty**. The part that mattered was an extrapolation past
the data, which is the exact shape this campaign distrusts everywhere else. So it was measured.*

Same method, same constant of ~200,000 total reads, release build. Reads are inlined into the loop
rather than made through a helper, so the number is the field scan and not argument passing:

| fields | 2 | 5 | 10 | 20 | 50 | 200 |
|---|---|---|---|---|---|---|
| µs per 1k reads | 444 | 265 | 209 | **188** | 223 | 415 |

**The curve is U-shaped, and the claim survives.** Per-read cost is at its *minimum* around ten to
twenty fields and rises in both directions: at width 2 because the loop's own overhead is amortised
over only two reads, at width 200 because the scan starts to dominate. In the band the claim was
about, the field lookup is **not** the cost — interpreter overhead is, and a per-instance hash map
would add an allocation to every record in order to speed up the part that is already free.

Two harnesses now agree on the wide-region slope (200/50 is 1.86 here against 1.82 in the table
above), which is what makes both trustworthy. The absolute numbers differ because the harnesses
differ; the shape is the result, as always.

What makes this a limit and not a defect: it is linear, not quadratic, in the thing being varied, and
the constant is small. Contrast C48, which was accidentally quadratic *and* allocated on every access
— that was fixed. This is inherent to the representation and is therefore reported with a number, per
the Constitution's own rule that where DeluluLang loses, the table says so.

## Runtime memory: `--trace-effects` buffers the whole trace in RAM (hardening P14, finding C56)

`TraceSink` is a `Vec<TraceRecord>` with no bound. Every traced effect is retained until the process
exits, so memory grows linearly with the number of effects performed — not with the program's live
data.

| run | peak working set |
|---|---|
| 100,000 console writes | **6.7 MB** |
| the same, under `--trace-effects` | **70.1 MB** |

≈ 633 bytes retained per traced effect. At a thousand effects a second — an ordinary rate for the
control loops Stage 10 exists to serve — that is roughly 2.3 GB per hour.

**Open, not fixed here.** The trace is evidence and must stay complete, so the fix is not truncation
by default; it is either streaming records out as they are produced (the audit chain already does
exactly this, to day files) or bounding the buffer with an explicit notice of what was withheld (the
D38 pattern for diagnostics). Streaming is the better answer and is not a small change: the actor
runtime merges per-worker sinks at turn boundaries to stamp causality, so the ordering guarantees have
to survive the move. Recorded with a number rather than attempted at the tail of a phase.

Worth stating what this is not: `--trace-effects` is opt-in and off by default, so no production run
pays it. What makes it worth a finding anyway is *which* runs turn it on — a long-lived controller
being diagnosed in the field is precisely the case where hours of uptime meet a flag that never frees.
