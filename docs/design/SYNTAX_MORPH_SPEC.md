# DeluluLang — Syntax Morph Specification (surface-syntax plugins)

**Status:** Design reference — **normative for the morph format and the canonical-form law, and NOT
YET IMPLEMENTED.** No `delulu morph` verb, no morph loader, and no `--morph` flag exist in the
toolchain as of v1.0.0; the string `morph` appears in no source file. An earlier revision of this
header claimed "Implemented by: Stage 8 tooling", which was false — `STAGE8_SPECIFICATION.md` §6
scheduled the `delulu morph` sibling of `delulu locale` as work to do "when building", and it was
never built. Recorded as finding **C22** in `docs/design/HARDENING_CAMPAIGN.md`; implementation is
scheduled there. Do not cite this document as evidence that surface-syntax plugins ship.
**When implemented:** Stage 8 tooling (fmt/LSP integration; morphs are tooling, never semantics —
invariant 38) with the plugin machinery of Stage 6. **Companion:**
`docs/design/LOCALIZATION_PLUGIN_GUIDE.md` (human *prose* localization — a different mechanism),
`docs/design/AI_NATIVE_DESIGN.md` (why AI defaults to canonical), `CONSTITUTION.md` §8.4/§8.5.

---

## 0. What a syntax morph is

A **syntax morph** changes the *surface syntax of DeluluLang source* — the keywords, and optionally
operators/punctuation — while preserving the program **exactly**. Two audiences, two uses:

- **Humans:** write DeluluLang with keywords in their own language — `fn` as `fonction` (fr), `函数`
  (zh), `कार्य` (hi) — lowering the literacy barrier without forking the language.
- **AI / agents / LLMs / robots / future systems:** apply a **token-minimizing profile** — shorter
  ASCII aliases chosen to reduce tokenizer cost — or any custom profile an AI prefers for its own
  efficiency. (Honesty clause: token savings are tokenizer-specific; no "lowest tokens" claim is ever
  made — measure per model. Constitution §5.11.)

Anyone or any AI may author, add, edit, or remove a morph, exactly like a locale catalog.

## 1. The canonical-form law (the one rule that makes this safe)

> **Law:** every morph is a **bijective, token-level mapping** to and from **canonical DeluluLang**
> (the Stage-1 grammar's keywords, exactly). The AST is identical under any morph; `fmt` can convert
> any file between any two morphs losslessly; every artifact (`.dwx`, `.dpx`, DIR), every hash, every
> diagnostic span, every registry entry, and everything the checker/engines/broker see is **canonical
> — always.** Morphs live at the *edge* (read/render); the toolchain's interior knows one language.

Consequences (each normative):
1. **Bijective or rejected.** A morph maps each canonical keyword to exactly one alias and no two
   keywords to the same alias; aliases must be single lexer tokens and must not collide with each
   other. Violation → the morph fails to load (DL17xx-class, exact colliding pair named).
2. **Storage is a project choice; identity is not.** A file may be *stored* in a morph (declared, §3)
   or in canonical; either way its canonical form — and therefore its content hash, its DIR, its
   authority — is identical. Two developers reading one file through different morphs are reading
   *the same program*, provably.
3. **Identifiers are never morphed.** A morph maps **keywords, operators, and fixed punctuation
   only**. User identifiers, string literals, comments, and module names pass through untouched
   (comments/strings are prose — the *catalog* system's domain, not the morph's).
4. **Diagnostics render in the reader's morph.** When the CLI/LSP quotes source or names a keyword in
   a message, it renders through the active morph — but the machine envelope (codes, spans in
   canonical byte offsets of the stored file, JSON) is morph-invariant, same law as locales.
5. **No morph, no meaning change — ever.** A morph cannot add syntax, remove syntax, change
   precedence, or introduce macros. It is a renaming, not a preprocessor. (Anything more is an RFC
   for the language itself.)

## 2. The morph format

```toml
[meta]
morph      = "hi-IN-keywords"        # id used in --morph / file pragma
name       = "हिन्दी कीवर्ड (Hindi keywords)"
version    = "1.0.0"
kind       = "human"                 # "human" | "compact" (AI/token-minimizing) | "custom"
authors    = ["…"]

[keywords]                            # canonical = alias (every entry optional; unlisted = canonical)
fn = "कार्य"        # (function)
let = "मान"        # (value binding)
match = "मिलान"     # (match)
actor = "कर्ता"     # (actor)
# … the full canonical keyword list is enumerated in docs/lang/en-US.md §2

[operators]                           # optional; same bijectivity rules
"->" = "→"

[notes]
gloss = "Every alias glossed in docs/lang/hi-IN.md; this table must stay in sync with it."
```

A **compact profile** example (AI-oriented; kind = "compact"):

```toml
[keywords]
fn = "f"   let = "l"   match = "m"   return = "r"   actor = "a"   spawn = "s"
# chosen for common-tokenizer efficiency; measured per model, never claimed universally
```

Delivery: embedded (none shipped by default except canonical) or as a **zero-authority verified
plugin** exporting `morph() -> Str`, managed via `delulu morph add|list|info|remove` — the same
lifecycle, prompts, and honesty text as `delulu locale` (§4 of the localization guide applies
verbatim, s/catalog/morph/).

## 3. Reading and writing morphed files

- **File pragma (stored-morph declaration):** a first-line comment `//! morph: hi-IN-keywords`
  tells every tool how to read the file. No pragma = canonical. The pragma names the morph; the
  morph must be installed (else a precise "install morph X" error).
- **Conversion:** `delulu fmt --to-morph <id>` / `--to-canonical` rewrites files losslessly
  (identity law: converting there and back is byte-stable modulo the pragma line).
- **Per-reader rendering (LSP):** an editor user may set a *view* morph; the LSP renders keywords in
  it and writes back in the file's stored morph. Two humans and an AI can share one repo, each
  reading their own surface, storing one truth.
- **Interchange & publish:** `delulu publish` and plugin/artifact builds always emit canonical
  (artifacts carry no morph). CI diffing, review tools, and agents default to canonical.
- **Mixed repos are fine** (per-file pragmas), but `fmt --check` can enforce a repo-wide policy via
  `delulu.toml` `[style] morph = "canonical"` (the recommended default for shared projects).

## 4. Why AI systems default to canonical (and when they shouldn't)

Canonical DeluluLang **is already the AI-native surface** — designed terse, regular, and
ASCII-stable. The machine interface (JSON, codes, DIR) never passes through a morph at all. So for
AI: use canonical unless *you* measure a real win from a compact profile with *your* tokenizer; the
mechanism exists so that choice belongs to the agent, not to us. A future AI that prefers a radically
different surface can ship its own morph without asking permission — bijectivity keeps it honest, and
humans can still read the same program through their own morph. **No party's preferred surface is
privileged; the canonical form is merely the shared coordinate system.** (No-discrimination, made
mechanical — same pattern as locales. See `AI_NATIVE_DESIGN.md`.)

## 5. Threat/abuse notes (honesty)

- A morph could make code *look* unfamiliar to a reviewer reading a different surface. Mitigations:
  review tools default to canonical; `fmt --check` policy; the pragma is visible line 1; bijectivity
  means nothing can *hide* (rendering is lossless both ways).
- Homoglyph mischief inside *identifiers* is not a morph concern (identifiers never morph) and is
  handled by the lexer's existing confusable rules.
- Token-savings claims are tokenizer-relative; the docs never promise a number (Constitution §5.11).

## 6. Cross-references to update when implementing

Stage 8 spec §6 gains a sibling subsection "§6.5 syntax morphs" pointing here; `STAGE8_PLAYBOOK.md`
Phase 8f extends to `delulu morph`; `docs/lang/en-US.md` §2 is the canonical keyword enumeration every
morph maps from; each `docs/lang/<locale>.md` §"keyword skin" table doubles as that language's
suggested human morph.
