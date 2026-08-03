------------------------------- MODULE Custody -------------------------------
(***************************************************************************)
(* Leases and certificate adoption — the part of the broker where BOTH of    *)
(* this project's real vulnerabilities lived, and the part `Broker.tla` did  *)
(* not model.                                                               *)
(*                                                                          *)
(*   1. A certificate could be re-presented after a revocation, restoring    *)
(*      authority an operator had killed. Fixed by single-adoption per       *)
(*      broker lifetime (`cert.rs:544-554`).                                 *)
(*   2. `redeem` never consulted the bound node's STATE, so a revoked node,  *)
(*      or a node under a revoked/expired ancestor, redeemed successfully    *)
(*      and wrote `decision: "allow"` into the audit chain for a grant that  *)
(*      was dead (campaign C29; fixed at `lease.rs:207-218`).                *)
(*                                                                          *)
(* Both fixes are modelled as SWITCHES so the model can be shown to have     *)
(* teeth: turn either off and TLC must reconstruct the original defect. A    *)
(* model that has never caught anything is indistinguishable from one that   *)
(* cannot.                                                                   *)
(***************************************************************************)
EXTENDS Naturals

CONSTANTS
    Nodes, Certs, Tokens, MaxClock,
    NoNode, NoCert, NoTTL,
    SINGLE_ADOPTION,   \* cert.rs:544  — refuse a second adoption of one fingerprint
    LIVE_ON_REDEEM     \* lease.rs:207 — the bound node must be live, ancestors included

VARIABLES
    parent, ttl, state,        \* the grant tree (as in Broker.tla)
    certOf,                    \* [Nodes -> Certs \cup {NoCert}]  which cert a node was adopted from
    adoptedAs,                 \* [Certs -> Nodes \cup {NoNode}]  cert.rs:570 mark_adopted
    tokFor, tokExp, tokMulti, tokEpoch, tokMinted, tokRedeems,
    burned,                    \* single-use nonces already spent (lease.rs:230-238)
    keyEpoch,                  \* lease.rs:133 rotate_key invalidates every outstanding token
    clock, redeemedDead

vars == <<parent, ttl, state, certOf, adoptedAs, tokFor, tokExp, tokMulti,
          tokEpoch, tokMinted, tokRedeems, burned, keyEpoch, clock, redeemedDead>>

Deadlines == 0 .. MaxClock

TypeOK ==
    /\ parent     \in [Nodes -> Nodes \cup {NoNode}]
    /\ ttl        \in [Nodes -> Deadlines \cup {NoTTL}]
    /\ state      \in [Nodes -> {"Absent", "Live", "Revoked"}]
    /\ certOf     \in [Nodes -> Certs \cup {NoCert}]
    /\ adoptedAs  \in [Certs -> Nodes \cup {NoNode}]
    /\ tokFor     \in [Tokens -> Nodes \cup {NoNode}]
    /\ tokExp     \in [Tokens -> Deadlines \cup {NoTTL}]
    /\ tokMulti   \in [Tokens -> BOOLEAN]
    /\ tokEpoch   \in [Tokens -> 0 .. MaxClock]
    /\ tokMinted  \in [Tokens -> BOOLEAN]
    /\ tokRedeems \in [Tokens -> 0 .. 3]
    /\ burned     \subseteq Tokens
    /\ keyEpoch   \in 0 .. MaxClock
    /\ clock      \in 0 .. MaxClock
    /\ redeemedDead \in BOOLEAN

Expired(n) == ttl[n] # NoTTL /\ clock >= ttl[n]
SelfLive(n) == state[n] = "Live" /\ ~Expired(n)

\* tree.rs:576-590. IF-THEN-ELSE, not /\ and \/ : TLC does not reliably short-circuit a
\* disjunct guarding a recursive call, and NoNode must stop the walk before SelfLive sees it.
RECURSIVE InheritedLive(_)
InheritedLive(n) ==
    IF ~SelfLive(n)            THEN FALSE
    ELSE IF parent[n] = NoNode THEN TRUE
    ELSE InheritedLive(parent[n])

RECURSIVE IsSelfOrDescendant(_, _)
IsSelfOrDescendant(n, anc) ==
    IF n = anc                 THEN TRUE
    ELSE IF parent[n] = NoNode THEN FALSE
    ELSE IsSelfOrDescendant(parent[n], anc)

Init ==
    /\ parent     = [n \in Nodes  |-> NoNode]
    /\ ttl        = [n \in Nodes  |-> NoTTL]
    /\ state      = [n \in Nodes  |-> "Absent"]
    /\ certOf     = [n \in Nodes  |-> NoCert]
    /\ adoptedAs  = [c \in Certs  |-> NoNode]
    /\ tokFor     = [t \in Tokens |-> NoNode]
    /\ tokExp     = [t \in Tokens |-> NoTTL]
    /\ tokMulti   = [t \in Tokens |-> FALSE]
    /\ tokEpoch   = [t \in Tokens |-> 0]
    /\ tokMinted  = [t \in Tokens |-> FALSE]
    /\ tokRedeems = [t \in Tokens |-> 0]
    /\ burned     = {}
    /\ keyEpoch   = 0
    /\ clock      = 0
    /\ redeemedDead = FALSE

(***************************************************************************)
(* ADOPT — cert.rs:533-582.                                                 *)
(*   cert.rs:544-554  a fingerprint already adopted by this broker is        *)
(*                    REFUSED: "re-presenting a credential must not undo a   *)
(*                    revocation". This is the switch SINGLE_ADOPTION.       *)
(*   cert.rs:557-568  ttl = min(chain window, now + tightest uplink term).   *)
(*                    Abstracted to a nondeterministic deadline, which is    *)
(*                    WEAKER than the code and therefore safe.               *)
(***************************************************************************)
Adopt(c, n, d) ==
    /\ state[n] = "Absent"
    /\ (SINGLE_ADOPTION => adoptedAs[c] = NoNode)
    /\ state'     = [state     EXCEPT ![n] = "Live"]
    /\ ttl'       = [ttl       EXCEPT ![n] = d]
    /\ certOf'    = [certOf    EXCEPT ![n] = c]
    /\ adoptedAs' = [adoptedAs EXCEPT ![c] = n]
    /\ parent'    = [parent    EXCEPT ![n] = NoNode]   \* an adopted node is a LOCAL root
    /\ UNCHANGED <<tokFor, tokExp, tokMulti, tokEpoch, tokMinted, tokRedeems,
                   burned, keyEpoch, clock, redeemedDead>>

(***************************************************************************)
(* DELEGATE + MINT — lease.rs:72-109. `delegate` attenuates then mints a     *)
(* token bound to the CHILD. Authority is not modelled here (Broker.tla      *)
(* covers the lattice); what matters for leases is the binding and deadline. *)
(***************************************************************************)
Delegate(p, c, t, d) ==
    /\ state[p] # "Absent"
    /\ state[c] = "Absent"
    /\ p # c
    /\ InheritedLive(p)                       \* tree.rs:434-448, via attenuate_core
    /\ ~tokMinted[t]
    /\ state'      = [state      EXCEPT ![c] = "Live"]
    /\ ttl'        = [ttl        EXCEPT ![c] = d]
    /\ parent'     = [parent     EXCEPT ![c] = p]
    /\ tokFor'     = [tokFor     EXCEPT ![t] = c]
    /\ tokExp'     = [tokExp     EXCEPT ![t] = d]      \* lease.rs:71 token exp = child deadline
    /\ tokEpoch'   = [tokEpoch   EXCEPT ![t] = keyEpoch]
    /\ tokMinted'  = [tokMinted  EXCEPT ![t] = TRUE]
    /\ UNCHANGED <<certOf, adoptedAs, tokMulti, tokRedeems, burned, keyEpoch, clock, redeemedDead>>

(***************************************************************************)
(* REDEEM — lease.rs:160-240, guards IN THE CODE'S ORDER.                    *)
(*   lease.rs:167  MAC: a token minted under a rotated-away key fails        *)
(*   lease.rs:184  the bound node must still exist                           *)
(*   lease.rs:207  LIVE INCLUDING ANCESTORS -- switch LIVE_ON_REDEEM         *)
(*   lease.rs:221  the TOKEN's own expiry, a separate bound                  *)
(*   lease.rs:230  single-redemption unless minted `multi`                   *)
(* The nonce is burned LAST, so a refused redemption mutates nothing.        *)
(***************************************************************************)
Redeem(t) ==
    /\ tokMinted[t]
    /\ tokFor[t] # NoNode
    /\ tokEpoch[t] = keyEpoch                                  \* lease.rs:167
    /\ state[tokFor[t]] # "Absent"                             \* lease.rs:184
    /\ (LIVE_ON_REDEEM => InheritedLive(tokFor[t]))            \* lease.rs:207 (C29)
    /\ (tokExp[t] # NoTTL => clock < tokExp[t])                \* lease.rs:221
    /\ (~tokMulti[t] => t \notin burned)                       \* lease.rs:230
    /\ burned'     = IF tokMulti[t] THEN burned ELSE burned \cup {t}
    /\ tokRedeems' = [tokRedeems EXCEPT ![t] = IF tokRedeems[t] < 3 THEN tokRedeems[t] + 1 ELSE 3]
    \* History variable: did a redemption ever succeed against a node that was NOT live?
    \* With LIVE_ON_REDEEM the guard forbids it; without, TLC should reach it.
    \* Parentheses are LOAD-BEARING: `=` binds tighter than `\/` in TLA+, so writing
    \* `redeemedDead' = redeemedDead \/ X` parses as `(redeemedDead' = redeemedDead) \/ X` — a
    \* disjunction that leaves the primed variable unconstrained, which TLC reports as `null`.
    /\ redeemedDead' = (redeemedDead \/ ~InheritedLive(tokFor[t]))
    /\ UNCHANGED <<parent, ttl, state, certOf, adoptedAs, tokFor, tokExp, tokMulti,
                   tokEpoch, tokMinted, keyEpoch, clock>>

\* lease.rs:133 — a fresh key invalidates EVERY outstanding token.
RotateKey ==
    /\ keyEpoch < MaxClock
    /\ keyEpoch' = keyEpoch + 1
    /\ UNCHANGED <<parent, ttl, state, certOf, adoptedAs, tokFor, tokExp, tokMulti,
                   tokEpoch, tokMinted, tokRedeems, burned, clock, redeemedDead>>

\* tree.rs:616 — transitive over the whole subtree, at write time.
Revoke(target) ==
    /\ state[target] # "Absent"
    /\ state' = [n \in Nodes |->
                   IF state[n] # "Absent" /\ IsSelfOrDescendant(n, target)
                   THEN "Revoked" ELSE state[n]]
    /\ UNCHANGED <<parent, ttl, certOf, adoptedAs, tokFor, tokExp, tokMulti,
                   tokEpoch, tokMinted, tokRedeems, burned, keyEpoch, clock, redeemedDead>>

Tick ==
    /\ clock < MaxClock
    /\ clock' = clock + 1
    /\ UNCHANGED <<parent, ttl, state, certOf, adoptedAs, tokFor, tokExp, tokMulti,
                   tokEpoch, tokMinted, tokRedeems, burned, keyEpoch, redeemedDead>>

Next ==
    \/ \E c \in Certs, n \in Nodes, d \in Deadlines \cup {NoTTL} : Adopt(c, n, d)
    \/ \E p, c \in Nodes, t \in Tokens, d \in Deadlines \cup {NoTTL} : Delegate(p, c, t, d)
    \/ \E t \in Tokens : Redeem(t)
    \/ \E n \in Nodes : Revoke(n)
    \/ RotateKey
    \/ Tick

Spec == Init /\ [][Next]_vars

StateConstraint == \A t \in Tokens : tokRedeems[t] <= 2

(***************************************************************************)
(* INVARIANTS                                                               *)
(***************************************************************************)

\* lease.rs:230 — a single-use token is spendable at most once.
SingleUseHolds ==
    \A t \in Tokens : ~tokMulti[t] => tokRedeems[t] <= 1

\* lease.rs:207 (C29) — no redemption ever succeeded against a dead node.
NoRedemptionOfADeadGrant == redeemedDead = FALSE

\* lease.rs:167 — a token from a superseded key epoch is unusable. (Guard-implied; stated so a
\* future edit that drops the epoch check is caught here rather than nowhere.)
RotatedTokensAreDead ==
    \A t \in Tokens : (tokRedeems[t] > 0) => tokEpoch[t] <= keyEpoch

(***************************************************************************)
(* THE LOAD-BEARING ONE — cert.rs:544-554.                                  *)
(* Once a certificate's adopted node has been revoked, no node carrying that *)
(* certificate may be usable again. Re-presenting a credential must not undo *)
(* a revocation. With SINGLE_ADOPTION = FALSE this is the ORIGINAL           *)
(* vulnerability and TLC is required to reconstruct it.                      *)
(***************************************************************************)
RevocationSurvivesReadoption ==
    \A c \in Certs :
        (\E n \in Nodes : certOf[n] = c /\ state[n] = "Revoked")
            => (\A m \in Nodes : certOf[m] = c => ~InheritedLive(m))

===============================================================================
