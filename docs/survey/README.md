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
| a `DLxxxx` cited anywhere | the registry in `crates/delulu-diag/src/codes.rs`, **and** the `UNALLOCATED` table beside it, which records why a code is absent — retired, never allocated, reserved, specified-but-not-implemented, or a test sentinel. A code in neither is reported. |
| a path named in prose or a comment | the set of files that exist |
| a `D<n>` ruling citation | the build order that allocates that number |
| a count quoted in a document | the tree, recounted |

The reader's own scepticism is the last check, and it is the one the citations exist to serve.

---

## How it is put together

`delulu doctor` is the interface; it is **not** where anything is decided.

```
delulu (CLI)  ──depends on──▶  delulu-survey  ──depends on──▶  (nothing in this workspace)
   doctor.rs                     health.rs
   • environment checks          • build the map, decide if it is behind, write it
   • orchestration               • the integrity invariants
   • rendering                   • the discrepancy tally
                                 • what a DeluluLang checkout looks like
```

Three properties hold this shape, and each is a test in `crates/delulu-survey/tests/architecture.rs`
rather than a promise in this paragraph:

1. **The Survey depends on no sibling crate.** Not the checker, not the runtime, not the
   diagnostics. This is what lets you read the map in the situation where you most need one — the
   tree is mid-refactor and the compiler does not build. A Survey that needed `delulu-check` would
   require building the thing the map was going to help you fix.
2. **The dependency runs one way.** The CLI knows about the Survey; the reverse would be a cycle
   and would put the map behind the binary it exists to help repair.
3. **The invariants have exactly one home** — `delulu_survey::integrity`. `delulu doctor` renders
   whatever that list returns and **contributes no invariant of its own** (it adds three status
   rows around them — where the tree is, whether the map was behind, what the discrepancies
   tally — and those are reports, not checks); the freshness suite asserts every entry holds. So
   **adding an invariant is a one-line change that the command reports and the test enforces on the
   same commit.** They were briefly written twice — once in the CLI, once in the test file — and
   neither was the source; a fourth added to either would have been invisible to the other.

The environment checks stay in the CLI, and that is the same boundary seen from the other side:
whether Python is compiled in, or the audit chain verifies, is not something the map knows or
should learn. Teaching the Survey about the broker would cost it property 1.

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
cargo run -p delulu-survey -- rdeps crate:delulu-diag     # what points at it — ONE HOP
cargo run -p delulu-survey -- findings                    # the discrepancy list
cargo run -p delulu-survey -- check                       # is the committed map current?
```

**The transitive questions** — the ones you actually have before changing code:

```
cargo run -p delulu-survey -- impact mod:crates/delulu-check/src/check.rs
cargo run -p delulu-survey -- affected-by crate:delulu-check
cargo run -p delulu-survey -- path crate:delulu crate:delulu-broker
```

`impact` is the honest form of "what breaks if I change this", and the difference is not small:
`mod:crates/delulu-check/src/check.rs` — the module that decides what type-checks — has **one
structural** edge arriving at it and reaches **134** nodes transitively. `rdeps` answers one hop;
reach for `impact` when the question is blast radius. `affected-by` is the same walk in the other
direction, and `path` prints one chain in full. `--depth N` bounds the first two.

**Every hop is cited, exactly like a single edge**, and that is what makes a chain admissible here
at all. Each reached node names the node it came from and the file and line the hop was read from,
so the whole chain can be walked back and disagreed with.

One caveat worth knowing when you read a raw in-degree: **documents are part of the map, so writing
about a file changes how many edges point at it.** `check.rs` had one incoming edge before this
paragraph existed and eight after, because several documents now mention it by name — each a real,
cited `links-to`. Nothing about the code moved. `impact` is unaffected (it follows only relations
that propagate), and this is why it compares against the structural subset rather than the raw
count.

### Before you change something: is it entrenched?

Some paths in this repository require the **project lead specifically, not any maintainer** —
Constitution §10, invariant 44. `query` says so, first, before any edge:

```
$ delulu-survey query doc:docs/design/CONSTITUTION.md
  ENTRENCHED — changing this needs @PENDING-PUBLIC-project-lead specifically, not any maintainer
          matched by `/docs/design/CONSTITUTION.md` at .github/CODEOWNERS:16
```

Read from `.github/CODEOWNERS`, cited to the line, and the owner string is carried **verbatim** —
the map has no opinion about who that handle is. Nineteen nodes carry it today: the constitution and
`DELULU_CORE.md`, `STABILITY.md`, `/rfcs/`, `SECURITY.md` and `/docs/security/`, the soundness audit
and its laundering suite, and the conformance machinery including `witnesses.toml`.

Two decisions worth knowing. **The `*` catch-all is deliberately ignored** — a rule matching every
path separates nothing, and marking all 900-odd nodes would make the word meaningless. And **a rule
that matches no path is an `error`**, not a note: renaming an entrenched file silently un-entrenches
it, and a rule guarding nothing reads in a diff exactly like a rule guarding something.

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

- **A cross-language coupling nobody wrote down is invisible, and that has already cost a real bug.**
  The Survey maps what the source *cites*: a `References` edge exists because a comment names a path.
  So a dependency that crosses out of Rust — the VS Code extension's dependence on `lsp.rs`, say — is
  in the map only if someone said so in a comment.

  Until 2026-08-07, `delulu-survey impact mod:crates/delulu/src/lsp.rs` reached exactly one node,
  `main.rs`, while `editors/vscode/extension.js` depended on that file so closely that a change to it
  left the language server dead in every workspace. The map was not wrong; it was silent, which is
  worse when it is consulted and believed. **A blast radius that omits the file you are about to
  break is more dangerous than no blast radius at all.**

  The fix is the one the design already implies: **name the other file in a comment.** `lsp.rs` now
  cites both `editors/vscode/extension.js` and `crates/delulu/tests/editor_contract.rs`, and the edge
  appears. If you create a coupling the compiler cannot see, write it down or the map will not know.

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
- **It does not compose relations that do not compose.** Every edge here is read from a file and
  cited, but a *path* is a separate claim from an *edge*. `A depends-on B` followed by
  `B depends-on C` genuinely means C's change can reach A. `README links-to CONTRIBUTING` followed
  by `CONTRIBUTING references cli.rs` means nothing about what breaks — it is two unrelated
  sentences laid end to end.

  This is not a hypothetical. The first version of `impact` followed every edge kind and reported
  **236 nodes reachable from every starting node in the repository**, including `doc:README.md` —
  a confident, precise, meaningless number. So a transitive walk follows only relations that
  propagate: crate dependencies, module declarations, use-sites, test targets. Narrative relations
  are reported by `query` and `rdeps` at **one hop**, which is the distance at which they are true.
  `crates/delulu-survey/tests/traversal.rs` holds that line, because a saturated answer looks
  exactly like a thorough one.

- **It does not answer "why does this exist" or "who owns this" as separate verbs, and that was
  measured too.** A `why` verb would print a *filtered subset* of what `query` already returns —
  the incoming `cites`, `links-to` and `documents` edges from rulings, findings and specs are
  already there. An `owners` verb would read `.github/CODEOWNERS`, where every rule currently names
  the same deliberate placeholder (`@PENDING-PUBLIC-project-lead`, unassigned until public launch),
  so it would be a constant function. Neither was built.

  **What CODEOWNERS carries that a verb could not is now an attribute — see below.** That was
  written here as "not built" for one release, on the reasoning that rejected the verb. The
  reasoning was right about the verb and wrong to travel: *who reviews this* and *is this
  entrenched* are different claims, and only the first one is a placeholder.

- **It does not judge.** `DISCREPANCIES.md` reports that two things disagree; which one is wrong is
  a person's call.
- **It does not touch the authority model.** The Survey is a reader. It has no opinion about what a
  grant means, and changing what it maps never changes what the language does.
