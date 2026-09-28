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

## P2 — 2026-09-20 — a running program loads a plugin (NE-01 closed)

`prim.rs` said *"plugin hosting is not available in the Stage-1 runtime"* and that one line was the
whole of NE-01. Everything around the load existed — the `.dpx`, two classes, signing, the DIR replay,
`plugin verify` giving real verdicts — and nothing could load one. `examples/plugin_shout/README.md`
said so out loud: "there is no host program in this directory to run."

    $ cd examples/plugin_shout && delulu plugin build . && \
      delulu run host.delulu --grant console --grant plugin=.
    hello!

**The grant, both spellings (D-V2-27).** `--grant plugin=<path-or-dir>` is the operator's half;
`[plugins] allow = ["blake3:…"]` in `delulu.toml` is the package's, and it is a CEILING — it says which
artifacts may load, never what they may do. `accept_manifest` does not touch it, so `--grant-manifest`
cannot confer loading (the `exec.native` argument: a package cannot grant itself the right to load
code), and it applies whether or not `--grant-manifest` was passed, because a restriction an operator
can drop by accident is not a restriction.

**Where, then which, then the sequence.** The path resolves inside the granted roots through the same
containment `fs.*` uses, so `..`, a symlink spelling or a case difference cannot reach an artifact the
operator did not permit. Then the blake3 of the bytes IN HAND must be on the pinned list — on the bytes,
because a path is not a name for bytes and a `.dpx` swapped between `verify` and `load` is the TOCTOU
case. Then `load_verified` steps 1–6, the same functions `plugin verify` calls, so the two cannot
disagree. Every refusal is a `PluginErr` VALUE a host can decide about.

**A Verified plugin executes its DIR**, and the container format settles that rather than a preference:
§3.2 calls `delulu:wasm` a compilation cache, "recompiled from DIR when invalid — never an error". The
DIR is the canonical, content-bound form `step5_verified` has just replayed the whole Stage-1 judgment
over, so running it needs no second lowering to trust, and the plugin's effects go through the same
primitive table and the same trace as the host's. The export runs in its own interpreter over the
plugin's module, because injecting its functions into the host's table would make a bare name inside the
plugin resolve against the HOST's functions.

**Revocation kills a callable the host already holds** (R-6c), checked per call rather than per `get`.
The reference binds the load-time GrantId, so a reload mints a fresh node and an old callable stays dead.

**The load is evidence.** A `Load` trace record carries the loaded node id, so a reader can follow the
plugin into the audit chain and revoke it. `Load` is absent from `trace::effect_for` on purpose — that
table maps capability METHODS and a load is a free call — so the record is appended by the interpreter.
The test that asserts this had a name that over-claimed for one commit: it said the load appears in the
trace while checking only that the console write did. Both are checked now, and the record is falsified
by mutant.

### Refused rather than ignored, each with its reason in the refusal

- **Non-zero `Grant.limits`.** Fuel, memory and wall clock are the WASM engine's instruments; a Verified
  plugin runs on the interpreter, which has no fuel meter and no preemption. A limit nothing enforces is
  worse than no limit, because it reads as one. Zeros mean "none requested" and still load.
- **`Declassify` or `ForeignCall` in a plugin grant.** Their enforcement lives in custody, and a
  plugin's export runs in its own interpreter which cannot share the host's. A fresh embedded custody
  always allows, so letting these through would move a broker decision into one.
- **The `Contained` class on Windows**, unchanged.

### The defect P2 surfaced, which was older than P2

`deps.rs` built the prelude's name→id index BY HAND with two of the seven entries — `IoErr` and
`NetErr` — directly beneath its own comment: *"all three refuse identically — a rule that holds on two
paths out of three holds nowhere."* So on the dependency-graph path `PyErr`, `ForeignErr`, `Limits`,
`Grant` and `PluginErr` were absent from every module's table, and the moment a program had reason to
write `Grant`, the checker PANICKED on `delulu check <package>`. Both halves fixed: the index is derived
from what `push_prelude` pushed, in all three paths, so the class of defect is gone rather than the
instance — and the indexing is gone too, because a missing prelude type is a checker defect and a defect
must be a diagnostic.

### The Survey found what grep could not

`grep` for "runtime stub" found five files. `rdeps` found two more that described the same gap in their
own words: `REMAINING_WORK.md` row 4.11 — the row that OWNS it — and `STAGE6_PLUGINS_GUIDE.md`'s opening
box. Row 4.11 is closed with what is still refused kept IN the row rather than deleted with it.

### Evidence

Windows suite 1,763/0; the plugin and example gates green on Linux too; sweep 38/38; doctor 28/28;
Survey 1174 nodes, 10814 edges, 0 errors. `plugin_load_cli.rs` has 11 cases, each paired with the
control that shows the same program succeeding once the refusal's cause is removed, and six gates
falsified by mutant (path containment, the hash ceiling, per-call liveness, the custody dimensions, the
limits refusal, the trace record). The core-invariance snapshot gained **five NEW cases, zero CHANGED,
zero GONE** — the checker change altered no recorded answer (D-NE-3's reviewed diff).

### Open, and deliberately

- The **daemon** holder-check path shares `step4_holder` with embedded and is exercised by the same
  code, but has no daemon-mode test of its own (P2-04's "`guard_e2e`-style" half). It needs a running
  broker, which is PS-B's territory.
- **`Contained`-class execution** — an opaque module on the WASM engine, with fuel — is where
  `Grant.limits` belong and where `kill_on_limit` already waits. Not P2.

## P4a — 2026-09-20 — the Agent Skill, and the gate that keeps it honest

The roadmap asked for `skills/delulu/SKILL.md` in the Agent Skills format, so a harness can install one
file and an agent knows how to drive this toolchain without a human translating. It is written, it is
145 lines, and `delulu skill [--json]` prints the same bytes from the binary — embedded with
`include_str!`, because a skill that is only correct when you happen to be standing in the checkout is
not shipped.

**What the skill actually teaches**, chosen by what has cost someone time: `--no-prompt` everywhere or a
command may wait for a human; read the exit code, not the verdict string (one bug on record where they
disagreed); batch `check` and nothing else (twenty files, 711 ms in twenty processes against 48 in one);
use `delulu lsp` in a long loop because it is the same compiler so its answers cannot drift; and never
apply a repair whose `authority_widening` is true. Then the four things that trip agents specifically —
effect rows are part of the signature, capability-relative paths, `val`/`ref` as a second axis,
and grants being the operator's to confer, not the program's.

**The honesty section is not decoration.** An agent says what it was told: that foreign C and Python
are *holes in that guarantee*, enumerated not eliminated; that the sandbox is a second wall **under**
the OS account boundary because a guest runs as the same user; that certification is NONE; that `List`
has four methods and there is no `Map`. A test asserts each of those sentences survives, because the
first thing to go when a document is edited for length is the paragraph that admits something.

### The gate the roadmap wanted, and the two gates it did not ask for

**Every `delulu <verb>` the skill teaches must exist in the binary's `--help`.** Derived from `--help`
at test time rather than from a second list, so the two cannot drift. A skill naming a command the tool
does not have sends an agent into a loop it cannot escape — the instructions it was handed are the
authority, so it will try, fail and try again. Falsified with a `reformat` mutant.

Two more, because a reference that rots is worse than one that is missing: **every `[agents.*]` anchor
the skill cites must exist in `docs/for-agents.md`** (a dead anchor is a dead end an agent cannot
diagnose), and **`delulu skill` must print the committed file byte for byte** — two copies of agent
instructions is two things to go stale, and this project has watched that happen twice.

Validated in-tree rather than by the reference Node validator, ruling **D-V2-28**: an npm dependency in
CI to check four single-line fields is supply-chain surface for no gain, and the four rules are cheaper
to state than to import.

### My own three test bugs, each of which passed on a correct skill

Worth recording because all three were the *test* being wrong while the artifact was right — the shape
that reads like a product defect.

1. The command extractor read the prose "Use when you see .delulu files" and reported the verb `files`.
   Fixed by scanning only command contexts: a fenced line beginning `delulu `, or an inline
   `` `delulu …` ``. An extractor that cannot tell an instruction from a sentence reports the sentence.
2. The `--json` body began with a stray `\n`, because the frontmatter split kept the newline that ends
   the closing `---`. The body must start at `# DeluluLang`, and now does.
3. The caveat search failed on *wrapped* prose — it looked for "holes in that guarantee" against a
   document where those words span two lines. Whitespace-collapse before searching.

### Evidence

Suite **1,768 passed / 0 failed**; clippy clean; sweep **41/41** (three new cases: `skill`,
`skill --json`, and the bad-option refusal); doctor 28/28; Survey 1176 nodes, 10823 edges, 0 errors.
`crates/delulu/tests/skill.rs` has 5 cases. `json_contract.rs` gained `skill` to its subcommand list and
a success-envelope case, so the new verb is held to the same envelope contract as every other.
`scripts/package-toolchain.sh` ships `skills/`, and `REPOSITORY_STRUCTURE.md` gained the tree entry and
§5.9a — a top-level directory nobody documented is a directory that gets deleted by the next cleanup.

### Open, and deliberately

The skill is one file for one audience. The *other* agent surfaces — MCP server, checked edits, the
Atlas/Survey tooling, the usability benchmark — are P4b–e, which is phase 8 and unstarted.

## P3 — 2026-09-20 — the standard library, and the two defects that were older than the phase

`REMAINING_WORK.md` 2.1 called the standard library "the largest gap between what DeluluLang *is* and
what someone arriving from another language expects": `List` had four methods, `Str` six, and there was
no map at all. `List` now has fifteen, `Str` ten, and `Map[K, V]` exists with eight. All additive, so
minor-version work rather than an RFC — and the core-invariance snapshot agrees: **56 NEW cases, 0
CHANGED, 0 GONE**, 382 → 438, with zero cases removed.

The interesting part of this phase is not the fifteen methods. It is the four places the honest answer
was "no", and the two defects found while looking for them.

### Refused, each with its reason in the diagnostic (ruling D-V2-29)

- **`sort` on `List[Float]`.** `Float` has no total order — NaN compares false against every value
  including itself — so every comparison sort places it by accident of the algorithm. That is a
  decision taken on a representation that does not admit the decision, which is this project's own
  recurring search key. `Int`, `Str` and `Bool` sort, stably, so equal elements keep their input order
  and the answer is reproducible across runs and platforms.
- **`contains` on an opaque element type**, with the **same code `==` already emits** (DL0605) and the
  same "use `Secret.verify`" hint. `contains` is `==` in a loop, so if the two disagreed one of them
  would be wrong; sharing the code is what keeps them from disagreeing. And look at what the
  alternative was: `Value::eq` answers `false` for two `Secret`s, so an admitted `xs.contains(k)` over
  secrets would have returned a confident, WRONG `false` — an equality answer derived from secret data,
  the shape of finding IF-1. No new code, per D-V2-26.
- **A `Map` key that is not `Str`, `Int` or `Bool`.** `Float` for the reason above plus `-0.0 == 0.0`
  making two distinct keys collide; a structural key because its canonical form is a design nobody has
  made. Iteration is ascending by key and `keys()`/`values()` share that order, so they can be paired
  position by position — a `BTreeMap`, not a hash map, deliberately: the hash-order nondeterminism that
  makes other languages' map output untestable does not exist here, which is what lets a test compare it.
- **WASM: none of the twenty-three lower**, and that is not a regression. The backend has no `List` in
  its `Ty` at all, so `xs.len()` has always answered `DL1201` — measured, not assumed:

      $ delulu build listwasm.delulu --target wasm
      error[DL1201]: WASM codegen does not support this expression form

  for a program whose only method call is `xs.len()`. So **2.1's sentence "Each method needs … a WASM
  lowering" is corrected rather than satisfied**: it stated a rule the four pre-existing methods
  already broke, and a rule the code does not follow is not a rule.

`join` lives on `List[Str]`, not on `Str` — a deliberate deviation from the roadmap's P3-03 line
(Python's `sep.join(xs)` wart), recorded rather than quietly implemented. `chars()` is *defined* as
`split("")`, with a test holding the two together. `replace("", to)` returns the receiver unchanged,
because unlike `split("")` an empty replacement pattern has no natural reading and inserting between
every character is a surprise, not a semantic. `to_upper`/`to_lower` are full Unicode and the reference
says in so many words that they are **not a security normalization**, which a test asserts — four of
P22's defects were security decisions taken on a differently-spelled string, and the paragraph
admitting something is the first one cut when a document is edited for length.

### The soundness change the phase forced

`fold`'s callback is argument **1**. The R-4 gate — the builtin-callback law, the C88 fix, the thing
that keeps an effectful lambda from escaping a "pure" function — read `arg_tys.first()`. Left alone,
`xs.fold(0, effectful_fn)` would have had the law decided about the *accumulator*. So
`is_higher_order_method` became `higher_order_callback_arg`, returning the callback's POSITION, and the
fail-closed branch moved with it. `every_higher_order_builtin_surfaces_its_callbacks_row` asserts the
law for all four by position rather than by luck at zero.

### MAP-PARAM-1 — a type error that escaped the checker into the interpreter

Found while writing the *design*, not the code:

    $ delulu check mapty.delulu
    ok: mapty.delulu checked clean                # [1,2,3].map(fn(s: Str) -> Int { s.len() })
    $ delulu run mapty.delulu
    error[DL0907]: ... runtime fault here

`List.map`'s arm read its callback's **return** type and never its **parameter** type. R-4's row half
was written with great care, twice, with a long comment about the worst defect of the C88 campaign —
and the argument half was never written at all. `Secret.map` had the identical hole, where it matters
more: the closure is handed the PLAINTEXT under a parameter type its author declared and nobody
verified. A rule that holds on one path out of two holds nowhere, so both are fixed in one edit, each
with its witness and its control.

While fixing it: a non-function callback used to produce **two** DL0401s, one from the R-4 gate and one
from `method_sig`. The gate's message is the better one — it says why the row must be known — so the
second is gone, and `exactly_one_diagnostic_for_a_non_function_callback` pins one-per-mistake for all
four methods. Two diagnostics for one mistake is how a reader learns to stop reading them.

### ARITY-LABEL-1 — the normative gate that three device receivers never had

`PRIM_TABLE`'s own documentation says the arity column is **NORMATIVE** and that "the checker's
too-many-arguments gate (DL0403) reads it at the method-call site". Witnessed against the pre-fix
binary:

    let s = root.sensor("arm/joint")
    let v = s.read(1, 2, 3, 4, 5)        # table says arity 0 -> checked CLEAN
    let a = root.actuator("arm/gripper")
    let r = a.command("open", 1, 2, 3)   # table says arity 1 -> checked CLEAN

while the control, `console.println("a","b","c")`, correctly gave DL0403. `prim_receiver_label` — the
function the gate calls to find a receiver's table label — is a **second list**, and `actuator`,
`sensor` and `compute` were never added to it when Stage 10 added them to the first. So for the three
AUTHORITY-BEARING DEVICE receivers the gate did not read the normative column and never had. That is
verbatim the fail-open skip branch the gate exists for — `root.console(1,2,3,4,5)` minting a cap and
ignoring the noise — reopened through a receiver added later.

**Why no test caught it, which is the part worth keeping.** `prim_table.rs` has tests that claim to
prove the arity column "in both directions … for every constructible receiver". Their
`receiver_binding` helper ends in `_ => return None`, and the three device receivers fell into it — so
the guards SKIPPED exactly the receivers that had the hole, and reported the easy ones passing. A test
that excuses its hardest cases is not a weaker test; it is a test of something else.

Closed as a class, not as three instances: `every_table_receiver_is_covered_or_explicitly_excused`
requires each table receiver to be either bound or named in an explicit `UNBOUND_RECEIVERS` list with a
reason, and `every_table_receiver_is_labelled_for_the_arity_gate` calls every primitive with arity+1
arguments and requires DL0403. The second fails on a mutant that removes one label, and passes when it
is restored.

Note what the snapshot's **0 CHANGED** means here: no recorded case ever passed a surplus argument to a
device receiver. That is the same blind spot from the other side.

### My own defect, caught by probing instead of reading

The first `Map` key check ran **before** `expect_arg`, so at `let m = Map()` it saw an unresolved
inference variable, correctly declined to rule — and then `expect_arg` bound `K := Float` with nobody
looking again. `m.insert(1.5, "x")` checked clean and the interpreter faulted with a DL0907 that said
"(checker bug)". It was right; it was that one. Moving the check after unification fixed it, and the
gate is now asserted on all four key-taking methods, because a rule that holds on three paths out of
four holds nowhere.

### Evidence

`List` 4 → 15, `Str` 6 → 10, `Map[K, V]` new with 8. **23 new conformance anchors, each with a positive
AND a negative witness** — `tests/conformance/accept/{29_stdlib_p3,30_map_p3}.delulu` plus 23 reject
files — taking anchor coverage to **353/353, 100%**, `prim` 82/82. The ratchet `COVERED_FLOOR` still reads
307 and is NOT raised here — see the section below; that file is entrenched.
`crates/delulu/tests/stdlib_p3.rs` holds the 19 claims a conformance program cannot state, including
`only_the_mutating_methods_demand_a_writable_receiver`, asserted in BOTH directions so a registry that
wrongly listed `keys` as a mutator would fail. `PRIM_TABLE_VERSION` 4 → 5 by the constant's own rule
("bump whenever either half changes"), the honest cost being that an existing `.dpx` carrying a v4 DIR
is refused with DL1503 and rebuilt. The fuzz generator now emits all four higher-order shapes rather
than only `map`, because a shape the generator cannot write is a shape the corpus never attacks.

### One edit P3 wants and did NOT make: the coverage ratchet (owner-reserved)

`crates/delulu-conform/src/tests.rs` holds `COVERED_FLOOR`, the ratchet the coverage law ratchets on,
and its own comment says "Raise it when you add witnesses." P3 added 23 witnesses, so the honest value
is **353** and it still reads **307**. It was raised, then reverted before the commit, because
`/crates/delulu-conform/` is ENTRENCHED in `CODEOWNERS` and entrenched files are the owner's regardless
of the phase delegation.

Nothing is green only because of that: the assertion is `covered() >= COVERED_FLOOR`, so 353 ≥ 307
passes, and `release_requires_full_coverage` demands **100%** — strictly stronger than any floor, and it
passes at 353/353. The floor is redundant while 100% holds. What is lost is only the ratchet's warning
value: a future change could drop up to 46 witnesses and still clear 307, and the 100% gate would be the
one to catch it rather than this one.

Awaiting the owner's word. It is a one-line change, `307` → `353`, and it STRENGTHENS the gate —
CODEOWNERS' stated reason for entrenching this path is that "weakening a gate is easier to hide in a
diff than breaking a rule outright", and a raised floor is the opposite of that. The witnesses
themselves needed no entrenched edit at all: conformance PROGRAMS in `tests/conformance/{accept,reject}`
carry their own `// anchors:` headers, so `witnesses.toml` — the entrenched file — was never touched.

### Open, and deliberately

No `Set`. No `Map` literal syntax — `Map()` follows the `Ok`/`Err`/`Some`/`None` precedent because the
language has neither a literal nor a static-method form to hang `Map.new()` on. No WASM lowering for
any collection method, which needs a heap layout, a key canonicalization and an ordering in the guest:
three designs, each a place for a security decision on an unnormalized representation, and none of them
P3's business.

### The run, read

Commit `3ab0cc9`, CI run `35522886721`: **success** — 12 jobs green, 2 skipped (`heavy-gates` and
`miri-slow`, both manual). Green on `ubuntu-latest`, `windows-latest`, `macos-latest` and arm64, with the
coverage-law report, the reference-in-sync gate, the CLI sweep and the fuzz campaign all passing on each.
The phase is complete by §0's gate: the commit is pushed and its run has been read.

## PS-B — 2026-09-20 — started: the dependency, measured before anything used it

PS-B is the phase that turns resource limits into **authority** rather than launcher knobs, and builds
the network client this project has never had. It is six tasks and the roadmap budgets 6–8 sessions.
This entry covers the opening step and is written to be **resumed from cold**.

### What the phase asked the owner, and what he ruled

PS-B-02 opens with the one decision the phase delegation does not cover: D-NE-28's TLS dependency, which
that entry itself reserved — "the largest this project would take and needs the owner and a `cargo deny`
pass." The owner ruled on 2026-09-20: **`reqwest` over `rustls`**, minimal explicitly-named features,
real HTTPS, TLS never implemented here, and HTTP-only egress explicitly refused as a production
implementation. Recorded as **D-V2-30**, which closes D-NE-28. D-NE-31 (resource-limit *defaults*,
PS-B-01) is still open and still the owner's. [Corrected 2026-09-25: it was not. The owner ruled
D-NE-31 on 2026-09-18 inside D-V2-25 — 1 GiB of memory and 5 minutes of CPU, never unlimited, the
operator may change them — and this sentence repeated a stale open-list. See the PS-B-02 entry below.]

### What landed in this commit, and why only this

`crates/delulu-runtime/Cargo.toml` gains `reqwest`, behind a new **default-on `net` feature** that
mirrors the existing `python` precedent: `--no-default-features` builds a network-less delulu where
`http.get` answers `NetErr::Refused`, which is what every build has answered since Stage 1 (NE-17), so
the off position is today's behaviour rather than a new one. The egress **policy** is deliberately NOT
behind the flag — a security rule tested in one build configuration is a security rule tested in one
build configuration.

Nothing uses the dependency yet, and that is the point of stopping here. The `cargo deny` pass the
ruling demanded had to be taken against a tree where **nothing else had changed**, or the number would
have been an estimate dressed as a measurement. It is in
`measurements/dependency-egress/RECORD.md` with the 81 new crates listed one per line:

| | before | after |
|---|---|---|
| distinct crates | 222 | **303 (+81, +36.5%)** |
| distinct licenses | 14 | 15 (`BSL-1.0`, from `ryu`'s `Apache-2.0 OR BSL-1.0`) |
| `cargo deny check` | advisories/bans/licenses/sources **ok** | **the same four ok** |

`cargo check --workspace --all-targets` passes. Nothing was removed.

**The finding worth carrying forward** is in that record and is not about size. Nineteen of the 81 are
the ICU stack, reached through `idna` because `url` does internationalized domain names. IDNA is a
**normalization performed on a host name**, and a host name is a string this project compares to make a
security decision — the exact shape of four P22 defects. The normalizer runs *inside the HTTP client*,
**after** the allowlist check has been made on the string the program wrote. So the client must never be
handed a name to re-derive: resolve host-side, classify, and **pin the address**
(`ClientBuilder::resolve`), so the host that was checked is the host that is connected to by
construction rather than by trust.

### PS-B-02, the rest of it — the design, settled, for whoever picks this up

The architectural fact that makes this tractable was established by reading, and should not be
re-derived: **`prim::call_cap_method` is the single host-side place an effect happens.** `sink.rs`'s
`EffectSink` is the one seam, `LocalSink` calls the primitive table in-process for L0, and
`channel.rs`'s `HostChannel::decide` dispatches a sandboxed guest's `CapMethod` to `sink.cap_method` —
**host-side**. So implementing egress in the `(ResourceKind::Http, "get")` arm of `prim.rs` gives L0 and
guests one implementation *by construction*, which is exactly what PS-B-02 requires, with no second code
path to keep in step. The guest has no resolver for the same reason: it never reaches this code.

`netclass.rs` (PS-0-09) already does special-use classification on a normalized host, including every
`inet_aton` spelling, and its own doc comment ends by naming this task: "Whether a public-looking NAME
resolves to such an address is decided at connect time, by the PS-B egress proxy (REMAINING_WORK 4.16)."
`host_allowed`/`host_of` in `prim.rs` already implement the allowlist with the C85 dot-boundary rule.

The shape to build, as a NEW `egress` module under `crates/delulu-runtime/src/` (it does not exist
yet — the Survey flagged an earlier draft of this paragraph for naming a path that is nowhere in the
tree, which was fair: a handoff describing a file to write reads exactly like a citation to a file that
exists, and the reader cannot tell which from the sentence):

- `Resolver` and `Transport` traits, so the whole policy is testable **offline** with fakes. The real
  `Transport` (reqwest) is the only thing behind `#[cfg(feature = "net")]`.
- `EgressPolicy { allow, allow_special, max_body, max_redirects, deadline }`, and one entry point
  `get(url) -> Outcome`.
- A machine-readable `Reason` enum — scheme, userinfo, not-allowlisted, special-use(class),
  no-address, too-many-redirects, body-too-large, tls, timeout.
- Per request: refuse a non-`https` scheme; refuse an authority carrying **userinfo** outright rather
  than parsing it; check the host against the allowlist; resolve; classify **every** candidate address
  with `netclass` and refuse unless `net.special=` granted; pin the chosen address; `Policy::none()` and
  follow redirects in our own loop so each hop re-runs the **whole** check; bound the body on what the
  program receives.

**One design point to decide deliberately, with an argument already worked out:** the program-visible
`NetErr` has exactly three variants (`Refused | Timeout | Other(Str)`) and adding a fourth would break
every exhaustive `match` in user code. Keep the three, and keep `Refused` **opaque to the program**:
telling a guest *why* — "that name resolved into a special-use range" — hands it DNS results it has no
resolver for, which is a resolver oracle rebuilt out of error messages. The machine-readable `Reason`
belongs in the **host-side audit record and the `--json` envelope**, where PS-B-05 and PS-B-06 already
need it. That satisfies "machine-readable refusal information" without opening a channel.

**Testing note, which is the awkward part.** Loopback is special-use, so an integration test against a
local server needs an explicit `net.special=127.0.0.1` grant — which is precisely what that spelling
exists for. HTTPS on loopback additionally needs a self-signed certificate and a way to trust it; a
test-only root must be `#[cfg]`-gated and never a release path, and an env-var trust injection would be
a hole, not a seam. Most of the policy should not need any of this: the fakes cover it.

**Owed gates, named so they are not forgotten:** a test that reads the manifest and fails if a refused
feature (`gzip`, `brotli`, `zstd`, `deflate`, `cookies`, `charset`) is ever enabled — a comment
explaining that decompression is off does not keep decompression off; a `doctor` line carrying the
**native trust-root count**, because a host with an empty store cannot make an HTTPS request and that
must be legible rather than mysterious; the network conformance family with C-03/C-04 flipping; and a
conformance case asserting the checked host equals the connected host.

### The other five tasks, untouched

PS-B-01 (budgets on every engine; defaults are D-NE-31 — **already ruled in D-V2-25**, corrected
2026-09-25), PS-B-03 (identity separation —
AppContainer per run on Windows, uid mapping where namespaces allow, a documented macOS recipe, reported
in `host_guarantees`; today a guest runs as the same OS user and `identity_separation` is always in
`limitations`), PS-B-04 (channel batching, gated on PS-A's measurement saying it pays), PS-B-05
(resource authority in the derived policy, the envelope, the authority report, the audit record; the
dimensions admitting a containment order join the Z3 model and `MATHEMATICS.md`), PS-B-06 (BREAK-GLASS
as an operator-held credential outside the guest, never activatable by program code).

PS-B-01 and PS-B-05 are the two that need no ruling to start and touch code PS-B-02 does not.

### The opening run, read

Commit `d0ae0f9`, CI run `35524134404`: **success** — 12 jobs green, 2 skipped (`heavy-gates`,
`miri-slow`, both manual). The two that mattered here: **`supply-chain` is green**, which is
`cargo deny` running on a RUNNER rather than on this disk — the clean-checkout lesson of 2026-09-14
says a gate verified only on this machine is a gate verified on this machine — and **`arm64` and
`test (macos-latest)` are green**, so the eighty-one new crates, `ring`'s assembly among them, build on
every platform this project ships to. That was the real risk in taking this dependency, and it is now
measured rather than assumed. The preceding commit `853664f` (P3's run recorded) is green too, as run
`35523603074`.

### One gap found by using the tools on this commit

The Survey has a `dependency-never-used-in-source` class, whose whole purpose is to notice a declared
dependency that no source file names. This commit declares `reqwest` and **no source file names it** —
and the class did not fire; it still reports only its one pre-existing `delulu-fuzz-targets` row. So the
check sees `path` dependencies and not registry ones, which means it would not have noticed any of the
eighty-one crates arriving unused either. Recorded here rather than fixed, because the next commit makes
`reqwest` used and the gap becomes invisible at exactly the moment it stops being demonstrable. It is
the same shape as ARITY-LABEL-1 from P3: a check that reports the easy cases passing while skipping the
category that would have failed.

The Survey did earn its keep on this commit twice over, though, both times on prose: it caught this
entry naming an `egress` module that does not exist yet, and caught D-V2-30 counting the ICU subtree in
bare digits followed by the word crates — which its `stale-count` class reads as a claim about the
workspace's own crate count, of which the tree has nine shipped. Both were genuinely ambiguous and both
are reworded above; neither was suppressed. (The second one fired a third time on the sentence you are
reading, when it quoted the offending phrase verbatim to explain it. A check that cannot tell a claim
from a quotation of a claim is a limitation worth knowing about, and spelling the number out is a
cheaper answer than teaching it the difference.) `doctor` is 28/28, and its network line still says
“there is no network client” — which is the honest reading of a commit where nothing uses the client yet.

## PS-B-02 — 2026-09-25 — the network client: one host-side implementation, and what reading the earlier phases turned up

The owner's instruction for the session: finish the phases, check that the earlier ones are working
and finished properly, use and update the Survey and `doctor`, update the `.md` files and `HANDOFF.md`,
and ask nothing. Before a line of PS-B-02 was written the earlier phases were checked: the last push
run (`35524913246`, `86f58a1`) green on every job; the four nightlies since then green on every job
but the two `miri-slow` jobs the 240-minute cap cancels (RW 5.6 — and `miri-slow (delulu-broker)`
finished for the first time on 2026-09-24, in three hours); the local suite 1,790/0; `doctor` 28/28;
the Survey fresh.

### What was built

`crates/delulu-runtime/src/egress.rs` — the only code in DeluluLang that sends a byte over the network,
called from the `(Http, "get")` arm of `prim.rs`, which is where a sandboxed guest's request is
performed too (`HostChannel::decide` runs it in the host). The design in the handoff above, made true;
D-V2-31 records each decision inside it. In one paragraph: a strict parse that refuses every spelling a
URL parser would rewrite; the allowlist on that parsed host; the name resolved once, host-side, every
candidate address classified and one special-use candidate enough to refuse unless the host came
through `net.special=`; the classified addresses pinned into a client whose own resolver refuses every
lookup; proxy variables ignored; redirects followed by this loop, each hop re-running the whole check;
the response bounded at 8 MiB; TLS by rustls against the platform store. The program sees `NetErr`;
the operator sees `Reason` — on stderr, in the run report's new `egress` object, and for a guest in
`sandbox.denied` and therefore in the hash-chained `channel-violation` record.

### What checking the earlier phases found — five things, all fixed in this commit

1. **The download would have had no network client.** The CLI crate takes the runtime with
   `default-features = false` and forwarded only `python`, so `net` reached the binary only when
   another workspace member happened to enable it — true of `cargo test --workspace`, false of
   `cargo install --path crates/delulu`, and false of the portable release build, which is
   `--no-default-features` to leave Python out. `d0ae0f9`'s manifest said "ENABLED BY DEFAULT", and it
   was, in the one crate that did not ship. Fixed and pinned (`tests/distribution.rs`).
2. **D-NE-31 was never open.** The owner ruled it on 2026-09-18 inside D-V2-25 (1 GiB, 5 min, never
   unlimited). The decision log's open list, this log twice, the phase status and the session memory
   all went on calling it "the owner's". PS-B-01 is not blocked on anyone.
3. **The changelog stopped at PS-0.** PS-A, P2, P4a, P3 and PS-B's opening had no entries. Written now,
   from this log.
4. **`doctor` still said "sandbox profiles arrive with PS-A"**, two phases after PS-A shipped them.
   Found by reading its output, as the owner asked.
5. **`cargo deny list` never counted dev-dependencies**, although the dependency record said it did.
   True only vacuously: until this commit the workspace had none. `cargo deny check` — the gate — does
   see them (a temporary ban of `rcgen` failed it), so nothing escaped a check; the record's method is
   stated precisely now and the eight test-only crates are counted separately.

And one class found on the way, NOT fixed here because it changes core output: **flattened string
continuations.** A `\`-newline continuation written through a shell heredoc arrives in the file as a
run of spaces inside the string, so the message prints with a gap mid-sentence. This commit's own
`doctor` lines had it and were rewritten with `concat!`; about a dozen pre-existing messages carry it —
four `delulu explain` texts in `codes.rs`, a repair reason in `check.rs`, three plugin refusals in
`interp.rs`, two `run` refusals in `run_cmd.rs` among them. Several reach the core-invariance snapshot,
so they get their own commit with a gate and a reviewed re-bless (D-NE-3), not a ride in this one.

### The defects this phase's own tests caught before they shipped

- **A TLS refusal reported as `connect`.** hyper wraps the `rustls::Error` in an `io::Error` inside an
  `io::Error`, and `io::Error::source()` answers the wrapped error's source rather than the wrapped
  error — so a walk over `source()` stepped straight past the one value that says TLS. Every nested
  `io::Error` is opened with `get_ref()` now.
- **A gate that could not fail.** The first version of "the client cannot resolve a host it was not
  pinned to" used a name no resolver knows, so it failed with or without the refusing resolver. It uses
  `localhost` now — the one name every system resolver answers, with the very address the test server
  listens on — and removing the refusing resolver makes it fail.
- **A counting error in the audit window.** `records_since` worked out refusals past the log's bound
  from the GLOBAL refused count, which includes refusals before the mark. A mark now snapshots both
  counters.
- **The resolver thread** was caught by D67's gate (every thread sizes its stack or is listed as never
  running a program) and listed, with its reason.

### Falsified, each by breaking the thing and watching its test fail

Certificate verification off (`danger_accept_invalid_certs`): two tests fail. `.no_proxy()` removed:
the proxy test fails (it runs as a child process with `HTTPS_PROXY` set, because changing the
environment of a multi-threaded test process is a race, not a test). The refusing resolver removed: the
unpinned-host test fails. reqwest allowed to follow redirects itself: the redirect test fails. `gzip`
enabled: both feature-accounting tests fail, naming it. The channel's egress recording removed: the
guest `denied[]` test fails.

### Evidence

`egress` unit tests 32 (+1 ignored: the proxy child), the channel test, `netclass` 5, feature
accounting 2, `egress_cli` 4 (C-03 flipped: a TLS handshake arrives from an L0 run AND from a guest),
distribution 1 new. Clippy with warnings as errors: clean. `cargo deny check`: advisories, bans,
licenses, sources ok. The shipped dependency graph is unchanged (303 distinct dependencies in
`cargo deny list`), and eight new dependencies are test-only.

**And against the real internet, by hand, on 2026-09-25** — the one thing the suite deliberately does
not do. A program fetching `https://example.com/` under `--grant net=example.com` received **559
characters, HTTP 200**: the name resolved to two public addresses, both classified, both pinned, the
connection went to `104.20.23.154:443`, and the certificate chain verified against the Windows store's
48 roots. The same program under `--sandbox` — a jailed guest in a Job Object — received the same 559
characters from the same peer, through the same host-side client. It is the first time in this
project's history that a DeluluLang program has received a byte over the network. The guide's own
example (`examples/guide/05_capabilities.delulu`, fetching `/health`) now takes its `Err` branch for an
honest reason the operator can read: `note: ... the server answered HTTP status 404 [egress: status]`.

### Open, and deliberately

- Only `GET`. Request bodies, headers, other methods and a per-run egress byte budget are not built.
- A LEASED run cannot reach a special-use range: the broker's node has no `net.special` dimension, so
  the lease carries none. Fail closed; it joins the grant tree with PS-B-05's resource authority work.
- PS-B-01, -03, -04, -05, -06 remain. PS-B-01 needs no ruling (D-NE-31 was ruled).

## 2026-09-25 — thirteen messages that printed a gap mid-sentence, and the gate that keeps them out

Found while writing PS-B-02's `doctor` lines. A Rust message long enough to wrap is written as a
string continued with a backslash at the end of the line; the compiler drops the newline and the next
line's indentation, and the value reads as one sentence. Written through a tool that flattens the
newline but keeps the indentation, the source becomes one long physical line and the program prints
the indentation: `the text to change is the grant                          request in your own
invocation`. Nothing failed — the text compiled, and every test matching on a phrase still matched.

**The instances, all user-visible:** four `delulu explain` texts (DL1908, DL1910, DL1912, DL1913, the
signing and compute-kernel entries); two `run` refusals (`--sandbox` said twice with different
answers, and a sandbox-only flag without `--sandbox`); the broker's narrowing repair; the checker's
row-narrowing repair; three plugin refusals in the interpreter; the actor-boundary plugin fault; and a
test's assertion message. Forty-five gaps across twelve source lines, each restored as the
continuation it was meant to be, so each now prints one space where the gap was.

**One instance is the owner's, not fixed:** `crates/delulu-conform/src/lib.rs:515`, the validation
message for a rejecting witness whose test body never mentions its code. `crates/delulu-conform/` is
entrenched (CODEOWNERS). The gate names it in its `OWNERS` list, and that entry fails the gate the
moment the owner fixes the line, so it cannot linger as an exemption.

**The gate** (`crates/delulu/tests/message_spacing.rs`) decodes every ordinary string literal the way
the compiler does — escapes, and a continuation skipping the next line's indentation — and reports a
run of six or more spaces between two visible characters when the literal's own source line is longer
than 150 characters. The length is part of the rule because a gap alone is ambiguous: the toolchain
prints about fifty deliberate columns (help tables, `key:   value` renders, TOML alignment). Measured
today, every real instance sat on a source line of 171 to 1,198 characters and every deliberate column
on one of 140 or fewer. The blind spot is stated in the gate: two SHORT lines flattened into one stay
under the bound. Falsified: restoring one of today's flattened lines fails it, naming the line; its
own tests show a flattened literal is found, a real continuation is not, and a short column is not.

**The snapshot moved, and the diff is the whole story (D-NE-3).** Two cases, both the `reason` of the
checker's row-narrowing repair in `check --json` — the machine channel had been carrying the gaps to
every agent that read it:

    tests/conformance/reject/DL0405_unknown_method.delulu :: check --json
    tests/conformance/reject/DL0504_row_conflict.delulu   :: check --json
    - "reason": "narrowing a declared row is a review decision, not a mechanical edit:<26 spaces>the effect …
    + "reason": "narrowing a declared row is a review decision, not a mechanical edit: the effect …

Nothing else in the snapshot changed.

**The cause, and the working rule it leaves.** A shell here-document written through this
environment's command tool collapses a backslash-newline; two of this session's own edits hit it
before the pattern was recognised. The rule: never write a backslash-newline continuation through a
shell here-document — edit such lines with the file editor, or build the message with `concat!`.
A stale doc comment was corrected on the way: `guest.rs` still said the guest "has no OS jail yet,
which PS-A2 adds".

## PS-B-02's run, read — and PS-B-01, the main program's budgets (2026-09-25)

**PS-B-02's run.** Commit `d59201a`, CI run `36114298041`: **success** on all twelve jobs — Windows,
Linux, macOS and arm64 test suites, clippy, `supply-chain` (`cargo deny` on a runner), fuzz, the formal
models, Miri on atlas/diag/FFI, the editor; `heavy-gates` and `miri-slow` skipped as manual. So the
real TLS tests against a loopback server, the proxy test that runs as a child process, and the guest
egress tests pass on every platform this project ships to, and the network-less build still compiles
there. The message-spacing commit `525d901` followed it.

**PS-B-01.** Every ordinary run is held to a budget now — 1 GiB of memory and 5 minutes of processor
time unless `--limits` sets others (D-V2-25; D-V2-32 records how it is enforced). Before this there was
no bound on either engine, and NE-22's flooded actor mailbox grew past a gigabyte under
`--grant console` alone.

`crates/delulu/src/budget.rs` is a host watchdog: every 25 ms it samples the process's peak memory and
processor time, and on the first breach it says which budget, from its own measurement, writes the run
report with `outcome.stopped_by`, and ends the run with exit 1. It is armed in `note_program_started`,
the one call every engine passes through, which is what "budgets on every engine" comes down to. The
report's `sandbox.limits` is the budget the run was held to (it was `null` at L0), with `enforced_by`
stating the mechanism and its 25 ms resolution. `--limits` without `--sandbox` is APPLIED now rather
than refused (PS-A-10's rule was right while an ordinary run had nothing to apply it to); zero, and a
dimension nobody enforces, are refused before the program starts.

**C-06, flipped** (`crates/delulu/tests/budget_cli.rs`): NE-22's own shape — an actor that works slowly
while `main` floods it — is stopped by a 256 MiB memory budget in under a second; so are an endless
loop (`cpu=1`), a doubling string (256 MiB), an operator's `wall=1`, and hours of `fib(60)` on the WASM
engine, which has no `while` and is held by the same watchdog. Every child runs against its own
deadline, so a missing watchdog makes a test fail instead of hang. **Falsified:** with the watchdog
never armed, four tests fail at their deadlines, and the two that do not depend on a stop still pass.

What this does not do, named: the host process of a `--sandbox` run is not itself budgeted (its guest
is held by the jail); `delulu test` runs are not budgeted; budgets are not yet in the derived policy's
order or the Z3 model (PS-B-05).

**PS-B-01's run, read.** CI `36117814329` failed on `test (windows-latest)` only: two `hw_adapter_cli`
tests, "adapter did not answer within 2000 ms", each the job's first start of the Python test driver,
launched together. Re-running the failed job passed, and every other job passed first time. That is
not a diagnosis, so it was settled by experiment (the 2026-09-14 rule, "starve, don't guess"). CPU
starvation did NOT reproduce it: 0 of 8 runs failed with a high-priority hog pinned to the test's own
CPU, 0 of 5 idle. A cold interpreter did: the installed Python's first start took **4251 ms**, its
next 177 ms, because a plain `python` imports `site` and walks every package installed beside it; an
interpreter with no site-packages started cold in 470-564 ms and warm in 51-58 ms. The test driver now
runs `python -I -S` and is started once, untimed, before any test times it. `EXCHANGE_TIMEOUT` is
untouched, and a mutant driver that sleeps 3 s before answering still fails the same two tests with
CI's exact message. What the tests exposed about the PRODUCT is recorded, not fixed quietly: a real
driver's start-up counts against its first exchange (`REMAINING_WORK.md` 4.19, a D23 protocol change,
the owner's).

**PS-B-05.** A budget is an authority dimension now (D-V2-08's direction; D-V2-33 records how).
`crates/delulu-broker/src/budget_scope.rs` gives memory and processor time a containment order
(componentwise `≤`) and a meet (componentwise minimum), with an absent budget as the top: every node
written before this one keeps its meaning, and an absent budget under a present one is refused as a
widening. `Scopes.budget` joins `attenuation_check`'s conjunction, which is ten dimensions now. The
certificate parser reads `"budget"` strictly; an unbudgeted authority serializes exactly as before, so
no fingerprint or audit hash moved.

Through the CLI: `grants delegate --budget mem=BYTES,cpu=SECONDS` hands one down; a delegation that
names none inherits its parent's (at the tree's attenuation chokepoint, before `⊑` is asked); one that
names more is DL0802 with the meet as its repair. A `run --lease` is held to its node's budget, which
is also its default; `--limits` may ask for less and is refused, before `main`, when it asks for more.
A `--broker daemon` run's root records the budget the run is held to. The wire maps an unreadable
budget to the smallest one, never to none.

Proof and tests. The Z3 model gained a budget section (reflexive, transitive, antisymmetric, lower
bound, GLB, idempotent, commutative, associative, over unbounded integers) and its full conjunction now
carries all ten dimensions: **26 obligations**. It had modelled seven set dimensions while its
obligation said "all nine" (the code has eight); nothing proved was false, but the sentence claimed a
dimension the model did not carry. Extending it showed something worth keeping: deleting the budget
from the model's conjunction left all four product laws discharged, because they hold for ANY product.
One more obligation, "no budget under a budgeted parent is never `⊑`", is what fails then. CI now runs
the model's three mutants (a widening meet, the asymmetry backwards, the budget dropped) and requires
each to end with obligations NOT discharged. In Rust: exhaustive laws over the top plus a 3×3 grid, a
lease-module test of inherit/narrow/refuse, the wire's fail-to-smallest test, and one end-to-end test
through the real daemon in which an endless loop under a lease delegated `cpu=1` is stopped at one
second, not D-V2-25's five minutes. **Falsified by five mutants**: no inheritance, no `within`, no
lease cap, unreadable→none, and an unbudgeted daemon root — each fails a test at the assertion meant
for it.

**Found on the way, fixed: `run --sandbox` dropped most of `run`'s flags in silence** (`REMAINING_WORK.md`
4.20). Following the Survey's blast radius of the `Scopes` change into `guest.rs` showed the sandboxed
path reading six of `run`'s options. `--sandbox --lease TOKEN` never redeemed the token: the guest ran
holding nothing, and the DL0703 that followed told a lease holder to "pass `--grant console`", which is
how a holder would step outside its delegation. `--broker daemon` got embedded custody. A misspelled
flag, a flag with no value and a second file were accepted too, because the ordinary path's refusals
ran after the dispatch. The sandboxed path now calls those same refusals first and accepts only an
allowlist of the flags it applies; everything else is refused before anything runs. PS-A-10 wrote this
rule for the other direction (a sandbox flag without a sandbox); this is its mirror.

`doctor`'s resource line names the delegated budget. What PS-B-05 did not do, named: the host process of
a `--sandbox` run is still not budgeted by the main-program watchdog (its guest is held by the jail),
and a guest cannot run under a lease at all yet (4.20); a budget for `delulu test` runs does not exist.

**PS-B-05's run, read.** CI `36138186335` (`32a5b96`): Linux, Windows, arm64, lints, formal (with the
Z3 model's three mutants caught for the first time on a runner), fuzz, Miri, supply-chain and the
editor green; **macOS 1866 passed, 1 failed**. The failure was the new end-to-end lease test, and the
product was right: its temp directory's descriptive name put the broker's socket at 113 bytes, macOS
allows 103, and the broker refused to start with exactly that sentence. The test now uses a short
directory and asserts, on every platform, that the socket path would fit on a Mac, so the limit is
checked where it is cheapest rather than discovered where it is smallest.

**The fix's run, read.** CI `36139795158` (`ca60614`): every job green, all three operating systems.
PS-B-05 is closed with its runs read.

**PS-B-03 — identity separation (Windows), and T14 on macOS.** Until now every PS-A boundary held the
guest to what it may DO, and none changed WHO it is: on Windows the guest ran with the operator's own
token inside a Job Object, so a guest that escaped the interpreter could read `broker.key`, the root
policy and every file the operator can, and — since a Job Object restricts no sockets — open network
connections. D-V2-34 records the decisions.

Measured before a line of product code: a throwaway spike started a child in a fresh AppContainer
(profile created in 40 ms), loaded from a directory with one added grant, talked to it over inherited
pipes, and compared it with the same binary uncontained. Contained, it could not read the operator's
file, read a key in a state directory, list or write the operator's directory, or connect out; the
control did all five. A live `delulu` process loads exactly two modules from outside the Windows
directory — itself and `python313.dll` — which is what the runtime copy carries.

Built: `identity.rs` (the per-run AppContainer, the runtime copy, the contained launch), `pipe_channel.rs`
(the channel over inherited pipes, with its read deadline), a `--stdio` guest whose standard-output slot
is pointed at standard error before anything can print into a frame, and one `launch` used by a run and
by `sandbox probe`. The probe now proves more on that path: a contained guest is handed a program that
does nothing and must answer with its exit, so "it opened its channel" means it loaded, ran under its
identity and spoke the protocol both ways. The run report's posture follows what was applied: identity
`a per-run AppContainer`, reads `none of the operator's files`, network `only the channel`, writes
`only its own per-run container folder` (not rounded up to "denied"), and privilege escalation still
`not confined`, because nothing measured it.

**Found while building, fixed before commit:**
- Windows refuses to start an AppContainer from an explicit environment without `LOCALAPPDATA`
  (error 203, found by the first test run and settled in the spike: the loader's four variables fail,
  adding that one succeeds). It is passed, and the test checks the guest sees its container's folder.
- The first pruning rule — delete every other build's runtime copy — would have pulled files out from
  under a running guest of another build (the CLI and a test binary are two builds in use at once).
  Pruning is now by a day unused, and a broken copy never fails a run.

**T14, measured on two platforms and deferred on one.** Windows: `identity::win::tests::t14_…`, the
same attempts contained and not; falsified by starting the guest without the AppContainer attribute,
when it read the operator's file. macOS: the Seatbelt profile has to allow reads (every narrower
allow-list aborted the guest), so the guest COULD read the state directory; one deny after the broad
allow now refuses it, measured by `jail::macos_tests` with a control (a Mac is only reached through CI,
and this is its first run). Linux: a real second identity needs a subordinate uid (RW 4.21, PS-B-03b);
Landlock already refuses the state directory there where the kernel has it.

Mutants: the launch without the AppContainer attribute (T14 fails: the guest read the operator's
secret) and a host that never takes the contained path (the report test fails, and its guarantees show
the report did not claim an identity it lacked).

**PS-B-03's run, read.** CI `36142494033` (`d3355d6`): **success on every job** — Windows (the contained guest, T14 and the report test on a Windows runner), Linux, arm64, lints (which proves the
Windows-only items are gated for Linux clippy), formal, fuzz, Miri, supply-chain and the editor green,
and **macOS green with the new Seatbelt test running on the runner**:
`jail::macos_tests::the_state_directory_is_unreadable_to_a_seatbelted_guest ... ok`, control included.
That settles the one assumption this machine could not test — that Seatbelt applies the last rule that
matches, so a deny after `(allow file-read*)` wins — and makes T14 measured on macOS for the state
directory. The owner's word during the build was "use github for mac os run", and that is what it was.

**PS-B-06 — BREAK-GLASS.** D-V2-09 is the owner's principle: an authorized external principal may
relax a restriction, a program never its own. D-V2-35 records how. The restriction it breaks ships
with it: an operator may require the sandbox on a host (`delulu sandbox require --break-glass-key
HEX`), after which `run` without `--sandbox`, `test` and `repl` refuse before reading a line. The way
past it is a ticket (`delulu sandbox ticket`, run where the private key is): Ed25519-signed, naming
one program by the hash of its bytes, alive at most a day, spent on first use by an atomic create,
announced by a banner, carried in the run report (`break_glass`, `break_glass_ticket`) and recorded
in the audit chain as `break-glass` — and a use that cannot be recorded does not happen. It relaxes
the sandbox and nothing else. A `policy-off` ticket is the only way the policy comes off through
`delulu`.

Tests: six unit tests of the ticket (one use; every wrong shape refused and named, without burning a
good ticket; the day cap even for a ticket signed to outlive it; no policy / damaged policy; no
in-place replacement; hex), and three end-to-end through the binary. The main one walks the whole
matrix: no policy → required → run/test/repl refused while `--sandbox` works → a ticket opens exactly
one run, with banner, report and audit → the replay refused and recorded → strict again → another
program, an unpinned key, a tampered ticket all refused and a good ticket unburnt by them → `doctor`
reading it back, including the key left on the host → released by a `policy-off` ticket → a ticket
then has nothing to break → the audit chain still verifies. The other two: a use whose record cannot
be written does not run; a damaged policy keeps the host strict and `doctor` says so. **Six mutants
falsified**: policy ignored, replay allowed, signature unchecked, audit failure ignored, `test`
ungated, program hash unchecked — each fails the test meant for it.

Found on the way: writing the `sandbox` usage line through a shell heredoc flattened its `\`
continuation into runs of spaces mid-sentence, the defect class the message-spacing gate exists for;
caught on sight and rewritten as `concat!`, the rule the V2 memory already carries.

**PS-B-06's run, read.** CI `36145276674` (`1b9b7e8`): **success on every job**, all three operating
systems — the three break-glass end-to-end tests ran on Windows, Linux and macOS, and the lints job
confirms nothing Windows-only leaked into the Linux build.

**PS-B-04 — the measurement it was waiting for.** PS-B-04 ("channel batching for epoch-class effects,
after PS-A's measurement says it pays") rested on a measurement that had never been taken. It is now a
committed script with its rule fixed before any number was seen — batching pays only above **50 µs of
channel cost per effect** — in `measurements/sandbox-channel/` (`bench.py`, `RECORD.md`). On this
Windows workstation, release build: **47.55 µs** (N = 2,000) and **49.20 µs** (N = 10,000) under
today's transport, against **71.0 / 57.37 µs** under PS-A's named pipe (forced in a local build, then
reverted). So PS-B-03's identity made the channel faster, not slower. A number within 2% of its
threshold on one machine is not a verdict: `.github/workflows/channel-measure.yml` (manual) runs the
same script on the three CI runners, and the decision waits for those rows.

**PS-B-03b — measured before built.** RW 4.21 (a second identity for the Linux guest) depends on the
host letting an unprivileged launcher map a subordinate uid. `host-capability-probe.yml` gains
`linux-subordinate-uid`: a 0600 file read by a child mapped to the runner's own uid (the control, which
must succeed or nothing is measured) and by one mapped to a subordinate uid through `newuidmap`
(`unshare --map-auto`), which must be refused.

**Both runs read.** Probe `36167365278` (`7910497`): on Ubuntu 24.04's default the control itself
failed — `unshare: write failed /proc/self/uid_map: Operation not permitted` — because AppArmor
(`kernel.apparmor_restrict_unprivileged_userns = 1`) leaves an unprivileged user namespace without
capabilities; `newuidmap` is installed setuid and the runner has `165536:65536`, so what was missing
was the distribution's permission, not the kernel's mechanism. `RESULT: NOT MEASURED`. A second step
lifting that one sysctl was added (`4cd7680`) and run as `36167600016`: control `uid=0 … operator
exit=0`, experiment `cat: …/secret: Permission denied exit=1`, a world-readable binary still ran —
`RESULT subordinate-uid-separates-relaxed: PASS`. Channel `36167360827` (`7910497`), by slope: Linux
19.5 µs, macOS 19.8 µs (its N = 2,000 point carries a ~0.1 s fixed cost the method assumed cancelled),
Windows Server 2025 **66.9 µs** — over the line.

**PS-B-04 decided (D-V2-36): no batching; the Windows transport fixed.** Batching would change when an
effect is observed on every platform to recover a cost one platform has, so the cost was taken apart
first: PS-B-03's pipes were read through a drain thread and a queue each way, four thread wakes per
round trip where Linux has two. Removing the hops one at a time in a local build (reverted): 48.6 →
40.4 → 30.1 µs. Built: the host reads the overlapped server end of ONE duplex pipe (owner-only, one
instance, no remote clients, random name, the client end opened by the host at once and inherited)
directly with a real deadline, and writes are now bounded too; the guest reads its end directly and a
watchdog ends it if a read outlives the deadline. Workstation after: **28.9 µs** per effect. Also:
every frame is now one write (sandbox and broker channels) — no measurable change, kept because it
cannot cost anything. Tests: the host pipe's round trip, deadline, recovery after a timeout and end of
channel; the watchdog fires on a read past the deadline and never on idle time (falsified: a watchdog
that never fires fails it). The Windows CI runner's figure after the change is read from the next
`channel-measure` run.

**PS-B-03b — a Linux guest as a subordinate uid (D-V2-37).** Built from the measurement: the guest is
born in a new user namespace as uid 1 / gid 1, mapped by `newuidmap`/`newgidmap` (absolute paths) to
one id per run from the operator's `/etc/subuid` and `/etc/subgid` ranges (a range holding the
operator's own uid is refused), then drops every supplementary group and capability — before the
jail's own steps, because changing ids clears the parent-death signal. The binary is executed through
a descriptor (`/proc/self/fd/N`) so no runtime copy is needed; the channel is an inherited socket pair;
a READY byte lets the host fall back when a guest cannot load as the stranger. Landlock and seccomp
still apply inside the namespace (measured: a WSL run printed both). Where the host forbids it the run
falls back loudly and `doctor` gives the launcher's own reason. The posture claims less than Windows':
identity and reads (`only what every account on the host may read`), never writes or network.

Witnesses: T14 on Linux (`identity::linux::tests`) with the operator's control succeeding at all five
attempts and the stranger refused four, `PUBLIC=true` for a world-readable file, `UID=1`, `GROUPS=0`;
an end-to-end run (`sandbox_run_cli`) whose granted write still happens and whose report names the
identity; the posture unit test for the Linux words. **Falsified twice** in WSL (Ubuntu 20.04, kernel
6.6, `uidmap` installed there for this): skipping the uid change (`SECRET=true`) and skipping the group
drop (the `GROUPS=0` assertion) each fail T14. On CI the x86-64 Linux job now lifts the restriction and
sets `DELULU_REQUIRE_SUBORDINATE_UID`, so the identity tests there must measure; arm64 keeps Ubuntu's
default and exercises the fallback.

Found on the way (RW 4.23): a Linux run report never counts what the guest applied to ITSELF —
Landlock and seccomp are announced on standard error but not sent to the host, so `posture` lists
writes, network and new programs as not confined on a run that confined all three. Safe direction;
recorded, not yet fixed.

**PS-B-03b/04's run, read.** CI `36171523136` (`2fb0895`): **success on every job**. The Ubuntu test job
lifted the restriction (`kernel.apparmor_restrict_unprivileged_userns = 0`, logged) and ran with
`DELULU_REQUIRE_SUBORDINATE_UID: 1`, so `identity::linux::tests::t14_a_subordinate_uid_guest_…`,
`on_linux_a_host_that_allows_it_runs_the_guest_as_a_subordinate_uid_…` and the posture test all
MEASURED there and passed; arm64 kept Ubuntu's default and passed on the fallback; Windows ran the new
duplex-pipe transport; lints proved the Linux-only code clean for clippy.

**The Windows CI figure after the change, read.** `channel-measure` `36171534543` (`2fb0895`), by slope:
Windows Server 2025 **43.0 µs** (was 66.9), Linux 13.2 µs (was 19.5 — its plain guest's only change was
the one-write frame; recorded as observed, one run), macOS 18.7 µs (was 19.8). Every runner is under the
50 µs rule; PS-B-04 is closed on that evidence (`measurements/sandbox-channel/RECORD.md`).

**RW 4.23 fixed (same day).** The guest now tells the host what it applied to itself, as its FIRST
request — `Confined { applied }`, after its lock-down and before the program's first line, so the
words come from the toolchain and not from a guest the program has been running in. The host takes
it only first, only once, and only in the words of `channel::SELF_APPLIED`; anything else refuses the
whole report, and a guest whose report is refused does not run. The protocol is now
`delulu-sandbox-channel/2`. Witnesses: a unit test (known words kept once each; a second report, a
report after any other request, and an unknown word all refused and recorded), killed by two mutants
(any word accepted; accepted at any time); the Linux posture test now requires the guest's seccomp
filter to be counted. In WSL a run's report now reads `filesystem_reads: confined to the system
paths`, `filesystem_writes: denied`, `new_programs: denied`, identity the subordinate uid — and only
`network` under `limitations`, which is right for that kernel (6.6 predates Landlock's TCP rules).

**RW 4.23's run, read — PS-B CLOSED.** CI `36173241488` (`7a94684`): **success on every job**, Windows,
Linux (the identity required and measured), macOS, arm64 (the fallback), lints, formal, fuzz, Miri,
supply chain, the editor. PS-B is complete on 2026-09-26: PS-B-01 (every run budgeted), PS-B-02 (the
network client), PS-B-03 (Windows identity) and PS-B-03b (Linux subordinate uid), PS-B-04 (decided by
measurement: no batching, the Windows channel rebuilt), PS-B-05 (budget as the tenth `⊑` dimension),
PS-B-06 (BREAK-GLASS), and the four things found on the way (RW 4.19 recorded for the owner, RW 4.20,
4.21 and 4.23 closed). Nothing inside the phase was deferred.

## P4b–e — agent surfaces (opened 2026-09-26)

D-NE-6 was the decision this phase asks at its start; under the owner's delegation it is taken as
proposed (D-V2-38): `delulu mcp` in the CLI, stateless, hand-written JSON-RPC, read-only by
construction. The phase begins with the three introspection surfaces the zero-shot loop names first.

**P4-02 — `delulu toolchain [--json]`.** This toolchain as data, every field read from the table the
binary itself uses: the 35 commands with the invocations and options their `--help` documents (the
same `usage_lines`/`documented_flags` the option parser refuses by), the ten core effects, the 16
`--grant` forms each with an example that parses, the 82-entry primitive table (v5), the run budget and
the three sandbox profiles' limits, the five isolation levels, the two modes, the channel version,
all 154 diagnostic codes and the seven explanation topics. Three facts lived only in code and were
lifted into tables so they could be read rather than restated: `broker::GRANT_FORMS` (a test reads
`Grants::add`'s own match arms and fails on a key parsed but not listed, or listed but not parsed — its
first run caught an actuator example missing the mandatory dead-man), `codes::TOPICS` (bound to
`topic_explain`'s arms the same way) and `sandbox::LEVELS` (now what the probe itself names).

**P4-09 — `delulu schema [<name>] [--json] | validate <name> <file>`.** Nine JSON Schemas — envelope,
diagnostic, repair, authority, atlas, sandbox, policy, run-report, toolchain — CLOSED: each object lists
every field its emitter writes and forbids the rest, except two that say they are open. The validator
is in the binary (a subset of JSON Schema 2020-12, no dependency), so `schema validate` is the same
check the tests make. `tests/schema_cli.rs` runs the real emitters over a corpus chosen so that every
optional shape occurs — a foreign C block, the shipped NumPy example, a compute device, a plugin load, a
`@jit` hint, a diagnostic with a typed repair, an ordinary, a sandboxed and an audit-mode run, the policy
preview — and validates each through the binary. Everything validated on the first run, which is what
a test that cannot fail looks like, so it was falsified: a field added to the sandbox emitter was
refused ("matches none of the 3 allowed shapes"), and that message then learned to name the nearest
shape's actual problem.

**P4-10 — `delulu examples [--json]`.** The nine single-file examples embedded (so they read outside a
checkout), each with its opening comment as a summary, its source, the authority report `delulu
authority` prints for it — through the same function, `source_authority_report`, extracted from
`cmd_authority` for this — and a `delulu run … --no-prompt --grant …` line spelled from that report's
required grants. Bound to the directory by a test; the package examples are named with the command that
checks them. `examples_run.rs` now runs every listed line exactly as listed and requires that its grants
are enough (no DL0703), parse, and hit no checker bug (DL0907). One example fetches
`https://example.com/health`; the test depends on nothing about that answer.

**P4-03 — `delulu mcp`, read-only by construction (D-V2-38).** A Model Context Protocol server on stdio
(newline-delimited JSON-RPC; revisions 2025-06-18, 2025-03-26, 2024-11-05, the client's answered when
spoken and the newest otherwise), stateless — `tools/list` and `tools/call` answer with or without
`initialize` — with a deterministic, name-ordered tool list: `atlas`, `atlas_query`, `authority`,
`check`, `examples`, `explain`, `sandbox_policy`, `sandbox_probe`, `schema`, `toolchain`, `why`, and
inside the source tree `survey_query`, `survey_impact`, `doctor_check`.

How "read-only" is made true rather than declared: every CLI-backed tool runs THIS binary's `--json`
subcommand with an argument vector built from a fixed table (never a shell), so its answer is
byte-for-byte what the CLI prints — the protocol test compares the `check` tool's `structuredContent`
with `delulu check --json` directly — and a tool argument that begins with `-` is refused before any
vector is built, so no flag can be smuggled in. `every_tool_is_read_only` builds every tool's command
line and holds it against the commands that act (run, test, repl, grants, broker, guard, secrets,
keygen, sign, publish, login, deploy, fleet, fix, fmt, new, add, lock, build, plugin, locale, lsp,
mcp), allows `sandbox` only as `probe`/`policy` and `doctor` only with `--check`, and requires every
tool's `readOnlyHint`. Each call is bounded (120 s). The Survey tools answer in-process through
`delulu_survey::answers` — `query_json`/`walk_json` moved out of the `delulu-survey` binary into its
library for this, so the binary and the server print one shape.

Witnesses: `tests/mcp_cli.rs` drives the real binary — handshake and version negotiation, the tool list
(read-only, no effector, no Survey outside the tree), the CLI-identity check, a hostile argument
refused, `-32602` for the `run` tool that does not exist, `-32601` for an unknown method, `-32700` for
a line that is not JSON, no reply to a notification, and inside the repository the Survey tools
answering in the Survey's own shape (entrenchment always present). Falsified: letting a `-`-argument
through fails both the unit test (`atlas --grant=net=evil.example --json` smuggled an option) and the
protocol test.

Found by an existing gate while verifying P4-03: `actors::boundary_authority_tests::every_thread_either_
sizes_its_stack_or_is_listed_as_never_running_a_program` failed on the full suite, because `mcp.rs`
spawns two threads (draining a tool subprocess's stdout and stderr) and was not in the list of files
whose threads never run interpreter code. It is now, with that reason. The gate did exactly its job.

**P4-02/09/10's run, read.** CI `36176470110` (`bc5c01c`): **success on every job**, all three operating
systems — `toolchain`, `schema` (the emitter corpus validated through the binary on each OS) and
`examples` (every listed run line run) included.

**P4-03's run, read.** CI `36178164572` (`67d058a`): **success on every job**, all three operating
systems — `mcp_cli` (handshake, tool list, CLI-identity, hostile argument, Survey tools in-tree) green on
each.

**P4-04 and P4-05 — `delulu edit`, checked edits.** An agent edits a file it read a moment ago; if
someone else wrote in between, byte offsets computed against the old bytes land in the wrong place,
silently. `delulu edit <file> --expect-hash <blake3>` refuses that: the edit names the hash of the bytes
it was computed against and is refused (exit 2, nothing written) when the file no longer has them, the
refusal carrying the file's CURRENT hash so the caller re-reads rather than guesses. Two ways to say
what changes: `--edits` in the repair edit shape (a repair's `edits` pass straight through; ranges on
character boundaries, no overlap — two inserts at one point count as one, their order would be a guess
— applied all or none), or `--node <Atlas id> --with <text>`, which replaces one function, type or
actor member (the Atlas names an actor's `new`, behaviors and helper fns `fn:pkg/mod.Actor.member`)
with text that must parse as exactly that one kind of item and is formatted on its own, so the rest of
the file keeps its form. An id the file no longer has is refused as stale. The edited source is
CHECKED and the answer is one envelope — the new diagnostics, `previous_hash` and `hash`, whether it
was written — plus `edit.authority`: the effects, required grants and foreign calls before and after,
and `widened`, what the edit ADDS. That is the rule `fix` keeps for its own repairs (never a widening
one unless named), brought to an edit an agent computed itself. `--dry-run` writes nothing,
`--if-checks` writes only a clean result, a morph-stored file is refused, and the write is one rename.
The answer has a closed schema (`delulu schema edit`, the tenth).

Witnesses (`tests/edit_cli.rs`, six tests through the binary, every answer validated against the
published `envelope` and `edit` schemas): the roadmap's falsifier — *edit after the hash* — a second
writer changes the file between the read and the edit, the edit is refused, their write survives, and
the hash the refusal reported lets the same edit through; a real `add_effect_to_row` repair taken from
`check --json` passed straight through and checked clean; a node replaced by the id `atlas --json`
printed, formatted, neighbours untouched, then refused as stale after a rename; a widening edit named
(`Read`, `fs.read=./data`) on both channels; `--if-checks` and a morph file refused; and the **corpus
witness**: every function, type and actor member the Atlas names in `examples/` — 49 nodes in 14 files
— resolves by its id, and replacing each with its own text gives back the file's exact bytes.
Falsified: nine mutants killed (hash check skipped, overlap allowed, morph allowed, `--if-checks`
ignored, `widened` always empty, a stale id resolved to any fn, a second behavior or a field allowed
in a replacement, an unknown edit field allowed), and an undocumented field in the answer fails the
closed schema.

Found while building it: the corpus witness failed first on `fn:guide.actors/guide.actors.Supervisor.
dispatch` — the Atlas names actor members, which the first resolver did not know; they are resolved
now rather than refused. And the MCP door rule was a list of effectors only, which `edit` — a command
that writes — would have walked past had anyone added it as a tool. It is now two-way:
`every_command_is_declared_read_only_or_acting` requires each command to be in exactly one of
`READ_ONLY` and `EFFECTORS`, and a tool may run only a read-only one. Classifying every command found
`audit` belongs with the actors (`audit bundle --out F` writes a file). `edit` is deliberately not an
MCP tool.

**P4-06 — `delulu-survey diff <rev>`: what a change breaks.** `impact` answers for one node; a review
asks about a CHANGE — several files, some added, some deleted, some the map has never read. `diff` asks
git which paths changed since a revision (`git diff --name-status -z --no-renames <rev> --`, so the
working tree counts, plus `git ls-files --others --exclude-standard` so a new file counts; a range
`A..B` compares commits only), maps each to its node in the current map, names the changed nodes that
are ENTRENCHED with the owner and the CODEOWNERS line, and walks toward what breaks from all of them at
once. The walk is `Survey::walk_many`, a multi-source breadth-first walk: the union of the changed
files' impacts, each node once at its nearest distance, every hop cited and carrying the changed node
it traces back to (`origin`). `walk` is now `walk_many` from one start, and the traversal tests passed
unchanged. Nothing named is dropped: a path with no node is listed with `node: null`. A revision that
begins with `-` is refused before git starts (`git diff --output=F` writes F), in the library, in the
binary's option check and in the MCP `word` guard. The positional parser also stopped discarding every
number when `--depth` is present: only the argument after `--depth` is its value, so an all-digit short
commit id survives. `delulu mcp` gains `survey_diff` (in-tree only, read-only).

Witnesses (`crates/delulu-survey/tests/diff.rs`): the roadmap's *synthetic diff* — five changes
written down by the test (two source modules, an entrenched document, a deleted file, a path the map
does not read) — whose reach must EQUAL the union of the three nodes' own `impact` walks at their
minimum depths, every hop cited and traced to a changed file, the two unplaceable paths listed without
a node, and the Constitution named as entrenched; the git layer on a repository the test makes
(modified, deleted, staged-new with a space in its name, untracked; then a range that excludes the
untracked file); and the binary on this repository (`diff HEAD --json`, an unknown revision exit 2, an
`--output=` revision refused with no file written). `mcp_cli.rs` calls `survey_diff` and has an option
refused. Ten mutants killed (union reduced to its first start, entrenched dropped, `-` allowed,
untracked dropped, untracked kept in a range, a rename's old path not deleted, `origin` as the parent,
starts reported as reached, and both unit-level parses). A lesson from the mutant run: `cargo test
--lib NAME --test T` applies the name filter to EVERY target, so the integration tests silently ran
nothing and six mutants looked alive; each target is now run on its own.

**P4-04/05's run, read.** CI `36203028170` (`05f26d4`): **success on every job**, all three operating
systems — `edit_cli` (the stale-hash falsifier, the corpus witness) and the two-way MCP door rule green
on each.

**P4-06's run, read.** CI `36203900224` (`f943097`): **success on every job**, all three operating
systems — `delulu-survey diff` (the synthetic diff, the git layer on a made repository, the binary)
green on each.

**P4-07 — the Guard, read-only, in the editor; declared and performed rows on hover.** Two halves.

*Declared and performed.* The checker's per-function `facts[..].effects` is the DECLARED row — the
sound upper bound the authority report is built on — and what a body PERFORMS was computed at every
function boundary (for DL0501/DL0502) and then thrown away. It is kept now as a side table,
`CheckResult::performed`, keyed like `facts` (functions and actor members), with each body's row
variables settled once inference is — so an effect that arrives only through a row-polymorphic call
counts. It is never serialized and never read by the authority report, so no artifact or answer
changes. The language server's hover on a function name keeps `authority: <declared>` and, when the
body performs something different, adds `performs: <row>` and names the difference: declared but
never performed (DL0502) or performed but not declared (DL0501).

*The Guard view.* `workspace/executeCommand` with `delulu.guardStatus` (advertised in
`executeCommandProvider`) answers with what `delulu guard status --json` prints for the environment's
state directory: the Guard's mode, rules, pending requests and live permits — or, with no broker
running, the CLI's own `DL1401` "broker unreachable", which is the true answer (the Guard fails closed
without one). It runs this binary's `guard status --json` rather than the request in-process, because
in-process the CLI's printer would write onto the server's standard output — the protocol channel —
and so it shares `mcp::run_self`, now taking its deadline: ten seconds here, since a language server
answers one request at a time. Only `status` is ever sent. The VS Code extension adds **DeluluLang:
Show Guard status (read-only)**, which asks the server and shows the JSON; approving, denying and rule
changes stay in the terminal behind the owner code.

Witnesses (`lsp_cli.rs`): hover on four functions — declares more (`{Read, Write}`, performs `{Write}`,
DL0502 named), agrees (one row, no `performs:`), declares nothing and performs `Write` (DL0501 named),
and `Write` arriving only through a row-polymorphic `apply` (one row: the tail is settled); the Guard
view advertised, equal to the CLI's JSON with no broker (DL1401) and with a real broker started in a
test state directory (mode `on`, the default `declassify:*` rule), and the policy unchanged by asking.
`editor_contract.rs` holds the new editor command against the server's. Six mutants killed (performed
replaced by declared, row tails ignored — which first SURVIVED, until the row-polymorphic case was
added — the difference never shown, the DL0502 note dropped, the command not advertised, another guard
verb sent).

**P4-07's run, read.** CI `36205056652` (`c8b9b58`): **success on every job**, all three operating
systems and `editor` (the extension with *Show Guard status* built, packaged and verified to activate).

**P4-11 — the Atlas's V2 chain.** `delulu atlas chain <file|package> [--sandbox-profile P] [--json]`
joins what the Atlas, the authority report and the sandbox derivation each answer separately into
one ordered view: program → authority → effects → capabilities → sandbox policy → resources →
plugins → actors → devices → execution boundary. It is a view over `atlas/1`, not a new graph and
not the Survey: every fact comes from the Atlas this command builds (its nodes, `performs` and
`requires` edges, embedded authority report) and from `SandboxPolicy::derive` — the same pure
derivation `delulu sandbox policy` uses, so the same hash. The answer states its order as data
(`links`), because a JSON object has none. `carried` — would `--sandbox` take this program — is the
guest's own `unsupported_surface` on the file's text, so it cannot disagree with `sandbox policy`;
for a package it is `null` with the reason (`--sandbox` runs one file). Custody and foreign
isolation belong to a run, which is why the Atlas leaves them out; the boundary states an ordinary
run's from `cli::ordinary_run_mode`, the same stamps `delulu authority` applies.

Building it found the Atlas short of the chain it had to carry. Its resource classes covered six of
the capability kinds, so a program that drove an actuator, read a sensor, dispatched to a compute
device or hosted plugins had NONE of those in its Atlas — the resources a physical-safety or
supply-chain audit needs most. They are resource nodes now (`actuator`, `sensor`, `compute`,
`plugin_host`), with `requires` edges from the functions that wield them. And a resource node now
carries `requested_scopes` — the scopes the code names, `root.fs_read("./config")` → `["./config"]` —
where before the Atlas showed `res:fs_read:*` and the scope was only in the authority report (the
rest of NE-14). Both are additive: no existing id changed, and a node with no scopes has no new
attribute. `ForeignLoad` and `Python` stay out of the resource classes because the authority report
discloses them as the foreign boundary (`foreign` nodes, `foreign_calls`, spec §6). A first draft
derived "carried" from the report's capability list and said a Python program would be carried;
`sandbox policy` said it would not — caught by hand before any test existed, and the reason the
consistency test below exists.

Witnesses (`tests/atlas_chain.rs`): **the snapshot on the corpus** — the chain of all 16 example
targets (every single-file program and every package) is byte-for-byte its committed snapshot under
`tests/snapshots/atlas_chain/`, with no host path in any (`DELULU_BLESS=1` rewrites; a snapshot with
no example fails); the chain agrees with `sandbox policy` (hash, limits, refusal words), `authority`
(effects, grants, custody, foreign isolation) and the Atlas (resource ids, function ids) for every
example file; a program with an actuator, a sensor, a compute device and a plugin host shows each
with its scopes, in the chain and in the Atlas; the actor example lists its actors; the profile
changes the hash exactly as `sandbox policy` does; bad profiles, unknown options and a second target
are refused. The chain has a closed schema (`delulu schema chain`, the eleventh), the JSON contract
sweeps it, and `delulu mcp` offers it as `atlas_chain`. Six mutants killed (the actuator class
dropped, scopes dropped, `carried` always true, the profile ignored, `performs` read as `requires`,
custody hard-coded). The full suite then caught one more thing: `message_spacing`'s gate read the
human rendering's padded labels (`"program            {}"`) as flattened line continuations; they are
format widths now (`{:<18}`), the same output.

**P4-11's run, read.** CI `36206223062` (`fcd2c2f`): **success on every job**, all three operating
systems — the 16 chain snapshots hold byte-for-byte on Windows, Linux and macOS.

**P4-08 — the AI usability benchmark, and its first result.** `delulu-measure ai-usability` is the
harness `V2_AI_NATIVE_DESIGN.md` §4 designed: five knowledge conditions (never seen DeluluLang; the
Skill; the Skill plus `toolchain`/`schema`/`examples`/`explain`; the Skill plus MCP/LSP; the full
documentation), seven measures (compile first try, task completion, repair iterations, tokens,
authority mistakes, sandbox-policy mistakes, security-test failures), tasks generated from a
grammar (three operations × three ways in and out = nine), and a record,
`measurements/ai-usability/` (METHODOLOGY, REPORT, results.json, and every run's evidence).
`prepare` lays out one workspace per (condition, task): the task, the condition's documentation
copied in, and `dl.py` — a wrapper that lets through only the condition's `delulu` subcommands,
refuses the rest, logs every call and snapshots the program at every `check`, so attempts and the
repair loop are recorded by the harness rather than reported by the model. `score` computes every
measure from those files alone, with NEGATIVE CONTROLS per task on every scoring — the reference
solution must score perfectly (so every task is solvable in least authority and carried by the
sandbox), an empty file and an unparseable one must fail, the reference widened by one grant must be
exactly one authority mistake, and the reference that reaches for the out-of-scope canary must be an
authority mistake AND a security failure — and the result is INVALID if any misbehaves. What was not
measured is UNRUN, never zero; what cannot be measured on a program that does not check is null.

**The first result (a pilot — two tasks, one run each, per condition; Sonnet 5 as the subject; published
as it came out; `REPORT.md`):** with no documentation the model completed 1 of 2 tasks and never
compiled on the first try, needing 13 repair iterations on average; with the Skill alone, 2 of 2 but
still 0% first-try and 11.5 iterations; with the Skill plus introspection, the Skill plus MCP/LSP, or
the full documentation, 2 of 2, first try in 50%, 100% and 50% of runs, 0–0.5 iterations. No
authority, sandbox-policy or security-test mistake in any program that checks. Two runs cannot rank
the conditions, and the report says so; what the pilot does show is that the Skill alone carried a
model to completion but not to a first-try compile, and that is the gap it pointed at.

**What the benchmark found in the language — fixed.** With no documentation, the model wrote
`fn main(r: Cap[FsRead], w: Cap[FsWrite])`. It CHECKED CLEAN: nothing held `main` to the one argument
the runtime passes it (`interp.rs`: `call_fn("main", vec![root])`). Its authority report asked for
`fs.read=PATH` as if a capability could arrive from nowhere, contradicting §5.1 ("All authority
originates in the `Root` value passed to `main`"); at run time the `Root` landed in `r` and the first
method call faulted as DL0907 — "checker bug", which is what it was. `fn main(n: Int)` bound the
`Root` to an `Int`. Verified by hand against the binary, then fixed in the checker: `main` takes no
parameter or one `Root` (any name) — DL0403 for more parameters, DL0401 for a non-`Root` one, no new
code. Nothing in the corpus, the examples, the Book or the conformance suite changes (all green); one
checker unit test had `fn main(m: mathlib)` at the top of a call chain and now derives the handle
from its `Root` as a runnable program must, and another used `main` as an ordinary helper's name and
is renamed. Mutant: skipping the rule fails its test. The pilot was first scored before the fix
(kept in the pilot's storage); the record is scored with it, where that run does not check.

**What the pilot showed about the documentation — acted on.** Both Skill-only runs spent most of
their checks discovering the standard functions a first program needs. The Skill now carries "`main`
takes the `Root`, and nothing else" and "the kit a first program needs" (`str`, `parse_int` →
`Option[Int]`, `len`/`push`/`get` → `Option`, `split`/`trim`, `for`, no `unwrap`, the braced match arm
for an assignment or `continue`) — every clause verified against the binary first. The record says
which documentation each run read: `prepare` now writes the knowledge commit and whether those files
had changed; the pilot's are the files at `fcd2c2f`, before these paragraphs. A later run measures
whether they helped.

**What the replay gate caught in the harness.** `tests/ai_usability.rs` replays the committed runs
and requires the committed `results.json` and `REPORT.md`. Its first run differed in one number — one
sandboxed run "failed" on replay. The program was deterministic (six sandboxed runs out of six gave
the answer); the harness was not: `score` named its scratch directory by process id, the test suite
scores twice in one process in parallel, and each scoring's cleanup deleted the other's staged runs
mid-run. Each scoring has its own directory now, and the gate passes repeatedly.

Committing the record raised the Survey's `duplicated-file` warnings from 2 to 16: a run's final
program is its last snapshot, the same task is shown under every condition, a model re-checked
without a change. That rule's premise — one of two copies will be edited and drift — is about files
people edit; a measurement record's run evidence is written once and never edited, and its copies
are the facts it records. The Survey now leaves `measurements/*/runs/` out of that one rule, with the
reason in the code, and the count is back to its 2.

**Agent operations.** The ten runs were Sonnet 5 subagents launched together; a service session limit
killed one (`c1__t6`) mid-run after 12 attempts, and three finished runs' usage existed only in the
conversation when it struck. On resuming, those were saved first — each agent's final reply and usage
into `D:\nelan\DeluluLang-agent-transcripts\2026-09-26-ai-usability-pilot\FINDINGS.md` and the
workspaces beside it — and the killed run was resumed as the same agent with the same rules. Its token
count is UNRUN: the killed segment reported none, and the resumed one re-reads the whole context. From
now on each agent's findings are written to that file the moment it reports (the owner's instruction,
2026-09-26).

**P4-08's run, read — P4b–e is CLOSED.** CI `36261139018` (`11a7af1`): **success on every job**, all
three operating systems — including the benchmark's replay test, which scores the committed pilot again
(its programs run plainly and under `--sandbox`) and requires the published numbers, on each. Every
P4b–e item (P4-02..P4-11) is built with its run read green; the phase closed 2026-09-26.

One fix rides with this record: the benchmark wrapper wrote `calls.jsonl` in text mode, so on Windows
the committed logs were CRLF in the working tree and LF in the repository — the clean-checkout hazard
on record (a gate built from this disk can disagree with every runner). The working copies were
restored from the repository (the Survey regenerated to the same map, so nothing depended on the
bytes), and the wrapper now writes with `newline="\n"`: a fresh workspace's log is LF on every system.

**The closure commit's run, read.** CI `36262232077` (`769a646`): success on every job.

## PS-C — the microVM on Linux + KVM (opened 2026-09-27)

**PS-C-01, the prerequisites — all present, so the phase proceeded.** The record is
`V2_PS_C_PREREQUISITES.md`: each prerequisite, the command that checks it, what it answered. The
`EffectSink` seam and the channel needed nothing new — `serve_as_guest` already took any stream, so
the microVM's guest is the L1 guest with a different way of being connected. Checking found six things,
each fixed where it was found: libffi's musl build needs the kernel's headers and only those; a
`-ldl` that only glibc could satisfy (linking with `musl-gcc` "fixed" it into a binary that crashed on
start — two sets of C startup files); **the first-run language picker captured PID 1** (the guest's
standard input is the VM's console, a terminal, and `__guest` was not a machine command — on a host
its streams had always been a socket, which is the only reason this never showed); Firecracker v1.10.1
and a 6.1 guest were both past their support window, so the pins moved to **v1.17.0** and **6.18 LTS**;
KVM refuses `KVM_GET_API_VERSION` unless its variadic argument is exactly 0 (the first launch was
refused as "API version -1" on a host where KVM works); and `tinyconfig` silently dropped Landlock
(it depends on sysfs) — caught by the build's own check that every configured line survived.
Decision D-V2-39 records the pins, the kernel, the manifest, the ceilings and what stays the owner's.

**PS-C-02, the image.** `scripts/microvm/build-image.sh` builds, on Linux x86_64: the guest kernel from
kernel.org's 6.18.54 source (pinned by sha256) with `scripts/microvm/kernel.config` on top of
`tinyconfig` — vsock and nothing else, no IP stack, no PCI, no ACPI, no block layer, no modules — and
refuses a kernel in which any configured line did not survive; the static, Python-less, network-less
guest (`--no-default-features`, musl, paths remapped, stripped); and the initramfs, written by
`scripts/microvm/mkinitramfs.py` byte-for-byte deterministically (four entries: `/dev`, `/dev/console`,
`/dev/null`, `/delulu`; the device nodes are archive entries, so no root is needed). The manifest holds
the sha256 of each. The first build took 446 s on four cores: an 11.6 MB kernel, a 26.9 MB initramfs.
`scripts/microvm/check-reproducible.sh` builds twice from clean in different directories and compares
every hash; its CI job is `microvm-reproducible`. No built kernel is committed, uploaded or attached
(D-NE-27).

**PS-C-03, the launcher** (`crates/delulu/src/microvm.rs`, rewritten from a 58-line probe that
refused on every path). It checks KVM by opening it and asking its API version, finds the VMM
(`DELULU_FIRECRACKER` or PATH) and asks its version, reads the manifest, sweeps VM directories that
killed hosts left behind, and copies the kernel and initramfs into a new owner-only directory WHILE
hashing them — refusing a mismatch before boot and booting the copy, so the bytes checked are the bytes
booted. The VMM starts under `pre_exec` ceilings (no new privileges, killed with its host, processor
time, data = the guest's memory + 256 MiB, no core) and a wall-clock watchdog that can only signal an
unreaped process (it kills under the same lock that reaping takes); it is driven over its API socket
by a hand-written, bounded HTTP/1.1 client — machine config (one vCPU, the run's memory), boot source,
the ONE device (vsock), start — and never asked for a network interface or a drive. The guest dials
the host on vsock port 1024, sends the ready byte, and from there the conversation is L1's, over the
same channel, served by the same host code; the report, the policy and the audit records are shared.
The guest's console reaches the operator bounded at 64 KiB; the kernel's own messages are silenced.
A finished run waits a moment for the guest to power off, then kills, reaps and removes the directory.
Inside the VM, `__guest --vsock <port>` asks its kernel for an IPv4 and an IPv6 socket first and
reports "no network stack in its kernel" only when both are refused as an unsupported family — a
measurement, carried in the one confinement report the host already validates — then powers the VM off
as PID 1 must.

`--isolation microvm` is now a sandboxed run at level 2: the run report says `backend: microvm`,
`level: 2`, `requested_level: 2` (a real field now, where it had been the constant 1), and the posture
answers from what was applied — the host's files "none of the host's: the guest has no filesystem
device", privilege escalation "confined to its own kernel", identity "same OS user" with
`identity_separation` listed, because the jailer is not applied. `--sandbox --isolation microvm` means
the same; two boundaries on one command line are refused; a host that cannot give L2 refuses with
DL1408 as before, through one function both routes call. `sandbox probe` answers L2 from attempts that
end in a real boot. The first program ever run this way printed "hello from inside the microVM" — by
the host, on the guest's request.

**PS-C-04, criterion 8 — met, restated for the no-NIC guest.** Stage 5's contract imagined scope mounts
and an egress proxy the V2 guest does not have. `microvm_criterion8.rs` now asserts what the V2 guest
must: a program asking for a host the grant does not name gets no bytes out, while a control shows the
machine can reach that address; the granted host's request leaves from the HOST, as TLS, recorded in
the host's egress log and pinned to the granted address; the granted path is readable; a sibling path
is refused and its contents never reach the guest; and the guest's own kernel refused both socket
families (T9). With it, `microvm_cli.rs` — the lifecycle family: a run at level 2 (and its report); a
finished run leaves no VMM and no directory (T10); a flipped byte in either image file is refused
before boot (T13); the processor-time ceiling ends a spinning guest (T8); a guest that exhausts its
memory ends its VM, not the host, and the next run is unaffected (T8); a host killed mid-run takes its
VMM with it, and the next launch sweeps its directory (T10); two VMs at once, each output reaching only
its own host (T11); an uncarried surface refused before boot; and `sandbox probe` answering from a boot,
with a CONTROL — a fake VMM that answers `--version` like Firecracker and cannot boot must make the probe
report the launch failed, or the probe's own sentence would be all the test read. **All ten pass on a
KVM host (WSL2), and each claimed property was falsified**: with the image hash not compared, the VMM
not killed with its host, the orphan sweep off, the probe not booting, the processor-time ceiling
removed, or the VM directory left behind, the test that claims it fails. One kill was read by hand
rather than counted: with the hash check off, the tampered image BOOTED and ran the program — one
flipped byte in the middle of the initramfs did not stop the guest — while the report went on claiming
"the image checked against its manifest before boot". The check is the only thing between a tampered
image and execution, which is why it is done on the copy that boots. They are gated on
`cfg(delulu_kvm)`; the `microvm` CI job builds the image on a KVM runner and runs them there.

Everything that expected the old refusal was re-pointed at a host that is certainly not provisioned
(`DELULU_MICROVM_IMAGE` set to a directory that does not exist), because the KVM job IS a provisioned
host and a refusal test must not depend on the machine lacking something.

Open in PS-C: **PS-C-03b**, the jailer (a per-VM uid and chroot, which needs root); **PS-C-05**, a
distributed image (owner-reserved, D-NE-27); **PS-C-06**, the red-team record.

**PS-C-01..04's run, read.** CI `36266702402` (`ab1bfdc`): **success on every job** — and the two new
ones say what no local run could. `microvm`: on a GitHub `ubuntu-latest` runner, `/dev/kvm` answered API
12 once opened to the runner, the pinned Firecracker installed, the image built from source (kernel,
static guest, initramfs) in under seven minutes, and all ten gated tests passed — the lifecycle family
and criterion 8 restated — with `sandbox probe` answering "L2 microvm present" from a boot.
`microvm-reproducible`: two clean builds in different directories, **every hash identical** (kernel
`3a35d1f4…`, initramfs `1417a0f4…`, guest `4bb784e5…`, kernel config `148fec4b…`). Those differ from this
machine's build (kernel `f19758e4…`): a different C compiler builds different kernel bytes, which is why
the manifest, not the CLI, carries the hashes (D-V2-39) and why reproducibility is claimed per machine.

**PS-C-03b, the jailer (D-V2-40).** Run as root, the VMM now runs under Firecracker's jailer: a uid of
its own from a reserved block (reserved by creating a file exclusively, so two hosts starting at once
cannot collide), in a chroot the host fills before the VMM starts, entered by `pivot_root` in a new
mount namespace. Root WITHOUT a jailer is refused — the alternative is a VMM running as root. Two
things the first attempts found: the jails cannot live under `/run` (mounted `nodev`, so the `/dev/kvm`
node the jailer makes cannot be opened — the first jailed launch failed there with KVM's "permission
denied"; they live under `/srv/delulu-jailer`, and a `nodev` base is refused with that reason); and the
jailer's `setuid` clears the death signal, so a jailed VMM does NOT die with its host. **Measured**: host
and reaper both killed, the console pipe held open, the VMM was still running 15.7 s later, bounded only
by its processor-time ceiling. The answer is a reaper per jailed VM — a process blocked on a pipe only
its host writes to, which kills the VMM by pid AND jail id when the pipe closes, and removes the jail.

The jailer test (`as_root_the_vmm_is_jailed_as_its_own_uid_and_dies_with_its_host`) runs the host as
root (directly, or through `sudo -n` on the CI runner) and checks the uid, what the VMM can see (its
jail, none of the host's directories), that its mount namespace is its own, the report's identity row,
the reaper, and the refusal. **Its first version could not fail where it mattered**: a mutant reaper that
killed nothing survived an 8-second bound, because the VM was ending ~5 s after its host anyway through
its console pipe breaking — incidental, as the held-pipe measurement shows. The bound is 2 s now, only
the reaper is that prompt, and the mutant is killed ("outlived its host by 2.0s"). Its first chroot
assertion was wrong the other way — `/proc/<pid>/root` reads `/` from outside a pivoted mount namespace
— and now checks what the VMM can see instead.

**PS-C-06, the L2 red team** — the record is `V2_PS_C_RED_TEAM.md`. A native probe booted on the image's
own kernel (`scripts/microvm/redteam/probe.sh`) found one virtio device (vsock), `lo` and no NIC, no block
layer, IPv4/IPv6/packet/Unix sockets refused, no `/proc`, and nothing of the host's in the guest's
arguments or environment. Five hostile guests (`redteam/hostile.sh`: a 1 MiB console flood, garbage and
4 GiB frames, other ports and contexts, silence) each got no program run, the 64 KiB console bound, no
second channel and nothing left behind. It found three things, all fixed: the dial-in listener stayed
open until the ready byte was read, so a second dial could land in its backlog (never read — it is now
closed at the first accept); two channel errors reached the operator as OS text ("failed to fill whole
buffer", "os error 11"); and the launch record did not name the image booted (it does now, by the hashes
of the copies checked and booted, in the report too). Both scripts are gates on the KVM CI job.
Measurements (launch, per-effect and compute cost at L0/L1/L2) are in the record, and so is what was not
attempted.

**PS-C-03b and PS-C-06's run, read.** CI `36272234370` (`b2bff06`). The `microvm` job passed on the
KVM runner — the jailer test through `sudo -n` (the VMM as a reserved uid in its own mount namespace,
ended by the reaper within 2 s of its host's death, root without a jailer refused), the ten other gated
tests, and the native probe's gate. **Reading the job's log rather than its verdict found the
hostile-guest gate had passed vacuously**: the step passed the host binary as a relative path, the script
runs each mode from a scratch directory, and every mode exited 127 in a millisecond while the verdict —
which checked only that nothing ran and nothing was left behind — said every expectation held. The
binary is now resolved to an absolute path; a control runs the same program on the real image first and
must succeed; and each mode must show its hostile guest booted. Falsified both ways (`/bin/false` as the
host fails at the control; the relative path now resolves and the gate runs for real). Recorded as G1 in
`V2_PS_C_RED_TEAM.md`.

One job failed: on the Windows runner the actors ping-pong criterion measured a 1.39× speedup at four
threads against its 1.5× bar, best of two — a measurement of the shared runner's parallelism, in a
runtime this change did not touch, on a test that had passed on every push run since run 6. It was not
loosened; the failed job was re-run to see whether it repeats, and it passed — run `36272234370`
is **success on attempt 2**. Recorded as what it is: a criterion that measures the runner's parallelism
as well as the runtime's. If it recurs, the project's rule for timing tests applies (reproduce the
runner's starvation, judge against what the threads were actually given), never a lower bar.

**PS-C is COMPLETE (2026-09-27)** — PS-C-01 to PS-C-04 and PS-C-06, and PS-C-03b, each with its CI run
read green; **PS-C-05, a distributed guest image, is the owner's** (D-NE-27: distributing a built GPL
kernel is a licensing act), and nothing in the tree distributes one.

## P6 — documentation consolidation (opened 2026-09-27)

**`HANDOFF.md` is a briefing again** (D-V2-41). It had grown to 1,201 lines — a ledger, a feature tour,
two dated sets of numbers, a problems list, a per-platform record and six CI runs told in full, beside
the briefing it was meant to be. It is 522 now: where things stand, the standing rules (§1, with the
owner's gate for the final public repository word for word), what DeluluLang is (§2, with a paragraph
on what V2 has added), the Survey (§4), how to work here (§7, its stale counts corrected), starting fresh
(§10, its pointers re-aimed) and all of §11, the project's memory written down. The rest moved, verbatim
and under the same numbers, to `docs/archive/v1/HANDOFF_HISTORY.md`, and the briefing's last section maps
each moved section to what replaces it. Nothing was deleted; the two rule lines that spell the banned
word were left exactly where they were and not copied (the move script asserts both). One deviation
from the roadmap, recorded: §11.2–§11.6 stay, because they are rules in force, not history.

**README and the Book, to V2's state.** README's status table now says what V2 has done and what it has
not, with the current CI counts (1,942 tests on Linux x86-64 and arm64, 1,932 on macOS, 1,926 on Windows,
0 failed, run `36266702402`), and its command list shows `--sandbox`, `--isolation microvm` and `sandbox
probe`. The Book gains the microVM in Chapter 9 (and corrects a sentence that said a guest's identity was
never separated "on every platform, without exception" — untrue since PS-B-03), the machine surface in
Chapter 13 (with the benchmark's pilot result stated at the size it has), and V2 in Chapter 20. Checking
the Book's new benchmark sentence against the published report corrected it before it landed: the first
draft credited "the skill and the checker"; the report says the Skill alone did not help first-try
compiles in the pilot, and the Skill with introspection or the MCP server did.

**P6's run, read.** CI `36273691693` (`15c025d`): **success on every job**, first attempt. The
hostile-guest gate on the KVM runner ran for real this time: the control on the real image exited 0
with the program run, each of the five hostile modes booted and was refused (flood 10.4 s, garbage,
huge and ports ~2.7 s, silent 67 s by the channel's deadline), no VM directory and no VMM left behind,
and `microvm-reproducible` built the same kernel, initramfs, guest and config twice. Suite counts:
Linux x86-64 and arm64 1,942, macOS 1,932, Windows 1,926, 0 failed, 145 binaries. **P6 is COMPLETE.**

## P5 — distribution (opened 2026-09-27)

**D-NE-7 and D-NE-8, asked at the start as the plan says** — and answered under the owner's delegation
where the answer is the project's, left with the owner where it is his (D-V2-42). The release workflow
is hand-written around `scripts/package-toolchain.sh` (D-NE-8's first half: no `cargo-dist`); **no
installer scripts** are built — a `curl | sh` posture is a security-policy choice, so the documented,
tested manual path stands until the owner chooses otherwise. **Publication stays the owner's (D-NE-7)**:
a run not started by a `v*` tag builds, checks and installs the archive on all four targets and then
throws it away — nothing uploaded, nothing written to the public (and permanent) attestation log; a tag
keeps the archives and attests them; a draft release needs the tag AND `vars.RELEASES == 'on'`, which is
off.

**What P5 built.**
- `.github/workflows/release.yml`: four targets (linux x86-64 and arm64, macOS arm64, Windows x64), the
  archive checked to be the target the job claims, the binary checked to name that target and the
  commit the run checked out, a tag refused without its `## [X.Y.Z]` section in `CHANGELOG.md`, the
  install gate, and — for a tag only — provenance attestations, kept artifacts and (owner's switch) a
  draft release.
- `scripts/check-install.sh`, the install gate: it reads the one fenced block marked `install-gate`
  out of `INSTALL.md` and runs it as written — verify the download's `.sha256`, unpack, verify
  `SHA256SUMS`, put `bin/` on `PATH`, `delulu --version`, `delulu new hello`, `test`, `authority
  --grants`, `run --grant console`, and the same run `--sandbox`. The page cannot drift from the test.
  Falsified three ways: a mistyped command on the page (rc 2), a tampered `.sha256` (rc 1), the block's
  marker removed (rc 2). Run green on a real Windows archive and a real Linux archive built in WSL (the
  guest a subordinate uid there).
- `INSTALL.md` §1 rewritten around that block, with the targets, the macOS/PowerShell variants, the
  sandbox line, and the attestation and `--version --json` check under "Verifying".
- The archive ships the surface morphs beside `bin/`, where the binary now looks last
  (`morph_file.rs::search_dirs_from`, unit-tested), and the guided tour `examples/guide/` that
  `GETTING_STARTED.md` walks (the archive had not carried it). `INSTALL.txt` names the sandbox and the
  morph commands.
- `delulu --version --json` names the `target` it was compiled for and the `commit` it was built from
  (`crates/delulu/build.rs`); `null` when the builder did not say, never a guess; a local package of an
  uncommitted tree says `-dirty`.
- `crates/delulu/tests/distribution.rs` +5: morphs, skill and guide shipped; the install-gate block's
  steps; the workflow publishes only a draft, only on the owner's switch, and uploads/attests only for a
  tag (a mutant removing the upload's tag condition fails it); the version's CHANGELOG section; the
  version JSON.

**Found by walking the new install path, fixed: `delulu run . --sandbox` did not work on a package.**
The command a new package's own output points at, run sandboxed, died on the raw OS error — `Access is
denied. (os error 5)` on Windows, `Is a directory` on Linux — because the sandboxed path read its target
as one file. That is campaign finding C27's defect in the one path C27's fix never reached. A package now
runs sandboxed exactly as it runs without the sandbox: the ordinary run's own loader resolves, checks and
flattens it (`run_cmd::load_package_for_run`), the guest is handed that text, and the host checks it
before a guest exists. Walking it found two more:
- **The manifest did not bind a sandboxed run — the sandbox was LESS strict than no sandbox.** DL0701
  (main's row outside what the manifest declares) was checked on the ordinary path only. Witnessed
  against the pre-change binary: a file declaring `! {Write}` beside a `delulu.toml` whose `effects = []`
  was refused by `run --grant console` (DL0701, exit 1) and RAN under `run --grant console --sandbox`
  (printed, exit 0). The grants still bounded what it did — this was the manifest's ceiling ignored,
  not an effect without a grant — but a run that adds confinement must never drop a check. The check
  is one shared function now (`run_cmd::manifest_gate`), called by both paths, for a package and for a
  file beside a manifest; a mutant removing it from the sandboxed path fails the new test.
- **A sandboxed program that did not check got no diagnostic.** The guest could only say "the host sent
  a program that does not check" and hang up; the host now prints the checker's diagnostics — line,
  code, fix — and starts no guest.
- `sandbox policy <dir>` had the same raw-OS-error defect; it previews a package from the same
  flattened text now (same policy hash as the run), and `atlas chain` answers `carried` for a package
  instead of saying a package has no sandboxed run. Three Atlas-chain snapshots changed accordingly
  (`greeter` now `carried: true`; the two plugin packages keep `null` with the reworded reason).
- Two existing tests' "actor" programs did not check (`C()` for `spawn C()`); they had passed only because
  the surface refusal happened to come before any check. Corrected, and the test now asserts its
  program checks, so its refusal is the surface's and not the checker's.


## 2026-09-27 — P5b: the multi-OS agent pass (Haiku 4.5 and Sonnet 5, Windows and Linux), and what it found

The owner asked for agents, in the sandbox, on more than one OS, deliberately attempting what they
should not be able to do. Four ran against the P5 archives — Haiku 4.5 and Sonnet 5, each on Windows and
on Linux (WSL2) — with a brief, their own state directories, and their findings saved as they went
(`D:\nelan\DeluluLang-agent-transcripts\2026-09-27-p5-multios\`: `BRIEF.md`, `FINDINGS.md` with the head
chef's verification of each, and every agent's scripts and logs). Every claim below was re-run against
the current binary before it was believed.

**GUARD-SCOPE-1 (CRITICAL, fixed).** Guard path rules were matched by EXACT equality. A seal on a
directory sealed the directory entry and nothing in it: with `fs_read:<data>/secret` sealed, a lease
holder granted `<data>` read `<data>/secret/key.txt` — witnessed on Windows under a lease, as an ordinary
run and as a sandboxed guest. `fs_read:<data>/secret/**` was stored and answered `ok: … → sealed` while it
could match nothing. The addendum (§2.3) had promised the broker's own vocabulary, and for paths that is
containment. Now: a path rule covers its subtree (`guard.rs::path_within`, the broker's
`is_descendant_or_equal`); at mint and at request time a path scope is judged by overlap either way; a
glob is refused when the rule is set; case is folded on Windows and macOS and for any drive-letter path
(a Guard rule REFUSES, so the lattice's case-sensitive choice would fail open here). Recorded as D-V2-43.
Tests: `a_path_rule_covers_its_subtree_at_use_mint_request_and_permit`,
`a_glob_is_refused_and_a_drive_path_matches_in_any_case`; the broker suite 191/0.

**Broker custody for sandboxed guests (REMAINING_WORK 4.20's other half, closed).** A sandboxed run
refused `--lease` and `--broker`, so nothing the Guard said applied to a guest. The host now authorizes
every operation a guest asks for through the custody the run chose, before performing it, with the
interpreter's own op mapping (`channel.rs::HostChannel::with_custody`; `interp::custody_op_for`). A lease
is redeemed by the ordinary path's code (`run_cmd::redeem_lease`): its grants become the node's, a local
`--grant` beside it is refused, and its budget narrows the guest's limits (never widens them). The run
report says who decided (`custody: {mode, node}`). `crates/delulu/tests/sandbox_guard_e2e.rs` tells the
whole story through the real binary: `authority --grants` → broker → seal + guard → `grants delegate` →
the sandboxed writer DL1410 naming the exact `guard request` → request → DL1411 → approve → the write →
the sealed read DL1413 for the guest and for the ordinary run → a local grant refused → revocation →
`audit verify`.

**AUDIT-WRITERS-1 (fixed; found by that test).** `audit verify` failed DL1405 "prev_hash chain break" on a
chain nobody had touched. The chain was designed single-writer; sandboxed runs, `reconcile` and a run's
own records made it multi-writer, and each writer cached its head, so the daemon's next record chained
onto a head it had read before the run's. The append lock moved from `guest.rs` into
`delulu-broker::audit` (`AppendLock`); `AuditLog::open` recovers and `append` catches up UNDER it (the
anchor and the tail file's length are the fast check); a lock taken over from a dead writer carries a
nonce so the old holder does not delete it; `verify` re-reads a failing pass under the lock before
believing it. Tests: two handles interleaved across a day boundary, four threads × 25 records, a stale
lock taken over — the first two fail against a mutant with the catch-up removed.

**FS-RACE-1 (CRITICAL, fixed).** Containment was check-then-use: the path was canonicalized and checked
(C84, SYMLINK-DANGLE-1), then the lexical path was handed to `std::fs::write`, which resolved it again.
A link inside `fs.write=./scope` flipped in a loop between an inside and an outside target let
`write_text` land OUTSIDE the grant — the agent's own script, re-run by the head chef against the
archive binary: 75 of 150 runs wrote the payload into `outside/race_target.txt`. It reproduced under
`--sandbox` (the host performs a guest's effects with the same code). Who can race: not a DeluluLang
program (the language cannot make a link) — another process with write access to the granted
directory, the confused-deputy case a grant exists to bound. Fix: `delulu-runtime/src/beneath.rs` opens
the approved canonical path one component at a time from the top of the filesystem and follows no link:
`openat(…, O_NOFOLLOW)` on Unix (`O_PATH` on Linux), `NtCreateFile` relative to the parent's handle with
`FILE_OPEN_REPARSE_POINT` on Windows, a directory found to be a junction or link refused, listing by
handle. Every file effect goes through it — the interpreter's `read_text`, `list_dir`, `write_text`,
`append_text`, the wasm engine's `read_text`, and a plugin artifact's bytes (whose check had been lexical
only). The same agent script against the fixed tree, Linux: L0 150 runs and L1 `--sandbox
hostile-agent` 80 runs, the outside file untouched both times. Unit race test (one thread flipping a
directory between real and a link to outside, another writing and reading through it for 3 s): Windows
444 flips, Linux 25,739, nothing written or read outside; against a mutant that opens by path after the
check, a write lands outside (Windows, witnessed). Not closed, and said so in the module: a hard link (a
file that IS inside the grant), and the granted directory's own path, which is resolved at use as before.
`libc` and `windows-sys` became direct dependencies of the runtime at the versions the CLI already uses;
the lockfile gained nothing.

**FIFO (HIGH, fixed with it).** Reading a FIFO placed in a granted directory hung for ever. The open is
non-blocking and only a regular file is read or written; a FIFO, device or socket is refused at once.

**The raw-OS-error class (MEDIUM, fixed).** `sign <dir>` and 35 other sites printed `os error N`;
`cli::unreadable` says it in words (a directory, not found, permission denied, or the OS's text without
the number), and `tests/unreadable_paths.rs` sweeps every command `toolchain --json` lists. A grant key
that takes a value, given none, now says so (`--grant fs.read` was "unknown grant"), and a `--limits`
value that is not a number names the units.

**Found by the head chef while writing the next brief — GUARD-ALIAS-1 (CRITICAL; open at `5e2c8fd`, fixed in P5c below).**
The Guard is handed the LEXICAL path of a use (`interp::fs_scope_arg`, no filesystem access), and the
filesystem follows links. With `<data>/secretstuff` sealed and a junction `<data>/alias → secretstuff`
inside the grant, `read_text("alias/key.txt")` returned the sealed key under a lease, plain and
`--sandbox` (witnessed on Windows, script `guard_alias.sh` in the pass folder); the direct path was
DL1413. A link that stays inside the grant is allowed by containment, and git can carry one. The fix
pins the resolved path before the decision and opens exactly that path (next entry).

**Open from the pass (not yet fixed):** a memory or processor ceiling hit under `--sandbox`/`microvm`
kills the guest but prints the allocator's own text, reports no `outcome.stopped_by`, and on Windows the
process exit (68, 9) disagrees with the report's `exit: 1`; `--limits wall=` is refused under `--sandbox`
although the report lists a wall-clock ceiling; the host's serving work is not counted against `cpu=`;
`--report-out` is not written when a sandboxed run is refused before it starts; a dangling link read is
explained as "where a WRITE would land"; a 500-deep `../` path is echoed whole. The hard-link alias is the
documented residual it was (P20-R1).


## 2026-09-27 — P5c: agent pass 2 (Haiku 4.5 and Sonnet 5), GUARD-ALIAS-1, SANDBOX-STOP-1, GUARD-STALE-1

The owner asked for agents in the sandbox of more than one OS, deliberately attempting what they should
not be able to do, with authority from `delulu authority` validated by the Guard, end to end — first with
Haiku 4.5, then "use sonnet 5", then "use haiku 4.5 and sonnet 5". The archives were built from clean
clones of `5e2c8fd` (both name that commit, not `-dirty`). Pass folder:
`D:\nelan\DeluluLang-agent-transcripts\2026-09-27-p5b-haiku-guard\` — `BRIEF.md` (with the two addenda),
`FINDINGS.md` (every agent claim re-run or read against its logs, with the verdict), each agent's
`work-*` folder and findings file.

**What the agents established, with evidence** (the Haiku summaries overclaimed; only what their logs
show is counted): at L0 and L1 on Windows and Linux, and at L2 under a lease — a sealed directory's
files DL1413 at any depth and under case and `..` spellings, a file beside it readable; a guarded write
DL1410 naming the exact `guard request`; a wrong owner code DL1414; bypass does not lift a seal; a
tampered token DL1407; single-use DL1407; a local `--grant` beside `--lease` exit 2; the run report's
`custody` names the node, `sandbox.level` 2 in the VM; a corrupted audit copy DL1405; examples run with
exactly the grants `authority --grants` prints; the sandbox refuses actors, plugins and foreign code
legibly; a secret's value reaches neither stdout nor the report. The microVM tester's three "findings"
were its own test mistakes (a guessed DL code, a single-use lease reused, a request never filed) — the
head chef re-ran the first properly: a `--ttl 2s` lease reads at L0/L1/L2 while fresh and is DL1402 at all
three after 4 s (`verify-ttl-microvm.log`). The Sonnet 5 testers' deeper pass is recorded separately.

**GUARD-ALIAS-1 (CRITICAL, fixed) — found by the head chef writing the brief.** The broker and the Guard
were handed the LEXICAL path of a use (`interp::fs_scope_arg`), and the filesystem follows links. With
`<data>/secretstuff` sealed, `read_text("alias/key.txt")` through a junction `alias → secretstuff` inside
the grant returned the sealed key under a lease, plain and `--sandbox` (Windows, witnessed; the direct
path was DL1413); the 8.3 name `SECRET~1` and `SecretStuff` did the same against the pass-1 archive. It is
GUARD-SPELL-1's shape: a decision on one spelling, the object reached through another. The fix keeps the
broker purely lexical (ruling 2) and resolves at the edges that have a filesystem:
- the runtime PINS a file effect's path once (`prim::pin_fs`: every link followed, the result checked
  inside the grant on disk), the custody gate decides on that pin (`prim::disk_spelling`, no Windows
  verbatim prefix), and the effect opens exactly the pin with no link followed (`beneath::*_at`, carried
  through `EffectSink::cap_method_pinned` by the interpreter and the host channel, and by the wasm
  engine's read) — so a link swapped in after the decision is refused at the open (FS-RACE-1's walk);
- operator paths are stored resolved (`cli::broker_path`): `grants delegate --fs-read/--fs-write`, the
  root a `--broker daemon` run issues, `guard policy set/unset`, `guard request --use`, and a
  `--guard-policy` file's rules. (A relative rule pattern is still refused, as GUARD-SPELL-1 decided.)
Verified: every spelling DL1413 on Windows (direct, junction alias, `SECRET~1`, other case) and on Linux
at L0, L1 AND L2 (symlink alias). `sandbox_guard_e2e.rs` step 4b asserts it for the guest and the ordinary
run; a mutant that decides on the lexical path again fails it with the key read. Unit tests: the pin is
where the path resolves and nothing outside is pinned; the verbatim prefix never reaches the broker.

**SANDBOX-STOP-1 (HIGH, fixed) — from pass 1.** A ceiling hit under `--sandbox`/`microvm` stopped the
guest but the operator read `memory allocation of N bytes failed` / "closed the channel without saying
goodbye", the report had no `outcome.stopped_by`, the Windows exit (68, 9) disagreed with the report's 1,
and `--limits wall=` was refused although the report listed a wall-clock ceiling. Now a stop is named only
from evidence:
- memory: a global allocator (`ceiling.rs`) ends a guest whose allocation is refused with status 197
  (the program cannot choose the guest's status); a microVM guest also writes a fixed line on its serial
  console, which the host's relay recognises and removes (the program cannot write to that console);
  on Windows also the job's peak-memory accounting;
- processor time: `SIGXCPU`; or `SIGKILL` (what equal soft and hard limits deliver, and also what a host's
  death or an operator sends) only when the OS's accounting of the reaped guest (`RUSAGE_CHILDREN`,
  before/after) shows the budget spent; on Windows the job's processor-time accounting;
- wall clock: `jail::Limits.wall_seconds` — a host watchdog for a process guest, stopped and joined before
  the guest is reaped so it can never signal a reused process id; the tighter of it and the VM's own
  ceiling for a microVM.
The run says the stop in the ordinary run's words, `outcome.stopped_by` carries dimension, budget,
observed (when measured) and `source`, the audit `sandbox-limit` record carries it, and the exit is 1 as
the report says. `wall_seconds` enters the policy's JSON and hash only when set, so every existing policy
hash is unchanged. Verified on Windows (memory, cpu with the job's measurement, wall) and on Linux: memory,
cpu (SIGKILL, 3.0 s accounted) and wall at L1; cpu (5.0 s accounted on the VMM) and wall (the VM's watchdog)
at L2 (`verify-fixes-linux.log` in the pass folder);
`sandbox_run_cli.rs::a_ceiling_that_stops_a_sandboxed_run_is_named_and_the_exit_agrees_with_the_report`.
**The memory stop at L2 — closed in P5d.** Two layers were missing. The image was rebuilt from a clone on
the Linux filesystem (from `/mnt/d`, `libffi-sys`'s configure failed); with the new guest the stop was STILL
unnamed, because nothing limited the guest's own address space inside the VM: Linux overcommitted, the
allocator never saw a refusal, and the guest kernel's OOM handling ended the VM silently. The microVM
guest now caps its own `RLIMIT_DATA` at 85% of the memory free when it starts (`sysinfo`, no `/proc`;
reported as the new known word "memory refused to the guest before its kernel runs out"), so the allocator
meets the ceiling first and `ceiling.rs` writes its console line. The next run named the stop — but the
line also reached the operator, because the serial console delivers a line in pieces; the relay now passes
the console a LINE at a time (a partial line held to one read's worth) and withholds that line whole.
Verified in WSL with the rebuilt image: `stopped_by` memory, source "the microVM guest's allocator", the
line withheld, an ordinary program unaffected, `microvm_criterion8` green, `microvm_cli` 9/10 (the tenth
needs root, which CI's microvm job has) — `verify-vm-memory3.log`, `verify-vm-relay.log`.

**GUARD-STALE-1 (MEDIUM, fixed) — found by the head chef's own end-to-end test.** Extending
`sandbox_guard_e2e.rs` with deny and a single-use permit: once the new request's permit was spent, the
next write was refused DL1412 with the OLD denial's comment, not DL1410; a retry while the new request was
pending would have said DL1412, not DL1411 with the new id. `guard_verdict_use` took the first (oldest)
matching request. Fail-closed throughout, but the principal's latest decision was misreported. Fixed
(`iter().rev()`); unit test fails against the old line.

**Lesson, recorded in memory:** Haiku 4.5 summaries overclaimed (checks marked done whose logs show they
never ran); a pass is judged on its logs, briefs now ask for each check's command and verbatim output, and
the headline behaviours are held by the head chef's own tests regardless.


## 2026-09-27 — P5d: `Secret.verify` under a lease (VERIFY-FABRICATED-1), SECRETS-STALE-1, and CI after P5c

**The session limit** cut the two Sonnet 5 testers off mid-pass; both had written checkpoints on the
head chef's instruction, and both were RESUMED with `SendMessage` after the reset (the standing rule:
resume, never respawn). The first CI runs of `5e2c8fd` and `0940ea1` were red on all three OSes; the causes
were fixed in `f284a37` (below) before anything else.

**CI after P5c (`f284a37`).** (1) Windows: a lease's scope is stored as the path it resolves to
(GUARD-ALIAS-1), but minting a capability compared it LEXICALLY with the program's path, spelled from the
runner's 8.3 working directory (`C:\Users\RUNNER~1\…`) — a legitimate `fs_read` was refused DL0703. The
mint is decided on disk only now (interpreter and wasm); `sandbox_guard_e2e.rs` step 4c names the granted
directory through a link and the 8.3 name, and fails against the old lexical gate. (2) Linux/macOS: with
no broker, `grants`/`guard`/lease runs printed `No such file or directory (os error 2)` — caught by the new
sweep (`unreadable_paths.rs`); `cli::broker_unreachable_detail` says it in words at all four sites. (3)
macOS: the e2e test's socket path was 109 bytes (limit 103) — shortened.

**VERIFY-FABRICATED-1 (found by BOTH Sonnet testers, independently; filed by them as CRITICAL — the
head chef's verdict: a real defect, but not a disclosure).** Under a lease a program's secrets are broker
HANDLES with no bytes in its process. `Secret.verify` was never routed to the broker: `SecretVal::verify`
of two handles returned a constant `false` (the code said "broker-side verify is post-chunk-3", flagged and
then forgotten). The Windows tester proved it with byte-identical broker-held secrets: `match: false`, even
with `declassify:*` SEALED, and no declassify record in the chain. A constant carries no information, so
nothing leaked; but the answer was fabricated — a program checking a token under a lease was told "no
match" for a match — and the Guard's decision was skipped. Fixed: `Broker::verify_guarded` computes the bit
where the bytes are (the node must hold `Declassify` and BOTH secrets in scope, the store must hold both),
the Guard decides for EACH secret (the bit depends on both; a permit on one does not buy the other), the
comparison is constant-time over equal lengths, and a `verify` record carries the decision; the wire gains
`ReqBody::Verify`/`Response::Verified`; `Custody::verify` (default: refuse — a custody without the bytes
must never answer); the interpreter routes two handles to it, two locals compare as before, and a mixed
pair is refused in words. Tests: broker units (the bit from the bytes; per-secret Guard), and
`secret_verify_cli.rs` through the binary (DL1410 until approved; then `same: true`, `diff: false`; the
chain records it). A mutant that removes the interpreter's route prints `same: false` with no Guard decision
— the original defect — and fails the test.

**SECRETS-STALE-1 (found writing that test).** `delulu secrets set` writes `secrets.json` from its own
process; the daemon read the store once, at start. A secret stored while the broker ran was refused
"not in the store" until a restart — which is what the testers' "SECRET-DELEGATE-DEAD-1: `expose` under a
lease can never succeed" was (the Windows tester had run `secrets set` after starting the broker). Fixed:
`SecretStore::refresh` re-reads the file before every `expose`, `verify` and `map` (a removed file is an
empty store; an unreadable one keeps what was loaded). `secret_verify_cli.rs` stores the secrets AFTER
the broker starts and exposes one after approval; a mutant without the refresh fails it. Also: `grants
delegate --secret NAME=VALUE` stored the whole string as a name no program could mint — refused now,
pointing at `secrets set`; and a secret the broker does not hold says how to store it rather than naming a
scope dimension.

**The Windows Sonnet tester's pass (finished; resumed once).** Backed by its logs: the permit machinery
(request/deny DL1412, approve with uses and TTL, exhaustion, permit revoke, no cross-node or cross-pattern
use, a sealed path not approvable); delegation trees (grandchild, cascading revocation at L0 and L1, effect,
scope and budget attenuation DL0802, no window after a parent's revocation); concurrency (five parallel runs
racing a seal and a revocation: no unauthorized effect at L1 or L0); the daemon as a target (stopped →
DL1401, cross-broker tokens DL1407 both ways, another state directory harmless); audit tamper (a byte, a
deleted record, a tampered bundle reconciled — all DL1405 or refused). Its remaining MEDIUM, recorded for
the owner rather than changed: a broker restart forgets delegated nodes and permits (fail-closed: a stale
token is "unknown node"), and the chain's `seq` restarts at 1 per daemon life (the chain's integrity is its
hashes and anchor — D-V2-43 item 2).

**The Linux Sonnet tester's pass (finished; resumed once).** It found VERIFY-FABRICATED-1 independently and,
on retest, confirmed the constant `false` for equal and unequal secrets alike; its "expose never works even
with `secrets set` first" used the order broker start → `secrets set` → delegate, which is SECRETS-STALE-1
exactly (the e2e test now uses that order and exposes after approval). Two more, both fixed here: a unix
**socket** in a granted directory surfaced `os error 6` (ENXIO) where a FIFO says "not a regular file" —
`beneath.rs` now maps ENXIO/ENODEV to the same words (Linux unit test with a bound `UnixListener`); and
`SKILL.md` said a program using secrets exits 2 under `--sandbox` before anything runs, while the sandbox
refuses a secret at its first use (exit 1, a channel-violation record) — the page now says what happens.
Backed by its logs: the permit machinery, delegation trees with cascading revocation through a grandchild,
the daemon as a target (no broker, two brokers, MAC-bound tokens refused across brokers, a garbage state
directory), audit tamper (subtle edit, deleted record, tampered bundle), five parallel sandboxed runs losing
the race to a revocation with nothing written, items 1/2/4 under `--isolation microvm` matching L0, FIFO and
`/proc/self` refusals. Informational, recorded for the owner: `grants revoke` needs no owner code (revocation
only narrows; a same-account process that can read a node id can end that lease).


## 2026-09-27 — P5 closes (the release dry run); P7 begins: NIST vectors, parser fuzz targets, Miri shrinks

**P5's last item — the release workflow's dry run on CI — is done.** Run `36332086632` (manual, on
`56eb5a6`): all four targets packaged, checked (target and commit named), and INSTALLED through
`INSTALL.md`'s install-gate block; the `release` job skipped (no tag); **0 artifacts, no release, the
attestation and "keep" steps skipped** — D-NE-7 held in practice, not only on paper. `gh attestation
verify` waits for the owner's first `v*` tag, which is the owner's act.

**CI after P5d.** `f284a37` green on every job (run `36330278053`). `56eb5a6`: every job green but macOS,
where the new unix-socket test's socket path was too long (104 bytes there) — fixed under `/tmp`.
`5065be1`: macOS again, two real ones: a unix socket on macOS answers `EOPNOTSUPP` (os error 102), not
Linux's ENXIO — mapped to the same "not a regular file" words (a product fix, not only a test fix); and
`unreadable_paths.rs`'s two tests could get ONE scratch directory, because macOS's clock resolves to
microseconds, and the short test deleted it under the sweep — a counter now joins the clock.

**The agents' verification of the fixes (pass-2 folder, `FINDINGS.md`).** On archives of `5065be1` built
from clean clones: the Sonnet 5 Linux tester found **all five fixes holding** at L0, `--sandbox` and the
microVM — including a 220-read race flipping an alias between a public and a sealed directory (no leak),
and "a permit on only one secret does not let `verify` through" — and the Haiku 4.5 Linux tester items 2, 4
and 5 holding, both with verbatim evidence. The Sonnet 5 Windows tester was stopped by the model's WEEKLY
usage limit mid-way; its file holds what it did, and it is resumed after the reset.

**P7 (verification depth), the three head-chef items:**
- **RW 4.10a — the NIST vectors, wired.** `crates/delulu-runtime/tests/nist_kat.rs` checks every saved case
  the pinned crates' public API can express, byte-exact against NIST's answer keys: ML-DSA-65 keyGen (5,
  pk and expanded sk), sigGen (15 — deterministic and randomized, external and both internal forms), sigVer
  (9, both verdicts present), ML-KEM-768 keyGen (5, ek and expanded dk), encaps/decaps and both key checks
  (12). The groups it cannot express are NAMED in the test (HashML-DSA — the crate has no pre-hash API and
  this project never pre-hashes; randomized signing over a precomputed μ). Each file's NIST provenance is
  asserted. A mutant with the wrong FIPS 204 domain separator fails it. This does not make PQC stable:
  D14b's second gate, an independent audit of the crates, stands, and `--unstable` (DL1910) with it.
- **RW 5.4 — the parser fuzz targets.** Five new targets beside `channel_frame`, chosen because an attacker
  controls the bytes: `source`, `manifest`, `dir`, `lease_token`, `grant` (D-V2-45). As for the channel,
  each property is one function in its crate that the ordinary suite replays over a seeded, deterministic
  mutation corpus on every commit; CI's `fuzz` job runs each target 30 s coverage-guided under
  AddressSanitizer (named in the command — it is cargo-fuzz's default, and the log should say it).
- **RW 5.6 — the Miri-slow tests, shrunk under `cfg!(miri)`.** Every test the two measured runs named as
  stalled or in flight: the Guard's policy generator, the nesting refusal, the path canonicalization
  generator, both DIR flip sweeps, the manifest's oversized name. Natively each keeps its full size (the
  100,000-level nesting input is a witness that must run at full size, and does, on every push). The
  measurement is a `miri-slow` run, started after this push.
- RW 5.2 (restating Progress as progress-or-fault in `DELULU_CORE.md`) is the owner's: the file is
  entrenched.

## 2026-09-28 — PS-D-01: an external launcher carries the guest (L3); the Windows verifier's two findings fixed

**The Sonnet 5 Windows verifier finished** after the weekly reset (pass-2 folder,
`work-sonnet-win-verify-FINDINGS.md`), on a clean build of `5065be1`: **all five fixes hold on Windows** —
GUARD-ALIAS-1 through a junction, a case-different spelling, an 8.3 short name and `..` traversal at L0 and
`--sandbox`, plus 40 reads racing a junction flipped between a public and a sealed directory (22 public,
17 refused, 0 sealed bytes); SANDBOX-STOP-1 for memory, processor time and wall clock; VERIFY-FABRICATED-1
(a permit on one secret of the pair does not suffice); GUARD-STALE-1; `--secret NAME=VALUE` refused. The
head chef read its logs, not its summary. It also recorded two observations; both were real, and both are
fixed with a test that failed first:

- **JSON-EXIT-1.** Under `--json`, a failure that printed no envelope of its own collected the fallback
  envelope, which said `"exit": 2, "kind": "usage"` whatever the exit was. A sandboxed run stopped by a
  ceiling ran and exits 1, so a caller reading the envelope saw a usage error — the two codes
  `[agents.exit-codes]` says never to collapse. `exit` is now the process's own code and `kind` is `usage`
  only for 2, `failed` otherwise. Gates: the contract sweep asserts `error.exit` equals the exit for every
  failing shape, and the ceiling test asserts it for a stopped run (on the old code: 2 against 1).
- **SANDBOX-CPU-LATE-1.** On Windows the guest's processor-time ceiling was the Job Object's
  `PerJobUserTimeLimit` alone, and Windows checks it late: `cpu=3` stopped at 5.1, 5.9 and 8.1 s, `cpu=6`
  at 12.9 s. Not an accounting artefact — sampled mid-run, the guest was 5.94 s user and 0.13 s kernel on
  one thread. The host's watchdog thread now also reads the job's accounting every 100 ms and ends the
  guest at the budget (2.03, 3.09 and 6.0 s measured); the job's limit stays as the backstop, and the stop
  is named as the host's watchdog. The ceiling test bounds a 2 s budget below 3 s; the mutant with the
  watchdog disabled ran to 8.7 s and failed it. Linux's `RLIMIT_CPU` was never late.

**PS-D-01 — `--sandbox --sandbox-backend external:CMD` (L3, D-V2-46).** The operator's launcher runs the
guest in THEIR environment — Docker with gVisor, Kata, a cloud sandbox, `ssh` — and carries the channel on
its standard input and output. The guest still holds no authority: every effect is decided and performed
by the host under the grants, the lease and the Guard, exactly as at L1 (the test's ungranted effect is
refused `DL0703`; its granted write is made by the host). What changes is who vouches for the wall, and the
report says so: level 3, backend `external`, `fully_enforced: false`, no host guarantee, the launcher's
program named and its arguments never recorded. A launcher that does not exist or exits without a guest
ends the run promptly and in words. `DEPLOYMENT.md` carries a Docker + gVisor + `--network none` recipe,
documented and not shipped, with the two things it does not do (ending the `docker` client does not end
the container; `--memory` is the launcher's enforcement, not DeluluLang's). Its stale "the microVM does not
exist yet" note is corrected. Tests: `sandbox_external_cli.rs`, the binary itself as the launcher —
Windows 2/2, Linux 3/3 (the third, Unix-only, checks the words and limits the launcher is told).

**CI.** `4b583e4` (P7's first half): push run `36336871608` GREEN on every job, the six fuzz targets
included. **The `miri-slow` measurement RW 5.6 asked for** (run `36336888036`, 240 minutes each): 0 UB
anywhere. `delulu-syntax` **finished** — 130 passed, 1 ignored, in 690 s, where it had stalled for over
three hours; the shrinks did what they were for. `delulu-check` ran 91 of 248 and then spent the rest of
its budget in ONE test, `rcaps::tests::criterion8_generated_sequences_never_reach_incompatible_aliases`
(10,000 generated sequences) — now 20 under Miri, 10,000 natively; the four `prim_table` sweeps that
type-check a program per row (eight minutes each there) visit every eighth row under Miri and every row
natively (D-V2-45). And `delulu-broker` ran 166 of 167 and **failed one**, which is a finding:

- **AUDIT-LOCK-TAKEOVER-1 (the audit chain's append lock).** `parallel_writers_leave_one_verifiable_chain`
  failed under Miri with the chain disagreeing with its own anchor, and a writer that could not take the
  lock at all. AUDIT-WRITERS-1's lock was a `create_new` file that a waiter TOOK OVER after five seconds,
  so that a writer killed mid-append could not block the chain. But a holder that is only slow is not
  dead: Miri is a hundred times slower, a waiter took the lock from a live writer mid-append, and both
  wrote. Natively the same happens to a paused process, a slow disk or a swapping host — and `verify` then
  reports an untampered chain as truncated. Two waiters timing out together could also each delete the
  other's fresh lock and both hold it. Witnessed natively by a new test, `a_slow_holder_is_never_taken_over`
  (red on the old code: the second writer took a held lock). **Fixed:** the lock is now the operating
  system's (`flock` / `LockFileEx`, through `File::try_lock`) on a file that is never deleted. The OS
  releases it with its holder's handle, including when the process dies, so nothing is ever taken over;
  a wait that runs out fails the append in words, and the daemon refuses a synchronous operation it could
  not record (invariant 26), rather than writing beside another writer. A new test kills a child process
  holding the lock and takes it straight after. Its Miri measurement is CI's: the `miri-slow` run started after this push (a local Miri run had written 54 of the parallel test's 100 records, with no error, when it was moved there to spare this machine's memory).

PS-D-02, the attestation seam with a fake attester, is next.

## 2026-09-28 — everything on GitHub for `30a6b8d` (the owner: "run everything on github")

Five runs on `30a6b8d`, each read:

- **CI push `36367985299` — GREEN on every job.** Tests: Windows 1,967 passed / 0 failed (14 ignored),
  Linux 1,985 / 0 (15), macOS 1,975 / 0 (15), 150 binaries each; clippy, the six fuzz targets under
  AddressSanitizer, Miri (diag, atlas, FFI), formal, supply chain, the editor, arm64, the microVM and its
  reproducible image all green. `sandbox_external_cli.rs` (PS-D-01) and the new ceiling assertions
  (JSON-EXIT-1, SANDBOX-CPU-LATE-1) ran on all three.
- **CI heavy `36368020490`** — `heavy-gates` green. `miri-slow`, the measurement: **0 UB in all three crates, and for the first time every test in them ran to the end.** `delulu-syntax` 130 passed (1 ignored) again; **`delulu-broker` 168 passed, 0 failed, in 176 minutes — the AUDIT-LOCK-TAKEOVER-1 fix holds under Miri**, the parallel writers included; `delulu-check` ran all 248 and failed one, `dir::tests::verify_is_comfortably_fast_for_per_load_use`: a wall-clock bound (5 ms a verify) that measured 14.8 s under Miri and spent 49 minutes of the budget failing. Its witness IS the clock, which Miri cannot judge, so it is ignored under Miri only and still runs natively on every push (D-V2-45); the same `verify` runs under Miri in every other test of its module. The `miri-slow` run after this push is the one expected to be green on all three.
- **Release dry run `36368022181` — green, nothing published:** all four targets packaged, checked and
  installed; the `release` job skipped (no tag); 0 artifacts, no release (D-NE-7 held again).
- **`channel-measure` `36368024526`** — every runner still under PS-B-04's 50 µs rule (slopes: Linux
  19.0, macOS 28.0, Windows 41.1 µs); Linux and macOS rose since `2fb0895` (13.2, 18.7), recorded as
  observed and not diagnosed in `measurements/sandbox-channel/RECORD.md`.
- **`host-capability-probe` `36368026333`** — every job green, and no result contradicts a claim the
  product makes: macOS `RLIMIT_CPU` kills a spinner (the processor-time ceiling claim holds) and
  `RLIMIT_DATA` does not bite (withholding the memory-ceiling claim stays correct); Seatbelt denies
  network, reads and writes; the Windows one-process job and the restricted child pass; KVM boots a
  guest to `/init`; the subordinate-uid separation passes in its relaxed form.

This machine ran out of memory once today (Claude Code reaped a full suite and a local Miri run while
the session was idle); from here the heavy runs are GitHub's.

PS-D-02 waits for the owner's word ("stop before PS-D-02").

## 2026-09-28 — the handoff to the cloud (the laptop is away until 2026-10-16)

The owner needs the laptop for other work until 2026-10-16; the work continues from Claude Code cloud
sessions on the GitHub repository and is synced back afterwards. Before handing over, the head chef read
Claude Code's current documentation on cloud sessions, environments and memory (2026-09-28): a cloud
session runs in a fresh Ubuntu 24.04 VM (4 vCPUs, 16 GB) with Rust, Python, Docker and `gh`; it can push
**only to its own branch**, so its work reaches `master` through a pull request the owner merges; and
auto memory is machine-local, so a cloud session has none. What was written for that:

- **`AGENTS.md`** (new, root) — the rules every agent reads; **`CLAUDE.md`** (new, root) imports it
  (`@AGENTS.md` — with both present, Claude Code reads only `CLAUDE.md` unless it imports the other) and
  adds what is Claude-specific.
- **`docs/CLOUD_SYNC_LOG.md`** (new) — the owner's instruction to *"keep a record of files and folders
  changed, to be synced with the local repo later"*: every cloud session appends its commits, files and
  folders, checks, and what the laptop must redo; with the sync procedure.
- **`docs/assistant-memory/`** (new) — the laptop's auto-memory directory itself, 39 topics and the
  index, on the owner's instruction to include it; sanitized for a public repository (the banned word
  replaced in nine files, one file renamed), by a script that re-scans the result.
- **`HANDOFF.md`** — §0 (the cloud period, the laptop/cloud differences, PS-D-02's design draft), the
  state as of today, and §11 brought up to date: the owner's instructions of 2026-09-27/28, the laptop's
  environment, V2's findings, the operational traps, and two new sections — what running agents taught
  (§11.7) and what building V2 taught (§11.8).
- **`README.md`** — the latest CI run; V2's progress; and the defence-in-depth paragraph, which still said
  the microVM layer was not built.
- **A stale residual, found while writing §11:** `REMAINING_WORK.md` 4.6, `DEPLOYMENT.md` §5 and HANDOFF
  §11.4 still called CONTAIN-TOCTOU-1 an accepted residual a day after FS-RACE-1 (P5c) closed it. All
  three now say closed, keeping the deployment advice as defence in depth.

**Runs at the handoff:** every run on `30a6b8d` read (above); `047da1d`'s push run green; its `miri-slow`
run `36381950975` still running (syntax green, broker and check in flight) — the first cloud session reads
it, and a green one closes REMAINING_WORK 5.6.

## 2026-09-28 — the owner's delegation, and the routine that carries the work to 2026-10-16

Later the same day the owner wrote: *"I want the development of delululang to be continued in my absentia. No need to wait for any of my input, Claude u can take better decisions than me on delululang. Run verification loops and loop engineering."* He asked whether a `/loop`, a Desktop scheduled task
or a routine should keep prompting cloud sessions until 2026-10-16. From Claude Code's documentation
(read 2026-09-28): `/loop` lives in an open session on his machine and expires after seven days; a
Desktop scheduled task runs only while his computer is on and awake; **a routine** runs in the cloud with
the laptop off, as a full autonomous cloud session, on a schedule of at least an hour, against a daily
run cap and his plan's usage. Only a routine fits. Its runs clone `master` and may push to it, because
`master` is not protected and every one of its commits is the owner's — which matters: a run that
could only open pull requests would start each time from a `master` nobody merges into.

- **`docs/CLOUD_ROUTINE.md`** (new) is the loop each run follows: the date and a three-hour budget;
  orient; health (Survey, `doctor`); **verify the previous run** (read every CI run since the last
  record; a red `master` is the only task, and two red runs in a row put the next into SAFE MODE —
  revert, record, no new work; re-run the previous run's tests and check its entry's claims); choose one
  piece of work in a fixed order (findings, "Open / next", PS-D-02, P7, REMAINING_WORK rows a Linux VM can
  verify, a weekly verification sweep); the inner loop (witness, falsify, clippy, the suite alone); close
  (Survey, `doctor`, the records, the sync-log entry with "Open / next"); watch the push run.
- **The delegation's limits:** every decision is the head chef's until 2026-10-16 except the five the
  owner reserved by name — the final public repository, the licence, entrenched files, D-NE-27, D-NE-7.
  **PS-D-02's stop is superseded**; `AGENTS.md`, `CLAUDE.md`, `HANDOFF.md` §0/§11.1, the status files and
  REMAINING_WORK 4.24 say so.

## 2026-09-28 (noon) — the routine's mandate

Creating the routine, the owner widened the delegation: *"create it. I want opus 5.5 at xhigh effort. run verification loops. routine should be set for every 5hr till oct 16 … Can start PS-D-02. Tell to finish all the phases and verify. Set up loop engineering. any file or folder is allowed to modified or created or even deleted. If every phase is built continue improving and verifying delululang the lang of the future."* `docs/CLOUD_ROUTINE.md` now says
so: every phase finished and verified (PS-D-02, P7 with RW 5.2 now allowed, P8 as far as software
reaches), then continuous improvement; any file or folder may change — entrenched edits recorded in
`ENTRENCHED_CHANGE_RECORD.md`, deletions named in the sync log; four things stay the owner's (the final
public repository, the licence, D-NE-7, D-NE-27); a step 8 that improves the loop itself each run. xhigh
effort comes from `.claude/settings.json` (`effortLevel`), which cloud sessions read. The routine itself:
Opus 5.5, every five hours, until the step-0 date check ends the work on 2026-10-16.

**CI read at 07:10 UTC, before the routine was created.** The handoff commits' push runs: `d41e558`
`36384606872`, `bc9192c` `36384932715`, `7e67f97` `36387621134` and `7bd018c` `36387678371` green on every
job; **`937aea8` `36385550908` red on Windows only** — `actors_pingpong`'s criterion 1 measured a 1.31x
speed-up at 4 threads against its 1.5x bar (20.6 s against 15.8 s, after its built-in retry) on a commit
that changed only documents, and the two commits after it passed. That is the timing criterion measuring
a busy 4-CPU runner, which `HANDOFF.md` §1.1 predicted and which had held until now; it is the first run's
first task, investigated by starving a runner, never by loosening the bar blindly. **`miri-slow`
`36381950975` (on `047da1d`): `delulu-syntax` green, and `delulu-check` green — every one of its tests run
under Miri to the end for the first time (59 minutes); `delulu-broker` still running.**


## 2026-09-28 — the first cloud routine run: CI read, and the ping-pong criterion judged against a control (D-V2-47)

The first scheduled run of `docs/CLOUD_ROUTINE.md` (Opus 5.5, xhigh, started 07:11 UTC in the 4-vCPU
Ubuntu VM). **Health:** `survey check` ok (1,457 nodes, 12,477 edges); `doctor --check` exit 0, all checks
pass. **CI read:** every push run of the handoff commits was already recorded by the laptop at 07:10;
`5bb39bc`'s push run `36390274072` was still running at the start. `miri-slow` `36381950975` on `047da1d`:
`delulu-syntax` green (20 min), `delulu-check` green (58 min — every test to the end), `delulu-broker`
still running at 07:30 (it took 176 min last time).

**The inbox's first item — `937aea8`'s Windows red — investigated the project's way.** `actors_pingpong`
criterion 1 asks for 1.5x at 4 workers against 1; Windows measured 1.31x (best of two) on a commit that
changed only documents, and the commits either side passed. Reproduced in the VM by STARVING it, the
bar untouched: idle 1.89x and 2.18x; one CPU held by a busy loop 1.82x; two 1.54x; **three 1.20x, red** —
the Windows shape, from the same runtime binary. The test was measuring how many of the runner's four
CPUs were free, which its own `hw < 4` comment already says it must not.

**Fixed (D-V2-47)** by giving the test a control rather than a looser bar: before and after every attempt
it measures what parallel speed-up the machine gives four perfectly parallel CPU-bound units (each phase
best of three); an attempt counts only where that was at least 3.0x on both sides, and is held to the
criterion's share of it — 1.5 of 4, so exactly 1.5x on four free threads and never under 1.125x.
Evidence, all on this VM: idle — measured, 2.03x / 2.05x / 2.15x / 2.16x against controls of 3.1–4.1;
three CPUs starved — NOT MEASURED, printed with every attempt's numbers (controls 1.9–2.2, speed-ups
1.10–1.22x), where the old test was red; **falsified** — a runtime mutant that starts one worker whatever
it is asked for is red on the idle VM (1.00x against bars of 1.21–1.41x, three attempts). One control
reading of 3.05–3.18 on the idle VM is why each phase is best-of-three. `cargo clippy -p delulu-runtime
--all-targets -D warnings` clean. The residual: a passing log does not say whether a run measured or
skipped, so how often CI's runners count as busy is not yet known (D-V2-47 §5).

A Survey note for the next run: a test file's node is `test:<path>`, not `mod:<path>` — `impact
mod:crates/delulu-runtime/tests/actors_pingpong.rs` answers "no node".

### PS-D-02 — the attestation seam, built (D-V2-48)

From the `HANDOFF.md` §0 design. **`delulu run … --sandbox --sandbox-backend external:CMD
--require-attestation HEX`**: the host picks a fresh 32-byte nonce, gives the launcher
`DELULU_ATTEST_NONCE` and `DELULU_ATTEST_OUT` (a path in its own per-run directory), and before it sends
the program waits — up to 10 s, and no longer than the launcher lives — for a `delulu-attestation-v1`
document: a statement `{attester, guarantees, nonce}` signed (ed25519, the detached public-key-then-
signature form) over `delulu-attestation-v1\n` and the statement's canonical JSON. It checks the
signature, that the signer is the pinned key, and that the nonce is this run's — nothing else. Refused:
the guest is ended having been told nothing, the run exits 1 in words ("… Nothing ran: the program was
never sent to the guest"), `sandbox-attestation` (deny, with the reason) and `sandbox-death` go into the
audit chain. Verified: the stderr line and the report's `sandbox.attestation = {key, attester,
guarantees, verified: true}` carry the claims **as the attester's**, beside `host_guarantees` and never
merged; level 3, `fully_enforced` false. A dry run reports the requirement, `verified: false`. The flag
is refused (exit 2) at L1 and L2, which the host measures itself, and without `--sandbox`.
**`delulu sandbox attest --key SEED --attester NAME --guarantee TEXT.. -- COMMAND..`** is the reference,
software attester (and the tests' fake): it signs, writes the document whole (temporary file, rename),
removes the two variables and becomes COMMAND (`exec` on Unix). Building it found that `delulu` read
words after a bare `--` — `docker run -h HOST` there would have printed `delulu sandbox`'s help — so
the CLI now reads nothing after `--` (`cli::own_words`).

**Tests.** `crates/delulu/src/attest.rs`, six: a pinned, fresh statement verifies and file whitespace
and key order do not matter; the canonical form is the documented one (the same bytes Python's
`json.dumps(sort_keys=True, separators=(",", ":"), ensure_ascii=False)` writes — checked); every wrong
document is refused and named (another run's nonce, a key nobody pinned, a claim added after signing,
the pinned key's bytes spliced before a stranger's signature, another format, an unknown field in the
statement or the document, not JSON, not UTF-8, over 64 KiB, a signature that is not hex); a statement
that could fill a terminal or a report (no claim, a blank name, an escape sequence, 257 characters, 33
claims) is refused by the attester AND, signed anyway, by the host; keys and nonces; the wait ends at
once when the launcher ends and at the deadline when it stays silent. `tests/sandbox_attest_cli.rs`,
five, with the binary as attester, launcher and guest: attested and served, the report checked field by
field and validated against `delulu schema sandbox`, the seed's path never recorded; refused before the
program is sent — another key, a launcher that never attests (the 10 s deadline), one that exits, an
attester that refuses to sign — each with the program's file NOT written, then the same launcher served
when pinned right; a document replayed from another run refused on its nonce (Unix: a script hands the
host a valid old document); the flag's refusals and the dry run; the attester's own refusals and the
`--` rule (`-- delulu check -h` prints `check`'s help, not `sandbox`'s). **Falsified, six mutants, each
red on its own assertion:** no nonce check (the replay test and the unit test), any signer accepted
(case 0: the program ran), served despite a refusal (case 0), `--help` read past `--`, the dry run
dropping the requirement, the report dropping the attestation. `cargo clippy --workspace --all-targets
-D warnings` clean.

**The full suite, alone in the VM** (`cargo test --workspace --no-fail-fast -j 4`, cargo's own exit read):
151 test binaries, **1,995 passed, 1 failed, 15 ignored** — the one failure environmental:
`egress_features::no_refused_reqwest_feature_is_enabled_anywhere_in_the_resolved_graph` runs `cargo
metadata --offline --locked`, which needs every platform's crates, and a fresh VM has fetched only
Linux's (`failed to download core-foundation … --offline was specified`). After `cargo fetch --locked` it
passes (2/2), so 1,996 of 1,996. Written into `docs/CLOUD_ROUTINE.md` step 2 for the next run.

**CI read during this run.** `5bb39bc`'s push run `36390274072`: **red on Windows only, the same
`actors_pingpong` criterion** — 1.31x, retry 1.28x (19.7 s against 15.0 s), extracted from the job's log
by a Haiku 4.5 sous-chef and checked against the job id. Two Windows runners, 1.31x and 1.31x/1.28x with
near-identical times: systematic on some Windows runners rather than a moment's noise — fewer real cores
behind the four vCPUs, or a busy image — which is what D-V2-47's control now tells apart. `9ac5477`
(D-V2-47) is the first push with the control; its run is this run's to read.

**Loop engineering (`docs/CLOUD_ROUTINE.md` step 8).** What this run lost time to, fixed where it lives:
the routine told a run to read CI with `gh`, which the cloud VM does not have (and the proxy refuses the
signed log URLs) — step 3, step 4's weekly sweep and step 7 now name the GitHub MCP tools, and CLAUDE.md says
`gh` is the laptop's; a fresh VM needs `cargo fetch --locked` before the suite (step 2); a test file's
Survey node is `test:<path>` (step 5, AGENTS.md); a red that a newer commit already fixed is read, not
re-fixed (step 3). And D-V2-47's own residual: `actors_pingpong` now writes its verdict — MEASURED with
its control's figures, or NOT MEASURED and why — to `target/tmp/actors_pingpong-criterion1.txt`, and
CI's test and arm64 jobs print it after the suite, pass or fail (idle VM: "MEASURED: 2.18x where the
machine gave perfectly parallel work 3.24x, against a bar of 1.21x — passed"). `9ac5477`'s push run
`36393016213` — the first with the control — was green on every job.

### P7 — RW 5.2 and 5.3: Delulu Core v0.3 (D-V2-49)

The routine names RW 5.2 as P7's; the owner's delegation allows the entrenched edit, recorded in
`ENTRENCHED_CHANGE_RECORD.md` and flagged for his review. **Witnessed first, on the current binary**,
what the calculus must describe: a program granted `fs.read=./data` reading `../outside.txt` checks
clean, runs, and faults `DL0904` (exit 1; its own `Err` arm never runs), and `--trace-effects` records
the attempted `Read` before the refusal; and a pure-declared function passing a `! {Write}` callback to
`List.map` is refused `DL0501`, while declaring `{Write}` checks clean. **`DELULU_CORE.md` v0.3:** a
configuration may be `fault(c)`; `E-Refuse` takes a present capability whose scope does not cover its
arguments there, emitting the attempted label as the trace does; `E-Fault` propagates it — the calculus
has no handlers; Theorem 1 is **progress-or-fault**; Preservation and Theorem 3 hold over faults, and
Theorem 3 for every finite prefix of a run. And RW 5.3 with it: `hop(e, ē)`, `T-HOp` joining the
callback's latent row (R-4 — the Lean fragment's `TypedGood.ho`, generalized from a literal closure),
`E-HOp` unfolding to applications. §8 gains three traceability rows; §9's two finding boxes each say
"repaired in v0.3" and keep their text. Lean was not re-run (no toolchain in the VM; the `.lean` file
is unchanged). RW 5.2 and 5.3 closed; RW 5.1 — the mechanization — is open and now aimed at a calculus
that states the construct that failed and the outcome the runtime has. `PROOF_CAMPAIGN.md` and
`MATHEMATICS.md` note the repair where they state the findings. P7 now waits only on RW 5.6's green
`miri-slow` run.

### ADAPTER-SPELL-1 — a hardware driver was verified as one file and started as another (D-V2-50)

Found while sizing P8 (RW 4.7, the signed adapter), by the question §11.4 says to ask of every string
a security decision is made on — *what else spells this?* `run_cmd.rs` checked the driver's provenance
with `check_adapter_signature(prog, …)`, which reads `--adapter-cmd`'s first word as a path from the
working directory, and then started it with `ProcessAdapter::spawn(…, prog, …)`, i.e.
`Command::new(prog)`, which looks a bare word up on `PATH`. **Witnessed** (Unix, a new end-to-end test):
the operator's driver signed by the operator's key in the working directory, a different unsigned
`drive` first on `PATH` that leaves a marker; `--adapter-cmd drive --adapter-signer KEY` printed
"`drive` signature verifies under the pinned key …" — and the marker appeared: **a pinned key vouched
for a file that never ran, and a driver nobody signed commanded the machine.** Fixed (D-V2-50): the
word is resolved once and that absolute file is verified and started; a word that resolves to nothing
is refused rather than handed to the OS's own search. The test now passes and the old code fails it
(the witness). Two assertions of an older test encoded the old reading of `sh driver.sh` ("first word
is not a file — nothing to verify", true only because `sh` was looked for in the working directory):
`sh` now resolves to the interpreter started, which is unsigned — still refused under the flag, and the
refusal now names `--adapter-artifact`; the assertion checks that instead. All 12 adapter tests pass.
**Stale claims found beside it:** `REMAINING_WORK.md` 4.7, `HANDOFF.md` §11.4 and
`V2_SECURITY_MODEL.md` said the adapter had "no signature check" — untrue since D52/D53; corrected, with
what is still missing (the Verified-class `.dpx`, P8). The residual — replacement between check and
start by someone who can write the driver's directory — is named in D-V2-50, `QUESTIONS.md` §3.2 and
RW 4.7.

**CI read, and PS-D closed.** `fef8ccd` (PS-D-02) push run `36395256154`: **success on every job** —
the attestation tests on Windows, Linux and macOS included. **PS-D is complete** (`V2_PHASE_STATUS.md`
row 13); RW 4.24 closed.

The full suite after ADAPTER-SPELL-1, alone in the VM (`-j 4`, cargo's exit 0): 151 binaries, **1,997
passed, 0 failed, 15 ignored**; clippy `--workspace --all-targets -D warnings` clean. Inside the suite
`actors_pingpong` wrote "MEASURED: 2.12x where the machine gave perfectly parallel work 3.54x, against
a bar of 1.33x — passed".

### ATTEST-FIFO-1 — the attestation seam could be made to hang its host (found and fixed the same day)

An adversarial reading of PS-D-02's own code, the question from P5b's FIFO finding: *what if the
thing at the path is not a file?* The host waits for the document with `metadata` and then
`File::open` — and it does so before any watchdog runs, at a path the (possibly hostile) launcher
controls. **Witnessed** (Unix): a launcher that runs `mkfifo "$DELULU_ATTEST_OUT"` and then the guest
held the host in a blocking `open` until the test killed it at 30 s; left alone it would never end. A
link there would have had the host read whatever the link names (it would then have been refused as
not a document, but the host should not be opening it at all). **Fixed:** a regular file only —
`symlink_metadata` first, then `O_NOFOLLOW | O_NONBLOCK` on Unix (a link swapped in afterwards is
`ELOOP`, refused the same way), then the open handle's own metadata; refusal `not-a-file`, "… is not a
regular file (a link, a pipe or a device)". The new test
(`a_launcher_cannot_make_the_host_open_a_pipe_or_follow_a_link`) hung on the old code and passes now,
promptly, for both a pipe and a link; the program never runs in either. D-V2-48 §8.

### P7 closes; ATTEST-FIFO-1's suite; `ed74683`'s verdicts

- **`miri-slow` `36381950975` (on `047da1d`): success on all three crates** — `delulu-syntax` 20 min,
  `delulu-check` 58 min, `delulu-broker` 172 min (05:27 → 08:19), 0 UB, every test run to the end
  within the 240-minute budget. **RW 5.6 closed.** With RW 5.2/5.3 (`b7abcfb`, push run `36396386742`
  green on every job), **P7 is complete** (`V2_PHASE_STATUS.md` row 12).
- **`ed74683`'s push run `36395623730`: green on every job**, the new verdict step included on all four
  test jobs. The arm64 runner's verdict, extracted from its log by a Haiku 4.5 sous-chef: "MEASURED:
  2.80x where the machine gave perfectly parallel work 4.11x, against a bar of 1.50x — passed" — the
  first time the criterion's full 1.5x bar is known to have been asserted, and met, on a CI runner.
- The full suite with ATTEST-FIFO-1, alone in the VM: **1,998 passed, 0 failed**, 15 ignored (cargo exit
  0); ping-pong inside it "MEASURED: 2.07x … control 3.26x … bar 1.22x — passed".
- **The verdict step moved to the end of each test job** (and became a `notice` annotation as well). Asked
  for Linux, macOS and Windows, the sous-chef found no verdict line, and — asked for its evidence rather
  than taken at its word — showed why: the MCP log tool returns a job's last 5,000 lines, and in those
  three jobs the CLI sweep and the fuzz campaign after the verdict step print more than that. The arm64
  job has nothing after it, which is why its line was found. `docs/CLOUD_ROUTINE.md` step 3 records the
  cap.

### P8 — designed (D-V2-51), not built

`V2_P8_DESIGN.md`: what exists (the device broker, the dead-man and the e-stop are already host-side;
the driver is a signature-checked subprocess; a device program is refused under `--sandbox` — witnessed
on the current binary, "`--sandbox` cannot carry this program yet: it uses Actuator", exit 2) and three
slices, each one routine run: **P8-01** the control program in a guest — `Actuator`/`Sensor` over the
existing channel, performed by the host's `DeviceBroker`, with five witnesses against the simulator
(same values sandboxed or not; a wedged guest loses its actuator on the host's wall clock; a revoke
mid-motion; DL1905 still binding; two falsifications); **P8-02** the driver's logic as a Verified-class
`.dpx` — pure, proved at load, signature-checked against a pinned key and run from the verified bytes,
the host keeping the transport — which ends D-V2-50's residual; **P8-03** a reference transport to the
simulator in its own process. A real device stays environment-blocked.

**CI read (continued).** `81c644f` (ADAPTER-SPELL-1) push run `36397905009`: **success on every job** —
`resolve_driver`'s `PATH` lookup (`.exe` appended) found the Windows rig's `python` and the Unix rigs'
`sh`; the new Unix witness passed on Linux and macOS.

### RW 7.4 — the editor's real end-to-end test, on Linux for the first time

`editors/vscode/e2e.js` launches a real VS Code against the real server; it had only ever run on
Windows, and reading it showed why: its "is the server alive?" check and its cleanup were PowerShell's
`Win32_Process`, so on Linux it could not pass. Given a POSIX branch (`pgrep -af` for the `delulu …
lsp` process, `pkill -f` for cleanup; `--no-sandbox` when running as root, which Electron requires),
it ran in the cloud VM — VS Code 1.139.1 from Microsoft's apt repository (`packages.microsoft.com` is
reachable; the direct download host is not), Node 22, `npm ci && npm run package`, `xvfb-run -a node
e2e.js target/debug/delulu`: **"OK — extension activated and the language server started with no errors
reported"**, with `initialize` sent and answered and `publishDiagnostics` received in the traced log.
**Falsified:** against a "server" that exits at once, exit 1 and "server crashed 5 times". RW 7.4 closed;
making it a CI gate on the Linux editor job is its "next".

### RW 7.3 — why this VM cannot build the Dockerfile

Its Docker daemon starts (`dockerd`, overlayfs, cgroup v1), but every pull from Docker Hub is refused:
"You have reached your unauthenticated pull rate limit" — the VM's outbound address is shared. So the
build moved to GitHub: `.github/workflows/container.yml`, by hand only, dispatched as run `36399908461`.

**RW 7.3, the first container build — run `36399908461`.** The **devcontainer passed** (`devcontainer
up` with its release build in 3 min 34 s; `cargo 1.96.1` and `delulu 1.0.0` answered inside). The
**Dockerfile failed** at `cargo build`: 10 errors, `couldn't read …/skills/delulu/SKILL.md` and nine
`…/examples/…` — `delulu` embeds them at compile time and the Dockerfile copied only `crates/`. The
prediction recorded in `17d4623`'s message (a loader failure for the missing libpython) was **wrong**
about the first failure — kept as written. Fixed: `skills/` and `examples/` copied; the portable build
the release ships (`--no-default-features --features net`, which the Dockerfile's own comment already
claimed); `ca-certificates` in the runtime image. Re-dispatched after the push.
