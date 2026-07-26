# Authority and Guard — the cross-stage capstone (hardening campaign P15)

**What this document is.** Every earlier phase tested one stage or one surface. This one asks whether
Delulu Authority and Delulu Guard are consistent *across all ten stages at once* — which is where a
system built of correct parts still fails. It discharges the two checklists the commission named, item
by item, and for each says **what was attacked, what the evidence is, and where the evidence lives.**

**What it is not.** It is not a claim that Authority is unbreakable. Several items below end in a named
limit rather than a proof, and the limits are stated in the same voice as the successes. An audit that
only records wins is not an audit.

**Method.** Nothing here is asserted from reading code alone. Each row cites either a test that runs on
every commit, a measurement in `measurements/`, or a ruling in `STAGE10_BUILD_ORDER.md` that records a
witness observed failing against the pre-fix code. Where a row rests on this phase's own new work, it
says P15.

---

## 1. The authority checklist (18 items)

### 1.1 Capability attenuation — HELD

`attenuation_check` tests all nine dimensions in **one conjunction** (effects + eight scope
dimensions), so no dimension can be silently skipped by an early return. `attenuate_core` is the
**only** path that creates a child node, and it fails closed on a missing or dead parent *including
ancestors*. Verified in P6; the device dimension's own lattice (`within` / `meet`) is tested from both
directions in `device_scope.rs`, including the counter-intuitive rule that **more dimensions is wider**.

### 1.2 Delegation — HELD

A child is created only by attenuation, so `child ⊑ parent` is an invariant of construction rather
than a check that could be bypassed. Device envelopes additionally constrain `heartbeat_ms`, `ttl_ms`,
`rate_hz` and `fail` in the narrowing direction only, each tested in both directions (P11).

### 1.3 Revocation — HELD, with the bound named and measured

- **Mid-run revocation works end to end**, with a control: `estop_cli.rs` revokes a healthy arm during
  a run and the supervisor survives to report it, while an identical run with nobody revoking keeps
  its arm. Revoking the parent stops the arm *and* the program — both blast radii witnessed.
- **P6's open question — "revocation racing an in-flight operation" — is discharged here.** There is no
  data race to find: the broker daemon is single-threaded and serializes at *request* granularity
  (one `accept` → one frame → one `handle` → one response), so a `check` and a `revoke` can never
  interleave inside the broker.
- What remains is a **logical window**, and it is the one the project already documents rather than a
  hidden one: revocation is guaranteed effective *before the next use*, not retroactively against an
  operation already authorized. Synchronous ops re-check per use; epoch-class ops validate against a
  snapshot refreshed every `--epoch-ms`. The Book's honesty clause states exactly this and forbids the
  word "immediate".
- **Measured, not asserted:** operator-to-stopped is 12.7 ms at p50 and **39.7 ms worst** over n=20
  (`measurements/robotics-demo/RECORD.md`).
- Closed along the way: a lease token for a **revoked** grant used to redeem `Ok` (C29/D36).

### 1.4 Custody — HELD

Two custody modes (embedded, daemon) sit behind one trait, so the loader and the interpreter are
written once. `validate` re-reads effective state **per operation** rather than trusting a cached
decision. A custody that cannot reach its broker fails closed (DL1401), which is the direction that
matters.

### 1.5 Authority lattice — HELD

`⊑` is one conjunction over nine dimensions; `⊓` (meet) is proved never wider than either input, is
symmetric, drops non-overlapping dimensions rather than widening them, and preserves the
`ttl ≥ heartbeat` invariant. Tested in `authority.rs` and `device_scope.rs`.

### 1.6 Authority propagation — HELD (and completed in this campaign)

Authority crossing an **actor boundary** was the last incomplete propagation path: `RootMsg` was a
hand-enumerated copy of `RootVal` and had fallen behind (C35). Every dimension now crosses (D46b), the
withheld list is **empty**, and a source-scanning gate fails the build if a future dimension is added
and not carried — in *both* directions, so a stale "withheld" claim cannot outlive the fact.

### 1.7 Authority serialization — HELD (closed in P15)

`Authority::to_json` (write) and `authority_from_json` (read) are two hand-enumerated lists of the same
eight dimensions, sitting on opposite sides of a certificate, an audit record and every `--json`
report. The **read** side already refuses an unrecognized dimension by rejecting the whole certificate
— RFC §4.9.3's load-bearing skip branch, tested. The **write** side had no equivalent protection: a
ninth dimension would simply not be emitted.

**P15 closes it** with a round-trip gate that populates every dimension, asserts write→read is the
identity, and **destructures `Scopes`** so that adding a field makes the test fail to *compile* until
someone decides how it serializes. Verified non-vacuous by deleting `foreign.python` from the writer
and watching the gate name exactly that dimension.

Why it mattered even though omission is fail-closed for the grant: the authority embedded in every
hash-chained **audit record** would have silently under-reported what a holder actually held, and
`render_compact` feeds the same list into DL0802's repair text. That is the C29/C30 class — not an
escalation, a loss of the accountability the system sells.

### 1.8 Distributed authority — HELD within a stated scope

Certificates carry authority between brokers; a chain is verified hop by hop with `⊑` enforced at each
step. The scope limit is published rather than implied: **a lease token cannot cross brokers** (its MAC
is broker-keyed, and a token presented to a different broker dies at the MAC — witnessed), and **audit
chains do not merge**.

### 1.9 Broker federation — HELD, two vulnerabilities closed earlier

- Certificate **replay could undo a revocation** — closed by single-adoption per certificate.
- A delegated child with `ttl: None` could **outlive its parent's expired uplink lease** — closed by
  inherited expiry (`effective_state_inherited` walks to the root).
- P11 re-verified both, then attacked what was uncovered: an unknown authority key, effect name or
  scope dimension **refuses the whole certificate**; a chain that does not start at a configured anchor
  dies at hop 0; **a long forged chain is not a DoS** because verification is sequential and fails at
  the first bad hop.
- Named, not a defect: single adoption is keyed **per certificate**, so a subordinate may adopt two
  roots, each separately bounded, revocable and audited.

### 1.10 Device delegation — HELD (three defects closed in P11)

The envelope grammar is the authority surface here, and it had three: a term stated twice was resolved
silently and **the two parsers resolved it oppositely** (C40), non-finite bounds made an envelope that
bounds nothing (C41), and the broker accepted any `fail=` string (C42). All refused now, and the law
that pins the two parsers was rewritten from one-directional over four good specs to **bidirectional
over a hostile corpus** (C43).

### 1.11 Adapters — HELD, with the gap named

The envelope is enforced **host-side, before one byte reaches vendor code** — proved by reading the
driver's own log: a program commanding 12° (in envelope) and 999° (out) leaves exactly **one** line in
it. An adapter can refuse more and can never permit more.

**The gap D23 named is now narrowed, not gone (rulings D52 and D53).** A driver's provenance is
checked before it is spawned: a signature that is present and does not verify refuses the run
*regardless of policy* (DL1510), absent is disclosed loudly and refusable with
`--require-signed-adapter` (DL1511), and when nothing resolves to a readable file — an
interpreter-hosted driver names the *interpreter* — the run says it could not check rather than
passing silently.

**D52 shipped that gate and it answered a weaker question than it appeared to.** `verify_detached`
reads the public key out of the signature file it is checking, and the `.sig` sits beside the driver
— so an attacker who can overwrite `drive.exe` can overwrite `drive.exe.sig` with one they signed
themselves, and D52's strongest flag accepted it. That is on the record as an observed run, not a
paragraph. **D53's answer: `--adapter-signer <hex>` pins the key** (any other signer is DL1510, the
"wrong-key" case that text always claimed), **`--adapter-artifact <path>` names which bytes were
signed** (without it, `--require-signed-adapter` was unusable for every script-hosted driver — a
control nobody can switch on), and an unpinned verify now states that it proves these bytes were
signed by that key, **not** that the key is trusted.

What remains true: this is still an operator-supplied subprocess, not spec §5.4's Verified-class
signed plugin loaded into the host. Signing buys **provenance**, not behaviour — the envelope is what
bounds behaviour. Unpinned there is still **no trust policy**, exactly as spec §10 states for
plugins. The verdict is **printed, not recorded** — no durable evidence of which key signed the
driver that drove the machine (finding C60, open). And **no driver for any real device ships
in-tree.**

### 1.12 Runtime enforcement — HELD

Authority is enforced at run time, not only in the checker: a foreign grant missing at startup is
DL1303 *before* main runs, capability scope is re-validated on every operation, and a device command is
checked twice — once against the capability value, once against the grant the broker recorded — with
the **grant winning** if they disagree (unit-witnessed).

### 1.13 Audit chain — HELD (one serious defect closed)

`hash = blake3(prev ‖ canonical_record)`, day files, append-only. **`audit tail`/`query` verified
nothing** until C30/D36: a forged `decision` displayed as authentic and a corrupted line made a record
*vanish* with no gap marker. Now the read path verifies, still shows the records (the investigator
needs them), warns loudly, exits non-zero, and always reports `chain_verified` in `--json`.

Growth is unbounded **by design** — day-file partitioned, append-only. A prunable audit log is not an
audit log.

### 1.14 Replay resistance — HELD

Certificate adoption is single-shot per certificate (so a replayed credential cannot undo a
revocation); lease tokens burn a nonce; a contact receipt cannot lengthen a lease beyond the window it
was adopted under, and replaying an old receipt cannot shorten one either.

### 1.15 Authority confusion — HELD

Guard classes key on **tree position, never holder identity** (criterion 9). `outcome_json` excludes
the holder, no decision path reads `.kind`, and a test varies the holder kind to prove the decision does
not move. P13 attacked the adjacent confusion — a lockfile misstating a dependency's authority — and
closed it.

### 1.16 Privilege escalation — HELD

Attenuation is the only child-creation path, so there is no route by which a child exceeds a parent.
P13's fifteen-case lockfile tampering matrix went from 15 accepted to 2, both remaining ones reasoned;
neither is an escalation (the manifest pin bounds a dependency independently of the lockfile).
`accepted_by` was verified to **confer nothing** — it is written by `--accept-authority` and never read
to make a decision, so forging it is a misleading label, not a privilege.

### 1.17 Authority leaks — HELD, with the honest limit stated

A raw secret **cannot cross the FFI** (DL0602 refuses it even with a shadowing trick). The authority
report names the exposure explicitly: `API_KEY declassifiable -> foreign code (outside the proof),
files/console` — reporting *capability*, not behaviour, and naming the safe case too.

The limit is stated in the report itself, not hidden: **an exposed secret is an ordinary value, and the
language cannot follow it past `expose`.**

### 1.18 Authority forgery — HELD

A certificate signature must verify **and** be made by the key the certificate names as issuer — a
valid signature by the wrong key is a forgery, not a pass. Lease-token MACs cover the *entire*
canonical payload and are checked before the claims parse. Unverifiable algorithms are refused rather
than shrugged at (DL1908). An unsigned artifact and a badly-signed one are **different codes** with
different remedies (DL1511 vs DL1705, C38).

---

## 2. The Guard surface checklist (14 items)

P6 covered the Guard's *classes and tiers* and said plainly that it had **not** covered all fourteen
surfaces individually. That debt is paid here. Three of the fourteen do not exist in v1.x, and saying
so is the honest discharge — a surface that cannot be reached needs a reason, not a checkmark.

| # | Surface | Status | Evidence |
|---|---|---|---|
| 1 | Parser | **Guarded** | Trojan Source refused (DL0107, D26) — raw bidi control characters are a hard error, scanned over raw bytes ahead of tokenizing so the rule cannot be skip-branched. A cyclic type alias no longer aborts the compiler (C54/D47a). 26 hostile programs: no panic, no hang. |
| 2 | Compiler | **Guarded** | All 16 builtin type names and 10 core effect names are reserved on all three declaration paths (C23/D30) — a shadow used to be accepted and silently inert. Alias targets resolve at their declaration (C53/D47b). |
| 3 | Optimizer | **Does not exist in v1.x** | Spec §2.1 *describes* a DIR-level optimizer (cross-package inlining, monomorphization); none is implemented. Nothing to guard, and the absence is the reason — not an omission from this audit. |
| 4 | Runtime | **Guarded** | The custody gate authorizes every effectful op **before** performing it; a denial is a fault (or, for `Actuate` alone and deliberately, a value). Capability scope is re-validated at run time, not merely type-checked. |
| 5 | WASM backend | **Guarded** | Fault parity with the interpreter is enforced (C20/D29): both engines report identical fault *codes*, with the one residual divergence (`%`-by-zero) named rather than hidden. A guest backtrace no longer floods 16,326 lines. |
| 6 | Native backend | **Does not exist — and is leashed** | No native tier ships in v1.x. `@jit` without `--grant exec.native` is DL1906, the hint is ignored, and the authority report discloses it on both surfaces. **A lease can never confer it**: `exec_native: false` is hard-coded on the lease path, so the leash holds across the federation boundary too. |
| 7 | FFI | **Guarded** | A raw secret cannot cross (DL0602). The grant gate is enforced at run time (DL1303 at startup), arity is checked, a missing row is DL0501, and `trace_foreign` fires *before* the call so it cannot be omitted by crashing. |
| 8 | Adapters | **Guarded, with a named gap** | Envelope enforced host-side before dispatch, proved from the driver's own log. Provenance checked before spawn, with the signer pinnable (D52, D53). Gaps: operator-supplied subprocess rather than a §5.4 signed plugin; **no trust policy unless the operator pins a key**; the verdict is printed, not recorded (C60). |
| 9 | Broker federation | **Guarded** | §1.9 above. |
| 10 | Hardware | **Gated, never exercised** | DL1905 refuses `--broker-profile hw:` without a sign-off record for those exact artifact bytes; `hw:` with no `--adapter-cmd` refuses rather than reporting success for a machine that never moved. **No real driver ships in-tree and no physical device has ever been commanded** — stated wherever the gate is. |
| 11 | Distributed execution | **Does not exist as a separate surface** | There is no distributed-execution crate; the distributed piece *is* broker federation (§1.9) plus the actor runtime, which is single-host and multi-threaded. Named rather than checked off. |
| 12 | Plugins | **Guarded** | **One** path to a loaded plugin's authority (`to_authority` → ceiling → holder), no alternative constructor. Class is never inferred or substituted. A present-but-invalid signature is matched **first and unconditionally**, so it refuses even when `require_signed` is false. Reload mints a fresh node, so an unload/reload authority swap is impossible. The `.dpx` reader bounds every length before allocating. |
| 13 | Certificates | **Guarded** | §1.9 and §1.18. |
| 14 | Authority envelopes | **Guarded** | §1.10, plus the simulator now charges a refused command the same time the wall clock does (C39/D43a), so a simulation can rehearse a dead-man revocation that hardware would produce. |

---

## 3. What this audit did not settle

Recorded so no reader mistakes the shape of the claim:

- **macOS has never been executed.** Windows and Linux are green at every commit; there is no Mac
  hardware. This is not "supported on three platforms".
- **No physical device has ever been commanded.** Every demonstration drives the simulator.
- **Certification is NONE.** No functional-safety or security certification of any kind.
- **The adapter has no signature check** (§1.11) — isolation and reach, not supply-chain assurance.
- **PQC is gated behind `--unstable`**: adopted implementations are unaudited by their own authors and
  have not been byte-validated against NIST vectors here. Both signing and verifying refuse without the
  flag, and verifying is the more important half.
- **`--trace-effects` buffers the whole trace in RAM** (C56, open): 70 MB per 100k effects, unbounded.
  The audit chain streams to day files; the trace does not.
- **The interpreter's field access is O(record width)** (C55): measured, published, and a named limit
  rather than a defect.
- **C21 is closed** (D51): the depth bound is now part of the API (`Interp::with_max_depth`), the
  per-frame native stack cost is measured (80 KiB per unit of depth in a debug build) and published,
  and the guard is witnessed firing on a deliberately small thread. An embedder that ignores the
  contract can still under-provision — but it is now a contract rather than a trap.

---

## 4. The pattern this campaign kept finding, stated once

Two failure shapes recurred often enough to be worth naming as design rules rather than incidents.

**A hand-maintained list of authority-bearing things falls behind the type that defines it, and
nothing notices.** Five instances: the Guard's fixed seven-element array (C31), a plugin ceiling
dropping three dimensions (C34), the actor boundary dropping `computes` (C35), the native-hint scan's
catch-all (C44), and the lockfile's recorded authority fields (C52). The answer is never "remember to
update the list" — it is to make the compiler or a source-scanning gate refuse to let the list rot.
Where a dependency edge allows it, the stronger answer is **one list referenced by both sides**
(`device_scope::FAIL_STATES`, D43d).

**A gate is blind to the failure it exists to catch — ask what SIGNAL it keys on, then ask what failure
produces a different signal.** Three instances: a coverage law that proved a witness *existed* rather
than that it *exercised* its anchor (D42a); a no-panic sweep keyed on exit 101 while the CLI
deliberately maps a worker-thread panic to exit 2 (D44c); and a sweep matching the text `panicked at`
against a stack overflow, which prints no such thing (D47a). A fourth near-miss belongs here too: the
cross-parser law that checked agreement only where both sides said *yes* (C43).
