# Tier 4 — software-engineering-scale, multi-module

Four packages, seven modules, dependency depth three, one diamond.

```
station (bin)  ──┬── policy  (lib, effects = [])      ──┐
                 ├── archive (lib, effects = [Write])  ─┤── reading (lib, effects = [])
                 └── reading (lib, effects = [])       ─┘
```

`station` also names `reading` directly. It has to: a **transitive** dependency is not importable,
so a package may only mention what its own manifest declares. That rule is why the graph above has
an edge that looks redundant and is not.

## What it is for

The single-file tiers can show a language feature. They cannot show the thing DeluluLang is
actually built around, which only appears once a program has more than one author-sized piece:

- **Authority is per package, and the reader can see it in four short manifests.** Three of the
  four packages declare `effects = []`. Exactly one may write, to exactly one directory. Nothing
  in the source can exceed that — `delulu build` recomputes it and refuses.
- **A dependency's ceiling is stated by its consumer**, in the `[dependencies]` line, not asserted
  by the dependency about itself.
- **The type that crosses every boundary carries no authority.** `Sample` is declared in the
  package that has none.
- **Both senses of "module" are here.** `reading` is three compilation units inside one package;
  the pipeline is four packages. They are different mechanisms — a sibling module needs no
  dependency entry, and `import` is per module, so a package importing something does nothing for
  its siblings.
- **Export is explicit in both directions.** `pub fn` exports a function, `pub type` exports a
  type, and `pub import` re-exports a module's names to *your* consumers. A public signature that
  mentions a type you did not re-export does not resolve for the package that depends on you.

## Running it

**This program runs.** It did not when the tier was written — `delulu run` took a single `.delulu`
file or a `.dwx`, so `kind = "bin"` was a manifest field the toolchain could not honour, recorded as
campaign finding **C59**. Ruling **D61** closed it.

```
delulu run       tests/corpus/tier4-multimodule/station \
    --grant console --grant fs.read=./data --grant fs.write=./out
delulu build     tests/corpus/tier4-multimodule/station     # the whole graph
delulu authority tests/corpus/tier4-multimodule/station     # what it can do, computed from the code
delulu why Write tests/corpus/tier4-multimodule/station     # and why it can do that
```

Given a `data/telemetry.txt`, it classifies each reading, rejects malformed lines with distinct
reasons, archives the good ones through the one package granted `Write`, and totals the alarms.
`crates/delulu/tests/package_run.rs` asserts that output and the files it writes; `conformance.rs`
builds every corpus package on every test run.

## The one thing it still cannot do

A package whose modules declare **the same top-level name** in two places is refused by the runner —
not silently mis-resolved. Visibility is per module, so two private `helper`s are legal, and running
the program flattens the modules into one scope where they are not distinguishable. `check`, `build`
and `authority` all handle such a program; only `run` refuses, and it says so in those words rather
than blaming the author. Lifting that needs per-module resolution inside the interpreter, for which
the checker already computes the map (`Program::call_owner`).
