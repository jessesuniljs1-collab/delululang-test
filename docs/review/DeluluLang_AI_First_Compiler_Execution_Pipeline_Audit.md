# DeluluLang — AI-first compiler, interpreter and execution-pipeline audit

**Written:** 2026-11-05 (UTC), in a Claude Code cloud session (`session_01Deh6W9bJdfxAeMARtXU9R9`). The configured and
last-served model was `claude-opus-5-5`, read from the session record. **Commissioned by:** the owner's prompt
[`docs/prompt/DeluluLang_Claude_Cloud_AI_First_Compiler_Pipeline_Audit_Prompt.md`](../prompt/DeluluLang_Claude_Cloud_AI_First_Compiler_Pipeline_Audit_Prompt.md),
read in full. **Companion to:**
[`DeluluLang_Claude_Independent_Strategic_Verification.md`](DeluluLang_Claude_Independent_Strategic_Verification.md),
whose §18 is this audit's follow-up section.
**Tree:** `master` = `0670e0bb4db8160469d1f092ab0b088aa4b76220`. The session's branch adds only documents (§2).
**Nature:** research, experiments and documents. **Nothing in this file is implemented unless it says so**, and every
proposal is marked **PROPOSED (not built)**.

**The owner, during this session (2026-11-05):** *"I allow u to suggest any changes that is against current
constitution.md, license or any other .md file rules and regulations. Do full research based prompt .md file I
attached earlier, No restrictions."* §14 holds the recommendations that cross a current rule; each names the rule.
They are suggestions only. No entrenched file was edited.

**Third verification pass (2026-11-05, same session, on the owner's request to *"double verify and research about
everything once again … test everything"*; no sub-agents).** §19 records it. Every probe of Appendix A.4 was replayed
and every defect below re-witnessed on the same code, now `master` `6397a98`. It found **one new defect, the most
serious of this audit: RW 4.95**, below. A few figures were corrected in place, each listed in §19.4.

Evidence labels: **OBSERVED HERE** (run in this session, output retained in the session's scratch) · **CURRENT
SOURCE** (read in `0670e0b`) · **CURRENT CI** (a named run whose result I read) · **HISTORICAL DOC** · **OFFICIAL
EXTERNAL** (a vendor page or repository) · **THIRD-PARTY** · **SUB-AGENT** (a Sonnet 5.5 sub-agent's report; marked
*re-verified* only where I re-ran it) · **INFERENCE** · **UNKNOWN**.

---

## 1. Executive recommendation

**Answer to the central question.** No: a DeluluLang program does **not** have to become a standalone binary before
it runs, and today it never does (OBSERVED HERE).

What happens today:
- **The interpreter** runs the checked syntax tree directly.
- **The WASM engine** compiles the checked program to a WebAssembly module in memory. Wasmtime's Cranelift then turns
  that module into machine code on every run.
- **A `.dwx`** is a portable WebAssembly artifact with its authority manifest. It still needs the `delulu` runtime,
  and Cranelift still compiles it on every run.

**What is mandatory in every model:**
- **The check** (resolution, types, effect rows, reference capabilities, authority). Every DeluluLang guarantee rests
  on it.
- **An engine whose every effect goes through the host's primitive table and custody** (grants, leases, the Guard,
  the audit chain).

**What is optional:**
- **text** (the parser);
- **human diagnostic prose**;
- **a native executable**;
- **the editor**.

So the recommended architecture is **CLI-first over one typed service layer, with text as the interchange and
structured edits on top, a hybrid interpreter + WASM engine with an optional authenticated precompiled cache, and no
new compiler backend, graph database or agent runtime.**

**What the measurements decide** (OBSERVED HERE, §10):

| Question | Measured answer | Consequence |
|---|---|---|
| Is parsing a cost worth removing for AI authors? | lex+parse is **3 µs** of an 85 µs front end for a 2-line program, and **64 ms of 381 ms** (17%) at 16,002 lines. Checking is 68%, resolving 2% | A parser-free path saves at most about a sixth of the front end. It is not worth a second input language |
| Is the existing machine form (DIR) a faster input than text? | **No.** At 16,002 lines DIR is **23.6 MB against 838 KB of text** (28×). Decoding it takes 322 ms against 64 ms to lex and parse the text; verifying it (decode + re-check) takes **716 ms against 381 ms** for the whole text front end | Keep text as the interchange. Use node-addressed edits over text, not a DIR or graph store, as the agent's authoring surface |
| What dominates a short WASM run? | Cranelift compiling the module: **1.6 ms** of a 1.8 ms hello run, **652 ms** of 676 ms at 16,002 lines | A precompiled artifact loads in **35 µs** (hello) and **5.7 ms** (16,002 lines), 45–115× faster |
| What dominates a short command? | The process itself. `delulu --version` takes 3.35 ms, against 0.9 ms for `/bin/true`, and **1.1–2.0 ms of it is loading `libpython`**: a Python-less build of the same source takes 2.21–2.86 ms (three comparisons, §10.2). `check hello` adds 0.5–0.9 ms (§19; first written as 0.3 ms) | Moving embedded Python out of the main binary is the cheapest measurable win for agent loops |
| Interpreter or WASM? | A 5M-iteration loop: interpreter **910 ms**, WASM **34 ms**, CPython 246 ms, Rust 10 ms. A 4,002-line program with little work: interpreter **136 ms**, `.dwx` 179 ms | Neither engine wins everywhere. Choose per program (RW 3.3's tiering), and cache compiled code |
| Do the baseline compiler (Winch) or the portable interpreter (Pulley) help? | Winch compiles 15–33% faster but runs the loop 42% slower, and Wasmtime 48 documents epoch interruption, which DeluluLang's limits engine uses, as *"not compatible with the Winch compiler"*. Pulley compiles within 11% of Cranelift's time either way and runs 11× slower | Neither replaces Cranelift. Pulley is a fallback for CPUs Cranelift does not support |
| Can an AI emit trustworthy machine code? | Wasmtime itself refuses to trust compiled bytes: `Module::deserialize` is `unsafe` and *"Arbitrary input could … replace valid compiled code"* (OFFICIAL EXTERNAL, vendored `wasmtime-48.0.8/src/runtime/module.rs`, lines 372–398) | No. Native code must come only from the deterministic compiler, behind an authenticated, version-bound cache |

**Seven new defects, all re-verified by me before they were recorded** (§12; `REMAINING_WORK.md` 4.93, 4.94, 5.7,
6.67–6.70):
1. **A DIR whose node ids collide passes `dir::verify` while the same program as text is refused** (DL1604: a write
   through a `val` receiver). This is a reference-capability soundness hole on the Verified-plugin path. It is not an
   effect escape: a missing effect is still refused.
2. **A DIR skips the lexer's rules.** A function name holding U+202E and an ANSI escape verifies inside a `.dpx`, and
   `delulu why` prints both raw to the terminal; in source the same name is DL0107.
3. **`delulu why` on a `.dpx` reports from unverified facts.** It says a forged plugin performs no `Read`, while
   `plugin verify` refuses that same plugin (DL1504).
4. **A run killed by SIGTERM or SIGINT writes no `--report-out` report**, although `run --help` and
   `docs/for-agents.md` promise one *"in every outcome"*.
5. **Inside a package, `edit --node` checks the file alone.** An edit adding `Net` and a new host answered exit 0,
   `widened: null` and was written even under `--if-checks`; the package check then refused it (DL1009).
6. **A morph skin is looked up in the working directory, unpinned, and may map look-alike characters.** The same
   source bytes printed `yes` in one directory and `no` in another, with the same authority report.
7. **The DIR fuzz target never runs `verify`.**

RW 6.58 also gained its cause: a closed pipe makes `delulu` **panic** (*"failed printing to stdout: Broken pipe"*) and
exit 2, even for a program that succeeded.

**Found in the third pass (§19), and first in priority:**
- **RW 4.95, HIGH. `p.get` never checks the type a host asks for against the plugin's export.** The spec's rule R-Get
  is written as runtime functions that the interpreter never calls. A host whose `main` declares `! {Load, Read}`
  took a Verified plugin's `! {Write}` export as `! {}`, and it wrote a file. `check`, `authority` (`effects: Load,
  Read`) and a package's DL1009 ceiling all said it could not. The run's grant still bounded it, so this is an
  effect-row and ceiling hole, not an authority escape.
- **RW 4.96, LOW.** A program can load only Verified plugins: `Plugin[Contained]` checks clean and is refused at
  run time.
- **RW 6.71, LOW.** `plugin verify` counts private functions as exports.

**The recommendations, in order** (§13, §15, §16):
1. Land the 2.0 language basics and the answer contracts already in `REMAINING_WORK.md` (§8 items 7 and 8). Nothing
   below matters to an agent that cannot read its arguments or parse JSON.
2. Fix the seven defects above before any machine-first input path is widened.
3. Make edits package-aware and content-addressed, and keep text canonical.
4. Move embedded Python into the existing foreign-worker process (−1.1 to −2.0 ms per command).
5. Add `--engine auto` plus an **authenticated, version-bound precompiled-module cache**. The cache is off by default,
   and native code is never accepted from an author.
6. Pull `cli.rs` apart into services shared by CLI, MCP and LSP.
7. Keep the parser, the checker, the formatter, the LSP and the thin VS Code client. Freeze morph skins until
   they are pinned.

---

## 2. Snapshot, method and what changed in the tree

| Item | Value (OBSERVED HERE) |
|---|---|
| Date | 2026-11-05, 06:11 UTC at the start |
| Branch | `claude/dazzling-goodall-iaapyw`. It carries this session's earlier report commits (`a7f3515`, `43d66f5`, `dfb5b62`) on top of `master` `0670e0b` and differs from it only in documents (`git diff --stat origin/master HEAD -- crates Cargo.toml Cargo.lock rust-toolchain.toml editors scripts .github` is empty) |
| Sync | `git fetch origin`: no ref moved; `HEAD..origin/master` empty; no stash; clean tree. `origin/autosave/laptop` `052c2ce` is an older snapshot (based on `f45c565`) of a prompt file `master` already supersedes; `launch/2.0.0` `728a582` unchanged |
| CI read | `41929573373` (push, `0670e0b`): success, 14 jobs and 2 skipped by design (`heavy-gates`, `miri-slow`). It covers test on ubuntu-24.04, macos-latest, windows-latest and arm64, plus lints, supply-chain, formal, fuzz, editor, microvm and miri. No newer run exists |
| Toolchain | rustc and cargo 1.96.1 (pinned), Python 3.13.18, Node 22.23.4, GCC; Linux 6.18.44 x86-64, 4 vCPU, 15 GiB; no `/dev/kvm`; no `newuidmap` |
| Build | `DELULU_BUILD_COMMIT=$(git rev-parse origin/master) cargo build --locked --release -j 4 -p delulu -p delulu-survey`: 4 min 07 s. `delulu` is 41,768,296 bytes, SHA-256 `751c6458a8cedef33d09c3f853bfe2ede5904ce755a2881393c997d415b3fafd`. `--version --json` gives `commit: 0670e0bb4db8160469d1f092ab0b088aa4b76220`, `target: x86_64-unknown-linux-gnu`. It links `libpython3.13.so.1.0` |
| Health at start | `delulu-survey check`: ok (1619 nodes, 19143 edges). `findings`: 0 errors, 6 warnings. `doctor --check`: 26 checks passed (state in scratch) |
| Read in full or at the parts the audit needs | the prompt (all 188 lines); `AI_ONBOARDING.md` §§1–10; `HANDOFF.md`'s commission, §1, §2, §4, §7, §10, §11 (headings and §§11.2, 11.5, 11.8); `STABILITY.md` (whole); `CONSTITUTION.md` §1, §2, §5.11–5.13, §8, §9, §10; `DELULU_CORE.md` header, §6–§8; `CONTRIBUTING.md` §4; `for-agents.md` (the `--report-out`, `edit` and P9-03 passages); `INSTALL.md` (the build configurations); both reviews; `REMAINING_WORK.md` §§2–8. The rest of `for-agents.md`, `DEPLOYMENT.md`, `README.md` and `SOUNDNESS_AUDIT.md` was read in this session's earlier passes, not again |
| Sub-agents | two Sonnet 5.5 agents, each verified by quoting its own system prompt (*"You are powered by the model named Sonnet 5.5. The exact model ID is claude-sonnet-5-5"*): one for primary-source web research, one for an adversarial code read. Both were read-only on the repository. Appendix B lists what each did and what I re-ran |
| Tree change in this commit | this file; a dated follow-up section (§18) in the earlier review; the prompt placed in `docs/prompt/`; rows and notes in current-state documents (§2 of the sync-log entry lists each); the regenerated Survey map. **No `.rs`, test, configuration, manifest, workflow, editor or entrenched file** |

**Method.** Six kinds of evidence, each kept in the session scratch:
1. **Code read:** every stage below is cited to a file and line in `0670e0b`.
2. **An in-process stage harness:** a scratch Cargo project outside the repository that links the workspace crates by
   path and times each stage separately (Appendix A.2).
3. **Fresh-process CLI timings:** 21 runs per case after one warm-up, with peak memory read by a small C
   fork-and-exec launcher (Appendix A.3).
4. **Functional probes:** exit code, output and, where a probe checks an effect, what the file system or the server
   held afterwards (Appendix A.4).
5. **Re-runs of the sub-agents' claims.**
6. **Primary sources on the web**, cross-checked by search.

---

## 3. The source-to-effect map, as built (CURRENT SOURCE)

```text
 INPUTS
  .delulu text ─┐   package dir + delulu.toml/delulu.lock ─┐   agent edits: `edit --edits|--node` (blake3-guarded)
  CLI argv ─────┤   `fix` (typed repairs)   LSP (didChange) │   MCP (12 read-only tools; each call forks `delulu <cmd> --json`)
  .dwx (WASM + `delulu:authority`) ── skips the front end ──┼──────────────────────────────┐
  .dpx (plugin: DIR CBOR | Contained wasm; optional signature)                             │
                │                                         │                                │
 FRONT END      ▼                                         ▼                                │
  lexer.rs `lex` (+ `lex_with_morph`: keyword skins) → parser.rs `parse` → ast::Module      │
  → resolve.rs `resolve` (DeclTable) → check.rs `check_module` (types · rows · R-4 callbacks)
  → rcap_check.rs (val/ref/iso) → authority.rs (`authority`, `--within`) ; deps.rs `check_workspace` (packages, pins, DL1009)
  lib.rs `check_source` = lex+parse+resolve+check ; `flatten_sources` re-parses CONCATENATED TEXT for packages
  dir.rs `serialize` (typed AST + facts, CBOR) ⇄ `verify` (decode → validate → resolve+check → compare)  [Verified plugins only]
                │
 EXECUTION      ├── interpreter: runtime/interp.rs `Interp::new`/`run_main` on a big-stack thread (L0, in-process)
                ├── WASM: wasm/codegen.rs `compile_checked_sited` → bytes → host.rs `optimizing_engine` (Cranelift, Speed,
                │         hardened features) → `Module::new` EVERY RUN → instantiate → `run_main` (deny-by-default host)
                ├── `.dwx`: artifact.rs `read_and_verify` (code hash ↔ manifest) → same host, held to its manifest (DL1202)
                └── `--sandbox`: guest.rs `cmd_run_sandboxed` → jailed `delulu __guest` (Landlock+seccomp | AppContainer+Job
                          | Seatbelt), L2 Firecracker, L3 external launcher; the guest performs NO effect
                │
 HOST EFFECTS   prim.rs (the primitive table, `resolve_in_scope` → DL0904) → custody (embedded | broker daemon: grants,
                leases ⊑, revocation, the Guard, the hash-chained audit) → beneath.rs (no-follow paths) · egress.rs (one
                HTTPS client: hosts × methods × paths) · device broker (sim | hw adapters) · foreign C / embedded CPython
                │
 OUTCOMES       stdout = the PROGRAM's output · stderr = human diagnostics · `--json` envelope (except `run`, RW 6.60)
                · `--report-out` run report (not on a signal: RW 6.68) · exit 0/1/2/3 (+128+n on a signal) · audit chain
```

Trust boundaries:
- **between the checker and everything after it:** an engine runs only a program that checked;
- **between the guest and the host:** a sandboxed guest holds handles only;
- **between the host and custody:** every effect asks first;
- **between a `.dwx` or `.dpx` and the host:**
  - a `.dwx` is held to its manifest at run time;
  - a Verified `.dpx` is re-checked from DIR;
  - a Contained `.dpx` runs under limits.

| Stage | Exact code (CURRENT SOURCE) |
|---|---|
| CLI dispatch and gates | `crates/delulu/src/run_cmd.rs`: `cmd_run` (line 1006) and `cmd_run_inner` (line 1188); the isolation gate (lines 1204–1219), the engine gate (lines 1220–1228), the WASM branch (line 1423), `run_dwx_artifact` (line 238) |
| Lex, parse | `crates/delulu-syntax/src/lexer.rs` `lex` (line 68), `lex_with_morph` (line 86); `parser.rs` `parse` (line 14); `lib.rs` `parse_file` (line 23) |
| Resolve, check | `crates/delulu-check/src/resolve.rs` `resolve` (line 186); `check.rs` `check_module` (line 188); `lib.rs` `check_source` (line 69), `flatten_sources` (line 103) |
| DIR | `crates/delulu-check/src/dir.rs`: `DIR_VERSION` (line 34), `PRIM_TABLE_VERSION = 9` (line 65), `decode` (line 169), `serialize` (line 187), `verify` (line 225), `validate_module` (line 270) |
| Plugin load | `crates/delulu-runtime/src/plugin.rs` `step5_verified` (lines 515–557) calls `dir::verify`; `crates/delulu-wasm/src/dpx.rs` binds DIR by `dir_blake3` in the artifact's own manifest (lines 280–284) |
| Interpreter | `crates/delulu-runtime/src/interp.rs` `Interp::new` (line 350), `run_main` (line 710), `on_interpreter_thread` (line 138) |
| WASM | `crates/delulu-wasm/src/codegen.rs` `is_compilable` (line 1027), `compile_checked_sited` (line 1321), exports by name (line 2083); `host.rs` `optimizing_engine` (lines 29–40), `harden_wasm_features` (lines 65–89), `run_main` (line 2966); `limits.rs` (fuel and epoch interruption, lines 250–280); `Cargo.toml` (Wasmtime 48 features, `cache` deliberately off, lines 39–51) |
| `.dwx` | `crates/delulu-wasm/src/artifact.rs` `embed_authority` (line 75), `read_and_verify` (line 101) |
| Verified WASM cache | `crates/delulu-wasm/src/lib.rs` `verified_executable_wasm` (lines 57–62), called only from tests (lines 1149, 1160) |
| Edit | `crates/delulu/src/edit.rs`: the whole-file blake3 (line 31); the package prefix discarded (line 138) |
| MCP | `crates/delulu/src/mcp.rs` `run_self` (line 367): a fork per call, `read_to_end` |
| Toolchain description | `crates/delulu/src/toolchain.rs` (lines 87 and 77): `engines: ["interpreter", "wasm"]`, the primitive table, no per-engine support |

---

## 4. Stage by stage: job, kind, bypass, cost

Kinds: **S** a semantic or security requirement · **I** an implementation choice · **H** a human-UX feature · **O**
an optional optimisation. Costs are the in-process medians of §10.1 at 2 / 1,002 / 16,002 lines.

| Stage | Precise job | Kind | Can a machine-first form skip it without skipping its guarantee? | What breaks if skipped | Tests that prove it | Cost | Concrete alternative |
|---|---|---|---|---|---|---|---|
| Lexing | tokens; bans bidi and control characters (DL0107), checks literals | I + S (the bans) | Only if every machine input re-applies the same character and literal rules. DIR does not today (RW 4.94) | terminal and review spoofing; homoglyph names | lexer tests, DL0107 conformance | 1 µs / 0.7 ms / 21 ms | keep |
| Parsing | the tree; nesting caps DL0210–0213; spans | I + S (the caps) | Yes, if the input is a validated tree with the same caps and **unique node ids** (RW 4.93) | the checker's side tables keyed by node id read the wrong entry | parser tests, formatter laws over the corpus | 3 µs / 4.3 ms / 64 ms | keep text; optional typed-tree input only after RW 4.93/4.94 |
| Resolution | names, imports, modules, the prelude | S | No | unknown names, shadowing rules (LATE-LET-1 class) | conformance, laundering suite | 65 µs / 0.5 ms / 8.9 ms | keep |
| Type, effect, rcap check | Theorems 1–3's premises; DL0501, DL1604 … | **S (the product)** | **No** | every guarantee | conformance (366 anchors), laundering, Lean fragment, differential fuzzer | 6 µs / 7.2 ms / 259 ms | keep; faster by a shared lookup table, not by skipping |
| Authority report, `--within` | what the program may ask for | S (for operators) | No | preflight, P9 | `within_cli`, authority tests | part of check | keep |
| Package check | ceilings, pins, visibility (DL1009, DL1010) | S | No | supply-chain guarantee | `package_run`, lockfile tests | — | keep; make `edit` use it (RW 6.69) |
| Diagnostics: codes, spans, repairs | the machine contract | S (stable codes) | No | repair loops | `cli_contract`, `json_contract` | small | keep |
| Diagnostics: prose, colour, catalogs | human reading | H | Yes: `--json` already omits prose; codes are stable | nothing for agents | locale invariance tests | ~0 | keep, optional |
| DIR | a checked program as data; Verified plugins | I | it IS the machine form | — | `dir.rs` tests, fuzz decode-only (RW 5.7) | 0.1 ms / 30 ms / 716 ms to verify; 28× text's size | keep for plugins; do not make it the authoring form |
| Interpreter | the reference semantics | S (as reference) + I | No (it is the oracle for WASM parity) | parity's reference | the differential fuzzer, parity corpus | 27 µs hello; 866 ms per 5M-loop | keep; compiled form RW 3.24 |
| WASM codegen | a portable module with capability imports | I + O | Yes for short runs (use the interpreter) | — | `*_parity.rs`, the 50,000-program differential | 15 µs / 4.6 ms / 165 ms | keep |
| Cranelift compile | machine code for the module | I | Yes, by a cache of its own output (§5, model 6) | — | Wasmtime's own | **1.6 ms / 39 ms / 652 ms** | cache, authenticated (PROPOSED) |
| `.dwx` | a portable, authority-carrying artifact | I + O | — | — | `wasm_cli`, `dwx` tests | 1.1 KB hello | keep; add a precompiled sibling only under §5.4's rules |
| Host primitives and custody | every effect performed and recorded | **S** | **Never** | the language's claim | conformance, broker laws, Miri, TLC | per effect | keep |
| Sandbox (L1–L3) | containment of the process | S where claimed | No (and it never silently downgrades) | the boundary | hostile-guest red team, `sandbox_confirm_cli` | +48 ms per run (L1) | keep |
| Formatter | one canonical text | H + I (canonical diffs, node edits) | Partly; `edit --node` uses it | stable node edits, review diffs | the corpus law | small | keep |
| Morph skins | keyword aliases for humans or compact profiles | H | Yes | nothing; none ships | morph tests | — | freeze until pinned (RW 6.70) |
| LSP, VS Code | inspection, editing, recovery | H + useful to agents | Yes, for agents that use the CLI | human emergency inspection | `editor` CI job, 15 extension tests | 0.16 ms per edit in one long-lived `delulu lsp` process | keep, thin |
| REPL, completions, welcome | human onboarding | H | Yes | nothing for agents | their tests | ~0 | keep (tiny) |

**What the table settles:**
- Only lexing and parsing are *bypassable implementation*. They are the cheapest stages, and they carry two security
  rules (character bans, nesting caps) that any bypass must carry too.
- Everything that costs real time is either the guarantee (checking) or an engine choice (Cranelift). The second is
  where an AI-first pipeline saves time: by choosing the engine and caching its output, never by skipping a check.

---

## 5. The execution models, and the direct-execution answer

| # | Model | Mandatory transformations | Optional | Where the instructions come from | Status in DeluluLang | Verdict |
|---|---|---|---|---|---|---|
| 1 | text → tree → **interpreter** | lex, parse, resolve, check | — | the interpreter's own Rust code walks the tree | **BUILT** (default engine) | Keep as default for short and cold tasks: 4.5 ms hello, but 27× slower than WASM on loops |
| 2 | AI → **validated typed tree / IR** → interpreter | structural validation **with the lexer's and parser's rules**, unique ids, resolve, check | text | as 1 | **Partly BUILT** as DIR for Verified plugins; not an authoring input | Not worth it as a second input: DIR verify is 1.9× slower than the text front end and 28× larger, and has two holes (RW 4.93, 4.94) |
| 3 | checked program → **portable bytecode / WASM** → JIT or interpreter | as 1, then codegen; validation by Wasmtime | WASI | Cranelift-generated machine code (or Pulley's interpreter) | **BUILT** (`--engine wasm`, `.dwx`) | Keep. It is DeluluLang's portable target (Constitution §5.13) |
| 4 | WASM → **precompiled target-specific artifact** | as 3, then `Engine::precompile_module` | — | machine code made earlier by the same Cranelift | **Not built** (no `precompile_module` or `deserialize` in the tree; the sub-agent's grep, re-checked) | PROPOSED as a **cache**, never as an interchange: 35 µs–5.7 ms to load against 1.6–652 ms to compile, but only behind §5.4's rules |
| 5 | **native AOT** or a JIT of DeluluLang itself | a new backend (LLVM ORC/MLIR or similar) | — | a native code generator | **Not built** (RW 3.2; D18) | Reject for now: WASM through Cranelift already runs the loop within 3.3× of `rustc -O`. A native backend needs its own soundness and parity argument |
| 6 | **hybrid** | as 1 or 3 per program | the cache | both | **Not built** (RW 3.3; the engine is chosen by hand) | PROPOSED: `--engine auto`, a static policy first (loops → WASM), then measured tiering |
| 7 | **operation or workflow plans** interpreted without a new agent framework | a plan is a DeluluLang program; its steps are capability calls | — | as 1 or 3 | expressible today (the drone and telemetry simulations ran) | Keep plans as ordinary programs. A durable workflow store is RW A14's gap, not a new framework |

**Must every program pass every conventional stage and become a binary?** No.

**Must pass:**
- the check;
- an engine bound to the host's primitive table.

**Need not:**
- **parsing**, if the input arrives as a validated tree; it does not save much;
- **machine code**: the interpreter runs the tree;
- **a standalone binary**: no DeluluLang run produces one. The engine is the `delulu` binary, which must exist per OS
  and CPU.

**Where the instructions come from:**
- **Interpreter:** the `delulu` binary's own compiled Rust.
- **WASM:** Cranelift translating the module at run time.
- **Precompiled (PROPOSED):** Cranelift's output saved from an earlier run.

Removing the parser or the editor removes a *human interface*. It never removes validation, because validation is the
checker, which every model above keeps.

**Portability, kept apart** (§9 is the matrix):
- **source portability:** one `.delulu`, every OS `delulu` runs on;
- **portable bytecode:** a `.dwx` on any `delulu` host;
- **target-specific precompiled code:** one Wasmtime version × CPU × OS × configuration;
- **a native executable:** per OS and CPU;
- **sandbox support:** per OS and level;
- **device drivers:** per adapter and device.

**No single native binary runs unchanged on every OS, CPU or device**, and nothing here claims one.

### 5.4 Rules for a precompiled cache (PROPOSED, not built)

Wasmtime's own documentation says why these rules are needed (OFFICIAL EXTERNAL):
- vendored `wasmtime-48.0.8/src/runtime/module.rs` lines 372–398: *"The structure of the binary blob read here is
  only lightly validated … Arbitrary input could, for example, replace valid compiled code with any other valid
  compiled code"*;
- `examples-pre-compiling-wasm.html`: *"only create `Module`s … from pre-compiled bytes you control and trust!"*.

| # | Invariant | Acceptance test |
|---|---|---|
| N1 | Native code loads only when authenticated, by a key or MAC the operator's policy holds, never by the artifact's own hash. An author, human or AI, never supplies it | a cache entry signed by another key → recompiled, never run |
| N2 | The authenticated tuple is (blake3 of the exact WASM, the Wasmtime version, the target triple, CPU features, the engine-configuration fingerprint, the host-import ABI version, the authority manifest's hash) | flip each element and the blob in turn → refused and recompiled |
| N3 | Read once into memory, verify, then deserialize those same bytes (no file-then-map race) | a swap between check and load cannot be observed |
| N4 | The cache is off by default and is a cache: losing it changes only speed | delete it mid-run → recompiles, same output |
| N5 | Budgets hold the same on cached code: the limits engine's fuel and epoch configuration is part of the fingerprint | a cached loop is still stopped by `--limits` |

Constitution §5.11 rule (2) (*"only human-controlled policy may grant a module the right to emit native code"*)
already requires N1. §14 suggests making it kind-blind.

---

## 6. Everything human-centric, classified

One category per item. *Human-readable* is not *human-only*: a readable artifact a reviewer can check is a safety
feature for autonomous systems.

| Item | Category | Evidence |
|---|---|---|
| The type, effect, rcap and authority check | **necessary for correctness/security** | the product |
| Diagnostic **codes**, spans, typed repairs with `authority_widening` | **necessary** | STABILITY §1; repairs that widen are refused unless named (`fix --accept-widening`, OBSERVED) |
| Text syntax: braces, short keywords, explicit effects | **useful to both** | Constitution §5.11 rejects whitespace syntax because it tokenises worse and edits worse for agents; text is 28× smaller than DIR (OBSERVED) |
| Comments | **useful to both** | DIR drops them (SUB-AGENT, re-checked in `ast.rs`): reviewers and agents lose intent |
| One canonical formatter, no options | **useful to both** | node edits and clean diffs rest on it |
| `--json` envelopes and closed schemas (12) | **useful to both** (machine-first) | `delulu schema --json` (OBSERVED) |
| `toolchain --json` | **useful to both**, incomplete | no per-engine support (OBSERVED; RW 6.61) |
| Diagnostic prose, explain bodies, catalogs, `--locale`, colour, themes | **human-optimized, worth retaining optionally** | STABILITY §2 already makes prose unstable and `--json` locale-independent |
| The welcome text | **human-optimized, retain** | byte-exact by Constitution §8.5; never shown to agents or in `--json` |
| REPL, shell completions, `delulu new` templates | **human-optimized, retain** | tiny |
| Morph keyword skins | **human-optimized; freeze until pinned** | none ships (`morph list --json`: `[]`); cwd lookup and look-alike aliases change behaviour (RW 6.70) |
| LSP | **useful to both** | the only persistent analysis process: 0.16 ms per edit, flat memory over 1,000 edits (this session's second pass) |
| VS Code extension (409 lines) | **human-optimized, worth retaining** | emergency inspection; thin |
| Atlas | **useful to both** | node ids drive `edit --node` (OBSERVED) |
| Survey and `doctor` | **useful to both** (maintainers, any kind) | blast radius, health; module granularity weak (RW 6.63) |
| Conformance, laundering, fuzzing, Lean/TLC/Z3, Miri | **necessary** | the evidence behind the claims |
| Source maps (`SiteMap`) | **useful to both** | WASM faults placed at source lines (B5) |
| Audit chain, run reports, OCSF export | **necessary** (operators and any reviewer) | revocation shown by audit seq (OBSERVED, DL1403) |
| MCP server | **useful to agents**; implementation redundant with the CLI | forks the CLI per call (RW 6.62) |
| `cli.rs` as one 11,866-line dispatcher | **redundant/duplicated** (structure, not features) | MCP and LSP rebuild argv or share nothing |
| `verified_executable_wasm` (the Verified plugin's shipped WASM cache) | **genuinely removable with evidence** or to be wired: dead in production | called only from tests (CURRENT SOURCE; RW 3.27) |
| Debugger integration, breakpoints | **unknown** (none exists) | no DAP server in the tree |
| Text **flattening** of packages for execution | **implementation choice with a cost** | node ids overlap per module, so packages re-parse concatenated text (`check/src/lib.rs` lines 103–121); a typed-tree input would need global ids first |

**The earlier conclusion, "no component has evidence of zero value", survives with one exception:** the Verified
WASM cache path, which no production code calls. Everything else that looks human-only is either cheap, or it is the
human-readable half of a machine contract (prose beside a code; a formatter that makes node edits stable).

---

## 7. Path-level decision table

| Path | Today | Candidate change | Benefit / drawback | Risk and proof dependencies | Compatibility | Measured saving | Rollback | Acceptance test |
|---|---|---|---|---|---|---|---|---|
| `crates/delulu-runtime` `python` feature (linked at load) | every invocation loads `libpython` | run embedded Python only in the existing foreign worker (`--foreign-isolation process`), so the main binary links none | −1.1 to −2.0 ms per command / one more process for Python programs | none to the checker; Python already sits outside the proof (Constitution §5.12) | `root.python` unchanged; worker on by default | **1.1–2.0 ms of 3.35–4.84 ms** per invocation (OBSERVED, §10.2) | re-enable the in-process feature | `--version` floor within 0.2 ms of the portable build; Python tests green |
| `crates/delulu-wasm/src/host.rs` `Module::new` every run | Cranelift every run | an authenticated precompiled cache under §5.4 | 45–115× faster WASM start / native-code trust surface | N1–N5; Constitution §5.11 (2) | additive flag; off by default | 1.6 ms → 35 µs; 652 ms → 5.7 ms | delete the cache | N1–N5's tests |
| `run_cmd.rs` engine gate | user chooses `interp` or `wasm` | `--engine auto`: static policy (loops or `while` → WASM if compilable, else interpreter), later RW 3.3 tiering | right engine by default / one more mode to test | parity corpus must cover every construct both ways | additive | up to 26× on loops, avoids 650 ms compiles for cold code | default stays `interp` | a parity run over the corpus under `auto` |
| `edit.rs` | single-file check and delta; package prefix ignored | package-level check and delta (`check_workspace`); a full id with a content hash | closes RW 6.69 / slower edits in large packages | delta must match `authority` for the package | stricter: refuses what packages refuse | — | flag to the old behaviour for one release | the greeter edit is refused; adding `Net` anywhere is reported |
| `dir.rs` | ids compared, not checked unique; no lexer rules | unique-id and lexer-rule validation; `why` uses `verify`; fuzz `verify` | closes RW 4.93, 4.94, 6.67, 5.7 | none | refuses forged DIRs only | — | none needed | the collision, bidi and forged-facts probes refused |
| `mcp.rs` | a fork per call, unbounded read | cap the read now; in-process services later (keep a child for crash containment until panics are isolated) | memory bound / in-process loses killability | services must not print to stdout | same schemas | 5.6 ms → ~0.1 ms per call (INFERENCE from the 85 µs front end) | keep the fork | 10 MB output refused; a panicking call leaves the next healthy |
| `cli.rs` (11,866 lines) | dispatcher holds logic | services with typed requests and results | one contract for CLI/MCP/LSP / a large refactor | `core_invariance` snapshots per moved command | none visible | maintenance | per command | snapshots byte-identical |
| morph lookup (`morph_file.rs`) | cwd first, unpinned, look-alikes allowed | pin by blake3 in the pragma or lock; refuse confusables; no cwd for meaning | reproducibility / none ship today | — | refuses unpinned skins | — | — | the yes/no probe gives one answer or a refusal |
| `run` signal handling | no handler; no report on SIGTERM | write the report on SIGTERM/SIGINT, `stopped_by: signal` | the contract holds / a handler | the guest still ends with its host (PS-E-02) | additive field | — | — | kill a run; the report says why |
| `verified_executable_wasm` | test-only | remove, or wire behind N1–N5 | less dead code | spec §3.2 already says the guarantee never rests on shipped code | none | — | git | — |

---

## 8. The AI-first authoring and execution loop

**The minimum an agent needs** (INFERENCE from §6.5 of the earlier review and this session's probes). Each item is
**built**, **partial** or **missing**:

| Need | Today | Status |
|---|---|---|
| A discoverable reference matched to the installed toolchain | `toolchain --json`, `schema`, `skill`, `examples` | **partial**: no per-engine or per-level support (RW 6.61) |
| Structured edits with ids, hashes, conflicts and rollback | `edit --edits` (repair shape), `edit --node` (Atlas ids), whole-file blake3, atomic rename, `--dry-run`, `--if-checks` | **partial**: single-file delta (RW 6.69); a file-level transaction only |
| Repairs that never silently widen | `fix` refuses widening repairs unless named (OBSERVED, L2) | **built**; exit code wrong (RW 6.55) |
| Stable outcome categories | check success, expected denial (DL0703, DL0904), unsupported target (DL1201), application failure, infrastructure failure | **partial**: zero tests is success (RW 6.51); handled application errors exit 0 (U2, U3); a signal leaves no report (RW 6.68); a closed pipe panics (RW 6.58) |
| One output protocol | `--json` envelopes; `--report-out` for runs | **partial**: `run --json` has two shapes (RW 6.60) |
| Preflight before writing or running | `authority --within`, `sandbox policy`, `run --mode audit` (OBSERVED C5: `ran: false` with the exact `required_grants`) | **partial**: no target preflight (RW 6.61) |
| Bounded output, timeouts, budgets, cancellation | `--limits` (wall stops at 1.017 s, OBSERVED C3/C4), human diagnostics capped at 50 | **partial**: MCP unbounded (RW 6.62); no report on signal |
| Minimal grants and proposals | `grants propose` → narrowest grant from the refusal (OBSERVED L5) | **built** (approval as strong as the OS account, RW 4.91) |
| Handoffs and delegation | leases, attenuation, revocation cascade (OBSERVED D1–D4) | **built** |
| Provenance | `--version` commit when injected (OBSERVED); `.dwx` code hash; signatures on `.dpx` and `.dwx` | **partial**: source installs report `commit: null` (RW 6.53) |
| A human-readable projection for inspection | text itself, the Atlas, the LSP | **built** |

**The loop as run in this session** (OBSERVED HERE, Appendix A.4):
1. `check --json` gives DL0501 with an exact repair marked `authority_widening: true`.
2. `fix` refuses that repair unless it is named; `fix --accept-widening add_effect_to_row` applies it, `errors_after: 0`.
3. `edit --node fn:review.utility/review.utility.task` reports authority before and after, with `widened: []`. An
   ill-typed widening is refused under `--if-checks`.
4. `authority --within` answers `exceeds`, naming what is outside.
5. A refused run's `--report-out` report becomes `grants propose`'s `["fs.write=./output"]`.
6. `run` follows.
7. A malformed input prints `{"ok":false,"error":"format"}` with **exit 0** (the outcome blocker, RW 2.40).
8. A missing grant is DL0703, exit 1.

**Delegation and revocation** (OBSERVED HERE, scratch broker):
- A child lease narrower than its parent read its own file and was refused the parent's (DL0703, naming the lease).
- Widening the child by delegation was DL0802.
- **Revoking the parent while the child's read loop ran stopped it at the next read**: DL1403, *"revoked by audit seq
  13"*, at `loop_read.delulu:8:11`, with the diagnostic in the run report.
- The audit chain then verified (14 records).

**Compared with Python, Rust and Zerolang** (OBSERVED for Python and Rust; OFFICIAL EXTERNAL for Zerolang, which was
not run):

| | DeluluLang | CPython 3.13 | Rust (`rustc -O`) | Zerolang (vendor docs) |
|---|---|---|---|---|
| Hello, fresh process | 4.5 ms, 13.9 MiB | 12.9 ms, 10.3 MiB | — | not run |
| 5M-iteration loop | 910 ms (interpreter), **34 ms (WASM)** | 246 ms | 10 ms, 2.0 MiB | not run |
| Authority of a program computed before it runs | yes (`authority`, `--within`) | no | no | effects in signatures (`fn main(world: World) … raises`, THIRD-PARTY docs) |
| Refusal of an ungranted effect at run time | yes, with no forbidden effect (OBSERVED U4: sentinel unchanged) | only by an outer sandbox | only by an outer sandbox | none documented; *"Run it in isolated workspaces"* |
| Structured edit | node id + file hash | none | none | graph patch with an expected graph hash (OFFICIAL EXTERNAL) |
| Output | interpreter or WASM via the runtime | bytecode via the interpreter | native executable | native executables, *"sub-10 KiB"* target (THIRD-PARTY site; not reproduced) |

Token and tool-call overhead per completed task were **not measured**. That needs the task-completion study in the
earlier review's §15.3, run after the 2.0 basics land.

---

## 9. Artifact and OS portability matrix

| Artifact | Portable across | Bound to | Verified by | Evidence |
|---|---|---|---|---|
| `.delulu` source | every OS and CPU `delulu` builds on | the language edition (`[package] language`) | the checker, every run | STABILITY §4 |
| `delulu` binary | one target triple | OS, CPU, libc, and `libpython3.13` in the default build | `--version` commit (when injected) | OBSERVED: 41.8 MB, links libpython |
| `.dwx` (core WASM + `delulu:authority`) | any `delulu` host of the same `DWX_VERSION` and import ABI | `delulu:cap` imports; **not WASI** | code hash ↔ manifest; the manifest enforced at run time (DL1202) | OBSERVED: 1,087 B hello; a one-bit flip → DL1202 |
| `.dpx` (DIR or Contained WASM) | any host of the same `DIR_VERSION` and `PRIM_TABLE_VERSION` (9) | the primitive-table version | Verified: re-check from DIR; Contained: limits | OBSERVED; RW 4.93/4.94 holes |
| Precompiled module (PROPOSED) | **nothing**: one Wasmtime version × target × CPU features × configuration | all of those | N1–N5 | OBSERVED sizes: 19 KB hello, 5.4 MB at 16,002 lines |
| Pulley bytecode (PROPOSED fallback) | 32/64-bit, either endian, any CPU | Wasmtime version | as above | OBSERVED: 11× slower than Cranelift |
| Sandbox level | L1 on Linux (Landlock+seccomp), Windows (AppContainer+Job), macOS (Seatbelt, reads not confined); L2 Linux+KVM; L3 operator's | OS | `host_guarantees`, `fully_enforced` | CURRENT CI (three OSes); SUB-AGENT for macOS reads, matching `DEPLOYMENT.md` |
| Device drivers | per adapter | the device | signed Verified `.dpx` drivers (P8-02) | earlier review §6.7 |
| CI-verified targets | ubuntu-24.04 x86-64, macos-latest (arm64), windows-latest, Linux arm64 | — | `41929573373` | CURRENT CI. RTOS, MCU, flight and vehicle platforms: **unverified** |

---

## 10. Measured probes

### 10.1 In-process stage timings (OBSERVED HERE; medians in µs; Appendix A.2)

| Stage | hello (2 lines, 72 B) | kernel (15 lines) | 1,002 lines (51 KB) | 4,002 lines (208 KB) | 16,002 lines (838 KB) |
|---|---:|---:|---:|---:|---:|
| lex | 1 | 2 | 674 | 4,084 | 20,659 |
| lex + parse | 3 | 9 | 4,259 | 12,619 | 63,749 |
| resolve | 65 | 46 | 504 | 1,048 | 8,909 |
| check | 6 | 14 | 7,150 | 53,078 | 259,432 |
| `check_source` (all front end) | 85 | 93 | 11,743 | 77,241 | 380,804 |
| DIR size (bytes) | 1,175 | 3,778 | 1,358,657 | 5,667,017 | 23,643,677 |
| DIR serialize | 7 | 15 | 8,616 | 44,157 | 214,841 |
| DIR decode + validate | 14 | 42 | 15,990 | 74,633 | 322,045 |
| DIR verify (decode + re-check) | 120 | 157 | 30,002 | 163,185 | 716,246 |
| WASM codegen (module bytes) | 15 (795 B) | 22 (863 B) | 4,620 (53 KB) | 32,694 (213 KB) | 165,172 (855 KB) |
| Cranelift compile | 1,594 | 1,729 | 38,661 | 166,290 | 652,098 |
| Winch compile | 1,073 | 1,362 | 32,962 | 130,917 | 554,210 |
| Pulley compile | 1,460 | 1,687 | 42,723 | 148,441 | 689,550 |
| precompiled size, Cranelift (bytes) | 19,016 | 19,224 | 354,920 | 1,364,176 | 5,415,664 |
| deserialize precompiled, Cranelift | **35** | 39 | 251 | 1,290 | **5,693** |
| `delulu_wasm::run_main` (compile + instantiate + run) | 1,791 | 26,902 | 43,467 | 152,616 | 676,191 |
| interpreter `run_main` | **27** | 866,255 | 2,964 | 11,842 | 84,253 |
| `work(5,000,000)`: Cranelift / Winch / Pulley | — | **27,145 / 38,664 / 309,587** | — | — | — |

The bare-engine numbers use a configuration that copies `harden_wasm_features` but is not the shipped engine object.
The shipped engine's own figure is the `run_main` row.

### 10.2 Fresh-process CLI (OBSERVED HERE; 21 runs after a warm-up; peak RSS by a C fork-and-exec launcher)

| Case | Median | p95 | Peak RSS |
|---|---:|---:|---:|
| `check hello` | 4.26 ms | 5.51 ms | 13.2 MiB |
| `run hello` (interpreter) | 4.51 ms | 5.53 ms | 13.9 MiB |
| `run hello --engine wasm` | 7.96 ms | 11.68 ms | 23.7 MiB |
| `run hello.dwx` | 11.52 ms | 16.30 ms | 21.9 MiB |
| `run hello --sandbox` (L1) | 52.03 ms | 54.93 ms | 16.0 MiB (host) |
| `run kernel` 5M (interpreter) | 909.93 ms | 979.38 ms | 14.3 MiB |
| `run kernel --engine wasm` | 34.13 ms | 46.80 ms | 23.7 MiB |
| `run kernel.dwx` | 36.32 ms | 45.43 ms | 22.0 MiB |
| `check` 1,002 / 4,002 / 16,002 lines | 25.4 / 118.4 / 493.8 ms | — | 25.6 / 65.3 / 223.8 MiB |
| `build` 4,002 lines `--target wasm` | 148.2 ms | 195.5 ms | 66.6 MiB |
| `run` 4,002 lines: interpreter / `.dwx` | 135.5 / 179.3 ms | — | 72.6 / 50.0 MiB |
| CPython 3.13: hello / `-S` hello / 5M loop | 12.9 / 12.2 / 245.7 ms | — | 10.3 MiB |
| Rust `-O` 5M loop | 10.2 ms | 10.9 ms | 2.0 MiB |
| process floor: `--version` default / Python-less / `/bin/true` (31 runs) | 3.35 / 2.21 / 0.90 ms | — | — |
| process floor, both builds stripped, two rounds of 40 runs | default 3.72 and 4.84 ms; Python-less 2.30 and 2.86 ms | — | — |

Two conclusions follow:
- **A `.dwx` is not faster than source today.** It skips the 0.1 ms front end but pays Cranelift's 1.6 ms or more and
  the artifact check.
- **The interpreter beats WASM for large programs that do little work**: at 4,002 lines, 136 ms against 179 ms.

### 10.3 The earlier reviews' findings, re-tested as hypotheses (OBSERVED HERE; Appendix A.4)

| Hypothesis | Result on `0670e0b` | Status |
|---|---|---|
| no args/env/exit (RW 2.40) | `root.args()` → DL0405 | still open |
| no JSON, math (RW 2.41, 2.38) | `parse_json`, `sqrt` → DL0301 | still open |
| no methods on own types (RW 2.33) | `p.norm()` → DL0405 | still open |
| interpolation prints braces (RW 2.37) | `hello {name}` printed literally, exit 0 | still open |
| three WASM refusals (RW 3.25) | `range`, non-generic HOF, `if` without `else` as an arm value: DL1201 each; the generic and block forms run on both engines | still open, exactly as narrowed |
| `test` with a missing path or no match (RW 6.51) | exit 0, `passed: 0, failed: 0` | still open |
| `fix` exit (RW 6.55) | `--dry-run --json`: exit 0 with `errors_after: 1` | still open |
| `run --json` two shapes (RW 6.60) | success: the program's JSON; refusal: the CLI's envelope | still open |
| null device (RW 4.88) | `COMMANDED`, then `REFUSED` (envelope), then `NO READING`; the report has no device field; with `sim`: `READ 12.0` | still open |
| hints (RW 6.54) | `actuator=review/motor:DIM=LO..HI`; `foreign.python=numpy` | still open |
| closed pipe (RW 6.58) | small output: exit 1 both ways. **Large output: exit 2, from a panic**, *"failed printing to stdout: Broken pipe (os error 32)"* in thread `delulu-main`, including a program that exits 0 into a file (3 of 3) | open; cause found |
| target preflight (RW 6.61) | `toolchain --json`: `engines: ["interpreter", "wasm"]`, primitives without per-engine support | still open |
| source-install identity (RW 6.53) | `commit` filled only when `DELULU_BUILD_COMMIT` is set (this build: set) | still as recorded |
| MCP output cap (RW 6.62) | `read_to_end` unchanged (CURRENT SOURCE) | still open |
| Survey granularity (RW 6.63) | not re-run; code unchanged | still open |

**Nothing the earlier reviews recorded has been fixed since `0670e0b`.** No code has changed since then.

---

## 11. Competitors and technologies (OFFICIAL EXTERNAL unless marked; fetched or searched 2026-11-05)

| Technology | What it is, at the inspected version | What it illuminates for DeluluLang |
|---|---|---|
| **Zerolang** (Vercel Labs) | *"an experimental graph-native programming language where the semantic graph is the program database"*; `zero.graph` is the checked input; `.0` files are *"human-readable projections"*; `zero patch` *"applies checked graph edits and rejects stale or invalid changes"*; README: *"Expect breaking changes, rough edges, and security issues. Run it in isolated workspaces"*. Apache-2.0. Releases to v0.3.4 (June 2026; the page shows no year), commits to 2026-09-16 (SUB-AGENT via the page, confirmed in outline by search). zerolang.app is a **community** site (THIRD-PARTY): *"Compiles to native executables targeting sub-10 KiB binaries"*, no method given | The hash-guarded, node-addressed edit is the idea worth having, and DeluluLang already has it over text (`edit --node --expect-hash`). The graph-as-database is not: here it would be DIR, measured 28× larger and slower to verify than text. No sandbox or capability enforcement is documented. **Not installed or run**: its installer is a remote shell script, and the session held push credentials |
| **Mojo** (Modular) | 1.2.0 of 2026-10-22; *"mojo build and mojo run both perform ahead-of-time (AOT) compilation"* (FAQ, SUB-AGENT raw read; an older second-hand quote says the REPL was JIT-compiled); MLIR and LLVM; *"supports Windows via WSL"* (confirmed by search) | A different problem: native performance across CPU and GPU. Its AOT path does not bear on authority |
| **CPython** | tokenize → AST → symbol table → instruction sequence → CFG → bytecode; a specializing adaptive interpreter; `.pyc` marshal caches invalidated by `MAGIC_NUMBER`; an experimental copy-and-patch JIT behind `--enable-experimental-jit` (InternalDocs, SUB-AGENT raw read) | "Compiling" without a native executable is normal: Python compiles to bytecode and caches it, keyed by a version number. DeluluLang's equivalent cache is §5.4's, and it needs authentication because its bytes are machine code, not bytecode |
| **Rust, Go, Java** | AOT native (Rust, Go); JIT with tiering (HotSpot) | Constitution §5.11 already names tiering as the normal design; measured here, WASM via Cranelift is 3.3× `rustc -O` on the loop |
| **Wasmtime** | 50.0.0 of 2026-10-20 (confirmed by search; 50.0.1 confirmed in the third pass as a patched version of advisory GHSA-6fr9-xmw9-5352, §19); LTS every 12th release, supported 24 months, so **48 is an LTS line**; Cranelift default; Winch *"performs only a single, quick pass … will run Wasm programs slower"*; Pulley *"expect a ~10x … slowdown"*; `precompile_module` output *"is safe to send to another host"*, but loading it is `unsafe`; tiers: Tier 1 x86-64 Linux, macOS, Windows | Measured here: Pulley 11.4× slower than Cranelift; Winch 42% slower code with 15–33% faster compiles. **Winch cannot serve the limits engine**: epoch interruption is documented as incompatible with it (vendored 48.0.8 `config.rs`, lines 776–784) |
| **WAMR** | 2.4.5 (June; year inferred); six modes (fast interpreter, classic interpreter, AOT via `wamrc`, LLVM JIT, Fast JIT, multi-tier); core text size on Cortex-M4F: about 58.9K (fast interpreter), 56.3K (classic), 29.4K (AOT runtime) (confirmed by search) | The candidate for RW 3.6's microcontroller tier, **not** for desktop or server: it would be a second host ABI for `delulu:cap` and a second engine to hold to parity |
| **WASI 0.3 / Component Model** | WASI 0.3.0 shipped 2026-06-11; components type their interfaces in WIT; *"a component may not export a memory"* | `.dwx` is a core module with custom `delulu:cap` imports (OBSERVED earlier). A component would buy typed interfaces and composition, not authority. Study it for `.dwx` 2, after the basics |
| **LLVM ORCv2, MLIR** | *"ORC provides a modular API for building JIT compilers"*; MLIR is a multi-level IR framework with dialects | The road to a native backend (RW 3.2). Rejected for now (§5, model 5) |
| **NVIDIA OpenShell** | v0.1.7 of 2026-11-03 is still the newest; no library or embedded mode without the gateway was found (SUB-AGENT keyword search: a negative result, not proof) | No change to the earlier verdict, **optional backend**. It sits outside the compiler and runtime boundary entirely |

---

## 12. Risks and security invariants, with the defects found in this audit

**Invariants every pipeline change must keep** (the earlier review's §8.4, plus these):
1. A machine-first input passes the same character, literal and nesting rules as text.
2. Node ids are unique (and canonical) before any table is keyed by them.
3. A review surface (`authority`, `why`, Atlas, edit deltas) never reports from bytes the loader would refuse.
4. Native code is never accepted from an author (§5.4).
5. Every abnormal end, a signal included, leaves a machine-readable reason.
6. Nothing that changes a program's meaning is read from the working directory unpinned.

**Defects found in this audit.** Each was re-run by me unless marked otherwise:

| RW | Defect | How it was shown (OBSERVED HERE) | Severity |
|---|---|---|---|
| 4.93 | DIR node-id collision bypasses the reference-capability check | A scratch crate (Appendix A.5) set every node id in a checked tree to 0, re-checked it, and built a self-consistent DIR. `fn f(xs: List[Int]) { xs.push(1) }` and `fn f(c: Cap[Console], xs: List[Cap[Console]]) { xs.push(c) }` are DL1604 as text and **verify** as DIR. Controls stayed refused: a missing effect (DL0501), a field write through `val` (DL1604). Reproduced at `dir::verify`, which the plugin loader calls; not driven end to end through a `.dpx`. A `.dpx` binds its DIR only by a hash in its own manifest (`dpx.rs`, lines 280–284), and signing is optional | MEDIUM: an rcap soundness hole on the Verified-plugin path; no effect or authority escape shown |
| 4.94 | DIR skips the lexer's character rules | The sub-agent's `bidi.dpx`: a function named with U+202E and `ESC[31m`. `plugin verify`: *"ok … verifies"*; `why Read bidi.dpx` printed both raw. The same name in source: DL0107 | LOW–MEDIUM: review and terminal spoofing |
| 5.7 | The DIR fuzz target never runs `verify` | `fuzz/fuzz_targets/dir.rs` → `fuzz_one_dir` decodes and re-encodes only (CURRENT SOURCE) | LOW: the evidence gap that let 4.93 and 4.94 through |
| 6.67 | `why` on a `.dpx` reads unverified facts | `cli.rs` line 6348 calls `dir_deserialize`. The sub-agent's `forged.dpx`: `plugin verify` → DL1504; `why Read` → *"has no export that performs `Read`"*; the honest one → `scan — Read` | LOW–MEDIUM: a review surface vouching for refused bytes (the class of finding C65) |
| 6.68 | A run killed by a signal writes no report | SIGTERM and SIGINT after 1 s, in-process and `--sandbox`: exit 143 / 130; `--report-out` absent; no guest left running | LOW–MEDIUM for agent harnesses that cancel runs |
| 6.69 | `edit` checks the file alone, so inside a package the delta and `--if-checks` go blind | `examples/greeter` copied; `main` replaced via `--node` with a body adding `Net` and `evil.example.com`: exit 0, `written: true`, `before: null`, `widened: null`, also under `--if-checks`; `check .` → DL1009. A bogus prefix `fn:BOGUSPKG/greeter.main` was accepted | MEDIUM for agent editing loops |
| 6.70 | Morph skins: cwd lookup, unpinned, look-alike aliases | Identical source bytes (SHA-256 `9dbf6040…`) printed `yes` and `no` under two skin files in two working directories' morphs folders mapping a Cyrillic look-alike of `false`; `authority` identical | LOW today (no skin ships); MEDIUM if skins ship |
| (6.58) | The closed-pipe exit is a panic | §10.3 | LOW |
| 4.95 | `p.get` does not enforce R-Get: an export is callable under a narrower row than its own | §19.2: a `! {Load, Read}` program wrote a file through a `! {Write}` export typed `! {}`. `check`, `authority` and a package's `check .` (DL1009) all passed | **HIGH** for the effect system; bounded by the run's grants |
| 4.96 | A program can load only Verified plugins | §19.2: `Plugin[Contained]` → `BadArtifact` (DL1508) at run time | LOW |
| 6.71 | `plugin verify` counts private functions as exports | §19.2: *"2 export(s)"* for a one-export plugin with a helper | LOW |
| — | `.dwx` authority output says *"re-verified"* and *"provably pure"* for an edited manifest | **Already closed** as RW 4.73 (ARTIFACT-CLAIM-1): the output also says the effects are *"its builder's claim"*, and `run` refuses an undeclared effect (DL1202, re-run). Only the wording remains | wording only; no row |

**What did not happen** (OBSERVED HERE, read on the far side):
- **A write outside the grant** was DL0904, and the sentinel `forbidden.txt` kept its SHA-256 (`23af755a…`) before and after.
- **A tampered `.dwx`** was DL1202 before it ran.
- **A revoked lease's in-flight loop** never printed its final line.

---

## 13. The eight recommendations the owner asked to have tested (prompt §10)

**1. CLI-first, not shell-only, one typed service layer.** **Confirmed.**
- **Best alternative:** an SDK-first library. It would make the CLI secondary, but the Constitution's terminal-first
  rule (§8.4) and today's agents both use the CLI.
- **Counterargument:** processes are cheap here (4.3 ms). True for small checks. But MCP still forks per call, and the
  edit delta (RW 6.69) shows two code paths disagreeing.
- **Migration:** `check` → `authority` → `edit` → `run` → `test` → `grants`, one at a time behind `core_invariance`
  snapshots.
- **Acceptance:** CLI, MCP and LSP give byte-identical JSON for the same request.
- **Complexity:** large (`cli.rs`, 11,866 lines).
- **Rollback:** per command.

**2. Machine-first authoring is a candidate, not a foregone conclusion.** **Rejected as a new representation; adopted
as structured edits over text.**
- **Evidence:** DIR is 28× larger and verifies 1.9× slower than the text front end, and it has two holes.
- **Best alternative:** Zerolang-style graph patches. DeluluLang's `edit --node --expect-hash` already gives the
  useful part.
- **Counterargument:** a graph avoids reparsing. But reparsing costs 3 µs to 64 ms, against checking's 6 µs to 259 ms.
- **Migration:**
  1. fix RW 6.69 (package-level deltas);
  2. content-hashed node ids;
  3. a multi-edit transaction over several files.
- **Acceptance:** the greeter probe is refused; an edit transaction is applied whole or not at all.
- **Complexity:** medium.
- **Code:** `edit.rs`, `deps.rs`.
- **Rollback:** a flag.

**3. Keep the semantics, optimise the path.** **Confirmed with numbers.**
- **The savings are in the engine and the process:** the Cranelift cache (§5.4), `libpython` out of the main binary
  (−1.1 to −2.0 ms per command), and `--engine auto`. None is in skipping validation.
- **Counterargument:** WASM start-up matters little to agents. True for hello (1.6 ms); false for large programs
  (652 ms).
- **Acceptance:** §7's rows.
- **Complexity:** small (`libpython`), medium (auto), medium (cache, because of N1–N5).

**4. Portable semantic core with target profiles.** **Confirmed.**
- **The core:** the checker plus the primitive table. **The adapters:** host primitives.
- **Profiles:**
  - **desktop and server:** the default `delulu`;
  - **agent utility:** the Python-less build, until RW 3.26 lands;
  - **embedded:** WAMR is the candidate, recorded under RW 3.6 and not built.
- **Explore before inventing a backend:** Pulley (a portable fallback, 11× slower than Cranelift) and precompiled
  artifacts.
- **Acceptance:** `toolchain --json` states each profile's support (RW 6.61).

**5. Everyday AI tasks first.** **Confirmed again.** Every hypothesis in §10.3 is still open; RW §8 items 7 and 8 come
before anything here.

**6. Human readability as a safety feature.** **Confirmed.**
- **The evidence:** the review surfaces matter. Three defects in this audit are review surfaces lying (6.67, 4.94,
  6.69).
- **Readable text** stays the artifact a person or another AI audits.
- **Cost of the editor:** 409 lines of JavaScript.

**7. Avoid another giant subsystem.** **Confirmed.**
- **Reject:** a program database, a new backend, a new agent runtime, WAMR on the desktop.
- **Prefer:** Wasmtime's own precompile API (no new dependency), the existing foreign worker for Python, and the
  existing checker for packages.

**8. No universal binary.** **Confirmed.** §9's matrix separates source, `.dwx`, precompiled code, native executables,
sandbox levels and drivers.

**The recommended architecture:**
- **Text:** canonical `.delulu`, with node-addressed, content-hashed, package-checked edits.
- **One checker.** **One service layer,** shared by the CLI, MCP and LSP.
- **Two engines** chosen by `--engine auto`, with an operator-authenticated precompiled cache, off by default.
- **One portable artifact** (`.dwx`).
- **Custody unchanged:** host primitives, the Guard, the broker and the audit chain.
- **Containment as today,** OpenShell optional.

---

## 14. Beyond the current rules (suggestions under the owner's permission of 2026-11-05)

Each suggestion names the rule it would change. None was acted on.

| # | Suggestion | The rule it crosses | Why | Cost and risk |
|---|---|---|---|---|
| B1 | Reword Constitution §5.11 rule (2) from *"only human-controlled policy"* to *"only an authorised principal's signed policy, held outside the program's own authority"* | Constitution §5.11; §5.14's *"the human holds the keys"* | Constitution §9 and `CONTRIBUTING.md` §4 are kind-blind; §5.11 and §5.14 are not, so the document contradicts itself, as §9 already admits it once did | An entrenched-adjacent change: an RFC and a `D-V2-nn`. The safety comes from N1–N5, not from the kind of signer |
| B2 | Let scope refusals optionally be values: a pure `permits(path)` query first, or an opt-in `try_` primitive returning `Err(Refused)` | Delulu Core §4 `E-Fault`, §8 (*"no handler catches a fault"*); STABILITY (a program refused today stays refused) | Agents need fallbacks (RW 2.42) | Theorem 1 holds either way (fault or value). A query reveals the grant's shape to the program; an opt-in primitive keeps the old rule the default. Prefer the query |
| B3 | Unfreeze the core for the named 2.0 basics by a recorded decision with a 3-day review, instead of RFCs with 14-day comment periods | `CONTRIBUTING.md` §5; README's change freeze since Stage 9 | There are no external users (*"Not distributed"*); the 14-day clocks are the critical path | Less public review; mitigated by the conformance anchors each addition needs anyway |
| B4 | Stop committing the generated Survey map; build and check it in CI and in `doctor` | `AGENTS.md` and `HANDOFF.md` (*"regenerate the Survey after any change"*, a committed, freshness-tested map) | A documents-only commit changed 1,168 lines of `survey.json` (this session's commit `dfb5b62`), which costs review and merge friction for nothing a reviewer reads | The map's history per commit is lost unless CI keeps it as an artifact |
| B5 | Allow a **different AI system run by the owner** (another vendor's model, recording its review in the PR) to be the independent reviewer for documentation-only and record-only PRs | `CONTRIBUTING.md` §4 rule 3; Constitution §9 (two-person review) | Today every cloud PR waits for the owner to read it. A second system is a distinct party; a sub-agent of the author is not | A model can rubber-stamp; require it to cite what it checked. Risk-class changes keep a human or a named sponsor |
| B6 | Retire morph skins, or pin them by hash and allow only confusable-free aliases | Constitution §8.5's human layer; `SYNTAX_MORPH_SPEC.md` | None ships; the lookup changes meaning by working directory (RW 6.70) | Loses a human-language feature nobody uses yet |
| B7 | Freeze PS-C (the microVM) and drop its GPL guest-image pipeline from the plan | the earlier commission's microVM program (D-NE-27, PS-C-05) | The earlier review's verdict; it also removes a licence obligation from the critical path | The L2 level stays as built; new VM-grade work goes to L3 |
| B8 | **No change to the licence** | — | Apache-2.0 plus `NOTICE` and `TRADEMARK.md` (D27) helps adoption; nothing measured here argues otherwise | — |

---

## 15. Keep / simplify / replace / remove (prioritised)

| Priority | Action | Target | Evidence |
|---|---|---|---|
| 0 | **Fix first** | RW 4.95: enforce R-Get at `p.get` (third pass) | §19 |
| 1 | **Fix** | RW 4.93, 4.94, 6.67, 5.7 (DIR and `why`) | §12 |
| 2 | **Fix** | RW 6.68, 6.58, 6.69 (signals, pipes, package edits) | §12 |
| 3 | **Build** | the 2.0 basics and answer contracts (RW §8 items 7–8) | §10.3 |
| 4 | **Simplify** | move embedded Python into the foreign worker (RW 3.26) | −1.1 to −2.0 ms per command |
| 5 | **Add, small** | `--engine auto` with a static policy; the capability matrix (RW 6.61) | §10.1 |
| 6 | **Add, guarded** | the precompiled cache under N1–N5 | §5.4 |
| 7 | **Simplify** | `cli.rs` → services; MCP in-process with a capped read | §7 |
| 8 | **Remove or wire** | `verified_executable_wasm` (RW 3.27) | test-only |
| 9 | **Freeze** | morph skins until pinned; PS-C | §14 |
| — | **Keep** | parser, checker, formatter, interpreter (as the reference), WASM, `.dwx`, custody, sandbox, LSP, VS Code, Atlas, Survey, `doctor`, conformance and formal gates | §4, §6 |
| — | **Reject** | AI-emitted machine code; DIR or a graph store as the authoring form; a native backend now; Winch for the limits engine; WAMR on desktop; a new agent runtime | §5, §11 |

---

## 16. Roadmap

1. **Now (records and fixes, about one to two weeks):** §15 priorities 1 and 2, each witnessed red first and
   falsified by mutants.
2. **2.0 bar (the basics):** as the earlier review's §13 Phase 1 and RW §8 items 7–8, with §14 B3 as the owner's
   choice on process.
3. **2.0 speed, measured:** RW 3.26 (Python out of process), `--engine auto`, the capability matrix. Exit criterion:
   the `--version` floor within 0.2 ms of the Python-less build; the parity corpus green under `auto`.
4. **2.x:** the precompiled cache (N1–N5), the service layer, package-level edit transactions. Then the task-completion
   study against Python in an equivalent sandbox.
5. **Later, only with a named target:** Pulley for CPUs without Cranelift; WAMR for microcontrollers (RW 3.6); a
   component-model `.dwx` 2 study.

---

## 17. Failed, unrun and unknown

| Item | State |
|---|---|
| Windows, macOS, arm64 execution of this audit's probes | **not run here**. CI `41929573373` is green on all four for the existing suite; every new finding is source-level, so it is expected to behave the same there, but this is unverified |
| RTOS, MCU, flight, vehicle, satellite hardware | **unverified**; nothing physical was touched. Drone and telemetry were simulations; the device ran under the `sim` profile |
| Zerolang execution | **not run** (see §11) |
| Token and tool-call overhead per task | **not measured** (needs the task-completion study) |
| DIR collision through a real `.dpx` load | first pass: at `dir::verify` only. **Third pass: through `plugin verify`**, the loader's step 5 (§19). A run that calls the colliding function was not attempted |
| Winch with epoch interruption on Wasmtime 50 | **unknown**; read for 48.0.8 only |
| Precompiled cache under the limits engine | **not built, not measured** |
| Cold start from an empty page cache | **not measured** (dropping caches is a system-wide side effect); all timings are warm |
| A first probe that passed `--grant console` with `--lease` | **my error**: a lease run refuses local grants (exit 2), as designed. Re-run correctly (§8) |
| Two probes with a wrong sensor grant and a dash shell | **my errors**, re-run correctly (§10.3) |

---

## 18. Sources

- Wasmtime: [releases](https://github.com/bytecodealliance/wasmtime/releases); [v50.0.0](https://github.com/bytecodealliance/wasmtime/releases/tag/v50.0.0); [pre-compiling](https://docs.wasmtime.dev/examples-pre-compiling-wasm.html); [fast compilation](https://docs.wasmtime.dev/examples-fast-compilation.html); [Pulley](https://docs.wasmtime.dev/examples-pulley.html); [platform support](https://docs.wasmtime.dev/stability-platform-support.html); vendored crate source `wasmtime-48.0.8` (`src/runtime/module.rs` lines 372–398; `src/config.rs` lines 776–784; `src/engine.rs` line 686).
- Zerolang: [repository](https://github.com/vercel-labs/zerolang); [community docs](https://zerolang.app/docs/intro/); [docs index](https://docsearch.algolia.com/mcp/docs/repo/vercel-labs/zerolang).
- Mojo: [FAQ](https://mojolang.org/docs/faq/); [get started](https://docs.modular.com/mojo/manual/get-started).
- CPython: [compiler.md](https://github.com/python/cpython/blob/main/InternalDocs/compiler.md); [interpreter.md](https://github.com/python/cpython/blob/main/InternalDocs/interpreter.md).
- WAMR: [repository](https://github.com/wasm-micro-runtime/wasm-micro-runtime); [product-mini README](https://github.com/bnason-nf/wasm-micro-runtime/blob/main/product-mini/README.md) (mirror); [Espressif component](https://components.espressif.com/components/espressif/wasm-micro-runtime).
- WASI and components: [WASI releases](https://wasi.dev/releases); [roadmap](https://wasi.dev/roadmap); [why the component model](https://component-model.bytecodealliance.org/design/why-component-model.html).
- LLVM: [ORCv2](https://llvm.org/docs/ORCv2.html); [MLIR LangRef](https://mlir.llvm.org/docs/LangRef/).
- NVIDIA OpenShell: [releases](https://github.com/NVIDIA/OpenShell/releases); [docs](https://docs.nvidia.com/openshell/latest/).

---

## 19. Third verification pass (2026-11-05, same session)

**Why.** The owner asked once more: *"Double Verify And Research About Everything Once Again … Test Everything. No
need to run agents this time … Check CI run in master and current branch and make fixes."* So this pass used **no
sub-agents**. Everything in it is OBSERVED HERE by the head chef, or OFFICIAL EXTERNAL where marked.

### 19.1 State and CI (CURRENT CI)

| Item | Value |
|---|---|
| `master` | `6397a98`, the owner's merge commit of PR #125 (06:56 UTC, a merge commit rather than a fast-forward). `3d6c881` is its second parent and the two trees are identical. No code changed since `0670e0b` (`git diff --stat 0670e0b 6397a98 -- crates scripts .github Cargo.toml Cargo.lock editors` is empty), so the provenance build of §2 is this code |
| PR run [`41983232404`](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/41983232404) on `3d6c881` | **success**: 14 jobs passed, 2 skipped by design (`heavy-gates`, `miri-slow`). Covered: tests on ubuntu-24.04, macos-latest, windows-latest and arm64; lints, supply-chain, formal, fuzz, editor, microvm, microvm-reproducible; miri for diag, atlas and ffi |
| `master` push run [`41983782231`](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/41983782231) on `6397a98` | **success**, the same 14 jobs and 2 skips. Both runs are recorded on the PR ([comment](https://github.com/jessesuniljs1-collab/delululang-test/pull/125#issuecomment-6652265892)) |
| CI failures to fix | **none**: neither run had a red job |
| `origin/autosave/laptop` | new snapshot `ea8a903` (06:27 UTC). Compared with its base `0670e0b`, it adds only the audit prompt, byte-identical to the committed one (SHA-256 `3dbe437e…`). Nothing to recover |

### 19.2 What was re-run

| Check | Result |
|---|---|
| Appendix A.4's whole probe ledger (71 probes), replayed on a fresh copy of the scratch work with fresh state | **71 of 71 identical**: the same exit code, stdout and first stderr line once scratch paths and timestamps are normalised |
| RW 4.94 (bidi name in a DIR) | `plugin verify bidi.dpx` → *"ok … verifies"*; `why Read` printed U+202E and `ESC[31m` raw. **Holds** |
| RW 6.67 (`why` on forged facts) | `plugin verify forged.dpx` → DL1504, exit 1; `why Read forged.dpx` → *"has no export that performs `Read`"*, exit 0. **Holds** |
| RW 6.70 (morph skins) | identical bytes (SHA-256 `9dbf6040…`) printed `yes` in one directory and `no` in the other. **Holds** |
| RW 6.69 (`edit` in a package) | `examples/greeter`, `main` replaced through `--node` with a `Net` body under `--if-checks`: exit 0, `written: true`, `before: null`, `widened: null`; `check .` → DL1009; the bogus prefix `fn:BOGUSPKG/greeter.main` accepted. **Holds** |
| RW 6.68 (signals) | SIGTERM → 143, SIGINT → 130, `--report-out` absent both times, no `delulu` left running. **Holds** |
| RW 6.58 (closed pipe) | large output into `head -1`: exit 2, *"failed printing to stdout: Broken pipe (os error 32)"*, one panic. **Holds** |
| RW 4.93 (DIR node-id collision) | the scratch harness rebuilt: both `push` cases verify after collision; the DL0501 and field-write DL1604 controls stay refused. **Holds, and now end to end through the loader:** a `.dpx` whose DIR is the collided form of a module holding `fn poke(xs: List[Int]) { xs.push(1) }` beside the honest export, re-bound by its manifest's `dir_blake3`, gets `plugin verify` → *"ok: `rcap.dpx` verifies — verified plugin, 2 export(s)"*, exit 0. `plugin verify` runs the load's own `step5_verified` (`plugin.rs` lines 935–961: *"identical verdicts to a real load"*) |
| Every file and line the §3 table cites | re-read: all match except `toolchain.rs`, whose `engines` line is 87, not 77 (corrected in place) |
| MCP | **12 tools** in an ordinary directory, **16** inside a DeluluLang checkout (adding `doctor_check`, `survey_diff`, `survey_impact`, `survey_query`). A `check` call over a live session costs a **median 5.79 ms** (41 calls; one outlier of 59 ms), against 5.6 ms in §7: the fork per call (RW 6.62) |
| Process floor, interleaved (41 rounds, all cases launched by the same fork-and-exec launcher, so every figure carries the launcher's cost) | `/bin/true` 2.17 ms; `--version`: default 4.91, default stripped 4.74, Python-less stripped 3.48; `check hello`: default 5.55, Python-less 3.99. **libpython: 1.3–1.4 ms per command** (inside §1's 1.1–2.0 ms), and **4.1 MiB of peak RSS** (11.8 against 7.7 MiB for `check hello`: new). `check hello` costs **0.5–0.6 ms** over `--version` here and 0.9 ms in §10.2's runs, not the 0.3 ms §1 first said |

**New findings, each re-run before it was written:**

1. **RW 4.95, HIGH: R-Get is not enforced.** The spec (`STAGE1_SPECIFICATION.md` line 716) says `p.get[F](name)`
   succeeds only if the export's type matches `F` with `row(export) ⊆ row(F)`. The checker leaves that half to run
   time (`check.rs` lines 2617–2630). `plugin.rs` implements it as `r_get_verified` and `r_get_contained` (lines
   583–668), but the interpreter's `plugin_get` (`interp.rs` lines 1902–1919) checks only that the name exists, and the
   annotated `F` never reaches the runtime. The functions arrived in `33f08d1` (2026-07-15, *"R-Get at runtime,
   fail-closed throughout"*); their only caller is the P8-02 device adapter. Witnesses (sources in §19.5):
   - **Program level.** `esc.delulu`'s `main` is `! {Load, Read}`. It loads the Verified plugin `putter` (export
     `put: fn(Cap[FsWrite], Str) -> Unit ! {Write}`) and takes `put` as `! {}`. `check`: clean. `authority`:
     `effects: Load, Read`. The run with `--grant plugin=. --grant fs.write=./out` **wrote `out/written-by-plugin.txt`**.
   - **Package ceiling.** The same program as package `escpkg`, whose `[authority] effects` are `["Load", "Read"]`:
     `check .` → *"authority within manifest and pins"*; the run wrote the file. DL1009 did not fire.
   - **Function level.** `pure_call`, declared with no row (`authority` lists it under `pure fns`), returned
     `TOPSECRET` from a file through the `! {Read}` export `scan` taken as `! {}`.
   - **Wrong parameter types are not refused either.** `scan` (two parameters) taken as `fn(Str) -> Str ! {}` faulted
     inside the plugin: DL0907, *"unbound name `p`"*.
   - **What still held:** the run needed `--grant fs.write=./out`, and the Guard scoped the write to it. `authority`'s
     `plugins:` section lists `putter [verified] grant.effects: [Write]`, the one place the effect shows. The WASM
     engine refuses `plugin_host` (DL1201), and `--sandbox` refuses a program that uses PluginHost (*"Nothing ran"*).
   - **Why the earlier passes missed it:** `plugin_load_cli.rs` only takes exports at their honest type, the
     laundering suite leaves F-1 to *"the runtime plugin loader (Stage 6)"* (`laundering.rs` lines 4–5), and the unit
     tests call `r_get_verified` directly. **A rule written as a function is enforced only where something calls it.**
2. **RW 4.96, LOW: programs load only Verified plugins.** `interp.rs` line 1879 calls `load_verified` for every
   `load`, so `let p: Plugin[Contained] = load(…)` on a Contained `.dpx` checks clean and returns `BadArtifact`
   (DL1508, *"the host requested `Plugin[verified]`"*). RW 4.11 names only Windows as refusing Contained. By accident
   this keeps `SOUNDNESS_AUDIT.md` F-1's Contained form from running.
3. **RW 6.71, LOW: `plugin verify` counts every DIR function as an export.** For a one-export plugin with a private
   `helper`: `plugin build` says *"1 export(s)"*, `plugin verify` says *"2 export(s)"*, and its JSON says `["helper",
   "scan"]` (`plugin.rs` line 953 reports `dir.fn_types`). `p.get("helper")` is refused, so the surface is overstated,
   not opened.

### 19.3 Primary sources refreshed (OFFICIAL EXTERNAL unless marked; searched 2026-11-05)

| Item | Now | Effect |
|---|---|---|
| Wasmtime | v50.0.0 (2026-10-20) and **50.0.1** exist; no v51 (monthly releases, so expected around 2026-11-20, INFERENCE). Advisories: **GHSA-mghf-wjfp-p5xm / RUSTSEC-2026-0379** (CVSS 9.9, aarch64 `br_table` jump to an arbitrary target) is fixed in 36.0.19, **48.0.7**, 49.0.4 and 50.0.0. **GHSA-897f-g5gm-fmxm** (a Cranelift miscompilation on any architecture; THIRD-PARTY write-up) is reported fixed in the same versions. **GHSA-9pvw-hwfg-92h5 / RUSTSEC-2026-0378** (a component-model panic) is fixed in 48.0.7. **GHSA-6fr9-xmw9-5352** (unchecked memory usage) is fixed in 36.0.20, **48.0.8**, 49.0.5 and 50.0.1 | DeluluLang pins **48.0.8** (`Cargo.lock`), which carries every fix found. The aarch64 fix applies to a platform CI tests (the arm64 job). Keep the LTS pin: the `supply-chain` job's `cargo deny check advisories` step gates every CI run (`ci.yml` lines 613–621), and it has caught Wasmtime advisories before (D-V2-67) |
| NVIDIA OpenShell | **v0.1.7** is still *"Latest"* in its docs. A `v0.1.7-pre.1` workflow ran on 2026-10-28. Its notes add OTLP exporters with auth headers and CLI browser login inside containers. The installation page still says it is not production-qualified | The earlier verdict stands: **optional backend**. The pin is still v0.1.2 (RW 7.28) |
| Zerolang | v0.3.4 (June 2026) is still the newest tag found (THIRD-PARTY directories); no v0.4 | No change |
| Mojo | Search found only the May–June nightlies of 1.0.0b2. The 1.2.0 of 2026-10-22 rests on the earlier passes' read of mojolang.org | Not re-confirmed this pass; no effect |
| WAMR | 2.4.4 was a security release (2025-11-25); 2.4.5 is listed with no date | No change: still a microcontroller candidate only |
| **MoonBit** (new here; the prompt names it) | Its docs call it *"an AI native programming language toolchain for cloud and edge computing"*, targeting wasm, wasm-gc, js and native. v0.10.17 (October 2026), 1.0 not yet shipped; reference counting on wasm and C, host GC on wasm-gc and JS (THIRD-PARTY detail) | The nearest design neighbour on the *target* question: one source compiled ahead of time to WASM or native, no authority model. It supports §5's answer: WASM as the portable target, a native backend only when measured. It offers nothing for effects, grants or custody |

### 19.4 Corrections made in place (old → new)

| Where | Old | New | Why |
|---|---|---|---|
| §1 table, §11 | Winch *"runs the loop 43% slower"* | 42% | 38,664 / 27,145 µs = 1.424 |
| §1 table | Pulley *"compiles no faster"* | *"compiles within 11% of Cranelift's time either way"* | §10.1: 8% and 11% faster at 2 and 4,002 lines, 6–11% slower at 1,002 and 16,002 |
| §1 table | *"`check hello` adds 0.3 ms"* | 0.5–0.9 ms | §10.2 (4.26 − 3.35) and §19.2's interleaved run |
| §5 model 5, §11 | WASM *"within 3.4× of `rustc -O`"* | 3.3× | 34.13 / 10.22 ms |
| §3 table | `toolchain.rs` (line 77) for `engines` | lines 87 and 77 | re-read |
| §11 Wasmtime | 50.0.1 *"unconfirmed"* | confirmed | §19.3 |
| §15 | priorities started at 1 | priority 0 is RW 4.95 | §19.2 |
| §17 | the DIR collision *"not end to end"* | through `plugin verify` | §19.2 |

None of these moves a recommendation.

### 19.5 Reproducibility for the new witnesses (scratch; not in the repository)

The Verified plugin `putter`:

```toml
[package]
name = "putter"
version = "0.1.0"
kind = "plugin"

[plugin]
api = 1
class = "verified"

[plugin.authority]
effects  = ["Write"]
requires = []

[plugin.exports]
put = "fn(Cap[FsWrite], Str) -> Unit ! {Write}"
```

```delulu
module putter

pub fn put(fs: Cap[FsWrite], s: Str) ! {Write} {
    match fs.write_text("written-by-plugin.txt", s) {
        Ok(_) => {}
        Err(_) => {}
    }
}
```

The host, `esc.delulu` (`delulu plugin build .` in the plugin's directory, then copy `putter.dpx` beside the host):

```delulu
module host

fn go(h: Cap[PluginHost], w: Cap[FsWrite]) -> Result[Int, PluginErr] ! {Load, Read} {
    let g = Grant {
        effects: ["Write"], fs_read: [], fs_write: [], net: [], secrets: [], declassify: [],
        limits: Limits { fuel: 0, mem_mb: 0, wall_ms: 0 },
        require_signed: false,
    }
    let p = load(h, "putter.dpx", g)?
    let f: fn(Cap[FsWrite], Str) -> Unit ! {} = p.get("put")?
    f(w, "the host's row has no Write")
    Ok(0)
}

fn main(root: Root) ! {Load, Read} {
    let w = root.fs_write("./out")
    match go(root.plugin_host(), w) {
        Ok(_) => {}
        Err(_) => {}
    }
}
```

```bash
delulu check esc.delulu                       # ok: checked clean
delulu authority esc.delulu                   # effects: Load, Read
delulu run esc.delulu --grant plugin=. --grant fs.write=./out --no-prompt
cat out/written-by-plugin.txt                 # the host's row has no Write
```

The package form adds a `delulu.toml` with `[package] name = "escpkg"`, `language = "1.0"` and `[authority] effects =
["Load", "Read"]`, with the host as `src/main.delulu` (`module escpkg`). The function-level form takes the
`scan` plugin of §12's fixtures (`fn(Cap[FsRead], Str) -> Str ! {Read}`) as `! {}` inside `fn pure_call(f: …, fs:
Cap[FsRead]) -> Str` and reads `secret.txt` under `--grant fs.read=./data`. The end-to-end DIR collision uses a
scratch crate `dirforge` (the §A.5 harness, writing the collided DIR to a file). A short script then replaces the
`delulu:dir` section of an honest `scan.dpx` and its manifest's `dir_blake3`.

### 19.6 The audit prompt, re-walked

Nothing in §§3–12 of the prompt was found unaddressed. These stay open, and are recorded rather than claimed:
- **Token and tool-call overhead per completed task**: still not measured. It needs the task-completion study.
- **Zerolang**: still not run, for the same reason as before (a remote installer).
- **Windows, macOS and arm64 runs of the new witnesses**: not run. The two CI runs above are the existing suite,
  which has no R-Get witness.
- **The fixes themselves**: not made. This pass recorded them. RW 4.95's fix touches the checker, the runtime and
  DIR, so it needs its own witnessed and mutant-tested change, not a documents PR.

---

## Appendix A. Reproducibility

All paths below were in the session scratch, which a cloud container discards; the sources are reproduced here so the
experiments can be run again. `$D` is the provenance build of §2; `$W` a scratch copy of Codex's work directory
(`docs/review/DeluluLang_Codex_Strategic_Review.md` Appendix A rebuilds it); `DELULU_HOME` and `DELULU_STATE_DIR`
pointed into scratch for every run.

### A.1 Build

```bash
DELULU_BUILD_COMMIT=$(git rev-parse origin/master) CARGO_TARGET_DIR=$SCRATCH/target-audit cargo build --locked --release -j 4 -p delulu -p delulu-survey
```

### A.2 The stage harness

A Cargo project outside the repository. Its `Cargo.toml` names `delulu-syntax`, `delulu-check`, `delulu-diag`,
`delulu-runtime` (`default-features = false, features = ["net"]`) and `delulu-wasm` by path, plus `wasmtime = "48"`
with the shipped feature list and `winch` and `pulley` added. `[workspace]` is empty. It uses the repository's
`Cargo.lock`, copied in. Run: `dl-stage-bench 21 hello.delulu`, `11 kernel.delulu 5000000`, `11 big500.delulu`,
`7 big2000.delulu`, `5 big8000.delulu`.

```rust
//! Scratch stage-timing harness for the AI-first pipeline audit (2026-11-05). NOT part of the repository.
//!
//! For each program: lex, parse, resolve, check, DIR serialize, DIR verify (decode + re-check), WASM codegen,
//! Wasmtime compile with Cranelift (the shipped tier), Winch (baseline) and Pulley (portable interpreter),
//! precompile + deserialize (what a cached native artifact would cost to load), and execution: the interpreter
//! (`Interp::run_main`), the shipped WASM host end to end (`delulu_wasm::run_main`, which compiles with its own
//! Cranelift engine first), and — for an exported pure function `work(n)` — the bare call on each Wasmtime tier.
//!
//! Usage: dl-stage-bench <reps> <file.delulu> [work_arg]
//! Prints one JSON object per stage: {"file", "stage", "median_us", "min_us", "reps", "bytes"?}.

use std::rc::Rc;
use std::time::Instant;

use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use wasmtime::{Config, Engine, Linker, Module, OptLevel, Store, Strategy};

fn harden(c: &mut Config) {
    // The same proposal switches as delulu-wasm's `harden_wasm_features` (host.rs:65-89).
    c.wasm_simd(false);
    c.wasm_relaxed_simd(false);
    c.wasm_threads(false);
    c.wasm_memory64(false);
    c.wasm_component_model(false);
    c.wasm_gc(false);
    c.wasm_function_references(false);
    c.wasm_exceptions(false);
    c.wasm_stack_switching(false);
    c.wasm_tail_call(false);
    c.wasm_multi_memory(false);
    c.wasm_custom_page_sizes(false);
    c.wasm_wide_arithmetic(false);
    c.wasm_shared_everything_threads(false);
}

fn engine(kind: &str) -> Engine {
    let mut c = Config::new();
    harden(&mut c);
    match kind {
        "cranelift" => {
            c.cranelift_opt_level(OptLevel::Speed);
        }
        "winch" => {
            c.strategy(Strategy::Winch);
        }
        "pulley" => {
            c.target("pulley64").expect("pulley64 target");
        }
        _ => unreachable!(),
    }
    Engine::new(&c).expect("engine")
}

fn time<T>(reps: usize, mut f: impl FnMut() -> T) -> (u128, u128, T) {
    let mut v = Vec::with_capacity(reps);
    let mut last = None;
    for _ in 0..reps {
        let t = Instant::now();
        let r = f();
        v.push(t.elapsed().as_micros());
        last = Some(r);
    }
    v.sort();
    (v[v.len() / 2], v[0], last.unwrap())
}

fn emit(file: &str, stage: &str, reps: usize, med: u128, min: u128, bytes: Option<usize>, note: &str) {
    let b = bytes.map(|b| format!(",\"bytes\":{b}")).unwrap_or_default();
    let n = if note.is_empty() { String::new() } else { format!(",\"note\":{:?}", note) };
    println!("{{\"file\":{file:?},\"stage\":{stage:?},\"median_us\":{med},\"min_us\":{min},\"reps\":{reps}{b}{n}}}");
}

fn run(reps: usize, path: &str, work_arg: Option<i64>) {
    let src = std::fs::read_to_string(path).expect("read");
    let file = path.rsplit('/').next().unwrap();
    emit(file, "source", 1, 0, 0, Some(src.len()), &format!("{} lines", src.lines().count()));

    let (m, n, _) = time(reps, || delulu_syntax::lexer::lex(0, &src));
    emit(file, "lex", reps, m, n, None, "");
    let (m, n, (module, pd)) = time(reps, || delulu_syntax::parse_file(0, &src));
    emit(file, "lex+parse", reps, m, n, None, &format!("{} parse diagnostics", pd.len()));
    let (m, n, (table, rd)) = time(reps, || delulu_check::resolve::resolve(&module));
    emit(file, "resolve", reps, m, n, None, &format!("{} resolve diagnostics", rd.len()));
    let (m, n, result) = time(reps, || delulu_check::check::check_module(&module, &table));
    let errs = result.diags.iter().filter(|d| d.is_error()).count();
    emit(file, "check", reps, m, n, None, &format!("{errs} check errors"));
    let (m, n, _) = time(reps, || delulu_check::check_source(0, &src));
    emit(file, "check_source(all front-end)", reps, m, n, None, "");

    let (m, n, dir) = time(reps, || delulu_check::dir::serialize(&module, &result));
    emit(file, "dir_serialize", reps, m, n, Some(dir.len()), "canonical CBOR of the typed AST + facts");
    let (m, n, _) = time(reps, || delulu_check::dir::deserialize(&dir).is_ok());
    emit(file, "dir_decode+validate", reps, m, n, None, "");
    let (m, n, ok) = time(reps, || delulu_check::dir::verify(&dir).is_ok());
    emit(file, "dir_verify(decode+re-check)", reps, m, n, None, if ok { "accepted" } else { "REFUSED" });

    let (m, n, wasm) = time(reps, || delulu_wasm::compile_checked(&module, &result));
    let wasm = match wasm {
        Ok(w) => w,
        Err(e) => {
            emit(file, "wasm_codegen", reps, m, n, None, &format!("REFUSED: {}", e.message()));
            interp(reps, file, &module);
            return;
        }
    };
    emit(file, "wasm_codegen", reps, m, n, Some(wasm.len()), "");

    for kind in ["cranelift", "winch", "pulley"] {
        let eng = engine(kind);
        let (m, n, md) = time(reps, || Module::new(&eng, &wasm));
        match md {
            Ok(_) => emit(file, &format!("compile_{kind}"), reps, m, n, None, ""),
            Err(e) => {
                emit(file, &format!("compile_{kind}"), reps, m, n, None, &format!("REFUSED: {e}"));
                continue;
            }
        }
        let (m, n, pre) = time(reps, || eng.precompile_module(&wasm).expect("precompile"));
        emit(file, &format!("precompile_{kind}"), reps, m, n, Some(pre.len()), "native artifact bytes");
        // SAFETY (scratch harness only): the bytes were produced by this very engine a moment ago.
        let (m, n, _) = time(reps, || unsafe { Module::deserialize(&eng, &pre) }.expect("deserialize"));
        emit(file, &format!("deserialize_{kind}"), reps, m, n, None, "load of the precompiled artifact");
        if let Some(arg) = work_arg {
            let module_w = Module::new(&eng, &wasm).unwrap();
            let r = (|| -> wasmtime::Result<(u128, u128, i64)> {
                let linker: Linker<()> = {
                    let mut l = Linker::new(&eng);
                    l.define_unknown_imports_as_traps(&module_w)?;
                    l
                };
                let mut store = Store::new(&eng, ());
                let inst = linker.instantiate(&mut store, &module_w)?;
                let f = inst.get_typed_func::<i64, i64>(&mut store, "work")?;
                let k = reps.min(5);
                let (m, n, v) = time(k, || f.call(&mut store, arg).expect("call"));
                Ok((m, n, v))
            })();
            match r {
                Ok((m, n, v)) => emit(file, &format!("call_work_{kind}"), reps.min(5), m, n, None, &format!("work({arg}) = {v}")),
                Err(e) => emit(file, &format!("call_work_{kind}"), 0, 0, 0, None, &format!("FAILED: {e}")),
            }
        }
    }

    let cfg = delulu_wasm::HostConfig { console: true, ..delulu_wasm::HostConfig::default() };
    let k = if work_arg.is_some() { reps.min(5) } else { reps };
    let (m, n, out) = time(k, || delulu_wasm::run_main(&wasm, &cfg));
    let note = match &out {
        Ok(s) => format!("ok, {} bytes of output", s.len()),
        Err(e) => format!("FAILED: {}", e.message()),
    };
    emit(file, "wasm_run_main(shipped host: compile+instantiate+run)", k, m, n, None, &note);
    interp(if work_arg.is_some() { reps.min(5) } else { reps }, file, &module);
}

fn interp(reps: usize, file: &str, module: &delulu_syntax::ast::Module) {
    let g = Grants { console: true, ..Default::default() };
    let (m, n, r) = time(reps, || {
        set_capture(true);
        let r = Interp::new(module).run_main(Value::Root(Rc::new(g.build_root())));
        let out = take_capture().unwrap_or_default();
        (r.is_ok(), out.len())
    });
    emit(file, "interp_run_main", reps, m, n, None, &format!("ok={} output_bytes={}", r.0, r.1));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let reps: usize = args[1].parse().unwrap();
    let path = args[2].clone();
    let work_arg = args.get(3).map(|s| s.parse::<i64>().unwrap());
    // The interpreter recurses on the native stack; give the whole harness a large one, as delulu's own
    // `on_interpreter_thread` does.
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(move || run(reps, &path, work_arg))
        .unwrap()
        .join()
        .unwrap();
}
```

The programs:

```delulu
module k

fn work(n: Int) -> Int {
  var i = 0
  var acc = 0
  while i < n {
    acc = acc + i % 7
    i = i + 1
  }
  acc
}

fn main(root: Root) ! {Write} {
  root.console().println(str(work(5000000)))
}
```

`hello.delulu` is `module h` followed by `fn main(root: Root) ! {Write} { root.console().println("hi") }`. The scaling
programs are `module scale.big`, a `type Rec { a: Int, b: Str }`, then `n` functions of the form
`fn f{i}(x: Int) -> Int { let r = Rec { a: x, b: "s" }` / ` if r.a > 3 { f{i-1}(r.a - 1) } else { r.a * 2 } }` (two
lines each), and `fn main(root: Root) ! {Write} { root.console().println(str(f{n-1}(10))) }`, for n = 500, 2,000 and
8,000.

### A.3 CLI timings and memory

The timing driver ran each case 22 times, discarded the first, and required every run's exit code and stdout to match.
Peak memory used this launcher, because a child forked from Python keeps Python's pre-`exec` peak (15.5 MiB):

```c
#include <stdio.h>
#include <unistd.h>
#include <sys/wait.h>
#include <sys/resource.h>
int main(int argc, char **argv) {
  pid_t p = fork();
  if (p == 0) { int fd = open("/dev/null", 1); dup2(fd, 1); dup2(fd, 2); execvp(argv[1], argv + 1); _exit(127); }
  int st; struct rusage ru; wait4(p, &st, 0, &ru);
  printf("%ld\n", ru.ru_maxrss); return 0;
}
```

The Python kernel is the same loop as `kernel.delulu` (`while i < n: acc = acc + i % 7; i = i + 1`); the Rust kernel
is the same loop in `fn work(n: i64) -> i64`, built with `rustc -O`, its bound passed through `std::hint::black_box`.

### A.4 Functional probes (each id is a row in the session ledger)

| Id | Command (in `$W`) | Exit | Key output |
|---|---|---|---|
| H01–H05 | `$D run {args,json,math,method}_missing.delulu --grant console`; `interpolation.delulu` | 1,1,1,1,0 | DL0405, DL0301, DL0301, DL0405; `hello {name}` |
| H06–H08 | `$D run {range,hof_mono,hof_gen,ifarm,ifblock}.delulu --engine {interp,wasm} --grant console` | 0/1, 0/1, 0/0, 0/1, 0/0 | DL1201 ×3 on WASM |
| H09 | `$D test /no/such/dir --json`; `$D test zzz_no_such_test --json` (in `package/`) | 0, 0 | `passed: 0, failed: 0` |
| H10 | `$D fix effect_missing.delulu --dry-run --json` | 0 | `errors_after: 1`, `skipped: 1` |
| H11 | `$D run utility.delulu --json` with and without `fs.read`, `fs.write` grants | 0, 1 | `{"ok":true,"value":42}`; a DL0703 envelope |
| H13 | `$D run device_sim.delulu` with the actuator grant of Codex's review and `sensor=review/elbow#angle_deg`; then `--broker-profile sim` | 0, 0 | `COMMANDED` / `REFUSED …` / `NO READING`; with sim `READ 12.0` |
| H17 | a 3,000-error file and a 200,000-line printer, each run three times as `… 2>&1 \| head -1` under bash, then once into `/dev/null` | errors: 2, 2, 1, then 1; printer: 2, 2, 2, then 0 | *"failed printing to stdout: Broken pipe (os error 32)"* |
| U1–U4 | `utility.delulu` with `value=10\nvalue=11`, `value=x`, empty; `outside.delulu` | 0,0,0,1 | `{"ok":false,"error":"format"}` twice; DL0904, sentinel unchanged |
| L1–L5 | `check --json`, `fix --accept-widening add_effect_to_row`, `authority --within`, `run --report-out`, `grants propose` | 1,0,1,1,0 | as §8 |
| X | hello and kernel on both engines, `build --target wasm`, `run *.dwx`, a one-bit-flipped `.dwx`, the package on both engines and as one `.dwx`, the plugin on both engines | — | DL1202 on the flip; plugin on WASM DL1201 |
| C1–C5 | `--sandbox`, `--sandbox --seed 1`, `--limits wall=1` on both engines, `--sandbox --mode audit --json` | 0,2,1,1,0 | `stopped_by: {dimension: wall}`; `ran: false` with `required_grants` |
| S | `drone.delulu`, `telemetry.delulu` on both engines; `device_sim.delulu` under `sim` and on WASM | — | identical drone output; telemetry DL1201 on WASM; devices DL1201 on WASM |
| D1–D4 | a scratch broker (`broker start`), `grants delegate` parent and child, `run --broker daemon --lease`, a widening delegation, `grants revoke` of the parent during a loop, `audit verify` | 1, DL0802, 1, 1 | §8 |
| K | SIGTERM and SIGINT after 1 s, in-process and `--sandbox`, with `--report-out` | 143, 143, 130, 130 | no report; no guest left |
| E1–E3 | `edit --node fn:review.utility/review.utility.task` dry run; an ill-typed widening; the same with `--if-checks` | 0, 1, 1 | `widened: []`; refused; not written |
| G | the greeter widening via `--node`, plain and `--if-checks`; `check .`; a bogus prefix (dry run) | 0, 0, 1; the bogus prefix answered with 0 errors (its exit was not captured) | §12 |

### A.5 The DIR collision check (my copy of the sub-agent's harness)

```rust
use ciborium::Value;
use delulu_check::{check_source, dir, Dir};

fn zero_ids(v: &mut Value, n: &mut usize) {
    match v {
        Value::Map(m) => {
            for (k, x) in m.iter_mut() {
                if matches!(k, Value::Text(t) if t == "id") {
                    if let Value::Integer(_) = x { *x = Value::Integer(0u32.into()); *n += 1; continue; }
                }
                zero_ids(x, n);
            }
        }
        Value::Array(a) => for x in a.iter_mut() { zero_ids(x, n) },
        _ => {}
    }
}

fn try_src(label: &str, src: &str) {
    let honest = check_source(0, src);
    let honest_errs: Vec<String> = honest.diagnostics.iter().filter(|d| d.is_error()).map(|d| format!("{}", d.code)).collect();
    println!("[{label}] text path (check_source) errors: {honest_errs:?}");
    // 1. a DIR built from the honest (erroring) check, un-mutated: verify must refuse
    let bytes = Dir::from_checked(&honest.module, &honest.result).encode();
    println!("[{label}] DIR of unmutated module verifies? {}", dir::verify(&bytes).is_ok());
    // 2. mutate: collide every NodeId to 0, re-run the checker on the mutated module, build a self-consistent DIR
    let mut cbor = Vec::new();
    ciborium::into_writer(&honest.module, &mut cbor).unwrap();
    let mut val: Value = ciborium::from_reader(&cbor[..]).unwrap();
    let mut n = 0; zero_ids(&mut val, &mut n);
    let mut cbor2 = Vec::new(); ciborium::into_writer(&val, &mut cbor2).unwrap();
    let module2: delulu_syntax::ast::Module = ciborium::from_reader(&cbor2[..]).unwrap();
    let (table, rd) = delulu_check::resolve::resolve(&module2);
    let result = delulu_check::check::check_module(&module2, &table);
    let errs: Vec<String> = rd.iter().chain(result.diags.iter()).filter(|d| d.is_error()).map(|d| d.code.to_string()).collect();
    println!("[{label}] NodeIds zeroed ({n} ids): re-check errors = {errs:?}");
    let bytes2 = Dir::from_checked(&module2, &result).encode();
    match dir::verify(&bytes2) { Ok(_) => println!("[{label}] >>> collided DIR VERIFIES"), Err(e) => println!("[{label}] collided DIR refused: {}", e.message()) }
}

fn main() {
    // DL1604 cases the sub-agent reported (a write through a `val` receiver), plus controls that must stay refused.
    try_src("push-val-list", "module m\nfn f(xs: List[Int]) { xs.push(1) }\nfn main(root: Root) ! {Write} { root.console().println(\"x\") }\n");
    try_src("push-cap-val-list", "module m\nfn f(c: Cap[Console], xs: List[Cap[Console]]) { xs.push(c) }\nfn main(root: Root) ! {Write} { root.console().println(\"x\") }\n");
    try_src("control-effect-missing", "module m\nfn main(root: Root) { root.console().println(\"x\") }\n");
    try_src("control-field-write", "module m\ntype P { x: Int }\nfn f(p: P) { p.x = 1 }\nfn main(root: Root) ! {Write} { root.console().println(\"x\") }\n");
    try_src("control-honest", "module m\nfn main(root: Root) ! {Write} { root.console().println(\"x\") }\n");
}
```

---

## Appendix B. Sub-agents

| Agent | Model (its own system prompt) | Task | What I re-verified |
|---|---|---|---|
| research | *"Sonnet 5.5 … claude-sonnet-5-5"* | primary-source facts for §11, with quotes and URLs; read-only; curl blocked for GitHub, so GitHub facts came through a summarising fetch | Pulley's ~10× and Winch's slower code (my own measurements agree); `Module::deserialize` and `precompile_module` wording (vendored source); Wasmtime 50.0.0's date, Zerolang's model, Mojo on WSL and WAMR's modes and sizes (web search). Not re-verified: Wasmtime 50.0.1, Zerolang's release dates, OpenShell's v0.1.8 absence |
| security | *"Sonnet 5.5 … claude-sonnet-5-5"* | an adversarial read of five candidate AI-first changes; read-only on the repository (its final `git status` showed only this session's prompt file) | every defect in §12 except where marked; the `.dwx` wording finding was found already closed (RW 4.73); its MCP finding is RW 6.62; its macOS-reads and Windows-limits notes match existing documents |

The sub-agents were bounded by their briefs, wrote only to scratch, and approved nothing. An agent this session
started is not an independent reviewer under `CONTRIBUTING.md` §4.
