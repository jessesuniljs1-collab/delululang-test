# DeluluLang for machines

**The one page an agent harness should pin.** Everything here is part of the stability contract
(`docs/design/STABILITY.md`) unless marked otherwise. Anchors on this page are stable.

---

## [agents.start] Start here

```
delulu check <file|package> --json     # diagnostics, with typed repairs
delulu authority <file|package> --json # what this program can do
delulu run <file> --json --no-prompt   # run, never block on a human
delulu test --json                     # run tests under an authority ceiling
delulu explain DL0501                  # long-form docs for any code
```

Three rules that will save you time:

1. **`--json` is the contract; the human text is not.** Prose may improve in any release. The JSON
   envelope's existing fields will not change meaning.
2. **`--no-prompt` everywhere.** Without it, a command that needs a grant may wait for a human.
   With it, the command refuses instead of hanging.
3. **Read the exit code, not just the output.** A verdict string and an exit code that disagree is
   a bug — one such bug is on record (`docs/security/DRILL-001.md`), which is why this warning is
   here.

## [agents.exit-codes] Exit codes

| Code | Meaning |
|---|---|
| `0` | Success. The command did what was asked. |
| `1` | Diagnostics or a failed verification. The program is wrong, or the check did not pass. |
| `2` | Usage error. Your invocation was wrong; the program was never examined. |

**`1` and `2` mean genuinely different things.** `2` means fix your command line; `1` means fix the
code. Never collapse them.

## [agents.json-envelope] The JSON envelope

Every `--json` command emits one object:

```json
{
  "command": "check",
  "schema": 1,
  "delulu_version": "0.1.0",
  "diagnostics": [ ... ],
  "summary": { "errors": 1, "warnings": 0 }
}
```

- `schema` is versioned and **additive**. New fields may appear; existing ones will not change type
  or meaning. **Ignore unknown fields** — that is what makes the additive promise usable.
- `summary.errors == 0` is the machine-readable definition of "this passed".

### [agents.locale-invariance] The machine surface is locale-invariant

Setting a locale changes human prose and **never** the JSON. Codes, field names, spans, and repair
ids are identical under every locale (invariant 39). If you see a translated key in `--json`, that
is a bug — report it.

## [agents.diagnostics] Diagnostics

```json
{
  "code": "DL0501",
  "severity": "error",
  "message": "function `f` performs effect `Write` not declared in its row",
  "explanation_id": "E-DL0501",
  "spans": [{ "file": "src/main.delulu",
              "start": { "byte": 102, "line": 5, "col": 4 },
              "end":   { "byte": 103, "line": 5, "col": 5 },
              "label": "declared row is here" }],
  "repairs": [ ... ]
}
```

- **`code` is the stable identity.** Match on it, never on `message`.
- **Byte offsets are the truth**; line/column are derived for humans. Edit by byte range.
- `explanation_id` feeds `delulu explain`.

## [agents.repairs] Repairs — and the one you must not apply

```json
{
  "id": "add_effect_to_row",
  "confidence": "exact",
  "authority_widening": true,
  "requires_human": false,
  "edits": [{ "file": "…", "range": { "start_byte": 123, "end_byte": 123 }, "insert": "! {Write} " }]
}
```

Apply edits **back to front** by `start_byte`, so earlier offsets stay valid.

> ### `authority_widening: true` means DO NOT APPLY AUTOMATICALLY.
>
> A widening repair silences a diagnostic by giving the program **more authority**. It is exact and
> it is correct for a human who has decided the effect belongs there. A machine applying it has not
> fixed the program — it has removed the objection. Surface it to a human instead.

`requires_human: true` means the same thing for a different reason: the repair needs a judgement
the tool cannot make.

**Honest coverage number:** as measured in Study B, **8.3%** of real defects currently offer a
machine-applicable repair, and a loop with no model reaches a clean program on **none** of them —
either the only repair widens authority (refused above) or it addresses a warning while the error
stands. Do not build a harness that assumes repairs will get you to green. See
`measurements/study-b/REPORT.md`.

## [agents.authority] The authority report

```
delulu authority <file|package> --json
```

```json
{ "authority": {
    "program": "greeter",
    "effects": ["Write"],
    "capabilities": [{ "kind": "Console", "scopes": ["stdio"] }],
    "secrets": ["API_KEY"],
    "foreign_calls": [],
    "foreign_isolation": "inproc",
    "custody": "embedded",
    "modules": [...], "pure_functions": [...] } }
```

This is the identity claim in one command: **everything this program can do, computed from the
code**. If `effects` does not list it, the program cannot do it — with one honest exception,
`foreign_calls`, which are holes in the proof and are enumerated as such rather than hidden.

**Credential exposure — the join you are expected to make.** These fields are deliberately raw; the
conclusion that matters is a join across three of them:

> `Declassify` ∈ `effects` **and** `secrets` non-empty **and** egress reach
> (`foreign_calls` non-empty, or `Net`, or `Write` ∈ `effects`)
> ⟹ *a credential can leave this program.*

`Declassify` means `expose(Cap[Declassify])` **can** be called — a capability, not an observed
behaviour. Once a secret is exposed it is an ordinary `Str` and the type system stops tracking it, so
this join is the last static point at which the question can be asked. Compute it before you accept a
program, generate one, or grant `declassify`; the human-readable report prints the same conclusion as
an `exposure:` line. (`STAGE4_SPECIFICATION.md` §6.1.)

If you are **writing** DeluluLang rather than auditing it: a hardcoded credential in a string literal
is not a secret and the language cannot know it was meant to be one. Secrets enter through
`root.secret("NAME")`, whose value the human supplies at grant time. `Secret[T]` is opaque —
`str(s)` is DL0604, `a == b` is DL0605, and it cannot cross the foreign boundary (DL1301) — so a
credential you thread as a `Secret[T]` cannot be printed, logged, compared, or marshalled by accident,
which is the property worth having in generated code.

`delulu authority --diff <old.lock> <new.lock> --json` reports whether an upgrade widened
authority. This is the check to run in CI on every dependency bump.

## [agents.effects] The effect kinds

`Read`, `Write`, `Net`, `Clock`, `Rand`, `Declassify`, `ForeignCall`. A module may declare more with
`effect`.

**Kind is static; scope is runtime.** The type proves what *kind* of thing a function can do; the
capability decides *which* file or host. Do not report path-level guarantees as static — they are
not, and saying so would be a claim the language deliberately refuses to make.

## [agents.tests] Running tests

```
delulu test --json
```

Tests hold **no ambient authority** (invariant 41): each gets exactly its declared row, bounded by
the package's `[test-authority]` ceiling. An absent ceiling means PURE. Exceeding it is `DL1703`.

## [agents.registry] Registry

```
delulu add <pkg> --index <dir|url> --json   # authority BEFORE download
delulu login --registry <url> --token <t>   # never echoes the token
```

The index line's authority is **recomputed server-side from the artifact**, so it is the artifact's
authority and not the publisher's claim (`docs/design/REGISTRY_POLICY.md`). Read it before you
install anything.

## [agents.determinism] Determinism

`delulu run --seed N --clock fixed:MS` fixes randomness and time. `delulu test` is deterministic by
construction. Byte-identical output across runs is a tested property for `--json`, the atlas, and
plugin inspection — if you see nondeterminism there, it is a bug.

## [agents.env] Environment variables

| Variable | Effect |
|---|---|
| `DELULU_NO_FIRST_RUN=1` | Suppress the first-run picker and welcome. **Set this in CI.** |
| `DELULU_HOME` | State directory (keys, credentials, locales). Isolate it per job. |
| `DELULU_COLOR=never` / `NO_COLOR` | No ANSI. `--json` is never colored regardless. |
| `DELULU_BROKER` | `embedded` or `daemon`. |

## [agents.limits] What to tell your users honestly

Repeating what the rest of the project says, because a harness author is the person most likely to
overstate it downstream:

- **The guarantee is about the authority boundary, not intent.** A dependency that was always
  granted `Net` and starts using it differently is not caught.
- **Foreign code is outside the proof.** `ForeignCall` is a hole, enumerated in the report.
- **Soundness is design-level plus audit-rule plus test-enforced.** The Delulu Core mechanization is
  open work.
- **Not everything is covered yet.** `docs/reference/coverage.md` reports per-item conformance
  status from a live run. Items marked other than `covered` are outside the stability promise.
- **Performance is measured, never promised** — `measurements/study-c/REPORT.md`.
  v1.0 is **not competitive with C** on the measured workloads (2.0×–60.5× slower), and the report
  says so in those words. Do not let a harness's marketing copy imply otherwise.
