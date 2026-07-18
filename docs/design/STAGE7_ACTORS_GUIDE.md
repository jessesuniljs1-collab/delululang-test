# DeluluLang Actors — the v0.7 Guide

**Status:** Living user guide. Normative: `STAGE7_SPECIFICATION.md`; rulings:
`STAGE7_BUILD_ORDER.md`; `delulu explain E-ACTOR` for the terminal version.

DeluluLang v0.7 gives you the actor model with **compile-time data-race freedom**: the
checker proves, before your program runs, that no actor can ever touch another actor's
mutable state. There is no lock, no atomic, no runtime race detector standing between you
and a heisenbug — the race is refused at `delulu check`, or it cannot happen.

## 1. Writing an actor

```delulu
module app

actor Counter {
  var count: Int                        // actor-internal state: owned, isolated
  let label: Str
  new(start: Int, label: Str) {         // exactly one constructor; assign EVERY field
    self.count = start
    self.label = label
  }
  be add(n: Int) { self.count = self.count + n }       // a behavior: an async message
  be report(out: Cap[Console]) ! {Write} {             // rows work exactly as everywhere
    out.println(str(self.doubled()))
  }
  fn doubled() -> Int { self.count * 2 }               // sync method: callable from self ONLY
}

fn main(root: Root) ! {Async, Write} {
  let c = spawn Counter(0, "hits")      // type: tag Counter — opaque identity
  c.add(5)                              // a send: {Async} ∪ row(add)
  c.report(root.console())              // the console cap is val — safely sendable
}
```

- `spawn A(args)` creates the actor (the constructor runs as its first turn) and yields a
  `tag A` reference. `a.behavior(args)` sends a message — always asynchronous, always
  `Unit` at the send site.
- Behaviors are **atomic**: an actor processes one message at a time, each to completion.
  There is no `await` and no suspension — deliberately (one concurrency system, not two).
- Outsiders hold `tag`: no field access, no sync calls (`DL1604`) — messages are the only
  cross-actor interface. `self` is the one `ref` to an actor that exists.
- The constructor must assign every field; `var` fields are legal actor state (this is NOT
  the banned module-level `var` — actor state is owned and reached only via its own turns).

## 2. Reference capabilities in five minutes

Every value has an rcap — who may alias, mutate, and send it:

| rcap | you | others | sendable |
|---|---|---|---|
| `iso` | read+write | nothing | yes, via `consume` |
| `trn` | read+write | local read only | no |
| `ref` | read+write | local read+write | no |
| `val` | read | global read (immutable forever) | yes |
| `box` | read | local writers may exist | no |
| `tag` | identity only | anything | yes |

Defaults when you write none: plain data (`Int Float Bool Str Unit`), capabilities,
secrets, plugin handles, and composites of only such types are `val`; other records and
lists are `ref`; fn-typed positions are `box` (any closure is callable there); actor types
are always `tag`; `PyObj` is `ref` and **pinned to its creating actor** (CPython affinity —
sending one is DL1601, always).

**Sendability is the whole game.** A message argument must ARRIVE at the parameter's rcap:

```delulu
be feed(vs: List[Int])        // List[Int] defaults val — callers may send immutable data
be take(vs: iso List[Int])    // unique transfer — callers must `consume`
```

- Sending a `ref List[Int]` → **DL1601** (it could be mutated behind the receiver's back).
- Sending an `iso` without `consume` → **DL1601 with an exact repair** that inserts it.
- After `consume xs`, the binding is dead — any later use is **DL1602** (including
  possibly-consumed branches and loop-carried consumes).
- Build-mutable-then-send with `recover`:

```delulu
let xs: iso List[Int] = recover { let ys = [1]  ys.push(2)  ys }
s.take(consume xs)
```

Inside `recover` only `val`/`tag` outer bindings are visible (**DL1605** otherwise; a
consumed `iso` may transfer in). A closure is `val` (sendable) iff every capture is
immutable — a `val` claim over a `ref` capture is **DL1603**.

## 3. Async is an effect — and only an effect

A function that spawns or sends is `!{Async, …}`. Every send site's row contains the target
behavior's row, so `row(main)` still bounds the whole program — across every actor:

```text
$ delulu why Write app.delulu        # main → send → Counter.report → console.println
```

`std.actors.Promise[T]` is a library **actor** (`new` / `be fulfill(v: val T)` /
`be then(f: val fn(val T) -> Unit ! e)`): first fulfill wins, later ones are dropped and
counted; an effectful callback surfaces `e` into the `then` caller's row; `fulfill`
callers stay `{Async}` (the spec's accounting: row `e` joins **then**'s send row).

## 4. Running

```text
delulu run app.delulu --grant console [--actors-threads N] [--on-quiesce report]
                      [--on-actor-death abort] [--assert-trace] [--debug-rcaps]
```

- Exit is **quiescence**: `main` returned, all mailboxes empty, no turn running.
- A behavior fault **poisons** its actor: later sends to it are dropped and counted,
  reported at exit; `--on-actor-death abort` opts into whole-program abort. There is no
  supervision/restart in v0.7.
- `--trace-effects` records gain `actor`/`member`/`turn`/`cause`; `--assert-trace` checks
  every effect against the executing behavior's row AND the send site's row via the cause
  chain. `--debug-rcaps` verifies every statically-proven iso move is truly unaliased at
  the boundary (a violation is DL1610 — a compiler bug to report, never a runtime safety
  net: race freedom is static).
- The native scheduler pins each actor to a worker at spawn (worker-owned actors — zero
  `unsafe` anywhere in the runtime); message payloads move by rebuild, observationally the
  spec's pointer handoff. The WASM engine runs actors cooperatively, single-threaded,
  within the backend's compilable subset, and says so in its output.

## 5. Honesty and threat-model caveats (spec §11, verbatim)

- Compile-time data-race freedom covers **DeluluLang code**; foreign code and Contained
  plugins are bounded by their Stage-4/5/6 layers, not by rcaps.
- **Deadlock, livelock, starvation, and mailbox exhaustion are not prevented** — the
  guarantee is race freedom, not liveness. Unbounded mailboxes can exhaust memory;
  backpressure is post-1.0.
- The rcap tables are adopted from Pony's proven design; our own property-test validation
  (criterion 8) is a ship-gate, and mechanized proof remains Delulu Core future work.
- WASM-engine concurrency is cooperative single-threaded in v0.7 — semantics identical,
  parallelism absent, labeled in output.
- Message *ordering* is per-sender-pair FIFO only; no global order, no delivery-time bounds.

## 6. What v0.7 deliberately does not do

`await`/suspension inside behaviors (rejected, not deferred — one system); supervision
trees; distributed actors; multi-threaded WASM (Stage 10); per-actor broker nodes
(authority slices are value-level: pass attenuated caps at `spawn`); field `consume`;
cross-module actors (actors and their uses live in one module in v0.7); actors inside
plugins. The full ledger with reasons: `STAGE7_BUILD_ORDER.md` §3 and §5.
