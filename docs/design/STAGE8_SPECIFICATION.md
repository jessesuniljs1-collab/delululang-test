# DeluluLang — Stage 8 Implementation Specification

**Version:** 0.8 ("Surface"). **Status:** Committed — buildable directly from this document.
**Depends on:** Stage 7 complete (the language core is now feature-frozen for v1.0; this stage
adds *no* semantics — invariant 38 makes that a rule, not a hope).
**Governing documents:** `CONSTITUTION.md` (§8.1–§8.5), the two-audience rule (§8.4: machine
surfaces optimize for speed/feedback/agent usability; human surfaces for learnability/review).

---

## 0. Scope and goal

**Goal:** make DeluluLang excellent to *use* — terminal-first for agents and humans, LSP for
every editor and agentic IDE, one canonical formatter, an authority-aware test runner, the
locale-keyed message layer (English (US) + Delulu Slang), the first-run human experience with
Jesse's welcome note, and the signing + registry-client groundwork that Stage 9's community
opening needs.

**In scope (must ship):**
- `delulu lsp` — the LSP server (contract in §3).
- `delulu fmt` — canonical formatter + the migration engine (`--migrate`).
- `delulu test` — authority-isolated test runner (grammar addition: `test` blocks).
- The **message catalog** system; `en-US` and `delulu-slang` catalogs; first-run locale picker;
  catalog plugins.
- The **first-run welcome** (§6.3) — human-TTY-only, verbatim.
- Package signing (`delulu sign|verify-sig`) and the registry **client + index format**
  (`delulu publish --dry-run`; hosted registry itself is Stage 9 ops).
- VS Code extension skeleton (LSP client + TextMate grammar) in-repo; generic instructions for
  any LSP-capable editor/agent IDE (Antigravity et al. consume the same server).
- New diagnostics: DL17xx.

**Non-goals:** semantic changes of any kind (invariant 38); hosted registry operations,
governance docs, measurement program (Stage 9); IDE-specific plugins beyond VS Code (community,
via LSP); translation of catalogs beyond the two shipped (plugins/AI-generated, by design —
Constitution §8.5).

---

## 1. Invariants (carried + new)

All prior invariants hold. New:

38. **No semantics in tooling.** Nothing in this stage changes what programs compile, what rows
    mean, or what runs. The formatter is AST-identity-preserving (§4); the LSP reports what the
    compiler computed; catalogs change prose, never meaning.
39. **The machine interface is locale-invariant.** Codes, JSON shapes, repair ids, spans, exit
    codes, `interface.json`, DIR, traces: byte-identical across locales. A locale can only ever
    change human prose. (CI asserts this — §9.7.)
40. **The welcome and the picker never block machines.** Any of: `--json`, `CI` env var, non-TTY
    stdout, `DELULU_NO_FIRST_RUN=1` ⇒ no picker, no welcome, defaults applied (`en-US`), zero
    prompts. Agents must be able to run `delulu` cold with no interactive surprise, ever.
41. **Tests hold no ambient authority.** `delulu test` grants each test file exactly its declared
    manifest — an undeclared effect in a test fails like production code (DL0501/DL0701). Tests
    are the first place agents cut corners; the language does not let them.

---

## 2. Grammar addition: `test` blocks

```ebnf
item = [ "pub" ] , ( fn_decl | type_decl | effect_decl | const_decl
                   | foreign_decl | actor_decl ) | test_decl ;
test_decl = "test" , STRING , [ effect_row ] , block ;      (* contextual keyword: `test` is a
                                                               keyword only in item position *)
```

- `test "name" { … }` — body typed like a function body returning `Unit`; omitted row = pure.
- Allowed in any file under `tests/`, and in `src/` files (unit tests co-located); compiled out
  of non-test builds.
- Test bodies receive a `test_root: Root` binding scoped by the test manifest (§5.1).
- Assertions are prelude builtins, pure: `assert(cond: Bool)`, `assert_eq[T](a: T, b: T)`
  (T must be non-opaque — R-5 applies; comparing secrets in tests is refused like everywhere
  else). Failure = panic with a structured payload (file/line/expected/actual).

AST: `Item::Test(TestDecl { name: String, row: Option<RowExpr>, body: Block, id, span })`.

---

## 3. `delulu lsp` — contract

Transport: stdio, LSP 3.17. One server binary flag (`delulu lsp`), one instance per workspace.

**Capabilities (normative list):**

| Capability | Behavior |
|---|---|
| diagnostics (push) | exactly the compiler's diagnostics — same codes, same spans, same repairs; produced by the incremental pipeline (§3.1) |
| codeAction | every typed repair becomes a code action; `authority_widening: true` repairs are tagged (`isPreferred: false`) and titled with a ⚠ prefix; `requires_human` repairs appear as documentation-only actions |
| hover | type + **effect row** + rcap + opacity + (for caps) resource kind and grant provenance; for a `fn`: its full signature and, one line below, `authority: {…}` computed transitively |
| inlayHint | **authority lens**: inferred rows on unannotated lambdas; `⚑ Net` markers on calls whose row adds effects the enclosing fn declares — the review surface (Constitution §8.4) |
| definition / references / rename | DefId-based, cross-package (read-only into `dep:` files) |
| documentSymbol / workspaceSymbol | items incl. actors, behaviors, tests |
| semanticTokens | full; distinct token kinds for effects, rcaps, capability types, secrets |
| codeLens | on `fn main` and each `test`: `▶ run` / `authority: …` lens invoking the CLI |
| command `delulu.authority` | returns the §10.5 authority report JSON for the workspace (agent harnesses call this instead of shelling out) |

§3.1 **Incrementality contract:** re-check after an edit is per-module with dependency
invalidation; hover/diagnostic latency budget ≤ 150 ms for a 10-kLoC workspace on CI reference
hardware (measured; regression = release blocker). The LSP never runs code, loads plugins, or
touches the broker — analysis only (a compromised workspace cannot use the LSP as an effector).

**Editor integration:** in-repo `editors/vscode/` extension (LSP client, TextMate grammar,
`.delulu` icon); `docs/editors.md` gives the three-line generic config (command: `delulu lsp`)
that any LSP-capable editor or agentic IDE (Antigravity, Zed, Neovim, Helix, JetBrains-via-LSP)
consumes. No editor-specific server features, ever — one server, every surface (Constitution
§8.4).

---

## 4. `delulu fmt` — canonical formatter + migration engine

- **One style, zero options** (Constitution §8.3). Normative choices: 4-space indent; 100-col
  soft limit; trailing commas in multiline lists; one blank line between items; rows formatted
  `! {Read, Net}` (space after `!`, alphabetical effect order); imports sorted (std, deps,
  local); attributes own-line.
- **Identity law:** `parse(fmt(src))` ≡ `parse(src)` (AST-equal including comments' attachment).
  Property-tested against the conformance corpus + fuzz corpus (violation = DL1702,
  compiler-bug class).
- **Idempotence law:** `fmt(fmt(src)) == fmt(src)` byte-equal (same test rig).
- `delulu fmt --check` (CI mode, no writes, exit 1 on diff); `--stdin` for editor/agent
  integration.
- **Migration engine:** `delulu fmt --migrate <version>` applies mechanical cross-version
  renames/rewrites (first user: Stage 7's `consume`/`recover`, DL1608). Migrations are
  AST-transformations shipped with the compiler, never regex.

---

## 5. `delulu test` — authority-isolated test runner

### 5.1 Test manifests

```toml
# delulu.toml
[test-authority]                 # ceiling for ALL tests in this package (default: pure)
effects  = ["Read"]
fs.read  = ["./tests/fixtures"]
```

Per-file override header (first item in a test file):
`test authority ! {Read} { fs_read: ["./tests/fixtures"] }` — must be ⊑ the package
`[test-authority]` (DL1703 otherwise). Tests requesting nothing run pure — the common case.

### 5.2 Execution contract

- `delulu test [pattern] [--json] [--engine interp|wasm] [--jobs N]` — each test file is a
  process-isolated run under a broker child node carrying exactly its manifest (Stage-5 tree:
  test nodes are children of a `test-session` node; the session ends with transitive
  revocation — nothing a test leaked survives the run).
- Deterministic by default: `Cap[Clock]` fixed, `Cap[Rand]` seeded per test name hash; `--seed`
  overrides; a test needing real time/entropy must grant-declare it (visible in review).
- JSON report (stable schema): per-test `{ name, file, span, status: pass|fail|panic, ms,
  failure?: {expected, actual, span}, effects_traced: […] }` — `effects_traced` comes from
  `--trace-effects` always-on in test runs: **a test's actual effect list is part of its
  report** (review surface; an agent-written test that quietly reads the network is visible even
  when it passes).
- Exit codes: 0 all pass / 1 failures / 2 internal.

---

## 6. Localization: catalogs, picker, welcome

> **Companion documents (Fable-5 planning pass):** `docs/design/LOCALIZATION_PLUGIN_GUIDE.md` (the
> full authoring/add/remove/edit guide for human-language plugins — treat it as the requirements spec
> for this section's implementation); `docs/lang/<locale>.md` (per-language packs: en-US and
> delulu-slang complete, 9 starters with decided terminology); `docs/design/SYNTAX_MORPH_SPEC.md`
> (the *separate* mechanism for re-skinning the programming language's own keywords — including
> AI token-minimizing profiles — via bijective morphs; plan a §6.5-style `delulu morph` sibling of
> `delulu locale` when building); `docs/design/AI_NATIVE_DESIGN.md` (why the machine envelope stays
> frozen under all of this).

### 6.1 Catalog format

`catalogs/<locale>.toml`, embedded in the binary for the two shipped locales:

```toml
[meta]
locale   = "delulu-slang"
version  = "1.0.0"
fallback = "en-US"

[DL0501]
message = "fn `{fn}` is out here doing `{effect}` but its row said no cap 💀"
label   = "this call does `{effect}`"
help    = "add `{effect}` to the row (that's an authority W, so CI might veto) or drop the call fr"

[cli.grant-prompt.title]
message = "this program wants: {summary} — u good with that?"
```

- Keys are **stable ids** (diagnostic codes + named CLI strings); placeholders are typed
  (`{fn}`, `{effect}` — unknown placeholder in a catalog: DL1704 at catalog load, entry falls
  back).
- **Fallback chain:** chosen locale → `en-US`. `en-US` is complete by construction (the compiler
  refuses to build with a missing en-US key — build-time check, not runtime).
- Coverage honesty (normative): `delulu-slang` ships 100% of CLI strings and the top-priority
  diagnostic set (all DL01–08xx short messages); long-form `delulu explain` docs are `en-US`
  only in v1.0 except the 50 most-hit codes — the catalog `[meta]` declares coverage and the
  picker says so.
- **Catalog plugins:** a `kind = "plugin"` package with class `verified`, zero authority
  (`effects = []`), exporting `catalog() -> Str` (the TOML). Installed via
  `delulu locale add <file.dpx>` — the plugin machinery's first zero-authority dogfood. AI-
  generated community translations are the intended path (Constitution §8.5).

### 6.2 Selection

Priority: `--locale X` > `DELULU_LOCALE` env > `~/.delulu/config.toml` `locale` key > first-run
picker (TTY only) > `en-US`. The picker is two lines, once:

```
pick your compiler's vibe (changes human text only — codes & JSON never change):
  1) English (US)   2) Delulu Slang        [1]:
```

### 6.3 The first-run welcome (normative placement; text is law)

Shown **once**, immediately after the picker, only when: interactive TTY, no `CI` env, no
`--json`, no `DELULU_NO_FIRST_RUN`; then `~/.delulu/config.toml` records `welcomed = true`.
Rendered in a plain box (no color requirements beyond terminal default; emoji passthrough), in
**both locales identically** — the note is never translated, never paraphrased, never altered by
any catalog (a catalog key attempting to override it is DL1704). The text, byte-exact:

> U r here becoz u maybe a delulu like me & wanna create something others think is not possible. There's nothing wrong with being delulu. Anyway, u can't decide what others think abt u. So start building with everything u've got. Welcome to the Delulu Gang🐦‍🔥🔥🫡🚀

Attribution line beneath, exactly: `— Jesse, The Creator of DeluluLang`. It never appears in any
machine output, log, trace, LSP payload, or CI stream (§9.6 tests all five).

---

## 7. Signing and registry client (groundwork for Stage 9)

- `delulu keygen` (ed25519, `~/.delulu/keys/`, 0600); `delulu sign <artifact>` /
  `delulu verify-sig <artifact> --key <pub>` for `.dwx`/`.dpx`/package tarballs (the Stage-6
  `delulu:sig` section generalized to all artifacts).
- **Registry index format (normative now, hosted later):** sparse HTTP index, cargo-style:
  `/index/<pkg-prefix>/<name>` returns JSONL — one line per version:
  `{ name, version, content_hash, authority_hash, effects, cap_kinds, scopes, api_row_hash,
  sig?, yanked }`. **The index line carries the authority summary** — `delulu add` can show the
  authority diff *before* downloading anything (the supply-chain UX no other registry has).
- `delulu publish --dry-run` validates: manifest completeness, semver-authority law vs. the
  index's prior line (DL1003 reused), signature present. Actual upload endpoints are Stage 9.
- `delulu add <pkg>` resolves via index, shows the authority summary + diff against the
  requested pin, writes `delulu.toml` + lockfile through the Stage-2 machinery unchanged.

---

## 8. Standard library additions

Prelude: `assert`, `assert_eq` (§2). Nothing else.

## 9. Acceptance criteria

1. LSP smoke suite (automated client): open the reference workspace → diagnostics arrive with
   codes/spans equal to `delulu check --json`; hover on `greet` shows `fn(Cap[Console], Str)
   -> Unit ! {Write}`; the DL0501 code action applies the repair and re-check goes green;
   rename of a cross-package symbol updates both packages.
2. Authority lens: inlay hints appear on unannotated lambdas; the ⚠-prefixed widening action is
   distinguishable in the payload (`authority_widening` surfaced in `data`).
3. Latency: ≤ 150 ms edit-to-diagnostics on the 10-kLoC reference workspace, CI-measured.
4. `fmt`: identity + idempotence laws hold over conformance + fuzz corpora (≥ 100k programs);
   `--check` exits 1 on an unformatted file; `--migrate 0.7` passes the Stage-7 criterion 10
   corpus.
5. `test`: the demo suite runs pure tests with zero grants; a test calling `http.get` without
   test-authority fails DL0501 at compile or DL0701 at session grant; per-test
   `effects_traced` appears in the JSON report; session-end revocation verified via
   `delulu grants tree`.
6. Welcome/picker: shown exactly once on a fresh TTY home; **never** with `--json`, with
   `CI=1`, with piped stdout, with `DELULU_NO_FIRST_RUN=1`, or on second run (five automated
   cases); text byte-equal to §6.3 (hash-pinned in the test).
7. Locale invariance: the full conformance JSON output is byte-identical under `en-US` and
   `delulu-slang` (machine fields); the human text differs; DL0501's slang rendering matches the
   catalog.
8. Catalog plugin: a third locale loads via `delulu locale add`, renders, falls back to en-US on
   its missing keys; a catalog attempting to override the welcome key → DL1704.
9. `delulu publish --dry-run` catches an authority-widening minor bump (DL1003) against a local
   index fixture; `delulu add` renders the authority summary from the index line alone (no
   download).
10. All prior suites green; `delulu fmt --check` and locale-invariance added to CI permanently.

## 10. Diagnostics (fresh range DL17xx)

| Code | Meaning | Repair |
|---|---|---|
| DL1701 | LSP/workspace configuration error | message-specific |
| DL1702 | formatter identity/idempotence violation (compiler-bug class) | none — file a bug |
| DL1703 | test authority exceeds package test ceiling | narrow — exact intersection |
| DL1704 | catalog invalid: bad placeholder / missing meta / attempts to override the welcome | fall back + warn |
| DL1705 | signature verification failed | none — `requires_human: true` |
| DL1706 | registry index line invalid / semver-authority conflict at publish | none — `requires_human: true` |

## 11. Honesty and threat-model caveats

- The LSP is analysis-only; it holds no leases and can effect nothing (its process needs no
  broker connection). Its *availability* is not a security property.
- Catalogs are prose: a malicious catalog can mislead a human reader (it cannot alter codes,
  repairs, or JSON). Catalog plugins are zero-authority and verified-class, which bounds them to
  exactly this prose surface — stated in `delulu locale add`'s confirmation prompt.
- Signing authenticates origin, not behavior (Stage-6 caveat, unchanged). The registry index's
  authority line is *the publisher's verified-at-publish claim*; consumers re-verify on first
  build (Stage-2 machinery) — trust-on-first-verify, not trust-on-read.
- Test determinism covers clock/rand caps; it does not make foreign code or the OS
  deterministic.

*Stage 8 gives the language its face — terminal-first, every editor, two voices, one law. Stage 9
freezes v1.0 and proves it. Stage 10 takes it to production stakes.*
