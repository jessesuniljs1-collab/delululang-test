# DeluluLang — Claude's independent strategic verification, architecture review and OpenShell decision

**Written:** 2026-11-04/05 (UTC), in a Claude Code cloud session (`session_01Deh6W9bJdfxAeMARtXU9R9`; configured and
last-served model both `claude-opus-5-5`, read from the session record; no model-fallback notice). **Commissioned by:** the owner's prompt
[`docs/prompt/DeluluLang_Claude_Code_Cloud_Strategic_Verification_Prompt.md`](../prompt/DeluluLang_Claude_Code_Cloud_Strategic_Verification_Prompt.md),
read in full, plus his request in this session to *"check for bugs and security and other issues also"*.
**Tree examined:** `master` = `0670e0bb4db8160469d1f092ab0b088aa4b76220`. **Nature:** research, testing and a report. No
source, test, configuration, existing document or memory file was changed. This file is the only addition to the
repository (§2.4 says exactly how it got there).

**Second pass (2026-11-05, same session), on the owner's request to *"double verify and research about everything once
again … don't miss out anything … test everything"*:**
- **Codex:** every one of its 102 Appendix B ledger entries was re-run with the fixture state it used (§3.3); every
  source link in its §§1–18 was checked (§3.3); and its naming claims were checked on the web.
- **Prompt:** every requirement was mapped to this report (Appendix E).
- **Surfaces neither review had run were tested** (§6.7): the L3 launcher and attestation seam; broker and guest loss
  mid-run; strict anchored roots; the Guard seal; the proposal → approval loop; the HTTPS egress client against a
  local server that logged what reached it; endpoint-bound secrets; Python; a signed driver through a separate device
  process; drone and satellite simulations; multi-package locking; long LSP and MCP sessions.
- **Corrections, all recorded where they land:**
  - F7 is downgraded: no reachable panic was found.
  - F1 is reframed: the null device is undocumented, not unenforced.
  - The WASM parity gap is wider than first written: three constructs, not two.
- **New findings:** F12–F18 (§6.3).
- **Decisions:** §1 is unchanged. None of the new evidence moves a verdict.

**Third pass (2026-11-05, same session), when the owner asked for the repository's other documents to be updated from
this report.** Two claims were re-checked before they became work rows, and both are corrected here, in place:
- **The WASM higher-order gap is narrower than written.** Only a **non-generic** function with a function-typed
  parameter fails (`fn apply(f: fn(Int) -> Int, x: Int)`, with a named function or a lambda: DL1201). The generic
  spelling `fn apply[T, U](f: fn(T) -> U, x: T) -> U` compiles and prints the interpreter's answer on WASM, with a named
  function and with a lambda (OBSERVED HERE, the same release build; `crates/delulu-wasm/tests/named_fn_parity.rs`
  witnesses the generic form). The first two passes had tested only the non-generic form.
- **README has no "production ready" status box.** That box is in `HANDOFF.md` (*"Status: PRODUCTION READY WITH
  DOCUMENTED DEPLOYMENT REQUIREMENTS"*), and since 2026-11-02 it says that it covers containment and deployment only, not
  the everyday language. §12 is corrected.

The findings went into `docs/REMAINING_WORK.md` as rows (2.41–2.44, 3.25, 4.88–4.92, 6.54–6.66, 7.27–7.28), and the
records were updated from this report (`docs/CLOUD_SYNC_LOG.md`, 2026-11-05 entry). With them the Survey was
regenerated, so §2.4's "the map does not list this report" no longer holds. No decision was recorded: §13's owner
decisions are still his.

**Follow-up (2026-11-05, same session): the AI-first compiler and execution-pipeline audit.** The owner commissioned
it with `docs/prompt/DeluluLang_Claude_Cloud_AI_First_Compiler_Pipeline_Audit_Prompt.md`. Its report is
[`DeluluLang_AI_First_Compiler_Execution_Pipeline_Audit.md`](DeluluLang_AI_First_Compiler_Execution_Pipeline_Audit.md).
§18 below records what it changed here: each old claim, the new observation, its evidence, and the effect on the
recommendation. Nothing above §18 was rewritten for it.

**Second follow-up (2026-11-05, same session): a third verification pass**, on the owner's request to *"double
verify and research about everything once again"*, with no sub-agents. §19 records it in the same form. One citation
above was corrected in place (C15's `build.rs` line); nothing else above §19 was rewritten.

Evidence labels, used throughout: **OBSERVED HERE** (run in this cloud session, output retained) · **CURRENT SOURCE**
(read in `0670e0b`) · **CURRENT CI** (a named run whose completed result and SHA I read) · **CODEX REPORTED** (from the
Codex review only) · **HISTORICAL DOC/DECISION** · **OFFICIAL EXTERNAL** (vendor page or repository, fetched
2026-11-04) · **INFERENCE / RECOMMENDATION** · **UNKNOWN / BLOCKED**. Outcome words in result tables keep five things
apart: **true failure**, **expected authority denial**, **unsupported-capability refusal**, **handled application error**,
and **passed useful task**.

---

## 1. Executive decision

**Architecture: option 3, CLI-first.** DeluluLang should be an AI-native programming language whose main product is
the `delulu` CLI. The CLI should sit on one reusable, versioned machine API, with the compiler, checker and runtime
inside it and a thin optional editor client on top. It should **not** be CLI-only (shell strings and scraped terminal
text), it should **not** be rebuilt, and it should **not** become an agent runtime that treats the language as
optional. The language's checked effects and computed authority are the only part that no competitor examined here
supplies. *(INFERENCE, from §§4, 6, 11.)*

**Compiler and VS Code.** No separate compiler product exists to remove. The `delulu` binary *is* the compiler
(`delulu --help`: *"the DeluluLang compiler and runtime"*), and the VS Code extension is a 409-line launcher for
`delulu lsp`. Keep both as they are. Stop adding editor features. *(OBSERVED HERE, CURRENT SOURCE.)*

**Primary AI user now:** coding and software agents that write **bounded local utilities**: read a structured input,
validate it, transform it, write a result, and report a machine-readable outcome under least authority. Multi-agent
delegation comes second; edge and simulation third. Physical systems stay at simulation tier. *(INFERENCE.)*

**OpenShell verdict: OPTIONAL BACKEND** (keep DeluluLang's own stack; keep OpenShell as a tested L3 launcher and policy
target; re-evaluate *PREFERRED HYBRID* for Linux server deployments once the gates in §8.6 pass). Do not replace the
Guard, broker, leases or the guest/host channel with OpenShell, now or later. They enforce DeluluLang semantics OpenShell
does not know about. The one piece of OpenShell overlap worth giving up is **further investment in the Firecracker microVM
layer (PS-C)**: freeze it, and send VM-grade isolation needs to external launchers, OpenShell included.

**Repository strategy:** **narrow the 2.0 scope and simplify incrementally**: no rewrite, no aggressive crate-splitting.
The code is disciplined, heavily tested, and its security claims held against every hostile probe I ran (§6.4). What is
wrong is *proportion*: 199k lines of Rust, 41 top-level commands, 208 recorded decisions, and still no way for a program to
read its arguments, parse JSON, call `sqrt`, or set an exit code.

**The five decisive actions, in order:**

1. **Hold the 2.0 launch behind an "ordinary agent utility" bar** (§13, Phase 1): argv and selected environment as
   capabilities; an explicit application outcome and exit code; JSON parse and serialise with size and depth limits;
   math and checked arithmetic; string search, formatting and interpolation; methods on user types. On WASM, the
   *documented* spellings must work: today `for i in range(0, n)`, a non-generic function with a function-typed
   parameter (the generic spelling compiles; third pass), and an `if` without `else` used as a value inside a `match`
   arm all fail to compile there. Most of this bar is core
   language, so it needs RFCs (14-day comment periods) or a recorded decision to unfreeze the core (§13).
2. **Fix the machine-contract defects agents trip on** (§6.3, §9):
   - *(added by §19, first)* `p.get` taking a plugin's export at a narrower effect row than its own (RW 4.95);
   - `test` reporting success when nothing ran;
   - `fix` exiting 0 with errors left;
   - null-device actuation reported as `COMMANDED`;
   - grant hints that are not this program's (DL0703, DL1303);
   - `run --json` putting two different things on stdout;
   - a guest killed mid-run leaving no reason in the run report;
   - exit 2 ("usage") on a closed pipe;
   - `--version` printing `commit: null` for every source install.
3. **Generate one capability matrix from the code** (primitive table × engine × containment × OS) and expose it as
   `toolchain --json` and a `check --target …` preflight, so an agent learns before running that a plan cannot run.
4. **Freeze breadth and consolidate the surface:** no new PS, P8 or P9 slices and no new commands until (1)–(3) land.
   Then pull `cli.rs` (11,866 lines) apart into services with typed requests and results, which the CLI, MCP and LSP
   share. Today MCP shells out to the CLI for every call.
5. **OpenShell:** keep it as an optional backend. Bump the pin from v0.1.2 to v0.1.7 and re-run both witnesses (the
   integration is five releases stale). Then run the proof of concept in §8.5, and only on its evidence consider *PREFERRED HYBRID*.

**Non-actions:** do not delete the checker, compiler, LSP, conformance, formal models or security witnesses; do not make
a graph database the canonical source (§11.2); do not build GPU, robotics or AGI-named subsystems; do not market
"whole-system proof", "safe autonomy" or "runs everywhere".

---

## 2. Research scope, snapshot and limitations

### 2.1 Snapshot (OBSERVED HERE)

| Item | Value |
|---|---|
| Working directory | `/home/user/delululang-test` (cloud checkout; no laptop paths are accessible) |
| Branch / HEAD | `claude/dazzling-goodall-iaapyw` at `0670e0bb4db8160469d1f092ab0b088aa4b76220` = `origin/master` |
| Cleanliness at start | `git status --porcelain` empty; `git stash list` empty |
| Origin | `https://github.com/jessesuniljs1-collab/delululang-test` (fetch and push) |
| `git fetch origin` | run once at start (CLAUDE.md, *GitHub first*); **no ref moved** (`HEAD..origin/master` empty). Fetch metadata may have been refreshed, so this session made Git metadata writes, though no ref changed. |
| Other branches | `launch/2.0.0` `728a582` (prepared, not launched: 1,044 commits not on `master` by ancestry, but only **36 files** of tree difference, all docs, release and editor packaging); `autosave/laptop` `052c2ce` (2026-11-04; per the sync log it holds only the prompt file); `autosave/dogfood` `ba3f0f2` (2026-10-24); `autosave/laptop-dl-launch`, `rc/1.0.0-drill`, `archive/superseded-2026-10-26`: historical |
| Change since Codex's `e97c747` | 3 commits (`f45c565`, `83a506f`, `0670e0b`), **records only**: 9 files (`git diff --name-status e97c747..HEAD`), no source. **Every code-level finding of Codex applies to this exact code.** |
| OS / hardware | Linux 6.18.44 x86_64 (Ubuntu 24.04 userland), 4 vCPU, 15 GiB RAM, no swap, **no `/dev/kvm`**, no `newuidmap` |
| Toolchain | Rust 1.96.1 (pinned), Python 3.13.18, Node 22.23.4 |
| Build | `cargo build --locked --release -j 4 -p delulu -p delulu-survey`, `CARGO_TARGET_DIR` in the session scratch directory, **default features** (`python`, `net`): **4 min 18 s**, exit 0. `delulu` = 41,768,640 bytes, SHA-256 `e82a774332935407b7436d55d3172811c5b1429df9da0d86f9a34277771a676a`, dynamically linked to `libpython3.13.so.1.0` |
| Build identity | `delulu --version --json` → `"delulu_version":"1.0.0","commit":null,"target":"x86_64-unknown-linux-gnu"` |
| State isolation | every run used `DELULU_HOME` and `DELULU_STATE_DIR` in scratch; programs, fixtures and logs lived only in scratch |
| Agents | none in the first pass. **Second pass:** the session's direct page-fetch tool was withdrawn mid-session, so one *page-fetch* helper (the harness's stand-in for that tool; I did not choose its model) read seven public URLs (§3.3, §8.1). It wrote nothing and ran nothing. Its one consequential claim, that `github.com/delulu-lang` is a 404, I re-checked myself with GitHub's user search (0 results) |
| Second-pass sync (2026-11-05) | `git fetch origin`: no ref moved; `origin/master` still `0670e0b`; the only new ref is this branch's own report commit |
| Scheduled routines (read only) | both **disabled** (`enabled: false`): `trig_01HFG6usfzmDAPQigiDu9DLq` (on demand, last fired 2026-11-02 07:11 UTC, run succeeded) and `trig_01GG41tCGxZXyt7bZUid8Vuu` (cloud period, ended). Nothing was changed, started or fired |

### 2.2 Health baseline: Survey and doctor first (OBSERVED HERE, clean tree)

| Check | Result |
|---|---|
| `delulu-survey check` | `ok: the Survey matches the tree (1618 nodes, 18863 edges)`, exit 0 |
| `delulu-survey findings` | **0 errors, 7 warnings, 37 notes** (exit 0). The 7th warning is the prompt naming *this* report before it existed |
| `delulu doctor --check` (and `--json`) | exit 0, **`ok: 28 check(s) passed`** on this VM (29 on the laptop per its records; the count is machine-dependent). Notes: LEGACY root issuance, L2 absent (no KVM), no identity separation (no `uidmap`) |
| Write behaviour | I read `survey check` (`Repair::ReportOnly`) and `doctor --check` (`check_only`) in source before running them. `doctor`'s `writable()` probe creates and removes a temp file in the state directory, so I pointed that at scratch. `git status` stayed empty. **No `survey build` and no plain `doctor` was run.** |

### 2.3 CI read (CURRENT CI; read only, nothing dispatched)

| Run | SHA | What I read |
|---|---|---|
| [`41929573373`](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/41929573373) push, 2026-11-04 22:56–23:18 UTC | `0670e0b` (HEAD) | **success: 14 jobs success, 2 skipped** (`heavy-gates`, `miri-slow`, by design on push). Ran: test on ubuntu-24.04, macos-latest and windows-latest; arm64; lints; supply-chain; formal (TLC, Z3 58 obligations plus 14 teeth mutants, Lean plus 3 teeth mutants); fuzz; editor; microvm (KVM runner: build guest image, lifecycle tests, hostile-guest red team); microvm-reproducible; miri (diag, atlas, ffi) |
| [`41813603704`](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/41813603704) schedule | `e97c747` | success, **18 jobs**: confirms Codex's "18/18" |
| [`41619070519`](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/41619070519) schedule | `e97c747` | failure (recorded as a runner disk death, RW 7.26) |
| [`41394552103`](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/41394552103) `openshell.yml` push | `4e63eb7` (2026-11-02) | success; prover job only, **OpenShell v0.1.2** |
| [`37234907676`](https://github.com/jessesuniljs1-collab/delululang-test/actions/runs/37234907676) `openshell.yml` dispatch | `fc676d1` (2026-10-04) | success: the **last** live run of the guest inside a real OpenShell sandbox (v0.1.2) |

**A green CI run is not a claim that a practical task works.** §6 is about tasks.

### 2.4 What this session did *not* do, and what changed in the tree

- Not accessible or not done: the owner's Windows laptop, WSL/KVM lab and installed `delulu`; his local Claude memory
  (`docs/assistant-memory/` and HANDOFF §11 were read instead); macOS and Windows execution (CI only); KVM / L2;
  a second OS account; OpenShell, Zerolang or Mojo installs (none was installed or run: no `curl | sh`); paid model calls;
  real networks or devices.
- **Tree change:** this report, `docs/review/DeluluLang_Claude_Independent_Strategic_Verification.md`, and nothing else.
  Adding it **makes `delulu-survey check` and `doctor --check` report the map as stale**: the map does not list this file.
  That is caused by this report, not a pre-existing defect. Measured after writing it: `survey check` exit 1 (`SURVEY.md`,
  `DISCREPANCIES.md` and `survey.json` behind the tree); `doctor --check` exit 1 (one problem: survey freshness);
  `survey findings` **0 errors, 6 warnings, 37 notes**. The prompt's own warning about this missing file disappeared, and
  the report adds no finding of its own. I did not regenerate the map, because the prompt forbids it. The next session
  that runs `survey build` makes it fresh.
  `docs/REPOSITORY_STRUCTURE.md` §5 already lists `docs/review/` as a group, so its structure test still accounts for the file.
- **Delivery:** a cloud container is discarded when the session ends, so to keep the report retrievable it is committed
  **alone** to this session's branch `claude/dazzling-goodall-iaapyw` and pushed there. There is no pull request, no
  merge, no change to `master`, and no tag. `ci.yml` runs on pushes to `master` or `main` and on pull requests, so this
  push starts no CI run.

---

## 3. Independent verification of the Codex review

Codex's review (`docs/review/DeluluLang_Codex_Strategic_Review.md`, unchanged) was read in full for §§1–18; Appendices
A, B (selected entries), C, E and F were audited where they back a conclusion. **Every one of the 35 Appendix A source files
was rebuilt from the review and matched the SHA-256 Codex recorded** (OBSERVED HERE). Its appendix is therefore an exact,
reproducible record, the strongest thing about the review. Because the code has not changed since `e97c747`, a
difference between its results and mine is a platform difference or an error, never staleness.

### 3.1 Claim ledger

| ID | Codex claim (location) | Codex's evidence | Replication here | Verdict | Impact |
|---|---|---|---|---|---|
| C01 | The `delulu` CLI is the front door and depends on syntax, check, runtime, WASM, Atlas, broker and Survey (§3.1) | Source | `crates/delulu/Cargo.toml` `[dependencies]`; every probe below ran through it | **CONFIRMED** | No separate compiler to delete |
| C02 | No detachable obsolete compiler product (§3.1, §6) | Source | One binary, whose help calls it "the DeluluLang compiler and runtime" | **CONFIRMED** | Owner's option "remove the compiler" removes nothing; keep internals |
| C03 | VS Code is a thin client of `delulu lsp` (§3.1) | Source | `editors/vscode/extension.js` (409 lines) starts `{command: serverPath, args: ["lsp"]}`; LSP probe: 15 capabilities, DL0501 published, clean shutdown | **CONFIRMED** | Keep; cost is small |
| C04 | 13 members (9 shipped + 4 tooling), 199,342 Rust lines (§3.2) | Survey count | Recount: 340 `.rs` = 199,342 lines; crate table identical; doctor "13 workspace members, 9 shipped" | **CONFIRMED** | Same code |
| C05 | Largest files: `cli.rs` 11,866, `codegen.rs` 6,510, `brokerd.rs` 5,313 … (§3.2) | Count | Identical | **CONFIRMED** | §12 |
| C06 | Fresh build 21 min, 35.45 MB portable Windows binary (§2.3) | Observed (Windows) | Linux: default profile 4 min 18 s (`-j 4`), 41,768,640 B (33,472,536 B stripped); portable profile (`--no-default-features --features net`) 5 min 27 s (`-j 3`), 41,652,600 B (33,403,408 B stripped), and it runs the utility correctly | **NOT REPRODUCIBLE HERE** (platform); order of magnitude **CONFIRMED** | Python costs only about 70 KB (it is linked dynamically); the bulk is Wasmtime/Cranelift, networking and crypto. A smaller edge profile means dropping or replacing those, not Python |
| C07 | Survey and doctor failed freshness; the untracked `docs/prompt/` may explain it (§2.4) | Observed | On the clean tracked tree: **fresh**, 0 errors | **PARTIALLY CONFIRMED**: Codex's caution was right; no committed defect | None |
| C08 | 18/18 CI on `e97c747` (§2.4) | CI | `41813603704`: 18 jobs, success | **CONFIRMED** | Run read |
| C09 | `root.args()`, `parse_json`, `sqrt`, `p.norm()`, `+=` refused; interpolation prints braces (§1, §5) | Observed | DL0405, DL0301, DL0301, DL0405, DL0202 respectively; `"hello {name}"` printed literally, and the only hint is a DL0414 "`name` never read" warning | **CONFIRMED** | P0 gaps (§6.2) |
| C10 | `range` runs on the interpreter, fails WASM codegen (§1) | Observed | `DL1201 … a call to range (not a compilable function)`, exit 1, **at compile time: nothing ran** | **CONFIRMED** (failure is pre-execution) | `range` is the *documented* loop spelling (RW 2.36) |
| C11 | Device APIs rejected by WASM (§1) | Observed | `DL1201 … the method .actuator` | **CONFIRMED** | Matrix needed |
| C12 | Sandboxed `--seed` unsupported (§4.2) | Observed | exit 2, "Nothing ran" | **CONFIRMED** | Honest refusal |
| C13 | Missing test path → success with zero tests (§1, A05) | Observed | In a package: `delulu test /no/such/dir` → exit 0, `0 passed, 0 failed`; `--json` identical to "no tests"; a filter matching nothing is the same | **CONFIRMED** | Agents read success when nothing ran |
| C14 | Package entry file with an import fails; package dir works (§1) | Observed | `run src/main.delulu` → DL0301 `twice`; `run .` → `42` | **CONFIRMED** | RW 6.52 |
| C15 | Installed `delulu` stale, 1.0.0 (§1) | Observed (laptop) | Laptop not accessible. **New:** a fresh source build reports `commit: null`; only `DELULU_BUILD_COMMIT` (the packaging script, the release workflow) fills it (`crates/delulu/build.rs` lines 9–15; first cited as line 11, which sets the target; §19) | **NOT REPRODUCIBLE HERE**; Codex's "RW 6.53 partly stale" **MODIFIED**: README's `cargo install --path` still yields an unidentifiable binary | Identity not fixed for source installs |
| C16 | `run --json` prints the program's own stdout on success and an envelope on failure (§4.3) | Observed | Same: success → `{"ok":true,"value":42}`; no grant → DL0703 envelope | **CONFIRMED** | Contract hazard (§9) |
| C17 | MCP: 12 read-only tools; a failing `check` is `isError:false` with `errors:1`; option-like path rejected; unknown tool → -32602 (§4.3) | Observed wire | Same, on protocol `2025-06-18` | **CONFIRMED** | By design; must be taught |
| C18 | MCP buffers subprocess output with no byte cap (§7) | Source | `crates/delulu/src/mcp.rs` (lines 367–410): `read_to_end`; input lines *are* capped at 16 MiB (`mcp.rs:508`). **New:** every tool call spawns a fresh CLI process | **CONFIRMED** | §7 |
| C19 | LSP initialises and publishes DL0501 (§4.3) | Observed wire | Same | **CONFIRMED** | Keep |
| C20 | Computed scope → `inconclusive`; runtime still runs it safely (§4.2) | Observed | `fs_read(scope())`, where `scope()` is a pure zero-argument function returning `"./input"`: `inconclusive`; run succeeds | **CONFIRMED** | Precision opportunity (§9) |
| C21 | No grant → DL0703; outside scope → DL0904; sentinel unchanged (§4.2) | Observed | Same on both engines, **and** under `--sandbox` (L1) | **CONFIRMED** | Core claim holds |
| C22 | 1 s wall budget stops at about 1.05 s (§4.2) | Observed | 1,014 ms (interpreter), 1,013 ms (WASM) | **CONFIRMED** | Not real-time |
| C23 | Plugin: allowed call works; overbroad grant refused; refused under sandbox (§4.2) | Observed | Same; **plus** a tampered `.dpx` (one byte flipped) is refused with "never falls back to Contained"; WASM refuses `plugin_host` (DL1201) | **CONFIRMED** + extended | §9 |
| C24 | Simulator: `COMMANDED`, envelope refusal, `READ 12.0` (§4.1) | Observed | Same with `--broker-profile sim`. **New:** without a profile, an in-envelope command still returns `COMMANDED` although no device exists (§6.3 F1) | **CONFIRMED** + new defect | Honesty gap |
| C25 | Windows medians (§8.1) | Observed (Windows) | Linux numbers in §10.1; same shape (WASM faster on loops, slower to start) | **NOT REPRODUCIBLE HERE** (platform); shape **CONFIRMED** | — |
| C26 | Only the monomorphic, closed-row fragment is machine-checked (§8.4) | Source | `docs/design/DELULU_CORE.md` (lines 7, 392); the Lean, TLC and Z3 jobs green in `41929573373` | **CONFIRMED** | Never claim whole-system proof |
| C27 | `Secret.verify` leaks one chosen bit and needs no `Cap[Declassify]` (§8.4) | Source | `DELULU_CORE.md` (lines 144–146) ("Holding two secrets buys one chosen bit per call") | **CONFIRMED** | Wording discipline |
| C28 | Default root issuance is LEGACY (§8.4) | Observed | `doctor` note; `grants delegate` with no parent auto-mints a root marked "origin unverified — DISC-1" | **CONFIRMED** | Deployment posture (§14) |
| C29 | Machine prose says grants are "a person's decision" (§7) | Source | MCP `initialize` instructions say exactly that | **CONFIRMED** | Reword to "an authorised principal's" (kind-blind) |
| C30 | `sandbox probe` prints stale L3/L4 lines (§15) | Observed | "none can be configured in this build (PS-D)", "attestation is deferred (PS-D-02)" | **CONFIRMED** (RW 6.50) | Doctor/probe truth |
| C31 | 12 schemas discoverable (§4.3) | Observed | `delulu schema --json`: 12 | **CONFIRMED** | — |
| C32 | Zerolang latest commit `7e1a64d…`, 16 Sep 2026 (§9) | External | GitHub page now shows 1,188 commits and no releases; date not shown | **STALE** | Re-check before relying on it |
| C33 | Mojo stable 1.2.0, 22 Oct; Windows only via WSL (§9) | External | Same (mojolang.org, requirements page) | **CONFIRMED** | — |
| C34 | Koka v3.2.8, 17 Sep 2026 (§9) | External | Two searches (2026-11-04/05) found v3.2.3 as the newest visible tag (a mirror page, "18 Mar", no year) and Alpine's edge package still at 3.2.2 on 2026-02-15; no source shows v3.2.8 | **INCONCLUSIVE** (unsupported by what I found) | Irrelevant to the decision |
| C35 | Naming facts (§18): KingLang repositories, a Chinese patent naming KingLang, a pending DELULU application | External | **Second pass:** `github.com/ShubhamBhut/kinglang` exists ("An interpreted programming language written in rust"); `github.com/publidoo/kinglang` exists ("Language app"); patents.google.com shows CN110737431A, which mentions "KingLang" and was granted as **CN110737431B on 2023-07-14**; a trademark aggregator lists DELULU serial 99795106 (Decart.Ai Inc., filed 2026-04-29, classes 9/41/42) as pending | **CONFIRMED** for the facts checked; **no legal conclusion** | Background only |
| C36 | Device simulation *"in Windows AppContainer at L1"* (§1) | Observed (Windows) | On Linux the same program runs at L1 under Landlock and seccomp, `fully_enforced:true` | **NOT REPRODUCIBLE HERE** (Linux equivalent CONFIRMED) | — |
| C37 | Survey: `crate:delulu-check` impact 242, affected-by 43; `crate:delulu-wasm` impact 132 (§7) | Observed | Identical numbers (§4.3) | **CONFIRMED** | — |
| C38 | `.dwx` artifact 1,196 bytes; pure plugin `.dpx` 1,307 bytes (§8.1) | Observed | 1,196 and 1,307 bytes here too | **CONFIRMED** | Artifacts are tiny; the runtime is the size |
| C39 | Codex's cited e-stop 9.81 s under a held audit chain (§12) | Recorded | RW 4.53 (closed 2026-10-31, D-V2-198) records exactly that | **CONFIRMED** as recorded | Software e-stop is not an interlock |
| C40 | Cargo repository metadata differs from the test origin (§11) | Source | Workspace `Cargo.toml` line 30 names `https://github.com/delulu-lang/delulu`; **`delulu-lang` is not a GitHub account** (404; GitHub user search: 0 results); `launch/2.0.0` already names `jessesuniljs1-collab/DeluluLang` | **CONFIRMED**, extended (F15) | Pre-launch namespace hygiene |
| C41 | Java's Security Manager is no longer a sandbox strategy (§9) | External | JEP 486 (JDK 24) permanently disables it | **CONFIRMED** | — |
| C42 | Zerolang ships data-format modules (§9) | External | Its repository docs (indexed copy) describe a `std.json` module (validation, parsing, cursor access) | **CONFIRMED** (secondary index) | DeluluLang's JSON gap is a competitive gap |
| C43 | Appendix B: 102 executed commands and outputs | Observed | **All 102 re-run** (§3.3): 101 of 101 comparable exit codes equal; 87 identical in exit, stdout and first stderr line; 14 differ only by platform, line endings, build profile or timing | **CONFIRMED** (whole ledger) | Codex's evidence is reproducible |
| C44 | Every source link in §§1–18 (26 distinct) | Source | All resolve to the cited text in `0670e0b`; `Cargo.toml#L24` is now one line above the `[[bin]]` it means | **CONFIRMED** | — |

### 3.2 Codex's recommendations

| Recommendation | Verdict | Why |
|---|---|---|
| Keep the language and checker; do not rewrite | **AGREE** | The effect/authority core works and held every attack I ran (§6.4). |
| Keep a lightweight LSP and a thin VS Code client | **AGREE** | 409 lines of JS. Removing it saves nothing; the LSP is also the only persistent analysis process. |
| CLI-first, usable through machine APIs | **AGREE, MODIFY** | The "machine API" should be an in-process service layer with typed requests and results, which MCP, LSP and CLI call. Today MCP forks the CLI per call (C18). Measured startup (~7 ms) makes forking acceptable for small programs, but it duplicates contracts. |
| Fundamentals first | **AGREE, MODIFY** | Add three things Codex did not list: **WASM support for the documented constructs** (`range`, non-generic higher-order functions; §6.2), **JSON *escaping*** (a program cannot even emit correct JSON for arbitrary strings today), and **catchable scope refusals** (DL0904 aborts the program; `match … Err` never runs). |
| Target capability matrix | **AGREE, strongly** | Generate it from `prim.rs`, `codegen.rs::is_compilable` and the guest channel's surface list, not by hand. |
| Versioned outcomes | **AGREE** | Defects F2–F5 (§6.3) are all outcome-contract defects. |
| Preserve Authority and the Guard | **AGREE** | Also against OpenShell (§8). |
| Simplify the CLI and semantic duplication | **AGREE on the CLI; MODIFY on semantics** | Do it narrowly: one shared *name-resolution and callable-compilability table*, consumed by checker, interpreter and codegen. Not a new IR. The recorded defects (LATE-LET-1, LOCAL-CALLEE-1, CTOR-SHADOW-1, INDEX-TARGET-1) were all lookup and walk divergences. |
| Hold the broad launch until A01–A11 and A18 | **AGREE, MODIFY** | Add A19 (WASM parity for documented constructs) and A20 (no success reported for actions that did not happen; F1). |
| Order: installation → tasks → matrix → workflows → performance → specialists | **MODIFY** | Run installation identity (small) **in parallel** with language basics. The basics are the long pole and the owner's own complaint. |
| Keep the working name; clear a final one | **No position** | Outside this decision; Codex's legal statements are background, not clearance. |

### 3.3 Second pass: the whole Codex ledger re-run (OBSERVED HERE)

All 102 entries of Codex's Appendix B were parsed from the review, mapped from Windows to this checkout, and re-run on
the release build. Before each entry I recreated the fixture state it used:
- the first drafts of `basics` (B002–B003) and `outside` (B010–B011);
- the four task-input variants (B044–B051);
- the original manifest with no `[authority]` (B052–B056);
- the in-package plugin hosts (B070–B072).

The sentinel `forbidden.txt` kept its bytes throughout.

| Result | Entries |
|---|---|
| Exit code equal to Codex's | **101 of 101** comparable (B096, the `rustc` control, is not a `delulu` command) |
| Exit, stdout and first stderr line identical | **87** |
| Same exit, stdout differs | 12, each explained: `target`/`commit` in `--version` (B001; Codex injected `DELULU_BUILD_COMMIT`, I did not); Windows vs Linux probe text (B063); byte offsets 1–3 lower in diagnostics because Codex's sources were CRLF (B035, B040, B060, B062, and the plugin's `dir` hash in B068); a trailing blank line (B012–B013); a `ms` field (B085); the Survey seeing documents added since `e97c747` (B080); Python compiled in here, not in Codex's build (B102) |
| Same exit and stdout, stderr differs | 2 (B064, B100): AppContainer text on Windows, Landlock/seccomp text and "no `uidmap`" on Linux |

**Every one of Codex's executed observations reproduces on Linux.** No result reverses. Its §§1–18 links (26 distinct)
resolve to the cited text. Its Appendix E (a requirement map and a zero-changed-files integrity statement) and Appendix F
(proposals A–G, eleven scenarios, a list of what it left partial) are consistent with the tree. Of what Appendix F lists
as *not* done, this session ran: two independent broker principals with ancestor revocation, the full `edit`
transaction, a signed device-adapter path, and the L3 and attestation boundary (§6.7).

---

## 4. The architecture as it actually is

### 4.1 Size (OBSERVED HERE, `git ls-files`)

1,161 tracked files: 340 `.rs` (199,342 lines), **309 `.md` (103,797 lines)**, 258 `.delulu` (5,270 lines), 69 JSON
(30,371), 55 TOML, 23 Python, 23 shell, 7 JS. `Cargo.lock` holds 348 packages (Wasmtime 48.0.8, PyO3 0.25.1, rustls
0.23.46).

| Crate | `src/` lines | `tests/` lines | Role |
|---|---:|---:|---|
| `delulu` | 48,011 | 45,423 | CLI (41 top-level commands), LSP, MCP, broker daemon, sandbox host/guest, microVM, OpenShell export, doctor |
| `delulu-runtime` | 24,891 | 1,900 | interpreter, primitives, devices, egress client, channel, plugins, FFI, Python |
| `delulu-check` | 17,196 | 1,004 | resolution, types, effects, authority, reference capabilities, manifests |
| `delulu-broker` | 14,548 | 1,306 | ⊑ lattice, grant tree, leases, Guard, audit chain, secrets, certificates |
| `delulu-wasm` | 14,100 | 4,900 | WASM codegen, host ABI, `.dwx` |
| `delulu-syntax` | 7,632 | 0 | lexer, parser, formatter, morphs |
| `delulu-survey` | 4,714 | 1,060 | repository map (tooling) |
| `delulu-diag` | 4,032 | 99 | 157 DL codes, envelopes, repairs |
| `delulu-measure`, `-conform`, `-atlas`, `-registry`, `-fuzz` | 2,982 / 2,171 / 2,294 / 1,579 / 798 | — | measurements; conformance (entrenched); program map; registry; fuzzing |

Within `delulu`, **containment** (`jail`, `guest`, `microvm`, `identity`, `attest`, `boundary`, `sandbox`, `openshell`,
`launcher`, `pipe_channel`, `breakglass`, `budget`, `foreign_worker`) is 13,255 lines; the **custody daemon side**
(`brokerd`, `broker_*`, `monitor_cmd`, `fleet`, `deploy`, `policy`) 9,680; the runtime's **channel, egress and `beneath`**
6,149. The single file `cli.rs` is 11,866 lines, and `run` alone takes roughly 40 flags in a 215-line top-level help.

### 4.2 Data flow and trust boundaries (CURRENT SOURCE, OBSERVED HERE)

```text
 agent / shell / editor
   │  argv + --json            MCP (12 read-only tools; each call forks `delulu <cmd> --json`)
   │                           LSP (persistent process, 15 capabilities)
   ▼
 delulu CLI ──► syntax ─► check (types · effects · rcaps · authority) ─► Atlas · edit/fix · within · grants diff
   │                                      │
   │                      ┌───────────────┴──────────────┐
   │                interpreter (L0, in-process)     WASM codegen → wasmtime (L0) / .dwx
   │                      │      both call the same primitive table + custody
   ▼                      ▼
 custody: grants (CLI) | leases (broker daemon, ⊑, revocation, audit chain) | Guard (sealed/guarded classes)
   │
   ├─ L1 --sandbox: guest process (Landlock+seccomp | AppContainer+Job | Seatbelt) holds handles only;
   │      every effect crosses the channel; host performs it under custody   [boundary confirmed before program is sent]
   ├─ L2 --isolation microvm: same guest inside Firecracker (Linux+KVM, image built from source)
   └─ L3 --sandbox-backend external:CMD: operator launcher (Docker+gVisor recipe, OpenShell via ssh) — guarantees "unknown"
 egress: one host-side HTTPS client (hosts × methods × paths, pinned DNS, special-use refused)
 devices: device broker (sim | hw adapter subprocess), envelopes, dead-man, e-stop via broker
 Survey = this repository's map (maintainers). Atlas = a program's map (users). doctor = install/posture/map health.
```

### 4.3 Survey blast radius (OBSERVED HERE, `delulu-survey impact` / `affected-by`, nodes reached)

| Node | impact | affected-by |
|---|---:|---:|
| `crate:delulu-syntax` | 258 | 21 |
| `crate:delulu-check` | 242 | 43 |
| `crate:delulu-broker` | 200 | 66 |
| `crate:delulu-runtime` | 180 | 103 |
| `crate:delulu-wasm` | 132 | 116 |
| `crate:delulu-survey` | 116 | 15 |
| `crate:delulu-atlas` | 108 | 50 |
| `crate:delulu` | 104 | 193 |
| **every one of 13 modules of `crates/delulu/src` I queried** (`cli`, `openshell`, `microvm`, `jail`, `guest`, `brokerd`, `lsp`, `mcp`, `fleet`, `attest`, `monitor_cmd`, `sandbox`, `identity`) | **106 each** | 0–124 |

**Finding (Survey limit):** inside the CLI crate, the Survey cannot tell one module's blast radius from another's: removing
`openshell.rs` "reaches" exactly as much as changing `cli.rs`. It is good at crate-level planning and silent at the level a
deletion decision needs. Any removal in §12 must be planned from `cargo` references and tests, not from this number.

---

## 5. AI-first mission: the first customer and workload

| Rank | User / workload | Feasible now? | Evidence | Next decisive evidence |
|---:|---|---|---|---|
| 1 | Coding and software agents writing bounded local utilities | **Almost**: blocked by inputs, JSON, outcomes | Codex's utility ran on both engines and under L1 (OBSERVED HERE); check→fix→edit→authority→run loop works (§6.5) | the §13, Phase 1 bar |
| 2 | Multi-agent delegation (parent/child principals) | **Yes, as custody**: narrowing, widening refusal and ancestor revocation work | §6.6 | two OS accounts; replay and idempotency semantics |
| 3 | Edge compute (bounded aggregation, Linux/ARM) | Compute yes (WASM); deployment profile no | §10 | a named ARM board; a minimal runtime profile |
| 4 | Robot, vehicle, drone, ground **simulation** | Logic yes; simulator device yes | Codex's simulation and device programs reproduced | seeded sandboxed simulator; independent no-command proof |
| 5 | Real physical supervisory agents | **No** | no in-tree driver for any real device (HISTORICAL DOC) | §14 |
| 6–7 | Safety-critical control; onboard spacecraft | **No**; research only | — | domain assurance cases |

AGI, ASI and "any future AI" are served by one property only: **kind-blind, explicit, self-describing interfaces** (grants,
schemas, versioned outcomes). There is no meaningful "AGI-ready" test, and nothing here should claim one.

---

## 6. Complete-program gap audit: what I ran

Programs, fixtures and outputs are in Appendix A and B. All runs used the fresh release binary.

### 6.1 What works (passed useful tasks, both engines unless stated)

| Program | Result |
|---|---|
| Codex `basics_while`, `utility`, `workflow`, `delegation`, `edge`, `simulation` | identical outputs to Codex on interpreter and WASM; `utility` also under `--sandbox` (L1, `fully_enforced:true`) and `hostile-agent` |
| Codex `package/` (two modules) | `run .` → `42` |
| Everyday corpus (18 programs, mine): for-in list, `map`/`filter` with closures, generic `first[T]`, sum types and `match`, `Result` and `?`, `Map`, string `split`/`trim`/`contains`/`to_upper`, `sort`, float and int arithmetic, deep recursion, nested records, `while`/`break`, `join` | **12 identical on both engines**; overflow (DL0901), divide-by-zero (DL0902) and index-out-of-bounds (DL0903) **fault identically** on both |
| Plugin `shout` (verified class) | `hello!`; overbroad grant refused; tampered artifact refused; no fallback |
| Delegation through the broker daemon | §6.6 |

### 6.2 Fundamentals: minimum failing cases, severity-ranked

| Sev | Gap | Minimal case → exact result (OBSERVED HERE) | Layer |
|---|---|---|---|
| P0 | No program inputs, no exit outcome | `root.args()` → DL0405 *type `Root` has no method `args`*; handled errors exit **0** (Codex `utility` "error" JSON, plugin refusals) | capability + `main` outcome |
| P0 | No JSON read **or correct write** | `parse_json` → DL0301; no JSON or escape primitive in `prim.rs`'s `Str` table (`len, trim, contains, starts_with, split, to_upper, replace, chars …`); JSON is built by concatenation, so a value containing `"` produces invalid JSON | pure stdlib |
| P0 | **WASM cannot compile documented constructs** | `for i in range(0,4)` → DL1201; `fn apply(f: fn(Int)->Int, x: Int)` called with **any** function (named or lambda) → DL1201 *"a call to `apply` (not a compilable function)"*. Cause: `codegen.rs:1027 is_compilable` requires every parameter to have a `wasm_ty`, and function types have none. **Third pass, narrowed:** the generic `fn apply[T, U](f: fn(T) -> U, x: T) -> U` compiles and agrees with the interpreter (a generic function is inlined; `named_fn_parity.rs`), so the gap is the non-generic spelling. **Second pass, a third construct:** `match … { Some(p) => if c { found = true }, … }` → DL1201 *"an `if` without `else` used as a value"* (my telemetry simulation, which runs on the interpreter). None of the three is named in RW 3.4 or 2.36 | codegen |
| P0 | `test` succeeds when nothing ran | §3 C13 | CLI |
| P1 | No math, no checked arithmetic | `sqrt(4.0)` → DL0301 | prelude |
| P1 | Strings: no interpolation, search or indexing | `"hello {name}"` printed literally | stdlib, then syntax |
| P1 | No methods or interfaces on user types | `p.norm()` → DL0405 | language |
| P1 | Scope refusals are not catchable | `match wr.write_text("../x", …) { … Err(e) => … }` → **DL0904 aborts the program**; the `Err` arm never runs | runtime semantics (decide deliberately) |
| P2 | `+=`, bitwise, tuples, sets, `sort_by` | `n += 2` → DL0202 | syntax and stdlib |
| P2 | Authoring traps a model meets (second pass) | `else` on the line after `}` → DL0202 *"expected an expression, found `else`"*; a parameter named `cap` → DL0106 (reserved); both cost me a cycle while writing the simulations | syntax and docs: teach both in the first-program kit |

The earlier claims are **true gaps, not wrong API guesses**. I checked each against the parser and primitive table.
The interpolation case is the most dangerous: it type-checks and prints the wrong text.

### 6.3 Defects found in this session beyond Codex's list (OBSERVED HERE unless marked)

| ID (mine) | Defect | Witness | Severity |
|---|---|---|---|
| **F1 NULL-DEVICE-OK** | With no `--broker-profile`, `arm.command(...)` inside the envelope returns `Ok` (`COMMANDED`, exit 0) though **no device exists**; the run report says nothing about devices. The source states it (`delulu-runtime/src/device.rs` (lines 720–727): *"With no broker (the null device profile) an in-envelope command answers `Ok(())`"*), at odds with the CLI's own principle at `cli.rs:9476` (*"a … run that commanded nothing while reporting success would be a lie"*). **Second pass, framing corrected:** the null profile is a real `DeviceBroker` (`cli.rs:9485` → `Profile::Null`), so envelope, lease and rate checks still run (its unit tests at `device.rs` (lines 1209–1235) rely on that). Only the device is absent. This is an **undocumented mode**, not a missing check: no RW row, Book or for-agents text names it | `device_sim.delulu` with actuator and sensor grants, no profile → `COMMANDED`, then `NO READING` | **Medium** for physical-AI honesty (nothing moves, but an agent concludes it actuated). Fix: name `device_profile` in the report and the command's result, or refuse `command` without a profile |
| **F2 HINT-INVALID** | DL0703 for an actuator tells the user to pass `--grant "actuator=review/motor:DIM=LO..HI"` (hard-coded at `delulu-runtime/src/prim.rs:752`); following it gives exit 2 *"no `heartbeat_ms` — a device grant with no dead-man is not a grant"* | `device_missing.delulu` | Low–medium (breaks an agent repair loop) |
| **F3 FIX-OUTCOME** | `delulu fix` exits **0** with `fix.errors_after: 1` (widening repair skipped), and after a successful fix `summary.errors` still says **1** (`errors_after: 0`), against for-agents.md's "1 = diagnostics" | `fix app.delulu --json` without and with `--accept-widening` | Low–medium |
| **F4 TTL-IS-A-TIMESTAMP** | `grants delegate --json` prints `"ttl_millis": 1793836483844`, an absolute epoch time, not a TTL; `Denial::Expired { ttl_millis: claims.exp_millis }` repeats it (`delulu-broker/src/lease.rs`) | broker run | Low (a misread contract) |
| **F5 IDENTITY-NULL** | Every non-packaged build reports `commit: null`; doctor cannot compare PATH vs tree (RW 6.53). *Second pass:* this is **documented and intended** (`INSTALL.md`: "One built without being told its commit says `null`, never a guess"). The finding is the consequence: the README's and INSTALL's own `cargo install --path` route yields a binary nobody can later identify | `--version --json` | Medium for adoption |
| **F6 LEASE-BURNED** | A single-use lease is consumed by a run that faults on an unrelated missing grant (console), so the agent must re-mint | broker run → DL1407 on retry | Low (by design, but surprising) |
| **F7 WATCHDOG-POISON** *(hypothesis, CURRENT SOURCE, not reproduced; **downgraded in the second pass**)* | The device dead-man thread takes `inner.leases.lock().unwrap()` every tick (`device.rs:543`; 18 such unwraps in production device code), the release profile is `panic = unwind`, and a panic while a device lock is held would poison it and stop the watchdog. **Second pass:** no production code that runs under those locks can panic: `sim.rs` (lines < 275), `adapter.rs` (< 426), `verified_adapter.rs` (< 376) and `device.rs` contain no `unwrap`, `expect`, `panic!` or bare index outside the lock calls themselves. No reachable trigger was found | — | **Low**: defensive hardening only (poison-tolerant locks that *engage* the fail-state), not a fix for a known fault |
| **F8 STALE-EDIT-AS-USAGE** | A stale-hash edit refusal is exit **2** ("usage") and returns the current hash, inviting a blind retry | `edit --expect-hash 0000` | Low (contract clarity) |
| **F9 SURVEY-GRANULARITY** | §4.3 | — | Medium for maintainers |
| **F10 CHECK-MEMORY** | `check` uses about 14 KiB of RSS per source line: 224 MiB for a 16,002-line program, 315 MiB to compile it to WASM | §10.2 | Medium for edge and long LSP sessions |
| **F11 STALE-OPENSHELL-PIN** | `openshell.yml` pins v0.1.2; upstream is v0.1.7 with breaking changes in 0.1.5–0.1.7; the live runtime witness last ran 2026-10-04 | CURRENT CI, OFFICIAL EXTERNAL | Medium (integration drift) |
| **F12 HINT-NUMPY** *(second pass)* | DL1303 tells every program to `--grant foreign.python=numpy` whatever it imports (hard-coded at `crates/delulu/src/cli.rs:9713`) | `py_json.delulu` imports `json` and `os` | Low (the same class as F2: a hint that is not this program's) |
| **F13 EPIPE-EXIT-2** *(second pass)* | When the reader closes the pipe before `delulu` finishes writing (`… \| head -2`), the run exits **2**, the "usage error" code, instead of 1. It depends on how much output is still unwritten | `telemetry.delulu` with a static error: exit 1 to a file, exit 2 into `head -2` | Low (a machine reader misclassifies) |
| **F14 GUEST-DEATH-UNSTRUCTURED** *(second pass)* | A sandboxed guest killed mid-run (SIGKILL) ends the run with exit 1 and the stderr line *"the guest closed the channel without saying goodbye"*, but the `--report-out` report says only `outcome: {exit: 1, ran: true}`: no `stopped_by`, no diagnostic. A budget stop does carry `stopped_by` | kill of `__guest` after 17 ticks | Low–medium (agents can't classify the failure) |
| **F15 UNREGISTERED-NAMESPACE** *(second pass)* | `master`'s workspace `Cargo.toml` (line 30) and `editors/vscode/package.json` name `https://github.com/delulu-lang/delulu`. **No GitHub account `delulu-lang` exists** (404; GitHub user search: 0), so anyone could register it and receive the project's metadata links. `launch/2.0.0` already points both at `jessesuniljs1-collab/DeluluLang`; the VS Code **publisher** id `delulu-lang` stays on both branches, and whether it is reserved on the Marketplace or Open VSX is unknown | `git grep`, `gh api search/users` | Low now; **Medium at launch** (reserve the names, or keep `launch/2.0.0`'s fix) |
| **F16 L3-DEFAULT-PROCEEDS** *(second pass; documented, D-V2-59/D-V2-92)* | Under the default `contained` profile, an external launcher's run proceeds with `requested: contained, granted: none` and every property `unknown`; only `hostile-agent` refuses. Stated in README and stderr, so not silent, but it is a weaker-confinement run under the default profile | `l3-ok` (§6.7) | Design choice to revisit: refuse a requested profile that is not granted unless the operator names `--accept-unmeasured` |
| **F17 LEASE-IN-ARGV** *(second pass)* | A lease token, a bearer credential, can be passed only as `--lease TOKEN` (`cli.rs:832`), so it appears in `ps`, `/proc/<pid>/cmdline` and shell history. The broker socket is in an owner-only state directory, so another account cannot simply redeem it; the exposure is logs and history, and Tier-2 setups where an agent account reaches the socket | `ps -o args` during a leased run | Low; hardening: `--lease-file` or standard input |
| **F18 APPROVAL-BY-NODE-ID** *(second pass; inside the documented same-user residual)* | `grants approve <pp> --parent g_ID` approves under any node named by id. On one OS account an agent can approve its own proposal by naming its parent (a child naming itself is refused, DL0802). Only a separate OS account, or strict anchored roots with the anchor key elsewhere, makes approval a boundary | §6.7 proposal flow | Same-uid residual (category 7, DISC-1); state it next to P9-03 in `for-agents.md` |

### 6.4 Hostile probes: all held (expected authority denials; OBSERVED HERE)

`narrow("..")` and `narrow("/etc")` → DL0904. A directory symlink inside the grant pointing outside, a file symlink
pointing outside, a write through a directory link, and a write through a file link → DL0904 each, plain and under
`--sandbox`; the outside file kept its bytes. Memory bomb (string doubling, 64 MiB budget) → stopped at 78 MiB on the
interpreter and 103 MiB on WASM: sampling overshoot, documented. `--report-out` inside a writable grant → refused, exit 2.
Lease-token MAC: a constant-time `blake3::Hash` comparison (`lease.rs` (lines 189–192)). Production `brokerd.rs` has essentially no
`unwrap` outside its test modules. **I found no containment or authority escape.**

### 6.5 The agent tool loop (OBSERVED HERE)

`check --json` (DL0501, exact repair, `authority_widening:true`) → `fix` (refuses to widen unless named; F3) →
`fix --accept-widening add_effect_to_row` (applied, rechecked) → `edit --expect-hash` (stale hash refused; correct hash
applied with an **authority before/after delta**; the old hash replayed is refused) → `authority --within` (`within`, or
`exceeds` naming `fs.write=./output` and its source line) → `run` → `--report-out` report. **Transport, analysis,
authorisation, program and task outcomes are separable**, but only by an agent that knows F3, F8 and C16–C17. That
knowledge should be in the contract, not tribal.

### 6.6 Two principals (OBSERVED HERE, same OS user; daemon custody)

Parent lease (`Read,Write`, `fs.read=input/`) → child (`Read,Write`, `fs.read=input/child`) → the child reads `child-ok`.
The child's program asking for `./input` faults with DL0703 *"the lease … does not carry fs.read=./input"*. The child
delegating `input/` gets **DL0802** (attenuation violation). Revoking the parent revoked 3 nodes; the next child run gets
**DL1403** *"revoked by audit seq 10"*; `audit verify` reports a good chain of 11 records. **Not shown:** separate OS accounts,
replay and idempotency of non-idempotent effects, and broker crash recovery.

### 6.7 Second pass: boundaries and scenarios neither review had run (OBSERVED HERE)

| Probe | What happened | Outcome class |
|---|---|---|
| **Broker killed mid-run** (SIGKILL of the daemon, at about 2.0 s into a leased run reading a file in a loop; timestamped) | ticks 0–29 before the kill; **the very next effect at 2.050 s → DL1401** *"broker unreachable … (fail closed, invariant 27)"*, exit 1 | expected denial: **fail-closed** |
| **Sandbox guest killed mid-run** (SIGKILL of `__guest` after 17 ticks) | host: *"the guest closed the channel without saying goodbye"*, exit 1; report lacks a reason (F14) | true failure, reported |
| **L3 launcher, honest** (`external:` script that runs the guest) | runs; report: level 3, backend `external`, `granted: none`, all five properties `unknown`, `launcher_blake3` recorded | passed, labelled honestly (F16) |
| **L3 launcher that lies** (prints text instead of a guest) | *"the guest never confirmed its boundary, so it was not sent the program"*, exit 1 | expected denial |
| **L3 launcher that stays silent** (`sleep 100`) | refused after about **60 s**, *"it said nothing within the channel's deadline"*; the launcher process was cleaned up | expected denial (slow) |
| **L3 with a wrong `--launcher-digest`** | refused before anything started, both digests named | expected denial |
| **L3 under `hostile-agent`** | refused: five properties not established or vouched for, exit 2 | expected denial |
| **`--require-attestation`**: no attester / valid software attester / wrong pinned key / at L1 | refused after the 10 s window / **runs, with the claims recorded as the attester's** (`verified: true`, `host_guarantees: []`) / refused / usage error, "Nothing ran" | as designed |
| **Guard seal** on `fs_write:<abs>/output` (daemon) | `grants delegate` of that scope → **DL1413** *"sealed authority is not runtime-approvable"*; but `run --broker daemon --grant fs.write=./output` **wrote** (a fresh root passes the Guard: DISC-1, documented, `QUESTIONS.md` lines 249–262) | denial, plus the documented same-uid bypass, **observed** |
| **Strict anchored roots** (`broker start --require-anchored-roots <pub>`) | `run --grant` and the delegate auto-root → **DL1421** *"root issuance refused"*; `doctor` reports `root issuance STRICT` | the mitigation works; the anchor key's custody is the residual |
| **Proposal → approval (P9-03)** | a run refused for `fs.write=./output` (`--report-out`) → `grants propose` gives exactly `fs.write=./output` from the source line → filed as `pp_0000` → the child approving under itself → DL0802 → approval under the parent mints a lease → the re-run prints `{"ok":true,"value":42}` | passed useful task (F18 caveat) |
| **HTTPS egress, local server with its own CA** (`SSL_CERT_FILE`); the server logged every request | granted `GET /v1/ok` → `GOT pong` (interpreter, WASM, `--sandbox`); `/v2/secret` outside the path scope → DL0904; `POST` without the method → DL0904; `:GET,POST/v1/**` → `POSTED`; `/v1/../v2/secret` → DL0904; a 302 to `https://localhost/…` → `Err(Refused)`, *not-allowlisted*; plain `http://` → `Err(Refused)`, *scheme*. **The server saw exactly** `GET /v1/ok` ×3, `POST /v1/ok`, `GET /redir`: no refused request reached it | expected denials, with **independent evidence of no forbidden network effect** |
| **Endpoint-bound secret** | `str(secret)` → DL0604 at check; sent with an audience → the server received it once, only in the `authorization` header; no audience → not sent (*audience*); another origin → not sent | as designed |
| **Python** (`--grant foreign.python=json`) | `json.dumps` works; `import os` → `ImportNotAllowed` (DL1305); `--sandbox` → refused, "Nothing ran" | expected denial / unsupported refusal |
| **Signed Verified driver + device process** (`examples/line_driver` signed with a test key; `delulu device sim` as transport behind my frame-logging tee) | commands 42.5 and 80.0 → `COMMANDED`; 120 → refused by the host envelope; **the device process saw only `CMD … 42.5` and `CMD … 80.0`**; a one-bit-tampered driver → DL1504, **0 frames** reached the device; the hardware profile required the simulator sign-off of this exact program hash first | **independent no-command proof** |
| **Drone mission simulation** (my `drone.delulu`, Appendix A) | identical on both engines: duplicate command dropped, geofence refusal, priority arbitration picks the safety command, comms loss → RTL, commands held in RTL, low battery → LAND | passed (logic only) |
| **Satellite telemetry simulation** (my `telemetry.delulu`) | interpreter: queue `[1, 4, 5, 6]`, 1 dropped at the bound, 1 duplicate, 1 stale beyond a clock-skew window; WASM: DL1201 (the third parity gap, §6.2) | passed on one engine |
| **Multi-package program** (`tests/corpus/tier4-multimodule` copied out) | `build` and `lock` ok; `authority` lists every module's reach; `run` asks for the grants; a dependency's manifest edited under `--locked` → **DL1010** *"possible tampering"* | as designed |
| **Budgets** | `--limits cpu=1` stops a busy loop at 1.0 s of CPU; `--sandbox --mode audit` → `ran: false` plus the exact `required_grants` | as designed |
| **Determinism** | `--seed 42 --clock fixed:1000` twice → identical `rand` and `clock` values; without them they vary | as designed |
| **Actors** (`examples/guide/06_actors.delulu`) | interpreter and WASM exit 0 (WASM: *"cooperative single-threaded"*); `--sandbox` → refused | unsupported refusal under sandbox |

---

## 7. CLI-first versus CLI-only, and the compiler and editor question

| Option | Verdict |
|---|---|
| Remove compiler internals; keep a terminal wrapper | **Reject**: the wrapper would have nothing to wrap. |
| Separate "compiler" product | **There is none.** Do not create one. |
| **CLI-only** (agents compose shell strings, parse terminal text) | **Reject.** Agents already need `--json`, MCP and LSP. Shell-only invites unsafe string building and makes outcome semantics (C16, F3) worse. |
| **CLI-first + one machine API + thin editor** | **Adopt.** Concretely: (a) split `cli.rs` into services (`check`, `authority`, `edit`, `run`, `test`, `grants`) with serde request and result types; (b) MCP and LSP call the services in-process instead of forking `delulu` (`mcp.rs::run_self`); (c) the CLI becomes a parser and printer over the same calls. |
| Persistent analysis service? | **Only where measured.** A small check costs about 7 ms per fresh process (p95 9 ms), so a daemon buys nothing for utilities. A 16k-line program costs 580 ms and 224 MiB per check, and *there* the LSP process already is the persistent service. **Do not add a new daemon.** **Second-pass soak (OBSERVED HERE):** one `delulu lsp` process took 1,000 alternating good/bad edits: 0 wrong diagnostics, **p50 0.16 ms**, p95 0.29 ms, memory flat at 15.6→15.7 MiB. On a 4,002-line file, 200 edits ran at p50 55 ms, plateauing at 76.7 MiB after edit 60. One `delulu mcp` process took 200 `check` calls: 0 wrong, **p50 5.6 ms** (each call forks the CLI), memory flat at 13.9 MiB. |
| VS Code | Keep the 409-line client; freeze features; test that the installed server identity matches (F5). |

Decision criteria to revisit: if an external agent client completes the §15 loop *without* the CLI binary on PATH, the
service layer is right; if it still needs `--json` scraping, it is not finished.

---

## 8. NVIDIA OpenShell: independent study and integration decision

### 8.1 What upstream actually is, at the inspected version (OFFICIAL EXTERNAL, fetched 2026-11-04)

| Fact | Value | Source |
|---|---|---|
| Version and cadence | **v0.1.7, 2026-11-03**; v0.1.1 (Sep 26), v0.1.2 (Sep 28), v0.1.3 (Oct 9), v0.1.4 (Oct 13), v0.1.5 (Oct 20), v0.1.6 (Oct 27): **about one release a week** | [releases](https://github.com/NVIDIA/OpenShell/releases) |
| Breaking changes | 0.1.3 (a `fix(cli)!`), **0.1.5** (OCSF log files renamed to `.jsonl`, Kubernetes labels need allowlisting, Go SDK `Create()` changed), **0.1.6** (gateway refuses to start without authentication), **0.1.7** (native OCSF only, settings deprecated, microVM runs as the image's `sandbox` account) | same |
| Licence, scale | Apache-2.0; about 21.1k stars; 1,957 commits; "AS IS" | [repo](https://github.com/NVIDIA/OpenShell) |
| Platforms | Linux Debian/Ubuntu x86_64 and aarch64 **supported**; macOS **Apple Silicon via Docker Desktop**; **Windows only as WSL 2 (Experimental)**; native Windows (MXC) on a **`windows` branch** | [support matrix](https://docs.nvidia.com/openshell/latest/about/support-matrix) |
| Requirements | Docker ≥ 28 or Podman 5 or Kubernetes ≥ 1.29; **Landlock ABI ≥ 3 (Linux ≥ 6.2)**, seccomp user notification, `process_vm_readv`; microVM via libkrun (KVM or Hypervisor.framework); **no GPU requirement stated** | same |
| Components | **gateway** (control plane, authentication, policy delivery, prover), **compute driver**, **supervisor** (trusted: policy, DNS, opens connections, **injects credentials**), **sandbox** (in the boundary; identifies the calling binary from `/proc`; "makes no policy decisions"), providers, SDKs (Python, TS, Go, Rust) | [architecture](https://docs.nvidia.com/openshell/latest/about/architecture) |
| Fail-closed | workload frozen if the supervisor link drops; a new supervisor cannot take over a running sandbox; tokens bound to a sandbox generation | same |
| Static vs live policy *(second pass)* | filesystem and process layers: "No. Requires sandbox re-creation"; **network and provider credentials: changeable on a running sandbox** (`openshell policy update`/`set`, or operator approval) | [security best practices](https://docs.nvidia.com/openshell/latest/security/best-practices) (v0.1.7) |
| Filesystem fallback *(second pass)* | a mandatory baseline needs Landlock ABI ≥ 3, and "Startup fails if this baseline cannot be enforced". The *additional* filesystem policy defaults to `compatibility: best_effort`, which "may skip unavailable policy paths" (with a High-severity OCSF finding); `hard_requirement` refuses startup instead. **DeluluLang's export pins `hard_requirement`** (`crates/delulu/src/openshell.rs:407`, OBSERVED in the export) | same; CURRENT SOURCE |
| Credentials *(second pass)* | the agent gets an opaque placeholder; "Provider-credentialed endpoints reject uninspected modes unless an operator explicitly opts in" (`allow_uninspected_credentials`); `tls: skip` disables credential rewriting and L7 inspection for that endpoint | same |
| Docs version | the "latest" documentation states "Latest (v0.1.7)" (a search index still showed v0.1.6 as default, which is stale) | [docs](https://docs.nvidia.com/openshell/latest/) |

### 8.2 Responsibilities compared

| DeluluLang responsibility | OpenShell counterpart | Classification |
|---|---|---|
| Type, effect and rcap checker; static authority (`authority`, `--within`); compile-time constraints | none (it governs binaries it cannot read) | **not applicable / unsafe to replace** |
| Capability values, attenuation (`narrow`), language-level delegation | none | **not applicable** |
| Grant tree, ⊑ lattice, leases, multi-principal delegation, revocation cascade | none at language level (per-sandbox policy only) | **unsafe to replace** |
| Guard (sealed and guarded classes, owner code, permits) | approval of policy proposals (prover plus human or auto) | **complementary** |
| Hash-chained audit, OCSF export (PS-E-06) | OCSF 1.8.0 records, logs uploaded to gateway | **overlapping**; keep DeluluLang's chain as the record of authority, emit OCSF for SIEMs |
| Secret custody, endpoint-bound secrets (P9-04) | providers: credentials injected by the supervisor at approved endpoints | **overlapping in purpose, different binding** (a DeluluLang secret binds to the *program's* grant audience) |
| Host-side egress client (hosts × methods × paths, pinned DNS) | supervisor-mediated egress, REST/GraphQL/MCP inspection, binary identity | **overlapping**; DeluluLang's is narrower and tied to the program's authority |
| L1 process sandbox (Landlock and seccomp, AppContainer and Job, Seatbelt), **native on all three OSes, no Docker** | Linux kernel enforcement; macOS and Windows only through a Linux VM | **partially replaceable on Linux only** |
| Boundary confirmed before program sent; host-loss ends guest | typestate confirm-before-start; generation-bound tokens; freeze on supervisor loss | **overlapping**; DeluluLang already copied the *design* (PS-E) |
| L2 Firecracker microVM (Linux and KVM; image not distributable, D-NE-27) | VM compute driver (libkrun) | **partially replaceable**: the strongest candidate |
| L3 external launcher and attestation seam | could *be* the L3 launcher (it already is, by recipe) | **complementary** |
| Device broker, envelopes, dead-man, e-stop (P8) | none; a network or filesystem policy cannot express speed, position or geofence | **not applicable** |
| Fleet, gateway, OIDC, Kubernetes | gateway, OIDC, Kubernetes | **OpenShell's domain**; DeluluLang should not build these |

The repository's own study put it best: *"OpenShell governs programs it cannot read. DeluluLang governs programs it
compiles"* (V2_OPENSHELL_STUDY.md §2). A DeluluLang guest performs **no** effects itself; it can simply be denied a
socket, where OpenShell must intercept one. That asymmetry is why OpenShell cannot replace the Guard and broker: it would
see only `delulu` opening sockets, never *which* capability, lease or grant the program used.

### 8.3 The options

**A. Keep everything; OpenShell only as an optional launcher (today).** Unchanged trust boundary. Maintenance: about 13k
lines of containment plus the microVM image pipeline. Platforms: native L1 on Windows, macOS and Linux (OBSERVED: L1 costs
about 45 ms per run on Linux, §10.1). Risk: low; cost: high and ongoing.

**B. Hybrid: OpenShell as an optional or preferred containment backend behind a versioned adapter; DeluluLang keeps
checker, Authority, Guard, broker, channel and a minimal standalone L1.**

```text
 delulu (host: custody, Guard, broker, egress client) ──channel(ssh via gateway | vsock)──► OpenShell sandbox
     ▲ policy export (`sandbox policy --format openshell`, never wider than authority)          └─ delulu __guest (no effects)
     └─ L1 native jail stays the default for laptops / offline / Windows / macOS
```

Trust boundary: DeluluLang's host decides; OpenShell's outer fence is a *second* wall, reported as the launcher's (level 3,
`unknown` unless attested). Affected code: `openshell.rs`, `launcher.rs`, `attest.rs`, `scripts/openshell-*.sh`,
`openshell.yml`. Upstream constraints: weekly releases, CLI transport quirks (`sandbox exec` waits for stdin to end, so
the launcher is `ssh` through the gateway: recorded in D-V2-85), Docker ≥ 28, Linux ≥ 6.2.

**C. OpenShell-first (Linux servers):** OpenShell owns confinement, network fence, credentials and lifecycle;
DeluluLang keeps only its semantic and custody enforcement. Would remove or freeze `microvm.rs` (1,476 lines plus the
image pipeline), much of `jail.rs`'s Linux path, and `identity.rs` there. Must still independently check: that the guest
is confirmed before the program is sent, that egress for *granted* hosts goes through DeluluLang's client (OpenShell's
`protocol: rest` endpoints terminate TLS with their own CA, an open question recorded in the study), that secrets never
enter the guest, and that a missing OpenShell **refuses** rather than falling back.

**D. Replace DeluluLang's sandbox infrastructure with OpenShell.** Loses native Windows and macOS L1, offline and
single-binary operation, and the Docker-free laptop path. Adds a gateway, a supervisor and a container runtime. Ties the
security posture to a 0.1.x API that broke three times in five releases. **Unsafe now.**

**E. (mine) "Policy-author" integration only:** DeluluLang emits OpenShell policy for *agents written in other
languages*, from a DeluluLang-checked manifest. Useful later; not a containment decision.

| Criterion | A (keep all) | **B (optional backend; today's shape)** | C (OpenShell-first on Linux servers) | D (replace) | E (policy author only) |
|---|---|---|---|---|---|
| Trust boundary | DeluluLang host + its L0–L2 walls | **host decides; OpenShell is a second wall at L3, reported as the launcher's** | host decides; OpenShell is the only wall on Linux servers | OpenShell's wall everywhere; DeluluLang host inside it | unchanged (export only) |
| DeluluLang code affected | none | **`openshell.rs` (692), `launcher.rs` (184), `attest.rs` (819), `scripts/openshell-*.sh`, `openshell.yml`** | + freeze/remove `microvm.rs` (1,476) and the image pipeline; parts of `jail.rs` (1,737) and `identity.rs` (1,223) on Linux | + most of the 13,255 containment lines | `openshell.rs` |
| Upstream surface used | none | `openshell sandbox create`, `ssh-config`/ssh proxy, policy YAML v1, `openshell-prover` (pinned tag) | gateway API or SDK, compute drivers, providers | all of it | policy YAML, prover |
| OS matrix | L1 native on Windows (AppContainer), macOS (Seatbelt), Linux (Landlock+seccomp); L2 Linux+KVM | **same, plus L3 where OpenShell runs (Linux ≥ 6.2 + Docker ≥ 28; macOS via Docker Desktop; Windows WSL 2 experimental)** | Linux servers via OpenShell; laptops keep L1 | Linux, plus macOS via Docker Desktop and Windows via WSL only | any |
| Dependency footprint | none added | **none linked**: OpenShell is a separate install, and no crate is a dependency (D-V2-52) | runtime dependency on a gateway, a supervisor and a container runtime | same, mandatory | none |
| Release / upgrade risk | none | **contained**: a broken upstream release breaks L3 only; pin plus a CI witness | high: weekly 0.1.x with breaking changes in 3 of the last 5 releases | very high | low |
| Maintainer burden | high (all walls are ours) | **medium**: adapter, pin bumps, witnesses | medium | low code, high integration | low |
| Attack surface added | none | **a launcher script and ssh through the gateway**; pinned by `--launcher-digest` / attestation | a gateway with auth (≥ 0.1.6 refuses to start unauthenticated), supervisor, container runtime | same, everywhere | none |
| Policy semantics | DeluluLang's ⊑ only | ⊑ decides; OpenShell policy is *derived* and never wider (export soundness rule) | two policies that must agree; OpenShell's network policy can be widened live, ⊑'s cannot | OpenShell's policy decides the wall; ⊑ still decides effects | derived |
| Security evidence | full CI + this session | **prover `within`/`exceeds` green on v0.1.2 (`41394552103`); runtime green once (`37234907676`, 2026-10-04)** | none | none | prover green |
| Performance / cold start | L1 ≈ +45 ms per run (§10.1) | **L3 via ssh: not measured** (§8.5 item 8) | not measured | not measured | n/a |
| Deployment complexity | one binary | **one binary; OpenShell only where the operator chooses** | gateway + Docker per server | gateway + Docker per machine | one binary |
| Operator experience | `delulu run --sandbox` | **same; or `--sandbox-backend external:<openshell launcher>`** | must run OpenShell before DeluluLang can confine | always run OpenShell | `delulu sandbox policy --format openshell` |
| Failure cases | ours | **launcher silent, lying or swapped → refused before the program is sent (§6.7)**; upstream break → L3 refuses | gateway down → no confined runs | gateway down → nothing runs confined | — |
| Rollback | — | **drop the flag; L1 untouched** | re-enable the frozen L2 | rebuild walls (hard) | — |
| Incremental migration | — | **today's state** → pin bump → §8.5 suite → only then C | B first, then per-server opt-in | not advisable | any time |
| An agent on a Windows or macOS laptop | full L1 | **full L1** | full L1 | **degraded**: needs WSL or Docker Desktop | n/a |
| Fleet and Kubernetes | none | via OpenShell | via OpenShell | via OpenShell | — |

**How a user runs it, with and without OpenShell (option B):** without it, `delulu run app.delulu --sandbox --grant …`
(L1 native, as tested here). With it, the operator installs OpenShell, writes the policy with
`delulu sandbox policy app.delulu --format openshell --grant … > p.yaml`, creates the sandbox from an image that carries
the same `delulu` version, and runs
`delulu run app.delulu --sandbox --sandbox-backend external:/usr/local/bin/delulu-openshell --grant …`. The recipe is in
`docs/DEPLOYMENT.md` (PS-E-05 (b)), and the L3 report says `unknown` unless an attester vouches.

### 8.4 Invariants that must survive any integration

1. The guest performs no effect; the host performs each one only after custody, the Guard and the audit append succeed.
2. The program is sent only after the boundary is confirmed for **this run's generation**.
3. Unsupported or unavailable containment is a **refusal before execution**, never a silent downgrade.
4. Static `inconclusive` never becomes a grant; a proposal is never a grant until approved under ⊑.
5. Revocation and expiry are checked at use, never served from a cache.
6. An exported policy is never wider than the program's authority and grants (`openshell.rs` soundness rule).
7. A launcher's guarantees are reported as the launcher's, never as the host's.
8. Secret bytes never enter the guest; they go only to the granted audience.
9. Device envelopes and e-stop stay in the DeluluLang device broker, out of reach of any network or filesystem policy.

### 8.5 Proof of concept to run later (designed here, not run)

On `ubuntu-24.04` (kernel ≥ 6.2, Docker ≥ 28), pinned **v0.1.7**, extending `openshell.yml`:
(1) granted and ungranted run pair: identical outcomes to L1; (2) `curl` inside the sandbox to an unlisted host is
denied by OpenShell (log line read); (3) a granted HTTPS `GET` through DeluluLang's egress client works or fails with a
named reason (the CA question); (4) kill the gateway mid-run → guest ends, run report says why, nothing performed after;
(5) revoke the lease mid-run → next effect refused (DL1403); (6) start with OpenShell absent → exit 2, nothing ran;
(7) prover `within` / `exceeds` on the export (exists); (8) overhead: 31 fresh runs of the §10.1 utility, median and p95
against L1 (about 54 ms here); (9) upgrade drill: the same suite on the previous tag, so a break is attributed to
upstream. **Pass** = all nine green on two consecutive releases. **Rollback** = remove the backend flag; L1 is untouched.

### 8.6 Verdict

**OPTIONAL BACKEND.**

**First thing to remove or defer if equivalence is shown:** further investment in **PS-C (Firecracker microVM)**. Freeze
it at its tested state. No aarch64 work and no jailer-as-user work; put VM-grade isolation behind L3 (OpenShell's VM
driver among the options). Its image cannot be distributed (D-NE-27) anyway.

**What would make me move to PREFERRED HYBRID (Linux servers):** an OpenShell API stability policy (1.0, or documented
stable interfaces for exec, ssh and policy); the §8.5 suite green on two consecutive releases; median overhead of 100 ms
or less over L1; the TLS-interception question answered.

**What would make me reconsider REPLACE:** native, non-experimental Windows and macOS support without Docker Desktop, *and*
a library or embedded mode with no gateway daemon. Neither exists today.

---

## 9. Authority, Guard, broker, Survey, Doctor, Atlas, plugins: improvement plan

| Component | Purpose (keep distinct) | Gap found | Change (exact target) | Acceptance |
|---|---|---|---|---|
| Authority | what code *can* ask for | `inconclusive` on a pure constant function (C20) | evaluate pure, total, zero-argument functions with literal bodies at check time (`delulu-check/src/authority.rs`) | `scope()` example → `within`; computed-from-input stays `inconclusive` |
| Guard and broker | what is allowed *now* | F4 naming; F6 single-use burn | rename to `expires_at_millis` (keep the old key one release); burn the nonce only after the first effect, or document it in `for-agents.md` | contract test |
| Devices | envelopes, dead-man | **F1**, **F2**, **F7** | null profile → refuse `command` (or `Err(NoDevice)`) and name `device_profile` in the report; fix the DL0703 hint to include `heartbeat_ms`; poison-tolerant locks that **engage the fail-state** (`delulu-runtime/src/device.rs`) | an agent following the hint succeeds; a panicking actor engages safe-park |
| Survey | this repository's map | F9 granularity | resolve module edges inside a crate from `use` and `crate::` paths, not only `mod` lines (`delulu-survey/src/scan.rs`) | `impact mod:…/openshell.rs` < `impact mod:…/cli.rs` |
| Doctor | install and posture health | F5; stale probe strings (C30) | inject `git rev-parse HEAD` in `build.rs` when building from a checkout (mark `-dirty`); compare the `delulu` on PATH with the tree | stale binary detected |
| Atlas | a program's map | fine; overlaps checker data | keep; feed it from the same service layer | — |
| MCP | read-only inspection | forks per call; unbounded output | in-process service calls; output cap; keep read-only | 10 MB output refused cleanly |
| Plugins | verified extension | WASM and sandbox refuse them | keep refusals; list them in the matrix | matrix test |
| Test runner | outcomes | C13 | missing path → exit 2; zero matches → distinct status (`"matched":0`) | contract test |
| `fix` and `edit` | repair | F3, F8 | exit 1 when `errors_after > 0`; `summary` describes the *after* state; stale hash → its own exit code or field `conflict: true` | contract test |
| `run --json` | outcome | C16 | always an envelope on stdout, with the program's stdout captured in a field (or an explicit `--program-stdout`) | parse stdout as one JSON value in every outcome |

---

## 10. Performance and portability truth table

### 10.1 Fresh-process benchmarks (OBSERVED HERE)

Method: Linux x86_64 4 vCPU VM; release default profile; 3 warm-ups, then **31 samples per case**, round-robin across
cases; a sample counts only if exit is 0 **and** stdout equals the reference (0 failures in 527 samples). Time includes
process start; RSS is `ru_maxrss` from `wait4`. Python 3.13.18 and Rust 1.96.1 `-O` controls are **not
security-equivalent**: no confinement, no authority. Raw samples are in Appendix C.

| Case | median ms | p90 | p95 | RSS MiB |
|---|---:|---:|---:|---:|
| `--version --json` | 6.51 | 8.27 | 8.89 | 10.7 |
| `check utility --json` | 7.43 | 8.92 | 9.04 | 13.2 |
| `authority utility --json` | 7.65 | 9.15 | 10.52 | 13.8 |
| utility, interpreter | 8.26 | 10.08 | 10.13 | 14.8 |
| utility, WASM | 14.01 | 16.83 | 17.42 | 24.9 |
| **utility, `--sandbox` L1** | **53.73** | 56.77 | 57.02 | 16.2 |
| utility, Python control | 29.23 | 37.05 | 43.56 | 12.6 |
| aggregate 100k, interpreter | 46.86 | 78.99 | 79.56 | 14.4 |
| aggregate 100k, WASM / `.dwx` | 12.58 / 11.99 | 15.14 / 14.24 | 15.24 / 14.74 | 23.9 / 22.1 |
| aggregate 100k, Python / Rust | 28.55 / 4.00 | 36.09 / 4.69 | 37.86 / 4.78 | 10.4 / 9.6 |
| **aggregate 5M, interpreter** | **1,968** | 2,088 | 2,117 | 14.4 |
| aggregate 5M, WASM / `.dwx` | 34.0 / 32.6 | 47.5 / 45.9 | 50.5 / 47.2 | 23.9 / 22.1 |
| aggregate 5M, Python / Rust | 623.6 / 10.0 | 723.0 / 14.9 | 748.1 / 16.9 | 10.4 / 9.6 |
| **agent loop** *(second pass)*: `check --json` → `authority --within` (3 grants) → `run --report-out` → parse the report's `outcome.exit` (one script, 31 samples after 3 warm-ups, all valid) | 34.31 | 37.80 | — | 17.5 |

The agent loop includes about 15 ms of Python start-up for the parse step, so the DeluluLang part of a full
analyse-authorise-run-report cycle is about 20 ms.

What this says: startup and analysis are **fast enough for agent loops** (≤ 10 ms p95). The tree-walking interpreter is
**about 3.2× slower than CPython** on a hot loop (RW 3.24); WASM is **about 18× faster than CPython** and about 3.4×
slower than optimised Rust, startup included. The L1 sandbox costs about 45 ms per run. **Make WASM the default
engine for programs it can compile, and fix parity (§6.2) so it can compile them**, rather than optimising the
interpreter further.

### 10.2 Analysis scaling (OBSERVED HERE; generated programs of *n* chained functions)

| Lines | `check` | `authority` | run (interpreter) | run (WASM) |
|---:|---|---|---|---|
| 1,002 | 30 ms, 26 MiB | 54 ms | 38 ms | 112 ms, 42 MiB |
| 4,002 | 136 ms, 65 MiB | 195 ms | 155 ms | 390 ms, 99 MiB |
| 16,002 | 580 ms, **224 MiB** | 636 ms | 690 ms | 1,716 ms, **315 MiB** |

Time is linear; **memory (~14 KiB per line) is the constraint** for edge devices and long LSP sessions (F10).

### 10.3 Portability and support tiers

| Tier | Status | Evidence |
|---|---|---|
| Linux x86_64 host tools; L0; L1 (Landlock ABI 7, seccomp) | **verified here** | §2.2, §6 |
| Linux L2 microVM | **verified by named CI** (`microvm` job, `41929573373`) | not runnable here (no KVM) |
| Windows (AppContainer), macOS (Seatbelt), Linux arm64 | **verified by named CI** (test jobs, `41929573373`) | not run here |
| Same source, same results, interpreter ↔ WASM | **partial**: of 17 valid everyday programs, 15 behave identically (12 outputs, 3 identical faults) and 2 are refused at compile time on WASM (`range`, a non-generic higher-order function); devices, plugins and actors with `Str` fields are refused too | §6.2 |
| Same *artifact* (`.dwx`) | **Delulu-host-specific**: a core-WASM module importing `delulu:cap` (`root_console`, `console_println`) with a `delulu:authority` section. **Not WASI**; it needs a `delulu` runtime | OBSERVED (`strings`) |
| Single-binary distribution (Linux x86_64) | **verified here**: about 33.4 MB stripped in either profile; the default profile needs `libpython3.13.so` at run time, the portable one needs only libc | §3 C06 |
| OpenShell L3 | **source and old CI only** (v0.1.2) | §2.3 |
| Simulated robot, vehicle, drone, satellite | **simulated** (logic and simulator device) | §5 |
| Constrained embedded, RTOS, hard real-time; qualified deployments | **unsupported / future research** | — |

Same source, same semantics, same artifact, same security posture and same performance are five different promises. Today
DeluluLang can make the first on its tested OSes, the second only for the WASM-compilable subset, the third only between
DeluluLang hosts, and the fourth and fifth only per named OS and tier.

---

## 11. Competitors (OFFICIAL EXTERNAL unless marked; nothing installed or run)

### 11.1 Capability matrix

| | DeluluLang | Zerolang | Mojo | Python + strong sandbox (OpenShell, gVisor, microVM) | Deno | Koka / Pony / Roc | Wasmtime + WASI | MoonBit |
|---|---|---|---|---|---|---|---|---|
| Target users | AI agents, then humans | agents | AI/HPC developers | everyone | web and back-end | researchers / actors / app-platform | embedders | Wasm and AI-assisted developers |
| Version and maturity | 1.0.0 released, 2.0 prepared | **experimental**, no releases | **1.2.0 stable** ("Oct 22"; nightly Nov 4) | mature | mature | Koka research (v3.2.x); ponyc 0.77.2 (28 Oct 2026); Roc: no numbered release shown | **v50.0.1** (29 Oct 2026) | pre-1.0 (v0.10.x per a secondary source) |
| Typed effects | **verified** | effects listed as graph facts | absent | absent | absent | Koka **verified**; Pony reference caps (aliasing, not IO) | absent | unknown |
| Computed least authority from source | **verified** | partial (capabilities via a `World` value) | absent | absent (policy written by hand) | absent (flags) | Roc: platform-defined IO | absent | partial (policy files for skills) |
| Runtime enforcement | **verified** (custody, Guard, broker) | "explicit capabilities" goal | absent | **verified** (kernel and VM) | **verified** permissions; `run`/`ffi` ≈ allow-all | — | capability-based imports | Wasm sandbox ("not OS-level") |
| Machine editing | hash-checked edits, Atlas ids, authority delta | **graph patches with hash guard** | none | none | none | none | — | AI skills in toolchain |
| JSON, args and task completeness | **absent (P0)** | std modules for data formats (CODEX REPORTED) | Python interop | **verified** | **verified** | partial | host-dependent | partial |
| Target capability discovery | absent (late DL1201) | **`zero targets --json`, TAR002** | requirements page | n/a | n/a | — | WASI worlds | targets list |
| Native Windows | **verified (CI)** | unknown | **no** (WSL) | sandbox: WSL experimental | yes | varies | yes | unknown |

### 11.2 Lessons, with a decision for each

| Source | Lesson | Decision |
|---|---|---|
| **Zerolang** (Apache-2.0; *"experimental … Expect breaking changes, rough edges, and security issues"*) | Structured query and patch over semantic node ids with hash guards; a target-capability query with a named diagnostic (`TAR002`) | **Borrow the principle**, not the storage: DeluluLang already has `edit --expect-hash` and Atlas node ids. Add `delulu query` (Atlas ids, types, effects, callers) and `edit --node ID` by Atlas id, both behind the content hash. **Keep text as the canonical source**: humans review it, git diffs it, and nothing measured shows a graph store completes tasks better. **Borrow `targets --json` plus a preflight diagnostic exactly** (via an existing DL code, not a new one without a decision). |
| **Mojo** | Accelerator and numeric performance; Python interop; no native Windows | **Avoid** competing; **integrate later** through adapters if a user needs GPU compute. |
| **Python, Rust, Go, JS inside strong sandboxes** | The honest baseline: huge libraries, model familiarity, OS-level containment | DeluluLang's only edge over it is **source-level authority, checked before running, plus per-capability host enforcement**. That edge matters only once ordinary programs can be written (§13). Measure it with the task study in §15. |
| **Deno** | Scoped permissions; documented escape classes (`run`, `ffi`); a permission-broker mode | **Borrow the wording discipline**: DeluluLang already refuses Python, FFI and plugins under `--sandbox`; say "≈ allow-all" plainly in `for-agents.md`. |
| **Koka** | Effect handlers | **Defer**: effect *typing* is enough for authority; handlers are not a 2.0 need. |
| **Pony** | Reference capabilities (aliasing) ≠ object capabilities (authority) | **Keep the distinction in docs**; DeluluLang has both. |
| **Roc** | Platform/host separation | **Already the design** (guest ⇄ host); express target profiles as "platforms" in the matrix. |
| **Wasmtime and WASI** | A maintained runtime (DeluluLang pins 48.0.8, a line still patched on 2026-10-29). *Second pass:* **WASI 0.3 is stable** (wasi.dev: 0.3.0 on 2026-06-11, native async with `stream`/`future`, `wasi:io` folded into the component model, `wasi:cli/exit` gained `exit-with-code`); Wasmtime ≥ 46 runs 0.3 components by default | **Integrate, don't rebuild**: research a WASI-component `.dwx` profile for host portability (§13 phase 3), and note that WASI's own `exit-with-code` is the shape DeluluLang's missing exit outcome (§6.2) could map onto. Do not write a new VM. |
| **Node.js permission model** *(second pass)* | Stable (`--permission`, `--allow-fs-read/-write`, `--allow-net`, `--allow-child-process`, `--permission-audit`); the docs say it restricts `node:fs` and "does not guarantee that users will not be able to access the file system through other means" | Same lesson as Deno: runtime permission flags are not a sandbox. DeluluLang's typed authority plus host-performed effects is stronger, and should say so precisely, not loosely. |
| **Python inside a microVM sandbox service** (e.g. E2B, Firecracker-based; secondary sources only) *(second pass)* | The baseline the prompt asks about: the full Python ecosystem, per-run kernel isolation, a managed or self-hosted service | **The honest competitor.** It wins on libraries and model familiarity today; it cannot tell you *before running* what the code will touch, nor refuse a single effect by capability. That difference is DeluluLang's whole case, and §15.3's study measures it. |
| **MoonBit** (included as the additional system: an AI-oriented, Wasm-first toolchain that ships agent skills and policy-gated skill sandboxes) | Agent skills shipped with the toolchain; formal checks in the AI loop; multiple backends | **Borrow**: ship the existing `delulu skill` as the first-program kit, with the capability matrix in it. Versions come from secondary sources; recheck before citing. |

---

## 12. Keep / simplify / combine / externalise / defer / remove

| Action | Exact target | Survey (impact) | Risk | Benefit and migration |
|---|---|---|---|---|
| **Keep** | `delulu-syntax`, `delulu-check`, `delulu-diag`, `delulu-broker`, `delulu-runtime`, `delulu-wasm`, `delulu-conform`, tests, formal models | 258 / 242 / — / 200 / 180 / 132 | — | the product |
| **Simplify** | `crates/delulu/src/cli.rs` (11,866) → `services/{check,authority,edit,run,test,grants}.rs` with serde types; CLI as parser and printer | 106 (crate-level) | medium (exit and diagnostic ordering) | one contract for CLI, MCP and LSP; migrate one command at a time behind `core_invariance` snapshots |
| **Combine** | MCP's `run_self` subprocess → in-process service calls; keep read-only | 106 | low | removes duplicate argv building; output bounded |
| **Combine** | name lookup and callable-compilability: one table consumed by checker, interpreter and codegen | 242 + 132 | high | closes the class behind LATE-LET-1 and friends; also makes WASM parity measurable |
| **Externalise** | VM-grade isolation → L3 launchers (OpenShell VM driver, gVisor, Kata) | `microvm.rs` 106 | low (the feature stays) | stop growing PS-C |
| **Defer** | P8 hardware drivers, new device stacks, fleet/deploy growth, new agent-named commands, Koka-style handlers, native/AOT backend | — | — | wait for named users |
| **Deprecate (2.x)** | rarely used top-level commands into groups: e.g. `login`/`publish`/`add` under `pkg`; `keygen`/`sign`/`verify-sig` under `sign`; `morph`/`locale`/`completions` under `tools` | — | low with aliases | 41 → about 15 groups; keep aliases one release |
| **Remove** | nothing now | — | — | no component has evidence of zero value; the gains are proportion and contracts, not deletion |
| **Remove from claims** | "total … across all code" wording in README's first line. *(Third pass: the "production ready" status box this row also named is `HANDOFF.md`'s, not README's, and since 2026-11-02 it says it covers containment and deployment only.)* | — | — | trust |
| **Fix metadata before launch** *(second pass)* | `Cargo.toml` line 30 and `editors/vscode/package.json` name an unregistered GitHub namespace (F15); reserve `delulu-lang` (GitHub, Marketplace, Open VSX) or keep `launch/2.0.0`'s corrected URLs | — | low | prevents namespace squatting at launch |

**The prompt's six questions, answered for each substantial change** (the Survey numbers are crate-level, because module
level is unreliable, F9):

| Change | What disappears | What stays | What breaks | Survey says | How confidence is kept | Migrating without two competing systems |
|---|---|---|---|---|---|---|
| `cli.rs` → services | the 11,866-line dispatcher's private logic | every command, flag, exit code and diagnostic order | nothing visible, if done command by command | `crate:delulu` impact 104, affected-by 193 | `core_invariance` snapshot, `cli-sweep.sh`, `json_contract`, CI on 3 OSes per move | move one command at a time; the old path is deleted in the same commit as the new one goes live |
| MCP in-process | `run_self` forking and its unbounded buffers | the 12 read-only tools and their JSON | nothing visible (same envelopes) | `mod:…/mcp.rs` affected-by 16 | the MCP wire tests plus my 200-call soak (0 wrong) as a regression bar | switch tool by tool behind the same schemas |
| Shared lookup and compilability table | duplicated name resolution in checker, interpreter and codegen | each engine's own evaluation | parity bugs become refusals before execution | `crate:delulu-check` 242, `crate:delulu-wasm` 132 | the differential fuzzer, conformance (366 anchors) and the parity corpus; **keep the conformance witnesses independent of the table** | introduce the table as a read-only oracle first, then switch each consumer |
| PS-C microVM freeze | new microVM work (aarch64, jailer-as-user) | the built L2, its CI job and its tests | nothing: the freeze changes no code | `mod:…/microvm.rs` 106 | the CI `microvm` job stays on | external launchers (OpenShell VM driver, gVisor) carry new VM-grade needs at L3 |
| Command grouping (2.x) | 41 flat top-level names | every verb, under a group, with aliases | scripts using old names, after the alias release | crate-level only | `cli-sweep.sh` both spellings; `skill` regenerated from `--help` | aliases for one release, with a deprecation warning that names the new spelling |

---

## 13. Roadmap: fundamentals first, AI first

**Phase 0 (now, about 1 week): contracts.** C13, F1–F5, F8, C16, C30 fixed, each witnessed red first. Pin bump to
OpenShell v0.1.7 and re-run. *Exit:* contract tests green; `doctor` names PATH vs tree.

**Phase 1 (the 2.0 bar): the ordinary agent utility.** In this order:
1. `Root` mints `Cap[Args]` and a selected `Cap[Env]` (each a grant, in `authority` and `--within`).
2. `main` may return `Result[Unit, E]` (or `Cap[Exit]`) with a documented exit mapping.
3. `json.parse` and `json.write` in the pure stdlib (byte, depth and number limits; secrets refused).
4. `math` and `checked_*`.
5. String `find`/`index_of`/`ends_with`/padding and interpolation (secret-checked like `str`).
6. Methods on user types.
7. WASM: `range`, non-generic higher-order functions, and an `if` without `else` as a value.
8. The generated capability matrix and `check --target`.

*Exit criteria (objective):* the §15 suite A01–A20 passes on Linux, Windows and macOS CI; three reference tasks (JSON task
runner, two-module transform, bounded retry workflow) pass on interpreter **and** WASM; no CHECKLIST gate weakened.

**Phase 2 (2.x): agents and delegation.** Service layer (§7); `delulu query` by Atlas id; replay and idempotency semantics
for effects; two OS accounts in CI; the task-completion study (§15.3).

**Phase 3: edge and simulation.** Memory per line (F10); a minimal runtime profile measured on a named ARM board; a
WASI-component `.dwx` study; seeded simulator under `--sandbox`; OpenShell PREFERRED HYBRID decision per §8.6.

**Phase 4: physical specialisation.** Only with a named device, an independent interlock and a safety owner.

**Deletion and consolidation order:** MCP fork → service calls; `cli.rs` split; command grouping with aliases; PS-C freeze.
**Owner decisions needed:** (1) hold 2.0 for Phase 1, yes or no; (2) adopt the OpenShell verdict; (3) catchable
scope refusals (P1 row in §6.2) vs fault: a semantic change to the Constitution's fault rules needs his word or a recorded
decision; (4) the exit-code mapping.

**Process constraint found in the second pass (CURRENT SOURCE):** README says *"The language core has been change-frozen
since Stage 9"*. `CONTRIBUTING.md` §5 requires any core change to go through `rfcs/`, with a **public comment period of
at least 14 days** and an irreducibility analysis. Most of Phase 1 is core syntax or semantics:
- core syntax or semantics: interpolation, methods, `+=`, `main`'s outcome, catchable refusals;
- new capabilities (argv, environment), which touch Authority, a guarded area;
- **library-only**: JSON, math and string functions; adding primitives still needs conformance anchors and touches the
  entrenched `delulu-conform`.

So **(5)**: either open the RFCs now (their 14-day clocks are the critical path), or record a `D-V2-nn` that
unfreezes the core for a defined set under the delegation. The library-only items can start immediately.

---

## 14. Threat model and assurance limits

- **Typed effects guarantee** that a checked program's runtime trace stays inside its row, for the language as
  implemented. They are machine-checked **only** for the monomorphic, closed-row fragment (C26), and backed by the
  differential fuzzer and conformance gates (CURRENT CI) for the rest.
- **The Guard and the broker enforce** grants, leases, revocation and sealed classes at the moment of use. Default root
  issuance is LEGACY: **any process of the same OS user can mint root authority** (C28). Hostile agents must run as a
  separate OS account (DEPLOYMENT Tier 2), with anchored roots.
- **Containment** is per OS and per level, as reported (`host_guarantees`, `fully_enforced`). This VM had **no identity
  separation** (`uidmap` absent), as the report said.
- **OpenShell, if used,** enforces its own fence around `delulu`; it proves nothing about DeluluLang's authority.
- **Neither proves:** intent correctness, prompt-injection resistance, side-channel freedom, or information flow beyond
  the `Secret` rules (the `verify` one-bit residue, C27). Plugins carrying `ForeignCall` and Python are outside the guest
  channel (refused under `--sandbox`).
- **Physical:** a software Guard is **not** an interlock. The recorded e-stop under a held audit chain took 9.8 s
  (HISTORICAL DOC, RW 4.53). There is no real driver, no timing analysis and no certification.

---

## 15. Acceptance suite (proposed; reuse existing tests wherever they witness the property)

### 15.1 Codex's A01–A18, re-scored here

| ID | State now (OBSERVED HERE) |
|---|---|
| A01 build identity | **fails** (`commit:null`, F5) |
| A02 JSON task | **fails** (no JSON) |
| A03 outcome mapping | **fails** (handled errors exit 0) |
| A04 package and entry file | **fails** (C14) |
| A05 test path semantics | **fails** (C13) |
| A06 no grant, outside scope, narrower child | **passes** (both engines, L1, broker) |
| A07 within, exceeds, inconclusive | **passes** |
| A08 engine preflight | **fails** (late DL1201) |
| A09 hash-checked repair and delta | **passes** (F3 and F8 caveats) |
| A10 CLI, MCP and LSP contract | **partial** (C16, C17) |
| A11 limits | **passes** (wall, memory, and CPU in the second pass); broker loss and guest loss end the run (§6.7). External cancellation of a run was not tested |
| A12 plugins | **passes** (allowed, overbroad, tampered, sandbox refusal); signed Verified *driver*: tampered → DL1504, nothing reaches the device (§6.7) |
| A13 two principals | **passes as custody** (same OS user), including the P9-03 proposal → approval → re-run loop (§6.7, F18) |
| A14 durable workflow | **not built** (in-memory only); the drone simulation shows the language can express duplicate rejection, but there is no durable store or idempotency key |
| A15 offline package | **passes** (build used only `cargo fetch --locked` sources; a multi-package program builds and locks, and a changed dependency under `--locked` → DL1010) |
| A16 simulation | **passes with a profile; F1 without one**; independent no-command proof via the device process's frame log (§6.7) |
| A17 performance record | **done here** (§10), including the agent loop and the LSP/MCP soaks |
| A18 evidence and claims | **partial** (Survey fresh; probe strings stale; F15 metadata) |

### 15.2 Added

- **A19:** every construct `docs/for-agents.md` and the Book teach compiles on WASM, or `check --target wasm` refuses it.
- **A20:** no success value for an action that did not happen (null device, `test` with zero matches, `fix` with errors left).
- **A21:** a poisoned device-broker lock engages the fail-state (F7; defensive: no reachable trigger was found).
- **A22** *(second pass)*: every run that ends abnormally (guest death, broker loss, launcher refusal) carries a
  machine-readable reason in `--report-out` (F14), and no outcome uses exit 2 except a usage error (F13).
- **A23** *(second pass)*: every diagnostic hint that suggests a grant produces, when followed literally, a grant the CLI
  accepts for *this* program (F2, F12).

### 15.3 The study that would settle "why a new language?"

Twenty fixed tasks (JSON and file utilities, two-module packages, delegation, cancellation, plugin use), two independent
agent clients, DeluluLang vs Python **inside an equivalent sandbox** (OpenShell or gVisor with an equivalent policy).
Measure: success against an independent oracle, unsafe attempts blocked, over-broad permission requests, repair
iterations, tool calls and tokens. Run it **after** Phase 1. Before Phase 1, DeluluLang would lose on fundamentals,
and the study would measure that instead of the authority model.

---

## 16. Unresolved questions and honest unknowns

- Windows and macOS behaviour of every new finding (F1–F18): source-level, so expected identical, **not observed**.
- The laptop's installed binary and extension (C15): **blocked** here.
- OpenShell runtime behaviour at v0.1.3–v0.1.7, its TLS interception of granted HTTPS, and its overhead: **unknown**
  until §8.5 runs (OpenShell was not installed: its installer is `curl | sh`, and the prompt forbids running one).
- F7: **no reachable trigger found** in the second pass, so it is kept as defensive hardening only.
- Zerolang's real task-completion advantage: **unknown**; nothing independent was found.
- Koka's exact current version (newest seen v3.2.3; Codex's v3.2.8 not found); MoonBit's version (secondary source only);
  whether the `delulu-lang` publisher id is free on the VS Code Marketplace and Open VSX.
- ~~Memory behaviour of a long LSP session~~: **measured in the second pass**, and it plateaus (§7).
- Cold start from an empty page cache: **not measured** (dropping the VM's caches is a system-wide side effect; all
  timings here are warm).
- ARM edge hardware, constrained devices, real-time behaviour: **not measured**.
- **Still not run, after the second pass:**
  - a durable workflow with restart, replay and idempotency (not built; A14);
  - broker *crash recovery* (loss was tested: fail-closed);
  - replayed or duplicated *effect* requests (only replay of a single-use *token* was tested: DL1407);
  - FFI (`foreign.c`) and Python process isolation (`--foreign-isolation process`);
  - two separate OS accounts;
  - P8-04's out-of-band monitor (`delulu monitor watch`; covered in CI by `monitor_cli.rs`, not re-run here);
  - external cancellation of a running program.

  Each needs the setup named in §15 or §8.5.

**Rejected alternatives:** full rewrite (no evidence the code is the problem); a graph database as canonical source;
CLI-only; OpenShell replacement (option D); a new agent-platform layer or daemon.

### 16.1 The owner's hypotheses, judged

| Hypothesis | Verdict | Evidence |
|---|---|---|
| A small core plus typed host adapters beats a sprawling command and broker stack | **Supported, with a limit**: the core already is small. The sprawl is in the CLI surface (41 commands, `run` about 40 flags), not in crates. Splitting crates further would add cost | §4.1, §12 |
| One shared resolved representation reduces divergence | **Supported, narrowly**: share name resolution and compilability; keep conformance witnesses independent | §3.2, §6.2 (WASM HOF) |
| Versioned structured outcomes matter more than new syntax | **Supported**: C13, C16, F1–F5 and F8 are all outcome defects; agents cannot tell success from nothing | §6.3 |
| Capability discovery and preflight prevent late failures | **Supported**: every WASM refusal (`range`, HOF, devices, plugins) comes after the plan was chosen | §6.2, §10.3 |
| OpenShell hybridisation removes duplicate infrastructure | **Partly**: only the VM layer is a real duplicate. It brings an extra control plane, weekly breaking releases, and Docker or WSL on macOS and Windows | §8 |
| A cheap, fast analysis service beats spawning the CLI | **Rejected for small programs** (7 ms p95 9 ms per process); **supported for large ones** (580 ms, 224 MiB at 16k lines), where the LSP already is that service. The second-pass soak shows it already exists and is stable: 0.16 ms per edit, flat memory over 1,000 edits | §7, §10.2 |
| Interop over rebuilding AI frameworks | **Supported**: keep Python, FFI and plugins as adapters; do not build GPU or robotics stacks | §11.2 |
| Security and UX are linked | **Supported**: F2 and F12 (hints that are not this program's), F6 (a burnt lease), F18 (approval is only as strong as the OS account) and DL0904 aborting instead of returning `Err` all push an agent toward broader grants or a human workaround. Conversely, the P9-03 loop (§6.7), which proposes the *narrowest* grant from the refusal itself, is the best AI-facing security UX found here | §6.3, §6.7 |
| Task-completion studies beat feature lists | **Supported**: §15.3, after Phase 1 | — |
| "Any AI" needs stable, self-describing interfaces, not AGI predictions | **Supported** | §5 |
| A lean optional editor and LSP preserve inspection without becoming the bottleneck | **Supported**: 409 lines | §7 |
| Remove complex unused features | **No candidate proven unused**; freeze instead (PS-C, P8 hardware) | §12 |

---

## 17. Final recommendation for the owner

**Change:** the definition of "ready" (§13, Phase 1 bar), the outcome contracts (Phase 0), WASM parity for what the docs teach,
one generated capability matrix, and the CLI's internal shape (services shared by CLI, MCP and LSP).
**Keep:** the language, checker, effects and authority; Guard, broker, leases and audit; the native L1 sandbox on all
three OSes; WASM; LSP and the thin VS Code client; conformance, fuzzing, Miri and formal gates.
**Freeze:** PS-C microVM growth, P8 hardware work, P9 extensions, new top-level commands, editor features.
**Integrate:** OpenShell as an optional L3 backend (pin current, witness both directions); Wasmtime and WASI rather than
any runtime of your own.
**Do not:** rewrite; delete the compiler or editor; hand security to OpenShell; adopt a graph store; claim whole-system
proof, safe autonomy, flight or vehicle readiness, or universal portability.

**Why:** DeluluLang's distinct value is a program whose authority is computed from its source and enforced per
capability at run time. Every probe here confirmed that core. What blocks AI users is not missing infrastructure. It is that
an agent cannot yet write the ordinary program that would use it, and cannot always tell from the tool's answer whether
the task succeeded.

---

## 18. Follow-up, 2026-11-05: what the AI-first pipeline audit changed in this report

The audit ran on a provenance build of the same commit (`master` `0670e0b`, `DELULU_BUILD_COMMIT` injected, SHA-256
`751c6458…`). It added an in-process stage harness, fresh-process timings with real peak memory, and a Sonnet 5.5
security sub-agent whose findings I re-ran before using them. Its report is the companion
[`DeluluLang_AI_First_Compiler_Execution_Pipeline_Audit.md`](DeluluLang_AI_First_Compiler_Execution_Pipeline_Audit.md).
**The owner, during it:** *"I allow u to suggest any changes that is against current constitution.md, license or any
other .md file rules and regulations."* The suggestions that cross a rule are its §14; none was acted on.

| Old claim (where) | New observation | Evidence | Effect on the recommendation |
|---|---|---|---|
| A small check costs *"about 7 ms per fresh process (p95 9 ms)"*, so a daemon buys nothing (§7, §10.1) | 4.3 ms median on the provenance build. Of that, 3.35 ms is the process floor, of which **1.1–2.0 ms is loading `libpython`**; the front end is 85 µs | the audit's §10.1–§10.2 | Unchanged (no daemon). Added: move Python out of the main binary (RW 3.26) |
| *"Machine-first authoring is a candidate"*; the service layer and validated edits are the machine API (§1, §7, §16.1) | The existing machine form, DIR, is **28× larger than text** at 16,002 lines and **1.9× slower to verify** than the whole text front end; parsing is 17% of the front end | the audit's §10.1 | Stronger: keep text as the interchange, and put structure in the edits (`edit --node`), not in a second representation |
| The WASM engine is *"faster on loops, slower to start"* (C25, §10.1) | Cranelift's compile is the start-up cost: 1.6 ms (hello) to 652 ms (16,002 lines). A precompiled module loads in 35 µs to 5.7 ms. Winch is incompatible with the limits engine's epoch interruption on Wasmtime 48; Pulley runs 11× slower | the audit's §5, §10.1, §11 | New: `--engine auto` (RW 3.3) and an authenticated precompiled cache (the audit's §5.4), never author-supplied native code |
| F13: a closed pipe gives exit 2, *"depends on how much output is still unwritten"* (§6.3) | The cause is a **panic** (*"failed printing to stdout: Broken pipe"*). It fired for a successful program with large output (3 runs of 3), not for a small error output (exit 1 into a pipe and into a file, one run each) | the audit's §10.3 | RW 6.58 now names the cause |
| The hostile probes found *"no containment or authority escape"* (§6.4) | Still true for effects and authority. But a DIR whose node ids collide **passes `dir::verify` while the same program as text is DL1604**: a reference-capability hole on the Verified-plugin path | the audit's §12; RW 4.93 | Fix before DIR is widened; I had not attacked DIR in either pass |
| `why` and `authority` were read as review surfaces that cannot vouch for refused bytes (C65, ruling D59, cited at §6.3) | `why` on a `.dpx` reads unverified DIR facts (RW 6.67); a DIR name bypasses DL0107 and reaches the terminal raw (RW 4.94) | the audit's §12 | Fix (§15 priority 1 there) |
| The agent loop `check → fix → edit → authority → run` *"works"* (§6.5) | In a **package**, `edit` checks the file alone: `widened: null` and `--if-checks` passed an edit the package refused (DL1009) | the audit's §12; RW 6.69 | The loop works for single files only; the docs now say so (`for-agents.md`) |
| A11: *"External cancellation of a run was not tested"* (§15.1) | Tested: SIGTERM and SIGINT end the run (exit 143, 130) and leave no guest, but **write no run report** | the audit's §12; RW 6.68 | A11 stays partial; A22 now covers signals |
| *"No component has evidence of zero value"* (§12) | One exception: the Verified plugin's shipped-WASM cache path is called only by tests (RW 3.27) | CURRENT SOURCE | Remove it, or wire it behind the cache rules |
| The WASM parity gap: three constructs (§6.2, third pass) | Unchanged on the provenance build | the audit's §10.3 | — |
| OpenShell: **OPTIONAL BACKEND** (§8.6) | v0.1.7 is still the newest release; no gateway-free library mode was found | the audit's §11 (SUB-AGENT) | Unchanged |

**The decisions in §1 stand.** The audit narrows how to make DeluluLang faster and safer for agents. It is the engine,
the process and the edit path that change, never the checker. Its first priority is the seven defects its §12 lists, before
any machine-first input is widened.

---

## 19. Second follow-up, 2026-11-05: the third verification pass

The pass replayed the companion audit's 71 probes (71 identical) and re-witnessed its seven defects on the same code,
now `master` `6397a98`. It re-read every cited source line, refreshed the primary sources, and read CI for the PR and
for `master`. Both runs were green, 14 jobs each, with nothing to fix. The details are in the companion's §19.

| Old claim (where) | New observation | Evidence | Effect on the recommendation |
|---|---|---|---|
| The hostile probes *"all held"*: no containment or authority escape (§6.4); C23 *"Plugin: allowed call works; overbroad grant refused"* | Still true for grants and containment. But **the effect type system does not hold across a plugin's export.** `p.get` never checks the type a host asks for against the export (rule R-Get; its runtime functions exist and nothing on a program's path calls them). A program whose `main` and `authority` say `Load, Read` wrote a file through a `! {Write}` export taken as `! {}`. A package's DL1009 ceiling passed it, and a function listed as pure read a file. The run's grant still bounded the write | companion §19.2; RW 4.95 (HIGH) | **A new first action** (§1 action 2 gains it, ahead of the contract defects): enforce R-Get at `p.get`, with the two witnesses red first. It strengthens §8.4's case: the language's own checks are the part no external sandbox supplies, so they must be enforced at every boundary they claim |
| C26: *"Only the monomorphic, closed-row fragment is machine-checked … Never claim whole-system proof"* | Confirmed, and now with a concrete case. `SOUNDNESS_AUDIT.md`'s F-1 obligation is met by no test on a program's path, because the laundering suite leaves it to the runtime loader | `laundering.rs` lines 4–5; RW 4.95 | Wording discipline stays; add a CLI witness per laundering obligation that names a runtime path |
| RW 4.11 is closed: *"A program can load a plugin at run time"*, refused only for Contained on Windows | A program can load **only Verified** plugins. `Plugin[Contained]` checks clean and is refused at run time (DL1508) | RW 4.96 (LOW) | None to the architecture; a record correction |
| `plugin verify` gives *"identical verdicts to a real load"* | True for the verdict. The export count is not: it counts private functions (RW 6.71, LOW). And the DIR node-id collision (RW 4.93) is now shown **through `plugin verify`**, not only `dir::verify` | companion §19.2 | 4.93 stays MEDIUM; fix with 4.95 |
| Wasmtime: *"keep the 48 LTS pin"* (implicit in §10 and §11) | 48.0.8, the pin, carries the fixes for every advisory found: the critical aarch64 `br_table` escape (RUSTSEC-2026-0379, fixed in 48.0.7), a component-model panic (fixed in 48.0.7) and unchecked memory usage (fixed in 48.0.8). 50.0.1 exists | companion §19.3 (OFFICIAL EXTERNAL: GHSA, RustSec) | Unchanged; the advisory gate in CI is what keeps it true |
| OpenShell: v0.1.7 newest (§8.1, §18) | Still newest; the docs still say not production-qualified | companion §19.3 | **OPTIONAL BACKEND** unchanged |
| §11's competitors had no AI-first language that targets WASM and native | MoonBit (*"AI native programming language toolchain for cloud and edge"*; wasm, wasm-gc, js, native; 1.0 not yet shipped) | companion §19.3 | Supports WASM as the portable target; offers nothing on authority |
| C15 cites `crates/delulu/build.rs:11` for the commit | Line 11 sets the target; the commit is lines 9–15 | re-read | Corrected in place |

**The decisions in §1 stand, with one change of order:** RW 4.95 comes first among the fixes, because it is the one
finding of the three reviews in which a checked guarantee (an effect row, a package ceiling) fails for an ordinary
program. Nothing in this pass changes the architecture, the CLI-first verdict, or the OpenShell verdict.

---

## 20. Third follow-up, 2026-11-05: the hidden-complexity audit

Answered in [`DeluluLang_Hidden_Complexity_AI_First_Experience_Audit.md`](DeluluLang_Hidden_Complexity_AI_First_Experience_Audit.md). That audit looked for the costs an AI pays to finish a task, on the same binary as §19. Old claim,
then new observation:

| Old claim (this report) | New observation | Effect |
|---|---|---|
| §1 action 2's list of soundness work | Unchanged; RW 4.95 re-run, still holds | Still first |
| `delulu lsp` as the agent's warm loop (§7) | It is package-blind: false DL0301 on the shipped greeter (RW 6.72) | The warm loop is right for loose files only until it is fixed |
| `run` outcome shapes (C16, RW 6.60) | Four shapes: a budget stop leaves standard output empty; a signal leaves no report | RW 6.60 amended |
| MCP "works" (C18, RW 6.62) | Works, but cannot be cancelled and holds the next request (RW 6.74); one call returned 3.4 MB | RW 6.62 amended, 6.74 new |
| — | `build --target wasm` writes an unrunnable `.dwx` for `fn main()` (RW 3.28); DL0905 becomes SIGABRT under `ulimit -v` (RW 3.29); grants refused after earlier effects (RW 6.73) | New rows |
| §6.3's sandbox advice (`hostile-agent` for untrusted code) | Still right for confinement; but under it a deep recursion aborts the guest with no code in the report (RW 3.29, corrected by the audit's §11), and no sandboxed fault reaches the report (RW 6.75) | Read a sandboxed exit 1 with stderr |
| — | The audit's second pass (§11): the signing commands ignore unknown options, so `verify-sig --kye` fails open (RW 4.97); MCP, `run`, `edit`, `schema validate` and the LSP answer outside their contracts (RW 6.76–6.81) | New rows |

## Appendix A. Programs written for this session (complete sources)

Codex's 35 Appendix A files were re-extracted byte-exact (SHA-256 match) and used as written. My own:

```delulu
// tpkg/src/main.delulu (with delulu.toml: [package] name="tpkg" kind="bin"; [authority] effects=["Write"])
module tpkg
fn double(n: Int) -> Int { n * 2 }
fn main(root: Root) ! {Write} { root.console().println(str(double(21))) }
test "double doubles" { assert_eq(double(21), 42) }
test "deliberately wrong" { assert_eq(double(2), 5) }
```

```delulu
// child_read.delulu — run under a child lease that holds fs.read=input/child only
module probe.child_read
fn main(root: Root) ! {Read, Write} {
    let out = root.console()
    let rd = root.fs_read("./input/child")
    match rd.read_text("safe.txt") { Ok(v) => out.println(v), Err(e) => out.println("missing") }
    let up = root.fs_read("./input")
    match up.read_text("task.txt") { Ok(v) => out.println("LEAK " + v), Err(e) => out.println("refused-as-value") }
}
```

```delulu
// adversarial set (box/sub/in.txt inside; box/link -> ../outside; box/sub/flink -> ../../outside/s.txt)
module adv.narrow_up
fn main(root: Root) ! {Read, Write} {
    let out = root.console()
    let rd = root.fs_read("./box/sub")
    let wider = rd.narrow("..")
    match wider.read_text("link/s.txt") { Ok(v) => out.println("LEAK1 " + v), Err(e) => out.println("r1 refused") }
    let abs = rd.narrow("/etc")
    match abs.read_text("hostname") { Ok(v) => out.println("LEAK2 " + v), Err(e) => out.println("r2 refused") }
}
// symlink.delulu reads box/link/s.txt and box/sub/flink, writes box/link/new.txt; s4/s5/s6 isolate each case.
module adv.membomb
fn main(root: Root) ! {Write} {
    var s = "x"
    var i = 0
    while i < 31 { s = s + s
 i = i + 1 }
    root.console().println(str(s.len()))
}
```

```delulu
// WASM higher-order isolation (h1: named fn argument; h2: non-capturing lambda) — both DL1201 on WASM
module h.a
fn apply(f: fn(Int) -> Int, x: Int) -> Int { f(x) }
fn inc(x: Int) -> Int { x + 1 }
fn main(root: Root) ! {Write} { root.console().println(str(apply(inc, 5))) }
```

*(Third pass: the program above is non-generic. Its generic form, `fn apply[T, U](f: fn(T) -> U, x: T) -> U`, compiles
on WASM with a named function and with a lambda, and prints `6` on both engines.)*

The everyday corpus `e01`–`e18` used one construct each (listed in §6.1). `e18` used an effect name (`Random`) that does not
exist (DL0306). That is **my own error**, not counted as a gap. Scaling programs (§10.2): `module scale.big`, then `fn f0(x)`
and `fn f{i}(x: Int) -> Int { let r = Rec { a: x, b: "s" } if r.a > 3 { f{i-1}(r.a - 1) } else { r.a * 2 } }` repeated *n*
times, with `main` printing `f{n-1}(10)`.

Fixtures: `input/task.txt` = `value=10\nvalue=11\n`; `input/child/safe.txt` = `child-ok`; `forbidden.txt` =
`sentinel-unchanged` (unchanged at the end); `outside/s.txt` = `secret-outside` (unchanged).

### A.2 Second-pass programs (complete sources; `$SCRATCH` = the session scratch directory, `$D` = the release binary)

`drone.delulu`

```delulu
module sims.drone
type Cmd { id: Int, src: Str, prio: Int, lat: Float, lon: Float }
type St { mode: Str, battery: Int, lat: Float, lon: Float, last_id: Int, silent: Int, log: List[Str] }
fn inside(lat: Float, lon: Float) -> Bool { lat >= 10.0 && lat <= 11.0 && lon >= 20.0 && lon <= 21.0 }
fn step(s: St, c: Option[Cmd]) -> St {
    if s.battery < 20 && s.mode != "LAND" {
        St { mode: "LAND", battery: s.battery - 1, lat: s.lat, lon: s.lon, last_id: s.last_id, silent: s.silent, log: s.log.concat(["LAND:low-battery"]) }
    } else {
        match c {
            None => {
                let quiet = s.silent + 1
                if quiet >= 3 && s.mode == "MISSION" {
                    St { mode: "RTL", battery: s.battery - 1, lat: s.lat, lon: s.lon, last_id: s.last_id, silent: quiet, log: s.log.concat(["RTL:comms-lost"]) }
                } else {
                    St { mode: s.mode, battery: s.battery - 1, lat: s.lat, lon: s.lon, last_id: s.last_id, silent: quiet, log: s.log }
                }
            }
            Some(k) => {
                if k.id <= s.last_id {
                    St { mode: s.mode, battery: s.battery, lat: s.lat, lon: s.lon, last_id: s.last_id, silent: 0, log: s.log.concat(["DROP:dup-or-stale:" + str(k.id)]) }
                } else if !inside(k.lat, k.lon) {
                    St { mode: s.mode, battery: s.battery - 1, lat: s.lat, lon: s.lon, last_id: k.id, silent: 0, log: s.log.concat(["REFUSE:geofence:" + str(k.id)]) }
                } else if s.mode == "RTL" || s.mode == "LAND" {
                    St { mode: s.mode, battery: s.battery - 1, lat: s.lat, lon: s.lon, last_id: k.id, silent: 0, log: s.log.concat(["HOLD:" + s.mode + ":" + str(k.id)]) }
                } else {
                    St { mode: "MISSION", battery: s.battery - 2, lat: k.lat, lon: k.lon, last_id: k.id, silent: 0, log: s.log.concat(["GOTO:" + str(k.id)]) }
                }
            }
        }
    }
}
fn arbitrate(a: Cmd, b: Cmd) -> Cmd { if a.prio >= b.prio { a } else { b } }
fn main(root: Root) ! {Write} {
    let out = root.console()
    var s = St { mode: "IDLE", battery: 30, lat: 10.5, lon: 20.5, last_id: 0, silent: 0, log: [] }
    s = step(s, Some(Cmd { id: 1, src: "op", prio: 1, lat: 10.6, lon: 20.6 }))
    s = step(s, Some(Cmd { id: 1, src: "op", prio: 1, lat: 10.6, lon: 20.6 }))
    s = step(s, Some(Cmd { id: 2, src: "op", prio: 1, lat: 12.0, lon: 20.6 }))
    s = step(s, Some(arbitrate(Cmd { id: 3, src: "op", prio: 1, lat: 10.7, lon: 20.7 }, Cmd { id: 4, src: "safety", prio: 9, lat: 10.5, lon: 20.5 })))
    s = step(s, None)
    s = step(s, None)
    s = step(s, None)
    s = step(s, Some(Cmd { id: 5, src: "op", prio: 1, lat: 10.8, lon: 20.8 }))
    var i = 0
    while i < 12 { s = step(s, None)
 i = i + 1 }
    out.println(s.mode + " battery=" + str(s.battery) + " " + s.log.join(" "))
}
```

`telemetry.delulu (final; runs on the interpreter, DL1201 on WASM)`

```delulu
module sims.telemetry
type Pkt { seq: Int, t_ms: Int, v: Float }
type Q { items: List[Pkt], dropped: Int, dups: Int, stale: Int, highest: Int }
fn has(xs: List[Pkt], seq: Int) -> Bool {
    var i = 0
    var found = false
    while i < xs.len() { match xs.get(i) { Some(p) => if p.seq == seq { found = true }, None => {} }
 i = i + 1 }
    found
}
fn admit(q: Q, p: Pkt, now_ms: Int, skew_ms: Int, limit: Int) -> Q {
    if p.t_ms + 5000 + skew_ms < now_ms { Q { items: q.items, dropped: q.dropped, dups: q.dups, stale: q.stale + 1, highest: q.highest } } else if has(q.items, p.seq) || p.seq <= q.highest - limit { Q { items: q.items, dropped: q.dropped, dups: q.dups + 1, stale: q.stale, highest: q.highest } } else {
        let hi = if p.seq > q.highest { p.seq } else { q.highest }
        let xs = q.items.concat([p])
        if xs.len() > limit { Q { items: xs.slice(1, xs.len()), dropped: q.dropped + 1, dups: q.dups, stale: q.stale, highest: hi } } else { Q { items: xs, dropped: q.dropped, dups: q.dups, stale: q.stale, highest: hi } }
    }
}
fn seqs(xs: List[Pkt]) -> List[Int] {
    var out: List[Int] = []
    var i = 0
    while i < xs.len() { match xs.get(i) { Some(p) => { out = out.concat([p.seq]) }, None => {} }
 i = i + 1 }
    out.sort()
}
fn main(root: Root) ! {Write} {
    let o = root.console()
    let now = 100000
    let skew = 200
    var q = Q { items: [], dropped: 0, dups: 0, stale: 0, highest: 0 }
    q = admit(q, Pkt { seq: 3, t_ms: 99000, v: 1.0 }, now, skew, 4)
    q = admit(q, Pkt { seq: 1, t_ms: 98000, v: 1.0 }, now, skew, 4)
    q = admit(q, Pkt { seq: 3, t_ms: 99000, v: 1.0 }, now, skew, 4)
    q = admit(q, Pkt { seq: 2, t_ms: 90000, v: 1.0 }, now, skew, 4)
    q = admit(q, Pkt { seq: 4, t_ms: 99500, v: 1.0 }, now, skew, 4)
    q = admit(q, Pkt { seq: 5, t_ms: 99600, v: 1.0 }, now, skew, 4)
    q = admit(q, Pkt { seq: 6, t_ms: 99700, v: 1.0 }, now, skew, 4)
    o.println("queue=" + str(seqs(q.items)) + " dropped=" + str(q.dropped) + " dups=" + str(q.dups) + " stale=" + str(q.stale) + " highest=" + str(q.highest))
}
```

`loop_effects.delulu (broker-kill and guest-kill witness)`

```delulu
module p2.loop_effects
fn spin(n: Int) -> Int { var i = 0
 var t = 0
 while i < n { t = t + i % 7
 i = i + 1 }
 t }
fn main(root: Root) ! {Read, Write} {
    let out = root.console()
    let rd = root.fs_read("./input")
    var k = 0
    while k < 80 {
        let _t = spin(400000)
        match rd.read_text("task.txt") { Ok(v) => out.println("tick " + str(k)), Err(e) => out.println("read-err " + str(k)) }
        k = k + 1
    }
    out.println("finished")
}
```

`arm.delulu (signed-driver run; `examples/line_driver` built with `--sign`)`

```delulu
module robot.arm
type Elbow { angle_deg: Float }
fn say(r: Result[Unit, ActuateErr]) -> Str {
    match r {
        Ok(u) => "COMMANDED"
        Err(e) => match e {
            Envelope(reason) => "REFUSED: " + reason
            LeaseRevoked(reason) => "REVOKED: " + reason
            NoDevice => "NODEVICE"
        }
    }
}
fn main(root: Root) ! {Write, Read, Actuate} {
    let c = root.console()
    let a = root.actuator("arm0/elbow")
    c.println(say(a.command(Elbow { angle_deg: 42.5 })))
    c.println(say(a.command(Elbow { angle_deg: 120.0 })))
    c.println(say(a.command(Elbow { angle_deg: 80.0 })))
}
```

`get_ok.delulu / post_ok.delulu (egress; server on 127.0.0.1:18443)`

```delulu
module n.get_ok
fn main(root: Root) ! {Write, Net} {
 let out = root.console()
 let h = root.http(["127.0.0.1/v1/**"])
 match h.get("https://127.0.0.1:18443/v1/ok") { Ok(b) => out.println("GOT " + b), Err(e) => out.println("ERR") }
}
```

`post_ok.delulu`

```delulu
module n.post_ok
fn main(root: Root) ! {Write, Net} {
    let out = root.console()
    let h = root.http(["127.0.0.1:GET,POST/v1/**"])
    let hs = Map()
    match h.post("https://127.0.0.1:18443/v1/ok", hs, "data") { Ok(b) => out.println("POSTED " + b), Err(e) => out.println("ERR") }
}
```

`sec_send_ok.delulu`

```delulu
module s.ok
fn main(root: Root) ! {Write, Net, Read} {
    let out = root.console()
    let s = root.secret("TOKEN")
    let h = root.http(["127.0.0.1"])
    let hs = Map()
    match h.get_with_secret("https://127.0.0.1:18443/v1/ok", hs, "Authorization", s) { Ok(b) => out.println("SENT " + b), Err(e) => out.println("ERR") }
}
```

`py_json.delulu`

```delulu
module py.json
fn parse(py: Cap[Python]) -> Result[Str, PyErr] ! {ForeignCall} {
    let json = py.import("json")?
    let v = json.call_method("dumps", [py.list([py.of_float(1.0), py.of_float(2.5)])])?
    py.to_str(v)
}
fn escape(py: Cap[Python]) -> Result[Str, PyErr] ! {ForeignCall} {
    let os = py.import("os")?
    let v = os.call_method("getcwd", [])?
    py.to_str(v)
}
fn main(root: Root) ! {ForeignCall, Write} {
    let c = root.console()
    let load = root.foreign_load()
    match root.python(load) {
        Ok(py) => {
            match parse(py) { Ok(v) => c.println("json: " + v), Err(e) => c.println("py error " + e.kind) }
            match escape(py) { Ok(v) => c.println("os.getcwd: " + v), Err(e) => c.println("os refused: " + e.kind + " " + e.message) }
        }
        Err(_) => c.println("python unavailable")
    }
}
```

`L3 launchers: l3ok.sh / l3liar.sh / l3attested.sh`

```sh
#!/bin/sh
exec $D $DELULU_GUEST_ARGS
```

`l3liar.sh`

```sh
#!/bin/sh
echo "I am not a guest"
exit 0
```

`l3attested.sh`

```sh
#!/bin/sh
exec $D sandbox attest --key $SCRATCH/home/keys/att1 --attester "test-ci" --guarantee "egress_confinement: test only" -- $SCRATCH/progs/l3/l3ok.sh
```

The frame tee between host and device process (`frametee.py`) copied each line in both directions into a log, prefixed
`HOST>` or `DEV<`, and the device process was `$D device sim --seed 7 --actuator "arm0/elbow:angle_deg=-120..120,heartbeat_ms=60000,ttl_ms=600000,fail=safe-park"`
(the bench is *wider* than the grant's `-95..95`, so a refused command missing from the log was refused by the host). The
HTTPS server (`server.py`) was Python's `http.server` behind `ssl`, with a leaf certificate signed by a throwaway CA
generated with `openssl` in scratch and given to `delulu` via `SSL_CERT_FILE`; it logged method, path and headers. The
secret's value was masked when the log was read.


## Appendix B. Command ledger

Common environment: `DELULU_HOME` and `DELULU_STATE_DIR` in scratch, `NO_COLOR=1`; `$D` = the release binary.
Representative argv (the rest follow the same shape):

```sh
$D run utility.delulu --grant console --grant fs.read=./input --grant fs.write=./output --engine interp|wasm --no-prompt --limits mem=134217728,cpu=2,wall=5
$D run utility.delulu --sandbox [--sandbox-profile hostile-agent] --grant console --grant fs.read=./input --grant fs.write=./output --no-prompt --report-out R.json
$D run device_sim.delulu --grant console --grant 'actuator=review/elbow:angle_deg=-30..95,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park' --grant 'sensor=review/elbow#angle_deg' [--broker-profile sim] [--seed 7] [--sandbox] --no-prompt
$D authority dynamic.delulu --within console --within fs.read=./input --json
$D test [/no/such/dir | doubles | zzz_nomatch] [--json]          # inside tpkg/ and Codex's package/
$D fix app.delulu --json ; $D fix app.delulu --accept-widening add_effect_to_row --json
$D edit app.delulu --expect-hash <H> --edits '[{"insert":"// edited by agent\n","range":{"start_byte":0,"end_byte":0}}]' --if-checks --json
$D broker start ; $D grants delegate --effects Read,Write --fs-read $W/input --ttl 10m --json
$D grants delegate --parent <PN> --effects Read,Write --fs-read $W/input/child --ttl 5m --multi --json
$D run child_read.delulu --broker daemon --lease <CT> --no-prompt ; $D grants revoke <PN> ; $D audit verify ; $D broker stop
python3 -I mcp_probe.py $D <dir> effect_missing.delulu effect_repaired.delulu   # initialize, tools/list, tools/call check ×2, unknown tool, option-like path
python3 -I lsp_probe.py $D effect_missing.delulu                                 # initialize, didOpen, publishDiagnostics, shutdown
```

The broker run printed a one-time Guard owner code and lease tokens. They are scratch-only, the daemon was stopped, and
none is reproduced here.

Per-probe outcomes (last run of each ID; `$SCRATCH` = the session scratch directory):

| Probe ID | Exit | ms | First line of stdout / stderr |
|---|---:|---:|---|
| `basics_while-interp` / `-wasm` | 0 / 0 | 8 / 12 | out: Some(65) |
| `basics_range-interp` / `-wasm` | 0 / **1** | 7 / 6 | wasm err: error[DL1201]: WASM codegen does not support a call to `range` (not a compilable function) |
| `utility-interp` / `-wasm` / `-sandbox` / `-hostile` | 0 / 0 / 0 / 0 | 8 / 14 / 56 / 51 | out: {"ok":true,"value":42} |
| `workflow-*`, `delegation-*`, `edge-*`, `simulation-*` (both engines) | 0 | 6–45 | out: done:0:0 … / child-ok / 4054095 / MOVE:40 |
| `utility-nogrant-*` | 1 | 6–13 | err: error[DL0703]: `console` was not granted to this program — pass `--grant console` |
| `outside-*`, `outside-sandbox` | 1 | 7–50 | err: error[DL0904]: path `../forbidden.txt` escapes the granted scope |
| `delegation_denied-*` | 1 | 7–13 | err: error[DL0904]: path `../task.txt` escapes the granted scope |
| `budget-interp` / `-wasm` | 1 | 1014 / 1013 | err: the run was stopped: it ran for 1 s of wall-clock time, its budget |
| `args_missing` / `compound_missing` / `json_missing` / `math_missing` / `method_missing` | 1 | 5–6 | DL0405 / DL0202 / DL0301 / DL0301 / DL0405 |
| `interpolation-run` | 0 | 7 | out: hello {name} |
| `effect_missing` (check) | 1 | 5 | DL0501 with repair `add_effect_to_row (exact) [widens authority]` |
| `pkg-dir` / `pkg-file` | 0 / 1 | 6 / 6 | out: 42 / err: DL0301 unknown name `twice` |
| `tpkg-test-all` / `-filter-ok` / `-filter-none` / `-missing-path` / `-missing-path-json` | 1 / 0 / **0** / **0** / **0** | 5–6 | 1 passed, 1 failed / 1 passed / 0 passed, 0 failed (×3) |
| `device-nogrant` → `device-follow-hint` | 1 → **2** | 7 / 5 | DL0703 hint `actuator=review/motor:DIM=LO..HI` → "no `heartbeat_ms` … is not a grant" |
| `device-sim-interp` (no profile) | 0 | 32 | COMMANDED · REFUSED … [-30, 95] · **NO READING** |
| `device-sim-codex-exact` / `-sandbox-profile` | 0 / 0 | 6 / 54 | COMMANDED · REFUSED … · READ 12.0 |
| `device-sim-wasm` / `-sandbox-seed` | 1 / 2 | 6 / 5 | DL1201 `.actuator` / "a sandboxed run does not apply `--seed` yet. Nothing ran" |
| `plugin-host` / `-overbroad` / `-tampered` / `-restored` | 0 | 6–8 | hello! / not granted: … exceeds the plugin's declared ceiling / verification failed … never falls back to Contained / hello! |
| `plugin-nogrant` / `-sandbox` / `-wasm` | 1 / 2 / 1 | 5–7 | DL0703 plugin loading / `--sandbox` cannot carry this program yet: it uses PluginHost / DL1201 `.plugin_host` |
| `utility-within` / `utility-exceeds` / `dynamic-within` / `dynamic-run` | 0 / 1 / 1 / 0 | 5–6 | within / exceeds fs.write=./output / inconclusive (computed) / value=10 |
| `loop-1-check` / `loop-2-fix-default` / `loop-3-fix-accept` | 1 / **0** / 0 | 5 | DL0501 / applied 0, errors_after 1 / applied 1, errors_after 0, summary.errors 1 |
| `edit-stale` / `edit-ok` / `edit-replay-old-hash` | 2 / 0 / 2 | 5–6 | refused (hash) / applied with authority before/after / refused |
| `adv-narrow-up` / `adv-symlink` / `adv-s4` / `adv-s5` / `adv-s6` / `adv-s6-sandbox` | 1 | 6–50 | DL0904 each |
| `adv-membomb-64m` / `-wasm` | 1 | 85 / 61 | stopped: used 78.0 / 103.1 MiB, budget 64.0 MiB |
| `adv-reportout-in-grant` | 2 | 5 | --report-out lies inside a scope this program is granted to write |
| MCP probe | 0 | 28 | 12 tools, all `readOnlyHint:true`; check bad → isError False, errors 1; `run` → -32602; `--help` path refused |
| LSP probe | 0 | 6 | 15 capabilities; diagnostics `['DL0501']` |

### B.2 Second-pass probe ledger (last run of each ID)

| Probe ID | argv (after `delulu`) | Exit | ms | First line of stdout / stderr |
|---|---|---:|---:|---|
| `new-hello` | `new hello --json` | 0 | 6 | out: { |
| `new-hello-run` | `run . --grant console --no-prompt` | 0 | 5 | out: hello, world |
| `new-hello-test` | `test` | 0 | 5 | out: test result: 1 passed, 0 failed |
| `fmt-check` | `fmt --check utility.delulu` | 1 | 6 | out: would reformat: utility.delulu |
| `explain` | `explain E-DL0703` | 0 | 4 | out: DL0703: root slice not granted |
| `examples` | `examples --json` | 0 | 7 | out: { |
| `skill` | `skill --json` | 0 | 4 | out: { |
| `toolchain` | `toolchain --json` | 0 | 7 | out: { |
| `atlas-mermaid` | `atlas utility.delulu --format mermaid` | 0 | 6 | out: graph LR |
| `why` | `why Write utility.delulu` | 0 | 4 | out: main (utility.delulu:30) — Write |
| `cpu-limit` | `run busy.delulu --grant console --no-prompt --limits cpu=1` | 1 | 1010 | err: error: the run was stopped: it used 1.0 s of processor time and its budget is 1 s (D-V2-25). Budgets are the operator's, set per run with `- |
| `audit-mode` | `run utility.delulu --sandbox --mode audit --grant console --no-prompt` | 0 | 5 | out: { |
| `trace` | `run utility.delulu --grant console --grant fs.read=./input --grant fs.write=./output --trace-effects --asse…` | 0 | 6 | out: {"ok":true,"value":42} |
| `openshell-export` | `sandbox policy utility.delulu --format openshell --grant console --grant fs.read=./input --grant fs.write=.…` | 0 | 4 | out: { |
| `l3-ok` | `run utility.delulu --sandbox --sandbox-backend external:$SCRATCH/progs/l3/l3ok.sh --grant console --grant f…` | 0 | 34 | out: {"ok":true,"value":42} |
| `l3-silent` | `run utility.delulu --sandbox --sandbox-backend external:$SCRATCH/progs/l3/l3silent.sh --grant console --gra…` | 1 | 8007 | err: error: the run was stopped: it ran for 8 s of wall-clock time, its budget. Budgets are the operator's, set per run with `--limits wall=SECON |
| `l3-liar` | `run utility.delulu --sandbox --sandbox-backend external:$SCRATCH/progs/l3/l3liar.sh --grant console --grant…` | 1 | 8 | err: sandbox: the guest never confirmed its boundary, so it was not sent the program: frame length exceeds the channel's bound |
| `l3-wrong-digest` | `run utility.delulu --sandbox --sandbox-backend external:$SCRATCH/progs/l3/l3ok.sh --launcher-digest 0000000…` | 1 | 7 | err: error: the sandboxed run failed: `$SCRATCH/progs/l3/l3ok.sh` is not the pinned launcher: its BLAKE3 is d75f231f5550f4c42f448504690c6e348bd5d |
| `l3-hostile` | `run utility.delulu --sandbox --sandbox-profile hostile-agent --sandbox-backend external:$SCRATCH/progs/l3/l…` | 2 | 31 | err: launcher: sandbox: the guest narrowed its own view — no file writes; reads only from the system paths; no TCP bind or connect |
| `att-none` | `run utility.delulu --sandbox --require-attestation 4ac1e7ac99c1305f07d553e970fea7a5b191ea0c87824dc90829fe76…` | 1 | 10023 | err: error: the sandboxed run failed: the guest was not served: the launcher wrote no attestation within 10s (an attester writes it to `$DELULU_A |
| `att-good` | `run utility.delulu --sandbox --require-attestation 4ac1e7ac99c1305f07d553e970fea7a5b191ea0c87824dc90829fe76…` | 0 | 52 | out: {"ok":true,"value":42} |
| `att-wrongkey` | `run utility.delulu --sandbox --require-attestation 87ffb825f3ff61f7163739ea8c32265bb47d6bfae10a86f9d285c72b…` | 1 | 27 | err: error: the sandboxed run failed: the guest was not served: the attestation is signed by 4ac1e7ac99c1305f07d553e970fea7a5b191ea0c87824dc90829 |
| `att-l1` | `run utility.delulu --sandbox --require-attestation 4ac1e7ac99c1305f07d553e970fea7a5b191ea0c87824dc90829fe76…` | 2 | 3 | err: error: `--require-attestation` is for an external launcher (`--sandbox-backend external:CMD`, level 3): this host measures the jailed proces |
| `net-get-nogrant` | `run get_ok.delulu --grant console --no-prompt` | 1 | 6 | err: error[DL0703]: network access to `127.0.0.1/v1/**` was not granted to this program — pass `--grant net.special=127.0.0.1/v1/**` |
| `net-get-plain-net` | `run get_ok.delulu --grant console --grant net=127.0.0.1/v1/** --no-prompt` | 2 | 4 | err: error: bad --grant: bad grant `net=127.0.0.1/v1/**` — `127.0.0.1/v1/**` is a special-use address: loopback (127.0.0.0/8). A plain `net=` gra |
| `net-get-ok` | `run get_ok.delulu --grant console --grant net.special=127.0.0.1/v1/** --no-prompt --report-out $SCRATCH/log…` | 0 | 10 | out: GOT pong |
| `net-get-outside` | `run get_outside.delulu --grant console --grant net.special=127.0.0.1/v1/** --no-prompt` | 1 | 5 | err: error[DL0904]: the path of `https://127.0.0.1:18443/v2/secret` is outside every path the grant names for its host |
| `net-post-noperm` | `run post_noperm.delulu --grant console --grant net.special=127.0.0.1/v1/** --no-prompt` | 1 | 5 | err: error[DL0904]: a `POST` to `https://127.0.0.1:18443/v1/ok` is outside the methods the grant names for it |
| `net-post-ok` | `run post_ok.delulu --grant console --grant net.special=127.0.0.1:GET,POST/v1/** --no-prompt` | 0 | 7 | out: POSTED posted |
| `net-redirect` | `run redirect.delulu --grant console --grant net.special=127.0.0.1 --no-prompt --report-out $SCRATCH/logs/ne…` | 0 | 7 | out: ERR redirect refused as value |
| `net-plain-http` | `run plain_http.delulu --grant console --grant net.special=127.0.0.1 --no-prompt` | 0 | 4 | out: ERR scheme refused as value |
| `net-dotdot` | `run dotdot.delulu --grant console --grant net.special=127.0.0.1/v1/** --no-prompt` | 1 | 4 | err: error[DL0904]: `https://127.0.0.1:18443/v1/../v2/secret` is refused under a path scope: a `.` or `..` segment in the path, which a URL parse |
| `net-sandbox-get` | `run get_ok.delulu --sandbox --grant console --grant net.special=127.0.0.1/v1/** --no-prompt` | 0 | 52 | out: GOT pong |
| `net-wasm-get` | `run get_ok.delulu --engine wasm --grant console --grant net.special=127.0.0.1/v1/** --no-prompt` | 0 | 13 | out: GOT pong |
| `net-within` | `authority post_ok.delulu --within console --within net.special=127.0.0.1/v1/**` | 1 | 4 | out: exceeds: the program asks for authority the boundary does not grant |
| `sec-print` | `check sec_print.delulu` | 1 | 4 | err: error[DL0604]: value of type `Secret[Str]` cannot be stringified |
| `sec-send-ok` | `run sec_send_ok.delulu --grant console --grant net.special=127.0.0.1 --grant secret:TOKEN@127.0.0.1:18443=s…` | 0 | 8 | out: SENT pong |
| `sec-send-noaudience` | `run sec_send_ok.delulu --grant console --grant net.special=127.0.0.1 --grant secret:TOKEN=<throwaway-test-value> --no…` | 0 | 4 | out: ERR |
| `sec-send-elsewhere` | `run sec_send_elsewhere.delulu --grant console --grant net.special=localhost --grant secret:TOKEN@127.0.0.1:…` | 0 | 4 | out: ERR |
| `sec-authority` | `authority sec_send_ok.delulu --json` | 0 | 4 | out: { |
| `py-check` | `check py_json.delulu` | 0 | 5 | err: ok: py_json.delulu checked clean |
| `py-nogrant` | `run py_json.delulu --grant console --no-prompt` | 1 | 4 | err: error[DL1303]: embedded Python was not granted — pass `--grant foreign.python=numpy` (an import allowlist pattern; the human chooses which m |
| `py-json-only` | `run py_json.delulu --grant console --grant foreign.python=json --no-prompt` | 0 | 410 | out: json: [1.0, 2.5] |
| `py-sandbox` | `run py_json.delulu --sandbox --grant console --grant foreign.python=json --no-prompt` | 2 | 5 | err: error: `--sandbox` cannot carry this program yet: it uses ForeignLoad, Python. Nothing ran. Run it with `--sandbox=off` if you accept no con |
| `py-authority` | `authority py_json.delulu` | 0 | 4 | out: Authority of `py.json` — what this program can do to your system: |
| `multi-build` | `build . --json` | 0 | 6 | out: { |
| `multi-lock` | `lock . --json` | 0 | 6 | out: { |
| `multi-run` | `run . --grant console --no-prompt` | 1 | 6 | err: error[DL0703]: filesystem read of `./data` was not granted — pass `--grant fs.read=./data` |
| `multi-authority` | `authority .` | 0 | 6 | out: Authority of `station` — what this program can do to your system: |
| `multi-build-after-widen` | `build . --locked --json` | 1 | 5 | out: { |
| `actors-interp` | `run 06_actors.delulu --grant console --no-prompt` | 0 | 6 | -:  |
| `actors-wasm` | `run 06_actors.delulu --engine wasm --grant console --no-prompt` | 0 | 10 | err: engine: wasm (actors: cooperative single-threaded — semantics identical, parallelism absent) |
| `actors-sandbox` | `run 06_actors.delulu --sandbox --grant console --no-prompt` | 2 | 4 | err: error: `--sandbox` cannot carry this program yet: it uses actors. Nothing ran. Run it with `--sandbox=off` if you accept no confinement, or  |

Not in this table because they were driven by scripts (their transcripts are summarised in §6.7): the 102-entry Codex
re-run (`rerun_appendix_b.py`), the timestamped broker-kill and guest-kill runs (`tsrun.py`), the attestation runs, the
proposal flow, the Guard and strict-root runs, the egress and secret runs against the logging server, the signed-driver
runs, the LSP and MCP soaks, and the agent-loop benchmark. Lease tokens and the Guard owner code printed by those runs
stayed in scratch and are not reproduced.

## Appendix C. Raw benchmark samples

Wall-clock ms per sample (31 per case, collection order) and median peak RSS in MiB:

```text
version-json             rss_med=10.67  6.7 6.1 5.8 6.4 7.2 6.5 9.3 6.3 6.4 6.8 6.5 6.5 5.9 6.9 6.1 6.4 8.3 6.5 6.5 6.1 6.4 6.2 6.5 6.8 6.8 8.9 7.4 7.0 7.0 12.9 5.9
check-utility-json       rss_med=13.20  8.2 8.6 6.3 7.3 8.0 8.7 8.7 7.5 6.9 8.3 7.4 7.0 6.4 8.9 6.8 6.8 7.2 7.2 7.4 6.5 7.3 9.6 7.3 8.8 9.0 7.9 7.8 7.9 9.6 7.4 6.6
authority-utility-json   rss_med=13.80  8.3 7.1 6.5 7.4 9.2 10.9 10.5 7.6 6.9 7.3 8.8 7.3 8.6 8.6 6.6 7.1 8.3 8.4 7.7 7.0 8.2 7.5 7.5 7.4 7.0 7.4 11.1 8.1 8.9 8.0 6.4
utility-interp           rss_med=14.78  9.4 8.9 7.3 10.0 9.7 8.7 10.1 8.3 7.9 8.2 7.6 8.0 6.9 8.6 7.2 7.6 10.1 8.0 8.0 7.7 8.0 9.8 12.0 10.0 7.9 7.8 10.6 9.0 10.1 9.8 6.8
utility-wasm             rss_med=24.92  16.8 13.8 11.7 12.8 15.7 15.0 15.2 14.0 14.7 14.9 14.2 12.7 11.7 17.4 12.1 13.3 13.2 12.4 13.5 12.5 14.0 18.3 14.9 14.3 14.6 13.1 15.6 15.4 18.6 13.5 11.6
utility-sandbox-L1       rss_med=16.19  58.4 53.3 52.0 53.5 56.4 55.2 54.5 54.0 52.6 54.6 53.3 52.6 52.1 53.9 52.6 53.2 53.1 52.9 52.8 54.3 53.7 55.8 53.7 57.0 53.6 52.7 57.3 56.8 53.6 56.5 55.1
utility-python           rss_med=12.64  28.8 33.0 29.1 28.0 29.2 46.9 28.9 29.2 34.2 28.7 37.0 43.6 35.6 32.4 28.7 29.2 29.1 46.4 37.0 28.9 28.4 29.0 29.1 30.3 29.5 29.2 30.2 33.3 29.2 28.2 32.7
edge100k-interp          rss_med=14.40  50.8 67.2 67.5 43.5 56.7 53.6 46.0 45.7 81.0 46.7 73.2 83.9 44.1 79.0 44.3 45.1 45.5 47.3 46.6 79.6 64.0 46.9 44.8 46.7 44.6 46.4 48.0 46.4 46.3 49.8 47.7
edge100k-wasm            rss_med=23.86  15.0 14.2 12.9 10.5 15.2 13.1 16.7 13.2 12.1 13.0 11.0 12.4 10.4 12.6 10.8 11.5 11.6 13.9 10.9 11.1 11.1 15.2 12.0 15.1 12.1 10.7 12.9 12.9 14.2 12.5 14.8
edge100k-dwx             rss_med=22.11  13.5 12.0 13.0 9.8 15.2 14.2 13.0 12.2 12.5 11.6 10.5 11.3 9.3 13.7 12.0 10.0 10.9 11.8 10.9 10.8 10.0 14.7 11.4 13.8 11.5 10.1 14.8 13.0 12.1 13.3 12.1
edge100k-python          rss_med=10.40  28.9 32.2 29.0 26.3 29.1 38.1 27.7 37.9 27.8 27.6 26.3 27.6 25.8 28.2 28.1 26.8 27.8 27.9 29.0 32.7 26.0 28.3 29.0 36.1 30.0 33.1 28.4 28.6 28.6 49.1 28.8
edge100k-rust            rss_med=9.62  4.1 4.7 4.5 3.6 4.0 5.2 4.0 5.3 3.6 3.7 4.0 3.7 3.6 3.9 3.7 3.5 4.0 4.0 4.3 3.6 3.5 4.4 3.8 4.0 4.0 3.7 4.0 4.3 4.3 4.8 3.7
edge5M-interp            rss_med=14.37  2067.4 2345.9 1963.5 2117.3 2030.4 2060.0 1881.5 1965.1 1908.7 1912.4 1910.5 1878.7 2153.8 1904.2 2006.5 1994.3 2001.7 1848.9 1909.1 2033.0 2034.5 1968.0 1912.3 1906.4 1855.4 1956.3 2030.6 2026.4 2087.9 1929.0 2028.7
edge5M-wasm              rss_med=23.91  35.4 51.9 32.0 35.2 34.6 43.2 36.3 32.7 32.1 32.0 34.2 33.9 53.6 32.1 34.6 47.5 32.6 31.8 31.2 32.9 34.6 37.2 34.0 33.8 33.2 38.4 33.1 35.7 33.4 31.9 50.5
edge5M-dwx               rss_med=22.09  33.4 31.9 30.9 54.1 32.9 33.8 45.9 32.5 31.7 47.2 38.9 33.2 58.8 31.6 32.5 32.2 35.8 31.5 32.6 36.7 39.1 32.2 31.6 32.0 32.6 38.2 32.8 36.7 32.5 31.4 31.7
edge5M-python            rss_med=10.39  578.6 572.8 635.7 760.1 683.6 623.6 553.4 571.0 596.9 551.4 645.9 626.2 570.6 561.4 693.9 748.1 699.2 613.6 623.0 556.0 632.5 649.6 765.2 641.3 633.4 617.9 659.2 602.0 587.0 543.5 723.0
edge5M-rust              rss_med=9.62  9.9 9.5 9.9 10.2 18.5 10.4 11.0 25.8 12.2 9.9 10.0 10.2 16.9 9.8 9.9 9.9 9.5 9.8 9.8 9.7 9.9 10.0 14.9 11.7 9.6 10.1 10.3 11.2 14.1 9.8 11.9
```

## Appendix D. Sources and references

**Repository (CURRENT SOURCE, `0670e0b`):** `crates/delulu/Cargo.toml`; `crates/delulu/build.rs` (lines 1–17);
`crates/delulu/src/mcp.rs` (lines 367–410, 469–508); `crates/delulu/src/lsp.rs` (lines 1213–1265); `crates/delulu/src/cli.rs` (lines 9468–9500);
`crates/delulu/src/doctor.rs` (lines 60–120, 780–830); `crates/delulu-survey/src/main.rs` (lines 309–346);
`crates/delulu-runtime/src/device.rs` (lines 520–575, 720–800); `crates/delulu-runtime/src/prim.rs` (lines 1023–1060);
`crates/delulu-wasm/src/codegen.rs` (lines 1027–1038, 4649); `crates/delulu-broker/src/lease.rs` (lines 183–255);
`crates/delulu/tests/repository_structure.rs` (lines 30–50); `editors/vscode/extension.js` (lines 255–290); `.github/workflows/openshell.yml`;
`.github/workflows/ci.yml` (lines 6–23); `docs/design/DELULU_CORE.md` (lines 7, 140–150, 392); `docs/DELULULANG_V2/V2_OPENSHELL_STUDY.md`
§§0–7; `docs/REMAINING_WORK.md` rows 1.15, 2.33–2.40, 3.24, 6.50–6.53; `docs/for-agents.md` [agents.exit-codes],
[agents.tests]; `HANDOFF.md` §§0, 1, 11; `docs/AI_ONBOARDING.md`.

**CI (CURRENT CI, read 2026-11-04):** runs `41929573373`, `41813603704`, `41619070519`, `41394552103`, `37234907676` (§2.3).

**External (OFFICIAL EXTERNAL, fetched 2026-11-04):**
[NVIDIA/OpenShell](https://github.com/NVIDIA/OpenShell) ·
[OpenShell releases](https://github.com/NVIDIA/OpenShell/releases) ·
[OpenShell architecture](https://docs.nvidia.com/openshell/latest/about/architecture) ·
[OpenShell support matrix](https://docs.nvidia.com/openshell/latest/about/support-matrix) ·
[vercel-labs/zerolang](https://github.com/vercel-labs/zerolang) · [zerolang.ai](https://zerolang.ai/) ·
[Zerolang target capabilities](https://zerolang.ai/target-capabilities) · [mojolang.org](https://mojolang.org/) ·
[Mojo requirements](https://mojolang.org/docs/requirements/) ·
[Deno security](https://docs.deno.com/runtime/fundamentals/security/) ·
[Wasmtime releases](https://github.com/bytecodealliance/wasmtime/releases) ·
[ponyc releases](https://github.com/ponylang/ponyc/releases) · [roc-lang.org](https://www.roc-lang.org/) ·
[MoonBit beta announcement](https://www.moonbitlang.com/blog/beta-release).
**Secondary, used only to find questions (not as evidence):** OpenShell native-Windows discussion
([#2006](https://github.com/NVIDIA/OpenShell/discussions/2006), [DeepWiki MXC page](https://deepwiki.com/NVIDIA/OpenShell/8.10-windows-native-support-(mxc-driver)));
Koka version listings ([gittrend](https://gittrend.io/repo/koka-lang/koka)); MoonBit version
([issoh.co.jp](https://www.issoh.co.jp/tech/details/8968/)).

**Second pass (fetched or searched 2026-11-05):**
- **OFFICIAL EXTERNAL:**
  - [OpenShell docs, latest = v0.1.7](https://docs.nvidia.com/openshell/latest/) ·
    [OpenShell security best practices](https://docs.nvidia.com/openshell/latest/security/best-practices)
  - [JEP 486](https://openjdk.org/jeps/486) ·
    [Oracle: the Security Manager is permanently disabled](https://docs.oracle.com/en/java/javase/24/security/security-manager-is-permanently-disabled.html)
  - [WASI releases](https://wasi.dev/releases) · [WASI P3](https://wasi.dev/releases/wasi-p3)
  - [Node.js permissions](https://nodejs.org/api/permissions.html)
  - [Roc platforms](https://www.roc-lang.org/platforms)
  - [CN110737431A on Google Patents](https://patents.google.com/patent/CN110737431A/en)
  - [ShubhamBhut/kinglang](https://github.com/ShubhamBhut/kinglang) · [publidoo/kinglang](https://github.com/publidoo/kinglang)
  - `github.com/delulu-lang`: HTTP 404; `gh api search/users q=delulu-lang` → `total_count: 0`.
- **Secondary:**
  - [Gleanmark: USPTO 99795106 DELULU](https://gleanmark.com/trademark/uspto-99795106)
  - [Better Stack guide to Zero](https://betterstack.com/community/guides/ai/zero-language/) and an
    [indexed copy of the zerolang docs](https://docsearch.algolia.com/mcp/docs/repo/vercel-labs/zerolang) (for `std.json`)
  - [E2B about](https://e2b.dev/about.md)
  - [Modular forum: Windows timeline](https://forum.modular.com/t/timeline-for-windows-support/3268)
  - Koka mirror [v3.2.3](https://github.xxxxic.workers.dev/koka-lang/koka/releases/tag/v3.2.3) and
    [Alpine package](https://pkgs.alpinelinux.org/package/edge/community/x86_64/koka)

## Appendix E. The prompt's requirements, one by one (second pass)

| Prompt § | Requirement | Where answered | Status |
|---|---|---|---|
| 0 | AI-first purpose; kind-blind authority; authorization ≠ physical safety; core vs stdlib vs adapter vs plugin vs external | §1, §5, §6.2 (layer column), §11.2, §14 | done |
| 1 | Direct choice among options 1–4; CLI-only vs CLI-first; simplify / modularise / replace / narrow / rebuild verdict | §1, §7 | done (option 3; narrow + incremental) |
| 2 | Read onboarding, AGENTS, HANDOFF (§0, §1, §11), REMAINING_WORK, README, INSTALL, CONTRIBUTING, Codex review in full | §2, §3 | done. **Correction:** the first pass had not read `INSTALL.md` or `CONTRIBUTING.md`; the second pass did, in full. Three consequences: F5 is documented behaviour; language-core changes need RFCs (§13); `CONTRIBUTING.md`'s Survey example is stale (`check.rs` impact is 244 now, not 134) |
| 2 | Record working dir, branch, SHA, cleanliness, origin, branches, tools, OS/CPU, resources, CI with IDs | §2.1–§2.3 | done; CI read, nothing dispatched |
| 2 | Don't start or unpause the routine; verify its state | §2.1 | done: both disabled, read only |
| 3 | Research, test and report only; no edits; new report only; disclose metadata writes and the delivery mechanism | header, §2.4 | done; report-only commits on the session branch |
| 4 | Survey and Doctor first, then reuse (`impact`, `affected-by`, `path`, `query`, `findings`, `rdeps` semantics); no `survey build`, no plain `doctor` | §2.2, §4.3, §12 | done; `path`/`rdeps` limits recorded (F9, §4.3 note) |
| 4 | Keep Authority, Guard, broker, Survey, Doctor, Atlas, plugins, MCP and LSP distinct; derive shared facts without merging security responsibilities | §4.2, §9, §12 | done |
| 5 | Codex claim ledger: ID, statement, evidence, availability, independent evidence, verdict, references, impact | §3.1 (C01–C44) | done |
| 5 | Re-examine architecture, scale, fundamentals, backend drift, test/outcome contracts, installed binary, authority, health/CI, programs, benchmarks, formal/security, naming, recommendations | §3.1, §3.2, §3.3 | done; the installed laptop binary is blocked |
| 5 | Reproducible command ledger, separate from the narrative | Appendix B (B.1, B.2), §3.3 | done |
| 6 | Zerolang, Mojo, Python, Rust, Go, Java, TS/JS, Zig, Koka, Pony, Roc, Deno, WASI/Wasmtime, one more, OpenShell; matrix with verified/partial/unknown/absent/roadmap; lesson verdicts; the "Python in a strong sandbox" baseline | §11.1, §11.2 | done at documentary level; nothing installed (`curl \| sh` forbidden). Go and Zig rest on Codex's sources plus general knowledge, labelled; Rust, C/C++ and Zig are trusted host/adapter languages, not competitors |
| 7 | OpenShell: upstream as of today; responsibilities matrix; options A–E with every criterion; boundary tests; PoC plan; unequivocal verdict; invariants; first removal; reversal evidence | §8.1–§8.6, §6.7 | done; no live OpenShell run (not installable here) |
| 7.3 | Boundary tests: allowed/denied files, network method/path, secret injection and refusal, inconclusive, narrowing, expiry, revocation, stale decisions, broker/supervisor loss, launch before confirmation, guest crash, competing grants, replay, duplicates, Python/FFI, root issuance, absent OpenShell, physical-envelope placement | §6.4, §6.6, §6.7 | most done; not done: duplicate *effect* idempotency, FFI, broker crash *recovery*, OpenShell-specific supervisor loss (§16) |
| 8 | Component audit with practical improvements; six questions per simplification | §9, §12 (six-question table) | done |
| 9 | Portability tiers; five kinds of "same"; physical vs supervisory separation | §10.3, §14 | done |
| 10 | Real programs, negative probes, scenarios 1–11, benchmarks (agent utility, analysis loop, edge compute; warm-up, samples, percentiles, RSS, interpreter vs WASM, cold vs warm, repeated sessions, fair controls) | §6, §6.7, §10, Appendix C | done except cold start (not measured; reason in §16) and scenario 4's durable restart (not built) |
| 11 | Which V2 phases are complete or overengineered; the smallest defensible release; roadmap with deletion order; pass/fail criteria; OpenShell gates | §13, §15, §8.6 | done |
| 12 | Owner's hypotheses proved or refuted | §16.1 | done |
| 13 | Sub-agents only if cheap and verified | §2.1 | first pass: none; second pass: one page-fetch helper, disclosed, its consequential claim re-checked |
| 14 | 18-section structure; evidence labels; five outcome classes | whole report | done |
| 15 | Completeness checklist | this appendix | done; the open items are in §16 |
