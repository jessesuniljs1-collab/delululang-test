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

*For a package directory too since routine run 21 (D-V2-124): the requirement list `authority DIR --grants` names, the call
named in its module's own file; a `.dwx` stays refused.*

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
  one-item authority carrying OLD's budget. `secret_reach_expansion` between grant files built with P9-04 (c) (D-V2-131); between nodes it waits for P9-04 (d). *The note, from P9-01's build:* between two GRANT SETS the lattice is too coarse for the same reason it was for
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
  sandboxed run's report is next, then `grants propose`. **Step 1b built** (routine run 21, `ebf393c`, D-V2-122): a sandboxed
  run's report records the HOST's refusal at the call the guest's request named, placed only inside the program the host sent;
  the same code, message and span as the ordinary run's report. **Next: `grants propose`** — measured while 1b was built: the
  requirement list keeps only each grant's FIRST call (`push` deduplicates by grant), so a refusal at a second call of the same
  grant maps to no requirement's `at`; the walk must keep every minting call (`ScopeWalk.sites`), and the report records no
  grant set, so a proposal is the refused call's own static requirement, not a difference from what the run held.
  **Step 2 built** (routine run 21, `93a74b7`, D-V2-123): `grants propose REPORT` — the report as a positional, not `--from` (a flag
  documented for `grants` reaches every verb of it). Hypothesis 5's first half witnessed: a proposal is never wider than the
  refused call's requirement (mutants M309, the whole requirement list, and M310, any literal of the kind, red). **Next: step
  3, the store and approval** (item 3 above); the rest of hypothesis 5 (the run's node unchanged by an approval; a rejection's
  reason reaching the agent) is witnessed there. **A package run's report is proposed for** since `9473789` (D-V2-124): the call
  found in the flattened text the run executed.
  **Step 3 built** (routine run 22, `45877b7`, D-V2-125): `grants propose REPORT --node g_ID` files the proposal with the broker
  as the request of the node the refused run held; `grants proposals` lists it with what an approval would mint; `grants approve
  pp_ID --parent g_ID` delegates, under the APPROVER's node, the node's authority joined with the ask — the Guard's mint gate and
  `⊑` bound it, so a proposer's own approval is DL0802 — and `grants reject pp_ID --reason` keeps the reason for the proposer.
  Hypothesis 5's rest witnessed: the proposer's node is unchanged by an approval (a new node, re-run), and a rejection's reason
  reaches the agent in `--json`. **Step 4 built** (the same run, `a7c409c`, D-V2-126): `grants auto-approve --parent g_B` — the
  standing boundary is a NODE, asked of what each approval MINTS (the join): every pending proposal whose mint is `⊑` B is
  approved under B by the broker's own check, each other one left pending with what `grants diff` finds it adds; never with
  the owner code, so guarded authority is approved by a person; revoking B revokes all it approved. **P9-03 complete.**
  5. **Hypotheses to witness when built:** a proposal is never wider than the refused call's requirement (mutants: propose the
     kind's placeholder, or the whole `--grants` list); the refused run's node is unchanged by an approval (re-run, never widened
     live — V2 §5); a rejection's reason reaches the agent in `--json`.
- **P9-04** secrets bound to endpoints (§4.11, RW 2.11). **Read against the code by routine run 22 (2026-10-13) — measured,
  before it is built:**
  1. **There is no request a secret can ride on.** `Cap[Http]` has one method, `get(url)` (`prim_table.rs`: `http.get`, arity
     1), answering `Result[Str, NetErr]`; the egress client's `PinnedRequest` carries a URL, host, port and pinned addresses —
     no header, no method. P9-04's language change begins as a REQUEST: a header-carrying form beside `get` (with P9-05 a
     method), new primitive rows, `PRIM_TABLE_VERSION` 5 → 6, conformance anchors positive and negative.
  2. **Where a secret can go today.** `Secret[_]` has `map` (an arbitrary pure function, run in the program's process — so under the daemon, where the bytes are
     the broker's, it is refused since routine run 24: MAP-HANDLE-1, D-V2-132; the broker's whitelisted `secret_map` exists and
     no program can reach it),
     `verify`, and `expose`, the only reader, which the checker admits only with `Cap[Declassify]` and which returns a plain
     `Str` that may then go anywhere — a URL's query included. So an API key reaches a host today only by being declassified,
     after which nothing follows it. P9-04 is the first path that SENDS a secret without declassifying it: the header
     position, its bytes resolved by the host at send time.
  3. **The grant grammar is `secret:NAME=VALUE | secret:NAME=env:VAR`** (`broker.rs`'s `GrantForm`), not the study's
     `secret=NAME@…`. An audience is a new form of that line — e.g. `secret:NAME@HOST[:PORT][/PATH]=VALUE` — whose parse must
     refuse every spelling that could read two ways (a name holding `@`, a value holding `@`), and whose host is compared as
     `net`'s is (`host_allowed`, the egress client's own strict parse, every redirect hop re-checked).
  4. **Custody.** Under the daemon a run's secrets are names only (invariant 23) and the bytes are the broker's (`expose`,
     `secret_map`), so the host's egress client would ask the broker at send time — an expose-class operation naming the
     destination, refused (the study's `credential_endpoint_mismatch`, a DeluluLang twin) when the destination is outside the
     audience; the broker's `secrets` dimension is names only today, so the audience joins the lattice there and in
     `grants diff` (`secret_reach_expansion`, waiting since D-V2-119).
  5. **The static half.** `authority` reporting, per secret, every destination a program can send it to needs the walk that
     maps mint literals (`ScopeWalk`, D-V2-123) to follow a secret into a request's header argument and the request's URL
     literal; a computed URL or a secret reached through a value is reported as such, never guessed.
  **Build order, proposed:** (a) the request with headers, no secret in it — the egress client carrying them; (b) a `Secret[Str]` in a header position, resolved by the
  host at send time, embedded custody first; (c) the audience in the grant line and the refusal at a destination outside it,
  each redirect hop re-checked; (d) the broker's audience dimension and `secret_reach_expansion`; (e) `authority`'s
  per-secret destinations. Each a slice, each witnessed red first.
  **Step (a) built** (routine run 23, `bee695f`, D-V2-128): `Cap[Http].get_with(url, headers: Map[Str, Str])` — the method in
  the NAME (so what a program does to a host stays readable from its calls, which P9-05 can decide statically), the URL
  argument one. The egress client judges the headers before anything is resolved (an RFC 9110 token per name; printable ASCII,
  space and tab per value, no whitespace at its ends; no two names differing only in case; bounds; none of `RESERVED_HEADERS`
  — `Host` above all, which would move the request to another site on the checked address) and sends them only to the
  URL's own ORIGIN, host and port: a redirect elsewhere drops them — step (c)'s audience rule applied first to everything a
  program says. Each hop records the header NAMES; a value is recorded nowhere. A `Map` crosses the sandbox channel now.
  **Next: steps (b) and (c) as ONE slice** — measured while (a) was built: today a secret leaves a program only through
  `expose`, which needs `Cap[Declassify]`; a `Secret[Str]` in a header sent to any allowlisted host would be a
  declassification WITHOUT it — a program holding a secret and `net` could send the secret's bytes to every host it may
  reach. So (b) never lands without (c)'s audience: the secret's grant names where it may go, and a request outside it is
  refused before a byte is sent. A `Secret[Str]` cannot sit in a `Map[Str, Str]`, so (b) is a method of its own (for example
  `get_with_secret(url, headers, name, secret)`), the host resolving the bytes at send time — embedded custody first; under
  the daemon the bytes are the broker's (item 4), so the host asks the broker at send time, naming the destination. A guest
  holds no secret today (`guest.rs`'s `CARRIED`), so (b) is L0 first.
  **Steps (b) and (c) built** (routine run 24, `91a1eb7`, D-V2-131): `Cap[Http].get_with_secret(url, headers, header,
  secret: Secret[Str])` — `Net`, no `Declassify`; the grant `secret:NAME@HOST[:PORT][,…]=VALUE`, each member parsed by the egress
  client's own URL parser (no path — P9-05's; no wildcard; none is the bottom: sent nowhere); the URL's origin judged against the
  audience before anything is resolved (reason `audience`), and again on every hop that would carry it, beside the origin rule
  (two walls — M383 survives on the first by design, M383b red); each hop records the secret's NAME, never its bytes; `Secret.map`
  and an actor boundary carry the audience; `grants diff` between grant files reports `secret_reach_expansion`. **Not yet:** a
  daemon or leased run (a handle — refused DL0904 with no custody op asked; the broker's audience dimension is (d)), a guest (no
  secret crosses the channel), `--engine wasm` (no `http`), `authority`'s per-secret destinations ((e)). **Next: (d)**.
  **Step (d), read against the code by routine run 24 (2026-10-14) — measured, before it is built:**
  1. **The broker knows names only.** `Scopes.secrets` is a `BTreeSet<String>` (`authority.rs`); `⊑` is `is_subset`, `⊓` is
     `name_intersect`; the wire's `AuthoritySpec.secrets` is a list of names, and `authority_spec_from_grants` (issue-then-run,
     `run --broker daemon --grant secret:…`) drops the audience a grant line names. A daemon or leased run's secret is a handle,
     refused by `get_with_secret` (DL0904, D-V2-131).
  2. **The proof already covers the shape the dimension wants.** `docs/design/models/authority_algebra.py` §1 proves the set
     laws (reflexive, transitive, antisymmetric, meet the GLB, idempotent, commutative, associative) for any exact-set ⊆
     dimension, and its full conjunction lists each set dimension by name. An audience is a set of `(name, origin)` pairs:
     spelled `NAME@host:port` (canonical — `parse_audience`'s spelling), it is one more exact-set dimension, **`secret_to`**,
     beside `secrets` — `⊑` is inclusion, `⊓` intersection, a name with no pair is the bottom (sent nowhere). It joins the
     conjunction in the model (one more name; the laws are uniform in the number) and `DELULU_Z3_MUTANT`'s list, and
     `authority.rs`'s `attenuation_check`/`intersect` beside `secrets`. A pair whose name the node's `secrets` does not hold
     is refused at the mint (it grants reach to nothing).
  3. **Where the bytes go.** Under the daemon the program process performs the request itself (the interpreter's egress
     client; the broker answers only the custody gate's `Net` op). So a send needs the bytes in that process at send time —
     a new broker op, `SendSecret { node, name, origin, span }`: the node live, its `secrets` covering the name's source (a
     `secret_map` derivative included), its `secret_to` holding `NAME@origin`; recorded (action `send`, target `NAME→origin`,
     the span), the lost-record rule of D-V2-109/111 applying (a lost record refuses, never answers the bytes); the bytes
     handed to the egress client only, after the client has judged everything else it can judge without them (headers,
     scheme, the URL's own origin). Invariant 23 is then stated as it holds: no secret byte enters the program process
     before an `expose` OR a recorded `send`, and a `send`'s bytes reach the egress client, never a program value. **The Guard:**
     `send` takes `Declassify`'s class and tier by default (guarded — DL1410 without a permit), because it moves a secret's
     bytes out of the broker; a looser tier is the operator's policy, never the default.
  4. **What else moves with it:** `grants_from_lease` fills `secret_audiences` from the node's `secret_to`; `grants diff`
     between two NODES reports `secret_reach_expansion` for a pair OLD lacks (`within.rs`'s `diff_nodes`, a dimension of its
     own, not `scope_expansion`); `grants tree`/`inspect` print the pairs (escaped — AUDIT-TEXT-1); the OCSF export names a
     `send` as HTTP Activity (4002) with the destination; a red-team pass on the new op (the guarantee: a secret's bytes reach
     only an origin its node's `secret_to` names, and appear in no record).
  5. **Witnesses to write first** (each red on the current binary): a leased run sends a secret to its node's audience (today
     DL0904); a child delegated a narrower `secret_to` cannot send to the parent's other origin; revoking the node stops the
     next send; the chain holds the `send` with its destination and no byte of the secret; a lost record refuses the send; the
     model's mutant for the new dimension is red.
  **Step (d)'s first half built** (routine run 25, `db88e98`, D-V2-133): `Scopes.secret_to` — items 2 and 4 above but the
  send: the dimension in `⊑`, the meet, the wire, the certificate, a pair living only beside its name (canonicalization drops
  an orphan; the model's new §2c), `grants delegate --secret-to`, the issue-then-run node keeping its grant line's audience,
  `grants_from_lease`, a proposal's join, `grants diff` between nodes. **Its second half built** (the same run, `6fe7076`,
  D-V2-134): item 3 — the broker's `send` (the node's `secret_to`, the Guard on the pair, a `send` record; a lost record hands
  over nothing), asked by the interpreter only after `egress::prejudge_secret` has judged everything it can without the bytes;
  a handle carrying its audience; the OCSF export names a `send` as HTTP Activity with its destination (`b8da460`). **Step (d)
  complete.** **Step (e) built** (the same run, `b6128d1`, D-V2-135): `authority --json`'s `secret_destinations` — per
  `get_with_secret` call, the secret's grant name and the URL literal's `host:port` as far as the source shows them, `null`
  where it does not. **P9-04 complete. Next: P9-05.**
  **Left open beside (d):** whether the language should reach the broker's whitelisted `secret_map` (`trim`, `upper`,
  `lower`) at all — today no program can (MAP-HANDLE-1, D-V2-132).
- **P9-05** method-and-path network scopes (§4.12, RW 2.12). **Read against the code by routine run 25 (2026-10-15) —
  measured, before it is built:**
  1. **Every request is a GET today.** `Cap[Http]` has `get`, `get_with` and `get_with_secret` — the method in the NAME since
     D-V2-128, so what a program does to a host is readable from its calls. A method dimension has nothing to restrict until a
     non-GET primitive exists; P9-05's method half lands WITH the first one (`post_with`, …), never before — a scope for a
     method no program can call is a gate that cannot fail.
  2. **A host is read in three places, and they disagreed.** The grant grammar (`net=HOST`, `*.SUFFIX` patterns —
     `broker.rs`'s `Grants::add`), the runtime (`prim::host_allowed` and the egress client's per-hop allowlist, both
     `prim::host_matches`, C85's dot boundary) and the broker (`validate.rs`'s `Op::Net` use check). The broker compared by
     EXACT STRING, so a pattern grant was refused under the daemon that embedded custody allowed — **NET-WILDCARD-1**, found
     reading this step and fixed beside it (`c69506f`/`016b522`: one matcher, `delulu_broker::validate::host_matches`). `⊑`
     between `net` sets stays exact-string (a pattern does not imply a host under it — conservative, sound).
  3. **Where a path would be judged.** The egress client parses every hop's URL (`parse_target`: the path is the text after the
     authority) — the per-hop check a path scope needs is there already. The broker never sees a path: the custody's `Net` op
     carries the HOST only (`custody_op_for`), and its records name the host. A path scope enforced by the broker needs the op's
     argument to carry the path (and the record to name it — the OCSF HTTP Activity then gains `http_request.url.path`). **Built 2026-10-21 (D-V2-169):** the use carries its path beside the host (`Custody::check_at`), the broker judges it, and
     the record names it in its payload as `url_path`; the OCSF export gains `http_request.url`.
  4. **The grammar.** `net=HOST` has no room for a path today, and a secret's audience REFUSES one (`parse_audience`: "a path in
     an audience waits for path scopes (P9-05)"). The study's shape — `HOST[:METHODS]/PREFIX/**`, `*` one segment, `**` only
     last, containment decided by the syntax — must refuse every spelling a URL parser would read another way (`%2F`, `..`,
     `//`, a trailing dot), as the URL parse already does for a request.
  5. **The static half** is P9-04 (e)'s walk again: a URL literal at each request call gives its path, a computed one is
     `null` — `secret_destinations`' shape, generalized to every `http` call.
  **Build order, proposed:** (a) path scopes in the grant grammar and the runtime — `net=HOST/PREFIX/**`, judged by the egress
  client on every hop, refused when a spelling could read two ways; (b) the broker: the `Net` op's argument and record carry the
  path, `Scopes.net` entries with a path, `⊑` as prefix containment — a new exact-set-plus-prefix order that joins the Z3 model
  with mutants; (c) `grants diff`/`authority`: path reach (`scope_expansion` on a wider prefix) and the static walk; (d)
  methods, with the first non-GET primitive. Each a slice, each witnessed red first.
  **Step (a) built 2026-10-17 (D-V2-138), on the laptop under the owner's commission of that day:** `net=HOST/PATTERN`,
  one reading (`delulu_broker::netscope`) at the grant, the first check, the broker-secret pre-check, every egress hop
  and the broker's use check (by host); an ambiguous request path refused where a pattern decides; the OpenShell export
  refuses a path scope. **Step (b) built the same day (D-V2-139):** `net` ordered by containment (host string, then
  pattern) in the broker's `⊑`, the runtime's mint and `grants diff`; the meet every entry's meet; the canonical form an
  antichain; Z3 §2d (41 obligations, three more CI mutants) and an exhaustive Rust check. Not taken: the path in the
  broker's op and record (OCSF reads `target` as the hostname) — its own slice. Next: (c)'s static half.
  **Step (d)'s first slice built 2026-10-23 (D-V2-182), on the laptop: the methods as an order, before any primitive
  uses one.** `net=HOST[:METHODS][/PATTERN]`, each set spelled one way (names from GET, HEAD, POST, PUT, PATCH, DELETE,
  each once, in that order; never `:GET` alone); an entry naming none allows GET alone, so no grant written before
  reaches more. The order and the meet in `netscope` (methods ⊆ and ∩), exact by enumeration with methods; Z3 §2e (9
  obligations, 50 in all, two more CI mutants). Every check asks with `GET` — the first check (DL0904 in words about
  the method), every egress hop, the broker's use check (`Denial::NetMethod`), `--within` — and the OpenShell export
  refuses a method scope. Next: the first non-GET primitive (its conformance anchors, the method in the broker's op and
  record, OCSF's `http_request.http_method`, a redirect's method — a 303 makes a GET), then the static half.
