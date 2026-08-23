# DeluluLang for machines

**The one page an agent harness should pin.** Everything here is part of the stability contract
(`docs/design/STABILITY.md`) unless marked otherwise. Anchors on this page are stable.

---

## [agents.start] Start here

```
delulu check <file>... --json          # diagnostics, with typed repairs — MANY FILES, ONE PROCESS
delulu check <package> --json          # or a whole package, resolved together
delulu authority <file|package> --json # what this program can do
delulu run <file> --json --no-prompt   # run, never block on a human
delulu test --json                     # run tests under an authority ceiling
delulu explain DL0501                  # long-form docs for any code
```

Four rules that will save you time:

1. **`--json` is the contract; the human text is not.** Prose may improve in any release. The JSON
   envelope's existing fields will not change meaning.
2. **`--no-prompt` everywhere.** Without it, a command that needs a grant may wait for a human.
   With it, the command refuses instead of hanging.
3. **Read the exit code, not just the output.** A verdict string and an exit code that disagree is
   a bug — one such bug is on record (`docs/security/DRILL-001.md`), which is why this warning is
   here.
4. **Batch your `check` calls, and do not batch anything else.** Starting the process is the cost,
   not compiling: on Windows 82% of a small `check` is the operating system creating a process, and
   the compiler's own work on a 35-line file is under 1.5 ms. Twenty files in twenty invocations
   cost 711 ms; the same twenty in one cost 48. Still **one** envelope, with every diagnostic
   carrying the file it came from in `spans[].file`. Every *other* command takes exactly one path
   and refuses a second rather than silently using the first — measurements and the defect that
   found this are in [`measurements/agent-loop/RECORD.md`](../measurements/agent-loop/RECORD.md).

   If you are running a long edit→check loop, `delulu lsp` pays the process cost once and every
   check after that is the sub-millisecond part.

5. **Over `delulu lsp`, ask for what you need rather than shelling out per question.** The server is
   the same compiler, so its answers cannot drift from `delulu check --json`. Three of them are worth
   knowing about specifically:

   - `workspace/executeCommand` with `delulu.authority` returns the §10.5 authority report as JSON —
     what this program can do to the machine — without a second process. **Send this name, not
     `delulu.showAuthority`**: that one is the *editor's* command, which exists to render the report
     for a human, and the two are deliberately distinct (a client that registers the protocol name
     itself collides with its own language client and kills the server — see `docs/editors.md`).
   - `textDocument/formatting` runs the same formatter as `delulu fmt`, byte for byte, enforced by a
     test. Format through the server in a loop rather than spawning `fmt` per file.
   - `textDocument/codeAction` carries the checker's own typed repairs, with
     `data.authority_widening` and `data.requires_human` set, so a harness can apply the safe ones
     and refuse the rest **by policy rather than by parsing prose**. A repair that widens what the
     program may do is never marked preferred.

   The server is analysis-only: it never runs code, never loads a plugin, and holds no broker lease.
   A compromised workspace cannot use it as an effector.

### [agents.survey] If you are here to change the compiler, not to use it

One command answers "is this checkout healthy?" — `delulu doctor` checks the environment and, when
run inside the DeluluLang source tree, regenerates the repository map if it is behind and verifies
its integrity. `delulu doctor --json` emits one envelope; `--check` never writes.

That promise was **false until 2026-08-03** on the run you are most likely to care about. `doctor`
printed its envelope without recording that it had, so any run that *found a problem* exited nonzero
and collected a second, fallback envelope on top — two objects, exactly when the answer mattered. It
escaped every sweep because the contract gate checked that each named command appears in `--help` and
never the reverse, and `doctor` was named in no list at all. Both are fixed and both are now guarded:
the gate reads the dispatcher itself, so the next command cannot be born unswept.

This page is about *driving* DeluluLang. If you are modifying the implementation, start instead at
[`docs/survey/SURVEY.md`](survey/SURVEY.md) — a map of the repository generated from the repository,
where every edge cites the file and line it was read from. Ask it the question you actually have:

```
cargo run -p delulu-survey -- impact <id>       # everything that breaks if this changes
cargo run -p delulu-survey -- affected-by <id>  # everything this rests on
cargo run -p delulu-survey -- path <a> <b>      # how one reaches the other, hop by hop
cargo run -p delulu-survey -- rdeps <id>        # what points at it — ONE HOP
```

**Every read-only verb takes `--json`** — `query`, `rdeps`, `impact`, `affected-by`, `findings`,
`check` — and answers with **one object** carrying `tool`, `verb` and `schema`, so you can branch
before reading anything else. Three properties are worth knowing before you build on it:

- **Every edge and every hop keeps its citation**: `"via": {"kind": …, "file": …, "line": …}`. The
  provenance law — *a relation that cannot be pointed at in the text is not in the map* — holds in
  the machine channel exactly as it does in the human one. You can disagree with any single hop by
  opening the file it names.
- **`query --json` always carries `entrenched`**, set to `null` for an ordinary node rather than
  omitted. A missing field cannot be told apart from "this tool did not answer", and those must never
  look alike — see the entrenchment note below before changing anything it names.
- **The JSON walk is uncapped.** The human render truncates at 25 children per parent because a
  saturating list stops informing a reader; a caller that asked for the whole blast radius gets all
  of it and can page through it itself.

```
cargo run -p delulu-survey -- impact mod:crates/delulu-check/src/check.rs --json
  → {"tool":"delulu-survey","verb":"impact","schema":1,"node":…,"reached":135,"hops":[…]}
```

An option the tool does not know is **refused with exit 2, never ignored**. Until 2026-08-03 `--json`
was itself in that category — accepted, unimplemented, and answered with prose. If you are reading a
version that does that, you are on an older build.

**Use `impact`, not `rdeps`, when the question is blast radius.** `rdeps` is one hop and says so:
`mod:crates/delulu-check/src/check.rs` — the module that decides what type-checks — has **one
structural** edge arriving at it and reaches **134** nodes transitively. Every hop names the node it
came from and the file and line it was read from, so a chain can be walked back and disagreed with
exactly like a single edge.

**`query` answers "may I change this?" before you ask "what breaks if I do?"** A handful of paths are
**entrenched** — Constitution §10 puts them behind the project lead specifically, not any maintainer,
and invariant 44 requires an entrenchment analysis before any of them moves. `query` prints that
first, with the `.github/CODEOWNERS` line it was read from:

```
ENTRENCHED — changing this needs @PENDING-PUBLIC-project-lead specifically, not any maintainer
        matched by `/docs/design/CONSTITUTION.md` at .github/CODEOWNERS:16
```

If you see it, stop and propose the change rather than making it. Nineteen nodes carry it: the
constitution, `DELULU_CORE.md`, `STABILITY.md`, `/rfcs/`, `SECURITY.md`, `/docs/security/`, the
soundness audit and its laundering suite, and the conformance machinery.

[`docs/survey/DISCREPANCIES.md`](survey/DISCREPANCIES.md) lists where the repository currently
disagrees with itself — worth reading before you trust a number you found in prose.

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
  "delulu_version": "1.0.0",
  "diagnostics": [ ... ],
  "summary": { "errors": 1, "warnings": 0 }
}
```

- `schema` is versioned and **additive**. New fields may appear; existing ones will not change type
  or meaning. **Ignore unknown fields** — that is what makes the additive promise usable.
- `summary.errors == 0` is the machine-readable definition of "this passed".
- **Exactly one object, including on failure.** A usage or I/O error — a missing argument, an
  unreadable path, a malformed flag — still emits one envelope, with `summary.errors = 1` and an
  additive `error` object (`kind`, `exit`, `message`); the human-readable reason is on stderr. No DL
  code is invented for these, because the code registry is a stable contract and a usage error is not
  a language diagnostic, so `diagnostics` is `[]`.

  This is gated, not merely promised: `crates/delulu/tests/json_contract.rs` sweeps every subcommand
  against malformed-argument shapes and asserts stdout parses as **exactly one** JSON value. It was
  added because the promise used to be false — most commands emitted nothing at all on failure
  (campaign C2) — and because the first fix made two commands emit *two* objects, which a
  "does it look like JSON?" check would have missed.
- **Diagnostic volume is bounded on the human channel and not on yours.** `--json` reports every
  diagnostic; the human render caps at 50 with a note saying how many were withheld. If you are
  parsing, use `--json` and you lose nothing. (Campaign C32: one 10 KB file used to produce 76 MB of
  stderr, which is a context-window attack whether or not anyone meant it as one.)

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
- **A code you cannot look up still has an answer.** `delulu explain <code>` resolves codes the
  registry does not allocate — retired, never allocated, reserved, or named by a specification the
  implementation never grew — and says which, with the reason. Exit 0 means answered; exit 1 means
  genuinely unrecorded, which is the only case that should be read as a typo. The full table is
  `docs/reference/diagnostics.md`, "Codes this compiler cannot emit".
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

**Or do not implement it at all: `delulu fix <file> --json` applies them for you**, under exactly
the policy below — only `exact` repairs, never an `authority_widening` one unless you name it with
`--accept-widening <id>`, and never at all if the file is stored in a surface morph (its repairs
describe the canonical text, not the bytes on disk). Every repair comes back with a `verdict`
saying what happened to it, so a skipped one is visible rather than looking like nothing to do.

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

There are **ten** core effects: `Read`, `Write`, `Net`, `Clock`, `Rand`, `Declassify`,
`ForeignCall`, `Load` (bringing in a plugin after compile time), `Async` (`spawn` and behaviour
sends — an effect only; there is no futures runtime and no `await`), and `Actuate` (commanding a
physical device). A module may declare more with `effect Name`; those are user effects and are
reported by name.

The list is closed — `delulu_check::ty::Effect::core_from_name` accepts exactly these ten and
nothing else, and a `type` or `effect` declaration that shadows one is refused rather than silently
ignored. If you are enumerating effects in a harness, enumerate all ten: a report carrying `Load`,
`Async` or `Actuate` is not malformed.

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
| `DELULU_MORPH_PATH` | Where to find surface morphs, searched after `./morphs`. |

## [agents.morphs] Surface morphs — your surface, if you measure a win

A **morph** renames the language's keywords. `morphs/compact-ai.toml` ships as a short-alias profile
(`F` for `fn`, `L` for `let`, …). `delulu morph render <file> --to compact-ai` converts a file;
`--to-canonical` converts it back, byte-identically. A converted file carries `//! morph: <id>` on
line 1 and `check`/`run`/`authority` read it directly.

**Default to canonical, and here is the honest reason.** Canonical DeluluLang is already terse,
regular, and ASCII-stable, and the machine envelope never passes through a morph at all — codes, JSON,
spans, DIR, and every hash are computed on canonical, so a morph buys you nothing on the surfaces you
actually parse. The only thing it can buy is tokenizer cost on the *source text*, and that is
tokenizer-specific: **measure it with your own tokenizer before adopting one.** This project claims no
number.

If you author your own morph, the rules that will refuse it are worth knowing up front: an alias may
not be another keyword's canonical spelling (DL1711 — `let = "fn"` would make the word `fn` mean
`let`, which is a lie told to whoever reviews the file next, including you), two keywords may not
share an alias (DL1710), and an alias must lex as exactly one token with no bidi controls (DL1712).
Scripts and emoji are otherwise unrestricted.

## [agents.limits] What to tell your users honestly

Repeating what the rest of the project says, because a harness author is the person most likely to
overstate it downstream:

- **The Guard does not contain a program running as the same OS user, and this is the one that
  matters most to a harness author.** DeluluLang's containment is enforced by a broker process; to
  the kernel, a program running as your user is the same principal as that broker, so it can read
  the broker key, edit its policy, or kill it. If your harness runs *generated* code, run it as a
  **separate OS account** — that is the only configuration in which the boundary is enforced by
  something other than the code's good behaviour, and it is verified with a real second UID. The
  three deployment tiers, what each is worth, and how to check which one you are in are in
  [`DEPLOYMENT.md`](DEPLOYMENT.md); `delulu doctor` reports a `security posture` section, and
  **running it as the agent account is how you test the boundary** — if it can write the broker's
  state directory, the boundary is not there.
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
- **Nesting is capped at 128 levels in all four recursive-descent classes** — expressions
  (`DL0210`), types (`DL0211`), patterns (`DL0212`) and blocks (`DL0213`). These are the limits in
  this list a code *generator* is realistically able to hit. A human never writes 128-deep nesting; a
  program emitting one nested construct per element of a large structure can. If you generate
  DeluluLang, emit a `let` binding per level rather than one deep expression, and prefer a loop over
  deeply nested blocks.

  **Deep runtime *data* is a separate axis and is now safe too.** Building a recursive value
  millions deep — `type Chain = Nil | Link(Chain)` in an accumulator loop — used to abort the host
  during teardown *after* the program had finished, with no diagnostic. Value teardown is iterative
  as of 2026-08-10, so depth costs heap rather than native stack. Call depth remains bounded by
  `DL0905`.

  Stated with its history because it is recent: before 2026-08-04 there was no limit, and a valid
  module nested 100,000 deep did not produce a diagnostic — it overflowed the stack and killed the
  process (exit 127, no code, no span, nothing catchable). `delulu check` is the gate your harness
  relies on, so a crash there is worse than a rejection. It now answers. Note that this **narrowed
  the accepted language**: input that used to compile at extreme depth now returns `DL0210`.
