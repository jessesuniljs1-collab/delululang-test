# Disk cleanup — 2026-11-10

A record of what was deleted from `D:\nelan`, from `C:` and from WSL's disk, what was kept, and **how each deletion
was checked before it happened**. The previous pass is [`DISK-CLEANUP-2026-10-17.md`](DISK-CLEANUP-2026-10-17.md),
whose method this one repeats; the two before it are in the archive
([`2026-08-04`](../archive/v1/maintenance/DISK-CLEANUP-2026-08-04.md),
[`2026-08-09`](../archive/v1/maintenance/DISK-CLEANUP-2026-08-09.md)).

This is an operations record, not a product document. Nothing here changes DeluluLang's behaviour, and nothing
tracked by git was touched.

## Why it happened

The owner saw `D:\nelan` at **234 GB** (Explorer: 251,260,194,091 bytes, 306,648 files, 22,742 folders) while the
GitHub repository's download (`delululang-test-master.zip`) is 7.9 MB, and asked: *"Why size of "D:\nelan"
delululang files and folders are 234 GB … when same GitHub repo download … is just 7.9 MB. Delete all unwanted
things. From C drive and other places also delete unwanted things of delululang. Check wsl and others. Make a .md
file and log everything deleted. Verify everything before deleting"*.

**The answer, again: Rust's build cache.** The zip holds only the files git tracks — the source. `cargo` writes
everything it compiles into `target\`, and it never removes an artifact it has superseded: every test binary (with
its debug information, tens to hundreds of MB each), every dependency at every version, and the incremental
recompile cache, for each of the hundreds of builds since the last `cargo clean` on 2026-10-17. Measured with
robocopy in list mode (bytes; nothing copied):

| Folder | Size | What it was |
|---|---:|---|
| `DeluluLang\target\debug\deps` | 157.58 GB | compiled crates and test binaries, mostly superseded copies (12,214 artifacts) |
| `DeluluLang\target\debug\incremental` | 51.50 GB | recompile cache (206,315 files) |
| `DeluluLang\target\release` | 3.26 GB | release builds |
| `DeluluLang\target\debug\build`, `examples`, `.fingerprint` | 1.20 GB | build-script outputs and cargo's bookkeeping |
| `DeluluLang\target\aarch64-apple-darwin`, `tmp`, the rest | 0.05 GB | the macOS lint build and test fixtures |
| **`DeluluLang\target` in all** | **214.83 GB** | 244,190 files |
| `dl-launch\target` | 18.13 GB | the build cache of the `launch/2.0.0` worktree |
| `DeluluLang\fuzz\target` | 0.27 GB | the fuzzers' build cache |
| everything else in `DeluluLang` (source, docs, `.git` 0.40 GB, `editors`, `dist`) | about 0.8 GB | |
| `D:\nelan\DeluluLang-agent-transcripts` | 0.18 GB | agent-pass records — **kept** |
| `D:\nelan\dogfood-campaign`, `github-zip`, `worktree-notes-2026-09-17`, the history bundle, two logs, one review | 0.03 GB | **kept** |

## How "safe to delete" was established

1. **GitHub first.** `git fetch origin --prune`: `master` = `origin/master` = `c0a6802`, the working tree clean, two
   stashes (kept). The worktree `D:\nelan\dl-launch` clean at `728a582` = `origin/launch/2.0.0` (only its `target/`
   ignored and present). No `cargo`, `rustc` or `delulu` process ran on Windows or in WSL.
2. **Pure caches only by their tag.** Every `target` deleted held cargo's `CACHEDIR.TAG`; the WSL script refused any
   that did not.
3. **`.md` files were found first** (the standing rule: never delete a `.md`). The Windows caches held none. The
   Windows temp folder held three (`delulu-pkgrun-{grants,tier4,tier4dwx}-13200\README.md`), each a blob in this
   repository (`git hash-object` → `git cat-file -e`). WSL's caches held six `libffi-build/README.md`, each
   **byte-identical** (`cmp`) to `libffi-sys-2.3.0/libffi/README.md` in the cargo registry, which keeps it. The three
   clones held 270, 272 and 262 `.md`, every one a blob in this repository. The kernel build tree held three, each
   byte-identical to its copy in the kept `linux-6.18.54.tar.xz` (extracted and compared). The deletion scripts
   re-ran these checks on every path and would have refused any path with an unmatched `.md`; none was refused.
4. **The three WSL clones held nothing this repository lacks.** Run as the owning user: no stash, every commit
   reachable from any of their refs present in `D:` (`git rev-list --all | git cat-file --batch-check`: 0 missing),
   and each HEAD an ancestor of `origin/master`. `dl-studyc` had eight modified files; each one's content is the blob
   at that path in `46caa70` on `origin/master` — the work was landed there, the clone was never cleaned up.
5. **Every test-scratch name was traced to the test that makes it** (`dlmon-` `monitor_cli.rs`, `dl462_`
   `guard_e2e.rs`, `dlgr-` `grants_cli.rs`, `dls-`/`dlp-`/`dll-`/`dlx-`/`dl_fresh_` `broker_cli.rs`, `dlb`/`dlc` the
   broker's audit and cert tests, …). Nothing modified in the hour before was touched.
6. **Nothing referenced the binaries removed.** `dogfood-bin` and the old scratch `.exe`s: no reference in the
   scheduled tasks (`DeluluLang-autosave` runs `AppData\Local\DeluluLang\autosave.ps1`, `DeluluLang-dogfood-save`
   runs `dogfood-campaign\save.sh`), in `.claude`, `dogfood-campaign`, `~/.bashrc` or the repository's `scripts`.

## Deleted

| Where | Path | Size | How |
|---|---|---:|---|
| `D:` | `D:\nelan\DeluluLang\target` | 214.8 GiB, 244,190 files | `cargo clean` (34 s) |
| `D:` | `D:\nelan\dl-launch\target` | 18.1 GiB, 34,328 files | `cargo clean` in the worktree — **the worktree itself kept** |
| `D:` | `D:\nelan\DeluluLang\fuzz\target` | 272.4 MiB, 970 files | `cargo clean` in `fuzz` |
| `C:` | `C:\Users\jesse\AppData\Local\Temp\delulu*`, `dl*` | 1.71 GB (1,706,243,746 bytes), 64,724 entries, 11,388 files | a script with the guards above; 0 failures |
| `C:` | 24 `.exe` in `AppData\Local\Temp\claude\D--nelan-DeluluLang\…\scratchpad\` | 0.90 GB (901,412,352 bytes) | pre-change `delulu` builds kept for before/after comparisons (RW 2.10, A14, …), 2026-10-21 → 10-29; the notes and scripts beside them kept |
| WSL | `/home/user/delulu-gnu-target` | 14,907 MB | pure cache — rebuilt afterwards, below |
| WSL | `/home/user/dl-dogfood-target` | 3,294 MB | pure cache, 2026-10-24 |
| WSL | `/home/user/delulu-kvm-target` | 3,265 MB | pure cache (the `--cfg delulu_kvm` build), 2026-10-17 |
| WSL | `/home/user/delulu-img-target` | 1,015 MB | pure cache (the guest image's musl build), 2026-10-17 |
| WSL | `/home/user/dl-target` | 654 MB | pure cache (release), 2026-11-01 |
| WSL | `/home/user/dl-psa-arch/target`, `/home/user/dl-mac-harden/target` | 103 MB | pure caches inside two labs — **their source kept** |
| WSL | `/home/user/dl-studyc` | 1,325 MB | clone at `e85fcb5` (`laptop/wasm-list`), most of it its `target` — verified above |
| WSL | `/home/user/dl-dogfood` | 49 MB | clone at `41f1dc6` — verified above |
| WSL | `/home/user/delulu-src-img` | 39 MB | clone at `f731e5b` — verified above |
| WSL | `/home/user/dogfood-bin` | 438 MB | one debug `delulu` binary of 2026-10-24 (the dogfood campaign) |
| WSL | `/home/user/.cache/delulu-microvm/work` | 1,784 MB | the unpacked `linux-6.18.54` build tree `scripts/microvm/build-image.sh` makes; the pinned tarballs beside it kept (deleted for the same reason on 2026-10-17) |
| WSL | `/tmp/delulu*`, `/tmp/dl*` owned by `user`, older than an hour | 133 MB, 2,303 entries | spent test scratch (`delulu_brokerd_*`, `delulu-compute-*`, `dlmon-*`, `delulu_plugin_*`, …) |

Each Windows path was confirmed gone with `Test-Path`; the WSL script logged each deletion and refused none.

The largest test-scratch families in the Windows temp folder, counted before the deletion: `delulu-registry-*`
4,419, `delulu-deploy-plan-*` 4,098, `delulu-compute-*` 3,754, `delulu-new-*` 2,751, `delulu_plugin_*` 2,650,
`delulu-cli-contract-*` 2,505, `delulu-fix-*` 2,369, `delulu-many-*` 1,867, `delulu_brokerd_*` 1,849,
`delulu-fleet-cli-*` 1,611, `dlmon-*` 1,429, `delulu-estop-*` 1,416, `delulu-lsp-ws-*` 1,267 — the tests leave
their scratch behind when a run is cut short, and Windows never empties its temp folder by itself. The oldest was of
2026-08-04. The full list of names was not kept; the counts and totals above are the script's own.

## Kept deliberately

- **The repository, its `.git`, its two stashes and every branch**; **`D:\nelan\dl-launch`**, the prepared
  `launch/2.0.0` worktree (only its cache was removed); **`DeluluLang-history-2026-10-17.bundle`** (the backup);
  **`DeluluLang-agent-transcripts`**, **`dogfood-campaign`**, **`worktree-notes-2026-09-17`**, the owner's
  `github-zip` download, and the loose review and logs in `D:\nelan`.
- **`DeluluLang\dist`**, **`editors\vscode\node_modules`** (the extension's working dependency), and the four tiny
  `target` folders of the tier-4 test corpus and `examples/greeter` (a few KB).
- **`C:\Users\jesse\.delulu`** — the broker's keys and audit chain. **`AppData\Local\DeluluLang`** — the autosave
  task's script and index. **`.cargo\bin\delulu.exe`** and the VS Code extension — the installed toolchain, updated
  only on the owner's word (RW 6.53).
- **Toolchains:** `.rustup` (4.39 GB: 1.96.1 pinned, nightly for Miri, stable), `.elan` (6.18 GB: Lean 4.32.2 —
  the version CI checks the proofs with — and 4.35.0, elan's `stable` default), the cargo registry. Removing one is a
  reinstall, not a cleanup.
- **Claude Code's own folders:** transcripts, memory, and the live session folders under `Temp\claude` (only the
  spent binaries above were removed from one old scratchpad).
- **In WSL:** `delulu-f1` and `delulu-linux2` (each holds superseded files `D:` lacks — the 2026-08-09 finding),
  `microvm-image-a`/`-b`, `bin` (Firecracker), `psc`, `agent-lab`, `agent-lab2`, `agent-lab3`, `delulu-dist`,
  `dl-psa-arch` and `dl-mac-harden`'s source, `dl_lab`, `vg`, the kernel and Firecracker tarballs and signing keys in
  `.cache/delulu-microvm`, `.cache/miri`, `~/.cargo`, `~/.rustup`, and the owner's own coursework files. Thirteen
  `/tmp/tmp.*` directories made by `mktemp` were left: nothing ties them to DeluluLang.
- **Outside DeluluLang:** nothing else on `C:` or `D:` was examined for deletion — the other projects on `D:` and
  their zips, `D:\Minidump copy`, `C:\tmp`, Docker's 0.1 GB WSL disk.

## Afterwards

- `cargo build -p delulu -p delulu-survey` on Windows (94 s), so everyday commands start warm; tests rebuild on their
  first run.
- The one warm Linux cache rebuilt: `delulu` and `delulu-survey` into `/home/user/delulu-gnu-target` (92 s, 3.3 GB;
  `delulu --version` answers).
- `fstrim -v /` as root (996.7 GiB trimmed), `wsl --shutdown`, then the compaction — `diskpart /s` with `select
  vdisk file="…\ext4.vhdx"`, `attach vdisk readonly`, `compact vdisk`, `detach vdisk`, started elevated
  (`Start-Process -Verb RunAs`, Windows' UAC prompt): *"DiskPart successfully compacted the virtual disk file."* WSL
  started again with its files in place.
- **Found, not caused, and not changed:** WSL has no `/dev/kvm` now. Its kernel log reads `kvm_amd: SVM not
  supported by CPU 3` although `.wslconfig` sets `nestedVirtualization=true` and Windows reports virtualization
  enabled — Hyper-V is not passing AMD-V through to WSL's VM. Deleting files cannot remove a device node; whether
  `/dev/kvm` was present before the cleanup was not checked. The microVM tests (`--cfg delulu_kvm`) cannot run in WSL
  until it returns. Separately, WSL 2.6.1 warns that `autoMemoryReclaim` and `sparseVhd` are unknown keys under
  `[wsl2]` (they belong under `[experimental]`), so the disk image does not shrink by itself.
- **`/dev/kvm` restored the same evening, at the owner's word** (*"Fix the /dev/kvm issue in WSL"*). The cause was a
  Windows regression, not the laptop's settings: the 2026-10 cumulative update **KB5127648** (installed here on
  2026-10-19, build 26200.9711) breaks the Host Compute Service's `ProcessorCapabilities` query on AMD processors, and
  WSL, catching the error silently, starts its VM without nested virtualization — the guest's `Hyper-V: Nested features:
  0x0`, no `svm`, no warning ([microsoft/WSL#41917](https://github.com/microsoft/WSL/issues/41917); its maintainer, on
  2026-11-02, names the fix **KB5127936**). Memory Integrity was ruled out: the boot log shows VBS with HVCI enabled on
  every boot since 2026-09-19, and KVM worked on 2026-09-26 and on 2026-10-17 all the same. KB5127936 (the 2026-10-28 *Preview* cumulative
  update for 25H2, build 26200.9818) was not offered by Windows Update here, so its two packages were taken from the
  Microsoft Update Catalog (`windows11.0-kb5043080-x64_9534…83e8.msu`, 533,761,740 bytes, and
  `windows11.0-kb5127936-x64_50ca…830b.msu`, 4,898,703,157 bytes), each checked before use — SHA-1 equal to the hash in
  its file name, Authenticode signature *Valid*, signer Microsoft Corporation — and installed elevated with `DISM /Online
  /Add-Package /NoRestart` (*"The operation completed successfully"*, exit 3010). After the owner's restart the build
  read 26200.9818, and in WSL: `Hyper-V: Nested features: 0x4a0000`, `svm` on 4 of 4 CPUs, `kvm_amd: Nested
  Virtualization enabled`, `/dev/kvm` `crw-rw---- root kvm` (the `/etc/wsl.conf` boot command, unchanged), and as the
  ordinary user `KVM_GET_API_VERSION` = 12 and `KVM_CREATE_VM` returning a descriptor. The 5.4 GB of packages were then
  deleted.
- **The microVM images rebuilt and the KVM tests run, the same evening, at the owner's word** (*"rebuild the microVM
  images and run the KVM tests"*). From a fresh clone of `e349208` on WSL's ext4 (`~/delulu-src-img`; libffi's configure
  fails on `drvfs`), `scripts/microvm/check-reproducible.sh` built the image twice from clean in 470 s: kernel
  `f19758e4…`, initramfs `7fc8b289…`, guest binary `cba376cd…`, kernel config `1ee82e55…` — *"two clean builds produced
  identical bytes"*. The first became `~/microvm-image-a`, the second `~/microvm-image-b`, replacing the stale images of
  2026-10-17 and 2026-09-27; the kernel tarball and Firecracker v1.17.0 matched their pins. Then, as CI's `microvm` job
  runs them (`RUSTFLAGS=--cfg delulu_kvm`, `~/delulu-kvm-target`, `--test-threads=2`): on image A, `microvm_cli` 10 of 11
  as the ordinary user — the eleventh, `as_root_the_vmm_is_jailed_as_its_own_uid_and_dies_with_its_host`, failed on its
  precondition, *"sudo: a password is required"* (CI has passwordless `sudo -n`; WSL does not), and **passed** when the
  same test binary ran as root (`wsl -u root`, 3.25 s); `microvm_criterion8` passed; `sandbox probe` answered *"L2
  microvm present — … booted a guest and heard it dial in, in 1340 ms"*; `redteam/probe.sh` and `redteam/hostile.sh`
  each *"every expectation held"* (one vsock device, no block device, IP/packet/Unix sockets refused; garbage, oversized,
  wrong-port and silent guests each refused before any program ran, nothing left behind). On image B, the ten
  non-root lifecycle tests and criterion 8 passed. The reproducibility check's 5.5 GB scratch (two kernel work trees
  and two cargo caches) was then deleted, its 8 `.md` byte-identical to the kernel tarball's or the cargo registry's.

## Result

| | Before | After |
|---|---:|---:|
| `D:\nelan` | 234 GB (251,260,194,091 bytes, Explorer) | **5.31 GB** (robocopy; `DeluluLang\target` 4.53 GB after the rebuild) |
| `D:` free | 16.5 GB (17,697,308,672 bytes) | **235.5 GB** (252,844,716,032 bytes) |
| WSL used (`df /`) | 33 GB | 6.6 GB after the deletions; 9.8 GB after the rebuild |
| `ext4.vhdx` on `C:` | 37.91 GB (37,908,119,552 bytes) | **13.44 GB** (13,444,841,472 bytes) |
| `Temp` on `C:` | 3.10 GB | 0.79 GB |
| `C:` free | 108.0 GB (115,999,539,200 bytes) | **131.5 GB** (141,217,447,936 bytes) |

(GB here are Windows' GB, 2^30 bytes.) **About 242 GB more free space in all — 219.0 GB on `D:`, 23.5 GB on `C:`.**
`D:\nelan` itself shrank by 228.7 GB; the 9.7 GB between that and `D:`'s gain was not traced — the Recycle Bin is
empty, and `D:` keeps restore points (Explorer shows a *Previous Versions* tab), whose storage needs an
administrator to read; nothing else on `D:` was touched. The cache grows again with use, fastest when the
whole suite is built (`cargo test --workspace`). `cargo clean` is always safe in this repository and in
`dl-launch` (CI keeps no state on the laptop), at the price of one full rebuild; the Windows temp folder can be
cleared of `delulu*`/`dl*` entries the same way once no test is running.
