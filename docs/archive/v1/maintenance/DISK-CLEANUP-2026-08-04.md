# Disk cleanup — 2026-08-04

A record of what was deleted from this machine, what was kept, and **how each deletion was verified
safe before it happened**. Written because a deletion you cannot audit later is indistinguishable
from data loss.

This is an operations record, not a product document. Nothing here changes DeluluLang's behaviour.

## Why it happened

`C:` had **18.2 GB free of 471.6 GB**, which is reason enough to clean up. The Ubuntu WSL disk
`ext4.vhdx` lives on `C:`, is *dynamically expanding*, and had grown to 134.3 GB. Inside WSL, `df`
cheerfully reported **825 GB available** — the guest's view of a virtual disk that the host can no
longer grow.

## Two wrong diagnoses, recorded rather than quietly dropped

A Linux build failure ran alongside this cleanup, and it was blamed on the disk. **That was wrong.**
Both explanations are kept here because a cleanup record that only lists correct conclusions teaches
nothing about how much to trust the next one.

```
error: failed to run custom build command for `libffi-sys v2.3.0`
.././configure: line 2247: config.log: No such file or directory
```

`libffi` is a non-optional dependency of `delulu-runtime` (Stage 4 C FFI), so the whole workspace
build went down with it and `cargo test --workspace` reported **0 suites**.

1. **"It's build contention."** Disproved: Miri writes to `/home/user/delulu-miri` and `cargo test`
   to `/mnt/d/.../target` — different trees — and the failure reproduced with Miri killed and the
   build running alone.
2. **"It's host disk exhaustion."** Disproved: it reproduced with `C:` at 37.4 GB free.

**The actual cause is the filesystem, not the disk.** `libffi-sys`'s configure creates a build
subdirectory (`--enable-builddir`), `cd`s into it and writes `config.log` through a relative path.
That sequence fails on **drvfs** — the `/mnt/...` mount of a Windows volume. Every failing run had
`OUT_DIR` on drvfs: first through the default `target/`, then through the
`/mnt/d/nelan/dl-linux-target` introduced as a "fix" for diagnosis 1.

This has a direct consequence for the cleanup below: **the deleted `/home/user/*-target`
directories were on ext4, and that was the only working Linux build configuration.** See
"A deletion that had to be undone".

## How "safe to delete" was established

Git commit membership was checked first: `198bf44`, `92ef3cc` and `0c98a58` are all **ancestors of
`D:` HEAD** (`c4a81cb`), so those histories are already in the authoritative repository.

Commit membership is not enough — a working tree can hold uncommitted edits. So every file in every
clone was content-hashed and looked up in `D:`'s object database:

```sh
find . -type f -not -path "./target/*" -not -path "./.git/*" \
  | git hash-object --stdin-paths \
  | git -C /mnt/d/nelan/DeluluLang cat-file --batch-check
```

| Clone | Files checked | Content **not** in `D:` | What those were |
|---|---:|---:|---|
| `delulu-linux` | 511 | 1 | a generated `interface.json` under the greeter example's gitignored build directory — **not** a tracked path in this repository, which is why nothing in the tree matches it |
| `DeluluLang` | 408 | 2 | same generated artifact, plus a superseded `Cargo.lock` |
| `dl-clone-test` | 574 | 1 | same generated artifact |
| `delulu-d23` | 16,303 | 15,861 | **see below — this number was an artifact of the check** |

### The check that was wrong, and how it was caught

`delulu-d23` appeared to hold 15,861 unique files. That was not believable — `SECURITY.md` and
`CONTRIBUTING.md` are in it and certainly exist in `D:`. Two causes, both found by testing rather
than assuming:

1. **Line endings.** The clone's files are CRLF on disk; `D:`'s committed blobs are LF, so
   `git hash-object` produced different hashes for identical content. Re-hashing a 40-file sample
   with CR stripped: **39 matched a blob in `D:`**, and the one that did not was
   `target-linux/debug/build/thiserror-*/build-script-build` — a build artifact.
2. **A missed exclusion.** The sweep excluded `./target/*` but the clone also had `target-linux/`,
   which is why its file count was 30× the others'.

Independently, `delulu-d23/SECURITY.md` was shown to be strictly **older** than `D:`'s — `D:` has 24
lines it lacks (the Trojan Source / DL0107 section). A superseded snapshot, not unique work.

## Deleted — inside WSL (freed 111 GB: 132 G → 21 G used)

Build caches, regenerable by definition:

| Path | Size |
|---|---:|
| `/home/user/delulu-target` | 25 G |
| `/home/user/delulu-linux-target` | 15 G |
| `/home/user/dl-target` | 9.8 G |
| `/home/user/delulu-linux2-target` | 7.8 G |
| `/home/user/delulu-miri` | — |
| `/tmp/*` | 8.2 G |

Source clones, each verified above:

| Path | Size | HEAD |
|---|---:|---|
| `/home/user/DeluluLang` | 23 G | `198bf44` (v1.0.0), clean |
| `/home/user/delulu-linux` | 8.4 G | `92ef3cc`, 503 modified files — all content present in `D:` |
| `/home/user/dl-clone-test` | 7.5 G | `0c98a58`, clean |
| `/home/user/delulu-d23` | 7.3 G | not a git repo; superseded snapshot |

`/tmp/miri_batched.txt` and `/tmp/lin.txt` were copied out and restored — they held in-flight
verification results.

## Deleted — on `C:` (free 25.3 GB → 37.3 GB)

| Path | Size | What it was |
|---|---:|---|
| `…\Temp\wsl-crashes` | 7.7 G | 10 WSL crash dumps — **see the finding below** |
| `…\Temp\DiagOutputDir` | 2.2 G | Remote Desktop `RdClientAutoTrace` `.etl` traces |
| `…\Temp\claude\**` | ~7 G | 43 finished Claude session scratchpads |

The **current** session scratchpad was preserved. Session *transcripts* live in
`C:\Users\jesse\.claude\projects\` and were **not touched** — only the `Temp` scratchpads were.

## Kept deliberately

* `/home/user/delulu-fed` (7.2 G) and `/home/user/delulu-f1` (7.2 G) — not yet verified; federation
  work is recent enough to warrant checking before deletion rather than after.
* `D:\nelan\DeluluLang\.claude\worktrees\` — a standing instruction forbids deleting it unattended.
* `~/.cargo/registry` (812 M) and `~/.rustup` — re-downloading a toolchain costs more than it saves.

## Finding: ten SIGSEGVs in the `delulu` binary

The crash dumps were not inert junk, and the finding is recorded here so it survives them.

Ten dumps, each ~784 MB — a suspiciously uniform size, consistent with one repeating crash:

```
wsl-crash-<epoch>-<pid>-_home_user_delulu-target_debug_delulu-11.dmp
```

* All from `/home/user/delulu-target/debug/delulu` — the CLI, not a test harness.
* Trailing **`-11`** is the signal: **SIGSEGV**.
* Timestamps run 2026-08-03 16:48 → 2026-08-04 04:02, i.e. straight through the P17 campaign.

A stack overflow presents as SIGSEGV, which pointed at the already-open **D87** robustness item
(*unbounded parser recursion*). The dumps were deleted without symbolication, so they proved
nothing on their own — but the hypothesis was **tested rather than assumed**, and it reproduced:

```
$ delulu check deep_100000.delulu
thread 'delulu-main' (26972) has overflowed its stack
exit 127
```

The input is a valid module whose body is `((((…1…))))` nested *n* deep. Bisected on Windows:

| nesting | result |
|---:|---|
| 2,000 / 10,000 / 30,000 / 50,000 / 70,000 | `ok`, exit 0 |
| 100,000 | **stack overflow, exit 127** |

So the parser is not merely slow at depth — it succeeds up to at least 70,000 and then dies between
70,000 and 100,000. The crash is a hard abort with no diagnostic: no `DL####` code, no span, nothing
a caller can catch.

**This has since been fixed** (`DL0210`, expression nesting capped at 128) and the full account —
including the second, non-obvious recursion in `Drop` that the first fix did not close — is in
`docs/design/PROOF_CAMPAIGN.md` §P17-F5. The same input now returns a diagnostic and exit 1.

The maintenance lesson stands regardless of the fix: **ten core dumps sat in `%TEMP%` for a day and
no test noticed.** They were found by looking at disk usage, not by the campaign. For a language
aimed at autonomous systems — where the compiler may be handed input by an agent rather than a
human — a crash on adversarial input is a denial-of-service surface, and crash dumps are evidence
worth reading before deleting.

## Deleted — on `D:` (free 187.4 GB → 222.0 GB, freed 34.6 GB)

`target\debug` measured **68.3 GB `deps` + 42.2 GB `incremental` + 1.3 GB `build`**.

Only `incremental` was deleted. It is a pure recompile cache: cargo regenerates it and losing it
costs nothing but the next incremental build. **`deps` was deliberately kept** — it is the live
build cache, and deleting it forces a full wasmtime rebuild on both platforms. Reclaiming that
further ~68 GB is a judgement call for the owner, not a cleanup decision, and `D:` now has 222 GB
free so it is not pressing.

## A deletion that had to be undone

The WSL-side target directories were **not** merely clutter, and deleting all of them was a mistake.
They did two jobs:

1. `D:\nelan\DeluluLang\target` is shared between the Windows and Linux toolchains, and cargo places
   host artifacts in `target/debug` for **both** — so each platform's build clobbers the other's and
   every switch forces a full rebuild.
2. More importantly, they were on **ext4**, which is the only filesystem on which `libffi-sys` — and
   therefore the entire workspace — will build under WSL.

The first attempt to restore the separation put the target on `D:`:

```sh
export CARGO_TARGET_DIR=/mnt/d/nelan/dl-linux-target   # WRONG — drvfs, libffi-sys cannot configure
```

That keeps artifacts out of `ext4.vhdx`, but it is drvfs, so it reproduced the build failure exactly.
The working configuration is an **ext4** path:

```sh
export CARGO_TARGET_DIR=/home/user/dl-target           # ext4 — builds
```

The cost is real and must be stated: a Linux target directory is 15–25 GB and it lives inside
`ext4.vhdx` on `C:`. Linux build artifacts therefore consume `C:` by construction, and no
rearrangement avoids it while the distro's root is a vhdx on `C:`. Compacting the vhdx (below) is
what makes that affordable rather than fatal.

## Still outstanding

* **`ext4.vhdx` compaction needs Administrator and has not run.** Deleting files inside WSL does not
  shrink the virtual disk, so ~111 GB is still trapped in a 134.3 GB file on `C:`. From an
  **elevated** shell:

  ```
  diskpart /s "C:\Users\jesse\AppData\Local\Temp\compact_vhd.txt"
  ```

  `wsl --manage <distro> --set-sparse true` was tried first and **refused**: Microsoft disables
  sparse VHD "due to potential data corruption". The `--allow-unsafe` override exists and was
  **not** used.
* Reproducing the SIGSEGV described above.
* `/home/user/delulu-fed` and `/home/user/delulu-f1` (7.2 G each) remain unverified and undeleted.

## Net result

| Volume | Before | After |
|---|---:|---:|
| `C:` free | 18.2 GB | 37.4 GB |
| `D:` free | 187.4 GB | 222.0 GB |
| WSL used | 132 G | 21 G |

A further ~111 GB returns to `C:` once the compaction above is run.
