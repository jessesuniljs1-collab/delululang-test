# DeluluLang — Stage 5 Implementation Specification

**Version:** 0.5 ("Custody"). **Status:** Committed — buildable directly from this document.
**Depends on:** Stage 4 complete (foreign workers isolate what Stage 4 opened; the grant flow
being separated is the one Stages 1–4 already use).
**Governing documents:** `CONSTITUTION.md` (§5.14, §5.16), `SOUNDNESS_AUDIT.md` (§A.3, R-7).

---

## 0. Scope and goal

**Goal:** move authority custody **out of the program's process**. Root capabilities live in a
broker the program cannot reach; grants become a persistent, auditable, revocable **grant tree**
obeying the holder model (§5.16): any holder — human, LLM orchestrating parallel agents, agent,
future AI — can delegate only attenuations of what it holds, and revocation cascades. This stage
also ships the isolation profiles: foreign-worker processes (redeeming Stage 4's honesty note)
and the Firecracker-class microVM profile for genuinely untrusted execution.

**In scope (must ship):**
- The **broker daemon** (`delulu broker …`), local IPC, OS-authenticated.
- The **grant tree**: issue, attenuate (`⊑`-checked), delegate (lease tokens), revoke
  (transitive), inspect.
- **Revocation semantics** with stated latency bounds (§4.4).
- The **append-only, hash-chained audit log** and `delulu audit …`.
- **Foreign workers**: C/Python execution in a separate sandboxed process.
- **MicroVM profile** (`--isolation microvm`, Linux-first) with default-deny egress.
- Broker-held secrets; `expose` synchronous through the broker.
- New diagnostics: DL14xx. (DL0802 — attenuation violation — was reserved in Stage 1 and fires
  for real from this stage.)

**Non-goals (later stages):** plugin loading (Stage 6 — but plugin grants will be *children* in
this stage's tree); multi-machine/remote brokers (post-1.0); key ceremony/HSM integration
(post-1.0, noted in §10); Windows/macOS microVMs (fallback matrix in §7.3).

**Language surface delta: none.** No new syntax, no new effects, no typing-rule changes. Stage 5
is entirely runtime/custody. (`Root`, caps, rows are untouched — that is the point: custody
changes must never require program changes.)

---

## 1. Invariants (carried + new)

All prior invariants hold. New:

23. **Keys leave the process (daemon mode).** In `--broker daemon` mode, root capability material
    (secret bytes, filesystem roots as granted, credential data) exists only in the broker
    process; the program holds opaque lease references. Compromising the program process yields
    at most *use* of currently-live leases until revocation — never the grants themselves, never
    siblings, never the tree. (Embedded mode remains for dev; it lacks this invariant and
    `delulu authority`/`run` label it `custody: embedded`.)
24. **The tree is the law.** Every grant is a node; every node's authority `⊑` its parent
    (audit §A.3, checked at creation — violation is DL0802); revoking a node revokes its whole
    subtree before the revocation call returns (tree-state-wise; enforcement latency per §4.4).
25. **No upward or lateral visibility.** The IPC surface offers no operation by which a lease
    holder can enumerate, inspect, or reach its parent's or siblings' grants (Constitution
    §5.16 law 3 — enforced by API shape, not by checks: the operations do not exist).
26. **Complete audit.** Every issue/attenuate/delegate/revoke/deny and every synchronous-class
    capability use (§4.4) produces exactly one audit record, hash-chained; a broken chain is
    detected by `delulu audit verify` (DL1405).
27. **Fail closed.** Broker unreachable ⇒ effectful ops fail (DL1401); they never fall back to
    embedded custody silently.

---

## 2. Architecture

```
 human ── delulu grants … (CLI) ──┐
                                   ▼
                          ┌── delulu broker ──┐        holds: root caps, secret store,
                          │   grant tree      │        grant tree, revocation epochs,
                          │   audit log       │        audit chain, broker key
                          └───┬──────────┬────┘
             lease token      │          │  lease token (⊑ parent)
                              ▼          ▼
                    program A (holder)   program B (delegated child)
                        │  foreign worker (C/Python, sandboxed process)
                        │  microVM guest (untrusted profile)
```

- **IPC:** Unix domain socket `$XDG_RUNTIME_DIR/delulu/broker.sock` (mode 0700); Windows named
  pipe `\\.\pipe\delulu-broker-<sid>` with owner-only DACL. Peer identity = OS peer credentials;
  the broker serves exactly one OS user (one broker per user; cross-user is out of scope, §10).
- **Wire protocol:** length-prefixed canonical CBOR, versioned (`broker/1`); mismatch DL1406.
- **Broker key:** random 256-bit key at first start (`~/.delulu/broker.key`, 0600), HMAC-signs
  lease tokens; rotation via `delulu broker rotate-key` (invalidates outstanding tokens — audit-
  logged, deliberate).

## 3. The grant tree

### 3.1 Node

```json
{ "id": "g_7f3a…", "parent": "g_1b02…",
  "holder": { "kind": "process|delegate", "desc": "orchestrator.delulu", "peer": "pid:4711" },
  "authority": { "effects": ["Read","Net"],
                 "scopes": { "fs.read": ["./data"], "net": ["api.example.com"],
                             "secrets": [], "declassify": [], "foreign.c": [], "foreign.python": [] } },
  "ttl": "2026-07-05T21:00:00Z" | null,
  "state": "live|revoked|expired",
  "created": "…", "audit_seq": 4182 }
```

The root node is created interactively (the Stage-1 grant prompt now runs against the broker) or
by `delulu grants issue` — both are **human actions at the top of the tree**; nothing programmatic
creates root nodes (Constitution §5.16 law 4).

### 3.2 Operations (IPC + CLI)

| Operation | Rule |
|---|---|
| `issue` (CLI only) | creates a root-level node; interactive confirmation unless `--yes` in a TTY-less session is *refused* (root issuance is never headless-silent; CI uses pre-issued delegation tokens instead) |
| `attenuate(parent_lease, authority)` | new child node iff `authority ⊑ parent` (DL0802 otherwise); returns lease |
| `delegate(parent_lease, authority, ttl)` | attenuate + mint a **portable lease token** (HMAC-signed, single-redemption by default) for handing to another process — this is how an orchestrating LLM gives each parallel agent its slice |
| `redeem(token)` | binds the token to the redeeming process; second redemption fails (DL1407) unless minted `--multi` |
| `revoke(lease, node_id)` | allowed on the caller's node or any descendant; transitive; idempotent |
| `check(lease, op, scope_args)` | the per-use validation (§4.4) |
| `expose(lease, secret_ref)` | synchronous declassification; always audit-logged with the calling span |

CLI: `delulu grants list|tree|inspect <id>|revoke <id>`,
`delulu grants delegate --effects Read --fs-read ./data --ttl 1h [--multi]` → prints token,
`delulu run app.delulu --lease <token>` (replaces `--grant` flags for delegated runs; `--grant`
flags in daemon mode are sugar for issue-then-run at the root).

### 3.3 The holder model, restated operationally

*Holder = the party a node was issued/delegated to.* The orchestration story this stage must make
true end-to-end: an LLM holding `g_orch` spawns five agents with five `delegate` calls; each
agent's node is `⊑ g_orch`; agent code — regardless of what it writes or executes — can acquire
nothing outside its node (invariant 25 + language rules); the LLM revokes one agent and only that
subtree dies; the human revokes `g_orch` and all six die. Identical mechanics if the orchestrator
is a human, a CI system, or a robot's supervisory computer. **No party kind is inspected anywhere
in the broker code path** — that absence is acceptance criterion 9.

## 4. Enforcement and revocation semantics

### 4.1 Two validation classes (normative table amendment)

Every primitive-table operation is classed:

- **Synchronous class** — validated by a broker round-trip per use: `Declassify` (expose),
  `FsWrite` ops, `Net` ops, foreign bind, plugin load (Stage 6), and later `Actuate` (Stage 10).
- **Epoch class** — validated locally against cached scope data + a **revocation epoch** the
  runtime refreshes at most every **50 ms** (config `--epoch-ms`, ceiling 250): `Read` ops,
  `Clock`, `Rand`, `Console`.

### 4.2 Stated latency bound (honesty, normative)

Revocation takes effect: synchronous class — **before the next use**; epoch class — within
**one epoch interval** (≤ 50 ms default). This bound appears in `delulu explain E-REVOKE` and in
the audit record of every revocation. No stronger claim is made anywhere.

### 4.3 Lease liveness

Every op (both classes) checks node state ≠ revoked/expired from the epoch snapshot; a dead lease
is DL1403 at the failing op, carrying the revocation's audit sequence number (so an agent's JSON
error says *why and when* its authority died — feedback quality for agents is a feature here).

### 4.4 Broker-held secrets

`root.secret(name)` now returns a handle to a **broker-resident** secret (env var, file, or
`delulu secrets set` store). Secret bytes enter the program process only on `expose` (synchronous
class, audited). Interpreter and WASM engines share this path (the Stage-3 host secret table
becomes a client-side cache of broker handles). `Secret.map` whitelist ops execute broker-side.

## 5. Foreign workers (redeeming Stage 4)

`--foreign-isolation process` (default in daemon mode) runs each granted foreign lib / the Python
interpreter in a **worker subprocess**: spawned with minimum OS privilege (no inherited handles;
`job object` with kill-on-close on Windows, `prctl(PR_SET_PDEATHSIG)` + seccomp basic profile on
Linux), speaking the Stage-4 marshalling protocol over a private pipe. Consequences (all tested):
worker crash ⇒ `ForeignErr::WorkerDied`, program continues; worker cannot reach the broker socket
(not inherited, path not disclosed); memory corruption in C code is confined to the worker.
`--foreign-isolation inproc` remains for dev (labeled in `delulu authority` output).

## 6. MicroVM profile

`delulu run --isolation microvm` (Linux x86_64/aarch64, KVM required): the program (WASM engine
mandatory) runs inside a Firecracker/cloud-hypervisor guest with: read-only rootfs containing the
runtime + artifact; virtio-fs mounts exactly matching granted `fs.*` scopes (read-only unless
write-granted); **default-deny egress** — a userspace proxy on the host is the only network path
and enforces the `net` allowlist; broker access via vsock to a **broker proxy** that forwards the
guest's single lease (invariant 25 preserved — the proxy holds one node, not the socket).
Non-Linux: `--isolation microvm` is DL1408 with the documented fallback (`--isolation process` —
worker-style OS sandboxing of the whole program; explicitly weaker, labeled in output). The
isolation matrix (none/process/microvm × capabilities) ships as a table in the docs, not prose.

## 7. Audit log

`~/.delulu/audit/YYYYMMDD.jsonl`; each record
`{ seq, ts, prev_hash, hash, actor_node, action, target, authority?, span?, decision }` with
`hash = blake3(prev_hash ‖ canonical_record)`. `delulu audit tail|query --node g_… --effect Net|
verify [--from N]`. Verification failure is DL1405 (`requires_human: true`, message states
possible tamper). Rotation daily; heads cross-linked. The log is **observability, not
enforcement** — stated in the file header line itself.

## 8. Diagnostics (fresh range DL14xx; DL0802 activates)

| Code | Meaning | Repair |
|---|---|---|
| DL0802 | requested authority ⊄ holder's grant (from Stage-1 registry, now live) | attenuate the request — exact repair computes the intersection, never widens |
| DL1401 | broker unreachable / protocol failure (fail closed) | start broker — exact command in message |
| DL1402 | lease expired (TTL) | re-delegate — `requires_human: true` |
| DL1403 | lease revoked (carries revoking audit seq) | none — `requires_human: true` |
| DL1405 | audit chain verification failure | none — `requires_human: true` |
| DL1406 | broker protocol version mismatch | upgrade — exact |
| DL1407 | delegation token invalid / already redeemed | re-mint — `requires_human: true` |
| DL1408 | isolation profile unavailable on this platform | fallback command shown; `requires_human: true` |

## 9. Acceptance criteria

1. Daemon mode: a running program's `FsWrite` succeeds; `delulu grants revoke` on its node from
   another terminal; the *next* write fails DL1403 with the revocation's audit seq.
2. Epoch class: after revocation, a tight `read_text` loop fails within 50 ms (measured in CI
   with tolerance ×3 for CI jitter).
3. Delegation: orchestrator holding `{Read, Net}/./data` delegates `{Read}/./data/sub`; child
   runs under the token; child requesting `{Net}` or `./data` at redemption → DL0802; the repair
   object contains the computed intersection.
4. Transitive revocation: revoke the orchestrator's node → both its children fail their next op;
   `delulu grants tree` shows the whole subtree `revoked`.
5. Token single-redemption: second `--lease` use → DL1407.
6. Secrets: `expose` round-trips through the broker, appears in the audit log with span; secret
   bytes absent from program-process memory before `expose` (Stage-3 scan test, now stronger:
   absent even from the host-process cap cache).
7. Foreign worker: a C function that segfaults kills only the worker → `ForeignErr::WorkerDied`;
   worker process cannot connect to the broker socket (test asserts ECONNREFUSED/absence).
8. MicroVM (Linux CI): `http.get` to a non-allowlisted host fails inside the guest although the
   host machine can reach it (egress-deny proven); granted path is readable, sibling path is not
   mountable-visible at all.
9. **Holder-neutrality audit:** grep-level and test-level assertion that no broker/IPC code path
   branches on holder kind; the same conformance delegation suite passes with the orchestrator
   role played by (a) the CLI, (b) a DeluluLang program, (c) a shell script standing in for an
   LLM harness.
10. `delulu audit verify` passes over the CI run's full log; a deliberately corrupted record
    fails with DL1405 at the right seq.
11. Embedded mode still passes the full prior conformance suite (no regression; custody label
    present in reports).

## 10. Honesty and threat-model caveats (carry into docs verbatim)

- The broker defends against **the program and its delegates**, not against the OS user: any
  process running as the same user with default OS permissions could read the broker key. The
  boundary is process compromise and code-behavior, per the threat model — not local-user
  malware, not root, not the kernel, not the hypervisor, not microarchitectural channels
  (Constitution §5.14 unchanged).
- Revocation bounds are those of §4.2 — "immediate" is never claimed.
- The audit log detects tampering after the fact; it does not prevent it.
- Foreign workers bound *blast radius*, not foreign behavior; the microVM profile is the strong
  container and it is Linux-first — the fallback matrix is honest about weaker platforms.
- TTLs bound *duration* of compromise, not its existence.

## 11. Implementation status (2026-07-11)

CHUNK 1 (phases 5a + 5b + 5c) — the transport-free custody core — **is implemented and green**
(271 → **305 workspace tests, +34**). It lives in a new workspace member, `crates/delulu-broker`,
which depends ONLY on `delulu-diag` (`Diagnostic`) and `delulu-check` (`Effect`) — no transport, no
`blake3`/`hmac` (those are chunk 2), no runtime/CLI wiring (chunk 3). The Stage-1 embedded broker
(`delulu-runtime::broker`) is untouched and remains the embedded/dev path; this crate is the single
source of truth for tree/lattice logic that chunk 3 will wire in. Language surface delta: **zero**
(no `delulu-syntax`/`delulu-check`/`delulu-runtime`/`delulu-wasm`/CLI edits — the only shared change
is registering the DL14xx codes in `delulu-diag`).

**Phase 5a — the `⊑` attenuation lattice — is implemented and green.** `Authority { effects:
BTreeSet<Effect>, scopes: Scopes }` with `Scopes` carrying seven `BTreeSet<String>` dimensions
(`fs_read`, `fs_write`, `net`, `secrets`, `declassify`, `foreign_c`, `foreign_python`) so
serialization is canonical (sorted, deduplicated). `attenuation_check(child, parent) -> Result<(),
Authority>` implements `⊑` per SOUNDNESS_AUDIT §A.3: effects `⊆`; fs paths descendant-or-equal; net
/ secrets / declassify / foreign_c / foreign_python exact-string name-set `⊆` (ruling 4 — no pattern
implication in v0.5; noted as a possible post-1.0 refinement). On failure it returns the **computed
intersection** (`child ⊓ parent`), which is `⊑` both operands and therefore **never widens** (spec
§8) — this is DL0802's exact repair. Path descendant semantics are **pure lexical** (ruling 2): no
filesystem access ever; `\`→`/` normalization, lexical `.`/`..` resolution (an escaping `..` is not
a descendant), component-wise case-sensitive prefix compare (Windows case-insensitivity caveat
documented in `src/path.rs`). Files: `src/authority.rs`, `src/path.rs`. Tests (~26): a table-driven
lattice suite covering every scope dimension incl. `./data` vs `./data/sub` (ok), `./data` vs
`./database` (the no-naive-string-prefix bug), `..` traversal escapes, and Windows `\` separators;
plus meet-is-symmetric and never-wider property checks.

**Phase 5b — the in-memory grant tree — is implemented and green.** `Broker { nodes: HashMap<GrantId,
Node>, epoch, audit_seq, ids, clock }` with `Node` per §3.1 (id, parent, holder, authority, ttl,
state, created, audit_seq). `GrantId` = `g_` + 32 hex chars from OS randomness (`getrandom`, ruling
3 — already in the workspace dep tree), behind an `IdSource` trait with a deterministic `SeqIdSource`
for tests; the TTL clock is behind a `ClockSource` trait with a hand-driven `ManualClock` for tests
(ruling 3). Ops: `issue` (root node), `attenuate` (child iff `⊑`, else a DL0802 `Denial` carrying the
intersection; refuses attenuation under a dead parent, fail-closed), `revoke(caller, target)`
(allowed only if target is the caller or a descendant — §3.2; transitive over the whole subtree;
idempotent; bumps the epoch and stamps `revoked_by_seq` on every affected node), `inspect`, `tree`.
`Holder { kind, desc, peer }` is **stored and displayed, never switched on** — zero `match`/`if` on
`holder.kind` on any decision path (criterion 9). The monotone `audit_seq` counter is consumed by
every issue/attenuate/revoke/deny so chunk 2's hash-chained log can emit records later without
renumbering (ruling 5). Files: `src/tree.rs`, `src/ids.rs`, `src/time.rs`, `src/diag.rs`. Tests: a
3-level tree → revoke the middle → leaf revoked / root live; attenuate-widening rejected with the
correct intersection; revoke idempotency; revoke of a non-descendant refused; the **criterion-9 unit
form** (identical op sequence run with `holder.kind` = "human"/"process"/"delegate" yields
byte-identical serialized custody outcomes, ids injected deterministically); and a **holder-neutrality
grep test** (`tests/holder_neutrality.rs`) that reads the crate's own `src/*.rs` and asserts every
`.kind` access is a comment or a `KIND_IS_DATA`-marked display line.

**Phase 5c — the two validation classes + revocation epochs — is implemented and green.** `Op`
classed per §4.1 — synchronous: `Declassify`, `FsWrite`, `Net`, `ForeignBind` (`PluginLoad`/`Actuate`
reserved variants, unused); epoch: `FsRead`, `Clock`, `Rand`, `Console`. `Broker::check(node, op,
arg) -> Decision` validates synchronous ops against **live** tree state (consuming one audit seq per
use — invariant 26); `Broker::snapshot()` freezes a cheap point-in-time `Snapshot` and
`Snapshot::check(...)` validates **epoch** ops against it (no seq). Liveness (§4.3): every op checks
state ≠ Revoked/Expired; a revoked lease is **DL1403** carrying the revoking audit seq (rich payload
on the `Denial` — "why and when your authority died"); TTL expiry (checked against the pluggable
clock) is **DL1402**; an out-of-scope argument is **DL0904**. File: `src/validate.rs`. Tests (no
sleeps — fake clock + manual snapshots): revoke → synchronous op fails on the very next call; epoch
op still passes against a stale snapshot but fails after a refresh; TTL expiry → DL1402 driven by the
fake clock; out-of-scope arg → DL0904; epoch-class uses consume no audit seq.

**Diagnostics.** The full DL14xx table from §8 is registered in `crates/delulu-diag/src/codes.rs`
(DL1401, DL1402, DL1403, DL1405, DL1406, DL1407, DL1408 — with **no DL1404**, deliberately). DL0802's
registry title was left as-is ("grant exceeds holder's grant (attenuation violation)") — it fits; the
"exact repair = intersection, never widens" semantics are realized at the construction site in
`src/diag.rs`, where the DL0802 `Diagnostic` carries a typed `Repair { authority_widening: false,
confidence: Exact }`.

**Deviations / decisions flagged for head-chef review:**
1. **Diagnostic payload (ruling 7).** `delulu_diag::Diagnostic` has no structured-payload field and
   Stage 4 did not add one (its DL13xx runtime refusals are a native `Fault { code, message, span }`
   with rich detail in the message and *no* structured field). Following that pattern exactly, the
   broker's structured payloads (the DL0802 intersection `Authority`, the DL1403 revoking seq, the
   DL1402 ttl/now) live on the broker-native `Denial` enum (the machine-consumable source of truth
   chunk 3 wires in), and `Denial::to_diagnostic()` renders a rich human message. The shared
   `Diagnostic` type was **not** extended. If chunk 3's daemon JSON contract wants these as
   first-class fields, that is a clean future extension.
2. **DL0904 for topology/scope denials.** Spec §8 defines no code for "revoke refused (non-descendant)"
   or "unknown lease" or "use outside the node's scope," and DL1404 is deliberately absent. These map
   to the existing runtime **DL0904** ("capability scope violation") — the caller's authority does not
   reach the target/arg, which is precisely a scope violation. Flagging in case the head chef wants a
   dedicated DL14xx code instead.
3. **Criterion-9 serialization excludes the holder.** `Broker::outcome_json()` (the projection the
   criterion-9 test compares) deliberately omits `holder`, because the test varies `holder.kind` and
   the *point* is that the authority outcome is independent of it; including the varied field would
   trivially differ and prove nothing. The full `tree()` render includes the holder for display.
4. **Timestamps are epoch-millis integers**, not ISO-8601 strings as the §3.1 example shows — to
   avoid pulling a datetime crate into a transport-free core. ISO rendering is a CLI/display concern
   (chunk 3). Node state serialization emits the *stored* state (live/revoked); TTL expiry is
   time-dependent and evaluated only in the check paths, keeping serialization clock-free/canonical.

CHUNK 2 (phases 5d + 5e) — the hash-chained audit log and MAC-signed lease tokens — still
transport-free (no sockets, no daemon — chunk 3). One new dependency total: `blake3` (same `"1"`
requirement as `delulu-check`/`delulu-wasm`, unified at the workspace's 1.8.x), used for BOTH the
chain hash and the token MAC (`keyed_hash`) — no `hmac`/`sha2` crates. All file paths stay INJECTED
into the library; only `cli.rs` resolves `~/.delulu/…`. Chunk-1 API is backward-compatible:
`Broker::new()`/`with_sources` unchanged; the sink and key are additive builder methods.

**Phase 5d — the append-only hash-chained audit log — is implemented and green (2026-07-11,
305 → 323 workspace tests, +18).** `src/audit.rs`. Record shape per §7: `{ seq, ts, prev_hash,
hash, actor_node?, action, target?, authority?, span?, decision }` with
`hash = blake3(prev_hash ‖ canonical_record)`. Canonicalization is defined precisely in the module
docs: `canonical_record` is the record's JSON with the `hash` field EXCLUDED, object keys sorted
lexicographically at every depth (own `canonical_json`, independent of any serde_json
`preserve_order` feature), absent optionals omitted entirely; `prev_hash` enters the hash as the
ASCII bytes of its 64-char lowercase-hex string; the first record's `prev_hash` is 64 hex zeros
(`GENESIS_HASH`). `ts` is epoch millis via the injected `ClockSource` (chunk-1 convention; ISO
rendering is the display-only `render_ts_utc`). Storage: one `YYYYMMDD.jsonl` per **UTC** day under
an injected base dir; the FIRST line of every file is a header (no `seq` key, not part of the
chain) stating **"observability, not enforcement"** verbatim (§7); a new day's first record has
`prev_hash` = the previous day's last hash (heads cross-linked); reopening recovers the head from
disk. Wiring: an `AuditSink` trait; `Broker` gains an optional sink (`with_sink`/`set_sink`;
default none — chunk-1 behavior and tests unchanged). With a sink attached, EXACTLY ONE record per
issue/attenuate/delegate/revoke/deny and per synchronous-class use — reusing the seq the operation
already consumed (chunk-1 ruling 5 pays off: records number 1,2,3,… with no renumbering); the
synchronous-use action is `"use"` with the scope argument as `target`; epoch-class uses emit
nothing (invariant 26). Two sinks ship: `MemSink` (in-memory, for tests) and the file-backed
`AuditLog`. Read/verify surface: `verify(dir)` walks day files in order recomputing every hash and
every cross-record/cross-file `prev_hash` link — any mismatch is **DL1405 at the failing seq**
(new `Denial::AuditChainBroken { seq, detail }`, `requires_human: true` per §8), and verification
stops at the FIRST break; `tail(dir, n)`; `query(dir, filter)` with simple filters (node id —
matches actor or target — action, effect-in-authority). CLI: `delulu audit tail [N] | query
[--node g_ID] [--action A] [--effect E] | verify`, all taking `[--dir DIR]` (default
`~/.delulu/audit` via HOME/USERPROFILE — only the CLI knows this path) and `[--json]`; a broken
chain exits 1 with the DL1405 diagnostic through the standard envelope; I/O problems exit 2 (a
missing/unreadable dir is never reported as tampering). Sink append failures are logged to stderr
and swallowed — the log is observability, not enforcement (playbook trap 6): no enforcement logic
reads it, and a sink failure never changes a decision. Tests: chain + canonicalization units;
write-N-then-verify; corrupt one byte on disk → DL1405 at the exact seq (lib + CLI forms); daily
rotation cross-links heads driven by the injected clock (no sleeps; lib + broker-driven forms);
head recovery on reopen; tail/query filters; broker-with-sink emits exactly one record per op
including denies, epoch ops emit none, and record seqs are gapless 1..=N.

**Phase 5e — lease tokens (delegate/redeem), MAC-signed — is implemented and green (2026-07-11,
323 → 333 workspace tests, +10; chunk 2 total +28 over the 305 baseline).** `src/lease.rs`.
**Broker key (§2):** random 256-bit; `load_or_create_key(path)` takes an INJECTED path (the CLI
will resolve `~/.delulu/broker.key` when the daemon lands in chunk 3), creating the key at first
use; `0600` under `#[cfg(unix)]`; on Windows v0.5 relies on the user-profile ACL (documented in a
comment — full owner-only DACL hardening arrives with the chunk-3 named pipe). The in-memory
`Broker` takes the key via additive `with_key`/`set_key`; absent injection it lazily self-mints
(library/test convenience — the CLI always injects from disk). **MAC (head-chef ruling):**
`blake3::keyed_hash` — no `hmac`/`sha2` crates. Token format is versioned:
`dlt1_<hex payload>.<hex mac>` where payload is the canonical JSON
`{ v: 1, node: "g_…", exp_millis, multi, nonce }` (same `canonical_json` as the audit chain) and
`mac = keyed_hash(broker_key, payload_bytes)`. MAC comparison is constant-time via `blake3::Hash`'s
constant-time `PartialEq` — `Hash` values are compared, never hex strings. **Operations (§3.2):**
`delegate(parent, authority, holder, ttl, multi) -> (GrantId, Token)` = the chunk-1 attenuation
core (`⊑` check, DL0802 carrying the intersection, fail-closed under dead parents) + mint a token
bound to the new child; exactly ONE `"delegate"` audit record — never an `attenuate` + `delegate`
double-log. `redeem(token, peer_desc) -> Result<GrantId, Denial>`: envelope/MAC failure (garbled,
tampered payload, rotated-away key, unknown bound node) → **DL1407** (new
`Denial::TokenInvalid { detail }`, `requires_human: true` per §8); expiry against the pluggable
clock → **DL1402**; single-redemption by default — a second redeem of the same token is DL1407
unless minted `multi`. Redeemed-nonce state lives in the `Broker`. Redemption binds the node to the
redeeming peer by writing `holder.peer` — storage/display ONLY, never a decision input: the
holder-neutrality grep test and the criterion-9 behavioral test stay green unmodified. One
`"redeem"` record per attempt (denied redeems record with `decision: "deny"`). `rotate_key()`
regenerates the key, which invalidates ALL outstanding tokens (their MACs no longer verify →
DL1407) — deliberate (§2), audit-logged as its own `"rotate_key"` action. Tests: delegate → redeem
→ `check()` works on the redeemed node; second redeem → DL1407; expired TTL at redeem → DL1402
(fake clock, no sleeps); token signed with a rotated-away key → DL1407; tampered payload → DL1407;
multi-token redeems twice; delegate-widening → DL0802 with the exact intersection; key-file
round-trip.

**Chunk-2 deviations / decisions flagged for head-chef review:**
1. **The audit day boundary is UTC** (pure-integer `civil_from_days`, no datetime dep — the same
   no-new-deps discipline as chunk 1's epoch-millis ruling). A local-time boundary would need a
   timezone database; UTC is deterministic and standard for audit trails.
2. **`verify` walks the full chain from genesis and stops at the first break.** Spec §7 sketches
   `verify [--from N]`; resuming mid-chain requires trusting an unverified `prev_hash` anchor, so
   it was left to chunk 3 (where the daemon can anchor on a verified head). Flagged, not silently
   dropped.
3. **Synchronous-use audit records use action `"use"`** with the scope argument as `target` and no
   `authority` payload (§7 names the actions for tree ops but not uses; `"use"` + target is the
   minimal honest shape). The op kind (FsWrite/Net/…) will ride along in chunk 3 when the runtime
   supplies spans too.
4. **Denied `redeem`s consume an audit seq** (parity with every other deny — invariant 26's "every
   deny produces exactly one record"), even though redeem is not itself a §4.1-classed capability
   use.
5. **Token `exp_millis` for a no-TTL delegation is `i64::MAX`** ("never expires"), keeping the
   payload shape fixed rather than making the field optional.
6. **`load_or_create_key` checks existence before creating** (rather than matching the read error's
   kind); the same-user TOCTOU window is inside the threat model (§10 — the broker does not defend
   against the same OS user).

**Phase 5f — the `Custody` trait + IPC daemon/client — is implemented and green (2026-07-12,
333 → 354 workspace tests over chunk 3; the daemon lifecycle is the ONE integration test allowed to
spawn a real process, playbook 5f).** `delulu-runtime::custody` defines the seam
`Custody { check(op, arg) -> CustodyDecision, expose(name, span), refresh_epoch(), mode() }`;
BOTH the interpreter and the WASM host call it (playbook §1 — no duplicated policy). `EmbeddedCustody`
is a pass-through: `check` always allows so the Stage 1–4 in-process `prim.rs` scope checks stay the
enforcement and the entire prior conformance suite passes UNMODIFIED (criterion 11); labelled
`custody: embedded`. `delulu::broker_client::BrokerClientCustody` is the daemon-mode impl: synchronous
ops (`FsWrite`/`Net`/`Declassify`/`ForeignBind`) round-trip per use against live tree state; epoch ops
(`FsRead`/`Clock`/`Rand`/`Console`) validate against a client-cached `Snapshot` rebuilt from a
`NodeState` round-trip and refreshed at most every `--epoch-ms` (`clamp_epoch_ms`: default 50, floor 1,
ceiling 250 — §4.1). **Transport** (`broker_transport`): Windows named pipe
`\\.\pipe\delulu-broker-<hash>` created with an owner-only DACL (`D:P(A;;GA;;;<SID>)`) plus a
`GetNamedPipeClientProcessId` + `EqualSid` peer-SID check (same-user only, playbook trap 7); Unix
domain socket in a `0700` dir (`SO_PEERCRED`/`getpeereid` flagged as post-chunk-3 hardening — the dir
mode already bounds to the same user). **Wire protocol** (`broker_ipc`): length-prefixed (u32-LE)
canonical CBOR via `ciborium` (fixed-field structs → deterministic encoding), versioned `broker/1`;
a mismatched version is answered **DL1406** and never acted on. The daemon (`brokerd`) owns one
`Broker` + `AuditLog` + `SecretStore` for one OS user, single blocking thread, one request per
connection; `delulu broker start|status|stop|rotate-key`; a synchronous-class op whose audit record
cannot be appended is REFUSED (fail-stop on inability to record, invariant 26). **FAIL CLOSED
(invariant 27, the stage's most dangerous possible bug):** `BrokerClientCustody` has NO embedded path
to fall back to — an unreachable broker makes every effectful op **DL1401** carrying the exact
`delulu broker start` command, fast (bounded, never a hang), and `mode()` stays `"daemon"`; the
epoch-class `refresh_cache` discards the stale snapshot *before* the round-trip, so a dead broker
cannot keep serving allows. The integration test proves it end-to-end through the real binary: a
daemon-mode run performs a granted write, `broker stop`, then the SAME run fails DL1401 and the
sentinel file stays absent (executable proof there was no silent embedded fallback), while an embedded
run still works (criterion 11). WASM host: a broker denial inside a host callback is RECORDED in
`HostState.refused` and surfaced after the call — **never an `Err` across the wasm frame** (playbook
trap 5, carried from Stage 3/4).

**Phase 5g — broker-held secrets; `expose` synchronous through the broker — is implemented and green
(2026-07-12).** `delulu-broker::SecretStore` (`src/secrets.rs`) holds secret bytes broker-side, loaded
at daemon startup from `<state>/secrets.json` (`delulu secrets set NAME VALUE` writes it — a daemon
started after the write picks it up on restart, v0.5). `Broker::expose(node, name, store, span)` is a
synchronous-class declassification: bytes cross into the program process ONLY here, audited with the
calling span; a node without `Declassify` + the secret in its `secrets` scope is refused (bytes never
cross for it). `Broker::secret_map(node, name, op, arg, store)` runs the whitelist map ops broker-side
and returns a handle to the derivative (which then exposes for the authorized node). In daemon mode
the Stage-3 WASM host secret table becomes a client-side cache of broker HANDLES — so "secrets never
enter the guest" (DL1205) composes with "bytes cross only on expose": neither the guest nor the
host-process cap cache holds bytes pre-`expose` (criterion 6, stronger than Stage 3's scan). The
integration test exposes a broker-resident secret through the daemon, confirms the bytes arrive only
via `expose`, that the audit record carries the calling span, and that a non-`Declassify` node is
denied.

**Chunk-3 deviations / decisions flagged for head-chef review (all resolved live by the head chef):**
1. **Three transport/daemon bugs were found by head-chef live verification and fixed** (the in-process
   unit tests could not surface them; only the real-binary integration test did): (a) a **named-pipe
   reconnect race** — the single-instance server re-creates its pipe between requests, so a client
   reconnecting in that sub-millisecond gap got `ERROR_FILE_NOT_FOUND`; `connect` now retries that
   error within a short bounded window (a genuinely-down daemon still fails closed fast). (b) The
   detached daemon **inherited the launching process's stdio pipe handles** (Rust spawns with
   `bInheritHandles = TRUE`), so a caller capturing output (`Command::output()`, a shell pipe, CI)
   hung forever waiting for an EOF the long-lived daemon held open; fixed by clearing
   `HANDLE_FLAG_INHERIT` on the parent's std handles before the detached spawn, and redirecting the
   daemon's own stdio to `<state>/broker.log`. (c) The detached daemon **inherited and locked the
   caller's working directory**; fixed by giving the daemon a stable cwd (its state dir). All three
   are real-world defects, not test artifacts.
2. **The custody label in the `authority` report** is carried in the JSON always (criterion 11:
   "custody label present in reports") but the HUMAN render prints it only when custody is non-default
   (`daemon`), so the default embedded report stays byte-identical to Stages 1–4 — matching how the
   `run` path gates its human custody line. (Criterion 11's two halves — "no regression" and "label
   present in reports" — are both honored this way.)
3. **`--epoch-ms` is clamped to [1, 250] with default 50**; a value above the ceiling would stretch
   the honest §4.2 revocation-latency window beyond what the spec states, so it is capped rather than
   trusted.
4. **`delulu secrets set` writes the store directly and the daemon reads it at startup** (no live
   reload). Live reload / a richer `secrets list|rm` surface is chunk-5 CLI polish; the minimal
   set-then-restart path is enough to prove the 5g semantics.

*Stage 5 puts the keys where code can't reach them. Stage 6 lets code arrive at runtime and still
not reach them.*
