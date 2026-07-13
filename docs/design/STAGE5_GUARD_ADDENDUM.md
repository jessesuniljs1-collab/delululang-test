# Stage 5 Addendum — The Guard (chunk 6, phases 5k–5m)

**Status:** BUILT 2026-07-14 (head-chef live-verified: all 11 criteria demoed against the real
binary; full workspace suite green on Windows and Linux/WSL). Specified 2026-07-13 (head chef);
implemented by the Opus 4.8 chef under this brief (commits 28fa620 → dbbcd46 → 5f4306b).
**Parent spec:** `docs/design/STAGE5_SPECIFICATION.md` (Custody). This addendum extends Stage 5;
it does not reopen the chunk-1–5 close-out.

---

## 1. Origin and credit

Inspired by [`dcg` (Destructive Command Guard)](https://github.com/Dicklesworthstone/destructive_command_guard)
by Jeffrey Emanuel — a shell hook that blocks destructive commands (`git reset --hard`,
`rm -rf`, `DROP TABLE`, …) before AI coding agents execute them, with graduated response
levels, single-use "allow-once" approval codes, an explain mode, and audit logging.
A read-only clone was inspected at head-chef level before this design was written.

`dcg` guards an *untyped* boundary (shell strings), so most of its bulk is string analysis:
SIMD-accelerated regex, heredoc/inline-script scanning, 50+ pattern packs, execution-vs-data
context classification. **Delulu does not need any of that**: effects are typed in the
language and authority is enforced structurally at the broker. What transfers is `dcg`'s
*policy layer* — the ideas about how a principal supervises agents.

### 1.1 Adopted / rejected

| `dcg` concept | Verdict | Delulu form |
|---|---|---|
| Graduated response (WARN / SOFT_BLOCK / HARD_BLOCK) | **Adopt, simplified** | Three tiers per rule: `warn` / `guarded` (approvable) / `sealed` (not runtime-approvable) |
| Allow-once codes (short-lived, scoped, single-use, audited) | **Adopt, strengthened** | Broker-held **permits** minted by principal approval — never bearer codes claimable by whoever runs a CLI verb |
| Explain mode (`dcg explain`) | **Adopt** | `delulu explain E-GUARD` topic + full explain bodies for DL1410–DL1414 |
| Awareness banners / agent-safe machine output | **Adopt** | Guard status line on every `run --lease`; `--json` on guard verbs; broker-start banner |
| Audit every guard event | **Adopt** | New event kinds in the existing hash-chained audit log (spec §5) |
| Bypass with mandatory warning | **Adopt** | `--dangerously-bypass-guard` — banner states why it is a bad idea; every bypassed use audited + agent-warned |
| Regex/heredoc/pack string-scanning engine | **Reject** | Effects are typed; the broker sees structured authority, not strings. Zero parsing. |
| Fail-open ("never block your workflow") | **Reject** | The guard is part of the authority boundary → **fail closed** (invariant 27). Unreadable guard policy ⇒ guarded classes refuse. |
| Occurrence-based graduation counters (session/history thresholds) | **Reject (future work)** | DRAFT even upstream; heavy state. The tier model + permits covers the need. |
| Per-agent trust profiles (`[agents.claude-code]`…) | **Reject** | Would violate holder-neutrality (criterion 9). Tree *position* (root vs delegated) replaces identity. |

---

## 2. The model

### 2.1 Principal and agent, holder-neutrally

Jesse's requirement: the guard must not be overridable by "agents or llms or anything else
writing code" unless allowed by the "main human user or main llm or main other AI"
(the **principal**).

Criterion 9 forbids switching on holder identity. The holder-neutral mapping is **tree
position**:

- **Root nodes** (minted by `Issue`, a CLI-only act against the broker socket) are
  principal-held *by construction* — whoever controls the broker daemon minted them.
- **Delegated nodes** (anything with a parent; everything a lease redeems into) are
  agent-held *by construction*.

Guard enforcement applies to **delegated (non-root) nodes only**. Root use of any authority
passes without guard interaction — the principal supervising themselves is not the threat
model. No holder field is ever consulted (criterion 9 intact).

### 2.2 The owner code (who may administer the guard)

At `delulu broker start` the daemon generates an **owner code** (`gow1_` prefix, from the
existing token-randomness machinery), prints it **exactly once** in the start banner, and
keeps it **only in daemon memory** — never on disk. Restarting the daemon rotates it.

Guard **admin verbs** (`approve`, `deny`, `policy set|unset`, `bypass on|off`,
`permits revoke`) require it via `--owner <code>` or `DELULU_GUARD_OWNER`. A missing or
wrong code refuses with **DL1414**. Read verbs (`status`, `pending`, `permits list`,
`request`) need no code — awareness must be free.

This is a real boundary inside our machinery: a subagent never sees the code unless the
principal puts it in the agent's context. A main LLM acting as principal starts the broker,
captures the code, and withholds it from subagents.

### 2.3 Guard policy

Broker-held (custody), persisted beside the broker state, edits audited. A policy is a set
of rules:

```
rule := <class>:<pattern> → <tier>
class ∈ { effect, fs_read, fs_write, net, secret, declassify, foreign_c, foreign_python }
pattern := same matcher vocabulary the broker already uses for that axis
           (path globs / host patterns / names / effect names; `*` = all)
tier ∈ { warn, guarded, sealed }
```

- `warn` — the use proceeds; a one-line warning goes to the agent's stderr (first use per
  rule per run) and a `guard_warn` event is audited.
- `guarded` — refused with **DL1410** unless a matching permit exists. Approvable at
  runtime via the request flow (§2.5).
- `sealed` — refused with **DL1413** always. Not approvable at runtime; only a principal
  policy edit (owner-coded) can unseal. **Bypass does not lift `sealed`.**

**Default policy** (zero-config protection, the dcg idea adapted to what is actually
dangerous in Delulu — the classes where the type system's guarantees end or a mistake is
catastrophic):

```
declassify:*      → guarded   (secret material leaving custody)
foreign_c:*       → guarded   (native code = end of effect-typing guarantees)
foreign_python:*  → guarded   (same)
```

Everything else defaults to no rule (grants alone bound it). `sealed` is empty by default —
so out of the box, bypass mode really does skip everything, matching the Claude Code
skip-permissions shape; `sealed` is opt-in principal hardening.

### 2.4 Enforcement points (broker-side, all of them)

1. **Use time.** A `Check` from a delegated node whose op falls in a `guarded`/`sealed`
   rule: `Decision{allow:false}` with DL1410/DL1413 unless a permit matches (`guarded`
   only). Permit matching reuses the existing `⊑` machinery: the use must be ⊑ the permit's
   approved subset, on the permit's node.
2. **Validation-class interaction.** Guarded/sealed-class ops **always validate
   synchronously**, regardless of `--epoch-ms` — a cached snapshot cannot consult permits.
   Ungated ops keep epoch caching unchanged. A policy edit bumps the epoch (the same
   mechanism revocation uses), so the stated bound is VERBATIM the revocation sentence
   pattern: *guard policy changes take effect: synchronous class — before the next use;
   epoch class — within one epoch interval (≤ 50 ms default).*
3. **Minting time.** `Attenuate`/`Delegate` whose child spec *includes* guarded/sealed
   authority is refused (DL1410/DL1413, minting context in the message) unless a matching
   permit exists or the request carries the owner code (the principal minting directly).
   An agent can never quietly re-delegate guarded authority to children — a parent's own
   permit does not cover minting; the child mint needs its own approval.

Fail closed everywhere: guard policy store unreadable/corrupt ⇒ guarded classes refuse
(never fail open); daemon down ⇒ DL1401 exactly as today.

### 2.5 The approval flow (request → approve/deny with comments)

- A DL1410 refusal names the **exact escalation command** in its message, dcg-style
  remediation: `delulu guard request <node> --use <class:pattern> --why "<justification>"`.
- `guard request` **requires `--why`** — the agent must explain why it needs the authority
  (Jesse's requirement, verbatim: "It should explain why it is so"). The broker stores a
  pending request `{id, node, requested subset, why, created}`; pending TTL 30 min;
  a repeat request for the same (node, subset) returns the existing id (dedup).
- While pending, a retried use refuses with **DL1411** carrying the request id.
- Principal: `delulu guard pending` lists requests **with justifications**;
  `delulu guard approve <id> --owner <code> [--ttl 15m] [--uses N] [--comment "…"]`;
  `delulu guard deny <id> --owner <code> --comment "…"` (**comment required on deny** —
  the agent must learn why; optional on approve).
- Approval mints a **permit**: broker-held, bound to (node, approved subset), default
  TTL 15 min, unlimited uses within scope unless `--uses N`; revocable
  (`guard permits revoke <id> --owner <code>`); daemon-memory only (a restart clears
  permits and pending requests — approvals are deliberately session-scoped, tighter than
  dcg's 24 h). No bearer token exists; the agent simply retries and the broker honors the
  permit.
- After a deny, a retried use refuses with **DL1412 carrying the principal's comment
  verbatim**.
- Every step audited: `guard_block`, `guard_request` (with why), `guard_approve` (with
  comment, ttl, uses), `guard_deny` (with comment), `guard_permit_use`,
  `guard_permit_expire`/`_revoke`, `guard_policy_edit`, `guard_warn`,
  `guard_bypassed_use`, `guard_bypass_on`/`_off`.

### 2.6 Bypass ("skip all", with awareness both ways)

- Enable at start: `delulu broker start --dangerously-bypass-guard` (the name says why,
  echoing Claude Code's `--dangerously-skip-permissions`). Toggle at runtime:
  `delulu guard bypass on|off --owner <code>`.
- Enabling **always prints the banner** (spec-fixed exact text, agent to draft, head chef
  to approve; it must state: the guard is the approval checkpoint between delegated agents
  and the classes where mistakes are catastrophic or the type system's guarantees end —
  declassification, native code; bypassing it means any agent holding any lease uses those
  classes without your knowledge until you read the audit log).
- While bypassed: `guarded` uses **proceed**, each audited as `guard_bypassed_use`, with a
  one-line agent-side stderr warning on first use per rule per run — the agent is aware
  too. `warn` behaves as before. **`sealed` still refuses** (DL1413).
- `guard status` and every `run --lease` awareness line show `BYPASSED` prominently.
  Audit chain remains fully live — bypass never touches auditing.

### 2.7 Awareness (both directions — a hard requirement)

- **Agent-side:** every `delulu run --lease` prints, before user code output, one guard
  status line — e.g. `guard: on — guarded: declassify:*, foreign_c:*, foreign_python:* —
  to request access: delulu guard request` or `guard: BYPASSED by the principal — guarded
  uses will proceed and be audited`. Sourced from a `GuardStatus` wire call post-redeem.
  `delulu explain E-GUARD` explains the whole model; DL1410–DL1414 get full explain bodies
  with §10-style honesty caveats.
- **Principal-side:** the broker start banner shows guard mode + rule digest + the
  print-once owner code; `delulu guard pending` surfaces the queue with justifications;
  `delulu guard status [--json]` is the machine surface (dcg robot-mode nod: plain JSON on
  stdout, decorations on stderr — matching our existing `--json` conventions).

### 2.8 Honesty (§10 tradition — goes in explain bodies and the spec)

The guard supervises what *Delulu programs holding delegated grants* can do. It does not
defend against a malicious same-OS-user process that never speaks Delulu — the same honesty
as DL1401's "not against the OS user". The owner code raises the bar only while the
principal keeps it out of agent-visible context. `warn` tier and bypass mode are awareness
mechanisms, not enforcement. Never claim otherwise anywhere.

---

## 3. New surface

### 3.1 CLI

```
delulu guard status [--json]
delulu guard pending [--json]
delulu guard request <node> --use <class:pattern> [--use …] --why "<text>" [--json]
delulu guard approve <req-id> --owner <code> [--ttl <dur>] [--uses <n>] [--comment "<text>"]
delulu guard deny <req-id> --owner <code> --comment "<text>"
delulu guard permits [list|revoke <id> --owner <code>] [--json]
delulu guard policy [show|set <class:pattern> <tier>|unset <class:pattern>] --owner <code> (show: no code)
delulu guard bypass on|off --owner <code>
delulu broker start --dangerously-bypass-guard [--guard-policy <file>]
```

### 3.2 Wire (additive `broker/1` variants — chunk-5 precedent, both ends one binary)

`ReqBody::GuardStatus`, `GuardRequest{…}`, `GuardPending`, `GuardApprove{…}`,
`GuardDeny{…}`, `GuardPermits`, `GuardPermitRevoke{…}`, `GuardPolicyShow`,
`GuardPolicySet{…}`, `GuardPolicyUnset{…}`, `GuardBypass{on: bool, …}` — admin variants
carry `owner: String`. Matching `Response` variants, fixed-field structs, deterministic
CBOR. Wire version stays `broker/1`.

### 3.3 Diagnostics (contiguous from DL1410; keep the no-DL1404 note untouched)

| Code | Meaning |
|---|---|
| **DL1410** | guard refusal: guarded authority, no permit — carries the matched rule and the exact `guard request` command; `requires_human: true` |
| **DL1411** | guard request pending — carries the request id |
| **DL1412** | guard request denied — carries the principal's comment verbatim |
| **DL1413** | guard sealed refusal — not runtime-approvable; names the policy-edit path; bypass does not lift it |
| **DL1414** | guard owner code missing/invalid — admin verb refused |

Plus topic `E-GUARD` in `topic_explain`. Every code gets a full explain body with the §2.8
honesty caveats; DL1410's body must contain the request-command remediation, DL1412's must
say the comment is the principal's words.

---

## 4. Acceptance criteria (all live-verified by the head chef against the real binary)

1. A delegated node using guarded authority without a permit refuses with DL1410 naming
   the exact request command; nothing executes.
2. Root use of the same authority passes with no guard interaction (position, not
   identity — criterion 9 intact; grep proves no holder field is consulted).
3. `Delegate`/`Attenuate` cannot mint guarded authority into a child without a permit or
   owner code; refusal carries minting context.
4. Request → approve → retry succeeds within the permit TTL; one audit chain shows
   block → request (with why) → approve (with comment) → permit use; `delulu audit verify`
   green.
5. Request → deny → retry refuses with DL1412 carrying the deny comment verbatim.
6. `sealed` refuses with DL1413 even with an approved request pending, and under bypass.
7. Bypass (start flag and runtime toggle) prints the exact banner; guarded uses proceed
   with per-rule agent-side warnings + `guard_bypassed_use` audit events; audit verify
   green.
8. Every `run --lease` prints the guard status line before user output, in both `on` and
   `BYPASSED` states; `guard status --json` is valid machine JSON.
9. Admin verbs without a valid owner code refuse with DL1414; the owner code appears
   exactly once in broker-start output and never on disk (grep the state dir).
10. Fail closed: an unreadable/corrupt guard policy store refuses guarded classes (test
    corrupts the store on purpose); ungated authority is unaffected.
11. Guarded-class ops validate synchronously under `--epoch-ms`; ungated ops keep epoch
    caching (existing epoch tests pass unchanged); the policy-change bound sentence appears
    verbatim where revocation's does.

---

## 5. Phases

- **5k — policy + enforcement.** Guard policy store (persist, audited edits) in
  `delulu-broker` (new `guard.rs`); default policy; enforcement at Check/NodeState epoch
  bump/Attenuate/Delegate; owner code at daemon start; DL1410/DL1413/DL1414 in
  `delulu-diag`; `broker start` flags; `guard status|policy` verbs + wire. Tests: criteria
  1, 2, 3, 9, 10, 11.
- **5l — approvals.** Pending requests + permits (daemon memory); `guard
  request|pending|approve|deny|permits` verbs + wire; DL1411/DL1412; required `--why` and
  deny `--comment`; audit event kinds. Tests: criteria 4, 5, 6.
- **5m — bypass + awareness + docs.** Bypass flag + toggle + banner; `warn` tier plumbing;
  `run --lease` status line; `E-GUARD` topic + five explain bodies; end-to-end
  orchestration test (grants_cli.rs style: block → request → approve → use → deny path →
  bypass → sealed → audit verify); .md updates (this file's close-out table below,
  STAGE5_SPECIFICATION.md pointer, playbooks README row, REPOSITORY_STRUCTURE.md if files
  are added). Tests: criteria 7, 8 + the e2e.

Commit per phase, message style matching chunk 5. **NEVER push to GitHub.**

---

## 6. Head-chef rulings (numbered, binding)

1. Fail closed, not dcg's fail open — the guard is authority boundary, not ergonomic hook.
2. Tree position replaces agent identity; no holder field consulted, ever (criterion 9).
3. Permits are broker-held, never bearer tokens; daemon-memory only; default TTL 15 min.
4. Guarded/sealed ops always validate synchronously; policy edits bump the epoch; the bound
   sentence reuses the revocation pattern verbatim.
5. Bypass lifts `guarded` to audited-and-warned, never touches `sealed`, never touches
   auditing.
6. Occurrence-graduation and a static `delulu check` guard-awareness warning are documented
   future work, not in scope.
7. Owner code: print-once, memory-only, rotates per daemon run.
8. The string-scanning universe of dcg is out of scope forever — if a future stage wants
   command scanning, it is a different tool, not the broker.

## 7. Deviations (implementing chef appends; head chef approves)

1. **Guard pattern matching is `*`-or-exact** (`pattern_matches` in `guard.rs`): `*` matches every
   token on the axis; anything else is exact-string equality. §2.3 says "the same matcher
   vocabulary the broker already uses for that axis", and the broker's scope lattice is
   deliberately exact-string (chunk-1 phase-5a ruling 4: no pattern implication in v0.5) with the
   fs dimensions adding lexical descendant checks. A guard rule gates *classes of use* rather than
   delegating authority, so `*` (the "all" the brief's own grammar names) plus exact matches
   covers the shipped need without inventing a new pattern language; fs-descendant-aware guard
   patterns are a possible refinement, noted, not shipped. **[Head chef: approved 2026-07-13.]**
2. **Permit matching is subset-COVER, not literal `⊑`** (§2.4.1 sketches "the use must be ⊑ the
   permit's approved subset"): a permit stores `(class, pattern)` entries and a use is honored iff
   an entry of the same class matches the use's token. `*` is not representable in the `Authority`
   lattice, so a literal `⊑` against an `Authority` value cannot express "approve all declassify";
   the cover check preserves the property that matters — the use is within what was approved, on
   the permit's node, and a permit for `declassify:foo` never honors `declassify:bar` (witnessed
   by `guard::tests::a_permit_covers_only_its_own_subset`). **[Head chef: approved 2026-07-13.]**
3. **Delegating a guarded slice requires the owner code at the CLI** (`grants delegate --owner`,
   or `DELULU_GUARD_OWNER`): §2.4.3's mint gate means the default-guarded classes (declassify /
   foreign) cannot be handed to an agent without the principal signing off. Intended behavior,
   stated here because it changes the chunk-5 `grants delegate` UX for those classes
   (Read/Write-only delegations are unaffected). **[Head chef: approved 2026-07-13, with the
   requirement — implemented — that the refusal names both ways forward: `--owner` for the
   principal, `delulu guard request` for an agent (`Denial::GuardMintBlocked`).]**
4. **`warn`-tier rules route their class's epoch ops synchronously too** (the wire's
   `guarded_classes` routing set carries every RULED class, any tier): a cached snapshot can
   neither emit the mandated `guard_warn` audit event nor surface the agent's one-line warning.
   §2.4.2 pins synchronous validation down only for guarded/sealed; extending it to `warn` is the
   only way §2.3's warn semantics can hold on epoch axes. Ungated classes keep epoch caching
   unchanged (criterion 11 intact).
5. **The bypass banner and awareness lines print to stderr even under `--json`** — stdout stays
   the machine surface, decorations ride stderr (the §2.7 dcg robot-mode convention, matching the
   codebase's existing envelope discipline).
6. **The detached daemon's owner-code handoff is via the child's environment**
   (`DELULU_BROKER_OWNER_CODE_INTERNAL`): the parent mints the code, prints it once to the
   principal's terminal, and hands it to the child in memory — never on disk, and never into
   `<state>/broker.log` (the child's serve loop does not reprint it). Criterion 9's never-on-disk
   half is witnessed by a recursive state-dir grep in `guard_cli.rs`.
7. **Guard events replace the plain `"use"` audit record for gated uses** (one event per use,
   invariant 26 preserved): a permit-honored use records `guard_permit_use`, a bypassed use
   `guard_bypassed_use`, a warn-tier use `guard_warn`, a refusal `guard_block`; ungated uses are
   unchanged. The `expose` path mirrors this (`Broker::expose_guarded`), so a delegated
   declassification is gated exactly like a synchronous `check`.
8. **The `secret` and `foreign_python` classes gate at MINT time only in v0.5**: neither has a
   per-use broker op on this wire (`secret` bytes cross via the already-guarded `declassify`;
   python import checks are runtime-local), so a rule on them bites when authority is delegated,
   not per use. The classes stay in the §2.3 vocabulary for completeness and forward
   compatibility.

## 8. Close-out table (2026-07-14) — for head-chef live re-verification

Workspace tests 372 (chunk-5 baseline) → **398** (+26; the 2 pre-existing ignored unchanged). All
test names are real and runnable; the e2e is `crates/delulu/tests/guard_e2e.rs`.

| # | Criterion (§4) | Status | Witnessing test |
|---|---|---|---|
| 1 | delegated guarded use, no permit → DL1410 naming the exact request command; nothing executes | **met** | `guard_e2e.rs::guard_block_request_approve_deny_bypass_seal_orchestration_end_to_end` (exact command + no file written); unit `guard::tests::delegated_guarded_use_without_permit_is_dl1410`; wire `brokerd::tests::guard_c1_c2_c3_delegated_blocks_root_passes_owner_mints` |
| 2 | root use passes with no guard interaction (position, not identity) | **met** | `guard::tests::root_use_of_guarded_authority_passes_without_interaction`; wire form in `guard_c1_c2_c3…`; grep: the chunk-1 `holder_neutrality.rs` mechanical test runs over `guard.rs` too (no `.kind` access in it at all) |
| 3 | Delegate/Attenuate cannot mint guarded authority without permit/owner code; refusal carries minting context | **met** | `guard::tests::mint_of_guarded_authority_needs_owner_code`; wire `guard_c1_c2_c3…` (`GuardMintBlocked` names both ways forward) |
| 4 | request → approve → retry succeeds in TTL; one audit chain shows block → request(why) → approve(comment) → permit use; `audit verify` green | **met** | `guard_e2e.rs` (CLI arc + `audit verify` + per-action `audit query`); wire `brokerd::tests::guard_c4_c5_request_approve_deny_and_audit_verifies`; unit `guard::tests::request_approve_and_deny_flow` |
| 5 | request → deny → retry refuses DL1412 carrying the deny comment verbatim | **met** | `guard_e2e.rs` (comment asserted verbatim on a real lease run); wire `guard_c4_c5…`; unit `request_approve_and_deny_flow` |
| 6 | `sealed` refuses DL1413 with an approved permit pending/minted, and under bypass | **met** | `guard_e2e.rs` (seal under live bypass); wire `brokerd::tests::guard_c6_sealed_refuses_with_permit_and_under_bypass`; units `sealed_use_is_dl1413_even_under_bypass`, `sealed_refuses_even_with_an_approved_permit` |
| 7 | bypass (start flag + runtime toggle) prints the exact banner; guarded uses proceed with per-rule agent warnings + `guard_bypassed_use` events; audit verify green | **met** | `guard_e2e.rs` (runtime toggle: banner, per-rule warning, event, verify) + `guard_e2e.rs::broker_start_dangerously_bypass_guard_prints_the_banner` (start flag) |
| 8 | every `run --lease` prints the guard status line before user output, `on` and `BYPASSED`; `guard status --json` valid machine JSON | **met** | `guard_e2e.rs` (both states asserted on real lease runs); `guard_cli.rs` + `broker_start_dangerously…` (`--json` parsed as JSON) |
| 9 | admin verbs without a valid owner code → DL1414; the code appears exactly once in start output and never on disk | **met** | `guard_cli.rs::guard_cli_owner_code_once_never_on_disk_and_policy_admin` (exactly-once count + recursive state-dir content grep after stop); wire `brokerd::tests::guard_c9_admin_verbs_need_the_owner_code`; unit `owner_gated_admin_verbs_refuse_dl1414_without_the_code` |
| 10 | fail closed: corrupt guard policy store refuses guarded classes; ungated unaffected | **met** | `brokerd::tests::guard_c10_corrupt_store_is_fail_closed` (the store is corrupted on purpose before the daemon starts); unit `poisoned_policy_refuses_guarded_but_not_ungated` |
| 11 | guarded ops validate synchronously under `--epoch-ms`; ungated ops keep epoch caching (existing epoch tests unchanged); the policy-change bound sentence verbatim where revocation's is | **met** | `brokerd::tests::guard_c11_guarded_epoch_op_validates_synchronously`; every chunk-1/3/5 epoch test runs UNMODIFIED; bound: `codes::tests::guard_policy_bound_reuses_the_revocation_pattern` (byte-suffix assertion against `REVOCATION_BOUND`) + stated at every `guard policy set|unset` (asserted in `guard_e2e.rs`) |
