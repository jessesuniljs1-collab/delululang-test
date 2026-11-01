# Language Pack — `delulu-slang` (Gen-Z spoken English) — SHIPPED DEFAULT VOICE

**Role:** the project's signature voice option, offered beside `en-US` in the first-run picker.
**Same content as `en-US`, casual Gen-Z register.** It is **English underneath** — so it changes only
voice, and the machine envelope is (as always) byte-identical to every other locale.

> **This pack is the authoring target, not the shipped state.** The block below is what a *complete*
> delulu-slang catalog declares. What is actually built into the binary today
> (`crates/delulu-diag/catalogs/delulu-slang.toml`) is **version `0.8.0`, 8 of 157 diagnostic codes,
> and the 3 named CLI strings there are** — its own `[meta].coverage` says so. The diagnostics gap is
> tracked in [`../REMAINING_WORK.md`](../REMAINING_WORK.md) §6.2.
>
> The **CLI half was blocked until 2026-10-17**, when `delulu_diag::catalog::CLI_STRINGS` was empty and
> every `cli.*` key got DL1704. It now holds three strings (§6.1), and this catalog speaks all three; the
> rest of the CLI's prose is en-US until it is registered at its call site, so `cli = 100` measures the
> registered strings, not every line the CLI prints.

```toml
# The target for a complete pack — NOT what ships today (see the note above).
[meta]
locale    = "delulu-slang"
name      = "Delulu Slang"
fallback  = "en-US"
version    = "1.0.0"
coverage  = { cli = 100, diagnostics = 90, explain = 25 }
direction = "ltr"
```

---

## 1. Voice rules (so any author/AI stays consistent)

- **Casual, warm, hype-positive — but never dishonest.** The Constitution §9 honesty clauses bind
  slang exactly as they bind en-US: no "faster than C", no "lowest tokens", no "unbreakable". Slang
  can be playful about *tone*, never about *facts*. An `authority_widening` repair is still flagged as
  a real authority decision, just phrased casually.
- **Keep the technical noun.** `effect`, `authority`, `row`, `capability`, `secret`, the DL code — all
  stay verbatim. Slang wraps them; it never renames them (and it can't touch the code/JSON anyway).
- **Emoji allowed, sparingly, meaningful** (🔥 for a clean build, 💀 for a hard error, 🫡 for
  "human, your call"). Never in machine output — this is prose only.
- **Placeholders identical to en-US** (`{fn}`, `{effect}`, `{program}`, `{summary}`, …).

---

## 2. Catalog (representative core; author completes to declared coverage)

### 2.1 Diagnostics

```toml
[DL0501]
message = "yo, `{fn}` is out here doing `{effect}` but its row said no cap 💀"
label   = "this call does `{effect}`"
help    = "add `{effect}` to the row (heads up: that's an authority W, so CI might veto) or cut the call fr"

[DL0701]
message = "`main` be wanting more power than the manifest allows 💀"
help    = "chill the effects, or open up `[authority]` in delulu.toml — but that's a real call, not a vibe"

[DL0702]
message = "grant denied: {detail} — not today"

[DL0602]
message = "nah, that `Secret` can't flow here — `Secret[{inner}]` is NOT `{inner}`, no shortcuts"
help    = "a secret only comes out through `expose`, and that carries `Declassify`. no sneaking"

[DL0802]
message = "you asked for more authority than you actually hold 🤨"
help    = "attenuate it — here's the exact slice you're allowed"

[DL1201]
message = "the wasm backend can't do this one yet — running it on the interpreter, no stress"

[DL1205]
message = "secrets stay host-side, they don't go into the wasm guest ({detail}). that's the whole point fr"
```

### 2.2 CLI strings

The CLI key space is exactly `delulu_diag::catalog::CLI_STRINGS` — since 2026-10-17 (RW 6.1)
`cli.authority.header` (`{program}`), `cli.authority.foreign-separator` and `cli.locale.add.confirm`
(`{locale}`). A `cli.*` key outside it names no surface and gets DL1704; `cli.grant-prompt.*` and `cli.run.ok`
name lines the CLI does not print, and the first-run picker is shown before any locale exists, so it is en-US by
design. What ships:

```toml
[cli.authority.header]
message = "what `{program}` can actually do to your system:"

[cli.authority.foreign-separator]
message = "-- outside the proof (contained at the process level, stay woke) --"

[cli.locale.add.confirm]
message = "catalogs = human text only, and this one holds zero authority. install `{locale}`? [y/N]: "
```

### 2.3 The welcome note

Unchanged and untranslated (it is *already* in Jesse's voice). Renders byte-identically here as
everywhere; no key; override attempt → DL1704.

---

## 3. Coverage honesty

`delulu-slang` is a *voice*, not a partial translation: the long-tail `explain` docs fall back to
en-US and the picker says so. **That fallback is the design. The current shortfall is not.**

Two different things get called "coverage" on this page and they must not be blurred:

- **By design** — `explain` long-form docs are en-US outside the top codes. A delulu reads the vibe
  on the common path and the precise en-US doc when they go deep.
- **Not by design** — the shipped catalog covers 8 of 157 diagnostics against the `diagnostics = 90` this
  page targets. Until 2026-08-23 this section read *"CLI + top diagnostics are 100%"*, which described the
  target as though it were the state. The diagnostics half is unwritten work; the CLI half, blocked until
  2026-10-17 on `CLI_STRINGS` being empty, covers all 3 registered strings
  ([`../REMAINING_WORK.md`](../REMAINING_WORK.md) §6.1). 🐦‍🔥
