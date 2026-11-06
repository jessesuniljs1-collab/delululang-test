---
name: windows-verification-2026-11-05
description: "Laptop re-ran the cloud reviews' rows on Windows (release build of 1414a58, commit injected) 2026-11-05 eve — 25 rows reproduce; RW 3.29 does NOT on the Windows release build; 6.70 is worse (shipped skins can be shadowed from cwd); small corrections to 6.54, 4.96; lsp ignores its args; the uncalled-rule sweep found nothing new. NOT yet recorded in the repo."
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-11-05T15:44:50.784Z
---

Jesse: *"Verify and learn more"* (2026-11-05 evening IST, after the sync to `1414a58`). Binary: `cargo build --release -p delulu`
with `DELULU_BUILD_COMMIT=1414a58…` (commit shows in `--version`), plus the 11-05 02:43 debug build; code last changed at
`04db8fb`, so both are current. Scratch: `scratchpad/verify/` (fixtures), `scratchpad/tools/` (`lsp_probe.py`,
`mcp_probe.py`, `uncalled_rules.py`). **Nothing committed** — the cloud's rows still say "Windows not observed"; a records
PR adding these Windows re-runs is the obvious next step if Jesse wants it.

**Reproduced on Windows, as recorded:** 4.95 (R-Get: `! {Load, Read}` host wrote `out/written-by-plugin.txt`),
4.97 (`verify-sig --kye` → valid exit 0; `sign --bogus`, `keygen --bogus` accepted), 3.28 (3 `main` shapes build exit 0,
run DL0904), 3.25 (range / non-generic HOF / if-arm DL1201; generic runs), 6.73, 6.75 (sandboxed DL0902 only on stderr,
report `diagnostics: []`), 6.77 (here *"Access is denied (os error 5)"*, exit 2 kind usage after the write), 6.58 (panic
*"The pipe is being closed (os error 232)"*, exit 2), 6.60 (budget stop, 0 stdout bytes), 6.51, 6.78 (`authority
--isolation bogus` exit 0; `test -v` exit 0), 6.79, 6.72 (LSP: 2 false DL0301 on greeter), 6.81 (rename `shout_count` →
`fn`/`let`/`Int` accepted; lower-case `content-length` → exit 1, empty stderr), 6.74 (cancelled check answered in full,
ping waited 163 ms), 6.76 (`check` isError false, `authority`/`atlas` true; non-UTF-8 line ends server), 6.69 (Net edit
written, `widened: null`, package DL1009; correct import edit refused DL0301; bogus prefix accepted), 6.80 (CRLF file →
7 CRLF + 2 bare LF; `pub pub fn` written, DL0208), 4.88 (null profile `COMMANDED`, report silent), 6.55, 2.42, 2.37,
4.96, 6.71, 6.50, 6.57, 6.53 (PATH delulu still 2026-08-07, extension 1.0.0).

**Different on Windows:** RW 3.29 — release build: DL0905 under `hostile-agent` and every `--sandbox --limits mem` down
to 256 MiB (no abort; Job limits count committed memory, the 781 MiB stack is only reserved). Debug build: guest
*"overflowed its stack"*, report `stopped_by: memory` (job peak accounting). So the abort is Linux's address-space cap
(and debug frame size), not Windows release. 6.75 still holds (DL0905 only on stderr).

**Corrections / extensions to rows:**
- **6.70 is wrong that "no skin ships":** `scripts/package-toolchain.sh` puts `morphs/compact-ai.toml` and
  `zh-CN-keywords.toml` beside `bin/`; `./morphs` is searched first, so a cwd file SHADOWS a shipped skin. Witness: a
  `compact-ai` program, cwd skin swapping `if`↔`while` aliases (legal under DL1710/1711) → same bytes print `tick` ×1 vs
  ×3. No surface (`check`, `authority`, run report) names the skin file used. Also searched: `~/.delulu/morphs` (HOME,
  not `DELULU_HOME`), never the source file's own directory.
- **6.54:** the hint isn't wholly fixed — `{d}` is the program's device; the `DIM=LO..HI` shape omits mandatory
  `heartbeat_ms` (prim.rs:752).
- **4.96 detail:** with nonzero `limits`, `Plugin[Contained]` is refused earlier with text naming *"a Verified plugin
  executes its DIR on the interpreter"*; zero limits → `BadArtifact … host requested Plugin[verified]`.
- **New, LOW:** `delulu lsp` ignores all arguments (`run_lsp(_args)`); `mcp` refuses them. A fix should accept
  `--stdio` (LSP clients append it). Sweep of all 43 commands with `--zzbogus`: only keygen/sign/verify-sig/lsp pass.
- **R-Get code read:** the checker's `pending_gets` settles each `get`'s `F` but acts only for Contained (R-6a); Verified
  `F` is dropped. `verified_adapter.rs` ~105 comment ("R-Get, as `p.get` applies it") is false today.
- **Uncalled-rule sweep** (security-named fns with no non-test caller, test modules brace-stripped): only test-only
  accessors, `resolve_in_scope` (production uses `resolve_in_anchor`, RW 4.72), `with_authority_watch` (production uses
  `with_adapter` with the probe), and the unbuilt Contained runtime (`run_contained_export`, `kill_on_limit`,
  `r_get_contained` — 4.96's cause; `interp.rs:1851` says it "arrives with the WASM execution path"). Limit: name-based.

**Traps hit:** Python `open()` on Windows defaults to cp1252 (UTF-8 em dash read as `â€”`) — pass `encoding='utf-8'`;
a Python edit inside a bash heredoc halved `\\n` again ([[python-edit-eats-backslash]]); MCP tools take `paths` (a
list), not `path`; `load` performs `Read`; the Console effect is `Write`.
Related: [[cloud-reviews-2026-11-05]], [[a-rule-is-enforced-where-it-is-called]], [[witness-a-denial-on-the-far-side]].
