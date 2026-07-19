# DeluluLang Surface — the v0.8 Guide

**Status:** Living user guide. Normative: `STAGE8_SPECIFICATION.md`; rulings:
`STAGE8_BUILD_ORDER.md`; the Atlas and Palette have their own guide in
`SURFACE_ATLAS_PALETTE_ADDENDUM.md`.

Stage 8 gives DeluluLang its face — terminal-first for agents and humans, one LSP server
for every editor, one canonical formatter, an authority-isolated test runner, two voices,
Jesse's first-run welcome, and the signing + registry-client groundwork Stage 9 opens.
**Nothing here changes what programs mean** (invariant 38): the formatter is
AST-identity-preserving, the LSP reports what the compiler computed, catalogs change prose
and never meaning.

## 1. `test` blocks

```delulu
module app

fn double(n: Int) -> Int { n * 2 }

test "double doubles" {
  assert_eq(double(21), 42)
  assert(double(0) == 0)
}

test "writes need a declared row" ! {Write} {
  let out = test_root.console()
  out.println("only the runner prints this")
}
```

- `test "name" [! {row}] { … }` — a contextual keyword in item position (`fn test` and
  `let test` stay legal). The body types like a `Unit`-returning fn with `test_root: Root`
  bound; the declared row bounds it exactly like a function's (omitted = pure).
- `assert(cond: Bool)` / `assert_eq[T](a, b)` are pure prelude builtins. `assert_eq` on an
  opaque type (`Secret`/`Cap`/`Root`) is refused (DL0605) — comparing secrets in tests is
  refused like everywhere else. A failed assertion is a **DL1707** panic carrying both
  values.
- Tests are **compiled out** of every build and artifact: they never run under `delulu
  run`, never enter authority reports, and are stripped from a plugin `.dpx` (a hand-crafted
  DIR carrying one is refused at load).

## 2. `delulu test` — authority-isolated

```toml
# delulu.toml
[test-authority]              # the ceiling for ALL tests here (absent = pure)
effects  = ["Write"]
fs.read  = ["./tests/fixtures"]
```

```text
delulu test [paths|patterns]... [--json] [--seed N]
```

- Each test holds **exactly its declared row, bounded by the ceiling** — a pure test holds
  nothing (invariant 41). A row wider than the ceiling is **DL1703**; an undeclared effect
  is the ordinary **DL0501** at check.
- **Deterministic by default:** `Cap[Clock]` fixed, `Cap[Rand]` seeded per test-name hash
  (`--seed` overrides). Repeated runs are byte-identical (modulo timing).
- Every test's **actual effect list is in its JSON report even on pass** (`effects_traced`)
  — an agent-written test that quietly reads the network is visible even when it's green.
- Under a live broker the run lives in a `test-session` node with a child per file,
  transitively revoked at session end — nothing a test leaked survives.

## 3. `delulu fmt` — one style, zero options

`parse(fmt(src)) ≡ parse(src)` (identity, comments preserved) and `fmt(fmt(src)) ==
fmt(src)` (idempotence) — both verified inline before any write; a violation is **DL1702**
(a compiler bug) and the file is left untouched. `delulu fmt --check` exits 1 on an
unformatted file; `--stdin` formats to stdout; `--migrate 0.7` renames pre-0.7
`consume`/`recover` identifiers. Unparseable input is refused with its real diagnostics,
never rewritten.

## 4. `delulu lsp` — one server, every editor

Stdio LSP 3.17. Diagnostics ARE the compiler's (same codes/spans/repairs as `check
--json`); code actions carry the typed repairs (⚠-titled and never preferred when they
widen authority); hover shows type + effect row + transitive authority; the inlay-hint
**authority lens** shows inferred rows on lambdas; definition/references/rename work across
open documents (locals refuse rename honestly); semantic tokens distinguish effects, rcaps,
capability types, and secrets; code lenses put `▶ run` / `authority: {…}` on `main` and
`▶ run test` on tests; `delulu.authority` returns the §10.5 report over the wire. See
`docs/editors.md` for the three-line config any editor or agent IDE uses.

## 5. Two voices

`--locale en-US | delulu-slang` (or `DELULU_LOCALE`, or `~/.delulu/config.toml`). A locale
changes **human prose only** — codes, spans, JSON, exit codes are byte-identical across
locales. The first-run picker + welcome appear once on a fresh interactive terminal and
**never** under `--json`, `CI`, a non-TTY, or `DELULU_NO_FIRST_RUN`. Community translations
ship as **catalog plugins**: `delulu locale add <file.dpx>` (verified-class, zero-authority
— the plugin machinery's first zero-authority dogfood); `delulu locale remove | list`.

## 6. Signing and the registry client

`delulu keygen` (ed25519, `~/.delulu/keys/`); `delulu sign <artifact>` / `delulu verify-sig
<artifact> [--key HEX]` — a detached `<artifact>.sig` over any `.dwx`/`.dpx`/tarball.
`delulu publish --dry-run <pkg> --index <dir>` validates manifest + semver-authority
(DL1003) + signature; `delulu add <pkg> --index <dir>` shows the authority summary from the
index line **before** downloading anything. Local index fixtures only in v0.8; hosted
operations are Stage 9.

## 7. Honesty and threat-model caveats (spec §11, verbatim)

- The LSP is analysis-only; it holds no leases and can effect nothing (its process needs no
  broker connection). Its *availability* is not a security property.
- Catalogs are prose: a malicious catalog can mislead a human reader (it cannot alter codes,
  repairs, or JSON). Catalog plugins are zero-authority and verified-class, which bounds them
  to exactly this prose surface — stated in `delulu locale add`'s confirmation prompt.
- Signing authenticates origin, not behavior (Stage-6 caveat, unchanged). The registry
  index's authority line is *the publisher's verified-at-publish claim*; consumers re-verify
  on first build (Stage-2 machinery) — trust-on-first-verify, not trust-on-read.
- Test determinism covers clock/rand caps; it does not make foreign code or the OS
  deterministic.

## 8. What v0.8 deliberately does not do

Hosted registry operations, governance, and the measurement program (Stage 9); IDE plugins
beyond the in-repo VS Code skeleton (community, via LSP); catalog translations beyond the
two shipped (plugins/AI-generated, by design); a full DefId graph for local-variable rename
(module-level names only); `delulu morph` (the syntax-skin sibling of `delulu locale` —
`SYNTAX_MORPH_SPEC.md` is written, building it is post-v0.8). The full ledger with reasons:
`STAGE8_BUILD_ORDER.md` §3.
