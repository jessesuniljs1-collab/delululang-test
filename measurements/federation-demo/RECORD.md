# The federated satellite demonstration — recorded run

**Phase:** RFC 0001 F6 (broker federation, demonstration). **Ruling:** `STAGE10_BUILD_ORDER.md`
D22. **Recorded:** 2026-07-22, Windows (x86_64-pc-windows-msvc), debug build.
**Re-run:** `bash measurements/federation-demo/run-demo.sh`

---

## 1. What this demonstration is for

`STAGE10_AUTONOMY_ADDENDUM.md` §2.5 named broker federation "a prerequisite for any real deployment
in this addendum's domains", and criterion 10's earlier satellite demonstration ran **both broker
roles inside one host over a simulated link** — witnessing grant *semantics*, not the federation
transport. Its recording said so.

This one is different in the ways that matter:

- **Two brokers, two ed25519 identities, no shared secret.** Neither side can mint the other's
  credentials. A lease token could never have done this: it carries no authority bytes and is MAC'd
  with a *symmetric* key, so any party able to verify is able to forge.
- **The credential crosses as a file.** No socket is opened between the two sides. Carrying the
  bytes is the operator's existing business.
- **Loss of signal is caused by geometry**, computed by the flight program itself — not by a timer.

---

## 2. The physics, and exactly how far to trust it

The flight program (`sat-federated.delulu`) computes its own orbit **in pure DeluluLang**. The
language's primitive table has arithmetic and comparison and **no transcendental functions at all**
(`crates/delulu-check/src/prim_table.rs`) — no `sqrt`, no `sin`, no `cos`. Trigonometry would
otherwise mean `foreign "c"`: the `ForeignCall` effect plus a granted binary path. So `sqrt` is
Newton–Raphson, `sin`/`cos` are range-reduced Taylor series, and `asin` is Newton on `sin`, all
built from `+ - * /` and `int()`.

A consequence worth stating: **the entire navigation half of the program is effect-free.** Not one
function that computes the orbit carries an effect row. A spacecraft's orbit determination is done
by code holding no capabilities whatsoever; only the four functions that touch the console and the
actuators carry `! {Write, Actuate}`.

That is not an assertion — the compiler reports it. `delulu authority` on the flight program:

```
Authority of `sat` — what this program can do to your system:
  effects:      Actuate, Write
  capabilities:
    - Console  stdio
    - Actuator (scope granted at runtime)
  secrets:      (none)
  pure fns:     abs, alt, asin, asin_iter, cos, deg, deg_per_rad, elev_at, elevation_deg,
                elevation_rad, floor, half_pi, inc, j2, mask_deg, mean_motion, mu, omega_earth,
                period_s, pi, raan0, raan_rate, range_km, re_km, rng_at, sat_x, sat_y, sat_z, say,
                semi_major, sin, sin_small, sqrt, sqrt_iter, stn_lat, stn_lon, stn_x, stn_y, stn_z,
                two_pi, u0, wrap_2pi, wrap_pi
  foreign:      (none — no code outside the guarantee)
```

Forty-four pure functions, and `foreign: (none)`. Every line of orbital mechanics above is proven by
the checker to be incapable of touching anything.

### 2.1 Observed self-check

```
sqrt(2)        = 1.414213562373095     expect 1.4142135623730951
sin(pi/6)      = 0.49999999999999994   expect 0.5
cos(pi/3)      = 0.5000000000000003    expect 0.5
sin(pi)        = 0.0                   expect ~0
asin(0.5)*deg  = 29.999999999999996    expect 30
period (min)   = 92.97037845453521     ISS is ~92.9
nodal drift    = -4.94664070785556 deg/day   expect ~-5
```

### 2.2 Independent cross-check — what was actually verified

The two orbital outputs were recomputed **outside DeluluLang**, in `awk` (which uses the platform
`libm`), from the same closed forms:

| Quantity | DeluluLang (hand-built math) | `awk` / libm | Agreement |
|---|---|---|---|
| Orbital period | `92.97037845453521` min | `92.97038` min | 13 significant figures |
| J2 nodal regression | `-4.94664070785556` °/day | `-4.946641` °/day | 13 significant figures |

**What that proves, precisely:** the hand-built `sqrt`/`sin`/`cos` reproduce `libm` to ~13
significant figures, so the *arithmetic implementation* is sound.

**What it does NOT prove:** it uses the *same formulas*, so it is a check of the implementation and
not of the model. The remark that these values are "close to the real ISS" is general
orbital-mechanics knowledge, **not** a comparison against a fetched operational ephemeris. No such
comparison has been made, and this document does not claim one.

### 2.3 What the model omits

- **Circular orbits only.** Eccentricity is not modelled, so there is no Kepler solver.
- **J2 secular only** (nodal regression). No higher geopotential terms, no drag, no solar radiation
  pressure, no third-body, no eclipse, no thermal, no attitude dynamics.
- **A spherical Earth for look angles**, while J2 oblateness *is* modelled in the orbit. That
  inconsistency is deliberate for tractability and costs a fraction of a degree of elevation.
- **It is an unvalidated model written by the same project that wrote the thing under test.** A
  realistic-looking simulator is more dangerous than an obviously-toy one, because it invites
  belief.

### 2.4 The load-bearing caveat

**None of this validates DeluluLang's authority claims.** The custody mechanism is entirely
indifferent to whether the plant model is a toy or NASA-grade. Better physics buys realistic
envelope, rate and lease *parameters*, and a demonstration whose LOS is caused by geometry. It buys
nothing at all about whether authority is correctly bounded — that is what the tests are for
(`crates/delulu/tests/federation_cli.rs`, and the unit tests in `delulu-broker/src/cert.rs`).

---

## 3. Observed — the pass

ISS-like orbit: 420 km circular, i = 51.64°, RAAN₀ = −49.3°, u₀ = 52°. Ground station at 48°N,
11°E, 5° horizon mask.

```
t=0s   elev=1deg  range=2173km below-mask
t=60s  elev=6deg  range=1761km VISIBLE
t=120s elev=12deg range=1353km VISIBLE
t=180s elev=22deg range=957km  VISIBLE
t=240s elev=42deg range=604km  VISIBLE
t=300s elev=88deg range=420km  VISIBLE     <- zenith
t=360s elev=43deg range=593km  VISIBLE
t=420s elev=22deg range=944km  VISIBLE
t=480s elev=12deg range=1339km VISIBLE
t=540s elev=6deg  range=1746km VISIBLE
t=600s elev=1deg  range=2159km below-mask
t=660s elev=-1deg range=2572km below-mask
```

Two internal consistency checks a reader can make without leaving this page:

1. **At the zenith the slant range equals the orbital altitude** — 420 km at 88° elevation. That is
   what "directly overhead" means, and it falls out of the geometry rather than being asserted.
2. **The profile is symmetric** about t=300 s (6/12/22/42 → 43/22/12/6). A circular orbit over a
   station produces a symmetric pass; an asymmetry would mean a sign error somewhere in the folding
   of the trigonometric range reduction.

The pass is ~9 minutes above the mask, which is the right order for a near-overhead LEO pass.

---

## 4. Observed — the federation loop

| Step | Observed |
|---|---|
| Two identities | distinct 32-byte ed25519 public keys; no shared secret anywhere |
| Ground mints, offline | `dlcert1`, 712 bytes, `parent: anchor`, corridor `slew_deg=-90..90` embedded, `uplink_ttl_ms: 3000` |
| Vehicle adopts | local **root** node, `holder: federated`, ttl = now + uplink term (not the certificate's 1 h) |
| Vehicle delegates onward | ordinary `grants delegate` from the adopted root |
| In contact | `hga COMMANDED` |
| **Loss of signal** | `DL1402: lease expired` — **nobody sent anything**; time simply passed |
| Re-contact | one signed `dlrcpt1` receipt → `hga COMMANDED` again |
| Reconciliation | 14 vehicle records bundled; the ground writes **one** `reconcile` cross-link; **both chains still verify independently** |

The uplink lease is set to **3 seconds** in the script so the demonstration fits in a terminal. On a
real vehicle it would be the contact cadence — hours or days. Nothing else changes with it; the
mechanism is the same at any scale, which is the point of measuring it in simulated seconds.

---

## 5. What this demonstration does **not** show

- **Both brokers ran on one machine, and the "link" was a file copy.** There is no radio, no
  latency, and no partition except the one created by letting a lease expire.
- **No hardware adapter exists.** `Profile::Hw` is a gate with nothing behind it (`device.rs`), so
  every device here is the in-tree deterministic simulator. This is the single largest gap between
  this demonstration and a deployment.
- **Multi-hop depth is exercised at 2**, no further.
- **Sensors are not covered.** `Scopes` still has no sensor dimension, so a lease confers no sensor.
- **DeluluLang holds no certification** under ISO 26262, DO-178C, ECSS, IEC 61508, or any other
  regime, and claims none (addendum §3). The WCET and hard-real-time refusals stand.

---

## 6. Files

| File | What it is |
|---|---|
| `sat-federated.delulu` | The flight program: orbital mechanics in pure DeluluLang + the control loop. Its header carries the same honesty caveats as §2 above. |
| `run-demo.sh` | The full two-broker loop, re-runnable. |
| `RECORD.md` | This file. |

The automated equivalents live in `crates/delulu/tests/federation_cli.rs` (eight end-to-end tests)
and `crates/delulu-broker/src/cert.rs` (the chain, adoption, uplink-lease and receipt unit tests).
The tests assert *patterns* — commanded, then not — never cycle counts or timings, so they do not
depend on how fast the interpreter runs.
