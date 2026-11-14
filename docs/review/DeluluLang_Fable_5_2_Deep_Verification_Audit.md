# DeluluLang — Fable 5.2 independent deep-verification audit (rolling)

**Status: IN PROGRESS — session 1 (14 November 2026) of an open number. This is not a final report.**
Commissioned by the owner on 14 November 2026
([the commission](../prompt/DeluluLang_Fable_5_2_Multi_Session_Deep_Verification_Commission.md)). **Sections 1 to
12 are the research part: through this branch's second commit no production code, test, existing workflow or
normative document was changed.** The owner's
[continuation of the same day](../prompt/DeluluLang_Fable_5_2_Engineering_Architecture_Continuation_Prompt.md)
then authorised bounded, tested code changes on this non-default branch; each is recorded in §13 as it lands, with
its failing witness and what remains unverified. One document, updated in place each session; the machine-readable
resume state is [`fable-deep-verification-state.json`](fable-deep-verification-state.json); the one priority queue
is the recovery plan's ([*Single execution queue*](../DELULULANG_AI_FIRST_RECOVERY_PLAN.md#single-execution-queue)).

| | |
|---|---|
| Source under audit | `ac2a16202e790657810b819e882b8d721dc0aef4` (`origin/master` on 2026-11-14, fetched at session start) |
| Model and effort, as observable | The session harness names the model Claude Fable 5.2 (`claude-fable-5-2`); the owner's `/effort` command set `max`. No fallback notice was shown. Neither can be verified beyond those two statements. |
| Where it ran | The owner's laptop (Windows 11 26200, x86-64; WSL2 Ubuntu 20.04 on the same machine) and GitHub-hosted runners of the test repository: Ubuntu 24.04 x86-64, Ubuntu 24.04 ARM64, macOS 26 ARM64, Windows Server 2025 x86-64 |
| A limit of this session | One response was stopped by a platform safety check while I was planning new adversarial artifact inputs. From there I built **no new hostile artifacts**: track T1 is source analysis, the repository's own tests and fix design. What that leaves unverified is in §12. |
| Evidence words | **executed** = I ran it and kept the raw output; **source** = I read the code at the pinned SHA; **CI** = I read the named run's job results or logs; **other-party** = Codex's or an earlier audit's result, not re-run by me; **not run** = exactly that. |

---

## 1. Executive verdict (provisional after one session)

**Do I agree with Codex's recovery plan?** Mostly, with one correction of scope, one of order, and one of emphasis.

- **CLI-first: agree.** A 352-line program was written, checked, authorised and run entirely through the CLI on
  the interpreter, the WASM engine and the OS sandbox, with byte-identical results on **five machines and four
  operating-system and processor combinations** (§5). Nothing observed argues for a separate compiler or editor
  product. Not yet measured: the same task against a sandboxed Python or TypeScript baseline.
- **The five blockers: all five reproduce** at the pinned SHA, on every platform tried (§4). Three are wider than
  the plan says:
  1. *Outcome truth.* Exit code 2 is documented as "your invocation was wrong; the program was never examined".
     It is also what **every internal panic** returns (`crates/delulu/src/main.rs` line 93), what a run returns
     **after its effects** when the report file cannot be written (`run_cmd.rs` 1220–1230; also for a merely
     mistyped directory), and what a closed output pipe returns. And a `main` that returns `Err(..)` exits 0.
  2. *Verified artifacts.* The plan's repair (unique node ids, valid identifiers) leaves out two more things the
     artifact path accepts that the text path cannot produce (§4 T1).
  3. *`.solulu`.* `fmt` refuses the extension, and a package whose entry is `src/main.solulu` **checks clean** with
     the entry silently unread.
- **One blocker is narrower than it reads.** "Ordinary input and data remain missing" is true of the standard
  library (no arguments, no environment, no JSON, no way to build a character from its number) and **not** of the
  language: a hand-written JSON parser and transformer passed 17 of 18 pre-registered oracle cases on all three
  execution modes after one repair turn (§5).
- **Custody holds where it was tested, with one gap.** With two real OS accounts and separate keys, 28 of 31
  pre-registered steps came out as expected: an unsigned root is refused under strict mode, a narrower lease works
  once, and replay, wider scope, sibling path, write, over-delegation, expiry, revocation, a tampered certificate
  and a self-signed one are all refused; the other account cannot read the key, list the state directory or reach
  the broker (§4 T5). The three misses are one finding: **a broker adopts, and can then use, a certificate its
  anchor issued to a different key** — adoption never compares the certificate's subject with the adopter.
- **A deployment hazard Codex did not report.** A default-feature build links the build machine's `libpython`
  dynamically; under another OS account every command, `--version` included, died at load with exit 127 (§4 T6).
  The Python-free build has no such dependency and passed everything on four platforms.
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
- **Preview criteria: agree**, with two additions: a package must refuse, not skip, a source file it does not
  recognise; and the distributed binary must be the Python-free one, named by commit in its plain `--version`.

**Where Codex was too cautious:** language adequacy; cross-platform behaviour (it is uniform — §6). **Too
optimistic:** that a uniqueness check closes the artifact boundary; that exit 2 means nothing ran. **Mistaken in a
detail:** its recorded SHA-256 for the built binary cannot be matched to the file on disk (debug builds here are
not bit-reproducible; three builds of the same commit gave three hashes, §2).

---

## 2. Ground-truth snapshot

| Fact | Observation (2026-11-14, IST) | Evidence |
|---|---|---|
| Remote | `origin` = `github.com/jessesuniljs1-collab/delululang-test` (fetch and push); nothing else configured | executed |
| Heads | `HEAD` = `origin/master` = `ac2a162`; local branch `codex/research-recovery` (never pushed, no commits of its own) carried Codex's uncommitted work: one modified file and four untracked documents. Left exactly as found (their SHA-256 values re-checked after my one mistake, §12). | executed |
| My branches | `laptop/fable-deep-verification` (this audit; its own worktree, so the owner's checkout was never switched) and `laptop/fable-research-runner` (throwaway: one research workflow and my fixtures, source identical to `ac2a162`; never to be merged, to be deleted once read) | executed |
| The two branches reconciled (2026-11-14, before any code change) | Both start at `ac2a162` and neither contains the other: `git merge-base` with `origin/master` gives `ac2a162` for each; `git log --left-right --cherry-pick` shows three commits only on the runner (`3087715`, `06b2794`, `fd79554`) and one only here (`b3142a1`). The runner adds one workflow and four files under `research/fable-s1/` and changes no crate, manifest or lock; this branch adds documents only. The only paths both touch are the Survey's two derived files. Everything the runner produced is recorded here (§4 to §6), so nothing is recovered from it by merge; **this branch is the working branch**, and the runner stays a harness. | executed |
| Other refs | `origin/claude/relaxed-brown-2l78i1` at `b601210` (code at `d5e647d`), 2 commits ahead of `master`; `origin/autosave/laptop` at `4271780` (2026-11-08, not an ancestor of `master`); two old stashes; no open pull requests | executed |
| CI on the pinned SHA | Scheduled run `43409171490`: **18 of 18 jobs success**. Push run `43302145758`: 14 success, 2 skipped by design (`heavy-gates`, `miri-slow`). | CI |
| CI on this audit's branch | Run `43506016428` at `b3142a1` (documents only on top of `ac2a162`), dispatched by me with `jobs=everything`: **18 of 18 success**, the three `miri-slow` jobs included | CI (A9) |
| CI on the unfinished branch | Run `43389262361` at `d5e647d`: 14 success, 4 failure (`arm64`, `test` on macOS, Windows, Ubuntu). In each, the same two tests fail and nothing else: `release_requires_full_coverage` and `the_generated_reference_is_not_stale` (26 passed, 2 failed). No run exists for the branch tip `b601210`. | CI; failed logs downloaded and searched (A7) |
| Release workflow, dry run | Run `43508844764` at `b3142a1` (no tag, so nothing published): the portable archive was built for four targets, each archive's checksum and per-file checksums verified, and `INSTALL.md`'s own install block run against it — test passed, plain and sandboxed run printed. Archives 12.4–14.3 MB. The `release` job was skipped, as designed. Provenance attestation and artifact upload happen only for a tag, so no archive can be downloaded from a dry run. | CI; four job logs read (F2) |
| Research runner | Runs `43506513620`, `43508697812`, `43509259836` on the throwaway branch: my fixtures on four runners; the two-account custody test on Linux x86-64 | executed on GitHub runners (R1–R3) |
| Toolchain | Windows: `rustc`/`cargo` 1.96.1, pinned by `rust-toolchain.toml`; `gh` 2.100.0. WSL: the same 1.96.1, gcc 9.4, glibc 2.31, kernel 6.6.87.2. Runners: the same pinned toolchain. | executed |
| Pinned binary (Windows laptop) | Built from a clean worktree with `DELULU_BUILD_COMMIT` set: `cargo build -p delulu --locked -j 4`, debug, default features. 72,949,248 bytes, SHA-256 `b0a3050c422d9f36…84b4`. `--version --json` reports the commit and `x86_64-pc-windows-msvc`. | executed (A3) |
| Pinned binary (WSL) | Clean Linux-native clone at the same SHA: ELF x86-64, 460,373,712 bytes (debug), SHA-256 `6fa1b995e2a0a3df…1100`; reports the commit and `x86_64-unknown-linux-gnu` | executed (F1) |
| Runner binaries | Built on each runner from the throwaway branch (crates, manifests, lock and toolchain identical to `ac2a162`); each reports its own target triple | executed (R1, R2) |
| Binary on `PATH` | `~/.cargo/bin/delulu.exe`, dated 2026-08-07, SHA-256 `85067fb1…6bce`. Its plain `--version` prints `delulu 1.0.0` — **exactly what the fresh build prints**; only `--version --json` tells them apart (the old one answers that with the same plain line). Not used for any result. | executed |
| Codex's binary hash | The plan records `cfa46889…`; the file at that path now hashes `0c3ccb08…` (same size, same commit stamp). Not confirmable. | executed |
| Survey | Clean worktree: `ok: the Survey matches the tree (1727 nodes, 20757 edges)`, exit 0 — on Windows and in WSL. The owner's checkout (untracked documents present): three generated files reported stale, exit 1. On the runners a single stray file my own harness wrote (a Python bytecode cache) made it stale too. So staleness is a statement about the working tree, not about committed source. | executed (A1, A2, F1, R1) |
| Doctor | Clean tree: exit 0, `28 check(s) passed` (Windows, macOS, Linux ARM64), `29` (Linux x86-64, WSL). The owner's checkout: exit 1, one problem, the map's freshness. Doctor prints the version but not the commit, and compares no other executable. | executed (A4, A5, F1, R2) |
| State isolation | Every run used a scratch `DELULU_HOME` and `DELULU_STATE_DIR` and `DELULU_NO_FIRST_RUN=1`; fixtures, keys and logs are outside the repository | executed |
| Process identity | Laptop runs: the owner's one OS account (not elevated). Runner runs: the runner's account; in T5, two further accounts created for the test (uids 1002 and 1003). Sandboxed guests ran as a per-run AppContainer (Windows), a subordinate uid in a user namespace (Linux x86-64, WSL), or the same OS user (macOS; Linux ARM64 with Ubuntu's default restriction), as the tool itself reports. | executed |

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
| `delulu` (CLI, broker daemon, sandbox, LSP, MCP) | 39,935 | 9,958 | 45,715 | 160 | `main.rs` 56–95; `cli.rs` 2300–2349, 6436–6462, 8352–8392; `run_cmd.rs` 500–547, 1206–1232; the Windows `connect` in `broker_transport.rs` (466–491, by search) — about 260 lines | black-box on five machines: `new`, `check`, `run` (three modes), `test`, `fix`, `fmt`, `build`, `authority`, `schema`, `doctor`, `edit`, `atlas`, `mcp`, `broker`, `grants`, `audit`; tests `plugin_cli`, `plugin_load_cli`, `signing_cli` (50 passed) |
| `delulu-runtime` | 17,846 | 6,562 | 1,900 | 37 | `plugin.rs` 380–709; `interp.rs` 1008–1057 and 1925–2224; `scrub.rs` 1–40; `beneath.rs` 1–135 and 196–350 (the design and the whole Unix walk) — about 1,010 lines | through every `run` above |
| `delulu-check` | 13,315 | 4,930 | 1,004 | 0 | `dir.rs` 1–545 (all of its production code); `rcap_check.rs` 1225–1334; fragments of `check.rs` (about 120 lines); `prim_table.rs` 1–60 and the whole table by script; `fuzz.rs` 20–45 — about 860 lines | `cargo test --locked -p delulu-check`: 329 passed |
| `delulu-wasm` | 12,799 | 2,410 | 6,110 | 0 | none | through `run --engine wasm` and `build --target wasm` |
| `delulu-broker` | 8,832 | 5,785 | 1,306 | 0 | `cert.rs` 84–112, 296–412, 500–580 — about 230 lines | black-box through the T5 matrix (strict mode, certify, adopt, delegate, revoke, expiry, audit) |
| `delulu-syntax` | 6,525 | 3,019 | 0 | 0 | `ast.rs` whole (697); parser rules located by search | through every `check` |
| `delulu-survey` | 4,125 | 602 | 1,060 | 1 | none | `check` and `build` on three systems |
| `delulu-diag` | 3,248 | 827 | 99 | 1 | none | indirectly |
| `delulu-measure` | 2,730 | 259 | 156 | 0 | none | not run |
| `delulu-conform` | 1,665 | 54 (+457 test module) | 0 | 1 | none | not run (CI only) |
| `delulu-atlas` | 1,860 | 440 | 0 | 0 | none | one `atlas` call |
| `delulu-registry` | 1,061 | 3 (+519 test module) | 330 | 0 | none | not run |
| `delulu-fuzz` | 671 | 130 | 0 | 0 | none | not run (CI only) |

**Honest summary.** About 3,100 production lines read closely this session, roughly **2.7% of production Rust**:
the artifact verification path, the process entry point, the run report's last step, certificate verification and
adoption, and the Unix half of the file-system containment walk. Everything else is known to me only through
black-box runs, the repository's tests, or CI.

**One boundary read and found to match its own description.** `delulu-runtime/src/beneath.rs` opens the path the
scope check approved one component at a time from `/`, each with `openat(…, O_NOFOLLOW)`, creates without
truncating, and reads or writes only after `fstat` says the thing opened is a regular file (267–338). A link that
appears after the check is refused as such (235–248). Its header names what it does not close — hard links, and
the granted directory itself before a capability is minted. The black-box results agree on every platform (§6).
One stale sentence elsewhere: Doctor's `filesystem enforcement` line still says "check-then-open (RW 4.6)", which
undersells this code.

**Where the `unsafe` is.** Of the 200 production lines containing the word `unsafe`, 175 sit in nine OS-boundary
modules: `broker_transport.rs` 35, `identity.rs` 34, `jail.rs` 26, `beneath.rs` 24, `microvm.rs` 20, `ceiling.rs`
15, `guest.rs` 8, `foreign.rs` 7, `foreign_worker.rs` 6; the other 25 are spread over eleven files (some are the
word in a string, not code). The checker, parser, broker lattice, WASM code generator and WASM host contain none.
A lead for where a memory-safety review belongs, not a finding.

**Explicitly unexamined production paths** (none read; highest risk first): `delulu/src/brokerd.rs` and most of
`broker_transport.rs` (custody over IPC); `guest.rs`, `jail.rs`, `identity.rs`, `sandbox.rs`, `launcher.rs`,
`microvm.rs` (the OS boundary); `delulu-runtime/src/beneath.rs`, `egress.rs`, `prim.rs`, `custody.rs`,
`foreign.rs`, `device.rs`, `actors.rs`; `delulu-broker`'s `guard.rs`, `tree.rs`, the lease code, `audit.rs`,
`validate.rs`; `delulu-wasm/src/codegen.rs`, `host.rs`, `dpx.rs`; most of `run_cmd.rs`; `mcp.rs`, `lsp.rs`,
`edit.rs`, `doctor.rs`, `schema.rs`, `fix.rs`; `delulu-check/src/package.rs`, `deps.rs`, `lockfile.rs`,
`authority.rs`, `resolve.rs`, `unify.rs`, and most of `check.rs` and `rcap_check.rs`; `delulu-registry`;
`delulu-atlas`; `editors/vscode`; `scripts/`.

---

## 4. Challenge table — Codex's hypotheses, T1 to T8

Status words: CONFIRMED, FALSIFIED, NARROWED, REGRESSED, UNVERIFIED for claims; PASS, FAIL, PARTIAL, UNSUPPORTED,
NOT RUN, BLOCKED, INDETERMINATE for runs. Raw outputs are indexed in Appendix B.

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

### T2 — Truthful outcomes (RW 6.77, 6.83, 6.58, 6.55, 6.51, 6.79, 6.54) — CONFIRMED and wider, on every platform

Fixture: a program that reads `./input/task.txt`, writes `./output/result.txt` (the effect sentinel) and prints one
line. Run on the Windows laptop, then on Linux x86-64, Linux ARM64, macOS ARM64 and Windows runners: **every case
below came out the same on all five** (19 cases compared field by field, 0 differences).

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
| m | endless loop under `--limits wall=1` | exit 1 after about 1 s; report `stopped_by {dimension: "wall", budget_seconds: 1}` | PASS — a stop is reported as a stop |
| l | 50,000 lines to a reader that closes after one, both engines | **panic** ("failed printing to stdout"); exit **2**; **no report** | CONFIRMED (6.58), and see below |
| h | `fix --dry-run --json` on a file with one error | exit **0** while `summary.errors` is 1 and `errors_after` is 1 | CONFIRMED (6.55): exit code and envelope disagree |
| i | `test <path that does not exist> --json` | exit **0**; `passed: 0, failed: 0, tests: []` | CONFIRMED (6.51). Cause, by source (`cli.rs` 2304–2311): a positional argument that is not an existing path becomes a test-name filter |
| j | `schema validate envelope` on `{}` | exit 1; `validate.valid: false`; `summary.errors` **0** | CONFIRMED (6.79): the other way round from h (`schema.rs` 966 builds it inside a success envelope) |
| k1, k2 | `authority --isolation bogus`, `--broker bogus` | exit 0, no complaint | CONFIRMED |
| k3, k4 | `check` and `run` with an unknown option | exit 2, "refused, never ignored" | negative control PASS |
| n | `fn main(root: Root) -> Result[Unit, Str]` returning `Err("…")` | checks clean; exit **0**; report `{exit: 0, ran: true}` | **new** — a program cannot signal failure (ties to RW 2.40) |

**The wider finding.** `main.rs` 89–94 maps every panic of the CLI thread to exit 2, on purpose, so that a test
sweep can see crashes. `docs/for-agents.md` (the exit-code table) defines 2 as a usage error in which "the program
was never examined". Both cannot be true. Today **exit 2 does not imply that nothing ran**, and cases b, d and l
are instances, not the whole class: any internal fault after the first effect has the same shape, with no envelope
and no report. A caller that follows the documented contract will correct its command line and run the effect
again. The report-sink case is not an accident either: `run_cmd.rs` 1225–1227 returns 2 when the program's own
code was 0 and the report could not be written (the same shape at `guest.rs` 1780).

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
| The complete-program task | §5 | 17 of 18 on three modes, five machines |

### T4 — `.solulu` (with RW 6.52) — CONFIRMED, with two additions; identical on every platform

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

### T5 — Independent principals — PASS on 28 of 31 pre-registered steps; one finding

**Design, fixed before the run.** Principal A holds an anchor key and nothing else. Principal B runs the broker in
strict mode pinned to A's anchor, adopts A's certificate, delegates narrower leases and runs programs. Run twice:
on the laptop as one OS user with two keys and two homes (labelled same-user), and on a Linux x86-64 runner as
**two real OS accounts** (uids 1002 and 1003, each with its own home; the harness a third, 1001) using the
Python-free build. Expectations were written into the script first; tokens never left it.

| Step | Expected | Two accounts | Same user |
|---|---|---|---|
| B asks for an unsigned root under strict mode | refused DL1421 | as expected | as expected |
| A certifies a read scope for B's key; B adopts; B delegates a narrower lease; the delegate reads the one file | allowed, prints the file | as expected | as expected |
| The same token again | refused | DL1407 | DL1407 |
| The lease's scope, asked for wider / for a sibling / for a write | refused, nothing read or written | DL0703 each, naming the scope | same |
| B delegates a scope, an effect, or a write scope the certificate lacks | refused at delegation | as expected (three cases) | same |
| A lease revoked, then used | refused | DL1403 | DL1403 |
| A 2-second lease used after 3.5 s | refused | DL1402 | DL1402 |
| A's certificate with its scope edited | refused | DL1415 (signature) | DL1415 |
| A certificate B signs itself, naming A's anchor; then naming its own key as the anchor | refused both times | DL1415 both: strict mode trusts the pinned anchor, not the command line | same |
| The adopted root revoked; an unused lease under it used; a new delegation under it | all refused | as expected | same |
| The audit chain after all of it | verifies | 28 records, verified | verified |
| **B adopts a certificate A issued to a *different* key** | **refused (wrong holder)** | **adopted** | **adopted** |
| **…and uses the authority that certificate carried** | **nothing to use** | **the file in that scope was read** | **same** |
| A reads B's key file; lists B's state directory; points the CLI at B's state | denied by the OS; refused | permission denied; permission denied; DL1401 "not this user's to open" | not applicable |
| Key and state modes | owner-only | state directory 700; key files 600 | not applicable |
| Is a lease token readable from the process table while the leased run is alive? | observation | on the runner the listing did not show it, but my harness had not confirmed the run was still alive — INDETERMINATE there | **yes** (re-done in WSL with a scratch broker and a run confirmed alive: the token is in `/proc/<pid>/cmdline`, mode 444, and in `ps`; no `hidepid`). The token was single-use and already spent — its replay was DL1407 — so what is exposed is a multi-use lease, to any process of the same OS user (RW 4.90 CONFIRMED) |

**The finding (FABLE-022).** `Broker::adopt` (`delulu-broker/src/cert.rs` 538–580) calls `verify_chain` (322–412),
which checks the algorithm, each signature and its named issuer, the validity window, that the first certificate
comes from a configured anchor, linkage, and attenuation. **It never compares the leaf's `subject` with the
adopting broker's own key.** The subject is used only to decide who may sign the *next* certificate in a chain
(396–401; `cli.rs` 8376). So a certificate is usable by any broker pinned to the same anchor that obtains the
file while it is valid. RFC 0001 calls these "bearer" certificates (line 445) and its design paragraph does not
claim holder binding, so this is not a broken promise of the RFC; but the field's own documentation says "the
party this authority is delegated TO" (`cert.rs` 100), `certify` takes `--subject <HOLDER>`, and an operator
reading either would expect the name to be enforced. No test pins it either way, and `docs/REMAINING_WORK.md` has
no row for it. **Class: design limitation with a reachable path; consequence demonstrated on disposable fixtures
only (one file read under a certificate issued to another key). It matters wherever one anchor issues different
authority to different holders.** A decision is needed before any code (§10).

### T6 — Install, upgrade, platforms — PARTIAL, much further than before

| Item | Result |
|---|---|
| WSL-native build at the pinned SHA from a clean clone under `/home/user` | **PASS**: `cargo build -p delulu --locked -j 3`, exit 0, 1 m 43 s with a warm registry cache (not a first download) |
| The same build from a Windows-mounted path (`/mnt/d/...`) | FAIL at `libffi-sys`'s configure step ("config.log: No such file or directory") — observed by accident (§12); it is why the build must be Linux-native |
| WSL: Doctor, Survey, the §5 task on three modes, a denied grant | exit 0 with 29 checks; map matches; PASS ×3; DL0703 and no file |
| A debug build from source on four GitHub runners | PASS ×4, default features and Python-free (`--no-default-features --features net`) alike |
| **A default-feature binary under another OS account** | **FAIL: exit 127, "error while loading shared libraries: libpython3.13.so.1.0"** — for every command. The default build links the build machine's Python dynamically (`libpython3.8` in WSL, `3.13` on the runner); where the loader cannot find that exact library, nothing starts and no `delulu` diagnostic is possible. The Python-free build links no Python on Linux and macOS (checked with `ldd` and `otool`) and ran under both test accounts. (FABLE-025) |
| The portable release archive | Built and installed by the release workflow's dry run on four targets, the way `INSTALL.md` says, with checksums verified: PASS ×4 in CI (§2). Built and installed by me in WSL: see the last row. |
| A released v1.0.0 **archive**, and an upgrade from it | UNSUPPORTED: the test repository has a `v1.0.0` tag and **no releases or assets**; archives are kept only for a tag. No upgrade route is documented that I have found. |
| Identity of an installed binary | Plain `--version` cannot tell a three-month-old binary from today's (§2); `--version --json` can, on builds that have it |
| First-time setup from the documents alone | PARTIAL: `delulu new` produced a package that checked, tested and ran at once on all five machines. Install from `INSTALL.md` by a fresh reader: the workflow runs that page's own block; a human or agent reading cold: NOT RUN |
| The portable archive built and installed by me, release profile | **WSL (Ubuntu 20.04, Python 3.8): FAIL** — `scripts/package-toolchain.sh` built the release binary, then stopped at the notices step with a traceback (`write_text() got an unexpected keyword argument 'newline'`, `scripts/third-party-notices.py` line 202; that argument needs Python 3.10). No archive. (FABLE-032) **Windows laptop (Python 3.13): PASS** — the same script produced `delulu-1.0.0-x86_64-pc-windows-msvc.tar.gz` (12,556,366 bytes; 34 entries; third-party notices for 241 dependencies and an SBOM of 232 components; release build 4 m 52 s with a warm cache), and `scripts/check-install.sh` ran `INSTALL.md`'s block against it as written: archive checksum and every per-file checksum OK, `--version`, `new hello`, `test .` (1 passed), `authority . --grants`, `run` plain and `--sandbox` (a per-run AppContainer) — every step succeeded. The binary inside is 36,079,104 bytes and reports the commit it was told (`DELULU_BUILD_COMMIT`, which I set to the audited SHA; the tree differed from it in documents only). (F3, F5) |

### T7 — Survey, Doctor, machine interfaces — PARTIAL

**Survey and Doctor:** §2. **Checked edits** (`delulu edit`, four cases on a scaffolded package): a stale hash is
refused with exit 2, nothing written, and the answer carries the current hash; a clean node replacement is
written; an edit that adds an effect beyond the package ceiling is not written under `--if-checks`, exit 1, with
`widened: ["Read", "fs.read=./x"]` and DL0502 and DL1009; the same under `--dry-run` reports the same. PASS ×4.

**MCP** (`delulu mcp`, black-box over standard input and output, same on all four runners):

| Probe | Observed |
|---|---|
| `initialize` asking for `2025-06-18` | answered `2025-06-18` |
| `initialize` asking for `2025-11-25`, `2026-07-28`, or `1999-01-01` | answered `2025-06-18` each time, no error (the legacy rule: answer with a version you support) |
| `server/discover` | −32601, not a method this server offers |
| `tools/list` | 12 tools outside a source tree, 16 inside one; every tool has an `outputSchema`; the order is stable; the reply is **229 kB to 249 kB** |
| a malformed JSON line, then `ping` | −32700, then the ping is answered |
| **one line of invalid UTF-8, then `ping`** | **the server ends silently with exit 0; the ping is never answered** (CONFIRMED, RW 6.76 family) |
| an unknown tool; an unknown method | −32602; −32601 |

By the 2026-07-28 specification's own terms (§8) this is a **legacy-era server**: a dual-era client falls back to
`initialize` and works; a modern-only client fails. Its newest version is two revisions behind.

**Timing** (median of 5 to 11 fresh processes, debug builds, so only the shape matters):

| Command | Windows laptop | Windows runner | Linux x86-64 | Linux ARM64 | macOS ARM64 |
|---|---:|---:|---:|---:|---:|
| `--version` | 9 ms | 9 | 4 | 3 | 8 |
| `check` (hello) | 11 | 12 | 6 | 7 | 11 |
| `run` interpreter / WASM / sandbox | 13 / 27 / 64 | 12 / 32 / 63 | 7 / 34 / 38 | 8 / 36 / 57 | 11 / 21 / 175 |
| **`test` (one test)** | **1,017** | **1,016** | 8 | 8 | 11 |

`delulu test` costs a fixed second on Windows and nowhere else. Cause, by source: it first asks whether a broker
daemon is reachable (`cli.rs` near 2434), and the Windows client retries a missing pipe for a full 1,000 ms
(`broker_transport.rs` 466–491) — a window meant for a daemon that is mid-reconnect, paid in full when no daemon
exists. (FABLE-024.) Also measured: `toolchain --json` is 69,674 bytes; `--help` is 22,993 bytes.

**Not run:** LSP, the Atlas beyond one call, the editor extension.

### T8 — Simulated devices — CONFIRMED (simulation only; nothing physical)

| Case | Observed |
|---|---|
| Named simulator (`--broker-profile sim`): command inside the envelope, command outside it, sensor read | `COMMANDED`; `REFUSED: angle_deg = 999 is outside the envelope [-30, 95]`; `READ 12.0` |
| **No profile** | **`COMMANDED`** for the in-envelope command, then `NO READING` — an `Ok` with no device behind it, although the error type has a `NoDevice` variant (RW 4.88 CONFIRMED) |
| The run report in either case | no device section at all: nothing machine-readable tells a simulated device from none |
| No actuator grant | DL0703 before anything |
| WASM engine | typed DL1201 before anything |
| Sandbox with the simulator | same three lines as in process |

Watchdog, heartbeat expiry, host loss during a command, emergency stop: NOT RUN.

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

| Machine | Interpreter | WASM engine | OS sandbox (L1) | Modes agree byte for byte | Largest input, ms (interp / WASM / sandbox) |
|---|---|---|---|---|---|
| Windows 11 laptop | 17 / 18 | 17 / 18 | 17 / 18 | yes | 1,281 / 698 / 1,366 |
| Windows Server 2025 runner | 17 / 18 | 17 / 18 | 17 / 18 | yes | 2,592 / 1,340 / 2,537 |
| Ubuntu 24.04 x86-64 runner | 17 / 18 | 17 / 18 | 17 / 18 | yes | 1,811 / 1,145 / 1,874 |
| Ubuntu 24.04 ARM64 runner | 17 / 18 | 17 / 18 | 17 / 18 | yes | 2,062 / 1,195 / 2,158 |
| macOS 26 ARM64 runner | 17 / 18 | 17 / 18 | 17 / 18 | yes | 2,518 / 1,205 / 2,828 |
| WSL2 Ubuntu 20.04 (first fixture only) | PASS | PASS | PASS | yes | — |

The one failure is the same everywhere: `\uXXXX` escapes — the language cannot build a character from its number,
so the program refuses them. Run without the write grant: DL0703, exit 1, no file, on every machine.
`authority --within` the three grants used: `within`, 5 requests in 3 dimensions. The Python-free build gives the
same answers (first fixture, three modes, four runners). The timings in the table are debug builds.

**Release profile, measured once** (WSL2 x86-64, the Python-free release binary the packaging script builds,
42,259,176 bytes; medians of 3 to 11 fresh processes; every output equal to the oracle's):

| Workload | Release | Debug, same machine | Python 3.8 running the oracle |
|---|---:|---:|---:|
| `--version` / `check` hello / `run` hello (interpreter) | 3 / 4 / 4 ms | 7 / 9 / 10 ms | — |
| `run` hello on WASM / in the sandbox | 7 / 31 ms | 24 / 43 ms | — |
| The 2,000-item task: interpreter | **165 ms** | 1,065 ms | 30 ms |
| The 2,000-item task: WASM | **71 ms** | 645 ms | 30 ms |
| The 2,000-item task: sandbox | 196 ms | 1,109 ms | — |

So a parser written *in* DeluluLang is 2.4 times (WASM) to 5.5 times (interpreter) slower than Python's C-backed
`json` on a 100 kB document, and the process starts in 3 to 4 ms. One machine, one task: a lead for the
performance work, not a benchmark.

Cases covered: escapes, raw non-ASCII text, trailing comma, missing key, wrong type, 40-deep nesting in an ignored
key, 64-bit overflow, empty list, duplicate key, invalid UTF-8 input, whitespace and negatives, a float where an
integer is required, a non-object document, trailing garbage, a leading zero, and a 2,000-item input.

**What this shows and does not.** It shows a useful, non-trivial data program can be written by an agent today,
runs identically on every execution mode and platform tried, and is held to its grants. It does not show an agent
can do it *cheaply*: the parser is about 200 lines a standard library should own, it cannot be complete (`\u`),
and it is slow.

---

## 6. Platform matrix (observed, by name)

| Platform | What ran at `ac2a162` source | Sandbox (L1) as the tool reports it | Class |
|---|---|---|---|
| Windows 11 x64 (laptop) and Windows Server 2025 x64 (runner) | builds; all of §4 and §5 | five properties established; guest is a per-run AppContainer | executed |
| Ubuntu 24.04 x86-64 (runner) | builds; §4 T2, T4, T5 (two accounts), T7; §5 | five established; guest is a subordinate uid in its own user namespace (with the one sysctl CI also sets) | executed |
| Ubuntu 24.04 ARM64 (runner) | builds; §4 T2, T4, T7; §5 | five established, **but the guest runs as the same OS user**: Ubuntu's default restricts unprivileged user namespaces, and the tool says so in Doctor, on stderr and in the property's reason | executed |
| macOS 26 ARM64 (runner) | builds; §4 T2, T4, T7; §5 | **`filesystem_confinement: absent`** ("writes denied; reads not confined"); guest runs as the same OS user under Seatbelt; the other four established | executed |
| WSL2 Ubuntu 20.04 x86-64 | native build; Doctor, Survey, the task on three modes | Landlock plus a user namespace | executed |
| MicroVM (L2) | — | `microvm`, `microvm-reproducible` jobs only | CI |
| Release archive install, four targets | the release workflow's own install step | plain and sandboxed run inside it | CI |

**File-system semantics follow the host**, with the scope check holding on each: a name differing only in case is
the same file on Windows and macOS and a different one on Linux; an NFD spelling of an NFC file name finds the
file on macOS only; `..` out of the granted directory and a symbolic link pointing out of it are refused with
DL0904 on every platform where the link could be created (all runners, WSL).

**Cross-platform differences found: two.** The fixed second in `delulu test` on Windows (§4 T7), and the sandbox
properties above. Everything else compared — 19 outcome cases, 11 extension cases, the 18-fixture task on three
modes — was identical.

---

## 7. Security and supply-chain findings so far

| Id | Finding | Class | Severity as demonstrated | Evidence |
|---|---|---|---|---|
| FABLE-006 | Artifact verification does not re-establish what the parser guarantees (ids, `recovered`, identifiers, shapes) | confirmed defect in a verifier | MEDIUM; opt-in path | source; other-party witness |
| FABLE-007 | `Module.recovered` is honoured on the artifact path | plausible issue with a reachable path | not assessed | source only |
| FABLE-008 | The R-Get per-site table is keyed by artifact-supplied ids | plausible issue with a reachable path | not assessed | source only |
| FABLE-009 | `why` reports from unverified stored facts | confirmed (review-surface integrity) | LOW–MEDIUM | source |
| FABLE-010 | The artifact fuzz target never reaches verification | assurance gap | LOW | source |
| FABLE-013 | Exit 2 covers both "nothing ran" and "an internal fault after effects" | confirmed (outcome integrity; duplicate effects on retry) | MEDIUM for agent harnesses | executed on five machines; source |
| FABLE-016 | A package check passes while skipping an unrecognised source file | confirmed (a gate that counts only what it reads) | LOW today | executed |
| FABLE-022 | Certificate adoption does not bind the subject to the adopter | design limitation with a reachable path | MEDIUM where one anchor serves several holders; consequence shown on fixtures only | executed (two accounts, and same user); source |
| FABLE-025 | The default build cannot start without the build machine's `libpython` | confirmed (availability and distribution) | MEDIUM for source installs; none for the portable archive | executed (runner, two accounts); `ldd` |
| FABLE-026 | One line of invalid UTF-8 ends the MCP server silently with exit 0 | confirmed (availability of a service an agent depends on) | LOW | executed on five machines |
| FABLE-027 | The null device answers `COMMANDED` | confirmed (a success with nothing behind it) | LOW in simulation; a hard stop before any physical use | executed |
| FABLE-031 | A lease token is passed on the command line and is readable from the process table | confirmed (credential exposure; RW 4.90) | LOW for single-use leases (spent at start), MEDIUM for multi-use ones on a shared account | executed (WSL) |
| FABLE-032 | The packaging script needs Python 3.10 or newer and says so only by a traceback, after the release build | confirmed (build reproducibility for others) | LOW | executed (WSL, Python 3.8) |

**Dependencies, read from their own repositories on 2026-11-14.** The lock file pins Wasmtime **48.0.8**, which is
the newest patch on the 48 line (released 2026-10-29); the newest stable is 50.0.1, and 51.0.0 is at its first
release candidate. Wasmtime published about thirty advisories between 2026-10-02 and 2026-10-29, three of them
critical; every one that names a patched 48.x version names 48.0.8 or earlier. So the pin is current today, and
the pace — four patch releases on that line in October — is the maintenance obligation: an engine this project
ships needs its security releases followed within days. `wasmtime-wasi` and `wasi-http`, where several of those
advisories live, are not in the lock file. Other pins read: `cranelift-codegen` 0.135.8, `reqwest` 0.12.28,
`rustls` 0.23.46, `pyo3` 0.25.1, `ciborium` 0.2.2, `ed25519-dalek` 2.2.0, `blake3` 1.8.5 — not checked against
advisories by me. The `supply-chain` job was green in runs `43409171490` and `43506016428` (CI only).

**Network refusals, twelve cases with no server and nothing leaving the machine** (laptop; reserved names and
special-use addresses only): no grant, and a host minted beyond the grant — DL0703; a URL whose host the
capability does not name, and a `user@host` spelling that really points at loopback — DL0904; `http://` and
`file://` — not delivered, an `Err` to the program, counted in the report as one request not delivered; and a
plain `net=` grant naming loopback (IPv4, IPv6, `localhost`, **and the decimal spelling `2130706433`**), the
link-local metadata address or a private range — refused at the grant itself, exit 2, each named as special-use.
All twelve refused before delivery (FABLE-036).

**The allowed path and its edges, on loopback only** (WSL, release Python-free build; a throwaway certificate
authority and an HTTPS server on `127.0.0.1` that records method and path; scratch keys deleted afterwards; the
grant was `net.special=127.0.0.1/v1/**`). Witnessed on the far side, from the server's own log:

| Case | Expected | Program saw | Server saw |
|---|---|---|---|
| Path inside the grant, the scratch authority trusted (`SSL_CERT_FILE`) | delivered | `BODY pong`, exit 0 | `GET /v1/ok` |
| The same, authority **not** trusted | not delivered | an `Err`; the note names the failed TLS handshake | nothing |
| A redirect that stays inside the path | followed | `BODY pong` | `GET /v1/redirect-in`, `GET /v1/ok` |
| A redirect that leaves the path | not followed | an `Err`; the note names the path scope | `GET /v1/redirect-out` only — **never `/other/secret`** |
| A path outside the grant | refused | DL0904, exit 1 | nothing |
| The program mints the whole host under a path-scoped grant | refused | DL0703, exit 1 | nothing |
| The allowed request on the WASM engine, and in the sandbox | delivered | `BODY pong` twice | `GET /v1/ok` twice |

Eight of eight as expected (G2). Not run by me: DNS rebinding, a redirect to another host or to a special-use
address, a slow or oversized response, proxies, and anything on Windows or macOS beyond the twelve refusals.

**Not assessed this session:** the licence graph, CI action pins and permissions, release keys, the microVM
image's obligations.

**Threat-model notes established so far.** A Verified artifact is untrusted bytes until `step5_verified` returns.
Its signature is optional and is not evidence of meaning. Loading requires the operator's `--grant plugin=<dir>`
and the program's `load`; a package may pin artifacts by hash. An export runs in its own interpreter under the
host's custody and is bounded by the capabilities the host passes it. One OS account runs the CLI and the
embedded broker and, in LEGACY mode (the default), can mint root authority — Doctor says so itself. In strict
mode a root enters only under the pinned anchor (verified, T5), but any certificate from that anchor, to anyone
(FABLE-022). Between two OS accounts the boundary is the operating system's and held in every probe.

---

## 8. External research (dated 2026-11-14)

Read from the publishers' own repositories through the GitHub API unless marked *searched*, which means a web
search restricted to the publisher's site and summarised, not a page I read in full.

| Subject | What the source says | Consequence for DeluluLang |
|---|---|---|
| **A2A** — `a2aproject/A2A` | Latest release **v1.0.1**, published 2026-05-28; v1.0.0 on 2026-03-12. The specification at that tag defines eleven operations (Send Message, Send Streaming Message, Get Task, List Tasks, Cancel Task, Subscribe to Task, four push-notification configuration operations, Get Extended Agent Card). §3.3.1: "Send Message operations MAY be idempotent. Agents may utilize the messageId to detect duplicate messages." §7: "Identity information is handled at the protocol layer, not within A2A semantics"; §7.5: "Authorization logic is implementation-specific". §3.6: clients send `A2A-Version` (major.minor); an unsupported version is a `VersionNotSupportedError`. | An A2A identity is not a grant, and an A2A send is not exactly-once. A bridge must map an authenticated caller to a lease under the broker's own policy and carry its own idempotency key for committed effects — which needs S3's outcome contract first. Adapter, not built-in; not before S1 to S4. |
| **MCP** — `modelcontextprotocol/modelcontextprotocol` | Revisions present: 2024-11-05, 2025-03-26, 2025-06-18, **2025-11-25**, **2026-07-28**. The 2026-07-28 changelog (against 2025-11-25): the protocol is stateless; `initialize` is removed and every request carries its version and capabilities in `_meta`; servers MUST implement `server/discover`; `ping` and `logging/setLevel` are removed; tasks move to an extension; every result carries `resultType`; list results carry `ttlMs` and `cacheScope`; Roots, Sampling and Logging are deprecated. The versioning page defines *modern*, *legacy* and *dual-era*, and its matrix says: modern client with legacy server — "Fails"; dual-era client with legacy server — "Works", by probing `server/discover` and falling back to `initialize`. | `delulu mcp` is a legacy-era server (§4 T7). It works with dual-era clients today. Becoming dual-era fits its design — each call is already an independent subprocess — and should be done when a real client needs it, not before S1 to S4. Keep the legacy path and its tests. |
| **OpenShell** — `NVIDIA/OpenShell` | Latest release **v0.1.8**, 2026-11-10; a release each week since v0.1.4 (2026-10-13) | Codex's figure confirmed. A weekly-moving dependency: an optional backend at most, pinned and re-tested per bump. |
| **Wasmtime** — `bytecodealliance/wasmtime` | §7 | the pin is current; follow security releases |
| **SLSA** *(searched, slsa.dev)* | v1.2 is the current specification (announced 2025-11-24); it adds a Source track beside the Build track | The release workflow already attests provenance for tagged builds. Claim a level only after verifying an attestation from a real tag. |
| **NIST SSDF** *(searched, csrc.nist.gov)* | SP 800-218 v1.1 is the latest final; SP 800-218 Rev. 1 (SSDF 1.2) is an initial public draft of 2025-12-17 | Use as a checklist of practices; claim nothing. |
| **UNECE R155 / R156** *(searched, unece.org)* | R155's latest adopted amendment found: Supplement 3, in force 2025-01-10. R156: original in force 2021-01-22; a new series was being prepared for WP.29 in 2026 — adoption not confirmed by my search | Vehicle type-approval rules bind manufacturers' management systems. Nothing here is addressed by a language's effect system; no claim should be made. |
| **ROS 2 security** *(searched, docs.ros.org)* | Security comes from DDS-Security's plug-ins (authentication, access control, cryptography, logging, data tagging), configured per enclave by the `sros2` tools; artifacts apply directly only to DDS-based middleware | If a robot integration is ever justified, DeluluLang sits above it as a supervisory program behind a host adapter. Not now. |
| ETSI C-ITS message standards | not checked by me | Codex's entries stand as other-party |

**Verdict on protocols.** No new protocol. One adapter at a time, each after the local job is trustworthy: MCP
dual-era first (smallest, and a client exists), an A2A worker bridge only after S3.

---

## 9. Keep / repair / consolidate / defer — entries with evidence so far

| Item | Verdict | Evidence | Condition to revisit |
|---|---|---|---|
| Text source, one checker, two engines, the OS sandbox | **Keep** | one program, three modes, five machines, identical results (§5, §6) | — |
| Serialised syntax trees as the Verified artifact format | **Repair now; reconsider the format** | §4 T1: every parser rule needs a twin at decode. Shipping canonical source text and re-parsing it would make the invariant true by construction; the earlier pipeline audit measured the tree at 28 times the size of text | after the minimal repair, measure verify time and size for text-in-artifact against the tree |
| Exit-code contract | **Repair** | §4 T2 | needs a recorded decision: it changes a published contract |
| `delulu why` on artifacts | **Repair** (verify first) | `cli.rs` 6450 | — |
| Standard library: JSON, arguments, character codes, exit status | **Build** | §4 T3, §5 | — |
| Embedded Python in the default build | **Make optional by default** — ship and document the Python-free build as *the* build; keep Python behind an explicit feature, loaded only when used | §4 T6: the default build does not start without one exact shared library | when a user needs `root.python` in a distributed binary |
| Custody: strict mode, leases, revocation, audit, the two-account boundary | **Keep** | §4 T5: 28 of 31 | — |
| Federation certificates | **Decide** whether the subject is enforced at adoption, then repair code or words | §4 T5 | the owner's or a recorded decision; `rfcs/` is entrenched |
| MCP server | **Keep, read-only**; fix the silent exit; add the modern era when a client needs it | §4 T7, §8 | a modern-only client appears |
| `.solulu` | **Defer behind the above**, then do as Codex's P1.2 with `fmt` and the skip-refusal added | §4 T4 | when outcome and input slices have landed |
| Survey, Doctor | **Keep**; make Doctor name the commit and the selected executable | §2 | — |
| Null device profile | **Repair**: refuse, or return `NoDevice`, when no device profile is named | §4 T8 | before any device work |
| Everything else in Codex's component table | not yet examined by me | — | later sessions |

### Governance, review and licence — first pass (facts read; recommendations only)

| Fact, as read on 2026-11-14 | Source |
|---|---|
| The test repository is public, Apache-2.0, issues on, discussions off; **`master` is not protected and there are no rulesets**; private vulnerability reporting is **enabled**; secret scanning and push protection are on; Dependabot security updates are off | GitHub API |
| One code owner for every path, the entrenched ones included | `.github/CODEOWNERS` |
| The project lead decides; anyone of any kind may contribute and may hold any role; **every RFC, and every change in a risk class, needs a named sponsor who is not its author**; "no change merges on the say-so of its own author"; one model reviewing its own output counts as one party | `GOVERNANCE.md` §1–§2, `CONTRIBUTING.md` §4 |
| The working rule for sessions is to land on `master` by fast-forward on a green CI run the session has read | `docs/AI_ONBOARDING.md` §3 |
| Security reports go through GitHub's private form only, with an acknowledgement in 3 working days and a response in 10; no e-mail address or key is offered, and the page says why | `SECURITY.md` §1 |
| `NOTICE` asserts the copyright and that "DeluluLang" is a trademark of the project lead, and restates the naming policy | `NOTICE`, `TRADEMARK.md` |

**What follows from them.**
- *Proposals.* The owner's policy is that anyone may propose without a sponsor. The written rule asks for a
  sponsor before an RFC is even opened. Recommendation, as Codex's: move the requirement from *opening* to
  *merging* a risk-class change, and call it what it is — independent technical review, recorded on the change.
- *Practice against rule.* By the documents' own description, sessions land their own work on green CI. I did not
  audit the history, but nothing mechanical enforces a second party: the branch is unprotected and there is one
  account. Either the rule or the practice should change in writing. For a one-maintainer project the honest form
  is: low-risk changes land on CI with sole review recorded as such; changes to authority, the Guard, custody,
  artifact verification and release wait for a reviewer who is neither the author nor run by the author — and
  when the owner waives that, the waiver is a recorded decision, never called a review.
- *Controls worth turning on* for the repository that will be released from (the owner's to decide; none was
  changed): required status checks, no force-push and no deletion on the default branch, tag protection, and
  dependency alerts. A required pull-request review cannot work with a single account and should not be switched
  on until a second identity exists.
- *Licence.* Nothing read argues for leaving Apache-2.0. Whether the trademark sentence in `NOTICE` should say
  "is a trademark" or "claims rights in" is a legal question I cannot answer; it needs counsel before a public
  release, as Codex also said.
- *Security policy.* The reporting route exists and is switched on where the policy says. The new finding in §4
  T5 is recorded here in the same public way this repository records its other open security rows; the owner may
  prefer to hold such a finding as a private advisory until fixed, and that is his call.

---

## 10. Revised execution queue — five code slices, and one decision

Codex's order was P0.1, P1.1, P1.2, then P1.3 to P1.6. Mine, with the decisive evidence for each move. Existing RW
numbers keep their identity. Sections of the recovery plan that would need the matching edit: *Decision*
(the "next three code milestones" sentence), *Single execution queue* P0.1 (scope), P1.1 (scope) and the order of
P1.2 against P1.3 and P1.4.

| # | Slice | RW | Where | Why here | Size | Independent negative oracle |
|---|---|---|---|---|---|---|
| S1 | **Artifact ingress, minimal and fail-closed.** Refuse duplicate node ids, a non-empty `recovered`, and identifiers the lexer would refuse; after replay, require the type table's size to equal the node count. `why` verifies before it reports. No wire-format change. | 4.93, 4.94, 6.67 | `delulu-check/src/dir.rs` (`Dir::decode`, `verify_checked`); `cli.rs` near 6450 | A soundness hole in the feature named "Verified"; one file; no compatibility decision; RW 4.95's guarantee inside a plugin depends on it | S | Honest artifacts from the existing corpus still verify byte for byte; each guard removed in turn makes exactly its own test fail |
| S2 | **False greens and small agent-loop faults.** `test` with a path that matches nothing refuses; `fix` exits 1 while errors remain; `schema validate` counts its failure in `summary.errors`; `authority` refuses option values it does not know; a package refuses a source-like file it does not read; the MCP server answers an undecodable line with an error and keeps serving; `test` on Windows does not wait a second for a daemon that is not there | 6.51, 6.55, 6.79, 6.54, 6.76 | `cli.rs` 2304–2311; `fix.rs` (around 277); `schema.rs` 966; `package.rs`; `mcp.rs`; `broker_transport.rs` 466–491 | An agent's inner loop is `check`/`test`/`fix`; a false green there defeats every later gate. Each item is tiny. | S | The existing passing cases keep their exit codes; `test .` on a package with no tests stays 0; a daemon mid-reconnect is still reached |
| S3 | **Outcome contract, widened.** The report sink is opened before the program starts; failure to persist after a run is `failed` with `ran: true`, never `usage`; an internal fault gets its own exit code and envelope; a closed standard output is a stop; a sandboxed pre-start refusal writes the report; `main` returning `Err` exits non-zero | 6.77, 6.83, 6.58, 6.84, 2.40 | `run_cmd.rs` 1220–1230 and its entry; `guest.rs` 1780; `main.rs` 93; console writes in `prim.rs`; `docs/for-agents.md` | T2: exit 2 does not mean nothing ran, on any platform | M | Case c (refused before start) still exits 2 with no effect; case a unchanged; the sentinel is written exactly once in every failing case |
| S4 | **Input and data.** `root.args()`; a standard `parse_json`/`to_json`; a way to build a character from its number; on both engines or with a typed early refusal | 2.40, 2.41 | `prim_table.rs`, `check.rs`, `prim.rs`, `delulu-wasm` | §5: the language can already do the work; the library makes it cheap and complete | M–L | The §5 oracle: all 18 fixtures, three modes, four platforms — the runner exists |
| S5 | **`.solulu` in packages**, as Codex's P1.2, plus `fmt` | 6.52 | `package.rs`, `deps.rs`, `cli.rs`, `lsp.rs`, the editor | Loose files already work; nobody is blocked today | M | Both extensions, both directions; a duplicate module name across extensions is refused |

**One decision before code — certificate subjects (FABLE-022).** Three honest options: (A) bind — at adoption the
leaf's subject must be the adopting broker's own key, with `DEPLOYMENT.md`'s recipe changed to take that key from
the host; (B) keep bearer certificates and say so in `cert.rs`, `certify --help` and `DEPLOYMENT.md`, and tell
operators a certificate file is as good as the authority it names; (C) add an explicit audience field. I recommend
(A) for strict mode: it makes per-holder authority real, costs a few lines in `Broker::adopt`, and refuses only
certificates that name someone else. It touches behaviour described by RFC 0001, which is entrenched, so it needs
a recorded decision first.

**Also small, for Codex's P1.5:** plain `--version` should print the commit and target; the portable (Python-free)
build should be what `cargo install` gives by default, or the default build should load Python lazily.

After these: Codex's P1.5 (identity), P1.6 (loss and stop), P2.1 (recover RW 2.50 from its branch — its four red
jobs are two stale generated-file tests, §2, so nothing in S1 to S3 is blocked by it), then the rest as planned.

**Preview gates.** I agree with Codex's nine, add "a package refuses what it does not read" and "the shipped
binary is the Python-free one and names its commit", and note that gate (i) cannot be called met by S1 alone: it
needs the by-construction test of §4 T1 or the format change of §9.

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

**A ready multi-platform acceptance harness exists** for S2 to S4: the throwaway branch
`laptop/fable-research-runner` holds one workflow and three scripts that build the binary on four runners and
compare 30 outcome cases and the 18-fixture task against fixed expectations. A coding session can copy them onto
its own branch; they assert nothing by themselves and are not part of the repository's suite.

---

## 12. What I did not verify, and what went wrong

**My mistakes this session.**
- *A misdirected command.* My first WSL build command lost its quoting on the way through `wsl.exe`. It ran in the
  Windows checkout instead of a Linux directory and created, in the owner's checkout root, a nested clone `dl/`
  (with 1.6 GiB of build output) and four `build.*` files. Nothing tracked changed and nothing was pushed. I saved
  the logs (H1), removed exactly those five entries after checking they were mine, and confirmed the checkout's
  status and the four untracked documents' hashes were as at the start. WSL commands now go through script files
  with absolute, guarded paths.
- *Harness errors, each caught before its result was recorded:* the first T4 control called a function the wrong
  way; a WSL script could not find `rustc` until its `PATH` was set; my runner's first round left a Python bytecode
  file in the tree, which made Doctor report the map stale on four runners; my first T5 script's leases lacked the
  effect the console needs, so three "refusals" were for the wrong reason and one allowed step failed; my compact
  MCP probe stopped reading one reply early. The first T5 round on the runner failed for a real reason instead
  (FABLE-025).
- *Sub-agents.* Three page-fetching sub-agents I started were stopped by the repository's own gate that gives
  sub-agents no authority. They did not work around it and neither did I; I read the same sources through the
  GitHub API.

**Not verified** (each is required by the commission; none may be read as done):

| Item | Why | The next runnable step |
|---|---|---|
| T1 at run time: does a refused-as-text artifact reach an export call; does the per-site table matter; what does a `recovered` literal do when executed | I built no hostile artifacts this session (see the header) | Becomes the red tests of §11 step 1, written inside the repository's test suite by the coding session |
| Codex's A.5 result, re-run by me | same | `cargo test` of §11 step 1 (i) before the fix |
| T5 on Windows and macOS with two accounts; across two machines | not attempted | Windows: two local accounts; macOS: `sysadminctl`; both possible on runners |
| The Windows half of the file-system walk (`beneath.rs` 381–580) | not read; black-box results only | read it beside a junction and an 8.3-name probe on a Windows runner |
| A released archive upgrade | no release assets exist | needs a tag, which the owner alone decides |
| A cold first-download build | registry caches were warm everywhere I built | a fresh `CARGO_HOME` |
| Release-profile performance beyond one task on one machine | §5 has one measurement | the same script on the four runners with a release build |
| The full workspace suite locally | not attempted (CI ran it at this SHA and on this branch) | — |
| LSP, the editor extension, the Atlas beyond one call | not attempted | start with a recorded LSP session on a package |
| Devices beyond three calls; watchdog, heartbeat, host loss, emergency stop | not attempted | the simulator with `--sim-step` |
| A paired agent study against Python or TypeScript; governance, licence and documentation-ownership review; ETSI | not started | later sessions |
| 97.6% of production source | §3 | by workstream, highest risk first: `beneath.rs`, `egress.rs`, `guard.rs`, the lease code, `brokerd.rs` |

---

## 13. Engineering changes on this branch

Authorised by the owner's continuation of 14 November 2026: bounded, tested changes on this non-default branch;
no merge, tag, release or change to the default branch. **At this commit (checkpoint 2): none yet.** Each later
change gets one row here — its failing witness before the change, the smallest repair, the negative control, the
tests and CI run read, and what is still unverified — and its own commit.

---

## Appendix A. Evidence registry

One line per proposition. All at source `ac2a162`, 2026-11-14. "Five machines" means the Windows laptop and the
four runners of §6.

| Id | Claim | Level | Verdict | Re-check when |
|---|---|---|---|---|
| FABLE-001 | Survey staleness comes from files in the working tree that are not in the map; a clean checkout matches (1727 nodes, 20757 edges) | executed, three systems | CONFIRMED | `delulu-survey` or `docs/survey` changes |
| FABLE-002 | Doctor: exit 0 clean, exit 1 with one problem when the tree has a stray file; it names no commit and compares no other executable | executed | CONFIRMED | `doctor.rs` changes |
| FABLE-003 | CI at the pinned SHA is 18 of 18; this audit's branch is 18 of 18; the unfinished branch is red on two generated-file tests only | CI | CONFIRMED | any new commit on any of them |
| FABLE-004 | The `PATH` binary is from 2026-08-07; plain `--version` cannot tell it from a fresh build; a build's hash does not identify a later rebuild | executed | CONFIRMED | reinstall; `--version` changes |
| FABLE-005 | Production Rust is about 114,400 lines of 209,666 | executed (script) | CONFIRMED | any Rust change |
| FABLE-006 | The artifact verifier does not re-establish parser guarantees; the `val` write rule is skipped on a type-lookup miss | source; other-party run | CONFIRMED (source) | `dir.rs`, `rcap_check.rs`, `ast.rs`, `parser.rs` |
| FABLE-007 | `recovered` is honoured by the replay and not required empty | source | UNVERIFIED at run time | `check.rs` 1843, `dir.rs` |
| FABLE-008 | Per-site loader tables are keyed by artifact-supplied ids | source | UNVERIFIED at run time | `plugin.rs` 559, `interp.rs` 2129–2196, `check.rs` 301–311 |
| FABLE-009 | `why` on a `.dpx` does not verify | source | CONFIRMED | `cli.rs` |
| FABLE-010 | `fuzz_one_dir` never calls `verify` | source | CONFIRMED | `fuzz.rs` |
| FABLE-011 | 329 checker tests and 50 plugin and signing tests pass; none covers id, `recovered` or identifier mutation | executed; source | CONFIRMED | those test files |
| FABLE-012 | A report sink that fails after the run yields "usage", exit 2, with the effect committed (directory, and missing parent) | executed, five machines; source | CONFIRMED | `run_cmd.rs` |
| FABLE-013 | Every CLI panic exits 2; the documented meaning of 2 is "nothing was examined" | source; executed (closed pipe, both engines, five machines) | CONFIRMED | `main.rs`, `docs/for-agents.md` |
| FABLE-014 | A sandboxed checker refusal writes no report; the in-process one writes a report without the diagnostic | executed, five machines | CONFIRMED | `run_cmd.rs`, `guest.rs` |
| FABLE-015 | `fix`, `test` and `schema validate` each disagree between exit code and outcome; `authority` accepts unknown option values | executed, five machines; source for `test` and `schema` | CONFIRMED | `cli.rs`, `fix.rs`, `schema.rs` |
| FABLE-016 | `.solulu`: loose files work except `fmt`; packages do not see it; one mixed package checks clean with its entry unread | executed, five machines | CONFIRMED | `package.rs`, `deps.rs`, `cli.rs` |
| FABLE-017 | No arguments, environment or JSON; standard input exists on the interpreter only; `main` returning `Err` exits 0 | executed; source (primitive table) | CONFIRMED | `prim_table.rs`, `check.rs` |
| FABLE-018 | The JSON task passes 17 of 18 oracle cases identically on three modes and five machines | executed, independent oracle | CONFIRMED | either engine, `prim.rs` |
| FABLE-019 | The pinned SHA builds and runs natively in WSL2 Ubuntu 20.04, sandbox included; a build from a Windows-mounted path fails | executed (WSL) | CONFIRMED | toolchain, `Cargo.lock` |
| FABLE-020 | The test repository has a `v1.0.0` tag and no release assets; a dry run keeps no archive | executed; source (`release-build.yml`) | CONFIRMED | a release |
| FABLE-021 | The locked Wasmtime is the newest patch of its line on 2026-11-14 | primary source (GitHub API) against `Cargo.lock` | CONFIRMED | any new Wasmtime release or advisory |
| FABLE-022 | Adoption does not compare a certificate's subject with the adopter; a certificate issued to another key is adopted and usable | executed (two OS accounts; same user); source | CONFIRMED | `cert.rs`, `cli.rs` adopt path, RFC 0001 |
| FABLE-023 | Custody refusals under strict mode — unsigned root, replay, scope, effect, expiry, revocation, tamper, foreign anchor — and the OS boundary between two accounts | executed (two OS accounts, Linux x86-64) | CONFIRMED for that platform | `delulu-broker`, `brokerd.rs` |
| FABLE-024 | `delulu test` costs about one second on Windows only; cause is the pipe-retry window | executed (two Windows machines against three others); source | CONFIRMED | `broker_transport.rs`, `cli.rs` |
| FABLE-025 | A default-feature binary needs the build machine's `libpython` to start; the Python-free build does not | executed (runner, two accounts; `ldd`, `otool`) | CONFIRMED | `Cargo.toml` features, PyO3 |
| FABLE-026 | `delulu mcp` is legacy-era, answers `2025-06-18` to any version, and ends silently on invalid UTF-8; its tool list is 229–249 kB | executed, five machines; primary source for the specification | CONFIRMED | `mcp.rs`; a new MCP revision |
| FABLE-027 | The null device answers `COMMANDED`; the report has no device section | executed (laptop) | CONFIRMED | `device.rs`, `run_cmd.rs` |
| FABLE-028 | `delulu edit` refuses a stale hash, writes a clean edit, and holds back a widening one, as documented | executed (laptop), four cases | CONFIRMED | `edit.rs` |
| FABLE-029 | The portable archive builds and installs by `INSTALL.md`'s own block on four targets | CI (run `43508844764`, four logs read) | CONFIRMED in CI | `package-toolchain.sh`, `INSTALL.md` |
| FABLE-030 | Sandbox properties differ by platform as §6 states, and the tool reports each difference itself | executed, five machines | CONFIRMED | `guest.rs`, `jail.rs`, `identity.rs` |
| FABLE-031 | A lease token on the command line is readable from the process table while the run is alive; a spent single-use token is refused on replay | executed (WSL, run confirmed alive) | CONFIRMED | `run_cmd.rs` lease handling |
| FABLE-032 | `scripts/package-toolchain.sh` fails under Python 3.8 at the notices step, after building the release binary | executed (WSL) | CONFIRMED | `scripts/third-party-notices.py` |
| FABLE-033 | The Unix file-system walk opens each component with no link followed and reads or writes regular files only, as its header says | source (`beneath.rs` 196–350); executed black-box on four systems | CONFIRMED for Unix | `beneath.rs`, `prim.rs` |
| FABLE-034 | In release profile the 2,000-item JSON task takes 165 ms (interpreter) and 71 ms (WASM) against 30 ms for Python's `json`; start-up is 3–4 ms | executed (WSL), oracle-checked | CONFIRMED for that machine | either engine |
| FABLE-035 | The test repository's `master` is unprotected with no rulesets; private vulnerability reporting is enabled; one code owner | executed (GitHub API) | CONFIRMED | repository settings |
| FABLE-036 | Twelve network requests that should be refused are refused before delivery, special-use addresses in four spellings among them; on loopback with a scratch authority, an allowed HTTPS request is delivered on three execution modes, an untrusted certificate and a redirect out of the granted path are not, and the server's log agrees | executed (laptop: refusals; WSL: the allowed path, eight cases) | CONFIRMED for those cases; DNS rebinding and cross-host redirects NOT RUN | `egress.rs`, grant parsing |
| FABLE-037 | The portable archive builds and installs by `INSTALL.md`'s own block on the Windows laptop (release profile), every checksum verified | executed (laptop) | CONFIRMED for Windows x86-64; WSL build blocked by FABLE-032 | `package-toolchain.sh`, `check-install.sh` |

## Appendix B. Evidence index (sanitised)

Raw outputs are on the owner's laptop, outside the repository, under `D:\nelan\fable-audit-evidence` (index files
`INDEX.tsv` and `INDEX.json` there list every file with its full hash). They are not in git and are not
recoverable from another machine; the runner logs can be fetched again from GitHub while the runs are retained.
A session elsewhere must treat this document's statements as historical and re-run. SHA-256 prefixes:

| Id | What | SHA-256 (first 16) |
|---|---|---|
| A1, A2 | Survey check, owner's checkout and clean worktree | `6be14980520bb5e7`, `0360acd59ce9b2f3` |
| A3 | pinned binary's hash record | `cbb9792c4617e278` |
| A4, A5 | Doctor, clean and dirty | `2b1c858bfe3c2c04`, `2a597100b3b22c9e` |
| A6 | inventory (JSON) | `9ca979836417e4cd` |
| A7 | failed-job logs of run `43389262361` (2.19 MB) | `4a6eb177acb2c23f` |
| A9 | job results of CI run `43506016428` | `8a7f8ea308d2c91a` |
| T1 | checker tests; plugin and signing tests | `58c9ff2a30a30afa`, `9baf96621c5a1d1a` |
| T2 | laptop cases b/d, e2, h, i1, j, l2 | `f545c0b18607224c`, `f8c3af2166f1da93`, `707efddd99064a0f`, `4c1f677ee157ebbc`, `c88499c57ac7b499`, `b4b69985f3993c25` |
| T3 | laptop results on interpreter, WASM, sandbox; the oracle; the program | `e5d9eedd2e1cb139`, `bfac22f50e0216c8`, `3245e5947404871f`; `36d8b22d157d7f31`; `ae9b42de6c1f269f` |
| T5 | two-principal matrix, same user (laptop); two accounts (runner log); the process-table redo (WSL) | `a44bf7b7c4f1e440`; `b8d18df68a340dec`; `079371a61f493b28` |
| T7 | MCP probe (laptop) | `87519a51788703fe` |
| R1, R2, R3 | runner job tables for the three rounds (each round's per-platform logs are in the private index) | `c9c4c6320fd34dce`, `53901d1d218b0aa1`, `7a2f26c90c535672` |
| F1, F2 | WSL verification transcript; release dry-run summary | `1f5ea3c13bcd50cf`; `ff7fb25ec86dd8c4` |
| F3, F4, F5 | WSL packaging failure; WSL release timings; Windows packaging and the install block's transcript | `850d2e6151f06461`; `e5ed5f3fc7a42e6c`; `5ac828a055bbfe2a`, `24e18aa13777d820` |
| G1, G2 | network refusals (laptop); the loopback HTTPS control (WSL) | `f2e67808b38682de`; `24f679c4944c0e31` |
| X1–X7 | external sources as fetched: A2A releases and specification, OpenShell and Wasmtime releases, Wasmtime advisories, MCP changelog and versioning page | `41ec6e2cf7c0187e`, `e3704e45b19da780`, `45cb8ee480688583`, `6b64dc3ca763c92c`, `b327e576f4c96b34`, `bc02f271700bdecd`, `627ccfe6ffb1be2c` |
| H1 | the misdirected build's log | `6117ae2f4ad7e687` |
