---
name: delulu-v2-execution
description: "DeluluLang V2 EXECUTION (owner approved the Next Evolution plan 2026-09-17 evening): where the active source of truth lives, the phase order, the agent model, the loss-protection storage, and the exact state of the current phase — START HERE for any session from 2026-09-17 evening on"
metadata:
  node_type: memory
  type: project
  originSessionId: eb34182b-835b-44af-b5ec-07064fcae52c
  modified: 2026-09-28T05:58:08.965Z
---

# DeluluLang V2 — executing (from 2026-09-17, evening)

**Commission:** `docs/design/DeluluLang_V2_Execution_Master_Prompt.md`. **Active source of truth:
`docs/DELULULANG_V2/`** (V2_README, V2_MASTER_PLAN, V2_IMPLEMENTATION_ROADMAP, V2_LOG,
V2_DECISION_LOG (D-V2-nn), V2_PHASE_STATUS, V2_SECURITY_MODEL, …). Archive under `docs/archive/v1/`.

**Phase order (owner's §8):** V2-0 → P1 → PS-0 → PS-A → P2 → P4a → P3 → **PS-B (HERE)** → P4b–e →
PS-C → P6 → P5 → P7 → PS-D → P8. After every phase: verification gate, **Survey regenerated LAST
before `git add`**, doctor, commit, push (testing remote only), read + record the CI run, pause ~60 s.

**Owner's standing instruction (2026-09-20, repeated 2026-09-25):** keep cooking through every phase,
check earlier phases work, don't ask questions ("Even if u ask me questions I will say 'Do whatever
good for delululang'"), be persistent, **use and update Survey and doctor, update the .md files and
HANDOFF.md**. Owner-reserved regardless: **entrenched files**, **the final public repo**, **licensing**.
Decisions under the delegation are recorded as `D-V2-nn … TAKEN (head chef, under the owner's
delegation)`. Agents: Sonnet 5 for small jobs only.

## STATE — 2026-09-25

Complete with CI read: V2-0 `e48f9c3` · P1 `d8dbc24` · PS-0 `32ba712` · PS-A `31643fe` · P2 `5f52eb8`
· P4a `2ad6e1d` · P3 `3ab0cc9`. PS-B opened `d0ae0f9` (D-V2-30 dependency, run `35524134404` green),
`86f58a1` recorded it (run `35524913246` green). Nightlies since: green except `miri-slow` syntax/check
(240-min cap, RW 5.6); miri-slow broker finished for the first time 2026-09-24.

**PS-B-02 DONE (2026-09-25, commit after `86f58a1` — read V2_PHASE_STATUS for the hash/CI):**
`crates/delulu-runtime/src/egress.rs`, reqwest+rustls, one host-side client for L0 AND sandboxed
guests (sandbox now carries Http). Strict URL spelling (refuse, never normalize), allowlist, resolve
once + classify EVERY candidate + pin with a client whose resolver refuses everything (NoDns),
`no_proxy`, redirects via own loop, 8 MiB/5 hops/30 s. `Refused` opaque; reason → stderr
`[egress: code]`, run report `egress`, guest `sandbox.denied`. `net.special` is its own dimension
(Grants→RootVal→CapScope::Net.special); a LEASE carries none (fail closed, open). D-V2-31 records it.
Real-internet evidence: example.com 200/559 chars from L0 and from a guest.

**Five things the check of earlier phases found (all fixed in that commit):** (1) CLI didn't forward
`net` → portable download (`--no-default-features`) would have had NO client; now `default=["python",
"net"]`, packaging builds `--no-default-features --features net`, pinned in tests/distribution.rs;
(2) **D-NE-31 was already RULED in D-V2-25 (2026-09-18): 1 GiB memory, 5 min CPU, never unlimited,
operator may change** — four docs + this memory wrongly called it open; (3) CHANGELOG stopped at PS-0
(PS-A/P2/P4a/P3/PS-B added); (4) doctor said profiles "arrive with PS-A"; (5) `cargo deny list` omits
dev-deps (`check` sees them — verified by a temp ban of rcgen).

**Since PS-B-02 (all 2026-09-25):** `525d901` message gaps fixed + `message_spacing.rs` gate (conform
lib.rs:515 left: entrenched). `936500f` **PS-B-01** every run budgeted (`budget.rs` watchdog, D-V2-32);
its CI `36117814329` Windows failed once on 2 `hw_adapter_cli` tests = a COLD Python driver start
(measured 4251 ms cold vs 177 ms warm; CPU-hog starvation 0/8 — NOT the mechanism) → test driver now
`python -I -S`, started once untimed; EXCHANGE_TIMEOUT untouched; slow-driver mutant still fails.
Product version recorded RW 4.19 (driver start-up counts against first exchange — D23 protocol change,
OWNER). **PS-B-05** budget = tenth ⊑ dimension (`budget_scope.rs`, absent = TOP, inherit when unnamed
at the tree chokepoint, wire unreadable → SMALLEST, `grants delegate --budget`, `run --lease` held to
node budget, daemon root records run budget), Z3 26 obligations + 3 CI-enforced model mutants
(`DELULU_Z3_MUTANT`), D-V2-33. The model had claimed "all nine" while modelling 7 set dims.
**Found+fixed:** `run --sandbox` silently dropped ~39 run flags incl. `--lease`/`--broker`, and even
unknown flags / 2nd file (RW 4.20; allowlist in `guest.rs`).

**Lesson (PS-B-05):** product laws of a lattice hold for ANY product — a model that drops a dimension
still proves them; add an obligation that only holds if the dimension is IN the conjunction.

PS-B-05 committed `32a5b96` (CI 36138186335: all green but macOS 1866/1 = a TEST temp path pushing the
broker socket past macOS's 103 bytes) → fixed `ca60614` (CI 36139795158 GREEN everywhere).
**PS-B-03 (Windows) built:** `identity.rs` per-run AppContainer, no capabilities, runtime copy in
%LOCALAPPDATA%\DeluluLang\guest-runtime (ALL_APPLICATION_PACKAGES RX), channel on inherited pipes
(`pipe_channel.rs`), `--stdio` guest; LOCALAPPDATA REQUIRED in the env block (error 203 otherwise);
T14 unit test with a control; macOS Seatbelt now denies the state dir (macos_tests, only CI can run it —
owner said "use github for mac os run"); Linux subordinate uid = RW 4.21 (PS-B-03b); D-V2-34.
Linux clippy: Windows-only items must be cfg-gated (PipeChannel allow(dead_code), STDIO_FLAG cfg(windows)).

PS-B-03 committed `d3355d6` (CI 36142494033: all green; the macOS runner RAN
`jail::macos_tests::the_state_directory_is_unreadable_to_a_seatbelted_guest ... ok` — Seatbelt's
last-matching-rule semantics confirmed there).
**PS-B-06 BREAK-GLASS built (D-V2-35):** `breakglass.rs` — `sandbox require --break-glass-key HEX`
(host policy `<state>/sandbox_policy.json`; unreadable = required with no key), `sandbox ticket` (Ed25519,
one program by blake3, ≤ 1 day, spent by atomic create in `<state>/break-glass/spent/`), `run
--break-glass T` (banner, report `break_glass_ticket`, audit `break-glass`; unrecordable = refused),
`sandbox release` needs a `policy-off` ticket; `test`/`repl` gated; `audit_required` in guest.rs fails
closed. 6 mutants killed. Owner asked mid-phase "which phase / how many more": answered phase 7 of 0–14,
7 more after PS-B.

PS-B-06 `1b9b7e8` CI `36145276674` GREEN everywhere. `7910497` committed the channel bench
(`measurements/sandbox-channel/`, manual `channel-measure.yml`) + probe job `linux-subordinate-uid`.
**Runs read:** probe `36167365278` = Ubuntu 24.04 default FORBIDS (AppArmor userns restriction; even
the control's uid_map write EPERM); `4cd7680` added a relaxed step (sysctl
kernel.apparmor_restrict_unprivileged_userns=0) → `36167600016` PASS (stranger refused 0600 file).
Channel `36167360827` by SLOPE: Linux 19.5, macOS 19.8 (its N=2000 point has a ~0.1 s fixed cost —
read slopes, not points), Windows Server 2025 66.9 µs (> the 50 µs rule).
**D-V2-36 PS-B-04: no batching** (changes when effects are observed, everywhere); instead the Windows
transport: drain threads = 4 wakes/round trip. Experiments (reverted): 48.6 → 40.4 → 30.1 µs removing
hops. Built: `pipe_channel.rs` = host `HostPipe` (overlapped server end of ONE duplex named pipe,
owner-only SD via `broker_transport::owner_only_for_this_user`, FIRST_PIPE_INSTANCE, 1 instance,
REJECT_REMOTE, random name, client opened by host + DuplicateHandle for stdout; read/write with
WaitForSingleObject+CancelIoEx deadline) + guest `GuestChannel` (direct reads, watchdog thread exits
the guest on silence; atomics only per read). Workstation 28.9 µs. Frames are one write now (both
channels; no measurable effect).
**D-V2-37 PS-B-03b built:** `identity::linux` — unshare(CLONE_NEWUSER) in pre_exec, helper thread runs
/usr/bin/newuidmap+newgidmap "1 <sub> 1" (per-run id from /etc/subuid), child: setgroups(0) →
setresgid(1) → setresuid(1) → capset(empty) — BEFORE jail::harden's pre_exec (id change clears
PDEATHSIG); exec via /proc/self/fd/N (no runtime copy); channel = UnixStream::pair as stdin; guest
`--stdio` on Linux writes READY byte 0x06, host `await_ready` falls back if the stranger can't load.
Posture: LINUX_GUARANTEE → identity + reads ("only what every account on the host may read"), never
writes/network. CI x86-64 `test` job sets the sysctl + `DELULU_REQUIRE_SUBORDINATE_UID=1`; arm64 = fallback.
WSL (Ubuntu 20.04, uidmap INSTALLED by me via `wsl -u root apt-get install uidmap`) = local Linux lab;
2 mutants killed (no setuid → SECRET=true; no setgroups → GROUPS assertion). RW 4.21 closed; **RW 4.23
opened**: Linux report omits guest self-applied Landlock/seccomp (planned fix: `ReqBody::Confined`
first request, allowlisted words, CHANNEL_VERSION /2).

`2fb0895` (PS-B-03b/04) CI `36171523136` GREEN (Linux identity MEASURED on x86 runner, fallback on
arm64); channel-measure `36171534543`: Windows CI 43.0 µs (was 66.9), Linux 13.2, macOS 18.7 — all
under the rule. `7a94684` RW 4.23 (guest sends `ReqBody::Confined{applied}` FIRST, words ∈
`channel::SELF_APPLIED`, channel `/2`) CI `36173241488` GREEN → **PS-B CLOSED 2026-09-26.**

**P4b–e IN PROGRESS (opened 2026-09-26; D-NE-6 → D-V2-38 taken as proposed).** `bc5c01c`:
P4-02 `toolchain.rs` (`delulu toolchain [--json]`, reads SUBCOMMANDS/usage_lines/documented_flags,
CORE_EFFECT_NAMES, `broker::GRANT_FORMS` [test parses Grants::add's match arms], PRIM_TABLE,
budget defaults, Profile::ALL, `sandbox::LEVELS`, REGISTRY, `delulu_diag::TOPICS`); P4-09
`schema.rs` (`delulu schema [name] [--json] | validate NAME FILE`, 9 CLOSED JSON Schemas, in-binary
validator, payload key `document` because envelope owns `schema`; tests/schema_cli.rs validates real
emitter output; mutant field caught); P4-10 `examples.rs` (9 single-file examples embedded,
`cli::source_authority_report` extracted & shared with `authority`, run line proven by
examples_run.rs; guide/05 fetches example.com). New subcommand checklist: SUBCOMMANDS + dispatcher +
usage() (`\x20` padding) + json_contract SUBCOMMANDS & cases + skill/for-agents mention.

**P4b–e items ALL DONE 2026-09-26** (each CI read green unless noted): `67d058a` P4-03 `delulu mcp`
(read-only by construction; MCP door rule now TWO-WAY: every command in READ_ONLY or EFFECTORS);
`05f26d4` P4-04/05 `delulu edit` (blake3 `--expect-hash`, repair-shape byte edits or Atlas-id node
edits incl. actor members, re-checked, `authority.widened`, closed `edit` schema); `f943097` P4-06
`delulu-survey diff <rev>` (`Survey::walk_many` union, entrenched named, MCP `survey_diff`);
`c8b9b58` P4-07 (checker side table `CheckResult::performed`; LSP hover declared vs performed;
`delulu.guardStatus` = `guard status --json` via `mcp::run_self`; VS Code *Show Guard status*);
`fcd2c2f` P4-11 `delulu atlas chain` (ten links, 16 corpus snapshots; Atlas gained
actuator/sensor/compute/plugin_host resources + `requested_scopes`); `11a7af1` P4-08
`delulu-measure ai-usability` (5 conditions, 7 measures, 9 grammar tasks, dl.py wrapper, 5 controls
per task, replayable record `measurements/ai-usability/`; Sonnet 5 pilot; FOUND `main` accepting
non-Root signatures → checker now DL0403/DL0401; Skill gained the first-program kit). **`11a7af1`'s
CI NOT yet read when this was written**; an LF fix for dl.py's `calls.jsonl` is uncommitted/next.

**P4b–e CLOSED 2026-09-26:** `11a7af1` CI `36261139018` green on every job; closure + dl.py LF fix
`769a646` (its run `36262232077` green).

**PS-C IN PROGRESS (2026-09-27), D-V2-39:** `run --isolation microvm` WORKS on Linux x86_64+KVM —
`microvm.rs` rewritten (launcher + in-VM guest half `__guest --vsock 1024`), Firecracker **v1.17.0**,
guest kernel **6.18.54 built from source** (`scripts/microvm/{build-image.sh,kernel.config,
mkinitramfs.py,check-reproducible.sh,fetch-firecracker.sh}`), image hashed on the COPY that boots,
`sha2` added (already in tree). Gated tests `microvm_cli.rs` (9) + `microvm_criterion8.rs` (criterion
8 restated) pass in WSL with `RUSTFLAGS=--cfg delulu_kvm`, 6/6 mutants killed. CI jobs `microvm` and
`microvm-reproducible` added. Open: **PS-C-03b jailer** (needs root; `wsl -u root` works here),
**PS-C-06 red-team record**, PS-C-05 = owner (D-NE-27). Then P6 → P5 → P7 → PS-D → P8.

**STATE 2026-09-27 (late) — P5 COMPLETE; P7 in progress.** Pushed: `5e2c8fd` P5+P5b, `0940ea1` P5c
(GUARD-ALIAS-1 pin+no-follow open, SANDBOX-STOP-1, GUARD-STALE-1), `f284a37` CI fixes (GREEN every job,
run 36330278053), `56eb5a6` P5d (VERIFY-FABRICATED-1 broker-side Guard-gated Secret.verify;
SECRETS-STALE-1 store refresh; socket msg), `262d5c4`/`081f20b`/`5065be1` L2 memory stop (guest caps
RLIMIT_DATA; line-at-a-time console relay; image-b rebuilt from a Linux-FS clone). Release dry run
36332086632 GREEN on 4 targets, nothing published. **In the tree (commit next): P7 batch 1** — RW 4.10a
`crates/delulu-runtime/tests/nist_kat.rs` (NIST byte-exact), RW 5.4 five fuzz targets (`fuzz/`, props in
`delulu_check::fuzz`, `lease::fuzz_one_lease_token`, `broker::fuzz_one_grant`; CI fuzz job extended),
RW 5.6 Miri shrinks, D-V2-45; + macOS fixes (EOPNOTSUPP socket, unreadable_paths scratch counter).
NEXT after push: trigger `miri-slow` (workflow_dispatch) to MEASURE RW 5.6; read CI; resume the
Sonnet Windows verifier (agent id in the session; weekly Sonnet limit resets 22:30 IST) with
`...p5b-haiku-guard/BRIEF.md` Addendum 3; then PS-D (external launchers + attestation seam), P8 owner-gated.
Pass-2 agent verdicts: `D:\nelan\DeluluLang-agent-transcripts\2026-09-27-p5b-haiku-guard\FINDINGS.md`
(Sonnet Linux verify: all 5 fixes HOLD incl. 220-run alias race at L0/L1/L2). D-V2-43/44 flagged for owner.

**Gate numbers at PS-B-02:** Survey 1212 nodes / 10957 edges / 0 errors / 2 warnings (owner's
OLD_PLAN.md placeholders); **doctor 29/29** (new: network client, 48 trust roots on this machine);
sweep 45/45; snapshot unchanged; clippy clean; cargo deny ok.

**Ops notes:** `$TMPDIR` unset in Bash — use the scratchpad path; heredoc with apostrophes breaks —
write Python scripts with the Write tool into the scratchpad and run them; Python on Windows needs
`C:/...` paths; commit messages via file (`git commit -F`); check `git ls-files --eol` for `w/crlf`.

**Why:** owner instructions 2026-09-17/20/25. **How to apply:** read
`docs/DELULULANG_V2/V2_PHASE_STATUS.md` first (outranks this), then the newest `V2_LOG.md` entry.
Related: [[delulu-next-evolution-2026]], [[agent-usage-rule-2026-09-17]], [[testing-repo-autopush]],
[[final-public-repo-gate]], [[delulu-survey-map]], [[delulu-clean-checkout]], [[delulu-licensing-intent]].

- **STATE 2026-09-28 — PUSHED `30a6b8d`** (PS-D-01 external launcher D-V2-46; JSON-EXIT-1; SANDBOX-CPU-LATE-1; AUDIT-LOCK-TAKEOVER-1 = audit lock is OS `File::try_lock`; Miri shrinks rcaps/prim_table). Local suite before the last doc edits: 1963 passed, only the stale-Survey 4 failed, fixed by regen (freshness 11/0, doctor_cli 5/0); doctor 29/29. **Jesse 2026-09-28: "run everything on github"** — heavy runs go to CI, not this machine (memory: Claude Code reaped background jobs at low RAM; use `-j 4`, one heavy job at a time, `wsl --shutdown` after WSL work). All five `30a6b8d` runs READ and recorded in `047da1d` (pushed): push GREEN (Win 1967 / Linux 1985 / mac 1975, 0 failed); heavy-gates green; miri-slow 0 UB and every test ran to the end (syntax 130, broker 168 in 176 min = lock fix holds, check 247/248 — the one failure a wall-clock test, now `cfg_attr(miri, ignore)`); release dry run green, nothing published; channel-measure under 50 µs (Linux/mac slopes rose, recorded); probe consistent. NEXT: read push run `36381942261` (GREEN) and miri-slow `36381950975` on `047da1d` — a green miri-slow CLOSES RW 5.6 (update REMAINING_WORK 5.6 + V2_LOG + P7 row).
- **2026-09-28 CLOUD HANDOFF** ([[cloud-period-2026-09-28]]): AGENTS.md + CLAUDE.md (`@AGENTS.md`) created at the root, `docs/CLOUD_SYNC_LOG.md` (the change ledger + sync procedure), `docs/assistant-memory/` (sanitized snapshot of THIS directory — the banned word replaced, `no-…-mentions.md` renamed `no-banned-word-mentions.md`), HANDOFF §0 (cloud period, PS-D-02 design) + §11 brought up to date, README status refreshed, CONTAIN-TOCTOU-1 recorded as CLOSED by FS-RACE-1 (RW 4.6, DEPLOYMENT §5, HANDOFF §11.4). **Jesse: "stop before PS-D-02"** — do NOT start PS-D-02 until he says; its design draft is in the session scratchpad (signed nonce-bound statement, pinned key, `--require-attestation HEX`, `delulu attest launch` reference attester; lease-level constraint = RFC/owner).
