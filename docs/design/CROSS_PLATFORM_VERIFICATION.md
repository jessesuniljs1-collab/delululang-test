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

### Re-verified 2026-07-26 (rulings D53–D56)

| Gate | Windows | Linux (WSL Ubuntu-20.04) |
|---|---|---|
| `cargo test --workspace` | ✅ **98 suites / 1,326 passed / 0 failed / 4 ignored** | ✅ **98 / 1,330 / 0 / 4** |
| `clippy --workspace --all-targets` | ✅ **65** / 0 errors | ✅ **66** / 0 errors |
| `conform --coverage` | ✅ 100% | (arch-independent) |
| `conform --check-reference` | ✅ 24 chapters in sync | (arch-independent) |
| macOS | **never run** — see §8 | |

Both baselines held across the float-literal rule, the new `parse_float` prelude function, the
four-package corpus tier, the adapter signer pin, the line-ending gate, artifact review (D59), the
bounded authority report and the Book correspondence gate (D60).

**Interoperability was exercised on Windows for the first time as part of this pass**, and is recorded
here because it is a platform-specific claim: DeluluLang called into the real Windows C runtime
(`msvcrt.dll` and `ucrtbase.dll` both giving `cos(0.0)=1.0`, `sqrt(144.0)=12.0`, `pow(2.0,10.0)=1024.0`)
and into real embedded CPython (`statistics.pstdev` = 2.0, a `base64` round trip). The C path on macOS —
`libm.dylib` — remains the one FFI arm that has never compiled or run anywhere (§8). The 4-test Linux surplus is the platform-specific
set Windows skips, and matches the historical delta.

**A reproduction trap, recorded because it cost a run.** Building for Linux *in the Windows working
tree* (`/mnt/d/...`) fails in `libffi-sys`'s `configure`, which cannot write its own `config.log` on
the DrvFs mount — it panics with "Configuring libffi" and no useful cause. The fix is to give the
Linux build its own target directory on the Linux filesystem:

```
cd /mnt/d/nelan/DeluluLang && CARGO_TARGET_DIR=$HOME/delulu-target cargo test --workspace
```

This is a WSL/DrvFs property, not a portability defect: a Linux build in a Linux checkout is
unaffected. It is here because the alternative is the next person concluding the tree does not build
on Linux.

**And a method warning about the measurement itself.** The first Linux count came back "70 suites /
1,042 passed" against Windows' 96 — not a platform difference but `tail -70` in the counting pipeline,
silently discarding 26 suites. It was caught only because the totals disagreed with Windows. This is
the third time in this campaign that a truncating pipe has produced a confident wrong number
(`head -3` hid two panics in P13). **Count first, truncate never.**

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

**The one macOS-specific hazard the static audit could put a number on (D68).** Every
conditional-compilation site in the tree was enumerated during the production-readiness review and
all are **exhaustive for macOS** — `cfg(unix)`/`cfg(not(unix))` and `cfg(windows)`/`cfg(not(windows))`
pairs cover it, and the two Linux-only mechanisms are handled deliberately (`PR_SET_PDEATHSIG` is
`cfg(target_os = "linux")` with the macOS substitute named in a comment; the microVM profile refuses
with `DL1408` on the not-Linux arm). That is evidence of care, not evidence of working.

What the audit *could* quantify is the Unix domain socket path limit: **`sun_path` is 104 bytes on
macOS against 108 on Linux**, and macOS hands out temp directories like
`/var/folders/j7/8k3l…0000gn/T/` — roughly fifty characters before a caller names anything. The
longest broker state directory the test suite builds leaves on the order of a dozen bytes of margin
there while being comfortable here. The path is now checked before `bind` and refused with both
figures and the remedy, so if it ever fires on a Mac it fires legibly instead of as
`ENAMETOOLONG`. **The mechanism is tested on Linux; the macOS constant is reasoned. No Mac has run
it, and the two claims are not the same.**

**Named, not fixed (future work):**

- **The sim watchdog wall-clock coupling — RESOLVED (D20).** `--broker-profile sim` was
  deterministic in its device readback (seeded) but its lease *timing* ran on real milliseconds, so
  demo LOS timing was machine-dependent. `--sim-step <ms>` now advances the lease clock by simulated
  time per device interaction, so the satellite demo replays byte-identically across debug and
  release (the tests still assert the *pattern*, not a cycle count). The wall-clock dead-man remains
  the default and the only real-time guarantee; stepping is a determinism tool for demonstrations,
  refused on any non-sim profile.
- **The first hardware adapter — SHIPPED (D23), verified on both platforms 2026-07-22.**
  `Profile::Hw` no longer has nothing behind it: an operator-supplied subprocess speaks a line
  protocol over stdio. **Portability note that matters here more than elsewhere**: the adapter uses
  only `std::process` and `std::sync::mpsc` — no `#[cfg]`, no platform API — but its *tests* need a
  scriptable shell, so they use PowerShell on Windows and `sh` elsewhere. Both ship with the OS and
  both take the script as one argument, so nothing depends on shell quoting surviving `Command`.
  That is the only place in this project where a test forks by platform, and it is the tests
  forking, not the code. **This is still not hardware**: no driver for any real device ships
  in-tree, and running the adapter against a shell script proves the socket works, not that anything
  physical moved.
- **Broker federation — RESOLVED (D22, RFC 0001 F2–F6), verified on both platforms 2026-07-22.**
  `STAGE10_AUTONOMY_ADDENDUM.md` §2.5 named it "a prerequisite for any real deployment in this
  addendum's domains". A grant tree now spans machines: two brokers, two ed25519 identities, no
  shared secret, and a credential that crosses as a **file** — the broker still opens no socket, so
  `broker_transport.rs`'s same-user guarantee is untouched because nothing was added to that
  transport. Verified sequentially and isolated: **Windows** 91 suites / 0 failed, 0 build warnings,
  clippy 65/0 (the exact pre-federation baseline across ~2,600 added lines), coverage 100%,
  reference in sync, fmt 0-change, python-less clean; **Linux** (WSL, ext4, isolated
  `CARGO_TARGET_DIR`) 90 suites / 0 failed, clippy 66/0, every gate exit 0. **macOS unchanged and
  still not run**: `cert.rs` and `device_scope.rs` carry no `#[cfg]` and no OS call, so the code
  both platforms run green *is* the macOS path — an argument, not an execution.
- **The device scope dimension — RESOLVED (D21, RFC 0001 F1), re-verified on both platforms
  2026-07-22.** `delulu_broker::Scopes` had no device dimension, so a delegation could say "you may
  actuate" but not "you may fly this corridor only" (D12e). It now carries one, with an interval
  lattice. Re-verified sequentially and isolated: **Windows** 89 suites / 0 failed, clippy 65/0 (the
  exact pre-F1 baseline — ~600 new lines, zero new warnings), coverage 100%, reference in sync, fmt
  0-change, python-less clean; **Linux** (WSL, ext4, isolated `CARGO_TARGET_DIR`) 89 suites / 0
  failed, clippy 66/0, every gate exit 0. **macOS unchanged and still not run**: the new module
  (`device_scope.rs`) is pure `std` with no `#[cfg]` and no OS call, so the code both platforms run
  green *is* the macOS path — an argument, not an execution, and not counted as one.
- **The core-invariance gate — ADDED 2026-08-02, verified on both platforms.** Everything built
  after 1.0 sits *around* the language; nothing proved it had not moved the language. The new gate
  records the exact bytes the toolchain answers with for all 108 targets the repository ships
  (`tests/core-invariance/SNAPSHOT.txt`, 360 cases). **The load-bearing cross-platform fact: the
  snapshot recorded on Windows passes byte-for-byte on Linux** — 222,762 bytes, md5
  `bdd56c6a6e40e791c394b4b4e9bc0b99`, unchanged, run against a Linux-built binary. **Windows** 109
  suites / 1,437 passed / 0 failed, clippy 65/0; **Linux** (WSL, ext4, isolated `CARGO_TARGET_DIR`)
  109 suites / 1,441 passed / 0 failed, clippy 66/0; both with coverage 100%, reference in sync, fmt
  0-change, `doctor --check` 12/12.

  One recorded file has to be valid on three platforms, so each way it could have differed was
  removed and then *checked* rather than assumed:

  | Hazard | How it is removed | What was checked |
  |---|---|---|
  | Path separators | paths are passed forward-slash and the CLI echoes back what it was given | zero separators in the file — all 15 backslashes are `\n`, `\"` or `\u{…}` inside message text |
  | Line endings | `.gitattributes` stores LF (D19d); the test normalises CRLF on both sides | zero CR bytes in the recorded file |
  | `read_dir` order — NTFS vs ext4 vs **APFS** | relative paths are byte-sorted before use | order is a property of the sort, not of the filesystem |
  | macOS **NFD vs NFC** filenames | — | no filename in the corpus contains a non-ASCII byte, so there is nothing to normalise differently |
  | Release version churn | `delulu_version` is rewritten to `<version>` | a version bump does not read as 108 semantic changes |
  | Platform-conditional output | — | zero occurrences of `windows`, `darwin`, `macos`, `linux` or `.exe` in the file |

  **macOS is still not run**, and this is not a claim that it was. `core_invariance.rs` is pure
  `std` with no `#[cfg]` and no OS call, and the surfaces it records are the same ones Linux
  executes green — so the argument is the usual one, and it is an argument. What the audit above
  *does* establish is narrower and worth stating exactly: there is no known mechanism by which this
  particular file could differ on macOS. That is a removed hazard, not a passing test.
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

---

## 8. Re-verification at the hardening campaign's close (2026-07-26, ruling D50)

The numbers in §2 were taken on 2026-07-21, before a sixteen-phase hardening campaign changed the
parser, the checker, the broker, the lockfile verifier and the runtime. They were re-taken from the
committed tree rather than assumed to have survived.

| Gate | Windows (native, x86_64-pc-windows-msvc) | Linux (WSL Ubuntu-20.04) | macOS |
|---|---|---|---|
| `cargo test --workspace` | ✅ **95 suites, 1289 passed, 0 failed** | ✅ **95 suites, 1293 passed, 0 failed** | **never run** |
| `clippy --workspace --all-targets` | ✅ **65** warnings / 0 errors | ✅ **66** / 0 | **never run** |
| `fmt --check examples` | ✅ 0 would change (13 clean) | ✅ | **never run** |
| `fmt --check docs/book/samples` | ✅ 0 would change (10 clean) | ✅ | **never run** |
| `check --no-default-features` (Python-less) | ✅ | ✅ | **never run** |
| `conform --coverage` | ✅ 100% | ✅ 100% | (arch-independent) |
| `conform --check-reference` | ✅ 24 chapters in sync | ✅ | (arch-independent) |

The clippy figures are unchanged from §2 — **65 on Windows, 66 on Linux** — across roughly 4,000 lines
added by the campaign. The four-test delta between platforms is the known one: Linux runs four tests
Windows ignores. Suite counts rose from the campaign's own regression witnesses, every one of which
was observed failing against the code it now guards.

### The front door, re-tested rather than assumed

P1 made "download, build, install, use" true. Sixteen phases later that promise was re-tested the only
way that means anything — from a **fresh `git clone` into an empty directory**, following the README's
own instructions verbatim:

```
$ cargo build --release                              # clean clone, no warm target/
$ delulu check hello.delulu                          → ok: hello.delulu checked clean
$ delulu authority hello.delulu                      → effects: Write / capabilities: Console
$ delulu run hello.delulu --grant console            → Hello, Delulu
$ delulu run hello.delulu                            → DL0703, exit 1
```

The last line is the one worth keeping: the README claims that leaving off `--grant console` fails
with DL0703 because a DeluluLang program holds zero ambient authority. It does.

### macOS — stated plainly, one more time

**macOS has never been executed. Not once, in any phase, in this campaign or before it.** There is no
Mac hardware, and no emulation was attempted or would have counted. Every macOS cell above says
"never run" rather than "untested" or "pending", because those words invite a reader to assume someone
tried. Nobody tried.

Nothing in this repository may describe DeluluLang as supported on three platforms. The CI matrix in
`.github/workflows/ci.yml` names `macos-latest` and **has never executed**, because the repository is
never pushed. A declared matrix is not evidence.

### Honesty scrub at close-out

- **Banned claims:** "quantum-proof"/"quantum-safe" appear **12 times**, every one inside a sentence
  prohibiting the term. (§2's historical note records 9 at the Stage-10 close-out; the campaign's own
  rulings added three more prohibition sentences.) No occurrence is a claim.
- **Measurements:** all nine records in `measurements/` now state **when they were taken**. Four did
  not — including the one this campaign wrote, which had itself argued that "a table should say when
  it was taken."
- **The performance clause is intact:** "competitive with C on hot paths, with safety C cannot offer …
  and only where a published benchmark shows it. Where DeluluLang loses, the table says so." Two new
  losses were published under exactly that rule this campaign (C55, C56).

### macOS readiness, which is NOT macOS verification (2026-07-26)

This is the one item of the three raised at close-out that **could not be done**, and the reason is not
a decision: there is no Mac. What was done instead is an audit of what a macOS run would encounter —
useful for whoever eventually has hardware, and worth nothing as evidence.

Every platform-conditional path in the tree was enumerated and resolved for macOS:

| Path | What macOS takes | Tested elsewhere? |
|---|---|---|
| `lease.rs`, `secrets.rs` — `cfg(unix)` / `cfg(not(unix))` | the **unix** branch | yes — Linux takes the same branch, and it is green |
| `adapter.rs` — `cfg(windows)` / `cfg(not(windows))` | the **not-windows** branch | yes — same as Linux |
| `cli.rs` microVM — `cfg(target_os = "linux")` | the **not-linux** branch: refuses with **DL1408** | yes — Windows takes the same refusal |
| `foreign_worker.rs` — `cfg(target_os = "linux")` | the not-linux branch | yes — same as Windows |
| `tests/cli.rs` FFI — three mutually exclusive arms | `cfg(target_os = "macos")`, naming `libm.dylib` | **no — this arm has never compiled or run anywhere** |

So the structure is sound: the three FFI arms are mutually exclusive and cover all three platforms, so
nothing collides, and every other conditional puts macOS on a branch Linux or Windows already exercises.
**One arm — the `libm.dylib` FFI test — is macOS-exclusive and has never been executed by anyone.**

**None of that is verification, and this section is not evidence that DeluluLang works on macOS.** It is
a reading of the source. A path that *should* work and a path that *has been run* are different claims,
and this project does not get to blur them merely because the reading was careful. Every macOS cell in
§2 and §8 still reads **never run**, and will until someone runs it on a Mac.
