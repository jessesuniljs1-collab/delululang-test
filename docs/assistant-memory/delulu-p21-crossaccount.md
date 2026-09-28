---
name: delulu-p21-crossaccount
description: "P21 DONE (2026-08-08, commits 4e9ea98 + b6fac61 + 33056f1): tested the 'separate OS account' boundary that the DISC-1 and IPC-1/DEADMAN-1 residuals rest on, using WSL's real second UID. Verified it HOLDS on a POSIX filesystem; found it is ABSENT on 9p (finding P21-F1: chmod is a silent no-op there); then FAIL-CLOSED it — keygen and the broker now refuse to write a secret before it lands on a filesystem that can't keep it owner-only. Carries the reusable WSL-harness lessons."
metadata: 
  node_type: memory
  type: project
  originSessionId: c99ba5c5-c39b-4a1f-b406-92fbf2c601b0
  modified: 2026-09-16T20:40:05.517Z
---

**P21 COMPLETE — 2026-08-08, Opus 4.8, max effort. Three commits on `master` (local, NEVER pushed):
`4e9ea98` (verify + finding + survey-cite fix), `b6fac61` (warn on the silent no-op), and `33056f1`
(FAIL-CLOSED — the real fix). Full evidence + reproducible scripts:
`docs/security/red-team-p21-crossaccount-2026-08-08/`.** Jesse's ask: "continue to next phase", "fix
survey if needed", "use wsl or docker" to TEST the category-7 residual instead of documenting it — then
"check if this can be fixed" (the warn). Did all of it.

## What was proven (the discovery mandate applied to a residual)
DISC-1 ([[delulu-root-issuance-bypass]]) and IPC-1/DEADMAN-1 ([[delulu-ipc-deadman-findings]]) both rest
their honest residual on ONE claim that had only ever been asserted: *"the boundary is a separate OS
account."* WSL gives a real second uid, so it became falsifiable. Broker ran as `user` (1002); adversary
was `attacker` (1003).

- **Scenario 1 — ext4 (/tmp): the boundary HOLDS.** `attacker` denied on all five vectors — dir
  traversal, `broker.key` read, `grants list` over IPC (fail-closed `DL1401`, and it explicitly *never
  falls back to local state*, "invariant 27"), raw `AF_UNIX` connect, and `kill` (Operation not
  permitted). The same-uid control (`user`) succeeded on all four, proving the test keys on UID. Crux
  evidence: `dir mode=700`, `broker.key mode=600`, both `owner=user`. So the residual is now
  **verified-by-demonstration** (a reproducible red-team differential, not a proof — the guarantee is
  still category 7 because it is an OS/filesystem property, not delulu code).
- **Scenario 2 — 9p (/mnt/d, a Windows-mounted volume): the boundary is ABSENT. FINDING P21-F1.**
  `chmod 700/600` is a **silent no-op** on 9p (reads back `777`); `attacker` read a "0600" file
  (`topsecret`) AND the broker's 32-byte `broker.key`. And the broker cannot even bind its `AF_UNIX`
  socket on 9p — `Operation not supported (os error 95)` — so it fails to serve there, but only after
  writing the world-readable key. Root cause: every `set_permissions` was discarded (`let _ = …`), so
  delulu could not tell the hardening failed. Deployment hazard (the default WSL `/home` is ext4, so the
  default posture is Scenario 1); by the same mechanism it applies to NFS-without-mapping, SMB, exFAT/FAT.

## The fix — TWO layers, both unix-only (Windows ACL hardening stays the documented v0.8 gap)
**Layer 1, warn (b6fac61):** `crates/delulu/src/signing.rs` `set_owner_only_or_warn(path, mode, what)` —
set owner-only perms, re-read the achieved mode, `eprintln!` a warning if group/other bits remain. Pure
predicate `owner_only(mode) = mode & 0o077 == 0`, unit-tested. But it fires AFTER the secret is already
on disk world-readable — Jesse's follow-up "check if this can be fixed" was right to push past it.
**Layer 2, FAIL-CLOSED (33056f1) — the real fix:** `dir_enforces_owner_only(dir)` probes with a
throwaway non-secret file (create 0600, read the mode back, delete) BEFORE any secret is written;
`perms_unenforced_refusal(...)` turns `Ok(false)` into an operator-facing refusal. Wired so **`keygen`
refuses to write the private key** (override `--dangerously-allow-insecure-perms`) and **the broker
refuses to start** in `brokerd.rs::serve_inner`, before `load_or_create_key` (NO override — a custody
daemon must not hold world-readable secrets; 9p can't bind its socket anyway). The warn is now a
post-write backstop under the override. **Witnessed on 9p: both REFUSE and write nothing — the broker no
longer even creates `broker.key` (it used to write a world-readable key then fail to bind); on ext4 both
proceed; the override proceeds + warns.** Verification (both commits): Windows delulu bin 69/0 +
doctor_cli 7/0 + evidence 3/0; Linux (WSL) bin 74/0 (2 `p21_perm` tests); clippy clean; Survey 0/0.
`docs/MATHEMATICS.md` §12 item 13 records it without inflating the category (still cat-7: the guarantee
is an OS/filesystem property — fail-closed refuses to trust a filesystem that can't provide it).

## Survey (Jesse's "fix survey if needed" + "use survey")
`cargo run -q -p delulu-survey -- findings` showed **9 `prose-cites-missing-path` warnings**, all in the
2026-08-08 red-team agent notes — my OWN `0d06a4e` fix had shifted the lines the Haiku agents cited
pre-fix, plus one wrong-crate cite (`device.rs` is only in `delulu-runtime`) and two wrong-file cites
(`tests/lib.rs` → the tests are inline in `src/lib.rs`). Fixed by correcting each path and rendering the
line ranges as prose (a bare `file:line-RANGE` is what the map can't resolve; a single `:line` it can),
plus a provenance note per file. My prior "just cosmetic code symbols" hand-wave was wrong. Now 0
warnings (3 tolerated-by-design notes remain: Lean C-tokens, bare `D<n>` convention, README test-count).

## Reusable WSL-harness lessons (cost ~half of the first sitting — DON'T relearn)
- **Launch wsl from PowerShell, NOT the Bash tool.** Git Bash (MSYS) rewrites `/mnt/c/...` args into
  `C:/Program Files/Git/mnt/c/...` and mangles inline `VAR=` assignments.
- **Script files ONLY**, never inline `bash -lc '...'` across the boundary (it drops assignments and
  truncates at spaces/quotes). Write to scratchpad, run `wsl -d Ubuntu-20.04 -u root bash /mnt/c/.../x.sh`.
- **cargo belongs to `user` (uid 1002), not root.** Build/test as user: `runuser -l user -c "cd
  /mnt/d/nelan/DeluluLang && export CARGO_TARGET_DIR=\$HOME/delulu-target && cargo …"`. Binary at
  `/home/user/delulu-target/debug/delulu`; install to `/usr/local/bin/delulu` for both uids.
- **`delulu` is a BINARY-only crate** — `cargo test -p delulu --lib` errors "no library targets"; use
  `--bin delulu`. Unix-only unit tests are gated `#[cfg(all(test, unix))]` (run in WSL, skipped on Win).
- `/home` = ext4 (honors chmod); `/mnt/*` = **9p** (chmod silently ignored — the P21-F1 surface).
- `wsl -u root` / `-u user` switch uid without sudo; `runuser -u <name> --` runs one command as a uid.
  `DELULU_STATE_DIR` points broker+client at the state dir; socket = `<state>/broker.sock`;
  `delulu grants list` = a real IPC round-trip (the cross-uid custody probe).

## Residual after P21 (precisely bounded, do not oversell)
The separate-account boundary is **verified on a POSIX filesystem** and **absent on non-POSIX ones** —
the warning makes the latter observable but creates no boundary where the filesystem provides none.
Within a single OS account nothing changed: a same-uid process still reads the same files and signals
the same daemon (the documented `crates/delulu-broker/src/lease.rs` line, spec §10). Related standing
rules unchanged: consult the Survey `rdeps` before code and regenerate after ([[delulu-survey-map]]);
harden-never-redefine Authority/Guard; push only to the testing remote (public since 2026-09-17; the
NEVER-push rule was superseded 2026-09-14 — [[delulu-github-remote]]).
