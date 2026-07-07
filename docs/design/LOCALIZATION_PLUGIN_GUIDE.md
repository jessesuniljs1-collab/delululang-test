# DeluluLang — Human-Language Plugin Guide

**Status:** Design reference (normative for the catalog format and the `delulu locale` flow).
**Implemented by:** Stage 8 (§6 — catalogs, picker, welcome) and Stage 6 (the plugin machinery a
catalog rides on). **Companion:** `docs/playbooks/STAGE8_PLAYBOOK.md` (Phase 8b, 8f).
**Related:** `docs/design/SYNTAX_MORPH_SPEC.md` (changing the *programming*-language surface — a
different mechanism), `docs/lang/*.md` (the per-language packs this guide tells you how to build),
`CONSTITUTION.md` §8.5 (localization; no discrimination; AI-generated translations are the intended
path).

---

## 0. What this is, and the one boundary that matters

A **human-language plugin** (a "catalog") changes the *prose a person reads* — diagnostic messages,
CLI text, the grant prompt, `delulu explain` docs — into another human language (Chinese, French,
Hindi, …). It changes **nothing a machine consumes.** This boundary is the whole design:

> **The machine interface is locale-invariant (Stage 8 invariant 39).** Diagnostic **codes** (DL0501),
> **JSON** shapes, **repair ids**, **spans**, **exit codes**, `interface.json`, **DIR**, **traces**,
> the manifest/lockfile formats, the catalog key space — all byte-identical across every locale. A
> catalog can only ever change human prose. CI asserts this; a catalog that could change a machine
> field is a bug in the loader, not a feature of the catalog.

**Why this boundary is load-bearing for the AI-side of DeluluLang.** AI agents, LLMs, robots, and
future systems consume the *machine* interface — codes and JSON, never prose. Because localization
touches only prose, **adding fifty human languages costs the AI side exactly zero** — no new codes to
learn, no changed schemas, no slower parsing. Humans get their language; machines get the invariant
they already relied on. This is the no-discrimination principle made mechanical: the human side is
richly localizable precisely *because* the machine side is frozen. (See `AI_NATIVE_DESIGN.md`.)

---

## 1. The two shipped locales, and where new ones come from

- **`en-US`** (English, United States) — the base. **Complete by construction:** the compiler refuses
  to build with a missing `en-US` key. Every other locale falls back to it.
- **`delulu-slang`** — Gen-Z spoken English, the project's default voice option. Ships 100% of CLI
  strings + the top-priority diagnostic set; long-form `explain` docs are en-US except the 50
  most-hit codes (the catalog `[meta]` declares its coverage; the picker shows it).

Both are embedded in the binary. **Every other language ships as a catalog plugin** — authored by
anyone (human or AI), added/removed/edited freely. AI-generated community translations are the
*intended* path (Constitution §8.5): an LLM reads the `docs/lang/<locale>.md` pack + this guide and
emits the catalog.

---

## 2. The catalog format (normative)

A catalog is a single TOML document. Shipped locales live at `catalogs/<locale>.toml` (embedded);
plugin locales are delivered as `.dpx` (see §4).

```toml
[meta]
locale       = "fr-FR"                 # BCP-47 tag; the id used by --locale / DELULU_LOCALE
name         = "Français (France)"     # shown in the picker, in the locale's own language
fallback     = "en-US"                 # always en-US in v1.0 (single-hop fallback)
version      = "1.0.0"                 # catalog semver (independent of language version)
coverage     = { cli = 100, diagnostics = 100, explain = 12 }   # honest %; the picker shows it
direction    = "ltr"                   # "ltr" | "rtl" (rtl → the CLI renderer mirrors box art; §5)
authors      = ["…"]                   # human or AI, credited in `delulu locale info`

# --- diagnostics: keyed by stable DL code; three sub-keys, each optional but message recommended ---
[DL0501]
message = "la fonction « {fn} » effectue « {effect} » mais sa ligne d'effets l'interdit"
label   = "cet appel effectue « {effect} »"
help    = "ajoute « {effect} » à la ligne d'effets (élargit l'autorité) ou supprime l'appel"

# --- CLI strings: keyed by dotted stable id ---
[cli.grant-prompt.title]
message = "ce programme demande : {summary} — d'accord ?"

[cli.authority.header]
message = "Autorité de « {program} » — ce que ce programme peut faire à votre système :"
```

**Rules:**
- **Keys are stable ids** — diagnostic codes (`DL0501`) and dotted CLI names (`cli.grant-prompt.title`).
  The full key list is enumerated in `docs/for-agents.md` / the Stage-9 reference; a catalog need not
  define all of them (missing keys fall back), but `[meta]` must be honest about coverage.
- **Placeholders are typed and fixed per key** — `{fn}`, `{effect}`, `{program}`, `{summary}`, etc.
  You may reorder them and surround them with any prose, but you may not invent new ones: an unknown
  placeholder → **DL1704** at catalog load, and that entry falls back to en-US (the rest of the
  catalog still loads). The canonical placeholder set per key is listed in the reference.
- **You translate prose, never structure.** No key controls a code, a JSON field, a repair id, or a
  span. Those do not exist in the catalog's vocabulary.
- **The welcome note is untouchable.** There is no catalog key for Jesse's first-run welcome (Stage 8
  §6.3). It renders byte-identically in every locale, is never translated or paraphrased, and a
  catalog key attempting to override it → **DL1704**. (It is human-emotional, creator-authored text,
  not UI prose — deliberately outside the localization surface.)

---

## 3. What a per-language pack (`docs/lang/<locale>.md`) provides

Each `docs/lang/<locale>.md` is the **content source** an author (AI or human) turns into a catalog.
A complete pack contains:
1. **Script & direction notes** — the writing system, LTR/RTL, any rendering caveats (combining
   marks, width, fonts) the CLI/LSP must respect.
2. **The DeluluLang vocabulary table** — for each core concept (effect, authority, capability,
   grant, secret, row, plugin, actor, …): the chosen term in the target language **plus the English
   gloss in parentheses**, so a non-native speaker or an AI reading the pack understands the choice.
   *(The keywords of the programming language are NOT translated here — see §6.)*
3. **The diagnostic message set** — a translation for each priority DL code, with placeholders shown.
4. **The CLI string set** — grant prompt, authority header, `run`/`build` status lines, picker line.
5. **Tone & register guidance** — formal vs. casual, how to render the project's honesty voice.
6. **Coverage declaration** — what this pack does and does not cover, matching the catalog `[meta]`.

The English gloss requirement is deliberate: the packs must be usable by (a) a non-native human
maintainer, and (b) an AI with no privileged knowledge of the language. Every non-English term in
every pack is glossed.

---

## 4. Authoring, adding, removing, editing a catalog plugin

### Author
A catalog plugin is a **`kind = "plugin"`, class `verified`, zero-authority** package (`effects = []`)
exporting one function: `catalog() -> Str` returning the TOML above. Zero authority is enforceable and
enforced (Stage 6): a catalog can render prose and *nothing else* — it cannot read files, reach the
network, or hold any capability. Build it with `delulu plugin build` → `fr-FR.dpx`.

```delulu
module fr_fr_catalog
pub fn catalog() -> Str ! {} { "<the TOML document as a string literal>" }
```

### Add
```
delulu locale add fr-FR.dpx
```
Loads the plugin (Stage-6 load sequence, zero-authority grant), validates the catalog (placeholders,
meta, welcome-override check → DL1704), registers it under its `[meta].locale`. The confirmation
prompt states plainly: *"catalogs change human text only; codes, repairs, and JSON never change; this
catalog holds no authority."*

### Use
```
delulu --locale fr-FR check app.delulu      # one invocation
DELULU_LOCALE=fr-FR delulu ...              # session
# or set `locale = "fr-FR"` in ~/.delulu/config.toml
```
Selection priority (Stage 8 §6.2): `--locale` > `DELULU_LOCALE` > config file > first-run picker >
`en-US`.

### Inspect / edit / remove
```
delulu locale list                 # installed locales + coverage
delulu locale info fr-FR           # meta, authors, coverage, missing-key count
delulu locale remove fr-FR         # unregister; plugin node revoked (Stage-5 transitive)
```
Editing = rebuild the `.dpx` and `delulu locale add` again (same locale id replaces the prior
registration; catalog `version` should bump). Because catalogs are ordinary zero-authority plugins,
add/remove/replace are the ordinary plugin lifecycle — nothing bespoke.

---

## 5. RTL and non-Latin scripts (loader responsibilities)

`[meta].direction = "rtl"` tells the human-text renderer to mirror box-drawing and align right for
CLI panels; **it never affects code, source files, or machine output** (source stays LTR; a program
is the same bytes regardless of the reader's locale). Non-Latin scripts (CJK, Devanagari, Arabic)
require the terminal/LSP to handle wide and combining characters — the renderer measures display width
(not byte or char count) when laying out panels. These are presentation concerns confined to the
human-text path; the machine path is untouched. Per-script specifics live in each `docs/lang/*.md`.

---

## 6. What this guide does NOT cover

- **Changing the programming language's own keywords/characters** (`fn` → something else, or a
  token-minimized skin for an AI): that is a *different* mechanism — a **syntax morph**, not a
  human-language catalog — described in `docs/design/SYNTAX_MORPH_SPEC.md`. A catalog never changes
  what compiles; a syntax morph changes the *surface syntax* while preserving the AST. Keep the two
  strictly separate: prose vs. program.
- **Translating `delulu explain` long-form docs beyond declared coverage** — allowed, encouraged,
  and simply more catalog keys; coverage is declared honestly in `[meta]`.

---

## 7. Acceptance (what "a good language plugin" means)

A catalog plugin is done when: it loads via `delulu locale add` with zero authority; renders its
declared coverage; falls back cleanly to en-US on undefined keys; declares honest coverage in
`[meta]`; never triggers DL1704 (no bad placeholders, no welcome override); passes the locale-
invariance CI check (machine output byte-identical to en-US); and its `docs/lang/<locale>.md` pack
glosses every non-English term. AI authorship is first-class — an LLM producing this from the pack +
this guide is the designed workflow, not a workaround.
