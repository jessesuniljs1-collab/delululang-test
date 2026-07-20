# The fleet-update drill — recording and measurements

**Stage 10 phase 10j, spec §9.3, the second half of acceptance criterion 9.** This file was
originally scaffolded while `delulu fleet update` was still being built in parallel; it has since
been run for real, end to end, and every number below is a real, witnessed observation.

## What this measures

Criterion 9 asks for "one fleet-update drill: staged rollout, health gate, approved-hash gate
(DL1905), and rollback exercised." This demonstrates exactly that, and nothing more: a signed
artifact rolled out to `--members N` simulated fleet members, staged one at a time; a health gate
that can be made to fail a specific member on purpose (`--fail-health-at`), to prove rollback
actually happens rather than being an untested code path; the approved-hash gate that spec §9.3
names explicitly as "DL1905 generalized" from the 10f device sim-to-hardware gate — the same code,
not a new one; and rollback to a pinned prior artifact when the health gate fails.

**"Fleet members" are simulated.** There is no real network, no real fleet of machines, and no
real staged rollout across physical or even separate-process targets. This is a mechanism
demonstration of the authority/gating logic, in the same posture Stage 10's other demonstrations
take toward their own domains — `measurements/robotics-demo`'s arm and `measurements/satellite-
demo`'s spacecraft are both simulated devices; this is a simulated fleet. It is not a
distributed-systems benchmark, and no number below should be read as evidence about real network
latency, real partial-failure handling under load, or real fleet scale.

## What's demonstrated

| # | Pass | What it demonstrates |
|---|---|---|
| 1 | Clean rollout (control case) | A correctly-signed artifact with a matching approval reaches every member |
| 2 | Health-gate failure at member 2 of 5 | Rollback fires; members after the failure point are never staged — not touched then skipped, never touched |
| 3 | Hash-mismatch refusal | DL1905, generalized: an artifact edited after its approval was signed is refused before any member is touched |
| 4 | Missing approval file | DL1905 again — the skip branch. A gate that read an absent record as "nothing to check" would be the gate opening on damage |

Pass 1 is the control the other three need: without it, "the rollout stopped" in passes 2-4 would
be equally consistent with the mechanism never having worked at all — the same reason
`measurements/satellite-demo`'s NO-PASS control exists. Passes 3 and 4 are both DL1905, and
deliberately for two different reasons: an edit-after-approval says "these exact bytes were not
what was approved," and a missing file says "nothing was approved, and I cannot tell otherwise."
Collapsing the second into "no record, so nothing to check" is precisely the failure DL1905 exists
to prevent — see `crates/delulu/tests/dead_man_cli.rs`'s
`a_hardware_profile_with_no_signoff_record_is_refused_dl1905` for the device-gate precedent this
generalizes.

## The approved-hash gate, generalized

`--approved <path>` names a sign-off record in `delulu_runtime::device::Approval`'s existing JSON
shape — `approved_artifact`, `approved_hash` (`blake3:<64 hex>`), `approved_under` — reused as-is
rather than inventing a fleet-specific shape (spec §9.3: "ride the existing machinery"). The rule
is the one spec §5.4 wrote for hardware actuation, generalized one level: **the bytes a fleet
receives are the bytes that were approved, full stop.** An edit after approval, even a one-line
comment, is a different artifact at the end of a wire that updates real machines; a missing
approval record is not "trust by default" — it is a refusal, because a gate that opens when it
cannot find its evidence is not a gate.

This demonstration's fixture approval records are built by borrowing the **already-shipped** 10f
device gate (`delulu run --broker-profile sim --signoff <path>`) purely to obtain a real `blake3:`
content-hash of a fixture artifact, rather than hand-typing a plausible-looking 64-hex-character
string. See judgment call 3 below for what this assumes about the not-yet-built `fleet update`.

## Results

**Real run, 2026-07-20, Windows 11, release build.** All four passes reproduce via
`bash measurements/fleet-update/run-demo.sh` (after the one contract reconciliation noted at the
top of that script: `--previous` is required on every invocation, not only the ones expected to
fail).

### Pass-by-pass outcome

| Pass | Expected exit | Actual exit | Members completed | Members skipped / rolled back | DL1905 fired |
|---|---|---|---|---|---|
| 1 — clean rollout | 0 | 0 | 5 of 5 | none | no |
| 2 — health-gate rollback | 1 | 1 | 2 (members 0–1) | 2 (members 3–4, never staged — confirmed via the journal, not just the exit code) | no |
| 3 — hash mismatch | 1 | 1 | 0 | all 5 (refused before rollout) | yes |
| 4 — missing approval | 1 | 1 | 0 | all 5 (refused before rollout) | yes |

Pass 2's member 2 is the one that fails its health gate (staged, then fails — staging always
precedes the check); it is not counted as "completed" or "skipped," it is the failure itself.
2 (completed) + 1 (failed) + 2 (never staged) accounts for all 5 members.

### Timing

**Platform: Windows 11, release build, n = 20, whole-process wall clock** (each pass is one
`delulu fleet update` invocation, matching exactly what `run-demo.sh` runs — not a differential
against a baseline, for the reason stated below).

| Path | min | p50 | p95 | max |
|---|---|---|---|---|
| PASS 1 — clean rollout (5 members) | 68 ms | 75 ms | 89 ms | 104 ms |
| PASS 2 — rollback (5 members, fails at 2) | 67 ms | 79 ms | 89 ms | 94 ms |
| PASS 3 — hash-mismatch refusal (DL1905) | 70 ms | 75 ms | 92 ms | 94 ms |
| PASS 4 — missing-approval refusal (DL1905) | 67 ms | 71 ms | 88 ms | 104 ms |

**All four cluster in the same 67–104 ms band, and that clustering is itself the finding.** A
differential attempt was made first — the same method `measurements/robotics-demo` and
`measurements/compute` use, isolating one command's own cost from process startup by comparing
`--members 1` against `--members 50` — and it came back as noise: the per-member differential
ranged from about −450 µs to +430 µs across 20 reps, straddling zero. At `--members` in the tens,
whatever `fleet update` spends per member (a journal push, a closure call) is too small to
separate from ordinary process-to-process timing jitter on this platform. **This drill's own logic
is fast; what these numbers measure is mostly `delulu`'s process startup and package/artifact
loading, the same floor every command in this CLI pays.** Refusing costs the same as succeeding
here — unlike 10h's compute dispatch (refusing ~8× the cost) or 10g's actuator commands (refusing
the same cost) — because none of the fleet-specific work (hashing, journaling five members) is
large enough to clear that floor at this scale. A real deployment rolling out to thousands of
members, where the loop itself dominates, would be a different — and more informative —
measurement; this drill's `--members` counts are chosen to be readable in a demo, not to stress
the mechanism.

## What this is not

Carried in substance from spec §9.4's honest boundary for Track H: DeluluLang bounds **its
programs'** authority over a fleet and, at cloud scale, over cloud-provider APIs. It does not
control, and does not claim to control, a provider's control plane, IAM, billing, or hypervisor,
and a deploy/fleet plan is least-privilege *input* to that layer, never a substitute for it. This
drill demonstrates that an update mechanism refuses bytes it was not told to trust and can undo a
bad rollout; it says nothing about the transport a real fleet would use, the real availability of
members during a real rollout, or any provider's own deployment guarantees.

It is also not a signature-verification demonstration. The fixed contract handed down for this
drill names exactly one gate for `fleet update` — the approved-hash gate, DL1905 — and that is
what this drill exercises. See judgment call 1 below.

DeluluLang claims no ISO 26262, DO-178C, ECSS, or other certification for anything in Stage 10,
this drill included.

## Reproducing

```
cargo build -p delulu --release        # also done automatically by run-demo.sh itself
bash measurements/fleet-update/run-demo.sh
```

`run-demo.sh` builds the binary itself rather than trusting one that may already exist — the same
discipline `measurements/robotics-demo/run-demo.sh` adopted after an earlier draft of that exact
script measured a stale build and printed a confident page of zeros.

## Judgment calls made while scaffolding this, and how each was resolved

Written before `delulu fleet update` existed, against the fixed contract handed down for phase
10j — recorded here rather than left implicit, and updated below now that a real run has settled
each one:

1. **No `delulu sign` step on the fixture artifacts.** `fleet update` names exactly one gate —
   `--approved`, the content-hash gate (DL1905) — and verifies no detached signature on the
   artifact itself. **Confirmed correct as scaffolded**; no signature step was needed.
2. **`--previous` is passed as a file path, not a hash.** `fleet.rs` accepts either form
   (`resolve_hash_or_path`), so this was never a real fork — a path was chosen so the demo shows
   the fleet is back on *specific, re-readable content*. **The one genuine reconciliation needed**
   was elsewhere: `--previous` turned out to be **required on every invocation**, not only ones
   expected to fail (`fleet.rs`'s own stated reason: a rollback target that is only sometimes
   supplied is a bound nobody enforces — the same lesson 10f drew about `rate_hz`). Passes 1, 3
   and 4 originally omitted it; all three now pass it, and PASS 3/4's DL1905 refusals still fire
   correctly regardless of `--previous`'s value, because that gate runs and returns before
   `--previous` is ever resolved.
3. **The approval records' hashes are real, obtained via `delulu run --signoff`.** **Confirmed
   correct**: `fleet update` hashes an artifact's raw bytes with the exact same
   `delulu_broker::content_hash` function the device and compute-kernel gates use, so this
   scaffold's real hashes matched on the first real run with no fixture regeneration needed.
4. **Exact output text for member-by-member progress, health-gate failure, and rollback was
   unknown when this was written.** Now known — human-readable output is `member N: staged, health
   OK|FAILED` per member and a `fleet update: completed|ROLLED BACK ...` verdict line; `--json`
   carries a `events` array plus a top-level `outcome`. `run-demo.sh`'s checks were left as
   exit-code-plus-`DL1905`-substring (not tightened to grep the now-known exact phrases) since
   that looser check already caught the real `--previous` regression above and remains robust to
   future wording changes in the summary line — a deliberate choice to keep, not an oversight.
5. **Member indexing is assumed 0-based**, per the task's own worked example (`--members 5
   --fail-health-at 2` implies members 0 and 1 precede the failure and 3, 4 follow it). If the
   real command is 1-based, `--fail-health-at 2` in `run-demo.sh`'s PASS 2 names a different
   member than intended and the prose describing "members 0-1 / 3-4" needs updating alongside it.
