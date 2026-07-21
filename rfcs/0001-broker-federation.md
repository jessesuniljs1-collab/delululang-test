# RFC 0001 — Broker federation: a grant tree that spans machines

- **Status:** draft
- **Author(s):** Claude Opus 4.8 (AI-authored)
- **Sponsor:** *(unfilled — an AI-authored RFC may not name its own sponsor; requires a named human
  who is accountable, per `CONTRIBUTING.md` §4. Until this line is filled, this RFC cannot open for
  comment.)*
- **Opened:** 2026-07-22 *(draft written; the comment clock starts when a sponsor signs, not now)*
- **Comment period closes:** *(sponsor-set, ≥ 14 days after opening)*
- **Touches Constitution §1 / §2 / §5.14 honesty limits:** **yes** — the broker's threat model
  changes from "OS-authenticated same user" to "a signed credential from another trust domain."
  §6 is therefore mandatory and is written below.

---

## 1. Summary

Today a DeluluLang grant tree lives inside **one broker, on one machine, serving one OS user.** This
RFC proposes **federation**: a subordinate broker on a vehicle that holds a `⊑`-attenuated subtree
delegated by a ground broker, enforces it locally at full speed while the link is down, and hands
back a verifiable record of what it did at re-contact.

The proposal's central choice: **the broker never opens a network socket.** Federation is mediated
by three signed, self-contained artifacts that the operator's *existing* link carries — a grant
certificate, a contact receipt, and an audit bundle. DeluluLang does not own the radio, does not
gain an async runtime, does not gain a network listener, and the local IPC transport's
same-user guarantee is left exactly as it is.

This closes `STAGE10_AUTONOMY_ADDENDUM.md` §2.5, which names federation as **"a prerequisite for any
real deployment in this addendum's domains."** It also subsumes build-order **D12e** (device-scoped
delegation), which is broken out as a phase that ships alone and first.

## 2. Motivation

### 2.1 The concrete thing that cannot be built today

The satellite profile (addendum §2.3) requires a spacecraft to hold pre-attenuated autonomy while
out of contact. Three facts in the current tree make that impossible, and each is verifiable by
reading the source rather than the docs:

1. **A lease token cannot cross to another broker.** `lease.rs` mints
   `dlt1_<payload>.<mac>` where `payload = {v, node, exp_millis, multi, nonce}` and
   `mac = blake3::keyed_hash(broker_key, payload)`. The token carries **no authority bytes** — only a
   *reference* to a node in the minting broker's `HashMap`. `redeem_inner` fail-closes on
   `self.inspect(&node).is_none()`. A remote broker receiving this token has nothing to look up and
   no key to verify with. Sharing the key is not delegation; it is making both parties able to mint
   as each other, which destroys the asymmetry that makes a grant tree mean anything.
2. **`Actuate` is synchronous-class** (`validate.rs`, `Op::class`): validated by a broker round-trip
   per command. That round-trip is what makes `delulu grants revoke` reach a running arm at all. No
   spacecraft can perform it against a ground broker between contacts.
3. **The audit chain is strictly linear.** `hash = blake3(prev_hash ‖ canonical_record)` over one
   monotone `seq` from one broker. Two brokers produce two chains that cannot be spliced without
   invalidating every hash after the splice point.

### 2.2 The narrower thing that also cannot be built today (D12e)

`delulu_broker::Scopes` has dimensions for `fs_read`, `fs_write`, `net`, `secrets`, `declassify`,
`foreign_c`, `foreign_python` — **and none for a device.** So the broker asks only "does this node
hold `Effect::Actuate`?" — a binary — and the physical envelope is enforced entirely runtime-side in
`device.rs`.

**The use-side plumbing is already complete, and this is the finding that most changes the cost of
fixing it.** Traced end to end:

```
interp.rs:1648   (ResourceKind::Actuator, "command") => (CustodyOp::Actuate, actuator_device(&c.scope))
                 actuator_device() → Some(env.device)          e.g. "sat0/hga"
interp.rs:931    custody.check(op, arg.as_deref())
broker_client.rs synchronous_check → ReqBody::Check { node, op: "Actuate", arg: Some("sat0/hga") }
                 …one IPC round-trip per command, to the daemon…
validate.rs:188  let ok = match op { FsRead|FsWrite|Net|Declassify|ForeignBind => …, _ => true };
```

**The device path is transmitted to the broker on every single actuator command, and `validate()`
returns `true` for it unconditionally** — `Op::Actuate` lands in the `_ => true` arm, whose comment
still reads *"Clock/Rand/Console/reserved: no scope argument to validate"* from when `Actuate` was a
reserved variant. Phase 10g activated it; the arm was not revisited.

Two things follow, and they must be stated together:

- **This is not a live fail-open.** Nothing is being bypassed, because there is no device dimension
  to check against and the envelope *is* genuinely enforced runtime-side. A sensor grant is still
  not a licence to command (`estop_cli.rs`), and the e-stop still reaches a running arm.
- **It is exactly the latent shape this project has been bitten by three times.** An argument sits
  in the position where every other op's scope is validated, is named `arg`, is passed to a function
  named `check` — and is silently accepted. The next person to add a device scope will reasonably
  assume the arg is checked. It is not.

Consequently F1 is **smaller and sharper than it looks**: no interpreter or IPC plumbing is
required. The grant side is what is missing — `cli.rs:3568` inserts the `Actuate` *effect* when
actuator grants exist and never carries the device paths into `AuthoritySpec`.

That is sound for the e-stop (revoke the node → every command denied) and it is what makes the
robotics and satellite demos work. But it means a delegating party can say *"you may actuate"* and
cannot say *"you may fly this corridor only."* The UAS two-grant lost-link pattern (addendum §2.2)
is inexpressible for that reason, and `run --lease` currently refuses a local device grant by name
rather than silently dropping it — a correct refusal standing in for a missing feature.

### 2.3 What is done instead today, and why it is not enough

Criterion 10's satellite demonstration runs **both broker roles inside one host over a simulated
link**, and its recording says so. That witnesses the grant *semantics* — expiry, attenuation,
re-delegation, and now (D20) deterministic replay — and it is honest. What it does not witness is
any of the four hard problems federation actually has: an offline credential, a second enforcement
domain, revocation across a partition, and two audit chains.

## 3. Irreducibility analysis

**Could this be a library?** *Partly — and the split is the design.*

- The **transport** must NOT be core. Radios, ground-station schedulers, store-and-forward queues,
  and CCSDS framing are somebody else's problem and change per mission. Making them core would
  entrench a networking model into a language runtime that today has neither a listener nor an
  async runtime.
- The **verifier** must be core. The load-bearing claim of this project is "no program can exceed
  the authority it was granted." If a certificate chain's `⊑` check can be supplied by a library,
  that claim is only as strong as whichever library got linked. `attenuation_check` is the
  mathematical heart of custody; a second, swappable implementation of it is a second place for the
  rule to die in an `else { continue }`.

So: certificate **format and verification** are core; certificate **carriage** is not. This RFC
proposes only the former, plus the minimum broker surface to hand artifacts in and out.

**Could this be a lint or a checker rule?** No. Nothing static can decide whether a credential
presented at runtime, minted by another machine hours ago, is within the authority its issuer held.

**Could this be a convention plus documentation?** No — that is exactly today's state. The
convention "run both roles in one host and describe the link" is what the demo does, and it cannot
witness partition behaviour, which is the entire risk.

**What does the language lose?** Nothing in the *language* — no syntax, no typing rule, no new
effect. `Effect::Actuate` is unchanged; `Cap[Actuator]` is unchanged. What grows is the **broker's**
surface and, unavoidably, its threat model (§6). A reader who never federates need not learn any of
it; a reader who does must learn one new noun (the certificate) and one new honest limit
(revocation degrades to expiry across a partition).

## 4. Design

### 4.0 The shape, in one paragraph

A **ground broker** mints a *grant certificate*: a signed, self-describing document carrying a full
`Authority`, an expiry, a subject key, and a hash-link to its issuing certificate. A **subordinate
broker** on the vehicle verifies the chain to a configured trust anchor, checks `⊑` at every hop
with the existing `attenuation_check`, and *adopts* the leaf as a local root. From that moment the
vehicle broker is a **real broker** — its own tree, its own audit chain, its own clock — and
`Actuate` round-trips to *it*, synchronously, locally, at full speed. The uplink lease is what
bounds how long that lasts without contact.

### 4.1 The three artifacts

| Artifact | Direction | Carried when | Purpose |
|---|---|---|---|
| **Grant certificate** | ground → vehicle | at contact | delegates a `⊑`-attenuated subtree offline |
| **Contact receipt** | ground → vehicle | each contact | renews the uplink lease; the *only* thing that extends vehicle authority |
| **Audit bundle** | vehicle → ground | at re-contact | the vehicle's own chain segment, self-verifying |

All three are byte-canonical (`canonical_json`, already in `audit.rs`: sorted keys at every depth,
absent optionals omitted) and signed. None requires a live connection at the moment of use.

### 4.2 The certificate, and why it is not a lease token

```text
dlcert1
alg: ed25519                      # or ed25519,ml-dsa-65 — the pqc.rs envelope, unchanged
issuer: <hex pubkey>
subject: <hex pubkey>
parent: <hex hash of the issuing certificate, or "anchor">
authority: <canonical JSON — the existing Authority::to_json() shape>
not_before: <epoch millis>
not_after:  <epoch millis>
nonce: <hex>
sig: <hex over the domain-separated canonical body>
```

Differences from `Token`, each deliberate:

- **It carries its authority.** A verifier with no shared state can read what is being granted.
- **It is asymmetrically signed**, so a verifier that can check it cannot mint it. `pqc.rs` already
  ships `sign_classical` (ed25519, the stable path, what v1.0 artifacts use) and `sign_hybrid`
  (ed25519 + ML-DSA-65). Cryptography is adopted, never hand-rolled; this RFC writes no lattice
  arithmetic and proposes no new primitive.
- **It chains.** `parent` is a hash-link, so the verifier reconstructs the delegation path and runs
  `attenuation_check(child, parent)` at every hop. **Federation introduces no new authority
  mathematics.** That is the strongest argument that this is tractable, and it is the property to
  protect through review.

**A new domain separator is mandatory.** `pqc.rs` signs artifacts under `ML_DSA_CTX =
b"delulu-artifact-v1"`. A grant certificate MUST be signed under a distinct context (proposed:
`b"delulu-grant-v1"`) or a signature over one becomes replayable as the other. This is a
security-critical one-line decision and it is called out here so it cannot be forgotten in
implementation.

**Post-quantum status is inherited honestly, not re-litigated.** `pqc.rs` refuses every
post-quantum operation — signing *and verifying* — without `--unstable` (DL1910), because the
adopted crates are unaudited by their own authors and official NIST ACVP vectors are not in hand.
Federation does not get an exemption. Phase F2 therefore ships on **ed25519**, with the hybrid
envelope as the crypto-agility path, and no material anywhere may describe a federated link as
quantum-safe or quantum-proof — those words appear in this project only in the sentences banning
them.

### 4.3 The device scope dimension (D12e) — ships first, ships alone

Add `device: BTreeSet<DeviceScope>` to `Scopes`, where a `DeviceScope` is a device path plus its
envelope (the ranges, `rate_hz`, `heartbeat_ms`, `ttl_ms`, `fail`) — the same content the
`--grant "actuator=arm0/elbow:angle_deg=-30..95,…"` string already parses runtime-side.

This needs a **third lattice**, alongside the two that exist:

| Existing | Subset relation | Meet |
|---|---|---|
| `path.rs` (fs) | descendant-or-equal | `intersect_path_sets` |
| name sets (net, secrets, foreign) | exact-string subset — no pattern implication, "conservative is sound" | set ∩ |
| **new: device** | same path *and* every envelope interval contained *and* `rate_hz ≤` parent | per-interval ∩ |

Mirroring `path.rs`'s three-function API (`is_descendant_or_equal` / `intersect_path_sets` /
`all_within`) keeps `attenuation_check` a flat conjunction with one more conjunct.

Two properties must be tested, not assumed:

- **The meet never widens.** `[0,40] ⊓ [10,50] = [10,40]`. An empty intersection yields an **empty
  envelope**, meaning "commands nothing" — the same fail-closed answer `name_intersect` already
  gives, and consistent with `attenuation_check`'s existing contract.
- **`ttl_ms >= heartbeat_ms`** is already a hard parse rule (`value.rs`). The meet must not be able
  to produce a pair that violates it; if it would, the meet is empty.

Start **exact-path only** for device names (no `sat0/*` globbing), following the existing
`numpy.* ⊉ numpy.linalg` precedent. Conservative is sound; widening later is additive, and
narrowing later is not.

**Definition of done for F1** — narrow, because §2.2 showed the use-side is already wired:

1. `Scopes` carries `device`; `attenuation_check` gains one conjunct; the meet gains one arm.
2. `cli.rs` carries device paths + envelopes into `AuthoritySpec` (today it inserts only the
   `Actuate` effect).
3. **`validate.rs`'s `_ => true` arm gains an explicit `Op::Actuate` arm**, and the stale
   "reserved" comment is corrected. This is the load-bearing line of the phase.
4. The **skip branch** (§4.9) is written first, not last: a device arg that cannot be matched
   against any granted scope is **refused**, never defaulted to `true`.
5. A regression test that fails against today's `_ => true` — i.e. one that would have caught this
   arm — so the fix is pinned by a witness rather than by a reviewer's memory.

D12e is independently valuable: it makes the UAS two-grant pattern expressible **locally**, with no
federation, no certificates, and no new threat model.

### 4.4 `Actuate` stays synchronous-class

This RFC does **not** propose making `Actuate` epoch-class. That would trade the e-stop's reach for
convenience, everywhere, including on machines that have a perfectly good local broker — precisely
the "quietly substitute something weaker" move that `--isolation microvm` refuses by name rather
than making.

What changes is *which* broker the round-trip reaches. On a federated vehicle the subordinate broker
is local, in-process-adjacent, and always up; the round-trip is the same round-trip. The link is
never in the command path.

### 4.5 Revocation across a partition — the honest core

**You cannot revoke during loss of signal.** There is no message to send and nobody to send it. Any
design that claims otherwise is lying about physics.

The only bound that survives a partition is **expiry**. Therefore:

- A federated subtree's **worst-case revocation latency is its uplink lease TTL**, not the epoch
  interval. That number is published per deployment, and it is never called "instant" — the same
  discipline addendum §1.3 already applies to revoke-to-fail-state latency.
- A **contact receipt is the only thing that extends** vehicle authority. Silence shrinks it. This
  makes the safe direction the default one: a lost link causes authority to *decay*, never persist.
- Revocation attempts are best-effort and **audited as attempts**, distinctly from confirmed
  revocations. A ground operator must never see "revoked" when the truth is "revocation queued for
  the next pass."

### 4.6 Two audit chains, cross-linked, never merged

Merging is impossible (§2.1.3) and faking it would be worse than not doing it. Instead:

- The vehicle keeps its **own** chain, self-verifying under the existing rules.
- At re-contact the vehicle ships an **audit bundle**; the ground writes one `reconcile` record into
  *its own* chain naming the bundle's head hash and seq range.
- The two chains are joined by hash reference — **exactly the mechanism the day-file boundary
  already uses**, where "the first record of each new file has `prev_hash` = the previous file's
  last record hash." The precedent is in-tree; this generalizes it across brokers instead of days.

One additive field is needed: an optional `origin` on `AuditRecord`. Because `body_value()` **omits
absent optionals entirely**, existing records hash identically with the field added — verified
against the canonicalization rule, not assumed.

Federated records are **not** globally ordered: `seq` is per-broker. Any tool that renders a merged
timeline must say so.

### 4.7 The Guard does not cross the link

`GuardState` is per-broker, daemon-memory, owner-code-gated, and guard enforcement applies to
delegated (non-root) nodes only. A federated subtree is by definition delegated — so a naive
implementation would put guarded classes on a vehicle where **no human owner can ever approve a
request.**

Proposed rule, deliberately the most conservative one available: **a grant certificate carrying any
guarded class is refused at mint time.** Guarded authority simply cannot be federated. No permit
crosses the link; no Guard semantics change; no second owner code exists.

This is a **mint-time rule, not a Guard change**, and that is intentional — the Guard is a defining
feature of this project and this RFC proposes no modification to it. If a future deployment needs
guarded authority on an autonomous vehicle, that is a separate RFC with its own owner decision, and
it should be a hard one to write.

### 4.8 Diagnostics

Broker codes DL1401–DL1414 are taken; federation claims **DL1415+**:

| Code | Condition |
|---|---|
| DL1415 | certificate chain does not verify to a configured trust anchor |
| DL1416 | a hop in the chain fails `⊑` (carries the intersection, like DL0802) |
| DL1417 | certificate outside its `not_before`/`not_after` window |
| DL1418 | **unknown scope dimension or unknown signature algorithm** — refuse the whole certificate |
| DL1419 | uplink lease expired; the subtree is no longer live |
| DL1420 | audit bundle fails hash verification (an incident, **not** a denial — see §4.9) |

Each needs an explain body and **both** an accepting and a rejecting conformance witness;
`--coverage` will not let them ship otherwise, and the floor rises accordingly.

### 4.9 The skip branch — what happens when the verifier cannot tell

This project has a `DL0803` fail-open, an arity gate that never fired, and a signature check that
returned success for an unsigned artifact on its record. Every branch below answers **refuse**.

1. **Unknown issuer key** → DL1415. Never "assume the anchor."
2. **Unknown signature algorithm** in the self-describing envelope → DL1418. Crypto-agility means
   the format *names* its algorithm so it can be replaced — it never means an unrecognized name is
   skipped. Verifying is a trust decision; refusing to verify must refuse the trust.
3. **Unknown scope dimension** → DL1418, refuse the **whole certificate**.

   > **This is the sharpest trap in the design and it deserves its own ruling.** `for-agents.md`
   > tells consumers to **ignore unknown fields** — that is what makes the JSON envelope's additive
   > promise usable, and it is correct *for the reporting surface*. Applying that habit to
   > **scopes** is a fail-open of exactly the DL0803 shape: an old verifier that ignores a scope
   > dimension it does not understand has silently *widened* the grant, because a dimension it
   > cannot see is a dimension it cannot enforce. Authority parsing is **not** additive-permissive.
   > The two rules must be stated together wherever either is stated, or one will be applied to the
   > other's domain.

4. **Clock unavailable, or stepped backwards** → a lease may **shrink** under clock uncertainty and
   may never be extended by it. A vehicle whose clock is wrong loses authority early, not late.
5. **Audit bundle fails verification** → DL1420 is raised and the incident is recorded, and it
   **does not deny anything**. The audit log is observability, not enforcement; making a failed
   reconciliation block operation would quietly convert the log into an enforcement input and
   violate a standing rule. Getting this backwards is the tempting mistake.

### 4.10 Phasing

| Phase | Content | Ships alone? | Risk |
|---|---|---|---|
| **F0** | Owner decisions (§8) — not code | — | — |
| **F1** | Device scope dimension + interval lattice (D12e) | **yes** | medium |
| **F2** | Grant certificate: format, sign, chain-verify, domain separator | no (needs F1 to be useful) | **high — security-critical** |
| **F3** | Subordinate broker: adopt a chain as a local root; trust-anchor config | no | **high — new enforcement domain** |
| **F4** | Uplink lease, contact receipt, published revocation latency | no | medium |
| **F5** | Audit bundle, `origin` field, `reconcile` record | no | medium |
| **F6** | Two-process demo over a link with a real outage + honesty review | no | low |

**F1 is the recommendation to do first regardless of whether F2–F6 are ever approved.** It closes a
named gap, unblocks a documented domain pattern, requires no threat-model change, and is the only
phase whose value does not depend on federation being finished.

F6 replaces the current "both roles in one host" caveat with two real processes, two state
directories, two audit chains, and a scripted outage — and D20's stepped clock is what lets that
demo replay byte-identically.

## 5. Drawbacks

- **The broker's threat model changes**, and that is not a hardening task but a new document.
  `broker_transport.rs` states today: *"Peer identity = OS-authenticated same-user… never
  multi-tenant auth."* That sentence stays true for the local transport and becomes **false if
  carelessly quoted about the federated path.** Every surface that states the same-user guarantee
  must be audited to scope it explicitly to local IPC.
- **A second enforcement domain is a second place for a rule to die.** D11e already notes that
  host-side and adapter-side envelope checks currently live in one process, so it is structural
  rehearsal rather than independent defense-in-depth. Federation makes that split real — which is a
  gain in defense and a loss in "one place to read the rule."
- **Key management appears, and it is the part nobody enjoys.** Trust anchors, key rotation across
  an intermittent link, and what happens when a vehicle's key is compromised are real operational
  burdens that did not exist when everything was one OS user.
- **The certificate is a second credential format** next to `Token`. Two formats is worse than one;
  the mitigation is that they do not overlap in purpose (local single-use binding vs. offline
  cross-domain delegation) and neither can be mistaken for the other (`dlt1_` vs `dlcert1`).
- **More surface to keep honest.** Six diagnostics, a lattice, a certificate format, and a demo all
  become things criterion-11 honesty review must re-read every release.

## 6. Entrenchment analysis

*(Required: this changes the broker's threat model, a Constitution §5.14 honesty limit.)*

**How hard is this to undo?** The phases differ sharply, and that is the argument for the phasing:

- **F1 is nearly free to reverse** in principle, but in practice a shipped `Scopes` dimension is
  entrenched by every grant string written against it. Reversing would break stored grants. Given
  it closes a named gap and adds no threat surface, that entrenchment is *chosen and acceptable*.
- **F2–F3 are deeply entrenching.** Once a certificate format is in the field, every deployed
  vehicle holds credentials in it, and format changes require a coordinated re-issue across an
  intermittent link — the worst possible upgrade environment. This argues for a **versioned magic
  (`dlcert1`) and an algorithm-naming envelope from the first byte**, both of which the design has,
  and for a long comment period rather than a fast one.

**What does it foreclose?** Adopting a certificate model forecloses a *capability-secrecy* design
(where possessing an unguessable handle **is** the authority, which is what `GrantId` and `Token`
are today). A signed bearer certificate is inspectable by anyone holding it; an opaque handle is
not. That is a genuine trade: inspectability is what makes offline verification possible at all, and
it is why the two formats must coexist rather than one replacing the other.

**Is that acceptable?** Yes, *if* federation is actually wanted — and that is precisely the F0
decision, not an engineering one. The effect system is deeply entrenched and that is the point; a
credential format would become similarly load-bearing, and it must be **chosen, not stumbled into**
by shipping F2 because F1 went well.

## 7. Rejected alternatives

- **Share the broker MAC key between machines.** The obvious minimal change, and the worst: two
  parties who can both verify a symmetric MAC can both *mint* it. The ground could no longer prove
  it issued a grant, and one captured vehicle key would mint ground authority. Rejected on the
  spot; recorded because it is the first thing anyone will propose.
- **Make `Actuate` epoch-class.** Would make federation trivial by making the e-stop weaker
  everywhere, including where nothing is federated. Rejected: this project refuses silent
  downgrades of a safety property by name.
- **A real network transport in the broker.** Rejected: it would add an async runtime and a network
  listener to a component whose entire security argument is "one OS user, one local pipe," and it
  would entrench a networking model into a language runtime. The artifact-mediated design gets the
  same result with none of it.
- **Merge the audit chains into one linear log.** Impossible without invalidating hashes, and a
  "merged" view that is really interleaved-and-relabelled would be a lie in the one artifact whose
  entire value is that it does not lie.
- **Let the vehicle mint its own sub-delegations freely.** Attractive for on-board multi-agent
  work; rejected for now because it multiplies the trust-anchor story before the single-hop case has
  ever run. Deferrable to a later RFC without redesign — the chain format already supports depth.
- **Do nothing until a hardware adapter exists.** Defensible, and genuinely arguable (§8). Rejected
  as the *default* only because F1 is useful with no hardware at all; F2+ may well wait.

## 8. Unresolved questions

1. **Should this be built before a hardware adapter exists?** No `Hw` adapter ships in-tree;
   `Profile::Hw` exists so the DL1905 hash gate has something real to gate. Federating simulators is
   provable but proves less. A defensible alternative order is F1 → adapter → F2+.
2. **What is the trust anchor, operationally?** A file of pinned public keys is the obvious first
   answer. Rotation across an intermittent link is not answered here.
3. **Does the vehicle's `run --lease` path need changes**, or does adopting a certificate as a local
   root make it identical to today's local case? The design intends the latter; it is not verified.
4. **How is a compromised vehicle key revoked** when the vehicle is the thing that is compromised
   and the link is intermittent? Lease expiry bounds it. Whether that is *sufficient* is a
   deployment question this RFC does not close.
5. **Multi-hop depth.** The format chains; nothing in this RFC exercises depth > 1.

## 9. Impact on the stability contract

- **Additive (minor)** for the language: no syntax, no typing rule, no new effect, no edition move.
  `Effect::Actuate` and `Cap[Actuator]` are untouched.
- **Additive** for the broker wire protocol: new `ReqBody` variants only, consistent with how the
  Guard verbs were added (`broker/1`, both ends one binary).
- **Additive with a caveat** for `Scopes`: adding a `device` dimension is additive in *type*, but
  §4.9.3 means an **older verifier must refuse** a certificate carrying a dimension it does not
  understand. That is a deliberate departure from "ignore unknown fields," scoped to authority
  parsing, and it must be written into `STABILITY.md` rather than left as folklore.
- **Deprecates nothing.** `Token` keeps its job.

---

## Disposition

*Open. No comment period has begun — this draft has no sponsor, and per `CONTRIBUTING.md` §4 an
AI-authored RFC cannot proceed without a named accountable human. Nothing in this document is
implemented, and nothing in it may be described as shipped.*
