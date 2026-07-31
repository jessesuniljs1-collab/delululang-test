# The Survey — the repository's map of itself

**Start here if you are about to read or change DeluluLang, and you are not yet sure where things
are.** This directory holds a map of this repository that is derived from the repository, checked
against it, and kept honest by a test.

| File | For | What it is |
|---|---|---|
| [`SURVEY.md`](SURVEY.md) | humans | The map: the crates, how they depend on each other, what each module is, where the decisions live. |
| [`survey.json`](survey.json) | tools & agents | The same graph, schema `survey/1`. One record per line. |
| [`DISCREPANCIES.md`](DISCREPANCIES.md) | everyone | Every place the repository currently disagrees with itself. |

All three are **generated**. Do not edit them. The command is always the same:

```
cargo run -p delulu-survey -- build
```

---

## The provenance law

> **Every edge in this map names the file and the line it was read from, and every edge whose
> target is a path was checked to exist. No edge is inferred from name similarity, embeddings, or
> proximity. A relation that cannot be pointed at in the text is not in the map.**

This is the point of the whole thing, so it is worth being blunt about what it rules out. A graph
that *guesses* relations gets more impressive as it gets less true: it will happily connect two
files because their names rhyme, and you cannot tell a real edge from a confident one. This map
cannot say anything it cannot cite. Ask it anything and it answers with a file and a line number,
so you can go and disagree with it.

Where the Survey is genuinely unsure, it does not draw a fainter edge — it files a
[discrepancy](DISCREPANCIES.md) and leaves the judgement to a person.

### How it stays honest

The Survey reads text, and text can be misread. So nothing it extracts is trusted on its own:
every relation is checked against a second, independent source, and **disagreement is reported
rather than resolved**.

| Claim | Checked against |
|---|---|
| `use delulu_check::…` in a source file | that crate's `Cargo.toml` dependencies |
| a `mod x;` declaration | an `x.rs` or `x/mod.rs` actually on disk |
| a `DLxxxx` cited anywhere | the registry in `crates/delulu-diag/src/codes.rs` |
| a path named in prose or a comment | the set of files that exist |
| a `D<n>` ruling citation | the build order that allocates that number |
| a count quoted in a document | the tree, recounted |

The reader's own scepticism is the last check, and it is the one the citations exist to serve.

---

## Not to be confused with the Atlas

`crates/delulu-atlas` is **the Atlas**, and it maps something else:

| | Atlas | Survey |
|---|---|---|
| Maps | a checked Delulu **program** | this **repository** |
| Derived from | compiler facts | the repository's own text |
| Needs | the compiler to work | nothing — it reads files |

They sit one level apart, and the naming is meant to keep them apart. In surveying, you *measure
the ground* before you *draw the atlas*. The Atlas answers "what can this program do?"; the Survey
answers "where is the code that decides that, and what else touches it?"

The Survey deliberately depends on **no other crate in this workspace**. A map you cannot open
while the thing it maps is broken is a map you cannot use to fix it.

---

## Using it

```
cargo run -p delulu-survey -- query crate:delulu-check    # a node, and everything touching it
cargo run -p delulu-survey -- rdeps crate:delulu-diag     # what breaks if I change this
cargo run -p delulu-survey -- findings                    # the discrepancy list
cargo run -p delulu-survey -- check                       # is the committed map current?
```

Node ids are readable and guessable:

```
crate:delulu-check                       mod:crates/delulu-check/src/ty.rs
doc:README.md                            test:crates/delulu/tests/guard_cli.rs
code:DL0501                              ruling:S10-D64
finding:C69                              dir:docs/design
```

`rdeps` is the one to reach for first when changing anything. `rdeps crate:delulu-diag` returns
every crate, module and test that touches it, each with the line that proves it.

### If you are a model

Read `SURVEY.md` — it is about 25 KB and it is the whole shape of the project. Then use `query`
and `rdeps` for specifics rather than reading `survey.json`, which is a megabyte and meant for
programs. Two habits worth having here:

- **Check `DISCREPANCIES.md` before believing a number you read in a document.** The repository is
  honest and long-lived, which means some of its prose is older than its code.
- **The map is not the territory, and it says which parts it cannot see.** `crates/delulu-survey/src/rust.rs`
  documents exactly what a lexical reader misses — macro-generated items, `#[cfg]`-gated modules,
  re-export chains. Those produce *missing* edges, never wrong ones.

---

## Keeping it true

**Regenerate the Survey whenever you change the repository, and commit the result with the change
that caused it.** One command does the whole loop — status, regenerate if behind, integrity checks:

```
delulu doctor
```

It writes **nothing** when the map is already current, so running it is not a change to the
repository. `delulu doctor --check` reports without ever writing, for a hook or a CI step.

This is not a request to remember something. `cargo test --workspace` runs
`the_committed_map_matches_the_tree`, which rebuilds the map and compares it to what is committed.
If the tree moved and the map did not, the suite fails and names the first line that differs. The
remedy is the one command at the top of this page.

Five other tests hold the map to its own standard: every edge carries a citation, no edge dangles,
the totals agree with the contents, every workspace member appears, and **two builds of the same
tree produce the same map**. That last one is not a formality — the first version of the Survey
walked its own output directory, so every run found new relations inside the report the previous
run had written, and it would never have converged.

---

## What it will not do

- **It does not count tests.** The number of passing tests comes from `cargo test`, not from
  reading files, and the Survey does not restate numbers it did not measure. It will tell you
  which documents quote a test count so you know what to re-check after a run.
- **It does not judge.** `DISCREPANCIES.md` reports that two things disagree; which one is wrong is
  a person's call.
- **It does not touch the authority model.** The Survey is a reader. It has no opinion about what a
  grant means, and changing what it maps never changes what the language does.
