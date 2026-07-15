# Stage 6 "Live" — Build Order

**Status:** COOKING — opened 2026-07-15 by the head chef.
**Binding spec:** `STAGE6_SPECIFICATION.md` (v0.6, Committed). This document adds nothing to the
spec's semantics; it fixes the build sequence, house rules, crate placement, and the close-out
discipline. Where this document and the spec disagree, **the spec wins** — record the conflict as
a deviation below instead of silently choosing.

---

## 1. Phases

Build in this order; each phase ends in its own commit with the full suite green.

### B1 — DIR: the Delulu typed IR (spec §2.3)

- Canonical serialization of the post-check typed AST (items, bodies, resolved `DefId`s, types,
  rows, interface metadata, primitive-table version). Versioned CBOR.
- The **replay-checker**: types and rows are asserted then *verified* (never inferred), O(nodes),
  no name resolution. This is the load-time trust anchor for `Plugin[Verified]` — build it as if
  every DIR file is hostile.
- Round-trip property: check → serialize → replay-check must accept byte-identically re-encoded
  DIR and refuse any single-byte body tamper (this becomes the §9.6 witness).
- DL1503 (version mismatch) activates here.

### B2 — The artifact and CLI (spec §2.1, §2.2, §3.3-sig, §6)

- `kind = "plugin"` packages (must expose no `fn main`); `[plugin]` / `[plugin.authority]` /
  `[plugin.exports]` manifest tables; export signature strings parsed with the **ordinary type
  grammar** — DL1501 when the manifest disagrees with the code (the manifest never overrides).
- `.dpx` container: reuse the `.dwx` custom-section technique from `crates/delulu-wasm`
  (sections per spec §2.2 table; blake3 content binding as in `.dwx`).
- `delulu plugin build [--sign keyfile] [-o out.dpx]`, `delulu plugin inspect <f> [--json]`,
  `delulu plugin verify <f> [--json]` — verify runs load steps 1, 2, 5 without instantiating and
  must give **identical verdicts to real loads** (§9.9; build the shared path once, not twice).
- ed25519 signatures over (plugin ‖ dir) / (plugin ‖ wasm). DL1507 activates here.

### B3 — The loader and runtime surface (spec §3, §4, §5.1–§5.3)

- The 7-step load sequence, in the normative order — ceiling check (DL1502, intersection as
  exact narrowing repair), holder check via broker `attenuate` (DL0802, child node, fresh
  `GrantId`), class verification (Verified replay → DL1504 with node revoked and nothing
  instantiated; Contained import-slice validation → DL1505), signature policy
  (`require_signed`), instantiation.
- `std.plugin` (`Grant`, `Limits`, `PluginErr` as ordinary records/sum), `root.plugin_host()` in
  the primitive table, manifest `plugins = true` + `--grant plugins`.
- `p.get[F]` per class (R-Get; **DL0803 at compile time** for function-typed parameters anywhere
  in `F` on a Contained plugin), calls threading capability attenuations under the plugin's node
  with R-6b invalidate-on-return, `p.unload()` (R-6c: DL0801 with the revoking audit seq;
  reload = new node/`GrantId`/values, old references dead forever).
- Invariant 31: plugins never see lease tokens or the IPC path — the operations must not exist
  in the plugin's world, not merely be refused.

### B4 — Limits, surface, docs, close-out (spec §5.1, §5.4, §6-report, §7, §9, §10)

- Wasmtime store fuel / memory cap / wall-clock watchdog → trap → DL1506 as `LimitExceeded`,
  instance dropped, node revoked. Interpreter-engine best-effort limits, honestly labeled per
  §5.4.
- Authority report gains `"plugins"` (spec §6 JSON shape); `delulu why` traverses Verified DIR
  to the primitive op and labels Contained boundaries
  (`→ [contained plugin <name>] — <Effect>`).
- Explain topic `E-PLUGIN`; docs updated (README/ARCHITECTURE as needed); spec §10 honesty
  caveats carried into user docs **verbatim**.
- The acceptance corpus: all 11 §9 criteria, each with a named witnessing test in the close-out
  table (§4 below). The flagship demo (§9.1) ships as a runnable example under `examples/`.

---

## 2. House rules (binding)

1. **Never push to GitHub.** Commit per phase on `master`; `git commit -F <msgfile>`.
2. **Zero pre-existing tests edited.** Baseline 461 passed / 0 failed / 2 ignored stays green on
   both engines after every phase. New behavior gets new tests.
3. **Machine channels are never styled.** `--json` output is byte-stable and never colored; the
   Palette applies only to human channels (Stage-8 early-drop ruling carries).
4. **Refusal honesty.** No partial artifacts on refusal; exact repairs exactly where the spec's
   §7 table says so; `requires_human: true` where it says none. A Verified failure never falls
   back to Contained (invariant 29).
5. **Dependencies.** Pre-approved new deps: one established CBOR crate (`ciborium` or
   `minicbor`) for DIR, and `ed25519-dalek` (v2) for signatures — cryptography is **never**
   hand-rolled. Wasmtime 27 / blake3 / serde_json are already in the workspace — reuse them.
   Any other new dependency: stop and escalate before adding.
6. **Crate placement.** DIR serialization + replay-check live in `delulu-check` (they replay the
   checking pass and need its internals). The `.dpx` container, manifest, signatures, and loader
   live in a new crate `delulu-plugin`. CLI wiring in `delulu`. If the internals argue for a
   different split, record a deviation and say why.
7. **Deviations ledger.** Any departure from the spec or this order is appended to §3 below,
   numbered, with what/why — and awaits a head-chef ruling. If blocked, in doubt, or the same
   error repeats more than twice: stop and report instead of thrashing.
8. **Determinism.** `plugin inspect --json` and `plugin verify --json` are byte-identical across
   runs on identical inputs.

## 3. Deviations

*(appended during the build; numbered; each awaits a head-chef ruling)*

## 4. Close-out

*(filled at the end: the 11 §9 criteria, each with its witnessing test(s); suite totals
461 → N; verification evidence)*
