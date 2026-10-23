# Agent findings — the head chef's verification ledger (2026-10-24)

Every agent claim is UNVERIFIED until re-run here against the binary. Status: **CONFIRMED** (reproduced by the head
chef), **FIXED** (with branch/PR), **DOCUMENTED** (a known, recorded limit), **NOT REPRODUCED**, **PENDING**.

## Agent C — zero authority (Haiku 5.5)
| ID | Claim | Status |
|---|---|---|
| C1, C2, C3 | Generic body decides its own parameters: `coerce`/`hide`/`forge` launder an effect, forge a cap | **CONFIRMED → FIXED** GENERIC-RIGID-1, D-V2-184, RW 4.81, `laptop/generic-rigid`, PR #66 |
| C4 | Top-level `let` value never type-checked or name-resolved | **CONFIRMED** (`let Y: Int = "not an int"` checks clean and runs) **→ FIX in progress** CONST-UNCHECKED-1, `laptop/const-check` |
| C5 | `authority` omits a top-level `spawn` (Async) | **CONFIRMED** as part of C4 (a constant's value was never held to a row) → same fix |
| C6–C16 | diagnostics/doc papercuts | PENDING |

## Agent B — least authority (Sonnet 5.5)
| ID | Claim | Status |
|---|---|---|
| B1 | A lease cannot carry `net.special`; `delegate --net 127.0.0.1` is accepted and can never connect | PENDING (setup consequence confirmed: B's net leases never connected) |
| B2 | `Secret.map` + `verify` is a chosen-prefix oracle | **DOCUMENTED** — P17-IF1 (`crates/delulu-check/tests/secret_oracle.rs`), open as RW 2.6: `verify` declares Declassify but needs no capability. Not new. |
| B3 | No port dimension in net grants | DOCUMENTED (D-V2-182: a `net` entry names no port) — re-read |
| B9 | `--within` says within for a literal use path escaping its capability; run refuses DL0904 | **CONFIRMED** (fsr.delulu) |
| B10 | Two doc contradictions (`--sandbox` + `--lease`; WASM secrets "not yet") | **CONFIRMED** both: `dl run B-read progs/fsr.delulu --sandbox` runs, the lease judges the guest (`guest: error[DL0904]: lease … does not grant …secret.txt`), while for-agents.md:658 says the pair is refused; GETTING_STARTED.md:320 and for-agents.md:720 say secrets are "not yet" on `--engine wasm` (they run there since D-V2-174 / 2026-10-23). Doc fix pending. |
| B4–B8, B11 | message papercuts | PENDING |
Proposal `pp_0000` (B-console asks fs.read=allowed): **rejected** by the head chef with a reason — the proposal flow
(P9-03) worked end to end under strict anchored roots.

## Agent A — the builder (Sonnet 5.5)
| ID | Claim | Status |
|---|---|---|
| A2 | DL1001's repair `insert_dependency_pin` writes TOML that cannot parse (inline table + `[dependencies.X.authority]`) | PENDING (agent's evidence detailed and reproduced twice) |
| A3 | `x % 0`: interpreter DL0902, WASM DL0901 | **CONFIRMED** (ovf/o8.delulu). Cause: `chk_rem_fn` traps `b == 0` with `unreachable`, which the host maps to overflow (C20 residual); `i64.rem_s` traps divide-by-zero by itself |
| A14 | Interpreter merge sort of 100k Ints 28 s (WASM 1 s) | PENDING |
| A17 | A lease without `Async` runs `spawn` | **DOCUMENTED** design (Async needs no capability; for-agents.md) — but a lease's effect list naming Async implies a bound it cannot keep; the budget stopped the spawn bomb cleanly. Candidate for a decision. |
| A18 | `--report-out` inside a granted `fs.write` scope refused on a normal run, accepted under `--sandbox` | PENDING (D-V2-21's rule skipped on one path) |
| A1, A4–A13, A15, A16, A18–A22 | various | PENDING |

## Found by the head chef while setting up
- CERT-SECRET-1 — **FIXED** RW 4.80, PR #65.
- `toolchain --json` `net` form without methods — **FIXED** PR #65.
- F-3, F-3b, F-4 laundering witnesses never parsed — **FIXED** with D-V2-184, PR #66.
- The gate passed the wrapper's own path, so an agent could read the wrapper (B did, 60 lines); no key or token reached.
