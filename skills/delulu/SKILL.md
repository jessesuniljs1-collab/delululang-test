---
name: delulu
description: Write, check and run DeluluLang (.delulu) — a language where every function carries its authority and effects in its type, so what a program can do to a machine is a compile-time fact. Use when you see .delulu files, a delulu.toml, a .dpx plugin artifact, or when asked what a program is allowed to do. Also use before running code you did not write: `delulu run --sandbox` executes it as a guest that holds no authority of its own.
---

# DeluluLang, for an agent

`--json` is the contract. The human text is not: prose may improve in any release, and the JSON
envelope's existing fields will not change meaning. `docs/for-agents.md` in the repository is the
pinned reference this file points at; read it when you need the field-by-field detail.

## The loop

```
delulu check <file>... --json           # diagnostics + typed repairs — MANY FILES, ONE PROCESS
delulu check <package-dir> --json       # or a whole package, resolved together
delulu authority <file|dir> --json      # what this program can do to the machine
delulu run <file> --json --no-prompt    # run; never block waiting for a human
delulu test --json                      # run tests under an authority ceiling
delulu fmt <file>                       # canonical form; the only correct spelling
delulu explain DL0501                   # long-form docs for any diagnostic code
```

Five rules, each of which exists because getting it wrong cost someone real time.

1. **`--no-prompt` everywhere.** Without it a command needing a grant may wait for a human. With it,
   the command refuses instead of hanging.
2. **Read the exit code, not only the output.** A verdict string and an exit code that disagree is a
   bug; one such bug is on record. Exit codes are below.
3. **Batch `check`, and batch nothing else.** Starting the process is the cost, not compiling: twenty
   files in twenty invocations cost 711 ms, the same twenty in one cost 48. Every *other* command
   takes exactly one path and refuses a second rather than silently using the first.
4. **In a long edit→check loop, use `delulu lsp`.** It pays the process cost once, it is the same
   compiler so its answers cannot drift, and `textDocument/codeAction` hands you the checker's own
   typed repairs with `data.authority_widening` and `data.requires_human` set — so you can apply the
   safe ones by policy rather than by parsing prose.
5. **Never apply a repair whose `authority_widening` is true** without a human saying so. A repair
   that widens what a program may do is never marked preferred, and that flag is why.

## What trips agents, specifically

**Effect rows are part of the signature.** `fn f(...) -> T ! {Read, Write}` declares what `f` may do.
A function that performs an effect not in its row does not compile. The row of a caller must contain
the rows of everything it calls, so authority composes upward to `main` — that is the whole design,
and `delulu why <Effect> <file>` prints the chain that puts an effect there.

**Capability-relative paths.** `root.fs_read("./data")` mints a capability rooted at `./data`; every
path used through it is relative to THAT root, not to the working directory. `w.write_text("out.txt", …)`
writes `<root>/out.txt`. A path escaping its root is refused, and so is one that only looks like it
does not escape — `..`, a symlink, a case difference on a case-insensitive filesystem.

**`val` and `ref`.** Reference capabilities are a second axis beside effects. Anything crossing an
actor boundary must be sendable: `iso` (consumed), `val` (deeply immutable) or `tag` (opaque identity).
A `val` closure can still be `!{Write}` — the two axes are independent and both are checked.

**Grants are the operator's, not the program's.** A program declares what it needs; a human grants it:

```
--grant console                      --grant clock          --grant rand
--grant fs.read=PATH                 --grant fs.write=PATH
--grant net=HOST                     --grant net.special=169.254.169.254   # metadata/loopback: explicit only
--grant "secret:NAME=env:VAR"        --grant declassify
--grant foreign.c=LOGICAL:PATH       --grant foreign.python=numpy
--grant plugin=PATH                  # where a program may LOAD a plugin artifact from
--grant-manifest                     # accept what the package's delulu.toml declares
```

`delulu authority <file> --json` lists `required_grants` — the flags to pass, derived from the code.
Do not guess them; that list is the answer.

**A manifest declares, a grant confers.** `[authority]` in `delulu.toml` is the package's ceiling, and
`--grant-manifest` accepts it. Two things it deliberately cannot confer: `exec.native`, and plugin
loading. A package cannot grant itself the right to emit native code or to load code.

## Running code you did not write

```
delulu sandbox policy app.delulu --json          # what a run WOULD be confined by; nothing runs
delulu run app.delulu --sandbox --mode audit     # what the run would NEED; still nothing runs
delulu run app.delulu --sandbox --sandbox-profile hostile-agent --grant console --report-out r.json
delulu sandbox status --json                     # what THIS host can confine, measured by launching
```

`--sandbox` runs the program as a guest process holding **no authority of its own**: its capabilities
are opaque handles, and the host performs every effect under the same checks an ordinary run makes.
Profiles are `dev`, `contained` (the default) and `hostile-agent`; they differ in what a guest may
CONSUME, never in who performs its effects. `--limits mem=N,cpu=S` may narrow a profile and never
widen it.

**Read the report, not the program's output** — the program writes to stdout and could forge anything
there. Under `--report-out F` the runtime writes `F`, and three fields in its `sandbox` object matter
more than the rest:

- `posture` — filesystem writes, filesystem reads, network, new programs, memory, processor time,
  privilege escalation, identity — each answered from what was **actually applied**.
- `limitations` — every one of those questions that **nothing is enforcing on this host**.
  `identity_separation` is always there, because the guest runs as the same OS user, and
  `fully_enforced` is true only when it is the only entry. **This is the field to read** before you
  decide to run something.
- `denied` — what the program TRIED and was refused, each entry naming its code.

The sandbox is **opt-in**: ask for it. It refuses rather than downgrading — a program using actors,
foreign C, Python, plugins, devices or secrets exits 2 with a named refusal, because a sandbox that
quietly did not apply is the failure the design exists to prevent. `--sandbox=off` is the explicit
opposite; say it deliberately.

`delulu explain E-SANDBOX` is the full model with every caveat.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | success |
| 1 | the program or its authority was rejected — diagnostics explain it |
| 2 | the invocation was wrong, or a requirement was refused (bad flag, missing grant, unsupported surface) |
| 3 | a trace escaped its row under `--assert-trace` (DL1101) — a soundness failure, not a usage error |

A command that refuses tells you what to add. A command that cannot is a bug worth reporting.

## The JSON envelope

Every `--json` command emits exactly one object: `command`, `schema`, `delulu_version`,
`diagnostics[]`, `summary{errors,warnings}`, plus the command's own payload. Diagnostics carry `code`,
`message`, `severity`, `spans[]` (each with its own `file`), and `repairs[]` with `confidence`,
`authority_widening` and `requires_human`. Read `summary.errors`, not the prose.

## What to tell your users honestly

- Authority and effects are compile-time facts about DeluluLang code. **Foreign C and Python are holes
  in that guarantee** — enumerated, not eliminated. A `ForeignCall` in a row is that hole, named.
- The sandbox is a second wall **under** the OS account boundary, not instead of it. A guest runs as
  the same user.
- Certification is NONE: no safety standard, no external audit, in any regime.
- The standard library is small. `List` has four methods; there is no `Map` type yet.

## References

- `docs/for-agents.md` — the pinned, field-by-field reference. Stable anchors: `[agents.start]`,
  `[agents.exit-codes]`, `[agents.json-envelope]`, `[agents.diagnostics]`, `[agents.repairs]`,
  `[agents.authority]`, `[agents.effects]`, `[agents.tests]`, `[agents.sandbox]`, `[agents.registry]`,
  `[agents.determinism]`, `[agents.env]`, `[agents.limits]`.
- `docs/reference/primitives.md` — every primitive with its row, generated from the compiler's table.
- `delulu explain <CODE>` — any diagnostic, and the named topics `E-SANDBOX`, `E-PLUGIN`, `E-GUARD`,
  `E-ACTOR`, `E-REVOKE`, `E-ATLAS`, `E-PALETTE`.
- `delulu doctor` — is this checkout healthy, and what does this host actually enforce.
