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

## What it does not show

**These packages are checked, not executed.** `delulu run` takes a single `.delulu` file or a
`.dwx` artifact; there is no way to run a multi-package program on the interpreter, so the
evidence this tier provides is compile-time evidence. `kind = "bin"` in `station/delulu.toml`
declares an intent the toolchain cannot yet carry out. Recorded as campaign finding **C59** rather
than papered over — a corpus tier that implied more than it demonstrates would be worse than the
empty directory this replaces.

## Running the checks

```
delulu build tests/corpus/tier4-multimodule/station     # the whole graph
delulu authority tests/corpus/tier4-multimodule/station # what it can do, computed from the code
delulu why Write tests/corpus/tier4-multimodule/station # and why it can do that
```

`crates/delulu/tests/conformance.rs::accepting_packages_build_clean` runs the first of these on
every package in the corpus on every test run.
