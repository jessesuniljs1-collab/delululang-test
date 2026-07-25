# The arm demonstration — recording and measurements

*Taken: 2026-07-20. A measurement without a date is a claim without a shelf life — these numbers describe the tree as it stood on that day, not a permanent property.*

**Stage 10 phase 10g, spec §5.5, acceptance criterion 4.** Everything below runs against the
in-tree reference simulator (`--broker-profile sim`). **No hardware was involved and none is
claimed.** A hardware adapter publishes its own numbers or it does not ship; that is what "per
adapter" means, and it is why this file names its adapter in the first sentence.

## What the demonstration is

An agent edits a simulated arm's control program three times. The first edit is correct, the
second asks for more torque than the operator granted, and the third puts a long computation in
the control loop. Then, with the corrected program running healthily, an operator stops it from
another terminal.

**The agent here is a script, not a model.** Spec §5.5 calls for a "scripted LLM harness"; the
edits in `arm.delulu`, `arm-overtorque.delulu` and `arm-wedged.delulu` are the edits such a harness
produces, written out and committed so the demonstration reproduces byte-for-byte. No model runs
during the demonstration, and nothing here should be read as a measurement of one. What is being
demonstrated is the *boundary*, which does not know or care what wrote the program on the other
side of it.

The four behaviors, and what each one shows:

| # | Behavior | What it demonstrates |
|---|---|---|
| 1 | 20 in-envelope commands land | Correct edits take effect, at rate |
| 2 | `torque_nm: 5.0` against a `0..2.5` envelope | Refusal is **per command**, by name, and the controller keeps running |
| 3 | The controller thinks past its 250 ms heartbeat | The lease dies **unasked**; `safe-park` engages while the program is still computing |
| 4 | `delulu grants revoke` on the arm's node | An operator stops the machine without the program's cooperation |

Plus the staged-deploy gate: a `hw:` profile with no sign-off record is refused (DL1905).

## Reproducing

```
cargo build -p delulu --release
bash   measurements/robotics-demo/run-demo.sh      # shows the four behaviors (bash only)
python measurements/robotics-demo/measure.py       # produces the numbers below (bash + python3)
```

`run-demo.sh` builds the binary itself. An earlier draft of it preferred an existing
`target/release` build, found one predating the feature, and printed a confident page of zeros —
so the script no longer trusts a binary it did not just build.

## Measurements

**Platform: Windows 11, release build, n = 20.** Linux and macOS are **not measured here.**
Extrapolating a scheduler-sensitive number across operating systems would be inventing evidence.

### What one command costs

Differential measurement: the same 2 000-iteration loop with and without the command
(`bench-ok.delulu` / `bench-refused.delulu` vs `bench-none.delulu`), so process startup, parsing,
checking and interpreter overhead cancel.

| Custody mode | Command | min | p50 | p95 | max |
|---|---|---|---|---|---|
| embedded | accepted | — | — | — | **< 1.4 µs** |
| embedded | refused | — | — | — | **< 1.4 µs** |
| daemon | accepted | 276 µs | 339 µs | 353 µs | **354 µs** |
| daemon | refused | 281 µs | 339 µs | 350 µs | **353 µs** |

**The embedded row is a bound, not a value.** The differential there is smaller than run-to-run
jitter — individual samples ranged from −0.8 µs to +1.4 µs, i.e. the loop with 2 000 commands
sometimes finished *faster* than the loop without them. All this method can honestly say is that
an embedded command costs under about 1.4 µs. Reporting the median of that noise as a
measurement would be dressing up a coin flip.

**The daemon row is the one that matters**, because daemon custody is where the grant tree lives
and therefore where an e-stop is possible at all. 10g put a broker round-trip on the command path,
and this is its price: **~339 µs per command, ~354 µs worst observed** — a ceiling of roughly
**2.8 kHz**. Spec §5.3 places DeluluLang at the 1–100 Hz command/policy layer, so the round-trip
costs about 3 % of the period at 100 Hz. It would not survive a 1 kHz servo loop, which is exactly
why §5.3 says DeluluLang is not the servo loop.

**Refusing costs the same as accepting** (339 µs vs 339 µs at p50). The round-trip dominates
completely; the envelope check itself is free at this resolution. There is no cheap path that
skips validation and no expensive path that performs it — which is the property you want, because
a refusal that were measurably slower would leak the envelope's contents to anyone timing it.

### Heartbeat loss → fail-state (behavior 3)

`heartbeat_ms = 250`, watchdog tick 25 ms (`clamp(heartbeat_ms / 4, 1 ms, 25 ms)`).

| Term | min | p50 | p95 | max |
|---|---|---|---|---|
| `overdue` — how late the beat was when the watchdog noticed | 3.54 ms | 4.53 ms | 4.97 ms | **5.15 ms** |
| `engage` — detection → `safe-park` applied at the adapter | 1 µs | 1 µs | 1 µs | **2 µs** |

**Worst observed, missed beat to fail-state engaged: 250 ms + 5.15 ms + 2 µs ≈ 255 ms.**

The `overdue` spread is much tighter than the 25 ms tick would suggest, and the reason is an
artifact of this scenario rather than a property to rely on: the lease's only heartbeat arrives
just after the broker starts, so the deadline and the tick sequence share an origin and the
crossing tick lands a few milliseconds past it (cumulative sleep overhead, ~0.4 ms per tick over
ten ticks). **Plan against the tick, not against this table** — a program that beats at arbitrary
times will see `overdue` anywhere in `[0, tick]`.

### The operator e-stop (behavior 4)

Measured from the harness's own clock: the instant before `delulu grants revoke` is invoked, to
the instant the harness reads the program's `REVOKED` line.

| Term | min | p50 | p95 | max |
|---|---|---|---|---|
| `revoke` call — the operator's command returns | 10.9 ms | 11.9 ms | 15.0 ms | **16.6 ms** |
| **end to end** — revoke issued → program sees `REVOKED` | 10.0 ms | 12.7 ms | 38.6 ms | **39.7 ms** |
| `engage` — fail-state applied at the adapter | ≤ 1 µs | ≤ 1 µs | ≤ 1 µs | **1 µs** |

**Worst observed operator-to-stopped: 39.7 ms.** The bimodality is the watchdog tick: when the
revocation commits just before a tick the total is ~10–13 ms, and when it just misses one it is
~39 ms. That is `revoke call + up to one 25 ms tick + engage`, which is what the budget predicts.

`engage` is reported as **≤ 1 µs** rather than 0. Several samples read zero, but that is the
timer's microsecond quantization, not an operation that took no time.

The end-to-end figure includes one command's worth of program-side slack — the gap between the
watchdog engaging the fail-state and the program's next command discovering it. `bench-supervisor.delulu`
exists to keep that slack small: it does nothing but command, so the slack is one command
(~339 µs, above), not one iteration of a planner. **The arm stops before the program is told**;
the program being told is what the harness can observe from outside.

### The budget

Criterion 4 asks that these be "within the adapter's published budget". For the simulator, this
document *is* the published budget:

| Path | Budget | Worst observed |
|---|---|---|
| Envelope refusal | one command round-trip | 353 µs |
| Heartbeat loss → fail-state | `heartbeat_ms` + tick + adapter | ≈ 255 ms at `heartbeat_ms = 250` |
| Operator e-stop → fail-state | revoke call + tick + adapter | 39.7 ms |

## What these numbers are not

They are not a safety case and not a real-time guarantee. Nothing here changes spec §5.3's
boundary: DeluluLang commands the policy/command layer at 1–100 Hz, and the mechanisms that must
hold when *everything* software fails — interlocks, e-stop chains, firmware limits — live below the
adapter and must not depend on DeluluLang existing (invariant 52). A dead-man lease and an operator
e-stop both shorten the window in which a program can keep commanding a machine. Neither closes
it, and no adapter's numbers can.

DeluluLang claims no ISO 26262, DO-178C, ECSS, or any other certification. It produces evidence a
safety case can cite; it is not one.

## One defect this demonstration found

The measurement harness could not stop the arm, and the reason was not in the harness.

A run's **device grant nodes were outliving the run**. After three benchmark programs had come and
gone, `delulu grants list` showed three `[live]` nodes for `arm0/elbow` and no way to tell which —
if any — a live process still held. Revoking one printed `ok: revoked 1 node(s)` and stopped
nothing.

**A successful-looking e-stop is worse than a missing one**, because it ends the search for the
real one. A run now revokes its device nodes as it exits; the regression is pinned by
`a_device_node_does_not_outlive_the_run_that_minted_it` in `crates/delulu/tests/estop_cli.rs`.

The revoked nodes stay visible in the listing rather than being deleted — the audit chain is a
record of what was held and when, and erasing it to tidy a display would trade evidence for
cosmetics. The residual is stated rather than fixed: a run's own `(process)` node still outlives
it, which is Stage 5 behaviour shared with every run and relied on by `run --lease`. What no
longer outlives a run is a **device**, which is what an e-stop is aimed at.
