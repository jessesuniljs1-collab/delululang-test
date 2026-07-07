# Language Pack — `delulu-slang` (Gen-Z spoken English) — SHIPPED DEFAULT VOICE

**Role:** the project's signature voice option, offered beside `en-US` in the first-run picker.
**Same content as `en-US`, casual Gen-Z register.** Ships 100% of CLI strings + the top-priority
diagnostic set; long-form `explain` docs are en-US except the top-50 codes (declare honestly in
`[meta].coverage`). It is **English underneath** — so it changes only voice, and the machine envelope
is (as always) byte-identical to every other locale.

```toml
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

```toml
[cli.grant-prompt.title]
message = "this program wants: {summary} — you good with that?"

[cli.authority.header]
message = "what `{program}` can actually do to your system:"

[cli.authority.foreign-separator]
message = "-- outside the proof (contained at the process level, stay woke) --"

[cli.run.ok]
message = "ok bet 🔥"

[cli.locale-picker.prompt]
message = "pick your compiler's vibe (this changes human text ONLY — codes & JSON never move):"

[cli.locale.add.confirm]
message = "catalogs = human text only. codes/repairs/JSON don't change, and this thing holds zero authority. add `{locale}`?"
```

### 2.3 The welcome note

Unchanged and untranslated (it is *already* in Jesse's voice). Renders byte-identically here as
everywhere; no key; override attempt → DL1704.

---

## 3. Coverage honesty

`delulu-slang` is a *voice*, not a partial translation — CLI + top diagnostics are 100%; the long-tail
`explain` docs fall back to en-US and the picker says so. That fallback is the design, not a gap: a
delulu reads the vibe on the common path and the precise en-US doc when they go deep. 🐦‍🔥
