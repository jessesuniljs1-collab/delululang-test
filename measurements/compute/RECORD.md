# Compute dispatch — recording and measurements

*Taken: 2026-07-20. A measurement without a date is a claim without a shelf life — these numbers describe the tree as it stood on that day, not a permanent property.*

**Stage 10 phase 10h, spec §7, acceptance criterion 7.** All numbers are from the **in-tree
`cpu-reference` adapter**. No GPU, TPU, or any other accelerator was involved, and none is claimed.

## Criterion 7, item by item

| Requirement | Status |
|---|---|
| Vendor-neutral interface passes conformance via the in-tree CPU reference adapter on every CI run | **Done** — `compute_cli.rs` (17), `compute.rs` (6), conformance fixture `27_compute.delulu` |
| At least one hardware accelerator adapter demonstrated end-to-end | **DEFERRED — see below** |
| An over-envelope dispatch is refused (DL1907) with the refusal measured | **Done** — measured here |
| An adapter that cannot attest below-adapter enforcement has its grant refused (DL1911), witnessed | **Done** — and it is the *default* path for the only adapter that ships |
| Kernels-are-data laundering tests: no closure crosses, unsigned kernel artifact refused | **Done** — DL1913 unsigned, DL1912 invalid, closure-as-kernel is a type error |

### The deferral, stated plainly

**No hardware accelerator adapter ships in Stage 10, and none was demonstrated.** Not a reduced
one, not a partial one, not one behind a flag. This machine has no GPU compute stack this project
could exercise honestly, and writing an adapter that cannot be run is how a deferral becomes a
claim.

What that costs is worth naming precisely: **the vendor-neutrality invariant (49) is tested against
exactly one adapter.** An interface that no second implementation has ever been fitted to is a
plausible interface, not a proven one. The unit test asserting nothing branches on `class` or
`adapter` shows the *dispatch path* is neutral; it cannot show the interface is *sufficient* for a
CUDA or Metal runtime, because none has tried.

This follows the invariant-45 pattern from 10b: the honest deferral is a passing outcome for the
phase, and the note travels with the criterion instead of being quietly dropped.

## What the reference adapter actually enforces

This table is the point of the file. "The envelope is enforced" is four different statements here,
and three of them are weaker than the phrase suggests.

| Term | Status |
|---|---|
| `memory_bytes` | **Enforced**, before submission, against the buffer |
| `kernel_ms` | **Enforced — after the fact.** You learn a kernel's duration by running it, so an overrun is caught on the measurement and the result is discarded. The work has already happened |
| `queue_depth` | **Enforced, and unreachable from a program.** Dispatch is synchronous, so a single-threaded run never has more than one in flight. Unit-tested with threads; the language surface cannot exceed it until async dispatch exists (RFC-gated) |
| `power_w` | **Carried, NOT enforced.** This adapter draws no measurable power and cannot attribute any. The term is mandatory in the grant so a real adapter inherits it — and it is not a bound anything checks |

`power_w` is the row that matters. It is required in every compute grant and enforced by nothing in
this build. Calling that "enforced" in a summary table is precisely how a double-enforcement claim
becomes silently single, which is the failure DL1911 exists to prevent one layer up.

The same statement lives in the code as `compute::ENFORCEMENT_NOTE`, so it cannot drift from a
document nobody re-reads.

## Measurements

**Platform: Windows 11, release build, n = 20, 2000 dispatches per run.** Linux and macOS are not
measured. Differential method: each baseline program constructs the **same buffer** its comparison
program does, so buffer construction cancels rather than landing in the result.

| Path | min | p50 | p95 | max |
|---|---|---|---|---|
| Accepted dispatch | — | **2.3 µs** | 3.8 µs | 6.1 µs |
| Refused dispatch (DL1907) | 14.4 µs | **18.7 µs** | 21.2 µs | 24.8 µs |

The accepted row's minimum sample was **negative** (−4.2 µs): at ~2 µs the differential is close to
run-to-run jitter, so read it as "a few microseconds", not as a precise figure. There is no IPC on
this path — the adapter is in-process — which is why it is so much cheaper than 10g's actuator
command (~339 µs, dominated by a broker round-trip).

### Refusing costs about 8× more than succeeding, and that is worth saying out loud

This is the **opposite** of the arm demonstration, where a refusal and an acceptance cost the same
because the round-trip dominated both. Here the refusal path formats its reason string and journals
it, while the accepted path sums four floats — so the refusal is the expensive one.

The consequence is mild and is not a security boundary: a program spamming over-envelope dispatches
burns more host CPU than one doing legitimate work. It is already bounded by its own execution and
by the envelope it cannot widen. But an asymmetry that favours the failing path is the kind of
thing that is obvious in a measurement and invisible in a design document, so it is recorded here
rather than discovered later under load.

Note also what this asymmetry does *not* do: it does not leak the envelope's contents by timing in
any useful way, because a refusal is already announced in the return value. The 10g concern about a
measurably-slower refusal leaking a bound does not apply when the refusal is the API.

## Reproducing

```
cargo build -p delulu --release
cargo test -p delulu --test compute_cli      # 17 behavioural witnesses
cargo test -p delulu-runtime --lib compute   # 6 adapter/interface witnesses
```

The measurement programs are generated by the harness rather than committed, because they are four
near-identical loop bodies whose only interesting property is that the baselines match their
comparisons; `results.json` carries the numbers above.

## What these numbers are not

They say nothing about what a kernel computes. DeluluLang bounds a kernel's **reachability** (no
dispatch without the capability), its **resources** (the envelope above), and its **provenance** (a
signed, hashed artifact). What happens inside the kernel is foreign code, outside the proof, and
`delulu authority` prints compute under the outside-the-proof separator so that boundary is visible
before anything runs:

```
  foreign:
    -- outside the proof (contained at process level) --
    - compute/gpu0 [reduce_sum]  (kernels are signed artifacts; what they compute is not proven)
```

## 2026-11-22 — today's binary, the laptop's GPU, and the experiment scheduled

*Taken 2026-11-22 on the laptop, the debug binary built from `c40e7db2`.*

**What the binary does with a program that mints `root.compute("gpu0")` and dispatches `reduce_sum`** (the program in
`crates/delulu/tests/compute_cli.rs`):

| Run | Answer |
|---|---|
| no `compute` grant | DL0703: *"compute device `gpu0` was not granted"*, naming the grant to pass |
| `adapter=cpu-reference`, attestation not waived | DL1911: it does not attest independent below-adapter enforcement, so `waive-attestation` must say single enforcement is accepted |
| `adapter=vendor-x-cuda`, attestation waived | DL1911: *"not one this build knows"* — refused rather than taken on trust |
| `--sandbox` | refused before anything runs: *"it uses Compute. Nothing ran."* |
| `--engine wasm` | DL1201: WASM codegen does not support `.compute` |

- **The laptop's GPU:** an NVIDIA GeForce RTX 4050 Laptop GPU with 6,141 MiB (`nvidia-smi`), beside the processor's
  integrated AMD Radeon 780M. A real adapter can be built and run here.
- **CI:** GitHub's GPU runners (NVIDIA T4, Linux and Windows) are larger runners, for organizations on the Team or
  Enterprise Cloud plans, billed per minute on public repositories too; `delululang-test` belongs to a personal account.
  A self-hosted runner on a public repository lets a fork's pull request run code on the host, and GitHub's own
  documentation advises against it. So CI tests an adapter's refusals and limits, and the GPU run is a laptop witness,
  as the microVM's KVM tests are.
- **The experiment** (D-V2-238, fifth amendment, step 5; RW 3.36): a signed `dot_self` on the RTX 4050, behind a build
  option, capped at 2–3 sessions. It succeeds when the named GPU runs it with no CPU fallback, the answer agrees with
  the CPU reference within a stated tolerance, a withheld grant blocks it, and the timings, transfers included, are
  recorded at several sizes. A reviewer expected the GPU to lose at every size, because each dispatch copies its buffer
  from the CPU and returns one number; that is a prediction, and the experiment records what it measures.
