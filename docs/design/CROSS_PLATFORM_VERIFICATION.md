# Cross-platform verification & production readiness

**Status:** Living record. First cut 2026-07-21 as part of the post-1.0 production-readiness pass
(`STAGE10_BUILD_ORDER.md` ruling **D19**). This document says what was actually run, on what, and
what remains unproven — in the project's usual voice: a gap named here is worth more than a green
badge that never executed.

## 1. Why this exists, and the honest starting point

`.github/workflows/ci.yml` declares a three-OS matrix (`ubuntu-latest`, `macos-latest`,
`windows-latest`) that, in its own words, "activates automatically once this repository is pushed to
GitHub." **Until 2026-09-14 this repository was never pushed** (owner policy), so that matrix **never
executed.** On 2026-09-14 the owner had it pushed to a **private testing remote** (`HANDOFF.md`
§1.1), which activated the workflow, and its first run is transcribed in **§9** — failures first. The fifth
run, the same day, was the first with no failures: all three operating systems green in one run.
Everything from here to §9 predates that run and is kept as the record of what was known before it:
a *local* replay of the same gates, which until then was the only cross-platform evidence this
project had.

The gates replayed (from `ci.yml`): `cargo test --workspace`; `delulu fmt --check examples`;
`delulu fmt --check docs/book/samples`; `cargo check -p delulu --no-default-features` (the
Python-less build); plus this project's own `delulu-conform --coverage` (invariant 42) and
`delulu-conform --check-reference` (the hard Book-vs-source gate). When this section was first
written, `clippy` was a tracked baseline rather than a gate. It is a gate now: `ci.yml`'s `lints` job
runs it with `-D warnings`.

**The lint baseline is per-platform, and the third number has never been seen.** Windows and Linux
**no longer differ**: cold and findings-only they are **14 on both** as of 2026-08-03. The one-warning
gap that had held since the old 65/66 figures was a single Linux-only lint on a `let job = ();`
placeholder — the non-Windows arm of a Windows-only Job Object handle — and removing the binding
closed it. (§2 explains why every earlier count in this document was measured by a method that
inflated it.) macOS would still be a *third* count: it
takes the `unix` branches Linux takes, but excludes the Linux-only ones (the microVM module,
`PR_SET_PDEATHSIG`) and the Windows ones. Nobody has run it, so nobody knows it. Quoting "65/66" as
though it were the whole story would repeat, in miniature, the mistake this document exists to
prevent. **None of the findings on either known platform is a `clippy::correctness` lint** — they are
style and complexity suggestions, which is why the count is watched for *movement* rather than driven
to zero.

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

### Re-verified 2026-08-03 (production-readiness pass), with the platform delta NAMED

| Gate | Windows | Linux (WSL Ubuntu-20.04) |
|---|---|---|
| `cargo test --workspace` | ✅ **115 suites / 1,500 passed / 0 failed / 4 ignored** | ✅ **115 / 1,506 / 0 / 4** |
| `clippy --workspace --all-targets` (cold, findings only) | ✅ **14** / 0 errors (was **34**) | ✅ **14** / 0 errors (was **35**) |
| CLI + compiler sweep (21 cases, exit-status assertions) | ✅ **21/21** | ✅ **21/21** |
| Packaged archive, run outside the workspace | ✅ **8/8**, 0 problems | ✅ **8/8**, 0 problems |
| `conform --coverage` | ✅ 100% | ✅ 100% |
| `conform --check-reference` | ✅ 24 chapters in sync | ✅ 24 chapters in sync |
| `fmt --check examples` | ✅ 0 would change, 13 clean | ✅ 0 would change, 13 clean |
| `doctor --check` | ✅ 12/12 | ✅ 12/12 |
| macOS | **never run** — see §5 and §8 | |

**Every clippy number in the older dated tables was measured by a method that inflated it.** Two
separate faults, found on 2026-08-03 when the same tree reported 26 and then 42 within the hour:

1. **The counter matched cargo's summary lines.** `grep -cE '^warning:|^error:'` also counts
   ``warning: `delulu-wasm` (lib) generated 1 warning`` — one such line per crate per target, 13–18 of
   them. They are totals, not findings, so every historical figure includes them.
2. **A warm `cargo clippy` under-reports.** Cargo does not re-emit warnings for units it did not
   re-lint, so the count depends on what happened to be cached — which makes a bare number
   unreproducible and not comparable between runs.

A clippy count is therefore only meaningful **cold, in an isolated target dir, with summary lines
excluded**:

```
CARGO_TARGET_DIR=<throwaway> cargo clippy --workspace --all-targets 2>&1 \
  | grep -E '^warning:|^error:' | grep -vE 'generated [0-9]+ warning' | wc -l
```

The dated figures earlier in this document and in `STAGE10_BUILD_ORDER.md` are **left as they were
recorded** — they are the honest output of the method used at the time, and rewriting them would hide
the mistake rather than fix it. Read them as "the old measure", not as findings. The before/after in
the table above (34→14 Windows, 35→15 Linux) was taken cold on one machine, at `0c98a58` and at the
working tree, so it is internally comparable even though it is not comparable to the older numbers.

**The 6-test delta, by name.** Earlier revisions of this document called it "four tests Windows
skips". Both halves of that had gone stale, so it was re-derived the only way that settles it —
`cargo test --workspace -- --list` on each platform, sorted under `LC_ALL=C`, and diffed:

| Linux-only (8) | |
|---|---|
| `broker_transport::imp::tests::a_socket_path_the_kernel_cannot_hold_is_refused_by_name` | Unix-domain sockets |
| `broker_transport::imp::tests::an_ordinary_state_directory_is_accepted` | Unix-domain sockets |
| `limits::tests::live_engine::a_well_behaved_module_returns_its_result_untouched` | real wasmtime engine |
| `limits::tests::live_engine::criterion5_a_bug_trap_under_generous_limits_is_not_a_limit_through_the_real_engine` | real wasmtime engine |
| `limits::tests::live_engine::criterion5_infinite_loop_dies_at_fuel_and_the_host_survives` | real wasmtime engine |
| `limits::tests::live_engine::criterion5_infinite_loop_dies_at_wall_when_fuel_is_generous` | real wasmtime engine |
| `limits::tests::live_engine::criterion5_memory_bomb_dies_at_mem_mb_and_the_host_survives` | real wasmtime engine |
| `tests::a_verified_plugin_runs_on_the_wasm_engine_under_the_contained_limits` | real wasmtime engine |

| Windows-only (2) | |
|---|---|
| `limits::tests::windows_refuses_contained_execution_rather_than_risk_a_fastfail` | asserts the refusal |
| `tests::a_verified_plugin_on_wasm_inherits_the_windows_enforcement_refusal` | asserts the refusal |

8 − 2 = 6, which is the whole of it. (`interp::on_interpreter_thread`'s doctest appears in both
listings under different path separators — `/` versus `\` — and is not part of the delta.)

**"Windows skips" was the misleading word.** Windows does not silently omit contained WASM
execution: the fastfail-risking path is not compiled there at all, and Windows compiles **its own
two tests that assert the refusal happens**. The platform difference is a decision with witnesses on
both sides of it, not a coverage hole on one. Anyone comparing raw suite totals across platforms
should expect Linux to be ahead by exactly this set, and should re-derive it with `--list` rather
than trusting a number in prose — including this one.

### The CLI and the compiler, driven by hand on both platforms

The suite proves the code; it does not prove *the program a person actually types*. So the shipped
binary was driven directly on each platform with 21 cases that assert **exit status**, because a
compiler that prints `error` and exits 0 is broken and a refusal that exits 0 is a security defect:

| | Windows | Linux |
|---|---|---|
| CLI + compiler sweep (21 cases) | ✅ **21 pass / 0 problems** | ✅ **21 pass / 0 problems** |

The results are identical on both, case for case. What it covers: `check` accepting a well-formed
program and rejecting a **parse error**, a **type error**, and an **undeclared effect** (exit 1 each);
`authority` reporting the effect row; `fmt --check`; `run` **refused with `DL0703` without the grant**
and succeeding with it; `--version`, `--help`; an unknown subcommand, an unknown flag, and a flag
**missing its value** all refused (exit 2); `new` creating a package and **refusing the reserved
device name `con` on Linux as well as Windows** — so a package authored on Linux cannot become
un-checkoutable on Windows; `run` on a package directory; `doctor --check`; and `--json` still
exiting 1 on a refusal while emitting an object on stdout.

Two of the sweep's own expectations were wrong before the product was: it demanded exit 1 where
§STABILITY.md specifies **2 for a usage error**, and it treated a second positional to `check` as an
extra argument when `check` accepts **many files by design**. Both were the sweep's defect. This is
the campaign's recurring shape — in the first feature-health run, sweep bugs outnumbered product
bugs 6:1 — and it is the reason a sweep's own failures get read before they get reported.

Both baselines held across the float-literal rule, the new `parse_float` prelude function, the
four-package corpus tier, the adapter signer pin, the line-ending gate, artifact review (D59), the
bounded authority report and the Book correspondence gate (D60).

**Interoperability was exercised on Windows for the first time as part of this pass**, and is recorded
here because it is a platform-specific claim: DeluluLang called into the real Windows C runtime
(`msvcrt.dll` and `ucrtbase.dll` both giving `cos(0.0)=1.0`, `sqrt(144.0)=12.0`, `pow(2.0,10.0)=1024.0`)
and into real embedded CPython (`statistics.pstdev` = 2.0, a `base64` round trip). The C path on macOS —
`libm.dylib` — remains the one FFI arm that has never compiled or run anywhere (§8). The Linux test
surplus is the platform-specific set; it is **named**, not estimated, in the 2026-08-03 subsection
below, and it grows as the campaign adds platform-specific witnesses.

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


### Re-verified 2026-08-03, second pass — the P16 adversarial fixes (rulings D78–D88)

P16 changed the **core runtime's filesystem enforcement**, which is the single most
platform-dependent thing in this codebase: it now calls `std::fs::canonicalize`, whose behaviour
differs across all three target platforms. So this pass is not a formality.

| Gate | Windows (native) | Linux (WSL) | macOS |
|---|---|---|---|
| `cargo test --workspace` | **115 suites, 1512 passed, 0 failed** | **115 suites, 1518 passed, 0 failed** | **never executed** |
| Conformance coverage | 100% | 100% | — |
| Generated reference in sync | yes (24 chapters) | yes (24 chapters) | — |
| `delulu fmt --check examples` | 0 would change, 13 clean | 0 would change, 13 clean | — |
| `delulu doctor --check` | 12/12 | 12/12 | — |
| clippy (findings, summary lines excluded) | **14** (cold) | **14** | — |
| CLI + compiler sweep | **21/21, 0 problems** | **21/21, 0 problems** | — |

The 6-test difference is the same one [named test-by-test above](#re-verified-2026-08-03-production-readiness-pass-with-the-platform-delta-named); it did not move.

### Re-verified 2026-08-10 (containment + deployment hardening campaign)

The figures above are left exactly as recorded — this is a living record, and a later run appends
rather than rewrites. `doctor` reports **15** checks now, not 12, because this pass added the
three-check `security posture` section; the older `12/12` rows are correct for the day they were taken.

| Gate | Windows (native) | Linux (WSL) | macOS |
|---|---|---|---|
| `cargo test --workspace` | **124 binaries, 1641 passed, 0 failed** | **124 binaries, 1650 passed, 0 failed** | **never executed** |
| clippy `--workspace --all-targets` | clean | **0 findings** | — |
| `delulu doctor --check` | all checks pass, exit 0 | all checks pass, exit 0 | — |
| Compiler — every shipped example | all check clean | all check clean | — |
| **LSP server, live protocol** | initialize / capabilities / publishDiagnostics with real `DL` codes / hover / shutdown | identical | — |
| VS Code extension | **17/17**, `.vsix` verifies | node absent in this WSL image | — |
| `fmt --check examples` / `docs/book/samples` | **0 would change** (13 clean / 10 clean) | — | — |
| macOS cross-check (`cargo check --target aarch64-apple-darwin`) | — | — | `delulu-diag`, `delulu-syntax`, `delulu-measure`, `delulu-survey` type-check; the rest blocked by third-party C build scripts — see the cross-check section above |

The **LSP was verified by speaking the protocol to the real binary**, not by trusting the suite. P19's
finding was that the whole suite was green while the extension had no working language server; an
in-process test cannot see that, and this is the gate that can.

#### The C84 fix, exercised with each platform's own link type

A lexical path check cannot see a link, so the fix asks the filesystem. Each platform has a
different link and a different `canonicalize`, so each was attacked with its own:

| | Windows | Linux |
|---|---|---|
| Link used | directory **junction** (`New-Item -ItemType Junction`) | real POSIX **symlink** (`ln -s`) |
| Read through it, granted the parent | `DL0904` — refused | `DL0904` — refused |
| Control: legitimate in-scope read | `READ OK: public` | `READ OK: public` |

Both were observed **escaping** before the fix and refused after, with the control passing in both
states — a refusal that also broke ordinary reads would not be a fix.

#### The case-sensitivity trap this fix could have sprung, and did not

`canonicalize` returns the filesystem's *real* casing. On a case-insensitive filesystem — Windows
**and macOS**, but not typical Linux — canonicalizing `./DATA` yields `./data`, so a naive
replacement of the lexical test would have silently **widened** scope: a grant of `./data` would
have started matching a mint of `./DATA`, reversing the deliberate fail-closed choice recorded in
`delulu-broker/src/path.rs`.

It does not, because the fix **adds** the filesystem check with `&&` rather than replacing the
lexical one. A conjunction can only ever narrow. Verified on Windows, on a genuinely
case-insensitive volume: grant `./data`, mint `./DATA` → **`DL0703`, exit 1**, unchanged.

This is the one place where a macOS run would be checking something Linux cannot, and it is worth
saying which way the risk points: **macOS shares the case-insensitivity that Windows has**, so the
Windows result above is direct evidence for the same code path — the same conjunction, the same
`canonicalize` semantics for casing. That is an argument, not an execution, and it is recorded here
as an argument.

#### macOS, for this pass specifically

Nothing changed about the standing position: **no Apple hardware exists for this project and macOS
has never been executed, not once, in any phase.** What can honestly be said about the P16 changes
in particular:

- The new code uses only `std::fs::canonicalize`, `Path::starts_with`, `Path::parent` and
  `Path::file_name` — all in `std`, all with defined behaviour on macOS, none conditionally compiled.
- macOS resolves symlinks in `canonicalize` exactly as Linux does (both are POSIX `realpath`
  semantics); the Linux result above is the direct evidence for that half.
- macOS's case-insensitivity matches Windows's, and the Windows result above is the direct evidence
  for that half.
- Therefore each half of the behaviour has been executed on a platform that shares it — but **no
  platform has executed both halves in the combination macOS presents**, and that combination is
  exactly where a surprise would live.

That is the most that can be claimed. It is not "verified on macOS", and this document will not say
it is.

### macOS cross-check, 2026-08-10 — measured instead of asserted

The standing position above is unchanged: **macOS has still never been executed.** But "blocked by a
C toolchain" had been recorded without anyone measuring *where* it is blocked, so this pass measured
it. Both Apple targets are installed; `cargo check --target aarch64-apple-darwin` was run per crate:

| Result | Crates |
|---|---|
| **Type-checks for macOS** | `delulu-diag`, `delulu-syntax`, `delulu-measure`, `delulu-survey` |
| **Blocked** | every remaining member — each by a **third-party C build script**: `blake3`, `zstd-sys`, `libffi-sys` |

Two things follow, and neither is "macOS works":

1. **No DeluluLang source has been shown to fail for macOS.** Every blocker fires in a dependency's
   `build.rs` before the compiler reaches this project's own code, so the blocker is an absent Apple
   cross-toolchain on this bench, not a portability defect in delulu.
2. **Nor has most of it been shown to compile for macOS**, because the check never got that far. Four
   workspace members are what was actually demonstrated, and that is what is claimed.

The residual macOS-specific surface is small and identifiable: exactly one line branches on the OS
(`broker_transport.rs`, `SUN_PATH_MAX = 104` on macOS versus 108 elsewhere — a real macOS-aware
detail, not an oversight), and everything else that differs is `cfg(unix)`, which Linux exercises.

**What would close it:** `cargo test --workspace` on any Mac. That is one command, and it is the only
missing evidence — this document will keep saying "unverified" until someone runs it.

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

**Linux and Windows are the platforms this project has evidence for.** macOS is *targeted* — the code
is written for it and its `cfg` branches were audited — but it has **never been executed**, so it is
not on the same footing and §5 states that at length. "The big three" below names the three the
codebase is written against, not three it has run on. Beyond them:

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

> **2026-09-14: superseded.** CI's macOS runner went green — the whole suite and every gate, in run 3
> (§9). What follows is what was known before that, kept as the record of it.

**Before 2026-09-14, macOS had zero live executions of DeluluLang code** (§9 has what changed).
Its standing rests on two things and no more: (1) the runtime's Unix half is `#[cfg(unix)]`, and that
*same code* compiles and passes the full suite on Linux, which exercises the socket transport, the
`0700` directory guard, and the interpreter; (2) static reading of the macOS-relevant `cfg` branches.
What is **not** proven: any macOS-specific native path, and the default-on `pyo3`/CPython embedding as
it links against a macOS Python. The **Python-less build** (`--no-default-features`, a first-class,
CI-gated configuration) is the macOS-safe path until a real Mac run exists. macOS is "engineered for,
analyzed, and unproven" — not "supported" in the sense the other two now are.

### First actual macOS *compilation* evidence (2026-08-04)

Until this date, the macOS position rested entirely on reading code. It now rests partly on the
compiler, which is a real if modest upgrade. Both Apple targets were installed and the pure-Rust
core (`delulu-syntax`, `delulu-diag`, `delulu-check`, `delulu-broker`) was type-checked against them:

| target | result |
|---|---|
| `x86_64-apple-darwin` | **compiles clean** — 0 errors |
| `aarch64-apple-darwin` | **could not be checked from this host** — see below |

#### Re-verified and widened, 2026-08-06

The 2026-08-04 result was reproduced from scratch rather than carried forward, and extended from the
four core crates to **every crate in the workspace that has no C dependency** — seven of them:

| crate | `x86_64-apple-darwin` | `aarch64-apple-darwin` |
|---|---|---|
| `delulu-diag` | **clean** | **clean** |
| `delulu-syntax` | **clean** | **clean** |
| `delulu-survey` | **clean** | **clean** |
| `delulu-check` | **clean** | blocked — `blake3` build script |
| `delulu-broker` | **clean** | blocked — `blake3` build script |
| `delulu-atlas` | **clean** | blocked — `blake3` build script |
| `delulu-conform` | **clean** | blocked — `blake3` build script |

Two corrections to the entry above, both in the direction of *more* being known:

- **Apple Silicon is not entirely uncheckable from this host.** Three crates type-check clean for
  `aarch64-apple-darwin`. The 2026-08-04 pass checked only crates that happen to depend on `blake3`,
  so a per-crate failure was read as a whole-target failure.
- **Every aarch64 failure has the same single cause**, verified by reading each one rather than
  assuming they matched: `failed to run custom build command for blake3 v1.8.5` →
  `failed to find tool "cc"`. It is one missing ARM C cross-compiler, not four problems.

**The full workspace still cannot be cross-checked for either Apple target**, and the blocker there
is different — `libffi-sys`, an unconditional dependency of `delulu-runtime`, selects its **MSVC**
build path from the *host* rather than the target and dies on `Could not locate cl.exe`. That is a
third-party build-script limitation; it says nothing about DeluluLang's macOS correctness, and it
cannot be worked around from Windows.

**Still zero macOS executions.** Type-checking is not running. Nothing below §5's standing position
changes.

The Apple Silicon result is **not** a defect in this project and must not be read as one. It fails in
`blake3`'s build script:

```
error occurred in cc-rs: failed to find tool "cc": program not found
```

`blake3` compiles a **NEON** path for ARM through `cc-rs`, and this Windows host has no Apple/ARM C
cross-compiler. On real Apple hardware `cc` is clang from the Xcode command-line tools and the path
builds normally. The x86_64 target passes because it needs no C for that path.

**`blake3`'s `pure` feature was deliberately NOT enabled to make this check go green.** That feature
changes which code ships, and turning it on so a verification step passes would be tampering with
the product to flatter the test.

**What this does and does not add.** It adds: the core's macOS-relevant `cfg` branches now provably
*type-check* for an Apple target rather than merely having been read. It does not add: any execution,
any linking, any test result, anything about the C-dependent crates (`delulu-runtime`'s `libffi`,
`delulu-wasm`'s wasmtime), and nothing at all about Apple Silicon. **Zero macOS executions remain the
standing position**, and the honest summary is unchanged: engineered for, analyzed, now partly
type-checked, still unproven.

The remaining macOS work needs Apple hardware and cannot be closed from this machine:
running the suite, the CLI sweep, the `SUN_PATH_MAX = 104` socket-path guard in
`broker_transport.rs` (reasoned about, never observed firing), and the `libm.dylib`
dyld-shared-cache FFI path in `cli.rs`.

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

**A portability trap closed in the other direction (D72a).** Cross-platform work usually means *does
our code run there*. This one was the reverse: `delulu new con` **succeeded** on Linux and macOS and
produced a package Windows can never check out, because `con`, `aux`, `nul`, `prn`, `com1`–`com9` and
`lpt1`–`lpt9` are reserved device names there — `git clone` fails on the directory itself. The author
would not find out; a colleague would. Those names are now refused on **every** platform, including
the two where they would have worked, because a cross-platform language must not hand you a name that
only works on yours. On Windows the old failure was two different raw OS errors for one cause
(`os error 87` for `con`, `os error 2` for `aux`), which is the shape D33 already refused for
`run <dir>`.

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
added by the campaign. The four-test delta between platforms was the known one *at this date*; it is
now six, and §2's 2026-08-03 subsection names every test on both sides. "Windows ignores them" was
never the right description — Windows compiles its own tests asserting the refusal instead. Suite
counts rose from the campaign's own regression witnesses, every one of which was observed failing
against the code it now guards.

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

---

## Re-verified 2026-08-03 — the P17 proof campaign, every phase

| Gate | Windows (native) | Linux (WSL) | macOS |
|---|---|---|---|
| `cargo test --workspace` | **117 suites, 1526 passed, 0 failed** | **117 suites, 1532 passed, 0 failed** | **never executed** |
| **CLI + compiler sweep** | **22/22, 0 problems** | **22/22, 0 problems** | — |
| clippy (findings, summary lines excluded) | **14** (baseline) | — | — |
| `cargo deny` bans / licenses / sources | ok / ok / ok | ok / ok / ok | — |
| `cargo deny` advisories | **RED — 4 reachable, named** | same | — |
| TLA+/TLC — grant tree, leases, certificates | 585,771 + 2,421 distinct states, no error; **3 teeth tests reconstruct 3 real historical bugs** | (platform-independent) | — |
| Miri | **RAN, TIMED OUT — not a pass** | — | — |

Re-verified after Phase 6 (2026-08-04). Miri is listed as *incomplete* rather than omitted: it was
killed by a 50-minute cap partway through, found no UB in what it reached, and produced no summary
line. Rounding that up to "Miri passes" is exactly the kind of claim this campaign exists to catch.
**A second, uncapped run was still executing after ~3 hours and had produced no summary either.**
Miri therefore remains **not a pass**, and the entry above is unchanged.

### CI: PREPARED BUT UNVERIFIED — and the distinction is the point

`.github/workflows/ci.yml` now carries every gate this campaign added: the 22-case CLI sweep, the
effect-soundness fuzz campaign, `cargo deny`, both TLA+ models, **all three teeth tests** (each
written to FAIL the build if TLC *succeeds*, because a model that stops catching its bug has lost
its teeth), the Z3 authority algebra, and the Lean development.

**None of it has ever executed on a runner.** This repository is not pushed and will not be. The
YAML parses (3 jobs, 28 steps, validated locally) and every command in it has been run by hand on
this machine — but *"the workflow is written"* and *"the workflow is green"* are different claims,
and this document does not blur them. The macOS row stays **never executed** for exactly the same
reason: a matrix entry naming `macos-latest` is a plan, not a result.

One step is deliberately non-blocking: `cargo deny check advisories` runs with
`continue-on-error`, because four advisories are reachable and are **not** silenced. Reporting them
without blocking is honest; suppressing them to earn a green tick would not be.

The 6-test difference is the same platform delta [named test-by-test above](#re-verified-2026-08-03-production-readiness-pass-with-the-platform-delta-named); it did not move.

### Containers: PREPARED, NOT BUILT (2026-08-07)

`Dockerfile` (two-stage: `rust:1-bookworm` build, `debian:bookworm-slim` runtime, non-root user,
`cargo build --release --locked`) and `.devcontainer/devcontainer.json` now exist. **Neither has ever
been built.** The Docker CLI is installed on the authoring machine — version 29.5.2 — and its daemon
was not running, so `docker build` was never executed.

This is the same distinction the CI row above turns on, and it is worth repeating rather than
assuming the reader carries it down the page: *a Dockerfile that has never been built is a plan.*
Dockerfiles fail for boring reasons — a base image that moved, a missing `pkg-config`, a `COPY` path
that does not exist in the build context — and none of those are visible by reading it. The first
`docker build` should be treated as an experiment with a real chance of failing.

What can be said: every command inside it has been run on this machine outside a container
(`cargo build --release --locked -p delulu` is the ordinary build, `delulu --version` is in the CLI
sweep), so the *contents* are not speculative. The **assembly** is.

### The sweep is a script now, so this row means something

Previous passes recorded "CLI + compiler sweep 21/21" from a sequence of commands run **by hand**.
That is a claim only as good as the transcript, and it is precisely the hand-maintained procedure
design rule 1 says will drift. It is now `scripts/cli-sweep.sh` — **27 cases** today, each asserting
an exact exit code, runnable by anyone:

```sh
scripts/cli-sweep.sh                       # uses target/debug/delulu
scripts/cli-sweep.sh /path/to/delulu       # or an explicit binary
```

Writing it immediately caught two errors *in the sweep itself* that the hand version had been
carrying: two package cases that passed unconditionally, and a bare `delulu test` where the scaffold
prints `delulu test .`. **A sweep that cannot fail is not a sweep.** Both are fixed and the script
now refuses the bare form as a usage error (exit 2) as a case in its own right.

**It was 22 cases when written at P17-F and is 27 now**, P19 having added `help <subcommand>`, the
two typo-suggestion *content* assertions (exit code alone cannot tell a suggestion from a wall of
usage — both exit 2) and `doctor --check`. The earlier records that say 22 were **true when
written** and are deliberately left alone: this project does not rewrite a measurement that was
correct at the time. The script counts its own cases and prints the total, so the number to trust
is the one in its output — never one copied into prose, which is how this line came to be wrong.

### macOS, for this pass specifically

Unchanged and unchangeable without hardware: **never executed, not once.** What the P17 changes add
to the earlier readiness audit:

- `harden_wasm_features` uses only `wasmtime::Config` methods that exist on every platform, with no
  `cfg`. macOS would take the identical path.
- The zeroization change uses `std::ptr::write_volatile` and `std::sync::atomic::compiler_fence` —
  both `std`, both platform-independent, no conditional compilation.
- `scripts/cli-sweep.sh` is POSIX `sh` and takes an explicit binary path, so **it will run on macOS
  the day there is a Mac** — the sweep is no longer a Windows/Linux-shaped procedure.
- **RUSTSEC-2026-0096 deserves naming here.** It is a sandbox escape in wasmtime's aarch64 Cranelift
  backend. Apple Silicon is aarch64. This project has never run on macOS *or* on any ARM target, so
  it has never been exposed — but it ships source, and anyone building on an Apple Silicon Mac would
  be. That is the first macOS-specific *security* consequence this document has had to record, and
  it is recorded as an unverified exposure rather than a measured one.

---

## 9. The first CI run — 2026-09-14 (run `34830053479`)

The repository's first push (commit `5471d05`, to a private testing remote) ran
`.github/workflows/ci.yml` for the first time in its life: **15 checks — 3 passed, 1 skipped, 11
failed**, in about eleven minutes. It was read with the GitHub CLI (`gh run view --log-failed`), job
by job, not from the summary page. Every failure is accounted for below, and **none of them is a
defect in the language**: two gates that had never been able to pass, two artifacts recorded from
the development machine's disk instead of from what git stores, and one third-party C build that
current Apple tooling rejects.

| Job | Result | Why |
|---|---|---|
| `lints` (clippy `-D warnings`) | ✅ passed | First run on a runner. |
| `editor` (build the `.vsix`, prove it activates) | ✅ passed | First run on a runner. |
| `formal` (both TLA+ models, the three teeth tests, Z3, Lean) | ✅ passed | First run on a runner. Each teeth step fails the job unless TLC fails, so a pass means all three still catch their bug. |
| `heavy-gates` | skipped | Schedule or manual trigger only, by design. |
| `miri` ×5, `miri-ffi` | ❌ ~25 s each | A bare `cargo miri` under the stable toolchain `rust-toolchain.toml` pins. Miri is nightly-only; the runner's error matched the local reproduction word for word. |
| `supply-chain` | ❌ | RUSTSEC-2026-0268 and RUSTSEC-2026-0269 against wasmtime 47.0.3, published after the gate's last clean run (2026-08-07). |
| `test (ubuntu-latest)` | ❌ | `the_core_still_answers_exactly_as_recorded`. Cargo stopped at that first failing binary: 21 binaries had reported, 263 tests passed. |
| `test (windows-latest)` | ❌ | The same single failure and the same stop: 21 binaries, 258 passed. |
| `arm64` (Linux aarch64) | ❌ | All 124 binaries ran (`--no-fail-fast`): **1,649 passed, 5 failed, 4 ignored.** The five are the snapshot test, three `delulu doctor` tests and the Survey's freshness test. |
| `test (macos-latest)` | ❌ ~1 min | The build stopped in `libffi-sys` (below). No test binary was produced. |

**Two were defects in this repository's own reproducibility**, and both had passed every local run,
because Windows and WSL read the same disk:

1. **The core-invariance snapshot counted carriage returns no checkout contains.** 58 tracked files
   had CRLF line endings on the development machine while git holds them as LF — `.gitattributes`
   normalizes what is committed, not what a tool writes into the working tree. Three were the
   depth-limit fixtures (`DL0211`/`DL0212`/`DL0213`), so the snapshot carried byte offsets that
   counted CRs. Every such file was rewritten to its committed bytes (each checked identical to its
   index blob first; git records no change) and the snapshot re-recorded. **Exactly 12 values moved,
   every one a `"byte"` offset in those three cases, and each now equals what the runners printed.**
   No line, no column and no other case changed.
2. **The Survey mapped a file git ignores.** The packaged VS Code extension — build output,
   gitignored — was a node in the committed map, so every clean checkout counted one node fewer and
   failed the freshness gate and the three `doctor` tests that lean on it. Build-output files are now
   excluded by extension (`EXCLUDED_FILE_EXTENSIONS`, beside `EXCLUDED_DIRS`), and
   `crates/delulu-survey/tests/clean_checkout.rs` asks git which of the files the Survey reads it
   ignores. Run before the fix, it failed naming the file; after it, it passes.

**macOS — the first measurement.** On real Apple hardware cargo compiled `blake3`'s C code — the very
build script that stopped the Windows cross-check on 2026-08-10, so that blocker really was the
bench and not the code — and four DeluluLang crates (`delulu-diag`, `delulu-syntax`, `delulu-check`,
`delulu-broker`) with no reported error. Then `libffi-sys` 2.3.0 compiled the libffi it bundles
(3.4.4), and current Apple clang rejected that version's aarch64 `sysv` assembly with
`invalid CFI advance_loc expression`. The build stopped there, so **no DeluluLang code has run on
macOS yet.** The next run links macOS's own libffi instead (`crates/delulu-runtime/Cargo.toml`,
scoped to macOS; Windows and Linux are untouched), and whether that links and runs is what it will
show.

**ARM — the first measurement.** Until this run nothing here had executed on any ARM target. The
arm64 job ran the whole suite on Linux aarch64, and its only failures were the two artifacts above,
which are about the development machine rather than the architecture.

**What this run does not show:** anything after `core_invariance` on the x86 jobs, which stopped at
their first failing test binary. They now run with `--no-fail-fast`, so the next run reports
everything at once.

**Fixed before the next push:** `a52dc39` (Miri via `+nightly`; wasmtime 47.0.4; the nightly schedule
made opt-in), and the commit after it (the snapshot, the Survey, macOS's libffi, and this record).

### Run 2 — the same day (run `34836508713`, on `6abdfae`)

The first run's fixes, pushed together after a fresh-clone check reproduced green. **15 checks — 8
passed, 1 skipped, 3 failed, 3 cancelled at their time cap.** Read job by job with `gh`; the three
test jobs ran `--no-fail-fast` this time, so each reported everything.

| Job | Result | Why |
|---|---|---|
| `test (macos-latest)` | ❌ **1,654 of 1,655 passed** | **The first execution of DeluluLang code on macOS.** The whole workspace built, on macOS's own libffi. The one failure: `a_revoked_certificate_stays_revoked_across_a_daemon_restart`, whose state directory under macOS's 49-character temp dir put the broker socket path at 106 bytes, over the 103 macOS allows. |
| `test (ubuntu-latest)` | ❌ 1,654 of 1,655 | The actor speedup criterion at 4 workers, on a 2-vCPU runner: 1.04x. |
| `test (windows-latest)` | ❌ 1,643 of 1,646 | The same speedup test (0.96x), and two hardware-adapter tests whose PowerShell driver did not start within the adapter's 2000 ms exchange budget. |
| `arm64` (Linux aarch64) | ✅ | The whole suite, green. |
| `supply-chain` | ✅ | wasmtime 47.0.4 cleared both advisories. |
| `miri` `delulu-atlas`, `delulu-diag`; `miri-ffi` | ✅ 4.2 / 1.5 / 1.7 min | Miri's first completed runs on a runner; 0 UB. |
| `miri` `delulu-broker`, `delulu-syntax`, `delulu-check` | ⛔ cancelled at 45 min | 0 UB in what they reached. broker: 95 of 150 tests, then 39 minutes on one; syntax: 103 of 130, then 24 minutes on one; check: 25 of 237 at about 30 s each. |
| `lints`, `editor`, `formal` | ✅ | As in run 1. |
| `heavy-gates` | skipped | By design. |

**What it found that was real — two defects in `broker start`, each witnessed failing before its
fix.** The daemon refuses a socket path the kernel cannot hold, naming the byte count, the limit
and the remedy; but it is detached and its output goes to `broker.log`, so on macOS that refusal
reached the user as "broker daemon did not come up within 5s". `broker start` now runs the same
check before spawning (witnessed on Linux against the old code: the five-second timeout). And a
state directory that did not exist yet could not be started in at all — the daemon is spawned with
it as its working directory, which failed with "os error 267" on Windows and ENOENT on Linux unless
`--guard-policy` or `--require-anchored-roots` happened to create it first. `broker start` now
creates it (witnessed on Windows against the old code). The second defect was found by accident: the
first witness for the socket-path fix forgot to create its directory, and the failure it hit was
this one.

**What it found that was the runner.** GitHub's standard runners for private repositories have two
vCPUs. The actor speedup criterion is stated at 4 workers, so it is now asserted wherever at least
4 hardware threads exist (2.16x on the development machine) and reported as not measured elsewhere;
every semantic assertion in that test still runs everywhere. The hardware-adapter test's Windows
driver is Python rather than PowerShell, since PowerShell's start-up on a loaded runner overran a
budget that is deliberately not stretchable. The federation fixture uses short directory names.

**What it found about Miri.** Three crates cannot finish under Miri inside a push's 45 minutes on
this hardware, and none of them contains an `unsafe` block. They moved to a `miri-slow` job that runs
nightly where `NIGHTLY` is on, and by hand, with a 240-minute budget no run has yet confirmed.
`delulu-atlas`, `delulu-diag` and the FFI decoder — the one place Miri has real `unsafe` to watch —
stay on every push.

### Run 3 — the same day (run `34841317790`, on `28e10e6`)

**macOS, Linux x86-64 and Linux arm64 went green end to end.** On each, every step of the test job
passed (2026-09-14): 125 test binaries with 1,657 tests passed and 0 failed; `fmt --check` clean on
the examples and the Book's samples; the Python-less build; conformance coverage at 330 of 330
anchors; the reference in sync; the CLI sweep at 27 of 27; and the effect-soundness fuzz campaign —
50,000 generated programs, 30,198 executed, no trace escaping its row, with identical counts on macOS
and Linux because the seed is fixed. clippy, `cargo deny`, the editor build, the formal models, and
Miri on `delulu-atlas`, `delulu-diag` and the FFI decoder passed; `miri-slow` and `heavy-gates` were
skipped, by design, on a push.

**Windows passed 1,646 of 1,647.** The one failure was
`adapter::tests::a_garbled_reading_is_an_error_never_a_none_and_never_a_number`, which passes on the
development machine and passed on the Windows runner in run 2. The cause: its fake driver was
PowerShell, whose start-up on a loaded two-core runner sits inside the adapter's deliberate 2000 ms
first exchange — and the test spawns five drivers in a row. It is the cause `hw_adapter_cli` hit in
run 2, with the same fix: the adapter's unit-test drivers are Python now. Until a run passes with it,
this document does not call Windows green on CI.

### Run 4 — the same day (run `34844151767`, on `ef9cb49`)

**Windows went green end to end — its first complete pass on CI** (2026-09-14). Every step of the
Windows test job passed: 125 test binaries with 1,647 tests passed, 0 failed and 4 ignored; `fmt
--check` clean; the Python-less build; conformance at 330 of 330 anchors; the reference in sync; the
CLI sweep; and the fuzz campaign, 30,198 programs executed with no trace escaping its row. The Python
fake drivers held. Linux x86-64 and arm64 were green again, as were clippy, `cargo deny`, the editor,
the formal models and Miri on `delulu-atlas`, `delulu-diag` and the FFI decoder; `miri-slow` and
`heavy-gates` were skipped, by design. So every operating system in the matrix has now passed the
whole suite on CI — macOS in run 3, Windows in run 4, Linux in both — though not yet all in one run.

**macOS failed one test, and the test was measuring the runner.**
`device::tests::a_beaten_lease_is_never_revoked` grants a lease with a 120 ms heartbeat and beats it
every 20 ms for 600 ms; on the macOS runner a command found the lease revoked for a missed heartbeat.
The watchdog revokes only when `now − last_beat` exceeds the heartbeat on a monotonic clock; it
samples `now` before taking the lease lock, so a beat that lands in between makes it *more* lenient;
and nothing else holds that lock for long. So the lease really had gone 120 ms without a beat: the
runner had left the test's thread unscheduled for more than 100 ms, early in a test binary whose
other tests are CPU-heavy. The dead-man did its job. The test's premise — *this lease is being
beaten* — is what failed.

That was measured rather than assumed, on the development machine (2026-09-14):

| Experiment | Result |
|---|---|
| The unmodified test, idle machine | 25 of 25 passed: nothing fires on its own |
| The unmodified test with its thread starved (a high-priority busy loop on the same CPU, 150 ms busy, 40 ms idle) | failed 5 of 8 runs, **with CI's message word for word** |
| One extra pause injected into the drive | a 20 ms period survives a stall of about 100 ms, a 50 ms period only 60–70 ms — so a longer sleep would have made it worse |
| A mutant watchdog that fires 200 ms early | caught by this test, and **by no other device test** |

**The fix judges each revocation against the gap the thread actually left.** The lease cannot have
gone unbeaten longer than the time since the last accepted command was sent. If the broker reports
more than that — the heartbeat plus its own `overdue_us` — the watchdog fired on a healthy lease and
the test fails at once; that is what the mutant hits. A revocation the gap explains proves nothing
either way, so the drive restarts on a fresh broker until one runs clean, and if none does within
10 s the test fails as *not measured* rather than passing on a property it never saw. Moving the test
to the stepped clock was considered and rejected: that clock never runs the wall-clock watchdog, and
a stepped version passed the mutant. After the fix: 25 of 25 idle, 12 of 12 starved (every
revocation it judged followed a real gap of 150–172 ms), and the mutant still caught. The two sibling
tests with tighter timing — `safe_park_returns_the_simulated_device_to_its_park_pose` (40 ms) and
`crates/delulu/tests/dead_man_cli.rs`'s `a_program_that_stops_beating_loses_its_device_mid_run` (25 ms
to its first command) — passed all 20 starved runs between them, so neither is recorded as exposed.
The fix passed in run 5, below.

### Run 5 — the same day (run `34849980129`, on `010c36c`)

**The first run with no failures: every job green, all three operating systems in one run**
(2026-09-14). Eleven jobs passed; `heavy-gates` and `miri-slow` were skipped, by design, because they
run only on a schedule or by hand.

| Job | Result |
|---|---|
| `test (macos-latest)` | ✅ 125 test binaries: 1,657 passed, 0 failed, 4 ignored. Then every step: `fmt --check` on the examples and the Book's samples, the Python-less build, conformance at 330 of 330 anchors, the reference in sync, the CLI sweep at 27 of 27, and the fuzz campaign — 50,000 generated, 30,198 executed, no trace escaping its row |
| `test (ubuntu-latest)` | ✅ The same, number for number |
| `test (windows-latest)` | ✅ 125 test binaries: 1,647 passed, 0 failed, 4 ignored — ten fewer tests are compiled there, all platform-gated — and every later step |
| `arm64` (Linux aarch64) | ✅ 125 test binaries: 1,657 passed, 0 failed, 4 ignored |
| `lints`, `supply-chain`, `editor`, `formal` | ✅ |
| `miri` on `delulu-atlas` and `delulu-diag`; `miri-ffi` | ✅ |
| `heavy-gates`, `miri-slow` | skipped — schedule or manual trigger only |

`device::tests::a_beaten_lease_is_never_revoked`, the test run 4 failed on macOS, passed on all four
test runners. `REMAINING_WORK.md` 7.2 closes on this run, as its own row said it would.

**What a green run does not show.** Nothing here has run on a developer's Mac, and the VS Code
extension has been exercised end to end on Windows only. The Tier-2 cross-account boundary was tested
with a real second user on Linux, not on macOS. `heavy-gates` and `miri-slow` have not yet run on a
runner at all, so the 240-minute Miri budget is still unconfirmed.
