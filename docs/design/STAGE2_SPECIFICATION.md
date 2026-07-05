# DeluluLang — Stage 2 Implementation Specification

**Version:** 0.2 ("Provenance"). **Status:** Committed — buildable directly from this document.
**Depends on:** Stage 1 complete (all 14 acceptance criteria green).
**Governing documents:** `CONSTITUTION.md`, `SOUNDNESS_AUDIT.md`. Where they conflict, they win.

---

## 0. Scope and goal

**Goal:** turn the single-file language into a **multi-package language whose dependency graph
carries compiler-verified, version-locked authority** — the supply-chain guarantee of
Constitution §5.6 made real — and harden the checker from "passes its tests" into a
semi-formalized, fuzz-tested, trace-witnessed artifact.

**In scope (must ship):**
- Packages: directory layout, package manifests, `path` and `git` dependencies, visibility rules,
  re-exports.
- **Package authority verification**: computed authority per package, checked against its
  manifest; **authority pins** on dependencies; the semver-authority law.
- The **authority lockfile** (`delulu.lock`): versions + content hashes + verified authority +
  public-API row hashes.
- **Effect tracing** (`--trace-effects`, `--assert-trace`): the executable soundness witness.
- **Delulu Core**: the formal core calculus document + property-based differential fuzz harness.
- New CLI: `delulu build`, `delulu lock`, `delulu why`, `delulu explain`,
  `delulu authority --diff`.
- New diagnostics: DL10xx (packages/authority versioning), DL11xx (trace/fuzz harness).

**Non-goals (later stages):** registry hosting and package publishing UX (Stage 8), WASM backend
(Stage 3), FFI (Stage 4), broker process (Stage 5), plugins runtime (Stage 6), concurrency
(Stage 7).

**Who this stage serves:** it is the *review* stage. Humans reviewing AI-written dependencies get
`delulu why` and `delulu authority --diff`; agents get trace-verified feedback loops. This is the
first stage where DeluluLang does something no mainstream toolchain can do at all.

---

## 1. Invariants (carried + new)

All nine Stage-1 invariants continue to hold, plus:

10. **Package authority never widens silently.** Any observable widening (effects, capability
    kinds, scopes, or public-API rows) fails the build unless the version change is major *and*
    the consumer explicitly re-accepts (§5.4).
11. **The lockfile is authority, not just versions.** A locked dependency whose verified
    authority no longer matches its lock entry cannot build (DL1002), even if its version string
    is unchanged — content hash and authority hash are the truth.
12. **Trace ⊆ row, always.** Every effect the interpreter performs is logged in trace mode, and
    on every conformance/fuzz run the trace must be a subset of the statically computed row
    (§7.2). A single violation is a stop-the-world compiler bug.
13. **Holder-model compatibility.** Nothing in package resolution introduces ambient authority:
    dependencies receive capabilities only through their public interfaces, exactly like all
    other code (no build scripts with I/O — see §5.6, this is normative).

---

## 2. Lexical and grammar additions

One production changes: re-exports. No new tokens, no new keywords.

```ebnf
item          = [ "pub" ] , ( fn_decl | type_decl | effect_decl | const_decl ) ;
import_decl   = [ "pub" ] , "import" , path , [ "as" , IDENT ] , term ;   (* CHANGED: pub import *)
```

- `pub import a.b` re-exports module `a.b`'s public items from the importing module.
- `pub import a.b as c` re-exports under alias `c`.
- Re-export cycles are DL1005 (same detector as import cycles, DL0304, but package-aware).

**Path meaning is now two-level:** in `import x.y.z`, `x` resolves first among (1) the current
package's module tree, then (2) declared dependency names. Shadowing a dependency name with a
local directory is DL1006 (ambiguity is refused, never resolved silently — agents must never
guess what an import meant).

### 2.1 AST delta

```rust
pub struct Import {
    pub public: bool,            // NEW: re-export
    pub path: Path,
    pub alias: Option<Ident>,
    pub span: Span,
}
// NEW: package-qualified definition identity (replaces Stage-1 file-local DefId)
pub struct PackageId(pub u32);
pub struct DefId { pub package: PackageId, pub index: u32 }
```

All symbol tables, side tables, and diagnostics move from file-local ids to `DefId`. Spans gain a
package-relative file path so diagnostics remain stable across machines
(`"file": "dep:mathkit/src/stats.delulu"` — the `dep:` prefix is part of the contract).

---

## 3. Package layout and manifests

### 3.1 Layout

```
mypkg/
  delulu.toml
  src/
    main.delulu          # bin package: has fn main(root: Root)
    lib.delulu           # lib package: root module (exactly one of main/lib required)
    util/stats.delulu    # module mypkg.util.stats
  tests/                 # .delulu test files (run by `delulu test` — Stage 8; dir reserved now)
```

### 3.2 Manifest (`delulu.toml`) — full Stage-2 schema

```toml
[package]
name    = "mypkg"           # [a-z][a-z0-9_]*, unique in the dependency graph
version = "0.2.1"           # semver, mandatory
kind    = "bin"             # "bin" | "lib"

[authority]                  # the package's own declared ceiling (Stage-1 semantics, unchanged)
effects  = ["Read", "Write"]
fs.read  = ["./config"]
fs.write = []
net      = []
secrets  = []

[dependencies]
mathkit = { path = "../mathkit" }
webby   = { git = "https://github.com/x/webby", rev = "9f2c41d" }   # rev/tag mandatory; branch forbidden

[dependencies.webby.authority]   # AUTHORITY PIN: consumer's ceiling for this dependency
effects = ["Net"]
net     = ["api.example.com"]

[dependencies.mathkit.authority]
effects = []                     # pinned pure — the strongest supply-chain statement possible
```

Normative rules:
- **A `git` dependency must pin `rev` or `tag`** — floating branches are DL1007. (Trust note:
  hash-pinning mitigates, not eliminates, git-host trust; recorded in §11.)
- **Every dependency must carry an authority pin.** A missing pin is DL1001 with an exact repair
  (insert the dependency's manifest-declared authority as the pin) flagged
  `authority_widening: true` — accepting a dep's own claim is a widening *decision*, made
  visible, never a default.
- Dependency graphs are acyclic (DL1005) and diamond-shared (one copy per package name; version
  conflicts are DL1008 — no silent duplication, because duplicated packages would double
  authority invisibly).

### 3.3 No build scripts — normative

DeluluLang packages have **no install hooks, no build scripts, no code execution at resolution
time**. Resolution and verification are pure functions of source text. (This is where npm/PyPI
supply-chain attacks live; the language refuses the category. Codegen use-cases wait for
verified plugins, Stage 6.)

---

## 4. Package authority: computation and verification

### 4.1 Computed authority (formal)

For package `P` with public functions `pub(P)` (including functions reachable through re-exports),
define:

```
row(P)    = ⋃ { row(f) | f ∈ reachable(pub(P)) }          // reachable = pub + transitive private callees
kinds(P)  = { R | some f ∈ reachable(pub(P)) takes or returns Cap[R] or Plugin[_] }
apiRows(P) = the map { f ↦ (type(f), row(f)) | f ∈ pub(P) }
```

`row(P)` uses declared rows (sound by Stage-1 T-Fn: declared ⊇ body). Row variables in generic
functions contribute their *closed* labels only; the variable itself is the caller's
contribution and is excluded from `row(P)` (a `map`-like function does not make a package
effectful — its callers' rows account for callbacks, per T-Call and R-4).

### 4.2 Verification (build-time, per package)

1. **Self check:** `row(P) ⊆ manifest(P).effects` and `kinds(P) ⊆ kinds(manifest(P))`, else
   DL1009 (package exceeds own manifest; repair: widen manifest — `authority_widening: true`).
2. **Pin check:** for every dependency `D` of `P`:
   `verifiedAuthority(D) ⊑ pin(P → D)` (attenuation order from `SOUNDNESS_AUDIT.md` §A.3),
   else DL1001 (dep exceeds pin; span points at the pin *and* at the offending function in `D`,
   using the `dep:` file prefix).
3. **Whole-program check (bin packages):** Stage-1 DL0701 unchanged, now over the full graph:
   `row(main) ⊆ manifest(root).effects`, and every granted scope covers the union of dependency
   scope requirements.

Scopes remain **runtime-enforced, manifest-declared** (kind vs. scope split, Constitution §5.3) —
the pin's scope lists are load/grant-time ceilings, not compile-time proofs. Stated in every
relevant diagnostic's explanation text.

### 4.3 The semver-authority law (normative)

Between two versions `old`, `new` of a package:

| Change | Required version bump |
|---|---|
| Authority narrowing, API unchanged | patch or higher |
| API additions, authority unchanged or narrower | minor or higher |
| **Any authority widening** (effects, kinds, scopes) | **major** |
| **Any public-API row widening** (a `pub` function's row grows) | **major** |

`delulu lock` enforces this against the previous lockfile: a violating upgrade is DL1003 and
cannot be locked without both a major version and the explicit flag
`--accept-authority <pkgname>` (which rewrites the pin, is logged in the lockfile's `accepted_by`
field, and is flagged `authority_widening` in JSON output — CI can forbid it wholesale).

### 4.4 The lockfile (`delulu.lock`) — schema

```toml
version = 1

[[package]]
name            = "webby"
version         = "1.4.2"
source          = "git+https://github.com/x/webby#9f2c41d"
content_hash    = "blake3:af31…"        # over canonicalized source tree
authority_hash  = "blake3:77b0…"        # over the canonical JSON of verified authority (§4.1)
effects         = ["Net"]
cap_kinds       = ["Http"]
scopes          = { net = ["api.example.com"] }
api_row_hash    = "blake3:d10c…"        # over apiRows(P), canonical JSON
accepted_by     = ""                     # non-empty iff --accept-authority was used, records flag+date
```

Rules: builds verify `content_hash` before anything else (mismatch: DL1010); then recompute
authority and compare to `authority_hash` (mismatch: DL1002 — this catches "same version string,
different code"); `delulu.lock` is committed to VCS; `--locked` (default in CI mode) refuses any
resolution not already in the lockfile (DL1011).

---

## 5. Typing and visibility rules (delta)

- **VIS-1:** an item is cross-package-visible iff `pub` and exported from the package root module
  (directly or via `pub import` chain). Referencing a non-visible item is DL1012.
- **VIS-2 (explicit boundary):** every `pub` item in a `lib` package must have fully explicit
  types and rows — already Stage-1 law for functions; extended to `pub` consts and `pub` type
  aliases (no inference crosses a package boundary, ever).
- **VIS-3 (opaque propagation crosses packages):** the `opaque` property (audit R-5) is computed
  from the *defining* package and carried in the exported type metadata, so a consumer can never
  `str`/`==` a type whose hidden fields contain a `Secret`/`Cap`.
- **GEN-1:** generic `pub` functions are exported with their row variables; instantiation happens
  in the consumer per Stage-1 rules. The exported metadata format (§5.5) preserves rows exactly —
  invariant 3 (rows never erase) now spans serialization.

### 5.5 Package interface metadata

`delulu build` emits, per lib package, `target/<pkg>/interface.json`: every `pub` item with its
full type, row, opacity, and `DefId` — canonical, sorted, and hashed into `api_row_hash`. This
file is a machine surface (agents introspect dependencies without reading source) and the input
to `delulu authority --diff`.

---

## 6. Runtime changes

Interpreter semantics are unchanged. Two additions:

### 6.1 Effect tracing (`--trace-effects`)

Every primitive-table operation executed appends one JSON line to the trace stream (fd or file
via `--trace-out`):

```json
{ "effect": "Read", "cap_kind": "FsRead", "scope": "./config",
  "op": "read_text", "arg": "./config/app.txt",
  "span": { "file": "src/main.delulu", "line": 21 },
  "stack": ["main", "load_config"], "grant_id": 3, "seq": 41 }
```

- `arg` is **redacted** (`"«opaque»"`) for any operation whose argument set contains a secret-
  derived value; the trace is itself a `Write`-like surface and must not become the leak (the
  trace writer is runtime-internal, not a language sink, but R-5 discipline applies to it by
  policy).
- `--assert-trace`: at process exit, verify every trace line's `effect` ∈ the statically computed
  row of every frame in its `stack`. Violation → exit code 3 and DL1101 (compiler-bug class,
  message says exactly that). This is audit §C's argument, executed.

### 6.2 Deterministic replay seeds

`--seed <u64>` fixes `Cap[Rand]`; `--clock fixed:<ms>` fixes `Cap[Clock]`. Required by the fuzz
harness (§7) and generally the right tool for agent debugging loops. Both print into the trace
header. No semantic change — determinism knobs live in the runtime, not the language.

---

## 7. Delulu Core — formalization and the fuzz harness

### 7.1 The calculus (deliverable: `DELULU_CORE.md`)

A small-step core calculus covering: lambda + application, `let`, pairs/sums, closed and
row-variable effect rows, capability tokens (with kinds, no scopes — scopes are runtime data by
Constitution §5.3), `Secret` wrap/verify/expose with `Declassify`, and the primitive-op rule
labeled with effects. Machine-checkable statements (proved on paper in v0.2; mechanization is a
standing invitation in `CONTRIBUTING.md`):

- **Preservation:** typing is preserved by steps.
- **Effect soundness:** if `⟨e, σ⟩ ⇓ trace`, then `labels(trace) ⊆ row(e)`.
- **No forgery:** every capability token in a reachable configuration descends, by explicit
  passing or attenuation, from the initial store's root tokens.

The calculus deliberately **excludes** what Stage 1 excludes (mutation is modeled as store
passing; no plugins — the R-1 rule makes Contained plugins a *typed axiom*, and that honesty is
stated in the document).

### 7.2 The differential fuzz harness (`crates/delulu-fuzz`)

Pipeline, run in CI nightly and by `delulu-fuzz run -n <N>`:
1. **Generate** ASTs from the grammar (seeded, shrinkable), biased toward the audit's danger
   zone: higher-order functions, stored closures, row variables, `Secret`, capability threading.
2. **Filter** through the checker; keep both accepted and rejected programs.
3. **Accepted programs:** run with `--trace-effects --assert-trace --seed …` under a synthetic
   full grant. Any trace violation, panic-that-should-typecheck, or nondeterminism across two
   identical runs is a minimized, committed regression test.
4. **Rejected programs:** assert the diagnostic has a code, a span, and (where the code promises
   one) a repair that — when applied — produces a program the checker accepts (the **repair
   closure law**, DL1102 when violated).

Acceptance floor: ≥ 100k generated programs with zero violations before Stage 2 ships.

---

## 8. CLI additions and contracts

```
delulu build   [--locked] [--json]        # resolve → verify hashes/pins → check all packages
delulu lock    [--accept-authority <pkg>] # (re)compute delulu.lock; enforces §4.3
delulu why     <Effect|CapKind> [--json]  # shortest call path main → an op with that effect
delulu explain <DLxxxx|E-...>             # long-form explanation of a code (locale-keyed, Stage 8)
delulu authority --diff <old.lock> [--json]   # authority delta between lockfile states
delulu run     … --trace-effects [--trace-out F] [--assert-trace] [--seed N] [--clock fixed:MS]
```

- `delulu why Net` (human mode) prints the chain with spans:
  `main → fetch_prices (src/main.delulu:14) → webby.get (dep:webby/src/http.delulu:88) — Net`.
  JSON mode: `{ "effect": "Net", "paths": [[ {fn, file, line}, … ]] }`, all *minimal* paths, ≤ 10.
  This is the review command: "why does this program have network access" answered mechanically.
- `delulu authority --diff` output is the supply-chain review surface:

```json
{ "added_effects": { "webby": ["Write"] },
  "added_scopes":  { "webby": { "fs.write": ["./cache"] } },
  "removed": {}, "api_row_changes": { "webby.get": { "old": ["Net"], "new": ["Net","Write"] } },
  "verdict": "WIDENING — requires major version + --accept-authority" }
```

- Exit codes unchanged (0 ok / 1 diagnostics / 2 internal / **3 trace-assert violation**).

---

## 9. Diagnostics (fresh ranges; never reused)

| Code | Meaning | Repair |
|---|---|---|
| DL1001 | dependency authority exceeds pin (or pin missing) | insert/widen pin — `authority_widening: true` |
| DL1002 | locked authority hash mismatch (same version, different authority) | none — `requires_human: true` |
| DL1003 | semver-authority violation on upgrade | bump-major + re-pin — `authority_widening: true`, `requires_human: true` |
| DL1005 | package/re-export cycle | none |
| DL1006 | import ambiguous between local module and dependency | rename local module or alias dep — two exact repairs |
| DL1007 | git dependency without pinned rev/tag | pin to current rev — exact |
| DL1008 | version conflict for one package name in graph | none — `requires_human: true` |
| DL1009 | package exceeds its own manifest authority | widen manifest — `authority_widening: true` |
| DL1010 | content hash mismatch | none — `requires_human: true` (possible tamper; message says so) |
| DL1011 | `--locked` build requires unlocked resolution | run `delulu lock` — exact |
| DL1012 | item not visible across package boundary | add `pub`/re-export — `authority_widening: false` |
| DL1101 | trace-assert violation (compiler-bug class) | none — file a bug; message links SECURITY.md |
| DL1102 | repair closure law violated (fuzz-only) | none — compiler bug |

---

## 10. Standard library additions

None. Stage 2 adds no stdlib surface — discipline: the stdlib grows only when a stage's
acceptance criteria require it. (The `tests/` directory convention is reserved; `delulu test`
lands in Stage 8.)

---

## 11. Honesty and threat-model caveats (carry into docs verbatim)

- Pins and hashes verify **what the code declares and contains**, at kind granularity; scope
  ceilings bind at grant time, not compile time (Constitution §5.3).
- Git dependencies trust the git host's content-addressing; `content_hash` re-verification
  reduces this to "trusted at first lock" (TOFU). The registry with signed artifacts (Stage 8/9)
  tightens it.
- The fuzz harness and trace assertion are **evidence, not proof**; Delulu Core's paper proofs
  cover the calculus, not the implementation. Both statements appear in `DELULU_CORE.md`.
- No-build-scripts closes an attack *category*, not all supply-chain risk: a malicious dependency
  can still lie in wait inside its granted authority. `delulu why` and `--diff` exist because
  *bounded* malice still needs review.

---

## 12. Acceptance criteria (Stage 2 is done when all pass)

1. A three-package workspace (bin → lib → lib) builds; `interface.json` emitted per lib;
   cross-package call with a row-polymorphic generic works.
2. Adding one `!{Write}` function to a pinned-pure dependency fails the consumer's build with
   DL1001, spans on both the pin and the offending function.
3. Editing a locked dependency's source without version change → DL1010 (hash) — and if hashes
   are regenerated maliciously with same version, DL1002 (authority hash) when authority changed.
4. An upgrade that widens `webby` from `["Net"]` to `["Net","Write"]` with a minor bump → DL1003;
   with a major bump + `--accept-authority webby` → locks, and `accepted_by` is recorded.
5. `delulu why Net` prints the exact minimal chain on the reference workspace; `--json` schema
   validates.
6. `delulu authority --diff` on criteria-4's two lockfiles reports the widening with verdict.
7. The xz-style scenario test: a "compression" lib dependency gains a function whose row includes
   `Net` in a patch release — the build refuses at DL1001/DL1003 *before any code runs*.
8. Full Stage-1 conformance suite passes under `--trace-effects --assert-trace` (exit 0).
9. Fuzz harness: ≥ 100k programs, zero DL1101/DL1102, shrinker produces minimal repros.
10. `DELULU_CORE.md` committed with the three theorem statements and paper proofs; every audit
    exploit (F-1…F-6) appears as a rejection/containment test in the suite.
11. `pub import` re-exports work; DL1005/DL1006 fire on the crafted cycles/ambiguities.
12. All green on Windows, macOS, Linux CI.

## 12a. Implementation status (2026-07-05)

Stage 2's **provenance core is implemented and green** (121 workspace tests). Delivered:
cross-module resolution with a global type registry (`program.rs`), the package-authority
self-check (DL1009), cross-**package** path-dependency resolution + one-registry workspace check
(`deps.rs`: DL1005/DL1006/DL1007/DL1008), authority pins (DL1001, attenuation-order effect+scope
subset, missing-pin repair flagged `authority_widening`), the `delulu.lock` file with blake3
content/authority/api-row hashes (`lockfile.rs`: DL1010/DL1002/DL1011) and the semver-authority
law (DL1003 + `accepted_by`), plus effect tracing and deterministic replay (`trace.rs`, §6). CLI:
`delulu build [--locked]`, `delulu lock [--accept-authority <pkg>]`, and `run` flags
`--trace-effects`/`--trace-out`/`--assert-trace` (DL1101, exit 3)/`--seed`/`--clock fixed:MS`.
Acceptance criteria 2, 3, 4, 7, 8 are met and tested (`crates/delulu/tests/provenance.rs`; the
xz scenario refuses at DL1001 before any code runs).

Implementation-forced deviations, recorded for honesty:
- **DL1004** "malformed package manifest" was registered (a free slot in the DL10xx range) for
  manifest syntax / missing-required-field errors — the §9 table skipped it.
- `[package] kind` is **optional, defaults to `bin`** (back-compat with Stage-1 single-package
  manifests that omit it).
- A dependency that carries a pin must use TOML **table form** (`[dependencies.<name>]` with an
  `authority` sub-table, or an inline `authority = { … }`); the illustrative §3.2 snippet mixing
  `x = { … }` with a following `[dependencies.x.authority]` is not valid TOML.
- **Git-dependency resolution is deferred, not faked**: unpinned git deps are DL1007; a pinned
  git dep is refused with a clear "resolution deferred" message rather than pretend-verified.

Phase 2a additions (2026-07-05, 127 tests green): criterion **1** `interface.json` emission
(§5.5 — written per package on `delulu build` to `<pkgdir>/target/<pkg>/interface.json` with pub
exports + api_row_hash); criterion **5** `delulu why <Effect>` (function-granularity origin path
from `main` — honest approximation of the op-level story, one path not all-minimal, documented in
code); criterion **6** `delulu authority --diff <old.lock> <new.lock-or-dir>` (per-package
effect/scope/api-row-hash diff with the WIDENING verdict — per-package rather than the spec's
per-function `api_row_changes`, noted); criterion **10** `DELULU_CORE.md` committed (Progress /
Preservation / Effect-Soundness theorems with paper-proof sketches + a traceability table; honesty
clause: sketches not mechanized, mechanization is open future work).

Not yet done: criteria **9** (fuzz harness ≥100k) and **11** (`pub import` re-exports — the AST
`Import` has no `public` field yet) — the last two Stage-2 increments. Also: `delulu authority
<dir>` still uses the single-package path (does not yet resolve cross-package imports). CI is
Windows-only so far (criterion 12 partial).

*Stage 2 makes the dependency graph a place where authority cannot hide. Stage 3 gives the
program a floor to stand on.*
