# Disk cleanup — 2026-10-17

A record of what was deleted from `D:\nelan` and, the same day, from WSL's disk, what was kept, and **how each deletion was checked before it
happened**. Same discipline as the two earlier passes, now in the archive:
[`DISK-CLEANUP-2026-08-04.md`](../archive/v1/maintenance/DISK-CLEANUP-2026-08-04.md) and
[`DISK-CLEANUP-2026-08-09.md`](../archive/v1/maintenance/DISK-CLEANUP-2026-08-09.md).

This is an operations record, not a product document. Nothing here changes DeluluLang's behaviour, and
nothing tracked by git was touched.

## Why it happened

Back from the cloud period, the owner saw `D:\nelan` at **180 GB** (Explorer: 193,992,521,581 bytes,
360,495 files, 26,943 folders) while a download of the GitHub repository is about 6 MB, and asked:
*"Why size of "D:\nelan" is 180 GB … While when I download delululang-test GitHub repo it is only 6 mb.
Delete unwanted things. Log things deleted in .md file"*.

**The answer: 178 GB of it was one folder, `DeluluLang\target` — Rust's build cache.** GitHub holds only
the source; the cache is what `cargo` writes when it builds. Cargo never removes an artifact it has
superseded, so every rebuild since July — 163 test binaries, wasmtime, several toolchains, the 281
commits of the cloud period built at once on 2026-10-17 — left its old copies beside the new ones.
Measured (robocopy list mode, bytes):

| Folder | Size | What it was |
|---|---:|---|
| `DeluluLang\target\debug\deps` | 127.08 GB | compiled crates and test binaries, mostly superseded copies |
| `DeluluLang\target\debug\incremental` | 42.20 GB | recompile cache |
| `DeluluLang\target\release` | 5.22 GB | release build of 2026-09-27 (P5) |
| `DeluluLang\target\debug\build` | 1.72 GB | build-script outputs |
| `DeluluLang\target\x86_64-unknown-linux-musl` | 0.54 GB | the microVM guest's cross build, 2026-09-27 |
| `DeluluLang\target\*-apple-darwin`, `miri`, `x86_64-unknown-linux-gnu` | 0.52 GB | cross and Miri builds, July–August |
| **`DeluluLang\target` in all** | **178.41 GB** | |
| everything else in `DeluluLang` (source, docs, `.git` 0.03 GB, `fuzz`, `editors`, `dist`) | about 0.5 GB | |
| `D:\nelan\delulu-build-target` | 1.44 GB | the build cache of the two clones below |
| `D:\nelan\DeluluLang-agent-transcripts` | 0.18 GB | agent-pass records — **kept** |
| `D:\nelan\delulu-build-5065be1`, `D:\nelan\delulu-build-p5b` | 0.16 GB each | two clones made for the P5 release dry run |
| `D:\nelan\fmtcheck_scratch` | 0 bytes | an empty scratch folder of 2026-09-20 |

## The owner's choices (at a confirmation gate, before anything was removed)

- **The build cache:** *"Delete all, rebuild delulu"* — the whole of `target`, then a rebuild of the
  `delulu` and Survey binaries only, so everyday commands stay warm; tests rebuild on their first run.
- **The rest:** delete the two build clones, their build cache, and the empty scratch folder.

## How "safe to delete" was established

1. **No build was running** — no `cargo`, `rustc`, `delulu` or `rust-analyzer` process — and the repository's
   working tree was clean at `ed120f1`, equal to `origin/master`.
2. **`.md` files were found first** (the standing rule: never delete a `.md`). In the caches: three, all
   `libffi-build/README.md` under `libffi-sys`'s build output — each **byte-identical** (`cmp`) to
   `libffi-sys-2.3.0/libffi/README.md` in the cargo registry, which keeps it and regenerates them. In the
   two clones: 212 tracked `.md` each, plus 6 in each ignored `dist\` — see 3.
3. **The two clones held nothing the repository lacks.** Each was a full clone of this checkout (`origin` =
   `D:/nelan/DeluluLang`), clean (`git status --porcelain --ignored` showed only the ignored `dist/`), no stash,
   and every branch tip present in this repository. Their HEADs — `5065be1` and `5e2c8fd` — are ancestors of
   `master`. Each `dist\` (41 MB, the packaged `delulu-1.0.0-x86_64-pc-windows-msvc`) held 6 `.md` whose
   git blob hashes are all objects in this repository.
4. **`target` and `delulu-build-target` are pure caches** (`CACHEDIR.TAG`); nothing in them is source.

## Deleted

| Path | Size | How |
|---|---:|---|
| `D:\nelan\DeluluLang\target` | 178.41 GB | `cargo clean` |
| `D:\nelan\delulu-build-target` | 1.44 GB | `Remove-Item -LiteralPath … -Recurse -Force` |
| `D:\nelan\delulu-build-5065be1` | 0.16 GB | the same |
| `D:\nelan\delulu-build-p5b` | 0.16 GB | the same |
| `D:\nelan\fmtcheck_scratch` | 0 | the same |

Each path was confirmed gone with `Test-Path`.

## Kept deliberately

- **`D:\nelan\DeluluLang-agent-transcripts`** — the agent passes' briefs, logs and verdicts (`HANDOFF.md` §11.3).
- **`DeluluLang\.claude\worktrees\`** — never deleted without asking (`HANDOFF.md` §11.3), and 0.02 GB.
- **`DeluluLang\dist`**, **`fuzz`**, **`editors\vscode\node_modules`** — small, and the last is the extension's
  working dependency.
- **The cargo registry and toolchains** (`C:\Users\jesse\.cargo`, `.rustup`). WSL was cleaned afterwards, at
  the owner's word — the next section.

## Result

`cargo clean` reported *"Removed 320697 files, 178.4GiB total"* (36 s). Then `cargo build -p delulu -p
delulu-survey` (86 s) and `cargo test -p delulu --test repository_structure` (4 passed) warmed the cache again.

| | Before | After the deletions | After the rebuild |
|---|---:|---:|---:|
| `D:\nelan` | 180 GB (193,992,521,581 bytes) | — | **5.07 GB** (5,441,958,554 bytes; `target` 4.45 GB) |
| `D:` free | 84 GB | 253 GB | 249.1 GB |

**About 179.6 GB reclaimed.** The cache will grow again with use, and fastest when the whole suite is built:
`cargo clean` is always safe in this repository (CI keeps no state on the laptop), at the price of one
full rebuild.

## WSL — the same day, at the owner's word

The owner, told that WSL's disk image on `C:` had grown to 57.9 GB (21.6 GB after the 2026-08-09 compaction):
*"clean it"*. The 2026-08-09 method, unchanged: measure read-only (as root, `du -x`), classify each directory
(`CACHEDIR.TAG` ⇒ a pure cargo cache; `Cargo.toml`/`crates/`/`.git` ⇒ source-bearing), verify every
source-bearing directory against `D:`, delete with a self-guarding script, keep one warm target, then
`fstrim` → `wsl --shutdown` → compact. The owner's choices for `D:` an hour earlier (every build cache, a warm
rebuild of `delulu`, the spent clones) were applied to the same categories here; everything else was kept.

**Measured:** `df /` 55 GB used. `/home/user` 52.8 GB, of which cargo caches 46.1 GB (counting the `target` inside one clone); `/tmp` 0.8 GB (5,098
`delulu-*` test scratch directories); `/usr` and `/var` 2.1 GB (the system — kept).

**How the clones were checked** (run as the owning user; the first attempt ran as root, where git declines a
repository another user owns — the script's guard REFUSED all three and deleted nothing, as it should):
each clean (`git status --porcelain` empty), no stash, HEAD an ancestor of `D:`'s `master`, every branch and
tag tip an object in `D:`. Every file outside `.git` and `target` was hashed against `D:`'s object store:
`delulu-src-img` 951 files, all present; `delulu-src-5065be1` and `delulu-src-p5b` 983 and 981 files, all
present but five each — the packaged release under the ignored `dist/` (`bin/delulu`, the `.tar.gz`, its
`.sha256`, `SHA256SUMS`, a generated `INSTALL.txt`), build output the release workflow regenerates; no `.md`.

**Deleted:**

| Path in WSL | Size | Basis |
|---|---:|---|
| `/home/user/delulu-target` | 23,598 MB | pure cache (`CACHEDIR.TAG`), 2026-09-26 |
| `/home/user/delulu-gnu-target` | 11,528 MB | pure cache — rebuilt afterwards, below |
| `/home/user/delulu-kvm-target` | 4,268 MB | pure cache (the `--cfg delulu_kvm` build) |
| `/home/user/delulu-img-target` | 3,455 MB | pure cache (the guest image's build) |
| `/home/user/dl-ps0-target` | 1,019 MB | pure cache, 2026-09-18 |
| `/home/user/delulu-miri`, `/home/user/delulu-miri-target` | 913 MB | pure Miri caches (`miri/CACHEDIR.TAG`) |
| `/home/user/delulu-linux2/fuzz/target` | 376 MB | pure cache inside a preserved snapshot — **its source kept** |
| `/home/user/.cache/delulu-microvm/work` | 1,784 MB | the unpacked `linux-6.18.54` build tree `build-image.sh` makes; the pinned tarballs beside it kept |
| `/home/user/delulu-src-img` | 1,120 MB | clone at `262d5c4` (977 MB of it its `target`) — verified above |
| `/home/user/delulu-src-5065be1`, `/home/user/delulu-src-p5b` | 383 MB | clones at `5065be1`, `5e2c8fd` — verified above |
| `/tmp/delulu-*` (5,098 entries), `/tmp/dl-dist` | 732 MB | spent test scratch and an old packaged build |

**Kept:** `delulu-f1` and `delulu-linux2`'s source (each holds superseded files `D:` lacks — the 2026-08-09
finding), `microvm-image-a`/`-b` (34 MB each — but stale: the host refuses them since the channel became `/3`,
and the clone and build tree they were made from are gone now; rebuild from a fresh clone on the Linux
filesystem, `HANDOFF.md` §11.3), `bin` (Firecracker), `psc`, `agent-lab*`, `dl-*` labs, `delulu-dist`, the
kernel and Firecracker tarballs in `.cache/delulu-microvm`, `.cache/miri`, `~/.cargo`, `~/.rustup`, the owner's
own coursework files, and `.aws`, `.azure`, `.docker`.

**Then:** `delulu` and `delulu-survey` rebuilt into `/home/user/delulu-gnu-target` (111 s, with `survey check`
ok, `survey findings` 0 errors, `doctor --check` 26 checks passed); `fstrim -v /` (991.6 GiB trimmed);
`wsl --shutdown`; and the compaction — `diskpart /s` with `select vdisk file="…\ext4.vhdx"`, `attach vdisk
readonly`, `compact vdisk`, `detach vdisk`, started elevated (`Start-Process -Verb RunAs`, Windows' UAC prompt): *"DiskPart
successfully compacted the virtual disk file."* WSL started again afterwards with its files in place,
`/dev/kvm` present and `delulu --version` answering.

| | Before | After |
|---|---:|---:|
| WSL used (`df /`) | 55 GB | 6.5 GB; 9.7 GB after the rebuild |
| `ext4.vhdx` | 57.97 GB | **12.84 GB** |
| `C:` free | 100.7 GB | **147.5 GB** |

**About 45 GB returned to `C:`.**
