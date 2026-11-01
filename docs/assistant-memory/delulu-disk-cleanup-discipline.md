---
name: delulu-disk-cleanup-discipline
description: "How Jesse wants junk/disk cleanups run for DeluluLang — no .md deletions, confirm gate, keep build caches, verify WSL against D: HEAD; 2026-08-09 reclaimed ~71 GB"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: c99ba5c5-c39b-4a1f-b406-92fbf2c601b0
  modified: 2026-10-17T10:53:12.202Z
---

When Jesse asks to delete junk/useless files on C:/D: for this project, follow this discipline
(established 2026-08-04, refined 2026-08-09):

1. **Never delete any `.md` file, anywhere** — including tiny drafts inside finished scratchpads or
   old snapshots. Scan for `.md` in every delete target first and preserve them (clean a mixed dir
   file-by-file, don't remove it wholesale).
2. **Double-verify and confirm at a gate before PERMANENT deletion.** Investigate read-only, present
   categorized buckets with sizes, and get explicit confirmation (AskUserQuestion) before removing
   anything. Permanent deletion is not covered by the overnight "auto-continue" mandate.
3. **Keep all build caches unless he says otherwise.** `target\debug\deps` (~70 GB) is an owner
   decision, not a cleanup decision — deleting forces a full wasmtime rebuild. Delete only
   `incremental` and spent temp by default; keep one warm target dir on each platform.
4. **WSL: never sweep unattended; clean only with verification.** The ext4 `target` dirs and clones
   inside the vhdx are the Linux build config (the 08-04 "deletion undone" lesson). They CAN be
   cleaned (done 2026-08-09): classify each dir (`CACHEDIR.TAG` ⇒ pure cargo cache, safe;
   `Cargo.toml`+`crates/` ⇒ source-bearing), **content-hash every source-bearing dir against `D:`
   HEAD** (08-04 method — `git hash-object --stdin-paths | git -C /mnt/d/... cat-file --batch-check`,
   CR-normalize the misses), preserve any file whose blob is absent from `D:`, and keep one warm
   target. Then reclaim to the host with the fstrim→compact sequence below.
5. **Read crash dumps before deleting them**; record their signatures so evidence outlives the bytes.
6. **Record every deletion** to `docs/maintenance/DISK-CLEANUP-<date>.md`, referencing the prior one.

**Why:** the 08-04 pass wrongly deleted needed files (the WSL ext4 targets) and older `.md`
snapshots; Jesse added "do not delete any .md" mid-task on 08-09 in direct response.

**2026-08-09 outcome:** ~71 GB reclaimed — **C: 83 → 148 GB free, D: 212 → 218**, `ext4.vhdx`
70.8 → 21.6 GB. Removed Windows temp/scratch + `target\debug\incremental` (~21.6 GB) and, inside WSL,
redundant target caches + `delulu-fed` (488/488 in D:) + `dl-before` (573/573, HEAD `0c98a58` in D:)
(~49.7 GB). **WSL /home/user now holds only:** `delulu-target` (13 GB — the warm ext4 Linux build
cache; reuse via `CARGO_TARGET_DIR=/home/user/delulu-target`), plus preserved *superseded* snapshots
`delulu-f1` (source; an old 6,760-line `cli.rs` not in D:) and `delulu-linux2` (holds an old
`STAGE10_BUILD_ORDER.md`) — **these are NOT junk, they carry unique bytes**; and unrelated
OS-coursework files. Full record: `docs/maintenance/DISK-CLEANUP-2026-08-09.md`.

**2026-10-17 outcome:** `D:\nelan` had reached 180 GB — 178.4 GB of it `DeluluLang\target` (cargo never removes
superseded artifacts: `debug\deps` 127 GB, `incremental` 42 GB). At the gate Jesse chose **"Delete all, rebuild
delulu"** → `cargo clean` (36 s, 320,697 files) + `cargo build -p delulu -p delulu-survey` (86 s); also deleted the
two P5 dry-run clones `delulu-build-5065be1`/`-p5b` (verified clean, HEADs in master, dist .md = git objects) and
`delulu-build-target`. D: free 84 → 249 GB; `D:\nelan` 5.07 GB. Record: `docs/maintenance/DISK-CLEANUP-2026-10-17.md`
(new records go in `docs/maintenance/`, listed in REPOSITORY_STRUCTURE §5 — the old two are archived). WSL's
`ext4.vhdx` on C: was 57.9 GB → Jesse: "clean it" → same method as 08-09 (caches 45 GB, 3 verified clones, kernel
build tree, 5,098 /tmp/delulu-* dirs; warm `~/delulu-gnu-target` rebuilt; fstrim; shutdown; diskpart compact
started elevated via `Start-Process cmd -Verb RunAs -Wait` with output redirected to a log — worked) → vhdx
12.84 GB, C: free 100.7 → 147.5. Trap: git run AS ROOT on user-owned clones declines them (dubious ownership) →
a guard reading `git status` refuses; run clone checks as the owning user. Measure fast with `robocopy <dir> C:\__x /L /S /NFL /NDL /NJH
/BYTES /XJ` (MSYS `du` over 360k files timed out at 10 min).

**How to apply:** the command-safety scanner blocks `Remove-Item` with *variable* paths (reads a
drive letter like `D:` as the target) or arithmetic near it (`/1MB,2`). Issue deletes with **full
inline literal paths, no adjacent math**. Run WSL work via **script files** (`wsl.exe -- bash x.sh`),
never inline `bash -lc '...'` (PowerShell/bash quoting collides). To reclaim WSL disk to the host the
order is **delete inside → `fstrim -v /` → `wsl --shutdown` → `diskpart compact` (elevated)**;
compaction on a used-up guest before deleting+trimming is a **no-op** (verified 2026-08-09: first
compact reclaimed 0; the full sequence then reclaimed 49.8 GB). Never delete `.claude\worktrees\`
([[delulu-stale-worktree]]), transcripts, or memory. Discovery-over-checklist still applies
([[discovery-over-checklist]]).


## The index line this entry had in MEMORY.md until 2026-10-20 (kept whole)

- [**Disk-cleanup discipline**](delulu-disk-cleanup-discipline.md) — **never delete any `.md`**; confirm at a gate before permanent deletion; keep build caches + one warm target per platform; record to `docs/maintenance/`; **2026-10-17: Jesse chose `cargo clean` of the 178 GB `target` (D:\nelan 180 → 5 GB)**. WSL cleaned only with content-hash verification vs D: HEAD. Ops: inline literal paths (the scanner blocks variable-path `Remove-Item`); reclaim = delete→`fstrim`→`wsl --shutdown`→`diskpart compact`.
