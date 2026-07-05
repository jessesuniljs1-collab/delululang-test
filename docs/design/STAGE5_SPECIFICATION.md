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

*Stage 5 puts the keys where code can't reach them. Stage 6 lets code arrive at runtime and still
not reach them.*
