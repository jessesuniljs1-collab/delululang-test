# Cross-platform verification & production readiness

**Status:** Living record. First cut 2026-07-21 as part of the post-1.0 production-readiness pass
(`STAGE10_BUILD_ORDER.md` ruling **D19**). This document says what was actually run, on what, and
what remains unproven — in the project's usual voice: a gap named here is worth more than a green
badge that never executed.

## 1. Why this exists, and the honest starting point

`.github/workflows/ci.yml` declares a three-OS matrix (`ubuntu-latest`, `macos-latest`,
`windows-latest`) that, in its own words, "activates automatically once this repository is pushed to
GitHub." **This repository is never pushed** (owner policy), so that matrix **has never executed.**
Everything below is a *local* replay of those same gates, which is the only cross-platform evidence
that actually exists for this project.

The gates replayed (from `ci.yml`): `cargo test --workspace`; `delulu fmt --check examples`;
`delulu fmt --check docs/book/samples`; `cargo check -p delulu --no-default-features` (the
Python-less build); plus this project's own `delulu-conform --coverage` (invariant 42) and
`delulu-conform --check-reference` (the hard Book-vs-source gate). `clippy` is tracked as a quality
baseline; it is **not** a CI gate (there is no `-D warnings` anywhere in the tree).

Toolchain is pinned by `rust-toolchain.toml` to **rustc 1.96.1** on every platform.

## 2. Per-platform results (2026-07-21)

| Gate | Windows (native, x86_64-pc-windows-msvc) | Linux (WSL Ubuntu-20.04, kernel 6.6 WSL2) | macOS |
|---|---|---|---|
| `cargo build --workspace` | ✅ | ✅ | not run |
| `cargo test --workspace` | ✅ all binaries `ok` (satellite 4/4) | ✅ (satellite 4/4 ×2, release 6/6; full suite green) | not run |
| `clippy --workspace --all-targets` | ✅ 65 warnings / 0 errors | ✅ 66 / 0 (zero `state_hash`) | not run |
| `fmt --check examples` | ✅ 0 would change (7) | ✅ 0 (7) | not run |
| `fmt --check docs/book/samples` | ✅ 0 would change (10) | ✅ 0 (10) | not run |
| `check --no-default-features` (Python-less) | ✅ | ✅ | not run |
| `conform --coverage` | ✅ 307/307 = 100% | ✅ 307/307 | (arch-independent) |
| `conform --check-reference` (hard gate) | ✅ 24 chapters in sync | ✅ | (arch-independent) |

Runs were done **sequentially, in isolation.** An earlier attempt ran Windows and a from-scratch
WSL build *concurrently*; the shared machine starved the timing-sensitive tests and both suites
flaked (different tests on each OS). That is a property of the reference machine, not the code — but
it is the reason CI runs one OS per runner, and the reason these numbers were re-taken uncontended.

The one-warning Windows/Linux `clippy` delta (65 vs 66) is a single platform-specific lint on the
Linux side, unrelated to any gate; it is tracked, not blocking.

## 3. Why it ports — the load-bearing design facts

- **Wire format is endianness-independent.** Every serialized integer uses `to_le_bytes` /
  `from_le_bytes` explicitly (DIR container, audit chain, lease tokens, `--json` envelopes), so a
  `.dwx` is byte-identical regardless of host CPU architecture, not merely "works on x86."
- **`fmt` canonical output is now byte-identical across platforms.** The formatter emits `\n`; the
  new `.gitattributes` (`* text=auto eol=lf`, D19d) makes LF an enforced repo invariant rather than
  an accident of each contributor's `core.autocrlf`, closing the one latent path by which a CRLF
  could reach the lexer's raw block-comment slice and leak into "canonical" output.
- **The broker transport is per-OS by construction, same guarantee.** Windows: a named pipe with an
  owner-only DACL plus peer-SID verification. Unix: a domain socket in a `0700` user-owned directory.
  Both reduce to "OS-authenticated same user," never multi-tenant auth. The Windows-only path hash
  is now `#[cfg(windows)]` (D19a) so the Unix build carries no dead symbol.

## 4. Platforms and isolation modes — what is claimed, and what refuses honestly

Only **Linux, macOS, and Windows** are claimed host platforms. Beyond the big three:

- **`--isolation microvm`** is **Linux x86_64/aarch64 + KVM only.** Everywhere else it *refuses by
  name* — "requires Linux x86_64/aarch64 with KVM; this platform is `<os>`" — and **no code path
  quietly substitutes a weaker isolation** (`cli.rs`, trap 8). A capability that cannot be honored is
  denied, not silently downgraded.
- **Contained-WASM resource limits** (memory/wall-clock caps) ride wasmtime; the plugin/limits
  engine keeps a deliberately separate `Config` with Windows host-safety settings (10l / 6f.2b).
- **WASM / WASI** is a *guest* sandbox target (`--target wasm`), not a host OS.
- **Android / the BSDs** appear in the tree only as *syscall contrast* in comments, never as claimed
  targets. **Bare-metal / MCU** execution is RFC-deferred (`THREADED_WASM_DEFERRAL.md` and the
  §2.4 deferrals). None of these are implied to work; the absence is stated where a reader would
  otherwise assume.

## 5. The macOS caveat, stated plainly

**macOS has had zero live executions, ever** — no Apple hardware or VM is available to this project.
Its standing rests on two things and no more: (1) the runtime's Unix half is `#[cfg(unix)]`, and that
*same code* compiles and passes the full suite on Linux, which exercises the socket transport, the
`0700` directory guard, and the interpreter; (2) static reading of the macOS-relevant `cfg` branches.
What is **not** proven: any macOS-specific native path, and the default-on `pyo3`/CPython embedding as
it links against a macOS Python. The **Python-less build** (`--no-default-features`, a first-class,
CI-gated configuration) is the macOS-safe path until a real Mac run exists. macOS is "engineered for,
analyzed, and unproven" — not "supported" in the sense the other two now are.

## 6. What this pass fixed (D19) and what it did not

Fixed, verified on Windows + Linux: a Linux/macOS-only dead-code warning (`state_hash` gating); three
supply-chain-honesty defects in `SBOM-1.0.json` (a mis-named `wasm-encoder`, two omitted PQC crates)
and the fool's-gold test that missed them; the committed `Cargo.lock` (discharging D14d, reconciling
two docs that already claimed it); line-ending determinism (`.gitattributes`); and — the real one — a
**criterion-10 acceptance test that only passed on fast (release) builds.** `cargo test` builds
debug, where one `fib`-bearing cycle of the satellite demo cost ~230 ms against a 200 ms dead-man
heartbeat, so the watchdog correctly revoked a compute-stalled controller via the exact mechanism the
test forbids. The close-out's "1098-0-4 / criterion 10 MET" was a fast-build snapshot; the raise-
heartbeat fix (D19e) makes it robust in both build modes. Full rationale: `STAGE10_BUILD_ORDER.md`
D19a–e.

**Named, not fixed (future work):**

- **The sim watchdog wall-clock coupling — RESOLVED (D20).** `--broker-profile sim` was
  deterministic in its device readback (seeded) but its lease *timing* ran on real milliseconds, so
  demo LOS timing was machine-dependent. `--sim-step <ms>` now advances the lease clock by simulated
  time per device interaction, so the satellite demo replays byte-identically across debug and
  release (the tests still assert the *pattern*, not a cycle count). The wall-clock dead-man remains
  the default and the only real-time guarantee; stepping is a determinism tool for demonstrations,
  refused on any non-sim profile.
- **The device scope dimension — RESOLVED (D21, RFC 0001 F1), re-verified on both platforms
  2026-07-22.** `delulu_broker::Scopes` had no device dimension, so a delegation could say "you may
  actuate" but not "you may fly this corridor only" (D12e). It now carries one, with an interval
  lattice. Re-verified sequentially and isolated: **Windows** 89 suites / 0 failed, clippy 65/0 (the
  exact pre-F1 baseline — ~600 new lines, zero new warnings), coverage 100%, reference in sync, fmt
  0-change, python-less clean; **Linux** (WSL, ext4, isolated `CARGO_TARGET_DIR`) 89 suites / 0
  failed, clippy 66/0, every gate exit 0. **macOS unchanged and still not run**: the new module
  (`device_scope.rs`) is pure `std` with no `#[cfg]` and no OS call, so the code both platforms run
  green *is* the macOS path — an argument, not an execution, and not counted as one.
- **`broker_unreachable_is_dl1401_fast`** asserts a 10 s wall-clock "fail fast" budget; it passes in
  isolation but can flake under pathological concurrent load (a starved scheduler, not a hang). A
  monotonic-deadline assertion less sensitive to scheduling would harden it.
- **`pyo3` is a default-on, all-OS build dependency** with no bundling story for a shipped binary.
  The Python-less build is the mitigation today; a real deployment story (bundle vs. system CPython)
  is unwritten.
- **macOS needs one real run** before it can be called supported.

## 7. Reproducing

```
# Windows (native), from a Git-Bash / MSYS shell:
cargo test --workspace && \
cargo run -q -p delulu -- fmt --check examples && \
cargo run -q -p delulu -- fmt --check docs/book/samples && \
cargo check -p delulu --no-default-features && \
cargo run -q -p delulu-conform -- --coverage && \
cargo run -q -p delulu-conform -- --check-reference

# Linux, on ext4 with an isolated target dir (do NOT share the Windows target/):
#   run the same six gates. Run OS suites one at a time — concurrent builds
#   starve the timing-sensitive dead-man tests.
```
