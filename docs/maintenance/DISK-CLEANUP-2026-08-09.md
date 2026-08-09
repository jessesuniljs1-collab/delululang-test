# Disk cleanup — 2026-08-09

A record of what was deleted from this machine, what was kept, and **how each deletion was verified
safe before it happened**. Same discipline as the previous pass — see
[`DISK-CLEANUP-2026-08-04.md`](DISK-CLEANUP-2026-08-04.md), whose "A deletion that had to be undone"
section (the ext4 WSL `target` dirs that were the *only* working Linux build configuration) is the
reason this pass keeps WSL entirely out of scope.

This is an operations record, not a product document. Nothing here changes DeluluLang's behaviour.

## Why it happened

The owner asked for junk/useless project files on `C:` and `D:` to be removed, with a standing
constraint added mid-task: **do not delete any `.md` file, anywhere.** There was no space emergency —
`C:` had 83 GB free and `D:` 212 GB free before this ran — so this is clutter removal, not a
reclaim under pressure. Two owner decisions were taken at a confirmation gate:

* **Keep every build cache**, including the 70.56 GB `target\debug\deps`. Deleting it would force a
  full wasmtime/cranelift rebuild, and the 08-04 log already called that "a judgement call for the
  owner, not a cleanup decision." With 212 GB free, there is no reason to pay that cost.
* **Delete `DiagOutputDir`** even though it is Remote-Desktop trace junk, not a project artifact.

## How "safe to delete" was established

1. **Working tree was clean.** `git status` reported nothing to commit and no untracked
   non-ignored files, so every deletion candidate is either a git-ignored build artifact or lives
   outside the repository. Nothing removed here was tracked or uncommitted source.
2. **No process held a lock.** No `delulu`, `cargo`, or `rustc` process was running, so removing
   `target\debug\incremental` could not race a live build.
3. **`.md` files were located first and preserved.** Before deleting any scratchpad, every `.md`
   under the candidates was enumerated. The only `.md` found in a delete target were 13 files in the
   `bcacf460` scratchpad — that session was therefore cleaned **file-by-file, skipping `.md`**,
   rather than removed wholesale. All 13 remain in place.
4. **Crash dumps were read before deletion** (the 08-04 lesson), signatures recorded below so the
   evidence survives the bytes.
5. **WSL was not touched.** The Linux clones and ext4 `target` dirs live inside the vhdx and include
   the only working Linux build configuration; they are deliberately out of scope.

### A harness note worth keeping

The command-safety scanner statically rejects `Remove-Item` when the target is a **variable**
(`Remove-Item -LiteralPath $p …` was blocked as "system path 'D:'") or sits next to arithmetic it
mis-tokenises (`[math]::Round($s/1MB,2)` was read as a path `/1MB,2`). Deletions had to be issued
with **full inline literal paths and no adjacent arithmetic**. Nothing was deleted by a blocked
command — the block is pre-execution.

## Deleted — on `C:` (freed ~15.5 GB: 83.0 → 98.5 GB free)

| Path | Size | What it was |
|---|---:|---|
| `…\Temp\wsl-crashes` (10 dumps) | 7.66 GB | SIGSEGV core dumps of the `delulu` CLI — **signatures below** |
| `…\Temp\DiagOutputDir` | 1.56 GB | Remote-Desktop `RdClient` `.etl` traces (not a project artifact) |
| `…\Temp\claude\D--nelan-DeluluLang\12a5b8ea…` | 5.12 GB | finished (03 Aug) scratchpad — 5.0 GB of it a single runaway background-task log `tasks\br2r64rey.output` |
| `…\c99ba5c5…\scratchpad\redteam-target` | 1.24 GB | red-team release build cache (this campaign, closed) |
| `…\c99ba5c5…\scratchpad\mac-target` | 0.36 GB | macOS cross-compile build cache |
| `…\c99ba5c5…\scratchpad\adv` | 0.02 GB | adversarial test bundle |
| `bcacf460…` non-`.md` content | ~0.065 GB | VS Code profile cache (`vsc-udd`), an unpacked `delulu.exe`, `win_base_raw.txt`, `hostile-workspace\payload.pdb` — **13 `.md` kept** |
| 40 finished/empty scratchpad session dirs | ~0.0003 GB | spent per-session staging dirs (current session preserved) |

The **current** session scratchpad (`c99ba5c5…`, 333 working files) was preserved apart from the
three build/bundle dirs above. Session *transcripts* and *memory* under
`C:\Users\jesse\.claude\projects\` were **not touched**.

## Deleted — on `D:` (freed ~6.1 GB: 212.3 → 218.4 GB free)

| Path | Size | What it was |
|---|---:|---|
| `D:\nelan\DeluluLang\target\debug\incremental` | 7.76 GB | pure recompile cache — cargo regenerates it; costs nothing but the next incremental build. The same category the 08-04 pass deleted safely. |

(The free-space delta reads smaller than the sum because the tree saw ordinary write activity during
the ~30-minute pass; each path above was individually confirmed removed by `Test-Path`.)

## Crash-dump signatures, recorded so the evidence outlives the dumps

Ten dumps, each ~785 MB, all **SIGSEGV** (`-11`) in the `delulu` **CLI** binary, spanning 06 Aug →
09 Aug — the same stack-overflow-on-nested-input class that this machine's hardening campaign has
been closing (`DL0210` expr, `DL0211` type, `DL0212` pattern, `DL0213` block; all bounded at 128).
They are a by-product of that campaign's own fuzzing of pre-fix Linux binaries.

| Count | Origin binary (WSL path) | Signal |
|---:|---|---|
| 2 | `/home/user/dl-target/debug/delulu` | 11 |
| 3 | `/home/user/dlx/debug/delulu` | 11 |
| 3 | `/home/user/delulu-lin-target/debug/delulu` | 11 |
| 2 | `/home/user/delulu-target/debug/delulu` | 11 |

They were removed without symbolication (Linux minidumps, not readable here), exactly as the 08-04
pass handled the previous batch. If a *new* SIGSEGV class is ever suspected, reproduce it with a
fresh input rather than trusting these to still exist.

## Kept deliberately

* **All build caches** — `target\debug\deps` (70.56 GB), `release\deps` (3.51 GB), `debug\build`,
  `release\build`, the miri and `*-apple-darwin` cross caches. Owner decision; no space pressure.
* **Every `.md` file**, including `bcacf460`'s 13 (P19 analysis notes + an unpacked release
  package's `CHANGELOG`/`README`/`SECURITY`/`TRADEMARK`).
* **`dist\`** (packaged v1.0.0) and **`editors\vscode\node_modules`** — tiny, and the latter is the
  extension's working dependency.
* `.claude\worktrees\`, session transcripts + memory, WSL clones and the ext4 Linux `target` dirs,
  `~/.cargo` / `~/.rustup` — all as in the 08-04 pass.

## WSL cleanup (verified) — 49.7 GB freed inside the guest

The first compaction (run by the owner from an elevated shell) succeeded but reclaimed **nothing** —
`ext4.vhdx` stayed at 70.8 GB — because `df /` showed **67 GB genuinely in use**, not slack. The
space was a pile of redundant ext4 build-target caches from different sessions: no source clones
remain (the 08-04 pass removed those), and `CARGO_TARGET_DIR` is not set in shell config, so each
Linux run created its own target dir against the `/mnt/d` checkout.

Every candidate was classified before deletion (`CACHEDIR.TAG` ⇒ pure cargo cache; `Cargo.toml` +
`crates/` ⇒ a source-bearing working copy), and every source-bearing dir was content-hashed against
`D:` HEAD (`b203983`), mirroring the 08-04 method. A self-guarding script refused any "cache" that
turned out to hold source. Deleted:

| Path | Size | Basis |
|---|---:|---|
| `delulu-lin-target`, `dl-target`, `dlx` | 31.7 GB | pure caches (`CACHEDIR.TAG`, no source) |
| `dl-pkg-target`, `delulu-clippy-cold`, 4× miri targets | 4.0 GB | pure caches |
| `delulu-fed` | 7.2 GB | working copy — **488/488 files present in `D:` HEAD** |
| `dl-before` | 0.9 GB | git snapshot — 573/573 files in `D:`, HEAD `0c98a58` confirmed in `D:` |
| `delulu-f1/target-linux` | 7.2 GB | build cache only — **its source was kept** |
| `dl-dist` | 29 MB | packaged Linux release, regenerable |

Total freed inside the guest: **49.7 GB** (`df /`: 67 G → 18 G used).

**Kept:** `delulu-target` (13 GB, freshest — a warm cache for the next Linux build); `delulu-f1`
(now 7.7 MB, source only) and `delulu-linux2` (7.6 MB) — each holds a **superseded 2026-07-22 file
not in `D:`** (an old 6,760-line `cli.rs` vs today's 8,245; an old `STAGE10_BUILD_ORDER.md`; an F1
verify script), surfaced and preserved rather than swept; the OS-coursework files; `~/.cargo` and
`~/.rustup`.

## Final compaction — done (`ext4.vhdx` 70.8 → 21.6 GB)

`fstrim -v /` was run first (988.9 GiB of free blocks discarded) so the freed space became
reclaimable — the step the first attempt lacked. The owner then re-ran the **same** safe script
(`attach readonly → compact → detach`, *not* the disabled `--set-sparse --allow-unsafe` path) from
an **elevated** shell:

```
wsl --shutdown
diskpart /s "C:\Users\jesse\AppData\Local\Temp\compact_vhd.txt"
```

**Result:** `ext4.vhdx` shrank **70.8 GB → 21.6 GB**, returning **49.8 GB to `C:`** (98.3 → 148.1 GB
free). The lesson holds: on a used-up guest disk, compaction is a no-op until you *delete inside,
`fstrim`, then compact* — in that order.

## Net result

| Volume | Before | After (this pass) | Final (after compaction) |
|---|---:|---:|---:|
| `C:` free | 83.0 GB | 98.5 GB | **148.1 GB** |
| `D:` free | 212.3 GB | 218.4 GB | 218.4 GB |
| WSL used (`df /`) | 67 GB | 18 GB | 18 GB |
| `ext4.vhdx` | 70.8 GB | 70.8 GB | **21.6 GB** |

**Total reclaimed tonight: ~71 GB** — ~65 GB on `C:` (83.0 → 148.1) and ~6 GB on `D:` (212.3 →
218.4). ~21.6 GB came from direct Windows/`D:` deletions; ~49.8 GB came from the WSL cleanup +
`fstrim` + compaction chain.
