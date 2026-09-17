# Stage 5 Playbook — "Custody" (broker, grant tree, revocation, microVM)

**Companion to:** `docs/design/STAGE5_SPECIFICATION.md` (normative). This file is *how to build it*.
**Depends on:** Stage 4 complete. **Language surface delta: none** — Stage 5 is entirely
runtime/custody, so **not one line of `delulu-syntax` or `delulu-check` typing changes.** If you
find yourself editing the type checker, stop: you have misread the stage.

> **The one-sentence goal:** move root authority *out of the program's process* into a broker the
> program cannot reach, and make grants a persistent, auditable, revocable **tree** where every
> node's authority `⊑` its parent and revoking a node kills its whole subtree. The holder model
> (§5.16) must hold with **zero code paths that branch on whether the holder is a human, an LLM, an
> agent, or a robot** — that neutrality is acceptance criterion 9 and the soul of the stage.

---

## 0. Orientation — read these before writing anything

- Spec §1 invariants 23–27 (keys leave the process; the tree is the law; no upward/lateral
  visibility; complete audit; fail closed).
- Spec §4 (the two validation classes — **synchronous** vs **epoch** — and the honest latency
  bound; this is the single most important design decision to get exactly right).
- `CONSTITUTION.md` §5.14 (threat model — what the broker does *not* defend against) and §5.16
  (the holder model laws).
- `SOUNDNESS_AUDIT.md` §A.3 (the `⊑` attenuation lattice) and R-7 (monotone attenuation).
- The existing `crates/delulu-runtime/src/broker.rs` — Stage 1's `Grants`/`build_root` is the
  *embedded* broker. Stage 5 keeps it (as `custody: embedded`, dev mode) and adds the daemon
  alongside. **Do not delete or rewrite the embedded path**; both must pass conformance (criterion 11).

### The central architectural choice: prove the design in embedded mode first

The grant *tree*, the `⊑` check, transitive revocation, the audit chain, the two validation
classes, and holder-neutrality are all **pure data-structure + policy logic**. None of them need a
socket. Build and test *all of it* in-process first (an in-memory broker behind a trait), then add
the IPC transport as a thin, separately-tested layer. This lets ~70% of the stage be verified with
ordinary unit tests before you touch daemon lifecycle, sockets, or OS peer credentials — the parts
that are slow and platform-specific to test.

---

## 1. Suggested crate topology

Add one crate: **`delulu-broker`** (the grant tree, audit chain, secret store, validation classes,
lease-token HMAC — all transport-agnostic). It depends only on `delulu-diag` and `delulu-check`
(for `ResourceKind`/`Effect` and the `⊑` lattice). Then:

- The **daemon** (`delulu broker`) and the **client** both live in `delulu` (the CLI crate) and
  talk to `delulu-broker` — the daemon by owning a `Broker` directly, the client over IPC.
- `delulu-runtime` gains a `Custody` trait it calls instead of reaching `Grants` directly:
  `check(op, scope_args) -> Decision`, `expose(secret_ref)`, `refresh_epoch()`. Two impls:
  `EmbeddedCustody` (wraps today's `Grants`) and `BrokerClientCustody` (IPC). **The interpreter and
  the WASM host both call the trait** — this is how Stage 3's host secret table becomes "a
  client-side cache of broker handles" (spec §4.4) without duplicating logic.

Rationale: keeping the tree/audit/lattice in a transport-free crate is what makes criterion 9
(holder-neutrality) *mechanically checkable* — you grep one crate and assert no `match holder.kind`.

---

## 2. Phase plan (each phase: build green → test green → update spec §"status" → commit)

Order chosen so every phase is independently testable and the risky, platform-specific work
(sockets, seccomp, microVM) comes *after* the policy core is proven.

### Phase 5a — the `⊑` attenuation lattice, extracted and tested
Move/expose the authority-comparison logic (`authority ⊑ authority`) into `delulu-broker` as the
single source of truth, returning either "ok" or the **computed intersection** (needed for DL0802's
exact repair — the repair *never widens*, spec §8). Unit-test the lattice hard: effects subset,
per-scope subset (fs paths are subtree-prefix, net is host-set, secrets/declassify/foreign are
name-sets). This is the mathematical heart; get it bulletproof before anything consumes it.
*Test:* a table of (parent, child, expected ok/intersection) pairs covering every scope dimension.

### Phase 5b — the grant tree (in-memory), issue/attenuate/revoke
`Broker` struct holding `nodes: Map<GrantId, Node>` (spec §3.1 shape). Implement `issue` (root
node), `attenuate` (child iff `⊑`, else DL0802), `revoke` (mark node + **transitively** all
descendants; idempotent), `inspect`/`tree`. `GrantId` = random opaque (`g_` + hex). **No holder-kind
branching anywhere** — `holder` is a descriptive `{kind, desc, peer}` blob that is *stored and
displayed, never switched on*.
*Test:* build a 3-level tree; revoke the middle; assert the leaf is revoked and the root is live;
assert attenuate rejects a widening request with the right intersection; **criterion 9 unit form** —
run the same tree operations with `holder.kind` set to `"human"`, `"process"`, `"delegate"` and
assert byte-identical tree outcomes.

### Phase 5c — the two validation classes + revocation epochs
Implement `check(node, op, scope_args) -> Decision`. Class each op per spec §4.1
(synchronous: Declassify/FsWrite/Net/foreign/plugin-load/Actuate; epoch: Read/Clock/Rand/Console).
The **epoch counter** increments on every revocation; the runtime holds a cached snapshot refreshed
every `--epoch-ms` (default 50, ceiling 250). A dead lease is **DL1403 carrying the revoking audit
seq** (spec §4.3 — this "why and when your authority died" message is a feature for agents; make the
JSON error rich). Synchronous ops re-validate against live tree state every call; epoch ops validate
against the snapshot.
*Test:* revoke → synchronous op fails on the *next* call; epoch op fails within one epoch interval
(unit-test the snapshot logic directly with a fake clock — do not sleep in unit tests).

### Phase 5d — the append-only hash-chained audit log
`~/.delulu/audit/YYYYMMDD.jsonl`, records per spec §7 with
`hash = blake3(prev_hash ‖ canonical_record)` (blake3 already a dep from Stage 2). One record per
issue/attenuate/delegate/revoke/deny and per synchronous-class use. `delulu audit
tail|query|verify`. **The header line of every log file states "observability, not enforcement."**
Verification failure = DL1405 (`requires_human: true`).
*Test:* write N records, `verify` passes; corrupt one byte of one record, `verify` fails with DL1405
at the correct seq; daily rotation cross-links heads.

### Phase 5e — lease tokens (delegate/redeem), HMAC-signed
Broker key: random 256-bit at first start (`~/.delulu/broker.key`, 0600), HMAC-signs portable lease
tokens. `delegate(parent, authority, ttl)` = attenuate + mint token; `redeem(token)` binds it to the
redeeming process, **single-redemption by default** (second redemption DL1407 unless `--multi`).
`rotate-key` invalidates outstanding tokens (audit-logged). This is *the orchestration primitive*:
"an LLM holding `g_orch` gives each parallel agent a token" — build the delegate/redeem path so that
story runs (spec §3.3).
*Test:* delegate → redeem → use; second redeem of a single-use token → DL1407; expired TTL → DL1402;
token signed with a rotated-away key → DL1407.

### Phase 5f — the IPC transport (daemon + client)
Now — and only now — add the socket. Unix domain socket `$XDG_RUNTIME_DIR/delulu/broker.sock`
(mode 0700); Windows named pipe with owner-only DACL. Peer identity = **OS peer credentials**
(`SO_PEERCRED` on Linux, `getpeereid`/equivalent on macOS, `GetNamedPipeClientProcessId` +
token-owner on Windows). Wire protocol: length-prefixed canonical CBOR, versioned `broker/1`,
mismatch DL1406. The daemon owns one `Broker` and serves **exactly one OS user**. `BrokerClientCustody`
speaks this protocol. **Fail closed:** broker unreachable ⇒ effectful ops fail DL1401, *never* silent
fallback to embedded (invariant 27).
*Test:* round-trip issue/attenuate/revoke over the socket == the in-memory outcomes; kill the daemon
mid-run → next effectful op is DL1401 (not a hang, not a fallback); protocol-version mismatch → DL1406.
*Platform note:* gate the socket tests on `cfg(unix)` / `cfg(windows)`; the daemon lifecycle test is
the one integration test allowed to spawn a real process.

### Phase 5g — broker-held secrets; `expose` synchronous through the broker
`root.secret(name)` returns a handle to a **broker-resident** secret (env / file / `delulu secrets
set` store). Bytes enter the program only on `expose` (synchronous class, audited with the calling
span). **The Stage-3 WASM host secret table becomes a client-side cache of broker handles** — so
Stage 3's "secrets never enter the guest" (DL1205) composes: in daemon mode the guest still never
sees bytes pre-`expose`, and now neither does the host-process cap cache (criterion 6, *stronger*
than Stage 3's scan). `Secret.map` whitelist ops execute **broker-side**.
*Test:* `expose` round-trips + appears in audit with span; secret bytes absent from program-process
memory before `expose` (extend the Stage-3 memory-scan test; now also scan the host cap cache).

### Phase 5h — foreign workers (redeeming the Stage 4 honesty note)
`--foreign-isolation process` (default in daemon mode): each granted foreign lib / the Python
interpreter runs in a **worker subprocess** at minimum privilege — no inherited handles, `job object`
kill-on-close (Windows), `PR_SET_PDEATHSIG` + a basic seccomp profile (Linux) — speaking the Stage-4
marshalling protocol over a *private* pipe. The worker **cannot reach the broker socket** (path not
disclosed, handle not inherited). Worker crash ⇒ `ForeignErr::WorkerDied`, host continues.
*Test (criterion 7):* a C function that segfaults kills only the worker → `WorkerDied`; assert the
worker process gets `ECONNREFUSED`/absence connecting to the broker socket. `inproc` stays for dev,
labeled in `delulu authority`.

### Phase 5i — the microVM profile (Linux-first)
`delulu run --isolation microvm` (Linux x86_64/aarch64, KVM): WASM engine mandatory; program runs in
a Firecracker/cloud-hypervisor guest with read-only rootfs, virtio-fs mounts matching granted `fs.*`
scopes exactly, **default-deny egress** (a host userspace proxy is the only network path, enforcing
the `net` allowlist), broker access via vsock to a **broker proxy that holds one node, not the
socket** (invariant 25 preserved). Non-Linux: DL1408 with the documented fallback
(`--isolation process`, explicitly weaker, labeled). Ship the **isolation matrix as a table**
(none/process/microvm × capabilities), not prose.
*Test (criterion 8, Linux CI only):* `http.get` to a non-allowlisted host fails inside the guest
though the host machine can reach it; granted path readable; sibling path not even mountable-visible.
*This phase is the most environment-heavy — isolate it so a non-Linux implementer can ship 5a–5h and
mark 5i platform-pending without blocking the stage's core.*

### Phase 5j — CLI surface + `delulu authority` custody label + honesty text
`delulu grants list|tree|inspect|revoke|delegate`, `delulu run --lease <token>`, `delulu broker
start|stop|status|rotate-key`, `delulu audit …`, `delulu secrets set`. `--grant` flags in daemon
mode = sugar for issue-then-run at the root. `delulu authority`/`run` print `custody: embedded|daemon`.
Wire `delulu explain E-REVOKE` to state the §4.2 latency bound verbatim. Copy spec §10 caveats into
docs and explain-text word-for-word.

---

## 3. The traps (learned from Stages 1–3, plus stage-specific)

1. **Do not touch the type checker.** Language surface delta is zero. `Root`/caps/rows are
   untouched — custody changes must never require program changes (that *is* the stage's thesis).
2. **Holder-neutrality is a grep-checkable invariant, not a vibe.** Write the assertion test early
   (criterion 9): the `delulu-broker` crate must contain no `match`/`if` on holder kind on any
   authority-decision path. Run the delegation conformance suite three times with the orchestrator
   role played by (a) the CLI, (b) a DeluluLang program, (c) a shell script standing in for an LLM
   harness — identical outcomes.
3. **Never claim "immediate" revocation.** The honest bounds are §4.2: synchronous = before next
   use, epoch = ≤ one interval. This exact wording goes in `delulu explain E-REVOKE` *and* every
   revocation's audit record. Acceptance criterion 2 measures the epoch bound with ×3 CI-jitter
   tolerance — mirror that tolerance in the test.
4. **Fail closed, provably.** The single most dangerous bug in this stage is a silent fallback to
   embedded custody when the broker is unreachable. Add a dedicated test that kills the daemon and
   asserts DL1401 (not a hang, not embedded fallback). Invariant 27.
5. **Do not return `Err` from a Wasmtime host callback** (carried trap) — the broker round-trip
   inside a synchronous-class WASM op must record the refusal in host state and surface it after the
   call, exactly like Stage 3's console/fs refusals.
6. **The audit log is not enforcement.** If you ever find enforcement logic reading the audit log,
   that is a bug — enforcement is the tree + epoch snapshot; the log only *observes*.
7. **One broker per OS user; cross-user is out of scope.** Do not build multi-tenant auth; peer
   credentials just confirm "same user." (Multi-machine/remote brokers are post-1.0, spec §10.)
8. **microVM is Linux-first and that is stated honestly.** Do not fake a weaker platform's isolation
   as equivalent; the fallback is labeled weaker in `delulu authority` output.

---

## 4. Definition of done (map to spec §9 acceptance criteria)

Ship when all 11 spec criteria pass. Minimum bar to call the stage core-complete even if 5i
(microVM) is platform-pending: criteria 1–7, 9, 10, 11 green on Linux+Windows; criterion 8 gated
`#[cfg_attr(not(kvm), ignore)]` with a tracking note. Update `STAGE5_SPECIFICATION.md` with an
`## 11. Implementation status` section (mirroring how Stage 3's §8a logs each phase), and record in
every revocation-related commit that the interpreter/embedded semantics are the reference the daemon
path must match.

*Stage 5 puts the keys where code can't reach them. Stage 6 lets code arrive at runtime and still
not reach them — its plugin grants are children in this stage's tree.*
