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
delulu toolchain --json                # this toolchain as data: commands, flags, grants, effects, codes
delulu schema <name> --json            # the closed JSON Schema of an output; `schema validate` checks one
delulu examples --json                 # the shipped programs, each with its authority and its run line
```

`toolchain --json` is read from the binary's own tables — the command list the dispatcher uses, the
usage text `--help` is cut from, the grant parser's forms (each with an example that parses), the
primitive table, the budgets, the sandbox levels, the diagnostic registry — so it cannot describe a
flag that does not exist. Ask it before guessing an option.

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
- **`run` is the exception, by design:** under `--json` its stdout is the PROGRAM's, so the run's
  own envelope — the `sandbox` object (isolation level, mode, limits, break-glass), the outcome and
  the `egress` record —
  goes to the file named by `--report-out <path>`, written by the runtime in every outcome and
  refused inside any `fs.write` grant; never trust a `sandbox` object read from a program's stdout.
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

## [agents.edit] Checked edits — when you are not the only writer

You read a file, compute an edit, and write it back. If a person or another agent changed the file in
between, your byte offsets now point somewhere else, and the edit lands in the wrong place without a
word. `delulu edit` refuses that:

```
delulu edit <file> --expect-hash <blake3> --edits '<JSON list>'  [--dry-run] [--if-checks] --json
delulu edit <file> --expect-hash <blake3> --node <Atlas id> --with '<one item>'  [--dry-run] [--if-checks] --json
```

- `--expect-hash` is the blake3 of the bytes you computed the edit against (hex; any case). If the
  file no longer has them the edit is **refused** — exit 2, nothing written — and the answer's
  `edit.hash` is the file's hash **now**, so you re-read and recompute instead of guessing. You do
  not need a hashing library: every `edit` answer, applied or refused, carries the hash.
- `--edits` takes the **repair edit shape** (`[{ "range": { "start_byte", "end_byte" }, "insert" }]`,
  `file` allowed and ignored), so a repair's `edits` pass straight through. Ranges must lie on
  character boundaries and must not overlap; they apply together or not at all. `@FILE` reads the
  list from a file.
- `--node` addresses one function, type, or actor member by the id the Atlas printed
  (`fn:app/app.main`, `type:app/app.Point`, `fn:app/app.Worker.job`) and `--with` replaces it. The
  replacement must be exactly one item of that kind; it is formatted on its own, so the rest of the
  file keeps its form. An id the file no longer has is refused as stale: ask the Atlas again.
- The result is **checked** before the answer: `diagnostics` and `summary` are the edited file's,
  exit 1 if it has errors. `--if-checks` writes only a result with no errors; `--dry-run` writes
  nothing. The write is one rename, so no reader sees half an edit.
- `edit.authority` compares what the program may do before and after (`effects`,
  `required_grants`, `foreign_calls`), and `edit.authority.widened` lists what the edit **adds**.
  A non-empty `widened` is the same signal as `authority_widening: true` on a repair: a person
  decides it, not a loop. It is `null` when either side does not check.
- A file stored in a surface morph is refused: offsets and Atlas ids refer to the canonical text.
- The answer is described by `delulu schema edit` (a closed `anyOf` of the applied and the refused
  shape).

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

### [agents.cap-paths] A path is relative to the capability, not to the working directory

**If you generate DeluluLang, this is the rule most likely to make your program wrong while every
check passes.** A `Cap[FsRead]` minted for `./config` is *rooted* there, so the path you hand it is
relative to that root:

```delulu
let reader = root.fs_read("./config")
reader.read_text("app.txt")            // reads ./config/app.txt
reader.read_text("./config/app.txt")   // looks under ./config/config — returns Err(IoErr::NotFound)
```

There is **no diagnostic** for the second line: it is a well-typed program that asks for a file that
is not there, so you get an ordinary `Err` and a plausible-looking "not found". The same holds for
`fs_write`. Two subtrees means two capabilities. `delulu explain E-DL0703` repeats the rule, and
`examples/guide/05_capabilities.delulu` had it wrong until 2026-09-18 — the CI gate now asserts that
the guide's read *succeeds*, not merely that it runs. Under `--trace-effects` a filesystem record
also carries `resolved_path` and `scope_root`, so the trace shows where the path actually pointed.

## [agents.tests] Running tests

```
delulu test --json
```

Tests hold **no ambient authority** (invariant 41): each gets exactly its declared row, bounded by
the package's `[test-authority]` ceiling. An absent ceiling means PURE. Exceeding it is `DL1703`.

**An effectful test needs a ceiling.** It comes from `delulu.toml`'s `[test-authority]`, or — for a
standalone file — from `delulu test f.delulu --test-authority 'effects = ["Write"]'` (the manifest's
syntax, repeatable per line; the file's only ceiling; inside a package it may only narrow the
package's, and a wider row is `DL1703` before any test runs). If you generate tests that perform
effects, prefer a package around them, so the ceiling is reviewable in one place:

```toml
[test-authority]
effects = ["Write"]
fs_read = ["./fixtures"]
```

Inside a package, bare `delulu test` targets the package; outside one it refuses rather than guess a
directory.

## [agents.sandbox] Running code you did not write

```
delulu sandbox policy app.delulu --json     # what confinement a run would have — nothing runs
delulu run app.delulu --sandbox --mode audit   # what the run would NEED — still nothing runs
delulu run app.delulu --sandbox --sandbox-profile hostile-agent --grant console --report-out r.json
```

`--sandbox` runs the program as a **guest that holds no authority of its own**. Its capabilities are
opaque handles; the host performs every effect, under the same checks a normal run makes, and the
operating system confines the guest as well: a Job Object on Windows, a deny-default Seatbelt profile
on macOS,
and on Linux resource limits, no-new-privs, a Landlock ruleset (nothing writable anywhere, reads only
from the system paths, no TCP) and a seccomp filter.

The guest prints what it applied to itself, and on a host that cannot apply it prints that instead —
so read those lines rather than assuming the list above. They are not in the report, deliberately:
the report says what the HOST applied, and a host cannot verify its guest's claim about itself.

Three profiles (the owner's ruling D-V2-25), differing in what a guest may consume, never in who
performs its effects: `dev`, `contained` (the default), `hostile-agent`. `--limits mem=N,cpu=S` may
**narrow** a profile and never widen it.

Read the report, not the program's output. Under `--report-out F` the runtime writes the run report
to `F`: the `sandbox` object with `requested_level` and the actual `level`, the backend, the limits,
the mode, `host_guarantees` (what was actually applied), a `policy_hash`, and three fields worth more
than the rest —

- `posture`: the questions you actually have, answered from what was applied — filesystem writes,
  filesystem reads, network, new programs, memory, processor time, privilege escalation, identity;
- `limitations`: every one of those questions that **nothing is enforcing** on this host.
  `identity_separation` is always there, because the guest runs as the same OS user (RW 4.4), and
  `fully_enforced` is true only when it is the only one;
- `denied`: what the program TRIED and was refused, each entry naming its code, with `denied_total`
  in case there were more than the report keeps. On an unfamiliar program this is the first field to
  read.

A preview (`sandbox policy`, or `--mode audit`) carries none of those three: they are measurements of
a run, and a run that did not happen has nothing to measure. The report is never written to standard
output, because the program writes there too and could forge it (D-V2-21).

**What it refuses, rather than quietly not applying:** an unknown profile, an unreadable limit, and
any program whose surface the channel cannot carry yet — today that means actors, foreign C, Python,
plugins, devices and secrets. A refusal names the surface and exits 2, having run nothing.
`sandbox policy --json` reports the same thing in advance, in `unsupported_surface`.

`--sandbox=off` is the explicit opposite. Say it deliberately: an unconfined run should be a sentence
someone wrote, not a default nobody noticed.

The sandbox is **opt-in**, and that is a ruling rather than an oversight (D-V2-26): it becomes the
default once the channel can carry the surfaces it currently refuses, in PS-B/PS-C. Until then, asking
for it is the only way to get it — so ask for it.

## [agents.mcp] The MCP server

`delulu mcp` is a Model Context Protocol server on standard input and output (newline-delimited
JSON-RPC, protocol revisions `2025-06-18`, `2025-03-26` and `2024-11-05`). Point an agent host at it
from the workspace you want answered:

```json
{ "mcpServers": { "delulu": { "command": "delulu", "args": ["mcp"] } } }
```

Its tools are `check`, `authority`, `why`, `explain`, `atlas`, `atlas_query`, `toolchain`, `schema`,
`examples`, `sandbox_policy` and `sandbox_probe`; inside the DeluluLang source tree also
`survey_query`, `survey_impact` and `doctor_check`. **Every one is read-only, by construction:** each
runs this binary's own `--json` subcommand with a fixed argument list, so its `structuredContent` is
exactly what the CLI prints, and no tool runs a program, grants authority, loads code or writes a
file — `delulu edit` is deliberately not a tool, and a test holds every command to a declared
read-only or acting class so a new one cannot become a tool unreviewed. A tool argument that begins
with `-` is refused, so nothing can be passed through as an option. A command that reports errors (a program that does not check) is a successful call — read
`summary.errors` in the answer — and a refused argument is `isError: true` with the reason. To run a
program, use the CLI: granting authority stays a person's decision.

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

### Every run has a budget

An ordinary `run` — not only a sandboxed one — is held to **1 GiB of memory and 5 minutes of processor
time** unless `--limits mem=BYTES,cpu=SECONDS,wall=SECONDS` says otherwise (the owner's ruling
D-V2-25). There is no unlimited: zero is refused before anything runs, as is a dimension nobody
enforces. A run that spends a budget is stopped, exits 1, and its report says so in
`outcome.stopped_by` — `{"dimension": "memory" | "cpu" | "wall", ...}` with the budget and the value
the watchdog measured. The report's `sandbox.limits` carries the budgets the run was held to and
`enforced_by`, which states the mechanism and its resolution. Nothing in a stop message proposes
raising the budget: a budget is the operator's decision about what a program may consume, not a fix
for the program. Under `--sandbox`, `--limits` may only narrow the profile's own limits.

A budget is also AUTHORITY (PS-B-05). Whoever delegates to you can hand one down with
`grants delegate --budget mem=BYTES,cpu=SECONDS`, and a delegation made below that one without naming
a budget inherits it. When you run under `--lease`, the node's budget is what you are held to and your
default: `--limits` may ask for less and is refused, before `main`, if it asks for more. A delegation
that asks for a larger budget than its parent holds is DL0802, and its repair is the meet. Under
`--sandbox`, a flag the sandboxed run does not apply (`--lease` and `--broker` among them) is refused
rather than ignored; a lease runs without `--sandbox`.

On Windows a `--sandbox` guest also runs as a separate identity (PS-B-03): a per-run AppContainer with
no network and none of the operator's files. On Linux, where the host allows user namespaces, it runs
as a subordinate uid (PS-B-03b): none of the files only the operator's account may read, though what
every account may read stays readable. Read it from the run report, not from this sentence:
`sandbox.posture.identity` is `a per-run AppContainer` or `a subordinate uid in its own user
namespace` only when the launch applied one, and otherwise `identity_separation` is listed under
`limitations`.

A host may REQUIRE the sandbox (`delulu sandbox status` says whether). There, `run` without
`--sandbox`, `test` and `repl` exit 2 with "this host requires the sandbox"; the fix is `--sandbox`,
not a workaround. Break-glass is the operator's alone: a ticket signed by a key that is not on the
host. An agent never mints, holds or asks for one on its own behalf.

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
- **The network client is `GET` over verified HTTPS, host-side, and nothing more.** `http.get` is
  performed by the runtime (for a `--sandbox` guest, by the HOST — the guest has no socket and no
  resolver): allowlisted host, name resolved once and pinned, special-use addresses refused unless the
  host was granted with `--grant net.special=HOST` (never plain `net=`), every redirect re-checked,
  response bounded, TLS verified against the platform store, no plain HTTP. The program sees only
  `NetErr`'s three variants; `Refused` is deliberately opaque. The reason is machine-readable in the
  run report: `egress.records[].reason` under `--report-out` (for a guest, also in `sandbox.denied`).
  `delulu sandbox probe --json` says which isolation level this host can give.
- **Foreign code is outside the proof.** `ForeignCall` is a hole, enumerated in the report.
- **The collection surface, and what it refuses.** As of 2026-09-20 (V2 phase P3) `List` has fifteen
  methods (`len`, `is_empty`, `get`, `push`, `pop`, `map`, `filter`, `find`, `fold`, `sort`,
  `reverse`, `concat`, `slice`, `contains`, `join`), `Str` has ten, and `Map[K, V]` exists
  (`Map()` constructs one; `len`, `is_empty`, `get`, `contains_key`, `insert`, `remove`, `keys`,
  `values`). There is no `Set`. Four things a generator needs to know:
  - **`fold`'s callback is argument 1** — `xs.fold(init, fn(acc, x) { … })`. Every higher-order one
    carries its callback's row into the caller (R-4), so a printing predicate makes the enclosing
    function `!{Write}`; `delulu authority` will say so.
  - **`Map` iteration is ascending by key, and `keys()`/`values()` share that order**, so they can be
    paired position by position. Map output is reproducible across runs, platforms and allocators —
    safe to compare in a test. Keys are `Str`, `Int` or `Bool`; anything else is refused.
  - **Three deliberate refusals**, each naming its reason in the diagnostic: `sort` on `List[Float]`
    (`Float` has no total order — NaN compares false against itself, so a comparison sort places it by
    accident of the algorithm); `contains` on an opaque element type (`DL0605`, the same code `==`
    gives — use `Secret.verify`, which is constant-time and carries `Declassify`); a `Map` key that is
    not one of the three scalars. Do not work around these by reaching for a flag; there is none, and
    the refusal is the answer.
  - **`to_upper`/`to_lower` are full Unicode and are not a security normalization.** They do not
    round-trip (`ß` uppercases to `SS`). **Never case-fold to compare a path, a host name, or
    anything that decides an authority question** — four defects in this project's own P22 campaign
    were security decisions taken on a differently-spelled string.
  - **None of them compile to WebAssembly.** `--target wasm` answers `DL1201` for any program using
    one; run it on the interpreter. This is not new to P3 — the backend has never had a `List` in its
    type lattice — but it is now written down.
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
