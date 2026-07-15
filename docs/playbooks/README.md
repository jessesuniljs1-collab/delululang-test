# DeluluLang Implementation Playbooks

Each `STAGE<N>_PLAYBOOK.md` is the **execution companion** to the matching
`docs/design/STAGE<N>_SPECIFICATION.md`. The spec is normative — it states *what must be true when
the stage is done*. The playbook is operational — it states *how to build it, in what order, and
which walls to avoid*, distilled from the way Stages 1–3 were actually built.

## How to use a playbook (for the implementing model — Opus 4.8)

1. **Read the spec first, then the playbook.** The spec wins on any conflict of intent; the
   playbook wins on build order and tooling.
2. **Work phase by phase.** Each playbook decomposes the stage into small, independently
   testable phases (the way Stage 3 became 3a–3r). After every phase: build green, test green,
   update the spec's `§ Implementation status` section, and commit. One phase = one commit.
3. **Two-engine parity is the correctness contract.** Wherever a runtime behavior exists on both
   the interpreter and the WASM engine, the interpreter is the *reference*; the WASM engine must
   match byte-for-byte. Any divergence is fixed by making an engine match the *specified*
   semantics — never "whichever is convenient."
4. **Honesty clauses are binding.** Copy every spec §"Honesty and threat-model caveats" verbatim
   into `delulu explain` text and docs. Never claim "immediate", "unbreakable", "faster than C",
   or "lowest tokens". State the threat model.

## Environment (carried from the Stage 1–3 build; Windows/MSVC)

- **cargo is not on the default PATH.** Every shell: prepend
  `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"` (PowerShell).
- **Windows Defender causes transient `LNK1104: cannot open file …exe`** on freshly-linked test
  binaries. Fix: `Get-Process | Where-Object { $_.ProcessName -like "delulu*" } | Stop-Process
  -Force`, then `cargo test --workspace --no-run` to link, then `cargo test --workspace` to run.
  A second attempt almost always succeeds; it is never a code fault.
- **Commit messages with parens/quotes break PowerShell here-strings.** Write the message to a
  file and `git commit -F <file>`. End messages with the `Co-Authored-By:` trailer.
- **Never return `Err` from inside a Wasmtime host callback** — it aborts the process
  ("non-unwinding panic") on Windows. Record the refusal in host state and surface it *after* the
  call returns (the pattern `HostState.refused: Option<String>` + `finish()`). This is load-
  bearing for every capability added from Stage 3 onward.
- **A wasm pointer is an unsigned 32-bit offset.** Read it as `ptr as u32 as usize` and use
  checked arithmetic before indexing guest memory, or a hostile pointer panics the host.

## Crate map (as of Stage 3)

`delulu-diag` (codes/spans/JSON envelope) ← `delulu-syntax` (lexer/parser/AST) ←
`delulu-check` (types/rows/resolve/authority) ← `delulu-runtime` (interpreter/broker/prim) ←
`delulu` (CLI) ; plus `delulu-fuzz` (differential harness) and `delulu-wasm` (WASM backend +
Wasmtime host). Diagnostic code ranges: DL01xx–09xx (Stage 1), DL10xx/11xx (Stage 2), DL12xx
(Stage 3), DL13xx (Stage 4), DL14xx (Stage 5), DL15xx (Stage 6), DL16xx (Stage 7), DL17xx
(Stage 8), DL18xx (Stage 9), DL19xx (Stage 10).

## Index

| Playbook | Stage | Theme | Build risk |
|---|---|---|---|
| `STAGE4_PLAYBOOK.md` | 4 | Foreign — C FFI + embedded Python | **BUILT 2026-07-11** (271 tests; spec §11 close-out; 3-OS CI matrix still pending) |
| `STAGE5_PLAYBOOK.md` | 5 | Custody — broker daemon, grant tree, microVM | **BUILT 2026-07-13** (372 tests; spec §11 close-out table; criterion 8 microVM egress-deny platform-pending — Linux+KVM CI, gated `cfg(delulu_kvm)`). **Chunk 6 "the Guard" BUILT 2026-07-14** (398 tests; the dcg-inspired principal-approval layer — `docs/design/STAGE5_GUARD_ADDENDUM.md`, §7 deviations + §8 close-out) |
| `STAGE6_PLAYBOOK.md` | 6 | Live — runtime plugins, DIR, `.dpx` | High (re-verification, resource limits) |
| `STAGE7_PLAYBOOK.md` | 7 | Concurrent — actors, reference-caps, async | High (scheduler, revocation-under-concurrency) |
| `STAGE8_PLAYBOOK.md` | 8 | Surface — LSP, fmt, test runner, localization | Medium (breadth, editor protocol). **Early drop BUILT 2026-07-15** (owner's order): the Palette (role-based CLI color, `delulu-diag/palette.rs`) + the Atlas (`delulu-atlas` crate, `delulu atlas` — typed deterministic code+authority graph, atlas/1, digest + query verbs, dot/mermaid/self-contained html, custody overlay) — `docs/design/SURFACE_ATLAS_PALETTE_ADDENDUM.md` §7 deviations + §8 close-out; DL1780/DL1781/DL1790 |
| `STAGE9_PLAYBOOK.md` | 9 | Delulu — v1.0 freeze, coverage law, governance | Medium (process + measurement rigor) |
| `STAGE10_PLAYBOOK.md` | 10 | Industrial — JIT policy, robotics Actuate, LTS | High (real-time, safety envelope) |
