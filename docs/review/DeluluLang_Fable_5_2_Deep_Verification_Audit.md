# DeluluLang — Fable 5.2 independent deep-verification audit (rolling)

**Status: IN PROGRESS — session 1 of an open number. This is not a final report.** Commissioned by the owner on
14 November 2026 ([the commission](../prompt/DeluluLang_Fable_5_2_Multi_Session_Deep_Verification_Commission.md)).
Research only: no production code, test, workflow or normative document was changed. One document, updated in
place each session; the machine-readable resume state is
[`fable-deep-verification-state.json`](fable-deep-verification-state.json).

| | |
|---|---|
| Source under audit | `ac2a16202e790657810b819e882b8d721dc0aef4` (`origin/master` on 2026-11-14, fetched at session start) |
| Model and effort, as observable | The session harness names the model Claude Fable 5.2 (`claude-fable-5-2`); the owner's `/effort` command set `max` for this session. No fallback notice was shown. Neither can be verified beyond those two statements. |
| Session 1 | 2026-11-14, the owner's laptop (Windows 11 26200, x86-64, 16 logical CPUs, 15.3 GiB), plus WSL2 Ubuntu 20.04 on the same machine |
| A limit of this session | One response was stopped by a platform safety check while I was planning new adversarial artifact inputs. From there I built **no new hostile artifacts**: track T1 is source analysis, the repository's own tests and fix design. What that leaves unverified is listed in §12. |
| Evidence words | **executed** = I ran it this session and kept the raw output; **source** = I read the code at the pinned SHA; **CI** = I read the named run's job results; **other-party** = Codex's or an earlier audit's result, not re-run by me; **not run** = exactly that. |

---

## 1. Executive verdict (provisional after one session)

**Do I agree with Codex's recovery plan?** Mostly, with one correction of scope, one of order, and one of emphasis.

- **CLI-first: agree.** A 352-line program was written, checked, authorised and run entirely through the CLI on the
  interpreter, the WASM engine and the OS sandbox, on Windows and in WSL, with byte-identical results (§5). Nothing
  observed argues for a separate compiler or editor product. Not yet measured: the same task against a sandboxed
  Python or TypeScript baseline.
- **The five blockers: all five reproduce** at the pinned SHA with my own fixtures or by source (§4). Three are
  wider than the plan says:
  1. *Outcome truth.* Exit code 2 is documented as "your invocation was wrong; the program was never examined".
     It is also what **every internal panic** returns (`crates/delulu/src/main.rs` line 93), what a run returns
     **after its effects** when the report file cannot be written (also for a merely mistyped directory), and what a
     closed output pipe returns. And a `main` that returns `Err(..)` exits 0.
  2. *Verified artifacts.* The plan's repair (unique node ids, valid identifiers) leaves out two more things the
     artifact path accepts that the text path cannot produce (§4 T1).
  3. *`.solulu`.* `fmt` refuses the extension, and a package whose entry is `src/main.solulu` **checks clean** with
     the entry silently unread.
- **One blocker is narrower than it reads.** "Ordinary input and data remain missing" is true of the standard
  library (no arguments, no environment, no JSON, no way to build a character from its number) and **not** of the
  language: a hand-written JSON parser and transformer passed 17 of 18 pre-registered oracle cases on all three
  execution modes after one repair turn (§5).
- **Severity of the artifact finding (RW 4.93): MEDIUM stands for what has been demonstrated** (a
  reference-capability rule skipped; no authority escape shown, by Codex or by me). By source, the same
  artifact-supplied ids also key the loader's per-site table behind RW 4.95's fix, so that fix's guarantee for code
  inside a plugin is conditional on the same repair. Untested; do not upgrade or dismiss it on this paragraph.
- **Order: I would change it** (§10). The very next code change should still be the artifact boundary, but in its
  **minimal fail-closed form** (small, one file, no wire-format change, no compatibility decision). Then the
  false-success exits (`test`, `fix`, `schema` — tiny, and they poison every agent's inner loop), then the outcome
  contract with the wider scope above, then arguments and JSON, and only then `.solulu`: loose `.solulu` files
  already run, and no user is blocked by the package half today, while false greens and missing input block every
  agent task.
- **Preview criteria: agree**, with one addition: a package must refuse, not skip, a source file it does not
  recognise.

**Where Codex was too cautious:** language adequacy (above). **Too optimistic:** that a uniqueness check closes the
artifact boundary; that exit 2 means nothing ran. **Mistaken in a detail:** its recorded SHA-256 for the built
binary cannot be matched to the file on disk (debug builds here are not bit-reproducible; three builds of the same
commit gave three hashes, §2).

---

## 2. Ground-truth snapshot

| Fact | Observation (2026-11-14, IST) | Evidence |
|---|---|---|
| Remote | `origin` = `github.com/jessesuniljs1-collab/delululang-test` (fetch and push); nothing else configured | executed |
| Heads | `HEAD` = `origin/master` = `ac2a162`; local branch `codex/research-recovery` (never pushed, no commits of its own) carried Codex's uncommitted work: one modified file and four untracked documents. Left exactly as found (their SHA-256 values re-checked after my one mistake, §12). | executed |
| My branch | `laptop/fable-deep-verification`, in its own worktree, so the owner's checkout was never switched | executed |
| Other refs | `origin/claude/relaxed-brown-2l78i1` at `b601210` (code at `d5e647d`), 2 commits ahead of `master`; `origin/autosave/laptop` at `4271780` (2026-11-08, not an ancestor of `master`, older than `master`'s last laptop commit); two old stashes; no open pull requests | executed |
| CI on the pinned SHA | Scheduled run `43409171490`: **18 of 18 jobs success**. Push run `43302145758`: 14 success, 2 skipped by design (`heavy-gates`, `miri-slow`). | CI, read with `gh run view --json jobs` |
| CI on the unfinished branch | Run `43389262361` at `d5e647d`: 14 success, 4 failure (`arm64`, `test` on macOS, Windows, Ubuntu). In each, the same two tests fail and nothing else: `release_requires_full_coverage` and `the_generated_reference_is_not_stale` (26 passed, 2 failed). No run exists for the branch tip `b601210`. | CI; failed logs downloaded and searched (A7) |
| Toolchain | Windows: `rustc`/`cargo` 1.96.1, pinned by `rust-toolchain.toml`; `gh` 2.100.0. WSL: the same 1.96.1, gcc 9.4, glibc 2.31, kernel 6.6.87.2 | executed |
| Pinned binary (Windows) | Built from a clean worktree with `DELULU_BUILD_COMMIT` set: `cargo build -p delulu --locked -j 4`, debug, default features. 72,949,248 bytes, SHA-256 `b0a3050c422d9f36…84b4`. `--version --json` reports the commit and `x86_64-pc-windows-msvc`. Every Windows result below used this file. | executed (A3) |
| Pinned binary (WSL) | Clean Linux-native clone at the same SHA: ELF x86-64, 460,373,712 bytes (debug), SHA-256 `6fa1b995e2a0a3df…1100`; reports the commit and `x86_64-unknown-linux-gnu` | executed (F1) |
| Binary on `PATH` | `~/.cargo/bin/delulu.exe`, dated 2026-08-07, SHA-256 `85067fb1…6bce`; prints `delulu 1.0.0` with no commit. Not used for any result. | executed |
| Codex's binary hash | The plan records `cfa46889…`; the file at that path now hashes `0c3ccb08…` (same size, same commit stamp). Not confirmable. | executed |
| Survey | Clean worktree: `ok: the Survey matches the tree (1727 nodes, 20757 edges)`, exit 0 — on Windows and in WSL. The owner's checkout (untracked documents present): three generated files reported stale, exit 1. So staleness there is a statement about the working tree, not about committed source. | executed (A1, A2, F1) |
| Doctor | Clean worktree: exit 0, `28 check(s) passed` (Windows), `29` (WSL). The owner's checkout: exit 1, one problem, the map's freshness. Doctor prints the version but not the commit, and compares no other executable. | executed (A4, A5, F1) |
| State isolation | Every run used a scratch `DELULU_HOME` and `DELULU_STATE_DIR` and `DELULU_NO_FIRST_RUN=1`; fixtures, keys and logs are outside the repository | executed |
| Process identity | All Windows runs as the owner's one OS account (not elevated). Sandboxed guests ran as a per-run AppContainer (Windows) and a subordinate uid in a user namespace (WSL), as the tool itself reports. No second OS account was used. | executed |

---

## 3. Coverage accounting

**Counting method.** `git ls-files` at the pinned SHA; physical lines, blanks and comments included. Inside each
crate's `src/`, everything from the first column-0 `#[cfg(test)]` to the end of the file is counted as unit tests.
Three test-only module files that the rule misses (2,492 lines) are subtracted by hand. The script is 120 lines of
Python kept with the private evidence (SHA-256 `ccccc1ca…`); its output is A6.

**Totals.** 1,268 tracked files. Rust: 349 files, 209,666 lines — of which **about 114,400 are production code**,
35,000 are unit tests inside `src/`, 57,680 are integration tests and the rest is test-only modules and 56 lines
outside `crates/`. So the Rust in this repository is roughly 55% production and 45% tests.

| Crate | Production lines | Unit-test lines in `src/` | Integration-test lines | `unsafe` lines (production) | Read by me this session | Executed by me this session |
|---|---:|---:|---:|---:|---|---|
| `delulu` (CLI, broker daemon, sandbox, LSP, MCP) | 39,935 | 9,958 | 45,715 | 160 | `main.rs` 56–95; `cli.rs` 6436–6462 — about 70 lines | black-box: `new`, `check`, `run` (three modes), `test`, `fix`, `fmt`, `build`, `authority`, `schema`, `doctor`; tests `plugin_cli`, `plugin_load_cli`, `signing_cli` (50 passed) |
| `delulu-runtime` | 17,846 | 6,562 | 1,900 | 37 | `plugin.rs` 380–709; `interp.rs` 1008–1057 and 1925–2224 — about 680 lines | through every `run` above |
| `delulu-check` | 13,315 | 4,930 | 1,004 | 0 | `dir.rs` 1–545 (all of its production code); `rcap_check.rs` 1225–1334; fragments of `check.rs` (about 120 lines); `prim_table.rs` 1–60 and the whole table by script; `fuzz.rs` 20–45 — about 860 lines | `cargo test --locked -p delulu-check`: 329 passed |
| `delulu-wasm` | 12,799 | 2,410 | 6,110 | 0 | none | through `run --engine wasm` and `build --target wasm` |
| `delulu-broker` | 8,832 | 5,785 | 1,306 | 0 | none | only indirectly (embedded custody in every run) |
| `delulu-syntax` | 6,525 | 3,019 | 0 | 0 | `ast.rs` whole (697); parser rules located by search | through every `check` |
| `delulu-survey` | 4,125 | 602 | 1,060 | 1 | none | `check` clean and dirty, on two systems |
| `delulu-diag` | 3,248 | 827 | 99 | 1 | none | indirectly |
| `delulu-measure` | 2,730 | 259 | 156 | 0 | none | not run |
| `delulu-conform` | 1,665 | 54 (+457 test module) | 0 | 1 | none | not run (CI only) |
| `delulu-atlas` | 1,860 | 440 | 0 | 0 | none | not run |
| `delulu-registry` | 1,061 | 3 (+519 test module) | 330 | 0 | none | not run |
| `delulu-fuzz` | 671 | 130 | 0 | 0 | none | not run (CI only) |

**Honest summary.** About 2,300 production lines read closely this session, roughly **2% of production Rust**; all
of it on the artifact verification path and the process entry point. Everything else is known to me only through
black-box runs, the repository's tests, or CI.

**Where the `unsafe` is.** Of the 200 production lines containing the word `unsafe`, 175 sit in nine OS-boundary modules:
`broker_transport.rs` 35, `identity.rs` 34, `jail.rs` 26, `beneath.rs` 24, `microvm.rs` 20, `ceiling.rs` 15,
`guest.rs` 8, `foreign.rs` 7, `foreign_worker.rs` 6; the other 25 are spread over eleven files (some are the word in a string, not code). The checker, parser, broker
lattice, WASM code generator and WASM host contain none. A lead for where a memory-safety review belongs, not a
finding.

**Explicitly unexamined production paths** (none read; highest risk first): `delulu/src/brokerd.rs` and
`broker_transport.rs` (custody over IPC); `guest.rs`, `jail.rs`, `identity.rs`, `sandbox.rs`, `launcher.rs`,
`microvm.rs` (the OS boundary); `delulu-runtime/src/beneath.rs`, `egress.rs`, `prim.rs`, `custody.rs`,
`foreign.rs`, `device.rs`, `actors.rs`; all of `delulu-broker` (`guard.rs`, `tree.rs`, `lease`, `audit.rs`,
`validate.rs`); `delulu-wasm/src/codegen.rs`, `host.rs`, `dpx.rs`; `delulu/src/run_cmd.rs`, `mcp.rs`, `lsp.rs`,
`edit.rs`, `doctor.rs`, `schema.rs`; `delulu-check/src/package.rs`, `deps.rs`, `lockfile.rs`, `authority.rs`,
`resolve.rs`, `unify.rs`, and most of `check.rs` and `rcap_check.rs`; `delulu-registry`; `delulu-atlas`;
`editors/vscode`; `scripts/`; the release workflows.

---

## 4. Challenge table — Codex's hypotheses, T1 to T8

Status words: CONFIRMED, FALSIFIED, NARROWED, REGRESSED, UNVERIFIED for claims; PASS, FAIL, PARTIAL, UNSUPPORTED,
NOT RUN, BLOCKED for runs. Raw outputs are indexed in Appendix B.

### T1 — Verified artifacts (RW 4.93, 4.94, 5.7, 6.67, 6.71) — CONFIRMED by source; run-time reach UNVERIFIED

**What I did.** Traced bytes to effect in source; ran the repository's own tests; built nothing hostile.

**The path, as built.** A Verified plugin ships a serialised syntax tree with stored facts.
`delulu_check::dir::verify_checked` (`dir.rs` 237–274) decodes it, runs a structural walk (`validate_module`,
282–536), re-runs `resolve` and `check_module` on the decoded tree, requires no error, and requires the stored
facts to equal the recomputed ones. Both `delulu plugin verify` and a program's `load` reach it through one
function, `step5_verified` (`delulu-runtime/src/plugin.rs` 518–560, called at 895 and 951) — so *verify and load
cannot diverge*, as the code claims. That claim is true and worth keeping.

**What the walk checks:** non-empty name paths; no actors, tests, written reference capabilities or actor
expressions. **Nothing else.** The module comment says deserialised ids are "only ever compared, never used to
index a table". That is true of `dir.rs` and false of what it calls.

**Rules a tree from the parser always satisfies and a decoded tree need not** (each read in source):

| # | Parser-side guarantee | Who relies on it afterwards | Re-established for an artifact? |
|---|---|---|---|
| 1 | Every expression and block has its own `NodeId` | `rcap_check.rs` 1239–1255: the write rule for a `val` list or map applies only when the type looked up by the receiver's id is a list or map; a miss or another type skips it silently. The same lookup at 514, 597, 900, 1109, 1135, 1293, 1315, 1368, 1541. The loader's per-site tables `plugin_gets` (RW 4.95's rule R-Get), `plugin_loads` (RW 4.96) and `foreign_binds` are maps keyed by id, built in `check.rs` 301–311, handed to the interpreter at `plugin.rs` 559 and `interp.rs` 2196, read at `interp.rs` 1883, 1956, 2129 | **No** |
| 2 | `Module.recovered` is empty in any module that parsed (it lists the parser's error placeholders) | `check.rs` 1843 gives a listed literal a fresh unknown type and no diagnostic, "its parse error already said" | **No** — and on this path no parse error ever said |
| 3 | Identifiers passed the lexer (DL0101–DL0108: characters, bidirectional controls, reserved words DL0106 and DL1608) | every surface that prints a name; any future print-and-reparse | **No** (RW 4.94) |
| 4 | Nesting no deeper than 128 (DL0210–DL0213) | every recursive walk | Only by accident: `ciborium` 0.2.2's default recursion limit of 256 (`de/mod.rs` 847) costs, by my reading of the encoding (not measured), at least two levels per tree level |
| 5 | A compound assignment's value is `target op rhs` and its target holds no call (`parser.rs` near 1636) | `interp.rs` 1019–1024 evaluates the place first when `compound` is set | **No** |
| 6 | A variable reference is one segment; a binding pattern is lower-case; attributes are known (DL1901); a foreign ABI is `"c"` (DL1308) | `check_var`, pattern resolution, `cli.rs` 4454, review output | **No** |

**Other consumers.** `delulu why <effect> <file.dpx>` decodes without verifying (`cli.rs` 6450,
`dir_deserialize`) and reports from stored facts — RW 6.67 CONFIRMED by source. The fuzz target `fuzz_one_dir`
(`fuzz.rs` 34–38) decodes and re-encodes and never calls `verify` — RW 5.7 CONFIRMED by source.

**Existing controls, executed.** `cargo test --locked -p delulu-check`: 329 passed, 0 failed, including the 16
`dir::tests` (round trip, version refusals, malformed bytes, every single-byte flip, forged facts, forged export
type). `plugin_cli`, `plugin_load_cli`, `signing_cli`: 50 passed. **None of them changes an id, sets `recovered`
or alters an identifier consistently** — single-byte flips cannot produce a self-consistent tree — so the suite is
green and silent on rows 1–3.

**Codex's witness** (other-party, same SHA): the audit's Appendix A.5 trees with every id set to one value verify
while their text is DL1604; a wrapped artifact gets `verdict: ok` from `plugin verify`; no hostile export was
executed. My source trace explains the mechanism (row 1) and agrees. I did not re-run it.

**Is "enforce unique node ids before replay" sufficient?** No, for three reasons.
1. Rows 2, 3, 5 and 6 are independent of it.
2. It must sit **inside `Dir::decode`**, the one place bytes become a `Module`, so that the unverified reader gets
   it too; a check added only in `verify_checked` leaves `why` as it is.
3. It must be computed **from the tree alone**, before `resolve`, never by comparing two tables built under the
   same ids.

**The irreducible invariant** is one sentence: *the decoded module is a tree the parser could have produced from
some source text.* Two ways to hold it. (a) By enumeration — a guard per row above; small, immediate, and it will
drift the next time the parser gains a rule. (b) By construction — print the decoded tree with the formatter,
parse the text with the real parser, and accept only if the two trees are equal apart from ids and spans; then
every present and future lexer and parser rule applies to artifacts without a second implementation. (a) is the
next patch (§11). (b) is the test that keeps (a) honest and, once measured, its replacement.

**A second, independent tripwire** worth adding with (a): after the replay, the number of entries in the
recomputed type table must equal the number of expression and block nodes counted by walking the tree. Any id
collision makes the table smaller. The repository already asserts this for honest input
(`node_tables_cover_every_expression_and_block`).

**Severity, kept honest.** Demonstrated (other-party): one reference-capability rule skipped on a path an operator
must opt into (`--grant plugin=<dir>`, then `load`). Not demonstrated by anyone: an effect the grants do not
cover, a sandbox escape, a cross-principal breach. Open by source and untested: whether the per-site table in row 1
weakens R-Get for a plugin that is handed another plugin. Classification: **confirmed defect in a verifier
(assurance and soundness), reachable only through an opt-in feature; MEDIUM; release-blocking for any claim that
third-party Verified plugins are safe to load.**

### T2 — Truthful outcomes (RW 6.77, 6.83, 6.58, 6.55, 6.51, 6.79, 6.54) — CONFIRMED and wider

Fixture: a program that reads `./input/task.txt`, writes `./output/result.txt` (the effect sentinel) and prints one
line. All on Windows with the pinned binary.

| Case | Command shape | Observed | Verdict |
|---|---|---|---|
| a | valid run, `--report-out` to a new file outside the write grant | exit 0; report has `outcome {exit: 0, ran: true}`; sentinel written | positive control PASS |
| b | `--report-out` names an existing directory | sentinel **written**; then stdout gains an envelope `error {kind: "usage", exit: 2, message: "the command could not run…"}`; exit **2** | CONFIRMED (6.77) |
| d | `--report-out` in a directory that does not exist | identical to b: effect committed, then "usage", exit 2 | **new variant** — an ordinary typo is enough |
| c | `--report-out` inside the granted write scope | refused before anything ran ("nothing ran"); sentinel absent; exit 2 | negative control PASS — a true pre-run refusal looks exactly like b and d |
| e1 | checker refusal (DL0501), in process, with `--report-out` | exit 1; report present with `ran: false` but an **empty** `diagnostics` | PARTIAL — the report omits the reason |
| e2 | the same with `--sandbox` | exit 1; **no report file** | CONFIRMED (6.83) |
| f1, f2 | grant missing, in process and sandboxed | exit 1; DL0703; report present with the diagnostic; sentinel absent | PASS |
| g | valid sandboxed run | exit 0; sentinel; report | positive control PASS |
| m | endless loop under `--limits wall=1` | exit 1 after 1.07 s; report `stopped_by {dimension: "wall", budget_seconds: 1}` | PASS — a stop is reported as a stop |
| l2, l3 | 50,000 lines to a reader that closes after one, both engines | **panic** ("failed printing to stdout … os error 232"); exit **2**; **no report** | CONFIRMED (6.58), and see below |
| h | `fix --dry-run --json` on a file with one error | exit **0** while `summary.errors` is 1 and `errors_after` is 1 | CONFIRMED (6.55): exit code and envelope disagree |
| i1, i2 | `test <path that does not exist> --json` | exit **0**; `passed: 0, failed: 0, tests: []` | CONFIRMED (6.51) |
| j | `schema validate envelope` on `{}` | exit 1; `validate.valid: false`; `summary.errors` **0** | CONFIRMED (6.79): the other way round from h |
| k1, k2 | `authority --isolation bogus`, `--broker bogus` | exit 0, no complaint | CONFIRMED |
| k3, k4 | `check` and `run` with an unknown option | exit 2, "refused, never ignored" | negative control PASS |
| n | `fn main(root: Root) -> Result[Unit, Str]` returning `Err("…")` | checks clean; exit **0**; report `{exit: 0, ran: true}` | **new** — a program cannot signal failure (ties to RW 2.40) |

**The wider finding.** `main.rs` 89–94 maps every panic of the CLI thread to exit 2, on purpose, so that a test
sweep can see crashes. `docs/for-agents.md` (the exit-code table) defines 2 as a usage error in which "the program
was never examined". Both cannot be true. Today **exit 2 does not imply that nothing ran**, and cases b, d, l2 and
l3 are instances, not the whole class: any internal fault after the first effect has the same shape, with no
envelope and no report. A caller that follows the documented contract will correct its command line and run the
effect again.

**Is Codex's M2 too small?** Yes. It names the report sink and the sandbox prestart. It needs, in the same
contract: a distinct code and envelope for an internal fault; the report sink opened and held before the program
starts; a closed standard output handled as a stop, not a panic; and an application failure channel. It does not
need, and must not promise, a report after the process is killed.

### T3 — Ordinary input, data and complete programs — CONFIRMED for the library, NARROWED for the language

| Probe | Observed | Verdict |
|---|---|---|
| `root.args()` | DL0405, `Root` has no method `args` | CONFIRMED absent |
| `root.env("HOME")` | DL0405 | absent (new probe) |
| `parse_json(..)` | DL0301 unknown name | CONFIRMED absent |
| `root.console().readline()` | works on the interpreter (returns a `Result`); on WASM a typed DL1201 before anything runs | standard input **is** an input channel, interpreter only |
| The whole primitive table | 137 entries read by script: no arguments, environment, JSON, sleep, exit status, or character-code conversion | source |
| Higher-order call of a user function, `apply(inc, 41)` | interpreter 42; WASM DL1201 at compile, exit 1, nothing ran | CONFIRMED (RW 3.25); the refusal is typed and early |
| Lambdas with `map`/`fold`/`filter`, generics, recursion, records, ranges, compound assignment, bitwise, f-strings, `sort` in one program | identical output on both engines | PASS |
| Tuple literal | DL0201 syntax error | unsupported (RW 2.39), as recorded |
| The complete-program task | §5 | 17 of 18 on three modes |

### T4 — `.solulu` (with RW 6.52) — CONFIRMED, with two additions

| Case | Observed |
|---|---|
| Loose `hello.solulu`: `check`, `run`, `run --engine wasm`, `authority` | all succeed |
| `fmt --check hello.solulu` | exit 2: "not a `.delulu` source file, so `fmt` has nothing to do with it" (**new**) |
| Package with only `src/main.solulu`: `check .` / `run .` / `test .` | exit 1 "no `.delulu` modules found" / exit 2 / **exit 0 "0 passed, 0 failed"** |
| `main.delulu` importing `helper.solulu` | DL0303 unknown module |
| `main.solulu` with `helper.delulu` | `check .` → **exit 0, "checked clean (1 package(s), 1 module(s))"**: the entry file was never read (**new**); `run .` → exit 2 "declares no entry module" |
| Control, both `.delulu`: `check .`, `run .` on both engines, `build --target wasm` | pass; prints 42 |
| Control, explicit entry file inside that package: `run src/main.delulu`, `check`, `authority` | DL0301 — the sibling module is not seen (RW 6.52 CONFIRMED) |

A harness error of mine, corrected before these results: my first control called `helper.answer()`; an import
brings public names in unqualified, so the call is `answer()`.

### T5 — Independent principals — NOT RUN

No second credential or OS account was used. What exists on this machine: two other enabled local Windows accounts
whose credentials I do not hold, and WSL, where separate Linux users can be created. Plan in §12.

### T6 — Install, upgrade, WSL — PARTIAL

| Item | Result |
|---|---|
| WSL-native build at the pinned SHA from a clean clone under `/home/user` | **PASS**: `cargo build -p delulu --locked -j 3`, exit 0, 1 m 43 s with a warm registry cache (not a first download) |
| The same build from a Windows-mounted path (`/mnt/d/...`) | FAIL at `libffi-sys`'s configure step ("config.log: No such file or directory") — observed by accident (§12); it is why the build must be Linux-native |
| WSL: Doctor, Survey, the §5 task on three modes, a denied grant | exit 0 with 29 checks; map matches; PASS ×3 (the sandbox is Landlock plus a user namespace); DL0703 and no file |
| WSL file semantics | `DOC.JSON` is not `doc.json` (case-sensitive, as Linux is); a symbolic link inside the read scope pointing outside it is refused with DL0904 and the run stops |
| The WSL binary's dependencies | default features link `libpython3.8.so.1.0` dynamically — a default build is tied to the host's Python |
| A released v1.0.0 **archive** | UNSUPPORTED: the test repository has a `v1.0.0` tag and **no releases or assets** (`gh release list` is empty). An archive upgrade cannot be performed from here. |
| Upgrade route | NOT RUN, and none is documented that I have found yet |
| First-time setup from the documents alone | PARTIAL: `delulu new demo` produced a package that checked, tested and ran at once, and its closing text explains the grant. Install from `INSTALL.md` by a fresh reader: NOT RUN |
| macOS, ARM64, Windows and Linux runners | CI only (run `43409171490`) |

### T7 — Survey, Doctor, machine interfaces — PARTIAL

Survey and Doctor clean-versus-dirty controls: §2. Front doors exercised: the T2 and T4 tables. Two observations
for later sessions: `delulu test` took about 1.06 s for zero tests and for one test alike, which suggests a fixed
floor worth measuring; `--help` output is very long. MCP, LSP, Atlas and `edit`: NOT RUN.

### T8 — Simulated devices — NOT RUN

---

## 5. Complete-program scorecard

**The task, fixed before any DeluluLang code for it ran** (the oracle and its 18 expected outputs were written and
hashed first; SHA-256 `36d8b22d…`): read `input/doc.json`; parse JSON; for `items` (objects with `name`, `qty`,
`tags`) write `{"count","total_qty","names" (upper-cased),"tag_counts" (sorted)}` to stdout and to
`output/out.json`; on failure print `{"error":"input"|"parse"|"schema"}` and write no file. The oracle is Python's
`json` module with a 64-bit integer bound.

**The program**: 352 lines of DeluluLang — a recursive-descent parser over a recursive sum type, `Result` and `?`
throughout, a `Map` for counting, hand-written escaping. **One repair turn**: `tag` is a reserved word (DL0106,
with a clear message).

| Measure | Interpreter | WASM engine | OS sandbox (L1) |
|---|---|---|---|
| Fixtures passed (output, file state and exit status all as the oracle says) | 17 / 18 | 17 / 18 | 17 / 18 |
| The one failure | `\uXXXX` escapes: the language cannot build a character from its number, so the program refuses them | same | same |
| Agreement between modes | byte-identical output on all 18 | | |
| Largest input (2,000 items, about 100 kB), debug build | 1.31 s | 0.75 s | 1.34 s |
| Run without the write grant | DL0703, exit 1, no file | | |
| `authority --within` the three grants used | `within`, 5 requests in 3 dimensions | | |
| Same program in WSL (first fixture) | PASS | PASS | PASS |

Cases covered: escapes, raw non-ASCII text, trailing comma, missing key, wrong type, 40-deep nesting in an ignored
key, 64-bit overflow, empty list, duplicate key, invalid UTF-8 input, whitespace and negatives, a float where an
integer is required, a non-object document, trailing garbage, a leading zero, and the large input.

**What this shows and does not.** It shows a useful, non-trivial data program can be written by an agent today,
runs identically on every execution mode, and is held to its grants. It does not show an agent can do it *cheaply*:
the parser is 200 lines a standard library should own, it cannot be complete (`\u`), and it is slow. Two
independent agents: NOT RUN.

---

## 6. Platform matrix (observed, by name)

| Platform | Evidence at `ac2a162` | Class |
|---|---|---|
| Windows 11 x64, native | my build; all of §4 and §5; L1 sandbox as an AppContainer | executed |
| WSL2 Ubuntu 20.04 x64 | my build from a Linux-native clone; Doctor, Survey, the task on three modes, Landlock sandbox, case and link checks | executed |
| Ubuntu 24.04 x64, macOS, Windows runners | `test (…)` jobs of run `43409171490` | CI |
| Linux ARM64 | `arm64` job of the same run | CI |
| MicroVM (L2) | `microvm`, `microvm-reproducible` jobs; locally absent on Windows, and in WSL only for want of the monitor on my `PATH` | CI |
| Release archive install or upgrade, any platform | none exists to install | UNSUPPORTED |

---

## 7. Security and supply-chain findings so far

| Id | Finding | Class | Severity as demonstrated | Evidence |
|---|---|---|---|---|
| FABLE-006 | Artifact verification does not re-establish what the parser guarantees (ids, `recovered`, identifiers, shapes) | confirmed defect in a verifier | MEDIUM; opt-in path | source; other-party witness |
| FABLE-007 | `Module.recovered` is honoured on the artifact path | plausible issue with a reachable path | not assessed | source only |
| FABLE-008 | The R-Get per-site table is keyed by artifact-supplied ids | plausible issue with a reachable path | not assessed | source only |
| FABLE-009 | `why` reports from unverified stored facts | confirmed (review-surface integrity) | LOW–MEDIUM | source |
| FABLE-010 | The artifact fuzz target never reaches verification | assurance gap | LOW | source |
| FABLE-013 | Exit 2 covers both "nothing ran" and "an internal fault after effects" | confirmed (outcome integrity; duplicate effects on retry) | MEDIUM for agent harnesses | executed; source |
| FABLE-016 | A package check passes while skipping an unrecognised source file | confirmed (a gate that counts only what it reads) | LOW today | executed |

Not assessed this session: the dependency and licence graph, advisories against the locked versions, CI action
pins and permissions, release keys and provenance, the microVM image's obligations. `supply-chain` was green in CI
run `43409171490`; that is CI evidence only.

**Threat-model notes established so far.** A Verified artifact is untrusted bytes until `step5_verified` returns.
Its signature is optional and is not evidence of meaning. Loading requires the operator's `--grant plugin=<dir>`
and the program's `load`; a package may pin artifacts by hash. An export runs in its own interpreter under the
host's custody and is bounded by the capabilities the host passes it. One OS account runs the CLI, the embedded
broker and (in LEGACY mode, the default) can mint root authority — Doctor says so itself.

---

## 8. External research — NOT STARTED this session

Codex's table of standards and runtimes (A2A, MCP, ROS 2/DDS, ETSI, UNECE, OpenShell, Wasmtime) stands as
other-party and unverified by me. Nothing in §10 depends on it.

---

## 9. Keep / repair / consolidate / defer — entries with evidence so far

| Item | Verdict | Evidence | Condition to revisit |
|---|---|---|---|
| Text source, one checker, two engines, the OS sandbox | **Keep** | one program, three modes, two systems, identical results (§5) | — |
| Serialised syntax trees as the Verified artifact format | **Repair now; reconsider the format** | §4 T1: every parser rule needs a twin at decode. Shipping canonical source text and re-parsing it would make the invariant true by construction; the earlier pipeline audit measured the tree at 28 times the size of text | after the minimal repair, measure verify time and size for text-in-artifact against the tree |
| Exit-code contract | **Repair** | §4 T2 | needs a recorded decision: it changes a published contract |
| `delulu why` on artifacts | **Repair** (verify first) | `cli.rs` 6450 | — |
| Standard library: JSON, arguments, character codes, exit status | **Build** | §4 T3, §5 | — |
| `.solulu` | **Defer behind the above**, then do as Codex's P1.2 with `fmt` and the skip-refusal added | §4 T4 | when outcome and input slices have landed |
| Survey, Doctor | **Keep**; make Doctor name the commit and the selected executable | §2 | — |
| Everything else in Codex's component table | not yet examined by me | — | later sessions |

---

## 10. Revised execution queue — five code slices

Codex's order was P0.1, P1.1, P1.2, then P1.3 to P1.6. Mine, with the decisive evidence for each move. Existing RW
numbers keep their identity. Sections of the recovery plan that would need the matching edit: *Decision*
(the "next three code milestones" sentence), *Single execution queue* P0.1 (scope), P1.1 (scope) and the order of
P1.2 against P1.3 and P1.4.

| # | Slice | RW | Why here | Size | Independent negative oracle |
|---|---|---|---|---|---|
| S1 | **Artifact ingress, minimal and fail-closed.** In `Dir::decode`: refuse duplicate node ids, a non-empty `recovered`, and identifiers the lexer would refuse; after replay, require the type table's size to equal the node count. `why` verifies before it reports. No wire-format change. | 4.93, 4.94, 6.67 | A soundness hole in the feature named "Verified"; one file; no compatibility decision; RW 4.95's guarantee inside a plugin depends on it | S | Honest artifacts from the existing corpus still verify byte for byte; each guard removed in turn makes exactly its own test fail |
| S2 | **False-success exits.** `test` with a path that matches nothing refuses (exit 2); `fix` exits 1 while errors remain; `schema validate` counts its failure in `summary.errors`; `authority` refuses option values it does not know; a package refuses a source-like file it does not read | 6.51, 6.55, 6.79, 6.54 | An agent's inner loop is `check`/`test`/`fix`; a false green there defeats every later gate. Tiny. | S | The existing passing cases keep their exit codes; `test .` on a package with no tests stays 0 |
| S3 | **Outcome contract, widened.** The report sink is opened before the program starts; failure to persist after a run is `failed` with `ran: true`, never `usage`; an internal fault gets its own exit code and envelope; a closed standard output is a stop; a sandboxed pre-start refusal writes the report; `main` returning `Err` exits non-zero | 6.77, 6.83, 6.58, 6.84, 2.40 | T2: exit 2 does not mean nothing ran | M | Case c (refused before start) still exits 2 with no effect; case a unchanged; the sentinel is written exactly once in every failing case |
| S4 | **Input and data.** `root.args()`; a standard `parse_json`/`to_json`; a way to build a character from its number; on both engines or with a typed early refusal | 2.40, 2.41 | §5: the language can already do the work; the library makes it cheap and complete | M–L | The §5 oracle: all 18 fixtures, three modes |
| S5 | **`.solulu` in packages**, as Codex's P1.2, plus `fmt` | 6.52 | Loose files already work; nobody is blocked today | M | Both extensions, both directions; a duplicate module name across extensions is refused |

After these: Codex's P1.5 (identity), P1.6 (loss and stop), P2.1 (recover RW 2.50 from its branch — its four red
jobs are two stale generated-file tests, §2, so nothing in S1 to S3 is blocked by it), then the rest as planned.

**Preview gates.** I agree with Codex's nine, add "a package refuses what it does not read", and note that gate
(i) cannot be called met by S1 alone: it needs the by-construction test of §4 T1 or the format change of §9.

---

## 11. Handoff for the next coding agent — slice S1

*For a session the owner has separately authorised to write code. This research commission authorises none.*

```
git fetch origin && git switch -c <your-name>/dir-ingress origin/master   # confirm HEAD is still ac2a162 or re-read this audit against the new SHA
cargo run -p delulu-survey -- check && cargo run -p delulu -- doctor --check
cargo run -p delulu-survey -- impact mod:crates/delulu-check/src/dir.rs
cargo test --locked -p delulu-check dir::tests -j 4                     # 16 passed today: the positive controls
```

1. **Red first.** In `crates/delulu-check/src/dir.rs`'s test module add four tests, each starting from a module in
   the existing `corpus()`: (i) two expression nodes given one id — the mechanics are already public in the
   pipeline audit's Appendix A.5; (ii) a literal's id listed in `recovered`; (iii) a function name the lexer
   refuses (the DL0107 case of RW 4.94); (iv) the honest module, untouched. Rebuild the stored tables with
   `Dir::from_checked` after re-running `resolve` and `check_module`, as A.5 does, so each tree is
   self-consistent. Assert (i) to (iii) are `Err(DirError::Malformed(_))` and (iv) is `Ok`. Today (i) to (iii)
   fail — that is the witness. Keep the fixtures as code that builds them; commit no artifact files.
2. **Fix, in `Dir::decode` only.** One walk that collects every `id` field of the tree into a set and refuses a
   repeat; refuse a non-empty `recovered`; apply the lexer's own identifier predicate (expose it from
   `delulu-syntax` rather than writing a second one) to every `Ident`. In `verify_checked`, after the replay,
   compare the type table's size with the walk's count of expression and block nodes.
3. **`why`.** In `crates/delulu/src/cli.rs` (the function containing line 6450) replace `dir_deserialize` with the
   verifying path and refuse with the loader's own diagnostic; add a `plugin_cli` test that a forged-facts
   artifact is refused by `why` exactly as by `plugin verify`.
4. **Falsify.** Remove each guard in turn (mutants from M1296) and watch only its own test go red.
5. **Property test, same change or the next.** For every module in the corpus and a seeded set of structural
   mutations of its serialised tree: *if `verify` accepts, then formatting the decoded module and checking that
   text raises no error and yields the same function types.* Extend `fuzz_one_dir` to call `verify` under the same
   property.
6. **Gates.** `scripts/clippy.sh -j 4`; `cargo test --locked -p delulu-check -j 4`; `cargo test --locked -p delulu
   --test plugin_cli --test plugin_load_cli --test signing_cli -j 4`; then the Survey and Doctor; push the branch;
   `gh workflow run ci.yml --ref <branch> -f jobs=everything`; read every job.
7. **Records.** RW 4.93, 4.94 and 6.67 rows; a `D-V2-227`-or-later decision; `V2_LOG.md`; `CHANGELOG.md` (an
   artifact that verified yesterday may be refused today only if it was never producible from source).
8. **Review.** This is security-critical. `CONTRIBUTING.md` §4 requires a named sponsor who is not the author for a
   change in a risk class, and counts one model reviewing its own output as one party. Say in the pull request
   who reviewed it, rather than waive the rule.
9. **Rollback.** The change only refuses; reverting it restores today's behaviour exactly. No migration.
10. **Do not** merge, tag, publish, change the wire format, or bundle S2 to S5 into this change.

---

## 12. What I did not verify, and what went wrong

**My mistakes this session.**
- *A misdirected command.* My first WSL build command lost its quoting on the way through `wsl.exe`. It ran in the
  Windows checkout instead of a Linux directory and created, in the owner's checkout root, a nested clone `dl/`
  (with 1.6 GiB of build output) and four `build.*` files. Nothing tracked changed and nothing was pushed. I saved
  the logs (H1), removed exactly those five entries after checking they were mine, and confirmed the checkout's
  status and the four untracked documents' hashes were as at the start. WSL commands now go through script files
  with absolute, guarded paths.
- *A harness error* in the first T4 control (§4 T4), and a WSL script that could not find `rustc` until its `PATH`
  was set. Neither produced a recorded result.

**Not verified** (each is required by the commission; none may be read as done):

| Item | Why | The next runnable step |
|---|---|---|
| T1 at run time: does a refused-as-text artifact reach an export call; does the per-site table matter; what does a `recovered` literal do when executed | I built no hostile artifacts this session (see the header) | Becomes the red tests of §11 step 1, written inside the repository's test suite by the coding session |
| Codex's A.5 result, re-run by me | same | `cargo test` of §11 step 1 (i) before the fix |
| T5, two independent principals | not attempted | In WSL: create two users, give each its own state directory and key, start the broker under a third, delegate, revoke, replay. Write the expected refusals first. |
| A release-archive install and upgrade | no release assets exist on the test repository | `gh workflow run release.yml` as a dry run on a branch produces archives without publishing; install one into a scratch prefix. Needs the owner's word, since it is a release workflow. |
| A cold first-download build | the WSL registry cache was warm | a fresh `CARGO_HOME` |
| A `--no-default-features --features net` build, and a release-profile build | not attempted | one `cargo build` each |
| The full workspace suite locally | not attempted (CI ran it at this SHA) | — |
| MCP, LSP, Atlas, `edit` (T7) | not attempted | start with `delulu mcp` and a recorded handshake |
| Devices (T8), supply chain (§7), external research (§8), the agent task study, governance and licence review | not started | later sessions |
| 98% of production source | §3 | by workstream, highest risk first |

---

## Appendix A. Evidence registry

One line per proposition. Platform for all `executed` entries: Windows 11 x64, pinned binary `b0a3050c…`, debug,
default features, embedded custody, unless WSL is named. All at source `ac2a162`, 2026-11-14.

| Id | Claim | Level | Verdict | Re-check when |
|---|---|---|---|---|
| FABLE-001 | Survey staleness in the owner's checkout comes from untracked files; a clean checkout matches (1727 nodes, 20757 edges) | executed, two systems | CONFIRMED | `delulu-survey` or `docs/survey` changes |
| FABLE-002 | Doctor: exit 0 clean, exit 1 dirty with one problem; it names no commit and compares no other executable | executed | CONFIRMED | `doctor.rs` changes |
| FABLE-003 | CI at the pinned SHA is 18 of 18; the unfinished branch is red on two generated-file tests only | CI | CONFIRMED | any new commit on either |
| FABLE-004 | The `PATH` binary is from 2026-08-07 and carries no commit; a build's hash does not identify a later rebuild | executed | CONFIRMED | reinstall |
| FABLE-005 | Production Rust is about 114,400 lines of 209,666 | executed (script) | CONFIRMED | any Rust change |
| FABLE-006 | The artifact verifier does not re-establish parser guarantees; the `val` write rule is skipped on a type-lookup miss | source; other-party run | CONFIRMED (source) | `dir.rs`, `rcap_check.rs`, `ast.rs`, `parser.rs` |
| FABLE-007 | `recovered` is honoured by the replay and not required empty | source | UNVERIFIED at run time | `check.rs` 1843, `dir.rs` |
| FABLE-008 | Per-site loader tables are keyed by artifact-supplied ids | source | UNVERIFIED at run time | `plugin.rs` 559, `interp.rs` 2129–2196, `check.rs` 301–311 |
| FABLE-009 | `why` on a `.dpx` does not verify | source | CONFIRMED | `cli.rs` |
| FABLE-010 | `fuzz_one_dir` never calls `verify` | source | CONFIRMED | `fuzz.rs` |
| FABLE-011 | 329 checker tests and 50 plugin and signing tests pass; none covers id, `recovered` or identifier mutation | executed; source | CONFIRMED | those test files |
| FABLE-012 | A report sink that fails after the run yields "usage", exit 2, with the effect committed (directory, and missing parent) | executed | CONFIRMED | `run_cmd.rs` |
| FABLE-013 | Every CLI panic exits 2; the documented meaning of 2 is "nothing was examined" | source; executed (closed pipe, both engines) | CONFIRMED | `main.rs`, `docs/for-agents.md` |
| FABLE-014 | A sandboxed checker refusal writes no report; the in-process one writes a report without the diagnostic | executed | CONFIRMED | `run_cmd.rs`, `guest.rs` |
| FABLE-015 | `fix`, `test` and `schema validate` each disagree between exit code and outcome; `authority` accepts unknown option values | executed | CONFIRMED | `cli.rs`, `schema.rs` |
| FABLE-016 | `.solulu`: loose files work except `fmt`; packages do not see it; one mixed package checks clean with its entry unread | executed | CONFIRMED | `package.rs`, `deps.rs`, `cli.rs` |
| FABLE-017 | No arguments, environment or JSON; standard input exists on the interpreter only; `main` returning `Err` exits 0 | executed; source (primitive table) | CONFIRMED | `prim_table.rs`, `check.rs` |
| FABLE-018 | The JSON task passes 17 of 18 oracle cases identically on three modes | executed, independent oracle | CONFIRMED | either engine, `prim.rs` |
| FABLE-019 | The pinned SHA builds and runs natively in WSL2 Ubuntu 20.04, sandbox included; a build from a Windows-mounted path fails | executed (WSL) | CONFIRMED | toolchain, `Cargo.lock` |
| FABLE-020 | The test repository has a `v1.0.0` tag and no release assets | executed | CONFIRMED | a release |

## Appendix B. Evidence index (sanitised)

Raw outputs are on the owner's laptop, outside the repository, under `D:\nelan\fable-audit-evidence` (index files
`INDEX.tsv` and `INDEX.json` there list 195 files with full hashes). They are not in git and are not recoverable
from another machine; a session elsewhere must treat this document's statements as historical and re-run. SHA-256
prefixes:

| Id | What | SHA-256 (first 16) |
|---|---|---|
| A1, A2 | Survey check, owner's checkout and clean worktree | `6be14980520bb5e7`, `0360acd59ce9b2f3` |
| A3 | pinned binary's hash record | `cbb9792c4617e278` |
| A4, A5 | Doctor, clean and dirty | `2b1c858bfe3c2c04`, `2a597100b3b22c9e` |
| A6 | inventory (JSON) | `9ca979836417e4cd` |
| A7 | failed-job logs of run `43389262361` (2.19 MB) | `4a6eb177acb2c23f` |
| T1 | checker tests; plugin and signing tests | `58c9ff2a30a30afa`, `9baf96621c5a1d1a` |
| T2 | cases b/d, e2, h, i1, j, l2 | `f545c0b18607224c`, `f8c3af2166f1da93`, `707efddd99064a0f`, `4c1f677ee157ebbc`, `c88499c57ac7b499`, `b4b69985f3993c25` |
| T3 | results on interpreter, WASM, sandbox | `e5d9eedd2e1cb139`, `bfac22f50e0216c8`, `3245e5947404871f` |
| T3 | the oracle; the program | `36d8b22d157d7f31`, `ae9b42de6c1f269f` |
| F1 | WSL verification transcript | `1f5ea3c13bcd50cf` |
| H1 | the misdirected build's log | `6117ae2f4ad7e687` |
