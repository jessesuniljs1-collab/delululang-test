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
