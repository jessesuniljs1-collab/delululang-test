---
name: delulu-production-readiness-2026-08-09
description: "Overnight autonomous production-readiness sweep (2026-08-09, Opus 4.8): phases A-Q, CAMPAIGN CLOSED, all committed (chain e5ae9ec->b203983). SIX real security defects found + fixed + FOUR evidenced negatives, each witnessed against old code. Fifth+sixth are twin compiler-frontend stack-overflow-on-untrusted-source bugs: PATTERN-DEPTH-1 (d25ee5c, DL0212, parse_pattern) and BLOCK-DEPTH-1 (7dbd63b, DL0213, while/for statement blocks via parse_block) -- both had NO recursion-depth guard where expressions (DL0210) and types (DL0211) do; both fixed with a 128-level guard + a linear brace-balanced skip-recovery (the naive guard turned the crash into a quadratic hang). The four recursive-descent nesting classes (expr/type/pattern/block) are now ALL bounded. Earlier four: (1) hardware-adapter reply-framing desync -> fail-closed (ffca9bd); (2) broker rotate-key did not persist the new key, so a restart resurrected all 'invalidated' tokens -> now persists fail-closed (0676183, ROTATE-1); (3) ADOPT-REPLAY-1 (6d3e9cf) -- a federation certificate revocation lived only in daemon memory, so a restart let a REVOKED cert be re-adopted into a fresh live node; now the revoked-cert denylist is persisted to revoked_certs.json + reloaded at startup (corrupt file poisons adoptions, fail-closed); (4) ADAPTER-LINE-1 (6d908f7) -- the D23 untrusted-adapter reader used BufReader::lines() with NO length bound, so a hostile driver streaming a reply with no newline could OOM the host (timeout bounds latency, not memory); fixed with a 64 KiB per-line cap (take+read_until) mirroring the broker IPC MAX_FRAME. Miri 0 UB on the interpretable unsafe. macOS designed-for but unverified. The 2-min one-shot-cron + nonce continuation protocol worked all night."
metadata: 
  node_type: memory
  type: project
  originSessionId: c99ba5c5-c39b-4a1f-b406-92fbf2c601b0
  modified: 2026-08-09T11:03:05.245Z
---

**START HERE for the 2026-08-09 overnight production-readiness campaign. Jesse's standing order:
"make delulang absolutely production ready, free of security vulnerabilities, deployable"; verify
Windows + Linux, macOS honestly; test CLI/compiler/VS Code/LSP; run Miri in tiny batches; use +
regenerate the Survey and keep .md docs current; 2-min pause then auto-continue between phases; NEVER
push; harden-never-redefine Authority/Guard.** The full evidence log is
`docs/design/PRODUCTION_READINESS_2026-08-09.md` (read it first). Commit chain: `e5ae9ec` → `6fd4e1c`.

## The 8 phases (all committed, Survey 0/0 + doctor_cli 7/0 every time)
- **A** `e5ae9ec` — full-workspace baseline. `cargo test --workspace` (not run since DL1421 shipped)
  exposed 2 `delulu-conform` failures the targeted suites missed: DL1421 had no conformance witness
  (99.7%) and the generated reference had drifted. Added both DL1421 polarity witnesses to
  `tests/conformance/witnesses.toml` (it is a broker diagnostic no `.delulu` can produce, so both are
  explicit Rust-test witnesses) + regenerated `docs/reference/`. Back to 328/328 100%.
- **B** `d66e600` — LSP server + VS Code extension. Drove `delulu lsp` LIVE over stdio JSON-RPC
  (initialize/hover/DL0407 diagnostics/executeCommand all work); the P18/P19 command-name collision
  (`delulu.showAuthority` vs server `delulu.authority`) cannot recur, confirmed live. Extension node
  tests 10/10 + verify-package OK.
- **C** `46be87e` — CLI + compiler dogfood, byte-identical on both platforms. `new`/`check`/`run
  --grant`/`build` + examples all clean; authority enforced (DL0703 without a grant); broken program →
  DL0407 not a panic.
- **D** `9af4611` — deployability. The portable `--no-default-features` archive (the real distributable;
  `cargo install` from crates.io is intentionally impossible) builds in 3m36s, unpacks + runs as a pure
  downloader; Python-less degrades to "unavailable" (DL1307 as a value); SHA256SUMS 20/20.
- **E** `ffca9bd` — SECURITY DISCOVERY, the one real finding. The D23 hardware-adapter line protocol
  (`crates/delulu-runtime/src/adapter.rs`) assumed one reply line per command but did not enforce it;
  an untrusted adapter emitting an EXTRA line left it buffered, and the NEXT command read that stale
  line as its reply (off-by-one reply desync, silent, no poison) — the half-open hazard rule 3 forbids,
  via an extra line rather than a late one. Fix: any unsolicited buffered output before a request
  poisons the adapter (fail closed). Witnessed to fail against old code (`first=Ok second=Ok
  poisoned=false`). **NOT a containment break** — the envelope is still checked host-side before
  dispatch (`device.rs:388`, verified). See [[delulu-hw-adapter]].
- **F** `6a55380` — Miri, tiny batches (Jesse's rule). 0 UB on the interpretable unsafe: value.rs
  secret-zeroing `write_volatile` (3 tests) + prim determinism/containment (8 tests). Honestly stated:
  the bulk of the `unsafe` is FFI/syscall — `broker_transport.rs` (~30) + `foreign.rs` pyo3 (~11) —
  which Miri cannot execute, so it does not validate it (defended by tests + isolation + OS boundary).
- **G** `8f9e6c1` — macOS honest. No Mac on this bench. cfg audit: macOS is deliberately designed-for
  (microvm Linux-only → DL1408 elsewhere; `PR_SET_PDEATHSIG` `cfg(target_os="linux")` with a portable
  Drop-guard fallback and libc as a Linux-only dep — a bare `cfg(unix)` would break macOS;
  SUN_PATH_MAX 104; macOS test branches). `cargo check --target aarch64-apple-darwin` compiles the Rust
  deps and stops at a C build script (no darwin `cc`) — the honest blocker is the C toolchain, not the
  Rust. Status: designed-for but UNVERIFIED (never built/run on hardware).
- **H** `6fd4e1c` — finale. Doc-consistency (Survey 0/0, no drift). Final `cargo test --workspace`
  GREEN on Windows AND Linux: **124 test binaries `ok` each, zero failures.**
- **I (bonus)** `0676183` — continued discovery beyond the A–H sweep, and it found a SECOND real
  defect. **ROTATE-1**: `delulu broker rotate-key` rotated the lease-MAC key only in daemon MEMORY and
  never rewrote `broker.key` on disk (witnessed: byte-identical hash before/after). Since `serve_inner`
  reloads the key from disk at startup, a RESTART reloaded the old key and re-validated every token the
  rotation "invalidated" — a revocation-bypass-via-restart. Fix: persist the new key to `broker.key`
  (0600) BEFORE applying it in memory (fail-closed; `Broker::rotate_key` returns the key, `rotate_key_to`
  applies a persisted one). broker.key now CHANGES after rotate on both platforms; test
  `rotate_key_is_persisted_so_a_restart_cannot_resurrect_old_tokens`; delulu-broker 144/0 both.

## Multi-agent discovery round → ADOPT-REPLAY-1 (`6d3e9cf`, the THIRD defect)
After the A–I sweep I ran Haiku/Sonnet agents (per Jesse's "use multiple agents, tell them
exactly what to do") to hunt the ROTATE-1 bug class — "what other security decision lives only in
daemon memory?" Every agent claim was re-verified firsthand against the current code, then live.
- **Clean negatives (verified, no finding):** the guard/lease/cert surfaces; device-scope envelope
  (`within`/`all_within`, `is_finite` NaN-reject, inclusive-bounds parse); the ordinary grant tree
  itself — it is intentionally EPHEMERAL (`Broker::new()` = empty `nodes`; `serve_inner` loads only
  key/audit/secrets/guard/root-policy, never the tree), which fails a bearer token CLOSED on restart
  (its node is gone), and grants+revocations are wiped together so no revoke-resurrection asymmetry.
- **The real one (Sonnet a805cffe found it, I confirmed + witnessed + fixed):** a federation
  CERTIFICATE is NOT a tree reference — it is a self-contained signed artifact that re-verifies
  against the anchor on its own. The in-memory `revoked_adoption_fps` denylist (the D22 "revocation
  can't be undone by replay" memory) was cleared on restart, so a REVOKED cert re-adopted into a
  fresh LIVE node holding the killed authority. ROTATE-1 shape, one level up. The code comment
  "a restart clears both together" was TRUE for tokens, FALSE for certs.
- **Fix (surgical, fail-closed):** persist ONLY the denylist (the one state whose loss weakens the
  broker) to `revoked_certs.json` (atomic temp+rename), reload in `serve_inner`. Do NOT persist
  `adopted` — a never-revoked cert SHOULD re-adopt after a wiped tree (the intended recovery path).
  Corrupt denylist → `poison_adoptions()` refuses every adopt (daemon keeps serving
  revoke/inspect/e-stop/local-issue). New Broker methods: `revoked_adoption_fps_snapshot`,
  `restore_revoked_adoption_fps`, `poison_adoptions`. Witnessed live in `federation_cli.rs`
  (adopt→revoke→restart→re-adopt: SUCCEEDED on old code `g_dfed49ef…`, REFUSED after).
- **Reach, honest:** `Adopt` is over the owner-only IPC transport and there is no auto-adopt on
  startup, so the re-presenter is an operator or same-uid (category 7) — but legitimate post-restart
  re-adoption is a documented expected workflow, so a revoked cert riding along and silently undoing
  a revocation is a real safety defect, not only an attacker story. See [[delulu-federation-scope]].

## Phase L — untrusted-input robustness → ADAPTER-LINE-1 (`6d908f7`, the FOURTH defect)
After the multi-agent rounds (custody LOGIC) I pivoted to resource BOUNDS on the surfaces that take
bytes from OUTSIDE the same-uid trust domain, where a panic/unbounded-alloc is a real vuln not a
category-7 footgun. Two surfaces: the broker/foreign-worker IPC frame reader is already defensive
(16 MiB `MAX_FRAME` checked before allocation, `broker_ipc.rs:302` — clean); the **D23 adapter
subprocess** (operator-supplied, explicitly untrusted) was NOT. Its reader thread used
`BufReader::lines()`, whose `read_line` grows a String with no bound — so a hostile adapter streaming
a reply with no newline OOMs the host in a detached thread (the module's rule 2 timeout bounds
LATENCY, not MEMORY; the child isn't killed until end-of-run). Witnessed: a 256 KiB single line was
swallowed WHOLE as `Refused("xxx…")`. Fixed with `(&mut reader).take(64 KiB).read_until(b'\n', …)` —
over-cap-without-newline tears the reader down → exchange fails closed (Closed+poison); `lines()`
UTF-8/newline semantics preserved exactly (all 10 prior adapter tests green). delulu-runtime lib
163/0; D23 end-to-end `hw_adapter_cli` 11/11. **Two of the four defects are on the D23 adapter — the
untrusted-driver surface is the sharpest edge in the tree, as its own module docs anticipate.** See
[[delulu-hw-adapter]].

## Phase M — federation parsers vs. adversarial input (NEGATIVE, `1b703d3`)
Continued the untrusted-input theme onto the PARSERS (remote-origin bytes via `grants adopt`/`renew`/
`audit reconcile`). Hunted a panic/hang/unbounded-alloc INSIDE the 16 MiB frame bound; found none.
`cert::parse`/`parse_receipt`: every field → Denial, integer parses reject overflow (not a wrapping
cast), `from_hex` uses `str::get` (None not index-panic), serde_json's 128-deep recursion limit turns
a nesting bomb into a parse error. `verify_chain`: only index guarded by is_empty; adopt TTL math is
`saturating_add`+`min` (u64::MAX uplink clamps to a past deadline, fail-closed). `parse_bundle` grows a
Vec line-by-line, trusts NO declared count (no alloc-from-count trap); `Bundle::verify` guards is_empty
before `records[0]`; reconcile reports errors as INCIDENTs. Proven 3 ways: code reading, two
fuzzing-style unit batteries (left as regression guards), and end-to-end CLI (`audit reconcile` on 5
hostile bundles → INCIDENTs exit 1, zero panics; 600-deep → "recursion limit exceeded at column 128").
**Lesson: the point of a discovery pass is to attack a surface and, when it HOLDS, prove it and leave
guards behind — a negative with evidence is a real result.** Tally of REAL defects stays at FOUR.

## Phase O — compiler frontend fuzzing → PATTERN-DEPTH-1 (the FIFTH defect, DL0212, `d25ee5c`)
The frontend eats UNTRUSTED source (LLM/agent emitting DeluluLang, cloned repo). Prior red-team passes
capped EXPRESSION nesting (DL0210, P17-F5) and TYPE nesting (DL0211, P20-R3) at 128 — explicitly to
stay stack-safe on 1–2 MiB LSP/tooling threads. Skip-branch rule paid off: those guards exist but miss
the one path they don't cover — **`parse_pattern` recurses on its own** (`Some(Some(…))`, a variant
pattern's fields are patterns), touching NEITHER counter. Witnessed: a 50,000-deep pattern parsed in
full (→ checker DL0401) where the identical EXPRESSION depth is refused DL0210 at 128; deeper
(~500k+) CRASHED `delulu check` with no diagnostic. Fix = TWO parts: (1) `MAX_PATTERN_DEPTH=128` +
`pat_depth` guard → DL0212; (2) **linear skip-recovery** consuming the over-deep remainder in one
paren-balanced pass — WITHOUT it the fix only turned the crash into a QUADRATIC HANG (50k hung >60s;
caught by a post-fix timing sweep, not a functional test — **a fix aimed at a crash can introduce a
hang; always re-measure timing, not just correctness**). Post-fix sweep 200/10k/50k/800k all → clean
DL0212 ≤2s. DL0212 registered in codes.rs (both tables) + `reject/DL0212_deep_pattern.delulu`
auto-witness + witnesses.toml (100% coverage held) + parser tests + reference regen. **OPERATIONAL
LESSON: lingering `delulu.exe` from hung/backgrounded fuzz runs held the binary lock → `cargo build`
failed "Access is denied" → a STALE binary silently ran the next sweep (26s anomaly). Kill stray
`delulu.exe` (taskkill /F /IM) + TaskStop the fuzz loop before rebuilding.** delulu-syntax 235+128/0.

## Phase Q — lexer + config parsers hold (NEGATIVE, campaign CLOSED, `b203983`)
The final planned surface: bytes a CLONED REPO supplies — source into the lexer, delulu.toml/lockfile/
plugin-manifest into the config parsers. Holds. Lexer is an ITERATIVE scan (no recursive-descent
overflow like the parser); nested-comment depth is a u32 bounded by input; literal overflow DL0104,
unterminated DL0105. Config parsers all go through the mature `toml` crate (its recursion limit makes a
nesting bomb an Err, not a crash) + Option extraction; `parse_semver` is unwrap_or-safe;
`check_plugin_module`'s lone `expect("fn in table has a checked type")` is GUARDED at the caller
(cli.rs runs it only `if errors == 0` — exactly the invariant it asserts; skip-branch rule confirmed
it before assuming a panic). Witnessed clean (no panic/hang) on: 2M-digit int, 500k-nested comment,
unterminated 1M string, 2M-char identifier, invalid UTF-8, garbage TOML, 100k-nested TOML bomb,
duplicate keys, 2M-char value, non-UTF8 manifest. Regression guard:
`adversarial_config_inputs_are_errors_never_panics`. **CAMPAIGN CLOSED: six real defects, four
evidenced negatives, frontend recursion fully guarded (DL0210/0211/0212/0213), untrusted-input surfaces
(driver/peer/source/config) all fixed-or-proven. Honest residuals unchanged: macOS UNVERIFIED, same-uid
category-7, no in-tree hardware driver.**

## Phase P — the last unguarded frontend recursion → BLOCK-DEPTH-1 (SIXTH defect, DL0213, `7dbd63b`)
Phase O guarded patterns; Phase P asked what else recurses. A block-EXPRESSION (`{…}` as a value) is
bounded by DL0210 (routes through parse_unary), but a `while`/`for` body is a STATEMENT block:
`parse_block → parse_stmt → (while) → parse_block` recurses touching no expr counter. Witnessed:
`while true { while true {…} }` nested 200,000 deep CRASHED `delulu check` (exit 127, no diagnostic);
20,000 checked clean; a nested `if` (expression) is DL0210. Fixed by guarding `parse_block` itself (the
choke point every block passes) with `block_depth` → DL0213 + the same linear brace-balanced
skip-recovery as DL0212. Confirmation sweep: nested for→DL0213, nested list literals→DL0210, nested
lambdas→DL0213 — **all four recursive-descent classes (expr DL0210 / type DL0211 / pattern DL0212 /
block DL0213) now bounded at 128 with a diagnostic instead of a stack overflow.** Full DL0213
registration (codes.rs both tables + reject/DL0213_deep_blocks.delulu + witnesses.toml + parser tests +
reference regen + 100% coverage). A CLOSING REPORT section was added to the doc log (six defects, three
negatives, honest residuals). **Frontend recursion hardening is COMPLETE; the next fresh surface for a
future session is the lexer + delulu-check config/manifest/lockfile/plugin parsers, or the runtime.**

## Phase N — dev-facing + cross-platform re-verify (CLEAN, no code, `d240d3a`)
Confirmed this session's broker/adapter changes didn't regress the dev surfaces, each driven LIVE (a
green suite is NOT proof — P19's LSP regression hid behind one because the EXTENSION, not the protocol
handlers, was broken). LSP live via `lsp_cli` against the real `delulu lsp` binary: **34/0** (full
protocol). VS Code extension: `node --test` **15/0** (CSP + `resolveServer` wiring) + `verify-package`
OK (vsix, modules resolve, 4 commands declared==registered); `extension.js` starts the client with
`{command: serverPath, args:["lsp"]}` and keeps `delulu.showAuthority`(VS Code)≠`delulu.authority`
(server executeCommand) — the P18 collision that killed the server, guarded + documented. macOS:
cross-compile stops at **blake3's C build script** (`cc` not found) — a transitive dep, not DeluluLang
Rust; same blocker as Phase G; my pure-Rust-std changes don't touch the macOS surface so posture is
unchanged (designed-for, UNVERIFIED — not overclaimed). Core-regression (Jesse's rule): the diff
touches ONLY broker/adapter, zero compiler-core files; live `check` clean, `run` enforces DL0703, a
broken program is DL0202 not a panic. **Lesson reinforced: drive dev-facing surfaces LIVE — the P19
regression proved a green Rust suite can coexist with a dead extension.**

## Verdict + residuals (honest)
DeluluLang builds/checks/runs/packages/deploys/installs cleanly on Windows + Linux; LSP + CLI work;
both new defects (adapter reply-framing, rotate-key persistence) are fixed; unsafe is Miri-clean or
honestly out of reach; macOS is designed-for and truthfully unverified. Residuals carried forward (not new, documented where they live): macOS
unverified; same-OS-user isolation needs a separate account ([[delulu-p21-crossaccount]],
[[delulu-root-issuance-bypass]], [[delulu-ipc-deadman-findings]]); default-features install embeds
CPython; FFI unsafe outside Miri.

## The continuation protocol worked (keep using it)
Per [[delulu-continuation-protocol]]: each phase re-armed a ONE-SHOT `CronCreate(recurring:false)` ~2
min out with a fresh nonce written to `continuation.json` (sibling of `memory/`), the same nonce in
the cron prompt; on fire I verified nonce + `consumed` + no newer owner message, set `consumed`, then
worked. It survived Jesse's usage-limit gap (session paused ~3 h, resumed by re-sending) with no
runaway. Long background runs (release build, Miri, full suites) each got a deletable watchdog cron.
WSL harness lessons from [[delulu-p21-crossaccount]] all held (PowerShell not Git-Bash; script files;
cargo belongs to `user`).
