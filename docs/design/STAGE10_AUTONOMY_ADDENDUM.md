# Stage 10 Addendum — The Autonomy Domains

**Companion to:** `STAGE10_SPECIFICATION.md` (Track D §5.6 makes this addendum normative for
domain scope). **Status:** Committed design, Rev 1 (2026-07-19, at the owner's direction). No code
exists for this addendum yet; every mechanism referenced below is either already built (Stages
1–9), a named Stage-10 criterion, or explicitly named as a gap (§2.5 — broker federation,
RFC-gated). Nothing here is shipped, and nothing here may be described as shipped until its
criterion is witnessed.

**The charge (the owner's, recorded):** DeluluLang is to be a key pillar of a safe and secure
autonomous future — the language in which the command layers of autonomous vehicles, aircraft,
satellites, and robots are written and modified, by humans and by AI, under authority they cannot
exceed. This addendum is that charge turned into device classes, domain profiles, and honest
boundaries. The vision is stated once, here; everything after it is bounded.

---

## 0. The thesis — one mechanism, many domains

A leaked API key and a runaway actuator are the same bug at different stakes: code exceeding
authority it was never meant to have (Constitution §7). Stage 10 does not invent a new mechanism
per domain. Four mechanisms, all specified in the Stage-10 spec, generalize everywhere:

1. **Envelope-scoped capabilities** (§5.1) — a device capability's scope *is* its physical
   envelope, enforced host-side and adapter-side on every command.
2. **Dead-man leases** (§5.2, invariant 47) — every physical authority has a TTL and heartbeat; a
   hung or partitioned program *loses* physical authority by default. (A *compromised but
   still-running* program keeps heartbeating — what bounds it is the independently enforced
   envelope and operator revocation, not the dead-man. The dead-man defends against silence, not
   malice, and no document in this project may conflate the two.)
3. **Declared fail-states** — what the device does when authority is lost is declared at grant
   time, per device, and owned by the layer below DeluluLang.
4. **The sim-to-real hash gate** (§5.4, invariant 48; DL1905) — what was simulated is what
   deploys, or a human re-approves, every time.

What differs per domain is the device-class catalog, the command rates, the fail-state vocabulary,
the link model, and — most importantly — where the honest boundary sits.

| Domain | Command layer rate | A lease maps to | Fail-state vocabulary (owned below) | Below the boundary — not ours |
|---|---|---|---|---|
| Robot arms / AMRs | 1–100 Hz | operator/task session | `hold` / `coast` / `safe-park` | servo loops, motor drivers, torque control |
| Road vehicles | 1–50 Hz mission layer | trip / ODD segment | `controlled-stop` / `limp` / `pull-over` (ADS-owned) | the ADS perception–planning–control stack, ABS/ESC firmware |
| Aircraft / UAS | mission layer | flight phase / link session | `return-to-launch` / `loiter` / `land` / `flight-terminate` (autopilot-owned) | autopilot control laws, certified flight software |
| Spacecraft / satellites | contact-window command layer | **the contact window** | `safe-mode` (sun-point, comms-listen) | AOCS/ADCS loops, radiation-hardened flight firmware |
| Energy systems | charge-policy layer | charge/discharge session | BMS-owned `disconnect` / `derate` | cell protection, thermal management firmware |

---

## 1. Device classes

### 1.1 Actuators and sensors

Specified in the Stage-10 spec (§5.1); the addendum adds nothing to the mechanism, only catalog
breadth: joints, wheels, control surfaces, thrusters, reaction wheels, grippers, gimbals — each is
a `Cap[Actuator]` with a kind-appropriate envelope, each command validated twice, each violation
killing the command and never the process. Sensor reads are `Read` under sensor scopes.

### 1.2 Energy systems — batteries, BMS, power distribution

A battery is commanded and read like any device, with an **energy envelope**:

```json
{ "kind": "Actuator",
  "scope": { "device": "battery0/bms", "kind": "energy",
             "envelope": { "charge_a": [0, 12.0], "discharge_a": [0, 40.0],
                           "cell_temp_c": [-10, 45], "soc_pct": [15, 90] },
             "rate_hz": 1, "heartbeat_ms": 5000, "ttl_ms": 600000 } }
```

Charge-policy commands (set charge rate, schedule discharge, derate on thermal forecast) are
`Actuate` within the envelope; pack telemetry (state of charge, cell temperatures, health) is
`Read`. **The honest boundary:** the BMS firmware owns cell protection — over-current,
over-temperature, cell balancing, hard disconnect — and can override everything above it at any
time. DeluluLang sets charge *policy* within an envelope the BMS enforces independently; a
DeluluLang bug can waste energy within its envelope, and must not be able to burn a cell. The
envelope floor `soc_pct: [15, …]` is how a mission planner is *mechanically prevented* from
draining a rover's battery past the level its heaters need to survive the night.

### 1.3 Safety mechanisms — e-stop chains, interlocks, watchdogs (invariant 52)

The rule with no exceptions: **the hardware safety chain must function with DeluluLang absent,
hung, or compromised.** E-stop circuits, hardware interlocks, watchdog timers, and BMS disconnects
are electrically and logically below the adapter. DeluluLang's contribution is *supervisory,
additive, and honest about its latency*:

- `delulu grants revoke` on a device subtree is the software e-stop — same mechanism as every
  other revocation, with a **published** revoke-to-fail-state latency budget, never the word
  "instant."
- The dead-man lease (invariant 47) is a *software watchdog above* the hardware one, not a
  replacement for it.
- **No deployment profile may route a hardware safety function through a DeluluLang program.** A
  design in which the e-stop works only if the DeluluLang process is healthy is refused at review,
  every time, regardless of who proposes it.

### 1.4 Microcontrollers and accessories

An MCU is a device you **flash and talk to** (spec §7.3): firmware images are signed artifacts;
flashing is an actuation-class operation behind an explicit grant and the DL1905 approved-hash
gate; message exchange with running firmware is `Read`/`Write` under device scopes. Accessories
and payloads — grippers, camera gimbals, science instruments, lighting rigs — are simply more
device adapters with kind-appropriate envelopes. There is deliberately no "miscellaneous device"
escape hatch: a device with no adapter and no envelope gets no capability.

---

## 2. Domain profiles

### 2.1 Road vehicles

DeluluLang is the **mission/behavior layer**: route and task selection, fleet dispatch, degraded-
mode policy, payload logic — commanding an automated driving system (ADS) through envelope-scoped
capabilities (speed ceilings, geofenced operational design domain, payload actuation). The ADS
itself — perception, planning, vehicle control, and the safety case behind them — is below the
boundary and is not written in DeluluLang. Fail-states (`controlled-stop`, `limp`, `pull-over`)
are the ADS's, engaged when a lease dies. What DeluluLang adds is the property the incident report
always wishes existed: *the mission layer, by construction, could not command what it was never
granted* — "by construction," not "provably": the mechanized core proof remains future work (§3) —
and every grant, delegation, and revocation is in the audit chain.

### 2.2 Aircraft and UAS

DeluluLang is the **mission layer above the autopilot**: waypoint and task logic, payload
management, fleet coordination — never the control laws, never the certified flight stack.
The domain's defining rule: **lost-link behavior is a pre-declared attenuated grant, not
improvisation.** At mission upload, the operator delegates two grants: the mission grant (live
while the link heartbeats) and the lost-link grant (a strict `⊑` attenuation — typically
return-to-launch corridor only). Link loss is lease death; the autopilot's declared fail-state
engages under the narrower grant. The aircraft never has to *decide* what it may do when alone;
it was told, mechanically, before takeoff.

### 2.3 Spacecraft and satellites — the contact window is a lease

The deepest fit in the catalog, because spaceflight already works this way and has for sixty
years — DeluluLang makes the existing operational discipline *mechanical*:

- **The ground segment holds root.** A ground station's pass is a delegation whose TTL is the
  contact window. Command authority handover between ground stations is grant delegation; the
  audit chain *is* the command log.
- **Between contacts, the spacecraft runs on pre-attenuated autonomy grants** held and enforced
  by an **on-board broker** (the federation gap, §2.5) — station-keeping within a box, payload
  scheduling within power and thermal envelopes (§1.2's `soc_pct` floor is survival, not
  convenience), momentum management within wheel-speed envelopes.
- **Anomaly response attenuates; it never widens.** Safe-mode entry is the flight firmware's
  (AOCS/ADCS, below the boundary); what DeluluLang guarantees is that no on-board program —
  including one an AI wrote and uplinked mid-mission — can exceed the autonomy grant it holds
  while nobody is watching.

This is the criterion-10 demonstration domain: simulated, seeded, reproducible — a contact-window
lease expires at loss-of-signal, the pre-attenuated autonomy grant engages, ground re-contact
re-delegates, all witnessed.

### 2.4 Robot fleets

The Stage-10 arm demonstration (§5.5), generalized by the holder model (Constitution §5.16),
whose *rules* need no extension — the transport does (§2.5): an orchestrator — human, CI, or
LLM — holds a fleet node; each robot's supervisory program holds a `⊑` child; each robot's
devices are grandchildren. Revoking one robot kills exactly its subtree; revoking the fleet kills
everything; no robot can reach a sibling's devices, mechanically. Identical mechanics whether the
orchestrator is a warehouse dispatcher or an AI planner — no code path inspects which.

### 2.5 The named gap — broker federation (the one mechanism these profiles need that does not exist)

The satellite and fleet models above imply a grant tree that **spans machines**: a ground
segment's broker delegating a subtree to a spacecraft across an intermittent link; a fleet node
whose children live on robots. Stated plainly, against the record: **Stage 5 built a local-only
broker** (named-pipe/UDS transport) and recorded multi-machine/remote brokers as post-1.0 — and
`Actuate` is synchronous-class, validated by a broker round-trip per use, which no spacecraft can
perform against a ground broker between contacts. Cross-link autonomy therefore requires **broker
federation**: an on-vehicle broker holding a `⊑`-delegated subtree, heartbeating its uplink
lease, enforcing envelopes locally, and reconciling its audit chain at re-contact. That is a
**new mechanism, RFC-gated, and a prerequisite for any real deployment in this addendum's
domains** — no Rev-2 sentence may imply it exists. Criterion 10's satellite demonstration
therefore runs **both broker roles inside one simulated host over a simulated link**: it
witnesses the grant *semantics* (expiry, attenuation, re-delegation), not the federation
transport, and its recording says so.

---

## 3. Certification honesty (normative — carries into every domain doc verbatim)

| Domain | The certification regimes that govern it | DeluluLang's claim |
|---|---|---|
| Road vehicles | ISO 26262, ISO 21448 (SOTIF), UL 4600 | none |
| Aircraft / UAS | DO-178C, ARP4754A, national UAS rules | none |
| Spacecraft | ECSS (Europe), NASA-STD (US), national equivalents | none |
| Industrial robots | IEC 61508, ISO 10218 / ISO/TS 15066 | none |

DeluluLang is **not certified** under any of these regimes and claims no certification credit.
What it contributes is *evidence a safety case can cite*: the whole-program authority report
(`delulu authority` — the mission layer's power, enumerated), machine-checkable envelopes at every
device boundary, the append-only audit chain of every grant and revocation, deterministic seeded
simulation for scenario evidence, and the artifact-hash gate tying what was tested to what flew.
Certification of the layers below the boundary belongs to the people who build them; a mechanized
proof of DeluluLang's own core (Delulu Core) is committed future work, not a present fact. Any
sentence in any DeluluLang material that reads as a certification claim is a defect, and the
honesty review (criterion 11) treats it as one.

Two further refusals, permanent: DeluluLang offers **no worst-case execution time (WCET)
guarantee** and **no hard real-time claim** — the command layer is soft real-time with published
latency budgets, and anything with a deadline measured in microseconds lives below the boundary.

---

## 4. What this addendum refuses to promise

- **No hardware ships in Stage 10.** The named deliverables are the vendor-neutral interfaces,
  the in-tree deterministic simulators, the CPU reference adapter criterion 7 names, the
  satellite simulation criterion 10 exercises, and the recorded demonstrations — honest about
  being simulation. Real vehicles, aircraft, and spacecraft involve partners, hardware, broker
  federation (§2.5), and certification regimes this project does not control.
- **"Key pillar of a safe and secure autonomous future" is the destination, not a deliverable.**
  The deliverables are the steps that can be witnessed: envelopes that refuse, leases that die,
  fail-states that engage, hashes that gate, and audit chains that answer "who commanded that?"
  The distance between those steps and the destination is real, and this project closes it by
  shipping proof, not adjectives.
- **The threat model is stated, not universal.** Everything here defends against the program and
  its delegates exceeding granted authority — including AI-written programs, which are the
  primary users. It does not defend against a compromised adapter, forged sensor physics, a
  malicious human operator with root, or an adversary with physical access. Those defenses live
  in other layers, and the docs of every domain say exactly where.

*One mechanism, many domains, every boundary named. That is what "safe and secure autonomous
future" has to mean for the words to survive review. Be delulu; ship the proof.* 🐦‍🔥
