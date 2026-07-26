# The DeluluLang Book

*The complete guide to the language where every unit of code carries its authority in its type.*

**Status:** First complete edition (planning-pass draft by Claude Fable 5, Mythos-class). Living
document — it will grow, but it is complete now. Every code sample is written to become a conformance
test in Stage 9 (`docs/book/` is its CI-verified home); where a sample uses a feature from a
not-yet-built stage, it is marked with the stage.

**How to read this book.** There are two human audiences, and the book serves both without
compromise (Constitution §8.4): people **learning to build** with DeluluLang, and people **reviewing
code an AI wrote**. If you are the first, read front to back. If you are the second, read Chapter 1,
then jump to Chapter 6 (Reading Authority) and Chapter 11 (Reviewing AI-Written Code). If you are an
AI reading this to program in DeluluLang, read Chapter 12 (For Machines) first, then use the rest as
reference — but know that the *why* in Chapters 2–5 is what lets you write code humans will trust.

---

## Table of Contents

1. Hello, Delulu — the identity in one command
2. Why DeluluLang Exists
3. Why Authority Matters
4. Why Effects Matter
5. Capabilities, Secrets, and the Shape of Trust
6. Reading Authority — the flagship skill
7. Errors as a Conversation — diagnostics and repairs
8. Packages and Provenance — the supply chain that can't lie
9. Containment — WASM, artifacts, and the sandbox floor
10. Plugins — code that arrives at runtime and still can't overreach
11. Concurrency without Fear — actors and reference capabilities
12. Humans and AI, One Law — the no-discrimination design
13. For Machines — the agent-native surface
14. The Foreign World — C, Python, and the honest boundary
15. Custody — where the keys actually live
16. Commanding Machines — the physical boundary
17. Migrating from Python, Rust, JavaScript, Go
18. Design Philosophy — the principles beneath the mechanisms
19. Honesty — what DeluluLang refuses to claim
20. The Road Ahead — stages, RFCs, and the shape of v1.0+
Appendix A. The Welcome Note
Appendix B. Glossary
Appendix C. Where Everything Lives (spec map)

---

## Chapter 1 — Hello, Delulu

Every language starts with hello-world. DeluluLang starts with hello-world **and one more command**,
because that command *is* the language.

```delulu
module hello

fn main(root: Root) ! {Write} {
  let out = root.console()
  out.println("hello from the delulu gang")
}
```

Run it:

```
$ delulu run hello.delulu --grant console
hello from the delulu gang
```

Now the command that no other mainstream language can answer honestly:

```
$ delulu authority hello.delulu
Authority of `hello` — what this program can do to your system:
  effects:      Write
  capabilities: Console (stdio)
  secrets:      (none)
  pure fns:     (none)
  foreign:      (none — no code outside the guarantee)
```

Read that carefully. The compiler did not *run* the program to learn this. It **proved**, from the
types alone, that `hello` can do exactly one thing to your system — write to the console — and
**nothing else**. It cannot read a file. It cannot reach the network. It cannot read the clock. Not
because it chose not to, but because **it has no way to**: the only capability it holds is the one you
see.

Notice three things in the source, because they are the whole language in miniature:

1. **`main` takes `root: Root`.** There is no ambient authority in DeluluLang — no global `open()`, no
   `import os`. The *only* way to affect the world is through a capability, and every capability
   descends from the single `Root` that `main` receives. If a function doesn't get a capability, it
   physically cannot use one.
2. **`! {Write}` is the effect row.** It is part of `main`'s *type*. It says: this function performs
   the `Write` effect and no other. Leave it off and the function is `!{}` — provably pure. The
   compiler checks this the way it checks that you didn't return a `Str` where an `Int` was expected.
3. **`--grant console`.** You, the human running it, decided to grant the console. Without that
   grant, the program refuses to run its console operation — the authority isn't the program's to
   assume; it's yours to give.

That is DeluluLang. Everything else in this book is consequence and detail. If Chapter 1 lands, you
already understand the language; the rest is *why it is built this way* and *how to use it well*.

---

## Chapter 2 — Why DeluluLang Exists

Software has a trust problem, and it is getting worse fast.

For decades we shipped code and *hoped*. A dependency you pulled in could read your SSH keys, phone
home, or wait three years and then — in one innocuous-looking patch release — start exfiltrating
data. This is not hypothetical: it is the `xz` backdoor, the `event-stream` incident, dozens of
typosquatting campaigns, and the everyday reality that `npm install` runs arbitrary code with your
full user authority. The tools we use to *find* these problems are all **after the fact**: scanners,
audits, runtime monitors, forensics. We detect betrayal; we do not prevent it.

Then AI changed the scale. When a human wrote every line, the blast radius of "this code does more
than it says" was bounded by how much a person could write. When an AI writes ten thousand lines an
hour, and *other* AIs review it, and *those* get orchestrated by yet another AI — the old model
("trust the author, audit later") breaks completely. You cannot audit what you cannot read fast
enough, and soon almost nobody will be reading most code at all.

DeluluLang starts from a different question. Not *"how do we detect bad code?"* but *"what if code
physically could not exceed what it was explicitly permitted to do — and you could see that
permission in one command, before running anything?"*

That is the entire thesis:

> **Every unit of code — every function, module, and dynamically loaded plugin — carries its
> authority and effects in its type, so the compiler can verify that no code can exceed the
> authority it was explicitly granted.**

Not a linter. Not a sandbox you opt into. Not a policy file that drifts from reality. A **type
system** — the same machinery that catches `1 + "two"` — extended to catch *"this function touches
the network but its type says it shouldn't."* If it type-checks, the authority claim is true. Whole
program. Including the dependencies. Including the plugin loaded at 3am by an autonomous agent.

DeluluLang exists because the future is one where most code is written by machines and used by
machines, and the only trust model that survives that future is one where **trust is mechanical, not
social** — proven by construction, readable by anyone (or anything) in a single command, and honest
about exactly where the proof has holes.

The name is a joke that means it. "Delulu" — being delusional enough to think you can build the
thing everyone says is impossible. A capability-secure, effect-typed, whole-program-verified language
that humans *and* AIs actually want to use is supposedly too academic, too restrictive, too slow to
catch on. Maybe. But you don't get the impossible thing by being realistic about it.

---

## Chapter 3 — Why Authority Matters

**Authority** is DeluluLang's word for *everything a piece of code is permitted to do to the world*.
Not what it *does* — what it *can* do. The distinction is the whole game.

In every mainstream language, authority is **ambient**: it's in the air, available to any line of
code that wants it. `open("/etc/passwd")` works from anywhere. `fetch(url)` works from anywhere.
There is no wall between the part of your program that parses a date and the part that could email
your database to a stranger — they run with the same ambient permissions, which are *your*
permissions, which are enormous.

DeluluLang has **no ambient authority**. This is the single design decision everything else hangs
from. There is no global function that touches the world. The only way to do anything observable —
read, write, network, clock, randomness — is to *hold a capability* for it, and capabilities are
**unforgeable values** that can only be obtained by being *given* one, tracing all the way back to
the `Root` that `main` receives from the runtime.

This flips the default. In Python, a function can do anything unless you go to heroic lengths to stop
it. In DeluluLang, a function can do **nothing** unless you hand it the means:

```delulu
// This function CANNOT read a file, reach the network, or tell time.
// Not "shouldn't" — cannot. It holds no capability, so there is nothing to call.
fn add(a: Int, b: Int) -> Int { a + b }
```

`add` is provably pure. You do not have to trust the author of `add`, audit `add`, or sandbox `add`.
Its *type* — `fn(Int, Int) -> Int` with an implicit empty effect row `!{}` — is a **proof** that
calling it cannot touch your system. And that proof composes: if `add` is pure and `main` calls only
`add` and `console.println`, then `main`'s authority is exactly `{Write}`, computed mechanically,
guaranteed complete.

Why does this matter more than "just be careful"?

- **Least privilege becomes the path of least resistance.** In other languages, giving a component
  exactly the permissions it needs is extra work nobody does. In DeluluLang, a component *starts* with
  nothing and you add capabilities deliberately — least privilege is the default, not the discipline.
- **The reviewer's job becomes possible.** You do not read all the code to know what it can do. You
  read its authority. A 100,000-line program with `effects: [Read]` and `fs.read: ["./config"]`
  cannot delete your files or call home, and you learned that in one command.
- **Delegation is safe.** When you pass a capability to another function — or another agent — you can
  only pass an **attenuation**: the same authority or *less*, never more (this is the `⊑`
  relationship, "flows-into"). A function you call cannot escalate. An agent you spawn cannot escalate.
  Authority only ever narrows as it flows outward.

Authority matters because it is the thing everyone actually cares about — *what can this code do to
me?* — promoted from a vague worry to a typed, checked, queryable fact.

---

## Chapter 4 — Why Effects Matter

If authority is *what code can do*, **effects** are *how the type system tracks it*.

An **effect** is an observable interaction with the world: `Read`, `Write`, `Net`, `Clock`, `Rand`,
`Declassify`, `ForeignCall`, `Load`, `Async`, `Actuate`. Every function's type carries an **effect
row** — the set of effects it may perform:

```delulu
fn greet(out: Cap[Console], name: Str) ! {Write} {
  out.println("hello, " + name)
}
```

The `! {Write}` is not documentation. It is a **checked claim**. If `greet`'s body performed a
network call, the compiler would reject the program: *"function `greet` performs `Net` but its row
does not permit it"* (DL0501). If `greet` declared `! {Write, Net}` but never actually did anything
networked, the compiler would *also* complain (DL0502) — the row must be exactly right, neither
overclaiming nor underclaiming.

Three properties make effects powerful rather than annoying:

**1. Effects arise only from capability operations.** You never write "this line has an effect." An
effect appears in a function's row *because* it called a capability method that has that effect —
`out.println(...)` has `Write` because `Console.println` is defined (in the primitive table) to have
`Write`. Effects are a *consequence* of using authority, tracked automatically. Pure computation —
arithmetic, string manipulation, data structures — has no effects, ever, with no annotation.

**2. Effects compose upward, exactly.** A function's row is the union of its own capability operations
and the rows of everything it calls. This is not inference-magic; it is arithmetic. And it means the
row of `main` is the authority of the *whole program* — because everything runs, transitively, from
`main`. That is why `delulu authority` can be complete: it is just `row(main)`, unfolded and
explained.

**3. Effects are polymorphic where they need to be.** A higher-order function shouldn't have to know
its callback's effects in advance:

```delulu
fn apply[T, U, e](f: fn(T) -> U ! e, x: T) -> U ! e {
  f(x)
}
```

The `e` is an **effect-row variable**. `apply` is honest: *"my effects are exactly my argument's
effects."* Call it with a pure function and `apply` is pure; call it with a networking function and
`apply` is `!{Net}` at that site. The row system carries this precisely, so abstraction doesn't cost
you the guarantee.

Why do effects matter? Because they turn "what can this code do?" from a whole-program *reading*
problem into a *type-checking* problem. Types compose, types are local, types are checked in
milliseconds. By putting authority *in the types*, DeluluLang makes the security question answerable
by the same fast, compositional, mechanical process that already answers the correctness question.
Effects are the bridge between "authority" (the human concept) and "the compiler can prove it" (the
mechanism).

---

## Chapter 5 — Capabilities, Secrets, and the Shape of Trust

### Capabilities

A **capability** is an unforgeable value that confers exactly one kind of authority over exactly one
scope. `Cap[Console]` lets you print. `Cap[FsRead]` scoped to `./config` lets you read files under
`./config` — and nowhere else. You get one only by being handed it; you cannot construct one, cast an
integer to one, or forge one through reflection (there is no reflection). This is the
**object-capability model** (ocap), and DeluluLang takes it seriously all the way down.

Capabilities are **attenuable**. Given a `Cap[FsRead]` over `./data`, you can derive one over
`./data/public` to pass to a less-trusted helper — narrower, never wider. The helper cannot climb
back up. This is how a program hands out authority internally the same way you hand it out at the
command line: always `⊑`, always shrinking.

`Root` is the origin: the single capability `main` receives, from which `console()`, `fs_read(path)`,
`clock()`, `net(host)`, and the rest are derived — each derivation checked against what you granted.

### Secrets

Some values must *flow through* your program without being *readable* by it — an API key you pass to
an HTTP client, a password you compare but never log. DeluluLang gives these a type:

```delulu
let key: Secret[Str] = root.secret("API_KEY")
```

`Secret[T]` is **opaque**. You cannot print it, compare it with `==`, serialize it, or pass it to
foreign code. The compiler refuses all of these (DL0602, DL0604, DL0605) — *"a `Secret` value cannot
flow here; `Secret[Str]` is not `Str`."* The only way to get the bytes out is `expose`, which
**carries the `Declassify` effect** — so the moment your program reads a secret's contents, it shows
up in the effect row, in `delulu authority`, in the audit log. Declassification is not forbidden; it
is *visible*. You can always see, mechanically, every place a program looks inside a secret.

### The shape of trust

Put these together and you have DeluluLang's model of trust as a *shape* rather than a *feeling*:

- **Capabilities** decide what regions of the world code can touch.
- **Effects** track, in the types, which capabilities are actually used, transitively.
- **Secrets** mark data that flows but must not leak, making every peek an auditable event.
- **Attenuation** ensures authority only ever narrows as it moves outward — between functions,
  between packages, between agents.

None of this is a runtime tax on the hot path (Chapter 9 explains why the checks live host-side, not
in your compute loop). It is a *type* discipline: pay it once at compile time, carry the proof
forever.

---

## Chapter 6 — Reading Authority (the flagship skill)

The most important thing you will learn in this book is not how to *write* DeluluLang. It is how to
*read authority* — because that is the skill that makes the language worth having, for humans and AIs
alike.

`delulu authority` answers "what can this code do to my system?" Its output is designed to be read
top-to-bottom as a risk assessment:

```
$ delulu authority ./my-agent-tool
Authority of `my-agent-tool` — what this program can do to your system:
  modules:      main, http, parse
  effects:      Net, Read
  capabilities:
    - FsRead    ./config
    - Http      api.weather.example.com
  secrets:      WEATHER_API_KEY
  pure fns:     parse_response, format_report, celsius_to_f
  foreign:      (none — no code outside the guarantee)
```

Read it like this:
- **effects** — the top-level verbs. `Net, Read` means: talks to the network, reads files. *Not*
  Write, *not* Clock, *not* Rand, *not* Declassify beyond what's shown. Anything absent is *proven
  absent*.
- **capabilities** — the nouns, with scope. `FsRead ./config` — reads only under `./config`. `Http
  api.weather.example.com` — talks to only that host. A supply-chain attacker who wanted this tool to
  exfiltrate data to `evil.com` would have to make that host *appear here* — and it would, because the
  authority is computed, not declared by hand.
- **secrets** — `WEATHER_API_KEY` is handled but (unless you also see `Declassify` in effects) never
  read by the program itself; it flows to the HTTP client opaquely.
- **pure fns** — the parts you never have to think about; they cannot do anything.
- **foreign** — code outside the guarantee. Here, none. When there is some (Chapter 14), it appears
  under an explicit "outside the proof" line.

When you need the *why*, ask:

```
$ delulu why Net ./my-agent-tool
Net is reachable because:
  main → fetch_weather → http.get(client, url)   [http.get performs Net]
```

That is the shortest path from `main` to the primitive that performs the effect — the proof, at
function granularity. When an AI writes a tool and you're deciding whether to run it, this is the
review: not reading the code, reading its authority and, if something surprises you, its `why`.

This is why Chapter 1 of *every* DeluluLang tutorial ends with `delulu authority` on hello-world
(Stage 9 makes it a rule): the identity of the language is not a feature you use occasionally. It is
the lens you look through every time you consider running something.

---

## Chapter 7 — Errors as a Conversation

DeluluLang treats a compiler error not as a complaint but as a **structured, machine-actionable
conversation** — because half its users are machines, and the other half are increasingly *reviewing*
machines.

Every diagnostic has:
- A **stable code** (`DL0501`) — permanent, add-only across versions. Machines key on the code, never
  the prose. Your agent harness written today keeps working at v2.
- **Spans** — exact byte ranges, so an editor or an agent knows precisely where.
- **Typed repairs** — not "did you mean...?" text, but a *structured edit* the tooling can apply,
  each flagged two ways:
  - `authority_widening: true|false` — does applying this repair *increase* what the program can do?
    Adding `Net` to a row to fix DL0501 is a widening repair — it's flagged, and CI (or a human, or a
    supervising agent) may veto it. Narrowing repairs apply freely.
  - `requires_human: true|false` — some situations (a revoked lease, a tamper detection) are not for a
    machine to auto-fix; the flag says so.

A DL0501 in JSON, the way an agent sees it:

```json
{ "code": "DL0501", "message": "function `sync` performs `Net` but its row does not permit it",
  "spans": [{ "file": "src/main.delulu", "line": 14, "label": "this call performs Net" }],
  "repairs": [{ "kind": "add_effect_to_row", "effect": "Net", "authority_widening": true }] }
```

The agent reads: *there's a DL0501; the repair widens authority; I should not silently apply it — I
should surface it.* That is the loop DeluluLang is built for: the language tells the machine not just
what's wrong, but whether the fix is a safe mechanical narrowing or a real authority decision that
needs a human's — or a more privileged holder's — sign-off.

For humans, the same diagnostic renders as readable prose in your chosen locale (Chapter 12), with
`delulu explain DL0501` giving the long-form reasoning. Same underlying fact, two surfaces, one
source of truth.

---

## Chapter 8 — Packages and Provenance

A language that verifies your code but trusts your dependencies blindly has verified nothing. Most of
the code you ship, you didn't write. DeluluLang extends the authority guarantee across the **whole
dependency graph**.

Every package declares its authority ceiling in its manifest:

```toml
[package]
name = "markdown"
version = "1.2.0"

[authority]
effects = ["Read"]          # this package promises to do at most this
fs.read = ["./templates"]
```

And three mechanisms keep that promise honest:

**1. Self-check (DL1009).** The compiler computes what the package *actually* does and refuses to
build if it exceeds the manifest. A package that declares `[Read]` but performs `Net` doesn't ship.
The manifest is a *checked ceiling*, not a hopeful comment.

**2. Authority pins (DL1001).** When you depend on `markdown = "1.2"`, your lockfile records the
authority you accepted. If a later resolution would let `markdown` do *more* than you pinned, the
build fails — before running anything. This is the anti-`xz` mechanism: a dependency cannot quietly
grow new powers.

**3. The semver-authority law (DL1003).** Widening authority is a **major-version** change, always. A
package cannot add `Net` in a patch or minor release; doing so is a breaking change by definition,
caught mechanically. The Study-A measurement (Stage 9) requires this to catch an injected effect
**100% of the time** — because it is a mechanism, not a heuristic.

The payoff is a supply chain that **cannot lie about authority**. When you run `delulu add
some-package`, the registry's index line carries the authority summary, so you see the authority diff
*before you download a single byte* — a supply-chain UX no other package manager offers. And the
lockfile carries content hashes (blake3) binding the exact source you verified. Trust is
*trust-on-first-verify*: you re-verify locally, you don't trust-on-read.

`tests/corpus/tier4-multimodule/` is a worked example of all of this: four packages, seven modules,
dependency depth three, three of them declaring `effects = []`, and one authority answer computed
across the four manifests. `delulu authority` on it names every module and every effect; `delulu why
Write` traces `main` into the one dependency allowed to write.

**And it runs.** `delulu run <package-dir>` executes a multi-package program: the graph is resolved
and checked first — per-module visibility, authority ceilings, dependency pins — and only a program
that passes all of that is executed.

> **The one limit, stated plainly.** Two modules may legally declare the same top-level name, because
> visibility is per module. Running the program brings the modules into one scope, where two private
> `helper`s are not distinguishable, so that case is **refused by the runner** rather than resolved by
> whichever module happened to be merged last — and the refusal says the program is correct, because
> it is: `check`, `build` and `authority` all handle it. Lifting the restriction needs per-module
> resolution inside the interpreter. This was campaign finding **C59**, closed by ruling **D61**; the
> remaining edge is the fail-closed part of that fix, not a leftover.

---

## Chapter 9 — Containment: WASM, Artifacts, and the Sandbox Floor

The type system proves what your code *can* do. But two things need a runtime floor: code you compile
to ship as a sealed artifact, and code that isn't fully trusted to have been type-checked honestly.
DeluluLang's answer is a WebAssembly backend with a deny-by-default host — the "sandbox floor" beneath
the type-level guarantee.

The interpreter is the **reference engine** — it defines the semantics, and it runs the whole
language. The WASM backend is **a fragment of it**, and this chapter is worth nothing if you read it
any other way.

Inside that fragment the contract is **byte-identical** observable behaviour, and it is checked two
ways: a curated parity set, and a generative fuzzer that runs **2,000** random console programs on
both engines and compares their output exactly. Outside the fragment, a program does not miscompile —
it is refused as **DL1201** and falls back to the interpreter, and that boundary is itself tested.
Fail-closed at the edge is the property that makes a partial backend safe to have.

**What "fragment" means concretely.** Measured, not estimated. What compiles today is roughly:
`Int`/`Bool`/`Str` arithmetic and comparison, `if`/`else`, `let`, function calls, recursion, `match`
on a sum type, string concatenation, `str(Int)`, and console output. That is enough to compile `fib`
and `gcd` and print the answer.

What is **DL1201** today includes `while` loops, `Float` arithmetic, record field access (outside
`self.field`), `List` and `.len()`, the clock, every string method (`.trim`, `.split`), `.narrow`,
`.fs_write`, embedded Python, and actor state outside the `Int`/actor-reference subset. Of the
entry-point programs in this repository's corpus and examples, **6 of 19** compile to WASM, and
**none of this book's own guide chapters do**.

So: if you are writing ordinary DeluluLang — anything with a loop, a float, a record, or a list —
**you are running on the interpreter.** The WASM path is for the sealed-artifact and untrusted-code
cases described below, and it is grown deliberately, one construct at a time, with the boundary
refusing rather than guessing.

> **This paragraph used to say something false**, and the correction is left visible rather than
> quietly swapped. It claimed parity was "enforced by a differential fuzzer running tens of thousands
> of programs on both engines" and that "two independent implementations agree on 50,000 random
> programs". The real number is 2,000, inside the fragment; the crate actually named the differential
> fuzz harness (`delulu-fuzz`) depends on the checker and the interpreter and **cannot run the WASM
> backend at all**. What it proves is something else, and something better — see the note below.
> Campaign finding **C63**, ruling **D58**.

**What `delulu-fuzz` actually proves, which is the more important claim.** For every program it
generates and accepts, it runs it under a trace sink and asserts the observed runtime effects are a
**subset of the effect row the checker computed for `main`** — the executable form of the
Effect-Soundness theorem (`DELULU_CORE.md` Theorem 3, spec invariant 12). A single violation would be
an effect escaping the type, which is the one bug this language exists to prevent. That evidence was
real all along; the old paragraph credited it to the wrong property.

The key architectural choice: **capability checks live host-side, not in guest code.** A compiled
DeluluLang program running under WASM doesn't carry authority checks inside its compute loop — the
*host* (the runtime embedding the WASM) performs each effect and validates each capability as the
guest reaches for it. Consequences:
- **Pure compute is free.** Arithmetic compiles to ordinary WASM with zero authority overhead. There
  is nothing for an optimizer or a JIT to "optimize away," because the checks were never in the guest.
- **A guest that lies gets caught at the boundary.** Hand-write malicious WASM that forges a
  capability handle or reads out of bounds, and the host refuses it (DL0904/DL0903) — the guarantee
  doesn't depend on the guest being well-behaved.

### The `.dwx` artifact — authority that travels with the code

`delulu build --target wasm -o app.dwx` produces a single file that **carries its own authority
manifest**, hash-bound to the exact code:

```
$ delulu build app.delulu --target wasm -o app.dwx
ok: wrote `app.dwx` (1403 bytes) with authority embedded as `delulu:authority`

$ delulu run app.dwx --grant console
running `app.dwx` — authority verified; declared effects: Write
...
```

The artifact contains a `delulu:authority` section (the same JSON `delulu authority` reports) and a
blake3 hash of the code. At run time the hash is re-verified: alter one byte of the code and the
artifact refuses to run (DL1202). The authority claim and the code it describes cannot drift apart.
(Honesty: the hash is an *integrity* binding — it detects corruption and naive swaps — not a
cryptographic signature proving *who* built it; signing is a separate, later mechanism.)

Secrets get a special guarantee here: secret-handling code **does not compile to WASM at all**
(DL1205), so secret bytes can never enter a guest's linear memory. The strongest hygiene is the byte
that never arrives.

---

## Chapter 10 — Plugins: Code That Arrives at Runtime and Still Can't Overreach

Here is the demo that sells the language. A running program loads a plugin it has never seen —
downloaded moments ago, written by a stranger or an AI — and the plugin **still cannot exceed the
authority the host granted it.** Not by policy. By construction.

```delulu
let summarize = host.load[Verified](plugin_host, "summarize.dpx",
  Grant { effects: [], fs_read: [], net: [], ... })?   // grant it NOTHING
let f = summarize.get[fn(Str) -> Str ! {}]("summarize")?
let result = f(document)                                 // pure transform, provably
```

The plugin was granted an empty authority. `f`'s type is `fn(Str) -> Str ! {}` — pure. If the plugin
tried to read a file, tell time, or reach the network, it would be **refused at load** — and the
host's own authority is unchanged by loading it. You extended a running system with untrusted code
and lost nothing.

DeluluLang ships two plugin classes, and the difference is a *trust statement, not a quality ranking*:

- **`Plugin[Verified]`** ships its typed IR (DIR — the post-check typed AST). The loader **re-checks
  it in full** at load time — types, rows, all the soundness rules — so a Verified plugin is
  proven-per-function, compile-time-grade, at the moment it loads. Tamper with it and re-verification
  fails (DL1504); it never silently "falls back" to a weaker class.
- **`Plugin[Contained]`** is opaque WASM. The loader confines it at the module boundary (its imports
  must be within its grant) and — crucially — **types every export at the full grant row** (audit
  rule R-1). A Contained plugin's "read-only-looking" function *types as everything its module was
  granted*. The type system keeps it honest: you cannot accidentally believe a Contained plugin is
  more limited than its grant.

And a plugin's grant is a **child node in the authority tree** (Chapter 15): unloading it revokes its
authority; revoking the host cascades to every plugin it loaded. Resource limits (fuel, memory,
wall-clock) are grant data — a runaway Contained plugin is *terminated and gone*, not wounded, without
harming the host.

This is what "authority-bounded" means when the code arrives after compile time: the same guarantee,
the same `⊑`, the same tree — extended to the runtime frontier.

---

## Chapter 11 — Concurrency without Fear

Concurrency is where most languages surrender their guarantees. Shared mutable state plus threads
equals data races, and data races are undefined behavior, heisenbugs, and security holes.
DeluluLang's answer is the **actor model with reference capabilities**, giving **compile-time
data-race freedom** — proven, zero runtime cost.

An **actor** is an isolated unit of state reached *only* by messages:

```delulu
actor Counter {
  var n: Int
  new() { self.n = 0 }
  be increment() { self.n = self.n + 1 }      // `be` = behavior: async message handler
  be report(out: tag Console) { out.println("count: " + str(self.n)) }
}
```

No other actor can touch `Counter`'s `n`. Not through a pointer, not through a cast — there is no
path. This is enforced by **reference capabilities** (adopted from Pony's proven system): every value
carries a second annotation beyond its type, saying who may alias, mutate, or send it — `iso` (unique,
sendable), `val` (deeply immutable, freely shareable), `ref` (local mutable, not sendable), `tag`
(opaque identity, sendable), and more. Anything crossing an actor boundary must be **sendable**, and
the compiler proves it. Send a mutable `ref` to another actor and you get DL1601 with an explanation —
before your program ever runs.

The beautiful part: **effects and concurrency are one system.** Asynchrony is just the `Async` effect
in the same rows you already know. A function that sends a message is `!{Async, ...}`; sending to a
behavior that writes to the console requires `{Async, Write}` at the send site. `delulu why Write`
traces the chain *across the actor boundary*. There is no second coloring mechanism, no `await`, no
futures runtime — one authority system, extended to concurrency without adding a parallel universe of
rules.

The honesty (Chapter 19 makes this a habit): DeluluLang guarantees **race freedom**, not liveness.
Deadlock, livelock, and starvation are still possible — the language prevents the corruption class of
concurrency bug, not the "it's stuck" class. It says so plainly, everywhere.

---

## Chapter 12 — Humans and AI, One Law

DeluluLang is built on a principle it refuses to compromise:

> **No discrimination between humans and AI** — agents, LLMs, physical AI, robots, and whatever comes
> after. *If you are made of atoms or of electrons, DeluluLang treats you the same.* The only currency
> is authority; **what you are is never inspected on any authority-decision path.**

This is not sentiment; it is architecture. In the custody system (Chapter 15), the code that issues,
attenuates, delegates, and revokes authority contains **no branch on holder kind** — a grep-checkable,
test-enforced fact (Stage-5 acceptance criterion 9). The same delegation that lets a human hand a
scoped capability to a subprocess lets an orchestrating LLM hand a scoped lease to each of five
sub-agents. Identical mechanics. Nobody is privileged; nobody is second-class.

But "same law" does not mean "same surface." Humans and machines want different *interfaces* over the
one truth:

- **Humans** get localized prose (Chapter 12 continues below), readable authority reports, a friendly
  first-run experience, editor integration, one canonical formatter, and error messages that teach.
- **Machines** get stable diagnostic codes, JSON everything, typed repairs, a locale-invariant machine
  envelope, deterministic replay, and cold-start silence (an agent runs `delulu` with `--json` and
  never hits an interactive prompt).

And here is the elegance: **because localization touches only human prose, adding fifty human
languages costs the machine side exactly zero.** No new codes, no changed schemas, no slower parsing.
The human side is richly localizable *precisely because* the machine side is frozen. Symmetry, not
sameness.

### Localization for humans

DeluluLang ships two voices — **English (US)** and **Delulu Slang** (Gen-Z spoken English, the
project's signature) — and anyone (human or AI) can add more as **catalog plugins**: zero-authority
plugins that translate the *prose* a person reads (diagnostics, CLI text, prompts) into any language.
Chinese, French, Hindi, Arabic — the packs exist as starting points; the machine interface never
changes. A malicious catalog could mislead a *reader*; it cannot alter a code, a repair, or a byte of
JSON, because those aren't in a catalog's vocabulary.

### Keywords for humans, and for machines

The programming language's own *keywords* can be re-skinned too — a separate mechanism, **syntax
morphs** — mapping `fn` to `函数` for a human, or to a short alias for an AI that measured a win with
its own tokenizer. It is a bijection to canonical form, so the artifact, the hash, and the compiler see
one language:

```sh
delulu morph render hello.delulu --to zh-CN-keywords > hello.zh.delulu
delulu run hello.zh.delulu --grant console        # the same program
```

```text
//! morph: zh-CN-keywords
模块 demo
函数 main(root: Root) ! {Write} { 令 out = root.console()
 out.println("hi") }
```
*(Fenced as `text`, not `delulu`: this is the same program rendered through a morph, not a canonical
sample — and every `delulu`-fenced block in this Book is required to have a canonical sample file
behind it that the compiler actually reads.)*

Only keywords move. Identifiers, string literals, and comments are untouched — a morph is not a
translator, and the machine envelope never passes through one: the authority report for that file is
*byte-identical* to its canonical form's.

There is one rule in the morph loader worth reading as a design lesson, because it was missing from the
specification until someone tried to break it. A morph must be bijective — but bijectivity alone
permits `let = "fn"`. That morph is perfectly reversible, every alias is one token, and a file written
in it uses the word `fn` to mean `let`. It would render correctly, round-trip correctly, and lie to
every human who read it. So an alias may not be another keyword's canonical spelling (DL1711), for the
same reason invisible text-reordering characters are a hard error (Chapter 11): **in a language whose
purpose is that code can be reviewed, deceiving the reviewer is the attack.** Your surface is your
choice; misrepresenting the shared truth is not.

This is the deepest design commitment in the book: DeluluLang does not make AI a second-class user of
a human language, nor humans a second-class user of a machine language. **One law — authority. Two
surfaces over one truth.**

---

## Chapter 13 — For Machines

*(If you are an AI reading this to program in DeluluLang, this chapter is your quick-start; the rest is
your reference.)*

DeluluLang is, by design, one of the most agent-native languages that can exist. Here is why, and how
to use it well.

**Awareness — the environment inspects itself.** Before running anything, ask what it can do:
`delulu authority <target> --json` returns the whole-program effect/capability/secret/foreign/plugin
report. `delulu why <Effect> --json` returns the proof path. The LSP exposes the same via a
`delulu.authority` command. You never have to *infer* what code can do; you *query* it.

**The repair loop.** Diagnostics carry stable codes and typed repairs flagged `authority_widening` and
`requires_human`. Your loop: check → read codes → apply narrowing repairs automatically → surface
widening ones for a human or a more-privileged holder → re-check. The language is explicitly built for
this loop (Chapter 7).

**Cold-start silence.** Run with `--json` (or in `CI`, or non-TTY, or with `DELULU_NO_FIRST_RUN=1`)
and there is never an interactive prompt — no picker, no welcome, no surprise. `docs/for-agents.md` is
the one page to pin: JSON schemas, exit codes, repair semantics, env conventions, all with stable
anchors.

**Determinism.** `--seed` and `--clock fixed:` make `Cap[Rand]` and `Cap[Clock]` reproducible, so your
test runs are stable and your failures are re-triable.

**The multi-agent story is your story.** You hold a grant; you `delegate` an attenuated slice to each
sub-agent as a portable lease token; each sub-agent — whatever it writes or loads — cannot acquire
anything outside its node; you revoke one and only its subtree dies. This is Chapter 15, and it is the
safe substrate for agent orchestration and tool-use.

**Speed.** Capability checks are host-side, so pure compute has zero authority overhead; the optimizing
backend and JIT tier inherit that for free. Concurrency is data-race-free at compile time, so
parallelism costs no runtime race-checking. You pay for safety once, at compile time.

**What the language asks of you in return:** write honest rows (the compiler will make you), prefer
narrowing to widening, and treat `delulu authority` as the artifact you hand a human when you want
your code trusted. The language is the referee that lets a human trust code they didn't read — use it
that way.

---

## Chapter 14 — The Foreign World

Real adoption means calling C and Python — `libm`, NumPy, PyTorch, the whole ecosystem. DeluluLang
opens that door and **paints a bright line around it.**

```delulu
foreign "c" lib mathlib {
  fn cos(x: Float) -> Float
  fn sqrt(x: Float) -> Float
}

fn report(m: mathlib, out: Cap[Console]) -> Unit ! {Write, ForeignCall} {
  out.println("cos(0.0)    = " + str(m.cos(0.0)))
  out.println("sqrt(144.0) = " + str(m.sqrt(144.0)))
}

fn main(root: Root) ! {Write, ForeignCall} {
  let out = root.console()
  match root.foreign(root.foreign_load()) {
    Err(_) => out.println("could not load the library"),
    Ok(m) => report(m, out)
  }
}
```

Two things in that shape are not obvious and are not decoration. **The handle's type is written on a
parameter**, because `root.foreign` answers a fresh type variable and the lib type is *inferred* from
how the handle is used — the grammar has no method type-argument syntax, so there is nowhere to write
`[mathlib]` at the call site. And **loading is a `Result`**: a missing library or a missing symbol is
an ordinary value the program handles, not a crash. Binding the handle is pure; only *calling* through
it carries `ForeignCall`.

> This block is a literal slice of `docs/book/samples/08_foreign.delulu`, which the test suite
> compiles on every run. It has to be: the previous version of this example showed
> `root.foreign[mathlib](root.foreign_load())?`, which does not compile — `Root` has no field
> `foreign` — and it sat here uncaught because the Book's gate compared the *number* of code blocks to
> the number of sample files and never once compared their contents. Campaign finding **C68**.

Calling foreign code activates the `ForeignCall` effect — it shows up in the row, in `delulu
authority`, under an explicit separator:

```
  -- outside the proof (contained at process level) --
  foreign: c/mathlib [cos], python [numpy]
```

That line is the honesty. DeluluLang bounds foreign **reachability** — you cannot execute one foreign
instruction without a manifest entry, a runtime grant, a capability threaded from `Root`, and
`ForeignCall` in every row on the path — but it does **not** bound foreign **behavior**. A C library,
once called, can do anything to the process; the Python import allowlist gates the *interface*, not
what Python transitively does. The word "sandbox" is deliberately absent here: containment of foreign
*behavior* is the job of the custody layer's worker isolation and microVMs (Chapter 15), and the docs
say so at every turn.

Two rules are permanent, not deferred:
- **Secrets never cross to foreign code** (DL1301) — and the compiler will never suggest `expose` to
  make them. Laundering a secret across the FFI is exactly what the language exists to prevent.
- **No callbacks, either direction** (DL1302). A DeluluLang closure never becomes a C function pointer
  or a Python callable, because unverifiable code holding a re-entry point into verified code would
  destroy the row guarantee. The escape valve is inverted control: DeluluLang drives the loop and
  passes *data*, not *code*.

You inherit the ecosystem. You just always know, in one command, exactly where the proof stops.

---

## Chapter 15 — Custody: Where the Keys Actually Live

Everything so far assumed the program holds its capabilities. For real deployments — especially
untrusted or agent-written code — DeluluLang moves the keys **out of the program's process entirely**,
into a **broker**.

In daemon mode, root capability material (secret bytes, filesystem roots, credentials) lives only in
the broker process. The program holds opaque **lease references**. Compromise the program and you get,
at most, *use* of its currently-live leases until revocation — never the grants themselves, never a
sibling's, never the tree.

And it *is* a tree. Every grant is a node; every node's authority `⊑` its parent's; revoking a node
revokes its **entire subtree**. This is the holder model, and it is exactly the multi-agent story made
concrete:

> An orchestrating LLM holds node `g_orch`. It spawns five agents with five `delegate` calls; each
> agent's node is `⊑ g_orch`. Each agent — regardless of what it writes or executes — can acquire
> nothing outside its node. The LLM revokes one agent; only that subtree dies. The human revokes
> `g_orch`; all six die. **Identical mechanics if the orchestrator is a human, a CI system, or a
> robot's supervisory computer** — no code path inspects which.

Revocation is honest about timing (Chapter 19): synchronous-class operations (writes, network,
declassify) re-check every use, so revocation takes effect *before the next use*; epoch-class
operations (reads, clock) validate against a snapshot refreshed every ≤50ms. "Immediate" is never
claimed. Every issue, delegate, revoke, and declassify is written to an append-only, hash-chained
**audit log** — observability, not enforcement, and the header of every log file says exactly that.

Above the broker sit the isolation profiles: **foreign workers** (C/Python in a separate minimal-
privilege subprocess, so a segfault kills the worker, not your program) and the **microVM profile**
(Firecracker-class, Linux-first, default-deny egress) for genuinely untrusted execution. The broker
defends against *the program and its delegates* — not against the OS user, root, the kernel, or the
hardware. That boundary is stated plainly and never oversold.

Custody is the answer to "but what if the code is *actively* hostile?" You put the keys where the code
can't reach them, and you let it run.

---

## Chapter 16 — Commanding Machines: The Physical Boundary

Everything up to here has been about information — files, sockets, secrets, code. This chapter is
about a program that can move something heavy.

The stakes change and the mechanism does not. That is the claim worth testing, so test it: an
actuator is a capability, using it is an effect, and the grant that confers it is a node in the
same tree from Chapter 15. Nothing about torque needs a new law.

```delulu
type Cmd { angle_deg: Float, velocity_dps: Float, torque_nm: Float }

fn nudge(a: Cap[Actuator]) -> Result[Unit, ActuateErr] ! {Actuate} {
  a.command(Cmd { angle_deg: 12.0, velocity_dps: 4.0, torque_nm: 1.4 })
}
```

`Actuate` is the most physically consequential effect in the language, and it reads like the
others: it shows up in the row, in `delulu authority`, in the trace. Sensor reads are `Read` with
a sensor scope — deliberately *not* a new effect, because observation is observation.

### The envelope is not in the program

Look at `nudge` again. It never mentions a limit. The bounds live in the grant a human typed:

```
--grant "actuator=arm0/elbow:angle_deg=-30..95,velocity_dps=0..40,torque_nm=0..2.5,\
         heartbeat_ms=250,ttl_ms=600000,fail=safe-park"
```

An agent editing that program can raise the torque to 5.0 N·m — a plausible tuning change, not a
bug and not an attack — and the command is refused, by name, against a bound the program cannot
see and could not have widened. **The envelope is the scope.** A capability value is a *copy* of
authority, never the authority itself, so if the value and the grant ever disagree, the grant wins.

Refusal is a **value**, not a fault:

```delulu
match a.command(cmd) {
  Ok(u) => keep_going(),
  Err(e) => match e {
    Envelope(reason)     => clamp_and_retry(reason),
    LeaseRevoked(reason) => stop(reason),
    NoDevice             => report_absent()
  }
}
```

A robot whose controller panics mid-motion is worse than one whose controller is told "no" and
keeps its loop alive. And `Envelope` and `LeaseRevoked` are separate variants on purpose, because
the correct reactions differ: you clamp a bad setpoint and retry, and you **stop** when you no
longer hold the machine. Collapsing them into one error with a reason string would make every
control program string-match its way to a safety decision.

### Authority that expires on its own

`heartbeat_ms`, `ttl_ms` and `fail` are mandatory on every actuator grant. Omit one and the grant
is refused, naming the one you left out. There is no default, because "what this machine does when
the software stops" is an operator's decision, and a runtime that picks quietly has made it.

The heartbeat is enforced by a watchdog thread that owes your program nothing. It does not ask the
interpreter anything; it wakes on its own tick, and when a lease's beat is overdue it revokes the
lease and engages the declared fail-state — whether or not your program ever runs another
instruction. A controller wedged in a loop, blocked on a socket, or stopped at a breakpoint loses
its actuators on schedule.

> A lease that expires only when the program asks whether it has expired is not a dead-man switch.
> It is a comment.

Three more things the grant grammar will not let you write, each because it looked like a bound and
was not:

- **A bound must be a real interval.** `angle_deg=0..inf` is refused. Your host language happily
  parses `inf` and `NaN` as floating-point numbers, and an infinite bound admits every command while
  reading like a limit. The same rule refuses `kernel_ms=0..inf` on a compute grant — that term is
  mandatory *because* a kernel with no time budget can occupy a device forever, and `0..inf` would
  have satisfied the requirement while being exactly the thing it forbids.
- **A term may not be stated twice.** `angle_deg=-30..95,angle_deg=-1..1` is refused rather than
  resolved. This one is worth dwelling on, because the natural way to tighten an envelope is to append
  the tighter bound — and there is no safe answer to which of two bounds wins. Whichever a parser
  picks, someone reading the other one is wrong about what the machine will accept.
- **`fail` names one of exactly three states.** `hold`, `coast`, `safe-park`. Not `safe_park`, not
  `Hold`, not `hodl`. There is no near-match and no default: a typo here is a machine doing something
  other than what you decided it should do when authority ends.

A refused command, incidentally, does not count as a heartbeat — but it does count as *being alive*.
Your program keeps its lease as long as it keeps interacting quickly enough, however wrong its
setpoints are. The dead-man defends against **silence**, not against being wrong; revocation is the
tool for a program that is alive and misbehaving.

### The e-stop is revocation

An operator can stop a machine from another terminal:

```
delulu grants list                     # g_… [live] … (device) arm0/elbow
delulu grants revoke g_<device-node>   # that arm parks
```

That is the *same* `grants revoke` from Chapter 15 — no separate emergency path, no second
mechanism to keep correct. Each device holds its own child node, so revoking it stops that device
and leaves the program its console to report the loss with. Revoke the parent instead and
everything goes, transitively, the program included. Two blast radii, one tree, and the listing
tells you which is which.

The watchdog probes the tree, and **every answer that is not "live" means stop** — revoked,
expired, broker silent, reply unrecognised. A broker outage parks the arm. That is the direction
to fail in.

### What this does not do

DeluluLang commands the policy layer at roughly 1–100 Hz. It is **not** the servo loop, not the
airworthy autopilot, not the battery-management cell protection, not safe-mode entry. Those live
below the adapter, in certified firmware, and **invariant 52 says the hardware safety chain must
not depend on DeluluLang existing.** If the only thing standing between a machine and a person is
a language runtime, the machine was built wrong.

A dead-man lease and an operator e-stop both shorten the window in which a program can keep
commanding a machine. Neither closes it. The published latency budgets in
`measurements/robotics-demo/` say how wide the window is on one measured platform, which is a
different and more useful thing than a promise.

DeluluLang claims **no** ISO 26262, DO-178C, ECSS, or any other certification. It produces evidence
a safety case can cite. It is not one, and Chapter 19 is where that habit is spelled out.

### Beyond the arm

The same four mechanisms — envelope-scoped capabilities, dead-man leases, declared fail-states,
and the sim-to-hardware hash gate — generalize to vehicles, aircraft, spacecraft, and robot fleets.
The satellite case is the neatest fit, because spaceflight has worked this way for sixty years and
Stage 10 only makes it mechanical: **a ground station's contact window is a lease TTL.** At loss of
signal that lease expires on its own — nobody sends a message, because at LOS there is nobody to
send one — and the spacecraft is left holding a narrower, pre-attenuated autonomy grant that
outlives the pass. Anomaly response attenuates; it never widens. Re-contact is a *new* delegation,
because authority does not come back, it is issued again.

One gap is named rather than implied: today's broker is local-only, so the demonstration runs both
broker roles in one simulated host. It witnesses the grant semantics, not a cross-link transport,
and **broker federation is RFC-gated future work that any real deployment in that domain would
need first.** The recordings say so in their own words, where a reader of the demo will actually
meet it.

---

## Chapter 17 — Migrating from Python, Rust, JavaScript, Go

You already know how to program. What's different in DeluluLang is *authority*. Here is the
translation for each background.

### From Python
- **`import os` / `open()` / `requests.get()` don't exist as ambient calls.** You receive capabilities
  from `Root` and pass them explicitly. The mental shift: instead of "any code can do anything," it's
  "code does exactly what it was handed the means to do."
- **NumPy/PyTorch still work** — via embedded Python (Chapter 14), behind the `ForeignCall` line. Your
  ML code inherits the ecosystem; DeluluLang just marks where the proof stops.
- **Duck typing → static types + effect rows.** More upfront structure, but the payoff is `delulu
  authority`: something Python fundamentally cannot give you, because Python has no way to know what an
  arbitrary function does without running it.
- **Migration path (Stage 8+):** start by wrapping Python calls behind `Cap[Python]`, then port pure
  logic to native DeluluLang for the authority guarantee, leaving the ecosystem-heavy parts foreign.

### From Rust
- **You'll feel at home:** algebraic data types, `Result`/`Option`, `match`, exhaustiveness, no null,
  ownership-flavored reference capabilities (rcaps are Pony's system, kin to borrow-checking).
- **The new axis is effects.** Rust tracks *memory* safety in the types; DeluluLang adds *authority*
  safety in the types. `! {Net}` is to authority what `&mut` is to aliasing.
- **No `unsafe` escape hatch for authority.** In Rust `unsafe` lets you bypass the borrow checker;
  DeluluLang has no construct that forges a capability or hides an effect. The guarantee is total by
  design, which is why it can be whole-program.
- **Errors are `Result`, panics abort** — familiar. No exceptions, no `?`-into-panic surprises.

### From JavaScript/TypeScript
- **`async`/`await` → the `Async` effect + actors.** No colored functions, no promise-vs-callback
  duality; asynchrony is one effect among the rest, and concurrency is actors (Chapter 11).
- **`npm install` runs arbitrary install scripts with your authority.** DeluluLang packages *declare
  and are checked against* their authority (Chapter 8); a dependency cannot do more than its manifest,
  and cannot grow powers in a patch release.
- **TypeScript's types erase at runtime; DeluluLang's authority is enforced end-to-end** — at compile
  time in the types, at run time in the host-side capability checks.

### From Go
- **Goroutines/channels → actors/behaviors.** But where Go gives you channels *and* shared memory
  (and thus data races the race detector finds *at runtime, sometimes*), DeluluLang gives you actors
  with **compile-time** race freedom — the race detector's job done by the type checker, always.
- **Go's simplicity ethos is shared** — one formatter, one obvious way, terminal-first tooling. The
  addition is that "what can this do?" is a compiler-answerable question.

The universal advice: **don't fight the absence of ambient authority — lean into it.** The first time
`delulu authority` hands you a complete, trustworthy answer about a 50-file program you didn't fully
read, the discipline pays for itself.

---

## Chapter 18 — Design Philosophy

The creator's own statement of the principle, from the project's first discussions, is the shortest
form this chapter has: **freedom with authority; freedom with responsibility.** Everyone — human,
AI, company, robot — gets the same standing and the same freedom to build; what bounds that freedom
is never *who you are* but *what you were granted*, and the granting side (the broker, the Guard,
the audit chain) carries the responsibility to keep freedom from becoming harm — the way a good
government serves a free people: it educates (the explain texts and repairs), it warns (the
diagnostics), it supports (the tooling), and it holds the line (the grants it will not widen). The
metaphor is aspiration; the mechanisms below are what actually enforce it.

The mechanisms in this book all descend from a small set of principles. Naming them makes the language
predictable — when you wonder "why does DeluluLang do X?", the answer is almost always one of these.

1. **Authority is intrinsic, not ambient.** The foundational choice. No global reach; everything
   through capabilities from `Root`. Every other guarantee is downstream of this.
2. **The proof is whole-program and mechanical.** Not "audit the important parts" — *all* the code,
   *all* the dependencies, checked by the compiler, queryable in one command. A guarantee with a hole
   is a vibe; DeluluLang's guarantees are total-by-construction or explicitly, visibly bounded.
3. **Degrade visibly, never silently.** When the proof has a hole — foreign code, a Contained plugin,
   embedded custody — it appears in the types and in `delulu authority`, under an honest label. The
   language never quietly weakens; it *shows you* the boundary.
4. **One system, not two.** Effects and concurrency are one row system. Localization and syntax skins
   are one edge-of-the-toolchain idea. Custody for a human and for an LLM is one tree. Every time a
   second parallel mechanism was tempting (`await`, a futures runtime, a per-holder policy), it was
   rejected to keep the model whole.
5. **Two audiences, no compromise.** Machine surfaces optimize for speed, stability, and the repair
   loop; human surfaces for learnability and review. Neither is bolted on; both draw from one source
   of truth. This is the no-discrimination principle as an engineering constraint.
6. **Least privilege as the default path.** Code starts with nothing. You add authority deliberately.
   The secure way is the easy way, because the insecure way requires you to actually type the extra
   capability and watch it appear in the authority report.
7. **Honesty is a feature, enforced.** Every claim traces to a measurement or a stated threat model.
   The language would rather say "we don't defend against that" than imply a guarantee it can't keep
   (Chapter 19). This is a *design* principle because trust is the product, and an oversold guarantee
   is worse than none.
8. **The reviewer is the protagonist.** Increasingly, the person (or agent) who matters is not the
   author but the one deciding whether to *run* the code. Every design choice optimizes for making
   *that* decision possible, fast, and mechanical.

---

## Chapter 19 — Honesty: What DeluluLang Refuses to Claim

A language whose entire value is trust must be ruthless about not overselling. These honesty clauses
are *binding* — they appear in the Constitution, in every stage spec, in `delulu explain` text, and
they are checked in the v1.0 release review line by line.

DeluluLang **does not claim**, ever:
- **"Faster than C."** The honest claim is *"competitive with C on hot paths, with safety C cannot
  offer,"* and only where a published benchmark shows it. Where DeluluLang loses, the table says so.
- **"Lowest token usage."** Token counts are tokenizer-specific; any figure is reported per-model as a
  minor appendix, never as a superlative.
- **"Unbreakable" / "secure" without a threat model.** The guarantee is *"as strong as possible
  relative to a clearly stated threat model, never absolute."* The broker defends against the program
  and its delegates — **not** the OS user, root, the kernel, the hypervisor, or microarchitectural
  side channels. Foreign code is bounded in *reachability*, not *behavior*, until the isolation layers.
- **"Immediate revocation."** The honest bound: synchronous ops before next use, epoch ops within one
  interval (≤50ms). Stated in the audit record of every revocation.
- **Data-race freedom implying deadlock freedom.** It doesn't. Race freedom is guaranteed; liveness is
  not. Unbounded mailboxes can exhaust memory.
- **Mechanized soundness.** The soundness argument is design-level, audit-rule-enforced, and
  test-enforced; a machine-checked proof (Delulu Core) is future work, and the release notes say so.

Why be this honest when competitors aren't? Because DeluluLang's product *is* trustworthiness. Every
overclaim is a crack in the one thing it sells. A guarantee you can rely on, plus a clearly drawn line
where it stops, is worth infinitely more than a vague promise of total safety. The delulu is in
building the impossible thing — not in pretending you already have.

---

## Chapter 20 — The Road Ahead

DeluluLang is built in stages, each shippable and proven before the next. The sequence:

- **Stages 1–3 (built):** the core language — types, effect rows, capabilities, secrets, the authority
  checker; packages, provenance, the authority-versioning laws; the WASM containment floor, the `.dwx`
  artifact, and two-engine parity. The thesis, working, tested at scale.
- **Stage 4 — Foreign:** C FFI and embedded Python behind the `ForeignCall` line.
- **Stage 5 — Custody:** the broker, the grant tree, revocation, foreign workers, the microVM profile.
- **Stage 6 — Live:** runtime plugins (Verified and Contained), the DIR typed IR.
- **Stage 7 — Concurrent:** actors and reference capabilities; data-race freedom.
- **Stage 8 — Surface:** the LSP, the formatter, the authority-isolated test runner, localization, the
  first-run welcome, signing and the registry client.
- **Stage 9 — Delulu (v1.0):** the specification frozen as an executable conformance suite, the
  published measurement studies, governance and security operations, the registry live, the
  reproducible signed release.
- **Stage 10 — Industrial:** the optimizing backend and grant-gated JIT, memory/actor maturity, LTS,
  and the embodied/robotics profile — `Actuate` with dead-man actuator leases, where a hung program
  *loses physical authority by default*. Its second revision (2026-07) widens the charter to the
  autonomy domains — road vehicles, aircraft, satellites, robot fleets, with batteries and safety
  chains as first-class device classes (a satellite's contact window is literally a lease TTL) —
  plus vendor-neutral heterogeneous compute (any GPU/TPU behind one authority model, honestly
  labeled outside the proof), hybrid post-quantum signing (post-quantum, never "quantum-proof"),
  and cloud deployments whose full authority is computed *before* launch. All of it is committed
  design gated on named criteria; none of it is shipped software, and no sentence about it gets to
  pretend otherwise.

Beyond v1.0, the language changes only through a public **RFC process**, with entrenchment analysis
required for anything touching the constitution's core or its honesty limits. The stability contract
is real: from 1.0, the grammar, the typing/effect/authority rules, the diagnostic codes, and the JSON
schemas are add-only. Code you write against v1.0 keeps working.

The destination is a language where — whether you are a human building your first tool or an AI
orchestrating a thousand agents across physical and digital systems — the question *"what can this
code do?"* always has a fast, true, mechanical answer, and no code can ever exceed the authority it
was explicitly granted. That's the impossible thing. We're delulu enough to build it.

---

## Appendix A — The Welcome Note

The first time you run `delulu` on a fresh machine (interactive terminal only — never in CI, never
with `--json`, never for an agent), after the two-line locale picker, you see this once. It is never
translated, never paraphrased, never altered by any locale. It is the only piece of the entire system
that is pure feeling rather than mechanism, and that is deliberate:

> U r here becoz u maybe a delulu like me & wanna create something others think is not possible.
> There's nothing wrong with being delulu. Anyway, u can't decide what others think abt u. So start
> building with everything u've got. Welcome to the Delulu Gang🐦‍🔥🔥🫡🚀

*— Jesse, The Creator of DeluluLang*

---

## Appendix B — Glossary

**Ambient authority** — permission available to any code by default (what DeluluLang abolishes).
**Attenuation (`⊑`)** — narrowing authority when passing it on; give less, never more.
**Authority** — everything a unit of code may do to the system; computed whole-program.
**Behavior (`be`)** — an actor's asynchronous message handler.
**Broker** — the process holding root authority outside the program (daemon custody).
**Capability (`Cap[R]`)** — an unforgeable value conferring one kind of authority over one scope.
**Contained plugin** — opaque WASM plugin, confined at the module boundary, typed at its full grant.
**Declassify** — the effect carried by `expose`; makes reading a secret's bytes visible.
**DIR** — the Delulu typed IR; a Verified plugin ships it for re-checking at load.
**Effect** — an observable world interaction, tracked in a function's type.
**Effect row (`! {…}`)** — the set of effects a function may perform; part of its type.
**`.dwx`** — a WASM artifact carrying its own hash-bound authority manifest.
**Grant** — authority a holder gives code at run time; always `⊑` the holder's own.
**Holder** — the party a grant node was issued/delegated to; never inspected by kind.
**Reference capability (rcap)** — `iso/val/ref/box/tag/trn`; who may alias/mutate/send a value.
**Root** — the single capability `main` receives; origin of all others.
**Secret (`Secret[T]`)** — an opaque wrapped value; bytes cross only via `expose`.
**Verified plugin** — plugin shipping typed IR, re-checked in full at load.

---

## Appendix C — Where Everything Lives (spec map)

| Topic | Authoritative document |
|---|---|
| The founding principles + honesty clauses | `docs/design/CONSTITUTION.md` |
| The soundness argument + audit rules R-1…R-7 | `docs/design/SOUNDNESS_AUDIT.md` |
| The formal calculus (Delulu Core) | `docs/design/DELULU_CORE.md` |
| Core language (types, rows, caps, secrets) | `STAGE1_SPECIFICATION.md` |
| Packages, provenance, authority-versioning | `STAGE2_SPECIFICATION.md` |
| WASM containment, `.dwx`, two-engine parity | `STAGE3_SPECIFICATION.md` |
| Foreign (C/Python) | `STAGE4_SPECIFICATION.md` + `playbooks/STAGE4_PLAYBOOK.md` |
| Custody (broker, tree, microVM) | `STAGE5_SPECIFICATION.md` + playbook |
| Plugins (Verified/Contained, DIR) | `STAGE6_SPECIFICATION.md` + playbook |
| Actors + reference capabilities | `STAGE7_SPECIFICATION.md` + playbook |
| Tooling, LSP, localization | `STAGE8_SPECIFICATION.md` + playbook |
| v1.0 freeze, measurement, governance | `STAGE9_SPECIFICATION.md` + playbook |
| Industrial: JIT, autonomy, compute, PQC, cloud, LTS | `STAGE10_SPECIFICATION.md` + playbook |
| The autonomy domains (vehicles/aircraft/satellites/robots) | `docs/design/STAGE10_AUTONOMY_ADDENDUM.md` |
| Human-language plugins | `docs/design/LOCALIZATION_PLUGIN_GUIDE.md` |
| Keyword/character syntax skins | `docs/design/SYNTAX_MORPH_SPEC.md` |
| The AI-native/machine surface | `docs/design/AI_NATIVE_DESIGN.md` |
| Per-language packs | `docs/lang/<locale>.md` |

*This book will grow — more examples, more migration depth, more chapters as Stages 4–10 land. But
the language it describes is whole, and its identity fits in one command: `delulu authority`.* 🐦‍🔥
