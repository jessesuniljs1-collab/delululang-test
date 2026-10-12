# V2 P9 — authority at the boundary: the build order, read against the code

**Status:** P9 is designed in `V2_OPENSHELL_STUDY.md` §4.8–§4.12 (D-V2-55). This file is the step before building, the one
P8 had (`V2_P8_DESIGN.md`, "the build order, read against the code"): what each slice reads already exists or does not,
**measured**, and what the slice's real design question is. Written by routine run 19 (2026-10-12), after P8 closed
(D-V2-116). Nothing of OpenShell is copied here; its prover is the question this phase asks of a *program* instead of a
policy.

**P9-01 is built** (routine run 20, 2026-10-12, D-V2-118 — `crates/delulu/src/within.rs`, `crates/delulu/tests/within_cli.rs`), with
one departure from step 2 below: each request is decided by the runtime's own mint (`prim::call_root_method` against the root
`Grants::add` builds), not by the lattice's per-dimension functions — the broker's `Authority` is coarser than a grant set
(`console` is folded into `Write`; it has no `net.special`, sensor, compute or plugin dimension), so a verdict read through it
would answer *within* where `run` refuses. The agreement test holds the two to one answer where both speak (paths, hosts, secret
names). The corpus witness ran as step 4 says; a package directory and a `.dwx` are refused under `--within` until each has its
own requirement list. Building it found SECRET-HIDDEN-1 (a secret read under a computed name was missing from `--grants`).

The order is the study's: **P9-01** `authority --within` (first — every later slice reads its verdict), P9-02 `grants
diff` findings, P9-03 proposals with `⊑`-bounded approval, P9-04 secrets bound to endpoints, P9-05 method-and-path network
scopes.

## P9-01 — `delulu authority <file> --within BOUNDARY`

*The question OpenShell's prover asks of a policy, asked of a program* (study §4.8): BOUNDARY is a grant set in the `--grant`
grammar; the answer is one of four — **within** (exit 0), **exceeds** (1, with a counterexample: the first requested scope
the boundary does not cover and the source span of the call that requests it), **unsupported** (a dimension the program needs
that the grammar cannot bound, named), **inconclusive** — and `--json` carries the result, the counterexample and the coverage.

### What it reads — measured, 2026-10-12

1. **The program's side exists, without spans.** `delulu authority FILE` builds one report (`cli.rs`
   `source_authority_report` → `authority_report` + `stamp_grants`): the effect row, the capability kinds, and — from
   `ScopeWalk`, a read-only walk of the parsed module — every LITERAL scope a `root.<kind>(…)` call writes, per kind, plus the
   kinds with a site whose scope the source does not show (`hidden`, SCOPE-HIDDEN-1). `required_grants` turns that into the
   flags to type. **What it does not keep is where:** `ScopeWalk::record` stores the literal and drops the call's `Span`
   (`ast::Expr::Method { span, .. }` carries one), and `hide` stores the kind alone. A counterexample "naming the call" needs
   both kept — step 1 below.
2. **The boundary's side exists, and is the runtime's own.** `delulu_runtime::broker::Grants::add` parses the `--grant`
   grammar (`GRANT_FORMS` is the grammar as data, and a test holds it to the parser); `cli.rs`
   `authority_spec_from_grants` → `brokerd::spec_to_authority` turns a `Grants` into the broker's `Authority` — the same
   conversion `run --grant` uses to mint a root under the daemon (`run_cmd.rs`, the daemon-custody branch). So a boundary means exactly what the runtime would enforce;
   there is no second reading of the grammar to drift.
3. **The lattice exists, but answers yes or no.** `delulu_broker::attenuation_check(child, parent)` is the `⊑` the lease tree
   enforces (`MATHEMATICS.md` §1): effects `⊆`, `path::all_within` for the two filesystem dimensions, set inclusion for `net`,
   secrets, `declassify` and the two foreign dimensions, `device_scope::all_within` for devices, `budget_scope::within`. Its
   `Err` is the INTERSECTION, not the first item that fails — a verdict, not a counterexample. P9-01 asks the same
   per-dimension functions item by item, in source order, so the first uncovered item is the counterexample and the answer
   can never disagree with `⊑` itself (a test asserts the two agree over generated pairs, as `tier_for_subset` is held to the
   use-time oracle).
4. **The corpus, measured** (`scripts/authority-corpus.py` — `delulu authority F --json` over every program in `examples/`,
   `examples/guide/` and `docs/book/samples/`, 23; its control fails a read in which no program asks for anything): **6 pure, 12 with a fully literal authority, 5 with a placeholder.** The five split in two, and
   the split is the slice's design question:
   - **(a) grant data the program never requests** — `secret:API_KEY=VALUE` (`examples/demo.delulu`,
     `docs/book/samples/05_secrets.delulu`: the request is the NAME; VALUE is the operator's), `foreign.c=mathlib:PATH`
     (`08_foreign`: the request is the logical library; the binary is the operator's), `actuator=arm0/elbow:DIM=lo..hi`
     (`09_actuator`: the request is the device; the envelope is the operator's). Each is decidable: the boundary names the name
     or it does not.
   - **(b) a scope the source computes** — `actuator=DEVICE` (`10_actuate_errors`), and in general any hidden site
     (`fs.read=PATH`, `net=HOST`). Here the program's request is not static.

### The design question, and the answer proposed

**A hidden site (class b) is decidable in two of three cases.** If the boundary grants NOTHING of that kind, every value the
site could compute exceeds it: **exceeds**, the counterexample the site itself ("a `fs_read` whose path this call computes",
its span). If the boundary grants the kind without bound (the grammar's widest form for it, where one exists), every value is
within. Otherwise — the boundary grants some paths and the program computes one — no static answer exists: **inconclusive**,
naming the site. The study defined *inconclusive* as "a resource limit reached"; this widens it to "no static answer", and the
`--json` reason says which. `within` is never answered while a hidden site of a bounded kind remains — the verdict is a claim
about every run, and a computed path is not one the checker has read. (To be taken as a `D-V2-nn` when built.)

**Unsupported**, read from the code: each of the checker's thirteen resource kinds (`delulu-check` `ty.rs`
`ResourceKind`) has a grant form today — `fs.read`, `fs.write`, `console`, `net` (Http), `clock`, `rand`, `declassify`,
`plugin` (PluginHost), `foreign.python`, `foreign.c` (ForeignLoad), `actuator`, `sensor`, `compute` — and secrets have
`secret:`; and the conversion a boundary goes through is the one `run --grant` mints a daemon root with (`run_cmd.rs`). So `unsupported` is empty on the current language, and stays in the enum for a kind added later without a
form — with a test that maps every capability kind to a grant form, so a new kind cannot fall through to `within` unseen (the
skip branch, §11.2).

### The build order

1. **Spans in the walk.** `ScopeWalk` keeps `(kind, literal, Span)` and `(kind, Span)` for hidden sites; `requested_scopes`
   returns them; `required_grants` and the report are unchanged (the snapshot corpus and `core_invariance` say so — read the
   bless: lines added, none removed).
2. **The verdict, a pure function** — `within(report, sites, boundary: &Authority) -> Verdict { result, counterexample,
   coverage }`, every dimension decided by the lattice's own per-dimension function, hidden sites by the rule above; unit
   tests over generated programs-and-boundaries, and the agreement test with `attenuation_check`.
3. **The surface.** `--within GRANT` (repeatable, the `--grant` grammar) and `--within-file FILE` (one grant a line) on
   `delulu authority`; refused beside `--diff` and on a `.dwx` or a package directory until each is built (a flag a command
   documents reaches every verb of it — §11.5); exit 0/1/2 as above; `--json` carries `within: {result, counterexample,
   coverage}` in the authority envelope (additive, so `json_contract` and the schema gain a field, not a shape). Not a new
   subcommand — `authority` stays READ_ONLY in `mcp.rs`'s door rule, and an agent may ask it.
4. **Witnesses** — the corpus: each of the 12 literal programs `within` its own `--grants`, and `exceeds` each one-dimension
   narrowing (one grant dropped), the counterexample's span the call that requests it; the three class-(a) programs within a
   boundary naming the name; `10_actuate_errors` inconclusive with a partial device boundary and exceeds with none.
   **Mutants:** a dimension skipped (answers *within* wrongly), the counterexample taken from the wrong call's span, a hidden
   site answered *within*.

## P9-02 to P9-05 — to be read against the code before each is built

- **P9-02** `grants diff` findings (study §4.9) reads P9-01's verdict between two authorities. **Built** (routine run 20):
  between two grant FILES (D-V2-119), as the note below predicted — P9-01's mint for each NEW line, the device lattice for an
  actuator's envelope, a compute envelope's own terms; and between two lease NODES (D-V2-120) — the lattice, each item asked as a
  one-item authority carrying OLD's budget. `secret_reach_expansion` waits for P9-04. *The note, from P9-01's build:* between two GRANT SETS the lattice is too coarse for the same reason it was for
  P9-01 (no `console`, `net.special`, sensor, compute or plugin dimension), so the diff is most likely P9-01's verdict with NEW's
  lines as the requests and OLD as the boundary — each finding's category read from the dimension (absent in OLD: an effect
  expansion; present: a scope expansion; `net.special`: special-use reach; a device: device reach) — plus the device lattice
  (`device_scope::all_within`) for an envelope widened on a device both name, which the mint does not see; between two lease NODES,
  the broker's `Authority` is the domain and `attenuation_check`'s per-dimension functions decide, `budget` included. There is no
  `grants diff` today (`grants` has `list`, `tree`, `inspect`, `revoke`, `delegate`, `certify`, `adopt`, `receipt`, `renew`,
  `pubkey`), and `grants` is an EFFECTOR in `mcp.rs`'s door rule — a read-only verb under it is a door-rule question to answer first.
- **P9-03** proposals (§4.10): an agent proposes a grant; approval is `⊑`-bounded by the approver's own node. **Read against the
  code by routine run 20 (2026-10-12) — measured, before it is built:**
  1. **The study's premise is false for the commonest refusal.** §4.10 says a refusal "already carries a structured repair". A
     DL0703 does not: `run --json` gives `{"code": "DL0703", "message": "filesystem read of ./config was not granted — pass
     --grant fs.read=./config", "repairs": []}` — the grant in PROSE only — and `run --report-out` records no diagnostic at all
     for a refused run (`diagnostics: []`, `outcome: {exit: 1, ran: true}`; `examples/guide/05_capabilities.delulu` with
     `--grant console`).
  2. **What a proposal can be computed from without trusting the program:** the refusal's diagnostic carries the call's SPAN
     (`05_capabilities.delulu:48:18`), and P9-01's `requirements` already map a call's span to its requirement — the same
     `48:18` names `fs.read=./config`. So `grants propose` needs the refused call's span from the report, never a grant line the
     runtime or the program words: the narrowest grant is the static authority of the refused call site, as §4.10 asks. **Build
     step 1:** the report records the refusal (code and span), additively; **step 2:** `grants propose --from REPORT` maps that
     span through `requirements` to one grant line (a hidden site proposes nothing — there is no static narrowest grant for a
     computed scope, and it says so).
  3. **The store.** The Guard's queue (`GuardRequest {id, node, subset, why, created, status}`, approve and deny gated by the
     owner code, every record exact since D-V2-115) is the nearest shape — but a Guard request asks for PERMITS on a node's uses
     (`class:pattern`), and a proposal asks for a new child NODE. A proposal kind beside it, not a reuse of `GuardSubset`.
     Approval mints a child under the approver's node through `attenuate`, whose `⊑` check is the tree's own: "approval bounded
     by `⊑`" needs no new check.
  4. **Auto-approval** (`grants auto-approve --within BOUNDARY`) is P9-02 already: a proposal is auto-approvable exactly when
     `grants diff BOUNDARY PROPOSAL` adds nothing (D-V2-119).
  **Step 1 built** (routine run 20, `c2dd5f0`, D-V2-121): the ordinary run's report records the refusal with its span; a
  sandboxed run's report is next, then `grants propose`.
  5. **Hypotheses to witness when built:** a proposal is never wider than the refused call's requirement (mutants: propose the
     kind's placeholder, or the whole `--grants` list); the refused run's node is unchanged by an approval (re-run, never widened
     live — V2 §5); a rejection's reason reaches the agent in `--json`.
- **P9-04** secrets bound to endpoints (§4.11, RW 2.11): the language has no request a secret can ride on; the broker's
  `secret_map` (D-V2-115) is the nearest existing shape.
- **P9-05** method-and-path network scopes (§4.12, RW 2.12): `net` is host-scoped in the grammar, the egress client and the
  broker's set inclusion alike; three places to change in step.
