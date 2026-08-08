# Red-team notes — audit chain tampering (agent3)

Binary: `redteam-target\release\delulu.exe`
State dir: `agent3\state` (`DELULU_STATE_DIR`), `DELULU_NO_FIRST_RUN=1`
Broker: started pid 18276, stopped cleanly at end of run.

## Setup — building an honest log

1. `broker start` → ok, guard on, owner code printed once.
2. `grants adopt bundle\legit.dlcert --anchor 31feca86a431ae340059831278232c011daefc57dfd2dfd38a2b8e45e1304035`
   → adopted node `g_feef7c56acfcfc4998a98fcdd5773777` (effects={Actuate}, device sat0/hga slew -45..45).
3. `grants delegate --parent g_feef7c56... --effects Actuate --device "sat0/hga:slew_deg=-10..10,..." --ttl 30m`
   → `g_d2153a4826712b465ecdd7d64606ed1b` (single-redemption token).
4. `grants delegate ... --device "...slew_deg=-5..5..." --ttl 30m --multi`
   → `g_f1bfb22ac6ed54fc632e19d97db87962` (multi-redemption token).
5. `grants revoke g_f1bfb22ac6ed54fc632e19d97db87962` → ok, audit seq 5, epoch 1.

**Deviation from the primer/task script, noted for accuracy:** the primer and the task both say
`delulu grants redeem <token>`. The actual binary has no such subcommand
(`grants --help` → `list | tree | inspect | revoke | delegate | certify | adopt | renew | receipt |
pubkey`). Redemption is actually done via `delulu run <file> --lease <token>` per the delegate
command's own printed instructions. Per the primer's own rule ("trust [--help] over your
assumptions") I did not force a `redeem` call; the 5-record log above (issue, adopt, delegate,
delegate, revoke) was sufficient to exercise the chain. This is a documentation/binary mismatch,
not a security finding.

## Pre-tamper finding: `audit verify`/`tail` default path is NOT the broker's state dir

`delulu audit --help` shows the default `--dir` is `~/.delulu/audit`, not `$DELULU_STATE_DIR/audit`.
Running `delulu audit verify` / `tail` with `DELULU_STATE_DIR` set but **no `--dir`** silently
verified/tailed a completely different, pre-existing global chain (12 unrelated records from
2026-07-26, a different scratchpad session) and reported `ok`. It never errored, never mentioned
the state dir was ignored. My own broker's real chain (5 records, written under
`agent3\state\audit\`) was only reachable via explicit `--dir agent3\state\audit`.

This isn't tamper detection failing — the global chain genuinely was intact — but it's an
operational footgun adjacent to invariant 6 ("Audit truth"): an operator who forgets `--dir` in a
multi-broker/multi-tenant machine gets a confident `ok` that verified the **wrong log entirely**,
not their broker's. Silent wrong-file success is worse than a loud error here. Flagging as a
finding, not filed as a full chain-integrity break.

## Located files

`agent3\state\audit\`:
- `20260808.jsonl` — the chain. Line 1 is an unhashed header (`{"audit":"delulu","day":...}`).
  Lines 2+ are hash-chained JSON records: `seq`, `ts`, `action`, `decision`, `hash`, `prev_hash`
  (blake3), plus `actor_node`/`target`/`authority` as applicable.
- `ANCHOR.json` — `{"head": <hash of last record>, "records": <count>}`, external to the log file.

Honest baseline: 5 records, head `ad2f38cb3771a7e3c1f2ee9e8353c09ee75140819ce7188ce06fe0051ec31152`.
`audit verify --dir agent3\state\audit` → `ok: audit chain verified — 5 record(s) across 1 file(s)`, exit 0.

## Tamper results

| # | Attack | Method | `audit verify` | `audit tail`/`query` (read path) | Invariant |
|---|--------|--------|-----------------|-----------------------------------|-----------|
| 1 | Modify one field | Flipped seq-3 delegate `"decision":"allow"`→`"deny"` in place; left `hash` field stale | **DETECTED.** `error[DL1405]: ... failed at seq 3: record hash mismatch (possible tamper)`, exit 1 | Printed a loud `WARNING: the audit chain does NOT verify ...` banner, still listed records (marked untrusted), exit 1 propagated | **HELD** |
| 2 | Reorder | Physically swapped the seq-3 and seq-4 lines (fields/hashes untouched) | **DETECTED.** `error[DL1405]: ... failed at seq 4: prev_hash chain break: expected fe5cbd6...cde6, found 8a3c663...018a`, exit 1 | (not re-checked separately; same code path as #1/#3) | **HELD** |
| 3 | Truncation | Deleted last 2 records (seq 4, 5), left `ANCHOR.json` at stale `{"head":...ad2f38...,"records":5}` | **DETECTED.** `error[DL1405]: ... anchor disagrees with the log: anchor says 5 record(s) ending ad2f38cb..., the chain holds 2 ending fe5cbd68... — records have been removed, replaced or rolled back`, exit 1 | Same loud `WARNING` banner + untrusted listing, exit 1 | **HELD** |
| 4 | Consistent rewrite (documented limit) | Kept the seq 4/5 truncation, then rewrote `ANCHOR.json` to `{"head":"fe5cbd68...cde6","records":2}` — i.e. forged both halves in agreement | **NOT DETECTED** (as documented/expected). `ok: audit chain verified — 2 record(s) across 1 file(s), head fe5cbd68...`, exit 0 | `tail`/`query` show only the 2 surviving records, no warning, exit 0 — the 3 deleted records (including the revoke) are gone with no trace inside this state dir | **Confirmed known limit — not a new finding.** No external witness exists in this setup to catch a consistent rewrite of log+anchor together. |

Read-path behavior (`audit tail`, `audit query`) on a **detectably** tampered log: both re-run
verification internally before printing, surface the same `DL1405` diagnosis, prefix output with an
explicit "MUST NOT be trusted" warning, and still exit nonzero — they do not silently present forged
data as authentic. Only the undetectable case (#4) shows clean output, which is expected: there is
nothing left in the log+anchor pair itself to contradict.

After each tamper, the honest baseline was restored from a backup and re-verified
(`ok: ... 5 record(s) ..., head ad2f38cb3771a7e3`) before moving to the next attack. Final state
left honest; broker stopped (`ok: broker shutdown requested`); temp `.honest` backup files removed.

## Summary

- Invariant 6 ("Audit truth"): **HOLDS** for single-sided tamper (content edit, reorder, truncation
  vs. stale anchor) — all three were caught by `verify` with a specific, correct diagnosis (which
  seq, what kind of break) and exit 1, and the read paths refuse to present tampered data as clean.
- The one case that passes tampered (#4, consistent log+anchor rewrite) is the documented limit,
  confirmed, not a new hole — closing it requires an external witness (a second custodian, remote
  anchor pin, etc.), which this broker deployment doesn't have.
- One real operational finding outside the four requested attacks: **`audit verify`/`tail`/`query`
  do not default to `$DELULU_STATE_DIR/audit`** — omitting `--dir` silently targets
  `~/.delulu/audit` instead, which on a multi-broker machine can return a confident `ok` for the
  wrong chain entirely. Worth a docs/UX fix (e.g., default `--dir` to
  `$DELULU_STATE_DIR/audit` when that env var is set, or refuse to guess and require `--dir`
  explicitly).
