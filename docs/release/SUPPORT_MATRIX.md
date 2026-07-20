# DeluluLang release trains, LTS, and the support matrix

**Status:** normative from 1.0 (Stage 10 §4, phase 10k). **Governs:** the release cadence, the LTS
designation, and the security-backport window. **Related:** `docs/design/STABILITY.md` (what is
stable), `docs/design/VERSION_COEVOLUTION.md` (broker/protocol/DIR majors across LTS windows),
`docs/design/REGISTRY_POLICY.md` (advisories and the feed).

This page is a **policy plus a schedule**. The policy binds from 1.0. The schedule below 1.0 is a
plan, and is labelled as such: at the time of writing only **1.0.0** has shipped, so every row after
it is a commitment about cadence, not a record of a release that happened. Dates are approximate
(trains are cut on readiness, not on a clock), and the table is updated as real minors ship.

## 1. Release trains

- **Minors ship on a ~12-week train.** `1.x` minor releases (`1.1`, `1.2`, …) target roughly every
  twelve weeks. A train carries additive change only within the major — new syntax, new primitives,
  new diagnostics (add-only), new stdlib — never a break to anything `STABILITY.md` §1 marks stable.
- **Patch releases ship as needed** (`1.x.y`), for security and correctness fixes, off any train
  that is still in a supported window (see §3). A patch never adds authority-observable surface.
- **A major (`2.0`) is reserved for a stable-surface break**, and does not ride the minor train. The
  co-evolution policy for a major transition — how long `1.x` and `2.x` are supported side by side —
  is `docs/design/VERSION_COEVOLUTION.md`.

## 2. LTS designation

- **Every 4th minor is an LTS** — `1.0`, `1.4`, `1.8`, `1.12`, … (minor ≡ 0 mod 4). `1.0` is the
  first LTS: it is the 1.0 stability baseline, and it anchors the whole contract in `STABILITY.md`.
- **An LTS gets 24 months of security backports** from its release date. A security fix that lands
  on the current train is **backported to every LTS still inside its 24-month window**, released as
  a patch on that LTS line, and — this is the part 10k builds — **published as an advisory with a
  DL1903 feed entry** so a `delulu build` on a still-vulnerable version is flagged, and
  `delulu build --deny-advisories` fails CI (see §4).
- **A non-LTS minor is supported until the next minor ships** (roughly the 12-week train length),
  plus a short overlap. It is not a backport target; the upgrade path off it is the next minor.

## 3. The support matrix

Status values: **Current** (the newest release, gets every fix) · **LTS** (in its 24-month
security-backport window) · **Maintenance** (a superseded non-LTS minor, security-only, until the
next minor) · **EOL** (no further releases; a build against it still works, but no fixes come).

**Planned schedule** — 1.0.0 is real (released 2026-07-20); the rest is the committed cadence, not
history. "Security-supported until" is the date through which security patches (and advisories) are
promised.

| Version | Kind | Released (planned) | Status today (2026-07-20) | Security-supported until |
|---|---|---|---|---|
| **1.0.x** | **LTS** | 2026-07-20 | **Current + LTS** | **2028-07 (24-month LTS window)** |
| 1.1.x | minor | ~2026-10 | not yet released | until 1.2 ships (+ overlap) |
| 1.2.x | minor | ~2027-01 | not yet released | until 1.3 ships (+ overlap) |
| 1.3.x | minor | ~2027-03 | not yet released | until 1.4 ships (+ overlap) |
| **1.4.x** | **LTS** | ~2027-06 | not yet released | ~2029-06 (24-month LTS window) |
| 1.5.x … 1.7.x | minor | ~2027-09 … 2028-03 | not yet released | each until its successor ships |
| **1.8.x** | **LTS** | ~2028-06 | not yet released | ~2030-06 (24-month LTS window) |

Two LTS lines are supported concurrently whenever their 24-month windows overlap (they do: 1.0's
window runs to 2028-07 and 1.4's opens ~2027-06). During that overlap a security fix is backported
to **both** live LTS lines, each with its own advisory feed entry.

## 4. How a support promise becomes a mechanism

A security-support window is only worth as much as the tooling that makes a vulnerable version
*visible*. That is Track C's second half, and it is built, not merely promised:

- A vulnerability found in a supported version is filed as an **advisory** on the registry
  (`delulu-registry advisory file`), naming the affected versions and the patched version to
  upgrade to. Advisories are stored per package and served over the feed
  (`GET /advisories/<package>`), and a token may file one only for a package it is scoped to.
- `delulu build` consults the feed (synced to a local `delulu.advisories.json`, read offline) and
  emits **DL1903 — a warning** when a resolved dependency is on an advised version. The build still
  succeeds: an advisory is information.
- **`delulu build --deny-advisories`** turns every match into an error — the CI gate. A pipeline
  that runs it will not build a known-vulnerable dependency graph. And the gate is honest about its
  own evidence: with `--deny-advisories`, a feed that is missing, unreadable, or partly unparseable
  is itself a failure, never a silent pass (`measurements/lts-cycle/RECORD.md`).

The end-to-end cycle — release, advisory, DL1903, CI gate, backported fix, clean build — is drilled
in `measurements/lts-cycle/`. The **timed** part of criterion 5 (a real 12-week train, a real
24-month backport aged in production, a real CVE/CNA registration) needs calendar time and is
recorded there as pending, not faked: DeluluLang has registered **no** CVE and is **not** a CNA
today, and no advisory in the drill is a real one.

## 5. What this does not promise

- It does not promise a fixed calendar. Trains ship on readiness; the ~12-week figure is a target,
  and a train may slip. What is promised is the *shape*: additive minors, LTS every 4th, 24-month
  LTS security windows, and the advisory mechanism that makes the window enforceable.
- It does not promise support for anything outside `STABILITY.md` §1. A behavior the reference marks
  other than `covered` is outside the guarantee until it is witnessed (invariant 42), LTS or not.
- It does not promise upstream-audited cryptography, hardware certification, or any of the honest
  boundaries the rest of Stage 10 draws. A supported version is a version that gets fixes and
  advisories — nothing more is implied.
