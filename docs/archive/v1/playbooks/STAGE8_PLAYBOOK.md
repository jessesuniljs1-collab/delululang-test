# Stage 8 Playbook — "Surface" (LSP, fmt, test runner, localization, welcome)

**Companion to:** `docs/design/STAGE8_SPECIFICATION.md` (normative). This file is *how to build it*.
**Depends on:** Stage 7 complete (the language core is **feature-frozen for v1.0** — invariant 38
makes "no semantics in tooling" a *rule*, not a hope).

> **The one-sentence goal:** give DeluluLang its face — terminal-first for agents and humans, one LSP
> server for every editor and agentic IDE, one canonical formatter, an authority-isolated test
> runner, the locale-keyed message layer (English (US) + Delulu Slang), Jesse's first-run welcome
> note, and the signing + registry-client groundwork Stage 9 opens. **Nothing here changes what
> programs mean.**

> **This playbook is the anchor for the localization work.** The message-catalog system built in
> Phase 8f is exactly the machinery the human-language packs (`docs/lang/*.md`) target. See
> `docs/design/LOCALIZATION_PLUGIN_GUIDE.md` (how a catalog plugin is authored/added/removed) and the
> per-language `docs/lang/<locale>.md` packs. When building 8f, treat those documents as the
> requirements spec for the catalog format and the `delulu locale add` flow.

---

## 0. Orientation — the two rules that dominate this stage

- **No semantics in tooling (invariant 38).** The formatter is AST-identity-preserving; the LSP
  reports exactly what the compiler computed; catalogs change prose, never meaning. If any Stage-8
  change alters what compiles or what a row means, it is a bug.
- **The machine interface is locale-invariant (invariant 39).** Codes, JSON shapes, repair ids,
  spans, exit codes, `interface.json`, DIR, traces — **byte-identical across locales.** A locale
  changes only human prose. CI asserts this (criterion 7); build the assertion early.

And the agent-safety rule that catches the subtle bug: **the welcome and the picker never block
machines (invariant 40).** Any of `--json` / `CI` env / non-TTY stdout / `DELULU_NO_FIRST_RUN=1` ⇒ no
picker, no welcome, defaults applied, zero prompts. An agent must be able to run `delulu` cold with
no interactive surprise, ever.

Read before writing: spec §3 (LSP contract), §4 (fmt identity/idempotence laws), §5 (test authority
isolation via the Stage-5 broker tree), §6 (catalogs/picker/welcome — the text of the welcome is
*law*, byte-exact), §7 (signing + registry client).

---

## 1. Where the work lands

- **`delulu-syntax`:** the `test` block grammar (contextual keyword `test` in item position only) →
  `Item::Test(TestDecl)`.
- **`delulu-check`:** `assert`/`assert_eq` prelude builtins (pure; `assert_eq` on opaque types refused
  per R-5); test-body typing (returns Unit; `test_root: Root` scoped by the test manifest).
- **`delulu-diag`:** the **catalog layer** — this is the center of gravity of the stage. Diagnostics
  gain a rendering step that looks up `code → catalog entry → interpolate typed placeholders`, with a
  fallback chain to `en-US`. The **machine envelope (codes/spans/repairs/JSON) is produced *before*
  catalog rendering** so locale can never touch it (invariant 39 by construction).
- **`delulu` (CLI):** `delulu lsp`, `delulu fmt`, `delulu test`, `delulu locale …`, `delulu keygen|
  sign|verify-sig`, `delulu publish --dry-run`, `delulu add`; the first-run picker + welcome; the
  registry index client.
- **New crate `delulu-lsp`** (or a module in the CLI): the LSP server, wrapping the *incremental*
  checking pipeline. **It never runs code, loads plugins, or touches the broker — analysis only.**
- **`editors/vscode/`:** the in-repo extension (LSP client + TextMate grammar + `.delulu` icon) and
  `docs/editors.md` (the three-line generic config any LSP editor/agent IDE consumes).

---

## 2. Phase plan (each phase: build green → test green → update spec §"status" → commit)

### Phase 8a — `test` blocks + `assert`/`assert_eq` (grammar + check + compiled-out builds)
Parse `test "name" ! {row}? { … }` (contextual `test` keyword in item position). Body typed like a
Unit-returning fn; omitted row = pure. `assert(cond: Bool)`/`assert_eq[T](a, b)` prelude builtins,
pure; `assert_eq` on an opaque `T` → refused (R-5, like everywhere). Tests compile out of non-test
builds. No runner yet.
*Test:* a `test` block type-checks; `assert_eq` on a `Secret` → the opacity error; test items absent
from a normal build.

### Phase 8b — the message-catalog layer (the localization foundation)
Implement `catalogs/<locale>.toml` loading (spec §6.1): keys are **stable ids** (diagnostic codes +
named CLI strings); placeholders are **typed** (`{fn}`, `{effect}` — unknown placeholder → DL1704 at
catalog load, entry falls back). **Fallback chain:** chosen locale → `en-US`; `en-US` is complete *by
construction* — the compiler **refuses to build with a missing en-US key** (build-time check). The
render step is the *last* thing that touches a diagnostic; the JSON envelope is frozen before it.
Ship `en-US` and `delulu-slang` catalogs embedded in the binary.
*Test (criterion 7):* the full conformance JSON is byte-identical under `en-US` and `delulu-slang`
(machine fields); human text differs; a DL0501 renders per the slang catalog; a missing en-US key
fails the build.

### Phase 8c — locale selection + the first-run picker + the welcome (agent-safe)
Selection priority: `--locale` > `DELULU_LOCALE` > `~/.delulu/config.toml` > first-run picker (TTY
only) > `en-US`. The picker is two lines, once. The **welcome** (spec §6.3) shows once, immediately
after the picker, **only** on interactive TTY with no `CI`, no `--json`, no `DELULU_NO_FIRST_RUN`;
then records `welcomed = true`. The welcome text is **byte-exact, hash-pinned in the test, identical
in both locales, never translated/paraphrased/overridden** (a catalog key attempting to override it →
DL1704). It never appears in any machine output/log/trace/LSP/CI stream.
*Test (criterion 6):* shown exactly once on a fresh TTY home; **never** with `--json`/`CI=1`/piped
stdout/`DELULU_NO_FIRST_RUN=1`/second run (five automated cases); text hash-equal to §6.3.

> **The welcome text, byte-exact (this is law — copy character-for-character, including emoji):**
> `U r here becoz u maybe a delulu like me & wanna create something others think is not possible.
> There's nothing wrong with being delulu. Anyway, u can't decide what others think abt u. So start
> building with everything u've got. Welcome to the Delulu Gang🐦‍🔥🔥🫡🚀`
> Attribution line beneath, exactly: `— Jesse, The Creator of DeluluLang`.

### Phase 8d — `delulu fmt` (identity + idempotence laws)
One style, zero options (spec §4 normative choices: 4-space indent, 100-col soft, trailing commas,
one blank line between items, rows `! {Read, Net}` alphabetical, imports sorted std/deps/local,
attributes own-line). **Identity law:** `parse(fmt(src)) ≡ parse(src)` (AST-equal incl. comment
attachment). **Idempotence:** `fmt(fmt(src)) == fmt(src)` byte-equal. Both property-tested over the
conformance + fuzz corpora; violation → DL1702 (compiler-bug class). `--check` (CI, exit 1 on diff),
`--stdin` (editor/agent). **Migration engine** `--migrate <version>`: AST-transformations shipped with
the compiler (never regex); first user is Stage-7's `consume`/`recover` (DL1608).
*Test (criterion 4):* identity + idempotence over ≥100k programs; `--check` exits 1 on unformatted;
`--migrate 0.7` passes the Stage-7 migration corpus.

### Phase 8e — `delulu lsp` (analysis-only, incremental)
Stdio, LSP 3.17, one instance per workspace, wrapping the incremental checking pipeline (per-module
re-check with dependency invalidation; ≤150 ms edit-to-diagnostics on a 10-kLoC workspace — measured,
regression = release blocker). Capabilities per spec §3 table: push diagnostics (= compiler's, same
codes/spans/repairs), codeAction (every typed repair; **widening repairs ⚠-tagged, `isPreferred:
false`**; `requires_human` repairs are documentation-only), hover (type + **effect row** + rcap +
opacity + cap provenance + transitive `authority: {…}`), **inlayHint = the authority lens** (inferred
rows on lambdas; `⚑ Net` on calls adding effects), definition/references/rename (DefId-based,
cross-package read-only), semanticTokens (distinct kinds for effects/rcaps/caps/secrets), codeLens
(`▶ run` / `authority:` on main and tests), and the `delulu.authority` command (returns the §10.5
report JSON — agent harnesses call this instead of shelling out). **The server holds no leases and
can effect nothing** (a compromised workspace cannot use the LSP as an effector).
*Test (criterion 1, 2, 3):* LSP smoke suite — diagnostics equal `delulu check --json`; hover shows the
full signature + row; the DL0501 code action applies and re-check goes green; cross-package rename;
the widening action is distinguishable in the payload; ≤150 ms latency on the reference workspace.

### Phase 8f — catalog plugins + `delulu locale add` (the localization dogfood)
A catalog plugin is a `kind = "plugin"`, class `verified`, **zero-authority** (`effects = []`) package
exporting `catalog() -> Str` (the TOML). Installed via `delulu locale add <file.dpx>` — **the plugin
machinery's first zero-authority dogfood** (this depends on Stage 6). AI-generated community
translations are the *intended* path (Constitution §8.5). This is where the `docs/lang/*.md` packs
become real: each pack specifies one such catalog. Removal/edit is symmetric (`delulu locale
remove`); the catalog `[meta]` declares its coverage and the picker shows it. See
`docs/design/LOCALIZATION_PLUGIN_GUIDE.md`.
*Test (criterion 8):* a third locale loads via `delulu locale add`, renders, falls back to en-US on
missing keys; a catalog attempting to override the welcome key → DL1704.

### Phase 8g — `delulu test` (authority-isolated runner)
`[test-authority]` package ceiling (default pure); per-file `test authority ! {…} { … }` header ⊑ the
ceiling (DL1703). Each test file is a **process-isolated run under a broker child node carrying
exactly its manifest** (Stage-5 tree: test nodes are children of a `test-session` node; session end =
transitive revocation — nothing a test leaked survives). Deterministic by default (`Cap[Clock]` fixed,
`Cap[Rand]` seeded per test-name hash; `--seed` overrides). **`--trace-effects` is always-on in test
runs — a test's actual effect list is part of its JSON report** (an agent-written test that quietly
reads the network is visible even when it passes). Exit codes 0/1/2.
*Test (criterion 5):* pure tests run with zero grants; a test calling `http.get` without test-authority
fails DL0501 (compile) or DL0701 (session grant); per-test `effects_traced` in the JSON; session-end
revocation shown in `delulu grants tree`.

### Phase 8h — signing + registry client groundwork
`delulu keygen` (ed25519, `~/.delulu/keys/`, 0600); `delulu sign`/`verify-sig` for `.dwx`/`.dpx`/
tarballs (the Stage-6 `delulu:sig` generalized). The **registry index format** (normative now, hosted
in Stage 9): cargo-style sparse HTTP index, one JSONL line per version **carrying the authority
summary** (effects/cap_kinds/scopes/hashes) — so `delulu add` shows the authority diff *before*
downloading (the supply-chain UX no other registry has). `delulu publish --dry-run` validates manifest
completeness + semver-authority (DL1003 reused) + signature; `delulu add` resolves via index, shows
the summary, writes manifest+lockfile through Stage-2 machinery unchanged.
*Test (criterion 9):* `publish --dry-run` catches an authority-widening minor bump against a local
index fixture; `delulu add` renders the authority summary from the index line alone (no download).

---

## 3. The traps

1. **Render prose last; freeze the machine envelope first.** The ordering "compute diagnostic (codes/
   spans/repairs/JSON) → *then* render human text via catalog" is what makes locale-invariance
   structural rather than tested-and-hoped. Build it that way and criterion 7 is nearly free.
2. **The welcome text is law — byte-exact, hash-pinned, never translated.** Do not "clean up" the
   spelling, the emoji, or the attribution. A catalog that tries to override it → DL1704. It never
   appears in machine output (criterion 6 tests all five channels).
3. **Agents run cold with zero prompts.** Invariant 40 — the single most important agent-usability
   rule of the stage. `--json`/`CI`/non-TTY/`DELULU_NO_FIRST_RUN` each independently suppress the
   picker+welcome. Test each channel separately.
4. **The LSP is analysis-only and holds no authority.** No code execution, no plugin loads, no broker
   connection. Its *availability* is not a security property. A workspace cannot weaponize the LSP.
5. **Tests hold no ambient authority (invariant 41).** This is the "agents cut corners in tests"
   defense — a test gets exactly its declared manifest, enforced by a real Stage-5 broker child node,
   and its effects are traced into the report even on pass. Do not add a "test mode" that relaxes this.
6. **fmt is AST-identity-preserving.** It reprints, it never rewrites meaning. The identity and
   idempotence laws are compiler-bug-class (DL1702) if violated — property-test them, don't spot-check.
7. **Catalog plugins are zero-authority verified-class.** They render prose and nothing else; the
   plugin machinery (Stage 6) bounds them to exactly that. State this in `delulu locale add`'s
   confirmation prompt (a malicious catalog can *mislead a human reader* but cannot alter codes/
   repairs/JSON — spec §11).
8. **One server, every surface.** No editor-specific LSP features — VS Code, Zed, Neovim, JetBrains-
   via-LSP, and agentic IDEs (Antigravity et al.) all consume the same `delulu lsp`. Resist per-editor
   forks (Constitution §8.4).

---

## 4. Definition of done (map to spec §9 acceptance criteria)

Ship when all 10 criteria pass — the load-bearing ones are criterion 6 (welcome/picker agent-safety,
five channels), criterion 7 (locale invariance byte-for-byte), criterion 4 (fmt identity+idempotence
over ≥100k programs), and criterion 5 (test authority isolation via the broker). Add `##
Implementation status` to `STAGE8_SPECIFICATION.md` per the Stage-3 §8a pattern. Cross-reference the
localization docs (`LOCALIZATION_PLUGIN_GUIDE.md`, `docs/lang/*.md`) from §6 of the spec.

*Stage 8 gives the language its face — terminal-first, every editor, two voices, one law. Stage 9
freezes v1.0 and proves it.*
