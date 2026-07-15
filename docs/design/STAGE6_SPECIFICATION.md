# DeluluLang — Stage 6 Implementation Specification

**Version:** 0.6 ("Live"). **Status:** Committed — buildable directly from this document.
**Depends on:** Stage 5 complete (plugin grants are children in the broker's grant tree; nothing
here works without `⊑`-checked attenuation and transitive revocation). Stage 3 (WASM engine)
required for Contained plugins.
**Governing documents:** `CONSTITUTION.md` (§6), `SOUNDNESS_AUDIT.md` (R-1, R-6a/b/c, R-7,
R-Get), Stage-1 spec §8 (the design fixed there is implemented here, unchanged in shape).

---

## 0. Scope and goal

**Goal:** runtime-loadable, runtime-unloadable, **authority-bounded plugins** — code that arrives
*after* compile time and still cannot exceed its grant. This is Constitution §4 Possibility 2
shipped: the demo that sells the language. Both plugin classes land: `Plugin[Verified]`
(re-checked typed IR — compile-time-grade, per-function) and `Plugin[Contained]` (opaque WASM —
module-granularity containment, honestly typed at full grant row per audit R-1).

**In scope (must ship):**
- The `.dpx` plugin artifact (both classes) and `kind = "plugin"` packages.
- **DIR** — the Delulu typed IR — serialization of checked programs for Verified loading.
- The loader: manifest check → class-specific verification → grant attenuation (broker child
  node) → instantiation; `Load` effect and `Cap[PluginHost]` activate.
- `p.get[F]` (R-Get), calls, `unload` (R-6c), resource limits as grant data.
- In-language `Grant` construction (`std.plugin`).
- CLI: `delulu plugin build|inspect|verify`; authority report extension.
- New diagnostics: DL15xx (DL0801/DL0803 activate).

**Non-goals (later stages/post-1.0):** plugin *registry* distribution and signing policy
enforcement (Stage 8/9 — signature *verification* mechanics ship now, §3.3, but trust policy is
governance); plugins exporting *types* or *effects* (exports are functions only in v1.0 —
declared-effect plugins would complicate row identity across load boundaries; RFC-gated);
cross-plugin direct calls (compose in the host); actor-hosted plugins (Stage 7 note in §9).

---

## 1. Invariants (carried + new)

All prior invariants hold. New:

28. **A plugin's grant is a child node.** Every load creates a broker grant node
    `⊑ the loading holder's node` (R-7; DL0802 on violation). Unload revokes it; host revocation
    cascades into every plugin it loaded (Stage-5 invariant 24 applies unchanged).
29. **Class is never inferred.** The artifact declares Verified or Contained; the loader verifies
    the declaration (DIR present and re-checkable vs. opaque WASM); a `.dpx` claiming Verified
    whose DIR fails any check is refused (DL1504) — it never "falls back" to Contained silently.
30. **Rows never overclaim across the load boundary.** Verified exports carry their re-verified
    per-function rows; Contained exports are typed at `effects(grant)` (R-1) — enforced in
    `p.get` (R-Get), tested by the audit F-1 program.
31. **No plugin reaches the broker.** Plugins receive capability values passed by the host
    (attenuations under their node) — never lease tokens, never the IPC path (Stage-5
    invariant 25 shape: the operations do not exist in the plugin's world).
32. **Resource exhaustion is contained.** A Contained plugin exceeding its granted fuel/memory/
    wall-clock is terminated without harming the host (DL1506 as a `PluginErr`); Verified plugins
    get the same limits when run on the WASM engine (and best-effort depth/step limits on the
    interpreter, honestly labeled — §5.4).

---

## 2. Artifacts and manifests

### 2.1 Plugin package

```toml
[package]
name = "summarize"
version = "0.1.0"
kind = "plugin"                      # NEW package kind; must expose no fn main

[plugin]
api = 1                              # plugin ABI version (independent of language version)
class = "verified"                   # "verified" | "contained"

[plugin.authority]                   # hard ceiling, as fixed in Stage-1 §8.1
effects  = ["Read"]
requires = ["Cap[FsRead]"]

[plugin.exports]
summarize = "fn(Cap[FsRead], Str) -> Result[Str, IoErr] ! {Read}"
```

`delulu plugin build` → `summarize.dpx`. Export signature strings are parsed with the ordinary
type grammar and must match the code (DL1501 on any mismatch — the manifest never overrides the
code).

### 2.2 `.dpx` layout

A `.dpx` is a container (same custom-section technique as `.dwx`):

| Section | Verified | Contained |
|---|---|---|
| `delulu:plugin` | manifest JSON (above) | manifest JSON |
| `delulu:dir` | **DIR payload** (§3) | absent |
| `delulu:wasm` | optional precompiled module (cache; never trusted — §3.2) | the opaque module (the artifact itself) |
| `delulu:sig` | optional ed25519 signature over (plugin ‖ dir) | optional, over (plugin ‖ wasm) |
| `delulu:lock` | build lockfile (provenance) | absent/optional |

### 2.3 DIR — the Delulu typed IR (normative format)

DIR is the **post-check typed AST**, canonically serialized (versioned CBOR): items, bodies,
every node's resolved `DefId`, type, and row; the package's interface metadata (Stage-2 §5.5);
and the primitive-table version it was checked against. DIR is designed for **cheap, complete
re-verification**: the loader replays the *checking* pass (types and rows are asserted, then
verified — not inferred), which is O(nodes) and requires no name resolution. DIR version
mismatch: DL1503. DIR deliberately contains no machine code — Verified plugins are re-verified
*source-grade semantics*, then executed by the host's own engines.

---

## 3. The loader

### 3.1 Load sequence (normative, in order)

```
load[C](host: Cap[PluginHost], path: Str, grant: Grant) -> Result[Plugin[C], PluginErr] ! {Load, Read}
```

1. Read artifact; validate container + `plugin.api` (DL1507 on unsupported).
2. Class check against `C` (invariant 29).
3. **Manifest ceiling:** `grant ⊑ plugin.authority` — the *host* may grant less than the ceiling,
   never more (DL1502-class refusal with the intersection as exact repair, `authority_widening:
   false` — narrowing).
4. **Holder check:** broker `attenuate(host's node, grant)` — `grant ⊑ holder` (DL0802). On
   success: child node + fresh `GrantId` (R-6c binding).
5. Class-specific verification:
   - **Verified:** replay-check the DIR in full (types, rows, R-rules, opacity — the whole §6 of
     Stage 1 plus Stage-2 visibility); check every export's verified row ⊆ its manifest string;
     failure DL1504, node revoked, nothing instantiated.
   - **Contained:** validate WASM imports ⊆ the `delulu:cap` slice derivable from `grant`
     (anything else: DL1505-refusal); no verification of internals — and **every export's type
     row is set to `effects(grant)`** (R-1).
6. If `delulu:sig` present: verify signature; record identity in the audit log. Policy on
   *unsigned* plugins is a host decision: `load` takes `grant.require_signed: Bool` (default
   false in v0.6; governance may flip defaults in Stage 9).
7. Instantiate (§5); return handle bound to the `GrantId`.

### 3.2 Verified execution never trusts caches

A `delulu:wasm` section in a Verified `.dpx` is a compilation cache: the loader uses it only if
its hash matches a local recompilation-or-receipt; otherwise it recompiles from DIR. The verified
*guarantee* never rests on shipped machine code.

### 3.3 `p.get` and calls

```
p.get[F](name: Str) -> Result[F, PluginErr]     // F must be a fn type; R-Get check per class
```

Verified: `row(export) ⊆ row(F)` with exact types. Contained: `effects(grant) ⊆ row(F)` and
scalar/Str/`Cap`-parameter signature match (function-typed parameters anywhere in `F`: **DL0803**
at the `get` call site — compile-time, per audit R-6a). Calls thread capability arguments as
attenuations under the plugin's node; host values obey R-6b (invalidated on return — enforced by
the call-scoped handle table). Effects of the call are already in the caller's row via `row(F)`
(T-Call, nothing new — that is the design paying off).

### 3.4 Unload / reload (R-6c mechanics)

`p.unload() -> Unit` revokes the plugin's node (broker; transitive if the plugin loaded plugins).
Retained function values fail their next call: **DL0801** carrying the revoking audit seq
(Stage-5 style). Re-loading the same artifact = new node, new `GrantId`, new values; old
references stay dead forever. Tested by the audit F-6c programs.

---

## 4. In-language surface (`std.plugin`)

```delulu
type Grant {                       // constructed by the HOST in code; values are ⊑-checked at load
  effects: List[Str],
  fs_read: List[Str], fs_write: List[Str], net: List[Str],
  secrets: List[Str], declassify: List[Str],
  limits: Limits,
  require_signed: Bool,
}
type Limits { fuel: Int, mem_mb: Int, wall_ms: Int }     // 0 = broker/profile default
type PluginErr = NotGranted(Str) | VerifyFailed(Str) | BadArtifact(Str)
               | Revoked(Int) | LimitExceeded(Str) | ApiMismatch(Str)
```

`Grant`/`Limits` are ordinary records (not opaque — they *describe* authority, they don't confer
it; conferral happens only at `load` under the holder check). `root.plugin_host()` joins the
Stage-1 primitive table (pure derivation; granted via manifest `plugins = true` +
`--grant plugins`).

---

## 5. Execution engines

### 5.1 Contained

One Wasmtime instance per load: store-level fuel metering (`limits.fuel`), memory cap
(`limits.mem_mb`), host-side wall-clock watchdog (`limits.wall_ms`) → trap → DL1506 as
`LimitExceeded`, instance dropped, node revoked (a limit-killed plugin is *gone*, not wounded —
deterministic for the host).

### 5.2 Verified

Executed by the host's current engine: interpreter hosts interpret the DIR; WASM hosts compile
DIR → module (own instance, shared host tables, same limits machinery as §5.1).

### 5.3 Isolation composition

Under `--isolation microvm` (Stage 5), plugins run inside the same guest as the host program —
the *broker tree* still separates their authority. A per-plugin microVM is post-1.0 (§9 honesty).

### 5.4 Interpreter limits honesty

On the interpreter, `fuel` maps to a step counter and `mem_mb` is best-effort (Rust allocator
accounting): stated in docs; the WASM engine is the enforcement-grade path for hostile plugins —
which Contained plugins always use by construction.

---

## 6. CLI additions

```
delulu plugin build  [--sign keyfile] [-o out.dpx]
delulu plugin inspect <file.dpx> [--json]     # manifest, class, exports, sig identity, DIR/wasm hashes
delulu plugin verify  <file.dpx> [--json]     # runs load-sequence steps 1,2,5 without instantiating
delulu authority …                            # report gains "plugins" (see below)
```

```json
"plugins": [
  { "name": "summarize", "class": "verified", "grant": { "effects": ["Read"], "scopes": {"fs.read": ["./docs"]} },
    "exports_used": ["summarize"], "loaded_at": [ {"file": "src/main.delulu", "line": 30} ], "signed_by": null },
  { "name": "opaque-tool", "class": "contained", "grant": { "effects": ["Read","Net"], … },
    "note": "module-granularity containment; exports typed at full grant row" }
]
```

`delulu why Net` traverses into Verified plugin DIR (real chains) and stops at Contained
boundaries with a labeled edge (`→ [contained plugin opaque-tool] — Net`).

## 7. Diagnostics (fresh range DL15xx; DL0801/DL0803 activate)

| Code | Meaning | Repair |
|---|---|---|
| DL0801 | call through revoked plugin reference (carries audit seq) | none — `requires_human: true` |
| DL0803 | function-typed value in a Contained export signature at `get` | none — rule R-6a |
| DL1501 | manifest/export string does not match plugin code | regenerate manifest — exact |
| DL1502 | requested grant exceeds plugin's declared ceiling | intersect — exact, narrowing |
| DL1503 | DIR version unsupported | rebuild plugin — exact |
| DL1504 | Verified re-check failed (plugin code unsound/stale) | none — `requires_human: true`; never falls back to Contained |
| DL1505 | Contained module imports outside its grant slice | none — `requires_human: true` |
| DL1506 | resource limit exceeded (runtime; plugin terminated + revoked) | raise limits — `authority_widening: true` |
| DL1507 | plugin API version mismatch | rebuild — exact |

## 8. Standard library additions

`std.plugin` (§4) only.

## 9. Acceptance criteria

1. **The flagship demo (Constitution §4 Possibility 2):** a third-party text-transform plugin
   loads with `Grant { effects: [], … }`; `p.get[fn(Str) -> Str ! {}]` succeeds; the host's row
   is unchanged; the plugin demonstrably cannot read a file, tell time, or reach the network —
   attempts in a rigged variant are refused at load (Verified: DL1504-class row violation;
   Contained: DL1505 import refusal).
2. **Audit F-1 end-to-end:** the `evil.wasm` scenario — `p.get[fn() -> Str ! {Read}]` under a
   `{Read, Net}` grant is refused (R-Get requires `{Read, Net} ⊆ row(F)`); with the honest
   annotation the program compiles only when `main`'s row includes `Net`; the runtime WASI slice
   still blocks any host not in the grant.
3. **R-6c:** unload → retained reference DL0801 with audit seq; reload → old reference still
   DL0801, new handle works; `delulu grants tree` shows the old node `revoked`, the new one live.
4. **R-7 composition:** a plugin granted `{Read}` loading its own sub-plugin with `{Read, Net}`
   → DL0802 at the broker; sub-plugin with `{Read}` succeeds and host-revocation kills all three
   levels transitively.
5. **Limits:** an infinite-loop Contained plugin dies at `fuel`/`wall_ms` with DL1506; host
   continues; node revoked; memory bomb dies at `mem_mb`.
6. Verified re-check catches tampering: flip one byte in a `.dpx` DIR body → DL1504 at load;
   flip one byte in the wasm cache section → cache ignored, recompiled from DIR, load succeeds.
7. `get` with a function-typed parameter in `F` on a Contained plugin → DL0803 at compile time.
8. Signature: `--sign`ed plugin verifies and its identity lands in the audit log;
   `require_signed: true` grant refuses an unsigned plugin.
9. `delulu plugin verify` gives identical verdicts to real loads across the whole plugin
   conformance corpus (no verify/load divergence).
10. `delulu why` chain traverses a Verified plugin to the primitive op and labels a Contained
    boundary correctly.
11. Full prior conformance suites still green on both engines, embedded and daemon custody.

## 10. Honesty and threat-model caveats (carry into docs verbatim)

- The Verified/Contained split is a **trust statement, not a quality ranking**: Verified =
  re-proved per-function at load; Contained = confined at module boundary. The type system keeps
  them honest by construction (R-1) — a Contained plugin's "read-only" export *types as*
  everything its module was granted.
- Signatures authenticate **origin**, not behavior; a signed plugin is not a safe plugin.
- Resource limits bound CPU/memory/wall-clock, not I/O volume within granted scopes (an
  I/O-quota grant dimension is a post-1.0 RFC).
- Interpreter-engine limits are best-effort (§5.4); hostile code belongs on the WASM engine.
- Plugins share the host's microVM in v1.0; per-plugin VMs are future work.

*Stage 6 is the promise kept: code that arrives at runtime and still cannot exceed its grant.
Stage 7 makes the language concurrent without surrendering one word of that.*

---

## 11. Implementation status

*(Logged per playbook phase, Stage-3 §8a pattern. Baseline at Stage-6 open: 461 passed / 0 failed /
2 ignored.)*

**Phase 6a — DIR serialize / deserialize round-trip — is implemented and green** (468 tests,
+7). New module `crates/delulu-check/src/dir.rs` defines **DIR** (§2.3): the post-check typed AST
serialized as versioned CBOR (`ciborium`, chosen over `minicbor` for being serde-native — DIR
mirrors the checker's own `serde`-derived types, so the codec is a derive, not a drift-prone
hand-written encoder). The payload carries the source `Module`, the whole-module authority `facts`
and `fn_types`, **every expression/block node's resolved type and effect row** (new side tables
`CheckResult::node_types`/`node_rows`, recorded at a single choke point in `check.rs` as a pure side
effect that changes no typing rule), `main` presence/row, the `foreign` bind sites, and the two
contract versions it was checked against (`DIR_VERSION`, `PRIM_TABLE_VERSION`). All collections are
`BTreeMap`/`BTreeSet`/`Vec` so the encoding is **canonical** — identical checks produce byte-identical
DIR (house rule 8, proven by test). `deserialize` is hostile-input hardened: malformed/truncated
CBOR, an unsupported `DIR_VERSION` or `PRIM_TABLE_VERSION` (**DL1503**), and a structurally
impossible AST (zero-segment name path) all refuse cleanly and **never panic** — witnessed by an
every-single-byte-flip and every-truncation test. Deserialized `DefId`s/`NodeId`s are only ever
compared, never used to index a table, so an out-of-range id can lie but cannot crash. No loading
yet (that is 6b onward); this phase proves a checked module round-trips through DIR losslessly and
byte-stably. Serde derives were added additively to the AST (`delulu-syntax`), `Span`
(`delulu-diag`), and the type-system types (`delulu-check::ty`) — behaviour-preserving; the full
prior suite stays green.

**Phase 6b — DIR re-verification (the Verified guarantee) — is implemented and green** (474 tests,
+6). `dir::verify(bytes)` replays the checking pass by **reusing the exact rule code of
`check_source`**: it reconstructs the module from DIR and re-runs the very same single-module
`resolve` + `check_module` the compiler ran — never a second, drift-prone implementation (playbook
trap 3; this is what makes `plugin verify` ≡ a real load, criterion 9). A DIR is accepted only when
**both** (1) re-checking the code raises no error — so a declared row narrower than the body needs is
caught by the checker's own T-Fn boundary rule (DL0501) — **and** (2) the DIR's stored
facts/types/node-rows equal what re-checking produces — so a validly re-encoded DIR that *lies*
about its authority (a forged narrower row, a forged export type) is refuted by comparison. Every
failure is **DL1504** (`requires_human`), and per invariant 29 it **never** falls back to Contained
(a malformed/truncated DIR verifies as DL1504 too — there is no fallback path to take). Witnessed by:
a narrowed-row program refused via the boundary rule; forged stored facts and a forged export type
each refused via comparison; and a soundness fuzz that flips every single bit of a valid DIR and
proves no flip ever *verifies* with altered authority while running the checker over thousands of
mutated modules without a panic. Because deserialized ids are only compared (never used to index) and
the module is structurally validated at decode, `verify` runs the real checker over hostile input
with a zero panic surface. *(Build-order Deviation 1 records the spec §2.3 "no name resolution / not
inferred" wording vs. this reuse-the-checker choice — **ruled: approved**, conditions (a) docs
honesty and (b) a timing witness attach at the B4 close-out.)*

**Phase 6c — the `.dpx` container, `kind = "plugin"`, and `plugin build`/`inspect` — is implemented
and green** (513 tests, +39). `kind = "plugin"` joins `PackageKind`; `delulu-check/src/plugin.rs`
parses the `[plugin]`/`[plugin.authority]`/`[plugin.exports]` tables (§2.1) and enforces the
manifest-vs-code fence: export signature strings are parsed with the **ordinary type grammar** (new
`delulu_syntax::parse_type_string`, a thin public entry over the existing `parse_type`), lowered
with the checker's **own `lower_type` rule code** (new `check::lower_export_signature`), and
compared for exact equality against the checked function type — any disagreement, in *either*
direction (an undersold row is still a lie), is **DL1501** with the regenerated code-derived
signature as the exact repair (§7), proven round-trip-stable (render → parse → lower ≡ identity).
`fn main` in a plugin package is DL1501; a generic export is DL1501 (exports are monomorphic
functions in v1.0); an unsupported `[plugin] api` is **DL1507**. The `.dpx` container
(`delulu-wasm/src/dpx.rs`) reuses the `.dwx` custom-section machinery (shared ULEB helpers, same
encoding): a wasm-format container carrying `delulu:plugin`/`delulu:dir`/`delulu:wasm`/`delulu:sig`/
`delulu:lock` as custom sections, with **blake3 content bindings** embedded in the manifest JSON.
Binding semantics are class-aware per criterion 6: a tampered `delulu:dir` is **DL1504** (a failed
Verified re-check precondition — never a fallback); a tampered *Contained* module is **DL1508** (a
corrupt artifact — deviation 2 allocates the code, mirroring Stage 3's DL1202); a tampered
*Verified* `delulu:wasm` is **not an error** — it is a cache, flagged `wasm_cache_valid: false` and
recompiled from DIR (§3.2). A Contained artifact smuggling a DIR section, an unknown class, and
duplicate sections are all refused (class is never inferred and never defaulted — invariant 29);
reading survives every truncation and every single-byte corruption without a panic. CLI: `delulu
plugin build <dir> [-o out] [--json]` (all checks run before any byte is written — a refused build
leaves **no partial artifact**) and `delulu plugin inspect <f.dpx> [--json]` (manifest, class,
exports, section hashes, signature identity — through the **same** container read path the loader
will use, §9.9). `plugin build` and `plugin inspect --json` are byte-identical across runs (house
rule 8, proven by test). Plugin packages are single-module in v0.6 (deviation 3 — refused cleanly,
never half-built). The DL15xx range is registered in the code registry.

**Phase 6d — the load sequence, steps 1–4 (container → class → ceiling → holder node) — is
implemented and green** (528 tests, +15). New `crates/delulu-runtime/src/plugin.rs` implements the
load sequence as an **ordered pipeline of individually testable steps**: (1) container + `plugin.api`
(**DL1507**); (2) class check against the requested `C` — refused, never substituted, never inferred
(invariant 29); (3) manifest ceiling `grant ⊑ plugin.authority` (**DL1502**) — reusing the broker's
own `⊑` lattice (`attenuation_check`), so the refusal carries the **intersection** as the exact,
narrowing repair (`authority_widening: false`, never wider than either side by construction); (4)
holder check via broker `attenuate(host's node, grant)` (**DL0802**) → **child node + fresh
`GrantId`** (invariant 28, the R-6c binding). The order is normative and **observable**: a grant
violating *both* the ceiling and the holder reports DL1502 because step 3 precedes step 4 — witnessed
by `ceiling_is_checked_before_the_holder_step_order_matters` (and a second ordering witness proves
step 1 precedes step 2). A refused prepare mints **no node**; an unreadable `[plugin.authority]`
ceiling reads as *empty* — fail closed, never permissive. `std.plugin`'s `Grant`/`Limits`/`PluginErr`
land as runtime data here (the in-language records arrive in 6i).

**Invariant 31 is guaranteed by absence, not refusal.** The `GrantId` minted at step 4 lives
host-side in `PreparedLoad`; it is never a value in the plugin's world. The plugin's vocabulary
holds only `Grant`/`Limits`/`PluginErr` — ordinary records that *describe* authority — plus
capability values the host passes explicitly. No `attenuate`, no `revoke`, no lease token, no IPC
path exists for it, so there is nothing to refuse.

**Custody topology.** The grant tree reached the `Custody` seam: `holder_node`/`attenuate`/
`revoke_node` join the trait with **fail-closed defaults**, implemented by `EmbeddedCustody`
(gaining an optional in-process `delulu_broker::Broker` via `with_root` — Stage 1–5 behaviour is
byte-identical when absent) and by `BrokerClientCustody` (IPC `Attenuate`/`Revoke` against the
daemon's live tree). The loader is therefore written **once** and works in **both custody modes**.
**Crate wiring (playbook §1 + head-chef amendment):** `delulu-wasm` already depends on
`delulu-runtime`, so the loader cannot reach it; instead the container is reified as plain data
(`PluginArtifact`) and the engine as a trait (`PluginEngine`), both declared in `delulu-runtime`,
implemented by `delulu-wasm::WasmPluginEngine`, and injected by the `delulu` crate — no cycle, and
`plugin verify` and a real load share one code path by construction (§9.9).

**Phase 6e — Verified verification, the plugin type surface, R-Get and R-6b — is implemented and
green** (563 tests). Step 5-Verified replays the DIR via `dir::verify` (the same `resolve` +
`check_module` the compiler ran) and checks each export as `verified_row ⊆ manifest_row` with exact
types — deliberately weaker than the build-time DL1501 equality fence, because load-time soundness
only needs "the code cannot exceed what its manifest advertises". Any failure is **DL1504** with the
node **revoked** and nothing instantiated; it never falls back to Contained (invariant 29). The
language surface landed: `Effect::Load` activates (`load` carries `{Load, Read}`), `Type::Plugin[C]`
with nominal `Verified`/`Contained` markers that **never unify** with each other (invariant 29 in the
type system), a plugin handle is **R-5 opaque** (no `str`/`==` — it binds a live broker node), and
`std.plugin`'s `Grant`/`Limits`/`PluginErr` are **ordinary** records/sum (constructing a `Grant` is
pure and confers nothing). Per build-order **deviation 4** (ruled approved), the spec's `load[C]` /
`p.get[F]` brackets are **notation**: the grammar has no turbofish (Stage-1 §6 states the rule
outright), so `[C]`/`[F]` are inference-from-context pinned by the binding's annotation —
`let shout: fn(Str) -> Str ! {} = p.get("shout")?`. R-Get's runtime half is fail-closed throughout,
and **R-1** is enforced as "every Contained export types at `effects(grant)`, whatever the manifest
claims" — the audit F-1 scenario is witnessed at the gate. **R-6b**: a call-scoped handle table
invalidates every host value on return (including on a faulting return); handle ids are **monotonic
and never reused**, because id recycling would let a retained handle *alias* a later value and
**succeed**.

**Security fix (deviation 5, ruled approved): R-6a was failing open; DL1509 closes it.** DL0803 was
decided by `type_contains_fn(F)`, which silently **skipped** when `F` was underdetermined. Two
programs escaped with zero diagnostics — an unpinned `get`, and **generic laundering**
(`fn helper[T](p: Plugin[Contained], x: T) { let f: fn(T) -> Str ! {} = p.get("g")?; f(x) }` called
with a closure): because a generic's variables are instantiated *fresh per call site*, the body's
`T` is never unified with the caller's closure, so a closure reached an opaque module while R-6a
never fired. The fault was **not** "`F` is not a `Fn`" — in the laundering case `F` *is* a `Fn`; it
is the type **variable inside it**. At a Contained `get`, the only accepting case is now a concrete
`Fn` containing no inference variable at any depth; an underdetermined `F` is **DL1509**.

**Phases 6f/6g — Contained import slice, limits, unload/reload — are implemented and green** (576
tests). **DL1505**: a Contained module's imports are validated against the slice *derived from its
grant* — a **whitelist**, never a blacklist. Two properties are invariants: no `root_*` constructor
is in any slice at any grant (a plugin holds no `Root`, so it can never mint a capability —
invariant 31's shape), and **signatures are part of the slice** (matching on name alone would leave
the engine's instantiation type-check as the only gate — refused: step 5 decides it). WASI by the
back door (`wasi_snapshot_preview1.fd_write`) is DL1505 at every grant; zero imports fits every
slice (a module importing nothing reaches nothing — the flagship's shape). **DL1506 (trap 5)**: a
limit-killed plugin is **gone, not wounded** — `kill_on_limit` takes the instance by value, drops
it, and revokes its node in the *same act*; no live node survives it. **R-6c**: `unload` returns the
revoking audit seq that **DL0801** carries; `PluginRef` binds the load-time `GrantId`, so a reload
mints a fresh node and an old reference stays dead **forever**. **Criterion 4** (R-7 composition) is
witnessed through the real loader across three levels: a sub-plugin exceeding its parent is DL0802,
a conforming one loads, and host revocation kills all three transitively.
