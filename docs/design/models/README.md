# Formal models — `model-checked` evidence

Machine-checkable models of DeluluLang subsystems, with the checker's **verbatim output**. A
model-checking claim without the output is an assertion, not evidence.

**Tool:** TLA+ / TLC **v1.7.4** (`tla2tools.jar`), Java 21 (Eclipse Adoptium 21.0.8).
The jar is **not vendored** — it is a 2.2 MB binary from
<https://github.com/tlaplus/tlaplus/releases>. Fetch it and run:

```sh
java -XX:+UseParallelGC -cp tla2tools.jar tlc2.TLC -config Broker.cfg    Broker.tla
java -XX:+UseParallelGC -cp tla2tools.jar tlc2.TLC -config BrokerBug.cfg Broker.tla
```

---

## `Broker.tla` — the custody grant tree

Models `crates/delulu-broker/src/tree.rs`: **grant → delegate/attenuate → revoke → expire**, with a
clock. Every action's guard cites the `file:line` it mirrors, and where the code is ambiguous the
model takes the **weaker** guard, so the model can never be kinder than the implementation.

Authority is abstracted to a subset of a two-element effect set. This checks the **protocol**, not
the path algebra — the ordering laws of `⊑` and `⊓` are checked separately and exhaustively by
`crates/delulu-broker/tests/order_laws.rs`.

### Invariants

| Invariant | Mirrors |
|---|---|
| `AttenuationInv` | every non-root node's authority is within its parent's — `tree.rs:450` |
| `RevokeCoversSubtree` | revocation is transitive at write time — `tree.rs:616` |
| `NoResurrection` | a revoked node is never usable again |
| `NoUsableOrphan` | **the load-bearing one** — a node the enforcement path treats as usable has no dead ancestor — `tree.rs:576-590` |
| `AuditAppendOnly`, `EpochMonotone` | temporal: the audit chain and revocation epoch never go backwards |

### Result 1 — today's code (`INHERIT_EXPIRY = TRUE`)

```text
Model checking completed. No error has been found.
3306347 states generated, 585771 distinct states found, 0 states left on queue.
The depth of the complete state graph search is 8.
Finished in 53s
```

### Result 2 — the teeth test (`INHERIT_EXPIRY = FALSE`)

**A model that has never caught anything proves nothing.** So the same spec is re-run with the
enforcement read switched to per-node expiry only — the behaviour *before* RFC 0001 F4 — and TLC is
required to fail. It does, at depth 4, and the counterexample is the **real historical bug**: a
child with no TTL outliving its parent's expired lease.

```text
State 2: <Grant>     ttl = (n0 :> 1 @@ ...)          state = (n0 :> "Live" @@ ...)
State 3: <Delegate>  ttl = (n0 :> 1 @@ n1 :> NoTTL)  parent = (n1 :> n0)
State 4: <Tick>      clock = 1        \* n0's deadline has passed; n1 is still usable
Invariant NoUsableOrphan is violated.
30495 states generated, 17190 distinct states found
```

`tree.rs:557-575` records why the walk exists in the code's own words: *"a holder could therefore
delegate itself a child with `ttl_millis: None` and keep commanding after its own lease died…
A subtree that can outlive its root is a federated grant that cannot be timed out."* The model
reconstructs that defect independently, from the guards alone.

---

---

## `Custody.tla` — leases and certificate adoption

Models `lease.rs` (delegate → mint → redeem, single-use nonces, `rotate_key`) and `cert.rs`
(adoption, single-adoption-per-broker-lifetime, uplink deadlines). **This is the part where both of
this project's real vulnerabilities lived**, and the part `Broker.tla` did not reach.

Two historical fixes are modelled as **switches**, so the model can be shown to have teeth twice:

| Switch | Models | Off = the original defect |
|---|---|---|
| `SINGLE_ADOPTION` | `cert.rs:544-554` | certificate replay undoing a revocation |
| `LIVE_ON_REDEEM` | `lease.rs:207-218` | campaign finding **C29** — redeeming a dead grant |

### Result 1 — today's code (both switches TRUE)

```text
Model checking completed. No error has been found.
7831 states generated, 2421 distinct states found, 0 states left on queue.
The depth of the complete state graph search is 9.
```

Invariants held: `SingleUseHolds` (a non-multi token spends at most once), `NoRedemptionOfADeadGrant`,
`RotatedTokensAreDead`, `RevocationSurvivesReadoption`.

### Result 2 — teeth test: certificate replay (`SINGLE_ADOPTION = FALSE`)

```text
Error: Invariant RevocationSurvivesReadoption is violated.
State 2: <Adopt>   state = (n0 :> "Live")     certOf = (n0 :> c0)   adoptedAs = (c0 :> n0)
State 3: <Adopt>   state = (n1 :> "Live")     certOf = (n1 :> c0)   adoptedAs = (c0 :> n1)
State 4: <Revoke>  state = (n0 :> "Revoked" @@ n1 :> "Live")
```

One credential, adopted twice, so revoking the node it was adopted as leaves a second live node
carrying the same authority. `cert.rs:549` refuses exactly this, in its own words: *"re-presenting a
credential must not undo a revocation."*

### Result 3 — teeth test: C29 (`LIVE_ON_REDEEM = FALSE`)

```text
Error: Invariant NoRedemptionOfADeadGrant is violated.
State 2: <Adopt>     n0 Live
State 3: <Delegate>  n1 Live under n0, token t0 bound to n1
State 4: <Revoke>    n0 AND n1 Revoked  (revocation is transitive at write time)
State 5: <Redeem>    tokRedeems = (t0 :> 1)   redeemedDead = TRUE
```

The redemption succeeds against a revoked node — which is what wrote `decision: "allow"` into the
audit chain for a grant an operator had killed. `lease.rs:188-206` describes the same defect at
length; the model reconstructs it from the guards alone.

### A trap worth recording

`redeemedDead' = redeemedDead \/ X` is **wrong** in TLA+: `=` binds tighter than `\/`, so it parses
as `(redeemedDead' = redeemedDead) \/ X` — a disjunction leaving the primed variable unconstrained,
which TLC reports as `null` rather than as an error you would notice. The parentheses in
`redeemedDead' = (redeemedDead \/ X)` are load-bearing. Caught here only because the teeth test was
*expected* to fail and failed the wrong way.

---

## `authority_algebra.py` — the nine-dimension order, proved in Z3

```sh
pip install z3-solver
python docs/design/models/authority_algebra.py
```

Symbolic verification of what `attenuation_check` actually computes (`authority.rs:150-165`): the
conjunction of seven exact-set dimensions, two path dimensions and the device dimension. **17
obligations, all discharged**, `RESULT: every obligation discharged`.

| Group | Proved |
|---|---|
| Set dimensions (`authority.rs:151-158`) | reflexive, transitive, **antisymmetric**, meet is a lower bound, meet is the **GLB**, idempotent, commutative, associative |
| Device (`device_scope.rs::within`/`::meet`) | reflexive, transitive, meet is a lower bound, meet is the **GLB**, antisymmetric on its fields |
| **The full conjunction** | reflexive, transitive, **meet ⊑ both operands — the no-widening law, all nine dimensions at once**, meet is the GLB |

The device model is faithful to the three asymmetries that are easy to get backwards: a **smaller
heartbeat is narrower**, a **smaller ttl is narrower**, and an **unbounded rate under a bounded
parent is a widening** (`device_scope.rs:222-229`). Well-formedness assumes what the parser
enforces — ordered intervals and `ttl ≥ heartbeat`.

### Where F1 actually comes from

Every dimension modelled here is antisymmetric **on its own representation**, including the device
fields. So the preorder finding (F1) is **not a property of the algebra** — it comes from the path
dimension's *encoding*: `./data` and `data` are distinct `String`s denoting one path, so two
structurally unequal `Authority` values are mutually `⊑`. That is why F1 is proved by exhaustive
enumeration over real path strings in `crates/delulu-broker/tests/order_laws.rs` and not here.
Knowing *which layer* the defect lives in is the useful part: fixing it is a canonicalization
change, not an algebra change.

### An obligation that was removed rather than kept

An earlier draft carried a "NO WIDENING" line encoded as `Implies(Not(Or(..., True)), True)` —
i.e. `Implies(False, True)`, **vacuously true**. Z3 discharged it and printed `PROVED` while
checking nothing. It has been deleted and the reason recorded in the source. **A vacuous obligation
reported as proved is worse than a missing one**, because the list is meant to be the evidence.
The real no-widening statement is the lower-bound obligation, proved for every pair of values.

---

## What these models do NOT cover

Named so the `model-checked` category is not read wider than it is:

- **Three nodes, two effects, clock ≤ 2, audit ≤ 6, epoch ≤ 2.** Bounded model checking. Invariants
  hold over every reachable state *within that bound*, which is not a proof for all sizes.
- ~~Lease tokens, redemption, and certificate adoption are NOT modelled~~ — **now covered by
  `Custody.tla`.** Still absent from it: the MAC itself (key rotation is modelled as an epoch
  counter, not as blake3), audit-chain hashing, and contact receipts extending a deadline.
- **Authority is abstracted away in `Custody.tla`.** It models the binding, deadline and state
  machine; the `⊑` lattice is `Broker.tla`'s job and `order_laws.rs`'s.
- **No concurrency.** Actions are atomic and interleaved by TLC, but the model has no notion of two
  brokers, a partition, or clock skew. Federation is unmodelled — which matters, because the uplink
  lease exists precisely to bound what happens during a partition.
- **The model is hand-written from the code.** Nothing mechanically checks that it stays faithful
  when `tree.rs` changes. The `file:line` citations are the only link, and they are maintained by
  hand — the project's own design rule 1 says such links rot. Treat a passing run as evidence about
  *the model*, and the citations as the claim that the model matches the code.
