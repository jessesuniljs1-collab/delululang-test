# Red-team P21 — the cross-OS-account boundary, tested with a real second UID (2026-08-08)

Two prior findings — DISC-1 (a same-user agent can mint a root and command a guard-sealed
device) and IPC-1/DEADMAN-1 (a same-uid stall can hang custody and the dead-man) — are both
category-7 and both rest their honest residual on **one** claim that had never been tested:
*"the boundary is a separate OS account."* Every test to date ran same-uid on a single Windows
account, so that claim was asserted, not falsified.

This pass tests it. WSL gives a **real** second uid, so for the first time a *different user*
either gets stopped (verifying the claim) or gets through (a finding). This is the discovery
mandate applied to the residual, not checklist-painting: *what assumption have we not attacked yet?*

- **Environment:** WSL2 Ubuntu-20.04. Broker runs as `user` (uid 1002); the adversary is
  `attacker` (uid 1003) — a genuinely separate account, not a same-uid process.
- **Binary under test:** `delulu 1.0.0`, built in WSL from this tree, installed at
  `/usr/local/bin/delulu`.
- Scripts: `scripts/scenario1.sh`, `scripts/scenario2.sh`. Raw machine output:
  `evidence/scenario1-ext4.txt`, `evidence/scenario2-9p.txt`.

## Scenario 1 — POSIX filesystem (ext4). The boundary HOLDS.

Broker state dir on `/tmp` (ext4), so the `0700` the broker sets is the *only* thing between the
attacker and the socket. Every vector run as `attacker` (uid 1003), then the same vectors as
`user` (uid 1002) as the discrimination control.

The crux, straight from the run: `dir mode=700 owner=user`, `broker.key mode=600 owner=user` —
the hardening actually stuck.

| Vector | `attacker` (1003) — separate account | `user` (1002) — control |
|---|---|---|
| `ls` the state dir | **Permission denied** (rc=2) | ✓ lists 7 entries |
| `cat broker.key` (lease-MAC secret) | **Permission denied** (rc=1) | ✓ 32 bytes |
| `grants list` over IPC | **DL1401 unreachable: Permission denied** — fail-closed, *"invariant 27: grants verbs never fall back to local state"* | ✓ round-trip: `(no grants — the tree is empty)` |
| raw `AF_UNIX` connect | **PermissionError [Errno 13]** | ✓ CONNECTED |
| `kill` the broker pid | **Operation not permitted**; broker still alive | — |

A separate OS account is denied on all five vectors; the same-uid control succeeds on all four,
proving the test keys on UID and not on a wrong path. **The category-7 residual "needs a separate
OS account" is empirically verified on a POSIX filesystem.** Incidental corroboration: the client
fails *closed* and refuses to fall back to local state when it cannot reach the broker.

## Scenario 2 — 9p (a Windows-mounted volume). The boundary is ABSENT. (finding P21-F1)

Same test with the state dir on `/mnt/d` (a 9p mount — how WSL exposes a Windows drive). Two
distinct results:

1. **The permission hardening is a silent no-op.** `chmod 700` on a dir and `chmod 600` on a file
   both read back as `mode=777`, and `attacker` read the "0600" `probe.txt` (`topsecret`) *and* the
   broker's freshly-written 32-byte `broker.key`. Every `set_permissions` delulu relies on is
   discarded (`let _ = …`), so the program cannot tell the hardening failed. Any secret delulu
   writes to a non-POSIX filesystem — 9p/DrvFs under WSL, and by the same mechanism NFS without
   mapping, SMB/CIFS, exFAT/FAT — is world-readable to other local users.
2. **The broker cannot bind its socket there anyway:** `broker serve loop failed: Operation not
   supported (os error 95)` — 9p does not support `AF_UNIX` bind. So it fails to *serve* on a
   Windows drive — but only after writing that world-readable `broker.key`. (This is why the
   attacker's socket connect and `grants list` return ENOENT: no socket was ever created.)

This is a **deployment hazard**, not a same-account break: WSL's default `/home` is ext4, so the
default posture is Scenario 1. It bites only if an operator deliberately places delulu state on a
Windows-mounted (or other non-POSIX) volume — e.g. to share `~/.delulu` between Windows and WSL,
which would expose the DISC-1 anchor **private** key written by `delulu keygen`. Category 7: an
OS/filesystem property outside the proof boundary.

## What ships in response

The filesystem cannot be fixed from inside delulu, but the *silence* can. The hardening commit
makes the swallowed `set_permissions` observable: after setting owner-only perms, delulu re-reads
the achieved mode and, if group/other bits remain, warns loudly that this filesystem does not
enforce POSIX permissions and that secrets here are exposed. Applied at the two security-critical
write points in the `delulu` crate — the broker state directory (`crates/delulu/src/broker_transport.rs`,
which covers `broker.key`, `secrets.json` and the audit log inside it) and the `keygen` private
key (`crates/delulu/src/signing.rs`). Witnessed: silent on ext4 (Scenario 1 re-run), warns on 9p
(Scenario 2 re-run). See the P21 hardening commit and `docs/MATHEMATICS.md`.

## Honest residuals (unchanged, now precisely bounded)

- The separate-account boundary is **verified on POSIX filesystems** and **absent on non-POSIX
  ones** — the warning makes the latter observable but does not create a boundary where the
  filesystem provides none.
- Within a *single* OS account, everything the earlier passes said still holds: a same-uid process
  can read the same files and signal the same daemon. That is the documented threat-model line
  (`crates/delulu-broker/src/lease.rs`: "the broker does not defend against the same OS user"), and
  the airtight answer to a co-resident adversary remains a separate OS account — which this pass
  shows is a real boundary when the state lives on a real filesystem.

## Reproduce

```
# in WSL, as root, with delulu built and users `user`/`attacker` present:
wsl -d Ubuntu-20.04 -u root bash scripts/scenario1.sh   # expect: boundary HELD
wsl -d Ubuntu-20.04 -u root bash scripts/scenario2.sh   # expect: LEAK + bind-fail on 9p
```
