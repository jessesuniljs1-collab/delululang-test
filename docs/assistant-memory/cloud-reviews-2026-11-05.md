---
name: cloud-reviews-2026-11-05
description: "The four AI reviews in docs/review/ (Codex 11-04; Claude cloud session_01Deh6W9bJdfxAeMARtXU9R9 on 11-05: strategic verification, AI-first pipeline audit, hidden-complexity audit) — verdicts, ~60 new RW rows, the fix order (RW 4.95 first), the owner decisions still open, and their traps"
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-11-05T15:23:39.436Z
---

**What landed (2026-11-05, all docs + Survey map, no code):** PRs #125 (Jesse merged, merge commit `6397a98`), #126–#129
(fast-forwarded by the cloud session under Jesse's in-session word *"u can merge, No need to wait for my approval"* —
scoped to that session, not a standing rule). Every PR and push run green; scheduled `42009076387` 18/18 incl. miri-slow.
Laptop synced to `1414a58` the same evening.

**The reviews (`docs/review/`; prompts in `docs/prompt/`, each byte for byte):**
1. Codex (GPT-6 Astra) strategic review, 11-04, on `e97c747` (#124).
2. **Claude's Independent Strategic Verification** — re-ran all 102 Codex commands on Linux (all reproduce); found no
   containment/authority escape in its first passes. Verdicts: **CLI-first** over one typed service layer (split
   `cli.rs` 11,866 lines into services shared by CLI/MCP/LSP); keep compiler, LSP, 409-line VS Code client; **OpenShell =
   OPTIONAL BACKEND** (bump pin v0.1.2→v0.1.7, RW 7.28; **freeze PS-C microVM**; PREFERRED HYBRID only after an upstream
   stability policy + 9-check PoC green on 2 releases + ≤100 ms overhead + TLS answer); **hold 2.0 behind an "ordinary
   agent utility" bar** (argv/env caps, exit outcome, JSON, math, strings/interpolation, methods, WASM parity: RW §8
   items 7–8); generated capability matrix + `check --target` (RW 6.61). Most basics touch the frozen core → 14-day RFCs
   or a recorded unfreeze.
3. **AI-First Compiler/Execution Pipeline Audit** — no program becomes a standalone binary; mandatory = the check + an
   engine bound to the host primitive table; text stays canonical (DIR 28× larger) — see [[measure-the-stage-before-removing-it]].
   Recommends `--engine auto`, an authenticated precompiled cache (off by default, rules N1–N5), Python out of the main
   binary (RW 3.26). §14 = rule-crossing suggestions B1–B8 ([[owner-may-suggest-rule-changes-2026-11-05]]).
4. **Hidden-Complexity AI-First Experience Audit** — per-command machinery is lean (≈170 syscalls, no keys/crypto on pure
   tasks, no prompt without a TTY, no Salsa needed, MCP process-per-call cheap but has a 5 ms poll); the real costs are
   wrong fast-path answers, late refusals, unrunnable artifacts, unbounded/uncancellable MCP — see [[a-fast-path-must-see-what-the-slow-path-sees]].

**New RW rows:** 2.41–2.44, 3.25–3.29, 4.88–4.97, 5.7, 6.54–6.81, 7.27, 7.28; RW 7.25 (flaky monitor test) red twice
now (macOS 10-29, Windows 11-05), narrowed to a 1.9 s broker queue.

**Fix order (newest sync-log Open/next):** (1) **RW 4.95 R-Get, HIGH** ([[a-rule-is-enforced-where-it-is-called]]) +
4.96 + **4.97** (`verify-sig --kye K` drops the key pin and says valid — signing cmds ignore unknown flags); (2) RW 6.72 +
6.69 (package-blind LSP/edit), 3.28, 3.29 (hostile-agent profile turns DL0905 into a guest abort), 6.73, 6.74; (3) 6.75,
6.77 (a run that succeeded then can't write `--report-out` exits 2 → agent repeats the effect), 6.76, 6.78–6.81; (4) DIR
4.93/4.94/5.7/6.67, 6.68; (5) answer contracts 6.50–6.52, 6.54–6.60, 4.88; (6) RW 7.28; (7) basics then RW 3.24; launch
only on Jesse's word. Also: cite launch runs `41416143488`/`41416147296` in CHECKLIST-2.0 at the next merge.

**Owner decisions still open:** hold 2.0 for the basics (asked 11-02); adopt the OpenShell verdict; catchable scope
refusals (RW 2.42); the exit-code mapping; RFCs vs recorded unfreeze; grants-before-execution default (RW 6.73);
WASM `main` shape; zero-tests exit; MCP protocol revisions (2026-07-28 is current/final); B1–B8.

**Traps they recorded (HANDOFF §11.3/§11.5):** `| head` makes `delulu` panic on EPIPE → exit 2 (RW 6.58) — capture `$?`
right after the command, never through a pipe; `} else {` must share a line (DL0202) and `cap` is reserved (DL0106);
the cloud VM's disk can't hold a debug test build (22 GB) — run the suite on CI; a broker socket path >107 bytes is
refused on Linux (short `DELULU_STATE_DIR`); Survey module-level `impact` inside `crates/delulu` is 106 for every module
(RW 6.63) — plan removals from cargo refs; `DELULU_HOME` does not hold theme/locale (those read `$HOME/.delulu`).
Related: [[release-program-progress]], [[readiness-includes-everyday-language]], [[openshell-study-2026-09-28]].
