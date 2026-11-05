# DeluluLang: hidden complexity and the AI-first experience, an audit

**Written:** 2026-11-05, by Claude (a Claude Code cloud session, the head chef; no sub-agents ran). This is the fourth
document from the same session, after the independent strategic verification and the AI-first compiler and
execution-pipeline audit (both in this folder).

**The brief:** `docs/prompt/DeluluLang_Claude_Code_Hidden_Human_Centric_Complexity_Audit_Prompt.md`, the owner's prompt,
kept byte for byte. It asked for a search for human-centric assumptions and avoidable costs across the whole path from
an AI's first command to a structured outcome. Every conclusion was to be measured or witnessed on the current code,
and any hypothesis the evidence did not support was to be dropped.

**Scope:** a research and documentation change. No `.rs`, test, workflow, manifest or entrenched file changed. Every
removal or redesign below is a recommendation for a separate, witnessed change.

**Second pass (later the same day; §11).** The owner asked for everything to be double-verified. Three Sonnet 5.5
sub-agents re-ran, reviewed and researched; the head chef re-ran every finding they reported before it reached this
file. The corrections below were made in place and are each listed in §11.4:
- the sandbox does not protect against item 5; it triggers it;
- every peak-memory figure;
- MCP `2026-07-28`'s status;
- the package run ratio;
- the Deno cache's key.

**A later correction (2026-11-05, the laptop, on the owner's request).** §11.1's reading of `master`'s red Windows run
(RW 7.25) went further than its evidence. It is corrected beside the claim, and RW 7.25 states what that run does and
does not show.

**How to read it:**
- **OBSERVED** means it ran here, with the command in §5 or Appendix A.
- **SOURCE** means it was read in the code at the cited line.
- **INFERENCE** means it is reasoned, not run.
- **EARLIER** means it was measured by the 2026-11-05 audit on the same binary and is cited, not re-run.

---

## 1. Executive verdict

**The short answer to the owner's suspicion.** The suspicion is half right.
- **Where it is wrong:**
  - The per-command machinery is lean. A `check` makes about 170 system calls.
  - A pure task touches no keys, crypto, plugins, audit log or broker.
  - A non-terminal agent is never left waiting on a prompt.
  - The compiler's own work on a small file is well under a millisecond once a process is warm.
- **Where it is right:** the costs that slow an AI down are not milliseconds of compiler work. They are:
  - answers that are wrong in the fast path;
  - refusals that come after the program has already acted;
  - artifacts that build and then cannot run;
  - interfaces that hand an agent megabytes, or cannot be interrupted.
- **What those costs have in common:** most come from one habit. A component built for one file or one human session
  was reused where the work is a package, a long-lived agent or a constrained host.

### 1.1 The ten most consequential hidden costs, most serious first

| # | What | Evidence | Status |
|---|---|---|---|
| 1 | **`p.get` does not enforce rule R-Get** (RW 4.95, HIGH). A program whose row is `! {Load, Read}` still ran a plugin's `! {Write}` export, which wrote a file. | OBSERVED again on this binary (§5.5) | Still open. **The P0 blocker** |
| 2 | **The warm path is package-blind.** `delulu lsp` checks each open file alone (`lsp.rs` line 204, `check_source(0, &self.text)`). Opening the shipped `examples/greeter/src/main.delulu` publishes two false `DL0301 unknown name` errors, though `delulu check examples/greeter` is clean. `docs/for-agents.md` recommends exactly this server for an edit loop, so for any multi-module program the fast answer is wrong. | OBSERVED (§5.2) | **New: RW 6.72** |
| 3 | **Grants are refused late, one at a time, after effects have happened.** `authority` computes every required grant (`required_grants`), but `run` grants lazily. A program granted `fs.write` but not `console` **wrote its file, then was refused** at `console`. Each missing grant costs a round trip. | OBSERVED (§5.3) | **New: RW 6.73** (a decision) |
| 4 | **`build --target wasm` writes artifacts that can never run.** The WASM host always calls `main` with one argument and no result (`host.rs` line 3018). `fn main() { … }` and `fn main() -> Int` check clean, run on the interpreter, and build to a `.dwx` with exit 0. Every run of that `.dwx` is then `DL0904 expected 0 arguments, got 1`, Wasmtime's own words. | OBSERVED (§5.4) | **New: RW 3.28** |
| 5 | **The recursion bound depends on a 781 MiB virtual reservation.** On a host that caps address space (`ulimit -v`, which some containers and devices use), the reservation fails and the CLI falls back to the 8 MiB main stack. A recursion that is `DL0905` elsewhere then **aborts the process (exit 134) and writes no run report**. *Corrected in §11:* the sandbox's own memory ceiling triggers it. Under `--sandbox-profile hostile-agent` (256 MiB), the profile recommended for untrusted code, or `--sandbox --limits mem` up to 768 MiB, the guest aborts and the report says only `{exit 1, ran}`. | OBSERVED (§5.6, §11.2) | **New: RW 3.29** |
| 6 | **The MCP channel cannot be interrupted, and its output is unbounded.** It answers `notifications/cancelled` by ignoring it: a cancelled 0.5 s call still answered, and a `ping` waited behind it. One noisy file produced a **3.4 MB** reply, 60% of it a pretty-printed copy of the structured result. No tool declares an `outputSchema`, though `delulu schema` holds the schemas. | OBSERVED (§5.7) | **New: RW 6.74**. RW 6.62 is amended with the measurements |
| 7 | **`run --json`'s standard output has four shapes.** It carries the program's output on success and an envelope on a refusal or fault. A budget stop leaves it empty (the report says `stopped_by`); a signal leaves it empty and writes no report. A handled application error exits 0, like success. | OBSERVED (§5.3 table) | RW 6.60 amended. RW 6.68 and 2.40 still hold |
| 8 | **`edit` inside a package is blind both ways.** Under `--if-checks` it wrote a `Net`-adding edit that the package then refuses (DL1009). It also falsely refused an edit that uses an imported name (`DL0301 salutation`). | OBSERVED (§5.5) | RW 6.69 amended |
| 9 | **Every command pays for things it does not use.** libpython: **1.35 ms and about 4.2 MiB per command** (100 interleaved runs). Cranelift: a `.dwx` saves nothing over source, 8.08 ms against 8.36 ms on a tiny program. A package run checks the program twice, costing about 2× a `check` (1.8–2.1× over five measurements). | OBSERVED (§5.1, §5.8, §5.2) | RW 3.26, 3.27 still hold. The double check is §4's item R4 |
| 10 | **Discovery is one 64.5 KB object.** `toolchain --json` has no section selector: 24 KB of it is usage prose and 13 KB the diagnostic table, and 32% of its bytes are indentation. An unknown command dumps the 22.5 KB usage text on stderr. Whether this build has Python appears only in `doctor`'s prose. | OBSERVED (§5.1) | Recommendations P3-1 to P3-3 (no row) |

### 1.2 Hypotheses this audit disproved, or found not worth acting on

| Hypothesis (the prompt's §4 and §5) | Finding |
|---|---|
| Pure tasks pay for crypto, keys, plugins or audit setup | **Disproved.** A `strace` of `check`, `run` and `authority` showed 170–176 system calls and 13–15 opens. Those opens are the source, shared libraries, `/proc/self/maps`, two missing config files and (for `run`) `sandbox_policy.json`. Nothing else is touched. |
| A hidden prompt blocks an agent | **Disproved for non-terminals.** With stdin at `/dev/null` and no `--no-prompt`, a missing grant is DL0703 in 9 ms. The first-run locale picker appears only on a terminal with `DELULU_NO_FIRST_RUN` unset. |
| An incremental query engine (Salsa-style) is needed | **Not now.** The warm server re-checks a small file in **0.21 ms** after an edit. A whole 1,482-line, 13-module package checks in **31 ms** cold, and a body edit and an effect edit cost the same. Model latency is seconds. What is missing is *package awareness* in the warm path (item 2), not memoization. Salsa itself is not used directly in rustc (rustc-dev-guide), and is at 0.28.5. |
| MCP's process per call is its main cost | **Disproved.** The child costs about 3.5 ms. The rest is a 5 ms polling sleep (`mcp.rs` line 400), which quantizes latency: 5.6 ms for a tiny check, 26.0–26.3 ms where the CLI takes 24 ms. The process per call also gives crash isolation and makes cancellation a `kill`. **Keep it; fix the poll, the bound and cancellation.** |
| `--engine wasm` silently falls back to the interpreter | **Disproved.** An unsupported construct is DL1201 before anything runs; the report says `ran: false`. The report does not name the engine that ran, though (P3-4). |
| libpython's cost comes from lazy symbol binding | **Disproved.** `LD_BIND_NOW=1` leaves both the time and the memory gap unchanged. The cause is not established. |
| A parser-free IR would help the agent loop | **Not revisited.** The earlier audit measured parsing at 17% of the front end and DIR at 28× the text's size; nothing here overturns that. |

---

## 2. Current state and evidence ledger

| Item | Value |
|---|---|
| Repository | `origin/master` = `35f1a2d` (the third verification pass, fast-forwarded). `git diff 0670e0b..35f1a2d` touches no `.rs`, `Cargo.*` or `*.toml`, so the code is that of `0670e0b` |
| Other branches | `origin/autosave/laptop` (`ea8a903`, 06:27 UTC) adds only the prompt already in `docs/prompt/`. `launch/2.0.0`, `autosave/laptop-dl-launch`, `autosave/dogfood` and `rc/1.0.0-drill` are older and unchanged. Nothing to recover |
| Binary | A release build of `0670e0b` with `DELULU_BUILD_COMMIT` injected (`--version --json` shows the commit), SHA-256 `751c6458a8cedef3…`, 41,768,296 bytes, glibc, linking `libpython3.13.so.1.0`. **The binary every finding of the earlier audit was measured on** |
| Comparison binary | The same tree built without default features (no Python; `commit: null`), 41,652,600 bytes |
| Platform | Linux 6.18.44 x86-64, 4 vCPU, 15 GiB, Ubuntu 24.04 container. No Windows, macOS, ARM or KVM was available, so nothing here is evidence for those |
| Isolation | Every probe ran in scratch with `DELULU_HOME`, `DELULU_STATE_DIR` and `DELULU_NO_FIRST_RUN=1` set. §5.9 shows that this does not isolate everything |
| Timing | A Python harness forks and `exec`s the binary and reads peak RSS from `wait4`. It adds about 1 ms against a shell loop (`--version`: 4.2–4.5 ms in the harness, 3.5 ms in a 200-run `bash` loop). Medians, p10/p90 and the standard deviation are given. Comparisons are interleaved A/B, so drift cancels |
| Survey | `survey check`: *ok: the Survey matches the tree (1624 nodes, 19328 edges)* |
| doctor | `doctor --check`: *ok: 26 check(s) passed* (its count varies by machine) |
| CI | Master's push run [41990062363](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/41990062363) on `35f1a2d`: **success**. The branch's PR run [41988110890](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/41988110890): **success**. There was no red job to fix |

### 2.1 Earlier claims re-checked on this binary

| Claim | Now |
|---|---|
| RW 4.95, R-Get unenforced | **Holds.** `check` clean, `authority` effects `[Load, Read]`, the plugin wrote `out/written-by-plugin.txt` (§5.5) |
| RW 6.69, `edit` blind to packages | **Holds, and is wider:** it also refuses correct edits (§5.5) |
| RW 3.26, libpython per command | **Holds.** 1.35 ms median on 100 interleaved runs of each of `--version` and `check` (the row said 1.1–2.0 ms); peak RSS +4.1 to +4.3 MiB (C launcher, §11.4) |
| RW 6.62, MCP per-call process | **Holds,** with the poll quantization and output sizes now measured (§5.7) |
| RW 6.61, no target preflight | **Holds.** `check --target` does not exist (exit 2). The `.actuator` refusal on WASM comes at run time (§5.6) |
| RW 6.60 and 6.68, run outcomes | **Hold.** A third and fourth stdout shape found (§5.3) |
| RW 2.40 and 2.41, no argv, exit code or JSON | **Hold.** The structured-data task in §5.3 had to hand-roll both |
| RW 4.93, 4.94, 6.67, 6.70, 6.71 | **Not re-run.** The binary is byte-identical to the one that witnessed them earlier today (same SHA-256), and no code changed |

**Unknown:**
- Every Windows, macOS, ARM and real-device behaviour.
- Strict overcommit (`vm.overcommit_memory=2`): the container cannot set it, so whether item 5 also fires there is INFERENCE.
- Token counts. Bytes are measured; tokens depend on the client's tokenizer and were not invented.

---

## 3. The end-to-end friction map

What an AI does to finish one task, what each step costs here, and whether it needs to understand the step.

| Stage | Command or interface | Measured cost | Necessary? | What the AI should need to know |
|---|---|---|---|---|
| Discover the toolchain | `toolchain --json`, or MCP `toolchain` | 64,536 B (44,094 compact); one process | Yes, once | Only the part it needs: one section (grants, effects, codes) |
| Discover examples | `examples --json` | 29,017 B | Optional | — |
| Discover features (Python, microVM, OS) | `doctor --json`, prose detail only | 5,508 B | Yes, before planning | A typed `features` object (P3-3) |
| Write and check | `check f --json` (fresh) or LSP `didChange` | **5.0 ms / 0.21 ms** median | Yes | Diagnostics with typed repairs. Shown working: §5.3's first error carried an exact `brace-match-arm-assignment` repair |
| Check a package | `check pkg --json` | 31 ms for 1,482 lines / 13 modules | Yes | **The warm path gives wrong answers here** (item 2) |
| Authority | `authority f --json` or LSP `delulu.authority` | 5 ms / **0.11 ms**; 927 B | Yes | `required_grants`: exactly the grants to pass |
| Target preflight | — (does not exist) | n/a | **Missing** | Which engine, sandbox level and OS can run it (RW 6.61) |
| Request grants | `--grant …` on `run` | 0 | Yes | A refusal that names **every** missing grant before anything runs (item 3) |
| Run | `run f --json --no-prompt --report-out r.json` | 6–7 ms tiny (interpreter); WASM +3 ms tiny, −95% on a hot loop | Yes | One outcome object (item 7) |
| Read the outcome | `--report-out` file plus stdout | 1 file, 4 stdout shapes | Yes | A single envelope; the program's output as a field |
| Edit and re-run | `edit --expect-hash … --if-checks` | 1 process | Yes | A package-level delta (item 8) |
| Test | `test --json` | 4 ms | Yes | That zero tests ran (exit 0, `passed: 0`) |
| Distribute | `build --target wasm`, `.dwx` | 5.3 ms build; Cranelift on every run | Optional | That the artifact runs (item 4) |

**A complete task, counted** (§5.3: a CSV validation utility that reads a file and prints a JSON summary).
- **The informed path:** `check` → apply the exact repair → `check` → `authority` → `run` with `required_grants`.
  That is **5 tool calls, about 3.7 KB of tool output and about 25 ms of tool time**, plus one optional 64.5 KB
  discovery call. One correction was needed.
- **The naive path:** `run` → DL0703 → add a grant → `run` → … costs one extra run per missing grant. It can leave
  partial effects behind (§5.3).
- **What the AI still has to work around:**
  - no JSON parser or writer (hand-escaped strings, RW 2.41);
  - no argv (the path is hard-coded, RW 2.40);
  - no exit code: the "input missing" outcome exits 0 like success, and the only signal is the program's own text.

**Things an AI should not have to understand:**
- the 781 MiB stack;
- the flattening of packages for the runner;
- that the MCP server shells out to the CLI;
- which build features were compiled in.

**Things it must keep seeing (and does):**
- every refusal, with its code;
- the authority report;
- the sandbox level, mode and limits in the report;
- `requested_computed: true` when a scope is dynamic (§5.5);
- the Guard's decisions.

Nothing in this audit argues for hiding any of them.

---

## 4. Path-by-path inventory

Decisions:
- **Keep:** keep as it is.
- **Opt:** keep optional.
- **Lazy:** make lazy.
- **Cache:** cache the result.
- **Cons.:** share or consolidate.
- **Fix:** keep, and correct a defect.
- **Defer:** decide later.

*Survey* is `delulu-survey impact <id>`. Inside `crates/delulu` every module reports 106 nodes (RW 6.63), so those
numbers say "the CLI crate" and nothing finer.

| ID | Path | What it costs or breaks | Decision | Risk / fallback | Survey |
|---|---|---|---|---|---|
| R1 | `delulu-runtime/src/interp.rs` 1902–1919 (`plugin_get`); `plugin.rs` 583–668 (`r_get_verified`, unused) | R-Get unenforced (item 1) | **Fix (P0)** | The checker must carry the annotated `F` into the program; DIR too. Fallback: refuse `p.get` whose `F` cannot be compared | `mod:…/plugin.rs` 182, `mod:…/interp.rs` 182 |
| R2 | `delulu/src/lsp.rs` 199–207 `Doc::analyze`; no `delulu.toml` handling anywhere in `lsp.rs` | False errors on every multi-module program (item 2) | **Fix (P1)**: analyze a file under a manifest through `resolve_workspace` / `check_workspace`, with open documents overlaid | A loose file keeps today's path. If a large package is slow per change, memoize parsed modules by content hash: a small cache, not Salsa | 106 |
| R3 | `delulu/src/edit.rs` (`--if-checks`, the authority delta) | Blind both ways (item 8) | **Fix (P1)**: the same package resolution as R2 | — | 106 |
| R4 | `delulu/src/run_cmd.rs` 84 (`check_workspace`), 117 (`check_source` of the flattened text) | A package run checks twice: 61 ms against `check`'s 31 ms (about 2×) | **Defer**: the second check is what the runner executes (D61). Removing it needs per-module resolution in `Interp` (`Program::call_owner`) | Keep until R2 has landed and a profile says it matters | 106 |
| R5 | `delulu/src/run_cmd.rs` grant handling (1306–1316); interpreter grants at first use | Late, one-at-a-time refusal; partial effects (item 3) | **Fix (P1)**, additive first: name every statically known missing grant in the refusal. Opt-in pre-execution refusal. Default is the owner's decision (§9) | A computed scope (`requested_computed`) cannot be pre-checked and stays lazy | 106 |
| R6 | `delulu-wasm/src/host.rs` 3015–3018 (`main.call(&[Val::I32(0)], &mut [])`); `codegen.rs` | Unrunnable `.dwx` (item 4) | **Fix (P1)**: refuse at build with DL1201 for any `main` other than `fn main(root: Root)` returning `Unit`, or generate a wrapper | — | `mod:…/host.rs` 134 |
| R7 | `delulu/src/main.rs` 82–102; `interp.rs` 33, 56, 90 (`INTERPRETER_STACK_BYTES` = 10,000 × 80 KiB + 256 KiB) | A fault becomes SIGABRT on an address-space-capped host (item 5) | **Fix (P1)**: when the thread cannot be spawned, lower the depth bound to what the main stack can hold, or refuse to run programs with a note, never run unbounded | — | 105 |
| R8 | `delulu/src/mcp.rs` 367–410 `run_self`; 455–510 the loop | Poll quantum, no cap, no cancellation, head-of-line blocking (item 6) | **Fix (P1/P2)**. Keep the process per call. Read stdin on its own thread; on `notifications/cancelled`, kill the child and send nothing. Wait without polling. Cap the bytes and say so; send the text block compact; declare `outputSchema` from `delulu schema` | Keep the 120 s deadline (line 35) | 106 |
| R9 | `delulu/src/run_cmd.rs` 437–466 `stop_for_budget` (`process::exit(1)`); no signal handler | Stdout empty on a budget stop; no report on a signal (item 7) | **Fix (P1)**: `--report-out` is the one outcome contract, so write it on a signal too (RW 6.68). Under `--json`, put an envelope on stdout for a budget stop as for a refusal | — | 106 |
| R10 | libpython linked at load time (default feature); `delulu-runtime/src/python.rs` 110–115 | 1.35 ms and about 4.2 MiB per command (item 9) | **Lazy (P2)**: `dlopen` on the first `root.python`, or a Python worker process. Keep the feature | A missing libpython stays `ForeignErr::Unavailable` (DL1307), as in the no-Python build today | — |
| R11 | Cranelift on every WASM and `.dwx` run; `verified_executable_wasm` unused (RW 3.27) | About 3 ms on a tiny program, hundreds of ms on large ones (earlier audit: 1.6–652 ms) | **Cache (P2)**: see §6.3 for the trust rules | Any miss or mismatch recompiles | 134 |
| R12 | `delulu/src/toolchain.rs` 20–37, 40–60 | One 64.5 KB object; usage prose inside it | **Opt (P3)**: `toolchain --json <section>`; keep the whole object as the default | Additive | 106 |
| R13 | `cli.rs` 1123–1155 (`to_string_pretty` everywhere) | 32% of `toolchain --json`'s bytes are indentation (pretty is 1.46× compact) | **Defer**: measure tokens with the real client tokenizer first. Whitespace runs tokenize cheaply, so the token saving is unknown | — | 106 |
| R14 | `cli.rs` unknown-command path | 22.5 KB of usage text on a typo it cannot match | **Fix (P3)**: name the command list, not the whole usage | — | 106 |
| R15 | `cli.rs` 148–157 (`theme.toml`), `locale.rs` 72–79 (`config.toml`): `$HOME/.delulu`, not `DELULU_HOME` | `DELULU_HOME` does not isolate a job's human-facing configuration; `for-agents.md` says it holds "locales" | **Fix the document now** (done in this change); make the code follow `DELULU_HOME` later (P3) | JSON is locale-invariant (invariant 39), so the machine channel is unaffected | 106 |
| R16 | The sandbox's operator notes on stderr | 4–5 lines per sandboxed run | **Keep**: an operator's view, and the report carries the level | — | — |
| R17 | Guard, broker, custody, leases, audit | An ordinary permitted embedded run opens only `sandbox_policy.json` | **Keep**: independent enforcement layers. No overhead was measured on the pure or permitted path | — | — |
| R18 | Morph skins, locales, themes, colour | Two `stat` calls per command; JSON untouched | **Keep / Opt**: RW 6.70 (the skin found through the working directory) remains the defect | — | — |
| R19 | Interpreter and WASM, both | Interpreter wins on tiny programs (5.4 against 8.4 ms), WASM on hot loops (33 against 663 ms) | **Keep both**; no automatic mode until parity holds (RW 3.25, 3.28) | — | — |
| R20 | Survey, doctor, Atlas, conformance, formal models | Developer-side; `mcp` calls `find_source_tree` once at start-up | **Keep**: no cost on an agent's task path was found | — | — |

---

## 5. Benchmarks and task-completion evidence

All OBSERVED on the ledger's binary. Programs and commands are in Appendix A.

### 5.1 Fresh process, warm server, MCP (E1)

40 edit-then-check cycles of a three-line program, each kind of client:

| Path | Median | p10 | p90 | max | σ | Memory |
|---|---|---|---|---|---|---|
| Fresh `check --json` | 5.00 ms | 4.52 | 6.37 | 11.33 | 1.17 | 12.3 MiB peak (corrected, §11.4) |
| Fresh `--version` | 4.24 ms | 3.87 | 5.01 | 7.84 | 0.73 | — |
| LSP `didChange` → `publishDiagnostics` | **0.21 ms** | 0.18 | 0.28 | 0.38 | 0.04 | server 14.0 MiB high-water; start 3.7 ms |
| LSP `delulu.authority` | 0.11 ms | — | — | — | — | — |
| MCP `tools/call check` | 5.64 ms | 5.58 | 5.78 | 22.76 | 2.77 | server 12.0 MiB |

All 120 answers were correct (0 errors on a clean program).

**MCP latency against the size of the check** (15 each):

| File | CLI median | MCP median | What it shows |
|---|---|---|---|
| tiny | 5.18 ms | 5.63 ms | The ±0.1 ms band is the 5 ms poll (`mcp.rs` line 400) |
| 500-line program | 24.4 ms | 26.0–26.3 ms | Rounded up to the poll |
| 2,000-line program | 112 ms | 108 ms | Noise dominates |
| 8,000-line program | 501 ms | 504 ms | Noise dominates |

**libpython, interleaved A/B, 100 runs each:**

| Command | With Python | Without | Peak RSS with / without |
|---|---|---|---|
| `--version` | 4.54 ms | 3.19 ms | 9.5 / 5.3 MiB |
| `check tiny --json` | 5.17 ms | 3.83 ms | 12.3 / 7.9 MiB |
| `explain DL0501` | — | — | 10.1 / 5.9 MiB |

With `LD_BIND_NOW=1` the two builds differ by the same amount. *Peak RSS corrected in §11.4:* the first figures came from a
Python launcher, whose own high-water mark `exec` carries into the child's `ru_maxrss`. These are from a C fork-and-exec
launcher, 15 runs each.

**Discovery bytes:**

| Command | Bytes |
|---|---|
| `toolchain --json` | 64,536 (commands 24,091; diagnostics 13,198; primitives 5,112; grants 2,824) |
| `examples --json` | 29,017 |
| `--help` | 22,520 |
| `schema --json` | 1,821 |
| `explain DL0501 --json` | 776 |
| unknown command `capabilities` | 22,559 on stderr |
| near miss `chek` | 129 (*did you mean `check`?*) |

### 5.2 A package: invalidation and run cost (E2)

Generated: 12 modules × 120 chained functions plus an entry importing all of them; 1,482 lines. 15 runs each:

| Command | Median | p90 | Peak RSS |
|---|---|---|---|
| `check pkg --json` | 30.7 ms | 33.6 | 25.9 MiB |
| `authority pkg --json` | 30.5 ms | 39.5 | 26.5 MiB |
| `check` after a body edit in one module | 31.3 ms | 33.1 | — |
| `check` after adding `! {Clock}` to one function | 30.7 ms (exit 1, 2 errors) | 33.8 | — |
| **`run pkg`** | **61.2 ms** | 63.9 | **42.1 MiB** |
| `check` of the same functions as one file | 26.3–30.5 ms | — | — |

**What the numbers show:**
- Nothing is reused between commands.
- An edit that changes nothing outside one function body costs the same as an edit that changes a signature.
- A package run is two checks, the workspace's and the flattened program's, plus the execution.

**The warm server on a package:** opening `examples/greeter/src/main.delulu` gave
`DL0301 unknown name salutation` and `DL0301 unknown name shout_count`. `src/greetings.delulu` gave 0 diagnostics, and
`check examples/greeter --json` gave 0 errors.

### 5.3 A complete structured-data task, and every outcome (E3)

**The task:**
- read `./data/people.csv`;
- validate each row (2 columns, a name, an integer age);
- print a JSON summary with the valid count, the age sum and each invalid line's reason;
- report "input missing" distinctly.

**What the agent did (this session, acting as the agent):**
1. Wrote `util.delulu`, 45 lines.
2. `check`: 2 errors at line 38 (a match arm assigning without braces). The first carried an exact, typed repair:
   `brace-match-arm-assignment`, `authority_widening: false`, `requires_human: false`.
3. Applied it. `check`: clean.
4. `authority`: `required_grants: ["fs.read=./data", "console"]`.
5. `run` with those grants. Correct output: 2 valid, age sum 121, line 2 *"age is not an integer"*, line 4 *"empty name"*.

**The outcome taxonomy, every row a real run** (`--json --no-prompt --report-out`):

| Outcome | Exit | Stdout | Report `outcome` | Distinguishable? |
|---|---|---|---|---|
| Passed | 0 | the program's text | `{exit 0, ran}` | Yes |
| Handled application error (input missing) | **0** | the program's own `{"ok":false,…}` | `{exit 0, ran}` | **Only by parsing the program's text** (RW 2.40) |
| Expected refusal (no grants) | 1 | envelope, DL0703 | `{exit 1, ran: true}` | Yes |
| Refusal after a partial effect | 1 | envelope, DL0703 for `console` | `{exit 1, ran: true}` | **The file it had already written is not in the report** |
| Unsupported construct on WASM | 1 | envelope, DL1201 | `{exit 1, ran: false}` | Yes |
| No tests executed | **0** | envelope, `passed: 0, failed: 0` | — | Only by reading `summary.passed` |
| Runtime fault (division by zero) | 1 | envelope, DL0902 | `{exit 1, ran}` | Yes |
| CPU budget spent | 1 | **empty** | `stopped_by: {dimension: cpu, …}` | From the report only |
| Wall budget spent | 1 | **empty** | `stopped_by: {dimension: wall}` | From the report only |
| Cancelled (SIGTERM) | −15 (killed) | empty | **no report** | Only as "the report is missing" (RW 6.68) |
| `--report-out` into a missing directory | 2 | envelope, `error.kind: usage` | none (cannot be) | Yes. The words *"a dangling link on the way?"* mislead for a plain missing directory |
| Missing source / bad grant | 2 | envelope, `error.kind: usage` | `{exit 2, ran: false}` | Yes |

**The partial effect:** `partial.delulu` writes `out/step1.txt`, then calls `root.console()`. Run with
`--grant fs.write=./out` alone, it exits 1 with DL0703 for `console`, and `out/step1.txt` exists. `authority` had
listed both grants before the run.

### 5.4 Engines (E4) and the `main` signature

| Program | Interpreter | `--engine wasm` | `.dwx` | Peak RSS (interpreter / WASM) |
|---|---|---|---|---|
| Hot loop, 3,000,000 iterations (7 runs) | 663 ms | **33.0 ms** | 33.1 ms | 13.8 / 22.6 MiB |
| Same, 10 iterations (21 runs) | **5.37 ms** | 8.36 ms | 8.08 ms | 13.7 / 22.6 MiB |
| `build --target wasm` | — | 5.27 ms | — | — |

`fn main() -> Int { 6 * 7 }` and `fn main() { let x = 6 * 7 }`:
- the interpreter exits 0;
- `--engine wasm` gives DL0904 *"expected 0 arguments, got 1"*; `fn main(root: Root) -> Int` gives *"expected 1 results,
  got 0"*;
- `build --target wasm` exits 0 with 0 errors and writes the `.dwx`;
- `run x.dwx` prints *"authority verified; declared effects: (none — provably pure)"* and then DL0904, exit 1.

### 5.5 Authority-sensitive probes (E5)

| Probe | Result |
|---|---|
| Permitted scoped read (§5.3) | Ran; read only `./data` |
| Write to `../escaped.txt` under `fs.write=./out` | DL0904 *"escapes the granted scope"*; **the file does not exist** |
| Scope computed from input (`fs_write(pick(line))`) | `authority`: `requested_computed: true`, `fs.write=PATH`. Input that picks `./out`: written. Input that picks `./other`: DL0703, nothing created |
| RW 4.95 (`esc.delulu` + `putter.dpx`) | `check` clean; `authority` effects `[Load, Read]`; `run --grant plugin=. --grant fs.write=./out` **exit 0, wrote `out/written-by-plugin.txt`**. Still bounded by the grant, as recorded |
| RW 6.69 (`examples/greeter` copy, `edit --node fn:greeter.main --if-checks`) | Adding `Net` and `evil.example.com`: exit 0, `written: true`, `before: null`, `widened: null`; `check .` then DL1009. A body calling the imported `salutation`: refused, DL0301 *unknown name* |
| A revoked lease, delegation | EARLIER (the 2026-11-05 audit, same binary); not re-run |

### 5.6 Edge and robotics conditions (E6)

**Mock arm on the `sim` profile, `actuator=arm0/elbow:angle_deg=0..90,…`:**
- *COMMANDED*, then *REFUSED: angle_deg = 120 is outside the envelope [0, 90]*, then *COMMANDED*, exit 0.
- Under `--sandbox`: the same output, exit 0, report clean.
- `--engine wasm`: DL1201 *"does not support the method `.actuator`"*, at run time. `check --target` does not exist.

**An address-space cap** (`ulimit -v`):

| Cap | `--version`, `check`, `run` | Recursion 100,000 deep | WASM |
|---|---|---|---|
| none | ok | DL0905, exit 1, report `{exit 1, ran}` | ok |
| 2,000,000 KiB | ok | — | DL0904 *"mmap failed to reserve 0x104000000 bytes"*, report written |
| 300,000 KiB | ok (falls back to the main stack) | **exit 134, "thread 'main' has overflowed its stack", no report** | as above |

~~With no cap, `--sandbox` (dev, contained, hostile-agent) and `--limits mem=256MiB` each gave DL0905.~~ **Corrected in
§11.2:** `--limits mem=256MiB` without the sandbox gives DL0905. `--sandbox` under dev or contained gives DL0905 on the
guest's stderr, though the report's `diagnostics` is `[]`. `--sandbox-profile hostile-agent`, or `--sandbox --limits mem`
up to 768 MiB, **aborts the guest** (*"has overflowed its stack"*), and the report says only `{exit 1, ran}`. The first
reading took an exit 1 for DL0905 without reading the guest's stderr. They limit
`RLIMIT_DATA`, not `RLIMIT_AS` (`jail.rs` line 76, `budget.rs` line 12).

**Feature discovery:**
- Both builds list `root.python` in `toolchain --json`.
- On the no-Python build the program checks clean and prints *"embedded Python is unavailable"*, exit 0: a handled
  value, as designed.
- Only `doctor` says *"not compiled in"*, in prose.

### 5.7 The MCP channel under stress (E7)

| Probe | Result |
|---|---|
| A client asking for `2025-11-25` or `2026-07-28` | Answered `2025-06-18` (allowed: the server names one it supports) |
| `tools/list` | 12 tools, 6,854 B, none with `outputSchema` |
| `check` of 4,000 undefined names (one call) | 187 ms. **3,405,418 B reply**: text block 2,042,659 B (pretty-printed), `structuredContent` 1,210,631 B. Server high-water 41.9 MiB |
| 200 such calls | median 183 ms, p90 202 ms, max 234 ms; high-water 47.6 MiB (bounded) |
| Not JSON / no `method` / unknown tool / an option-like argument | −32700 / −32600 / −32602 / `isError: true` with the reason. The next request was answered |
| `notifications/cancelled` 50 ms into a 0.5 s call, then `ping` | The cancelled call **answered with a result at 522 ms**; the `ping` waited and answered at 522 ms |
| The client closes its read end mid-call | The server exited 0, no hang |

### 5.8 What a pure command touches

`strace -f` of `check`, `run` and `authority` on a tiny file:
- 170, 176 and 175 system calls; 13, 15 and 14 opens;
- the opens beyond the source and the shared libraries: `/proc/self/maps`, `$HOME/.delulu/theme.toml` and
  `$HOME/.delulu/config.toml` (both missing), and for `run` `$DELULU_STATE_DIR/sandbox_policy.json`;
- 1 thread created.

### 5.9 `DELULU_HOME` isolation

With `DELULU_HOME` and `DELULU_STATE_DIR` set to scratch, every command still opened `/root/.delulu/theme.toml` and
`/root/.delulu/config.toml`. Those paths come from `HOME` (`cli.rs` 155–156, `locale.rs` 77–78), or from
`DELULU_THEME_FILE` / `DELULU_CONFIG_FILE`. `for-agents.md`'s table said `DELULU_HOME` holds "keys, credentials,
locales". Corrected in this change.

---

## 6. A streamlined architecture and workflow

### 6.1 One entry point, three doors, one service

The CLI, the LSP and MCP already share the compiler crates. What differs is how much of the program each door sees.

**Proposed:**
- The **checker service** is the set of functions behind `check`, `authority`, `atlas`, `explain` and `schema`.
- **Every door resolves the package the same way.** A file under a `delulu.toml` is checked as part of its package, in
  `check`, `lsp`, `edit` and MCP alike.
- **The LSP keeps its per-document cache.** It gains a per-package cache invalidated by any open-document change or
  file `mtime`, as its workspace index already is.
- **MCP keeps its process per call**, for isolation and kill-based cancellation, and fixes its loop (R8).
- **No new framework, database, protocol or service.**

### 6.2 The agent's workflow: from about ten steps to three, without hiding any

1. **`check <file|pkg> --json`.** Diagnostics with typed repairs, plus (proposed, additive) the authority summary and a
   `targets` block. The block says which engines, sandbox levels and OS can run this program, and why not (RW 6.61).
   One call instead of `check` + `authority` + a trial run per target.
2. **`run … --grant <exactly required_grants> --report-out r.json --no-prompt`.** If a statically known grant is
   missing, refuse before the first effect and name every missing grant. That is the owner's decision (§9); the
   additive half, naming them all, is safe now.
3. **Read `r.json`.** One envelope in every outcome: the program's output (or a pointer to it), the exit, `stopped_by`,
   `engine`, and a reason for a signal (RW 6.68).

Each step still establishes what the old steps did. `authority` and the target matrix are still computed and shown;
they arrive in the answer the agent already reads.

### 6.3 A precompiled-module cache that is safe

Wasmtime's own documentation says `Module::deserialize` "is only designed to receive the previous input from
`Module::serialize` and `Engine::precompile_module`. If the exact output of those functions (unmodified) is passed …
calls … can be considered safe". It also says arbitrary bytes "may trick Wasmtime into arbitrary code execution".
Bytes from another Wasmtime version fail with an `Err`.

**So a cache is safe only if DeluluLang can prove a cached file is bytes it serialized itself, unmodified.** That
needs:
- **Key:** BLAKE3 of the `.wasm` bytes, Wasmtime's version, the `Engine` configuration (fuel, epoch, memory settings)
  and the host CPU features.
- **Integrity:** a MAC over the key and the bytes, keyed by a per-installation secret under `DELULU_STATE_DIR` (0600),
  verified before `deserialize`. A cache directory that another account can write is not used.
- **Authority is not in the cache.** The `.dwx`'s manifest and the run's grants are checked on every run exactly as
  now. The cache holds only machine code for the bytes `dir::verify` and codegen produced.
- **Fail closed to compile:** any miss, mismatch, bad MAC or `deserialize` error means compile from source, never
  refuse.
- **Never accept a precompiled artifact from outside** (a `.dwx` section, a plugin, a registry).

**Expected benefit:** 3 ms on a tiny program and up to about 650 ms on a large one (earlier audit). **Proof
obligations:**
- a flipped bit in a cache file is refused (MAC);
- a file from another configuration misses;
- revoking a grant still refuses.

### 6.4 Physical AI and edge

What this audit shows for constrained targets:
- **Memory:** check a program on the development host; on the device, keep only the runtime. The package `check` above
  peaked at 26 MiB and a run at 42 MiB (C launcher).
- **Two runtime reservations a device may refuse:** the interpreter's 781 MiB virtual stack, and WASM's 4 GiB guard
  (§5.6). A device profile must be told about both before a program is chosen, through the target matrix.
- **What DeluluLang can honestly offer:**
  - typed, bounded and audited actuation through the broker;
  - a deterministic simulation profile;
  - envelopes, enforced as §5.6 shows.
- **What it cannot offer:** hard real-time guarantees or safety certification. Planning in DeluluLang should stay
  separate from the safety-critical control loop, which belongs to a certified controller. That is INFERENCE from the
  design, and the earlier reviews' wording stands.

---

## 7. Prioritized changes

Each change below lists its benefit, its proof (a witness red before the change), compatibility, rollback, and a stop
condition. None is made in this change.

### P0: correctness and security

- **P0-1: enforce R-Get at `p.get`** (RW 4.95; R1).
  - Benefit: a function's row, `authority` and DL1009 become sound for plugin users.
  - Proof: `esc.delulu` is refused at check time, or `p.get` refuses at run time, and the file is never written. A
    correctly annotated host still runs.
  - Compatibility: programs that under-annotate now break, and that is the point.
  - Rollback: none needed. Stop: when the witness is red before the change and green after it.
- **P0-2: close RW 4.93 and 4.94** (DIR node-id uniqueness and the lexer's rules in `dir::verify`). As recorded.

### P1: agent task blockers

- **P1-1: a package-aware LSP** (RW 6.72; R2).
  - Proof: `lsp_cli` opens `examples/greeter/src/main.delulu` and receives 0 diagnostics (red before). An error
    introduced in `greetings.delulu` appears under that file's URI.
  - Stop: if a 10,000-line package takes more than 100 ms per change, add the per-module memo, then measure again.
- **P1-2: a package-level `edit`** (RW 6.69; R3).
  - Proof: the `Net` edit is refused under `--if-checks` and reported as widening; the `salutation` edit is accepted.
- **P1-3: name every missing grant at the first refusal** (RW 6.73; R5).
  - Proof: `partial.delulu` with no grants lists `fs.write=./out` and `console` in one envelope.
  - Pre-execution refusal stays opt-in until the owner decides (§9 Q1).
- **P1-4: no unrunnable `.dwx`** (RW 3.28; R6).
  - Proof: `build --target wasm` of `fn main() {}` is DL1201 (or runs).
- **P1-5: a recursion bound that holds on every host** (RW 3.29; R7).
  - Proof: under `ulimit -v 300000`, the deep program is DL0905 with a report.
- **P1-6: MCP cancellation and bounds** (RW 6.74 and 6.62; R8).
  - Proof: a cancelled call gets no reply and a `ping` behind it answers at once; a 4,000-error answer is capped and
    says so.
  - Compatibility: additive.
- **P1-7: one outcome envelope** (RW 6.60 and 6.68; R9).
  - Proof: under `--json`, a budget stop, a refusal and a fault each produce one envelope; SIGTERM writes the report.

### P2: measured performance

- **P2-1: MCP waits without polling** (R8). Up to 5 ms per call; the test is the §5.1 quantization table.
- **P2-2: libpython loaded on first use** (RW 3.26; R10). 1.35 ms and about 4.2 MiB per command.
  - Proof: the A/B in §5.1 shows no gap, and `examples/numpy_mean.delulu` still runs.
  - Stop: if `dlopen` makes the Python path less reliable across the CI matrix.
- **P2-3: the precompiled-module cache** (§6.3; RW 3.27; R11).
- **P2-4: drop the second check on a package run** (R4). Only after P1-1, and only if a profile of a real package says
  it matters.

### P3: smaller cleanups, and later hardware work

- **P3-1:** `toolchain --json <section>`.
- **P3-2:** an unknown command names the command list, not the usage text.
- **P3-3:** a `features` object (Python, microVM, OS, engines) in `--version --json` and `toolchain`.
- **P3-4:** `engine` in the run report.
- **P3-5:** a note on zero tests, and perhaps `test --require-tests`.
- **P3-6:** theme and config follow `DELULU_HOME`.
- **P3-7:** a clearer message for a missing `--report-out` directory.
- **Later:** ARM and real-device runs of §5.6's address-space probes, which need hardware (blocked here).

---

## 8. Remove, or don't remove

| Tempting removal | Saves | Verdict |
|---|---|---|
| Colourful human diagnostics | Nothing on the machine channel: JSON is separate and locale-invariant | **Don't remove** |
| The human usage text and `explain` prose | Nothing per command; it is read only when asked | **Don't remove**; stop dumping it on a typo (P3-2) |
| Pretty-printed JSON | 32% of `toolchain --json`'s bytes; tokens unknown | **Don't change** until a tokenizer measurement says it pays |
| Embedded Python | 1.35 ms and about 4.2 MiB per command | **Don't remove; load it lazily** (P2-2): the capability is a documented feature |
| The interpreter, or WASM | Neither: each wins a workload (§5.4) | **Keep both** |
| MCP's process per call | 3–5 ms per call | **Keep**: isolation and cancellation by `kill`. Fix the poll |
| The LSP's per-document cache, for an incremental engine | Nothing measured: 0.21 ms per change | **Keep**; add package awareness, not Salsa |
| The second check on a package run | About 30 ms per 1,500 lines | **Defer** (R4): it guards the runner's flattening |
| Guard, broker, custody, leases, audit layers | No overhead measured on the permitted path | **Never** (independent enforcement) |
| Morph skins, locales, themes | Two `stat` calls | **Keep optional**; fix RW 6.70 |
| The 781 MiB stack reservation | Nothing (it is virtual) | **Keep**, and make its failure honest (P1-5) |
| The Survey, doctor and the Atlas | Nothing on the agent's path | **Keep** |

---

## 9. Questions for the owner

Each of these is a tradeoff the evidence cannot settle alone.

1. **Grants before execution.** Should `run` refuse before the first effect when a statically known grant is missing?
   - For: no partial effects, one round trip.
   - Against: a branch that never runs would now need its grant.
   - It changes what a run means. Under the delegation it is a `D-V2-nn`. If it touches the Constitution's fault
     rules, it is an entrenched change with an `ENTRENCHED_CHANGE_RECORD.md` entry.
   - **Recommendation:** an opt-in flag now; consider making it the default for `--sandbox` runs.
2. **WASM's `main`.** Should the WASM backend support `fn main()` and a non-`Unit` `main`, or should the checker teach a
   single `main` shape for both engines? **Recommendation:** refuse at build now (P1-4); decide later whether a value
   `main` returns means anything.
3. **Application exit codes** (RW 2.40). An exit code from `main` returning `Result` touches `STABILITY.md`'s exit-code
   table (entrenched): a `D-V2-nn` plus an `ENTRENCHED_CHANGE_RECORD.md` entry. Without it, "handled application
   error" stays indistinguishable from success except by parsing the program's own output.
4. **MCP protocol revisions.** `2025-11-25`, and `2026-07-28` (stateless: no `initialize`, `server/discover`
   required, `ping` removed, `resultType` required on every result). Adopt them when a client the project depends on
   needs them; until then, cancellation, `outputSchema` and bounds are the adoption benefit. *Corrected in §11.4:*
   `2026-07-28` is final. The specification's versioning page says *"The current protocol version is 2026-07-28"*,
   and its GitHub release, dated 2026-07-28, is marked Latest. Under it, stdio clients fall back to `initialize` when
   `server/discover` fails, so today's server stays usable.
5. **Zero tests.** Should `delulu test` that finds no test exit 0 (today) or a distinct code? A distinct code is a
   `STABILITY.md` question. A `--require-tests` flag is not.

---

## 10. Recommendation

**What to do next, in order:**
1. P0-1: R-Get. It is the one finding where a guarantee the language states is false.
2. P1-1 and P1-2: package awareness in the warm path and in `edit`. Same root cause, same fix.
3. P1-3, P1-4, P1-5 and P1-7: refusals that name everything, artifacts that run, faults that stay diagnostics, one
   outcome envelope.
4. P1-6, then P2-1: MCP cancellation, bounds and the poll.
5. P2-2 and P2-3, the lazy libpython and the module cache, each with its A/B before and after.

**Not worth changing now:**
- the parser or the IR;
- an incremental query framework;
- removing the interpreter or WASM, or making the engine choice automatic;
- removing human diagnostics, colours, locales or the usage text;
- removing MCP's process isolation;
- any layer of Authority, the Guard, the broker or the audit;
- pretty-printing, until tokens are measured;
- a new protocol, service or database.

**The guiding question, answered.** *Will this change make an AI's real task simpler, faster and more dependable
without hiding failures or weakening authority?*
- **Yes, measurably:** P0-1 through P1-7. Each removes a wrong answer, a wasted round trip or an unrunnable result,
  and none hides a failure.
- **Yes, but small:** P2. Milliseconds per command; worth having, measured before and after.
- **Unproven:** any change to pretty-printing or to the default of the grant preflight. Run the experiment or ask the
  owner first.

---

## 11. Second pass, 2026-11-05 (later the same day): double verification, code review and research

The owner asked for everything to be verified again: this audit, the pipeline audit, the strategic verification and
every prompt in `docs/prompt/`. He also asked for README and the other documents to be updated, CI to be checked and
fixed, the repository's code to be checked, and sub-agents to be used where they help.

**Who did what:**
- **Three Sonnet 5.5 sub-agents** (`claude-sonnet-5-5`, each naming its model in its report):
  - an independent verifier of this audit's claims;
  - an adversarial reviewer of the code behind the agent surfaces;
  - a researcher refreshing external facts.
- **The head chef re-ran each finding before a word of it was used.**
- **No code was changed.** The binary is the one in §2.

### 11.1 State and CI

**`master` = `5fb865c`** (this audit, PR #127). Its push run
[42015437785](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/42015437785) was **red**:
- the failing job was `test (windows-latest)`, in `monitor_cli`'s `a_long_chain_does_not_kill_a_healthy_monitor`;
- the same commit's PR run, 42012298873, was green on all 14 jobs, and the commit changes documents only;
- this is RW 7.25's test, red once before on macOS. That row asked that the next red be read to tell its three causes
  apart, and this one was:
  - the monitor was still running;
  - the daemon was running and hearing;
  - but the broker had answered a `ListPage` 1,879 ms late, *"waiting for the requests ahead of it"*, when the 500 ms
    dead-man fired.
- RW 7.25 now carries that evidence;
- **corrected 2026-11-05 (the laptop, later the same day):** the three readings above overreach, and RW 7.25 now says
  so. The exit status `None` was read after the monitor had printed *"ended"*, so it does not exclude an earlier pause
  of the monitor. *"Hearing"* rests on the absence of a deaf-interval line, which says the hearing thread looked, not that
  the readers ran. The `ListPage` was the test's own `grants list` at that check; its 1,879 ms is stamp-to-handler time,
  not queue occupancy, and its order relative to the firing is unknown. Neither red's cause is established;
- the failed job was re-run once to confirm the failure is not the commit's; the re-run's result is recorded in the pull
  request and `docs/CLOUD_SYNC_LOG.md`.

**The scheduled run** [42009076387](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/42009076387)
on `35f1a2d`: 16 of 18 jobs green, `heavy-gates` among them; the two `miri-slow` jobs were still running when read.

**A full local test suite could not be run.** A debug build of the workspace's tests filled this session's disk
allowance at 22 GB. The run died and the cache was deleted. The suite's evidence is CI's: four platforms, green on this
commit's PR run.

### 11.2 Re-verification: what held, and what did not

**The verifier sub-agent re-ran §1.1's items, writing its own programs**, and the head chef re-ran its divergences.

| Claim | Verdict | Evidence |
|---|---|---|
| RW 6.72, LSP package-blind | **Holds** | Opening either file first changes nothing; `check src/main.delulu` alone gives the same two DL0301, so the server behaves like a single-file check |
| RW 6.73, late grants | **Holds** | `a.txt` written, then DL0703 for `console`; with no grants only the first is named; `authority` listed both |
| RW 3.28, unrunnable `.dwx` | **Holds** | All three shapes, `fn main(root: Root) -> Int` giving *"expected 1 results, got 0"* |
| RW 3.29, abort under a cap | **Holds, and is worse than recorded** | Under `ulimit -v`, the abort appears below about 940,000 KiB. **The audit's claim that the sandbox was unaffected was wrong.** `--sandbox-profile hostile-agent` (256 MiB) and `--sandbox --limits mem` up to 768 MiB abort the guest (*"has overflowed its stack"*), and the report says only `{exit 1, ran: true}`, `diagnostics: []`. DL0905 returns from 896 MiB. The first reading saw exit 1 and did not read the guest's stderr. Re-run by the head chef: hostile-agent and `--sandbox --limits mem=268435456` abort; contained gives DL0905 on stderr; `--limits mem=268435456` without the sandbox gives DL0905 |
| RW 6.74, MCP no cancel | **Holds** | On an 8,000-line and a 40,000-line check, the cancelled call answered in full and `ping` waited behind it |
| RW 6.62, the poll | **Holds, sharper** | MCP latency sits on about 5.1 ms plateaus (5.6, 10.8, 15.9, 21.0, 31.2 ms) while fresh-process time rises smoothly. For a tiny file MCP is **slower** than a fresh process: 5.7 against 4.2 ms |
| RW 3.26, libpython | **Time holds; memory corrected** | 1.27–1.69 ms per command. Peak RSS **+4.2 to +4.4 MiB on every command** by a C launcher. The audit's "+0.4 MiB" was an artifact: a Python launcher's `ru_maxrss` includes the parent's own high-water mark, which `exec` carries over. Re-run by the head chef with the earlier audit's C launcher (`rss.c`): `--version` 9.5 / 5.3 MiB, `check` 12.3 / 7.9, `explain` 10.1 / 5.9 |
| RW 6.60, budget stop | **Holds; and a fifth shape** | With `--sandbox`, a budget stop does print an envelope (`error.kind: failed`); without it, nothing |
| Pure tasks open no keys, plugins or audit files | **Holds** | A wider `strace -e trace=file` |
| No prompt blocks a non-terminal | **Holds** | Also with stdin a pipe or a pty; the prompt that exists is for foreign C libraries on a terminal only (`cli.rs` about line 9673) |
| Package check against run | **Ratio holds** | On its own 5,778-line package, run/check = 1.95–2.08 |
| Zero tests exit 0 | **Holds** | |

**Found while verifying, and re-run by the head chef: RW 6.75 (new).** Under `--sandbox` the guest's fault code
reaches neither the envelope nor the report. `fault.delulu` gives DL0902 in both without the sandbox; with it, both say
`diagnostics: []` and the code appears only on the guest's stderr.

**Re-witnessed by the head chef on this binary:**
- **RW 4.93:** `dirforge` collides the node ids of `cap` and `push`, whose text is DL1604, and `dir::verify` says *ok*;
  the control `effect` is refused.
- **RW 4.94:** `bidi.dpx` verifies, and `why Read` prints its RLO and ANSI bytes raw.
- **RW 6.67:** `forged.dpx` is DL1504 for `plugin verify` while `why Read` answers.
- **RW 6.70:** the same source bytes print `yes` and `no` under two working-directory skins.
- **RW 4.95 and 6.69:** as in §5.5.

**The pipeline audit's 71-probe ledger, replayed:** 65 identical, and 6 differences, all from scratch state or the
harness:
- 2 were a 3,000-character against a 4,000-character capture;
- U2 and U3's input file had been overwritten by later probes; restored, they print `{"ok":false,"error":"format"}`
  as recorded;
- L2's `fix` had already applied its repair;
- 1 was a shell's own `argv[0]` wording.

### 11.3 Code review: fourteen confirmed defects in the agent-facing surfaces

The code-review sub-agent read `mcp.rs`, `toolchain.rs`, `schema.rs`, `edit.rs`, `run_cmd.rs`, `cli.rs` and `lsp.rs`, and
probed each on the binary. It also listed the probes that held (§11.5). Every finding below was re-run by the head chef
(F12 and part of F10 read in source), and none was on record. They are grouped into eight rows.

| Row | Finding (sub-agent's id) | Severity |
|---|---|---|
| **RW 4.97** | `verify-sig`, `sign` and `keygen` ignore unknown options (F3). `verify-sig art.dwx --kye <key>` drops the pin and says **valid**, exit 0, where `--key` says `invalid`. `sign --name second` signs with the default key. `keygen --bogus` mints a key | MEDIUM–HIGH |
| RW 6.76 | MCP: `isError: true` for errors on every tool but `check`, against `for-agents.md` (F1); a non-UTF-8 line ends the server (F10); a batch, and an `id`-bearing `notifications/…`, get no reply (F10); `depth` truncated `as u32`, and `additionalProperties: false` not enforced (F12) | MEDIUM |
| RW 6.77 | A run that succeeded, then could not write `--report-out`, exits 2 `kind: usage` after its effect (F2) | MEDIUM |
| RW 6.78 | Option validation uneven (F6, F7, F8, F14): `authority --isolation bogus` is reported, not refused; `toolchain`'s flag lists are scanned from prose (`run` lists `--stdio-pipes`); `test -v` is a name filter; a repeated `--report-out` is last-wins; `test . .` counts every test twice | LOW–MEDIUM |
| RW 6.79 | `schema validate --json` reports `summary.errors: 0` for an invalid document (F9) | LOW |
| RW 6.80 | `edit`: `--node` writes `pub pub fn`, stacks attributes, cannot take an inline `@…` value, replaces a symbolic link with a file, and mixes line endings (F5, F13) | LOW–MEDIUM |
| RW 6.81 | LSP rename accepts `fn`, `let` and `Int` (F4); a lower-case `content-length`, a missing length or a non-UTF-8 body ends the server silently (F11) | LOW |

**What they share.** Each is a surface that answers in a shape its contract does not describe, or accepts input its
contract says it refuses. That is §1's pattern: a component built for one human session, now driven by agents that
read the exit code and the envelope literally. None is a containment escape. 4.97 is the one an attacker could use: a
typo turns a pinned signature check into an unpinned one.

### 11.4 Corrections made in place in this file

| Where | Was | Now | Why |
|---|---|---|---|
| §1.1 item 5, §5.6 | the sandbox and `--limits` are not affected | the sandbox's memory ceiling, `hostile-agent` included, triggers the abort; `--limits` alone does not | misread exit code (§11.2) |
| §1.1 item 9, §2.1, R10, §5.1, §8 | libpython +0.4 to +4.5 MiB | about +4.2 MiB on every command | Python launcher artifact |
| §5.1 | fresh `check` 13.5 MiB; `--version` 9.4/9.0; `check` 13.5/9.1; `explain` 10.9/9.0 | 12.3; 9.5/5.3; 12.3/7.9; 10.1/5.9 MiB | C launcher |
| §5.2 | 27.1 / 27.9 / 43.6 MiB | 25.9 / 26.5 / 42.1 MiB; the two edit rows unmeasured by C | C launcher |
| §5.4 | 15.3/26.8 and 15.5/26.6 MiB | 13.8/22.6 and 13.7/22.6 MiB | C launcher |
| §6.4 | 27 and 44 MiB | 26 and 42 MiB | C launcher |
| §1.1 item 9, R4 | a package run 1.9× a check | about 2× (1.8–2.1× over five measurements) | more data |
| §9 Q4 | `2026-07-28`'s status disputed | final: the specification's versioning page, *"The current protocol version is 2026-07-28"*, read by the head chef | primary source |
| Appendix B | Deno caches by a hash of each module | one check-cache hash over the checked modules' sources, options, npm state and Deno's version | source read by the research sub-agent |

The latency figures were not affected: every one was a parent-side wall clock.

### 11.5 What the prompt asked that §1–§10 had not answered

**More experiments the prompt named:**
- **Batch against single checks (prompt §7.1), on Linux:** 20 files in one `check` took **13.0 ms** median (p90 16.7);
  in 20 processes, **121.3 ms** (p90 128.6), 9.4×. One envelope, 0 errors. `docs/for-agents.md`'s rule 4 holds here
  too.
- **Permitted against denied actions under custody (prompt §4).** 21 runs each, with the sub-agents loading the
  machine, so the spread is wide:

  | Case | Embedded | Daemon broker |
  |---|---|---|
  | Console only | 10.3 ms | 10.5 ms |
  | Three granted writes | 10.3 ms | 16.1 ms |
  | Refused | 6.1 ms | 6.9 ms |

  The broker used a short `DELULU_STATE_DIR`, because the scratch path's 116-byte socket was refused: Linux allows 107.
  So a daemon costs about 2 ms per brokered effect, and nothing on a run that brokers none. An approval loop through the
  Guard was not measured.
- **The Survey's `diff` and `path`, and the Atlas:**
  - `survey diff 35f1a2d` names this pass's changed files and reaches 0 nodes from them;
  - `survey path mod:crates/delulu/src/lsp.rs crate:delulu-check` is one hop, at `lsp.rs` line 21;
  - `atlas examples/greeter` maps both modules: 8 nodes, 11 edges, and the caveat that function values may be
    under-approximated.
  - **The Survey maps the repository and the Atlas maps a program. They overlap nowhere, and both stay.**

**Q2: which recomputation is load-bearing.** Three things are recomputed on every command and must stay that way:
- the authority of the code that will run (no report is trusted across an edit);
- the grants and the lease, against the broker's current state, so that a revocation takes effect;
- `dir::verify` of every plugin at load, because bytes can change between loads.

What could be cached without weakening any of them is the *derived* work for unchanged bytes, keyed by content and
toolchain identity, and never by a grant:
- a parsed and checked module;
- Cranelift's machine code (§6.3).

The measurements say only the second pays today.

**Q4: a warm session's rules.** The language server already holds no authority, loads no plugin and runs nothing
(`for-agents.md`). It drops a document's analysis on every edit (`lsp.rs` `Doc::apply`). What it lacks is
package context (RW 6.72). Its memory grows with the open text (RW 2.44, a 76.7 MiB plateau at 4,002 lines), and its
rollback is a restart, which costs 3.7 ms. MCP keeps no state between calls at all.

### 11.6 Research refresh

Read by the research sub-agent from primary sources, with dates; the head chef re-read the MCP and versions rows.

| Fact | Status 2026-11-05 |
|---|---|
| MCP | `2026-07-28` is the current, final revision, released 2026-07-28. Under it, stdio clients probe with `server/discover` and fall back to `initialize`; cancellation on stdio is `notifications/cancelled`, which a server SHOULD honour; structured results MUST conform to a declared `outputSchema`. Nothing newer exists |
| Wasmtime | **50.0.1** is the latest stable (2026-10-29); 51.0.0-rc.1 exists. **48.0.8 is still the newest 48.x** (2026-10-29) and carries the fixes for the five advisories of 2026-10-29 (GHSA-6fr9-xmw9-5352, -hmmr-87qq-258j, -qw58-7h65-crq2, -3982-jqvv-277m, -h4wp-g4x3-7398; RUSTSEC-2026-0420 to 0424). Nothing newer in RustSec |
| Rust | Stable is **1.99.1** (2026-10-15); the repository pins **1.96.1**, three minor versions behind. Wasmtime 48.0.8 needs 1.95. The pin is a deliberate reproducibility choice; moving it is a change of its own, with CI on every platform |
| Salsa | 0.28.5 (0.28.3 yanked); rustc-dev-guide still says it is not used directly in rustc |
| OpenShell / Zerolang | v0.1.7 (2026-11-03) / v0.3.4, both still the newest |
| MoonBit | v0.10.19 (2026-11-05), still before 1.0 |
| Mojo | 1.0.0 stable on 2026-08-11; its compiler open source since 2026-08-18 (Apache 2.0 with LLVM exceptions); 1.2.0 on 2026-10-22 |
| WASI | 0.3.0 shipped 2026-06-11, 0.3.2 on 2026-10-13; Wasmtime supports it by default from 46 |
| Deno | `deno run` does not type-check by default; the check cache is one hash over the checked inputs and Deno's version |
| Measured agent-task overhead for an agent-first language | **No credible 2026 publication found.** The nearest are three arXiv papers (abstracts read by the head chef). "Beyond Human-Readable" (2604.07502): compressing logs cut input tokens 17% but raised total session cost 67%. That is a caution for §8's decision not to compact JSON blindly. "AI Coding Agents Need Better Compiler Remarks" (2604.13927): structured remarks raised success. "The Best Programming Language for Tokenmaxxing" (2607.22807): token cost varies by language. None measures calls, tokens and latency together |

### 11.7 What changed in the decisions

1. **RW 3.29 moves up.** The profile recommended for untrusted code turns a recursion fault into a crash with no code
   in the report. P1-5's witness must include `--sandbox-profile hostile-agent`.
2. **Two P1 contract items.**
   - RW 6.75 joins P1-7 (one outcome envelope): a sandboxed run must carry its fault code.
   - RW 4.97 joins P0 as a small fix: refuse unknown options in the signing commands, because it fails a verification
     open.
3. **The rest join P1 (6.76, 6.77) and P3 (6.78–6.81).** Each is small, local and witnessable. 6.77 can repeat an
   effect, so it goes first among them.
4. **Unchanged:** the order P0-1 (R-Get), then P1-1/P1-2 (package awareness), and the list of things not worth changing.

## Appendix A: the programs and commands

The binary is `B`. Every command ran with `DELULU_HOME`, `DELULU_STATE_DIR` (scratch) and `DELULU_NO_FIRST_RUN=1`.

**E1, `tiny.delulu`.** The constant `K` changed every cycle.

```
module tiny
fn add(a: Int, b: Int) -> Int { a + b + K }
fn main(root: Root) ! {Write} { root.console().println(str(add(2, 3))) }
```

The three clients:
- **fresh:** `B check tiny.delulu --json`;
- **LSP:** `didOpen`, then `didChange` with the full text, waiting for `publishDiagnostics`;
- **MCP:** `tools/call` `check` with `{"paths": ["tiny.delulu"]}`.

**E1, address space, `deep.delulu`.**

```
module deep
fn down(n: Int) -> Int { if n == 0 { 0 } else { 1 + down(n - 1) } }
fn main(root: Root) ! {Write} { root.console().println(str(down(100000))) }
```

Run as `( ulimit -v 300000; B run deep.delulu --grant console --no-prompt --report-out r.json )`.

**E2, the package.** `delulu.toml` is `kind = "bin"`, `[authority] effects = ["Write"]`. The generator:

```python
for m in range(12):
    lines = [f"module pkg.m{m}", ""] + [
        f"pub fn f{m}_{i}(n: Int) -> Int {{ {'n + 0' if i == 0 else f'f{m}_{i-1}(n) + {i % 7}'} }}" for i in range(120)]
    open(f"pkg/src/m{m}.delulu", "w").write("\n".join(lines) + "\n")
# src/main.delulu: `module pkg`, `import pkg.mK` for each K, and main printing the sum of f{K}_119(1)
```

The LSP probe opened the copy of `examples/greeter/src/main.delulu`.

**E3, `util.delulu`** (45 lines; the CSV validator, as written after its one repair). Its signature:
`fn main(root: Root) ! {Read, Write}`. It reads `root.fs_read("./data").read_text("people.csv")` and prints
`{"ok":…,"valid":…,"age_sum":…,"invalid":[…]}`. The input was `name,age / ada,36 / lin,x9 / grace,85 / ,40`.

`partial.delulu`:

```
module partial
fn main(root: Root) ! {Write} {
    let w = root.fs_write("./out")
    let _ = w.write_text("step1.txt", "written before the refusal")
    root.console().println("done")
}
```

The other outcome programs:
- `loop.delulu`: `while true` inside a function;
- `fault.delulu`: `1 / 0` through a function;
- `notests.delulu`: no `test` block.

Budgets were `--limits cpu=1` and `--limits wall=1`. The SIGTERM came after 1 s.

**E4, `kern.delulu`** (`work(3000000)`, and `work(10)`):

```
module kern
fn work(n: Int) -> Int {
    var acc = 0
    var i = 0
    while i < n { acc = (acc + i * 7) % 1000003
        i = i + 1 }
    acc
}
fn main(root: Root) ! {Write} { root.console().println(str(work(3000000))) }
```

`pure.delulu` is `module pure` / `fn main() -> Int { 6 * 7 }`. The commands:
- `B build pure.delulu --target wasm -o pure.dwx`;
- `B run pure.dwx --no-prompt`.

**E5.**
- `oos.delulu` writes `"../escaped.txt"` through `fs_write("./out")`.
- `dyn.delulu` picks `./out` or `./other` from a line of standard input.
- RW 4.95: the putter plugin and `esc.delulu`, exactly as in the pipeline audit's §19.5:
  - `B plugin build .` in the plugin's directory;
  - then `B run esc.delulu --no-prompt --grant plugin=. --grant fs.write=./out`.
- RW 6.69: `B edit src/main.delulu --expect-hash H --node fn:greeter.main --with @newmain.txt --if-checks --json`, where
  `newmain.txt` declares `! {Write, Net}` and calls `root.http(["evil.example.com"]).get("https://evil.example.com/x")`.

**E6.** Run as
`B run arm.delulu --no-prompt --grant console --grant actuator=arm0/elbow:angle_deg=0..90,heartbeat_ms=60000,ttl_ms=60000,fail=hold --broker-profile sim --seed 7`.
The variants added `--sandbox` or `--engine wasm`. The arm program is in the pipeline audit's scratch set: three
`command`s, at 42.5°, 120° and 80°.

**E7.** `noisy.delulu` is `module noisy` plus 4,000 lines `fn fK() -> Int { undefined_K }`. The probe sent, over one
`delulu mcp` session:
- `initialize` (asking `2025-11-25`, then `2026-07-28`);
- `tools/list`;
- the noisy `check`;
- four malformed lines;
- `explain`;
- a `check` of an 8,000-line program, with `notifications/cancelled` for its id 50 ms later and a `ping`;
- 200 noisy checks;
- one more call, after which the client closed its read end.

**System calls:** `strace -f -c` and `strace -f -e trace=openat,…` of `check`, `run` and `authority` on `tiny.delulu`.

## Appendix B: sources read for §6 and §9

- **MCP `2026-07-28`** (`docs/specification/2026-07-28/` in `modelcontextprotocol/modelcontextprotocol`, `main`,
  read 2026-11-05): `server/tools.mdx`, `basic/patterns/cancellation.mdx` and `changelog.mdx`.
  - Servers **SHOULD** stop processing a cancelled request and not respond.
  - stdio clients **MUST** send `notifications/cancelled`.
  - Structured results **MUST** conform to a declared `outputSchema`.
  - A text copy **SHOULD** accompany structured content for backwards compatibility.
  - The revision removes `initialize` and `ping`, and adds `server/discover`, `resultType`, and `ttlMs`/`cacheScope`
    on list results.
- **Wasmtime** (`docs.wasmtime.dev`, the `Module` API page showing 52.0.0-dev, and *Pre-Compiling Wasm Programs*):
  - `deserialize` is `unsafe`;
  - only the unmodified output of `serialize` and `precompile_module` is safe;
  - other versions' output fails with an `Err`;
  - compiling and running can be split into separate builds.
- **Salsa:** README (`salsa-rs/salsa`); crates.io (0.28.5, updated 2026-09-24); the rustc-dev-guide's `queries/salsa.md`
  (*"it is not used directly in rustc"*; rust-analyzer uses it).
- **Deno** (docs; its `cli/type_checker.rs` and `cli/cache/check.rs` on `main`, read by the research sub-agent): one
  check-cache hash covers the checked modules' sources, the options, the npm state and Deno's version. *Corrected from
  "a hash of each module".* `deno run` does not type-check by default. A design DeluluLang must not copy, because its check is its authority guarantee.
- **The earlier comparisons** (Mojo, Zerolang, MoonBit, WAMR, OpenShell) are in the pipeline audit's §19.3 and were
  not repeated.
