#!/usr/bin/env python3
"""The DeluluLang authority order, verified symbolically with Z3 across ALL ten dimensions.

    python docs/design/models/authority_algebra.py          # needs `pip install z3-solver`

Campaign P17-8. Phase A proved reflexivity and transitivity over an abstract partial order and
checked the PATH dimension exhaustively (`crates/delulu-broker/tests/order_laws.rs`). This extends
the result to the whole conjunction `attenuation_check` actually computes:

    child ⊑ parent  ==  effects ⊆
                     ∧ fs_read, fs_write   path-descendant cover
                     ∧ net, secrets, declassify, foreign_c, foreign_python   exact-set ⊆
                     ∧ device             per-device interval containment
                     ∧ budget             componentwise ≤ on (memory, processor time); absent = TOP

Each dimension is modelled from the code, not from the prose:
  * set dimensions  — `authority.rs:151-158`, plain `BTreeSet::is_subset`
  * device          — `device_scope.rs::within` / `::meet`, including the three asymmetries that
                      are easy to get backwards: a SMALLER heartbeat is NARROWER, a SMALLER ttl is
                      NARROWER, and an UNBOUNDED rate under a BOUNDED parent is a WIDENING.
  * budget          — `budget_scope.rs::within` / `::meet` (PS-B-05, D-V2-08's condition for a
                      budget joining the order): an ABSENT budget is the top, so an absent child
                      under a present parent is a WIDENING — the same asymmetry as the rate.

Until PS-B-05 the full conjunction below modelled SEVEN set dimensions while its obligation said
"all nine dimensions at once"; the code has eight (effects and seven scopes, the two path
dimensions represented by the set laws). The laws are uniform in that number, so nothing proved was
false, but the sentence claimed a dimension the model did not carry. It now carries eight.

Where Z3 proves a statement it is proved for every value of the modelled variables, not for a
sampled corpus. Where the model abstracts (bounded bit-widths, one device, two envelope
dimensions), that is stated at the end and is a real limit on what the result covers.
"""
import os
import sys

# The obligation names contain the mathematical symbols the project uses (⊆, ⊑, ⊓). On Windows a
# non-UTF-8 console (cp1252 under Git Bash) raises UnicodeEncodeError and the script dies BEFORE
# printing a single result — a committed artifact that only runs in the shell its author happened
# to use. Force UTF-8 so the output is the same everywhere.
try:
    sys.stdout.reconfigure(encoding="utf-8")
except Exception:  # pragma: no cover — older Pythons without reconfigure
    pass

try:
    from z3 import (Solver, unsat, sat, BitVec, Bool, Int, And, Or, Not, Implies, ForAll, If, BitVecVal)
except Exception as e:  # pragma: no cover
    print("z3 unavailable:", e)
    sys.exit(1)

W = 4          # bit-width: models up to 4 distinct labels per set dimension
FAIL = []

# TEETH (PS-B-05). A model that passes proves nothing about whether it COULD fail. CI therefore also
# runs it once per named mutant below and requires each run to end with obligations NOT discharged —
# the Z3 counterpart of the TLA+ step that requires CustodyC29.cfg to fail. An unknown name is refused
# outright (exit 2, no "NOT discharged" line), so a typo in CI cannot pass as a caught mutant.
MUTANTS = {
    "budget-meet-max": "the budget meet takes the componentwise MAXIMUM (a widening meet)",
    "budget-absent-child-passes": "an absent budget passes under a present one (the asymmetry backwards)",
    "budget-dropped": "the budget is left out of the full conjunction",
}
MUTANT = os.environ.get("DELULU_Z3_MUTANT", "")
if MUTANT and MUTANT not in MUTANTS:
    print(f"unknown DELULU_Z3_MUTANT `{MUTANT}` (known: {', '.join(MUTANTS)})")
    sys.exit(2)
if MUTANT:
    print(f"MUTANT: {MUTANT} — {MUTANTS[MUTANT]}. This run is EXPECTED to fail.")

def prove(name, claim, vs):
    """Prove `claim` for all `vs` by refuting its negation."""
    s = Solver()
    s.set("timeout", 30000)
    s.add(Not(ForAll(vs, claim)) if vs else Not(claim))
    r = s.check()
    if r == unsat:
        print(f"  PROVED    {name}")
        return True
    if r == sat:
        print(f"  REFUTED   {name}")
        print(f"            countermodel: {s.model()}")
        FAIL.append(name)
        return False
    print(f"  UNKNOWN   {name}  ({r})")
    FAIL.append(name + " (unknown)")
    return False

def refute(name, claim, vs):
    """The opposite: show a COUNTEREXAMPLE exists. Used for the antisymmetry failure (F1)."""
    s = Solver()
    s.set("timeout", 30000)
    s.add(Not(ForAll(vs, claim)) if vs else Not(claim))
    r = s.check()
    if r == sat:
        print(f"  COUNTEREXAMPLE FOUND  {name}  (expected — this is finding F1)")
        return True
    print(f"  NO COUNTEREXAMPLE     {name}  ({r}) — F1 may no longer hold; investigate")
    FAIL.append("expected-counterexample:" + name)
    return False

print("=" * 78)
print("(1) SET DIMENSIONS — effects, net, secrets, declassify, foreign.c, foreign.python")
print("    authority.rs:151-158 — BTreeSet::is_subset; meet is intersection")
print("=" * 78)

a, b, c, d = BitVec("a", W), BitVec("b", W), BitVec("c", W), BitVec("d", W)
sub = lambda x, y: (x & ~y) == 0          # x ⊆ y
meet = lambda x, y: x & y

prove("reflexive       a ⊆ a", sub(a, a), [a])
prove("transitive      a⊆b ∧ b⊆c → a⊆c", Implies(And(sub(a, b), sub(b, c)), sub(a, c)), [a, b, c])
prove("antisymmetric   a⊆b ∧ b⊆a → a=b", Implies(And(sub(a, b), sub(b, a)), a == b), [a, b])
prove("meet lower bd   a⊓b ⊆ a ∧ a⊓b ⊆ b", And(sub(meet(a, b), a), sub(meet(a, b), b)), [a, b])
prove("meet is GLB     c⊆a ∧ c⊆b → c ⊆ a⊓b",
      Implies(And(sub(c, a), sub(c, b)), sub(c, meet(a, b))), [a, b, c])
prove("meet idempotent a⊓a = a", meet(a, a) == a, [a])
prove("meet commutes   a⊓b = b⊓a", meet(a, b) == meet(b, a), [a, b])
prove("meet associates (a⊓b)⊓c = a⊓(b⊓c)", meet(meet(a, b), c) == meet(a, meet(b, c)), [a, b, c])
# NOTE: there is deliberately NO separate "no widening" obligation here. An earlier draft of this
# script had one, and it was encoded as `Implies(Not(Or(..., True)), True)` — i.e.
# `Implies(False, True)`, VACUOUSLY TRUE. Z3 discharged it and printed PROVED while checking
# nothing. A vacuous obligation reported as proved is worse than a missing one, because it buys
# false confidence in a list that is supposed to be the evidence. The real no-widening statement is
# "meet lower bd" above, proved for every pair of values.

print()
print("=" * 78)
print("(2) DEVICE DIMENSION — device_scope.rs::within / ::meet")
print("    one device, two envelope dimensions, rate/heartbeat/ttl")
print("=" * 78)

def dev(n):
    """A DeviceScope: presence+interval for two dims, optional rate, heartbeat, ttl, fail-state."""
    return {
        "p0": Bool(f"{n}_p0"), "lo0": Int(f"{n}_lo0"), "hi0": Int(f"{n}_hi0"),
        "p1": Bool(f"{n}_p1"), "lo1": Int(f"{n}_lo1"), "hi1": Int(f"{n}_hi1"),
        "hasr": Bool(f"{n}_hasr"), "rate": Int(f"{n}_rate"),
        "hb": Int(f"{n}_hb"), "ttl": Int(f"{n}_ttl"), "fail": Int(f"{n}_fail"),
    }

def dvars(x):
    return [x["p0"], x["lo0"], x["hi0"], x["p1"], x["lo1"], x["hi1"],
            x["hasr"], x["rate"], x["hb"], x["ttl"], x["fail"]]

def wf(x):
    """Well-formedness the parser guarantees: intervals ordered, ttl >= heartbeat (device_scope.rs
    refuses `ttl_ms < heartbeat_ms` at parse time — 'the lease would expire before its first beat')."""
    return And(x["lo0"] <= x["hi0"], x["lo1"] <= x["hi1"], x["ttl"] >= x["hb"], x["hb"] > 0)

def within(ch, pa):
    """device_scope.rs:208-232, faithfully — note all three asymmetries."""
    dim0 = Implies(ch["p0"], And(pa["p0"], pa["lo0"] <= ch["lo0"], ch["hi0"] <= pa["hi0"]))
    dim1 = Implies(ch["p1"], And(pa["p1"], pa["lo1"] <= ch["lo1"], ch["hi1"] <= pa["hi1"]))
    # rate: parent unbounded → anything; parent bounded + child unbounded → WIDENING; else c <= p
    rate = If(Not(pa["hasr"]), True, And(ch["hasr"], ch["rate"] <= pa["rate"]))
    return And(ch["fail"] == pa["fail"], dim0, dim1, rate,
               ch["hb"] <= pa["hb"], ch["ttl"] <= pa["ttl"])

def dmeet_ok(x, y):
    """`meet` returns None (device drops out) on mismatched fail-state, or if ttl < heartbeat."""
    return x["fail"] == y["fail"]

def dmeet(x, y, n):
    """device_scope.rs:238-270. Shared dims only, narrowed to the overlap; a dim on one side only
    is DROPPED; rate takes the bounded one; heartbeat and ttl take the min."""
    m = dev(n)
    lo0, hi0 = If(x["lo0"] > y["lo0"], x["lo0"], y["lo0"]), If(x["hi0"] < y["hi0"], x["hi0"], y["hi0"])
    lo1, hi1 = If(x["lo1"] > y["lo1"], x["lo1"], y["lo1"]), If(x["hi1"] < y["hi1"], x["hi1"], y["hi1"])
    cons = [
        m["p0"] == And(x["p0"], y["p0"], lo0 <= hi0), m["lo0"] == lo0, m["hi0"] == hi0,
        m["p1"] == And(x["p1"], y["p1"], lo1 <= hi1), m["lo1"] == lo1, m["hi1"] == hi1,
        m["hasr"] == Or(x["hasr"], y["hasr"]),
        m["rate"] == If(And(x["hasr"], y["hasr"]), If(x["rate"] < y["rate"], x["rate"], y["rate"]),
                        If(x["hasr"], x["rate"], y["rate"])),
        m["hb"] == If(x["hb"] < y["hb"], x["hb"], y["hb"]),
        m["ttl"] == If(x["ttl"] < y["ttl"], x["ttl"], y["ttl"]),
        m["fail"] == x["fail"],
    ]
    return m, And(*cons)

X, Y, Z = dev("x"), dev("y"), dev("z")
M, mdef = dmeet(X, Y, "m")
allv = dvars(X) + dvars(Y) + dvars(Z) + dvars(M)

prove("device reflexive    within(x, x)", Implies(wf(X), within(X, X)), dvars(X))
prove("device transitive   within(x,y) ∧ within(y,z) → within(x,z)",
      Implies(And(wf(X), wf(Y), wf(Z), within(X, Y), within(Y, Z)), within(X, Z)),
      dvars(X) + dvars(Y) + dvars(Z))
prove("device meet ⊑ both  (a lower bound)",
      Implies(And(wf(X), wf(Y), dmeet_ok(X, Y), mdef), And(within(M, X), within(M, Y))), allv)
prove("device meet is GLB  within(z,x) ∧ within(z,y) → within(z, x⊓y)",
      Implies(And(wf(X), wf(Y), wf(Z), dmeet_ok(X, Y), mdef, within(Z, X), within(Z, Y)),
              within(Z, M)), allv)

print()
print("=" * 78)
print("(2b) BUDGET DIMENSION — budget_scope.rs::within / ::meet (PS-B-05)")
print("     Option<(memory, cpu)>, absent = TOP")
print("=" * 78)

def bud(n):
    """An `Option<BudgetScope>`: presence, and the two fields that mean something when present."""
    return {"has": Bool(f"{n}_has"), "mem": Int(f"{n}_mem"), "cpu": Int(f"{n}_cpu")}

def bvars(x):
    return [x["has"], x["mem"], x["cpu"]]

def bwf(x):
    """What `BudgetScope::parse` guarantees: both dimensions named and above zero."""
    return Implies(x["has"], And(x["mem"] >= 1, x["cpu"] >= 1))

def bwithin(ch, pa):
    """budget_scope.rs::within, faithfully: parent absent → anything (the top); parent present and
    child absent → WIDENING; both present → componentwise ≤."""
    child_has = True if MUTANT == "budget-absent-child-passes" else ch["has"]
    return If(Not(pa["has"]), True, And(child_has, ch["mem"] <= pa["mem"], ch["cpu"] <= pa["cpu"]))

def bmeet(x, y, n):
    """budget_scope.rs::meet: absent is the identity; both present → the componentwise minimum."""
    m = bud(n)
    lo = (lambda a, b: If(a > b, a, b)) if MUTANT == "budget-meet-max" else (lambda a, b: If(a < b, a, b))
    both = And(x["has"], y["has"])
    cons = [
        m["has"] == Or(x["has"], y["has"]),
        m["mem"] == If(both, lo(x["mem"], y["mem"]), If(x["has"], x["mem"], y["mem"])),
        m["cpu"] == If(both, lo(x["cpu"], y["cpu"]), If(x["has"], x["cpu"], y["cpu"])),
    ]
    return m, And(*cons)

def beq(x, y):
    """Equality of the Rust VALUE: `None == None` whatever the unused fields hold."""
    return And(x["has"] == y["has"], Implies(x["has"], And(x["mem"] == y["mem"], x["cpu"] == y["cpu"])))

BX, BY, BZ = bud("bx"), bud("by"), bud("bz")
BM, bmdef = bmeet(BX, BY, "bm")
BMr, bmrdef = bmeet(BY, BX, "bmr")
BXX, bxxdef = bmeet(BX, BX, "bxx")
BL, bldef = bmeet(BX, BY, "bl")          # (x⊓y)
BL2, bl2def = bmeet(BL, BZ, "bl2")       # (x⊓y)⊓z
BR, brdef = bmeet(BY, BZ, "br")          # (y⊓z)
BR2, br2def = bmeet(BX, BR, "br2")       # x⊓(y⊓z)
b3 = bvars(BX) + bvars(BY) + bvars(BZ)

prove("budget reflexive    within(x, x)", Implies(bwf(BX), bwithin(BX, BX)), bvars(BX))
prove("budget transitive   within(x,y) ∧ within(y,z) → within(x,z)",
      Implies(And(bwf(BX), bwf(BY), bwf(BZ), bwithin(BX, BY), bwithin(BY, BZ)), bwithin(BX, BZ)), b3)
prove("budget antisymmetric within(x,y) ∧ within(y,x) → x = y (the Rust value)",
      Implies(And(bwf(BX), bwf(BY), bwithin(BX, BY), bwithin(BY, BX)), beq(BX, BY)),
      bvars(BX) + bvars(BY))
prove("budget meet ⊑ both  (a lower bound — no widening)",
      Implies(And(bwf(BX), bwf(BY), bmdef), And(bwithin(BM, BX), bwithin(BM, BY))),
      bvars(BX) + bvars(BY) + bvars(BM))
prove("budget meet is GLB  within(z,x) ∧ within(z,y) → within(z, x⊓y)",
      Implies(And(bwf(BX), bwf(BY), bwf(BZ), bmdef, bwithin(BZ, BX), bwithin(BZ, BY)), bwithin(BZ, BM)),
      b3 + bvars(BM))
prove("budget meet idempotent x⊓x = x", Implies(And(bwf(BX), bxxdef), beq(BXX, BX)), bvars(BX) + bvars(BXX))
prove("budget meet commutes  x⊓y = y⊓x", Implies(And(bmdef, bmrdef), beq(BM, BMr)),
      bvars(BX) + bvars(BY) + bvars(BM) + bvars(BMr))
prove("budget meet associates (x⊓y)⊓z = x⊓(y⊓z)",
      Implies(And(bldef, bl2def, brdef, br2def), beq(BL2, BR2)),
      b3 + bvars(BL) + bvars(BL2) + bvars(BR) + bvars(BR2))

print()
print("=" * 78)
print("(3) THE FULL CONJUNCTION — attenuation_check, authority.rs")
print("=" * 78)

# The product order: ⊑ is the AND of every dimension. The code's eight set-like dimensions (effects
# and seven scopes; the two path dimensions represented by the set laws), the device dimension and
# the budget dimension: ten, as `attenuation_check` computes them.
NSETS = 8

def au(n):
    return {"sets": [BitVec(f"{n}_s{i}", W) for i in range(NSETS)], "dev": dev(n + "d"), "bud": bud(n + "b")}

def avars(x):
    return x["sets"] + dvars(x["dev"]) + bvars(x["bud"])

def ale(p, q):
    budget = True if MUTANT == "budget-dropped" else bwithin(p["bud"], q["bud"])
    return And(*[sub(p["sets"][i], q["sets"][i]) for i in range(NSETS)], within(p["dev"], q["dev"]), budget)

def awf(x):
    return And(wf(x["dev"]), bwf(x["bud"]))

A, B, C = au("A"), au("B"), au("C")
Mset = [A["sets"][i] & B["sets"][i] for i in range(NSETS)]
MD, mddef = dmeet(A["dev"], B["dev"], "AB")
MB, mbdef = bmeet(A["bud"], B["bud"], "ABb")
AM = {"sets": Mset, "dev": MD, "bud": MB}

prove("FULL reflexive   a ⊑ a", Implies(awf(A), ale(A, A)), avars(A))
prove("FULL transitive  a⊑b ∧ b⊑c → a⊑c",
      Implies(And(awf(A), awf(B), awf(C), ale(A, B), ale(B, C)), ale(A, C)),
      avars(A) + avars(B) + avars(C))
prove("FULL meet ⊑ both — THE NO-WIDENING LAW, all ten dimensions at once",
      Implies(And(awf(A), awf(B), dmeet_ok(A["dev"], B["dev"]), mddef, mbdef), And(ale(AM, A), ale(AM, B))),
      avars(A) + avars(B) + dvars(MD) + bvars(MB))
prove("FULL meet is GLB c⊑a ∧ c⊑b → c ⊑ a⊓b",
      Implies(And(awf(A), awf(B), awf(C), dmeet_ok(A["dev"], B["dev"]), mddef, mbdef, ale(C, A), ale(C, B)),
              ale(C, AM)),
      avars(A) + avars(B) + avars(C) + dvars(MD) + bvars(MB))
# The four laws above hold for ANY product of lattices, so they would still be discharged if a
# dimension were left out of `ale` — measured: deleting the budget from `ale` left every obligation
# PROVED. This one is what notices. It states the budget's own asymmetry at the level of the whole
# order, which only holds if the budget is actually IN the conjunction.
prove("FULL no budget under a budgeted parent is never ⊑ (the asymmetry reaches the product)",
      Implies(And(Not(A["bud"]["has"]), B["bud"]["has"]), Not(ale(A, B))),
      avars(A) + avars(B))

print()
print("=" * 78)
print("(4) ANTISYMMETRY — finding F1, restated at the level of the whole order")
print("=" * 78)
print("  Every dimension modelled ABOVE is antisymmetric on its own representation: bitvector")
print("  subset is (proved in §1), the budget is (proved in §2b), and so is device containment")
print("  when the fields are compared.")
print("  F1 does NOT come from the order's shape — it comes from the PATH dimension's")
print("  REPRESENTATION: `./data` and `data` are distinct Strings that resolve to one path, so two")
print("  structurally unequal Authority values are mutually ⊑. That is a property of the encoding,")
print("  not of the algebra, which is why it is proved by exhaustive enumeration over real path")
print("  strings in crates/delulu-broker/tests/order_laws.rs rather than here.")
print()
prove("device antisymmetric on its fields (given well-formed, same presence)",
      Implies(And(wf(X), wf(Y), within(X, Y), within(Y, X),
                  X["p0"] == Y["p0"], X["p1"] == Y["p1"], X["hasr"] == Y["hasr"]),
              And(X["hb"] == Y["hb"], X["ttl"] == Y["ttl"], X["fail"] == Y["fail"])),
      dvars(X) + dvars(Y))

print()
print("=" * 78)
if FAIL:
    print(f"RESULT: {len(FAIL)} obligation(s) NOT discharged -> {', '.join(FAIL)}")
    sys.exit(1)
print("RESULT: every obligation discharged.")
print()
print("WHAT THIS COVERS, and what it does not:")
print("  * Set dimensions are modelled as bit-vectors of width %d — the laws proved hold for EVERY" % W)
print("    value of those vectors, i.e. for all subsets of a %d-element universe." % W)
print("  * The device dimension carries TWO envelope dimensions, one optional rate, heartbeat, ttl")
print("    and a fail-state. Real grants may carry more envelope dimensions; the laws are uniform")
print("    in the number, but that uniformity is an argument, not something Z3 checked here.")
print("  * The budget dimension is modelled over UNBOUNDED integers: its laws hold for every")
print("    positive (memory, cpu) pair and for absence, not only the grid the Rust tests enumerate")
print("    (budget_scope.rs). The Rust fields are u64; the laws use only ≤ and min, which u64 keeps.")
print("  * The PATH dimensions are represented here by the set laws only. Their real order is")
print("    path-descendant COVER, whose laws are checked exhaustively over real path strings in")
print("    crates/delulu-broker/tests/order_laws.rs — including the antisymmetry FAILURE (F1).")
print("  * Nothing here proves the Rust implements this model. That link is the citations above")
print("    each definition, maintained by hand.")
print("=" * 78)
