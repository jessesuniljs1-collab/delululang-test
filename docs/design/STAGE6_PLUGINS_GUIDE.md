# DeluluLang plugins — user guide (Stage 6 "Live")

Runtime plugins are **code that arrives after compile time and still cannot exceed its grant.** This
guide is the user-facing companion to `STAGE6_SPECIFICATION.md`; for the same material inside the
tool, run `delulu explain E-PLUGIN`. A runnable flagship lives in `examples/plugin_shout/`.

## The two classes (a trust statement, not a quality ranking)

- **`Plugin[Verified]`** ships **DIR** (the Delulu typed IR) and is **re-checked at load**: the loader
  replays the compiler's own `resolve` + `check_module` over the shipped module. A Verified export
  carries its re-verified per-function effect row. Verified plugins run on the host's own engine.
- **`Plugin[Contained]`** ships opaque WASM and is **confined at the module boundary**: its imports
  must fit the grant-derived slice (DL1505), and — rule R-1 — *every export types at `effects(grant)`*,
  whatever its manifest claims.

The class is **declared, never inferred**: a `.dpx` claiming Verified whose DIR fails any check is
refused (DL1504) and **never falls back to Contained**.

## The load sequence (an order, spec §3.1)

1. container + `plugin.api` (DL1507) → 2. class check → 3. manifest ceiling `grant ⊑ plugin.authority`
(DL1502, intersection = exact narrowing repair) → 4. holder check `grant ⊑ holder` at the broker
(DL0802) → a child grant node → 5. class-specific verification → 6. signature policy → 7. instantiate.

`delulu plugin verify` runs steps **1, 2, 5** without instantiating and gives **identical verdicts to
a real load** — one code path, not two (this is why there is never a verify/load divergence).

## The CLI

```
delulu plugin build   <package-dir> [-o out.dpx] [--sign keyfile] [--json]
delulu plugin inspect <file.dpx> [--json]     # manifest, class, exports, section hashes, signature
delulu plugin verify  <file.dpx> [--json]     # load steps 1,2,5 — identical verdicts to a real load
delulu authority      <program>  [--json]     # gains a "plugins" array for programs that load plugins
delulu why <Effect>   <file.dpx> [--json]     # traverses a Verified plugin's DIR; labels Contained
```

## Signatures authenticate origin, not behavior

A `delulu:sig` section is a 32-byte ed25519 public key ‖ a 64-byte signature over the canonical
manifest ‖ the class payload. Signing (`--sign`) and verification (load step 6) share one message so
they cannot drift. **A signed plugin is not a safe plugin.** An unsigned plugin under a
`require_signed` grant is refused with **DL1511**; a plugin whose signature does not verify is a
*different* fault, **DL1510** — reusing one code would make a message a lie.

## Honesty and threat-model caveats (spec §10, verbatim)

- The Verified/Contained split is a **trust statement, not a quality ranking**: Verified = re-proved
  per-function at load; Contained = confined at module boundary. The type system keeps them honest by
  construction (R-1) — a Contained plugin's "read-only" export *types as* everything its module was
  granted.
- Signatures authenticate **origin**, not behavior; a signed plugin is not a safe plugin.
- Resource limits bound CPU/memory/wall-clock, not I/O volume within granted scopes (an I/O-quota
  grant dimension is a post-1.0 RFC).
- Interpreter-engine limits are best-effort (§5.4); hostile code belongs on the WASM engine.
- Plugins share the host's microVM in v1.0; per-plugin VMs are future work.

## Ruled implementation notes (build-order deviations)

These are recorded so no reader is surprised; each was ruled during the build.

- **Re-verification is stronger than the spec's sketch (deviation 1).** Spec §2.3 sketches DIR
  re-verification as an assert-only replay with "no name resolution / not inferred." The implementation
  does something **strictly stronger**: it re-runs the compiler's *exact* single-module `resolve` +
  `check_module` and then refuses (DL1504) unless the DIR's stored facts/types/rows **equal** what
  re-checking produces. So a dishonest DIR that lies about its authority is refuted by comparison, and
  the load-time check is byte-for-byte the same code path as the original compile-time check — never a
  drift-prone second implementation. It is comfortably fast for per-load use: re-verifying a realistic
  multi-function plugin measures **well under a millisecond** (~0.75 ms on a dev machine; witnessed by
  `verify_is_comfortably_fast_for_per_load_use`).

- **A corrupt container has its own code (deviation 2).** A malformed or tampered `.dpx` (bad magic,
  truncated sections, an unreadable manifest, a Contained module failing its blake3 binding) is
  **DL1508** — the container-corruption code the spec §7 table lacked, mirroring Stage 3's DL1202. A
  tampered *DIR* body inside an otherwise-valid container is DL1504 (a failed Verified re-check), never
  DL1508.

- **Plugin packages are single-module in v0.6 (deviation 3).** A multi-module plugin package is refused
  cleanly at build with **DL1004** ("plugin packages are single-module in v0.6 — found N module
  file(s)"), with no partial artifact and no fake repair. Multi-module packages are on the post-v0.6
  RFC ledger.

- **`load[C]` / `p.get[F]` are annotation-from-context, not bracket syntax (deviation 4).** The grammar
  has no turbofish (Stage-1 §6), so the spec's `load[C](…)` and `p.get[F](…)` are notation. Write them
  as `let p: Plugin[Verified] = load(host, path, grant)?` and
  `let f: fn(Str) -> Str ! {} = p.get("shout")?`. **Do not copy the bracket form out of the spec — it
  will not parse.** R-6a still fires at the `get` site: a function-typed parameter in a Contained `F` is
  DL0803, and an underdetermined `F` is DL1509.

- **Windows contained execution is refused, not attempted (deviation 7).** In-process CPU/wall
  enforcement for a Contained (or Verified-on-WASM) plugin uses host-initiated wasm traps, whose unwind
  fastfails the host process on Windows with wasmtime 27. So on Windows, contained execution is
  **refused up front with an honest `EnforcementUnsupported`** — before any store exists — rather than
  risk an uncatchable host crash. It is neither a limit hit (never DL1506, never an authority-widening
  repair) nor a plugin fault; it names its reason, and it is never a crash, a hang, or a silent no-op.
  The enforcement-grade platform for hostile Contained code is the WASM engine on Linux (spec §5.4);
  **Verified plugins run on every platform via the interpreter.** Verified platforms are **Windows**
  (with the caveat above) and **Linux**; **macOS** compiles the full enforcement path by construction
  (the gate is `cfg(not(windows))`, and wasmtime treats macOS as a first-class Unix signal-path
  platform) and is *expected* to work, but is **unverified** in this kitchen — no witness has ever run
  on a Mac, and nothing is claimed passing where unrun.

- **Two signature faults, two codes (deviation 8).** A present-but-invalid signature (DL1510) and an
  unsigned-but-required plugin (DL1511) are different faults with different remedies; see above.
