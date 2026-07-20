# The satellite scenario — recording

**Stage 10 phase 10g, spec §5.6, addendum §2.3, acceptance criterion 10.** The second simulated
domain beyond the arm.

## The honesty note, first, verbatim from addendum §2.5

> Criterion 10's satellite demonstration therefore runs **both broker roles inside one simulated
> host** over a simulated link: it witnesses the grant *semantics* (expiry, attenuation,
> re-delegation), not the federation transport, and its recording says so.

This is that recording, and it says so. **There is no on-board broker, no ground broker, and no
link between them.** Stage 5 built a local-only broker; `Actuate` is synchronous-class, validated
by a broker round-trip per use, which no spacecraft can perform against a ground broker between
contacts. Cross-link autonomy therefore requires **broker federation** — an on-vehicle broker
holding a `⊑`-delegated subtree, heartbeating its uplink lease, enforcing envelopes locally and
reconciling its audit chain at re-contact. That is a new mechanism, RFC-gated, and **a
prerequisite for any real deployment in this domain.** Nothing below may be read as evidence that
it exists.

Nothing here simulates a radio, an orbit, or a spacecraft. What is simulated is a *device*, and
what is demonstrated is what happens to *authority* when a contact window closes.

## The mapping

The demonstration is worth something only because the mapping from the domain to the mechanism is
exact rather than decorative:

| Spaceflight | This system | Enforced by |
|---|---|---|
| A ground station's pass | A lease with `ttl_ms` = the contact window | The dead-man watchdog (10f) |
| Loss of signal | That TTL expiring — with nobody to send a message, because at LOS there is nobody | `RevokeCause::TtlExpired` |
| Pre-attenuated autonomy grant | A second device grant, narrow envelope, TTL outliving the pass | The envelope check (10e) |
| Anomaly response attenuates, never widens | A wide slew offered to the narrow grant, refused — before *and after* LOS | The envelope check |
| Ground re-contact re-delegates | A new run under a new grant with a fresh lease | The grant is issued again |

**The contact window is the lease's `ttl_ms`.** That is the whole idea. Spaceflight has worked
this way for sixty years; what Stage 10 adds is that it is now mechanical rather than procedural.

## What one pass looks like

One program, one continuous run, holding two subsystems under two different grants:

- `sat0/hga` — the ground-delegated authority for this pass. Wide (±45° slew),
  `ttl_ms = 250`, `fail = safe-park`.
- `sat0/wheels` — the pre-attenuated autonomy grant. Narrow (±0.5°), `ttl_ms = 600000`,
  `fail = hold`.

The program does not know which is which. It commands both on every cycle and reports what it is
told. **Nothing in it polls for loss of signal, checks a clock, or handles a "you are now
autonomous" event** — a spacecraft that must be *told* it has lost the ground has already assumed
the one thing it cannot assume.

Observed (this machine, release build, 100 cycles):

```
=== PASS 1 — acquisition of signal, then LOS mid-pass ===
  hga  commanded: 17   revoked: 83   no-device: 0
  wheels commanded: 100   wide slew refused: 100
  loss of signal at output line 71
  lease revoked (ttl-expired), heartbeat_ms=200, ttl_ms=250, held 5127 µs past its ttl
  hga REVOKED: the lease on `sat0/hga` was revoked (ttl-expired); the `safe-park` fail-state is engaged
```

Read the second row carefully. **The wheels never stop**, before or after LOS — the autonomy grant
"engages" by being the grant that did not end. And **the wide slew is refused on every one of the
100 cycles**, including every cycle after the ground is gone. Losing supervision is precisely when
a system must not acquire authority.

The exact cycle at which LOS falls is machine-dependent — it is wall-clock against a TTL — so the
tests assert the *pattern* (commanded before, never commanded after, cause is the TTL) and never a
cycle count.

## Re-contact

Pass 2 is a separate run with a fresh grant. The HGA commands again, then expires again on its own.

**Authority does not come back; it is issued again.** The expired lease is not revived and the old
node is not reopened — a new pass is a new delegation. That is the honest shape and it is also the
safe one: a system that could resurrect an expired authority would have a path from "the window
closed" back to "the window is open" that no operator authorised.

## The control

The same program with **no ground delegation at all**: the HGA is refused at the **mint**
(`DL0703: actuator sat0/hga was not granted`) — earlier, and for a different reason, than a lease
expiring. The autonomy grant still works, so the run is not simply broken.

Without this control, "the HGA stopped working" would be equally consistent with the ground grant
never having done anything at all.

Likewise, the HGA's heartbeat (200 ms) is longer than the program's cycle time, and the test
asserts `missed-heartbeat` never appears. A program that simply stopped beating would also lose
the HGA and would look identical in the output. The scenario has to prove it is demonstrating the
*contact window* and not the dead-man.

## Reproducing

```
bash measurements/satellite-demo/run-demo.sh     # builds, then runs both passes and the control
cargo test -p delulu --test satellite_demo       # the same three witnesses, per commit
```

## A second named gap, found while building this

Beyond §2.5's federation gap, this phase surfaced a related and narrower one, stated here rather
than discovered later by someone trusting a delegation:

**A grant tree node carries the authority to actuate; it cannot carry an envelope.**
`delulu_broker::Scopes` has dimensions for files, network, secrets and foreign libraries — and
none for a device. So a delegating party can say "you may actuate" but not "you may slew ±5°"; the
envelope is applied where the program runs, by that host's device broker, from that host's
`--grant`.

For an arm on a bench that is a modelling detail. **For a spacecraft it is the crux**, because the
ground segment is supposed to be the authority. Until a device envelope is expressible in a grant
node, a ground station cannot delegate a *bounded* one.

The consequence is enforced rather than papered over: `delulu run --lease` now **refuses** a local
`--grant actuator=`/`sensor=` and says why. Before 10g it accepted the flag, silently discarded it,
and let the program die at the mint with `DL0703` — blaming the program for the CLI having thrown
the grant away. See ruling D12e in `docs/design/STAGE10_BUILD_ORDER.md`.

## Certification honesty (addendum §3, carried verbatim)

DeluluLang claims **no** ISO 26262, DO-178C, ECSS, or any other certification. It is the
command/mission layer above certified firmware — never the servo loop, never the airworthy
autopilot, never the BMS cell protection, never safe-mode entry, which belongs to the flight
firmware below the boundary. It produces evidence a safety case can cite; it is not one. And
invariant 52 stands: the hardware safety chain must not depend on DeluluLang existing.
