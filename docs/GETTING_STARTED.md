# Getting started with DeluluLang

This page takes you from an installed `delulu` to writing real programs. It assumes you can
program in something else and want the parts of DeluluLang that are *not* like that thing.

**Every code block on this page is a real file in [`examples/guide/`](../examples/guide/), and a
CI gate checks that all of them compile and that none of them fails at runtime with a compiler
bug.** That gate exists because this project shipped a reference sample that checked clean and
could not run; see [`design/HARDENING_CAMPAIGN.md`](design/HARDENING_CAMPAIGN.md) C13.

Install first — see [Installing](../README.md#installing) in the README.

---

## 1. The one idea that is different

Your program starts with **nothing**. Not "nothing dangerous" — nothing at all. It cannot print,
read a file, tell the time, or generate a random number until a human hands it that right on the
command line.

```delulu
module hello

fn main(root: Root) ! {Write} {
    let out = root.console()
    out.println("Hello, Delulu")
}
```

```sh
delulu run hello.delulu                    # DL0703 — `console` was not granted
delulu run hello.delulu --grant console    # Hello, Delulu
```

Two halves make this work, and you need both:

- **`Cap[Console]`** is the *permission* — an unforgeable value you must be handed.
- **`! {Write}`** is the *row* — the function's public promise about what it may do.

A capability without a matching row is a type error. A row you cannot back with a capability is a
promise you cannot keep. Together they mean `delulu authority` can compute, from the code alone,
everything a program can do to your machine:

```sh
delulu authority hello.delulu
```

```
Authority of `hello` — what this program can do to your system:
  effects:      Write
  capabilities:
    - Console  stdio
  secrets:      (none)
  pure fns:     (none)
  foreign:      (none — no code outside the guarantee)
```

## 2. Types — records and sums

[`examples/guide/01_types.delulu`](../examples/guide/01_types.delulu)

A **record** uses braces:

```delulu
type Point {
    x: Float,
    y: Float,
}
```

A **sum** uses `=` and `|`. This is the one most people guess wrong — it is not braces:

```delulu
type Shape = Circle(Float) | Rect(Float, Float) | Empty
```

Sums are recursive, which is what makes trees, lists, and syntax nodes possible:

```delulu
type Tree = Leaf | Node(Tree, Int, Tree)
```

`match` takes a sum apart, and the checker proves the match is **exhaustive** — miss a variant and
you get `DL0407` before the program runs:

```delulu
fn area(s: Shape) -> Float {
    match s {
        Circle(r) => 3.14159 * r * r
        Rect(w, h) => w * h
        Empty => 0.0
    }
}
```

## 3. Lists and `Option`

[`examples/guide/02_option_and_lists.delulu`](../examples/guide/02_option_and_lists.delulu)

`List.get` is **total**: it returns `Option[T]`, never a crash and never a silent wrong answer.
There is no way to index a list unsafely, so "index out of bounds" is not a class of bug here.

```delulu
fn at_or(xs: List[Int], i: Int, fallback: Int) -> Int {
    match xs.get(i) {
        Some(v) => v
        None => fallback
    }
}
```

`push` **mutates in place and yields `Unit`** — it is a statement, not an expression. Note the
binding is `let`: what changes is the list cell, not which list the name refers to.

```delulu
fn evens_up_to(n: Int) -> List[Int] {
    let out = []
    var i = 0
    while i <= n {
        if i % 2 == 0 { out.push(i) }
        i = i + 1
    }
    out
}
```

There is no `()` unit literal, so "else do nothing" is written by leaving the `else` off — an `if`
without `else` is a statement.

## 4. Errors

[`examples/guide/03_errors.delulu`](../examples/guide/03_errors.delulu)

There are no exceptions. A function that can fail says so in its type, and the caller deals with
it. No invisible control flow is what keeps the effect row an honest summary of what a call can do.

The error type is an ordinary sum, so a caller can `match` on *why* something failed instead of
picking apart a message string:

```delulu
type ParseErr = Empty | NotANumber(Str)

fn to_int(s: Str) -> Result[Int, ParseErr] {
    if s.len() == 0 {
        Err(Empty)
    } else {
        match parse_int(s) {
            Some(n) => Ok(n)
            None => Err(NotANumber(s))
        }
    }
}

fn describe(s: Str) -> Str {
    match to_int(s) {
        Ok(n) => "parsed " + str(n)
        Err(e) => match e { Empty => "empty input", NotANumber(bad) => "not a number: " + bad }
    }
}
```

That also shows the conversion you will write constantly: the prelude builtin `parse_int` answers
`Option[Int]` — "a number or not" — and wrapping it in a `Result` records *which* failure happened.

`?` propagates an `Err` to the caller unchanged, and only where the error types already line up —
there is no implicit conversion, because a silent error conversion is exactly the invisible
behaviour this language refuses:

```delulu
fn read_two(fs: Cap[FsRead]) -> Result[Str, IoErr] ! {Read} {
    let a = fs.read_text("a.txt")?
    let b = fs.read_text("b.txt")?
    Ok(a + b)
}
```

**Filesystem calls fail with `IoErr`; network calls fail with `NetErr`.** Different sums, different
variants, and `?` will not bridge them for you.

## 5. Effects and rows

[`examples/guide/04_effects.delulu`](../examples/guide/04_effects.delulu)

Omit the row and the function is **pure** — it may perform no effect at all, and the compiler
proves it rather than trusting a comment:

```delulu
fn normalize(name: Str) -> Str {
    name.trim()
}
```

Effects accumulate up the call graph. A function that calls an effectful one must declare that
effect too, all the way up to `main`:

```delulu
fn greet(out: Cap[Console], name: Str) ! {Write} {
    out.println("hello, " + normalize(name))
}

fn banner(out: Cap[Console]) ! {Write} {
    greet(out, "world")
}
```

**Row polymorphism** is what stops that from being unbearable. `e` is a row variable, so `apply`
has *exactly* the row of the function it is handed — a pure callback keeps it pure:

```delulu
fn apply[T, U, e](f: fn(T) -> U ! e, x: T) -> U ! e {
    f(x)
}
```

Named functions are values: `apply(double, 21)` works, as does `xs.map(double)`.

## 6. Capabilities — effects and resource kinds are different words

[`examples/guide/05_capabilities.delulu`](../examples/guide/05_capabilities.delulu)

This trips up nearly everyone:

| In the row (an **effect**) | In the type (a **resource kind**) |
|---|---|
| `! {Read}` | `Cap[FsRead]` |
| `! {Write}` | `Cap[FsWrite]`, `Cap[Console]` |
| `! {Net}` | `Cap[Http]` |
| `! {Declassify}` | `Cap[Declassify]` |

Writing `Cap[Net]` is `DL0307`. The split is deliberate (constitution invariant 6): **the row
proves what *kind* of thing a function can do and is checked at compile time; the capability's
scope decides *which* file or host and is checked when it runs.** Nothing claims a compile-time
guarantee about a specific path or hostname — the language cannot prove one without dependent
types, and it refuses to imply otherwise.

Minting a capability is **pure**. Deriving the handle performs no effect; *using* it does.

Secrets have no string form, no equality, and no way out except `expose`, which needs
`Cap[Declassify]` and carries the `Declassify` effect — so "this function can reveal a secret" is
visible in its type. `verify` is the constant-time comparison you usually want instead.

## 7. Actors

[`examples/guide/06_actors.delulu`](../examples/guide/06_actors.delulu)

An actor owns its state; a reference to one is `tag`, which lets you send and read nothing.

```delulu
actor Worker {
    var done: Int

    new(worker_id: Int) { self.done = 0 }

    be job(n: Int) {
        self.done = self.done + n
    }
}
```

**The rule that is easy to miss:** sending to another actor is an *effect*. A behavior that sends
must declare `! {Async}`, like any other effect anywhere else. Concurrency is not a parallel
system here — it rides the same rows as everything else.

```delulu
be dispatch(w: tag Worker, n: Int) ! {Async} {
    w.job(n)
}
```

## 8. Packages

A package is a directory with a `delulu.toml` and a `src/`:

```toml
[package]
name = "greeter"
version = "0.1.0"
kind = "bin"

[authority]
effects  = ["Read", "Write"]
fs.read  = ["./config"]
fs.write = []
net      = []
secrets  = []
```

```delulu
// src/main.delulu
module greeter
import greeter.greetings

fn main(root: Root) ! {Write} {
    let out = root.console()
    out.println(salutation("world"))
}
```

```sh
delulu check examples/greeter        # resolves deps, verifies pins AND authority
delulu authority examples/greeter    # the whole package's authority
```

**Sources must live under `src/`.** A `.delulu` file sitting beside `delulu.toml` is not part of the
package — the toolchain only looks in `src/`. `delulu build` refuses a package it found no modules in
rather than reporting a clean build of nothing, so if you see

```
note: no `.delulu` modules found under <pkg>/src — a DeluluLang package keeps its sources in `src/`
```

move the file into `src/`. Note also which commands take which argument: `build`, `check`,
`authority`, and `plugin build` accept a **package directory** (`check` and `authority` also accept a
single file); `run` takes a **single file** only. Naming a directory where `run` expects a file says
so and points you at `build`.

The `[authority]` block is a **ceiling**. Code that exceeds what the manifest declares is an error,
so a dependency cannot quietly grow new powers in a patch release — that is what
`delulu authority --diff <old.lock> <new.lock>` is for, and it belongs in your CI.

**`import` must come before the first item**, directly under the `module` header.

## 9. Tests

`test` blocks live next to the code and are compiled out of builds:

```delulu
test "double doubles" {
    assert_eq(double(21), 42)
}

test "the console writes under a declared row" ! {Write} {
    let out = test_root.console()
    out.println("hi")
}
```

Tests hold **no ambient authority**: each gets exactly its declared row, bounded by the package's
`[test-authority]` ceiling. An absent ceiling means pure. Run them with `delulu test`.

## 10. Things that will bite you

Collected from actually writing programs against this language. Each one is a rule that is real,
enforced, and easy to meet once you know it.

| Symptom | Cause |
|---|---|
| `DL0201` on `type T { A, B(X) }` | Sums use `=` and `\|`, not braces. |
| `DL0405` "has no method" on a list element | `List.get` returns `Option[T]`. Match it. |
| `DL0401` expected `List[Int]`, found `Unit` | `push` mutates and returns `Unit`; it is a statement. |
| `DL1603` `ref` where `val` is required | A `let`-bound list is `ref`; parameters default to `val`. Annotate `let xs: val List[Int]`, or take `ref List[Int]`. |
| `DL0202` expected an expression, found `)` | There is no `()` literal. Drop the `else` instead. |
| `DL0201` "a match arm's body is an expression" | Wrap an assignment in a block: `Some(d) => { acc = d }`. |
| `DL0302` "is a prelude builtin" | You named a function `str`, `len`, `int`, `float`, `parse_int`, `range`, `push`, `load`, `assert`, `assert_eq`, `Ok`, `Err`, `Some`, or `None`. Rename it. |
| `DL0307` "not a capability resource kind" | You wrote an effect name in `Cap[…]`. See §6. |
| `DL0501` "performs effect Async" in a `be` | A behavior that sends to another actor must declare `! {Async}`. |
| `DL0208` on a perfectly good `import` | Imports go before the first item. |
| `DL0905` at runtime | Recursion passed the interpreter's bound of **10,000** frames. It is not configurable. |
| `DL0703` at runtime | You did not grant the capability. The message now tells you the exact flag. |
| `DL0107` "bidirectional control character" | Your source (often pasted or AI-generated) contains an invisible text-reordering character that can make code render differently than it runs. It is refused, not warned. If a *string* truly needs the code point, write it as `\u{202e}`. |
| Integer result is wrong | It is not — overflow is `DL0901` and aborts. Arithmetic is checked, not wrapping. |

## 11. Where to go next

- [The DeluluLang Book](book/THE_DELULULANG_BOOK.md) — 20 chapters. Chapter 6, *Reading Authority*,
  is the flagship skill; Chapter 11 is for reviewing AI-written code.
- [`for-agents.md`](for-agents.md) — pin this if you are an agent harness.
- [`reference/`](reference/) — grammar, tokens, primitives, diagnostics, CLI contracts.
- `delulu explain <CODE>` — long-form docs for any diagnostic you hit.
- [`design/HARDENING_CAMPAIGN.md`](design/HARDENING_CAMPAIGN.md) — what is currently known to be
  wrong with all of this.
