# The DeluluLang Stability Contract

**Status:** normative from 1.0. **Governs:** invariant 43 (spec §1), §2.2 (deprecation policy and
editions). **Enforced by:** the conformance coverage law (invariant 42), `DL1801`, `DL1802`, and the
CI gates named below.

This document says what we promise not to break, what we explicitly do not promise, and what
happens when something must change anyway. It is written to be *checkable*: every promise below
either has a mechanism enforcing it or is marked as a promise of process rather than of code.

---

## 1. What is stable

From 1.0, the following are stable. A breaking change to any of them requires a **major** version.

| Surface | What "stable" means |
|---|---|
| **Surface grammar** | A program that parses today parses tomorrow, with the same tree. New syntax may be *added*; existing syntax never changes meaning. |
| **Typing, effect, and authority rules** | The rules in `docs/reference/semantics-*.md`. A program that type-checks today still does, and a program refused today is still refused — with the same code. |
| **Diagnostic codes and repair ids** | **Add-only.** A code is never reused for a different meaning, never renumbered, and never silently retired. Its *prose* may improve; its *identity and trigger* may not. |
| **JSON schemas** | Versioned and **additive**. Fields may be added; existing fields never change type or meaning. Consumers must ignore unknown fields. |
| **DIR major version** | A DIR the toolchain accepts today it accepts tomorrow, within the same major. |
| **`delulu:cap` / broker protocol majors** | Wire compatibility within a major. |
| **CLI exit codes** | `0` success, `1` diagnostics/failure, `2` usage error. A command's exit code for a given outcome does not change. |
| **Manifest and lockfile formats** | Additive. A lockfile written by 1.x is readable by every later 1.y. |
| **Catalog key space** | A message key, once shipped, keeps its meaning and its placeholder set. |
| **The welcome text** | Byte-exact (Stage 8 pinned it). It is not "prose" for the purposes of §2 below. |

### 1.1 The coverage qualifier — read this before relying on anything

**A behavior not covered by a test is not stable, and the reference says so per item.**

This is invariant 42, and it is not a disclaimer — it is a mechanism. `docs/reference/` reports each
anchor's conformance status from a live run, and `delulu-conform --coverage` fails on any anchor
without both an accepting and a rejecting witness. Where the reference marks an item other than
`covered`, that item is **outside** the guarantees in §1 until it is witnessed.

Today that is a real, published list, not a hypothetical: see `docs/reference/coverage.md` and the
classification in `STAGE9_BUILD_ORDER.md` D10. Release criterion 1 requires the list to be empty
before 1.0 ships, which is precisely why it is stated here rather than discovered later.

## 2. What is explicitly NOT stable

Naming these matters as much as naming the promises. If it is not in §1, it is not promised, and in
particular:

- **Human-facing prose.** Diagnostic messages, help text, explain bodies, and the Book. Keyed
  machine output (`--json`) is stable; the sentences a human reads are free to get better.
  *(The welcome text is the deliberate exception — §1.)*
- **Performance.** Speed, memory, and compile times are measured and published (Study C), never
  promised. A release may be slower; the numbers say so.
- **Trace record ordering.** The *set* of effects in a trace is law (invariant 12 — `--assert-trace`
  enforces it). The order records appear in is not.
- **Anything marked experimental** in the reference.
- **Internal crate APIs.** The Rust crates are an implementation detail; the stable interface is the
  language, the CLI, and the machine schemas.
- **Isolation profile *strength* on a given platform.** The *label* is honest and stable (a profile
  that is unavailable is refused or reported as a weaker fallback, `DL1408`); the underlying
  mechanism may improve.

## 3. Deprecation policy

A deprecation is a **warning, never a break**.

1. **RFC-gated.** No deprecation lands without a merged RFC. A deprecation nobody argued for in
   public is not policy.
2. **`DL1801`** is emitted at every use, naming the version it was deprecated in, its RFC, and — if
   the migration is mechanical — the exact replacement. Where the migration needs judgement, the
   diagnostic says so and points at the RFC rather than fabricating a substitute.
3. **`delulu fmt --migrate <version>`** performs the mechanical rewrites.
4. **At least two minor versions** must pass between deprecation and any removal.
5. **Removals are major-version-only.**

The deprecation registry lives in `delulu_check::deprecation::DEPRECATIONS`. **It is empty at 1.0**
— a language deprecating parts of itself on release day would be announcing that the freeze it just
declared is not real. The *mechanism* is complete and tested against synthetic tables, so the first
real deprecation is a data change rather than a feature built under pressure.

## 4. Language editions

`[package] language = "1.x"` pins the language edition a package is written against.

- **Optional.** A package that declares nothing is read at the toolchain's own edition. This is the
  default and it is back-compatible with every manifest written before 1.0.
- **An older edition is always fine.** Minors are strictly additive, so a 1.0 package means exactly
  what it said under a 1.4 toolchain.
- **A newer edition is refused** — `DL1802`, with the exact repair (upgrade the toolchain). The
  build is refused rather than attempted because compiling code written for newer rules under older
  ones would not fail loudly; it would silently reinterpret the program.
- Checked for **every package in the graph**, not only the root: a dependency from the future is
  exactly as unreadable as a root from the future.

## 5. How a change is classified

| Change | Version bump | Notes |
|---|---|---|
| New syntax, new diagnostic code, new JSON field, new CLI subcommand or flag | **minor** | Strictly additive only. The language edition moves with these. |
| Improved diagnostic prose, faster compile, better repair text | **patch** | Explicitly not stable (§2). |
| Fixing a diagnostic to fire where the spec always said it should | **minor**, and it must be in the release notes | Pre-1.0 this is free; after 1.0 it changes an observed code and is called out. |
| Removing a deprecated feature | **major** | Only after ≥2 minors of `DL1801`. |
| Changing what a program means | **major** | And it needs the §10 entrenchment analysis if it touches Constitution §1, §2, or §5.14. |

### 5.1 The pre-1.0 window, and why it is used

Between now and the 1.0 cut is the **only** time a code that was allocated but never reachable can
be made to fire without it being a breaking change. Stage 9 used that window deliberately:
`DL0203`, `DL0408` and `DL0601` were registered but shadowed by more general codes — the rules were
enforced, but under `DL0201`/`DL0401`, so the specific codes were unreachable. They now fire.

That was not tidying. Freezing an unreachable code at 1.0 has two bad ends: an agent keying off
`DL0408` never sees it, and any later fix that starts emitting it becomes a breaking change. The
window closes at 1.0; `STAGE9_BUILD_ORDER.md` D10 records what remains and its disposition.

## 6. Enforcement

| Promise | Mechanism |
|---|---|
| Diagnostic codes are add-only | `delulu-conform --coverage` (every registered code is anchored); the reference's diagnostics chapter is generated from the registry |
| The reference matches the implementation | `delulu-conform --check-reference` — a **hard CI gate** |
| Coverage never silently regresses | `coverage_never_regresses` — a ratchet whose floor may only rise |
| Typing/effect/authority rules hold | The conformance suite (`tests/conformance/`) + the laundering suite (audit rules R-1…R-7) |
| `--json` stays byte-identical for untouched programs | The locale-invariance and byte-identity tests in the CLI suites |
| Deprecation policy | `DL1801` + `delulu_check::deprecation` tests |
| Editions | `DL1802` + the edition tests |
| CLI exit codes and refusals | `crates/delulu/tests/cli_contract.rs` |

## 7. What this contract does not do

It does not make the implementation correct. It makes the implementation's *promises* explicit,
mechanically checked where a mechanism exists, and honestly bounded where one does not. Soundness
claims remain design-level plus audit-rule plus test-enforced; the Delulu Core mechanization is open
work, and the release notes say so.

A stability contract is a statement about what we will do when we are wrong — not a claim that we
will not be.
