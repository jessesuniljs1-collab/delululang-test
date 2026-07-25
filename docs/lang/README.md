# DeluluLang Language Packs (`docs/lang/`)

Each `<locale>.md` here is the **content source** an author (human or AI) turns into a message-catalog
plugin (see `docs/design/LOCALIZATION_PLUGIN_GUIDE.md`) and, eventually, a keyword syntax-morph (see
`docs/design/SYNTAX_MORPH_SPEC.md` — **specified but not yet implemented**; the keyword tables here
are the content it will map from, not a feature you can use today: finding C22 in
`docs/design/HARDENING_CAMPAIGN.md`). A pack changes **human prose only** — never diagnostic codes,
JSON, spans, DIR, or anything a machine consumes (Stage-8 invariant 39).

## Priority / status

| Pack | Status in this planning pass | Role |
|---|---|---|
| `en-US.md` | **complete (reference)** | the base; holds the canonical keyword enumeration (§2) every morph maps from; every other locale falls back to it |
| `delulu-slang.md` | **complete** | the project's default voice option (Gen-Z spoken English) |
| `zh-CN.md` `ja-JP.md` `ko-KR.md` | **starter (overview)** | CJK — Opus 4.8 completes the full string set from the term tables + samples here |
| `hi-IN.md` `ar-SA.md` | **starter (overview)** | Devanagari / RTL — script+direction notes given; Opus completes strings |
| `fr-FR.md` `de-DE.md` `es-ES.md` `pt-BR.md` | **starter (overview)** | Latin-script — Opus completes strings |

The starter packs are deliberately **overviews**: script/direction notes, the core term table with
English glosses, a representative sample of translated diagnostics + CLI strings, and register
guidance — enough for Opus 4.8 (or a human, or another AI) to mechanically complete the full catalog
without re-deciding terminology. This conserves effort on work that does not require Mythos-class
reasoning while locking every *decision* that does.

## The invariants every pack obeys (repeat of the guide's law)

1. **Keys are stable ids** (diagnostic codes like `DL0501`; dotted CLI names like
   `cli.grant-prompt.title`). Translate the *values*, never the keys.
2. **Placeholders are typed and fixed** (`{fn}`, `{effect}`, `{program}`, `{summary}`, …). Reorder
   and surround freely; never invent new ones (→ DL1704).
3. **The welcome note is never translated** (Stage-8 §6.3) — no key for it; it renders byte-identically
   in every locale.
4. **Every non-English term is glossed in English** in the pack, so non-native maintainers and AIs can
   verify the choice.
5. **`en-US` is complete by construction** — the compiler refuses to build with a missing en-US key;
   all other packs may be partial and declare honest `coverage` in `[meta]`.
