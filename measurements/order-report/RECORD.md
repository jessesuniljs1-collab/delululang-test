# The order report against Python — a single-machine baseline and the default engine's speed target

*Taken: 2026-11-22 15:53 UTC (`results.json`). A measurement without a date is a claim without a shelf life: these
numbers describe one laptop and one binary on that day.*

**Why it exists.** D-V2-238's fifth amendment (`docs/DELULULANG_V2/V2_DECISION_LOG.md`) makes the order report the
first users' workload and its 100,000-line run the default engine's speed target (`docs/DELULULANG_AI_FIRST_RECOVERY_PLAN.md`,
the roadmap's M2). This file is the baseline that target is measured against.

**Reproduce:** `cargo build --release -p delulu`, then
`python -I measurements/order-report/bench.py target/release/delulu <a work directory outside the checkout> --reps 7 --built-from <commit> --json measurements/order-report/results.json`.
The harness exits 1 if any repetition of any lane failed.

## What is measured

- **The report suite (the gate).** `programs/orders.delulu` and its twin `programs/orders.py` — byte-for-byte copies of
  the owner's evidence folder's `S5-harness/orders.delulu` (SHA-256 `44e7b968b7622076…`) and `orders.py`
  (`06f0e2f54a8ccedc…`), unchanged. Each reads `data/orders.csv`, parses each line, groups by customer, sorts the
  totals and writes `out/report.txt`. Three lanes: plain `delulu run` (the default engine, today the interpreter),
  `delulu run --engine wasm`, and CPython. Each timing is a whole process, start-up included.
- **The fixture.** For `n` lines, `random.Random(n)` draws a customer from **60 keys** (`c00` to `c59`), an item from 9,
  a quantity from 1 to 8 and a price from 100 to 99,998 cents. Every `(n // 5)`th line is replaced by `broken,row`, so 5
  lines are refused and listed as skipped. The same 60 keys at every size, so the target is reproducible.
- **The micro suite.** Nine small programs in `programs/micro/`, each with a Python twin that prints the same line. They
  say where the report's time goes: start-up, a counting loop, recursive calls, a library sort, an in-memory group-by,
  and its four parts — f-strings, `split`, `parse_int`, and Map `get`/`insert`.

## The rules the harness enforces

- **Every repetition must pass.** It must exit 0 and write exactly the expected bytes (the report) or print exactly the
  expected line (micro). One failed repetition marks the lane FAILED, with its reason, and the lane gets no time.
- **The expected value is fixed first.** It comes from one untimed run of the Python twin, and its SHA-256 is recorded.
- **The order rotates.** Repetition `r` runs the lanes in an order rotated by `r`, so no lane always runs first or last.
- **Every raw time is kept,** in run order, in `results.json`. The tables give the median of 7.

**The negative controls (2026-11-22, the harness's own falsification).** The harness was pointed at a wrapper binary
in two modes:

| Wrapper mode | What the harness reported | Exit |
|---|---|---|
| Ran the real binary, then corrupted the report of the first of three repetitions only | `repetition 1: report differs from the expected bytes (2121 vs 2112 bytes)` | 1 |
| Exited 1 after running | each `delulu` lane FAILED, naming the exit | 1 |

The harness it replaces kept only each lane's last report and left a failed lane out of the comparison, so it would
have missed the first case. A reviewer, Codex GPT 6.2 Astra, found both flaws; see the amendment.

## Results — the laptop (Windows 11, AMD Ryzen 7 7840HS, 8 cores, 16 threads; Python 3.13.5; `delulu` built from `8eb845cb`)

**The report** (median of 7, ms; every repetition passed and wrote the expected bytes):

| Lines | `delulu run` (default) | `--engine wasm` | Python | Expected report |
|---|---|---|---|---|
| 1,000 | **18.2** | 22.1 | 49.6 | 2,112 bytes, `77a33ea90bce036f…` |
| 10,000 | 60.8 | **53.1** | 58.5 | 2,175 bytes, `7e2567f9da5ceceb…` |
| 100,000 | 519.1 (496.4–540.6) | 361.4 (349.5–394.3) | **148.6** (145.3–155.5) | 2,235 bytes, `bddfd4b933f48116…` |

**The micro suite** (median of 7, ms; every repetition printed the expected line):

| Program | `delulu run` (default) | `--engine wasm` | Python |
|---|---|---|---|
| start-up (prints `0`) | 9.8 | 14.1 | 21.6 |
| a counting loop, 10,000,000 steps | 1639.6 | 39.7 | 818.8 |
| `fib(30)`, recursive | 627.1 | 23.8 | 95.4 |
| sort 200,000 `Int`s (library `sort`) | 86.8 | 22.6 | 88.9 |
| group-by, 200,000 in-memory lines | 625.2 | 245.0 | 115.2 |
| — its f-strings alone | 253.6 | 27.0 | 65.8 |
| — its `split` alone | 140.9 | 108.7 | 51.3 |
| — its `parse_int` alone | 95.8 | 55.4 | 40.3 |
| — its Map `get`/`insert` alone | 206.6 | 86.4 | 46.4 |

## What it shows

- **A small job is already faster than Python.** At 1,000 lines `delulu run` takes 18.2 ms and Python 49.6 ms: start-up
  dominates, and DeluluLang starts faster.
- **At 10,000 lines they tie.** At 100,000 lines Python is fastest: 148.6 ms, against 361.4 ms on WASM and 519.1 ms on
  the default engine.
- **The compiled code is not the gap.** On WASM, the counting loop, the calls, the sort and the f-strings beat Python.
  `split`, `parse_int` and Map `get`/`insert` lose to it (108.7, 55.4 and 86.4 ms against 51.3, 40.3 and 46.4).
  Those are the built-in operations the report leans on (`REMAINING_WORK.md` RW 3.35).
- **The default engine is the interpreter.** It is slowest on loops and calls (RW 3.24). Plain `delulu run` picking WASM
  where it can is the roadmap's M2.

**The target (D-V2-238, fifth amendment; the roadmap's M2).** On the 100,000-line fixture, plain `delulu run`'s median
is at most Python's on the same machine and run. Every repetition of every lane must exit 0 and write the expected
bytes, or the run does not count. Today it is 519.1 ms against 148.6 ms, so the target is not met.

## Threats to validity

- **One machine, one run, on Windows.** A laptop not isolated from its own background work; no Linux or macOS
  numbers; the spread is in `results.json`. The target is read on the same machine as Python, never against an old
  number.
- **Whole processes.** Start-up is included on purpose — that is what a user waits for — so small inputs measure
  start-up more than work.
- **One workload shape.** 60 customers, short lines, ASCII. A report with many more distinct keys, longer lines or
  non-ASCII text could weigh `split`, the Map and the sort differently.
- **The Python twin is the reference.** A defect shared by both programs would go unseen. The 100,000-line report was
  read by hand: 65 lines, the 60 customers in descending order of total, then the 5 skipped lines.
- **`delulu --version` does not name its commit** (the roadmap's M4). The binary's commit is the `--built-from` the
  caller passed.
