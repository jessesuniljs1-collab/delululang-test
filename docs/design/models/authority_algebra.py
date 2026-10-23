#!/usr/bin/env python3
"""The DeluluLang authority order, verified symbolically with Z3 across ALL eleven dimensions.

    python docs/design/models/authority_algebra.py          # needs `pip install z3-solver`

Campaign P17-8. Phase A proved reflexivity and transitivity over an abstract partial order and
checked the PATH dimension exhaustively (`crates/delulu-broker/tests/order_laws.rs`). This extends
the result to the whole conjunction `attenuation_check` actually computes:

    child ⊑ parent  ==  effects ⊆
                     ∧ fs_read, fs_write   path-descendant cover
                     ∧ net                entries ⊆ by host string, methods ⊆, then path-pattern containment (§2d, §2e)
                     ∧ secrets, secret_to, declassify, foreign_c, foreign_python   exact-set ⊆
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
  * net paths       — `netscope.rs` (P9-05 (b), D-V2-139): an entry `HOST/PATTERN`, its pattern's segments a literal,
                      `*` or a final `**`; `pattern_within` and `pattern_meet` modelled from the code and proved
                      SOUND against what the patterns match, the meet exact (§2d)
  * net methods     — `netscope.rs` (P9-05 (d), D-V2-182): an entry `HOST[:METHODS][/PATTERN]`, its methods a set
                      of six (none named: GET alone); `entry_within` and `entry_meet` proved SOUND against the
                      requests an entry takes, by method and path, the meet exact (§2e)
  * secret_to       — `authority.rs` (P9-04 (d), D-V2-133): each secret's audience, a set of
                      `NAME@host:port` pairs, ordered by inclusion like every exact-set dimension —
                      plus the one rule the others lack, a pair lives only beside its NAME in
                      `secrets` (`Scopes::canonicalized` drops an orphan), proved in §2c.

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
    "secret-to-dropped": "the secret audience is left out of the full conjunction",
    "secret-to-meet-union": "the secret audience's meet is the UNION (a widening meet)",
    "secret-to-keeps-orphans": "canonicalization keeps a pair whose secret the authority does not hold",
    "net-star-under-literal": "a `*` segment is called within a literal one (it matches other segments)",
    "net-rest-under-fixed": "a pattern ending `**` is called within one of the same fixed segments and no `**`",
    "net-meet-star-wins": "the meet of `*` and a literal is `*` (a widening meet)",
    "net-method-dropped": "an entry's methods are left out of its order (a POST entry within a GET one)",
    "net-method-meet-union": "the meet of two entries' methods is their UNION (a widening meet)",
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
print("(2c) SECRET AUDIENCES — authority.rs `secret_to` beside `secrets` (P9-04 (d), D-V2-133)")
print("     a pair (name i, origin k) is bit i of P[k]; K = 2 origins per name")
print("=" * 78)

K = 2

def sa(n):
    """An authority's (secrets, secret_to): N names, P[k] the names whose secret may go to origin k."""
    return {"N": BitVec(f"{n}_N", W), "P": [BitVec(f"{n}_P{k}", W) for k in range(K)]}

def savars(x):
    return [x["N"]] + x["P"]

def sa_wf(x):
    """Every pair beside its name: the names the pairs mention are within `secrets`."""
    held = x["P"][0]
    for k in range(1, K):
        held = held | x["P"][k]
    return sub(held, x["N"])

def sa_canon(x):
    """`Scopes::canonicalized`: a pair whose name `secrets` lacks is dropped (`well_formed_secret_to`)."""
    if MUTANT == "secret-to-keeps-orphans":
        return {"N": x["N"], "P": list(x["P"])}
    return {"N": x["N"], "P": [p & x["N"] for p in x["P"]]}

def sa_le(x, y):
    return And(sub(x["N"], y["N"]), *[sub(x["P"][k], y["P"][k]) for k in range(K)])

def sa_meet(x, y):
    pm = (lambda a, b: a | b) if MUTANT == "secret-to-meet-union" else (lambda a, b: a & b)
    return {"N": x["N"] & y["N"], "P": [pm(x["P"][k], y["P"][k]) for k in range(K)]}

def sa_eq(x, y):
    return And(x["N"] == y["N"], *[x["P"][k] == y["P"][k] for k in range(K)])

SX, SY = sa("SX"), sa("SY")
prove("canon is well-formed            wf(canon x)", sa_wf(sa_canon(SX)), savars(SX))
prove("canon never widens              canon x ⊑ x", sa_le(sa_canon(SX), SX), savars(SX))
prove("canon idempotent                canon(canon x) = canon x", sa_eq(sa_canon(sa_canon(SX)), sa_canon(SX)), savars(SX))
prove("canon keeps a well-formed x     wf x → canon x = x", Implies(sa_wf(SX), sa_eq(sa_canon(SX), SX)), savars(SX))
prove("canon monotone                  x ⊑ y → canon x ⊑ canon y",
      Implies(sa_le(SX, SY), sa_le(sa_canon(SX), sa_canon(SY))), savars(SX) + savars(SY))
prove("meet keeps well-formedness      wf x ∧ wf y → wf(x⊓y)",
      Implies(And(sa_wf(SX), sa_wf(SY)), sa_wf(sa_meet(SX, SY))), savars(SX) + savars(SY))
prove("meet ⊑ both                     x⊓y ⊑ x ∧ x⊓y ⊑ y",
      And(sa_le(sa_meet(SX, SY), SX), sa_le(sa_meet(SX, SY), SY)), savars(SX) + savars(SY))

print()
print("=" * 78)
print("(2d) NET PATHS — netscope.rs `pattern_within` / `pattern_meet` (P9-05 (b), D-V2-139)")
print("     a pattern: n fixed segments (a, b or *) and an optional final **; L = 3")
print("     a path: m segments (a, b, c, or an empty LAST one — a trailing /); M = 5")
print("=" * 78)

from z3 import Int  # noqa: E402 — the section's own sort, beside the bit-vectors above

L, M = 3, 5
LIT_A, LIT_B, STAR, EMPTY = 0, 1, 2, 3

def pat(n):
    return {"n": Int(f"{n}_n"), "s": [Int(f"{n}_s{i}") for i in range(L)], "r": Bool(f"{n}_r")}

def pvars(x):
    return [x["n"], x["r"]] + x["s"]

def pwf(x):
    """A pattern the grammar accepts: at most L fixed segments, each a literal or `*`, and never empty."""
    return And(x["n"] >= 0, x["n"] <= L, Or(x["n"] >= 1, x["r"]), *[And(v >= LIT_A, v <= STAR) for v in x["s"]])

def path(n):
    return {"m": Int(f"{n}_m"), "p": [Int(f"{n}_p{j}") for j in range(M)]}

def pathvars(y):
    return [y["m"]] + y["p"]

def pathwf(y):
    """A plain path: segments a, b or c (a literal no pattern names), an empty one only last (a trailing `/`)."""
    return And(y["m"] >= 0, y["m"] <= M, *[And(v >= LIT_A, v <= EMPTY) for v in y["p"]],
               *[Implies(j < y["m"] - 1, y["p"][j] != EMPTY) for j in range(M)])

def segmatch(sv, xv):
    """A literal its own segment; `*` any segment but the empty one (`path_matches`)."""
    return Or(And(sv == LIT_A, xv == LIT_A), And(sv == LIT_B, xv == LIT_B), And(sv == STAR, xv != EMPTY))

def pmatch(x, y):
    """`path_matches`: the fixed segments in order, then exactly nothing more — or anything more under `**`."""
    return And(If(x["r"], y["m"] >= x["n"], y["m"] == x["n"]), *[Implies(i < x["n"], segmatch(x["s"][i], y["p"][i])) for i in range(L)])

def pwithin(c, p):
    """`pattern_within`: at each of the parent's fixed segments the child has a fixed one, equal to a literal or under
    `*`; past them, a parent `**` takes everything, and without one the child ends there too, with no `**`."""
    def seg_ok(i):
        if MUTANT == "net-star-under-literal":
            return If(p["s"][i] == STAR, True, Or(c["s"][i] == p["s"][i], c["s"][i] == STAR))
        return If(p["s"][i] == STAR, True, c["s"][i] == p["s"][i])
    fixed = And(*[Implies(i < p["n"], And(i < c["n"], seg_ok(i))) for i in range(L)])
    tail = If(p["r"], True, c["n"] == p["n"] if MUTANT == "net-rest-under-fixed" else And(c["n"] == p["n"], Not(c["r"])))
    return And(fixed, tail)

def pmeet(a, b):
    """`pattern_meet`: segment by segment up to the shorter fixed part (two different literals meet in nothing; `*`
    takes the other side's segment); then the longer side's segments, if the shorter ends in `**`; `**` if both do."""
    k = If(a["n"] < b["n"], a["n"], b["n"])
    def seg(i):
        if MUTANT == "net-meet-star-wins":
            common = If(a["s"][i] == STAR, STAR, If(b["s"][i] == STAR, STAR, a["s"][i]))
        else:
            common = If(a["s"][i] == STAR, b["s"][i], a["s"][i])
        return If(i < k, common, If(a["n"] < b["n"], b["s"][i], a["s"][i]))
    segs_ok = And(*[Implies(i < k, Not(And(a["s"][i] != STAR, b["s"][i] != STAR, a["s"][i] != b["s"][i]))) for i in range(L)])
    ok = And(segs_ok, If(a["n"] == b["n"], True, If(a["n"] < b["n"], a["r"], b["r"])))
    m = {"n": If(a["n"] < b["n"], b["n"], a["n"]), "s": [seg(i) for i in range(L)],
         "r": If(a["n"] == b["n"], And(a["r"], b["r"]), If(a["n"] < b["n"], b["r"], a["r"]))}
    return ok, m

PA, PB, PC, PY = pat("PA"), pat("PB"), pat("PC"), path("PY")
POK, PM = pmeet(PA, PB)
prove("within reflexive                wf x → x within x", Implies(pwf(PA), pwithin(PA, PA)), pvars(PA))
prove("within transitive               a within b ∧ b within c → a within c",
      Implies(And(pwf(PA), pwf(PB), pwf(PC), pwithin(PA, PB), pwithin(PB, PC)), pwithin(PA, PC)),
      pvars(PA) + pvars(PB) + pvars(PC))
prove("within SOUND                    c within p ∧ c matches a path → p matches it",
      Implies(And(pwf(PA), pwf(PB), pathwf(PY), pwithin(PA, PB), pmatch(PA, PY)), pmatch(PB, PY)),
      pvars(PA) + pvars(PB) + pathvars(PY))
prove("meet exact (no wider)           a⊓b matches a path → a and b both match it",
      Implies(And(pwf(PA), pwf(PB), pathwf(PY), POK, pmatch(PM, PY)), And(pmatch(PA, PY), pmatch(PB, PY))),
      pvars(PA) + pvars(PB) + pathvars(PY))
prove("meet exact (no narrower)        a and b both match a path → a⊓b exists and matches it",
      Implies(And(pwf(PA), pwf(PB), pathwf(PY), pmatch(PA, PY), pmatch(PB, PY)), And(POK, pmatch(PM, PY))),
      pvars(PA) + pvars(PB) + pathvars(PY))
prove("meet within both                a⊓b within a ∧ a⊓b within b",
      Implies(And(pwf(PA), pwf(PB), POK), And(pwf(PM), pwithin(PM, PA), pwithin(PM, PB))), pvars(PA) + pvars(PB))
prove("meet is GLB                     c within a ∧ c within b → a⊓b exists ∧ c within a⊓b",
      Implies(And(pwf(PA), pwf(PB), pwf(PC), pwithin(PC, PA), pwithin(PC, PB)), And(POK, pwithin(PC, PM))),
      pvars(PA) + pvars(PB) + pvars(PC))

print()
print("=" * 78)
print("(2e) NET METHODS — netscope.rs `entry_within` / `entry_meet` with methods (P9-05 (d), D-V2-182)")
print("     an entry of one host: a set of the six methods (none named: GET alone), and no path or a pattern of §2d")
print("     a request: one of the six methods, or a seventh no entry names, and a path of §2d")
print("=" * 78)

NM = 6
GET_ONLY = 1

def ent(n):
    return {"m": BitVec(f"{n}_m", NM), "has": Bool(f"{n}_has"), "p": pat(f"{n}_p")}

def evars(x):
    return [x["m"], x["has"]] + pvars(x["p"])

def ewf(x):
    """An entry the grammar accepts: a non-empty method set; a pattern well formed where one is named."""
    return And(x["m"] != 0, Implies(x["has"], pwf(x["p"])))

def req(n):
    return {"k": Int(f"{n}_k"), "y": path(f"{n}_y")}

def rvars(r):
    return [r["k"]] + pathvars(r["y"])

def rwf(r):
    return And(r["k"] >= 0, r["k"] <= NM, pathwf(r["y"]))

def takes(x, r):
    """`judge` is Allow: the path under the pattern (or no pattern), and the request's method in the set — a seventh
    method (k = NM) is in none."""
    in_set = Or(*[And(r["k"] == i, has_bit(x["m"], i)) for i in range(NM)])
    return And(Or(Not(x["has"]), pmatch(x["p"], r["y"])), in_set)

def has_bit(v, i):
    return (v & BitVecVal(1 << i, NM)) != 0

def ewithin(c, p):
    """`entry_within` for one host: the child's methods a subset of the parent's; a parent with no path takes every
    path; a child with none is within only such a parent; two patterns by §2d's order."""
    methods = True if MUTANT == "net-method-dropped" else (c["m"] & ~p["m"]) == 0
    paths = If(Not(p["has"]), True, And(c["has"], pwithin(c["p"], p["p"])))
    return And(methods, paths)

def emeet(a, b):
    """`entry_meet`: the methods intersected (none in common: no meet), and the paths as §2d meets them."""
    pok, pm = pmeet(a["p"], b["p"])
    m = (a["m"] | b["m"]) if MUTANT == "net-method-meet-union" else (a["m"] & b["m"])
    ok = And(m != 0, Or(Not(a["has"]), Not(b["has"]), pok))
    mp = {"n": If(Not(a["has"]), b["p"]["n"], If(Not(b["has"]), a["p"]["n"], pm["n"])),
          "s": [If(Not(a["has"]), b["p"]["s"][i], If(Not(b["has"]), a["p"]["s"][i], pm["s"][i])) for i in range(L)],
          "r": If(Not(a["has"]), b["p"]["r"], If(Not(b["has"]), a["p"]["r"], pm["r"]))}
    return ok, {"m": m, "has": Or(a["has"], b["has"]), "p": mp}

EA, EB, EC, ER = ent("EA"), ent("EB"), ent("EC"), req("ER")
EOK, EM = emeet(EA, EB)
prove("entry within reflexive          wf x → x within x", Implies(ewf(EA), ewithin(EA, EA)), evars(EA))
prove("entry within transitive         a within b ∧ b within c → a within c",
      Implies(And(ewf(EA), ewf(EB), ewf(EC), ewithin(EA, EB), ewithin(EB, EC)), ewithin(EA, EC)),
      evars(EA) + evars(EB) + evars(EC))
prove("entry within SOUND              c within p ∧ c takes a request → p takes it",
      Implies(And(ewf(EA), ewf(EB), rwf(ER), ewithin(EA, EB), takes(EA, ER)), takes(EB, ER)),
      evars(EA) + evars(EB) + rvars(ER))
prove("entry meet exact (no wider)     a⊓b takes a request → a and b both take it",
      Implies(And(ewf(EA), ewf(EB), rwf(ER), EOK, takes(EM, ER)), And(takes(EA, ER), takes(EB, ER))),
      evars(EA) + evars(EB) + rvars(ER))
prove("entry meet exact (no narrower)  a and b both take a request → a⊓b exists and takes it",
      Implies(And(ewf(EA), ewf(EB), rwf(ER), takes(EA, ER), takes(EB, ER)), And(EOK, takes(EM, ER))),
      evars(EA) + evars(EB) + rvars(ER))
prove("entry meet within both          a⊓b within a ∧ a⊓b within b",
      Implies(And(ewf(EA), ewf(EB), EOK), And(ewf(EM), ewithin(EM, EA), ewithin(EM, EB))), evars(EA) + evars(EB))
prove("entry meet is GLB               c within a ∧ c within b → a⊓b exists ∧ c within a⊓b",
      Implies(And(ewf(EA), ewf(EB), ewf(EC), ewithin(EC, EA), ewithin(EC, EB)), And(EOK, ewithin(EC, EM))),
      evars(EA) + evars(EB) + evars(EC))
# The laws above hold of any product, so they are discharged with the methods left out of the order; these name the
# dimension's own refusals, as the budget's and the audience's obligations do in §3.
prove("an entry naming no methods takes GET alone   m = GET → no other method's request",
      Implies(And(ewf(EA), rwf(ER), EA["m"] == GET_ONLY, ER["k"] != 0), Not(takes(EA, ER))), evars(EA) + rvars(ER))
prove("a child taking a method its parent does not is never within it",
      Implies(And(ewf(EA), ewf(EB), (EA["m"] & ~EB["m"]) != 0), Not(ewithin(EA, EB))), evars(EA) + evars(EB))

print()
print("=" * 78)
print("(3) THE FULL CONJUNCTION — attenuation_check, authority.rs")
print("=" * 78)

# The product order: ⊑ is the AND of every dimension. The code's nine set-like dimensions (effects
# and eight scopes; the two path dimensions represented by the set laws; `secret_to` the ninth, P9-04
# (d)), the device dimension and the budget dimension: eleven, as `attenuation_check` computes them.
NSETS = 9
ST = 8   # the index of `secret_to` among the set dimensions

def au(n):
    return {"sets": [BitVec(f"{n}_s{i}", W) for i in range(NSETS)], "dev": dev(n + "d"), "bud": bud(n + "b")}

def avars(x):
    return x["sets"] + dvars(x["dev"]) + bvars(x["bud"])

def ale(p, q):
    budget = True if MUTANT == "budget-dropped" else bwithin(p["bud"], q["bud"])
    sets = [i for i in range(NSETS) if not (MUTANT == "secret-to-dropped" and i == ST)]
    return And(*[sub(p["sets"][i], q["sets"][i]) for i in sets], within(p["dev"], q["dev"]), budget)

def awf(x):
    return And(wf(x["dev"]), bwf(x["bud"]))

A, B, C = au("A"), au("B"), au("C")
Mset = [(A["sets"][i] | B["sets"][i]) if (MUTANT == "secret-to-meet-union" and i == ST) else (A["sets"][i] & B["sets"][i])
        for i in range(NSETS)]
MD, mddef = dmeet(A["dev"], B["dev"], "AB")
MB, mbdef = bmeet(A["bud"], B["bud"], "ABb")
AM = {"sets": Mset, "dev": MD, "bud": MB}

prove("FULL reflexive   a ⊑ a", Implies(awf(A), ale(A, A)), avars(A))
prove("FULL transitive  a⊑b ∧ b⊑c → a⊑c",
      Implies(And(awf(A), awf(B), awf(C), ale(A, B), ale(B, C)), ale(A, C)),
      avars(A) + avars(B) + avars(C))
prove("FULL meet ⊑ both — THE NO-WIDENING LAW, all eleven dimensions at once",
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
# The same teeth for `secret_to` (P9-04 (d)): measured as for the budget — a conjunction without the
# audience discharges the four laws above — so this states the dimension's own refusal at the level of
# the whole order: a pair the parent does not hold is never within it.
prove("FULL a secret_to pair the parent lacks is never ⊑ (the audience reaches the product)",
      Implies(Not(sub(A["sets"][ST], B["sets"][ST])), Not(ale(A, B))),
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
print("  * The NET dimension's entries carry path patterns (§2d): their order is proved SOUND and its meet")
print("    exact for patterns of up to %d fixed segments against paths of up to %d; that it calls nothing" % (L, M))
print("    within too little (completeness) is checked exhaustively in netscope.rs's tests. A SET of entries")
print("    (every child entry within a parent entry) is a preorder; the product below models net, like the")
print("    path dimensions, by the set laws.")
print("  * The PATH dimensions are represented here by the set laws only. Their real order is")
print("    path-descendant COVER, whose laws are checked exhaustively over real path strings in")
print("    crates/delulu-broker/tests/order_laws.rs — including the antisymmetry FAILURE (F1).")
print("  * Nothing here proves the Rust implements this model. That link is the citations above")
print("    each definition, maintained by hand.")
print("=" * 78)
