# V2 log

The running log (owner, 2026-09-18, D-V2-22): one short block per phase, newest last. Details live
in the commits and in project storage (`D:\nelan\DeluluLang-agent-transcripts\<date>-<phase>\`).
Phase state: [`V2_PHASE_STATUS.md`](V2_PHASE_STATUS.md). Decisions: [`V2_DECISION_LOG.md`](V2_DECISION_LOG.md).
The long logs, [`V2_EXECUTION_LOG.md`](V2_EXECUTION_LOG.md) and [`V2_AGENT_LOG.md`](V2_AGENT_LOG.md),
are frozen at P1.

## V2-0 — 2026-09-17 — complete
- Commit `e48f9c3` (CI `35258166713` green), record `94c6e29` (CI `35259510471` green).
- 32 historical documents moved to `docs/archive/v1/`; the V2 folder; the Survey's archive-mirror rule.

## P1 machine-contract truth — 2026-09-18 — complete
- Commit `d8dbc24` (CI `35299533343` green on 3 OSes), record `a93cfd8`. Opus 5 sous-chef; head chef verified.
- One JSON envelope on every reachable success; 145 → 1 diagnostics; `authority --grants`; editless
  repairs need a human; `test --json` diagnostics; bare `delulu test`; guide paths; accounting gate.
- Suite 1,674/0 on 126 binaries; snapshot 58 cases, all attributable. Rulings D-V2-19 to D-V2-21.
- Left open: P1-F1 to P1-F4 (roadmap, P1), 6.13, P1-11 (D-V2-17).

## P1-F closing P1 — 2026-09-18 — complete (CI recorded in the next block)
- F1 DL0404 cascade gone by poison propagation (generic `apply[F]` still DL0404); F2 `grants`/`guard`
  `--json` successes enveloped, broker-backed sweep; F3 undocumented shared flags refused by every
  command, allowlist read from the usage text `--help` prints; F4 `test <package-dir>` tests the whole
  package; F5 (RW 6.13 closed) fs trace records carry `resolved_path`/`scope_root`; F6 (D-NE-17, owner)
  `test --test-authority <row>`, narrows a package ceiling only, and every route to a test file meets its nearest enclosing package's ceiling.
- Snapshot: 12 cases, only DL0404 counts fell (greeter + 5 tier4-multimodule loose checks); blessed.
- Every task witnessed on `delulu-pre-p1f.exe` and falsified; evidence in agent storage `2026-09-18-p1f/`.
- P1-11 not built (owner ruling, D-V2-23).
- Head chef: found that F6 could be widened by reaching a package's test file by its path from outside; fixed
  and re-witnessed on 4 routes, both ways; snapshot re-checked structurally (12 cases, only DL0404 fell).
- CI `35347357572` red on Windows only: a pre-existing temp-dir race in `for_loop_cli.rs` (clock-only tags);
  fixed with a counter, pinned by a 16-thread test that fails without it.
- CI `35348430717` on `70864a2`: green on every job. P1 and P1-F complete.

## PS-0 sandbox truth — 2026-09-18 — complete
- 01 honest docs (no network client; `process` = foreign code only; RW 4.12–4.16); 02 `run --report-out`
  (D-V2-21), refused in `fs.write` scopes with `--trace-out`, no final-symlink follow; 03 DL1408 repair;
  04 `sandbox probe` (attempts only); 05 `doctor` sandbox section; 06 Windows hostile path spellings
  refused (DL0904) + NUL; 07 worker read deadline (DL1409); 08 three CI experiments written, not run;
  09 `net.special=` (D-NE-28).
- Every task witnessed on `delulu-pre-ps0.exe` and falsified; snapshot unchanged; Linux build and
  clippy checked in WSL. Open: a DL code for 09, POSIX name normalization, the 60 s deadline.
- Handoff (owner moved to another account, 2026-09-18): the next steps, the head chef's leanings on the
  sous-chef's questions a–e and a patch backup are in `D:\nelan\DeluluLang-agent-transcripts\2026-09-18-ps0\RESUME-PS0.md`.
- Head chef verified 02/04/06/09 on the binary against the pre-PS-0 one (the pre binary WROTE a `CON` file and
  accepted every special address; 26 extra address spellings refused, public ones allowed). Found and fixed:
  a bare `--report-out rep.json` was refused as unresolvable (empty parent), with a falsified test; an
  unresolvable write scope now refuses instead of being skipped. Questions a–e: D-V2-24.
- Commit `32ba712`, CI `35376528795` green on every job (the macOS Seatbelt code's first compile).
- PS-0-08 experiments, run `35376590803`: Linux KVM needs a permission step, then a VMM boots to `/init`
  (L2 is testable on GitHub). Seatbelt blocks reads and writes; its network result is void (the probe's own
  baseline timed out). Windows restricted-token child works; the job's one-process limit did not stop a
  grandchild (limit or probe: RW 4.18, settled at PS-A's start).
- RW 4.18 closed: both were probe flaws (`if errorlevel 9` means 9 or higher; the baseline was not ready). Fixed
  in `85501ec`; run `35378619727`: the job blocks the grandchild (1816) and Seatbelt denies the network. Every
  L1 building block PS-A needs works on GitHub's Linux, macOS and Windows runners.

## PS-A1 and PS-A2 (in progress) — 2026-09-19 — the guest runs jailed on all three systems
- The effect seam (one door out of the interpreter), `delulu-sandbox-channel/1` (canonical CBOR over the
  broker framing, deadlines both ways), the `__guest` child, and minting through the host: a program
  runs holding only host-minted handles and performs nothing itself.
- Jails: Windows Job Object (one process, 1 GiB, 5 min CPU, kill-on-close, UI limits, applied to a
  SUSPENDED child); Linux pre-exec (no-new-privs, pdeathsig, RLIMIT_DATA/CPU/CORE); macOS Seatbelt
  (no file writes, no network but the channel). Limits are D-V2-25's. Each platform reports only what
  it applied.
- CI green on Windows, Linux and macOS at `35397042554`. Five red runs first, each a real defect:
  RLIMIT_NPROC counts the whole user; RLIMIT_AS caps reservations not use (the wasm engine reserves
  gigabytes); a cleared environment needs each platform's LOADER variables; a Seatbelt literal must
  name the RESOLVED path (`/private/var/folders/…`); and an error dropped in one branch made three of
  those runs read as an unexplained timeout.
- Open: Landlock and seccomp (the approved crates), a deny-default macOS profile, the cargo-fuzz
  target, and the guest-mode fuzz run (trace ⊆ row).

## PS-A3 and PS-A4 (partial) — 2026-09-19 — the sandbox is a feature you can use
- `run --sandbox`, `--sandbox=off`, `--sandbox-profile dev|contained|hostile-agent`, `--limits` (narrows
  only), `--mode audit` (performs nothing; reports the policy and the grants a run would need), and
  `sandbox policy <file> --json` (the same policy, hash included, without running). `SandboxPolicy` is
  pure, JSON-stable and hashed.
- Refusals rather than a sandbox that quietly did not apply: unknown profile, unreadable limit, unknown
  value, and any surface the channel cannot carry (actors, foreign, Python, plugins, devices, secrets).
- Documented where it will be read: `docs/for-agents.md` [agents.sandbox] and `DEPLOYMENT.md` Tier 2,
  including what the sandbox is NOT — a second wall under the account boundary, not instead of it.
- Green on all three systems. Open in PS-A: Landlock file rules, a deny-default macOS profile, audit
  lifecycle records (PS-A-08), the mode transition matrix (PS-A-10), and the cargo-fuzz target.

## PS-A, the night of 2026-09-19/20 — Landlock, the transition matrix, and two fuzz campaigns

Four commits, each green on all three systems and on arm64 before the next one started.

**`a8889dd` — Landlock (PS-A2).** The guest now narrows its own view of the filesystem before the
program runs: no write right anywhere, reads only from the system paths a process needs to keep
running (`/usr`, `/lib`, `/bin`, `/etc`, `/proc`, `/sys`, `/dev`, plus its own channel directory), and
no TCP port rule at all, so every bind and connect is refused. seccomp says which syscalls may be
made; Landlock says which files they may reach, and a guest needs to open none of the operator's — the
host performs every read and write.

What is deliberately not claimed: Landlock mediates TCP only, so the guarantee says `TCP`, not
`network`. Rights are requested at ABI v3 because v3 is where `truncate` became mediated; a kernel
below that gets the shorter sentence `no file writes but truncation` rather than the same sentence
with less behind it. What the kernel supports is read back from the syscall, never from a version
string, and a kernel with no Landlock makes the run SAY so — an absent boundary must never read like
an applied one, and `guest_cli` now requires one of the two sentences to be present.

The guest's self-restrictions stay OUT of the run report on purpose: the report says what the HOST
applied, and a host cannot verify a claim its guest makes about itself.

Measured, not asserted. `restrict_self` cannot be undone, so the gate runs in a child process — the
test binary re-entered with one environment variable — twice, once without the ruleset and once with
it. The unconfined half is the falsification. On WSL (kernel 6.6, effective ABI 3): control
`WRITE=true SECRET=true SYSTEM=true`, confined `WRITE=false SECRET=false SYSTEM=true`. Mutated to grant
the channel directory write access, the test FAILS.

**`d059da4` — the transition matrix (PS-A-10), and two silent transitions it found.**
1. `--mode audit` without `--sandbox` PERFORMED THE RUN. The mode was read inside the sandboxed path
   and nowhere else, so a caller asking for a dry run got a real one: the effect on disk, exit 0, and
   nothing saying the flag had been ignored. `--sandbox-profile` and `--limits` were the same shape.
   All three now refuse outside a sandbox, and refuse with `--sandbox=off` too.
2. `--sandbox --sandbox=off` was resolved by argument order — a wrapper's default and a caller's
   appended argument decided the boundary between them. Contradictory requests are now refused in
   either order; repeating the same answer is still fine, and a test says so, or the rule would only
   be "do not repeat the flag".
   `sandbox_modes_cli.rs`, 7 cases, each with its falsification. Against the pre-fix binary both new
   gates FAIL, which is how the two defects were confirmed rather than assumed. Break-glass has no row
   (PS-B); "a child inheriting a weaker policy" has none because it is vacuous — a guest can start no
   child at all, and every surface that could ask for one is refused before the run.

**`ad66c9e` — the channel's `cargo-fuzz` target (PS-A-02).** `fuzz/channel_frame`, aimed at the one
decoder that reads bytes from a peer the host has deliberately assumed is compromised. Three claims:
no panic and no unbounded allocation; re-encoding is byte-stable (byte equality, not value equality,
because an `f64` NaN is not equal to itself); and an empty host — no root, no minted handle — never
answers `Ok` except to `Done`. The property lives in `delulu-runtime::channel::fuzz_one_frame`, not in
the target, and the ordinary suite replays it over a seeded structure-aware corpus (a third noise, a
third valid frames generated from the types, a third valid frames with one byte flipped) on every
commit. One rule, one copy. CI gained a `fuzz` job: one minute, coverage-guided, every push.
`fuzz/` is its own Cargo workspace; the Survey counted it as a fourteenth member and the stale-count
gate fired, correctly, so the Survey now knows what a separate workspace is and names it beside the
member count rather than hiding it.

**`9615a4d` — the guest-mode fuzz campaign (PS-A-01), and the hole in the obvious version.** Every
accepted program in the 50,000-program campaign now runs twice: locally, and as a guest over the real
channel through `channel::Loopback`. The guest's trace is still ⊆ row(main), and it EQUALS the local
trace. But the trace is written by the GUEST's interpreter before the frame is sent, so that pair
proves only what the guest ASKED for: a host that silently performed nothing would pass with a full
trace and an empty disk. So the host's sink is wrapped and every operation it performs is recorded on
the host's side, and the campaign asserts the two sequences agree — 30,198 executed programs, exact
agreement, six seconds. Falsified twice: silence the recorder (179 violations), or make the host
answer `Ok` to `println` without performing it (198 violations, naming what was asked and what was
done). The first mutant was aimed at a method called `print`, which does not exist, and changed
nothing — a mutant that changes nothing proves nothing about the gate either.

**macOS deny-default: the answer is yes, with evidence.** Experiment `35478590757` reported
`RESULT real-reads-everything: PASS rc=142` — the real guest, CPython and wasmtime and all, starts and
binds its channel under a DENY-DEFAULT Seatbelt profile (rc 142 is the probe's own alarm firing while
it waits for a hello). The belief that deny-default aborts a Mach-O binary was WRONG: the missing
clause was `(allow file-read*)`, and the earlier exit 134 was measuring an allow list that was too
short. The boundary found: narrowing reads to `/usr`, `/System`, `/Library`, `/private/etc`, `/dev`
aborts. Round two (`macos-seatbelt-deny-default-narrowing`) looks for the missing read root, asks
whether writes need the channel directory or only the socket file, trims each allowance to find which
are load-bearing, and carries two negative controls.
Round one's first attempt was a PROBE FAULT, not a result: every rung came back `FAIL rc=127 —
timeout: command not found`, because macOS ships no GNU `timeout`. The control line said "the probe
itself is wrong", which is what a control is for — the third time in this campaign that a "failure"
was the probe.

**Still open in PS-A:** the deny-default macOS profile itself (round two's evidence lands first);
`sandbox status`/`sandbox kill` and the `denied[]` half of PS-A-07; PS-A-08's per-effect audit records
and guest id (the chain carries launch and death only, and nothing per effect — not built, not
claimed); `explain E-SANDBOX` and the new DL codes with both witnesses; `--isolation process`
strengthened and announced; the Book Ch. 15 and `MATHEMATICS.md` §12's sandbox claims.

## PS-A closed — 2026-09-20 — what it built, and what it deliberately did not

**The arrangement.** `run --sandbox` runs the program as a guest process whose `root` grants nothing.
Every capability it holds is an opaque handle the host minted; every effect is one canonical-CBOR frame
the HOST performs, under exactly the checks an ordinary run makes. Under that, an OS jail per platform,
and each run reports the guarantees it actually got — never the ones it hoped for.

**The last increment (PS-A-08 completed).** A refusal on the channel now gets its own
`channel-violation` audit record, written only when the host actually said no, so an ordinary run does
not grow the chain and a guest that was refused fifty things is visible as one record an operator cannot
delete along with the report. A ceiling that fired gets `sandbox-limit`, and only what the exit status
actually carries: `SIGXCPU` names the processor-time ceiling unambiguously, `SIGKILL` does NOT — it is
what a hard ceiling escalates to, what `PDEATHSIG` sends, and what `kill -9` sends — so the record says
that instead of picking one. On Windows nothing is claimed, because no status bit says "a limit did
this". Falsified: with the violation record suppressed, the test fails on a refused run.

**Owner rulings taken in this phase:** D-V2-25 (profiles, crates, budgets, default-on as the
destination) and D-V2-26 (no new DL codes; D-V2-24(a) resolved the same way; opt-in until PS-B/PS-C
widen the channel). No entrenched file was touched.

### Deliberately NOT built, each with its reason

- **`sandbox kill`.** A guest cannot outlive its host on Windows (kill-on-close) or Linux
  (`PDEATHSIG`), and on macOS the remaining case — a guest computing while its host is gone — is bounded
  by the processor-time ceiling measured in run 35480762820. A safe `kill` needs a pid-to-executable
  check on three platforms, because a dead host's pid gets reused and killing by pid alone eventually
  kills an innocent process. Shipping it without that measurement would be a verb that looks careful
  and is not. The refusal says so where an operator will find it.
- **`sandbox-kill` as a separate record.** `sandbox-death` already carries the end of the guest's life
  WITH its reason, on every path including a channel that failed. A second record for the same moment
  would be evidence that says nothing new.
- **A guest id on every brokered effect.** The chain carries launch, violation, limit and death, and
  nothing per effect. Adding per-effect records would make every sandboxed effect a writer to a
  single-writer chain — the exact defect this phase already fixed once, when parallel runs put two
  records on one line and `doctor` refused its own machine. It needs the broker to own the chain, which
  is PS-B.
- **`external:<name>` profiles.** They name an operator-supplied launcher, and there is none to name
  until PS-D.
- **`[sandbox]` in `delulu.toml`.** A manifest section bounded by `[authority]` is a real surface and a
  real risk: a manifest that could widen its own confinement is the shape D-V2-25 forbids. It belongs
  with PS-B's limits-as-authority work, where the bounding rule is being built anyway.
- **"limits and remaining" in the report.** "Remaining" needs live resource accounting, and the only
  platform that offers it cheaply is Windows (`JOBOBJECT_BASIC_ACCOUNTING_INFORMATION`). A field that is
  a real number on one system and absent on two would be read as a measurement everywhere.
- **AppContainer and a user cgroup as probe attempts.** Both would be honest additions to `probe`;
  neither is load-bearing for L1 as built, and AppContainer needs new FFI. PS-B.
- **The P21 identity-separation vectors from inside the guest.** Vacuous here: the guest runs as the
  same OS user, so there is no identity boundary to attack. That absence is the `identity_separation`
  limitation the report prints on every run.
- **New DL codes**, by the owner's ruling (D-V2-26).

### The phase's numbers

Windows suite 1,748/0; the sandbox test binaries green on Linux too; sweep 38/38 on both; doctor 28/28;
Survey 1172 nodes, 10796 edges, 0 errors, 2 known warnings; the 50,000-program fuzz campaign SOUND with
every accepted program run twice, locally and as a guest; the `fuzz` CI job green on every push. Eleven
commits, each with its CI run read to green before the next started.

**PS-A's verification gate passed.** Closing commit `31643fe`, CI run `35492572666`: success on every
job — Windows, Linux, macOS, arm64, clippy, `cargo deny`, the editor artifact, the formal models, Miri
on atlas/diag/FFI, and the new coverage-guided `fuzz` job. Phase 3 is complete; phase 4 is P2, real
plugin loading, which asks D-NE-10 at its start.
