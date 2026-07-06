# Stage 6 Playbook — "Live" (runtime plugins, DIR, `.dpx`)

**Companion to:** `docs/design/STAGE6_SPECIFICATION.md` (normative). This file is *how to build it*.
**Depends on:** Stage 5 complete (a plugin's grant is a **child node in the broker's grant tree** —
nothing here works without `⊑`-checked attenuation and transitive revocation) and Stage 3 (the WASM
engine is mandatory for Contained plugins).

> **The one-sentence goal:** ship code that arrives *after* compile time and *still cannot exceed its
> grant* — the demo that sells the whole language (Constitution §4, Possibility 2). Two classes:
> **Verified** (ships typed IR, re-checked at load → compile-time-grade, per-function rows) and
> **Contained** (opaque WASM → module-granularity containment, honestly typed at the *full grant row*
> per audit R-1). The type system keeps them honest *by construction*: a Contained plugin's
> "read-only" export **types as everything its module was granted**.

---

## 0. Orientation — read these before writing anything

- Spec §1 invariants 28–32 (grant is a child node; class is never inferred; rows never overclaim
  across the load boundary; no plugin reaches the broker; resource exhaustion is contained).
- Spec §3.1 (the **load sequence**, in order — this is the spine of the stage; build it as a literal
  ordered pipeline where each step is its own testable function).
- `SOUNDNESS_AUDIT.md` R-1 (Contained exports typed at full grant row), R-6a/b/c (no fn-args to
  Contained exports; host values invalidated on return; unload/GrantId binding), R-7 (attenuation),
  R-Get (the `p.get[F]` row check per class).
- Stage-1 spec §8 — the plugin design was *fixed* there and is *implemented* here unchanged in shape.
  Re-read it; do not redesign.
- The already-reserved diagnostics DL0801/DL0803 (Stage 1) **activate for real** this stage.

### The central idea: DIR is "re-check, don't re-infer"

The Verified guarantee rests entirely on **DIR** (the Delulu typed IR — the post-check typed AST,
canonically serialized). The loader **replays the checking pass**: every node already carries its
resolved `DefId`, type, and row, so the loader *asserts and verifies* them rather than inferring —
O(nodes), no name resolution. That is what makes re-verification cheap enough to do at every load.
**DIR contains no machine code.** Verified plugins are re-verified *source-grade semantics*, then run
by the host's own engine. Internalize this before touching the format: DIR is a serialization of what
`delulu-check` already produces, plus the primitive-table version it was checked against.

---

## 1. Suggested crate topology

- **DIR lives in `delulu-check`** (or a thin `delulu-dir` beside it): it *is* the checker's output
  types, so serialize/deserialize belongs where those types are defined. Provide
  `dir::serialize(&CheckResult) -> Vec<u8>` (versioned CBOR) and `dir::verify(bytes) -> Result<...>`
  that replays the checking assertions. **`verify` must reuse the exact same rule code as
  `check_source`** — never a second, drifting implementation (that is how you guarantee "verify gives
  identical verdicts to real loads", criterion 9).
- **The loader lives in `delulu-runtime`** (interpreter host) and calls into `delulu-wasm` for the
  Contained path and the Verified-on-WASM path. It calls the Stage-5 `Custody`/broker for the child-node
  attenuation (step 4 of the sequence).
- **`.dpx` container reuses `delulu-wasm`'s `.dwx` custom-section machinery** (Stage 3, Phase 3g) —
  same technique, different sections (`delulu:plugin`, `delulu:dir`, `delulu:wasm`, `delulu:sig`,
  `delulu:lock`).

---

## 2. Phase plan (each phase: build green → test green → update spec §"status" → commit)

### Phase 6a — DIR serialize/deserialize round-trip
Define the versioned-CBOR DIR format (spec §2.3): items, bodies, each node's `DefId`/type/row, the
package interface metadata (Stage-2 §5.5), and the primitive-table version. Implement serialize +
deserialize. **No loading yet** — just prove a checked module round-trips through DIR losslessly.
*Test:* for every conformance/accept program, `check → serialize → deserialize` yields a structure
equal to the original `CheckResult`; DIR version bump → DL1503 on mismatch.

### Phase 6b — DIR re-verification (the Verified guarantee)
`dir::verify(bytes)` replays the checking pass: assert every node's stored type/row, then verify them
against the rules (types, rows, all R-rules, opacity — the whole Stage-1 §6 plus Stage-2 visibility).
A DIR that fails *any* check is refused (DL1504) — **never falls back to Contained** (invariant 29).
*Test:* a valid DIR verifies; flip one byte in a DIR body → DL1504; a DIR whose stored row is *narrower*
than the code actually needs → DL1504 (the re-check catches the lie). Reuse the checker's rule code —
add a test asserting `dir::verify` and `check_source` agree on a corpus of tricky programs.

### Phase 6c — the `.dpx` container + manifest + `delulu plugin build`
`kind = "plugin"` packages (spec §2.1): must expose **no `fn main`**; `[plugin]` (api, class);
`[plugin.authority]` (the hard ceiling); `[plugin.exports]` (signature strings parsed with the
ordinary type grammar). `delulu plugin build` emits `summarize.dpx` with the right sections per class
(§2.2 table). Export signature strings **must match the code** (DL1501; the manifest never overrides
the code). `delulu plugin inspect` dumps manifest/class/exports/hashes.
*Test:* build a Verified and a Contained `.dpx`; `inspect` shows correct sections; a manifest export
string that disagrees with the code → DL1501.

### Phase 6d — the load sequence, steps 1–4 (container → class → ceiling → holder node)
Implement `load[C](host, path, grant)` steps 1–4 (spec §3.1) as an ordered pipeline:
(1) read + validate container + `plugin.api` (DL1507); (2) class-vs-`C` check (invariant 29);
(3) **manifest ceiling** `grant ⊑ plugin.authority` — the host may grant *less* than the ceiling,
never more (DL1502, intersection as exact repair, `authority_widening: false`); (4) **broker
`attenuate(host_node, grant)`** → `grant ⊑ holder` (DL0802) → child node + fresh `GrantId` (R-6c
binding). Stop before instantiation.
*Test:* wrong class → refuse (no silent fallback); grant exceeding ceiling → DL1502 with intersection;
grant exceeding the *holder's* node → DL0802 (this is the Stage-5 tree doing its job).

### Phase 6e — Verified verification + instantiation (interpreter host)
Step 5-Verified: replay-check the DIR in full; check every export's verified row ⊆ its manifest
string; failure → DL1504, **node revoked, nothing instantiated**. Instantiate on the interpreter
(interpret the DIR). Implement `p.get[F]` with the R-Get check: Verified requires `row(export) ⊆
row(F)` with exact types. Calls thread capability args as attenuations under the plugin's node;
host values obey R-6b (invalidated on return — a call-scoped handle table).
*Test (criterion 1 + 2):* the flagship — a text-transform plugin loads with `Grant { effects: [] }`,
`p.get[fn(Str)->Str ! {}]` succeeds, host row unchanged, plugin cannot read/clock/net; the audit F-1
`evil.wasm` scenario refused by R-Get.

### Phase 6f — Contained verification + WASM instantiation + limits
Step 5-Contained: validate WASM imports ⊆ the `delulu:cap` slice derivable from `grant` (anything
else → DL1505); **no internal verification**; **every export's row is set to `effects(grant)`**
(R-1). One Wasmtime instance per load with store-level **fuel** (`limits.fuel`), **memory cap**
(`limits.mem_mb`), and a host-side **wall-clock watchdog** (`limits.wall_ms`) → trap → DL1506 as
`LimitExceeded`, instance dropped, **node revoked** (a limit-killed plugin is *gone*, not wounded).
`p.get` on Contained: `effects(grant) ⊆ row(F)` and scalar/Str/Cap-parameter match; a **function-typed
parameter anywhere in `F` → DL0803 at the get call site, compile-time** (R-6a).
*Test (criterion 5, 7):* infinite-loop Contained plugin dies at fuel/wall_ms (DL1506), host continues,
node revoked; memory bomb dies at mem_mb; a Contained `get` with a fn-typed param → DL0803 at compile time.

### Phase 6g — unload / reload (R-6c mechanics)
`p.unload()` revokes the plugin's node (broker; transitive if it loaded sub-plugins). Retained
function values fail their next call: **DL0801 carrying the revoking audit seq** (Stage-5 style).
Reload = new node, new `GrantId`, new values; old references dead forever.
*Test (criterion 3):* unload → retained ref DL0801 with seq; reload → old ref still DL0801, new handle
works; `delulu grants tree` shows old node revoked, new one live.

### Phase 6h — Verified-on-WASM + isolation composition + signatures
Verified plugins on a WASM host: compile DIR → module (own instance, shared host tables, same limits
as 6f). Under `--isolation microvm` (Stage 5), plugins run *inside the same guest* as the host — the
broker tree still separates authority (per-plugin microVM is post-1.0). `delulu:sig` (ed25519 over
`plugin ‖ dir` / `plugin ‖ wasm`): verify if present, record identity in the audit log; `grant.require_signed`
refuses unsigned (default false in v0.6).
*Test (criterion 6, 8):* flip a byte in DIR → DL1504; flip a byte in the wasm *cache* section →
ignored, recompiled from DIR, load succeeds (§3.2 — verified guarantee never rests on shipped machine
code); a signed plugin's identity lands in audit; `require_signed: true` refuses an unsigned plugin.

### Phase 6i — `std.plugin` in-language surface + `delulu plugin verify` + authority report
Ship the `std.plugin` records (spec §4): `Grant`, `Limits`, `PluginErr` (ordinary records — they
*describe* authority, don't confer it; conferral is only at `load` under the holder check).
`root.plugin_host()` joins the primitive table (granted via manifest `plugins = true` + `--grant
plugins`). `delulu plugin verify` runs load-sequence steps 1,2,5 **without instantiating** and must
give **identical verdicts to real loads** across the whole corpus (criterion 9). Extend `delulu
authority` with the `plugins` array (§6) and make `delulu why` traverse Verified DIR (real chains) and
stop at Contained boundaries with a labeled edge (`→ [contained plugin X] — Net`).

---

## 3. The traps

1. **Class is never inferred, and Verified never silently falls back to Contained.** A `.dpx`
   claiming Verified whose DIR fails any check is *refused* (DL1504), full stop (invariant 29). This
   is a security property — a bug here defeats the stage.
2. **R-1 is the honesty keystone.** A Contained plugin's exports type at `effects(grant)`, not at
   whatever the export "looks like." The audit F-1 program exists precisely to catch a backend that
   types a Contained export more narrowly than its grant. Wire that test first.
3. **`dir::verify` must reuse the checker's rule code.** Two implementations *will* drift, and the
   drift is exactly a soundness hole. Criterion 9 (verify ≡ load) is the guardrail — make it a
   corpus test, not a spot check.
4. **No plugin reaches the broker** (invariant 31). Plugins get capability *values* passed by the
   host (attenuations under their node) — never lease tokens, never the IPC path. The operations to
   reach the broker simply do not exist in the plugin's world (same shape as Stage-5 invariant 25).
5. **A limit-killed plugin is gone, not wounded.** DL1506 drops the instance *and* revokes the node —
   deterministic for the host. Do not try to "resume" a fuel-exhausted plugin.
6. **Interpreter limits are best-effort, and you say so.** `fuel` → step counter, `mem_mb` →
   allocator accounting on the interpreter (spec §5.4). Hostile plugins belong on the WASM engine —
   which Contained plugins always use by construction. Label this honestly in docs; do not claim the
   interpreter contains hostile code.
7. **Signatures authenticate origin, not behavior** (spec §10). A signed plugin is not a safe plugin.
   Copy §10 caveats verbatim into docs and explain-text.
8. **Exports are functions only in v1.0.** Plugins do not export types or effects (RFC-gated). Don't
   build it; reject it cleanly.

---

## 4. Definition of done (map to spec §9 acceptance criteria)

Ship when all 11 spec criteria pass — most importantly criterion 1 (the flagship demo: a
zero-authority plugin that provably cannot read/clock/net, refused at load if it tries), criterion 2
(audit F-1), and criterion 9 (verify ≡ load across the corpus). Full prior conformance suites stay
green on **both engines** and **both custody modes** (embedded + daemon), criterion 11. Add
`## Implementation status` to `STAGE6_SPECIFICATION.md` logging each phase, per the Stage-3 §8a pattern.

*Stage 6 is the promise kept: code that arrives at runtime and still cannot exceed its grant. Stage 7
makes the language concurrent without surrendering one word of that.*
