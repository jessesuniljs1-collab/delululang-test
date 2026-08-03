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

## What these models do NOT cover

Named so the `model-checked` category is not read wider than it is:

- **Three nodes, two effects, clock ≤ 2, audit ≤ 6, epoch ≤ 2.** Bounded model checking. Invariants
  hold over every reachable state *within that bound*, which is not a proof for all sizes.
- **Lease tokens, redemption, and certificate adoption are NOT modelled yet** — `lease.rs` and
  `cert.rs` are untouched here. Two real vulnerabilities were previously found in exactly that area
  (certificate replay undoing a revocation; inherited uplink expiry), so this is the most valuable
  place to extend.
- **No concurrency.** Actions are atomic and interleaved by TLC, but the model has no notion of two
  brokers, a partition, or clock skew. Federation is unmodelled.
- **The model is hand-written from the code.** Nothing mechanically checks that it stays faithful
  when `tree.rs` changes. The `file:line` citations are the only link, and they are maintained by
  hand — the project's own design rule 1 says such links rot. Treat a passing run as evidence about
  *the model*, and the citations as the claim that the model matches the code.
