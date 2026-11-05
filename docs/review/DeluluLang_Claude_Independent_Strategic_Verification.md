# DeluluLang — Claude's independent strategic verification, architecture review and OpenShell decision

**Written:** 2026-11-04/05 (UTC), in a Claude Code cloud session (`session_01Deh6W9bJdfxAeMARtXU9R9`; configured and
last-served model both `claude-opus-5-5`, read from the session record; no model-fallback notice). **Commissioned by:** the owner's prompt
[`docs/prompt/DeluluLang_Claude_Code_Cloud_Strategic_Verification_Prompt.md`](../prompt/DeluluLang_Claude_Code_Cloud_Strategic_Verification_Prompt.md),
read in full, plus his request in this session to *"check for bugs and security and other issues also"*.
**Tree examined:** `master` = `0670e0bb4db8160469d1f092ab0b088aa4b76220`. **Nature:** research, testing and a report. No
source, test, configuration, existing document or memory file was changed. This file is the only addition to the
repository (§2.4 says exactly how it got there).

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
   *documented* spellings must work: today `for i in range(0, n)` and any user-defined higher-order function fail to
   compile there.
2. **Fix the machine-contract defects agents trip on** (§6.3, §9): `test` reporting success when nothing ran; `fix`
   exiting 0 with errors left; null-device actuation reported as `COMMANDED`; a DL0703 hint that recommends an invalid
   grant; `run --json` putting two different things on stdout; `--version` printing `commit: null` for every source
   install.
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
| Agents | none spawned (the prompt allows only verified-cheap ones; I did not need them) |

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
| C15 | Installed `delulu` stale, 1.0.0 (§1) | Observed (laptop) | Laptop not accessible. **New:** a fresh source build reports `commit: null`; only `DELULU_BUILD_COMMIT` (the packaging script, the release workflow) fills it (`crates/delulu/build.rs:11`) | **NOT REPRODUCIBLE HERE**; Codex's "RW 6.53 partly stale" **MODIFIED**: README's `cargo install --path` still yields an unidentifiable binary | Identity not fixed for source installs |
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
| C34 | Koka v3.2.8, 17 Sep 2026 (§9) | External | My search found v3.2.3 as the newest visible tag; the official book was not re-read | **INCONCLUSIVE** | Irrelevant to the decision |
| C35 | Trademark and naming findings (§18) | External | Not re-researched (out of the prompt's decision scope) | **NOT RE-VERIFIED** | Background only; no legal conclusion |
| C36 | Device simulation *"in Windows AppContainer at L1"* (§1) | Observed (Windows) | On Linux the same program runs at L1 under Landlock and seccomp, `fully_enforced:true` | **NOT REPRODUCIBLE HERE** (Linux equivalent CONFIRMED) | — |

### 3.2 Codex's recommendations

| Recommendation | Verdict | Why |
|---|---|---|
| Keep the language and checker; do not rewrite | **AGREE** | The effect/authority core works and held every attack I ran (§6.4). |
| Keep a lightweight LSP and a thin VS Code client | **AGREE** | 409 lines of JS. Removing it saves nothing; the LSP is also the only persistent analysis process. |
| CLI-first, usable through machine APIs | **AGREE, MODIFY** | The "machine API" should be an in-process service layer with typed requests and results, which MCP, LSP and CLI call. Today MCP forks the CLI per call (C18). Measured startup (~7 ms) makes forking acceptable for small programs, but it duplicates contracts. |
| Fundamentals first | **AGREE, MODIFY** | Add three things Codex did not list: **WASM support for the documented constructs** (`range`, user higher-order functions; §6.2), **JSON *escaping*** (a program cannot even emit correct JSON for arbitrary strings today), and **catchable scope refusals** (DL0904 aborts the program; `match … Err` never runs). |
| Target capability matrix | **AGREE, strongly** | Generate it from `prim.rs`, `codegen.rs::is_compilable` and the guest channel's surface list, not by hand. |
| Versioned outcomes | **AGREE** | Defects F2–F5 (§6.3) are all outcome-contract defects. |
| Preserve Authority and the Guard | **AGREE** | Also against OpenShell (§8). |
| Simplify the CLI and semantic duplication | **AGREE on the CLI; MODIFY on semantics** | Do it narrowly: one shared *name-resolution and callable-compilability table*, consumed by checker, interpreter and codegen. Not a new IR. The recorded defects (LATE-LET-1, LOCAL-CALLEE-1, CTOR-SHADOW-1, INDEX-TARGET-1) were all lookup and walk divergences. |
| Hold the broad launch until A01–A11 and A18 | **AGREE, MODIFY** | Add A19 (WASM parity for documented constructs) and A20 (no success reported for actions that did not happen; F1). |
| Order: installation → tasks → matrix → workflows → performance → specialists | **MODIFY** | Run installation identity (small) **in parallel** with language basics. The basics are the long pole and the owner's own complaint. |
| Keep the working name; clear a final one | **No position** | Outside this decision; Codex's legal statements are background, not clearance. |

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
| P0 | **WASM cannot compile documented constructs** | `for i in range(0,4)` → DL1201; `fn apply(f: fn(Int)->Int, x: Int)` called with **any** function (named or lambda) → DL1201 *"a call to `apply` (not a compilable function)"*. Cause: `codegen.rs:1027 is_compilable` requires every parameter to have a `wasm_ty`, and function types have none | codegen |
| P0 | `test` succeeds when nothing ran | §3 C13 | CLI |
| P1 | No math, no checked arithmetic | `sqrt(4.0)` → DL0301 | prelude |
| P1 | Strings: no interpolation, search or indexing | `"hello {name}"` printed literally | stdlib, then syntax |
| P1 | No methods or interfaces on user types | `p.norm()` → DL0405 | language |
| P1 | Scope refusals are not catchable | `match wr.write_text("../x", …) { … Err(e) => … }` → **DL0904 aborts the program**; the `Err` arm never runs | runtime semantics (decide deliberately) |
| P2 | `+=`, bitwise, tuples, sets, `sort_by` | `n += 2` → DL0202 | syntax and stdlib |

The earlier claims are **true gaps, not wrong API guesses**. I checked each against the parser and primitive table.
The interpolation case is the most dangerous: it type-checks and prints the wrong text.

### 6.3 Defects found in this session beyond Codex's list (OBSERVED HERE unless marked)

| ID (mine) | Defect | Witness | Severity |
|---|---|---|---|
| **F1 NULL-DEVICE-OK** | With no `--broker-profile`, `arm.command(...)` inside the envelope returns `Ok` (`COMMANDED`, exit 0) though **no device exists**; the run report says nothing about devices. The source states it (`delulu-runtime/src/device.rs` (lines 720–727): *"With no broker (the null device profile) an in-envelope command answers `Ok(())`"*), contradicting the CLI's own principle at `cli.rs:9476` (*"a … run that commanded nothing while reporting success would be a lie"*) | `device_sim.delulu` with actuator and sensor grants, no profile → `COMMANDED`, then `NO READING` | **Medium** for physical-AI honesty (nothing moves, but an agent concludes it actuated) |
| **F2 HINT-INVALID** | DL0703 for an actuator tells the user to pass `--grant "actuator=review/motor:DIM=LO..HI"`; following it gives exit 2 *"no `heartbeat_ms` — a device grant with no dead-man is not a grant"* | `device_missing.delulu` | Low–medium (breaks an agent repair loop) |
| **F3 FIX-OUTCOME** | `delulu fix` exits **0** with `fix.errors_after: 1` (widening repair skipped), and after a successful fix `summary.errors` still says **1** (`errors_after: 0`), against for-agents.md's "1 = diagnostics" | `fix app.delulu --json` without and with `--accept-widening` | Low–medium |
| **F4 TTL-IS-A-TIMESTAMP** | `grants delegate --json` prints `"ttl_millis": 1793836483844`, an absolute epoch time, not a TTL; `Denial::Expired { ttl_millis: claims.exp_millis }` repeats it (`delulu-broker/src/lease.rs`) | broker run | Low (a misread contract) |
| **F5 IDENTITY-NULL** | Every non-packaged build reports `commit: null`; doctor cannot compare PATH vs tree (RW 6.53) | `--version --json` | Medium for adoption |
| **F6 LEASE-BURNED** | A single-use lease is consumed by a run that faults on an unrelated missing grant (console), so the agent must re-mint | broker run → DL1407 on retry | Low (by design, but surprising) |
| **F7 WATCHDOG-POISON** *(hypothesis, CURRENT SOURCE, not reproduced)* | The device dead-man thread takes `inner.leases.lock().unwrap()` every tick (`device.rs:543`; 18 such unwraps in production device code). The release profile is `panic = unwind` (no `[profile]` override), and actors carry actuator envelopes onto other threads. A panic while holding `leases`, `sim`, `adapter` or `journal` would poison the mutex, and the watchdog would then die silently instead of engaging the fail-state | — | Medium *if* reachable; fix cheaply: poison-tolerant locks, and make a poisoned lock **engage** the fail-state |
| **F8 STALE-EDIT-AS-USAGE** | A stale-hash edit refusal is exit **2** ("usage") and returns the current hash, inviting a blind retry | `edit --expect-hash 0000` | Low (contract clarity) |
| **F9 SURVEY-GRANULARITY** | §4.3 | — | Medium for maintainers |
| **F10 CHECK-MEMORY** | `check` uses about 14 KiB of RSS per source line: 224 MiB for a 16,002-line program, 315 MiB to compile it to WASM | §10.2 | Medium for edge and long LSP sessions |
| **F11 STALE-OPENSHELL-PIN** | `openshell.yml` pins v0.1.2; upstream is v0.1.7 with breaking changes in 0.1.5–0.1.7; the live runtime witness last ran 2026-10-04 | CURRENT CI, OFFICIAL EXTERNAL | Medium (integration drift) |

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

---

## 7. CLI-first versus CLI-only, and the compiler and editor question

| Option | Verdict |
|---|---|
| Remove compiler internals; keep a terminal wrapper | **Reject**: the wrapper would have nothing to wrap. |
| Separate "compiler" product | **There is none.** Do not create one. |
| **CLI-only** (agents compose shell strings, parse terminal text) | **Reject.** Agents already need `--json`, MCP and LSP. Shell-only invites unsafe string building and makes outcome semantics (C16, F3) worse. |
| **CLI-first + one machine API + thin editor** | **Adopt.** Concretely: (a) split `cli.rs` into services (`check`, `authority`, `edit`, `run`, `test`, `grants`) with serde request and result types; (b) MCP and LSP call the services in-process instead of forking `delulu` (`mcp.rs::run_self`); (c) the CLI becomes a parser and printer over the same calls. |
| Persistent analysis service? | **Only where measured.** A small check costs about 7 ms per fresh process (p95 9 ms), so a daemon buys nothing for utilities. A 16k-line program costs 580 ms and 224 MiB per check, and *there* the LSP process already is the persistent service. **Do not add a new daemon.** |
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

| Criterion | A | **B** | C | D |
|---|---|---|---|---|
| Keeps native Windows and macOS | yes | **yes** | yes (L1 kept off-Linux) | no |
| Offline, single binary | yes | **yes** (OpenShell optional) | partly | no |
| Extra control plane | none | **optional** | required on Linux servers | required |
| Upstream break exposure | none | **contained in an adapter** | high | very high |
| Maintainer burden | high | **medium** (adapter plus pin CI) | medium | low code, high integration |
| Fleet and Kubernetes story | none | **via OpenShell** | via OpenShell | via OpenShell |
| Evidence today | full | **prover green on v0.1.2; runtime last green 2026-10-04** | none | none |

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
| Same source, same results, interpreter ↔ WASM | **partial**: of 17 valid everyday programs, 15 behave identically (12 outputs, 3 identical faults) and 2 are refused at compile time on WASM (`range`, user higher-order functions); devices, plugins and actors with `Str` fields are refused too | §6.2 |
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
| **Wasmtime and WASI** | A maintained runtime (DeluluLang pins 48.0.8, a line still patched on 2026-10-29); the component model and WASI p2/p3 | **Integrate, don't rebuild**: research a WASI-component `.dwx` profile for host portability (§13 phase 3). Do not write a new VM. |
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
| **Remove from claims** | "total … across all code" wording in README's first line; "production ready" status box for a language that cannot read argv | — | — | trust |

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
7. WASM: `range` and higher-order user functions.
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
| A11 limits | **passes** (wall, memory); cancellation and descendants not tested here |
| A12 plugins | **passes** (allowed, overbroad, tampered, sandbox refusal) |
| A13 two principals | **passes as custody** (same OS user) |
| A14 durable workflow | **not built** (in-memory only) |
| A15 offline package | **passes** (build used only `cargo fetch --locked` sources) |
| A16 simulation | **passes with a profile; F1 without one** |
| A17 performance record | **done here** (§10) |
| A18 evidence and claims | **partial** (Survey fresh; probe strings stale) |

### 15.2 Added

- **A19:** every construct `docs/for-agents.md` and the Book teach compiles on WASM, or `check --target wasm` refuses it.
- **A20:** no success value for an action that did not happen (null device, `test` with zero matches, `fix` with errors left).
- **A21:** a poisoned device-broker lock engages the fail-state (F7).

### 15.3 The study that would settle "why a new language?"

Twenty fixed tasks (JSON and file utilities, two-module packages, delegation, cancellation, plugin use), two independent
agent clients, DeluluLang vs Python **inside an equivalent sandbox** (OpenShell or gVisor with an equivalent policy).
Measure: success against an independent oracle, unsafe attempts blocked, over-broad permission requests, repair
iterations, tool calls and tokens. Run it **after** Phase 1. Before Phase 1, DeluluLang would lose on fundamentals,
and the study would measure that instead of the authority model.

---

## 16. Unresolved questions and honest unknowns

- Windows and macOS behaviour of every new finding (F1–F11): source-level, so expected identical, **not observed**.
- The laptop's installed binary and extension (C15): **blocked** here.
- OpenShell runtime behaviour at v0.1.3–v0.1.7, its TLS interception of granted HTTPS, and its overhead: **unknown**
  until §8.5 runs.
- F7 reachability (a panic while a device lock is held): **hypothesis**.
- Zerolang's real task-completion advantage: **unknown**; nothing independent was found.
- Koka's exact current version; MoonBit's version (secondary source only).
- Memory behaviour of a long LSP session over many edits: **not measured**.
- ARM edge hardware, constrained devices, real-time behaviour: **not measured**.

- **Scenarios and boundary tests the prompt listed that were not run here:** a durable workflow with restart, replay and
  idempotency (not built; A14); vehicle and drone geofence or comms-loss and satellite out-of-order telemetry beyond
  Codex's `simulation.delulu` (reproduced, logic only); loss of the broker or supervisor mid-run and broker crash recovery;
  replayed or duplicated effect requests; Python and FFI process spawning; two separate OS accounts. Each needs the
  setup named in §15 or §8.5.

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
| A cheap, fast analysis service beats spawning the CLI | **Rejected for small programs** (7 ms p95 9 ms per process); **supported for large ones** (580 ms, 224 MiB at 16k lines), where the LSP already is that service | §7, §10.2 |
| Interop over rebuilding AI frameworks | **Supported**: keep Python, FFI and plugins as adapters; do not build GPU or robotics stacks | §11.2 |
| Security and UX are linked | **Supported**: F2 (a hint that fails), F6 (a burnt lease) and DL0904 aborting instead of returning `Err` all push an agent toward broader grants | §6.3 |
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

The everyday corpus `e01`–`e18` used one construct each (listed in §6.1). `e18` used an effect name (`Random`) that does not
exist (DL0306). That is **my own error**, not counted as a gap. Scaling programs (§10.2): `module scale.big`, then `fn f0(x)`
and `fn f{i}(x: Int) -> Int { let r = Rec { a: x, b: "s" } if r.a > 3 { f{i-1}(r.a - 1) } else { r.a * 2 } }` repeated *n*
times, with `main` printing `f{n-1}(10)`.

Fixtures: `input/task.txt` = `value=10\nvalue=11\n`; `input/child/safe.txt` = `child-ok`; `forbidden.txt` =
`sentinel-unchanged` (unchanged at the end); `outside/s.txt` = `secret-outside` (unchanged).

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
