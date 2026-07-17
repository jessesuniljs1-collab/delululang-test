# The flagship plugin: a zero-authority text transform

This is the demo that sells the language (Constitution §4, Possibility 2): **code that arrives after
compile time and still cannot exceed its grant.** `shout` is a third-party text transform whose
authority ceiling is *empty*. Because its type is a pure `fn(Str) -> Str` and it holds no capability,
the type system — re-checked at load — forbids it from reading a file, telling the time, or reaching
the network. There is nothing to trust; the guarantee is by construction.

## Build it, verify it, inspect it

```console
$ delulu plugin build examples/plugin_shout -o shout.dpx
ok: wrote `shout.dpx` (…) — verified plugin `shout` v0.1.0, 1 export(s)

$ delulu plugin verify shout.dpx
ok: `shout.dpx` verifies — verified plugin, 1 export(s)

$ delulu plugin inspect shout.dpx
plugin `shout` v0.1.0 — class verified, api 1
  authority ceiling: effects []; requires []
  exports:
    shout: fn(Str) -> Str
  sections:
    delulu:plugin  present
    delulu:dir     blake3 …
  signature: none
```

`plugin verify` runs the load sequence's verification steps (1, 2, 5) **without instantiating**, and
gives *identical verdicts to a real load* (criterion 9): the same container read, the same DIR
re-check. The empty `authority ceiling` is the flagship's point — a host may grant this plugin
nothing at all, and it still runs.

## The rigged variant is refused

`rigged/` ships the same shape but its code *tries to tell the time* while its manifest claims a pure
row. The build refuses — the manifest can never override the code:

```console
$ delulu plugin build examples/plugin_shout/rigged
error[DL1501]: export `shout` signature does not match the plugin code — the manifest never overrides the code
  manifest: fn(Cap[Clock], Str) -> Str ! {}
  code:     fn(Cap[Clock], Str) -> Str ! {Clock}
  repair: regenerate_manifest_export (exact)
```

(If a dishonest DIR were somehow smuggled past the build, a *load* would still refuse it with a
DL1504 row violation — and Verified code never falls back to Contained.)

## Sign it (optional)

Signatures authenticate **origin, not behavior** — a signed plugin is not a safe plugin (spec §10).

```console
$ delulu plugin build examples/plugin_shout -o shout-signed.dpx --sign my.key
ok: wrote `shout-signed.dpx` (…) — verified plugin `shout` v0.1.0, 1 export(s) (signed)

$ delulu plugin inspect shout-signed.dpx
  …
  signature: valid — signed by 79b5562e…
```

`my.key` is a raw 32-byte ed25519 seed (or 64 hex characters). A grant with `require_signed: true`
refuses an *unsigned* plugin (DL1511); a plugin whose signature does not verify is a *different*
fault (DL1510).

## What actually runs it

When a host loads this plugin, it holds `p.get("shout")` as
`let f: fn(Str) -> Str ! {} = p.get("shout")?` (the annotation form — the spec's `p.get[F]` bracket
notation does not parse; see `delulu explain E-PLUGIN`), and calling `f("hello")` returns `"hello!"`
— with the host's own effect row unchanged, because a pure export adds nothing to any caller's row.

For the full model — the two plugin classes, the load sequence, the resource limits, and every
honesty caveat — run `delulu explain E-PLUGIN`.
