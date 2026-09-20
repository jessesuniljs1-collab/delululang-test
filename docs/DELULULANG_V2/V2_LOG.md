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
PS-B-01) is still open and still the owner's.

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

PS-B-01 (budgets on every engine; **defaults are D-NE-31, the owner's**), PS-B-03 (identity separation —
AppContainer per run on Windows, uid mapping where namespaces allow, a documented macOS recipe, reported
in `host_guarantees`; today a guest runs as the same OS user and `identity_separation` is always in
`limitations`), PS-B-04 (channel batching, gated on PS-A's measurement saying it pays), PS-B-05
(resource authority in the derived policy, the envelope, the authority report, the audit record; the
dimensions admitting a containment order join the Z3 model and `MATHEMATICS.md`), PS-B-06 (BREAK-GLASS
as an operator-held credential outside the guest, never activatable by program code).

PS-B-01 and PS-B-05 are the two that need no ruling to start and touch code PS-B-02 does not.

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
