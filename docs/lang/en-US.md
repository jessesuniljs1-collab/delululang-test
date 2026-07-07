# Language Pack — `en-US` (English, United States) — REFERENCE / BASE

**Role:** the base locale. **Complete by construction** — the compiler refuses to build with a
missing `en-US` key; every other locale falls back here. This file is also the **canonical
enumeration** of (a) the DeluluLang concept vocabulary, (b) the catalog key space, and (c) the
canonical keyword list every syntax morph maps from. Companions:
`docs/design/LOCALIZATION_PLUGIN_GUIDE.md`, `docs/design/SYNTAX_MORPH_SPEC.md`.

```toml
[meta]
locale    = "en-US"
name      = "English (US)"
fallback  = "en-US"          # the base falls back to itself
version    = "1.0.0"
coverage  = { cli = 100, diagnostics = 100, explain = 100 }
direction = "ltr"
```

---

## 1. Concept vocabulary (the terms every other pack translates + glosses)

These are the words DeluluLang prose uses for its core ideas. Other packs give their language's term
**plus this English gloss**. Definitions are the source of truth for translators.

| Term | Meaning (the gloss other packs cite) |
|---|---|
| **authority** | everything a unit of code is permitted to do to the system — its effects + capabilities + secrets, computed whole-program |
| **effect** | an observable interaction with the world (Read, Write, Net, Clock, Rand, Declassify, ForeignCall, Load, Async, Actuate); arises only from a capability operation |
| **effect row** | the set of effects in a function's type, written `! {Read, Net}`; omitted = pure `!{}` |
| **capability** (`Cap[R]`) | an unforgeable value that grants exactly one kind of authority (a `Cap[Console]` may print; it can do nothing else) |
| **Root** | the single capability `main` receives; every other capability is derived (attenuated) from it |
| **grant** | authority a holder hands to code at run time (`--grant`), always ⊑ what the holder has |
| **attenuation** (`⊑`) | narrowing authority when passing it on; you may give less, never more |
| **secret** (`Secret[T]`) | an opaque wrapped value; its bytes cross into code only via `expose` (which carries the `Declassify` effect) |
| **capability scope** | the concrete bound on a capability (which paths, which hosts, which envelope) |
| **plugin** | code loaded at run time that still cannot exceed its grant (`Plugin[Verified]` re-checked; `Plugin[Contained]` module-confined) |
| **actor** | an isolated unit of concurrent state, reached only by messages; no shared mutable state, statically |
| **reference capability** (rcap) | who may alias/mutate/send a value: `iso val ref box tag trn` |
| **broker** | the process that holds root authority outside the program; grants form a revocable tree |
| **manifest** | `delulu.toml`; declares a package's authority ceiling |
| **diagnostic** | a compiler message with a stable code (DL0501), spans, and typed repairs |
| **repair** | a machine-applicable fix carried by a diagnostic, flagged `authority_widening` / `requires_human` |

---

## 2. Canonical keyword enumeration (every syntax morph maps FROM this list)

The complete set of DeluluLang keywords, canonical form. A human keyword-morph or an AI compact
profile is a bijection over this list (see `SYNTAX_MORPH_SPEC.md`). Contextual keywords are marked.

```
# declarations / items
module  import  pub  fn  type  effect  const  foreign  actor  test
# bindings / control
let  var  if  else  match  return  while  spawn  consume  recover
# actor-body contextual
be  new  self
# types / rows / capabilities
Int Float Bool Str Unit  Cap Root Secret Result Option List  Plugin
# reference capabilities
iso  trn  ref  val  box  tag
# effect names (not keywords, but reserved concept words)
Read Write Net Clock Rand Declassify ForeignCall Load Async Actuate
# literals / operators are punctuation, not keywords: ! { } [ ] ( ) -> => : , . ? + - * / % && || == != < <= > >=
```

*(The authoritative machine list is extracted from the parser tables at build time in Stage 9; this
enumeration must stay in sync and is the human-readable mirror.)*

---

## 3. Catalog key space + canonical en-US strings

The stable keys a catalog may define, with their canonical English values and typed placeholders.
**This is the enumerated key list other packs translate.** (Representative core set; the Stage-9
reference lists every key. Diagnostics use their DL code as the key with `message`/`label`/`help`
sub-keys.)

### 3.1 Diagnostics (one `[DLxxxx]` table each)

```toml
[DL0501]
message = "function `{fn}` performs `{effect}` but its row does not permit it"
label   = "this call performs `{effect}`"
help    = "add `{effect}` to the row (this widens authority, so CI may veto) or remove the call"

[DL0701]
message = "`main`'s authority exceeds the manifest"
label   = "declared here"
help    = "narrow the program's effects or widen `[authority]` in delulu.toml (a review decision)"

[DL0702]
message = "authority grant refused: {detail}"

[DL0602]
message = "a `Secret` value cannot flow here — `Secret[{inner}]` is not `{inner}`"
help    = "a secret only crosses via `expose`, which carries the `Declassify` effect"

[DL0802]
message = "requested authority is not within the holder's grant"
help    = "attenuate the request; the exact intersection is offered as the repair"

[DL1201]
message = "this construct is not supported by the WASM backend; it runs on the interpreter"

[DL1205]
message = "secret contents cannot enter the WASM guest ({detail})"
```

### 3.2 CLI strings (dotted keys)

```toml
[cli.grant-prompt.title]
message = "this program requests: {summary} — allow?"

[cli.grant-prompt.effects]
message = "effects: {effects}"

[cli.authority.header]
message = "Authority of `{program}` — what this program can do to your system:"

[cli.authority.foreign-separator]
message = "-- outside the proof (contained at process level) --"

[cli.run.ok]
message = "ok"

[cli.locale-picker.prompt]
message = "pick your compiler's vibe (changes human text only — codes & JSON never change):"

[cli.locale.add.confirm]
message = "catalogs change human text only; codes, repairs, and JSON never change; this catalog holds no authority. add `{locale}`?"
```

### 3.3 The welcome note — NOT a catalog key (documented here for completeness only)

The first-run welcome (Stage-8 §6.3) has **no catalog key** and is never translated. Byte-exact,
identical in every locale:

> U r here becoz u maybe a delulu like me & wanna create something others think is not possible.
> There's nothing wrong with being delulu. Anyway, u can't decide what others think abt u. So start
> building with everything u've got. Welcome to the Delulu Gang🐦‍🔥🔥🫡🚀

Attribution, exact: `— Jesse, The Creator of DeluluLang`. A catalog key attempting to define/override
it → DL1704.

---

## 4. Register & voice (en-US)

Plain, precise, honest. No hype, no superlatives (Constitution §9). Diagnostics state *what* is
wrong, *where*, and the *repair*; they never blame. `authority_widening` repairs are phrased as
review decisions, not casual fixes. This is the reference tone; `delulu-slang` is the same *content*
in a casual Gen-Z voice; other locales match en-US's precision in their own register.
