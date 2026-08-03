-------------------------------- MODULE Broker --------------------------------
(***************************************************************************)
(* A TLA+ model of the DeluluLang custody broker's grant tree.             *)
(*                                                                         *)
(* Every action's guard mirrors what `crates/delulu-broker/src/tree.rs`     *)
(* actually enforces, cited by file:line above the action. Where the code   *)
(* is ambiguous the model takes the WEAKER guard, so the model can never    *)
(* be kinder than the implementation.                                       *)
(*                                                                         *)
(* Authority is abstracted to a subset of a tiny effect set. This models    *)
(* the PROTOCOL, not the path algebra -- the ordering laws of `⊑` and `⊓`   *)
(* are checked separately and exhaustively by                               *)
(* `crates/delulu-broker/tests/order_laws.rs`.                              *)
(*                                                                         *)
(* INHERIT_EXPIRY selects which read the enforcement path uses:             *)
(*   TRUE  -- today's code: `effective_state_inherited` (tree.rs:576-590)   *)
(*   FALSE -- the PRE-RFC-0001-F4 behaviour: per-node expiry only           *)
(* Running with FALSE is how this model is shown to have teeth: TLC must    *)
(* rediscover the real historical bug (a `ttl: None` child outliving its    *)
(* parent's expired lease). A model that has never caught anything proves   *)
(* nothing.                                                                 *)
(***************************************************************************)
EXTENDS Naturals, FiniteSets

CONSTANTS
    Nodes,          \* finite set of grant ids
    Effects,        \* the abstract authority lattice: authority == SUBSET Effects
    MaxClock,       \* bound on the modelled clock
    NoParent,       \* distinguished "this is a root" marker
    NoTTL,          \* distinguished "never expires" marker  (Rust: ttl_millis: None)
    INHERIT_EXPIRY  \* BOOLEAN, see above

VARIABLES
    parent,   \* [Nodes -> Nodes \cup {NoParent}]
    auth,     \* [Nodes -> SUBSET Effects]
    ttl,      \* [Nodes -> Nat \cup {NoTTL}]   absolute deadline (Rust: epoch millis)
    state,    \* [Nodes -> {"Absent", "Live", "Revoked"}]
    clock,
    auditLen, \* length of the append-only audit chain
    epoch     \* revocation epoch, bumped on every revoke (tree.rs:94-96)

vars == <<parent, auth, ttl, state, clock, auditLen, epoch>>

Deadlines == 0 .. MaxClock

TypeOK ==
    /\ parent   \in [Nodes -> Nodes \cup {NoParent}]
    /\ auth     \in [Nodes -> SUBSET Effects]
    /\ ttl      \in [Nodes -> Deadlines \cup {NoTTL}]
    /\ state    \in [Nodes -> {"Absent", "Live", "Revoked"}]
    /\ clock    \in 0 .. MaxClock
    /\ auditLen \in Nat
    /\ epoch    \in Nat

(***************************************************************************)
(* Node predicates.                                                        *)
(*                                                                         *)
(* Rust: `effective_state(n, now)` folds revocation and TTL for ONE node.   *)
(***************************************************************************)
Expired(n) == ttl[n] # NoTTL /\ clock >= ttl[n]

SelfLive(n) == state[n] = "Live" /\ ~Expired(n)

(***************************************************************************)
(* tree.rs:576-590 `effective_state_inherited` -- a node is live only if    *)
(* EVERY node on the path to the root is live. Terminates because parents   *)
(* always point at an already-existing node and ids are never reused, so    *)
(* the tree is acyclic by construction (the Rust carries a 1024-hop bound   *)
(* as fail-closed insurance for the same reason).                           *)
(***************************************************************************)
\* IF-THEN-ELSE rather than /\ and \/ : TLC does not reliably short-circuit a
\* disjunct guarding a recursive call, and `parent[n] = NoParent` must stop the
\* walk BEFORE `SelfLive(NoParent)` is ever evaluated.
RECURSIVE InheritedLive(_)
InheritedLive(n) ==
    IF ~SelfLive(n)             THEN FALSE
    ELSE IF parent[n] = NoParent THEN TRUE
    ELSE InheritedLive(parent[n])

\* The read the enforcement path performs. Selecting SelfLive here is the
\* pre-F4 behaviour and is expected to break NoUsableOrphan.
Usable(n) == IF INHERIT_EXPIRY THEN InheritedLive(n) ELSE SelfLive(n)

(***************************************************************************)
(* tree.rs:604-613 `is_self_or_descendant` -- walks parent pointers upward. *)
(***************************************************************************)
RECURSIVE IsSelfOrDescendant(_, _)
IsSelfOrDescendant(n, anc) ==
    IF n = anc                   THEN TRUE
    ELSE IF parent[n] = NoParent THEN FALSE
    ELSE IsSelfOrDescendant(parent[n], anc)

Init ==
    /\ parent   = [n \in Nodes |-> NoParent]
    /\ auth     = [n \in Nodes |-> {}]
    /\ ttl      = [n \in Nodes |-> NoTTL]
    /\ state    = [n \in Nodes |-> "Absent"]
    /\ clock    = 0
    /\ auditLen = 0
    /\ epoch    = 0

(***************************************************************************)
(* GRANT -- the operator mints a root. Roots may carry a deadline: under    *)
(* federation the root IS the uplink lease (RFC 0001 F4), which is exactly  *)
(* the case where a subtree outliving its root matters.                     *)
(***************************************************************************)
Grant(n, a, t) ==
    /\ state[n] = "Absent"
    /\ parent'   = [parent   EXCEPT ![n] = NoParent]
    /\ auth'     = [auth     EXCEPT ![n] = a]
    /\ ttl'      = [ttl      EXCEPT ![n] = t]
    /\ state'    = [state    EXCEPT ![n] = "Live"]
    /\ auditLen' = auditLen + 1
    /\ UNCHANGED <<clock, epoch>>

(***************************************************************************)
(* DELEGATE / ATTENUATE -- tree.rs:420-475 `attenuate_core`.               *)
(*   tree.rs:428      parent must exist                                    *)
(*   tree.rs:434-448  parent's INHERITED state must be Live (fail closed:   *)
(*                    "no child may be born under a dead parent -- or under *)
(*                    a dead ANCESTOR")                                     *)
(*   tree.rs:450      the ⊑ check: authority must be within the parent's    *)
(*   tree.rs:468      NOTE: ttl is stored AS GIVEN. `attenuate` bounds a    *)
(*                    child's authority but NOT its deadline. This is       *)
(*                    faithful, and it is why the read-time walk exists.    *)
(***************************************************************************)
Delegate(p, c, a, t) ==
    /\ state[p] # "Absent"
    /\ state[c] = "Absent"
    /\ p # c
    /\ InheritedLive(p)
    /\ a \subseteq auth[p]
    /\ parent'   = [parent   EXCEPT ![c] = p]
    /\ auth'     = [auth     EXCEPT ![c] = a]
    /\ ttl'      = [ttl      EXCEPT ![c] = t]
    /\ state'    = [state    EXCEPT ![c] = "Live"]
    /\ auditLen' = auditLen + 1
    /\ UNCHANGED <<clock, epoch>>

(***************************************************************************)
(* REVOKE -- tree.rs:481-548.                                              *)
(*   tree.rs:604  caller may only reach itself or a descendant             *)
(*   tree.rs:616  transitive over the whole subtree, at WRITE time          *)
(*   tree.rs:94   bumps the revocation epoch                                *)
(***************************************************************************)
Revoke(caller, target) ==
    /\ state[caller] # "Absent"
    /\ state[target] # "Absent"
    /\ IsSelfOrDescendant(target, caller)
    /\ state' = [n \in Nodes |->
                   IF state[n] # "Absent" /\ IsSelfOrDescendant(n, target)
                   THEN "Revoked" ELSE state[n]]
    /\ epoch'    = epoch + 1
    /\ auditLen' = auditLen + 1
    /\ UNCHANGED <<parent, auth, ttl, clock>>

Tick ==
    /\ clock < MaxClock
    /\ clock' = clock + 1
    /\ UNCHANGED <<parent, auth, ttl, state, auditLen, epoch>>

Next ==
    \/ \E n \in Nodes, a \in SUBSET Effects, t \in Deadlines \cup {NoTTL} : Grant(n, a, t)
    \/ \E p, c \in Nodes, a \in SUBSET Effects, t \in Deadlines \cup {NoTTL} : Delegate(p, c, a, t)
    \/ \E cl, tg \in Nodes : Revoke(cl, tg)
    \/ Tick

Spec == Init /\ [][Next]_vars

(***************************************************************************)
(* State-space bound. `revoke` is idempotent but still consumes an audit    *)
(* seq and bumps the epoch (tree.rs:481-499), so both counters grow without *)
(* limit and TLC would never finish. Bounding them is a MODELLING           *)
(* restriction, not a claim about the implementation: it means the invariants*)
(* are checked over every reachable state within this bound, not over all   *)
(* histories.                                                               *)
(***************************************************************************)
StateConstraint == auditLen <= 6 /\ epoch <= 2

(***************************************************************************)
(* INVARIANTS                                                              *)
(***************************************************************************)

\* Every non-root node's authority is within its parent's. tree.rs:450.
AttenuationInv ==
    \A n \in Nodes :
        IF state[n] = "Absent" \/ parent[n] = NoParent
        THEN TRUE
        ELSE auth[n] \subseteq auth[parent[n]]

\* Revocation is transitive at write time. tree.rs:616.
RevokeCoversSubtree ==
    \A n \in Nodes : \A d \in Nodes :
        (state[n] = "Revoked" /\ state[d] # "Absent" /\ IsSelfOrDescendant(d, n))
            => state[d] = "Revoked"

\* A revoked node is never usable again, by any read.
NoResurrection ==
    \A n \in Nodes : state[n] = "Revoked" => ~Usable(n)

(***************************************************************************)
(* THE LOAD-BEARING ONE. A node the enforcement path treats as usable must  *)
(* not have a dead ancestor. With INHERIT_EXPIRY = FALSE this is the        *)
(* pre-RFC-0001-F4 code, and TLC is expected to produce a counterexample:   *)
(* a child with ttl = NoTTL under a parent whose deadline has passed.       *)
(***************************************************************************)
NoUsableOrphan ==
    \A n \in Nodes :
        IF ~Usable(n)                THEN TRUE
        ELSE IF parent[n] = NoParent THEN TRUE
        ELSE InheritedLive(parent[n])

\* The audit chain is append-only: its length never decreases. (Temporal.)
AuditAppendOnly == [][auditLen' >= auditLen]_vars

\* The revocation epoch is monotone. (Temporal.)
EpochMonotone == [][epoch' >= epoch]_vars

===============================================================================
