---
name: delulu-federation-scope
description: "The load-bearing findings behind DeluluLang broker federation (RFC 0001) — read before touching the broker's credential, audit, or revocation design"
metadata: 
  node_type: memory
  type: project
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-08-08T09:49:22.551Z
---

**STATUS: SHIPPED 2026-07-22 as build-order D22 (RFC 0001 F2–F6).** Everything scoped below was
built and every predicted resolution held. Two things the scope did NOT anticipate, both found by
asking whether the mechanism could be defeated, both closed with witnesses **observed to fail**
against the old code:

1. **Certificate replay could undo a revocation.** Nothing stopped adopting the same certificate
   twice, so `grants revoke` — the only tool an operator has while the link is up — was defeated by
   re-presenting the credential. Fix: a chain may be adopted once per broker lifetime.
   **→ THAT FIX WAS INCOMPLETE. Finding P20-R4 (2026-08-08, commit a59ae34):** single-adoption keyed
   on the LEAF fingerprint only, so extending a revoked chain by one self-delegation (`[A]` →
   `[A,B]`, mintable by any holder of the leaf key — the vehicle itself) yielded a new leaf and a
   fresh un-revoked node, restoring revoked `{Actuate}` within one broker lifetime. Reproduced
   end-to-end. Fix: revoking an adopted node retires EVERY fingerprint in its chain
   (`revoked_adoption_fps` in tree.rs); `adopt` refuses any chain containing a retired fingerprint;
   every extension shares the anchored root, so retiring the root stops them all. The **TLA+ model
   could not catch this** — `Custody.tla` abstracts a credential as one opaque object, not a chain of
   arbitrary length, so it cannot state a chain-extension property (same lesson as F1/Z3/antichain).
   Pinned by `revoking_an_adopted_node_cannot_be_undone_by_extending_the_chain`, falsified.
2. **A delegated child could outlive its parent's expired uplink lease.** `attenuate_core` bounded a
   child's *authority* by `⊑` but never its *deadline*, and `effective_state` judged one node. The
   party the uplink lease bounds — the vehicle — is precisely the party that can mint children. Fix:
   `effective_state_inherited` walks to the root; a node is live only if its whole ancestry is.

Also learned: DL1419/DL1420 were penciled into the RFC and **not** added — the uplink lease *is* the
node TTL (so an expired uplink is the ordinary DL1402) and a bad bundle is the existing DL1405.
Fewer codes, no parallel liveness path.

Findings from reading the DeluluLang broker source on 2026-07-22 while scoping federation
(`rfcs/0001-broker-federation.md`, commit 3669f57). Each was read out of the code, not the docs —
several contradict what the design docs imply. Re-verify before relying on them.

**A lease token CANNOT cross brokers, and not incidentally.** `lease.rs` mints
`dlt1_<payload>.<mac>` where payload is only `{v, node, exp_millis, multi, nonce}` — **no authority
bytes**, just a reference into the minting broker's `HashMap` — MAC'd with `blake3::keyed_hash`
under the broker's own **symmetric** key. So a verifier must be able to mint. `redeem_inner`
fail-closes on unknown node, so today federation fails cleanly rather than dangerously. Sharing the
key between machines is the first thing anyone proposes and is the worst option: one captured
vehicle key mints ground authority.

**Federation needs NO new authority mathematics.** A certificate chain runs the existing
`attenuation_check` at every hop. This is the property to protect through review — if a design
introduces a second `⊑` implementation, push back.

**Audit chains cannot merge.** Linear `blake3(prev_hash ‖ canonical_record)` over a per-broker
`seq`. Cross-link by hash instead — the day-file boundary in `audit.rs` already does exactly this
across days, so generalize that, don't invent. `origin` is safely additive because
`AuditRecord::body_value()` omits absent optionals, so existing records hash identically.

**Revocation cannot cross a partition.** The only bound that survives LOS is expiry, so a federated
subtree's worst-case revocation latency IS its uplink lease TTL. Publish that number; never call it
instant.

**The Guard must not cross the link.** Conservative resolution that changes no Guard semantics: a
certificate carrying any guarded class is refused **at mint time**. Honors Jesse's standing "don't
tamper with delulu authority and guard" guardrail.

**The threat model changes categorically** — `broker_transport.rs` says "OS-authenticated
same-user… never multi-tenant auth", which stays true for local IPC and becomes false if quoted
about a federated link. Every surface stating it must be scoped to local IPC.

**Design choice that shrinks the surface enormously:** the broker never opens a network socket.
Federation is mediated by three signed self-contained artifacts (grant certificate, contact receipt,
audit bundle) that the operator's existing link carries. No async runtime, no listener, and it
matches how spacecraft ops actually work (files over a pass).

Related: [[rfc-process-deviation-d21]], [[delululang-project]].
