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

**What CI's runners are — the verdicts of `0a7b881`'s push run `36399295189`** (green on every job;
the lines extracted by a Haiku 4.5 sous-chef, in the test's own words and with their `##[notice]`
annotations): **arm64 — MEASURED 2.93x on a 4.11x control, bar 1.50x, passed**; macOS — NOT MEASURED,
three hardware threads; **Linux x86-64 — NOT MEASURED, controls 2.28–2.31x, runtime 2.09–2.16x;
Windows — NOT MEASURED, controls 1.99–2.12x, runtime 1.74–1.81x.** Nothing else ran on those runners:
their four vCPUs give about two cores' worth of parallel throughput (SMT). That is the real account of
`937aea8`'s and `5bb39bc`'s Windows reds — that hardware, not a busy moment, and not a regression.
D-V2-47 §6 records it; the verdict now says "busy, or its hardware threads share fewer cores (SMT)".
The criterion is asserted on every push on arm64.

**RW 7.3 closed — container run `36400717604`: both jobs green**, read in the log, not only the
conclusion: the image built with the fixed Dockerfile answered `delulu 1.0.0`, ran a program granted
the console ("hello from the image") and refused the same program without the grant — `error[DL0703]:
console was not granted to this program`; the devcontainer built again (`devcontainer up`, then the
toolchain answered inside).

## 2026-09-28 (evening) — the laptop synced; NVIDIA OpenShell studied; PS-E, P8-04 and P9 designed

**The routine, read before anything was touched.** Run 1 (`cse_013AZJ6RqYq59CeMgkvLV1BM`, 07:11–09:16
UTC) pushed the 13 commits recorded above. The next three fires (10:11, 10:12, 10:44 UTC) each ended
within seconds on the account's five-hour usage limit (`rate_limit: rejected (five_hour)`, read in each
run's log), having done nothing. The owner paused the routine at 14:30 UTC and asked, in order: sync the
laptop with what the cloud pushed; study NVIDIA OpenShell, launched that day with the Open Agent Safety
Platform, *"not just as copy but as real engineering for the sandbox"*; update every document it
changes; resume the routine.

**The sync.** The laptop was clean at the baseline `5bb39bc`; `git pull --ff-only` fast-forwarded it to
`dd543e5` (13 commits, all run 1's). `git ls-files --eol`: no CRLF working-tree file. On Windows:
`survey check` — ok, 1,462 nodes, 12,602 edges; `survey findings` — 0 errors, 14 warnings, 38 notes (the
same as the cloud's); `doctor --check` — 29 checks passed. The full suite was not run on the laptop:
CI ran every OS on each of those commits and run 1 read each green; the sync procedure's full-suite
and editor steps stay for the owner's return (`docs/CLOUD_SYNC_LOG.md`).

**The study.** Read in full: OpenShell's 57 documentation pages as Markdown (v0.1.2 and dev — the same
set), its seven architecture diagrams from their SVG sources, label by label, and its repository at
`36b0386` — `architecture/`, RFCs 0001/0002/0005/0012, and the source of the typestate isolation
contract, the binary-identity resolver and the Z3 prover. Then each of its controls was compared line
by line with DeluluLang's code. The record is `V2_OPENSHELL_STUDY.md`; the decisions D-V2-52 (the terms
and the order), D-V2-53 (PS-E), D-V2-54 (P8-04) and D-V2-55 (P9).

**The finding that shaped it:** OpenShell governs binaries it cannot read, so it must intercept their
sockets and decide each request; a DeluluLang guest performs no effects, so it can be *denied* what
OpenShell must *allow*. The comparison produced, from the code and not from the other system's claims:

- the boundary is confirmed by convention in `guest.rs`, not by construction, and an absent layer
  never refuses a run, even under `hostile-agent` (RW 4.25 → PS-E-01);
- a macOS guest computing without asking outlives its host until its CPU limit, and an external
  launcher is started with no death signal (RW 4.26 → PS-E-02);
- **six hypotheses about the Linux guest after lock-down** — syscalls the filter does not name, sockets
  other than TCP (Landlock mediates TCP only), all of `/proc` readable, a dumpable host, the Landlock
  ABI requirement, and the other two systems' equivalents (RW 4.27 → PS-E-03). **None is a finding
  yet**: each needs an escaped-guest witness red on the current binary;
- the external launcher found on `PATH` at spawn and never hashed — ADAPTER-SPELL-1's shape (RW 4.28 →
  PS-E-04);
- no program-against-boundary check, no risk report on a change of grants, no way for a refusal to
  become a request to the operator (RW 4.29 → P9-01…03); no way to send a secret to an endpoint (RW
  2.11 → P9-04); host-only network scopes (RW 2.12 → P9-05); no OCSF export (RW 6.14 → PS-E-06);
  OpenShell neither a tested backend nor a policy target (RW 4.30 → PS-E-05).

**Not taken, and why** (study §5): deny rules, hot-widening, binary identity for the guest, `audit` as a
default, an HTTP endpoint inside the guest, content-inspecting middleware, a fleet control plane, L7
protocols beyond REST for now. **Terms:** nothing of OpenShell copied, none of its crates a dependency
(Apache-2.0; `NOTICE` is the owner's).

**Order:** PS-E → P8 (P8-01 on PS-E-01's confirmation; P8-04 added) → P9 — in `V2_MASTER_PLAN.md`,
`V2_IMPLEMENTATION_ROADMAP.md`, `V2_PHASE_STATUS.md` and `docs/CLOUD_ROUTINE.md` step 4. Documents
updated with it: `V2_SECURITY_MODEL.md` (§6's L3 note, a new §9b, §10's stale "PS-D-02 is not built"
corrected), `V2_P8_DESIGN.md`, `V2_README.md`, `REMAINING_WORK.md`, `HANDOFF.md`, `AGENTS.md`, `README.md`,
`DEPLOYMENT.md`, `REPOSITORY_STRUCTURE.md`. Nothing was built; no test changed.

## 2026-09-28 (evening, later) — the routine's brief checked against the cloud docs; Sonnet 5.5

The owner asked whether the routine's instructions need updating for Claude Code's cloud, and said
Sonnet 5.5 and Haiku 5.5 had just launched. Read, as Markdown: the official models overview, Claude Code's
model configuration, routines and cloud-environments pages. What held, and what changed:

- **Models.** Sonnet 5.5 (`claude-sonnet-5-5`) is released — the models page lists it, retirement "not
  sooner than September 28, 2027"; it needs Claude Code ≥ 2.1.284 (the laptop has 2.1.284). **Haiku 5.5
  is not released**: the page's newest Haiku is 4.5, and the announcements say "in the coming weeks".
  Testing passes now use Sonnet 5.5 and Haiku 4.5, Haiku 5.5 once it is listed (`AGENTS.md`,
  `docs/CLOUD_ROUTINE.md`). The routine's head chef stays Opus 5.5 at xhigh — the owner's choice.
- **Effort.** Claude Code runs Opus 5.5 and Sonnet 5.5 at `medium` by default; the repository's
  `effortLevel: xhigh` is what gives the routine xhigh. Unchanged, now said.
- **Classifier fallback.** A cybersecurity-flagged request on Opus 5.5 re-runs on Opus 4.8 and the
  session stays there. PS-E-03's escaped-guest work is defensive but could be flagged; the commit trailer
  must then name Opus 4.8 (`CLAUDE.md`, the brief).
- **`gh`.** The docs list it as pre-installed and proxy-authenticated; run 1 found none. The brief now
  says: check (`command -v gh`), record, use it when there; read CI with the GitHub MCP tools either way.
- **The VM's reach.** Release assets and API calls reach only the attached repository — OpenShell's
  releases cannot be fetched in the VM, so PS-E-05 exercises OpenShell only on GitHub's runners.
  Commands wait at most 10 minutes, then move to the background. A cached environment setup script is
  the owner's to add (the environment dialog), if he wants the toolchain pre-installed.
- **Usage.** Recorded in the brief: routine runs draw on the owner's subscription, and three fires on
  2026-09-28 ended in seconds on the five-hour limit.
- **The owner's ruling, minutes later:** *"run Opus 5.5 at xhigh effort. if needed use sonnet 5.5 (latest) as agents and haiku 5.5 will be launched in coming weeks, use haiku 5.5 as agent after launching"* Agents are Sonnet 5.5; Haiku 5.5 once it is
  released; no Haiku 4.5 meanwhile. `AGENTS.md`, the brief, `HANDOFF.md` §11.1/§11.3 and the memory say so.
- **CI read:** `fed54cb` (Cloud handoff 7) push run `36463058082` — success on every job; arm64's
  ping-pong MEASURED 2.88x against a 4.12x control, bar 1.50x, passed. Two runner notices to act on
  before they bite: `actions/checkout@v4` and `actions/cache@v4` target Node.js 20, which GitHub now forces
  onto Node 24; and `ubuntu-latest` migrates to Ubuntu 26 from 2026-10-19.


## 2026-09-28 (night) — routine run 2: CI read; the workflows off Node 20 and pinned to Ubuntu 24.04

**Health first.** `survey check`: ok (1,464 nodes, 12,653 edges); `doctor --check`: ok, all checks pass
(26 in this VM; `identity separation none` here — the VM has no `uidmap`). `command -v gh`: none again —
CI read with the GitHub MCP tools. `cargo fetch --locked`: done before any suite.

**CI read (step 3).** Every push run since the last recorded one, each by id:
- `045c21a` (Cloud handoff 8) — `36463375580`: success. `7e9d97d` (Cloud handoff 9) — `36464748581`: success.
- `cfbfbdc` (Cloud handoff 10) — `36465304879`: success on every job (16: three test OSes, arm64, lints,
  supply-chain, formal, editor, fuzz, miri ×2, miri-ffi, microvm, microvm-reproducible; heavy-gates and
  miri-slow skipped by design). The ping-pong verdicts, each job's last step: **arm64 MEASURED 2.91x
  against a 4.13x control, bar 1.50x — passed**; Linux x86-64 NOT MEASURED (controls 2.03–2.05x, SMT);
  Windows NOT MEASURED (controls 1.51–1.67x); macOS NOT MEASURED (three hardware threads; 2.19x, control
  3.24x/2.78x).
- Also completed since: the nightly `CI` `36402530469` on `dd543e5` (schedule) — success; `release`
  `36413736270` (schedule, a dry run) — success; `editor-e2e` `36401854421` (by hand) — success.
  Routine run 1's own last pushes (`e430c2a` … `dd543e5`) were green in the listing, as the laptop's
  evening entry found.

**The inbox's CI hygiene (sync log, "Open / next" 1b), done.** Every job's log ends with GitHub's
warning that `actions/checkout@v4`, `actions/setup-python@v5` (and, in other jobs, `actions/cache@v4`)
"target Node.js 20 but are being forced to run on Node.js 24". Each replacement major was **measured,
not remembered**: the `runs.using` line of its `action.yml` at that tag, fetched from
`raw.githubusercontent.com` — `checkout@v5` node24, `cache@v5` node24 (`v4` node20), `setup-node@v5`
node24, `setup-python@v6` node24, `setup-java@v5` node24, `upload-artifact@v6` node24 (`v5` still node20),
`download-artifact@v7` node24 (`v6` still node20), `attest-build-provenance@v3` a composite over
`actions/attest` v3.0.0, node24 (`v2` wraps v2.4.0, node20). The inputs this repository passes
(`subject-path`, `path`, `merge-multiple`) exist at the new tags; `editors/vscode/package.json` has no
`packageManager`, so `setup-node@v5`'s automatic caching does not switch on. `Swatinem/rust-cache@v2`
is already node24; `cargo-deny-action` is a Docker action. **Every `ubuntu-latest` is pinned to
`ubuntu-24.04`** (what `ubuntu-latest` is today) in all six workflows: `ubuntu-latest` becomes Ubuntu 26
on 2026-10-19, and the microVM job's kernel build, the user-namespace step and the red-team gates were
all measured on 24.04 — a runner change should be a deliberate, measured step (a manual run on
`ubuntu-26.04` first), never a Monday surprise. The witness is the next push run: the Node-20 warning
gone from every job, and every job green.

## 2026-09-28 (night) — PS-E-01, first step: the boundary is confirmed before the program is sent (D-V2-56)

**The gap, witnessed.** `V2_OPENSHELL_STUDY.md` §4.1 said the order in `guest.rs` was right "by
convention". It was not right at all for the guest's own layers: the host's FIRST frame on
`delulu-sandbox-channel/2` was the `Hello` — the program itself — and the guest locked itself down and
reported what it applied (RW 4.23's `Confined`) only afterwards. So a guest that never confined itself
had already been handed the program. Witness, `tests/sandbox_confirm_cli.rs`, **red on `ff701ae`**: an
external launcher (L3) that records every byte the host sends it for two seconds and never answers
captured the whole program, canary included (`…println("CANARY-PS-E-01-14920")…`), exit 1. The second
witness, red on the same binary: no `sandbox.generation` in a run report — only an attested L3 run had a
nonce.

**Built — `delulu-sandbox-channel/3`, and the order as a type.**
- The host's first frame is `Open { version, generation }` — no program. The guest reads it, checks the
  version, locks itself down (Landlock, seccomp: unchanged), and reports `Confined { applied, generation }`.
  `HostChannel` accepts the report only for the generation it was opened with (`with_generation`), and a
  host that opened none accepts no report at all. Only then does the host send `Program { program, hash,
  seed, fixed_clock_ms }`; the guest checks the hash and the program after its lock-down (the checker
  needs nothing the lock-down removes — the whole suite passes on Linux L1).
- `crates/delulu/src/boundary.rs` (new): `open` → `Opened::confirm` → `Confirmed::send_program`, each
  consuming the one before, fields private to the module — `confirm` is the only constructor of
  `Confirmed`, and `send_program` the only way the program frame is written. A first request that is not
  the confinement report is **not answered** (an answer to `root.console()` would be an effect performed
  for a guest nobody confirmed). `guest.rs`'s run path and the `sandbox probe`'s contained-guest path both
  go through it.
- **A generation for every run** at every level: 32 bytes of the OS's randomness, in the `sandbox-launch`
  and `sandbox-death` audit records and the report (`sandbox.generation`, required by the closed
  `sandbox_run` schema). An attested run's nonce IS its generation (PS-D-02 unchanged otherwise).
- A guest that does not confirm ends the run with exit 1 in words ("the guest never confirmed its
  boundary, so it was not sent the program: …"), its report says `outcome.ran: false`, and its death
  record `confirmed: false`.
- An older host's first frame decodes (the generation is defaulted) so a version-skewed guest refuses on
  the VERSION, in words — the case an operator's stale launcher image meets.

**Witnesses and falsifiers.** `sandbox_confirm_cli.rs` (2 tests, red → green): the non-confirming
launcher receives no program byte and the run says `ran: false`, `confirmed: false`; two runs have
distinct 64-hex generations, each named by exactly one launch and one death record, the chain verifying.
Unit tests: `boundary.rs` (an honest guest is confirmed and only then sent the program; four guests that
do not confirm — hangs up, another run's generation, a word nobody applies, asks for the console first —
are never sent it, and the fourth is never answered), `channel.rs` (a report only for the host's
generation; the fuzz corpus now carries generations and decodes both host frames). **Mutants, each seen
to land in the source and each killed:** M1 the program written before `confirm` → the e2e witness red
(the canary captured); M2 any generation accepted → the channel test and the boundary test red; M3 a
first request that is not the report answered → the boundary test red with "asks for the console first:
confirmed" (its root DOES grant the console, so the mutant performed the mint).

**The suite, alone, `-j 4`:** 2,002 passed, **1 failed**, 15 ignored, 152 binaries — the failure a real
gate doing its job: `every_thread_either_sizes_its_stack_or_is_listed_as_never_running_a_program`
(`actors.rs`) cuts each file at its first `#[cfg(test)]`, and `boundary.rs`'s test module was
`#[cfg(all(test, unix))]`, so its scripted-guest thread read as shipped code with the OS default stack.
Now `#[cfg(test)]` + `#[cfg(unix)]`; the gate and the module's tests re-run green. Clippy `--workspace
--all-targets -D warnings` clean. Every sandbox, guest and schema test binary green before the suite
(guest_cli 5, sandbox_attest 6, sandbox 3, sandbox_confirm 2, sandbox_external 3, sandbox_guard_e2e 1,
sandbox_modes 11, sandbox_run 17, schema 4; the ten microVM tests are KVM-gated — CI's `microvm` job,
whose guest image is built from this tree and so speaks `/3`).

**CI read:** `ff701ae` (the workflows' Node-24 move) push run `36477748775` — success on every job; the
Node-20 warning is gone from the logs read (arm64: `checkout@v5`; formal: `checkout@v5`, `setup-python@v6`,
`setup-java@v5`); arm64's ping-pong MEASURED 2.88x against a 4.12x control, bar 1.50x, passed.

**Not in this step (PS-E-01's next):** the five properties each `established { mechanism, evidence }`,
`absent { why }` or `unknown`; the profiles' required sets (`hostile-agent` all five; `contained`
filesystem, egress, resource; `dev` none); refusal before the program is sent when a required property
is missing, and `host_guarantees` derived from the confirmation. The typestate now has the one place
where that check goes: `Opened::confirm`.

## 2026-09-28 (night) — PS-E-01, second step: every run reports its boundary's five properties (D-V2-57)

**Why report before requiring.** D-V2-53 §2 gave the profiles required sets — `contained` (the default)
requires filesystem, egress and resource. Read against `jail.rs` before building the refusal, that set
would refuse **every macOS run**: Seatbelt leaves reads open (so filesystem confinement is not
established) and macOS refuses `RLIMIT_DATA` (so no memory ceiling is claimed). A required set that a
supported platform can never meet is a decision to be taken on measurements, not on a reading. So this
step REPORTS the properties, on every operating system CI runs, and refuses nothing; the required sets and
the refusal come once each OS's answers are read (the next step).

**Built.** `boundary::properties(guarantees, measured_by_host)` → `sandbox.properties` in every run
report (the closed schema's `properties`/`property`): `filesystem_confinement`, `egress_confinement`,
`privilege_floor`, `host_loss_ends_guest`, `resource_ceiling`, each `{state: established, by}`,
`{state: absent, why}` or `{state: unknown, why}`. **One source:** each is answered from the posture the
same report carries (`SandboxPolicy::posture` over the applied words) — filesystem = writes and reads
held; egress = the network row; privilege floor = the privilege row, or a separate identity (an
AppContainer, a subordinate uid, the jailed VMM); host loss = "killed with the host"; resource = memory
and processor time held. An external launcher's (L3) are all `unknown`: its wall was measured by nobody,
and what its guest says of itself is the word of a binary the launcher chose.

**What each platform answers (unit test over each launch's real words):** Linux L1 with Landlock — all
five established (this VM's run reports exactly that); Linux without Landlock — filesystem and egress
absent; **macOS L1 — filesystem absent (reads open), host loss absent (PS-E-02), resource absent (no
memory ceiling)**; Windows L1 — all five, the AppContainer as the privilege floor; the microVM — all
five; L3 — unknown. CI's three OSes will say whether the runners agree: **each test job (three OSes and
arm64) now prints one L1 run's properties** as a step and a `notice` annotation, just before the
ping-pong verdict (which stays last) — the step run locally exactly as YAML parses it printed all five
established for this VM.

**Witnesses and falsifiers.** `sandbox_confirm_cli.rs` gained two tests, red on `062a78c` (no
`properties` in the report): a run's five properties agree with its own posture on whatever OS runs it,
and each says by what or why not; an external launcher's are all `unknown`. Mutants, each landed and
killed: M4 an L3 run's properties established (the e2e test and the unit test red); M5 egress always
established (the unit test's no-Landlock case red); M6 the identity not counted as a privilege floor (the
unit test's Windows case red).

**Verified:** clippy `--workspace --all-targets -D warnings` clean; the full suite alone: 2006 passed, 0 failed, 15 ignored, 152 binaries — cargo exit 0.

## 2026-09-28 (night) — the red-team pass on `delulu-sandbox-channel/3`: seven defects around a guarantee that held (D-V2-58)

**The pass.** One sous-chef (Sonnet 5.5, the `sonnet` alias — `AGENTS.md`), briefed to break PS-E-01's
guarantee and to report evidence only, worked against a frozen copy of `6ceaf2d`'s binary in a scratch
directory outside the repository (it wrote nothing into it; it ran one read-only `git status`, against the
brief, and said so). About 85 scripted attempts — wrong, empty, replayed and malformed generations, words
outside the allowlist, requests before the report, reports twice, junk and oversized and deeply nested
frames, a version-skewed guest, attestation with a lying guest — and 700 mutated first frames: **the
guarantee held every time**: no program byte and no effect reached a guest before an accepted report
(baselines — the honest launcher, an honest Python guest, a plain L1 run — all succeeded, so each refusal
was evidence). Around it, it reported fifteen items. **Each was re-run by the head chef before a word of
it was used** (`AGENTS.md`: judge an agent by its logs); the witnesses below are the head chef's own,
each red on `6ceaf2d`.

**Verified and fixed (each witness red on `6ceaf2d`, green now; mutants landed and killed):**
- **GUEST-WAIT-1 (F4)** — a guest whose channel failed was WAITED for, not ended: `serve_under` called
  `wait()` with no kill, so a launcher that closed its channel and lingered held the host for its whole
  life (witness: a launcher that closes its output and sleeps 40 s — the host took 40 s). Now `end_guest`:
  a failed conversation ends the guest (after 250 ms, so one already dying keeps its own exit status), and
  one that said goodbye gets 10 s to exit (a launcher removing its container) before it is ended. A guest
  the host ended is not named as a ceiling; the death record says `ended_by_host`. Mutant M7 (never kill)
  red.
- **RAN-SENT-1 (F1)** — `outcome.ran` said "confirmed", not "sent": a guest that confirmed and went away
  before reading a program larger than its pipe was reported as having run it. Now `ran` is true only once
  the program frame was written whole (`Evidence.sent`; the death record carries `confirmed` and `sent`),
  and a program larger than the channel's frame (16 MiB less 64 KiB) is refused before a guest exists
  (exit 2, in words). Mutant M8 (`ran` = confirmed) **survived the first witness**: its fake guest closed
  its input before reading the host's acceptance, so a race chose which write failed. The witness now
  reads the acceptance first; M8 red three times in three.
- **GUEST-TEXT-1 (F2, F3)** — what a guest says reached the operator raw and unbounded: a refused
  confinement word carrying `\n` and ANSI escapes forged rows in `delulu audit query` and lines on the
  terminal, and a 10 MB word became a 10 MB report and a 20 MB audit chain (`denied` was bounded in count,
  never in size). Now `channel::shown` escapes control characters and bounds the text (64 characters for a
  quoted word, 512 per refusal, 1,024 per channel error) — at the refusal itself, in `note_denied`, and
  once over the conversation's error before the terminal, the report and the chain see it. Mutant M9
  (`shown` a pass-through) red in the e2e test and the unit test.
- **PIPE-WRITE-1 (F5)** — a write to an external guest that stopped reading had no deadline: a program
  larger than the pipe held the host for ever. `HostPipes` now writes through a writer thread, each write
  acknowledged within the channel's deadline or the channel fails (and then the guest is ended, above).
  Unit witness red on the old code (the write never returned).
- **PIPE-FLOOD-1 (F6)** — the queue between an external guest's output and the host was unbounded: while
  the host was stuck, a guest streaming zeros grew it to 3 GB in eight seconds (the sous-chef's RSS trace).
  Now 16 chunks of 64 KiB; past them the guest's own writes block. Unit witness red on the old code (64 MiB
  buffered).
- **F8** — a first request that was not the report was (rightly) not answered, and not recorded either:
  now `HostChannel::record_unanswered` puts it in `denied`. **F10** — the death record of a run refused on
  its attestation now carries `generation`, `confirmed`, `sent` and `ended_by_host`, like every other.

**Recorded, not fixed here:**
- **F7 (a design gap, the next step's):** an EXTERNAL guest's self-report — the word of a binary the
  launcher chose — enters `host_guarantees` and the posture, so a lying guest's report says "writes denied,
  network only the channel" while its `properties` (correctly) say `unknown`: the report contradicts
  itself. RW 4.31.
- **F9:** the audit chain carries `confirmed`/`sent` but not the guest's accepted words (they are in the
  report). **F11:** the channel's deadline is per read, not per frame (a guest dripping one byte every 40 s
  keeps a frame open past 60 s). **F12:** bytes after a frame's CBOR value are ignored. **F13:** some
  decoder messages reach the operator in the decoder's words (now bounded and escaped). **Residual:** a
  guest's standard error is the operator's terminal by design, so a guest can print lines that look like
  the host's. RW 4.32.
- **Not defects:** F14 (a guest accepts an `Open` without a generation — so a version-skewed host is
  refused on its version, in words); F15 (an external launcher inherits the operator's environment and can
  read the program from disk: it is the operator's own program; the generation binds a report to a run and
  is not a secret).

**Verified:** clippy `--workspace --all-targets -D warnings` clean; the full suite alone: 2013 passed, 0 failed, 15 ignored, 152 binaries — cargo exit 0.

## 2026-09-28 (night) — PS-E-01, third step: `hostile-agent` refuses a boundary that lacks a property (D-V2-59)

**Measured first.** `6ceaf2d`'s push run `36481810253` printed each runner's L1 properties: Linux x86-64
(a subordinate uid), Linux arm64 (the operator's own uid, `no_new_privs`) and Windows (an AppContainer) —
all five established; macOS — egress and the privilege floor only (reads open, no memory ceiling, no death
signal). An unattested external launcher establishes nothing (`unknown`). The microVM establishes all five
by its construction (the unit test over its words).

**Decided on those answers (D-V2-59).** `hostile-agent` — the profile for code nobody trusts — requires all
five; a boundary without one is refused **before the program is sent**, in `Opened::confirm`, with DL1408
(exit 2): the missing properties named with why, and the ways out (the microVM, an attested launcher, or a
weaker profile chosen by a person). `contained` (the default) and `dev` require nothing yet: D-V2-53's
`contained` set (filesystem, egress, resource) would refuse every macOS run and every unattested L3 run by
default, and that waits for the gaps to be closable (PS-E-02 for host loss; a read-deny Seatbelt profile
and a macOS memory ceiling are open) and for RW 4.31 — decided when they are, not before. The design's
second consequence (`hostile-agent` refusing on a Linux host without user namespaces, the privilege floor
read as "a separate identity") is NOT taken here: CI measured `no_new_privs` as that floor on arm64, and
reading it as identity would refuse a boundary the posture calls privilege-escalation-denied — recorded
for the owner's review with the required sets.

**Consequence, now true:** `run --sandbox --sandbox-profile hostile-agent` on macOS L1 refuses (DL1408);
`sandbox_run_cli`'s two `hostile-agent` tests expect exit 2 there and 0 elsewhere, and still read the
profile's limits from the report (which is written, with `ran: false`). An external launcher under
`hostile-agent` refuses unless an attester's claims can establish the properties (not yet mapped — PS-E-01's
remainder).

**Witness and falsifier.** `sandbox_confirm_cli.rs::hostile_agent_refuses_a_boundary_that_lacks_a_property_before_the_program_is_sent`
— red on `8b2994a` (the unattested launcher ran the program: the canary printed); now exit 2, DL1408 naming
`filesystem_confinement`, the canary absent, `ran: false`, and the same launcher under `contained` still
runs. Mutant M10 (`hostile-agent` requiring nothing) red.

**Verified:** clippy `--workspace --all-targets -D warnings` clean; every sandbox test binary green; the
full suite alone: 2014 passed, 0 failed, 15 ignored, 152 binaries — cargo exit 0.

## 2026-09-28 (night) — RW 4.31 closed: an external guest's words about itself are its own (D-V2-58 §5)

**CI read first.** `8b2994a` (the red-team fixes) push run `36486821458` — success on every job;
`42f5ea0` (`hostile-agent`'s required set) push run `36487993092` — success on every job: macOS's two
`hostile-agent` tests, written for a refusal nobody here could run, passed there (the refusal is real), and
the `microvm` job's `hostile-agent` runs at L2 still ran (all five established by its construction).

**The defect, witnessed by the head chef** (the sous-chef's F7): at L3 the guest is a binary the operator's
launcher chose, yet the words it reported applying to itself (RW 4.23) were merged into `host_guarantees`
and the posture — so a run through `external:<delulu> __guest --stdio-pipes` on Linux reported the guest's
Landlock and seccomp as the host's guarantees and "writes denied" in its posture, beside `properties` that
said `unknown`. Witness `an_external_guests_own_words_are_its_own_not_the_hosts` — red on `42f5ea0`.

**Fixed.** Where the host did not start the guest, the guest's words go to `sandbox.guest_reported` (the
closed schema gained it), never to `host_guarantees` or the posture: an L3 report now says `granted: none`,
no host guarantee, every posture row "not confined", and what the guest says it did, as the guest's word.
`sandbox_external_cli.rs`'s assertion ("only what the guest measured of itself, if anything") became the
strict one. Mutant M11 (the words merged again at L3) red.

**Verified:** clippy `--workspace --all-targets -D warnings` clean; the full suite alone: 2015 passed, 0 failed, 15 ignored, 152 binaries — cargo exit 0.

## 2026-09-28 (night) — PS-E-02, first part: an external launcher ends with its host (Linux)

`V2_OPENSHELL_STUDY.md` §4.2 named two gaps in "host loss ends the guest": the macOS guest (no death
signal) and the external launcher (started with neither a death signal nor a Job Object). The Linux half
of the second is witnessable in this VM: `an_external_launcher_ends_when_its_host_is_killed` starts a run
whose launcher records its pid and `exec`s `sleep 60`, kills the HOST with SIGKILL, and requires the
launcher gone (exited, or a zombie) within 3 s — **red on `71221d3`: the launcher outlived its host.**

**Fixed:** `launch_external` sets `PR_SET_PDEATHSIG = SIGKILL` in the launcher's `pre_exec`, as the jailed
guest always had, and exits at once if the host already died between the fork and the `prctl` (the
parent-pid re-check). The launcher now dies with its host (the witness passes in 0.02 s, twice). What the
launcher itself started — a container — is the launcher's to end, and nothing claims otherwise
(`host_loss_ends_guest` stays `unknown` at L3).

**Open (PS-E-02's rest, CI-only):** the macOS guest's watcher (`kqueue` `NOTE_EXIT` on the host, `getppid`
fallback) — the red one; the external launcher's Job Object with kill-on-close on Windows; each OS's
witness measured from outside both processes.

**Verified:** clippy `--workspace --all-targets -D warnings` clean; the full suite alone: 2016 passed, 0 failed, 15 ignored, 152 binaries — cargo exit 0.

## 2026-09-28 (night) — RW 4.32, one item: a channel frame means exactly one value

The red-team pass's F12: `channel::read_frame` decoded a frame's CBOR value and ignored whatever followed it
inside the frame's length, so one frame had many spellings (the sous-chef's `trailing` guest was confirmed
and served with 16 garbage bytes after its report). Witness `a_frame_with_bytes_after_its_value_is_refused`
— red on `aeea324` (decoded); now the frame is refused, "the frame carries 16 byte(s) after its value".
Every frame this protocol writes is exactly its value, so nothing legitimate changes (the sandbox, guest
and channel tests all pass). RW 4.32's other items stay open: a per-frame deadline, the accepted words in the
death record, the guest's standard error relayed with a prefix.

**Verified:** clippy `--workspace --all-targets -D warnings` clean; the full suite alone: 2017 passed, 0 failed, 15 ignored, 152 binaries — cargo exit 0.

**CI red, read and fixed at its root (the same commit).** `71221d3`'s push run `36490575764` — **failure**,
one job: `microvm`, step "The pinned Firecracker": `curl: (22) The requested URL returned error: 500` from
GitHub's release download, before any test body ran; every other job green. Not this diff's (it touched no
workflow and no script), and not a flake to shrug at: `fetch-firecracker.sh` fetched with no retry at all.
Reproduced in the VM against a local server that answers 500 once — plain `curl -f` exits 22; with
`--retry 4 --retry-delay 5` it retries and exits 0. The four downloads CI depends on now retry (the
Firecracker release, the kernel source in `build-image.sh`, TLA+'s tools, Lean's installer); plain `--retry`
covers HTTP 500 and runs on the laptop's older WSL curl, which `--retry-all-errors` would not; the sha256
pins still decide what is accepted. The failed job was re-run once (allowed: it died in a download step).

**Routine run 2, closed (22:45 UTC).** Every push run it made was read: `ff701ae` `36477748775`, `062a78c`
`36480421762`, `6ceaf2d` `36481810253`, `8b2994a` `36486821458`, `42f5ea0` `36487993092`, `71221d3`
`36490575764` (attempt 1 red — the upstream 500; attempt 2, the one re-run, success), `aeea324`
`36491419602`, `5d63119` `36492440031` — all success; `master` green at `5d63119`. The run built PS-E-01's
first three steps and RW 4.31, fixed seven red-team findings, began PS-E-02, and left the next run E-02's
macOS watcher (the red one) first — `docs/CLOUD_SYNC_LOG.md`'s entry has the inbox.

## 2026-09-29 — routine run 3: PS-E-02 on macOS — a watcher outside the guest ends it with its host (D-V2-60)

**Verified first (step 3).** Routine run 2's last push run, `7b9aac8` `36494315402` — success (read by id).
No nightly since `36402530469`. The Survey matched the tree (1,466 nodes, 12,743 edges) and `doctor --check`
passed (26 checks in this VM).

**Loop engineering first (`a39b423`).** Every open PS-E-02 item was CI-only, and the only way to see a macOS
test fail was the push run on `master` — which turns `master` red. `.github/workflows/witness.yml` runs one
test target on one runner at any ref, by hand; `scripts/check-macos.sh` runs clippy `-D warnings` for both
macOS targets from the Linux VM (a stand-in C compiler — check and clippy never link — and
`PYO3_CROSS_PYTHON_VERSION`), falsified with a planted macOS-only type error (E0308). Windows cannot be
checked that way (libffi-sys runs `configure` for an msvc target).

**The witness, red on macOS, off `master`.** `a_computing_guest_ends_when_its_host_is_killed`
(`sandbox_confirm_cli.rs`, Linux and macOS): a jailed guest past its boundary — the host has printed the
program's first line — spins, asking for nothing; the host is killed with SIGKILL; the guest's pid must be
gone within 3 s, read from outside both with `ps`. On Linux it is the control: green in 9 ms, from the jail's
`PR_SET_PDEATHSIG`, which no test had witnessed — and red with that `prctl` removed (a mutant, restored). On
macOS, `2d08622`, `witness.yml` run `36526271163`: **red** — "the guest outlived its host by more than
3.15 s". The external launcher's witness, extended to macOS: `9c38027`, run `36526351005`: **red** — "the
external launcher outlived its host".

**The fix (`jail.rs`, D-V2-60).** A watcher process, `__host_watch <pid>`, started right after the guest,
waits with `kqueue` on a pipe only the host holds and on the guest's exit; the host gone first ends the guest
with SIGKILL, the guest gone first ends the watcher, and an exit racing the end of file is looked for once
more before the kill. The `Jail` owns it, so a finished host ends its guest as the Windows job does.
"killed with the host" is claimed only once the watcher says it is armed. The study's design put the watcher
inside the guest as a thread; it runs outside instead, where an escaped guest cannot stop it (Seatbelt
denies it every signal). The external launcher gets the same watcher on macOS, claimed for nothing (L3).

**Green on macOS, on the same branch:** `36526707055` (`sandbox_confirm_cli`, 12 passed — the guest gone 3.6 ms after its host's SIGKILL, the launcher's witness green too), `36526709263` (`guest_cli`, 5 passed — the jail's report names "killed with the host"), `36526711462` (the boundary unit tests, 3 passed), all on `2d9d2a0`.

**Verified in the VM:** clippy `--workspace --all-targets -D warnings` clean; `scripts/check-macos.sh` clean
on both macOS targets; the full suite alone: 2,018 passed, 0 failed, 15 ignored (152 binaries) — cargo exit 0.

**Open (PS-E-02's rest):** the Windows external launcher's Job Object with kill-on-close (a Windows witness
through `witness.yml` first).

## 2026-09-29 — routine run 3: PS-E-02 on Windows — the external launcher ends with its host; PS-E-02 complete

**Loop engineering first.** `scripts/check-macos.sh` became `scripts/check-other-os.sh`: Windows (msvc) is
now linted from the Linux VM too — libffi-sys's build script, which runs `configure` and knows no Rust
triple, is replaced through its `links = "ffi"` key (Cargo's build-script override), and the stand-in
archiver understands `lib.exe`'s `-out:`. Falsified with a planted Windows-only type error (E0308).

**The witness, red on Windows, off `master`.** `an_external_launcher_ends_when_its_host_is_killed_on_windows`
(`sandbox_confirm_cli.rs`): the launcher is PowerShell, which writes its own pid and waits; the host is ended
with `TerminateProcess`; the launcher must signal its exit within 3 s (`OpenProcess(SYNCHRONIZE)`, a zero
wait). A jailed guest always lived in a Job Object with kill-on-close; the launcher was started in none:
`37828ab`, `witness.yml` run `36527876892` — "the external launcher outlived its host by more than 3.0 s".

**The fix (`jail.rs::end_with_host`, `guest.rs::launch_external`).** The launcher is created suspended, joins a
job whose ONLY limit is kill-on-close, and is resumed; the job's handle is the host's alone, so the host's end
closes it and ends the launcher and what it started inside the job. The Windows `Jail` holds it as a second
handle beside the jailed guest's measured job, so a level-3 run gains no processor-time watchdog and no stop
named from the launcher's accounting (D-V2-60 §4). The jail test's pointer cast into `Jail` became an
accessor. Green on Windows, the same branch: `1a63829`: `36528140459` (`sandbox_confirm_cli`, 7 passed — the launcher gone 20 ms after its host was killed), `36528149629` (`sandbox_external_cli`), `36528152057` (the jail unit tests), `36528154367` (`sandbox_run_cli`) — all success.

**PS-E-02 is complete:** the guest ends with its host on every backend — Linux (the death signal: guest, VMM,
external launcher), Windows (the job: guest and launcher), macOS (the watcher: guest and launcher) — each
witnessed red where it was missing and green after, and Linux's own death signal witnessed for the first
time. What an external launcher STARTS outside itself (a container a daemon runs) remains the launcher's.

## 2026-09-29 — routine run 3: PS-E-03, first step — the escaped guest; H1, H2, H3 confirmed and closed, H7 found (D-V2-61)

**The escaped guest (`jail::escaped_tests`).** A test-only child that applies exactly the guest's own
lock-down — Landlock, then the syscall filter, as `serve_as_guest` does — and then makes raw system calls,
printing `NAME=true|false`; the parent runs it FREE (the control: each attempt must succeed, or its refusal
proves nothing) and ESCAPED, beside an operator's world: a Unix socket outside anything granted (an SSH
agent's), a process of the same user holding `DELULU_ESCAPED_SENTINEL=the-operators-key` in its
environment, and a pseudo-terminal. A hypothesis is a finding only where the escaped attempt succeeds.

**Red on `1a63829`, all three hypotheses tested** (this VM, Linux 6.18, Landlock with network rights; the
control succeeded at every attempt):
- **H1 — GUEST-SYSCALL-1:** `memfd_create`, `io_uring_setup`, `userfaultfd`, `pidfd_open`, `fsopen` and
  `clone(CLONE_NEWUSER)` all succeeded after lock-down (`unshare` was denied; `clone` did the same thing).
- **H2 — GUEST-SOCKET-1:** UDP, netlink and Unix sockets were created, and the guest CONNECTED to the
  operator's socket outside its grant. And the report had said `network: only the channel` all along,
  answered from Landlock's TCP rule alone.
- **H3 — GUEST-PROC-1:** the guest read the operator's other process's `environ` — the sentinel — and its
  `cmdline`.
- **H7 — GUEST-DEV-1** (not in the study; found while narrowing `/proc`): `/dev` was granted whole, so the
  guest opened the operator's terminal (`/dev/pts/N`) for reading — keystrokes. Witnessed by mutant M3
  below (the old rule restored), red.

**Traced before narrowing.** `strace -f` of a sandboxed run: after `landlock_restrict_self` and `seccomp`,
the guest opens no file and makes no `socket`, `clone`, `memfd` or `io_uring` call at all — it interprets
and talks over the channel it already holds. So nothing global under `/proc` or `/dev` is granted.

**The fixes.** `lock_down_self` refuses `socket`, `socketpair`, the H1 calls, `clone` with any namespace
flag (one argument rule per flag), and answers `clone3` ENOSYS by a second filter; it reports the new known
word "no sockets but the channel", and the posture's network row now requires it (D-V2-61 §2).
`confine_filesystem` grants `/proc/self` (resolved in the guest, so its own entry) and five devices instead
of `/proc` and `/dev`. The explanation text (the SANDBOX topic) says so; the generated reference is in sync.

**Green, and falsified.** The four witnesses pass (every escaped attempt refused; `/proc/self` and
`/dev/null` still readable — the controls of the narrowing). Six mutants, each one fix reverted, each red:
M1 sockets allowed (UDP, NETLINK, UNIX_SOCKET, UNIX_CONNECT), M2 `/proc` whole (PROC_ENVIRON,
PROC_CMDLINE), M3 `/dev` whole (TERMINAL), M4 the `clone` flag rules dropped (CLONE_NEWUSER), M5 the
`clone3` filter dropped (CLONE3), M6 `memfd_create` allowed (MEMFD). The harness itself was caught twice
before it could be trusted: libtest's name line swallowed the child's first `NAME=` line, and a key filter
that allowed no digits dropped `CLONE3` — so an unreported attempt now fails the test.

**Verified:** clippy `-D warnings` clean on Linux, and by `scripts/check-other-os.sh` on arm64 Linux (whose
syscall table differs — now in the script's defaults), macOS and Windows; `delulu-conform --check-reference`
in sync; the full suite alone: 2,023 passed, 0 failed, 15 ignored (152 binaries) — cargo exit 0.

**Open (PS-E-03):** H4 (the host non-dumpable during a run), H5 (the Landlock ABI as a `hostile-agent`
requirement), H6 (macOS and Windows, through the same harness on their runners). CI's own proof of this
slice is the next push run: the x86-64 job's subordinate-uid guest, arm64, and the microVM guest (which
locks itself down with the same filter) all run it.

## 2026-09-29 — routine run 3: CI red on macOS, read and fixed — a guest gone before the host opens is told in words

**Read:** `3ec690b`'s push run `36527491801` — **failure**, one job, `test (macos-latest)`, one target,
`sandbox_external_cli`: `a_backend_that_is_not_one_or_a_launcher_that_dies_fails_legibly` requires a launcher
that exits at once (`delulu --version`) to fail in words, and the host printed `Broken pipe (os error 32)`.
Read again alone (`witness.yml` `36529502237`): the same. Every other job green. `c9739db`'s push run
`36529103180` carries the same code.

**Cause:** this run's macOS watcher. Starting it (a process, ~50 ms) delays the host's first write until such
a launcher has died, and `boundary::open`'s write error went to the operator raw — the reads beside it were
already in words. The race is older than the watcher; on Linux the first write lands before the launcher is
gone. The green reads of the watcher covered `sandbox_confirm_cli`, `guest_cli` and the boundary unit tests,
not `sandbox_external_cli`: a green read must cover every target that runs the changed code on that OS
(`witness.yml` now takes several, or `all`).

**Witness** `a_guest_gone_before_the_host_opens_the_channel_is_told_in_words` (`boundary.rs`, Unix): the
guest's end closed before `open` — red on `dd2a542` (`Broken pipe (os error 32)`), green after. **Fixed:**
`open` and `send_program` put a failed write in words ("it had closed the channel before the host opened
it"); the watcher exits 3 when the process it was given had already ended (`ESRCH`), and the host says so
instead of "did not say it was armed within 10s". Green on the runners: `ff251bb` — `36530276934` (macOS `sandbox_external_cli`, 3 passed, the failing test among them), `36530279678` (macOS `sandbox_confirm_cli`, 12 passed, the guest gone 4.9 ms after the kill); and PS-E-03 on the other Linux runners: `36530282206` (arm64, the escaped-guest tests, 5 passed — a non-root runner whose FREE control read the operator's `environ`, opened the terminal, made a user namespace and every socket kind, and whose escaped guest reached none; `userfaultfd` and `fsopen` unmeasurable there, refused to an unprivileged user even unconfined), `36530285142` (x86-64 with the subordinate-uid guest, `sandbox_run_cli` 17 passed), `36530287510` (arm64 `sandbox_run_cli`, success).

## 2026-09-29 — routine run 3: PS-E-03 H4 — a serving host is closed to its own user (D-V2-62)

**Measured by hand first, as a non-root user in the VM** (`runuser -u delulutester`): a sandboxed host
spinning a program, started with `DELULU_H4_SENTINEL=the-hosts-secret`; a second process of the same user
read `/proc/<host>/environ` — the sentinel was there — and the file belonged to that user: the host was
dumpable. With H3 closed an escaped guest can no longer reach it, but any other process of the operator's
could, and where Yama is off (this VM, many containers) its memory too.

**Witness** `a_serving_hosts_environment_and_memory_are_closed_to_its_own_user` (`sandbox_confirm_cli.rs`,
Linux): the host serves a spinning program; the test, as the same user, reads the host's `environ` and opens
its `mem`. Red on `30e3262` run as `delulutester` ("a process of the same user read the serving host's
environment"): **HOST-DUMPABLE-1**. As root it says "unmeasurable" — root reads any process — and CI's
runners are not root.

**Fixed** (`guest.rs::serve_under`): `PR_SET_DUMPABLE = 0` right after the guest is launched — not before,
because a child forked from a non-dumpable process keeps the flag until its `exec`, and the identity path
writes that child's uid map then. Green as `delulutester`: `sandbox_confirm_cli` 13 passed; the red run is
the mutant (the same tree without the line).

**Verified:** clippy `-D warnings` clean; the full suite alone (as root): 2,025 passed, 0 failed, 15 ignored (152 binaries) — cargo exit 0 (the witness, as root, reports itself unmeasurable); as `delulutester`, `sandbox_confirm_cli` 13, `sandbox_run_cli` 17, `guest_cli` 5, `sandbox_external_cli` 3 — all passed.

**Open (PS-E-03):** H5 (the Landlock ABI as a `hostile-agent` requirement) and H6 (macOS and Windows under
the same harness).

## 2026-09-29 — routine run 3: PS-E-03 H8 — an escaped guest typed into the operator's terminal (D-V2-63)

**Asked while closing H4:** what else does an escaped guest hold? Its standard error stays attached to the
operator's terminal (the host's, by design, so a failing guest can say why), and it shares the host's session
— so that terminal is its controlling terminal. `dev.tty.legacy_tiocsti` is 1 in this VM (kernel 6.18).

**Witness** `h8_an_escaped_guest_types_nothing_into_the_operators_terminal` (`jail::escaped_tests`): the child
takes the pseudo-terminal as its controlling terminal before its lock-down, as the real guest inherits it,
and after it tries `ioctl(TIOCSTI)`. **Red** on this run's tree: `still reached: ["TIOCSTI"]` —
**GUEST-TIOCSTI-1**: a keystroke in the operator's input, for their shell to run after the guest is gone.

**Fixed** (`lock_down_self`): `ioctl` with `TIOCSTI` or `TIOCLINUX` refused, compared on the command's low 32
bits. **Mutants:** M7 (the rule dropped) red on `TIOCSTI` and `TIOCSTI_HIGH`; M8 (compared on 64 bits) red on
`TIOCSTI_HIGH` alone — the same command with its high bits set, which the kernel truncates, walked past a
64-bit comparison.

`libc::Ioctl` is `c_int` on musl — the microVM's static guest — and `c_ulong` on glibc, so the rule's casts
stay, allowed with that reason; `scripts/check-other-os.sh` now lints `x86_64-unknown-linux-musl` too
(without the cast musl fails, E0308, where glibc compiles — only CI's `microvm` job would have said so).

**Verified:** clippy `-D warnings` clean on Linux and the five other targets; the full suite alone: 2,026
passed, 0 failed, 15 ignored (152 binaries), cargo exit 0. **CI:** `30e3262`'s push run `36531207166` —
success on every job, `master` green again after `3ec690b`/`c9739db` (the macOS test job, and the `microvm`
job whose guest runs PS-E-03's filter).

**Open:** the guest's own output still reaches the terminal raw (escape sequences) — RW 4.32's relay; and
the guest could be given no controlling terminal at all.

## 2026-09-29 — routine run 3: PS-E-03 H9 — an escaped guest could signal the operator's processes (D-V2-64)

**The next "what else":** the filter named no signal call. **Witness**
`h9_an_escaped_guest_signals_no_other_process` (`jail::escaped_tests`), with signal 0, which asks only whether
a signal WOULD be delivered: **red** on `e584f8d` — `still reached: ["SIGNAL_OTHER", "SIGNAL_GROUP",
"SIGNAL_QUEUE"]` — **GUEST-SIGNAL-1**: another process of the same user, the guest's process group (its host,
the terminal's foreground job), a queued signal; `kill(-1, SIGKILL)` would end the operator's session.

**Fixed** (`lock_down_self`): `kill`, `tgkill`, `rt_sigqueueinfo`, `rt_tgsigqueueinfo` refused unless their
first argument is the guest's own pid (fixed when the filter is made, compared on 32 bits); `tkill` refused.
The control `SIGNAL_SELF` — a guest must still signal itself, as `abort` does — stays true. **Mutants:** M9
(the rules dropped) red on all three; M10 (the comparison reversed) red on the control.

**Verified:** clippy `-D warnings` clean on Linux, arm64 Linux and musl; the full suite alone: 2,027 passed,
0 failed, 15 ignored (152 binaries), cargo exit 0.

## 2026-09-29 — routine run 3: PS-E-03 H10 — an escaped guest could change the operator's processes (D-V2-65)

**The rest of H9's class:** the calls that act on another process by pid. **Witness**
`h10_an_escaped_guest_changes_no_other_process` (`jail::escaped_tests`), each call setting the value it just
read, so nothing is changed: **red** on `407e423` — `PRLIMIT_OTHER`, `PRIORITY_OTHER`, `AFFINITY_OTHER`,
`SCHEDULER_OTHER`, `IOPRIO_OTHER` — **GUEST-PROCESS-1**: an escaped guest could lower the operator's
processes' limits and priorities, pin them to one CPU, or change their scheduling.

**Fixed** (`lock_down_self`): `setpriority`, `sched_setaffinity`, `sched_setscheduler`, `sched_setparam`,
`sched_setattr`, `ioprio_set`, `process_madvise` refused; `prlimit64` refused unless its pid is 0 or the
guest's own (the C library reads its own stack limit — the control `PRLIMIT_SELF`). **Mutants:** M11 (the
setters allowed) red on four; M12 (`prlimit64` allowed) red on `PRLIMIT_OTHER`; M13 (pid 0 refused too) red on
the control. The musl lint caught the witness's `sched_param` spelled glibc's way (musl's has more fields).

**Verified:** clippy `-D warnings` clean on Linux, arm64 Linux and musl; the full suite alone: 2,028 passed,
0 failed, 15 ignored (152 binaries), cargo exit 0.

## 2026-09-29 — routine run 3: PS-E-03 H5 — "no file writes but truncation" read as "writes denied" (D-V2-66)

**Meant as a pin, found a defect.** H5 asked for Landlock ABI ≥ 3 (truncation mediated) as a `hostile-agent`
requirement; reading the posture it seemed to hold already — so a test was added to PIN it: the words of a
kernel before ABI 3 ("no file writes but truncation" …) must leave `filesystem_confinement` absent. **Red** on
`be749d1`: `established`, by "filesystem_writes: denied". The posture's `has` matches a word inside another,
and "no file writes but truncation" contains "no file writes" — **LANDLOCK-TRUNCATE-1**: on kernels 5.13–6.1
the report claimed writes denied while an escaped guest could `truncate` any file its user can write, and
`hostile-agent` ran there.

**Fixed** (`policy.rs`): writes are "denied" only on the exact word; `has`'s substring match stays where the
microVM's longer words rely on it. Green; below ABI 3 the report says writes are not confined and
`hostile-agent` refuses. Kernels ≥ 6.2 (CI's 6.8) are unchanged. The search key of §11.4 again: a security
decision on a looser spelling — here a word containing a stronger one.

**Verified:** clippy `-D warnings` clean on Linux and all five other targets; the full suite alone: 2,028
passed, 0 failed, 15 ignored (152 binaries), cargo exit 0.

**Routine run 3, closed.** It built `witness.yml` and `scripts/check-other-os.sh` (macOS, Windows, arm64 Linux
and musl linted from the VM — each falsified with a planted error), then used them: PS-E-02 completed on
macOS (a watcher outside the guest) and Windows (the launcher's kill-on-close job), each witnessed red on a
runner and green on the same branch before `master` moved; PS-E-03's escaped guest confirmed H1, H2 and H3
and found H7, H8 (keystrokes into the operator's terminal), H9 (signals) and H10 (other processes) — all
closed, each with mutants — and H4 (the host non-dumpable) as a non-root user. `master` went red once
(`3ec690b`/`c9739db`, macOS, a raw `Broken pipe`), was read, fixed at its cause and read green
(`36531207166`). Push runs: `a39b423` `36526041627` success; `3ec690b` `36527491801` **failure** (macOS `sandbox_external_cli`, a raw `Broken pipe` — fixed in `ff251bb`); `c9739db` `36529103180` **failure** (the same); `30e3262` `36531207166` success (every job — `master` green again); `e584f8d` `36533139190` success; `407e423`, `be749d1` and this entry's commit — running as it was written (Open / next (1))

**Every push run read (07:34 UTC):** `407e423` `36533983121`, `be749d1` `36534873999`, `33c20e5` `36535808138`
— success. `master` is green at `33c20e5`; the only run left unread is the closing commit's own.

## 2026-09-29 — routine run 4: `master` red on a records-only commit, read and fixed — Wasmtime 48 (D-V2-67); an attestation read half-written

**Read first (step 3).** Routine run 3's closing commit `9fc4d86` — records only — had a red push run,
`36537537718`: two jobs, two unrelated causes, neither in its diff. Every other job green (Linux x86-64 and
arm64, Windows, the microVM, fuzz, Miri, the formal models). The nightly on the same commit, `36548984501`,
was running when this run started. `gh`: absent again (`command -v gh` empty) — CI read with the GitHub MCP
tools.

**1. `supply-chain` — new advisories.** RUSTSEC-2026-0315 and RUSTSEC-2026-0316, both published after the last
clean run, both against wasmtime 47.0.4. **Witnessed** in the VM (cargo-deny 0.20.2, installed for it): exit 1
on 47.0.4 naming both, exit 0 on 48.0.3; bans, licences and sources ok. **Fixed:** `wasmtime = "48"` (48.0.3,
the LTS line); D-V2-67 has the reachability — 0315 in principle, since fuel is a granted limit and function
references and exceptions are not refused; 0316 not, the component model is off — and why 48 rather than 49.
No source line changed. RW 4.33 records the narrowing that would have made 0315 unreachable.

**2. `test (macos-latest)` — `sandbox_attest_cli`, one test.**
`a_document_replayed_from_another_run_is_refused_on_its_nonce` got "the attestation is not a
`delulu-attestation-v1` document (EOF while parsing a value at line 1 column 0)" where it expected "not made for
this run" (read from the job's whole log: 3,423 lines, the failure at line 1,556). The host reads the attestation
the moment it exists — the protocol says an attester writes it whole, a temporary file then a rename, and
`delulu sandbox attest` does — but the test's fake replaying launcher used a plain `cp`, which creates the file
and then fills it, and on the macOS runner the host read it in between. **The fake broke the protocol it fakes.**
**Witnessed** deterministically, not by rerunning: the same launcher with its `cp` held open (the file created,
`sleep 1`, then filled) red with the runner's exact words. **Fixed:** the launcher copies to
`$DELULU_ATTEST_OUT.part` and renames it; M3, that launcher slowed the same way, stays green — the rename is
what holds.

**And the words.** An operator whose attester writes in place was shown a parser's position. New refusal
`incomplete` (`attest::Refusal::Incomplete`): a document that ends early — `serde_json`'s end-of-input error,
or bytes that stop inside a UTF-8 character — is refused as "the attestation is incomplete … an attester writes
it whole — a temporary file, then a rename", still before the program is sent. **Witness**
`a_document_written_in_place_is_refused_in_words_that_name_the_rename` — an empty file and half a document, held
still so it is not a race — red on `9fc4d86`, green after; the unit test cuts a good document at four places
and inside a multi-byte character. **Mutants:** M1 (the end-of-input mapping removed) red on both tests; M2
(the mid-character mapping removed) red on the unit test.

**Verified:** clippy `-D warnings` clean; `sandbox_attest_cli` 7 passed, the attest unit tests 6 passed; the full
suite alone: 2,029 passed, 0 failed, 15 ignored (152 binaries), cargo exit 0.

## 2026-09-29 — routine run 4: RW 4.33 — a Contained plugin could still reach the proposals RUSTSEC-2026-0315 lived in (D-V2-68)

**The consequence D-V2-67 named, witnessed.** The upgrade closes the advisory; it does not close the door.
**Witnesses** (`delulu-wasm/src/host.rs`, `feature_hardening_tests`), red on `63a375e`:
`both_hardened_engines_refuse_every_wasm3_proposal_codegen_never_emits` — the program engine accepted
function references, exceptions, GC, tail calls and multi-memory (each fixture first proven VALID on a
control engine with only its proposal on); `a_contained_plugin_using_a_wasm3_proposal_never_runs` — the
plugin store, the engine that meters GRANTED fuel, ran a module calling through `call_ref` and answered
`Some(7)` (its control, the same module with a direct `call`, runs too).

**Fixed** (`harden_wasm_features`, called by both engines): the WebAssembly 3.0 proposals refused at
validation — GC, function references, exceptions, stack switching, tail calls, multi-memory, custom page
sizes, wide arithmetic, shared-everything threads. **Kept:** the WebAssembly 2.0 set a default wasm32
toolchain emits, and extended constants — `the_webassembly_2_baseline_still_loads_on_the_hardened_engine`
holds that side. Legacy exceptions got no fixture and no line: no engine here can switch them on
("not supported on this compiler configuration" — found when the fixture could not be proven valid), and
their switch is deprecated, internal to wasmtime — found by the Windows lint, which failed on the
deprecation.

**Mutants:** M5 (function references allowed) red on both witnesses — on the plugin engine the fixture then
reached "no export `run`" instead of failing to compile; M6 (exceptions allowed) red; M7 (over-narrowing:
reference types and bulk memory refused too) red on the baseline test ("bulk memory support is not enabled").

**Loop engineering, found here:** `scripts/check-other-os.sh` linted `delulu` alone, so `delulu-wasm`'s
tests had never been linted for Windows, where a helper used only off Windows is dead code under
`-D warnings`. It now lints `delulu`, `delulu-runtime` and `delulu-wasm` — falsified: with the helper's
`cfg` removed, the Windows lint is red ("function `plugin_limits` is never used").

**Verified:** clippy `-D warnings` clean on Linux; `scripts/check-other-os.sh` clean for Windows, macOS arm64
and Linux arm64 (three packages each); the full suite alone: 2,032 passed, 0 failed, 15 ignored
(152 binaries), cargo exit 0 — the parity suites among them (actors 7, conformance 6, fault 2, foreign 8,
hostile guest 5). The two-engine differential is a heavy gate the suite ignores, so it was run by hand at its
full size: `cargo test -p delulu-wasm --release --test differential -- --ignored` — 50,000 programs, passed in
147 s (5,000 first, 15 s).

**`63a375e`'s push run, `36556961783` — success on every job: `master` green again.** macOS: egress, host loss
and privilege established, filesystem and memory absent, as before; ping-pong MEASURED 2.66x against a 1.41x
bar (control 3.75x) — passed. Windows: all five properties established; ping-pong NOT MEASURED, the runner
busy (controls 1.93x–2.50x, under 3x). Read on a green run the verdicts sit ~100 lines from the end — the cache
save follows them — so the routine's step 3 now says how to reach them.

**Slice 2 read on the other runners before `master` moved** (`witness.yml` on `claude/friendly-thompson-w49pvi`
at `f33183c`, `delulu-wasm` `lib`, filter `feature_hardening`): macOS `36560326859` — 5 passed; Windows
`36560329484` — 4 passed (the plugin-store test is Unix-only: Windows refuses Contained plugins up front);
Linux arm64 `36560332509` — 5 passed. Then `master` fast-forwarded to `f33183c`.

**Not run: the red-team pass on the Linux guest's filter** that routine run 3's inbox named. A safety classifier
stopped the response that was briefing the Sonnet 5.5 sous-chef, before the agent started; the pass was dropped
for this run, and nothing of it ran. No fallback notice was shown.

## 2026-09-29 — routine run 4: RW 2.2 — overtaken by P3, and the pin it relied on did not exist

**Checked against the tree, the row was stale:** `List.filter` exists since P3 (`prim_table.rs:149`), and
`is_higher_order_method` became `higher_order_callback_arg`, which carries the callback's position. **But its
doc comment named a pinning unit test, `every_higher_order_callback_position_is_where_method_sig_expects_a_function`,
that exists nowhere in the tree** — a gate claimed and absent. What does pin it: `stdlib_p3.rs`'s
`every_higher_order_builtin_surfaces_its_callbacks_row`, end to end — M8 (`fold`'s callback read at index 0)
red ("`fold` dropped its callback's row", DL0401 instead); M9 (`filter` dropped) red. **And nothing pinned the
fifth entry:** M10 (`Secret.map` removed) passed every test — delulu-check's 270, `stdlib_p3`,
`secret_verify_cli` — because DL0603 refuses an effectful mapper for another reason. **Pinned now:**
`secret_oracle.rs::an_impure_mappers_row_surfaces_in_its_caller` — the callback's `{Write}` must surface in its
caller's row (DL0501); green, and M10 red (`["DL0209", "DL0401", "DL0603"]`, no DL0501). The comment names the
two real pins; RW 2.2 closed.

**Verified:** clippy `-D warnings` clean; the full suite alone: 2,033 passed, 0 failed, 15 ignored (152
binaries), cargo exit 0.

**RW 2.4 measured, not fixed.** The row said 16 KB of nested list literals took 17.8 s to check. The parser has
refused nesting past 128 levels since 2026-08 (DL0210), so that shape no longer parses; at the deepest legal
depth, 16 KB (depth 120, 64 nests) checks in 0.39 s, and time is linear in size at a fixed depth (debug build,
an upper bound). Per nest the cost still grows about as depth^1.6. One idea was tried and refuted: typing a
list literal by its first element instead of a fresh type variable made the same file ten times slower
(1.63 s → 17.0 s) — reverted, byte for byte, and recorded in the row so it is not tried again. RW 2.4: Partial,
bounded.

**Routine run 4, closed.** `master` was red on arrival — `9fc4d86`, a records-only commit, against new Wasmtime
advisories and a test's fake attester that wrote in place — and green from `63a375e` (D-V2-67, the `incomplete`
refusal). The run then shut the door the advisory used (D-V2-68: the plugin store had run a `call_ref` module;
both engines now refuse the WebAssembly 3.0 proposals), read that slice on macOS, Windows and Linux arm64 before
`master` moved, found a comment naming a test that never existed and an R-4 entry pinned by nothing (RW 2.2),
and measured RW 2.4 bounded (one idea refuted). Push runs: `63a375e` `36556961783` success; `f33183c`
`36560942558` success; `2ea4208` `36562163647` success on every job. Unread: the nightly `36548984501` (still running) and the
closing commit's own run. The red-team pass on the guest's filter did not run (its brief was stopped by a safety
classifier before the agent started).

**The closing commit's run, read:** `eb3af0b` `36564217831` — success on every job. `master` green at `eb3af0b`.
The nightly `36548984501` (`9fc4d86`) was still running at 12:08 UTC — the next run reads it.

## 2026-09-29 — routine run 5: CI read; PS-E-04 — the external launcher resolved once, hashed, pinnable, started as the file hashed (D-V2-69)

**CI, read first.** `master` was green on arrival: `cfc5b01` (routine run 4's last commit, records only)
`36566189830` — success on every job. **The nightly `36548984501` on `9fc4d86` — failure, one job:**
`supply-chain`, RUSTSEC-2026-0315 and -0316 against wasmtime 47.0.4 — the advisories `63a375e` fixed, so
the old commit's record, as run 4 expected. Every other job green, the three `miri-slow` jobs included
(`delulu-syntax` 20 min, `delulu-check` 67 min, `delulu-broker` 2 h 45 min); `test (macos-latest)` green
this time. `gh`: absent in this VM too. `cargo deny --all-features check advisories`: ok on `cfc5b01`.

**Witnessed first, on `cfc5b01` — LAUNCHER-SPELL-1.** `launch_external` gave the command's first word to
`Command::new`, and the operating system's `PATH` search honours an empty or relative entry: with `.`
ahead of the operator's directory, `external:lnch` ran a `lnch` planted in the working directory, and the
report said `lnch` (`a_bare_launcher_name_never_means_a_file_in_the_working_directory`: "the file planted
in the working directory ran"). The same shape ADAPTER-SPELL-1 had for hardware drivers. The other four
new witnesses were red for want of the feature (no `launcher_path`; `--launcher-digest` unknown).

**Built (`crates/delulu/src/launcher.rs`, new; `guest.rs`):** the word resolved by `cli::resolve_driver`
(shared with ADAPTER-SPELL-1's fix — `PATH`'s absolute directories only); the file opened once,
non-blocking, judged by its descriptor's `fstat`, and hashed with BLAKE3; `sandbox.launcher_path` and
`sandbox.launcher_blake3` in the report and the `sandbox-launch` record; `--launcher-digest HEX` refuses
any other file before anything starts — exit 1 in words naming the path and both digests, and a
`sandbox-launcher` deny record. **On Linux the descriptor hashed is the file started** (`fexecve`, the last
`pre_exec` step; argv and envp built before the fork; a `#!` script's descriptor kept open for its
interpreter's `/dev/fd/N`). macOS and Windows start the resolved path by name — that window is named in
`DEPLOYMENT.md`, not claimed closed (D-V2-69 §7).

**The environment witness caught the first build.** It started the launcher with `environ` as the
environment, on the reading that the standard library installs the command's environment there before a
`pre_exec` step runs: it does not — `the_launcher_is_told_the_guest_words_and_the_limits` went red, the
launcher started with no `DELULU_GUEST_ARGS` and printed `delulu`'s help. The environment is now built
from the command (`vars_os` with `get_envs` applied), before the fork.

**Witnesses** (`sandbox_external_cli`, five new): the digest in the report and the launch record equals an
independent BLAKE3 of the file; a pinned run with another file exits 1 naming both digests, runs nothing,
records a `sandbox-launcher` deny and no launch; a pinned script launcher that is not the pinned file is
never STARTED (its marker absent), the pinned one is; the planted `PATH` entry is not what runs; and **the
path swapped while the host is seen hashing is not what runs** — a 64 MiB launcher holds the window open,
and the test renames B over A the moment `/proc/<host>/fd` shows the host holding A. **Mutants:** M1 (the
path started by name, not the descriptor) — B ran, 3 of 3 (and A, 3 of 3, restored); M2 (the pin never
compared), M3 (the digest of the name, not the bytes), M4 (the word opened as it is spelled), M5 (a
script's descriptor closed at the start), M6 (the refusal unrecorded) — each red; each restored byte for
byte (`cmp`).

**Verified:** clippy `-D warnings` clean; `scripts/check-other-os.sh` clean for macOS (arm64 and x86-64),
Windows, Linux arm64 and musl; the full suite alone (`-j 4`): 2,034 passed, 4 failed, 15 ignored (152
binaries) — the four failures `doctor_cli` (three) and the Survey's freshness test, the map stale because the
Survey is regenerated last; regenerated, both targets were re-run green. **Read on the other runners before
`master` moved** (`witness.yml` on `claude/friendly-thompson-gsorc5` at `370d641`, targets
`sandbox_external_cli sandbox_attest_cli sandbox_confirm_cli`): macOS `36595372725` — 7, 12, 7 passed;
Windows `36595377128` — 4, 7, 4 passed (the report's digest and the pin witnesses included; the
script-launcher witnesses are Unix-only); Linux arm64 `36595380398` — 7, 13, 8 passed, the swap witness
among them (`fexecve` on aarch64). Then `master` fast-forwarded to the branch.

**Open in E-04** (RW 4.28): Windows could hold the launcher open deny-write between the hash and the start
— not built, because nothing yet witnesses it; macOS has no `fexecve`; and an attestation that binds a
launcher digest the attester measured itself.

## 2026-09-29 — routine run 5: TERMINAL-TEXT-1 — a pure program wrote escape sequences to the operator's terminal (D-V2-70)

**Found while scoping RW 4.32** (a guest's standard error is the operator's terminal): what does a guest
itself print there? Its own fault line, `error[CODE]: message` — and a fault can quote the program's values.
**Witnessed on `3489b57`:** `fn main(root: Root) { assert_eq("\u{1b}]0;PWNED\u{7}\u{1b}[2K\rsandbox: forged", "x") }`
— a program with an empty effect row and no grant — printed `ESC ] 0 ; PWNED BEL ESC [ 2 K CR sandbox: forged`
raw to standard error, in a plain run (the renderer) and under `--sandbox` (the guest's line): a title set, a
line erased, a host line forged; OSC 52 would set a clipboard. The same through a refusal quoting the
program's path (DL0703, in the message and in the `--grant` it suggests), a test's name and failure in
`delulu test`, and a quoted source line. JSON was never affected (`\u001b`).

**Fixed at the printers** (D-V2-70): `delulu_diag::terminal_safe` (every control but a line break or a tab,
C1 included, and the bidi and line-separator characters, shown as `\u{…}`; borrowed when clean) in the
renderer's message; `terminal_line` (the line break too) for labels, file names, the guest's fault line,
`delulu test`'s `ok:`/`FAIL:` lines and the REPL's fault line; a quoted source line shows each as `?`, so its
caret keeps its column.

**Witnesses** (`tests/terminal_text_cli.rs`, new — five, each red on `3489b57`): an assertion's values in a
plain run; the same as a sandboxed guest; a refusal's path (and the JSON envelope still exact); a quoted
source line; a test's name carrying a line break and a forged `test result:` line (exactly one summary
survives). And `render.rs`'s unit test over C0, C1 (the 8-bit CSI), bidi, U+2028 and NUL. **Mutants:** M7
(the renderer's message raw), M8 (the guest's fault raw), M9 (`FAIL:` raw), M10 (a quoted line raw), M11 (C1
and bidi not escaped — the unit test), M12 (a one-line context keeps its line break — the forged summary
appeared) — each red, each restored byte for byte.

**Named, not closed:** a line break inside a rendered message (one checker message is laid out on three
lines), RW 4.34; an ESCAPED guest's raw standard error and the microVM's raw console relay, RW 4.32.

**The core-invariance snapshot moved by one line, reviewed and re-recorded (D-NE-3):** of every case in
`tests/core-invariance/SNAPSHOT.txt`, only `DL0107_bidi_override.delulu :: check` changed — its quoted source line
carried the raw U+202E RIGHT-TO-LEFT OVERRIDE, so the diagnostic that warns a bidi control can make code
render differently than it runs was itself re-ordering the operator's terminal; it now shows `?` in the
character's column, the caret unmoved. Every other case unchanged, byte for byte.

**Verified:** clippy `-D warnings` clean; the full suite alone (`-j 4`), the map regenerated before it: 2,043
passed, 1 failed, 15 ignored (153 binaries) — the one `core_invariance`, the snapshot above; re-recorded,
green. No code in this slice is specific to an operating system (the new witnesses run on all three in the
push run).

## 2026-09-29 — routine run 5: RW 4.32 — a guest's standard error, relayed by the host (D-V2-71)

**Witnessed on `6cd68c8`** (TERMINAL-TEXT-1 had escaped only the guest's own fault line): with a launcher
standing in for a guest that escaped its interpreter, `ESC ] 0 ; PWNED BEL ESC [ 2 K CR` and a forged
`sandbox: the guest is confined — forged` line reached the operator's terminal raw and unmarked; 3 MiB on the
launcher's standard error reached it whole; and the guest's own lines carried no mark of whose they were.

**Fixed** (`guest.rs::relay_stderr`, D-V2-71): the stream is a pipe the host reads, a line at a time, each
escaped and marked — `guest:` at L1, `launcher:` at L3 — cut at 1 MiB (drained past it, said once), and the
host waits up to 2 s after the guest has gone for the last line. `launch` now separates capturing the stream
from staying quiet (the probe does both; a run captures and speaks). The microVM's console relay is
unchanged — still raw, still capped — and stays in RW 4.32.

**Witnesses** (`terminal_text_cli`, four new, each red on `6cd68c8`): the guest's fault arrives as
`guest: error[DL1707]` on every OS and never unmarked (on Linux, its boundary lines too); a launcher's control
sequences arrive escaped and its forged line as the launcher's; a 3 MiB flood is cut near 1 MiB, said, and
the run goes on; a line written a second after the guest has gone, by a process the launcher left holding
the stream, is still relayed. **Mutants:** M13 (a jailed guest's stream inherited), M14 (the relay raw), M15
(no cap), M17 (a launcher's stream inherited) — red; **M16 (the last line not waited for) survived the first
four witnesses** — the relay thread wins that race in practice — so the fourth was written to hold the
window open: red 3 of 3, green 3 of 3 restored.

**Verified:** clippy `-D warnings` clean; `scripts/check-other-os.sh` clean on five targets; the full suite
alone (`-j 4`, the map regenerated first): 2,048 passed, 0 failed, 15 ignored (153 binaries), cargo exit 0.
**Read on the runners before `master` moved** (`witness.yml` at `ac65c0e`, six targets — `terminal_text_cli`,
`sandbox_run_cli`, `sandbox_external_cli`, `sandbox_attest_cli`, `sandbox_confirm_cli`, `guest_cli`): Linux
x86-64 `36599642591` (the subordinate-uid launch) — all green, 9, 17, 8, 7, 13, 5; macOS `36599633669` — all
green, 9, 16, 7, 7, 12, 5; **Windows `36599638102` — one red**: TERMINAL-TEXT-1's refusal witness, already on
`master` in `6cd68c8`, asked for a path holding `sandbox: forged`, and Windows refuses a `:` in a component (an
alternate data stream, DL0904) before the grant is asked — its message escaped, the product holding, the
witness assuming one OS's path rules. Fixed in `d1dbf2e` (the same bytes without the `:`, DL0703 on every
OS; M7 re-run red on it), read green on Windows (`36600749661`). `6cd68c8`'s push run was red on Windows for
that one test; `d1dbf2e` is its fix, and the lesson is in `HANDOFF.md` §11.5 and the routine's step 5: a
slice's NEW tests are read on every runner before `master`, whatever its code touches.

## 2026-09-29 — routine run 5: RW 4.34 — a program's line break began a line of its own (D-V2-72); H6 stopped

**Witnessed on `41ac869`:** `assert_eq("a\nsandbox: the guest is confined — forged\nerror[DL0000]: forged", "x")`
in a plain run put two lines the host never wrote at column 0 of its diagnostic — TERMINAL-TEXT-1's named
residual, no control sequence needed. **Fixed** (`render.rs`, D-V2-72): a message keeps its line breaks and
every line after its first is indented, so none begins where the host's own lines begin. Witness
`a_programs_line_break_never_starts_a_line_of_its_own` (`terminal_text_cli`) red on `41ac869`, green after;
M18 (the indentation removed) red, restored byte for byte.

**Verified:** clippy `-D warnings` clean; the full suite alone (`-j 4`, the map regenerated first): 2,049 passed, 0 failed, 15 ignored (153 binaries), cargo exit 0. Its new witness read on the other runners before `master` moved — the rule this run wrote down —
(`witness.yml` at `b7393c4`, `terminal_text_cli`): Windows `36602360567` and macOS `36602364313`, success.

**Not done: PS-E-03 H6.** The head chef began it — the macOS Seatbelt profile under an escaped-guest harness
— and a safety classifier stopped that response before any of it ran; nothing was written or dispatched, and
no model fallback notice was shown. As with run 4's stopped red-team brief (`HANDOFF.md` §11.7), the work
was dropped for this run rather than re-attempted in other words. The profile itself is deny-by-default with
no `mach-lookup` (`jail.rs::seatbelt_launcher`); what H6 asks — each word the macOS report claims, witnessed
against an escaped guest — stays open, and may be one to do with the owner at the laptop.

## 2026-09-29 — routine run 5: RW 4.32 — the death record names the guest's words

The audit chain recorded that a guest confirmed its boundary (`confirmed`, `sent`, the generation) but not
with WHAT — the words it claimed to have applied to itself were in the run report only, and a report can be
discarded. **Witnessed on `d566295`**: the `sandbox-death` record carried no words. Now it carries
`guest_words` — the words the host accepted at confirmation, its own vocabulary, nothing a guest can spell —
equal at L3 to the report's `guest_reported`, and at L1 each among what the report says was applied; the
attestation refusal's death record carries an empty list, so every death record keeps the same fields (F10).
Witness `the_death_record_names_the_words_the_guest_confirmed_with` (`sandbox_confirm_cli`), red on
`d566295`, green after; M19 (the words dropped) red, restored byte for byte.

**Verified:** clippy `-D warnings` clean; the full suite alone: 2,050 passed, 0 failed, 15 ignored (153 binaries), cargo exit 0. Read on the other runners before
`master` moved (`witness.yml` at `30d1563`, `sandbox_confirm_cli sandbox_attest_cli`): Windows `36602983681` and macOS `36602987750`, success.

**`41ac869`'s push run, `36601247472` — success on every job (16): `master` green again** after `6cd68c8`'s
Windows red; Windows' properties line on `6cd68c8`'s run read all five established, its ping-pong NOT
MEASURED (the runner busy).

## 2026-09-29 — routine run 5: RW 4.32 — the microVM's console, escaped

The last raw path of RW 4.32's terminal item: `microvm.rs::relay_console` wrote the microVM guest's console —
its kernel's text and its standard error — to the operator's terminal as bytes (capped at 64 KiB), and kept
the same bytes for the probe's answer. **Witnessed on `ec4d39d`** with the relay driven by a real child printing
`ESC ] 0 ; PWNED BEL`: relayed raw. **Fixed:** each line the relay already assembles (CR LF folded) is shown
through `terminal_line`, for the operator and the probe alike; an ordinary line is unchanged; no prefix — the
console is the VM's only voice. Witness `microvm::tests::the_console_relay_shows_a_guests_control_sequences_escaped`
red on `ec4d39d`, green after; M20 (the line emitted raw) red, restored byte for byte.

**Verified:** clippy `-D warnings` clean; the full suite alone: 2,051 passed, 0 failed, 15 ignored (153 binaries), cargo exit 0. **The KVM job read on the branch before
`master` moved** (`ci.yml` dispatched at `1721539`, run `36604242542`): `microvm` success — the lifecycle tests, criterion 8, the probe from a boot and the hostile guests against the host, all on the image built in the job. The same run's `test (macos-latest)` failed on four tests only — `doctor_cli` three times and the Survey's freshness test — because the branch commit carried the code without the regenerated map; every other macOS test passed. The run was then cancelled (its Miri jobs run for hours); the routine now says to commit the map with a branch commit the whole CI will read.

**Routine run 5, closed.** `master` green on arrival (`cfc5b01`); the nightly on `9fc4d86` red on `supply-chain`
only, the advisories `63a375e` had fixed. Built: **PS-E-04** (D-V2-69) — the external launcher resolved once,
hashed, pinnable, and on Linux started as the file hashed; LAUNCHER-SPELL-1 found and closed. Found while scoping
RW 4.32: **TERMINAL-TEXT-1** (D-V2-70) — a pure program wrote escape sequences to the operator's terminal
through its own strings; escaped at every printer, one core-snapshot line re-recorded (DL0107's own quoted line
carried the raw U+202E it warns about). Then **RW 4.32's stream** (D-V2-71) — a guest's standard error relayed by
the host, escaped, marked, bounded; the microVM's console escaped; the death record's `guest_words` — and **RW
4.34** (D-V2-72), a line break that began a line of its own. Twenty mutants, M1–M20, each red — M16 only once the witness it had survived was written. `master` went red
once, on Windows, for a TEST whose path held `:` (`6cd68c8`), fixed in `d1dbf2e` and green from `41ac869`; the
lesson is in the routine — a slice's new tests are read on every runner before `master`. Not done: PS-E-03 H6,
whose start a safety classifier stopped before anything ran.

**`ec4d39d`'s push run, `36603987907` — red on one job, `arm64`, one test** (read at 17:36 UTC):
`estop_cli::with_nobody_revoking_anything_the_same_supervisor_keeps_its_arm` — the control run, which nobody
revokes, printed `REVOKED: the lease on arm0/elbow was revoked (operator-revoke)`. Not this run's code: `ec4d39d`
is records only over `30d1563`, whose change is a sandbox death record, and `estop_cli` runs no sandbox; the same
device and broker code passed on arm64 at `41ac869` and `d566295`. **Not root-caused:** the program's line says
`operator-revoke` for every probe that came back dead — revoked, expired, or a broker silent past DEADMAN-1's 1 s,
a collapse `device.rs`'s own unit test holds on purpose, the journal keeping the reason — and the test printed only
that line. Eight busy loops on the VM's four CPUs did not reproduce it (two runs, 5 passed each, 29 s against the
runner's 12 s). **Done:** the control test now prints the journal beside a failure, so the next occurrence names
its reason; RW 7.17 holds the question, and whether an unanswered probe deserves its own recorded cause.

**`fa69efb`'s push run, `36606231595` — success on every job (16): `master` green at `fa69efb`**, arm64's
`estop_cli` among them (RW 7.17 stays open until a failure's journal names its reason, or it is shown not to
recur). The run's close: `HANDOFF.md` §11.5 gains that a failure message is all the evidence a CI run keeps.

## 2026-09-30 — routine run 6: CI read; RW 4.32 closed — FRAME-DRIP-1, a frame dripped past every channel's deadline (D-V2-73)

**CI on arrival.** `dcf4fcb` (routine run 5's close) push run `36608843586` — success on every job (16).
macOS: level 1, egress established, filesystem confinement absent (reads not confined), host loss established,
privilege floor established, resource ceiling absent; ping-pong NOT MEASURED (3 hardware threads). Windows: all
properties as run 5 read them; ping-pong NOT MEASURED (the runner busy). No nightly since `36548984501` (recorded by
run 5). `gh`: absent. The VM's first `rustup target add`, started beside the first `cargo` call, raced the pinned
toolchain's install and broke it (`cargo` "not applicable"); reinstalled — the routine's step 2 now installs the
toolchain with its targets first.

**RW 4.32's last item, and the same hole twice more.** The sandbox host's channel deadline was on each READ: a
guest sending one byte just inside it held a frame open for ever. Asking where else a deadline is per read found two
more channels that read a peer DeluluLang does not trust — and on both the hole was worse:

- **The broker daemon** (`brokerd.rs`): one connection at a time, its read bounded at 5 s, and a comment promising
  that a client which "dribbles a partial frame" is dropped so it cannot deny "every other custody op, INCLUDING the
  operator's e-stop revoke". IPC-1's fix (2026-08-08) had recorded the indefinite hang closed; its witness was a client
  that stalled. **Witnessed on `dcf4fcb`**: a client sending one byte every 2 s held the loop — a `Status` behind it
  waited 23.7 s, the dribbler's whole life (macOS 24.25 s, `witness.yml` `36651091801`); on Windows the `Status` was
  refused instead (`connect: win32 error 231`, all pipe instances busy — a busy pipe refuses a second client after 1 s
  where a Unix socket queues it; `36651089599`).
- **A foreign call** (`foreign_worker.rs`): foreign code owns its worker's end of the channel; answering one byte
  every 300 ms, each inside the 1.5 s call deadline, it held the host past the witness's own 20 s watchdog (Linux;
  macOS 20.04 s, `36651091801`).

**Fixed (D-V2-73):** one reader, `delulu_runtime::channel::Within`, owes a frame WHOLE within a bound — from a
connection's acceptance for the broker (5 s), from the call for a foreign reply (the call's deadline; DL1409, the
worker killed), from a frame's first byte for the host reading a guest (`FRAME_DEADLINE`, 60 s, in `serve` and the
confinement report; a guest may still be quiet between frames). Checked around each read, so a frame is abandoned no
later than its bound plus one read deadline. A guest's slow frame ends the run in its own words, told apart from a
silent one. Commits: `626c3c5` (the two witnesses alone, read red on macOS and Windows), `12eeb51` (the fix and five
more witnesses), `11a152b` (the broker witness asks again, as an operator on Windows must).

**Witnesses:** `brokerd::tests::a_client_that_dribbles_its_request_cannot_hold_the_serve_loop`,
`foreign_worker::a_foreign_call_that_drips_its_reply_is_dl1409_within_the_deadline` (Unix),
`channel::tests::{a_frame_dripped_past_the_frame_deadline_is_refused, a_pause_before_a_frame_is_not_a_slow_frame,
from_now_counts_the_wait_before_the_first_byte}`, `boundary::tests::a_confinement_report_dripped_past_the_frame_deadline_does_not_confirm`,
`guest::tests::a_slow_frame_and_a_silence_are_told_apart`. **Mutants M21–M27**, each red and restored byte for byte:
M21 `Within` never refusing (all five witnesses red), M22 `serve` reading per read, M23 the broker reading per read
(re-run after `11a152b`: red, 23.7 s), M24 the foreign call reading per read, M25 `from_first_byte` counting from its
making, M26 the confirm read reading per read, M27 a slow frame worded as silence.

**Verified:** clippy `-D warnings` clean; `scripts/check-other-os.sh` clean on five targets (and again for Windows and
macOS after `11a152b`); the full suite alone (-j 4, the map regenerated first): 2,058 passed, 0 failed, 15 ignored
(153 binaries), cargo exit 0. **Read on the runners before `master` moved:** red at `626c3c5` — macOS `36651091801`,
Windows `36651089599`; green — macOS `36651763687` at `12eeb51` (`delulu` bin 138 passed, `foreign_worker` 5) and
`36651767914` (`delulu-runtime` `channel::` 22), Windows `36651765721` at `12eeb51` (`channel::` 22) and
`36652035525` at `11a152b` (`delulu` bin 132 passed — the broker dropped the dribbler 6 s after accepting it — and
`foreign_worker` 4). A Windows read at `12eeb51` (`36651761315`) was cancelled once `36651089599` showed its broker
witness could not pass there as written.

**Not changed, recorded:** the broker still serves one connection at a time, so clients queued behind one another each
take their bound; on Windows a client that finds the pipe busy is refused after 1 s rather than queued; the operator's
`request` still waits without a bound for a broker that accepted and never answered (the dead-man probe's does not).

## 2026-09-30 — routine run 6: PS-E-04 on Windows — the launcher held open, sharing reads only, until the run ends (D-V2-74)

E-04's Windows remainder, from the inbox. D-V2-69 left Windows starting the resolved launcher by name — "Windows could
hold the file open deny-write, but no witness for that exists yet, so it is not built and not claimed". The witness,
`the_launcher_started_is_the_file_that_was_hashed_on_windows` (`sandbox_external_cli`, Windows): A is the binary plus
64 MiB its loader never reads, pinned; B is a copy of `hostname.exe`, not a guest; the moment Windows says the host
holds A open, B is renamed over it, and again for as long as the host holds it. "The host holds A" is asked of Windows
itself — `NtQueryInformationFile(FileProcessIdsUsingFileInformation)` through a handle that reads attributes only and
so takes part in no sharing check — the Windows view the Linux witness gets from `/proc/<pid>/fd`; never by re-running
until a race shows.

**Red on `34cfc38`** (`witness.yml` `36652740569`, Windows): the host hashed A through the standard library's
`File::open`, which shares reading, writing AND deletion; the rename succeeded while it hashed; the pin passed on A's
bytes; B was started under A's name — "the guest never confirmed its boundary, so it was not sent the program: it closed
the channel first", exit 1. **Fixed (`8aa6249`, D-V2-74):** `Launcher::resolve` opens the file sharing reads only, and
the `Launcher` lives until the run's report is written, so nobody can write, rename or delete it from the hash to the
end of the run; starting it is a read, still shared. **Green on `8aa6249`** (`36653274720`, Windows):
`sandbox_external_cli` 5 passed (the swap witness among them), `sandbox_confirm_cli` 8, `sandbox_attest_cli` 4,
`sandbox_run_cli` 17. The red read is the falsification: the same witness against the code before the fix. macOS keeps
its window (no `fexecve`, no share modes), and `DEPLOYMENT.md` says so.

**Verified in the VM:** clippy `-D warnings` clean; `scripts/check-other-os.sh` clean for `x86_64-pc-windows-msvc` and
`x86_64-apple-darwin`; `sandbox_external_cli` 8 passed on Linux (the change and its witness are Windows-only).

## 2026-09-30 — routine run 6: the red-team pass on FRAME-DRIP-1; REQUEST-HANG-1 and PROBE-DRIP-1 (D-V2-75); BROKER-RELDIR-1

**The pass.** One Sonnet 5.5 sous-chef, against a frozen copy of the binary at `11a152b` in the scratch directory,
briefed to break D-V2-73 and to list every oddity, with the evidence of any absence (it examined about 300 read sites
across every crate of the workspace, all 34 non-test `read_frame` callers). It wrote nothing into the repository (`git status`
empty, re-checked). **The guarantee held** on all three channels (the broker dropped dribblers at 6.0 and 9.0 s and a
silent client at 5.1 s; a dribbled guest frame ended at 100.3 s — 60 s plus one 50 s gap, as stated; a dribbled
foreign reply at 5.16 s against a 5 s deadline). **Its findings, each re-run by the head chef before use:**

- **PROBE-DRIP-1 (its F1)** — the dead-man probe's `request_timed` bounded each READ; D-V2-73 had called it bounded.
  Re-run on `9bf09b9`: an answer dribbled a byte every 250 ms held a 1 s probe 6.0 s. The pass had run it end to end:
  an e-stop printed "revoked" while the arm, its probe fed that answer through a proxy on the broker's socket, kept
  moving (a same-user process on the socket — category 7 — but the record was wrong, and the fix is one line).
- **REQUEST-HANG-1** (the head chef's, from D-V2-73's own "not changed" line) — `request`, which every daemon-mode
  custody op and every operator command uses, had no bound: 40.2 s against a broker that accepted and never answered.
  **Fixed together (`68b8888`, D-V2-75):** `request` is `request_timed` with 15 s, and `request_timed` reads the whole
  answer through `Within::from_now`; past it, "the broker accepted the request but did not answer within 15s".
  Witnesses `request_fails_closed_in_words_on_a_broker_that_accepts_but_never_answers` and
  `request_timed_bounds_the_whole_answer_not_each_read`; mutants M28–M30 red.
- **BROKER-RELDIR-1** (its oddity) — `broker start` with a relative state dir: the daemon, spawned with the state dir as
  its working directory and the same relative words, served at `st/st`; the caller said "did not come up within 5s" and
  left it running. Re-run on `68b8888` (the head chef's own experiment met it first: a relative `DELULU_STATE_DIR`).
  **Fixed (`3ca49e4`):** made absolute at the edge. Witness `broker_start_with_a_relative_state_dir_serves_where_the_caller_looks`,
  red on `68b8888`, green after.
- Recorded, not fixed this run (RW 4.35–4.40): the broker's reply write has no bound (F2, re-run: a ~250 KB answer
  left unread held `broker status` past 8 s); `delulu-registry serve` — one idle TCP connection stalls every client, and
  a huge `Content-Length` crashes it (F3, re-run on the current registry binary); the host's writes to a guest on a
  socket channel have no deadline (F5, code reading only); a FIFO in the audit directory hangs `audit verify` and
  `audit tail` (F6, re-run: killed at 12 s); the broker wire's lax frame reader, the LSP's and MCP's unbounded sizes;
  the queue of silent clients (F4, 12 of them: 61.5 s).

**Verified:** clippy `-D warnings` clean; `check-other-os.sh` clean for Windows and macOS after each change; the full
suite alone at `68b8888`: 2,060 passed, 0 failed, 15 ignored (153 binaries); at `3ca49e4`: 2,061 passed, 0 failed,
15 ignored, cargo exit 0. **Read on the runners before `master` moved:** `68b8888` — macOS `36655171914` (`delulu` bin
140, both new witnesses among them; `broker_cli` 3, `dead_man_cli` 16, `estop_cli` 5), Windows `36655174168` (bin 132,
`broker_cli` 2, `dead_man_cli` 16, `estop_cli` 5); `3ca49e4` — `broker_cli` success on Windows `36655636658` and macOS
`36655638846`.

## 2026-09-30 — routine run 6: REGISTRY-BOUNDS-1 — `delulu-registry serve` bounds every connection (D-V2-76)

RW 4.36, from the red-team pass's F3, re-run by the head chef on the current registry binary first (one idle TCP
connection stalled a GET past 5 s; `Content-Length: 18446744073709551615` crashed the server). The registry's port is
reachable by whoever can reach it, and reading it needs no token. **Witnessed on `76aa676`** (`crates/delulu-registry/tests/serve_bounds.rs`):
a client behind one idle connection got no answer in 3.2 s; the huge `Content-Length` panicked the server
(`raw_vec` capacity overflow) and the next connect was refused; a 1 MiB request line with no end was read on and never
answered. **Fixed (`5db4df1`, D-V2-76):** `Limits` (10 s a read, 60 s a whole request through `Within::from_now`, 8 KiB a
line, 64 headers, a 16 MiB body, 64 connections) and `serve_with`; a thread per connection, the requests still handled one
at a time under one lock (a publish rewrites a package's index); a panic kept on its connection's thread; 413, 431 or 400
past a bound, with DL1706. A fourth witness drips a request past a 1 s bound (dropped at about 1.2 s). **Mutants M31–M34**
(no body cap, one connection at a time, lines unbounded, per read only) each red on its own witness, restored byte for
byte. The line witness first failed on the fix itself — the server's close with the client's bytes still unread reaches
the client as a reset, which can cut the 431 short — so it accepts the refusal or the reset; the old server did neither.

**Verified:** clippy `-D warnings` clean; the full suite alone: 2,065 passed, 0 failed, 15 ignored (154 binaries), cargo
exit 0. Read on the runners before `master` moved (`witness.yml`, `delulu-registry`, every target): Windows
`36657264839` (`serve_bounds` 4 passed, `criterion5_e2e` 3, lib 20) and macOS `36657267414`, success.

## 2026-09-30 — routine run 6: AUDIT-FIFO-1 — an audit day log is read only if it is a regular file (D-V2-77); the run's close

RW 4.38, the red-team pass's F6, re-run first (`audit verify` and `audit tail` held until killed at 12 s). **Witnessed
in-crate on `2734ba9`**: with a FIFO named `20260930.jsonl`, `verify` was still waiting at 5 s. **Fixed (`cc3857b`,
D-V2-77):** `read_day` opens non-blocking, refuses what it opened unless it is a regular file, in words, for every
reader and the log's open (which no longer reads such a file as empty); `libc` on Unix for `delulu-broker` (already in
the tree); the flag left out under Miri. Witness `audit::tests::a_day_log_that_is_not_a_regular_file_is_refused_not_waited_on`
(verify, tail, open); M35–M37 red. **Verified:** clippy clean; `check-other-os.sh` clean for Windows, macOS and musl; the
full suite alone 2,066 passed, 0 failed, 15 ignored (154 binaries), cargo exit 0 — its first attempt died on ENOSPC (the
VM's disk full of cross-target builds and incremental caches: 3.6 GB and 6.8 GB, deleted). Read on the runners first
(`delulu-broker` lib): macOS `36658766644 (170 passed, the witness among them)`, Windows `36658768992 (169 passed; the witness is Unix-only)`.

**Routine run 6, closed.** CI on arrival green (`dcf4fcb`). Built: **RW 4.32's last item and FRAME-DRIP-1** (D-V2-73 — a
peer's frame owed whole on the broker, a foreign call and the sandbox host); **PS-E-04 on Windows** (D-V2-74 — the
launcher held sharing reads only; red then green on a Windows runner). A red-team pass (Sonnet 5.5) on FRAME-DRIP-1 held
its guarantee and found **PROBE-DRIP-1** (closed with **REQUEST-HANG-1**, D-V2-75), **BROKER-RELDIR-1** (closed), the
registry's unbounded port (**REGISTRY-BOUNDS-1**, D-V2-76) and a FIFO in the audit directory (**AUDIT-FIFO-1**, D-V2-77);
the rest are RW 4.35, 4.37, 4.39, 4.40. Mutants M21–M37, each red. Every slice read on macOS and Windows before
`master` moved; every push run of the run read green up to ``2734ba9``.

**After the close, one more slice (RW 4.39's first half):** `broker_ipc::read_frame` — the broker daemon's wire, and the
foreign worker's — decoded a frame's value and ignored whatever followed it inside the frame's length; the red team's
`Status` with 70 more bytes was answered. **Witnessed on `cc157bc`** (`broker_ipc::tests::a_frame_with_bytes_after_its_value_is_refused`:
the padded frame read as `Status`); refused now in words, as the sandbox channel's reader has since F12. The red run on
`cc157bc` is the falsification. **Verified:** clippy clean; `broker_ipc` 5, `foreign_worker` 4, `broker_cli` 5 passed;
the full suite alone 2,067 passed, 0 failed, 15 ignored (154 binaries), cargo exit 0; read on the runners first (`delulu` bin, `broker_cli`, `foreign_worker`): macOS
`36659893368 (bin 141, the witness among them; broker_cli 4, foreign_worker 5)`, Windows `36659895529 (bin 133; broker_cli 3, foreign_worker 4)`.

## 2026-09-30 — routine run 7: CI read green; PS-E-06 — the audit chain exported as OCSF 1.8.0, still verifiable (D-V2-78)

**CI, read first.** Routine run 6's last two push runs, unread when it ended: `cc157bc` `36659209716` and `7940188`
`36660801170` — success on every job. No nightly since `36548984501` (the 2026-09-30 schedule had not fired by 05:20
UTC). `gh`: absent. `cargo deny --all-features check advisories`: ok. Survey `check` ok (1,472 nodes, 12,976 edges);
`doctor --check` ok, 26 checks passed.

**PS-E-06, built.** `delulu_broker::ocsf` maps each audit record to one OCSF 1.8.0 event; `delulu audit export
--format ocsf [--since SEQ] [--out F] [--device-name NAME]` writes them as JSON Lines, and `delulu audit verify --ocsf F
[--expect-start HASH]` re-verifies an export from the file alone. Each event carries its **whole** record under
`unmapped.delulu` — the study's three chain fields would show a removed line but not an edited one — and every OCSF
field is a pure function of that record and two labels, so the verifier recomputes each event and requires the line to
be exactly it, then re-verifies the chain with `Bundle::verify`. Class UIDs checked against the published schema
files, read in the VM: Detection Finding 2004 (a refusal; break-glass and a bypassed Guard, High), User Access
Management 3005 (grants and revocations), Process Activity 1007 (a guest's launch and end, paired by the run's
generation), Base Event 0 (the rest). **A capability use is a Base Event**: the chain records a use's argument, not its
effect (`validate.rs` drops the op), and the export does not guess a class from a spelling. D-V2-78.

**The OCSF schema IS reachable from the VM** — run 6's entry said it was not. `raw.githubusercontent.com` serves
`github.com/ocsf/ocsf-schema` at `v1.8.0` file by file (the tarball is refused, 403; `schema.ocsf.io` is refused by the
proxy). `scripts/ocsf-validate.py` reads the class, object, profile and dictionary files at run time — nothing of OCSF is
committed — and checks every event: the class's uid and category, `type_uid`, required attributes through `extends` and
`$include`, no undefined attribute (recursively), each object's constraints, types and enums. Its first run failed 69
times on a correct export — the validator's own three errors (a class's category is named by its category-level parent;
a profile's attributes are optional unless the event names the profile; an `_id`'s enum is the dictionary's merged with
the class's) — fixed in the validator, then **falsified**: thirteen mutations of an export (a required uid removed, a
severity out of its enum, a misspelt attribute, a user with no identity, privileges not an array, a wrong `type_uid`, a
missing process or device, a device type out of its enum, a class claimed as another, a wrong category, the wrong
schema version, a time as a string), each caught.

**A fake peer kept the wrong protocol — again (§11.5).** The first fixture wrote each sandbox record's generation as an
integer; the host writes 64 hex characters. The mapping read an integer, and a real sandboxed run's chain showed every
guest named by its record's hash, its launch and death unpaired. Fixed to read the host's spelling, and a seventh
witness runs real sandboxed programs rather than trusting the fixture.

**Witnesses** (`crates/delulu/tests/audit_ocsf_cli.rs`, 7): every record kind to its class, `type_uid`, category and
the base event's required fields, and the export's head equal to `audit verify`'s; a removed middle event fails
(DL1405), a removed first one only against `--expect-start`; an edited OCSF field, a consistently forged record, a
relabelled line — each fails; a secret's bytes appear in no export (its name does); `--since` verifies from the head
before it and from nothing else; a broken chain is not exported, and every flag is checked; a real run's refused read
and clean exit, launch and death paired by generation. **Mutants M38–M43, each red:** the export drops the embedded
record (5 tests red); the verifier never recomputes the event (1); it chains only the first record (3); it ignores the
labels (1); a refusal is not a finding (3); a refused guest's end is a finding (2).

**Verified:** clippy clean; `check-other-os.sh` clean for macOS (arm64) and Windows (the host-name lookup is per OS);
the full suite alone 2,074 passed, 0 failed, 15 ignored (155 binaries), cargo exit 0; the export of a real sandboxed chain and of the corpus validated against OCSF 1.8.0, 0 problems.

## 2026-09-30 — routine run 7: AUDIT-TEXT-1 — the investigator's and the approver's screens printed an adversary's text raw (D-V2-79)

Found reading `render_audit_record` while building PS-E-06: a record's `target` printed as stored, and a use's target is
the path the PROGRAM chose. **Witnessed on `b50bb36`, end to end in the VM:** a broker daemon, a lease for `fs.write=./out`,
and a program writing `./out/ESC]0;PWNED BEL ESC[2J ESC[31mFORGED.txt` — inside its grant, an ordinary allowed use; `delulu
audit tail | cat -v` showed the raw `^[]0;PWNED^G^[[2J^[[31m` (on a terminal: the title set, the screen cleared, the text
red). Reading on, `guard pending` printed each request's `why` — the requesting agent's own text — raw too. Two witnesses,
each red on the unfixed binary: `audit_cli::a_programs_string_in_a_record_is_shown_escaped_never_obeyed` (a target holding
ESC, BEL, U+202E and a line break followed by a forged `seq 9 … revoke allow`: the raw bytes reached stdout and the forged
record printed as a line of its own) and `guard_e2e::guard_pending_shows_an_agents_why_escaped_never_obeyed` (a `why`
of `ESC[2K CR ESC[1A` and a line break, then a forged request for `fs_read:./data` — on a terminal the owner would have
seen that request, not the real `fs_write:*`). **Fixed (D-V2-79):** every field of both listings is shown with
`terminal_line`; `--json` is exact, and asserted so. **Mutants M45** (the target unescaped) **and M46** (the why unescaped),
each red; restored. The leased-program witness re-run on the fixed binary: `\u{1b}]0;PWNED\u{7}…` shown, no raw byte.
`grants tree`/`list`/`inspect` print broker-rendered text an agent may partly choose — recorded as RW 4.42, a hypothesis to
witness. **Verified:** clippy clean; the full suite alone 2,076 passed, 0 failed, 15 ignored (155 binaries), cargo exit 0.

## 2026-09-30 — routine run 7: PS-E-06 complete — a use's record names its effect, and the export maps it (D-V2-80)

D-V2-78 exported a capability use as a Base Event because its record named its argument and not its effect
(`check_use` dropped the op). **Witness (the state before, as mutant M47 restores it):** a new broker test,
`audit_wiring::every_use_record_names_its_effect` — an allowed and a refused write, an allowed and a refused network use,
a use asked of a node that does not exist — found no effect in any of the five records. **Built:** every record
`check_use` writes carries `{"op": <wire name>}` in the payload slot a revocation's bound already uses (spec §11, chunk-5
deviation 3, now noting this) — hashed, optional, so every chain verifies under an older or a newer `delulu`; the export
maps an allowed `FsRead`/`FsWrite` use to File System Activity 1001 (Read/Update, the path and its name) and `Net` to HTTP
Activity 4002 (Get, the host as `dst_endpoint.hostname`), and a refused use's finding names the effect in its `types`.
A record written before names none and stays a Base Event — the corpus now holds one, and the witness holds it so.
**Mutants:** M47 (no effect recorded) red in both targets; M48 (the export ignores a named effect) and M49 (a refused
use's finding without it) red in `audit_ocsf_cli`; restored. **The schema:** the new corpus (13 records) validated
against OCSF 1.8.0 with 0 problems, and five more mutations — a file with no name, no file, a misspelt `http_request`
field, a numeric host, an activity out of the file class's enum — each caught. **Verified:** clippy clean; the full suite alone 2,077 passed, 0 failed, 15 ignored (155 binaries), cargo exit 0.
**PS-E-06 is complete.**

## 2026-09-30 — routine run 7: RW 4.38's remainder — the secret store, read and written only if it is a regular file

D-V2-77 closed AUDIT-FIFO-1 and named `secrets.rs` as read the same way, not tried. **Tried, on `3d84304`:** the new
witness `secrets::tests::a_store_that_is_not_a_regular_file_is_refused_not_waited_on` replaces a real store with a FIFO
and found `SecretStore::load` still waiting at 5 s. Reading on: `refresh` — which the broker daemon runs before EVERY
secret operation, on its one-connection serve loop — read the same way, so a FIFO there held every client behind it, the
e-stop's revoke included; and `set` opened the path for writing, blocking, so it waited for a reader — and with a reader
attached the write-open succeeds and the store's secrets go to whoever is reading. **Fixed (D-V2-77's rule, shared
rather than copied):** `audit::read_day` became `read_regular(path, what)`, the store's one reader; a store that is not
a regular file loads as none and a refresh keeps what was loaded; the Unix write opens non-blocking, maps ENXIO (a FIFO
with no reader) to the same refusal in words, judges what it opened and only then truncates. The witness covers the
reader's case too (nothing reaches the reader). **Mutants M50** (the blocking read), **M51** (the blocking write-open),
**M52** (the write without the check), each red; restored. **Verified:** clippy clean; the full suite alone 2,078 passed, 0 failed, 15 ignored (155 binaries), cargo exit 0; read before `master` moved at `6c7aca3`: macOS `36679046053` (`delulu-broker` lib 171, the witness among them) and `36679062748` (`secret_verify_cli`, `broker_cli`), Windows `36679048557` (lib; the witness is Unix-only) — success.

## 2026-09-30 — routine run 7: RW 4.42 — `grants tree`, `list` and `inspect` printed a delegation's strings raw

AUDIT-TEXT-1's next surface, recorded as a hypothesis in `9c58cbb`. **Witnessed on `634e9f0`** in the VM and in the new
`grants_cli::a_delegations_strings_are_shown_escaped_in_tree_list_and_inspect`: a real broker daemon, a delegation whose
`fs.write` path carries `ESC]0;PWNED BEL ESC[2J`, a line break and `g_forged [live] effects={Read,Write,Net}`, and whose
`net` host carries `ESC[31m` — `grants tree` and `grants list` printed four raw escapes each, and the forged node line
began a line of its own (`inspect` the same). **Fixed (D-V2-79's rule):** `Broker::tree` builds one line per node, so
each field is escaped where the line is built; the CLI escapes its list line and each of inspect's fields; `--json`
stays exact, and the witness asserts it. **Mutants M53** (the tree's authority), **M54** (the list's), **M55**
(inspect's) unescaped, each red; restored. **Verified:** clippy clean; the full suite alone 2,079 passed, 0 failed, 15 ignored (155 binaries), cargo exit 0; read before `master` moved at `297f976` (`grants_cli guard_e2e broker_cli`): macOS `36680482439` and Windows `36680485122` — success.

**Routine run 7, closed.** CI on arrival green (`cc157bc`, `7940188`). Built: **PS-E-06, complete** — the audit chain as
OCSF 1.8.0, each event carrying its record (D-V2-78), a use's record naming its effect (D-V2-80), the schema read at run
time in the VM and on a runner (`ocsf.yml`, read green by hand, then on push). Found and closed: **AUDIT-TEXT-1**
(D-V2-79) — `audit tail`/`query`, `guard pending`, then `grants tree`/`list`/`inspect` (RW 4.42) printed a program's or an
agent's stored strings raw; **RW 4.38's remainder** — the secret store read and written the FIFO way. Mutants M38–M55,
each red. Every slice read on macOS and Windows before `master` moved; push runs read green up to `9c58cbb`. Loop
engineering: the routine's step 5 gains the OCSF check, a real-run witness beside every fixture that fakes the host, and
overlapping a slice's runner read with its suite; HANDOFF §11.3, §11.4, §11.5.

## 2026-09-30 — routine run 7, after the close: RW 4.39's rest — `delulu lsp` and `delulu mcp` bound one message

From the red-team pass on FRAME-DRIP-1. **Witnessed on `eff5367`:** `lsp_cli::a_content_length_past_the_bound_is_refused_in_words_not_allocated`
— `Content-Length: 18446744073709551615` panicked the server (`capacity overflow`, exit 101), because `read_message`
allocated `vec![0; n]` before reading a byte; `mcp_cli::a_request_line_past_the_bound_is_refused_and_the_session_goes_on`
— a 17 MiB line was read whole (`lines()`) and answered "not JSON". **Fixed:** the LSP reads what arrives (`take(n)`,
`read_to_end`), bounds a message at 64 MiB and a header line at 8 KiB, and past either ends the session in words — a
stream whose framing cannot be trusted cannot be resynchronised; the MCP server reads at most 16 MiB of a line, skips
the rest without holding it, answers -32600 naming the bound, and serves the next request. **Mutants M56** (the LSP's
bound removed — no panic now, since nothing is pre-allocated, but no refusal in words either) **and M57** (the MCP's),
each red; restored. **Verified:** clippy clean; the full suite alone 2,081 passed, 0 failed, 15 ignored (155 binaries), cargo exit 0; read before `master` moved at `dca734a` (`lsp_cli mcp_cli`): macOS `36681858200` and Windows `36681860992` — success. RW 4.39 keeps one item: the broker log's three
wordings of one class of drop.

## 2026-09-30 — routine run 8: CI read green; SCOPE-HIDDEN-1 — `authority --grants` under-reported a program's scopes (D-V2-81)

**Arrival (routine run 8, 10:09 UTC).** `survey check` ok (1,476 nodes, 13,071 edges); `doctor --check` ok, 26 checks
passed. `gh`: absent (`command -v gh` printed nothing). **CI:** routine run 7's three unread push runs — `5d3e37f`
`36683068019`, `53d96e3` `36683327300`, `381fed8` `36684530677` — success; the nightly `36694905248` on `381fed8` was in
progress at 10:12 (its `miri-slow` runs for hours). Run 7's new witnesses re-run in the VM on `381fed8`: `audit_ocsf_cli`
7, `audit_cli` 12, `guard_e2e` 3, `grants_cli` 2, `lsp_cli` 38 (1 ignored), `mcp_cli` 3, `delulu-broker` lib — all
passed. Every claim in run 7's entry that the tree can show holds.

**SCOPE-HIDDEN-1, found building PS-E-05.** The export needs to know whether a program's literal scopes are all it asks
for, so the head chef asked `authority` of a program whose `main` mints `root.fs_read("./data")` and passes `root` to
`fn helper(r: Root)`, which mints `r.fs_read("./secret")`. `authority --grants` printed `--grant fs.read=./data` alone;
`delulu run … --grant fs.read=./data` then faulted DL0703 on `./secret` — the runtime held, the list an operator reads
before running unfamiliar code did not. The scope walk matched the receiver `root` by NAME and silently skipped every
other; and a computed scope beside a literal one was likewise dropped, because the placeholder was printed only when a
kind had NO literal. **Witnessed red** (`cli::a_scope_the_source_does_not_show_leaves_its_placeholder_beside_the_literals`
on the unfixed binary: `["fs.read=./data"]` where `["fs.read=./data", "fs.read=PATH"]` is owed). **Fixed (D-V2-81):** the
walk marks a kind hidden at a computed scope or a renamed receiver, and `required_grants` adds the kind's placeholder
beside the literals — for paths, hosts and devices; the renamed receiver's literal is not attributed. **Mutants M58**
(the renamed receiver skipped again), **M59** (a computed item in a host list not marked), **M60** (the placeholder only
when no literal) — each red on its case; **M61** (the placeholder always) red on the all-literal control; restored byte
for byte. **Verified:** clippy clean (`-p delulu --all-targets`); `cli` 50, `atlas_chain`, `examples_run`, `mcp_cli`,
`sandbox_modes_cli`, `sandbox_run_cli`, `atlas_cli`, `atlas_e2e`, `core_invariance`, `attributes_cli` green; no program of
the repository's 251 `.delulu` files has a mixed kind, so none's report changes; the full suite alone 2,082 passed,
0 failed, 15 ignored (155 binaries), cargo exit 0.

## 2026-09-30 — routine run 8: PS-E-05 (a) — `delulu sandbox policy --format openshell` (D-V2-82)

The study's §4.5 (a): DeluluLang writes the policy an OpenShell sandbox should enforce around `delulu run`, from the
program's authority and its grants. **Read first, at run time, for facts only** (nothing copied, D-V2-52): OpenShell's
policy schema, prover and default-policy pages and its `install.sh` at `36b0386` from raw.githubusercontent.com — the
format is `version: 1` and five sections, unknown fields refused; the prover compares paths only when both policies
list the SAME path, answers `unsupported` for an exact and a wildcard host on one port, and exits 0/1/2/3; OpenShell adds
baseline paths (`/tmp` read-write among them) to a policy with a network rule unless the policy lists them; the Debian
package carries `openshell-prover`, checked against the release's SHA-256 list. **Witnessed absent** on `290a741`:
`sandbox policy p.delulu --format openshell --grant …` → "does not know this option", exit 2.

**Built (D-V2-82):** `crates/delulu/src/openshell.rs` — `export(program, report, grants, limits, options)`, pure over
the program's authority report (`cli::authority_of_text`, the pipeline `authority` uses) and the grants (the runtime's
own `Grants::add` first); `sandbox policy … --format openshell [--grant K[=V]].. [--workdir DIR] [--binary PATH]
[--run-as ID[:ID]] [--json]`; the `openshell` schema. The nine decisions are D-V2-82's. **Found on the way:**
SCOPE-HIDDEN-1 (D-V2-81) — which is why the export emits a grant's scope, not the source's literal.

**Witnesses.** `sandbox_openshell_cli` (5): the whole wall for a program that reads, writes and fetches from an exact and
a wildcard host — the exact filesystem lists, `GET` on `/` and `/**` and nothing else, no preset, `enforce`, one binary,
`**.cdn.example.org`, every grant accounted for, the YAML stating what the JSON states, valid against the published
schema through the binary; a grant the program cannot use omitted and named; five grants refusing the export by name
(exit 1, no document); seven malformed requests (exit 2) and the shaping flags refused without the format; a secret's
value nowhere in the output and a hostile path quoted whole. Unit (5): **the soundness rule over every shipped
example** (each with its own required grants plus a grant of each kind it may not use: every path on the wall is a
runtime path, the program's file, or a grant of a kind the program uses; every endpoint a grant's host; every rule
`GET`; every grant accounted for), paths, hosts, quoting, `--run-as`. In the VM, PyYAML reads the document as exactly the
JSON policy, a path holding `"`, `:`, `[`, `#` included. **Mutants M62** (every kind treated as used), **M63**
(`net.special` emitted), **M64** (a relative path resolved against this machine), **M65** (`..` collapsed), **M66** (a
quote unescaped), **M67** (a `HEAD` rule in the text), **M68** (a secret's value in the account), **M69** (`fs.write=/`
allowed), **M70** (`*.` for `**.`) red; **M67** passed the unit gate — the document and the JSON were two code paths — so
the document is now written from the JSON (D-V2-82 (9)), and **M67b** (a `HEAD` rule in the policy) and **M71** (a host
unquoted) are red; every mutant restored byte for byte. **The prover script** (`scripts/openshell-prove.sh`) was run in
the VM against a crude stand-in prover (scratch, not evidence) to debug its control flow: every expectation reached. The
stand-in's first run found the export's host unquoted — the binary was the one the mutant loop had last built (M71): the
source was restored, the binary was not (a lesson, HANDOFF §11.5).

**Read on the runners, and what the loop found.** `15f68bf` (witness.yml: `sandbox_openshell_cli schema_cli cli_contract
bin:delulu`): macOS `36704827508` success; **Windows `36704830856` red on two TESTS, the product holding** — §11.5's
lesson again: the quoting case's path held `:`, which Windows' grant parser refuses first as an alternate data stream,
and the absolute-path case used this machine's path, `C:\…` on Windows, which is no path in a (Linux) sandbox — refused,
but called "relative". Fixed in `8c7894e`: a Windows spelling (a backslash, a drive letter) is refused by name, and the
witnesses' inputs follow each host's rules; **M72** (the check removed) red; read green on Windows `36705354867`, macOS
`36705357824`, arm64 `36705360430`. **The full suite alone at `8c7894e`: 2,091 passed, 1 FAILED, 15 ignored (156
binaries), cargo exit 101** — `sandbox_attest_cli` expected the dispatcher's words for `sandbox attest --grant`; with
`--grant` now documented for `sandbox`, `attest` refuses it in its own (exit 2 either way). Asking what else the
dispatcher now let by found a **regression on the branch, never on `master`**: `sandbox status --grant console` and
`sandbox probe --workdir /w` (and `--format`, `--binary`, `--run-as`) ran and IGNORED the flag, exit 0, where `290a741`
refused each. Fixed in `b50aa21`: the export's five flags are refused on every verb but `policy`; witness
`the_exports_flags_belong_to_policy_alone` (two verbs × five flags); **M73** (the check removed) red; read green on Windows
`36706294573` and macOS `36706297664`; the full suite alone at `b50aa21`: 2,093 passed, 0 failed, 15 ignored (156
binaries), cargo exit 0. **Then an adversarial pass by the head chef, before `master` moved — EXPORT-CASE-1:** asking of
every string the export compares what else spells it. DeluluLang compares a granted host exactly with a URL's host and
accepts a URL's host only in canonical form, so `net=EXAMPLE.com` and `net=example.com.` grant nothing (`delulu run` of
`root.http(["example.com"])` under the first: DL0703); the export lowercased the first and passed the second through,
and OpenShell matches hosts case-insensitively — a wall wider than the grant. Witnessed on `b50aa21`
(`a_host_grant_delulu_would_never_match_is_omitted_not_lowercased`: the wall held `example.com` and `example.org.` beside
the one effective host); fixed in `7dfb566` — a host `egress::parse_target` would not accept as written is omitted and
named; an IPv6 host refuses the export; **M74** (the check always true) red. **Verified:** clippy
clean; the full suite alone at `7dfb566`: 2,094 passed, 0 failed, 15 ignored (156 binaries), cargo exit 0; read before `master` moved at `7dfb566` (`sandbox_openshell_cli sandbox_attest_cli bin:delulu`): Windows `36707268456`, macOS `36707271273` — success; `master` fast-forwarded to `7dfb566`, and `openshell.yml` dispatched at `master` (a workflow is dispatchable only once on the default branch) — its reading is the entry after this one.

## 2026-09-30 — routine run 8: PS-E-05 (a) read by OpenShell's own prover (`openshell.yml`); (b)'s first attempt

**`openshell.yml` run 1, `36708161562` at `7dfb566` (by hand).** The pinned release's Debian package
(`openshell_0.1.2-1_amd64.deb`, 74.5 MB; its SHA-256 matched the release's list) installed `openshell-prover 0.1.2`, and
the prover answered every check as the study said it should: **the export against a boundary written by hand —
`within_boundary`, exit 0**, coverage `filesystem, network_l4, network_rest, process, landlock` — so OpenShell's
fail-closed parser took the document as its own format and its SMT model agreed with DeluluLang's derivation; a boundary
without `data.example.net` — `exceeds_boundary`, that host the counterexample; the `read-only` preset in place of the
written-out `GET` — `exceeds_boundary` (`OPTIONS`); another binary — `exceeds_boundary` (`/usr/bin/curl`); an unknown
field — `error`, exit 2 ("unknown field 'landlock.delulu_extra'"); the wildcard program within its boundary, and
`exceeds_boundary` (`a.cdn.example.org`) against one without it. **The run was red on the SCRIPT, not the export:** it
parsed the prover's stdout and stderr together, and the solver's warnings on stderr made six JSON answers "unparsed";
and an added `/tmp` came back `unsupported` (`unresolved_filesystem_path`) — the prover compares only paths both policies
name, as its documentation says, so that falsifier could never have been red. Fixed in `d447b29` (the JSON from stdout
alone; the filesystem falsifier makes a granted read writable). **Run 2:** `36708633558` at `d447b29` — every answer parsed and each as expected but one: the read made writable came back `unsupported` again (the boundary grants write access elsewhere, and the prover compares only paths both policies name); `a3b2c90` replaced it with the prover's documented `exceeds` cases. **Run 3, `36709035019` at `a3b2c90`: the prover job GREEN — every expectation held:** both exports `within_boundary`; `exceeds_boundary` for a missing host (`data.example.net`), the preset (`OPTIONS`), Landlock `best_effort` (`landlock`), another binary (`/usr/bin/curl`), a write where the boundary grants none (`filesystem write /sandbox/b.delulu`), a missing wildcard host (`a.cdn.example.org`); `error` for an unknown field. **PS-E-05 (a)'s falsifier is read green.** Runs 4 (`36709806472`) and 5 kept the prover job green.

**(b), first attempt.** `scripts/openshell-install.sh` (the install, shared by both jobs) and `scripts/openshell-runtime.sh`
(a `runtime` job, dispatched with `runtime: true`): a local gateway (the package's systemd user service), an image with a
non-root user, `/sandbox`, `iproute2`, `curl` and this build's `delulu`, a sandbox created under exactly the exported
policy, then (1) the granted program through `sandbox exec`, (2) a greedy one refused by DeluluLang, (3) `curl` to an
unlisted host denied by OpenShell, (4) the effective policy — OpenShell's baseline additions — checked against the export
by the prover. **Its reading:** six attempts, each reading the next one's fix. Run 3 (`36709035019`): the gateway's systemd user service started beside Docker 28.0.4, but `openshell status` succeeds with NO gateway registered, so none was added — and the service's first act was `podman info` (unset, the gateway prefers Podman to Docker). Run 4 (`36709806472`): the config this script wrote lacked `[openshell] version = 2` (schema version 2 is required; the service's preflight refused it some twenty times), and `gateway add --local` needs the client TLS the service provisions once started. Run 5 (`36710715856`): the gateway came up (mTLS, 0.1.2, "Connected"), **OpenShell created the sandbox under exactly the exported policy** (phase `Ready`), **`curl` to `example.org` inside it was refused** ("Couldn't connect to server", exit 7), and **the sandbox's effective policy equals the export in both directions** (the prover: `within_boundary` each way — with no network rule OpenShell adds no baseline); `delulu` itself would not start: the default build links the embedded Python. Run 6 (`36711475163`): built as the `Dockerfile` builds it, it needed glibc 2.39 where Debian bookworm has 2.36. Run 7 (`36712236680`), on Ubuntu 24.04's image: **`delulu` ran inside the OpenShell sandbox under the exported policy**; its read printed "unreadable" — the WITNESS's mistake (a capability's paths are relative to its scope: `read_text("./data/in.txt")` under `fs_read("./data")` names `./data/data/in.txt`; checked in the VM); **a program file the export never names was refused by OpenShell's Landlock** ("permission denied"); OpenShell's own log showed the policy fetched, the isolation boundary "attached" and "enforcement confirmed", TLS termination with an ephemeral CA, and the policy revision loaded by its hash — as OCSF lines. **(5), E-05 (b) itself: the GUEST inside a second OpenShell sandbox refused to run** — "could not lock itself down (the syscall filter could not be installed: `seccomp`: Operation not permitted) — nothing ran" — **and the host never sent the program** ("the guest never confirmed its boundary"); the report: level 3, backend `external`, all five properties `unknown`. PS-E-01's rule held inside someone else's wall; and the nested guest is the open problem (b) must resolve — a design choice for the next run (the guest's own filter refused under an outer wall that forbids adding one), not a patch to make a job green. **Run 8 (`36712891798` at `cb96c40`): both jobs GREEN, each step with its evidence** — (1) the granted program printed "hello from the granted file" INSIDE the OpenShell sandbox, under exactly the exported policy; (2) the same program under a narrower grant (`fs.read=./data/sub`) refused by DeluluLang, DL0703; (2b) a program file the export never names refused by OpenShell's wall ("permission denied"); (3) `curl` to `example.org` refused, and OpenShell's own OCSF log READ: `NET:REFUSE [MED] DENIED example.org [reason:policy_dns_ineligible]` and `NET:OPEN [MED] DENIED /usr/bin/curl(0) -> example.org:443 [reason:transparent_tcp_policy_denied]` — the study's witness, and the script now asserts that line; (4) the effective policy equal to the export both ways; (5) the nested guest failed closed, as witnessed. **The study's runtime witnesses for the export (a granted/ungranted pair, a denied `curl` with its log line) are read green; E-05 (b) — a guest that RUNS inside OpenShell — is not, for the reason (5) names.**

## 2026-09-30 — routine run 8, the close: OpenShell's deny line asserted; the nightly green; `openshell.yml` on push

**Run 9, `36713676712` at `74fb2b0`:** the runtime job red on the NEW assertion alone — OpenShell's log, read the instant
`curl` returned, held no `DENIED` line yet, where run 8 had found it at the same point (the sandbox's log reaches the
gateway asynchronously). Waited for with a bound in `af09d42` (twenty reads, 3 s apart; the log's tail printed if it never
comes) — the assertion kept. **Run 10, `36714033297` at `af09d42`: both jobs green**, the runtime job's evidence read
line by line: (1) "hello from the granted file" inside the sandbox; (2) DL0703 under the narrower grant; (2b)
"permission denied" for the file the export never names; (3) `NET:REFUSE [MED] DENIED example.org
[reason:policy_dns_ineligible]` and `NET:OPEN [MED] DENIED /usr/bin/curl(0) -> example.org:443
[reason:transparent_tcp_policy_denied]`, asserted; (4) the effective policy equal to the export; (5) the nested guest
failing closed. The prover job now runs on every push that changes what decides the export (read green by hand eight
times, runs 3–10); the runtime job stays by hand.

**CI:** the nightly `36694905248` (`381fed8`) — **complete, every job success**, `miri-slow` included (run 7 had asked for
it: it interprets `read_regular` and the secret store's write). Push runs `7dfb566` `36708157193`, `804ddc8`
`36708404843`, `d447b29` `36708628013`, `a3b2c90` `36709029216`, `154310c` `36709802148`, `9594e87` `36710708824`,
`625b6aa` `36711469433`, `cc3990f` `36712231055`, `cb96c40` `36712886514`, `74fb2b0` `36713671193`, `af09d42` `36714026123`
— success; `adff7a8` `36714603166` — 16 jobs, none failed; `openshell.yml`'s first push run `36714603164` (`adff7a8`) —
success (the prover job; the runtime job skipped on push). Only the run's last records commit is left for the next run.

**Routine run 8, closed.** SCOPE-HIDDEN-1 (D-V2-81) found and fixed; **PS-E-05 (a) complete** — `delulu sandbox policy
--format openshell` (D-V2-82), read on macOS and Windows before `master` moved, checked by OpenShell's own prover and
ENFORCED by a real OpenShell sandbox on a runner; on the way, the loop caught a Windows-only test assumption, a
regression on the branch (a flag reaching a verb that ignored it), and EXPORT-CASE-1 (a widening in the new code), each
before `master` moved. E-05 (b) opened with a real finding: the guest cannot yet run nested in OpenShell (its own
`seccomp` refused; it fails closed). Mutants M58–M74. Loop engineering: HANDOFF §11.5 (a mutant loop's stale binary; a
workflow dispatchable only on `master`; falsifiers from the checker's documented cases; a flag documented for a command
reaches every verb), the routine's steps 5 and 7 (wait on a run from the VM with `curl`).

## 2026-10-04 — routine run 10: the nightly red four times — a cold cache and three Wasmtime advisories (D-V2-84)

**Arrival.** Four days after routine run 8 closed (2026-09-30). `survey check` ok (1,482 nodes, 13,114 edges); `doctor
--check` ok, all checks passed. `gh` is present in the VM but its `GH_TOKEN` is refused ("invalid") — CI read with
`curl` against the REST API and the GitHub MCP tools.

**CI since run 8.** Run 8's last commit `e936ea5`: push run `36716667440` — **success** (its "Open / next" (1)). Then
four nightlies on the same commit, **all red**: 2026-10-01 `36844480022` (`supply-chain` alone), 10-02 `36988889261`,
10-03 `37110949841`, 10-04 `37191610614` (`supply-chain`, `test` on Linux x64, macOS and Windows, and `arm64` — the
full suites). Every other job green (18 jobs each). On 10-04 the Linux x64 suite's annotations: ping-pong NOT MEASURED
(the runner busy: three attempts, 2.01–2.25x against a control of 2.05x), all five properties established; macOS ping-pong NOT MEASURED (3
hardware threads, 2.21x), egress, host loss and privilege floor established, reads and memory not confined.

**Cause 1, the four suites — `egress_features` on a cold cache.** Its `cargo metadata --offline` reads every
platform's crates; a build downloads only the runner's; a warm rust-cache held them, and **Rust 1.99.0 (stable,
2026-10-01) changed the cache key** — `dtolnay/rust-toolchain@stable` installs it beside the pinned 1.96.1 and both are
hashed — so every nightly from 10-02 restored nothing ("No cache found"), failed `failed to download core-foundation
v0.10.1 … --offline was specified`, and a red job saves no cache. Witnessed in the VM with a fresh `CARGO_HOME` (red,
`addr2line v0.26.1`), then green after `cargo fetch --locked` in it. **Cause 2, `supply-chain` —** RUSTSEC-2026-0325,
-0326, -0327 against wasmtime 48.0.3 and `yoke-derive 0.8.3` yanked; witnessed with cargo-deny 0.20.2 (exit 1 naming
all four, exit 0 after). **Fixed (D-V2-84):** `ci.yml`'s `test` and `arm64` jobs fetch before the suite; wasmtime 48.0.5
and yoke-derive 0.8.4, lockfile only. Clippy clean; the full suite alone **2,094 passed, 0 failed, 15 ignored** (156
binaries), cargo exit 0; the two-engine differential by hand (`--release --test differential -- --ignored`) passed in 110.8 s.

**Also found on arrival — routine run 9 never reached `master`.** Branch `claude/stoic-ptolemy-uyw9g4` holds one commit
after `e936ea5`: `f446bfa` "PS-E-05 (b): a guest runs under an outer wall's syscall filter only when its launcher declares
it (D-V2-83)", 2026-09-30 15:43 UTC, session `session_019ymhQRJFgC6Kzb8PjiKtDj` — with no `V2_LOG`, sync-log or status
record. Its runner reads: `witness.yml` `36738984491`, `36738988731`, `36738993182` — success; `openshell.yml` (runtime)
`36738997626` — **red**: inside a real OpenShell sandbox the declared guest still failed closed. The next entry takes it
up. The two other branches with a commit `master` lacks (`claude/wonderful-hamilton-bo8k53` `7d6a22b`,
`claude/friendly-thompson-vz6m23` `d3f0d52`) are witness branches of runs 3 and 6 whose work landed in revised form
(`a_computing_guest_ends_when_its_host_is_killed`; D-V2-73's `Within` in place of `FrameDeadline`) — nothing stranded.

## 2026-10-04 — routine run 10: routine run 9's PS-E-05 (b) recovered — the kernel answers; OpenShell's `exec` cannot carry a conversation

**What run 9 left.** `f446bfa` (D-V2-83): an external launcher may declare an outer wall (`__guest --stdio-pipes
--outer-syscall-filter`); declared, a guest whose own `seccomp` filter is refused with EPERM while a filter is in force
runs under that one and says so ("an outer syscall filter, not its own", in place of its filter's four words); the host
refuses the word from a guest it started itself. Its witnesses re-run here: 4 of 4 green; its runner reads
`36738984491`, `36738988731`, `36738993182` success. Its OpenShell reading `36738997626` — never recorded — **red**: (5a)
held, but the declared guest failed closed too, "`/proc/self/status` shows no filter in force". **Merged** into `master`
(`83a62ba`, not cherry-picked: the runs above name `f446bfa`).

**Why it was red, read from the same run:** step (4) printed OpenShell's effective policy — `/usr`, `/lib`, `/etc` and the
program, **no `/proc`**, and OpenShell adds none. Under its Landlock the guest could not read its own status;
`unwrap_or_default` made the unreadable file "no filter". Safe — it failed closed — and useless where it was built for.
**Fixed (`9e75017`, D-V2-83 amended):** the in-force check is `prctl(PR_GET_SECCOMP) == 2`, the kernel's own answer, no
file. **The witness could not see it:** its simulated wall was the filter alone. It now runs every expectation under two
walls — the filter, and the filter plus a Landlock layer under which nothing in `/proc` can be read (checked first:
`cat /proc/self/status` refused inside it, a file elsewhere read; `NOT MEASURED` where a kernel has no Landlock). Red on
`f446bfa`'s code with OpenShell's exact words; green after. Unit: `a_filter_in_force_is_the_kernels_own_answer`.
Mutants M84–M88 red (M85 — the check always yes, run 9's surviving M77 — now red on the unit test). Clippy clean;
`scripts/check-other-os.sh` clean on all five targets; the full suite alone **2,102 passed, 0 failed, 15 ignored** (157
binaries), cargo exit 0; `witness.yml` on `9e75017`: arm64 `37233188734`, macOS `37233190495`, Windows `37233191959`,
Linux x64 `37233193360` — success (arm64's push-run properties say Landlock is there, so its hidden-`/proc` case measured).

**Read inside OpenShell again** — `openshell.yml` `37233187076` (`9e75017`): the prover job green; the runtime job's
(1)–(4) green as before, (5a) held, and **(5b) got past the filter** — "the guest's own syscall filter was refused by a
filter already in force on it — the outer wall its launcher declared stands in" — and then failed on the CHANNEL: the
host heard nothing for 60 s, and the guest read the end of its input ("failed to fill whole buffer").

**The channel, measured instead of guessed** (my first hypothesis — the CLI's line-buffered standard output — was
refuted by the first measurement). `37233707240` (`0ed045f`) and `37234148291` (`4cae1f9`) time each byte across the
relay: through `openshell sandbox exec`, a command's standard output and error arrive **as written** (a byte with no
newline at 0.0 s), but **the command starts only when its standard input ENDS** — `printf x; sleep 3; printf y; sleep 3`
piped in, and the command's first line came at 6.0 s, both bytes with it; the same with `--tty`. v0.1.2's `exec` reads
the input to its end before it runs anything, so no conversation can cross it: in (5b) the guest started only after the
host's 60 s deadline closed its input. **Through SSH it streams:** `openshell sandbox ssh-config dlg` prints a `Host
openshell-dlg.default` entry whose `ProxyCommand` is `openshell ssh-proxy`, and over it the command started at 0.1 s,
`x` arrived at 0.1 s and `y` at 3.0 s. So (b)'s launcher is `ssh`, not `exec` — the next commit.

**Then the launcher over `ssh` (`fc676d1`, D-V2-85) — read green inside a real OpenShell sandbox:** `openshell.yml`
`37234907676`, both jobs success. Step (5), line by line: the launcher `exec ssh -T -F …/guest-ssh.cfg -o BatchMode=yes
openshell-dlg.default /usr/local/bin/delulu $DELULU_GUEST_ARGS $GUEST_EXTRA`; **(5a)** undeclared — "could not lock itself
down (… `seccomp`: Operation not permitted …) — nothing ran", "the guest never confirmed its boundary, so it was not sent
the program", exit 1; **(5b)** declared — "the guest's own syscall filter was refused by a filter already in force on it —
the outer wall its launcher declared (`--outer-syscall-filter`) stands in for it; the guest's own filter is NOT
installed", then "hello through the OpenShell guest" (a granted `fs.read` the host performed), exit 0; the report level 3,
`external`, `host_guarantees` empty, `guest_reported` ["no file writes", "reads only from the system paths", "no TCP bind
or connect", "an outer syscall filter, not its own"], all five properties `unknown`. "openshell-runtime: every
expectation held." **PS-E-05 is complete:** (a) the export, (b) the guest inside OpenShell — the recipe in
`docs/DEPLOYMENT.md`. Still open beside it, recorded in the study: whether a `protocol: rest` endpoint's TLS, terminated by
OpenShell's own CA, verifies inside the wall when `delulu run` itself runs there ((a) with a network grant).

## 2026-10-04 — routine run 10: REPLY-HOLD-1 — the broker daemon's reply owed within a bound (RW 4.35, D-V2-86)

**Chosen** from run 8's "Open / next" (the red-team rows): RW 4.35 was "found, witnessed" and touches the e-stop's path.
**Witnessed red on the VM** (`brokerd::tests`, two new witnesses): a `GuardRequest` with a 4 MiB `why`, then a client that
asks for `GuardPending` and never reads — `broker status` behind it: "the broker accepted the request but did not answer
within 15s"; and a client that reads 32 KiB every half second — the same. **The first fix** (a 1 s `SO_SNDTIMEO` and the
reply through a whole-write `Within`) turned the silent client green and left the slow one red: Linux's send timeout is
per buffer waited for, so the single 4 MiB `write` never returned to the check. **Fixed:** `Within` offers 64 KiB per
write. **Windows, read on its runner (`37236818679`): the first Windows design failed** — `SetNamedPipeHandleState` refused
`PIPE_NOWAIT` on the connected handle (win32 error 231), and the loop stayed held anyway in `FlushFileBuffers` (the
connection's `Drop` waits for the client to read everything, so no reply is discarded at disconnect). Withdrawn whole:
Windows' transport is as it was, the witnesses `cfg(unix)`, RW 4.35 open for Windows. Mutants M89, M90, M91 red. Clippy clean;
`check-other-os.sh` clean on all five targets at `9b6b4a1`, and Windows and macOS again at `2bb26f8`; the full suite alone 2,104 passed, 0 failed, 15 ignored (157 binaries), cargo exit 0, at `9b6b4a1` (`2bb26f8` changes only Windows code and `cfg` attributes); the new tests on the other runners first: macOS `37236820414` green (both witnesses); Windows `37236818679` red — the first Windows design, withdrawn (above).

**Then Windows (`4d4d033`), the same run.** The first design's failure, read: the connected handle refused `PIPE_NOWAIT`
(win32 error 231), and `Drop`'s `FlushFileBuffers` held the loop for a reader that never reads. The second asks the pipe
(`NtQueryInformationFile`, `FilePipeLocalInformation`) how much it can take and whether it has drained: a write no
larger than the room is buffered at once, a flush after the drain returns at once, each waited for at most the bound,
and `Drop` skips the flush of an answer nobody took. Windows lint clean; the witnesses un-`cfg`'d; read on the runners
with the broker's unit tests and every broker-facing target (`broker_cli`, `estop_cli`, `guard_cli`, `guard_e2e`,
`secret_verify_cli`): `witness.yml` at `4d4d033` — Windows `37238378527` and macOS `37238381140` green (Windows: 140 unit tests, the two witnesses among them, its log naming both drops — "the reader took nothing" and "did not take one whole frame within 5s" — and `broker_cli` 3, `estop_cli` 5, `guard_cli` 2, `guard_e2e` 3, `secret_verify_cli` 1). **RW 4.35 closed on every OS.**

## 2026-10-04 — routine run 10, the close

**Built and read:** the nightly's two reds fixed (D-V2-84 — every platform's crates fetched before a suite; Wasmtime
48.0.5); routine run 9's stranded PS-E-05 (b) merged and made to work inside a real OpenShell sandbox (D-V2-83 amended:
the kernel's `PR_GET_SECCOMP`; D-V2-85: the launcher over `ssh`, because `sandbox exec` starts its command only once its
input ends) — **PS-E-05 complete**; REPLY-HOLD-1 (RW 4.35, D-V2-86) closed on every OS, Windows by its second design.
Mutants M84–M91. **CI, all read:** push runs `37232688376` (`9bc405e`), `37234832894` (`95a9789`), `37236037454`
(`9b77312`), `37238153709` (`99f0c94`), `37239615098` (`db73893`) — success, 16 jobs each, none failed; `heavy-gates`
`37236067731` success; `openshell.yml` `37234907676` both jobs green. **Loop engineering:** the routine's step 1 lists every
branch's commits `master` lacks; step 3 names `gh`'s refused token; step 5 times a stalled channel before a fix and lints
one target; step 6 names the author; HANDOFF §11.4 REPLY-HOLD-1, §11.5 four lessons.

## 2026-10-05 — routine run 11: CI read green; PS-E-01 — an attester's claim that names a property meets it at L3 (D-V2-87)

**On arrival (CI):** `master` at `2cb3f87` (routine run 10's last reading); its push run `37240818377` — **success**, 14
jobs, none failed (`miri-slow`, `heavy-gates` skipped as on every push). Ping-pong: arm64 MEASURED 2.89x against a control
of 4.06x, passed; Linux x64 and Windows NOT MEASURED (the runner busy); macOS NOT MEASURED (3 hardware threads). The five
properties as on every run since run 8 (macOS: filesystem confinement absent, reads not confined). No nightly since run
10's reading (10-04's red one, `37191610614`, was fixed by D-V2-84 and its push runs read green). Every `claude/*` branch
listed after unshallowing: none carries a commit `master` lacks but the two run 10 recorded as superseded witnesses
(`7d6a22b`, `d3f0d52`). Survey `ok` (1,483 nodes), `doctor --check` all checks passed. `gh`: not used (the MCP tools and
`curl`).

**Chosen:** PS-E's next item in run 10's list — E-01's remainder, "attesters' claims as properties". **Witnessed red on
`2cb3f87`:** `hostile-agent` at L3 with a verified attester claiming all five properties by name — refused DL1408 with
every property `unknown`, while the refusal named "an external launcher whose attester vouches for it" as the way out.
**Built (D-V2-87):** a claim `PROPERTY: how` (exact name) answers that property at L3 only; the state stays `unknown`, the
claim kept beside it as `attested = {attester, by}`; DL1408's refusal and `explain` say how to vouch; the schema gains the
optional field; `DEPLOYMENT.md` and `for-agents.md` show it. Six mutants M92–M97 red (M93's first form was a no-op the
code after it overwrote — rewritten and red). Clippy clean; `check-other-os.sh` clean for Windows and macOS (aarch64);
the full suite alone 2,108 passed, 0 failed, 15 ignored (157 binaries), cargo exit 0; the new tests on the other runners before `master` moved — `witness.yml` at `3ec6abe`:
macOS ``37247548752` green (153 unit tests, `sandbox_attest_cli` 8, `sandbox_confirm_cli` 13)`, Windows ``37247550461` green (142 unit tests — the `cfg(unix)` typestate test not among them — `sandbox_attest_cli` 5, `sandbox_confirm_cli` 8)`, Linux arm64 ``37247552077` green (172 unit tests, `sandbox_attest_cli` 8, `sandbox_confirm_cli` 14)` (`sandbox_attest_cli`, `sandbox_confirm_cli`, the binary's unit tests).

## 2026-10-05 — routine run 11: ANSWER-HOLD-1 — a guest that stops reading cannot hold its host (RW 4.37, D-V2-88)

**Chosen** from run 10's "Open / next" (the red-team rows): RW 4.37 was "plausible, not witnessed" and, unlike RW 4.40,
leaves a host held with nothing to end it. **Slice 1's push runs read first:** `5bd76aa` — CI `37248188675` success, 14 jobs, none failed (arm64 ping-pong MEASURED 2.84x against a control of 4.13x; the other three NOT MEASURED),
`openshell` `37248188760` and `ocsf` `37248188651` success. **Witnessed red on `5bd76aa`** (the host's end configured by a
helper factored out of `open_channel` with no change in behaviour): a guest that asked for an 8 MiB file's text and read
nothing held the host 20 s, until it hung up. **Fixed (D-V2-88):** every guest socket gets a write deadline as well as a
read one, and `serve` owes each answer whole within the frame deadline (`Within`); the operator reads "did not take the
host's answer whole". Mutants M98–M102 red (M101 first survived the guest test — `Within`'s check fires before the
transport's with equal deadlines — and got a unit witness). Clippy clean; `check-other-os.sh` clean on all five targets;
the full suite alone 2,110 passed, 0 failed, 15 ignored (157 binaries), cargo exit 0; CI dispatched at the branch, `37249221745` — every job read green but the two long `miri-slow` jobs (check, broker), cancelled once the rest were read: `test` on Linux, macOS and Windows, arm64, `microvm` (the KVM runner — a microVM guest's channel goes through `bounded` now), `microvm-reproducible`, `heavy-gates`, `miri-slow` (syntax), lints and the rest; and `witness.yml` naming the witness green on macOS `37250364770` (all four cases, 5.8 s) and Windows `37250366783` (the two by-name cases, over a named pipe, 1.8 s).

## 2026-10-05 — routine run 11: PS-E-04's attestation binding — a statement for the launcher its attester measured (D-V2-89)

**Slice 2's push runs read first:** `396653b` — CI `37250662612` success, 14 jobs, none failed (arm64 ping-pong MEASURED 2.90x against a control of 4.10x; the other three NOT MEASURED); `ocsf` `37250662608` success. **Chosen:** PS-E's next
buildable item (E-04's attestation binding; H6 and `contained`'s set wait — the first on a harness twice stopped by a
classifier, the second on macOS's gaps). **Witnessed absent on `396653b`:** `sandbox attest` refused `--measure-launcher`,
and a statement had no field for a launcher. **Built (D-V2-89):** `launcher_blake3` in the statement, measured by the
attester; a bound statement is `delulu-attestation-v2` (its signed first line too), refused unless the launcher started has
that digest; v1 unchanged. Mutants M103–M107 red. Clippy clean; `check-other-os.sh` clean for Windows and macOS; the full
suite alone 2,112 passed, 0 failed, 15 ignored (157 binaries), cargo exit 0; read on the runners before `master` moved: `witness.yml` at `3c74e97` — macOS `37251102472`, Windows `37251104259`, Linux arm64 `37251106129`, all green, `sandbox_attest_cli`'s five attestation tests and the binary's attestation unit tests named in each log.

## 2026-10-05 — routine run 11, the close

**Built and read:** PS-E-01's attesters' claims as properties (D-V2-87 — `hostile-agent`'s advertised way out at L3 had
never worked); ANSWER-HOLD-1 (RW 4.37, D-V2-88 — a guest that stopped reading held its host for ever); PS-E-04's
attestation binding (D-V2-89 — a v2 statement naming the launcher its attester measured). Mutants M92–M107. **CI, all
read:** push runs `37248188675` (`5bd76aa`) and `37250662612` (`396653b`) — success, 14 jobs each; `openshell`, `ocsf`
success; CI dispatched at the branch `37249221745` (every job but two long `miri-slow` jobs, cancelled); `witness.yml`
eight runs, all green. **Loop engineering:** step 5 — a surviving mutant is first a question about the mutant; a branch
CI run's suite logs never name a unit test, so a new one is read by `witness.yml` with its filter; HANDOFF §11.5 three
lessons. **PS-E left:** H6 (macOS and Windows under the escaped-guest harness), `contained`'s required set, macOS's
`fexecve`.

**The last reading (routine run 11):** `52f1d8e`'s push runs — CI `37251922235` success, 14 jobs, none failed (arm64
ping-pong MEASURED 2.93x against a control of 4.13x; the other three NOT MEASURED); `ocsf` `37251922186` and `openshell`
`37251922219` success. `master` green through `52f1d8e`.

## 2026-10-05 — routine run 12: CI read green; PS-E-01 — a macOS guest's memory ceiling is the host's sampler (D-V2-90); RUNDIR-PERM-1 (D-V2-91)

**On arrival:** `ed95f0d`'s push run (run 11's last commit, unread by it) — CI `37253089773` success, 14 jobs, none failed
(arm64 ping-pong MEASURED 2.93x against a control of 4.10x; Linux x64 and Windows NOT MEASURED — busy; macOS NOT MEASURED —
3 threads; the properties as on every run since run 8). No nightly since run 10's reading (the four reds of 10-01..10-04 were
run 10's, fixed by D-V2-84). Every `claude/*` branch listed after unshallowing: none carries a commit `master` lacks beyond
run 10's two recorded witness branches. `gh` is installed in this VM (`/usr/local/bin/gh`), not used — the MCP tools and the
REST API answered. Run 11's tests re-run in the VM: `sandbox_attest_cli` 9 passed; the binary's `attest`, `boundary` and
`guest::tests` unit tests 20 passed; `delulu-runtime`'s `channel` tests 23 passed.

**Chosen:** PS-E-01's open item, `contained`'s required set, waits on macOS's two gaps — reads and memory. The memory gap is
buildable and witnessable: **witnessed red on a macOS runner** (`witness.yml` `37267847333` at `d9d4803`) — a guest
allocating without end under `mem=64 MiB` was stopped by `wall=` at 8 s, `memory: not confined`. **Built (D-V2-90):** the
host's watchdog reads the guest's peak footprint (`proc_pid_rusage`) and ends it at its budget; claimed as `memory ceiling
(the host's sampler)`, the stop named with `observed_bytes`. The first green read (`37268383827`, the whole `delulu`
package — red only in `doctor_cli`'s three stale-map tests, the map not regenerated on that branch commit) observed **247 MB
against 64 MiB at the 25 ms interval**; at 5 ms, 68.8 MB and 92.8 MB on two reads (`37269100267`, `37269929073`). Mutant M108 (the reader answering a constant: claimed, never
firing) red on macOS (`37268731314`: the new witness, the existing ceiling test — now run there because macOS claims the ceiling
— and the unit reading test).

**Found in that witness's log, fixed (RUNDIR-PERM-1, D-V2-91):** every macOS sandboxed run printed the guest's "not
owner-only (mode 0755); this filesystem does not enforce POSIX permissions" — the host made the run's directory with the
default mode, and a macOS guest, which may write nothing but its socket, could not narrow it. Witnessed red on macOS
(`37268778120`); the host now makes it 0700 and never adopts one already there. M109 (0755) and M110 (adopted) red in the VM.
**Then the Windows read** (`37269102715`, green) **showed `unused_mut`** in the new function — linted for Windows before the
function existed; split by OS in `07045d5`, `check-other-os.sh` clean for both.

**Verified:** clippy clean; `check-other-os.sh` clean for Windows and macOS; the full suite alone 2,115 passed, 0 failed, 15 ignored (157 binaries), cargo exit 0; read on the
runners before `master` moved: at `3c6e67b` — macOS `37269100267` (the whole `delulu` package, 877 passed, 0 failed, 12 ignored, 94 binaries), Windows
`37269102715`, Linux arm64 `37269105326` (216 passed); at `07045d5` — Windows `37269926751`, macOS `37269929073`.

## 2026-10-05 — routine run 12: PS-E-01's `contained` set (D-V2-92); PS-E complete with three residuals (D-V2-93)

**Slice 1's push runs read first:** `07045d5` — CI `37270189858` success, 14 jobs (arm64 ping-pong MEASURED 2.80x against a control of 4.12x; the other three NOT MEASURED; `resource_ceiling` established on all four, macOS's included — the first push run to say so); `ocsf` `37270189893` success; `4471d6f`'s (records) read with the close. **Chosen:** `contained`'s set — its wait on macOS's gaps ended with D-V2-90
for memory, and reads alone remain absent there. **Witnessed red on `4471d6f`:** a boundary this host measured without a
memory ceiling was confirmed under `contained` and sent the program. **Built (D-V2-92):** `contained` requires egress,
resource and host loss; refuses what this host measured `absent`; an external launcher's `unknown` is reported, not refused
(`Profile::requires_proof` — `hostile-agent` alone). M112–M116 red. **Verified:** clippy clean; the full suite alone
2,116 passed, 0 failed, 15 ignored (157 binaries), cargo exit 0; CI dispatched at `0560898` (`37271099512`): every job green — `test` on Linux, macOS and Windows (the whole workspace each), arm64, `microvm` (the KVM runner: a microVM run under `contained`), `microvm-reproducible`, `heavy-gates`, lints, formal, fuzz, Miri (diag, atlas, ffi), supply-chain, editor; the three long `miri-slow` jobs cancelled once the rest were read. Each OS's properties: Linux x64, arm64 and Windows all five established; macOS four, `filesystem_confinement` absent (reads not confined), `resource_ceiling` established — read before `master` moved, then the long jobs cancelled.
**Decided (D-V2-93):** PS-E closes; H6, macOS's launcher window and macOS's reads stay open where they are recorded; P8 is
next.

## 2026-10-05 — routine run 12: SILENT-QUEUE-1 — silent connections no longer queue the broker (RW 4.40, D-V2-94)

**Slice 2's push run read first:** `8cbd1e0` (carrying `0560898`) — CI `37272692095` success, 14 jobs, none failed (arm64 ping-pong MEASURED 2.86x against a control of 4.07x; the other three NOT MEASURED). **Chosen:** the `Open / next` list's red-team rows; RW 4.40 is the one on the
e-stop's path (P8 is next, and a robot's stop goes through this daemon). **Witnessed red on `8cbd1e0`:** six silent
connections held a `Status` 30.6 s. **Built (D-V2-94):** an accept thread, a reader thread per connection (each request
still owed whole within 5 s), one handler for whole requests; at most 64 reading. M118 (one reader at a time) red, 4.8 s.
**Verified:** clippy clean; `check-other-os.sh` clean for Windows and macOS; the 26 daemon unit tests; the full suite alone
2,117 passed, 0 failed, 15 ignored (157 binaries), cargo exit 0 — on its second run: the first lost `doctor_cli`'s three tests to a script written into the tree mid-suite, and found the runtime's thread-stack gate refusing the daemon's two new threads, unclassified (fixed in `101cb1e`); read on the runners before `master` moved: `witness.yml` at `7c55af9` — Windows `37273275501` (160 passed), macOS `37273278082` (175 passed), Linux arm64 `37273280678` (193 passed): the daemon's unit tests and six daemon-facing targets (`broker_cli`, `estop_cli`, `guard_cli`, `guard_e2e`, `sandbox_guard_e2e`, `secret_verify_cli`), the witness named in each.

## 2026-10-05 — routine run 12: RW 4.40's own bound (D-V2-94, amended); the run's close

**Slice 3's push runs read first:** `101cb1e` — CI `37274536930` success; `633437e` (records) — CI `37274733834` success. **Found re-reading `7c55af9` (on `master`):** a reader counted against the 64
only while it read, not while it waited on a busy handler — 300 whole requests behind an unread answer grew the process
from 4 to 240 threads (red on `101cb1e`). Counted until the handover now (`77d436e`); M119 red; the flood witness's own
teardown asks until the daemon answers (a full pool drops a connection — the `Shutdown` included — so the first version of
the test hung its stop). **Verified:** the 27 daemon unit tests; clippy and `check-other-os.sh` clean; the full suite alone
2,118 passed, 0 failed, 15 ignored (157 binaries), cargo exit 0; read on the runners: `witness.yml` at `77d436e` — Windows `37275973144` (158 passed), macOS `37275975885` (173 passed), Linux arm64 `37275979053` (187 passed, the flood witness among them), the daemon's unit tests, `broker_cli` and `estop_cli` (and `guard_cli`, `guard_e2e` on Windows and macOS).

**The run, closed:** PS-E's last E-01 steps (D-V2-90 macOS's memory ceiling; D-V2-92 `contained`'s set) and PS-E complete
with three residuals (D-V2-93); RUNDIR-PERM-1 (D-V2-91); SILENT-QUEUE-1 (RW 4.40, D-V2-94). Mutants M108–M119. Loop
engineering: `scripts/ci-log-summary.py`; CLOUD_ROUTINE steps 3 and 5; HANDOFF §11.5 five lessons. **Next: P8-01.**

## 2026-10-05 — routine run 12: RW 4.39 — the broker log's drops in words

**Witnessed red on `89f69c8`** (the wording first extracted unchanged into `brokerd::dropped_because`): one class of drop had
three wordings — the frame deadline's, the OS's "Resource temporarily unavailable (os error 11)", the library's "failed to
fill whole buffer". **Now** each way a request fails to arrive is said once, in its own words; what a client sent is quoted
escaped and bounded. M120, M121 red. **Verified:** clippy and `check-other-os.sh` clean; the full suite alone 2,119 passed, 0 failed, 15 ignored (157 binaries), cargo exit 0;
read on the runners: `witness.yml` at `f76eafd` — Windows `37277293552` (149 passed), macOS `37277296503` (164 passed), the daemon's unit tests and `broker_cli`, the witness named in each. RW 4.39 closed.

**The last reading (routine run 12):** every push run of the run read green — CI `37276656840` (`77d436e`), `37276819686` (`89f69c8`), `37277912297` (`f76eafd`), `37278057068` (`11ef757`): success, 14 jobs each, none failed (arm64 ping-pong MEASURED 2.87x/4.07x, 2.90x/4.12x, 2.96x/4.06x, 2.92x/4.04x; the other three NOT MEASURED); `master` green through `11ef757`. The 2026-10-05 nightly (cron 03:00, fired about 09:15 on recent days) had not fired by 07:46. P8-01's build order written into `V2_P8_DESIGN.md` from the code as it
stands (minting already crosses the channel; using a device does not — the interpreter's device calls move behind a
function `HostChannel` shares).

## 2026-10-05 — routine run 13: CI read green; P8-01 — a control program runs in a guest (D-V2-95)

**CI on arrival:** routine run 12's last push run, `f912084` — CI `37279799389` success, 14 jobs, none failed (arm64
ping-pong MEASURED 2.91x against a control of 4.12x; Linux x64 and Windows NOT MEASURED — busy; macOS NOT MEASURED — 3
threads; the properties as on every run since run 8: macOS `filesystem_confinement` absent, the rest established). The
2026-10-05 nightly, `37293332206` on `f912084` (fired 09:56): every job green — `test` on three OSes, arm64 (MEASURED
2.86x/4.11x), `heavy-gates`, `microvm`, `microvm-reproducible`, Miri (diag, atlas, ffi), fuzz, formal, lints, editor,
supply-chain — with the three `miri-slow` jobs still running at the close of this slice. No branch carries a commit
`master` lacks beyond run 10's two recorded witness branches. Run 12's daemon tests re-run in the VM: 28 passed
(`--bin delulu brokerd`; `delulu` has no library target, so `--lib` answers "no library targets").
**Chosen:** P8-01 (the routine's order; PS-E complete). **Witnessed red:** on `f912084` a control program under
`--sandbox` is refused "it uses Actuator. Nothing ran." — reproduced as mutant M122, which turns all seven new witnesses
red. **Built (D-V2-95):** the interpreter's device bodies moved behind `device::actuate`/`sense`; `HostChannel::with_devices`
performs a guest's `command` and `read` after its custody gate; `run_cmd.rs`'s device code shared with the sandboxed path
(`device_terms`, `watch_devices`, `plan_devices`, `close_devices`); `CARRIED` gains Actuator and Sensor; the broker starts
when the guest is sent its program. **Verified:** clippy clean; `check-other-os.sh` clean for Windows and macOS (twice — the second after the last code edit); read on the runners before `master` moved — at `76ba9ca` Windows `37298032325` (73 passed) and Linux arm64 `37298039715` (74 passed) green, macOS `37298035733` red on the e-stop witness alone (the test's broker socket path, 113 bytes, past macOS's 104 — the test's scratch name, fixed in `45e65eb`); at `45e65eb` macOS `37299710143` (47 passed), Windows `37299713468` (46 passed), Linux arm64 `37299716481` (47 passed): `sandbox_devices_cli`'s eight witnesses named on each, with `sandbox_modes_cli`, `hw_adapter_cli` and `dead_man_cli` (and at `76ba9ca` `estop_cli`, `actuate_cli`, `atlas_chain`, `sandbox_run_cli`); the full suite alone 2,127 passed, 0 failed, 15 ignored (158 binaries), cargo exit 0 — its second run: the first (2,124 passed, 2 failed) lost the Survey's freshness test to a test file edited while it ran, and found `sandbox_modes_cli`'s unapplied `--approved` (item 7).

## 2026-10-05 — routine run 13: P8-01's rest — level 2 read on the KVM runner; the simulator's clock and sign-off under the sandbox (D-V2-95, amended)

**Push runs read first:** `45e65eb` (carrying `76ba9ca`) — CI `37300708685` success, 14 jobs, none failed (arm64 ping-pong MEASURED 2.93x against a control of 4.12x; the other three NOT MEASURED), `ocsf` `37300708671` success; `6796c31` (records) — CI `37301082817` success, 14 jobs (arm64 MEASURED 2.90x/4.11x). **Level 2:** a KVM-gated witness in `microvm_cli.rs` — the control program in a microVM,
its first act a command against a 150 ms heartbeat — green on the KVM runner at `07f6cd9` (`37301435891`, the `microvm` job:
12 passed); mutant M128 (the broker started before the launch) survives at L1 and is red there (`37301599674`: the VM's boot
revoked the lease first), reverted. **The simulator's parity (`5b89f9f`):** `--sim-step` and `--signoff` apply under
`--sandbox`; two witnesses; M129–M132 red. **Verified:** clippy clean; `check-other-os.sh` clean for Windows and macOS after the last code edit; read on the runners at `5b89f9f` before `master` moved — macOS `37303092995`, Windows `37303096425`, Linux arm64 `37303099029`: `sandbox_devices_cli` 10 passed and `sandbox_modes_cli` 11 passed on each, both new witnesses named; the full suite alone through `scripts/suite.sh`, 2,129 passed, 0 failed, 16 ignored (158 binaries — the new KVM-gated test among the ignored), cargo exit 0, the tree unmoved.

## 2026-10-05 — routine run 13: P8-01 complete — the devices' journal in the sandbox report (D-V2-95, amended)

**Push runs read first:** `fee5353` — CI `37302336639` success, 14 jobs, `microvm` among them (the L2 witness green on `master` too); `5b89f9f` — CI `37304139892` success, 14 jobs (arm64 MEASURED 2.97x), `ocsf` `37304139836` success; `521d688` — CI `37304345347` success, 14 jobs (arm64 MEASURED 2.92x). **Built (`bc8bd91`):** the run report's `devices`, the host watchdog's journal (a sandboxed
run has no trace), declared in `delulu schema run-report`; M133, M134 red. **Verified:** clippy clean; `check-other-os.sh` clean for Windows and macOS after the last code edit; read on the runners at `bc8bd91` before `master` moved — macOS `37305292797` (28 passed: `sandbox_devices_cli` 10, `json_contract` 14, `schema_cli` 4), Windows `37305296588` and Linux arm64 `37305299628` green on the same three targets; the full suite alone through `scripts/suite.sh`, 2,129 passed, 0 failed, 16 ignored (158 binaries), cargo exit 0, the tree unmoved. **P8-01 is
complete** — every item of its design built and read at L1 and L2. **Next: P8-02**, the Verified-class adapter as a `.dpx`.

## 2026-10-05 — routine run 13: P8-02's build order, read against the code

With P8-01 complete and too little of the run left to build P8-02 whole, its build order was read from the code and written
into `V2_P8_DESIGN.md` (as run 12 did for P8-01): a Verified `.dpx` already runs by interpreting its re-proved DIR, and
`load_verified`'s DL1504 (a verified type matched exactly to an empty-row manifest signature) is the purity rule; the
broker already calls its adapter after the lease, rate and envelope checks, so the slot becomes an `Adapter` trait; the DIR
is plain AST data (no `Rc`), so a `VerifiedAdapter` owns one thread that interprets it per call; the PIN on the embedded
signature key and `--adapter-dpx` are the new surface. Nothing built — the next run starts at step 1.

## 2026-10-05 — routine run 13: P8-02 begun — the driver slot is a trait; the DIR is asserted `Send`

**Push runs read first:** `bc8bd91` — CI `37306161197` success, 14 jobs (arm64 MEASURED 2.88x), `ocsf` `37306161210` success; `ac31cf2` — CI `37306380748` success, 14 jobs (arm64 MEASURED 2.92x); `5c98e65`'s, `418dcca`'s and this commit's push runs were in progress at the close — the next run reads them first, with the 2026-10-05 nightly's last `miri-slow` job (`delulu-broker`; `delulu-syntax` and `delulu-check` read green). **Built (`418dcca`, P8-02 build-order steps 2–3, behaviour-preserving):**
`adapter::Adapter` (`command`, `read`, `Send`) — the one place a permitted command leaves the host — implemented by
`ProcessAdapter` unchanged; the broker holds `Option<Box<dyn Adapter>>`; and a compile-time assertion that
`delulu_check::Dir` is `Send` (falsified: an `Rc` in its place is E0277). Mutant M135 (the trait's `command` answering
without the driver) red on `hw_adapter_cli`. **Verified:** read on the runners at `418dcca` before `master` moved — Windows `37307415352` and macOS `37307419324`, `hw_adapter_cli`, `dead_man_cli` and `sandbox_devices_cli`, green; the full suite alone through `scripts/suite.sh`, 2,129 passed, 0 failed, 16 ignored (158 binaries), cargo exit 0, the tree unmoved. **Next:** P8-02's step 3 proper — the
`VerifiedAdapter` thread — then steps 4–7.

**The last reading (routine run 13):** every push run of the run read green — CI `37306707736` (`5c98e65`), `37308525902`
(`418dcca`), `37308742427` (`e561983`): success, 14 jobs each, none failed (arm64 ping-pong MEASURED 2.93x/4.11x,
2.86x/4.12x, 2.97x/4.12x); `master` green through `e561983`. The 2026-10-05 nightly `37293332206` green on every job read —
its `miri-slow (delulu-broker)` was still running at 12:36.

## 2026-10-05 — routine run 14: CI read green; P8-02 steps 3–4 — a Verified driver's logic on a thread of its own (D-V2-96)

**Read first (step 3):** `master`'s last push run, `7a895f0` — CI `37310919853` success, 14 jobs (arm64 ping-pong MEASURED
2.87x against a control of 4.11x; Linux x64 and Windows NOT MEASURED — busy; macOS NOT MEASURED — 3 threads; the sandbox
properties as on every run since run 8, macOS `filesystem_confinement` absent); the 2026-10-05 nightly `37293332206` on
`f912084` — success, every job, `miri-slow (delulu-broker)` included (finished 12:45). `release` (schedule) `37304777289`
success. No `claude/*` branch carries a commit `master` lacks beyond run 10's two recorded witness branches. Survey `ok`
(1,486 nodes, 13,391 edges), `doctor --check` all checks passed. `gh` is installed in this VM (`/usr/local/bin/gh`); the
MCP tools and the REST API through `curl` were used.

**Built (`45d1181`):** `delulu_runtime::verified_adapter` — `VerifiedAdapter`, an `adapter::Adapter` whose logic is a
re-proved Verified DIR and whose transport is the host's (`Transport`: write one frame, read one line, within the time
left). The driver interface is four pure exports, each held by R-Get (`r_get_verified`, the check `p.get` applies) to
exactly its type and an EMPTY row before the adapter exists — `encode_command(Str, Map[Str, Float]) -> Result[Str, Str]`,
`decode_command(Str, Str) -> Option[Str]`, `encode_read(Str) -> Result[Str, Str]`, `decode_read(Str, Str) ->
Result[Option[Float], Str]`. The logic runs on one thread that owns the DIR (16 MiB of stack and the depth bound that
fits it, so deep recursion is DL0905, never the host's stack), a fresh interpreter per call under the best-effort budget;
one deadline (`EXCHANGE_TIMEOUT`) covers the two logic calls and the transport; a frame that is not one line of at most
64 KiB, a fault in the plugin, a value outside its type, a non-finite reading, or any transport failure poisons the
adapter before anything more is written. **Eight witnesses** against an in-process device that logs every frame written.
**Falsified:** M136 (no line check), M137 (a logic fault does not poison), M138 (no interface check), M139 (a fresh
deadline for the transport), M140 (a non-finite reading accepted), M142 (a transport's refusal passed through), M144 (a
device's refusal poisons) — each red; M141 (no depth bound on the logic's thread) aborts the test process (SIGABRT, the
host's stack overflowed); **M143** (a late transport answer accepted) SURVIVES on a second wall — the decode's share of
the same deadline has no time left — and **M143b**, both walls removed, is red. Found while writing the witness: the
language's `parse_float` refuses `NaN` and `inf`, but its arithmetic computes them (`0.0 / 0.0` prints `NaN`), so the
host checks what the plugin RETURNS, not what the device sent. **Verified:** clippy clean; read on the runners at
`45d1181` before `master` moved — arm64 `37333709607`, Windows `37333705883`, macOS `37333701001`: 8 passed each, every
witness named; the full suite alone through `scripts/suite.sh`, 2,137 passed, 0 failed, 16 ignored (158 binaries), cargo exit 0, the tree unmoved.
**Next:** steps 5–7 — `--adapter-dpx`, its signer pin and its transport, end to end.

## 2026-10-05 — routine run 14: P8-02 complete — `--adapter-dpx`, a driver read once, pinned, re-proved and interpreted (D-V2-97)

**Built (`a8040fd`):** a hardware run's driver may be a signed Verified plugin. `--adapter-dpx FILE` names its logic,
`--adapter-transport CMD` the process that carries its frames to the device, `--adapter-signer HEX` the key it is pinned
to; `cli::load_dpx_driver` reads the bytes ONCE, verifies the embedded signature against that key (DL1510 another key or
a signature that does not verify, DL1511 none), checks the container and API (DL1507) and the class (never inferred),
replays the DIR through the checker (DL1504) — and the DIR interpreted is the one every check was made on, so nothing is
started by name and D-V2-50's check-then-start window does not exist for a Verified driver. The driver interface is
checked before the transport is even looked up, so a plugin that is not a driver resolves and starts no process; the
provenance decision is recorded before a refusal is acted on (C60), as a process driver's is. `verified_driver_flags`
refuses every combination that would drop a flag in silence: `--adapter-dpx` beside `--adapter-cmd` or
`--adapter-artifact`, either new flag without the other, `--adapter-dpx` without its signer, and both on a run that is
not `hw:` (exit 2, "Nothing ran"). Both flags join the sandbox's allowlist, so a sandboxed control program drives a
device through the host's Verified driver.
**`adapter::LineTransport`** is the reference transport (P8-03's shape): a process whose standard input and output carry
the device's line, sharing `ProcessAdapter`'s reader (`line_reader`, one factored function) and keeping its three laws —
a bounded reply line, the deadline it is GIVEN (never one of its own), and a line nobody asked for as a misframing.
**`examples/line_driver`** is the reference driver: `adapter.rs`'s `CMD`/`READ` protocol as a Verified plugin of four
pure exports, holding no authority — and `verified_adapter`'s unit tests read it from the shipped example, so the
example cannot drift from what the tests prove.
**Witnesses:** five end to end (`crates/delulu/tests/hw_dpx_cli.rs`) — a pinned driver drives the device (the frames its
logic computed are the frames the device logged; the device's own hard stop comes back as a refusal; a command outside
the envelope never reaches it), the same under `--sandbox`, a stranger's/unsigned/tampered driver each refused with its
code and the transport never started, a plugin that is not a driver refused before the transport is looked up, and each
flag combination refused — and three for the transport. Read by hand as well: the pinned run prints the signer and the
three verdicts, and the device's log holds exactly the two in-envelope frames.
**Falsified:** M145 (the pin not compared), M146 (an unsigned driver accepted), M147 (the interface checked only after
the transport starts), M148 (no combination check), M149 (the new flags not hardware-only), M150 (the sandbox does not
carry them), M151 (the transport's reply unbounded), M152 (the transport takes a deadline of its own), M153 (an unasked
line read as the answer) — each red. **A witness of an ORDER is made deterministic by a transport that does not exist**
(`hw_through`): a driver refused before its transport is looked up says why, while one whose transport came first says
"could not be started" — watching a started process die first would be a race.
**A new example is a new case in four gates, and only the whole suite sees them** (`681b6f6`): `cli_contract`'s
`delulu fmt --check examples`, `edit_cli`'s Atlas round-trip (`edit` re-prints what it inserts, so an item's own text
round-trips only in a canonically formatted file — the formatting fix closed this one too), `atlas_chain`'s snapshot
corpus (two snapshots, blessed) and `core_invariance` (150 lines added, NONE removed — no existing answer of the core
moved; the new example's own `authority` report reads "effects: (none — provably pure), capabilities: (none)", the
driver's purity seen from outside). None was reachable from the witness targets the runners read, which is why a slice's
local suite runs whole before `master` moves.
**Verified:** clippy clean for the workspace; `check-other-os.sh` clean for Windows and macOS after the last code edit;
read on the runners before `master` moved — at `a8040fd` macOS `37336594722` (27 passed), Windows `37336599645` (26) on
`hw_dpx_cli`, `hw_adapter_cli` and `sandbox_devices_cli`, every new witness named, and arm64 `37336603458` (26) on
`delulu-runtime`'s adapter tests, the three new transport ones among them; at `681b6f6` macOS `37337864832` and Windows
`37337869032` (35 each) on the four gates the example changed, every one green there too. The full suite alone through
`scripts/suite.sh`: 2,145 passed, 0 failed, 16 ignored (159 binaries), cargo exit 0, the tree unmoved. **P8-02 is complete; next: P8-03** (the simulator as a device behind this
transport, across a real process boundary) and P8-04 (the out-of-band monitor).

**Model:** the session was switched from Opus 5.5 to **Opus 5 (1M context)** by the runtime partway through this run
(after `fe6eb18`); commits from `a8040fd` on name Opus 5, as `CLAUDE.md` requires.

## 2026-10-05 — routine run 14: P8-03 — the simulator as a device process, the whole stack across a real boundary (D-V2-98)

**Built (`e92870a`):** the reference simulator moved out of `device.rs` into `delulu_runtime::sim` — `Sim` (the state and
what passes for its physics), `serve`/`answer` (the `CMD`/`READ` line protocol), `park_point` and `synthetic_signal` —
and `delulu device sim [--seed N] [--actuator DEVICE:dim=lo..hi,…] [--sensor NAME] [--json]` runs it as a DEVICE on
standard input and output. **One simulator, two callers:** the broker drives it in-process under `--broker-profile sim`,
and the verb drives the same code from the wire; the move is behaviour-preserving (the runtime's 260 unit tests and the
device suites unchanged, `cargo test -p delulu-runtime --lib` green before the verb existed).
**The bounds the bench is given are the DEVICE's, not the grant's.** `--actuator` here says what the bench can do;
the run's `--grant` says what the program may ask for. A narrower bench is a hard stop, and a run can tell the two
refusals apart: the host's envelope refusal never reaches the device at all, while the bench's comes back as `ERR` and is
reported as the hardware's ("the hardware refused the command: the bench refuses it: …"). The dead-man terms in the
spec's grant form are ignored by the bench on purpose — a lease is the host's, and a device enforcing one would be a
second dead-man nobody declared. Every answer is ONE bounded line with its control bytes escaped (TERMINAL-TEXT-1's
shape from the device's side: a device that could forge a second line would have it read as the answer to the next frame),
and a request line past 64 KiB is answered with nothing at all.
**Witnesses:** `crates/delulu/tests/sim_device_cli.rs` (three) — the same program, seed and command sequence read the
same numbers in-process and across the device process (seed 7: `-0.19014357968088236` then `0.15388365616105615`, both
ways, read by hand as well as asserted), a command outside the GRANT never reaches a bench that would have accepted it
(the bench deliberately WIDER than the grant, so a refusal can only be the host's), and the verb refuses every bench it
cannot be while `--json` describes one without serving it. Six more in `sim.rs` for the protocol itself.
**Falsified:** M154 (the bench's own bounds ignored — red on the parity witness and two unit ones), M155 (an unknown
device reads a number), M156 (the request line unbounded), M157 (the read index does not advance — the seed's promise),
M158 (the device serves nothing — red on the parity witness, which is what proves the device process is really in the
loop). **Verified:** clippy clean for the workspace; `check-other-os.sh` clean for Windows and macOS after the last code
edit; **a new SUBCOMMAND is five gates** (the run's second lesson of this shape): the help line, the dispatcher's arm and
`SUBCOMMANDS` (`completions_cli` proves the three name one set), `json_contract`'s failing sweep AND its success table
(the `--json` form is what a blind sweep can drive), and the MCP server's door rule — every command in exactly one of
`READ_ONLY` and `EFFECTORS`, so one added later cannot reach a tool unreviewed. `device` is acting (`cb79333`): it
writes no file and performs no effect, but it SERVES, as `lsp` and `mcp` do, and what it answers is what a control loop
acts on. Only the local suite sees the last two; read on the runners at `e92870a` before `master` moved — macOS `37341164996` (56 passed), Windows `37341170246` (55), arm64 `37341173453` (the 6 `sim::` tests), every new witness named; the full suite alone through
`scripts/suite.sh`, 2,154 passed, 0 failed, 16 ignored (160 binaries), cargo exit 0, the tree unmoved. **P8-03 is complete; next: P8-04**, the out-of-band monitor.

## 2026-10-05 — routine run 14: P8-04's build order, read against the code (its design question named)

With P8-03 complete and too little of the run left to build P8-04, its build order was read from the code and written
into `V2_P8_DESIGN.md` (as run 12 did for P8-01 and run 13 for P8-02). Everything the monitor READS exists
(`audit::query` over the hash-chained log, where a device command's refusal is a `deny` record on the device's own node
because `Op::Actuate` round-trips to the broker per command; PS-E-06's OCSF export is the same records), and everything
it DOES exists (`tree::revoke`, transitive, idempotent, and the fail-state and the guest's death already follow from it).

**The slice's real design question, found by the reading:** `revoke` is allowed only when the target is the caller's own
node or a DESCENDANT of it, so a monitor must be an ANCESTOR of the run — and a parent bounds its child (R-7), so an
ancestor of a device run necessarily holds `Actuate`. "A separate principal holding exactly revoke" (D-V2-54) is not
expressible in today's tree. Three ways are written out: (a) the monitor as an automated operator, buildable today and
as strong as the operator — refused; (b) a monitor node minted between the operator and the run, which can revoke its
own subtree and nothing else, with the residual named (its node holds the run's authority because a parent must; what
keeps it unused is that the monitor performs no effect, plus the same-uid boundary) — recommended, and what the next run
builds; (c) a revoke-only principal in the tree, the only form that makes the claim exact. (c) is put in front of the
OWNER rather than taken under the delegation: the argument that it hardens rather than redefines is that revocation is
monotone — a principal that can only revoke cannot widen what any program may do — but it still changes who may revoke,
and that is the Authority model's own rule (`HANDOFF.md` §1, "harden, never redefine").

Nothing built. **The next run starts at step 3, option (b).**

## 2026-10-05 — routine run 14: RW 7.17's decision half closed — the journal's detail is a gate now (D-V2-99)

RW 7.17 asked, beside its unreproduced arm64 flake, whether a probe the broker never answered deserves its own
`RevokeCause`. Read against the code: a program behaves identically for a revoked node, a non-answer and no answer (it
loses the device, the fail-state engages, it stops), so a fourth cause would be a distinction with no action behind it;
and the difference is already recorded where a person reads — the probe's own words travel verbatim from
`mint_device_nodes` into the `lease.revoked` detail in the journal. **That was a comment; it is a gate now:**
`the_journal_tells_the_three_dead_probes_apart_even_though_the_cause_cannot` asserts each of the three reasons verbatim
while asserting that the program-visible cause still collapses. Mutants M159 (the probe's reason dropped) and M160 (the
detail reduced to the cause's name) are red, and two older witnesses go red with them. The row's other half — the flake
itself — stays open for a recurrence, which is now readable.


## 2026-10-09 — routine run 15: `master` red four days on a stale map, fixed; run 14's stranded commit merged; a commit hook

**CI on arrival — red.** `75bb33b` (run 14's closing commit) push run `37344075684`: **failure** on all four test jobs
(Linux, macOS, Windows, arm64), and the nightlies of 2026-10-06 (`37444870874`), 10-07 (`37602353806`), 10-08
(`37759591993`) and 10-09 (`37914430616`) red on the same four jobs, on the same commit. Every other job of each run
green (lints, supply-chain, editor, formal, fuzz, `microvm`, `microvm-reproducible`, the Miri jobs, `miri-slow` on the
nightlies, `heavy-gates` on the nightly). Each test job's log read (Linux, macOS, Windows and arm64 of the push run,
Linux of the 10-09 nightly): **the same four failures and nothing else** — `doctor_cli`'s
`doctor_reports_a_healthy_checkout`, `check_mode_is_read_only` and `doctor_never_writes_when_the_map_is_current`, and
the Survey's `the_committed_map_matches_the_tree` (Linux 2,150 passed / 4 failed; Windows 2,087 / 4; macOS 2,124 / 4;
arm64 2,150 / 4). The VM agreed at the start of this run: `survey check` **stale** (SURVEY.md, survey.json) and
`doctor --check` one problem, the map's freshness.

**Cause.** `75bb33b`'s map was built before its last edits to documents: rebuilt on that commit alone it gains 21
Markdown lines and five edges (13,511 → 13,516) — the closing entry recorded "1,495 nodes, 13,511 edges", the numbers of
a build made before the entry itself was finished. The rule ("regenerate the map after the LAST edit") had been written
since 2026-09-28; nothing enforced it, and no run read the push run or the nightlies for four days.

**Run 14's last commit was stranded.** `6dbc54c` (RW 7.17's decision half, D-V2-99) sat on `claude/jolly-hamilton-7fd30a`
only — pushed, read on three witness runs (`37344931813` arm64 21 passed with the new test named, `37344936032` Windows,
`37344939689` macOS, all success), never on `master`. Found by step 1's branch listing (the two other branches ahead,
`7d6a22b` and `d3f0d52`, are recorded as superseded since run 10). **Merged** into `master` (a merge commit, so the hash
its runs name stays in its history), after re-verifying it here: the device tests 21 passed; M159 (the probe's reason
dropped) and M160 (the detail reduced to the cause's name) each red, three tests each, as claimed.

**The fix, `1732d9e`:** the merge with the map regenerated after it — `survey check` ok (1,495 nodes, 13,516 edges),
`doctor_cli` 11 passed, the Survey's tests and the document gates (repository_structure, evidence_claims, governance,
distribution, book, core_invariance) green, `doctor --check` all checks passed.

**Loop engineering — the rule made mechanical.** `scripts/hooks/pre-commit` (enabled per clone with `git config
core.hooksPath scripts/hooks`, now in `CLOUD_ROUTINE.md` step 2) refuses a commit whose map does not match the files
the commit HOLDS: it exports the index to a scratch directory and asks the Survey there, as CI does. Falsified three
ways: a staged document edit with no rebuild — refused; the map rebuilt and staged — accepted; and the case a working-tree
check cannot see — an UNSTAGED edit the map was built from, `survey check` on the tree **ok**, the hook **refusing** —
which is exactly how a commit carries a stale map. It reuses the built Survey unless the Survey's own sources are newer
(a `cargo build` would wait on the build lock for a whole suite); about 3 s.

## 2026-10-09 — routine run 15: P8-04 step 1 measured — an envelope's refusal was not in the chain (AUDIT-REFUSAL-1, D-V2-100); AUDIT-SEQ-1 found

**The hypothesis.** P8-04's build order (run 14) said its first step needed nothing built: "everything the monitor READS
exists … a refused command is a `deny` record on the device's own node." Measured against the binary before anything was
built on it: **false.** A daemon in a scratch state directory, a program granted `arm0/elbow` with `angle_deg=-30..95`,
one command at 12° and three at 150°: the program was told `COMMANDED` once and ``REFUSED: `angle_deg` = 150 is outside
the envelope [-30, 95]`` three times — and the chain held FOUR `use allow` records, all on the RUN's node (not the
device's), and no `deny` at all. The broker answers the device's identity (`validate.rs`: "the magnitudes never reach this
layer") and records `allow`; the run's device broker judges the envelope afterwards, and that decision went only to
standard error. **AUDIT-REFUSAL-1:** the chain said three refused commands were allowed; an investigator could not tell,
and an out-of-band monitor could never see a refusal.

**The fix (`5cd4474`, D-V2-100).** `Custody::note_device_refusal` (additive, default no-op, the `note_plugin_signature`
pattern) is called after `device::actuate` returns, at both of its callers — the interpreter and a sandboxed guest's host
channel — for `command.refused` (the envelope's and the rate's refusals; a dead lease is not this event). The daemon
client sends `DeviceRefused { node, device, reason, overrides_seq }`; `Broker::record_device_refusal` writes one `use`
record, decision `deny`, on the node that used the arm, at the device, with `refused_by: envelope`, `code: DL1904`, the
reason in the words the program was told (bounded at 512 bytes on a character boundary) and `overrides_seq` — the
`use allow` this command was given, so the pair reads as what happened. It records, never decides. A refusal the broker
cannot record is said once on standard error; nothing is recorded for a node the tree does not hold.

**Witnesses.** `crates/delulu/tests/monitor_cli.rs` (P8-04's process-spawning file): the ordinary run and the
`--sandbox` run each assert 4 allows, 3 denies, every deny's fields, each citing its own command's allow, none citing the
in-envelope one — red before the fix ("the chain held no deny"). `validate.rs`: an unknown node records nothing; the
reason is cut on a character and says so. Mutants, each red: M161 (the interpreter's note removed — the ordinary witness),
M162 (the host channel's — the sandboxed one), M163 (recorded `allow`), M164 (`overrides_seq` dropped), M165 (the first
allow's seq kept for every refusal), M166 (the unknown-node refusal removed), M167 (the bound removed). clippy clean;
`check-other-os.sh` clean for Windows and macOS. **Read on the runners at `5cd4474`** over the nine targets that drive an
actuator (`monitor_cli`, `actuate_cli`, `estop_cli`, `sandbox_devices_cli`, `device_delegation_cli`, `hw_adapter_cli`,
`hw_dpx_cli`, `sim_device_cli`, `robotics_demo`): macOS `37954533949` 56 passed, Windows `37954537458` 55, arm64
`37954540761` 56 — both new witnesses named on each, no refusal left unrecorded in any output. The full suite alone: 2,158
passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved. **A trap paid for again:** the mutant loop restored each file with
`shutil.move`, which keeps the backup's OLD mtime, so cargo kept the last mutant's binary and the green re-run went red —
touch a restored file (or copy it) before rebuilding.

**AUDIT-SEQ-1 — found on the same chain, open (RW 4.44).** A sandboxed run under the daemon writes its own records
(`guest.rs`: `sandbox-launch`, `channel-violation`, `sandbox-death`), numbered "the chain's last seq + 1", while the daemon
numbers its records from its own counter. One verified chain read `1, 2, 3, 4 (sandbox-launch), 4 (use), 5 … 10, 11
(channel-violation), 12 (sandbox-death), 11 (revoke)` — seq is neither unique nor monotone along the chain. **Witnessed:**
`audit export --format ocsf --since 12` returned `sandbox-death` alone and dropped the `revoke` written after it, so an
incremental reader of the chain (a SIEM feed by `--since`, or a monitor) silently loses a revocation. The OCSF event's
`uid` is the record's hash (unique), its `metadata.sequence` is the seq. Not fixed in this run: two writers with two
counters is a design question (one numbering authority, or numbering under the append lock with the daemon's references
following it) — sketched in RW 4.44. **It decides P8-04's cursor:** a monitor resumes from a record's HASH (its position in
the chain), never from a seq (D-V2-100 item 4).

## 2026-10-09 — routine run 15: P8-04 step 3 (b) — `delulu monitor watch` quarantines a run under its node (D-V2-101)

**Built (`b0a5a80`).** The operator mints a monitor node `g_M` (`grants delegate`) and delegates each run's lease under
it (`grants delegate --parent g_M`). `delulu monitor watch --node g_M --rule envelope|denies=N/MS [--poll MS] [--for MS]
[--json]` reads the audit chain from its head at start, resuming after the last record HASH (D-V2-100), maps every node
under `g_M` to its run (the child of `g_M` on its path) from the daemon's own listing each poll, and when a rule fires on a
run's `deny` records sends `Quarantine { caller: g_M, target: <the run's node>, why }` — `Revoke` with the monitor's account
(rule, count, window, evidence seqs and hashes) carried into the revocation's own record by `Broker::revoke_saying`. The
broker's self-or-descendant rule is what confines it. It ends when `g_M` is no longer live, when `--for` elapses, or
(exit 1) when it cannot reach its broker or a quarantine is refused. A new subcommand's five gates are in the same commit.

**Witnessed (`monitor_cli.rs`).** A run probing out of its envelope under `g_M` (60 probes, a `fib(21)` burn between) is
quarantined after its third refusal — **measured: 13 ms after the third refusal was recorded, at a 25 ms poll** — and never
finishes: its arm's lease dies of the monitor's audit seq and the declared fail-state engages, then the program's next
console write faults DL1403 naming the same seq (the guest is ended — revoking a run's node takes what it held, as
`estop_cli.rs`'s parent revoke does; the first form of this witness expected the program to survive and print `REVOKED`,
and the run showed why it cannot). The revoke record is `g_M`'s, targets the run, and says ``rule `denies=3/60000` — 3
deny record(s)``; each evidence hash is an envelope refusal of that run. A monitor holding a SIBLING node with the
`envelope` rule read the run's records (`records_read` ≥ 11 — asserted, so "quarantined nothing" cannot mean "read
nothing") and quarantined nothing. Unit: the subtree-to-run map; the rules' parser. Mutants, each red: M168 (the subtree
check removed — both witnesses), M169 (the window never fills), M170 (the monitor's own node targeted), M171 (the run
revokes itself — the record's actor is wrong), M172 (allowed uses counted), M173 (the why dropped), M174 (nothing read —
the sibling witness's vacuity). clippy clean; `check-other-os.sh` clean for Windows and macOS after the last code edit.
**Read on the runners at `b0a5a80`** over `monitor_cli`, `json_contract`, `completions_cli`, `estop_cli`, `grants_cli`,
`broker_cli` and `bin:delulu`: macOS `37957000733` 198 passed, Windows `37957004727` 182, arm64 `37957008755` 217 — every
new test named; and `delulu-broker`'s unit tests on Windows `37957012378`, 170 passed. **The quarantine, measured on each
runner at a 25 ms poll:** Windows 11 ms, arm64 24 ms, macOS 107 ms after the third refusal was recorded (the VM: 13 ms).
The Windows broker read also showed a PRE-EXISTING compiler warning in every Windows test build — `secrets.rs`'s
`in_memory_at` unused, its only caller a Unix-only FIFO test — which `check-other-os.sh` never saw, because it lints
`delulu`, `delulu-runtime` and `delulu-wasm` but not `delulu-broker` (fixed in the next commit).
**The full suite alone:** 2,162 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved.

**Not built: step 5, the monitor's own death** — a run whose monitor goes quiet is not yet treated as one whose monitor
fired; and the study's other two rules (break-glass use, special-use reach) wait until their records are measured the way
step 1's were. **Residual, named (D-V2-101):** `g_M` holds the runs' authority because a parent bounds its child; the
monitor performs nothing but a revoke, and the same-uid boundary is what keeps it so. Option (c) stays the owner's.

## 2026-10-09 — routine run 15: loop engineering — `scripts/mutants.py`; the cross-OS lint reaches `delulu-broker`

**`scripts/mutants.py SPEC.json [NAME…]`.** This run wrote two mutant loops by hand and paid the restore-mtime trap once
(the green re-run went red on the last mutant's binary). The script takes the mutants as data — file, exact anchor,
replacement, an optional per-mutant command — refuses an anchor that is not unique, tells a mutant that does not compile
(BUILD-ERROR, not a falsification) from a red witness, restores each file byte for byte with a NEW mtime, and ends with a
CONTROL run of the same command on the restored tree, which must be green. Exercised on M163 and M172 (both RED, the
failing tests named), a non-unique anchor (refused, not run), a non-compiling mutant (BUILD-ERROR) and the control
(green, 4 passed) — 28 s.

**`check-other-os.sh` now lints `delulu-broker`.** The Windows read of the broker's unit tests (`37957012378`) showed a
warning that had been in every Windows test build: `secrets.rs`'s `in_memory_at`, used only by a Unix-only FIFO test,
was dead code there — and the script that lints the other operating systems named three packages, not the broker.
Witnessed: with the broker added, the lint was RED on the unfixed source (``associated function `in_memory_at` is never
used``), and green once the helper is `cfg(all(test, unix))`, for Windows and macOS. The broker's 172 unit tests pass.

## 2026-10-09 — routine run 15: AUDIT-SEQ-1 closed — the audit chain has one numbering (D-V2-102, RW 4.44)

**Wider than first recorded.** Reading where the daemon's count starts (`tree.rs`: `audit_seq: 1`) showed the defect
needs no sandbox at all: a daemon started, stopped and started again on one state directory wrote `1, 2, 3, 1, 2, 3`,
the chain still verified, and `audit export --format ocsf --since 3` returned both `delegate` records and dropped the
second daemon's first two. With a sandboxed run beside the second daemon (its host numbered "last + 1", read outside the
append lock) the witness chain read `1, 2, 3, 1, 2, 3 (attenuate), 4 (sandbox-launch), 4 (use) … 11 (channel-violation),
12 (sandbox-death), 11 (revoke), 12, 13`.

**The fix (`2546425`, D-V2-102).** The number is settled where the lock is: `AuditLog` keeps the highest seq the chain
holds (recovered with the head whenever another writer moved the chain) and `append` gives each record `max(its seq,
highest + 1)` under the append lock — a writer that asks for none (the sandbox host now passes 0; an adapter's provenance
record always did, and said seq 0) is given the next, one whose count is behind is moved past. And so the daemon's own
references stay true, `AuditSink::next_seq` (default `None`; `AuditLog` answers under its lock) is a floor `Broker::take_seq`
takes before each seq: a restarted daemon resumes after the chain's highest seq, and a revoked node's `revoked_by_seq` —
what DL1403 tells a program — is the seq of its revocation's record.

**Witnesses.** `audit.rs`: two handles and a reopened log — `1, 2`, a writer asking for none gets `3`, the stale daemon's
`3` becomes `4`, the reopened log resumes at `5`, the chain verifies. `monitor_cli.rs`: two daemons on one chain and a
sandboxed run beside the second — seqs strictly increasing along the whole chain, `--since` from the middle returns
exactly the records from there on (compared by hash), and the revoked node's `by_seq` equals its revocation's seq. Red
before the fix with the chain above. Mutants, run through `scripts/mutants.py` (control green): M175 (append keeps the
caller's seq — both witnesses), M176 (the daemon's floor dropped — red only through the reference check: uniqueness alone
would have let it live), M177 (catch-up forgets the highest seq), M178 (recovery does not compute it) — each red. clippy
clean; `check-other-os.sh` (now with `delulu-broker`) clean for Windows and macOS.
**Read on the runners at `2546425`** over the fourteen targets that read a seq (`monitor_cli`, `audit_cli`,
`audit_ocsf_cli`, `break_glass_cli`, `broker_cli`, `federation_cli`, `grants_cli`, `guard_e2e`, `sandbox_confirm_cli`,
`sandbox_external_cli`, `sandbox_guard_e2e`, `sandbox_run_cli`, `secret_verify_cli`, `estop_cli`): macOS `37959503886`
90 passed, Windows `37959508618` 83, arm64 `37959514499` 93 — the new witness named on each; `delulu-broker`'s unit tests
on macOS `37959518422`, 173 passed, the new unit witness named.
**The full suite alone:** 2,164 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved.

**What it leaves.** A chain written BEFORE this fix may hold repeated seqs; it still verifies, and the next record
continues after its highest. The P8-04 monitor keeps its hash cursor (D-V2-100) — it also detects a chain rewritten under
it, which a seq cannot.

## 2026-10-09 — routine run 15: HTTP-SCHEME-1 — a plain-`http://` fetch died under the broker daemon

**Found while measuring P8-04's special-use rule.** A runtime special-use refusal needs a NAME that only resolves to a
special-use address (the grant parser refuses `127.0.0.1` and `localhost` outright), so the VM's `/etc/hosts` was pointed
at loopback for one ordinary-looking name (restored afterwards) — and the first run, with an `http://` URL, did not reach
the egress client at all: under `--broker daemon` it died DL0904, "lease … does not grant `http` in scope `net`", and the
chain held `use deny` with target `http`. The same program under embedded custody printed "not delivered" and finished
(`only https:// is fetched [egress: scheme]`). **Cause:** `custody_op_for` takes the host with `prim::host_of`, which strips
only `https://`; for `http://host/x` the "host" is `http`. Embedded custody checks the scheme first and answers a value;
the daemon's gate runs before the effect and asked the broker about `http`. Fail-closed both ways — but under daemon
custody, and so in every `--lease` run, a program could not receive the refusal it is documented to receive, and the
chain named a host nobody asked for.

**The fix (`e7921cb`):** `custody_op_for` mirrors the effect — a URL that is not `https://` is refused by `prim` before
anything is reached, so no custody op is asked. **Witness** (`monitor_cli.rs`): the program embedded (the control), under
the daemon, and sandboxed under the daemon — each prints "not delivered" and finishes, with the scheme's reason, and no
record names `http`. Red before the fix; mutant M179 (the guard removed) red through `scripts/mutants.py`, control green.
**Read on the runners at `e7921cb`** over `monitor_cli`, `egress_cli` and `sandbox_run_cli`: macOS `37961192830` 28
passed, Windows `37961196373` 29, arm64 `37961200403` 29 — the new witness named on each (and the quarantine measured
again: 22, 22 and 5 ms). clippy clean; `check-other-os.sh` clean for Windows and macOS after the last code edit.
**The full suite alone:** 2,165 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved.

**The special-use measurement itself is not finished:** with an `https://` URL to the name pointed at loopback, a run
would reach the egress client's special-use refusal — whether the chain records it (the custody gate records `use allow`
for the host first, as it did for the envelope) is the next measurement, needed before the monitor's `special-use` rule.

## 2026-10-09 — routine run 15: P8-04's special-use rule measured — the egress client's refusal is not in the chain (RW 4.46, open)

With `https://` this time, to a name the VM's `/etc/hosts` pointed at loopback for one run (restored, checked): the
program was told "not delivered" and the operator's stderr said why (`the host is, or resolved to, a special-use address
(loopback (127.0.0.0/8)) … [egress: special-use]`), the run's report carries it — and the chain holds `use allow
dlx-special-witness.com` and nothing else. AUDIT-REFUSAL-1's shape, for the egress client: the custody gate records the
host as allowed, and the refusal that follows is decided in the run. **Not fixed in this run:** the fix has the same
shape as D-V2-100's (a custody note after the effect, a `deny` citing the `allow`), but its witness needs a name that
resolves to a special-use address on every runner, and the tests have no resolver seam — the egress client resolves
through the operating system on purpose. Recorded as RW 4.46 with the measurement; P8-04's `special-use` rule waits on
it. The `break-glass` rule waits on its own gap: `audit_required` writes a break-glass record with no node at all, so a
monitor cannot attribute it to a run under its node.

## 2026-10-09 — routine run 15: the red-team pass on this run's work — ACTOR-CUSTODY-1 found; F3 fixed; AUDIT-SEQ-1's references corrected

**The pass.** One Sonnet 5.5 sous-chef (`sonnet`, the routine's agent model), about 27 minutes, against a frozen copy of
the binary at `73203fc` in a scratch directory outside the repository (`git status` clean after it), briefed to break the
monitor's confinement, the envelope-refusal records, the chain's one numbering and the `http://` fix, and to list every
oddity. Each finding below was re-run by the head chef against the current binary before a word of it was recorded.

**F1 — ACTOR-CUSTODY-1 (HIGH, pre-existing, open; RW 4.47).** Re-run: a run under a lease whose ACTOR loops 60 in-envelope
actuator commands; `grants revoke` of the run's node at t = 3 s printed "revoked 2 node(s)" and the device journal said
`lease revoked (operator-revoke) … by audit seq 7` — and the actor was told `COMMANDED` 60 times of 60 and the run exited
0; the same loop in `main` dies DL1403 after the revoke. Cause, read in `actors.rs`'s `worker_loop`: an actor's worker
interpreter is built with NO custody and NO device broker, so its commands are checked against the envelope and then
answered `Ok` by `device::actuate`'s no-broker branch — never reaching a device, never checked against the daemon (so a
revocation, the e-stop, does not reach an actor's uses — its network uses too: the red team saw no `use` record for an
actor's fetch), never recorded. Physically fail-safe for a device (no command reaches one), but the program is told its
command landed, and the authority model's "revocation reaches the next use" does not hold for an actor's effects under
daemon custody. **The next run's first item.**

**F2 — AUDIT-SEQ-1's references under contention (MEDIUM, open; RW 4.48).** Re-run: 72 runs in parallel beside the daemon
(60 sandboxed, 12 not): 613 records, 0 duplicate seqs, 0 out of order, the chain verifies — and 2 wrong references: one
node's `by_seq` names a `sandbox-death` record, another's `audit_seq` a `sandbox-launch`. D-V2-102 item 2 named this race
("a true race between that floor and the append") and called it microseconds; under contention it is hit about once in 36
runs. The numbering holds; the daemon's references do not, in that window. The fix is to hold the append lock from the
floor to the write, within one daemon operation — D-V2-102 is corrected, not softened.

**F3 — a monitor quarantined its own node (MEDIUM, fixed: `495204e`).** Re-run: `denies=1/60000`, one refused delegation
under `g_M` (a `deny` whose actor is `g_M` itself) → the monitor revoked `g_M` and the innocent run under it. Records whose
actor is the monitor's own node are skipped now; witness `a_refusal_of_the_monitors_own_node_quarantines_nothing` red
before the fix; M180 red, control green.
**F3 read on the runners at `495204e`** over `monitor_cli` and `bin:delulu`: macOS `37965843494`, Windows `37965846952` (148 unit tests and 7 in `monitor_cli`), arm64 `37965849965` — the new witness named on each; clippy clean; `check-other-os.sh` clean for Windows and macOS.
**The full suite alone at `495204e`:** 2,166 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved.

**F4–F6 (LOW, not re-run; code reading agrees; RW 4.49).** F4: the monitor reads the chain with `audit::query` and never
verifies it, so a same-uid process that appends a forged `deny` line makes it quarantine an innocent run (the chain then
fails `audit verify`) — category 7, but a monitor could verify before acting. F5: each poll re-reads the whole chain; the
quarantine's latency grew to about 200 ms at 10,000 records, and one more refusal slipped through before the revoke. F6:
`record_device_refusal` answers `Recorded` even when the audit sink's append failed (`record_op` logs and continues).

**What held** (the red team's counts): the monitor's confinement in about 22 experiments (siblings, nesting three deep,
two bad runs and one good, twelve monitors on one node, missing/revoked/expired/odd node names, a broker restarted
mid-watch, a truncated chain, junk files in the audit directory); about 8,500 envelope refusals recorded with correct
`overrides_seq` (NaN, ±inf, 1e300, unlisted dimensions, non-numeric fields, rate refusals, 2,500 in one run, ordinary and
sandboxed); 0 duplicate or out-of-order seqs over 10,000+ records, five daemon restarts and a `kill -9`, and `--since`
matching `query` for 30 values; the `http://` fix across 36 URL spellings in three modes (108 runs). **Oddities kept for
judgment:** a command that omits a bounded dimension is accepted (the envelope bounds what is commanded); an audit-sink
failure reaches a program as `LeaseRevoked`; the scheme refusal leaves no record (as the special-use one, RW 4.46);
`denies=+1/+5` parses; a monitor ends on its first broker error, with no retry; the root or `g_M` itself may be named as
`--node` without a guard.

## 2026-10-10 — routine run 16: CI read green; ACTOR-CUSTODY-1 closed — an actor's effects answer to the run's custody and brokers (D-V2-103, RW 4.47)

**Health and the previous run, verified first.** `survey check` ok (1,499 nodes, 13,627 edges), `doctor --check` all checks
passed, `cargo deny --all-features check advisories` ok; `gh` is installed in this VM (`/usr/local/bin/gh`) and was not
needed — the MCP tools and the REST API answered everything. Run 15's last three push runs, unread at its close, are green
on every job: `da6518c` `37962831019`, `b0597d3` `37964495079`, `742b11b` `37967062925`. The nightlies of 10-06 to 10-09
(`37444870874`, `37602353806`, `37759591993`, `37914430616`) are red on `75bb33b` — the stale map run 15 found and fixed in
`1732d9e`, recorded there; the 10-10 nightly had not fired when this run began (it fires about 09:45 UTC). Branches: the two
with a commit `master` lacks (`7d6a22b`, `d3f0d52`) are superseded since run 10; the `v1.0.0` tag and `rc/1.0.0-drill` the
unshallowed fetch listed are the owner's own of 2026-07-19/20 (`HANDOFF.md` §11.3), not a run's. Run 15's new tests ran in
this run's full suite. The container restarted once mid-run and stopped the first suite; nothing on disk was lost.

**ACTOR-CUSTODY-1, measured again on `742b11b`'s binary before a line changed.** The e-stop supervisor's 14-command loop
moved into an actor, under `--broker daemon`, nobody revoking: 14 `COMMANDED` and **0** `use` records in the chain — the
same loop in `main` leaves 14 `use allow … arm0/elbow`. Without any daemon, under `rate_hz=1`: `main`'s second command is
refused ("granted at most 1 Hz; this command arrived 41 µs after the last") while the actor's two commands both answered
`COMMANDED`. So the defect was wider than the red team's reading: not only under daemon custody — in EVERY mode an actor's
actuator command was answered `Ok` by nobody (no lease, no rate bound, no dead-man, no device), and an actor's compute
dispatch answered `NoAdapter` beside a bound adapter.

**The fix (D-V2-103).** `ActorSystem::start_governed` takes `WorkerEffects` — a factory that builds each worker's custody on
the worker's own thread, and the run's device and compute brokers (`Arc`s, shared as `main`'s are). `delulu run` passes
`BrokerClientCustody::same_node_clients()` — one client per worker for the run's own node, each starting with an empty epoch
cache, so a revocation that already happened is seen before its first use — and its `devices` and `computes`.
`ActorSystem::start` and `start_with` keep their meaning (`WorkerEffects::default()`: embedded pass-through custody and no
brokers — `main`'s own interpreter when nothing is attached to it).

**Witnesses — five, red first.**
- `estop_cli` `an_actors_commands_are_uses_of_the_runs_node_in_the_audit_chain` — red on the old code: 0 of 14 in the chain;
- `estop_cli` `an_operator_revoke_of_the_device_stops_an_actors_arm_too` — red: the device journal said `lease.revoked …
  operator-revoke` and the actor never saw `REVOKED`; the operator acts once the ACTOR has printed `COMMANDED` (its output is
  read line by line), never on a guess about timing;
- `estop_cli` `revoking_the_runs_node_ends_an_actors_effects` — red: no `[DL1403]`, the actor drove on to `ACTOR DOWN`;
- `actuate_cli` `an_actors_commands_answer_to_the_runs_device_broker_and_its_rate_bound` and `compute_cli`
  `an_actor_dispatches_through_the_runs_compute_broker` — written after the fix; their red is the by-hand probe above (the
  rate) and the mutants that remove exactly the attachment the old code lacked.

**Mutants** (`scripts/mutants.py`): M181 (`run_cmd` passes no custody) and M183 (the worker drops it) — red on the chain and
run-node witnesses; M182 (no device broker) and M184 (the worker drops it) — red on the rate and device-revoke witnesses;
M185 and M186 (no compute broker) — red on the dispatch witness; the control green after each set. M182's first read named
only the rate witness: cargo stopped at the first red TARGET, so the device-revoke witness never ran under the mutant —
`scripts/mutants.py` now adds `--no-fail-fast` itself (loop engineering, below).

**Runner reads at `33f0518`** over `estop_cli`, `actuate_cli`, `compute_cli`, `actors_cli`, `actors_bounded_cli` and
`actors_cycles_cli`: macOS `38009025737` (49 passed, 0 failed), Windows `38009027757` (49 passed, 0 failed), arm64 `38009029739` (49, 0) —
the five new witnesses named on each, no compiler warning. Clippy clean; `check-other-os.sh` clean for Windows and macOS.
**The full suite alone at `33f0518`:** 2,171 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved (`scripts/suite.sh`).

**Not changed, recorded.** An actor still gets no plugin engine and no foreign bindings (code reading), so an actor's `load`
and `root.foreign(load)` fail closed rather than run ungoverned. **A sibling, a hypothesis until witnessed (RW 4.50,
PLUGIN-CUSTODY-1):** a plugin export runs in a nested interpreter (`interp.rs` `call_plugin_export`) built with the host's
trace and effect sink and NO custody and no brokers, and an export may take `Cap[_]` parameters — so a host that hands one a
`Cap[Actuator]` may see its commands answered `Ok` by nobody, and under the daemon none of a plugin's uses checked or recorded.

**The lesson.** `Interp::new` is a pass-through custody and no brokers by default, so every place an interpreter is built
other than `run_cmd`'s is a place the run's authority can silently stop: an actor's worker, a plugin's nested interpreter.
Ask of each `Interp::new` what custody and brokers it holds.

## 2026-10-10 — routine run 16 (continued): PLUGIN-CUSTODY-1 witnessed and closed — a plugin export runs under the host's custody and devices; plugins callable under the daemon (D-V2-104, RW 4.50)

**The hypothesis (RW 4.50, opened by this run's ACTOR-CUSTODY-1 entry), witnessed on `2eb7fca`'s binary.** A Verified
plugin `drive` whose one export, `twice(a: Cap[Actuator]) -> Str ! {Actuate}`, commands the actuator its host hands it,
twice. Embedded custody, `rate_hz=1`: the host's own second command was refused ("granted at most 1 Hz; this command arrived
75 µs after the last") and the plugin's two commands, milliseconds later, both answered `COMMANDED` — the export ran in a
nested interpreter with no device broker, so nothing held its commands to the lease or the rate, and none reached a device.

**Under the broker daemon, a second defect stood in front of the first (PLUGIN-DAEMON-1).** The same host under `--broker
daemon` printed `Revoked`: no plugin had EVER been callable under the daemon. `BrokerClientCustody` never implemented R-6c's
`liveness`, and the trait's fail-closed default (`Unknown`) refused every `p.get` as `Revoked(0)`. P2 recorded this path as
untested ("no daemon-mode test of its own", `V2_LOG.md` P2, *Open, and deliberately*), while D-V2-27 item 4 describes the
holder check as working "in embedded and daemon custody alike" — true of the node's creation (the daemon's tree held it,
seq 6 in the probe), never of a call. **Fixed:** `liveness` asks the daemon (one `NodeState` round-trip per call; live,
revoked with its seq, or anything else `Unknown` — dead). **And with only that fixed, the custody half of RW 4.50 went red**:
the plugin was called and the audit chain held **0** of its 2 commands — the nested interpreter's fresh embedded custody
allows everything and records nothing. A fail-closed path had been hiding the defect behind it.

**The fix (D-V2-104).** The nested interpreter shares the HOST's custody (the field is an `Rc<RefCell<…>>` now; the host is
suspended while the export runs, and no borrow is held across the call) and the run's device broker. A plugin's use of a
capability its host handed it is checked and recorded exactly as the host's own use of it is — the run's node's use.
`load` still refuses grants carrying `Declassify` or `ForeignCall`: custody alone decides those, and the host's custody
answers for the RUN's node, which may hold more than the plugin's grant. Compute is not passed: a plugin cannot be granted
`ForeignCall`, so it cannot dispatch.

**Witnesses — three in `plugin_load_cli`, red first:** `a_plugins_commands_answer_to_the_runs_device_broker_and_its_rate_bound`
(red: `COMMANDED | COMMANDED`); `under_the_broker_daemon_a_plugin_is_called_and_its_commands_are_uses_of_the_run` (red:
`NOT CALLED: Revoked 0`; then, with liveness alone, 0 of 2 uses in the chain); `under_the_broker_daemon_unloading_kills_a_callable_the_host_already_holds`
(red: the first call never succeeded). **Mutants:** M187 (the nested interpreter drops the device broker) red on the rate
witness; M188 (a revoked node answers `Live`) red on the unload witness; M189 (custody not shared) red on the chain witness;
M190 (a live node answers `Unknown` — the old behaviour) red on both daemon witnesses; the control green.
**Runner reads at `8f2b848`** over `plugin_load_cli`, `plugin_cli`, `hw_dpx_cli`, `locale_plugin_cli`, `examples_run` and
`estop_cli`: macOS `38011420070` (`plugin_load_cli` 14 of 14, the three new witnesses named; the job green over all six targets), Windows `38011422634` (the same), arm64 `38011424714` (47 passed, 0 failed). Clippy clean; `check-other-os.sh` clean for Windows and macOS;
`delulu-runtime`'s 261 unit tests green. **The full suite alone at `8f2b848`:** 2,174 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved.

**What stays open.** A revocation that lands WHILE an export runs now reaches its next use (the custody is the host's); no
test holds an export long enough to witness that window by itself — the chain witness proves the custody is the daemon's.
The plugin's node is not revoked at the end of a run under the daemon (it stays `live` in the tree, as device nodes once
did — RW 4.51).

**The lesson.** A path that fails closed on everything hides what stands behind it: nobody looks at a refusal that is safe.
When a fail-closed path is made to work, witness again what it was in front of.

## 2026-10-10 — routine run 16 (continued): RW 4.48 closed — the daemon holds the audit chain from a seq's floor to its write; AUDIT-DAY-1 found and closed — a record's day file follows the chain (D-V2-105)

**RW 4.48, measured again before a line changed.** The red team's F2 (run 15) as a probe: one daemon, 60 sandboxed runs
(their hosts write `sandbox-launch` and `sandbox-death` beside the daemon) and 12 daemon runs holding an actuator, 24 at a
time; then every node's `audit_seq` (its creation record) and every revoked node's `by_seq` checked against the chain. On
`80c566c`'s binary: **2 and 3 wrong references** in two probes (169 records each, the chain verified, no duplicate seq) —
nodes whose creation seq named a `sandbox-death` or `sandbox-launch` record. The cause, D-V2-102 item 2's own words: the
broker takes the chain's next seq as a floor, releases the append lock, stamps the seq into a node and an answer, and appends
later — a host's record landing between takes the seq, the daemon's is renumbered at the lock, and its references name the
host's.

**The fix (D-V2-105).** `AuditLog` gains `take_lock`, `next_seq_under` and `append_under` — a held `AppendLock` is the token,
and one for another chain is refused. The daemon's `TrackingSink` takes the lock at a request's first seq or record and
keeps it to the last; the serve loop releases it after `handle` and BEFORE the reply is written (a client that reads its
answer slowly never holds the chain), and once after the startup record. **On this commit's binary the probe found 0 wrong
references in three runs.**

**Witnesses** (both in `brokerd.rs`'s tests): `a_seq_the_daemon_reserved_is_the_seq_its_record_is_written_under` — another
writer is started inside the window and given 300 ms; red first (reserved 2, written 3); `the_daemon_leaves_the_chain_free_between_requests`
— after the daemon starts, and after a request that wrote a record, another writer appends within 2 s (were the lock kept,
it would wait out its 5 s bound and lose its record). **Mutants:** M191 (the sink's floor stops holding the lock) red on the
first; M192 (no release after a request) and M193 (none after the startup record) red on the second; the control green.

**AUDIT-DAY-1 — found while writing the second witness.** Its first form stamped the other writer's records with a fixed
time — which fell on the previous UTC day — and `AuditLog::open` then refused the chain as truncated. The test was wrong; the
question it raised was not: a record's day FILE was chosen by its own stamp, and a record is stamped before it waits for the
append lock (the daemon's up to 5 s). Near midnight a record stamped 23:59:59.9 can be written after one stamped 00:00:00.5;
it went into the EARLIER day's file, behind records it chains after, and `verify` — and every writer's recovery, so the
daemon's fail-stop-on-record — read the whole chain as broken. Witness `audit::tests::a_record_stamped_before_midnight_but_written_after_keeps_the_chain_whole`,
red first (a `prev_hash` chain break at seq 3). **Fixed:** a record is filed under the later of its own day and the chain's
current day; it keeps its own stamp. M194 (filed by its own day again) red; the control green.

**Runner reads at `7038885`:** macOS `38013294529` (198 passed) and `38013300816` (`delulu-broker` lib, 174), Windows `38013296565` (184) and `38013302335` (172), arm64 `38013298889` (217) and `38013303853` (174) — 0 failed on each, the three new witnesses named over `bin:delulu`, `audit_cli`, `audit_ocsf_cli`, `monitor_cli` and `estop_cli` and the broker's lib. Clippy clean; `check-other-os.sh` clean for Windows and macOS. **The full suite
alone at `7038885`:** 2,177 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved at its end — but touched DURING it: a test file was edited and one target compiled while it ran (a separate binary hash; the file restored to HEAD before the suite reached it, which ran its own build), recorded in §11.5.

**The lesson.** A key derived from a timestamp — a day file, a partition — is an ORDER claim, and a clock is not an order:
ask what a record stamped before the last one does. And a fixed date in a test is a date: this one fell on yesterday, and
that is how the defect showed itself.

## 2026-10-10 — routine run 16 (closing): RW 4.51 closed — a plugin's grant node does not outlive its run; the run's close

**RW 4.51, measured by this run's PLUGIN-CUSTODY-1 probe.** After a run under the daemon that loaded and called a plugin,
`grants list` showed the plugin's node `[live]` beside the device node `[revoked@9]`: a live grant nobody holds, offered to an
operator reading the tree — the shape device nodes had before 10g's `a_device_node_does_not_outlive_the_run_that_minted_it`.
Reachable only since D-V2-104 made plugins callable under the daemon. **Fixed** (no new decision: 10g's rule for device nodes,
applied to plugin nodes): `Custody::end_of_run` (a no-op by default — embedded custody's tree ends with the process); the
daemon's client remembers each plugin node it attenuated, and when the run ends each revokes itself (a node may always revoke
itself; idempotent after an `unload`). `delulu run` calls it once `main` and every actor are done. **Witness**
`plugin_load_cli` `under_the_broker_daemon_a_plugins_node_does_not_outlive_its_run`, red first (`live`); **mutants** M195
(`run` never calls it), M196 (it revokes nothing), M197 (minted nodes are not remembered) red, the control green.

**Its first form (`d164a68`) broke a control elsewhere — caught by the full suite and by all three runners, never on
`master`.** It remembered EVERY node `attenuate` minted, and a run's device nodes are minted through `attenuate` too
(`mint_device_nodes`): `end_of_run` revoked them while the device watchdog still ran, which read it as an operator's e-stop —
`estop_cli`'s `with_nobody_revoking_anything_the_same_supervisor_keeps_its_arm` lost its arm to an "operator-revoke" nobody
made (suite 5 locally; runner reads `38014455713`, `38014457962`, `38014459712`, all red on that test). The slice's own witness
and mutants were green throughout: the defect was in a caller the slice never looked at. **Fixed in `d7e2b8f`:** only a
holder of kind `plugin` is remembered; a device's node stays `close_devices`' to revoke, after the device broker stops. A
deterministic witness beside the timing one — `brokerd::tests::at_the_end_of_a_run_its_plugin_nodes_are_revoked_and_its_device_nodes_left_to_the_device_close`
— red under M198 (every node remembered again); `estop_cli` green three times in a row locally.
**Runner reads at `d7e2b8f`:** macOS `38015462027` (194 passed), Windows `38015464061` (180), arm64 `38015466138` (213) — 0 failed on each over `bin:delulu`, `plugin_load_cli`, `estop_cli` and `examples_run`, both witnesses and `estop_cli`'s control named green. Clippy clean; `check-other-os.sh` clean for Windows and macOS. **The full suite
alone at `d7e2b8f`:** 2,179 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved.

**This run's push runs:** `33f0518` `38010274353`, `2eb7fca` `38010619232`, `80c566c` `38012262994`, `7038885` `38013916987` (its OCSF run `38013916957`), `9eb6f90` `38014095655` — success on every job; `d7e2b8f`'s and the closing commit's are read by the closing entry or the next run.

**Where the next run starts — P8-04 step 5, option (c), sized by this run and not begun** (too large to finish verified inside
what remained): a broker dead-man for a node. Its pieces, read against the code: two IPC requests (`ArmDeadman { node,
period_ms, on_death }` and `Beat { node }`, a disarm on a clean exit); the daemon's serve loop waits on `requests.recv()`, so
the timer is a `recv_timeout` at the nearest deadline, and an expiry is `tree::revoke(node, node)` with the cause recorded
(arming a FUTURE self-revocation adds no authority — any node may revoke itself); `monitor watch --on-monitor-death
quarantine|continue` (the default `quarantine`; `continue` arms none) arms it on `g_M` and beats each poll. A new flag on an
existing subcommand: `json_contract`'s tables and the help text, not the five gates. Witness: kill a monitor mid-run (its
run under `g_M` loses its node within a measured bound, the cause in the chain); `continue` keeps it; a clean exit disarms.

## 2026-10-10 — routine run 16 (postscript): the push runs read; the red-team pass lost to a restart; RW 4.53 measured

**Push runs read, all success on every job:** `d7e2b8f` `38016042725` and the closing commit `dd724f6` `38016175557` (with
the run's earlier six, recorded in its closing entry). `master` is green at the close.

**The red-team pass was lost.** One Sonnet 5.5 sous-chef, launched at `dd724f6` against a frozen copy of the binary in
`/tmp`, was about fifteen minutes in when the container restarted (the run's second restart) and killed it; it left
experiment files in its scratch directory and no report — so NO finding of it is recorded, and the pass is the next run's to
redo. **Loop engineering:** a sous-chef's brief now says to append each finding to a file in its scratch directory as it is
found (`docs/CLOUD_ROUTINE.md` step 5), so a restart loses the time and not the findings.

**One of its experiments, re-run by the head chef (RW 4.53).** It timed a daemon run while N loops of sandboxed runs wrote
their records beside the daemon. Re-measured on both binaries, 15 daemon runs each, no run failed, every chain verified:

| | median | p90 | max |
|---|---|---|---|
| 8 sandbox loops, `80c566c` (before RW 4.48) | 0.205 s | 0.879 s | 0.916 s |
| 8 sandbox loops, `7038885` (after) | 0.123 s | 0.315 s | 1.361 s |
| 12 sandbox loops, `80c566c` | 0.388 s | 0.883 s | 1.290 s |
| 12 sandbox loops, `7038885` | 0.167 s | 0.456 s | 1.077 s |
| control: 12 loops of plain runs (no chain writes), `7038885` | 0.067 s | 0.080 s | 0.085 s |

So the slowdown is contention on the audit chain, not the CPU, and it predates this run — RW 4.48's per-request lock made
the medians better, not worse. The sous-chef's own files show one 9.6 s run among twelve loops; not reproduced here (max
1.3 s). Code reading names a likely cause: every writer's `catch_up` re-reads the whole chain whenever another writer moved
the anchor, so each daemon request beside busy sandbox hosts pays a full re-read under the lock — and the operator's
e-stop revoke is one such request. Recorded as RW 4.53 (measured, not fixed).

## 2026-10-10 — routine run 17: CI read green; P8-04 step 5 — a dead monitor quarantines: a dead-man for its node, held by the broker (D-V2-106)

**CI read first.** Run 16's closing commit `ed7f7b6` (unread at its close): push run `38017544108` — success on every job.
The 2026-10-10 nightly had not fired when the run began (05:08 UTC). Every `origin/claude/*` branch listed after unshallowing
`master`: none holds a commit `master` lacks except the two witness branches already recorded as superseded (V2_LOG
2026-10-04). Health: `survey check` ok, `doctor --check` all checks passed.

**What was missing.** P8-04's monitor (D-V2-101) quarantines a run that misbehaves; nothing happened when the MONITOR died.
**Witnessed red on the pre-fix monitor:** a monitor on `g_M` killed while a well-behaved run under `g_M` kept commanding its
arm — 20 s later the run's node was still live, and the run went on unwatched
(`a_killed_monitors_runs_are_revoked_by_its_broker_and_the_chain_says_why`); the `continue` and clean-end witnesses were red
the same way (an unknown flag; no `deadman-*` record in the chain).

**Built (option (c) of `V2_P8_DESIGN.md` P8-04 step 5; D-V2-106).** `delulu-broker`'s `deadman.rs`: a dead-man FOR A NODE —
`arm_deadman` (recorded, `deadman-arm`, with the period), `beat_deadman`, `disarm_deadman` (recorded, `deadman-disarm`) and
`fire_expired_deadmen`, which revokes the node AS ITSELF with the cause in the record (`why: "dead-man: … no beat within N
ms …"`). Any node may revoke itself, so arming a future self-revocation adds no authority. A beat is a proof of possession:
arming returns a 128-bit key the daemon keeps only as its BLAKE3 hash; a beat or disarm with another key, a second arming, and a
late beat change nothing — a late beat FIRES it. Deadlines are monotonic `Instant`s. The daemon: `ArmDeadman`, `Beat`,
`DisarmDeadman` over IPC (misuse DL1401, a fired one DL1403); its loop waits for a request no longer than the nearest deadline
and fires every dead-man past due BEFORE it handles the next request. `delulu monitor watch --on-monitor-death
quarantine|continue` (default `quarantine`) and `--death-after MS` (default `max(2000, 8 × poll)`): it arms before it says
"watching", beats before and after each chain read, disarms on a clean end (`--for`), leaves the dead-man armed when it ends on
an error, exits 1 and says so when its node was revoked by its OWN dead-man (read from the chain — a monitor that was stopped
must not end as if asked to), and ends with an error when its node vanished from the tree (a broker restarted — it had gone on
watching nothing, an oddity of run 15's red team). On Linux it makes itself non-dumpable before it holds the key.

**Witnesses** (`monitor_cli`): a killed monitor's run is revoked by its broker, the arm parks, the program dies DL1403 naming
the dead-man's seq, and the chain holds the arming and the revocation's `why` — **measured 2,025 ms after the kill** (period
2,000 ms, poll 25 ms; on the runners 2,018 / 1,990 / 2,187 / 2,007 ms — a bound measured from the kill can be a poll SHORTER than
the period, because the last beat precedes the kill); `--on-monitor-death continue` keeps the run going a full period after the
kill, with no arming in the chain; `--for` disarms (recorded) and the run lives on; a STOPPED monitor (SIGSTOP past its period)
finds its node revoked by its own dead-man on resuming, exits 1 and says so (Unix); the monitor's memory is closed to its own user
on Linux (`a_monitors_memory_is_closed_to_its_own_user` — **read red on a non-root `ubuntu-24.04` runner at `c299e47`,
`38028218741`**: "a process of the same user opened the memory of a monitor holding a dead-man's key"; green with the
`PR_SET_DUMPABLE` step at `3e9c496`; as root it measures nothing and says so). `brokerd` unit tests: an idle daemon fires a
300 ms dead-man on its own (302 ms after arming), a busy one still fires it, and over the wire only the arming key beats.
`deadman.rs`: seven unit tests (subtree only, never upward; each beat buys a period from the beat; key, second arming, late beat,
disarm, a node revoked meanwhile, the edges). `json_contract`: both modes' reports.

**Mutants** (`scripts/mutants.py`): M199 (the monitor never arms), M200 (the daemon never times out), M202 (any key beats), M203
(a late beat rescues), M204 (a second arming replaces), M205 (no disarm on a clean end), M206 (a fired dead-man read as an
operator's revoke), M207 (a beat moves nothing) — red; controls green. **M201 (fire only on a timeout) SURVIVED twice** — the
"busy daemon" witness's clients asked for a cheap `Status` (first with pauses, then eight back to back), the queue still drained,
and the loop's zero-length wait at the deadline timed out and fired anyway: the ordering the code relies on was invisible. The
witness's eight clients now MINT nodes (each request writes a record, so the handler is slower than its clients and a request is
always waiting): M201 red 3 of 3, the fixed test green 3 of 3 (about 3,150 requests in 1.6 s).

**Read on the runners at `3e9c496`** over `monitor_cli`, `bin:delulu` and `json_contract`: ubuntu-24.04 `38028453751` (213
passed), macOS `38028455193` (193), Windows `38028456795` (177), ubuntu-24.04-arm `38028458119` (213) — 0 failed on each, every
new witness named (the stopped-monitor one is Unix-only; the memory one Linux-only, real on the non-root runners); `delulu-broker`'s
dead-man unit tests on Windows `38028459469` (7 passed). Clippy clean; `check-other-os.sh` clean for Windows and macOS. **The full
suite alone at `3e9c496`:** 2,194 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved. `3e9c496` pushed to
`master` (a fast-forward from `ed7f7b6`).

**Residuals.** The beat's key on macOS and Windows rests on the OS account (no non-dumpable flag there) — category 7, as is root
everywhere. Ctrl-C on a `quarantine` monitor is a death: the operator ends a watch with `--for` or by revoking `g_M`. P8-04 is
left with step 4's two rules: `special-use` (waits on RW 4.46) and `break-glass` (its record must carry the run's node).

**The red-team pass on run 16's four slices (redone, as run 16's entry asked).** One Sonnet 5.5 sous-chef (`sonnet`), about 38
minutes, against a frozen copy of the binary at `ed7f7b6` in `/tmp/redteam17` (`git status` clean after it), writing
`FINDINGS.md` as it went. Each finding below says whether the head chef re-ran it on the current binary (`3e9c496`).
**What held** (its counts): actors' file, network, secret and device uses all recorded and stopped by a revoke (720 of 720 file
uses across 24 actors; 1,161 envelope refusals each citing its own `use`; an epoch-class revoke reached an actor within 45 ms);
nested actors; every `Interp::new` site holds the run's custody; plugin liveness under the daemon (get, call, unload, a retained
callable dead after unload); 7 parallel-writer soaks and 65 `kill -9`s of the daemon with 0 duplicate seqs, gaps, broken links or
wrong references on small chains; 4 midnight crossings under a clock shim; RW 4.51 for a normal end, a fault and DL0905; two runs
sharing a plugin path.
- **F-03 → RW 4.55, CLOSURE-SCOPE-1 (HIGH) — re-run, reproduced.** A closure resolves its function names in the module of the
  interpreter that CALLS it: a host closure passed to a Verified export calls the plugin's same-named function, which receives
  the captured capabilities — a plugin loaded with an all-empty grant wrote its own file through the host's `FsWrite` (and,
  in the red team's run, read a broker-held secret's plaintext). The next slice of this run.
- **F-07 → RW 4.56 (HIGH for a control loop) — re-run, reproduced:** revoking the plugin's node 2.5 s into a 120-command export
  — 96 commands landed after it, and the CLI had said "before the next use".
- **F-01 → RW 4.57, F-06 → RW 4.58 — code reading agrees, not re-run.** **F-04, F-05 → RW 4.53, not re-run.** **F-02** (a plugin
  node's own grant is never consulted for a capability the host handed it) and **F-08** (a `Cap[Compute]` answers `NoAdapter`
  inside an export) are D-V2-104's documented design. Oddities kept for judgment: a run whose actors died of a revoke exits 0;
  `plugin build` accepts a plugin declaring an actor that `load` refuses; one DL0406 printed five times; `ts` not monotone
  along the chain under load (the links are the order); `audit verify` flags no seq gap or dangling reference.

## 2026-10-10 — routine run 17 (continued): CLOSURE-SCOPE-1 closed — a closure's names are its own module's (D-V2-107, RW 4.55)

**The defect** (the red team's F-03, re-run by the head chef on `3e9c496` before a word of it was recorded). A closure was
`{ params, body, env }`; a call by name inside its body looked in the function table of the interpreter that CALLED it. A
Verified plugin export may take a callback (R-4), and the export runs in a nested interpreter built from the plugin's module —
so a host callback calling the host's `save(fs, name)` ran the PLUGIN's `save` when the plugin defined one, with the host's
captured `FsWrite`. Witnessed: a plugin loaded with an all-empty grant wrote `PWNED-by-plugin.txt` and the host's file was never
written; under the daemon the red team's variant read a broker-held secret's plaintext through a captured `Secret` and
`Cap[Declassify]` (re-run: the same). The reverse held too: a closure the plugin built and returned called the HOST's `helper`
when the host invoked it.

**Fixed** (D-V2-107): `interp.rs`'s `Home` — a module's functions and globals — held by the interpreter that runs the module and by
every closure that module's code builds (a lambda, or a named function taken as a value). `call_closure` runs the body against the
closure's home and restores the caller's before it returns; custody, brokers and the effect sink stay the calling interpreter's (a
nested interpreter shares the host's). A closure rebuilt from an actor message or the cycle collector's probe has no home and
resolves in the interpreter that calls it — an actor's worker runs the program's own module.

**Witness** `plugin_load_cli` `a_closures_names_resolve_in_its_own_module_across_the_plugin_boundary`, red first ("callback
answered 1000", the plugin's file written): the callback writes the host's file and answers the host's 1; the plugin's closure and a
plugin function handed out as a value answer the plugin's 1000; a callback reaches a host function the plugin lacks (before the fix,
DL0907 `unbound name`). The red team's three repros by hand on the fix: the host's file written, the plugin's closure 1000, and
"host redacted the secret" under the daemon. **Mutants** M208 (no swap), M209 (a lambda records no home), M210 (a named function's
value records none), M211 (the caller's home not restored) — red; the control green. **Read on the runners at `c83ebe6`** over `plugin_load_cli`, `actors_cli`, `examples_run` and `bin:delulu`: macOS `38029968775`, Windows `38029970294`, arm64 `38029972153` — 0 failed, the witness named on each; `delulu-runtime`'s unit tests on Windows `38029973402` (260 passed). Clippy clean; `check-other-os.sh` clean for Windows and macOS. **The full suite alone at `c83ebe6`:** 2,195 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved; `c83ebe6` pushed to `master`.

**Residuals.** A closure a plugin built keeps running the plugin's code after the plugin is unloaded — its liveness is not
checked (RW 4.56's neighbour: a plugin's authority is checked at the CALL of an export, not at each use). In a host callback run
inside an export, a foreign bind meets the nested interpreter's bind table and grants, which hold none of the host's: it is refused
(`NotGranted`) where the host would have bound — fail-closed, and the wrong reason.

## 2026-10-10 — routine run 17 (continued): RW 4.56 closed — a plugin node's revoke reaches its running export's next use (D-V2-108); `master` red once on the busy-daemon witness's baseline, fixed

**RW 4.56** (the red team's F-07, re-run by the head chef on `3e9c496`: 96 of 120 commands landed after the plugin node's
revoke). **Witness** `plugin_load_cli` `under_the_broker_daemon_revoking_a_plugins_node_stops_its_running_export_at_the_next_use`,
red first: 3 commands before the revoke, **117 after it**, `landed=120`. **Fixed** (D-V2-108): `custody.rs`'s `PluginUseCustody` —
the export's nested interpreter holds the host's custody behind a gate on the plugin's node; once it is revoked, an actuator
command is told `LeaseRevoked` and any other effect faults DL0801. Green: 3 before, **0 after**. **Mutants** M212 (the gate always
open), M213 (`check` skips it), M214 (the nested interpreter gets the host's custody unwrapped) — red; control green. **Read on
the runners at `ff91be5`** over `plugin_load_cli`, `estop_cli` and `examples_run`: macOS `38030868405` (4 before, 0 after),
Windows `38030870802` (3, 0), arm64 `38030872242` (3, 0) — 31 passed, 0 failed on each. Clippy clean; `check-other-os.sh` clean
for Windows and macOS. Residual: a use refused by the gate is not recorded (the revoke is; RW 4.46's shape); each gated use under
the daemon costs one liveness round trip.

**`master` red once — `3e9c496`'s push run `38029067385`, `test (ubuntu-24.04)`:** `a_busy_daemon_still_fires_an_unbeaten_dead_man`
asserted a BASELINE of 200 requests answered in 1.6 s, and CI's test job — the whole suite in parallel — answered 45; the dead-man
assertion never ran. `02cc186`'s run (`38029490424`), the same code, passed by chance. Re-run on a quiet VM, **M201 then SURVIVED**:
its earlier red 3 of 3 (this run's first entry) had come while the red-team sous-chef loaded the VM — cheap requests let the queue
drain, the zero-length wait at a past deadline times out, and the dead-man fires anyway. **Fixed in `a76018f`:** eight clients list
a 2,001-node tree, so each answer costs the loop milliseconds and a request is always waiting; the dead-man assertion comes first;
the baseline is that requests were answered AFTER the deadline. M201 red 4 of 4 on a quiet VM, M200 red, controls green; the test
green 3 of 3 with all four CPUs busy (40–51 listings). The wire-key test's period widened from 400 ms to 1.5 s for the same reason
(M204 still red). The fixed witness read on the runners at `a76018f`: macOS `38031308292` (71 listings, 43 after the deadline), Windows `38031306607` (45, 31) — green. **The full suite alone at `a76018f`:** 2,196 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved; `ff91be5` and `a76018f` pushed to `master`.

## 2026-10-10 — routine run 17 (continued): RW 4.58 closed for revocations and mints — a lost audit record is never answered as success (D-V2-109)

**The defect** (the red team's F-06; code reading agreed — `tree.rs`'s `record_op` logs a failed append and continues — and now
witnessed). When another writer held the chain's append lock past the daemon's 5 s wait, the operation still happened and its
answer claimed a record: **witnessed red** — a `Revoke` answered `Revoked { by_seq: 3, … }` for a seq that was never written,
and an `Issue` answered `Issued`, a live node with no record of its creation. Invariant 26 already refused a synchronous-class USE
the daemon could not record (`Check`, `Expose`, `Verify`); revocations and mints had no such check.

**Fixed** (D-V2-109): a revocation is never refused for want of its record — it is the e-stop — but `Response::Revoked` carries
`unrecorded` with the reason, `grants revoke` prints `warning[DL1401]: … the revocation IS in force, but audit seq N names no record`
and exits 1 (`--json` carries `unrecorded`), a monitor's quarantine reports it and the monitor exits 1, and a dead-man that fired
says so in `broker.log`. A mint (`Issue`, `Attenuate`, `Delegate`) whose `issue` record was lost is refused DL1401 and the node
WITHDRAWN from the tree (`Broker::withdraw_unrecorded` — a leaf just minted, removed without a second record the held chain
would refuse too). **Witnesses:** `brokerd` `a_revoke_whose_record_was_lost_says_so_and_a_mint_whose_record_was_lost_is_refused`
(the test holds the chain's lock itself; red first, as above) and `monitor_cli`
`a_revoke_whose_record_was_lost_warns_and_exits_one_and_still_takes_effect`. **Mutants** M215/M215b (no `unrecorded`), M216 (a
lost mint record answered `Issued`), M217 (the node not withdrawn), M218 (`grants revoke` exits 0) — red; controls green. **Read on the runners at `75e748e`** over `bin:delulu`, `monitor_cli`, `grants_cli` and `json_contract`: Windows `38032573759`, macOS `38032575295`, arm64 `38032576941` — 0 failed, both witnesses named on each. Clippy clean; `check-other-os.sh` clean for Windows and macOS.
**The full suite alone at `75e748e`:** 2,198 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved; pushed to `master`.

**Left of RW 4.58:** the lost seq can still be TAKEN by another writer (a sandboxed host numbers "last + 1" under the lock it
finally gets), so a reference the daemon handed out — now flagged — may later name another record; `Adopt`, `DeviceRefused`,
`ArmDeadman`/`DisarmDeadman` and the guard's records do not yet check; and why a writer holds the lock that long is RW 4.53.

## 2026-10-10 — routine run 17 (postscript): the last push runs read — `f0a39e7` green, `75e748e` red on macOS (RW 4.59, open)

**`f0a39e7`** (the closing commit, which holds all of this run's code): push run `38033226509` — success on every job; its
`openshell` and `ocsf` runs were green on `75e748e` (`38033108172`, `38033108184`). **`75e748e`**: push run `38033108211` —
**failure**, one job, `test (macos-latest)`, one test of 2,171: `sandbox_devices_cli`
`a_wedged_guest_that_never_asks_again_still_loses_its_actuator` — the sandboxed guest's FIRST command was answered `REVOKED: the
lease on arm0/elbow was revoked (missed-heartbeat)`, "beat overdue by 28056 µs" on a 150 ms heartbeat. The path is P8-01's (a
sandboxed run's device broker, no daemon) and none of this run's changes touch it; the same code passed in `f0a39e7`'s run; the
witness passed 5 of 5 on Linux with six busy loops on four CPUs. Recorded as **RW 4.59**, open — measure the program-send-to-first-
command interval on a macOS runner under the parallel suite before choosing a fix, and never lengthen the beat to make it green.
`master` is green at the close (`f0a39e7`).

## 2026-10-11 — routine run 18: CI read green; RW 4.59 closed — a guest's devices start at its first request (D-V2-110)

**The previous run, verified.** Run 17's last push, `b77c64d`: push run `38034360521` — success on every job (`heavy-gates` and
`miri-slow` skipped on a push, as always). The nightlies since: 2026-10-10 `38040982137` and 2026-10-11 `38128985173` — success on
all 18 jobs each, `heavy-gates`, the three `miri-slow` jobs, `microvm` and `microvm-reproducible` included; today's nightly is this
Sunday's sweep's CI read. `actors_pingpong` on the 2026-10-11 nightly: arm64 MEASURED 2.87x against a control of 4.12x (bar 1.50x,
passed); ubuntu-24.04 and Windows NOT MEASURED (no 3x of perfectly parallel work in 3 attempts — busy or SMT runners); macOS NOT
MEASURED (3 hardware threads). The sandbox properties: Linux and arm64 all five established; Windows established, its writes only
its own per-run container folder; macOS `filesystem_confinement absent` (reads not confined) — unchanged, PS-E's named residual.
Every `origin/claude/*` branch checked after unshallowing: two carry one commit `master` lacks (`7d6a22b`, `d3f0d52`), both recorded
as superseded since run 10. The `v1.0.0` tag seen on this run's first full fetch is the owner's of 2026-07-20 (`HANDOFF.md` §11.3),
not a new one. Survey `ok` (1,500 nodes, 13,796 edges) and `doctor --check` all checks passed at the start; `cargo deny … check
advisories` ok; `gh` is installed and its token refused, as run 10 found — CI read with `curl` and the MCP tools.

**RW 4.59, measured and then found.** Run 17 saw one macOS push run lose `sandbox_devices_cli`'s wedged witness: the guest's FIRST
command answered `REVOKED (missed-heartbeat)`, 28 ms past a 150 ms beat. The row said: measure first. Reading the code first showed an
asymmetry the measurement would only have sampled: an ordinary run starts its device broker after its check, just before `main`
(`run_cmd.rs`); a guest's started when the guest was SENT its program (D-V2-95 item 4), and the guest checks that program again
(`delulu_check::check_source`) before running a line of it — so the guest's own check was charged against the first heartbeat. The
window was a few milliseconds for the witness's small program, so a quiet machine never showed it. **Witnessed deterministically by
growing what the window hid**: `a_guests_own_check_of_its_program_is_not_charged_against_its_first_heartbeat` doubles a program of
small functions until `delulu check` takes 450 ms on the machine (the baseline is that state, measured, never assumed), then runs it
plain and sandboxed at a 150 ms beat — `COMMANDED` unsandboxed, `REVOKED: the lease on arm0/elbow was revoked (missed-heartbeat)` in
the guest. **Red at `b0a520c`** in the VM, on Windows (`38152502475`: 4,000 functions, the check 1.23 s; 11 others passed) and on macOS
(`38152500629`, the whole `delulu` package under `--nocapture`: the check 474 ms; 931 passed, 1 failed). The same commit made the wedged
witness print when its one command beat the lease, on the devices' clock: **2 ms after the send on macOS** under the package's
parallel run, 4 ms on Windows and in the VM — so run 17's 178 ms was a runner stall, of which the guest's check was one part.

**Fixed** (D-V2-110): `HostChannel::with_devices_at_first_request` starts the run's devices at the first request the host answers after
the program is sent — the first instant it knows the program is running; `guest.rs` hands it the plan and takes the started broker
back after `serve` for the close and the report. A guest gains nothing by asking late: until it asks it holds no capability. A guest
that never asks never starts them, so the close now revokes the run's device nodes on that path too (`revoke_device_nodes`) — the
second witness, `a_guest_that_never_asks_leaves_no_device_node_live` (a daemon run whose guest computes until its wall ceiling ends it;
the run's device node exists and is not live), green before the fix and red under its mutant. **Mutants:** M219 (devices started at
the send) red on the first witness; M220 (no revoke when never started) red on the second; M221 (the started broker not handed back)
red on three witnesses (no lost-device line, no e-stop); **M222** (start on the confinement report too) **survived — an equivalent
mutant**: `confirm` answers the report before the host is given the devices, so its guard could never fire; the guard was removed
rather than kept as a check no witness can fail. Control green. Clippy clean; `check-other-os.sh` clean for Windows and macOS.
**Read on the runners at `25a53d0`:** Windows `38153193744` and arm64 `38153195727` (`sandbox_devices_cli`, `sandbox_cli`,
`guest_cli`: 20 passed, 0 failed each), macOS `38153204486` (the whole `delulu` package: 932 passed, 0 failed) — the new witnesses
named on each, and the measurement **0 ms from the first request to the first command** on all three. **The full suite alone at
`25a53d0`:** 2,200 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved.

**Left, watched:** the window the wedged witnesses still assume is the guest's first request to its first command — two round trips.
If a runner stalls there, the next step is to make the witnesses' first request the actuator's own mint; never a longer beat.

## 2026-10-11 — routine run 18 (continued): RW 4.58's rest for device refusals and adoptions; one wait for the chain per request (D-V2-111)

**`439fdbb`** (RW 4.59 and its records): push run `38154053591` — success on every job (`miri-slow` and `heavy-gates` skipped on a
push); its `ocsf` run `38154053594` success. `actors_pingpong`: arm64 MEASURED 2.79x against 4.11x (bar 1.50x); the other three NOT
MEASURED, as on the nightlies. `master` green with D-V2-110.

**RW 4.58's rest, witnessed red first** — each holding the chain's append lock as D-V2-109's witness does. `brokerd`
`a_device_refusal_whose_record_was_lost_is_not_answered_as_recorded`: with the chain free a refusal is `Recorded`; held, it was still
answered **`Recorded { seq: 4 }`** for a record the chain never got — so the client's "the chain lacks this refusal" warning
(`broker_client::note_device_refusal`), written for exactly that case, could never fire. `an_adoption_whose_record_was_lost_is_refused_and_the_certificate_still_adopts_later`:
held, the adoption did not answer within the client's 15 s deadline (`TimedOut … did not answer within 15s`) — the daemon's sink
asked for the lock afresh at each of the adoption's two seqs and two records, 5 s each, and answered `Adopted` after 20 s to a client
that had gone.

**Fixed** (D-V2-111): a refusal whose record was lost is DL1401 in words; an adoption is a mint across a broker boundary, refused and
withdrawn through `unrecorded_mint`, and `withdraw_unrecorded` now clears the withdrawn node's adoption (`adopted`,
`adopted_chain_fps`) so the certificate adopts once the chain is free; and the sink waits for a busy chain once per request
(`ChainLock::NotFree`, forgotten when the serve loop releases the request's lock) — an e-stop's revoke now waits at most 5 s for a
held chain, where it waited 10 s. **Mutants:** M223 (no `DeviceRefused` check), M224 (`Adopt` answered without the mint check), M225
(the adoption's bookkeeping left behind — the later adoption refused "already adopted"), M226 (a not-free lock waited for again — the
adoption times out) — red; control green. Clippy clean; `check-other-os.sh` clean for Windows and macOS. **Read on the runners at
`aa20498`** over `bin:delulu`, `federation_cli`, `monitor_cli` and `grants_cli`: Windows `38154807791` (179 passed), macOS
`38154809795` (194), arm64 `38154811843` (214) — 0 failed, the two new witnesses and D-V2-109's two named on each. **The full suite
alone at `aa20498`:** 2,202 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved.

**Left of RW 4.58:** the lost seq can still be taken by another writer; the dead-man's arm and disarm and the guard's records do not
yet check (D-V2-111 item 4 says why each needs its own answer); the long hold itself is RW 4.53.

## 2026-10-11 — routine run 18 (continued): RW 4.46 closed — an egress refusal is a `deny` record citing its `allow` (D-V2-112)

**`9c758c3`** (RW 4.58's rest's records): push run `38155661919` — success on every job; `ocsf` `38155661891` success.

**RW 4.46, witnessed without a resolver seam.** Run 15 measured it (a name pointed at loopback through `/etc/hosts`: `use allow
dlx-special-witness.com` and nothing else) and left it open because "the witness needs a name that resolves to a special-use address on
every runner". This run first tried `net=localhost` — the GRANT layer refuses it, and every other spelling of loopback tried (`127.1`,
`2130706433`, `0x7f000001`, `0177.0.0.1`, `::1`, `localhost.`): a pin that holds. But the egress client refuses more than special-use
after the gate has allowed the host — and a URL's **userinfo** (`https://user@example.com/a`, granted `net=example.com`) is refused
by it before any resolution or connection, on every operating system. `monitor_cli`
`an_egress_refusal_is_a_deny_record_citing_the_use_it_overrides` and `a_sandboxed_guests_egress_refusal_is_recorded_the_same_way`:
two such fetches under `--broker daemon`, the program told "not delivered" twice, stderr `[egress: userinfo]` — and the chain must hold
two `use deny` records at `example.com`, `refused_by: egress`, `reason: userinfo`, each citing its own fetch's `use allow`. Red on the
pre-fix behaviour (mutants M227 and M228 remove the note from the interpreter and from the guest's host: each path's witness red).

**Fixed** (D-V2-112): `Custody::note_egress_refusal`, called where the interpreter and a guest's host perform a fetch the gate allowed,
once per refusal the egress client logged for that URL (`egress::refusals_since` — a refusal is a reason whose `answer()` is `Refused`;
a timeout, TLS or connection error or HTTP status is a failure, not a refusal); `BrokerClientCustody` sends `EgressRefused` with
`overrides_seq` from the last `use allow` of a host (`last_net_allow`); the daemon records it (`Broker::record_egress_refusal`: `use`,
`deny`, `op: Net`, `refused_by: egress`, the reason) and answers DL1401 for a lost record (D-V2-111's rule; its own witness,
`brokerd` `an_egress_refusal_whose_record_was_lost_is_not_answered_as_recorded`). **By hand in the VM, the special-use case** (a
one-run `/etc/hosts` entry, `127.0.0.1 dlx-special-witness.com`, restored and checked): under the daemon, `3 allow` → `4 deny
{reason: special-use, overrides_seq: 3}`; sandboxed, `7 allow` → `8 deny {… overrides_seq: 7}`; the chain verified. **Mutants**
M227 (no note in the interpreter), M228 (none in the guest's host), M229 (no `overrides_seq`), M230 (no refusal found), M231 (a lost
record answered `Recorded`) — red; control green. Clippy clean; `check-other-os.sh` clean for Windows and macOS. **Read on the runners
at `ced5dc4`:** macOS `38156513763` (the whole `delulu` package: 937 passed), Windows `38156516142` (the whole package: 907), arm64
`38156518032` (`monitor_cli`, `bin:delulu`: 206) — 0 failed, the three new witnesses named on each. **The full suite alone at
`ced5dc4`:** 2,205 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved.

**Unblocked:** P8-04 step 4's `special-use` rule.

## 2026-10-11 — routine run 18 (continued): P8-04 step 4's `special-use` rule (D-V2-113); the `envelope` rule seen firing at last

**`8084e4e`** (RW 4.46's records): `openshell` `38157216942` and `ocsf` `38157216971` success; its CI run was running at this entry.

**Built:** `delulu monitor watch --rule special-use` — a `use` `deny` with `refused_by: egress` and `reason: special-use` (D-V2-112's
record) quarantines the run at the first one. `Rule::fires_on` decides every rule one record fires; the watch loop routes on its
answer and has no per-rule arm left. **By hand in the VM** (a one-run `/etc/hosts` entry, `127.0.0.1 dlx-special-witness.com`,
restored and checked): a monitor node `g_M`, a run's lease delegated under it, `monitor watch --node g_M --rule special-use --poll
25`, and a program fetching that name 40 times — the run printed `not delivered` ONCE and died `DL1403 … revoked by audit seq 9`;
the monitor's report: one quarantine, `rule: special-use`, `count: 1`, the run's node; the revocation's own record: `monitor … quarantined
…: rule special-use — 1 deny record(s) within 0 ms (seq 8; …)`. **Witnesses:** `monitor_cmd` `the_special_use_rule_fires_on_a_special_use_refusal_and_nothing_else`
(unit: not on userinfo or not-allowlisted, not on an envelope's refusal, not on another action, not without the host's record) and the
parse test. **A gap found on the way:** mutant M233 — the `envelope` rule never firing — passed the WHOLE `monitor_cli` target: every
test that used `envelope` asserted an absence (a sibling's monitor, a dead monitor). `the_envelope_rule_quarantines_the_run_at_its_first_refusal`
now witnesses a single-record rule firing end to end on any runner — the path `special-use` shares. **Mutants:** M232 (special-use
fires on any egress refusal), M233 (envelope never fires: red on the new integration witness), M233b (the same, on the unit test),
M234 (the rule not parsed) — red; control green. Clippy clean; `check-other-os.sh` clean for Windows and macOS. **Read on the runners at
`db40109`** over `monitor_cli`, `bin:delulu`, `completions_cli` and `cli_contract`: Windows `38157840795` (199 passed), macOS
`38157843623` (214), arm64 `38157845681` (234) — 0 failed, both new witnesses named. **The full suite alone at `db40109`:** 2,207
passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved.

**Left of step 4:** `break-glass` — `audit_required` writes its record with no node, so a monitor cannot attribute it to a run.

## 2026-10-11 — routine run 18 (continued): the Sunday sweep's red-team pass — F1/F2 and F3 fixed the same run; RW 4.60 and 4.61 opened

**The sweep** (step 4 item 5; today is Sunday): the CI read is the 2026-10-11 nightly `38128985173` (all 18 jobs green — `heavy-gates`,
the three `miri-slow` jobs, `microvm`); the document scan found no open `REMAINING_WORK.md` row the log calls closed, and four live
passages that said a guest's devices start at the send (updated with D-V2-110); and an adversarial pass on this run's four new
guarantees — **one Sonnet 5.5 sous-chef** (`sonnet`), ~27 minutes, against a frozen copy of the binary at `db40109` in `/tmp/redteam18`,
briefed to append each finding to `FINDINGS.md` as found and to log what held. After it: `git status` showed only the head chef's own
edits, `/etc/hosts` (edited by the pass for one-run entries) compared equal to the backup, no broker, server or lock holder left.
`f875a6b`'s push runs: CI `38158626570` and `openshell` `38158626534` — success.

**F1 (real — fixed in `bc124e9`):** D-V2-112's first form read a fetch's refusal from the run's egress log, kept to its first 64
records — from a run's 65th request on, refused fetches left `use allow` and no `deny`. Re-run by the head chef on the frozen binary:
two runs of 70 refused fetches, 128 `deny`s of 140. **F2 (its consequence):** a `special-use` monitor walked past by 64 fetches to port
0 (refused `malformed`) and then one special-use reach — `quarantines: []`, the run's node left live. Fixed (D-V2-112 item 5): the
refusal comes from the calling thread's own outcome (`egress::fetch_refusal`); `every_egress_refusal_is_recorded_past_the_logs_bound`
and its guest form, 70 of 70 (140 of 140 by hand); M235/M236 red, control green. Read at `bc124e9`: Windows `38159257751` (23 passed),
macOS `38159260568` (24), arm64 `38159262871` (25).

**F3 (HIGH — fixed in `db70f1f`):** D-V2-110's late device start moved the e-stop's first look past it: the watchdog first checked the
grant tree one tick (≤ 25 ms) after the start, so a device node revoked while the guest still computed let that tick's commands reach
the arm — 13, 11 and 8 of 300 in three runs, 20 of 300 with four devices (the sweep probes devices one at a time). Fixed (D-V2-110 item
5): `device::estop_sweep` runs once synchronously as the broker is built, then on every tick. `an_operator_revoke_during_a_guests_compute_stops_its_first_command`:
the node revoked during the guest's compute, then 40 commands — none COMMANDED, the lease revoked 0–3 ms after the devices started.
M237 (no synchronous check) red 3 of 3 — once under the pass's load, twice on a quiet VM; control green. Read at `db70f1f` over
`sandbox_devices_cli`, `estop_cli`, `dead_man_cli` and `robotics_demo`: Windows `38159975775`, macOS `38159978489`, arm64 `38159981058` —
green, the witness named on each.

**F5 (reproduced by the head chef — RW 4.61, open):** with another writer holding the chain's lock, `guard bypass on --owner CODE` printed
`ok: guard bypass ENABLED` (exit 0, 5,014 ms); `guard status` said BYPASSED; the chain held only seq 1 — no `guard_bypass_on`. **F5b**
(the pass's, not re-run): `monitor watch` answered `armed at seq 4` for a seq with no record, and its beats, queued behind lock waits,
let its own dead-man revoke every run under it. The handlers that never read `failed`: `GuardBypass`, `GuardPolicySet`/`Unset`,
`GuardApprove`/`Deny`/`Request`, `GuardPermitRevoke`, `ArmDeadman`, `Redeem`. Not fixed this run: each needs its own answer (a downgrade
rolled back, an upgrade said `unrecorded`), witnessed per handler — the next run's first item.

**F4 and F6 (the pass's, run twice and once — added to RW 4.53):** the daemon is serial, so queued requests each pay their own lock
wait — an e-stop behind four answered `did not answer within 15s` while the revoke landed at ~25 s; and a flooded chain (82,587 records
from 8 parallel runs) made the monitor's whole-chain read per poll 1.9 s late against a 2 s default dead-man. **D1 (RW 4.60, low):** a
guest that never asks never starts its devices, its report has no `devices` entry, and its device node shows `live` until the run ends.

**Held, as logged by the pass:** fourteen URL spellings through both paths — each one `allow` and one `deny` at the same host citing the
right seq, the plain chain equal to the sandboxed; userinfo variants; `not-allowlisted` reached through a redirect with real TLS (a local
CA through `SSL_CERT_FILE`); every numeric and named spelling of loopback, metadata and `0.0.0.0` refused at the grant; contention —
12 daemon and 12 sandboxed runs with 60 refused fetches each beside 6 delegate/revoke pairs: delegate ≤ 0.22 s, revoke ≤ 0.10 s, a
2,959-record chain with unique contiguous seqs, 1,440 `allow`s and 1,440 `deny`s, every `deny` citing an earlier `allow` of its node and
host; one 5 s wait per request under a held lock (revoke, delegate, adopt — refused and withdrawn); the monitor never quarantined
outside `g_M` nor fired on anything but a special-use reach. **Oddities recorded:** O1 (attribution by the custody gate's parse; the
egress client refuses the ambiguous spelling), O3 (a redirect hop's refusal names the first hop's host) — RW 4.46; O2/O4 (`grants
revoke`'s `ok:` line and its `--json` summary beside `unrecorded`; `audit verify` passes a seq hole) — RW 4.58. **Not tested by the
pass, by its own account:** actors (cross-thread citations — the F1 fix reads a thread's own outcome, so this no longer applies by
construction), `hw:`, the monitor's node-list/chain-read race (source only).

## 2026-10-11 — routine run 18 (postscript): RW 4.61's guard bypass closed (D-V2-114); the closing commit's push runs read

**RW 4.61, the bypass** (the red team's F5, reproduced by the head chef: under a held chain lock `guard bypass on` answered `ok`,
exit 0, 5,014 ms, the bypass in force and no `guard_bypass_on` record). **Fixed** in `d908884` (D-V2-114): a bypass turned on whose
record was lost is undone in the same request (`withdraw_unrecorded_bypass`, no second record) and refused DL1401; turned off it
stays off and answers DL1401 that the chain lacks the record. **Witness:** `brokerd` `a_guard_bypass_whose_record_was_lost_does_not_happen`
(the test holds the chain; its baseline turns the bypass on with the chain free, then off under a held chain). **Mutants:** M238 (answered
`Ok` — the pre-fix behaviour), M239 (refused but not undone), M240 (the bypass-off answered `Ok`) — red; control green. Clippy clean;
`check-other-os.sh` clean for Windows and macOS. **Read on the runners at `d908884`** over `bin:delulu`, `guard_cli` and `guard_e2e`: Windows `38161584149` (165 passed), macOS `38161586576` (179), arm64 `38161588717` (198) — 0 failed, the witness named on each. **The full suite alone at `d908884`:** 2,211 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved.

**The closing commit `00112e5`** (every code change of this run before `d908884`): push run `38161113083` — success on every job (`miri-slow` and `heavy-gates` skipped on a push), `openshell` `38161113026` success; `db70f1f`, `bc124e9` and the records commits between were pushed to `master` only inside `00112e5`'s fast-forward.

**Left of RW 4.61:** the guard's policy edits, approvals, requests and permits, the dead-man's arm and `Redeem` — each its own answer
by D-V2-114's rule (a downgrade undone and refused, an upgrade in force and said).

## 2026-10-12 — routine run 19: CI read green; the cloud period's closing entry begun; RW 4.61 closed — no daemon operation is answered as recorded when its record was lost (D-V2-115)

**CI at the start, read:** run 18's closing push `e95acf1` — CI `38162468215` success on every job (`miri-slow` and `heavy-gates`
skipped on a push), `ocsf` `38162468217` success; the 2026-10-12 nightlies — CI `38234415113` all 18 jobs success (`heavy-gates`,
the three `miri-slow` jobs, `microvm`, `microvm-reproducible`), the release dry run `38248184651` success. `actors_pingpong`, from
the annotations: arm64 MEASURED 2.88x against a control of 4.11x (push) and 4.12x (nightly), bar 1.50x; macOS NOT MEASURED (3
hardware threads); Windows and Ubuntu NOT MEASURED (busy or SMT). Sandbox properties per OS as since D-V2-57. The Survey matched
the tree (1,500 nodes) and `doctor --check` passed every check; run 18's witness re-run green in the VM; no branch held a commit
`master` lacks but the two superseded since run 10.

**The closing entry** (`docs/CLOUD_SYNC_LOG.md`, *The closing entry*), begun four days early as the routine's prompt asks — a
weekly limit can end the period's last runs: what the period built (P7, PS-D, PS-E complete; P8-01 to P8-03 complete, P8-04 in
progress; the defects closed; the loop's tools), what stays open, what is the owner's, how the laptop syncs. `CLOUD_ROUTINE.md`
step 0 and step 6 now say every run keeps it current in place. Pushed as `5bd440d`.

**RW 4.61's rest (`96b7b87`, D-V2-115).** Run 18 closed the guard bypass and left eight handlers that never read the daemon's
`failed` flag. Before fixing them, every `record_op` call in `delulu-broker` was read against the daemon handler that reaches it:
four more wrote a record and answered as if it were written — `DisarmDeadman` (`Recorded { seq }`), `Renew`, `SecretMap` and
`RotateKey`. All twelve witnessed red under a held chain — seven daemon tests, each asserting its handler's answer AND the state
after it: `guard policy set`'s lowered tier answered `Ok` (and was in force); an approval answered `GuardApproved { gp_0001 }`; a
redemption `Redeemed`; a renewal `Renewed` with the later deadline; `Secret.map` `Mapped { s_derived_00000000 }` — a name a
counter chose, so a guessable one; a key rotation `Ok`; an arming was in force (the baseline's second arming was refused "already
armed"). **Fixed** by D-V2-115's three answers: a loosening undone in the same request and refused (a policy edit that lowers or
removes a rule — `GuardPolicy::loosened_by`, restored and never persisted; an approval; a disarm, re-armed as it stood; a
redemption, the token unspent; a renewal, the deadline back); a step whose answer would claim a record withdrawn and refused (a
request, an arming, a `Secret.map` and its derivative); a tightening in force and said (a rule added or raised, a denial, a permit
revoked, a key rotation). The undo is explicit — `redeem_undoable`, `renew_undoable`, `disarm_deadman_undoable` return what they
replaced; the daemon snapshots the policy before an edit — and only the daemon puts it back, in the same request.
**Mutants M241–M257** (`scripts/mutants.py`, one spec): no undo of a loosening, a lowered tier not counted as loosening, a raise
counted as one, the loosened policy persisted anyway, a tightening not persisted, the approval's permit kept, its request left
approved, a request kept, a denial's and a permit revocation's answers claimed, an arming kept, a disarm not re-armed, a
redemption kept, its nonce left spent, a renewal kept, a derivative kept, a rotation's answer claimed — each red, by its own
witness only; the control green. The policy witness first let M244 live in reading (the tightenings, persisted after the
loosenings, rewrote the disk) — reordered so the loosenings come last before any mutant ran. Clippy clean; `check-other-os.sh`
clean for Windows and macOS. **Read on the runners at `96b7b87`** over `bin:delulu`, `guard_cli`, `guard_e2e` and `federation_cli`:
Windows `38274676231` (181 passed), macOS `38274680188` (195), arm64 `38274684944` (214) — 0 failed, the seven witnesses named on
each. **The full suite alone at `96b7b87`:** 2,218 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved.

**Left of RW 4.61:** a beat's own firing path (reachable only in a race — the daemon fires every due dead-man before the next
request; `fire_deadmen` already says when a firing's record is lost) and the daemon's startup records. The lesson (§11.5): a
finding's list is what its author tried; enumerate the class from its mechanism.

## 2026-10-12 — routine run 19 (continued): P8-04's `break-glass` rule (D-V2-116) — every step of P8-04 built

**`cad7c47`'s push runs:** CI `38276195408` success on every job, `ocsf` `38276194840` success; `5bd440d`'s CI `38270912985` success.

**The gap, read against the code before building.** `breakglass::gate` writes its `break-glass` record with `audit_required`,
which names no node — and it runs before a lease run has one (the lease is redeemed later in `run_cmd`); the monitor's loop
skipped every record that was not a `deny`. So a run that broke the glass under a monitor was invisible to it twice over.
**Witness first** (`monitor_cli`, `the_break_glass_rule_quarantines_the_run_under_a_ticket_and_no_other`): red on the code
before — `break-glass` was not a rule; with the rule, mutant M258 (no binding record) shows the run is still invisible.
**Built (`0d3f514`, D-V2-116):** once a lease run has redeemed and before its program runs, it asks the daemon to record its use
on its own node (`ReqBody::BreakGlassRun` → `Broker::record_break_glass_run` → `break-glass-run` `allow`, target the ticket's
fingerprint); a lost record is DL1401 and the run refuses itself (exit 2). `--rule break-glass` fires on that record; the loop now
hands every record to the rules, the refusal rules read the decision themselves and `denies=N/MS` counts refusals only. And
P8-04 step 7's last witness over the wire: `a_quarantine_outside_the_callers_subtree_is_refused` (a `Quarantine` across or
upward is `NotRevocable`, DL0904; the caller's own child is the baseline) — a pin, falsified by M265. **Mutants M258–M265** red,
each by its own witness (M263 — `denies` counting `allow`s — by the existing `a_monitor_quarantines_the_run_that_keeps_probing…`);
controls green. Clippy clean; `check-other-os.sh` clean for Windows and macOS; the help, completion, JSON-contract, skill and
MCP gates green. **Read on the runners at `0d3f514`** over `bin:delulu`, `monitor_cli` and `break_glass_cli`: Windows
`38277449931` (190 passed), macOS `38277453631` (205), arm64 `38277459675` (225) — 0 failed, the new witnesses named on each.
**The full suite alone at `0d3f514`:** 2,222 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree unmoved.

**P8-04's build order is complete** — steps 1 to 7, the four rules of `V2_OPENSHELL_STUDY.md` §4.7 among them — and with it P8 as
far as software reaches (D-V2-116 item 5, recorded once this push run is read; the residuals are named there).

## 2026-10-12 — routine run 19 (closing): P8 complete as far as software reaches; P9-01's build order read against the code

**`9000c57`'s push runs** (every code change of this run): CI `38278927092` — success on every job (`miri-slow` and `heavy-gates`
skipped on a push), `ocsf` `38278927078` and `openshell` `38278927163` success. `actors_pingpong`: arm64 MEASURED 2.95x against a
control of 4.12x (bar 1.50x); macOS NOT MEASURED (3 hardware threads); Windows and Ubuntu NOT MEASURED (busy or SMT). **P8 is
complete as far as software reaches** (D-V2-116 item 5; `V2_PHASE_STATUS.md`'s row carries the commits and the runs).

**P9-01, read against the code before it is built** (`docs/DELULULANG_V2/V2_P9_DESIGN.md`): the program's side exists without
spans (`ScopeWalk` drops each call's `Span`); the boundary's side is the runtime's own (`Grants::add` → `authority_spec_from_grants`
→ `spec_to_authority`, the conversion `run --grant` mints a daemon root with — read at `run_cmd.rs`); the lattice answers yes or no
(`attenuation_check`'s `Err` is the intersection), so the verdict asks its per-dimension functions item by item. **The corpus,
measured** (`scripts/authority-corpus.py`, new): 23 programs — 6 pure, 12 with a fully literal authority, 4 whose placeholders are
grant DATA the program never requests (a secret's VALUE, a library's PATH, an actuator's DIM bounds — decidable by the name), 1
with a scope the source COMPUTES (`10_actuate_errors`, `actuator=DEVICE`). The design question answered there: a hidden site is
`exceeds` against a boundary that grants nothing of its kind, `within` against one that grants the kind unbounded, and otherwise
`inconclusive`, named — never `within`. `unsupported` is empty on today's language (each of the thirteen `ResourceKind`s and
secrets have a grant form), and stays for a kind added later, held by a test.

**The first corpus read was vacuous** — a wrong JSON path read all 23 programs as pure, caught because programs that plainly print
said "no grants". `authority-corpus.py` carries the control that makes that read fail (exit 1 when no program asks for anything);
run against one pure program it fails as it should.

**Loop engineering** (`CLOUD_ROUTINE.md`): the closing entry kept current by every run (steps 0 and 6); three lines in step 5 —
enumerate a finding's class from its mechanism before fixing it; a corpus measurement carries a control that must come out
non-empty; a disk check reads the LAST write, so the edit a witness is about persists last.

## 2026-10-12 — routine run 19 (postscript): RW 4.60 closed (D-V2-117); the break-glass witness's race, found by the run's own suite

**RW 4.60 (`11b48cc`, D-V2-117).** Witnessed red first (`sandbox_devices_cli`, `a_report_says_when_a_guests_devices_never_started`:
a guest that computes until `--limits wall=2` left a report with no `devices` entry). Each planned device is journaled
`device.not-started` now. The first form read the plan at report time and stayed red — the plan goes to the host when the program
is sent — so `GuestDevices` keeps the names from construction. M266 (no names) and M268 (journaled even when started, via the
control) red; **M267 survived**: a second "none once they started" check that the report's branch can never reach — removed (§11.5,
M222). Clippy and `check-other-os.sh` clean. Read on the runners at `11b48cc` over `sandbox_devices_cli` and `schema_cli`: Windows
`38282229036`, macOS `38282233982`, arm64 `38282237136` — 18 passed each, the witness named on each.

**The break-glass witness's race (`0df4302`).** The full suite at `11b48cc` was 2,222 passed and **1 failed**:
`the_break_glass_rule_quarantines_the_run_under_a_ticket_and_no_other`. Read: the monitor had quarantined the run (DL1403 at its
first console write) before the program printed `started`, which the witness's baseline asserted — the product right, the witness
assuming a timing, and already on `master` (the push runs at `9000c57` and `3a91ab7` were green: their runners were quick enough).
Reproduced with four CPUs busy: 1 of 5 red, the same message. The baseline is now the gate's `BREAK-GLASS` banner: 0 of 8 red under
the same load. **The full suite alone at `0df4302`:** 2,223 passed, 0 failed, 16 ignored (161 binaries), cargo exit 0, the tree
unmoved. `3a91ab7`'s push run CI `38281286403` — success. The lesson (§11.5): a witness whose subject races the program is
timing-shaped even when it measures no time.

**`e2ea6a5`'s push runs** (RW 4.60, the race fix and their records): CI `38284697941` — success on every job, `ocsf` `38284697894`
success. Every code change of routine run 19 is on `master` and read green there.

## 2026-10-12 — routine run 20: CI read green; P9-01 — `delulu authority FILE --within BOUNDARY` (D-V2-118); SECRET-HIDDEN-1 found and closed

**The previous run, verified.** `21421ec`'s push run CI `38286315257` — success (run 19's last commit, unread at its entry). Run
19's witnesses re-run in the VM: `sandbox_devices_cli`'s `a_report_says_when_a_guests_devices_never_started` and `monitor_cli`'s
`the_break_glass_rule_quarantines_the_run_under_a_ticket_and_no_other` — green. `survey check` ok (1,502 nodes, 13,979 edges),
`doctor --check` all checks passed. Branches: every `origin/claude/*` read against `master` after `git fetch --unshallow`; the two
with a commit `master` lacks (`7d6a22b`, `d3f0d52`) are superseded since run 10. `gh` present at `/usr/local/bin/gh`, not needed.

**SECRET-HIDDEN-1 — found while reading the walk for P9-01, witnessed, closed (`3611ac5`).** `root.secret(name)` with a computed
name: the checker records a secret's name only for a string-literal argument, and the walk marked the kind hidden — but nothing
read the hidden secret. `delulu authority --grants` printed `--grant console` alone and the report said `secrets: []`; `delulu
run --grant console` refused DL0703 at line 6. SCOPE-HIDDEN-1's class (D-V2-81), one kind it never reached. The walk now hides the
kind for a computed name only (a literal through a `Root` under another name is already named by the checker, which reads the
TYPE — its control case), and `required_grants` carries `secret:NAME=VALUE`. Witnessed red first: two new cases in `cli.rs`'s
`a_scope_the_source_does_not_show_leaves_its_placeholder_beside_the_literals` (`left: ["console"]`).

**P9-01 built (`3611ac5`, D-V2-118).** `delulu authority FILE --within GRANT… | --within-file FILE` — within (exit 0), exceeds,
unsupported, inconclusive (exit 1), each request named with the call that makes it. Step 1 (spans in the walk) and step 2 (the
verdict as a pure function, `crates/delulu/src/within.rs`) as `V2_P9_DESIGN.md` ordered, with one departure: **each request is
decided by the runtime's own mint (`prim::call_root_method` against the root `Grants::add` builds), not by the broker's
lattice** — the lattice is coarser than a grant set (`console` folded into `Write`; no `net.special`, sensor, compute or plugin
dimension), so `attenuation_check` would call a printing program within an `fs.write`-only boundary that `run` refuses
(pinned). Where both speak of the same thing they agree over 400 generated pairs. `required_grants` became `requirements()` —
the same list structured — and every report stayed byte-identical (`core_invariance`, `atlas_chain`, `json_contract`).

**Witnesses** — `within_cli` (new): every corpus program within its own required grants and exceeding each one-grant narrowing,
the counterexample the dropped grant at the line of the call that asks (19 programs, 27 narrowings, a control); a computed
scope exceeds against none and is inconclusive against some; an exceeding literal beside inconclusive scopes is `exceeds`; the
empty boundary names `05_capabilities`' four requests in SOURCE order (console, line 44, first — neither the list's order nor its
reverse); a boundary file says what the flags say; six refusals (exit 2); both answers validate against the authority schema.
`within.rs`: seven unit tests. **Mutants M269–M280:** M269–M271, M273–M275, M277–M279 red at once; **M276** (precedence
inverted) survived its first read because its command never ran the unit test that sees it (§11.5, M233) — red under an
integration witness added for it and under the unit test; **M280** (source order reversed) survived because the unit test's
source-first request was also listed last, so the reverse picked it too — red once the order witnessed differs from both the
list's order and its reverse; **M272** (the special-use check dropped) survived because the check was unreachable: `Grants::add`
refuses a special-use host under plain `net=` (read on four hosts), so the mint alone decides — the code was removed (§11.5,
M222). Clippy clean.

**Read on the runners at `3611ac5`** over `within_cli`, `cli`, `schema_cli`, `bin:delulu`: Windows `38305782405` (237 passed), macOS `38305785803` (250), arm64 `38305788572` (269) — 0 failed, `within.rs`'s seven unit tests and `within_cli` named on each, no compiler warning. **The full suite alone at
`3611ac5`:** 2,235 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, and no `TREE-MOVED` line — `suite.sh` cannot see an edit undone before its end, which is why the slip below is recorded by hand.

**The tree moved once under that suite** — a slip recorded, not tidied away: the records' patch script was given a dry-run
switch by a `replace` whose anchor did not match, so its "dry run" wrote these nine records into the tree at about 20:42; they were
restored byte for byte at 20:42:47, while the suite ran its first `delulu` targets (`actuate_cli` to `atlas_chain`, none of which
reads them). The document gates were re-run after the records were applied (below). §11.5 has the lesson.

**Not taken in this slice:** a package directory and a `.dwx` under `--within` (refused by name; each needs its own requirement
list); P9-02 `grants diff`, which reads this verdict between two authorities, is next.

## 2026-10-12 — routine run 20 (continued): P9-02's first slice — `delulu grants diff OLD NEW` between two grant files (D-V2-119)

**`52b2572`'s push runs** (P9-01 and its records): CI `38306908871` success on every job (`miri-slow` and `heavy-gates` skipped on a push), `openshell` `38306907585` success; `actors_pingpong` — arm64 MEASURED 2.94x (control 4.12x, bar 1.50x), Windows and Ubuntu NOT MEASURED (busy or SMT), macOS NOT MEASURED (3 hardware threads).

**P9-02, first slice (`4f0fdd3`, D-V2-119).** `delulu grants diff OLD NEW [--json]` — two grant files; each NEW line decided against
OLD as P9-01 decides a request (the runtime's own mint), an actuator's envelope by the broker's device lattice and a compute
device's by its bounds, adapter, formats, kernel names and waiver; four categories (`effect_expansion`, `scope_expansion`,
`special_use_reach`, `device_reach_expansion`), no grade; exit 1 when the change adds anything. Offline; a lease node refused by name.
Witnessed red first (no verb); the witness reads every grant form's own example from `toolchain --json`. Two expectations of the
witness were wrong and were corrected against the product, not the reverse: `grants` verbs print their fields beside the envelope's
own (not under `grants`), and the reverse of the widening pair is no narrowing (`./data` is wider than `./data/sub`) — the diff
named it, and a pure-narrowing pair now carries the "adds nothing" case. **7a again:** `json_contract`'s broker-verbs sweep failed
the new verb at once — every advertised `grants` verb must be driven with `--json` — and drives it now, as does the success table.
Mutants M281–M289 red, the control green; clippy clean.

**Read on the runners at `4f0fdd3`** over `within_cli` and `json_contract`: Windows `38308017895` (`json_contract` 13, `within_cli` 6), macOS `38308020954` (14, 6), arm64 `38308023699` (14, 6) — 0 failed, no compiler warning. **The full suite alone at `4f0fdd3`:** 2,236 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved.

**Next:** lease nodes under `grants diff` (the broker's `Authority`, `budget_expansion` with it), then P9-03 — proposals whose
approval reads this diff.

## 2026-10-12 — routine run 20 (continued): P9-02 between two lease nodes (D-V2-120) — P9-02 built

**`02147ac`'s push runs** (P9-02's first slice and its records): CI `38309152070` success on every job (`miri-slow` and `heavy-gates` skipped on a push), `openshell` `38309152114` success; `actors_pingpong` — arm64 MEASURED 2.73x (control 4.08x, bar 1.50x), macOS MEASURED 1.73x (control 3.61x, bar 1.35x — a runner that offered four threads this time), Windows and Ubuntu NOT MEASURED (busy or SMT).

**Lease nodes (`3f96546`, D-V2-120).** `delulu grants diff g_OLD g_NEW` reads each node from its broker and compares them by the
lattice — a node's authority IS the broker's `Authority`, so here `attenuation_check` is the enforcement's own answer, asked of
each item of NEW as a one-item authority. Tried by hand on a scratch broker before the witness: seven findings one way, one the
other, none against itself. **One trap, found by reasoning before the first run:** a one-item probe built with no budget holds the
top of that dimension and fails against any budgeted OLD; the probe carries OLD's budget, and the budget is decided once (M290 red
without it). A first by-hand try put the broker's state directory in the scratchpad and the daemon refused it: a Unix socket path
is bounded (108 bytes on Linux, 104 on macOS) — the witness uses a short name under the temporary directory, as `json_contract`
does. Mutants M290–M295 red, the control green; clippy clean.

**Read on the runners at `3f96546`** over `within_cli` and `json_contract`: Windows `38309762033` (`json_contract` 13, `within_cli` 7), macOS `38309764907` (14, 7), arm64 `38309768345` (14, 7) — 0 failed, no compiler warning. **The full suite alone at `3f96546`:** 2,237 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved.

**An orphan daemon, found by reading the green reads whole (`94a7b21`).** The macOS and arm64 reads at `3f96546` ended with the
runner's cleanup line `Terminate orphan process: pid … (delulu)` — absent at `4f0fdd3`, before the node witness existed. The witness
removed its state directory before its broker guard dropped, and `broker stop` finds the daemon through that directory, so the
daemon outlived the test; three such daemons were running in the VM. The guard now drops first: none after, read again on the
runners: `within_cli` at `94a7b21` — Windows `38310713607`, macOS `38310716813`, arm64 `38310720046`, 7 passed each and no orphan `delulu` in any cleanup.

**A duplicated heading in this run's own records.** `52b2572`'s patch inserted `HANDOFF.md`'s new §11.5 lessons before `### 11.6`
with the heading repeated at the end of its text, so `master` carried `### 11.6` twice; the next patch's anchor check counted 2 and
stopped before writing. Fixed in this commit; §11.5 has the lesson.

**P9-02 is built** — two grant files (D-V2-119) and two lease nodes (D-V2-120); `secret_reach_expansion` waits for P9-04.
**Next: P9-03**, proposals whose approval reads this diff — read it against the code first (`V2_P9_DESIGN.md`).

## 2026-10-12 — routine run 20 (postscript): `1fc3aa0`'s push runs read; a gate for a heading written twice

**`1fc3aa0`'s push runs** (P9-02's lease nodes, the orphan fix and their records): CI `38311372015` success on every job (`miri-slow` and `heavy-gates` skipped on a push), `openshell` `38311371940` success; `actors_pingpong` — arm64 MEASURED 2.90x (control 4.11x, bar 1.50x), macOS NOT MEASURED (3 hardware threads), Windows and Ubuntu NOT MEASURED (busy or SMT).

**The repeated-heading gate (`b566969`)** — loop engineering from this run's own slip (`52b2572` put `HANDOFF.md`'s `### 11.6` in
twice, every document gate green). `repository_structure` reads every markdown file the Survey lists and refuses a heading repeated
on the next heading line; blank lines between do not hide one, and a section name reused under another date or a repeat inside
fenced code is not one. Measured before it was written: no such repeat in the tree; `52b2572`'s `HANDOFF.md` has one. Witnessed red
with that duplicate put back (`HANDOFF.md:1293` named), green once restored; the rule's own synthetic cases beside it. Read on the
runners at `b566969` over `repository_structure`: Windows `38311565631`, macOS `38311568364`, arm64 `38311571579` — 4 passed each, the two new tests named. **The full suite alone at `b566969`:** 2,239 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, and no broker daemon left running after it.

## 2026-10-12 — routine run 20 (last): P9-03 read against the code — the study's premise measured false for DL0703

Before P9-03 is built, as `V2_P9_DESIGN.md` did for P9-01 (its P9-03 bullet has the whole reading). **§4.10's premise is false for the
commonest refusal:** a DL0703 carries its grant in prose only (`run --json`: `"repairs": []`), and `run --report-out` records no
diagnostic for a refused run (`diagnostics: []`, `outcome: {exit: 1, ran: true}` on `05_capabilities` with `--grant console`). What
holds instead: the refusal's diagnostic carries the call's span (`48:18`), and P9-01's `requirements` map that very span to
`fs.read=./config` — so a proposal can be the static authority of the refused call site, computed by the checker and never worded
by the program or the runtime. Build order recorded: the report records the refusal with its span; `grants propose --from REPORT`
maps it to one grant line (a hidden site proposes nothing); a proposal kind beside the Guard's queue; approval mints a child through
`attenuate` (the tree's own `⊑`); auto-approval is `grants diff BOUNDARY PROPOSAL` adding nothing (D-V2-119). Three hypotheses named
there, each to be witnessed when built.

## 2026-10-12 — routine run 20 (closing): P9-03 step 1 — a refused run's report records the refusal at its call (D-V2-121)

**`ec026a0`'s and `9357ddc`'s push runs:** CI `38313420752` (`ec026a0`) and CI `38313748666` (`9357ddc`) — success on every job (no `openshell` run: neither push touched its paths).

**P9-03 step 1 (`c2dd5f0`, D-V2-121)**, from the reading recorded above: the ordinary run's report carries the fault that ended
the run as the envelope's diagnostic object, with the call's span — the span P9-01's requirements map to the narrowest grant.
Witnessed red first (`diagnostics: []`); the witness holds the report's span to `authority --within`'s `at` (48:18,
`fs.read=./config`). Mutants M296 and M297 red, the control green; clippy clean. **Read on the runners at `c2dd5f0`** over
`within_cli`, `json_contract` and `schema_cli`: Windows `38314043269`, macOS `38314046573`, arm64 `38314049378` — each job green over its three targets (`within_cli` 8 passed on each), no orphan process. **The full suite alone at `c2dd5f0`:** 2,240 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left.

**Next for P9-03:** a sandboxed run's report (written by its host), then `grants propose --from REPORT`.

## 2026-10-13 — routine run 21: CI read green; P9-03 step 1b — a sandboxed run's report records the host's refusal at its call (D-V2-122)

**Start (00:08 UTC).** `master` = `fc95094`, the session's branch the same; every `origin/claude/*` branch read after unshallowing —
none holds a commit `master` lacks but the two run 10 recorded as superseded (`d3f0d52`, `7d6a22b`). `survey check` ok (1,504 nodes),
`doctor --check` all checks passed. **`fc95094`'s push run** `38315802078` (CI) — success on every job (`miri-slow` and `heavy-gates`
skipped on a push); `actors_pingpong` — arm64 MEASURED 2.97x (control 4.12x, bar 1.50x), macOS NOT MEASURED (3 hardware threads; 1.58x
observed), Windows and Ubuntu NOT MEASURED (busy or SMT); sandbox properties as before (macOS's reads `not confined`). The nightly of
2026-10-12 (`38234415113`) and the release dry run (`38248184651`) at `e95acf1`: success. `gh` is installed in this VM; CI was read
with `curl` and the MCP tools.

**P9-03 step 1b (`ebf393c`, D-V2-122)** — measured first: a sandboxed run of `05_capabilities` with `--grant console` wrote
`diagnostics: []` (the refusal in `sandbox.denied` as prose and on the guest's standard error); the greeter package the same. The
host now keeps its own refusal of a capability request with the span the request named, clears it on a later `Ok`, and at a
failing goodbye its report records it as the ordinary run's report records a fault — same code, message, file, start and end. The
span is the guest's word: placed only on character boundaries of the program the host sent (inside a character it panicked the
renderer); another file's is omitted by the renderer (C83 — so the host's own file check was dropped as a wall no witness could
fail). A guest's own fault stays out of the host's report. Witnessed red first (`within_cli`, a file and a package); a channel
unit test and a guest unit test beside it; mutants M298–M306 red (M306's first form did not compile and was rewritten), the
control green; clippy clean. **Read on the runners at `ebf393c`** over `within_cli`, `sandbox_run_cli`, `sandbox_modes_cli`,
`sandbox_external_cli`, `schema_cli` and `delulu`'s unit tests: Windows `38327053568` (226 passed), macOS `38327056172` (241), arm64
`38327058865` (262) — 0 failed, both new tests named, no compiler warning, no orphan `delulu` (Windows' one orphan is `vctip`, the
MSVC toolchain's telemetry). **The full suite alone at `ebf393c`:** 2,243 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0,
the tree unmoved, no daemon left.

**Found while building it, for the next step:** `schema_cli` validates only reports of runs that SUCCEED — a refused run's report
(which carries a diagnostic since D-V2-121) was never held to the `run-report` schema; both kinds validate by hand
(`schema validate run-report`), and a witness for them goes with the next slice. And `grants propose` cannot map a refusal through
the requirement list as it stands: `push` keeps each grant's FIRST call only (a refusal at a second call of the same grant matches
no `at`), and the report records no grant set (`sandbox.granted` is the sandbox level) — so the walk must keep every minting call,
and a proposal is the refused call's own static requirement.

## 2026-10-13 — routine run 21 (continued): P9-03 step 2 — `delulu grants propose REPORT` (D-V2-123)

**`grants propose REPORT` (`93a74b7`, D-V2-123)** — the narrowest grant for the call a refused run was refused at: the report's
refusal (code and span, D-V2-121/D-V2-122) and the program's source as it is now; the proposal is the static requirement of the
minting call at exactly that span, filtered from P9-01's requirement list by the call's kind and literals — never a line the runtime
or the program worded. **Measured while step 1b was built, and the reason for `ScopeWalk.sites`:** the requirement list keeps each
grant's FIRST call, so a refusal at a second call of a grant matched no `at`; the walk now keeps every minting call. Nothing is
proposed (exit 1, a `reason`) for a clean run, a code a grant does not answer (only DL0703 — every DL0703 is raised at a mint), an
unplaced refusal, a package (open, with `--within` for one), a program that does not check, a computed scope, a mint whose grant
names the operator's data, or a program changed since the run; the report's offsets are checked on character boundaries before
anything slices them (M312 without the check panicked the binary). The report is a positional — a `--from` documented for `grants`
would reach every verb of it.

**Two gates the verb met:** `json_contract`'s success table (a fixture whose report a real refused run writes) and its broker-verbs
sweep, which refused the verb as "advertised but never driven" until it was — one more gate for a new surface, now in
`docs/CLOUD_ROUTINE.md` step 7a. **And `schema_cli`'s gap closed:** a refused run's report, ordinary and sandboxed, is held to the
`run-report` schema (`a_refused_runs_report_matches_the_run_report_schema`; mutants M314 and M315, a malformed diagnostic in each
report, red).

**Witnessed:** `within_cli`'s `grants_propose_reads_the_refused_calls_own_requirement_from_the_report`, red first (no verb), and the
generated `grants_propose_names_the_dropped_grants_call_for_every_kind` (eight requirements across seven kinds, each dropped once;
each proposal exactly the dropped call's requirement, both hosts for the call that names two). Mutants M307–M316 red — among them
M307 (only sites that are a requirement's first call), M309 (the whole requirement list) and M310 (any literal of the kind) — the
control green; clippy clean. **Read on the runners at `93a74b7`** over `within_cli`, `json_contract`, `schema_cli` and `delulu`'s unit
tests: Windows `38328775607` (207 passed), macOS `38328777737` (222), arm64 `38328781069` (241) — 0 failed, every new test named, no
compiler warning, no orphan `delulu`. **The full suite alone at `93a74b7`:** 2,246 passed, 0 failed, 16 ignored (162 binaries), cargo
exit 0, the tree unmoved, no daemon left.

## 2026-10-13 — routine run 21 (continued): `master` red on `187494a` — two witnesses that measured more than their subject, fixed

**`187494a`'s push runs:** `ocsf` `38328124557` success; CI `38328124600` **failure** — `test (ubuntu-24.04)` and `test
(macos-latest)` red, every other job green (Windows' test job included; `actors_pingpong` arm64 MEASURED 2.91x, control 4.10x, the
others NOT MEASURED). Neither red is this run's diff, and neither is the product: each witness measured something its subject
does not control. Fixed on the harness branch, each read red first, before `master` moved.

**Ubuntu — `monitor_cli`'s `a_monitor_quarantines_the_run_that_keeps_probing_and_the_chain_says_why`** (`ea31897`): "three refusals
came first" counted `REFUSED:` lines on the program's standard output and found two. The trace held all three refusals (seq 3, 7,
11) and a fourth operation, the `println` of the third, refused DL1403 at 24:5 — the monitor quarantined the run between its third
refusal and the line that reports it. Run 19's lesson again (a witness whose subject races the program is timing-shaped though it
times nothing). Witnessed: with work put between a refusal and its line, the old assertion red 3 of 3 in the VM; the third refusal
is now counted in the TRACE, where the program records it at the refusal — green 3 of 3 with the window open, and `monitor_cli`
19 passed 3 of 3 with all four CPUs busy; a monitor firing on two refusals turns the new assertion red. Read on `ubuntu-24.04` at
`ba60f60` (`38330138742`): 19 passed, the quarantine 34 ms after the third refusal. **Found beside it, RW 4.62:** the trace records an
operation at its dispatch, before the custody gate decides, and a custody fault adds no record after it — so the trace listed the
`println` that never printed.

**macOS — `jail::macos_tests::the_host_reads_a_processes_peak_footprint_and_it_follows_what_is_touched`** (`fe621c6`): "64 MiB
touched moved the peak footprint only from 66504000 to 105874816". The reader is a PEAK (`ri_lifetime_max_phys_footprint`) of the
TEST process, which every unit test's thread shares. Three witness forms on the runner, each its own commit: `ba60f60` — another
thread's 128 MiB touched and freed at once — GREEN (`38330136441`): the guess did not reproduce; `4c2ba86` — the block held 1.5 s,
then freed, every reading printed — GREEN (`38330478020`) and the measurement: the peak was 136,987,904 while held and before, and
204,146,048 after 64 MiB, so a freed block stays in the footprint and the first guess (a footprint falling under its peak) is
refuted; `0e92a3e` — a SAME-size 64 MiB block freed first — **RED** (`38330675493`: 69,878,912 to 69,911,744): the allocator served
the test's block from memory the footprint already counted. The witness now reads a child process of its own
(`footprint_probe_child`), with that window kept open in the parent; read on macOS at `fe621c6` (`38331001159`): `delulu`'s unit tests
193 passed, both footprint tests named. Mutant M317 — the reader answers a constant — read RED on the runner as its own commit
(`e879816`, `38331248991`: "from 1 to 1"), reverted in `995581e` (the tree byte for byte `fe621c6`'s). (`fe621c6`'s message names the
first form wrongly as "`0e92a3e`'s parents"; it is `ba60f60`.)

## 2026-10-13 — routine run 21 (continued): `authority DIR --within` and `grants propose` for a package (D-V2-124)

**Run 20's open item (4), half of it** (`9473789`, D-V2-124): both P9 verbs refused a package in words. `authority DIR --within`
now decides the requirement list `authority DIR --grants` names over every module, the counterexample's call named in its
module's own file (relative to the package, as `authority DIR` names a module); `grants propose` finds a package run's refused
call in the flattened text the run executed and proposes that call's requirement. A `.dwx` stays refused (a manifest, not
calls). A first form read the package's own requirement list for a proposal and was removed before commit: no witness could
tell it from the flattened text's (a proposal filters by the call's kind and literals).

**Witnessed:** `within_cli`'s `a_package_is_checked_against_a_boundary_and_its_refusal_is_proposed_for`, red first (exit 2); P9-01's
generated corpus walks every package under `examples/` (three: one requirement, two with none); the refusal table's package row
became a `.dwx` row. Mutants M318–M321 red, the control green; clippy clean. **Read on the runners at `9473789`** over `within_cli` and
`json_contract`: Windows `38332620471` (`json_contract` 13, `within_cli` 12), macOS `38332623451` (14, 12), arm64 `38332625666` (14, 12) — 0 failed, the new test named, no compiler warning, no orphan `delulu`. **The full suite alone at `9473789`:** 2,247 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left.

**`f3d4076`'s push runs** (the records of the red episode): CI `38332234517` success on every job (`miri-slow` and `heavy-gates` skipped on a push) — **`master` green again**, the two fixed witnesses green on Ubuntu and macOS — and `openshell` `38332234499` success; `actors_pingpong` arm64 MEASURED 2.95x (control 4.11x, bar 1.50x), the others NOT MEASURED.

## 2026-10-13 — routine run 22: CI read green; P9-03 step 3 — the proposal store, approval under the approver's node (D-V2-125)

**Verified first:** run 21's last push runs, at `ae84188` — CI `38333635019` success on every job (`miri-slow` and `heavy-gates`
skipped on a push), `openshell` `38333634973` success; `actors_pingpong` arm64 MEASURED 2.88x (control 4.11x, bar 1.50x), Windows,
Ubuntu and macOS NOT MEASURED (busy or 3 hardware threads); each test job's sandbox properties annotated as before. The Survey
(`ok`, 1504 nodes, 14154 edges) and `doctor --check` (26 checks) passed at the start; no `origin/claude/*` branch holds a commit
`master` lacks beyond the two run 15 recorded as superseded.

**Run 21's open item (2), P9-03 step 3** (`45877b7`, D-V2-125): `grants propose REPORT --node g_ID` files the refused call's
requirement with the broker as the request of the node the refused run held; `grants proposals` lists each with the
proposer's account (data) and what an approval would mint; `grants approve pp_ID --parent g_ID` delegates, under the APPROVER's
node, the node's authority joined with the ask — the Guard's mint gate and `⊑` bound it, with no new check; `grants reject pp_ID
--reason` keeps the reason for the proposer. A new broker module, `proposal.rs`; four IPC requests; the daemon's lost-record rule
(RW 4.61) for each. Two choices measured while it was built: the approval mints the JOIN (a lease of the ask alone is refused at
the re-run's first call the old node admitted), and so `grants proposals` shows the join (`mints`) — a proposal names its node,
and an approval carries that node's authority, so the approver reads the whole new node.

**Witnessed:** `within_cli`'s `a_proposal_is_filed_approved_under_the_approvers_node_and_rejected_with_a_reason_the_agent_reads`,
red first (`--node` a usage error, exit 2, after the lease run's expected refusal); `brokerd`'s
`a_proposal_or_its_approval_whose_record_was_lost_does_not_happen`; seven unit tests in `proposal.rs`; `json_contract`'s
broker-verbs sweep drives the four forms (its advertised-verbs check refuses a verb it never drove). Mutants M322–M336 red, the
control green. **M331 first survived:** the lost-record witness filed the same ask again once the chain was free and the dedup
answered with the leftover's id; it now reads the list before anything else is filed (red). Clippy clean.
**Read on the runners at `45877b7`** over `within_cli`, `json_contract` and `delulu`'s unit tests: Windows `38352400253` (205 passed), macOS `38352403113` (221), arm64 `38352405496` (239), and the broker's unit tests on Windows `38352408072` (`proposal`, 7) — 0 failed, every new test named, no compiler warning, no orphan `delulu`.
**The full suite alone at `45877b7`:** 2,256 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left.

## 2026-10-13 — routine run 22 (continued): P9-03 step 4 — `grants auto-approve`, the operator's standing node (D-V2-126); P9-03 complete

**`11c9d94`'s push runs** (step 3's records): CI `38353462220` **failure** — `test (ubuntu-24.04)` only: `monitor_cli`'s two monitors revoked by their own dead-men (DEADMAN-BUSY-1, found and fixed below); every other job green, `actors_pingpong` arm64 MEASURED 2.99x (control 4.12x), the others NOT MEASURED; `ocsf` `38353462196` and `openshell` `38353462310` success.

**Step 3's new audit records checked against OCSF** (the routine's rule for a change to the records): a chain holding two
`proposal`, one `proposal_approve` and one `proposal_reject` record, exported with `audit export --format ocsf`, re-verifies
(`audit verify --ocsf`, 10 events) and validates against OCSF 1.8.0's published schema with `scripts/ocsf-validate.py` — 0
problems over classes 0, 2004 and 3005 (a filing and an approval are Base events, a rejection a Finding); its self-test caught
11 of 11 mutations.

**Step 4** (`a7c409c`, D-V2-126): `grants auto-approve --parent g_B` offers every pending proposal to the broker for approval
under B — the tree's own `⊑` decides what each would MINT — and leaves each refused one pending with its code and what
`grants diff` finds its mint adds beyond B. The boundary is a node rather than a grant file: what an approval mints is a node,
and a file translated into the node domain would approve a console wherever it grants `fs.write` (`console` is the `Write`
effect there); a node is also the operator's delegation in the tree it governs — revoking it revokes everything it approved.
It never takes the owner code: `--owner` is refused (saying why) and `DELULU_GUARD_OWNER` is not read, so guarded authority
is approved by a person, one proposal at a time. **P9-03 complete** (`V2_P9_DESIGN.md` items 1–4, hypothesis 5 witnessed).

**Witnessed:** `within_cli`'s `auto_approval_approves_under_the_standing_node_what_it_bounds_and_never_guarded_authority`, red
first (no verb, exit 2); `json_contract`'s sweep drives the verb. Mutants M337–M341 red, the control green — M338 (the
`--owner` refusal removed) only after the witness asserted the refusal's words: without them `--owner` is refused as an
unknown flag anyway, exit 2, and the dedicated refusal was code no witness could fail. Clippy clean. **Read on the runners at
`a7c409c`** over `within_cli` and `json_contract`: Windows `38353918721` (27 passed: `json_contract` 13, `within_cli` 14), macOS `38353921081` (28), arm64 `38353923894` (28) — 0 failed, the new test named, no compiler warning, no orphan `delulu`. **The full suite alone at `a7c409c`:** 2,257 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left.

## 2026-10-13 — routine run 22 (continued): `master` red on `11c9d94` — DEADMAN-BUSY-1, a healthy monitor revoked while the daemon was held (D-V2-127)

**The red** (`11c9d94`'s CI `38353462220`, `test (ubuntu-24.04)` job `115150876593`; every other job green): `monitor_cli`'s
`a_monitor_quarantines_the_run_that_keeps_probing_and_the_chain_says_why` and `a_monitor_sees_nothing_outside_its_own_subtree` —
each monitor exited 1, "the broker's dead-man revoked `g_…` and every run under it: this monitor sent no beat within 2000 ms".
In the first, nothing reached the chain between the arming (seq 5) and the firing (seq 6) — not even the test program's lease
redemption: the daemon served nothing for over two seconds. Not reproduced in the VM: `monitor_cli` 3 of 3 green quiet, 3 of 3
with all four CPUs busy; nothing this run had changed reaches the monitor or the serve loop.

**The class, witnessed** (the question the window asked: what holds a serial daemon for seconds? — RW 4.53's answer, a request
waiting on an audit chain another writer holds): `brokerd`'s `a_beat_sent_in_time_keeps_its_dead_man_though_a_held_request_
delays_its_answer` — 25 beats every 100 ms against a 1,000 ms dead-man while one request waited 2 s on a held chain — **RED**,
every beat answered DL1403, the holder revoked. The daemon's half fixed (each whole request stamped on arrival; dead-men fired
as of the handled request's arrival; a beat and a disarm judged by arrival, a beat's period restarted at its answer), the
monitor's half: `monitor_cli`'s `a_monitor_waiting_on_a_held_daemon_is_not_revoked_by_its_own_dead_man` — a real `monitor
watch`, `--death-after 1000`, the daemon held 2.5 s — **RED 2 of 2**: its `List` queued behind the held request, and the beat it
sent once the answer came was late. Fixed: while the watch is blocked inside a request to the daemon, and only then, a thread of
the monitor's own beats every quarter period. Both green; `monitor_cli` 20 passed and the daemon's dead-man tests 9 passed,
each three more times with all four CPUs busy — `a_busy_daemon_still_fires_an_unbeaten_dead_man` and
`a_stopped_monitor_is_a_dead_one_and_says_so_when_it_resumes` among them. Mutants M342 (fire as of now), M343 and M344 (a beat
judged when handled), M345 (the beater silent) red, the control green; **M346 (the beater beats whatever the watch does)
SURVIVES** — no witness reaches a watch wedged outside the daemon: RW 4.63, opened. Clippy clean; Windows and macOS linted from
Linux (`check-other-os.sh`). `e074330`.
**Read on the runners at `e074330`** over `monitor_cli` and `delulu`'s unit tests: Ubuntu `38357155428` (233 passed), Windows `38357158233` (198), macOS `38357161227` (214), arm64 `38357164145` (233) — 0 failed, both new witnesses named on each, no compiler warning, no orphan `delulu`. **The full suite alone at
`e074330`:** at `e074330`, 2,258 passed and 1 failed — the runtime's thread-stack gate, red for a thread no list named, the beater (`ae9dffe` lists it; `delulu-runtime`'s library then 262 passed); at `ae9dffe`, with these records in the tree, 2,259 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left.

## 2026-10-13 — routine run 22 (continued): `76f83a2` red on Ubuntu — DEADMAN-BUSY-1's arming half; P9-04 read against the code

**`76f83a2`'s push runs:** CI `38359276890` **failure** — `test (ubuntu-24.04)` only: `brokerd`'s `an_idle_daemon_fires_an_unbeaten_dead_man_without_any_request` (DEADMAN-BUSY-1's arming half); every other job green, Windows' and macOS' test jobs among them; `openshell` `38359276774` success. The red test (`an_idle_daemon_fires_an_unbeaten_dead_man_without_any_request`, "never before
its period: 201.8 ms") took its clock from the arming's ANSWER, which comes only once the arming's record is written. Moved to
the daemon's own records (the arming's and the firing's), with the arming written under a 200 ms hold of the chain, it read red
2 of 2 — fired 99 ms after the arming RECORD: the product's, not the test's. The deadline was decided before the record could
wait on the chain; the first period now starts when the arming is answered (D-V2-127 item 6, `eb7b8f8`): green 3 of 3 (301–302
ms after the arming); mutants M347 and M348 red, the control green; the daemon's dead-man tests and `monitor_cli` green, twice
more with every CPU busy; clippy clean; Windows and macOS linted from Linux. **Read on the runners at `eb7b8f8`** over `delulu`'s
unit tests and `monitor_cli`: Ubuntu `38361582689` (233 passed), Windows `38361586297` (198), macOS `38361590312` (214), arm64 `38361593590` (233) — 0 failed, the idle test named on each, no compiler warning, no orphan `delulu`. **The full suite alone at `eb7b8f8`, with these records:** 2,259 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left.

**P9-04 read against the code** (`V2_P9_DESIGN.md`): `Cap[Http]` has one method, `get(url)`, and the egress client's request no
header — there is no request a secret can ride on; a `Secret` leaves a program only through `expose`, behind `Declassify`, after
which nothing follows it; the grant grammar is `secret:NAME=VALUE | secret:NAME=env:VAR`, not the study's `secret=NAME@…`; under
the daemon the bytes are the broker's (invariant 23), so the host's egress client would ask the broker at send time, naming the
destination. Build order proposed there: a request with headers first, then a secret in a header position, the audience in the
grant, the broker's audience dimension, and `authority`'s per-secret destinations.

## 2026-10-13 — routine run 22 (last): `44f5b7f`'s push run read green — `master` green again

CI `38362909613` at `44f5b7f` — success on every job (`miri-slow` and `heavy-gates` skipped on a push), Ubuntu's test job among
them: the red of `11c9d94` and `76f83a2` (DEADMAN-BUSY-1's two halves, D-V2-127) is over. The run's records end here; its
postscript commit's own push run is the next run's to read.

## 2026-10-14 — routine run 23: CI read green; P9-04 step (a) — `http.get_with`, a request with headers (D-V2-128)

**Start.** `date -u` 2026-10-14 13:59. The closing entry exists (run 19) and is kept current below. `master` at `b7a5c3b`
(run 22's last record); every `origin/claude/*` branch read after `git fetch --unshallow` — none holds a commit `master`
lacks but the two run 10 recorded as superseded (`7d6a22b`, `d3f0d52`). `survey check` ok (1,505 nodes), `doctor --check`
all checks passed. `gh` present at `/usr/local/bin/gh`, not needed (the REST API answers `curl`).

**CI read since run 22's last record:** `b7a5c3b`'s push run `38365143257` — success on every job (`miri-slow`, `heavy-gates`
skipped on a push); the nightlies `38380513605` (2026-10-13) and `38534326546` (2026-10-14) — success on every job,
`miri-slow` on all three crates and `heavy-gates` included. `actors_pingpong`: arm64 MEASURED 2.93x (control 4.11x) on the push
run and 2.91x (4.12x) on the nightly; Ubuntu and Windows NOT MEASURED (the machine never gave 3x perfectly parallel work); macOS
NOT MEASURED (3 hardware threads). Sandbox properties as before on every OS (macOS: reads not confined). **`master` green.**

**P9-04 step (a)** (`V2_P9_DESIGN.md`'s build order; D-V2-128). **Witnessed absent first:** on the current binary a program
calling `h.get_with(url, headers)` was refused `DL0405` (`Cap[Http]` has no method `get_with`); and a `Map` could not cross the
sandbox channel (`WireValue::from_value` refused every kind it did not name). **Built:** `Cap[Http].get_with(url: Str, headers:
Map[Str, Str]) -> Result[Str, NetErr] ! {Net}` — the checker's arm, `PRIM_TABLE` 82 → 83 rows, `PRIM_TABLE_VERSION` 5 → 6; the
runtime's arm (the map's entries in its own order); the egress client's `check_headers` (each rule a refusal: an RFC 9110 token
for a name; printable ASCII, space and tab for a value, no whitespace at its ends; no two names differing only in case; 32 / 128 B
/ 8 KiB / 16 KiB; none of `RESERVED_HEADERS` — `Host`, the framing and connection headers, `Proxy-*`, `Accept-Encoding`) before
anything is resolved; the origin rule (headers only on hops to the URL's own host and port); the hop record's header NAMES;
`custody_op_for` and the trace (`Net`, the URL only); `WireValue::Map` (a key twice refused, a capability inside reminted); the
`run-report` schema's optional hop `headers`; the conformance anchor with four witnesses (an entrenched edit, recorded); the
generated reference (354 of 354 anchors covered). The operator's egress note now names "the request to `URL`", `get` and
`get_with` alike.

**Witnesses** (each falsified): `egress`'s `a_programs_headers_are_checked_before_anything_is_resolved` (20 spellings and every
reserved name in two cases, each refused by its OWN rule, nothing resolved or sent), `headers_that_follow_the_rules_are_sent_
lower_cased_in_the_order_given` (and each bound exactly), `the_headers_go_to_the_origin_the_program_named_and_to_no_other` (six
hops: same origin, same origin spelled with its port, another host, another port, back — 2, 2, 2, 0, 0, 2 headers; no value in
the record), and over real TLS `a_programs_headers_arrive_over_tls_and_the_host_header_stays_the_checked_one` (one `User-Agent`
— the program's replaces the client's — one `Host`, the checked one); `channel`'s `a_map_crosses_and_one_that_names_a_key_twice_
is_refused`, `a_capability_inside_a_map_the_host_returns_is_reminted`, `a_guests_request_with_a_refused_header_is_refused_by_the_
host_and_recorded`; `egress_cli`'s `a_request_with_headers_leaves_as_tls_at_l0_and_from_a_guest_and_the_report_names_them` (and
the report validates as `run-report`) and `a_refused_header_connects_nowhere_and_says_why_to_the_operator`; `monitor_cli`'s
`a_request_with_headers_answers_to_the_daemons_custody` and its sandboxed twin (the chain's `use allow` and the `deny` citing it,
reason `header`). **Mutants M349–M367:** every one RED, the control green — M349 (the value rule off), M350 (the reserved list
compared case-sensitively), M352 (the edge rule), M353 (an empty name), M354 (the total bound), M355 (the count bound off by
one), M356 (headers on every hop), M357 (the port not compared), M358 (names not lower-cased), M359 (the transport sends none),
M360 (the record keeps values), M361 (a map's capabilities not reminted), M362 and M367 (a key twice accepted, host and guest),
M363 (the primitive drops the map), M364 (`custody_op_for` without `get_with`), M365 (the trace's effect), M366 (the schema
without `headers`). **M351 SURVIVED its first run** — the duplicate check compared the WRITTEN name, and the witness's colliding
pair put the lower-case spelling second, which both forms catch; the witness now spells the pair both ways round, and M351 is red.
Clippy clean; macOS, Windows, arm64 and musl linted from Linux (`check-other-os.sh`). `bee695f`.

**Read on the runners at `bee695f`:** `delulu-runtime`'s library — Windows `38581940557 (268 passed)`, macOS `38581945945 (269)`, arm64 `38581950457 (269)`;
`delulu`'s `egress_cli`, `monitor_cli` and `schema_cli` — Windows `38581958301 (31 passed)`, macOS `38581962693 (32)`, arm64 `38581967281 (33)`.
0 failed on every read, every new test named on each, no compiler warning, no orphan `delulu` (Windows' only orphan is `vctip`, the MSVC toolchain's telemetry).

**The full suite alone at `bee695f`:** 2,270 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left.

## 2026-10-14 — routine run 23 (continued): `fa2d34e` red on supply-chain — rustls 0.23.46 (RUSTSEC-2026-0346 to -0349)

**`fa2d34e`'s push runs:** CI `38584248231` **failure** — `supply-chain` only (`cargo deny check advisories`): four advisories
RustSec published that day against `rustls 0.23.45` — 0346 (`Debug` of client session values exposing TLS secrets), 0347 (data
buffered for the outer server after an ECH rejection), 0348 (the `brotli` certificate decompressor, a feature this tree does not
enable) and 0349 (a pre-authentication ServerHello pinning later key shares to a classical group). Every other job green;
`openshell` `38584248260` success. A records commit's red, run 4's lesson exactly: the gate reads RustSec as it is on the day.
**Witnessed in the VM** (`cargo install cargo-deny --locked`, 0.20.2): `cargo deny --all-features check advisories` FAILED with the
four; `cargo update -p rustls` (0.23.45 → 0.23.46) — and it is ok, with bans, licences and sources ok. The update also re-unified
errno, quinn-udp, rustix, tempfile and winapi-util onto `windows-sys` 0.61.2 (from 0.52.0, within their own ranges): read in the
lockfile's diff, then Windows and macOS linted from Linux, and the egress client's TLS tests (35) green. `deny.toml` records it.
`5fa3261`; its push run ``38586917971``: **failure** — `test (ubuntu-24.04)` only (the DIR verify timing witness — the next entry); `supply-chain` green with rustls 0.23.46, every other job green.

## 2026-10-14 — routine run 23 (continued): RW 4.62 closed — a refused operation is followed by its refusal in the trace (D-V2-129)

**Witnessed first** on the current binary: under the daemon, a leased write the Guard holds (`fs_write:*` guarded) was refused
DL1410, no file was written, and `--trace-out` held one record — `write_text`. **The class, from the mechanism** (routine run 19's
rule): any fault between the dispatch record and the operation's end leaves the same unpaired record — the custody gate, the
broker's refusal of a daemon `expose`/`verify` (found by reading the dispatch: `expose_secret` faults after it), and the effect's
own refusal (DL0904, a path outside the capability — with no broker at all). **Fixed** (`interp.rs`: `trace_dispatch` returns what
it recorded; `trace_refusal` follows a custody denial and any faulted result): `<method>.refused`, the dispatch's effect and
`cap_kind`, the call's span, `detail` the code and its words — the code alone where a secret was involved. The WASM engine, which
traces `read_text` at dispatch too, writes the same record (`push_refusal_trace`), so the engines' traces stay byte-identical.
**Witnesses:** `guard_e2e`'s `a_use_the_custody_refuses_is_followed_by_its_refusal_in_the_trace` (a guarded write; a guarded
`expose`, `detail` exactly `DL1410`; a guarded read under both engines, the traces equal) and `cli`'s `an_operation_its_effect_
refuses_is_followed_by_its_refusal_in_the_trace` (DL0904 under both engines). Mutants M368 (no record after a custody denial),
M369 (none after a faulted result), M370 (a secret's refusal carries its words), M371 (another effect label), M372 and M373 (the
WASM engine's two records) — every one RED, the control green. The trace-reading targets (`cli`, `actuate_cli`, `estop_cli`,
`json_contract`, `sandbox_modes_cli`, `plugin_load_cli`, `dead_man_cli`, `robotics_demo`, `satellite_demo`, `core_invariance`,
`atlas_chain`) green unchanged. A false lead re-run before use: a `guard policy set fs_read:*` looked as if it dropped
`fs_write:*` — the default policy never had it; the hand run had set it. **Residual, RW 4.64 (opened):** the WASM engine checks
custody before it traces console, clock and rand and refuses a hostile path before tracing `read_text` (both predating this
change), and writes no refusal after `rand.int`'s `hi <= lo` fault, which the interpreter now records (this change's own).
Clippy clean; the five other targets linted from Linux. `7c653e0`.
**Read on the runners at `7c653e0`** (`guard_e2e` and `cli`): Windows ``38587247926` (56 passed)`, macOS ``38587253051` (55)`, arm64 ``38587258506` (55)`. 0 failed on each, both new witnesses named, no compiler warning, no orphan `delulu`.
**The full suite alone at `7c653e0`:** 2,272 passed, 0 failed, 16 ignored (162 binaries), cargo exit 0, the tree unmoved, no daemon left.

## 2026-10-14 — routine run 23 (continued): `5fa3261` red on Ubuntu — the DIR verify timing witness measured the runner (D-V2-130)

**`5fa3261`'s push run** (CI `38586917971`): **failure** — `test (ubuntu-24.04)` only, `delulu-check`'s `dir::tests::verify_is_
comfortably_fast_for_per_load_use` ("verify per-load must be fast for per-load use: 5.745101ms" against 5 ms); 2,269 passed and 1
failed in that job; `supply-chain` green again (rustls 0.23.46), every other job green. The commit changed only `Cargo.lock`, and
`delulu-check` uses none of what moved; the test was green on `fa2d34e`. **Reproduced by starving the VM** (eight busy loops on four
CPUs): 2.2 ms quiet, 6.8–7.4 ms loaded — red 3 of 3. **Fixed (D-V2-130):** the bound is relative to a control timed in the same
window — a check of the same source; best of seven interleaved rounds each; ratio under 5 (2.1 quiet, 2.0–2.6 loaded), a 250 ms
absolute ceiling. Green 5 of 5 with every CPU busy; mutant M374 (verify re-checks ten times) red, the control green; clippy clean.
The class enumerated: the only speed claim in absolute time (four other absolute time assertions are ceilings of 8–60 s on things
that must end). `76f54a0`. **Read on the runners** (`delulu-check`'s library, the ratio printed): `delulu-check`'s library — Ubuntu `38590951265` (248 passed; ratio 1.90, 1.70 ms a verify), Windows `38590955781` (248; 2.02, 2.19 ms), macOS `38590961151` (248; 1.91, 1.16 ms), arm64 `38590966445` (248; 1.98, 1.77 ms).

## 2026-10-14 — routine run 23 (continued): `6971d9c`'s push run read green — `master` green again; RW 4.64 closed — both engines trace a refusal the same

**`6971d9c`'s push run** (CI `38591853430`): success on every job (`miri-slow` and `heavy-gates` skipped on a push) — the
supply-chain red of `fa2d34e` and the Ubuntu red of `5fa3261` are over; RW 4.62 and D-V2-130 are on `master`.

**RW 4.64** (opened by RW 4.62 the same run). **Witnessed first:** a new two-engine test, `delulu-wasm`'s
`refusal_trace_parity` — one program on both engines, under the crate's own kind of denying fake custody where the case is the
custody's — read the WASM engine's trace `[]` against the interpreter's two records for a refused `println` (it checked custody
before tracing), and one record against two for `rand.int(5, 5)` (D-V2-129's own divergence). **Fixed** (`host.rs`): console,
clock and rand are traced at dispatch (reading the guest's string performs nothing), then the custody decides and
`push_refusal_trace` follows a refusal; a hostile path (an embedded NUL; on Windows a device name) is refused after the
`read_text` record; `rand.int`'s `hi <= lo` writes its refusal. The two traces are byte-identical in five cases (console, clock
and rand refused by the custody; `hi <= lo`; a NUL path, granted `.` so nothing is created or read). Mutants M375–M379 (each
refusal record removed) red, the control green. The WASM crate's tests (91 and the rest), `cli`, `guard_e2e`, `json_contract` and
`core_invariance` green; clippy clean; the five other targets linted. **The two-engine differential** (release, 50,000
programs): — 50,000 programs, `two_engine_differential_gate` ok in 145 s. **The disk filled once** (`No space left on device` linking a test, 28 GB of `target/` after the lockfile
moved every crate's hash — run 6's trap): the incremental caches and the test executables built before the lockfile moved were
deleted, 13 GB free, and the targets re-run green. `b3e17fc`.
**Read on the runners at `b3e17fc`** (`delulu-wasm`'s library and `refusal_trace_parity`): `delulu-wasm`'s library and `refusal_trace_parity` on Windows `38595321024` (86 + 2 passed), macOS `38595325757` (91 + 2), arm64 `38595333185` (91 + 2) — 0 failed, both witnesses named, no compiler warning, no orphan `delulu`
**The full suite alone at `b3e17fc`:** 2,270 passed and 4 failed, 16 ignored (163 binaries) — the four were `doctor_cli` (three) and the Survey's `freshness`, a stale map: `scripts/suite.sh` was edited (this run's loop engineering) after the map was built and before the suite started, which `TREE-MOVED` cannot see; the map rebuilt, both targets green (11 and 5 passed). `suite.sh` now refuses to start on a stale map, witnessed refusing.

## 2026-10-14 — routine run 23 (last): `87c0e2f`'s push run read green

CI `38598988825` at `87c0e2f` — success on every job (`miri-slow` and `heavy-gates` skipped on a push): `master` green at the run's end, with P9-04 step (a), RW 4.62, RW 4.64, D-V2-130 and rustls 0.23.46 on it. `actors_pingpong`: arm64 MEASURED 2.93x (control 4.12x); Ubuntu and Windows NOT MEASURED (never 3x perfectly parallel work); macOS NOT MEASURED (3 hardware threads). The run's records end here; its postscript commit's own push run is the next run's
to read.
## 2026-10-14 — routine run 24: CI read green; P9-04 steps (b) and (c) — a secret in a request, only to its grant's audience (D-V2-131)

**Start.** `date -u` 19:11 (budget to about 22:10). `master` at `48ef512`, the harness branch `claude/zealous-carson-tidpo3` the
same commit. Survey `ok` (1,506 nodes, 14,340 edges) and `doctor --check` all checks passed. Every `origin/claude/*` branch read
after `git fetch --unshallow`: two hold a commit `master` lacks (`7d6a22b`, `d3f0d52`), both superseded since run 10 — nothing
stranded. `gh` is installed with its `GH_TOKEN` refused ("invalid"), as run 10 found; CI read through the REST API (`curl`).

**CI read.** `48ef512`'s push run (CI `38601952201`, run 23's last records): success on every job (`miri-slow`, `heavy-gates`
skipped on a push). `actors_pingpong`: arm64 MEASURED 2.96x (control 4.12x); Ubuntu and Windows NOT MEASURED (never 3x perfectly
parallel work); macOS NOT MEASURED (3 hardware threads). Sandbox properties: Linux x86-64 and arm64 — egress, filesystem (reads
confined to the system paths), host loss and the privilege floor established; Windows — filesystem writes only its own
container folder, reads none of the operator's files; macOS — filesystem confinement absent (reads not confined), as recorded
since run 12. No nightly since run 23's reads (`38534326546`, success).

**P9-04 steps (b) and (c), as one slice (D-V2-131).** Witnessed absent first on `48ef512`'s binary: `get_with_secret` refused
DL0405; `--grant secret:API_TOKEN@127.0.0.1:9=…` read as a secret NAMED `API_TOKEN@127.0.0.1:9` (the program's
`root.secret("API_TOKEN")` faulted DL0703). Built: `Cap[Http].get_with_secret(url, headers, header, secret: Secret[Str])` (`Net`,
no `Declassify`); the grant line `secret:NAME@HOST[:PORT][,…]=VALUE`, each member parsed by the egress client's own URL parser; the
egress client's `request_with_secret` — the secret's header held to every header rule over the whole set, the URL's origin judged
against the audience before anything is resolved (reason `audience`, the program told `Refused`), and the secret carried on a hop
only where the program's headers are AND the hop's origin is in the audience (two walls); a secret granted with no audience is
sent nowhere; each hop records the secret's NAME (`secret` — the `run-report` schema's new optional hop field), never its bytes;
`Secret.map` and the actor boundary carry the audience; under the daemon or a lease a secret is a broker handle, refused DL0904
in words that name the way out (taken by the witness) with NO custody op asked (HTTP-SCHEME-1's rule — the chain holds no `use`
for a request never made); `grants diff` between grant files reports `secret_reach_expansion`. `PRIM_TABLE_VERSION` 7; the anchor
`ref.prim.http.get_with_secret` (four witnesses — the period's third entrenched edit, recorded); the reference regenerated (355 of
355 anchors). Also checked by hand: a `--sandbox` guest faults DL1401 at `root.secret` (no secret crosses the channel — nothing is
sent from a guest; unchanged), and `--engine wasm` refuses `http` itself DL1201 (unchanged, before and after).
**The Survey was asked after the first edits, not before** — `impact mod:crates/delulu-runtime/src/egress.rs` (155 nodes) and
`query doc:docs/design/STAGE1_SPECIFICATION.md` (not entrenched) — recorded as a lapse of the routine's order, not hidden.
**Falsified:** mutants M380–M400 (`scripts/mutants.py`) red but one: the origin judged against the audience before resolution removed (M380), the empty audience let through (M381), the per-hop origin rule ignored for the secret (M382), both walls removed at once (M383b), the hop's `secret` name dropped (M384), `audience` answered as something other than `Refused` (M385), a path let into an audience (M386), the secret's header unchecked (M387), the grant line split at its last `@` (M388), the audience not stored (M389), `Secret.map` dropping it (M390), the mint dropping it (M391), a broker handle sent (M392), a custody op asked for one (M393), the actor boundary dropping a secret's (M394) or a root's (M395), `grants diff` ignoring it (M396) or calling it `scope_expansion` (M397), the checker taking a `Str` for the secret (M398), the schema's hop `secret` mistyped (M399), `PinnedRequest`'s `Debug` printing values (M400) — and **M383, the per-hop audience check alone, survives on the first wall by design** (D-V2-131 item 3); the control green
**Clippy** clean (workspace, all targets, `-D warnings`); **the other OSes linted from Linux** — macOS (arm64 and x86-64), Windows, arm64 Linux and musl linted from Linux after the last code edit (`scripts/check-other-os.sh`, clippy `-D warnings`, exit 0).
**Read on the runners at `91a1eb7`:** `egress_cli`, `broker_cli`, `within_cli` on Windows `38626954263` (25 passed), macOS `38626958695` (26), arm64 `38626962999` (26); `delulu-runtime`'s library on Windows `38626966826` (274 passed, 1 ignored), macOS `38626971957` (275, 1), arm64 `38626976826` (275, 1); `delulu-check`'s `a_secret_rides…` on Windows `38626992133`, macOS `38626999206`, arm64 `38627003325` (1 each) — 0 failed, every new test named, no compiler warning, no orphan `delulu`
**The full suite alone at `91a1eb7`:** 2,283 passed, 0 failed, 16 ignored (163 binaries), cargo exit 0, the tree unmoved, no daemon left

## 2026-10-14 — routine run 24 (continued): MAP-HANDLE-1 — `Secret.map` of a broker-held secret ran over an empty string (D-V2-132); `91a1eb7` and `d61dd70` read

**Found** while reading the broker's secret path for P9-04 (d) (read-only, the suite running): the interpreter's `Secret.map`
hands its function `s.reveal()`, and a daemon or leased run's secret is a HANDLE whose `reveal()` is `""`. **Witnessed on
`48ef512`'s binary** (before any edit): `s.map(fn(x: Str) -> Str { x + "!" }).expose(d)` printed `hunter2!` embedded and `!`
under `--broker daemon`, exit 0 — and the mapped value, a LOCAL secret, was exposed with no broker gate and no `expose` record.
**The class enumerated** (every `reveal()`): `map` was the only call a handle could reach. **Fixed** (`interp.rs`): refused
DL0904 before the function runs, naming the ways out; `--engine wasm` refuses secrets itself (DL1205, unchanged). **Witness**
`broker_cli`'s `a_broker_held_secret_is_never_mapped_over_bytes_it_does_not_hold` (control embedded `hunter2!`; daemon refused,
nothing printed; the way out — `expose` then transform under the daemon — `hunter2!`); M401 (the refusal disabled) red, the
control green; clippy clean; Windows and macOS linted from Linux. `f4c428c`. **Read on the runners:** `broker_cli` on Windows `38629378991` (5 passed), macOS `38629384172` (6), arm64 `38629387987` (6) — 0 failed, both broker-held-secret witnesses named, no compiler warning, no orphan `delulu`
**The full suite alone at `f4c428c`:** 2,284 passed, 0 failed, 16 ignored (163 binaries), cargo exit 0, the tree unmoved, no daemon left
**Records corrected beside it:** `V2_P9_DESIGN.md` said `map` was "broker-side under the daemon" — the broker's `secret_map`
exists and no program reaches it; RW 4.49's third item (`record_device_refusal` answering `Recorded` for a lost record) was
closed by routine run 18 (D-V2-111) and the row never said so.

**Push runs read.** `91a1eb7`'s CI `38628412194` and `d61dd70`'s CI `38628780402` — success on every job (`miri-slow` and `heavy-gates` skipped on a push); `openshell` `38628412189` success. `actors_pingpong` at `d61dd70`: arm64 MEASURED 3.00x (control 4.11x); macOS NOT MEASURED (3 hardware threads); Ubuntu and Windows NOT MEASURED (never 3x perfectly parallel work).

## 2026-10-14 — routine run 24 (last): `4f3084b` red on supply-chain — wasmtime 48.0.6; claims the slice overtook corrected; P9-04 (d) read against the code

**Push runs.** `f4c428c`'s CI `38630746024` — success on every job (`miri-slow` and `heavy-gates` skipped on a push); `actors_pingpong` arm64 MEASURED 2.97x (control 4.12x), the others NOT MEASURED.

**`4f3084b` red on supply-chain** (CI `38631973104`, a records-only commit; every other job green): eleven advisories RustSec
published that evening against wasmtime 48.0.5 — RUSTSEC-2026-0353 to -0359 and -0361 to -0364. Eight are in the component
model, which this tree does not enable; 0359 (dropping a long chain of recursive types) and 0364 (exception tags) sit behind the
WebAssembly 3.0 proposals D-V2-68 refuses. **Witnessed** in the VM: `cargo deny --all-features check advisories` failed with the
eleven on the old lockfile. **Fixed** (`bf51fe0`): `cargo update -p wasmtime --precise 48.0.6` — the lockfile's diff read: only
the Wasmtime, Cranelift and Pulley release moved — 29 lockfile entries (versions and checksums, none added or removed; run 23's
lesson); `cargo deny` then ok on advisories, bans, licences and sources; `delulu-wasm`'s tests green (91 and the rest); the
two-engine differential (50,000 programs) ok in 116 s; the five other targets linted from Linux; `deny.toml` records why.
**`bf51fe0`'s push run** (CI `38636976623`): success on every job, `supply-chain` included — **`master` green again**; `actors_pingpong` arm64 MEASURED 2.98x (control 4.13x), macOS NOT MEASURED (3 hardware threads), Ubuntu and Windows NOT MEASURED. **The full suite alone at `bf51fe0`:** 2,284 passed, 0 failed, 16 ignored (163 binaries), cargo exit 0, the tree unmoved, no daemon left.

**A sweep of what P9-04 (b)/(c) made false** (`52193ba`): `SecretVal::reveal`'s doc said it was "the only in-process reader —
reached solely through `Secret.expose`" — it is reached from `expose`, `Secret.map` and `http.get_with_secret`, each refusing or
rerouting a broker handle before it (the comment now says a fourth caller must do the same); `prim.rs`'s "the only reader:
expose" — the only one that hands the bytes to the PROGRAM; the book's secrets section — a secret can also be SENT without the
program learning it, and an audience is a statement of trust (a server in it may answer the secret back). Comments and prose;
the runtime library (275 passed) and the book gate green.

**P9-04 step (d), read against the code** (`V2_P9_DESIGN.md`): the broker knows secret NAMES only (`Scopes.secrets`, the wire's
`AuthoritySpec.secrets`, and issue-then-run drops a grant line's audience); an audience is a set of `NAME@host:port` pairs, one
more exact-set dimension (`secret_to`) whose laws `authority_algebra.py` §1 already proves for any such dimension; under the
daemon the program process performs the request, so a send needs a recorded broker op handing the bytes to the egress client
only (`SendSecret`, guarded like `Declassify` by default); the witnesses to write first are listed there.

**Loop engineering:** `docs/CLOUD_ROUTINE.md` step 5 — stop a background job by PID, never `pkill -f PATTERN` (it killed its own
shell this run, exit 144, and the lint's children lived on); a one-test witness log is read with `tail_lines` 40 (this run pulled
three whole ~550-line logs into the context — run 13's mistake again); and the two-walls line with `scripts/mutants.py`'s
multi-edit mutants. `HANDOFF.md` §11.5 the `pkill` trap.

## 2026-10-15 — routine run 25: CI read green; P9-04 step (d)'s first half — the broker's `secret_to` (D-V2-133)

**CI read.** `41ce7c1`'s push run (CI `38640213195`): success on every job (`miri-slow` and `heavy-gates` skipped on a push);
`actors_pingpong` arm64 MEASURED 2.95x (control 4.10x), macOS NOT MEASURED (3 hardware threads, 2.45x observed), Ubuntu and
Windows NOT MEASURED (never 3x perfectly parallel work); each test job's sandbox properties as before (macOS's filesystem
confinement absent — its reads, PS-E's residual). The nightly `38702203302` (07:56 UTC) still running at the run's start.
**Start of run:** `survey check` ok (1,506 nodes, 14,421 edges), `doctor --check` 26 checks passed; z3 5.3.0 and cargo-deny
installed (advisories ok); every `origin/claude/*` branch 0 commits ahead of `master` but two already recorded as superseded.

**P9-04 step (d), its first half (D-V2-133)** — `db88e98`. A secret's audience died at every node boundary: **witnessed** on
`41ce7c1`'s binary — a `run --broker daemon --grant secret:API_TOKEN@127.0.0.1:PORT=…` node showed no audience (`secret_to`
`Null`), and `grants delegate --secret-to` was an unknown argument. Built: `Scopes.secret_to` (`NAME@host:port`, the egress
client's canonical spelling; inclusion; the meet an intersection; a pair lives only beside its name — canonicalization drops an
orphan; serialized only when non-empty, so no chain hash or certificate changes); the wire (`AuthoritySpec`, `NodeInfo`); the
grant line's audience into the node; a lease's audiences back into its run's grants; a proposal's join; a certificate's
authority; `grants delegate --secret NAME --secret-to NAME@HOST[:PORT]` (the egress parser's own reading — an orphan, a path, a
wildcard and an upper-case host refused in words); `grants diff` between nodes reports `secret_reach_expansion`; one conversion
per direction (a literal ending `..Default::default()` and a second copy of `authority_spec_of` would each have dropped the
dimension silently). The Z3 model: the ninth set dimension, a §2c for the pair-beside-its-name rule, a full-conjunction
obligation that notices the dimension missing — **34 obligations**, three more CI mutants (six). Live documents that quoted
"ten dimensions" or "26 obligations" updated (`MATHEMATICS.md`, `models/README.md`, `AUTHORITY_GUARD_CAPSTONE.md`,
`QUESTIONS.md`, the book, `ci.yml`'s step names); historical records left as written.
**Falsified:** M402–M421 all red, the control green (D-V2-133 lists each). **The witness's own mistake, caught by its first green
run:** it assumed `--secret-to K@API.example.com` would be lowercased — the egress parser refuses an upper-case host rather than
normalizing it, so the audience does the same; the witness now asserts the refusal.
**Verified:** clippy clean; Windows and macOS (arm64) linted from Linux (`check-other-os.sh`, exit 0); read on the runners at
`db88e98` — `within_cli`, `broker_cli` and the binary's unit tests on Windows `38720731643` (15, 5 and 181 passed), macOS
`38720736853` (15 for `within_cli`), arm64 `38720741646` (6 and 15), `delulu-broker`'s library on Windows `38720745681` (success)
— 0 failed, every new test named, no orphan `delulu`; **the full suite alone** — 2,287 passed, 0 failed, 16 ignored (163
binaries), cargo exit 0, the tree unmoved, no daemon left.
**Next — step (d)'s second half:** the broker SENDS a secret for a daemon or leased run (`SendSecret`, the Guard on the pair,
a `send` record), then (e).

## 2026-10-15 — routine run 25 (continued): P9-04 step (d)'s second half — the broker sends a secret, for one origin its node names (D-V2-134)

**Push runs read.** `a97a953` (the first half's records): CI `38723018372` — see the next entry; `ocsf` `38723018286` and
`openshell` `38723018349` success. **The nightly at `41ce7c1`** (CI `38702203302`): success on every job — `heavy-gates`,
`miri-slow` (syntax, check, broker), `miri`, `microvm`, `formal`, `fuzz` and the three test OSes.

**The second half (D-V2-134)** — `6fe7076`. **Witnessed red first** on `a97a953`'s source: a `--broker daemon` run and a `--lease`
run of `get_with_secret` both refused DL0904. Built: the broker's `send` (`Broker::send_secret_guarded`, the wire's
`SendSecret` → `SecretBytes`): the node live, the source in `secrets`, `SOURCE@origin` in `secret_to`; the Guard on the PAIR (a
delegated node's send DL1410 until `declassify:NAME@host:port` is approved — an expose permit does not buy it); a guard event,
then a `send` record with the pair and the span, never a byte; a lost record hands over nothing (DL1401). The run
(`Interp::send_broker_secret`): the host's custody check first; the effect's own refusals; then `egress::prejudge_secret` —
headers, the URL's parse, the origin in the handle's audience, judged before the bytes exist (a refusal there asks the broker
nothing, recorded as the egress client's own, `egress::refuse_unsent`); then the bytes for that ONE origin, sent with a secret
whose audience is that origin alone. A handle carries its audience from the mint and across an actor boundary. `STAGE5`'s
criterion 6 restated as it holds (no byte before an `expose` or a recorded `send`).
**The witnesses' own mistake, caught by their first run:** the daemon witness expected the `send` record's span to name the
file — a record's span is the interpreter's `FILE:START:END` (numeric), as an `expose` record's is.
**Falsified:** M422–M438 (D-V2-134 lists each): all red but M428 and M432, implied walls that survive by design (M428b red);
the controls green. **Verified:** clippy clean; Windows and macOS linted from Linux; read on the runners at `6fe7076` — `broker_cli`, `egress_cli`, `secret_verify_cli` and the binary's unit tests on Windows `38724166783`, macOS `38724169549` and arm64 `38724172459` — success; `delulu-broker`'s library on Windows `38724182113` — success; `delulu-runtime`'s library on macOS `38724178976` — success, and on Windows `38724174798` **red** — `compute::tests::queue_depth_refuses_work_past_the_granted_depth`, a race in code this slice does not touch (below), fixed in `df0fe6a` and read green there: Windows `38725247096` (274 passed, 0 failed, 1 ignored), arm64 `38725250746`, macOS `38725253078` (the test alone);
**the full suite alone** — 2,290 passed, 0 failed, 16 ignored (163 binaries) at `df0fe6a`, cargo exit 0, no daemon left; it reported `TREE-MOVED` — a false alarm: its fingerprint compared `git status` and `git diff`, which the commit made during the suite (the routine allows one) changed while no file moved, and `doctor_cli`'s freshness check passed inside it; `scripts/suite.sh` now fingerprints the files themselves.
**Next:** P9-04 (e) — `authority`'s per-secret destinations; the OCSF mapping of a `send` (HTTP Activity with the destination).

**A race witness made certain** (`df0fe6a`): `compute::tests::queue_depth_refuses_work_past_the_granted_depth` was red once on
the Windows runner (the witness run above) — four threads spawned one after another, each running a 4-element `burn` of a
millisecond or two, had run one after another, so nothing was ever in flight beside anything and nothing was refused. Its window
grown, never its patience (run 18's rule): a barrier releases all four at once, and a 64-element `burn` holds the one slot while
the other three arrive. Green 6 of 6 in the VM with all four CPUs busy; M439 (the queue depth unenforced) red, the control green;
read green on Windows, arm64 and macOS. **Loop engineering:** `scripts/suite.sh`'s fingerprint now hashes the files themselves
(a commit during a suite read as `TREE-MOVED`); `docs/CLOUD_ROUTINE.md` step 5 — a slice written in a git worktree while the
suite runs, and the backgrounded-variable trap.

## 2026-10-15 — routine run 25 (continued): the OCSF export names a broker's `send` as HTTP Activity (D-V2-134's follow-up)

**Push runs read.** `9ed939f`: `ocsf` `38727198688` and `openshell` `38727198636` success; its CI `38727198569` read in the
next entry. **`b8da460`:** an allowed `send` exported as a Base Event (class 0, witnessed on the seeded chain); now HTTP
Activity (4002), Get, `dst_endpoint` `{hostname, port}` from `NAME@host:port`, the secret named only in the message; a refused
send stays a Detection Finding. `audit_ocsf_cli`'s seeded chain gains a send to the audience and one outside it. The corpus
validates against OCSF 1.8.0 (`scripts/ocsf-validate.py`: 15 events, 0 problems; its self-test 16 of 16 mutations caught);
M440 and M441 red, the control green; clippy clean; `audit_ocsf_cli` and `monitor_cli` on Windows `38727666655` and macOS
`38727669814` — success; `docs/DEPLOYMENT.md`'s mapping paragraph. **The full suite alone at `b8da460`:** 2,290 passed, 0
failed, 16 ignored (163 binaries), cargo exit 0, the tree unmoved.

## 2026-10-15 — routine run 25 (continued): P9-04 step (e) — `authority --json` names where a program sends each secret (D-V2-135); P9-04 complete

**Push runs read.** `9ed939f`'s CI `38727198569` — success on every job. `3f03ab0`: see the next entry.
**Step (e)** — `b6128d1`: `secret_destinations` on `authority`'s report (D-V2-135) — per `get_with_secret` call, the secret's
grant name and the URL literal's `host:port`, `null` where the source does not show them. Witnessed absent first on
`3f03ab0`'s source; the witness covers five call shapes (a parameter's secret; a `let`-bound mint through `map`; a computed URL;
a `for` rebinding the name; a block's `let` that ends with it and an assignment that rebinds) and the closed schema; M442–M448
red, M449 survived as an implied reset and was removed. clippy clean; `within_cli` 16 passed. **Verified on the runners:** `within_cli` and `json_contract` on Windows `38729783946` and macOS `38729787305` — success.
**The full suite alone at `b6128d1`:** 2,291 passed, 0 failed, 16 ignored (163 binaries), cargo exit 0, the tree unmoved, no daemon left.
**P9-04 is complete** — steps (a) to (e): a request with headers, a secret in a request only to its audience, the audience in
the grant tree, the broker's send, and the static half. **Next: P9-05** (method-and-path network scopes).

## 2026-10-15 — routine run 25 (continued): P9-05 read against the code; NET-WILDCARD-1 found and closed (D-V2-136)

**Push runs read.** `3f03ab0`'s CI `38729124643` and `ocsf` `38729124578` — success; `0f73f09`'s `openshell` `38731264794` —
success (its CI read in the run's last entry).
**P9-05, read against the code** (`V2_P9_DESIGN.md`): every request is a GET (methods wait for the first non-GET primitive); a
host is read in three places; the egress client already parses each hop's path; the broker sees hosts only; the grammar and the
audience have no room for a path; the static half is P9-04 (e)'s walk generalized. Build order (a)–(d) proposed there.
**NET-WILDCARD-1** (D-V2-136), found while reading: a pattern `net` grant was honored embedded and refused DL0904 under the
daemon — the broker compared the host by exact string. Witnessed both ways in the VM, then by `broker_cli`'s new test (red on
the old source); one matcher now (`delulu_broker::validate::host_matches`, `c69506f`); M450–M452 red. **A slip caught before
`master`:** `c69506f` put the matcher after `validate.rs`'s test module, which clippy refuses (`items_after_test_module`) — the
VM's clippy had been read through `| head`'s exit code (AGENTS.md's own trap), so the push to the harness branch carried it;
moved in `016b522`, clippy re-read by cargo's own exit code (0). Read on the runners at `016b522`: `broker_cli` and
`egress_cli` on Windows `38732093503` and macOS `38732097100`, `delulu-broker`'s library on Windows `38732100229` — success.
**The full suite alone**, at `fa15654` (with F1's fix, below): 2,294 passed, 0 failed, 16 ignored (163 binaries), cargo exit 0, the tree unmoved, no daemon left; an earlier one at `016b522`, run while the red-team pass loaded the VM, was red once on `satellite_demo`'s re-contact scenario and green 3 of 3 alone at once — load, per run 17's rule. F1's fix read on the runners: `broker_cli`, `guard_cli`, `guard_e2e`, `secret_verify_cli` on Windows `38734744376` and macOS `38734749233`, `delulu-broker`'s library on arm64 `38734752723` — success.

## 2026-10-15 — routine run 25 (continued): the red-team pass on the broker's send — the guarantee held; F1 fixed (D-V2-137)

**The pass.** One Sonnet 5.5 sous-chef (`claude-sonnet-5-5`), against a frozen copy of `0f73f09`'s binary in the scratchpad,
briefed to break "a secret's bytes reach only an origin its node's `secret_to` names, reach no program value and appear in no
record", ~21 minutes, 96 tool calls; it built a local test CA and a TLS server to see the bytes actually delivered.
**The guarantee held.** About 40 origin spellings (ports with leading zeros, trailing dots, `127.1`, decimal and hex addresses,
`[::ffff:127.0.0.1]`, userinfo, `#@`, upper-case scheme) — only the exact origin connected; ~14 `--secret-to` spellings
refused at delegate time; raw-wire origins refused by the exact-string compare; a redirect to another origin dropped the secret
even when the grant named both; `Secret.map` of a handle refused; an actor's send held to the audience; a revoked node's token
DL1403 (and over the wire); six parallel runs against a one-use permit — exactly one send; the canary in no audit record, OCSF
export, report, trace, `--json`, stdout, stderr, `grants list`, `broker.log` or `guard.json`; 15 invalid header and URL forms
refused before the broker (0 `send` records, 0 connections). Each finding re-read before use:
- **F1 (Medium) — fixed, D-V2-137**: a seal on a secret's name did not cover its sends (above).
- **F2 (Low) — open, RW 4.65**: a send's DL1410 names the matched rule (`declassify:*`) as the thing to request, never the
  pair the permit needs.
- **F3 (Low, within the same-uid residual) — open, RW 4.66**: a raw `SendSecret` naming a node id returns the bytes to any
  same-uid caller of the socket, as `Expose` does for a node with `Declassify` — but a node with only `secret_to` (no
  `Declassify`) now yields bytes too; the broker cannot tell the run's egress client from another process (RW 4.27 H2).
- **F4 (Low) — open, RW 4.67**: after a send, one copy of the plaintext stays in the program process's heap until exit (the
  serialized request; `reveal()` and the header clones are not zeroized). Before the send: 0 copies, scanned every 50 ms.
- *Informational:* a same-origin redirect loop re-sends the secret on each hop (bounded by the redirect limit) under one `send`
  record; a secret whose VALUE breaks a header rule is refused after the broker's `send allow` (the request never left; the
  program is told `Refused`).
**The pass's own slip:** it stopped both test brokers once with `pkill -x delulu` (run 24's trap, in another form); restarted.

## 2026-10-15 — routine run 25 (postscript): the last push runs read

`0f73f09`'s CI `38731264861` — success on every job; `actors_pingpong` arm64 MEASURED 2.99x (control 4.11x). `22501f6`'s
CI `38736592686` — success on every job (16; `miri-slow` and `heavy-gates` skipped on a push), arm64 `actors_pingpong` MEASURED 2.99x (control 4.12x), and `ocsf` `38736592620` success: **`master` green at the end of the run**.

## 2026-10-17 — the owner is back: the laptop synced to `b82a145`; the routine's merged branches deleted

The cloud period (2026-09-28 → 2026-10-16) is over. The laptop fast-forwarded from `cfbfbdc` (`Cloud handoff (10)`) to
`b82a145` — 281 commits, the 25 routine runs — with no line-ending drift; the Survey matches the tree and `doctor --check`
passes on Windows (29 checks) and in WSL (26 checks, KVM available). `master` is green on GitHub: `b82a145`'s push run
`38739567829`, and the nightly `38869800819`, whose `miri-slow` and `heavy-gates` passed. At the owner's word, the 23
`claude/*` branches whose tips `master` already holds were deleted from GitHub; the two holding a superseded commit cited
by hash (`7d6a22b`, `d3f0d52`) were kept. The routine is disabled; the owner had edited its prompt (quoted in
`docs/CLOUD_SYNC_LOG.md`, whose 2026-10-17 entry has the details and the branch list). Not redone on the laptop, for the
owner's word: the full suite on Windows and in WSL, the WSL microVM guest images, the editor's end-to-end test.

## 2026-10-17 — `e1bc4df`'s push run red on macOS: a hex canary spelled by an audit hash; the canary made unspellable

**What.** `e1bc4df` — a records-only commit, the disk-cleanup record — went red on one job: macOS's `broker_cli`
`a_leased_send_is_guarded_on_its_destination_and_a_narrower_child_cannot_reach_its_parents_other_origin`, *"the chain holds
the bytes"* (CI `39025580650`, job `117375792215`; every other job green). The commit changes no code, so the test was read.
P9-04 (d)'s two secret tests store the secret `hunter2-8f2c` and search every audit file for `8f2c` — four hex digits, in a
chain whose every record carries a 64-hex-digit `hash` and `prev_hash` and a hex `actor_node`. A record's hash spells
`8f2c` by chance — a chain holds some 800 (the first test) to 1,500 (the leased one) distinct four-digit hex strings,
counted on chains left on disk, of 65,536 possible.

**Witnessed** on the laptop (Windows), on the old code: 1 failure in 3 runs, then **8 in 100** (6 of the leased-send test,
2 of `a_broker_held_secret_is_sent_only_to_its_nodes_audience_and_the_send_is_recorded`). A failed run leaves its state
directory, so each chain was read: in all 9, `8f2c` stood only in one record's `hash` and the next one's `prev_hash`, and
`hunter2` appeared 0 times. Nothing leaked; the test raised a false alarm.

**Fix.** The secret is `hunter2-wq7z` and the canary `wq7z` (`broker_cli.rs`'s `SECRET`/`CANARY`): `w`, `q` and `z` are not
hex digits, and the chain's other fields are decimal (`seq`, `ts`, ports) or words, so no field can spell it. The same change
in `egress_cli.rs` (`SECRET_BYTES`/`SECRET_CANARY` — a sandboxed run's report carries two 64-hex-digit digests, so the same
false alarm, rarer) and in `brokerd`'s `a_send_whose_record_was_lost_hands_over_no_bytes` (its message is a fixed string
today; the daemon's other refusals embed the hex node id). Left as they are, each read: `delulu-runtime`'s egress unit tests
search for the whole `tok_8f2c-never-in-a-record`, or a reason built from fixed inputs; `delulu-broker`'s secrets tests
compare for equality only.

**Evidence.** After: **0 failures in 100 runs**. Falsified with two mutants (applied by a script, restored, rebuilt): an
extra chain record holding the bytes' masked tail `****wq7z` after every allowed send turned both broker tests red at the
canary check (`broker_cli.rs:310`, `:404`); printing the secret header's value to stderr where the client holds it turned
`egress_cli`'s red at `:318` (`Bearer s3cr3t-wq7z-never-printed` in its message). `cargo clippy --workspace --all-targets -- -D warnings`: exit 0. The suite (`cargo test --workspace --no-fail-fast -j 4`,
Windows, 163 binaries): 2,225 passed, 4 failed, 15 ignored — the 4 are the map's freshness (`doctor_cli`'s three and
`delulu-survey`'s `freshness`; `doctor`: "survey freshness: 2 behind the tree"), the map being rebuilt after the last
edit; those targets after it: in the commit message.

**Observed, not chased (RW 7.19).** On Windows most runs of these two tests — passing or failing — leave their temp
directory behind (the programs and `state/`, a few KB): the final `let _ = std::fs::remove_dir_all(&base)` fails without a
word. The daemon runs with its working directory in the state directory (`brokerd.rs:1920`) and may still be exiting when
the test removes it — not confirmed. CI's runners are discarded.

**Also read:** `b4fb0a9`'s push run `39026595221` — success on every job (`miri-slow` and `heavy-gates` skipped on a push).

## 2026-10-17 — the owner's commission: take DeluluLang public; every outstanding decision delegated

**What.** The owner put `docs/design/DeluluLang_Autonomous_Development_Prompt.md` (his, 148 lines, committed here as he
wrote it) in front of the head chef: *"my 32 michelin star head chef claude, let's cook this for entire world to use.
update memory and other files"*. It delegates every outstanding decision — the owner-reserved ones included — and the
public release once objective criteria have passed; it keeps its own limits (never invent contacts, keys, results or
compliance claims; preserve this development repository and a backup; no microVM image until its GPL obligations are
met). The terms: `HANDOFF.md`, *The owner's commission of 2026-10-17*. Its order: audit → P9-05 → the decisions → WSL
and VS Code validation → the scheduled task → the release → a final report.

**A correction first.** Asked earlier the same day what needed his decision, the head chef listed "the licence — still
undecided". That was wrong: the owner chose Apache-2.0 on 2026-07-24 (ruling D27 — `LICENSE`, `NOTICE`,
`TRADEMARK.md`, `GOVERNANCE.md`, `license = "Apache-2.0"` on every crate); the rule "owner-reserved" covered a *change*,
and the answer read it as "undecided". The prompt's licence section was written from that answer. What is open is a
binary distribution's compliance (third-party notices) and the GPL microVM image.

**Recorded** in this commit: the prompt, with its row in `REPOSITORY_STRUCTURE.md` §5.3; `HANDOFF.md` (a section at the
top, which §1's two rows and §11.1 now point to); `AGENTS.md` (the hard rules and the delegation); `CLAUDE.md`;
`V2_PHASE_STATUS.md` (row 17, the release program); `docs/assistant-memory/` (two new files — the commission, and the
owner's pronouns, he/his, which he stated the same day — and three updated); `docs/CLOUD_SYNC_LOG.md`. The laptop's
memory has the same.

**Commit identity.** From this commit the repository commits as `227307678+jessesuniljs1-collab@users.noreply.github.com`
(a repository-local `git config user.email`; account id 227307678 read with `gh api user`) — the prompt's *"Use an
appropriate verified GitHub noreply address for future public commits"*. Of the history before it, 459 commits name
the owner's personal address, 189 the no-reply one and 105 Anthropic's; that history is preserved, as the prompt
requires of the development repository.

**Also read:** `2f1992d`'s push run `39028791103` — success on every job, `test (macos-latest)` among them (`miri-slow`
and `heavy-gates` skipped on a push).

**Next:** the prompt's §1 — the audit of the repository's state — then P9-05 step (a).

## 2026-10-17 — P9-05 step (a): network grants scoped to paths (D-V2-138)

**What.** The first engineering of the owner's commission: `net=HOST/PATTERN` (`*` one segment, `**` the rest, last
only), read by one module, `delulu_broker::netscope`, at every place a `net` entry is read — the grant parser, the
runtime's first check (DL0904 naming the host or the path), the broker-secret pre-check, the egress client's every hop
(redirects included) and the broker's `Net` use check (by its host part, so `--broker daemon` and embedded custody agree —
NET-WILDCARD-1's question asked of a path). Where a pattern decides, a path a server could read two ways is refused, never
matched. The special-use rule reads the host part (`net=127.0.0.1/x/**` refused, with the `net.special=` repair). The
OpenShell export refuses a path scope rather than widen it to the host. User docs: `CHANGELOG.md`, the Agent Skill's grant
block, `docs/for-agents.md`; `delulu toolchain`'s grant forms (`GRANT_FORMS`) name `net=HOST[/PATH]`.

**Every reader found first** (`grep` over `scopes.net`, `grants.net`, `root.net`, `host_matches`): the runtime's mint
(exact string — unchanged, so the program names the scope it gets), `host_allowed` (now `url_verdict`/`url_allowed`), the
egress hop, `interp::send_broker_secret`'s pre-check (found by the grep, not by the design — without it a path outside the
scope would have had the broker release the bytes and record a `send` before the client refused the request), the broker's
use check, `accept_manifest` (copies a manifest's entries unparsed, so the matcher fails closed: an entry whose pattern
does not parse matches nothing), the dependency pins (exact sets — sound with paths), `within.rs` (string sets — step (c)),
the OpenShell export.

**Evidence.** Tests written first and witnessed red: the egress hop test (`a_path_scope_is_judged_on_every_hop` — the
in-scope request was refused by the old matcher), the broker's host-part test (red against the exact comparison). New
tests: `netscope` (4 — the grammar, matching, ambiguity, the verdicts), the grant parser
(`a_net_grant_may_name_a_path_pattern`), the hop test, `broker_cli`'s daemon parity
(`a_path_scoped_net_grant_is_honored_under_the_daemon_as_embedded`) and its broker-secret case (no `send` for a path
outside the scope), `egress_cli`'s loopback end to end at L0 and from a guest
(`a_path_scoped_grant_reaches_its_paths_and_no_other` — TLS for the path, nothing connects for another), the export's
refusal row. **Falsified:** M1–M5, M7 and M8 red, M6 witnessed red (D-V2-138); the stale-binary scan then found the M2 build
of the broker's unit-test binary still on disk (another package set names it differently) — rebuilt, scanned clean
(`HANDOFF.md` §11.5). Gates: in the commit message.

**Next:** P9-05 step (b) — the broker's op and record carry the path; `⊑` by containment, in the Z3 model with mutants.

## 2026-10-17 — P9-05 step (b): the path order, by containment, proved (D-V2-139)

**What.** `net` entries ordered by containment — the same host string, then the path pattern — in the broker's `⊑` and
meet, the runtime's mint (a program may ask for less than it was granted) and `grants diff` (a narrower path is no
addition); the meet every entry's meet; the canonical form an antichain (P17-F3 for this dimension); every `net` parser
calling `netscope::check_entry` (the two `--net` flags stored anything before). The Z3 model gains §2d. User docs:
`CHANGELOG.md`.

**Evidence.** The agreement test (`within.rs`, the `--within` verdict against the lattice) extended with path-scoped
entries went red with only the broker switched ("verdict false, lattice true" for `api.example.com/v1/*/x` within
`api.example.com/v1/**`), green with the mint switched too. New: the exhaustive exactness test (every pattern of up to three
segments against every path of up to five: syntactic order = semantic inclusion, the meet = the intersection), the
canonical-form test over every subset of seven entries, three attenuation-table rows, `a_net_set_is_canonical_like_a_path_set`,
`grants diff`'s path and special-use cases, `broker_cli`'s delegation test (narrow allowed, wide refused DL0802 with the
narrower repair, a malformed `--net` refused, a leased run held to the narrow path). Z3: 41 obligations discharged; the
three new mutants each leave 2–3 undischarged. Rust mutants N1–N7 (eight) all red, no stale binary. Gates: in the commit
message.

**A wrong reason, caught before it was written.** Drafting the decision, the head chef first gave the monitor's
`special-use` rule as the reason the chain cannot carry a path in `target`; reading `monitor_cmd.rs` showed the rule reads
the record's `reason` field. The real consumer is the OCSF export (`dst_endpoint.hostname`). Recorded so: D-V2-139 item 6.

**Also read:** step (a)'s `94b6c36` push run `39034873839` — success on every job.

**Next:** P9-05 (c)'s static half, then the commission's decisions.

## 2026-10-17 — P9 closes as far as GET reaches (D-V2-140); the Constitution's containment claim narrowed to what is built (D-V2-141)

**P9.** With P9-05's path dimension enforced, ordered and proved, P9 is complete as far as the request primitives that
exist reach. Recorded open, each with its reason: `authority`'s static path report (an AST walk cannot tell an `Http`
`get` from a `Map`'s without the checker's types — no guessed claim), the method dimension (waits for the first non-GET
primitive, now ranked first after the public release), the path in the chain's record (OCSF's hostname, D-V2-139).

**The Constitution (entrenched).** §5.15 guarantee 5 promised containment "at the WASM/microVM layer" for every program;
the microVM is built on Linux x86-64 with KVM only, and an ordinary run has no OS boundary. Now it names the layer per
way of running, and §5.14's third layer says where it is built. Recorded in `ENTRENCHED_CHANGE_RECORD.md` with why it is
not an RFC (it strengthens §5.14's honesty limits; the 2026-08-08 precedent). RW 7.10a closed. Gates: in the commit
message.

## 2026-10-17 — the licence's compliance and the GPL image (D-V2-143); the release's build reusable, Scorecard (D-V2-144, first part)

**What.** The licence stays Apache-2.0 (D27, the owner's; weighed against MIT and the dual licence in D-V2-143). Every
archive now carries `THIRD-PARTY-NOTICES.txt` and `SBOM.cdx.json`, generated by `scripts/third-party-notices.py` from the
portable build's linked graph for its target; packaging refuses a missing legal file or a crate whose licence it cannot
show. The microVM image is never distributed (the user builds it from pinned sources); D-NE-27 and PS-C-05 resolved. The
release's build moves into a reusable workflow (`release-build.yml`) for SLSA Build Level 3 — claimed only once the
first release verifies with `--signer-workflow`; `scorecard.yml` measures the public repository.

**Found on the way.** The owner's account already holds an **empty public repository `DeluluLang`** (created 2026-07-03,
default branch `main`) — the destination the records called "a different one, a step the owner takes personally";
recorded in `HANDOFF.md` §1.1. The SBOM SECURITY.md §6 called live per release was `docs/release/SBOM-1.0.json`, 1.0's,
direct dependencies only — the per-archive SBOM makes the claim true from this release on.

**Evidence.** The generator refused first (`ittapi`, `ittapi-sys`: no licence file at the top) — the gate working — then
produced the notices for four targets (Windows 257 linked packages / 150 texts; Linux gnu and musl 257 / 156; macOS arm64 253 /
150), no GPL text, the SBOM with all 248 components checksummed. `package-toolchain.sh` end to end on Windows: exit 0, 35
files, both new files in the archive. `distribution.rs` 10 passed; its new assertion witnessed red against the old
script. The three workflows parse. The release dry run on the runners: read in the next commit. Gates: in the commit
message.

## 2026-10-17 — rustfmt adopted (D-V2-145)

**What.** `rustfmt.toml` (`max_width = 130`, `use_small_heuristics = "Max"`), `cargo fmt --all` over the workspace (283
files), `cargo fmt --all -- --check` in CI's `lints` job, `CONTRIBUTING.md` §6, RW 7.9 closed. Measured before choosing:
the default style disagreed with the tree 7,065 times; the chosen two options, 2,371.

**Checked.** Over the whole diff, the comment lines removed and added carry the same text — rustfmt does not wrap
comments — except three that differed only by a UTF-8 byte-order mark rustfmt removed. Two of those showed
double-encoded characters on BOTH sides (`â€”` for `—`, `Â§` for `§` — `actors_promise.rs`, `actors_trace.rs`, from long
ago): corrected here, 14 characters. The core-invariance snapshot pins what the compiler says about the shipped
programs, not the Rust text, so it cannot move with formatting — and the suite says it did not.

**What the reformat found.** The first suite after it: 2,241 passed, 2 failed — both gates that read Rust layout.
`json_contract`'s sweep searched for `vec!["device"`, which rustfmt had broken across lines (now matched with whitespace
removed). `cli_contract`'s prefix gate read `eprintln!("…` on one line only, so every message long enough to wrap had
always escaped it; rustfmt joined one, and the gate — taught to read the next line — found **three error lines with no
`error:` prefix** (`unknown command` twice, the `refused` closing line of `check`), now prefixed. A formatter run is a
cheap way to learn which gates read layout. Gates: in the commit message.

**Also read.** `bfb5a84`'s push run `39036694401` and `6e0637e`'s `39037513753` — success on every job. The release dry
run on `6e0637e` (`39037514962`, by hand) — success on all four targets through the new reusable `release-build.yml`;
each runner generated the notices and the SBOM (Windows 257 linked packages / 248 components, Linux x86-64 257 / 248,
macOS arm64 253 / 244, Linux arm64 245 components) and passed the install gate. D-V2-143's machinery works on every
release target.

## 2026-10-17 — the on-demand cloud routine (D-V2-146)

**What.** At the owner's request a new routine, `trig_01HFG6usfzmDAPQigiDu9DLq`, created **paused** for him to switch on
when he cannot use the laptop: the first routine's settings, push and e-mail notifications, `5 */5 * * *` UTC (cron has no
5 h 5 min interval). `docs/CLOUD_ROUTINE.md` gained its *Since 2026-10-17* paragraphs (no end date; the release program as
the work order; what a VM run leaves for the laptop); `AGENTS.md` *Read first* step 0 and `HANDOFF.md` §11.1 now start
every session by fetching `origin`. The official routines documentation was read the same day (scheduling, pausing, Run
now's `routine-fire-payload`, usage).

**Checked.** The routine read back from the API: `enabled: false`, `notifications.channel` push and e-mail `true`, model
`claude-opus-5-5`, the cron above. CI of `43d8a7c` (rustfmt) read: CI `39039217277`, ocsf and openshell — success.

## 2026-10-17 — RW 4.19: a driver says READY before the program runs (D-V2-147)

**What.** `adapter.rs`: a driver's (and a transport's) first line must be `READY`, awaited before `main` for
`--adapter-startup-ms` (default 10 s, 1..=120 s; refused on a run without a driver, and on a value outside that or not a
number); silence, an early exit or another first line is `AdapterError::NotReady` and the run is refused before one
command is sent. `delulu device sim` says `READY` (`sim::serve`). Every test driver prints it (`hw_adapter_cli.rs`,
`hw_dpx_cli.rs`, `sandbox_devices_cli.rs`). `EXCHANGE_TIMEOUT` is unchanged. DEPLOYMENT.md, CHANGELOG (breaking for driver
authors), RW 4.19 closed. While there: `line_reader`'s doc comment had drifted onto `await_ready`; moved back.

**Witnessed and falsified.** The unit tests `a_slow_starting_driver_is_waited_for_and_its_first_command_answered` and
`a_driver_that_does_not_say_ready_is_refused_before_any_command`; the CLI test
`a_driver_that_never_says_ready_refuses_the_run_before_main_and_the_wait_is_the_operators`. Five mutants, each applied
alone, built, run and restored from a copy: M1 any first line accepted — killed by the unit test (the CLI test's silent
driver does not reach that branch; recorded so nobody reads the CLI test as covering it); M1b the wait skipped, M2 the
flag ignored (the refusal said 10000 ms), M3 the flag dropped from the hardware-only list — killed by the CLI test; M4
the sim device silent — killed by `sim_device_cli` (both runs refused "did not say READY"). Suite: 2,242 passed and 4 failed before the Survey was regenerated — all four the stale-map gates (doctor_cli 3, the Survey's freshness 1), which pass after it (rerun after the last edit).

## 2026-10-17 — the code of conduct; four decisions recorded (D-V2-148 to D-V2-151)

**What.** `CODE_OF_CONDUCT.md`: the Contributor Covenant 2.1 verbatim from its official source, its contact placeholder
filled by the repository's private reporting form (no e-mail invented), linked from `CONTRIBUTING.md` (D-V2-151).
Recorded: RW 4.5 strict anchored roots stay opt-in in 2.0, the flip planned for a later major with
`ROOT_ISSUANCE_TRUST_BOUNDARY.md` §10's migration (D-V2-148); the release is 2.0.0, the bump made in the commit the tag
names (D-V2-149); P8-04 option (c) goes to an RFC after the release (D-V2-150). `V2_DECISION_LOG.md`'s list of decisions
carried from V1 now says where each stands.

## 2026-10-17 — the machine-level validation: WSL microVM images, the WSL suite, VS Code on Windows

**What ran** (the commission's step 4), on the owner's laptop against `f731e5b` (the code of `f2a19db`; later commits are
records only):
- **VS Code end-to-end on Windows 11**: `node editors/vscode/e2e.js target/debug/delulu.exe`, a freshly packaged
  `delulu-lang.vsix` — passed: the extension activated and the language server started with no errors reported (exit 0).
- **The microVM image rebuilt from source in WSL2** (Ubuntu 20.04, kernel 6.6.87.2, KVM): a fresh ext4 clone of
  `delululang-test` at `f731e5b`, `scripts/microvm/build-image.sh` with the pinned kernel tarball and Firecracker v1.17.0
  from the cache (D-V2-39) — exit 0: `vmlinux` sha256 `f19758e4…6fad`, `initramfs.cpio` `3272f0c6…3121`, `manifest.json`
  `5d40ebd2…a335`. The image stays on the laptop: never committed, never distributed (D-NE-27, D-V2-144).
- **The KVM-gated tests** against that image (`RUSTFLAGS="--cfg delulu_kvm"`): `microvm_cli` 10/10 as the user, and its
  root-only test `as_root_the_vmm_is_jailed_as_its_own_uid_and_dies_with_its_host` 1/1 run as root from the same binary
  (as the user it fails only at `sudo -n` — WSL's sudo asks for a password; recorded so nobody reads that as a defect);
  `microvm_criterion8` 1/1.
- **The whole suite on Linux (ext4, WSL)**: `cargo test --workspace --no-fail-fast -j 4` — **2,311 passed, 0 failed**,
  16 ignored.
- **CI**: `f2a19db` (RW 4.19) run `39051817373` — success on every job (Linux, Windows, macOS, arm64, the microVM jobs,
  Miri, fuzz, lints, formal, supply chain, editor); `d68ff67` `39050623478` and `0fc64e1` `39052018825` — success.

## 2026-10-17 — the autosave: a session's unpushed work survives its usage limit

**What.** `scripts/autosave.ps1`, registered as the Windows Task Scheduler job `DeluluLang-autosave` (every 10 minutes,
7 hours — the owner: *"Maybe next 7 hrs is enough"*; `-Hours N` sets another span — as the owner's user, no console window): it pushes every local branch that is ahead of its upstream — a plain
push, never a force — and snapshots uncommitted work (tracked edits and new unignored files) onto `autosave/laptop`
through a private index, so HEAD, the index and the working tree are never touched. A snapshot's parents are the
previous snapshot and HEAD, so the branch only fast-forwards; it is never merged — the next session recovers from it
(`AGENTS.md` *Read first* 0, `docs/CLOUD_ROUTINE.md` step 1, `HANDOFF.md` §11.1).

**Why.** The owner, 2026-10-17: *"continue cooking till usage limit hits. Can u run something parellely so that
everything will get pushed to github repo even if usage limit hits"*. Anything inside Claude — a sous-chef, the routine —
spends the same usage and stops with it; a job outside Claude does not.

**Evidence.** By hand: snapshot `5fb039a` pushed (base `9829662`, one path), working tree unchanged; the second run
`unchanged since snapshot 5fb039a`; the scheduled task's first run: result 0, the same log line. The banned-word guard
falsified: with a probe token in place of the word and a probe file in the tree it refused (exit 1) and pushed nothing.

## 2026-10-17 — the 2.0 release gate written; the support matrix made true for 2.0 (D-V2-155) — on `launch/2.0.0`

**What.** `docs/release/CHECKLIST-2.0.md`: thirteen criteria for the first published release — suites on every OS,
conformance, the hardening gates, the supply chain, one version, licence obligations in the tree and in every archive,
security claims no further than their proof, the artefacts themselves, the editor, the microVM layer, a fresh and clean
public history, the development history preserved, documents true at 2.0 — each MET or PENDING with its evidence, the
ten-step release procedure, and a verdict to be written before the draft is published. `SUPPORT_MATRIX.md` rewritten for
2.0 (D-V2-155): 2.0.x is the first published release and an LTS; 1.0.x was tagged and never distributed, so it is
unsupported; no wire or DIR major changed (`WIRE_VERSION` `"broker/1"`, `DIR_VERSION` 1, as at `v1.0.0`). The README's
status line says the same.

**Why.** The commission: *"Do not claim release readiness merely because CI is green. Confirm that the actual release
artefacts, metadata, licence obligations and security claims are correct."* Reading the release documents for that
found the matrix promising 1.0.x security backports to 2028-07 — for a version no one could have.

**Evidence, 2026-10-17 on the launch tree:** `delulu-conform --coverage` 355/355; `--check-reference` in sync (24
chapters); `delulu fmt --check examples docs/book/samples` 0 would change; a bundle of every ref,
`D:\nelan\DeluluLang-history-2026-10-17.bundle` (14,095,723 bytes, sha256 `a62498fc…f353e7`, `git bundle verify` okay);
`repository_structure`, `evidence_claims`, `release`, `distribution`, `governance` passed.

## 2026-10-17 — the autosave refuses to carry history `origin` does not have

After the launch `origin` is the public repository with a fresh history (D-V2-153); a snapshot's parents are HEAD and
the previous snapshot, so a laptop still on the development history would have pushed all of it there. `scripts/
autosave.ps1` now fetches with `--prune` and snapshots only when HEAD is on some branch of `origin`. **Falsified** on
throwaway repositories: HEAD on an unrelated history — the guarded script refused (exit 1, no remote branch), the previous
script pushed; control, HEAD on a branch origin has — a snapshot pushed.

## 2026-10-17 — the release's metadata and "nothing is released" statements made true for 2.0 — on `launch/2.0.0`

Checking the metadata (the commission: *"Confirm that the actual release artefacts, metadata … are correct"*) found the
workspace `Cargo.toml` and the VS Code extension's `package.json` naming `github.com/delulu-lang/delulu` as their
repository — an address that is not this project's. Both now name `https://github.com/jessesuniljs1-collab/DeluluLang`.
Every statement that nothing is released — `README.md` (status table, *Installing*, the V2 row), `docs/QUESTIONS.md`
item 13, the Book's maintainer list, `REMAINING_WORK.md` 7.5 (closed at 2.0.0), the extension's install text — now
describes the 2.0.0 archives on the Releases page, their checksums and attestations, and the absence of a package-manager
entry. Records of the past (`CHECKPOINT-1.0.md`, the stage build orders) are left as written. `node --check` on the
extension; `evidence_claims`, `editor_contract`, `repository_structure`, `distribution`, `release` passed.

## 2026-10-17 (night) — the owner: only the test repository until he says otherwise

*"Only push to and use test repo until I tell u to change. Do not change repo, unless told by me. Always push only to my
repo."* Recorded in `AGENTS.md`'s push rule, `HANDOFF.md` §11.1 and the decision log (it outranks D-V2-153 §3 and the
release procedure's steps 4–10). Nothing had been pushed to `DeluluLang`; the prepared release stays on `launch/2.0.0`.

## 2026-10-17 (night) — RW 4.65: a Guard refusal asks for the narrowest permit

**What.** `Denial::GuardBlocked` carries `wants`, the narrowest permit that covers the refused use
(`guard::narrowest_permit`): the use's own item on its axis — `declassify:NAME@host:port` for a send, the file for a
write — or `effect:<E>` for an op with no axis. DL1410's command asks for it and names the matched rule beside it; it
asked for the rule, so one send's refusal told the agent to request `declassify:*`.

**Found while making it.** The item is the program's — a file name, a destination — and the hint is a command agents
paste into a shell: a file named `x$(…)` would have put a command substitution into it, run with the agent's authority,
not the program's. So an item holding a character a shell splits, expands or runs is never put in the command (the rule
is named instead, as before); one with a space is double-quoted; a path is written with `/`, the same path to the
Guard's `path_norm` and one word in every shell.

**Evidence.** `a_dl1410_hint_names_the_narrowest_permit_for_the_use` red on the old code (the hint read
`--use declassify:*`), green after; the exact item, requested and approved, honors that send and refuses
`foo@evil.example:443` (DL1410). `a_dl1410_hint_never_carries_an_item_a_shell_would_run`: nine hostile names fall back to
the rule; falsified by two mutants (every character admitted → red; never quoted → red). `guard_e2e` now runs the
command exactly as DL1410 prints it, and the approved one-file permit lets the write happen (Windows). clippy 0; the
suite on Windows 2,244 passed, the 4 map-freshness tests red until the Survey was regenerated, then green.

## 2026-10-18 — RW 6.1: the CLI's own lines can be translated (begun on the laptop 2026-10-17; recovered from its autosave and finished by the on-demand routine's first run)

**What.** `CLI_STRINGS` was empty, so no CLI prose was localizable and every `cli.*` key a guide documented got DL1704.
Three strings are registered as constants their call sites name (`CliString`, so a missing en-US text is a compile
error): `cli.authority.header` (`{program}`), `cli.authority.foreign-separator`, `cli.locale.add.confirm` (`{locale}`).
`render_cli` gives the active catalog's text, or en-US whole — never half a message — and passes either through
`terminal_line` (one line, every control character shown escaped), since a catalog is text anyone may write and an
argument may be a name a program chose — the laptop's draft used `terminal_safe`, see below. The slang
catalog speaks all three. Not registered, on purpose: `cli.grant-prompt.*` and `cli.run.ok` (the CLI prints no such
line) and the picker (shown before any locale exists). The guide, `docs/lang/README.md`, `en-US.md` and
`delulu-slang.md` say so; the nine starter packs' `cli.*` blocks predate the key space.

**Evidence.** `a_named_cli_string_speaks_the_locale_and_the_json_does_not_move` (`locale_cli`) red on the old code — the
slang header was en-US — green after: only the header line moves, `authority --json` is byte-identical across locales.
`a_third_catalog_speaks_the_cli_strings_and_locale_add_asks_in_the_active_voice` (`locale_plugin_cli`) red on the old
code — the pirate catalog's `[cli.authority.header]` was DL1704 — green after; `locale add`'s question asked in en-US and
in slang, `n` installing nothing. Three `catalog.rs` unit tests (placeholders declared = used; the slang catalog covers
every string; fallback whole and terminal-safe — falsified: `render_cli` without `terminal_safe` → red).

**Recovered and finished (2026-10-18, the on-demand routine's first run, `trig_01HFG6usfzmDAPQigiDu9DLq`).** The laptop
session stopped before it committed. The autosave's newest snapshot, `b4a9e18` (2026-10-17 21:15 UTC; base `640a7d5`,
`master`'s last commit; 14 paths), held the slice with its records written; the four older snapshots are byte-identical
to commits that followed them (`424c391`, `3f652cf`) or to `master`. Applied to `master` and verified as a stranger's
work: both end-to-end witnesses re-run red on the old product code (`cli.rs`, `catalog.rs`, `lib.rs` and the slang
catalog put back to `640a7d5`, the new tests kept) — the slang header was en-US; the pirate catalog's key was DL1704 at
`locale add` — and the mutant re-run red (`render_cli` with no escape at all). **Two defects in the draft, found by
reading it adversarially:** (1) `terminal_safe` keeps a line break — right for a diagnostic, which lays itself out on
several lines and is indented after its first (RW 4.34), wrong for a string that fills ONE line of a report: a catalog
whose `[cli.authority.header]` held `{program}:\n  effects:      (none)` started a forged `effects:` line under
`authority`'s header (witnessed red: `"m:\n  effects:      (none)"`). `render_cli` uses `terminal_line` now; the unit
tests assert that every en-US text is one line and that a catalog's or an argument's line break is shown escaped.
(2) The new `locale_cli` test had been inserted between `criterion7_json_bytes_identical_across_locales_human_differs`'s
doc comment and its `#[test]`, so that comment described the wrong test — moved back (routine run 20's lesson: an
insertion before an anchor must not carry the anchor). In the VM: `delulu-diag` 50/50; `locale_cli` 7/7,
`locale_plugin_cli` 3/3; clippy 0; `cargo deny --all-features check advisories` ok; the full suite 2,318 passed, 0 failed, 16 ignored (cargo's exit 0, the tree unmoved).

**CI read by this run, before its push:** `master`'s push runs on `640a7d5` — CI `39069505317` and `ocsf`
`39069505339` — success; the nightly `39108708245` (2026-10-18 07:43 UTC, on `640a7d5`, all 18 jobs, `heavy-gates`,
the three `miri-slow` and `microvm` included) — success. `actors_pingpong`: arm64 MEASURED 2.93x where the machine
gave 4.11x, passed; Ubuntu and Windows NOT MEASURED (busy runners — speedups 2.00–2.05x and 1.41–1.73x against
controls of 2.17–2.24x and 2.42–2.45x); macOS NOT MEASURED (3 hardware threads; 2.26x observed). Sandbox properties
unchanged: Linux x64 every property established, the guest a subordinate uid; arm64 the same as the same OS user;
Windows a per-run AppContainer, privilege escalation not confined; macOS filesystem reads not confined.

**The Sunday sweep (step 4 item 6), 2026-10-18.** CI: the nightly above ran every job on `master`'s code (a by-hand
`everything` dispatch would have repeated it). An adversarial pass, by hand, on the newest security-relevant feature —
RW 4.65's DL1410 hint, a command agents paste into a shell, built from an item the program chose. Two hypotheses, both
**refuted**: (a) `,` is in `plain_shell_char` and left unquoted, and PowerShell's argument mode reads `a,b` as an array —
but PowerShell 7.6.7 (installed in the VM from Microsoft's apt repository) passes `fs_write:/srv/out/report,final.txt`
to a native command as one argument, and `=+~@:` literally; (b) a `*` in a file name would make the "narrowest" permit
a wildcard wider than the use — but `dead_pattern_reason` refuses a glob in a path pattern and any `*` in a `declassify`
one, so such an item gets no hint item and the rule is named. The quoted alphabet (space and `()'&#`) reads literally
in bash, dash and pwsh 7.6.7. Unread here: `cmd.exe` and Windows PowerShell 5.1 (the laptop's). The live-document scan
found RW 7.10 still saying `CODE_OF_CONDUCT.md` is absent (closed in the record — D-V2-151 added it 2026-10-17).

## 2026-10-18 — the Survey mapped a worktree's `.git` file (on-demand routine run 1, continued)

**What.** Release procedure step 1 — `master` merged into `launch/2.0.0` — was done in a `git worktree` (its own
target directory, so the main checkout stayed on `master`). Its map, rebuilt there, had one node more than the same
files exported from the index: `file:.git`. A worktree's (or a submodule's) `.git` is a FILE, one line `gitdir: …`,
and `scan::walk` excluded `.git` only as a directory, so the map of a worktree could never match a clone — the
pre-commit hook (run 15) refused the merge commit, exactly as CI's freshness gate would have refused it on every
runner. `clean_checkout`'s gate asks `git check-ignore`, which never names `.git` itself, so nothing could see it.
`CLOUD_ROUTINE.md` has recommended a worktree for writing a slice since run 25; none had BUILT the map in one.
Fixed: the walk skips `.git` whatever its shape.

**Evidence.** `clean_checkout.rs` `a_worktrees_git_file_is_not_mapped` (a temporary tree with a `.git` file) red on the
old walk — it listed `.git` beside the tree's one document — green after; it also asserts the walk read the tree at all.
Every other tree walk in the Survey goes through `scan::walk` (`grep read_dir`). clippy 0; the full suite 2,319 passed,
0 failed, 16 ignored (cargo's exit 0, the tree unmoved).

**CI read:** `a3d7678`'s push runs — CI `39163449226` success (14 jobs, `miri-slow` and `heavy-gates` skipped as on
every push; `actors_pingpong`: arm64 MEASURED 2.88x where the machine gave 4.10x, passed; Ubuntu and Windows NOT
MEASURED, busy; macOS NOT MEASURED, 3 threads) and `openshell` `39163449144` success.

## 2026-10-18 — RW 4.17: a Guard seal and the other Unicode normalization on macOS (D-V2-156; on-demand routine run 1)

**What.** RW 4.17 named POSIX names that change under Unicode normalization as an open question. Asked as "what else
spells this?" of the Guard — a deny-list — on the one filesystem that treats two spellings as one name (APFS). Two
witnesses on a branch, read on `macos-latest` before any fix: `prim.rs`
`two_normalizations_of_one_file_resolve_to_one_spelling` printed `one file under both spellings: true` and both
spellings resolved and pinned to one (green: `realpath` gives an existing file one spelling — that half of the
hypothesis refuted); `guard.rs` `a_path_rule_covers_the_other_unicode_normalization_where_the_filesystem_ignores_it`
was RED — `"/srv/café" over "/srv/cafe\u{301}/new.txt"`: a rule on a name not yet created does not cover the other
spelling, which the filesystem creates as the same name. Linux and Windows read both green as two files (nothing
claimed, nothing folded). Fixed (D-V2-156): on macOS `path_within` compares in NFC after its case fold.

**Evidence.** Read on `macos-latest` (`witness.yml`): the measurement `39165640550` (`ccb97f5`) green — one file under
both spellings, one resolved spelling; the Guard witness `39165785709` (`80a6243`) RED as quoted; at the fix (`7d04370`)
the broker's whole unit suite `39166154433` 204 passed, 0 failed, the measurement `39166156467` green and printing `an
absent name keeps its spelling: NFC Some(true), NFD Some(true)`, and `guard_e2e`, `sandbox_guard_e2e`, `broker_cli`
`39166158256` green. Windows: the measurement `39165642238` green. In the VM: `delulu-broker` 204/204;
`check-other-os.sh aarch64-apple-darwin` clean; clippy 0; the full suite 2,321 passed, 0 failed, 16 ignored (cargo's
exit 0, the tree unmoved). `Cargo.lock` gains one dependency line (`icu_normalizer` under `delulu-broker`), no version
moved.

**CI read:** `c4be914`'s push run `39165158474` — success. This commit's push run: read by this run if its budget allows
(the next entry), else first by the next run.

## 2026-10-18 — the e-stop witness observes its baseline; `launch/2.0.0` kept current, its CI read (on-demand routine run 1)

**What.** `launch/2.0.0`'s `jobs=everything` run `39165405002` (on `9763787`, the first merge) went red in one job:
`test (windows-latest)` — `sandbox_devices_cli` `an_operator_revoke_stops_a_guests_arm_mid_motion`, "the arm was driving
before the e-stop": the output was `SUPERVISOR UP`, then `REVOKED: … (operator-revoke); the safe-park fail-state is
engaged` at every command. The product was right — revoked, parked, nothing commanded after; the witness's baseline
("mid-motion") was a 300 ms sleep after the node appeared, and on that runner the revoke landed before the guest's first
command. Its first recorded failure. Six CPU hogs did not reproduce it in the VM (0 of 5); growing the window did — a
warm-up before the first command (`fib(27)`) made the old test red 3 of 3 with the runner's own message (routine run
18's lesson: grow what the window holds). The test now streams the guest's lines and revokes once a `COMMANDED` line has
come; the warm-up stays, so a sleep could never pass again.

**Evidence.** Old test with the warm-up: red 3 of 3. New: green 3 of 3; the whole `sandbox_devices_cli` target (14
tests) green 4 of 4 with six CPU hogs. Read on runners at `65a4ec7` (`witness.yml`, the whole target): `windows-latest`
`39168481629` — 14 passed, 0 failed (the runner terminated only `vctip`, no `delulu`); `macos-latest` `39168483913` —
success.

**The launch branch.** Release procedure step 1 twice: `9763787` (RW 6.1 and the Survey's worktree fix) and `b8f9f16`
(GUARD-NORM-1's fix, D-V2-156 after D-V2-155 in the decision log), each with its CHANGELOG lines in the 2.0.0 section
and the gates green on the launch tree. `39165405002` (`jobs=everything` on `9763787`): 15 jobs success, `test
(windows-latest)` red in this witness alone (2,253 passed, 1 failed), `miri-slow (delulu-check)` and `(delulu-broker)`
still running when this was written; the release dry run `39165442908` on `9763787` — success. `39167478125`
(`jobs=everything` on `b8f9f16`) running, unread; its release dry run `39167480432` — success. This test's fix reaches
the launch branch with the next merge of `master`.

## 2026-10-19 — RW 4.57: a run's own node ends with the run, however it ends (D-V2-157; on-demand routine run 2)

**What.** RW 4.57 (the red team's F-01, routine run 17, "code reading agrees, not re-run") said a killed run leaves its
nodes live under the daemon. Re-run first, by hand, on a scratch daemon (`/tmp/rt`): a program that loads the `shout`
plugin and then spins, stopped by `--limits cpu=3`, left its root (`process`) and its plugin's node `[live]`; and the
CONTROL — the same program ending cleanly — left its own root `[live]` too (RW 4.51 had revoked only the plugin's). The
daemon's tree lives in its memory: every run added root authority nobody held, for the daemon's whole life. Fixed
(D-V2-157): `broker_client::RunNode` holds the root a run issues for itself for exactly as long as the run — revoked as
itself at its end (after `close_devices`), and reaped by the broker's dead-man (D-V2-106; 10 s, beaten every 2 s by a
thread of the run's own) when the run never gets there. At both places a root is issued for a run (`run_cmd.rs`, the
sandboxed host in `guest.rs`). A lease run's node is the delegator's and is not held — the half RW 4.57 keeps open.

**Evidence.** Witnesses in `plugin_load_cli.rs`, red on the old code: `under_the_broker_daemon_a_runs_own_node_ends_with_the_run`
(`("…", "process", "live")` after a clean end) and `…_a_killed_runs_nodes_are_revoked_by_its_dead_man` (30 s after
`Child::kill`, both nodes `live`; the baseline asserts both live while the run runs). Beside them, green before and after
and each falsified: `…_a_sandboxed_runs_own_node_ends_with_the_run`, `…_a_live_run_outlasting_its_dead_mans_period_keeps_its_node`
(a 1.5 s period, a 4 s run that then prints — the dead-man must not reap a run that is alive), and the unit test
`the_run_dead_man_period_can_only_be_shortened`. `scripts/mutants.py`: **M455** (the end revokes nothing) red in both
clean-end witnesses; **M456** (armed for an hour) red in the kill witness; **M457** (the beater never beats) red in the
live-run witness; **M458** (the knob may lengthen) red in the unit test; **M459** (the sandboxed host forgets its hold) and
**M460** (the ordinary run forgets it) red each in its own; the control green (M455's first text did not compile — a typo,
rewritten and re-run). The six RW 4.57 tests with the machine loaded by four concurrent copies of their binary: 12 of 12
rounds green (the harness refused shell busy loops — the routine now says how to load the CPUs). clippy 0.
Read on runners at `abd71c2` before `master` moved (`witness.yml`, `plugin_load_cli` filtered to `under_the_broker_daemon`): `windows-latest` `39181218534` — 8 passed, 0 failed (the runner terminated only `vctip`, no `delulu`);
`macos-latest` `39181220241` — 8 passed, 0 failed, no orphan.
The full suite at `abd71c2`: 2,325 passed, 1 failed, 16 ignored, the tree unmoved — the one red `delulu-runtime`'s thread-classification gate (`every_thread_either_sizes_its_stack_or_is_listed_as_never_running_a_program`) naming the new beater, `broker_client.rs:296`; listed with its reason (`4b0adcb`), green. Again over the fix and these records (`4b0adcb` and this entry's tree): **2,326 passed, 0 failed, 16 ignored** (cargo's exit 0, the tree unmoved).

**CI read at the start of this run:** `3f2ac0d`'s push run `39170518002` and `0c84447`'s `39170730116` — success (14 jobs;
`miri-slow` and `heavy-gates` skipped, as on every push). `actors_pingpong` on `0c84447`: arm64 MEASURED 2.87x where the
machine gave 4.11x, passed; Ubuntu and Windows NOT MEASURED (busy); macOS NOT MEASURED (3 threads). Sandbox properties as
recorded before (macOS `filesystem_confinement absent`, its reads not confined). On `launch/2.0.0`: `39169269103`
(`jobs=everything` on `e098438`, the branch's newest commit) — **success on all 18 jobs**, `miri-slow (delulu-broker)` 1 h
35 min; `39165405002` (`9763787`) — 17 success, `test (windows-latest)` red in the e-stop witness fixed since, its Miri jobs
all green; `39167478125` (`b8f9f16`) — **success on all 18 jobs**, its `miri-slow (delulu-broker)` done at 00:12 UTC (2 h 51 min).

## 2026-10-19 — RW 4.57's lease half; LEASE-DROP-1 (RW 4.68); RW 4.35's stale cell (on-demand routine run 2, continued)

**RW 4.57's lease half (`6f98ae5`).** A `--lease` run's node is the delegator's, so it is never ended by a run; what the run
mints under it — its devices' nodes (a lease carries no plugin authority: measured, `--lease` with `--grant plugin=.` met
DL0703) — is dead-manned as it is minted. `RunNode` now holds a `Reaper`: the set of `(node, key)` one beater keeps alive —
the run's own root, or, under a lease, each node its custody client mints (`attenuate` arms it; one that cannot be armed is
revoked and refused). Witnesses: `under_a_lease_a_killed_runs_minted_node_is_revoked_by_its_dead_man_and_the_lease_lives`
and its sandboxed twin (the device node reaped as itself; the lease still `live`), `under_a_lease_a_run_that_ends_leaves_the_lease_live`.
Mutants M461 (no arming at the mint — the old behaviour) red in both killed-lease witnesses; M462 (the lease revoked at the
run's end) red in the ended-lease witness; M463 and M464 (each call site's lease branch skipped) red each in its own. The
root half re-falsified on the refactor: M465 (the beater beats nothing) red in the live-run witness, M466 (the end revokes
nothing) in both clean-end witnesses, M467 (armed for an hour) in all three kill witnesses; controls green. The reaper is
not handed to actor-thread clients: under a lease nothing mints from another thread, so that code could not be witnessed
(routine run 18's rule) — removed. 11 RW 4.57 tests, four concurrent copies, 12 of 12 rounds green.
Read on runners at `6f98ae5` before `master` moved (`witness.yml`, `plugin_load_cli` filtered to `under_`): `macos-latest`
`39184608216` and `windows-latest` `39184606157` — 11 passed, 0 failed each (Windows terminated only `vctip`).

**LEASE-DROP-1 (RW 4.68), found on the way.** `redeem_lease` refuses each local grant a lease cannot take by name — except
three. Witnessed on a scratch daemon: `run plain.delulu --lease T --grant plugin=.` (and `exec.native`, and the full compute
envelope) printed `ran`, exit 0 — the grant dropped; `--grant console` was refused. Fixed by naming each with its reason. The
witness, `satellite_demo.rs` `run_lease_refuses_every_local_grant_but_a_foreign_library_path`, reads every grant form from
`delulu toolchain --json` and asks each beside `--lease` (`foreign.c` the control that passes). Red on the old code naming
exactly `exec.native`, `compute=…` (DL1911 answering first) and `plugin=./plugins` (DL1401 at the redeem). The compute
attestation gate (DL1911) now steps aside beside `--lease`: a lease run never uses a local compute grant, and answering
DL1911 first sent the operator to a waiver that led only to the lease's refusal. Mutants M468–M470 (each kind's refusal
removed) and M471 (the gate answering first) red; `compute_cli` 18/18. Runner reads at `86ad969`: the sync-log entry.

**RW 4.35's status cell** said `Open — found, witnessed` beside its own text recording the close (routine run 10, D-V2-86) —
the fix (`channel::Within` on the daemon's replies) and its witness are in the tree; corrected, as RW 4.64's was.

**The container restarted at 01:20 UTC** (every process's start time) and took the third full suite with it, mid-run at
`egress_cli`; the tree and the scratchpad survived. The suite was run again over all of it (suite 4): **2,329 passed, 1 failed** — `monitor_cli`
`with_on_monitor_death_continue_a_killed_monitors_runs_live_on` asserted the chain held no `deadman-arm` at all, its claim
being that `--on-monitor-death continue` arms nothing; the run under that monitor is a lease run with a device, whose
device node is now dead-manned (`6f98ae5`). The question was too wide, not the product wrong: it now asserts that every
arming is of a node the run minted (a child of the run's node) and that there is one — never the monitor's. Mutant M472
(the monitor arms under `continue`) red; `monitor_cli` 22/22. Suite 5 over the final tree: 2,330 passed, 0 failed, 16 ignored (cargo's exit 0, the tree unmoved).

**CI read:** `d144882`'s push runs — CI `39183143824` and `ocsf` `39183143907` — success (14 jobs; `actors_pingpong`: arm64
MEASURED 2.97x where the machine gave 4.10x, passed; Ubuntu and Windows NOT MEASURED, busy; macOS NOT MEASURED, 3 threads).
`launch/2.0.0` at `b4748fa` (this run's merge of `d144882`): the release dry run `39183562777` — success; `jobs=everything`
`39183559939` — at 02:05 UTC 16 of 18 jobs success, `miri-slow (delulu-check)` and `miri-slow (delulu-broker)` still
running: the next run reads them.

## 2026-10-19 — MONITOR-CHAIN-1: a long chain killed a healthy monitor's fleet; the monitor reads only what was appended, verified (D-V2-158, RW 4.49; on-demand routine run 3)

**Start of run** (05:05 UTC, no `routine-fire-payload`): `master` at `49f6c6f` (run 2's last commit), nothing newer;
`origin/autosave/laptop`'s newest snapshot still `b4a9e18` (recovered by run 1); `claude/wonderful-hamilton-bo8k53` and
`claude/friendly-thompson-vz6m23` one commit each, superseded as recorded 2026-10-17; every other `claude/*` branch in
`master`. Survey `ok` (1522 nodes, 14894 edges); `doctor --check` all checks passed. Run 2's claims re-run: `monitor_cli`
22/22, `plugin_load_cli` 24/24, `satellite_demo` 5/5, `compute_cli` 18/18 — all hold. **CI read:** `49f6c6f`'s push run
`39190850972` — success (14 jobs; `miri-slow` and `heavy-gates` skipped, as on every push; `actors_pingpong`: arm64
MEASURED 2.93x where the machine gave 4.05x, passed; Ubuntu and Windows NOT MEASURED, busy; macOS NOT MEASURED, 3 threads,
2.33x observed); `launch/2.0.0` at `3aafb77`: `jobs=everything` `39189498514` — **success on all 18 jobs** (`miri-slow`:
syntax 20 min, check 77 min, broker 97 min) — and the release dry run `39189501390` — success; `39183559939` (`b4748fa`) —
success on all 18. The sandbox properties per OS as on every run since PS-E-01 (macOS `filesystem_confinement` absent,
reads not confined; every other OS established). The 2026-10-19 nightly had not fired (07:4x UTC).

**Found: a monitor killed its own fleet once the chain was long enough.** RW 4.49 recorded, from routine run 15's red team
(F5), that each poll of `monitor watch` re-reads the whole chain — "≈200 ms at 10,000 records", LOW, a latency. Measured
here it is worse than a latency: the monitor beats its dead-man between polls, so once ONE whole read outlasts the
dead-man's period, a healthy monitor stops beating and its own broker revokes `g_M` — every run under it — for nothing but
the chain's length. By hand: a chain of 250,000 records (written by the chain's own `AuditLog`), a daemon, one node, `monitor
watch --rule envelope` with the defaults (poll 250 ms, dead-man 2,000 ms): `ended: the broker's dead-man revoked … this
monitor sent no beat within 2000 ms`, `records_read: 0`, the node `[revoked]` — no run had done anything. A whole read
(`audit::query`) costs about 3 µs a record in a release build (100,000 records: 0.22–0.34 s) and 11.5 µs in a debug one
(1.13–1.31 s); `AuditLog::open` re-reads it twice (250,000 records: 0.61–0.85 s, release). So a release monitor with the
default period dies past roughly 600,000 records — a control loop recording 100 commands a second writes that in under
two hours — and sooner with a shorter period.

**Fixed (`90d6ddb`): `delulu_broker::audit::Follower`.** It follows the chain from a point and reads only what was
appended after it — the rest of the day file it stopped in, from the byte it stopped at (a seek), and every later day file
(a record is never filed behind the newest file, AUDIT-DAY-1). Complete lines only: a writer puts down a record and then
its line break, so a line without its break is left for the next read, and the end a follower starts from never counts
one. Each record must name the previous record's hash as its `prev_hash`, and its own hash must recompute — `verify`'s
link check; a line that does not is returned as UNVERIFIED, never as a record, and the follower goes on from its stored
hash so one bad line does not blind it to what follows. A day file cut short under it (or gone) is a rewritten chain: the
follower says so, reads nothing in between, and moves to the end — the monitor's old "the record it last read is no longer
in the chain" warning, kept. The monitor starts at the chain's end with one and reads only what is new each poll; it warns
for an unverified line (`delulu audit verify` names where the chain breaks) and judges nothing on it — RW 4.49's other read
residual (F4: a forged `deny` line quarantined an innocent run). The class, stated: this catches a damaged line and one
written without the format; a same-user writer who computes the hash is category 7, as for every file in that directory.

**Witnesses.** `monitor_cli` `a_long_chain_does_not_kill_a_healthy_monitor`: its state is a chain whose ONE whole read,
measured in the test, takes twice a 500 ms dead-man period — grown with the writer's own lines (`into_record`, `to_line`)
until it does (101,251 records here, one read 1.32 s), its anchor removed so the daemon's opening anchors it again, and
`verify` asked before the watch (a control: the chain is real). The monitor must stay live four and six periods into the
watch and then have read at least two records past its start (its own arming and a delegation made after it began — the
witness of an absence proves it looked). **Red on the old code** (the run's lease "revoked by audit seq 101257", the
dead-man's), green after; **4 of 4 green with four concurrent copies of its test binary loading the machine** (each
1.03–1.16 s per whole read). Four `audit` unit tests: `a_follower_reads_only_what_was_appended_after_it_across_days` (a
record it had passed is overwritten in place with a line that would not verify — nothing returned is about it),
`a_follower_leaves_a_line_without_its_break_for_the_next_read` (both a reading follower and one started meanwhile),
`a_follower_returns_lines_that_do_not_verify_as_unverified_and_reads_on` (an altered deny, a non-record, a record naming
another predecessor; the record after them read), `a_follower_whose_file_was_cut_short_says_the_chain_was_rewritten`; and
the FIFO refusal (RW 4.38) asks the follower too. **Mutants** (`scripts/mutants.py`, control green): M473 (each read from
the file's start) red in three unit tests, M473b (the same) red in the monitor witness, M474 (no link check) and M475 (no
hash check) red in the unverified test, M476 (a line without its break read) red in the partial test, M477 (a cut file not
noticed) red in the cut test, M478 (no regular-file check) red in the FIFO test, M479 (the monitor re-starts at the end
each poll — reads nothing) red in five `monitor_cli` tests. Clippy 0; `scripts/check-other-os.sh` clean for Windows and
macOS. **Read on runners before `master` moved** (`witness.yml` at `90d6ddb` on `claude/monitor-chain-follower`):
`monitor_cli` on `macos-latest` ``39205018972` (22 passed — the witness read 101,251 records, one whole read 1.22 s)` and `windows-latest` ``39205021062` (21 passed — the witness 1.23 s; Windows terminated only `vctip`)`; `delulu-broker` `lib` filtered to
`audit::tests` on `macos-latest` ``39205023031` (23 passed)` and `windows-latest` ``39205025243` (22 passed; the FIFO test is Unix-only)`.

**The full suite** over the code: 2,335 passed, 0 failed, 16 ignored (163 binaries; cargo's exit 0, the tree unmoved).

**The same class on the writing side (RW 4.53), named:** `AuditLog::catch_up` re-reads and re-counts the whole chain under
the append lock whenever another writer moved it, and `AuditLog::open` does on every opening — so each daemon request after
a sandboxed host's record pays for the chain's history (the red team's 4.9 s `grants revoke` at 60,000 records). The next
slice.

## 2026-10-19 — RW 4.53's writer half: a writer catches up by reading only what was appended (D-V2-159; on-demand routine run 3, continued)

**The same class on the writing side.** `AuditLog::catch_up` runs under the append lock before an append once another writer
has moved the chain, and it re-read and re-counted the WHOLE chain — every day file's last hash, then every day file's
count (`recover`). The daemon is one such writer and every sandboxed run's host another, so on a long chain each daemon
request after a host's record — an operator's e-stop revoke among them — paid for the chain's history (routine run 17's red
team: `grants revoke` 4.9 s median at 60,000 records beside six sandbox loops). **Measured here** (a scratch script, the
same 100,000-record chain for each, the daemon a debug build, another writer appending one record every 50 ms through
`AuditLog`): ten recorded daemon requests (`grants delegate`) — at `90d6ddb`, alone median 15 ms (max 20), beside the
writer median **3,817 ms** (max 4,479); at `7ae1524`, alone 14 ms (max 17), beside the writer **14 ms** (max 21).
`audit verify` passed after each (100,072 and 100,098 records).

**Fixed (`7ae1524`): `AuditLog::read_appended`.** A catch-up first reads only what was appended since this handle last
wrote — the rest of its day file from the byte it stopped at, and every later day file — each record linked to the one before
and its hash its own (`verified_link`, the check the follower now shares), and then the anchor must agree with the head and
the count that reading gives. Anything else — no file this handle wrote, a file cut short, a line that is not a record or
does not verify, an anchor that disagrees — falls back to the whole recovery, which refuses what it refused before. A first
draft also refused a line without its break; it was removed: a writer puts down its line, its break and then the anchor,
all under the lock, so such a line can never come with an agreeing anchor — a wall no outcome depends on.

**Witnesses** (`audit` unit tests): `a_writer_catching_up_reads_only_what_another_writer_appended` (one writer's catch-up
across a day boundary chains onto the other's last record although a record behind both was made unreadable in place — the
old code re-read it and refused; `verify` still finds it), `a_line_appended_without_its_anchor_is_refused_as_before`, and
`a_file_cut_short_under_a_writer_is_refused_as_before`. **Mutants:** M480 (the old behaviour: no incremental read) red in
the first; M481 (no anchor check) red in the second; M482 (later day files unread) red in the first; M483 and M484 (the
shared link or hash check gone) red in the follower's unverified test; **M485 (a cut file read as "nothing appended")
SURVIVED the first round** — the next record would have chained onto a head the file no longer held — and is red in the
third witness, written for it. Control green. Clippy 0; `check-other-os.sh` clean for Windows and macOS. Read on runners
before `master` moved (`audit::tests`): `macos-latest` `39206620758` (26 passed), `windows-latest` `39206622817` (25 passed; the FIFO test is Unix-only; Windows terminated only `vctip`).

**The full suite** over the code: 2,338 passed, 0 failed, 16 ignored (163 binaries; cargo's exit 0, the tree unmoved).

**Left in RW 4.53:** `AuditLog::open` still reads the whole chain on every opening — each sandboxed run's host and each CLI
command that records opens it (the red team's 3–4.5 s sandboxed run at 60,000 records). Its fast path would trust the
anchor as `catch_up` already does, which needs the anchor to name the newest file and its length; a format change, its own
slice.

**CI read:** `200df1c`'s push runs — CI `39206009719` — success (14 jobs; `miri-slow` and `heavy-gates` skipped, as on every push; `actors_pingpong`: arm64 MEASURED 2.95x where the machine gave 4.03x, passed; Ubuntu and Windows NOT MEASURED, busy; macOS NOT MEASURED, 3 threads) and `ocsf` `39206009590` — success.

## 2026-10-19 — RW 4.53's last part: an opening reads one line when the files measure as the anchor says (D-V2-160; on-demand routine run 3, continued)

**Measured.** `AuditLog::open` — every sandboxed run's host and every command that records opens the chain — read every
day file twice (each file's last hash, then each file's count). On the same 100,000-record chain, release builds: at
`7f1a4a1` three openings took 303, 266 and 180 ms; at `5dea22e` the first took 247 ms (it re-affirms the anchor, now
with its note) and the next two 0.106 and 0.033 ms. `audit verify` passed after (100,000 records).

**Fixed (`5dea22e`): the anchor notes where the chain stood** (`AnchorAt`): the newest day file, its length and record
count, every older day file's bytes together, and the top seq — fixed-size, written beside the head and count it always
held (an older binary reads those two as before). An opening takes the anchor's count and seq when every file still
measures so — the newest file named and of its length, the older files' bytes unchanged — and the newest file's last record,
read from its end (64 KiB, or the whole file for a longer line), is the anchor's head. Anything else is the whole recovery,
which refuses what it always refused; an anchor without the note (every log written before this) is read the whole way once
and re-affirmed with it. A writer keeps the note current as it appends, rolls to a new day file, or catches up after another.
**Not claimed:** an older record edited in place to the same length is `verify`'s to find — the opening never recomputed a
hash; any length change in any file still sends the opening to the whole recovery.

**Witnesses** (`audit` unit tests): `an_opening_reads_only_the_newest_files_last_line_when_the_files_measure_as_the_anchor_says`
(an older record made unreadable in place, the newest record short and 70 KB — the opening takes the anchor's word and the
next record chains onto the head; `verify` still finds the damage), `the_fast_opening_agrees_with_the_whole_recovery_after_every_append`
(two writers, 24 appends across three midnights, each catching up after the other: after every append the fast opening is
taken and equals what reading every file gives, field by field), `an_opening_still_refuses_an_older_line_deleted_or_the_newest_record_replaced`
(three cases), `an_anchor_without_its_note_is_read_the_whole_way_once`. **Mutants:** M486 (the old behaviour) red in the
first; M487 (older bytes unchecked) and M488 (the head unchecked) red in the refusal test; M489 (the newest file's length
unchecked) **survived** its first round on the head check — and the length is the only wall for a line deleted from the
MIDDLE of the newest file, so that case was added and M489 is red in it; M489b (length and head both) red there and in
`audit_truncation`'s `reopening_a_truncated_log_refuses`; M490 (a writer's roll to a new file not counted), M491 (a
catch-up's records not counted in the newest file) and M492 (the note never written) red in the agreement test (M490 and
M492 also in the first); M493 (a newest line longer than the window read as no record) red in the first. Control green.
Clippy 0; `check-other-os.sh` clean for Windows and macOS. Read on runners before `master` moved (`witness.yml`, the whole
`delulu-broker` lib with `audit_truncation` and `audit_wiring`): `macos-latest` `39209490393` (224 passed across the three targets), `windows-latest` `39209493185` (222 passed — the FIFO tests are Unix-only; Windows terminated only `vctip`).

**The full suite** over the code: 2,342 passed, 0 failed, 16 ignored (163 binaries; cargo's exit 0, the tree unmoved).

**RW 4.53 now:** the chain's history no longer costs a daemon request (D-V2-159) or an opening (D-V2-160). What is left is
routine run 16's original measurement — the serial daemon's queue behind other writers' appends on a SHORT chain — not re-run.

**CI read:** `7f1a4a1`'s push runs — CI `39208000005` — success (14 jobs; `miri-slow` and `heavy-gates` skipped, as on every push; `actors_pingpong`: arm64 MEASURED 2.98x where the machine gave 4.05x, passed; Ubuntu and Windows NOT MEASURED, busy; macOS NOT MEASURED, 3 threads) and `ocsf` `39207999996` — success. `launch/2.0.0` at `d2814e8` (this run's merge of `7f1a4a1`): the release dry run `39208470155` — success; `jobs=everything` `39208467616` — at 06:33 UTC 16 of 18 jobs success, `miri-slow (delulu-check)` and `miri-slow (delulu-broker)` running (they take hours): the next run reads them.

## 2026-10-19 — on-demand routine run 3 (postscript): `4c8e44a`'s push runs read; `launch/2.0.0` at `6c01264`; the monitor's listing measured

**CI read:** `4c8e44a`'s push runs — CI `39210944095` — success (14 jobs; `miri-slow` and `heavy-gates` skipped, as on every
push; `actors_pingpong`: arm64 MEASURED 2.95x where the machine gave 4.12x, passed; Ubuntu and Windows NOT MEASURED, busy;
macOS NOT MEASURED, 3 threads) and `ocsf` `39210944303` — success. `master` is green on every slice of this run.

**`launch/2.0.0` at `6c01264`**, the merge of `4c8e44a` (release procedure step 1): the CHANGELOG change and the decision log
merged by themselves (D-V2-155, D-V2-158 to D-V2-160); the map rebuilt there (1524 nodes, 14955 edges, 0 errors); `doctor
--check` ok; book, core_invariance, distribution, doctor_cli, evidence_claims, governance, repository_structure, release and
`monitor_cli` (23) green, `audit::tests` 30 and `audit_truncation` 4 green on the launch tree. Dispatched: the release dry
run `39212180218` — success; `jobs=everything` `39212177717` — running at this entry. `39208467616` (`d2814e8`): 17 of 18
success, `miri-slow (delulu-broker)` still running — superseded by `39212177717`, kept as evidence.

**Measured, not a defect at this size: the monitor's listing.** MONITOR-CHAIN-1's question put to the monitor's other per-poll
work — it asks the daemon for the whole grant tree (`List`) every poll, and the tree keeps every revoked node. A scratch
daemon (debug build) with 3,002 nodes: the CLI's `grants list --json` 238–304 ms for 1.7 MB (most of it the CLI's own
rendering); a recorded daemon request (`grants delegate`) took median 12 ms (max 16) alone and 13 ms (max 23) with a monitor
polling that tree every 25 ms — the daemon's own listing costs it little at this size. **At 30,003 nodes** (a 17 MB listing;
the CLI's `grants list --json` 2.7 s) the median stayed 13 ms but the **max went from 17 ms to 371 ms** beside the monitor —
each whole-tree listing holds the serial daemon, and an e-stop's revoke waits behind it. Not a deadline for the monitor (its
beater beats while it waits on the daemon, D-V2-127). Recorded as RW 4.69 (open, measured): ask only for the live subtree
under the monitor's node.

## 2026-10-19 — RW 4.69: a tree too large for one listing killed a healthy monitor's fleet (MONITOR-FRAME-1, D-V2-161)

**What.** The on-demand routine's third run measured the monitor's per-poll `List` as a latency (371 ms at 30,003 nodes) and
recorded RW 4.69. Putting its own question further — what does the listing's growth meet? — the laptop found a constant:
`broker_ipc::MAX_FRAME`, 16 MiB a frame. The tree keeps every node for the daemon's life, each with its holder's description
(no bound), and past the cap the daemon's `write_frame` refused, the serve loop dropped the connection, and the client read
`broker unreachable`. By hand on Windows (debug build, a scratch daemon): a monitor on `g_M` with one run under it, then
nodes of another holder with 30,000-byte descriptions — at 500 the listing answered (15.6 MB of JSON), at 550 `grants list`
failed, the monitor ended `broker unreachable: PeekNamedPipe: win32 error 233`, and four seconds later `g_M` and its run were
revoked by the monitor's own dead-man (`by_seq 1106` both).

**Why this way (D-V2-161).** The monitor needs two things of the tree each poll: its node's state, and the run each newly read
record's actor belongs to. `RunsUnder { root, nodes }` answers exactly those, from parent pointers (`Broker::run_under`), so
its cost is what is asked — never the tree. Asking AFTER the read also closes what the old order left open: a node minted
between the listing and the read had its records skipped for good (by construction; not witnessed). An oversize answer is
now `DL1401` in words, so no client reads "unreachable" from a broker that is there. The human surfaces that read the whole
tree (`grants list`, `grants tree`, the atlas overlay) still cannot read one past a frame: RW 4.70, open.

**Evidence.** `monitor_cli.rs` `a_tree_too_large_for_one_listing_does_not_kill_a_healthy_monitor` — the state measured by
`grants list` itself failing past one frame (562 nodes of 30,000 bytes on Windows, 142 of 120,000 elsewhere, four writers at
once), then three dead-man periods, then the run under the monitor quarantined at its first envelope refusal: **red on the old
product code** (`("revoked", "revoked")` for the monitor's node and its run), green after (14 s locally).
`brokerd.rs` `a_monitors_question_is_answered_from_the_nodes_it_names` (only asked nodes under the root, with their runs; a
revoked node still names its run; an unknown root `None`; 4,097 names refused; two 9 MiB descriptions make `List` a DL1401 in
words and the daemon serves on); `tree.rs` `a_node_maps_to_its_run_under_a_root_and_nothing_outside_is_under_it` (the old
`monitor_cmd.rs` unit test's tree, moved to where the walk lives). Mutants, each on its own target: M494 — the oversize
answer dropped as before: the daemon test red, the e2e red at its last assertion (the monitor itself survived: the two halves
are separate); M495 — `run_under` answering any node as its own run: the tree test and `a_monitor_sees_nothing_outside_its_own_subtree`
red; M496 — the asked node instead of its run: the tree test red; M497 — the cap removed: the daemon test red. Locally green:
`monitor_cli` 22, `audit_cli`, `broker_cli`, `budget_cli`, `estop_cli`, `grants_cli`, `json_contract`, `plugin_load_cli`,
`sandbox_devices_cli`, `within_cli`, the `delulu` binary's 183 unit tests and `delulu-broker`'s; clippy 0; fmt clean. **The
full suite did not run on the laptop** (the owner's rule of 2026-10-19: it runs on GitHub — the run is named in
`CLOUD_SYNC_LOG.md`).

## 2026-10-19 — RW 4.63: a watch held outside the daemon lets its dead-man fire — M346 red at last

**What.** DEADMAN-BUSY-1's fix (D-V2-127, routine run 22) gave the monitor a thread that beats for it while its watch waits
on the daemon, and only then; a watch held anywhere else must go silent, so its dead-man fires as a stopped monitor's does.
No test held the watch outside the daemon for a period, and mutant M346 — the beater beating whatever the watch does —
survived (RW 4.63). The row asked for a seam: "a test-only pause, or a chain read made slow on purpose".

**Why this way.** Not a pause: no test-only switch in a safety component. The hold is the monitor's own work on data a second
writer may legitimately append: one record whose authority carries a long padding, written through the chain's own writer
(`AuditLog::append` — its lock and anchor, as a sandboxed run's host writes) while the monitor watches. The follower reads that
line whole, parses and hashes it before it can beat again; the test first measures, with the same `audit::Follower` on a
scratch chain, the size whose one read takes three periods on this machine (doubling from 4 MiB), so the state is measured
rather than assumed.

**Evidence.** `monitor_cli.rs` `a_watch_held_outside_the_daemon_lets_its_dead_man_fire`: a 32 MiB record, one read 1.4 s here,
`--death-after 200` — the monitor's node revoked by its dead-man, the monitor ending `the broker's dead-man revoked …` with a
failing exit, in 11.7 s. **M346** (the beater's `waiting` test removed) **red**: the watch was still being beaten for 31 s after
the record landed. `monitor_cli` 23 passed together; clippy 0; fmt clean. **The full suite did not run on the laptop** — it
runs on GitHub (`CLOUD_SYNC_LOG.md`).

## 2026-10-19 — RW 4.70: the whole-tree reads come in pages; TREE-STATE-1; and the pull request's suite read red (D-V2-162)

**The CI of RW 4.69 and 4.63, read** (pull request #1, branch `laptop/rw-4-69` — the owner's rule of 2026-10-19: the suite
the laptop does not run runs on GitHub and is read before `master` moves): `39224116908` (`ea7d42b`) and `39224995083`
(`3203d3f`) — **failure**: every job but the four suites success, and each suite red on ONE test,
`message_spacing.rs` `no_message_carries_a_flattened_continuation` — Windows 2,279 passed, Ubuntu 2,344, macOS 2,318, arm64
2,344, 1 failed on each. The cause was mine: D-V2-161's DL1401 text was written into `brokerd.rs` through a Python string,
and a backslash-newline continuation lost its backslash, so the message printed thirty spaces mid-line — the class that test
exists for. Rewritten on one line here; `message_spacing` green. Both new monitor witnesses passed on all four runners
(`monitor_cli` 23–25 by OS). `master` did not move.

**What (RW 4.70).** Past one frame `grants list`, `grants tree` and the atlas overlay could not read the tree. Reading
`Broker::tree` to page it found two more: each node's children by a scan of every node — measured 0.13 s at 2,000 nodes,
0.54 s at 4,000, 2.6 s at 8,000, 10.9 s at 16,000 (debug, a flat tree), the serial daemon serving nothing else, an e-stop's
revoke included; and each node drawn in its own state — with the real binary, a grant delegated `--ttl 2s` and a child
under it, three seconds later: `grants list` and `grants inspect` said the child was `expired`, `grants tree` said `[live]`
(TREE-STATE-1). D-V2-162: `ListPage` (4 MiB pages in id order), `brokerd::list_all` for all three surfaces, the tree drawn by
the CLI from the pages in the daemon's format, and the daemon's `Tree` kept for older clients with one index, a stack and the
inherited state.

**Evidence.** `grants_cli.rs` `grants_list_and_tree_read_a_tree_past_one_frame` — 562 nodes of 30,000-byte descriptions under
one parent (Windows; 142 of 120,000 elsewhere): every node listed once, one tree line each, each under the parent; **red on
the old product code** (`grants list` refused), green after (7.8 s). `brokerd.rs`
`the_listing_comes_in_pages_and_the_clis_tree_is_the_daemons_tree` — the CLI's tree equals the daemon's `Tree` text byte for
byte over nested nodes, a revoked subtree, a child of an expired grant and a description carrying a line break and an escape;
then four 5 MiB descriptions: `List` refused in words, the pages each past the one before, within the budget, together `List`'s
nodes. `tree.rs` `the_tree_shows_a_child_of_an_expired_parent_as_expired` (red on the old render). Mutants M498–M502 red, each
on its own target. The RW 4.69 witness measured its state by `grants list` failing; it reads the paged listing now and asks
that the descriptions alone pass one frame. Locally green: sixteen integration targets (the atlas, grants, monitor, broker,
audit, e-stop, CLI-contract and JSON-contract ones, `message_spacing`), the `delulu` binary's 184 unit tests,
`delulu-broker`'s 215; clippy 0; fmt clean. The full suite runs on GitHub (`CLOUD_SYNC_LOG.md`).

## 2026-10-19 — RW 3.4: loops on the WASM engine; its memory grows; WASM-HYGIENE-1 (D-V2-163)

**`master` moved.** Pull request #1's third run `39229088627` on `98edf2f` (RW 4.69, 4.63, 4.70) — **success on every job**:
Windows 2,283 passed, Ubuntu 2,348, macOS 2,322, arm64 2,348, 0 failed on each; `fuzz`,
`formal`, `microvm`, `miri` (diag, atlas, ffi), `supply-chain`, `editor`, `lints` success. `master` fast-forwarded
`0a9afbf` → `98edf2f` and pushed; GitHub marked the pull request merged.

**What (RW 3.4).** The WASM backend refused every loop, so a program that counted, accumulated or built a string ran only
on the interpreter — five of the six Study-C kernels among them. Now: `while`, `break`, `continue` (a label depth kept by
`Cx::emit`, a loop stack), assignment to a local, and `Str.len()`. Two defects found on the way, each witnessed on the OLD
backend with a program that compiled there: **its heap was a fixed 1 MiB, never reused** — a recursion building 4 MB of
strings stopped with `DL0903: memory access out of bounds` where the interpreter printed the string — so every allocation
now goes through `__alloc`, which grows the memory, and the host's allocator grows it too; and **WASM-HYGIENE-1** — the
inliner resolved a lambda's names through the generic function's frames: `apply(fn(v) { v + n }, 3)` with `apply` holding
its own `n = 100` printed 103 on WASM, 4 on the interpreter. An inlined body now sees the frames of where it was written.

**Evidence.** `delulu-wasm/tests/loop_parity.rs` (six): loops with `break`/`continue`, nested loops, `return` from a loop,
`while false`, shadowing, a `Bool` loop flag and `"héllo ✓".len()` == 7, against the interpreter and against the values
themselves; the growth witness and the hygiene witness — **both red on the old backend** (`DL0903`; `103 210 101` against
`4 12 2`); the host growth (a 3 MiB file read); 400 generated loop programs (agreed, both faulted, looping — all counted);
and what stays out (`for`, a lambda assigning its frame's variable). The 50,000-program differential gate now draws a fifth
of its programs from the loop generator: **50,000 checked, 46,855 agreed, 3,145 both faulted, 0 mismatches** (release, the
laptop). `budget_cli.rs`: an endless `while` on WASM stopped by `--limits cpu=1`, a doubling string on WASM stopped by its
256 MiB memory budget. Mutants M503 (no growth in `__alloc`), M504 (none in the host), M505 (`len` counts bytes), M506 (no
hygiene), M507 (`break` continues), M508 (an inlined body may assign its definer's variables), M509 (`continue` breaks) —
all red. `wasm_cli`'s DL1201 example was a `while`; it is a `for` now. Study-C's kernels on WASM: 4 of 6 compile, each
printing the interpreter's output (debug build: `loop_sum_1m` 29 ms against 2,244; not a measurement of record —
`HOT_PATH_TABLE.md` says the table is not re-measured). The book's DL1201 list corrected (it also named the clock, which
compiles since phase 3k). Locally: `delulu-wasm`'s whole suite, the ten `delulu` targets that run the WASM engine,
clippy 0, fmt clean. The full suite runs on GitHub (`CLOUD_SYNC_LOG.md`).

**Postscript (RW 3.4): `str(Bool)` in any program.** It was DL1201 unless the program also used a foreign library, which
was what interned `"true"` and `"false"`; every module interns them now (a loop's flag is a `Bool`). `loop_parity.rs`'s first
test prints `str(ok)` and `str(t > 4)` on both engines. `delulu-wasm` green; `wasm_cli` and `cli` green.

## 2026-10-19 — on-demand routine run 4: the laptop's RW 3.4 verified and merged; WASM-HYGIENE-2 (RW 3.8)

**The laptop's pull request #2 (`laptop/wasm-loops`, RW 3.4, D-V2-163) — verified, then merged.** Its CI `39233385903` on
`4d11575` — **success on every job** (the suites on Ubuntu, Windows, macOS and arm64, `fuzz`, `microvm`, `miri` diag, atlas
and ffi, `supply-chain`, `formal`, `editor`, `lints`; `heavy-gates` and `miri-slow` skipped, as on every pull request). Re-run
in the VM on `4d11575`: `delulu-wasm`'s whole suite (91 lib, `loop_parity` 6, every parity target), `wasm_cli` 10,
`budget_cli` 10, and the 50,000-program differential by hand in release — **50,000 checked, 46,855 agreed, 3,145 both
faulted, 0 mismatches**, the laptop's figures exactly. The autosave snapshot `bbb8ea3` (09:39 UTC) is the draft of `c3445be`;
the two committed later are a superset of it (its `codegen.rs` lacked the `str(Bool)` commit, its `loop_parity.rs` the same
test's two values). `master` fast-forwarded `98edf2f` → `4d11575`. Also read: `master`'s push runs on `98edf2f` — CI
`39232237537`, `openshell` `39232237530`, `ocsf` `39232237519` — success; the release schedule `39235210830` and the CI
schedule `39222644536` (on `0a9afbf`) — success; `launch/2.0.0`'s `jobs=everything` runs `39208467616` (`d2814e8`) and
`39212177717` (`6c01264`) — **success**, their `miri-slow` jobs included.

**Found while reading the diff: WASM-HYGIENE-2.** The backend resolves locals through `Cx::scopes` and function values
(the lambdas it inlines) through `Cx::callables`. `compile_block_inner` pushed a frame onto the first only, so `let f = fn…`
inside an `if`, `else` or `while` body went into the FUNCTION's frame of function values and replaced the outer `f` for the
rest of the function. Probes, both engines (`delulu run … --engine wasm`): an `if` whose body shadows `f` — the interpreter
`2` then `1`, WASM `2` then `2`, **and the same on `98edf2f`'s backend** (the `if` form compiled before RW 3.4); a `while` whose
body shadows it — `300 1` against `300 0` (new with RW 3.4, which made `while` compile). Silent wrong output. Four other
probes agreed on both engines (a capture shadowed or re-assigned after the definition reads the new value on both — the
interpreter's frame is shared; an inner `var n` shadowing a captured `n`; a loop that calls a capturing lambda). The
differential could not have seen it: the generator wrote no function value at all, as WASM-HYGIENE-1's own note says.

**Fixed:** a block opens a frame of function values beside its locals (`compile_block_inner` pushes and pops both, on its
error path too). **And the gate is taught the shape:** `random_loop_program` binds `h = fn(x: Int) -> Int { … }` (capturing
`a`) before the loop in half its programs, calls it in the loop body and in the tail, and in three of four of those shadows
it in the loop body or in an `if` there — drawn from an RNG stream of its own, so every other line is what each seed gave
before. `random_loop_programs_agree_with_the_interpreter` counts the shadowing programs and asserts more than 80 of 400.

**Evidence.** `loop_parity.rs` `a_function_value_bound_in_a_block_is_that_blocks` (the `if`, the `else` and the `while`
forms, against the values themselves) — **red on the old backend**: `2 23 33 300` against `7 21 31 301`; and
`random_loop_programs_agree_with_the_interpreter` with the new generator — **red on the old backend** at its first shadowing
program (`interp=Ok(1651022093) wasm=Ok(2)`). Green after: 7 of 7; the sample's readings, printed by the test — 346 agreed,
54 both faulted, 371 looping, **149 shadowing a function value**. The differential with the new generator, release:
**50,000 checked, 46,235 agreed, 3,765 both faulted, 0 mismatches**. Mutants (`scripts/mutants.py`): M510 (no frame of
function values pushed) red in three tests, M511 (pushed, never popped) red in two, M512 (the generator never shadows) red
on the count; control green. clippy (`delulu-wasm`, all targets) 0; fmt clean.

## 2026-10-19 — on-demand routine run 4 (continued): Study C re-measured; the harness checks each lane's answer and times in microseconds

**Why.** RW 3.4 put four of Study C's six kernels on the WASM engine, and `HOT_PATH_TABLE.md` said its own figures were
NOT re-measured — "re-running Study-C (release, all three lanes) is the next step before any row changes" (the laptop's
inbox, and this run's). The VM has gcc 13.3 and CPython 3.11, so every lane runs here.

**Two harness defects, found by reading it and by its first run here.** (1) A lane was timed whenever its process exited 0:
nothing compared what it printed with the interpreter's answer, though the code's own comment said "a benchmark where the
lanes do different work measures nothing" — and the WASM lane had just gone from one benchmark to four. Now `checked` times a
lane only when it printed the interpreter's answer (line endings aside); any other answer is listed, with both, among the
benchmarks that lane could not run. (2) It timed in whole milliseconds. The first run here read every C process as `1` or
`0` ms, and `nested_calls_macro`, whose C read 0, vanished from both ratio tables without a word (the laptop's C processes had
read 5–6 ms, so it never showed there). Now microseconds, published as fractional milliseconds, and a benchmark a ratio leaves
out is named. The record also carries its date and machine (`measured_at`, `machine`: OS, CPUs, kernel, processor, the
`delulu`, gcc and CPython versions; schema `study-c/2`), and the report no longer prints "These numbers are the v1.0
baseline" over whatever it measured.

**Evidence.** `study_c::tests` (five): the civil date against Python's at a leap day and a century; the answer check; a
report from a lane that printed `2` where the interpreter printed `46368` — listed, untimed, in no ratio; a 0.64 ms C time
kept (545.3×) and a 0 µs one named; `time_it` reads a real process (the test binary's `--list`) at ≥ 100 µs and keeps what it
printed. Mutants M513 (the mismatch arm skipped), M514 (every answer the same), M515 (no machine table), M516 (`as_millis`
again), M517 (a zero-C benchmark not named), M518 (microseconds published as milliseconds) — all red; control green. clippy
(`delulu-measure`) 0.

**The measurement** (`measurements/study-c/REPORT.md`, `results.json`; 2026-10-19 14:14 UTC; release `delulu 1.0.0` from
`79f83d6`; Linux 6.18 x86_64, 4 vCPUs, Intel Xeon @ 2.10GHz; nothing else running): C 0.99–1.30 ms on every benchmark —
process start-up; WASM 7.64–11.97 ms on the four it runs (`wordcount_macro` needs `split`, `list_map_macro` a `List`);
interpreter 8.63–365.36 ms; CPython 13.74–80.39 ms. **wasm ÷ C 6.7×–11.7× (geo-mean of the four 9.3×); interp ÷ C
7.5×–311.4×.** Every WASM answer matched the interpreter's. The interpreter is within a run's spread of 2026-07-20
(`loop_sum_1m` 365 ms against 363, `fib_recursive_24` 83 against 77): the ratios grew because the C lane's start-up shrank
(5–6 ms there). Where both engines ran, WASM took 1.7× (`string_build_20k`) to 30.6× (`loop_sum_1m`) less time than the
interpreter. The first run, before the microsecond fix, read the same shape (interp 82/349/17/9/10/284 ms; WASM 8/10/8/11).
**Criterion 1 is still not met** — coverage 4 of 6, and each of the four past 2.5× — and the hot-path table says so, with
the 2026-07-20 table kept beneath it verbatim and that run whole in `measurements/study-c/runs/2026-07-20/`. The same range
was updated where it is quoted (`README.md`, `docs/QUESTIONS.md` twice, `docs/for-agents.md`, `measurements/METHODOLOGY.md`
§3.2–3.3); the dated phase records and the 1.0 announcement keep theirs. **Not measured:** a compute-only ratio — every
figure is a whole process, and the study has no start-up control per lane (an empty `main`); RW 3.4 names it.

## 2026-10-19 — on-demand routine run 4 (continued): Study C measures start-up; `master` red on `lints` once, fixed (`45ca06d`)

**`master` red, and why.** `81da0ae`'s push run `39268713452` went red on `lints` alone — clippy's `manual_range_contains`
on `time_it_reads_a_process_in_microseconds`, a test written after this run's last clippy, in a commit whose message said
"clippy 0". Read from the job's log (one error, nothing else); fixed in `45ca06d` — made in a worktree at `81da0ae` so it held
nothing else, `cargo clippy -p delulu-measure --all-targets -D warnings` exit 0 and the crate's tests green there — and pushed
within minutes. The run's other jobs were not waited for before the fix went up; its push run is read below.

**What (the start-up control).** The hot-path table said a compute-only ratio needs a start-up control per lane. Study C now
runs one — a program that prints `0` and nothing else — through the same per-lane code as every benchmark, then takes it out
of the benchmarks (`take_control`), so no table, count or ratio includes it. The report shows its figure per lane, with its
spread, and each benchmark less its lane's floor; no ratio is computed from those differences, and the report says why.

**The published record is now this run** (2026-10-19 14:26 UTC; the same build, machine and quiet as 14:14): start-up C
0.92 ms, interpreter 4.70, WASM 7.01, CPython 11.91. WASM 8.31–11.14 ms on its four kernels, 1.30–4.13 ms above its floor;
C 0.97–1.50 ms, **0.05–0.58 ms above its floor** — so the C lane's work is inside a millisecond's noise and no compute ratio
is offered. `wasm ÷ C` **6.5×–11.5×** (geo-mean 8.8×), `interp ÷ C` **8.9×–359.0×**. Between the 14:14 run and this one,
single ratios moved by up to a quarter (`loop_sum_1m`'s `interp ÷ C` 290.4× then 359.0×), mostly through C's ~1 ms
denominator — the table now says so. Every document quoting the range was updated to this run (README, QUESTIONS twice,
for-agents, METHODOLOGY, CHANGELOG, RW 3.4); this log's previous entry keeps the 14:14 figures as what that run read.

**Evidence.** `study_c::tests` seven: two new — the floor shown per lane with its spread, each benchmark less it
(360.00, 4.75, 0.27), a lane where the control failed named, and the control absent from the timings table and the WASM
count; and `take_control` removes the control and only it (`startup_program`'s kind is the one taken). Mutants M519 (the
control copied, not taken out), M520 (no subtraction), M521 (the control's failures unshown) red; control green. clippy
(`delulu-measure`, all targets) 0 after the last edit.

## 2026-10-19 — on-demand routine run 4 (continued): `master` red on `formal` (Lean's host refused the runners), fixed (`205f0f4`); WASM-OOM-1 (RW 3.9)

**`master` red, not this run's code.** `0e4e9a2`'s push run `39270059515` and `6c26aed`'s `39271301242` went red on `formal`
alone, each at the same step and before any proof ran: elan downloads Lean 4.32.2 from `releases.lean-lang.org`, which
refused the runners' TLS (`[35] SSL connect error`) eight minutes apart; `45ca06d`'s `formal`, about five minutes earlier,
had passed. Neither commit touched the model. The VM got no answer from that host either, and a 200 from the same
tarball as a release asset of `leanprover/lean4` on GitHub. **Fixed in `205f0f4`:** the job fetches that asset with curl's
retries, checks a pinned SHA-256 (`5f2069e6…72aa`, taken from the asset on 2026-10-19 — Lean publishes no checksum file to
check it against), and runs that `lean` on `DeluluCore.lean`, a single core-Lean file. Verified in the VM before the push:
the step's script, taken from the workflow by a YAML parser and run with `RUNNER_TEMP` set to a scratch directory — hash
OK, Lean 4.32.2, every theorem "does not depend on any axioms", exit 0. Its push run is read below or by the next run.

**WASM-OOM-1 (found reading the trap mapper while verifying RW 3.4).** `__alloc` trapped with `unreachable` when the
memory could not give an allocation, and the host reads every `unreachable` as the arithmetic helpers' overflow — so a
program that doubles a string past WebAssembly's 4 GiB, under `--limits mem=12884901888`, printed `error[DL0901]: … integer
overflow` with no arithmetic in it (by hand, on the old backend; under the default 1 GiB budget the budget stops a program
first, on both engines). Now `__alloc` traps as an access out of bounds (a 4-byte load at 0xFFFF_FFFF, past any 32-bit
memory) and the host's DL0903 text says "memory access out of bounds — or the WASM engine's memory exhausted (WebAssembly's
4 GiB)"; the same program now prints that. DL0903 is the code the fixed heap's end reported before RW 3.4. Residual, in RW
3.9: the exhaustion has no code of its own (DL0903's explanation is about indexing) — a code would be a decision.

**Evidence.** `codegen.rs` `alloc_tests::an_allocation_the_memory_cannot_give_is_not_an_overflow`: a module holding
`__alloc` alone and an exported caller, both failure branches — an end that wraps past 2^32 (`-8` bytes from a heap pointer
of 16; the test's first form used `-64`, which does not wrap — the memory simply grew to ~4 GiB — corrected before any
fix) and a growth the memory's maximum refuses — **red on the old `__alloc`** (`DL0901: integer overflow`), green after.
Mutants M522 (the wrap branch back to `unreachable`), M523 (the growth branch), M524 (DL0903's old text) red; control green.
`delulu-wasm` 129 passed; `wasm_cli` 10, `budget_cli` 10; the release differential 50,000 checked, 46,235 agreed, 3,765 both
faulted, 0 mismatches; `scripts/clippy.sh` clean (stamped).

## 2026-10-19 — the laptop: run 4 synced; the routine held at xhigh by an environment variable; `launch/2.0.0` at `c3b33f3`

**Synced.** `master` fast-forwarded to `8fc42f8`. Run 4's open CI items were read green: `205f0f4` (`39274259721`),
`8fc42f8` (`39275805346`), and `launch/2.0.0`'s `everything` and release dry run on `ece2454` (`39272359850`,
`39272366257`). The routine's own log, read through the routine API, shows what the usage limit cut off at 15:12 UTC: two
WASM fixes built but not committed (negating `Int::MIN`; `&&`/`||` short-circuit), and a red-team pass whose notes never
left the VM. Among its findings the log shows an interpreter panic in `rand.int` when `hi - lo` overflows. All are to be
redone on the laptop, each witnessed first (`docs/CLOUD_SYNC_LOG.md`, this date).

**Effort.** The owner saw the routine running Opus 5.5 at `medium`, although `.claude/settings.json` set `effortLevel` and
`modelSettings` to `xhigh`. Claude Code puts an explicit choice (`--effort`, `/effort`, `CLAUDE_CODE_EFFORT_LEVEL`) above
both settings keys, and its environment-variable reference says the variable outranks `--effort` and `/effort`. The file
now sets `env.CLAUDE_CODE_EFFORT_LEVEL = xhigh`. The routine API refuses environment variables on a trigger (HTTP 400,
tried with the routine's full unchanged configuration). Nothing was changed there. Whether the next run starts at `xhigh`
is read from that run, not assumed.

**`launch/2.0.0`.** `c3b33f3` merges `master` up to `8fc42f8`. On its tree the map is 1532 nodes with 0 errors,
`doctor --check` is ok, clippy is clean and stamped, and the document gates are green. Its `everything` run `39296491537`
and release dry run `39296497024` were dispatched. Their results are read in a later entry.

## 2026-10-19 — the laptop: run 4's lost WASM fixes and its red team's findings, redone (RW 3.10–3.14)

**Where they came from.** On-demand routine run 4 built two WASM fixes and ran a Sonnet 5.5 red-team pass on the WASM
fragment, and the usage limit stopped it before either was committed. Its run log, read through the routine API, names
the defects and some of the red team's programs; the VM and the agent's notes are gone. Each was witnessed again on the
laptop's binary at `6a26c1f` before any fix.

**The five.**
- **WASM-SHORT-1 (RW 3.10).** `&&`/`||` compiled both operands, then `i32.and`/`i32.or`. `let a = z != 0 && 10 / z > 1`
  with `z = 0`: `false` on the interpreter, `DL0902: division by zero` on `--engine wasm`. Now the right operand is compiled
  inside an `if` on the left (`compile_binary`, before either operand). The `And | Or` arm of the `Int` match is
  `unreachable!`, because the typed `&&` never reaches it.
- **WASM-NEG-1 (RW 3.11).** `-x` was `x * -1` with a bare `i64.mul`, so `-Int::MIN` printed `-9223372036854775808` where
  the interpreter faults `DL0901 integer overflow negating Int::MIN`. Now it calls the checked-multiply helper.
- **WASM-REFUSAL-1 (RW 3.12).** A program that matched on the error of an `fs.read_text("../x.txt")` outside its grant
  reported `DL0903: memory access out of bounds` on WASM and `DL0904 … escapes the granted scope` on the interpreter.
  Ungranted, it reported `DL0903` against `DL0703`. A refused host call returns a placeholder and the guest runs on, and
  `finish` reported whatever trapped next. `trap_or_refusal` now reports the recorded refusal first, in `run_main` and in
  the actor `main` alike.
- **WASM-OUTPUT-1 (RW 3.13).** On the same program the interpreter printed `start`, the first read and `NotFound` before
  its fault, and WASM printed nothing. The CLI printed the engine's captured output only on `Ok`, so a run that faulted,
  was refused or was stopped by its budget lost every line it printed. `HostConfig::stream_console` (default `false`, so
  every harness that captures is unchanged) prints each line as it is printed. The CLI sets it for `.delulu` and `.dwx`
  runs.
- **RAND-SPAN-1 (RW 3.14).** `r.int(0 - 1, 9223372036854775807)` panicked the debug interpreter at `prim.rs`
  (`attempt to subtract with overflow`). A release build wrapped to the right values. Now `hi.wrapping_sub(lo) as u64` and
  `lo.wrapping_add(…)` are exact, which is what the WASM host computes.

**Why the 50,000-program gate missed all five.** Its generators wrote no `&&`/`||` and no negation. They never refused a
host call, and the comparison is fault against fault, never the output before a fault. `gen_cmp` now writes a guarded
short-circuit in a third of its conditions (`d != 0 && x / d …`, `d == 0 || x % d …`), whose right side faults exactly
when the guard is false. `gen_expr` writes `(-e)` where it had written a second leaf.

**Evidence.** `crates/delulu-wasm/tests/fault_parity.rs` (new): `negating_int_min_faults_on_both_engines`,
`and_and_or_short_circuit_on_both_engines` (faults guarded off and right-side effects on both engines),
`rand_int_over_a_span_past_int_max_draws_the_same_values_on_both_engines` (three seeds),
`a_refusal_is_reported_even_when_the_guest_traps_after_it`. `wasm_cli.rs`
`what_a_program_printed_before_a_fault_reaches_stdout_on_both_engines`; `budget_cli.rs`'s WASM CPU-budget test now asserts
the line printed before the stop. Mutants (`scripts/mutants.py`): M525 (WASM-SHORT-1 restored), M526 (the bare `i64.mul`),
M527 (`hi - lo` in `Int`), M528 (the trap over the refusal), M529 (no streaming), each red; control green. Readings:
M525 red in three targets (`and_and_or_short_circuit_on_both_engines`, `random_loop_programs_agree_with_the_interpreter`,
`tests::wasm_matches_interpreter_on_random_programs_including_faults` — so both generators now write the shape); M526,
M527, M528 each red on their one test; M529 red on the `wasm_cli` witness and on `budget_cli`'s WASM CPU-budget test;
control green (`delulu-wasm` lib 87, `fault_parity`, `loop_parity` 7). After the last edit: `scripts/clippy.sh -j 4` clean
and stamped; `delulu-wasm` whole (87 lib, `fault_parity` 6, `loop_parity` 7 and the rest) and the CLI targets that run
the WASM engine (artifact_review, budget_cli, cli, guard_e2e, json_contract, plugin_cli, release, sandbox_modes_cli,
wasm_cli, message_spacing) green on Windows. The 50,000-program release differential runs in CI's `fuzz` job.

**A slip, caught before the commit.** `fault_parity.rs` already existed (C20's two fault-parity tests), and the new
tests were first written over it — the file tool did not refuse an overwrite of a file not yet read. `git status`
showed it modified, not added; the original was restored from `HEAD` and the new tests appended to it, both sets
green.

## 2026-10-19 — the laptop: the microVM's host against 2026's KVM escapes (RW 4.71)

**The owner asked** (2026-10-19): *"Linux recently patched vm issues, Search and learn about it, Is DeluluLang affected by
it, What should DeluluLang do differently"*. Read that day:
- **PwnEscape** (CVE-2026-99999, CVSS 9.3). The host's IPv4 stack trusted checksum-offload metadata that a guest sends on
  a virtio-net device, and its checksum write overwrote an IPv4 header already parsed. It was demonstrated on KVM and
  Firecracker hosts. Upstream fix `6785011f8b16` (mainline 2026-10-14). AlmaLinux shipped testing kernels on 2026-10-16;
  Red Hat says a local user can reach it too.
- **Januscape** (CVE-2026-53359), a KVM x86 shadow-MMU use-after-free reached through nested virtualization; fixed in
  stable on 2026-07-04.
- **Zapscape** (CVE-2026-64561), a KVM x86 shadow-page reclaim use-after-free reached two nesting levels deep, also a
  local root where `/dev/kvm` is world-usable. A batch of smaller KVM CVEs followed on 2026-10-18.
Two names (the CVE numbering of PwnEscape, an aggregator's framing) looked doubtful; AlmaLinux's and CloudLinux's
advisories confirm it.

**Is DeluluLang affected?** Its guests reach none of the paths, read from the code:
- The launcher configures one vsock and nothing else.
- The guest kernel is `tinyconfig` plus vsock, with no IPv4, no packet or Unix sockets, no KVM and no modules.
- The process sandbox (`jail.rs`) denies `socket`, `socketpair`, `unshare` and namespace-creating `clone`.
- Egress is the host's ordinary sockets.

What it cannot escape is that the microVM's wall *is* the host's KVM.

**What changed.**
- The launcher's device list is one function, `vm_api_calls`, which `the_vm_is_given_one_vsock_and_nothing_else` pins.
- `kernel.config` names network devices, virtio-net, TUN, vhost-net, virtualization and KVM as not set. `build-image.sh`
  already refused a kernel in which any `not set` line is on, so the absences are now enforced.
  `microvm_guest_kernel.rs` pins the lines on every platform; it is red with `# CONFIG_TUN is not set` removed. The
  options were off under `tinyconfig` already, so the built `.config` should be unchanged. That is reasoned from
  Kconfig, not built on the laptop: CI's `microvm` job builds the image under the new file and applies the refusal.
- `doctor` reports **KVM host posture** (each vendor module's `nested`, `/dev/kvm`'s mode, the kernel release). It
  notes nesting on or a world-usable device, with the escapes and the remedy, and never fails: GitHub's runners open
  `/dev/kvm` to every user. `kvm_posture` is a pure function, so `kvm_posture_tests` run on every platform.
- DEPLOYMENT.md *The microVM's host* gives the table, the sources and the host requirements: a patched kernel,
  `nested=0`, `/dev/kvm` for the group, no network device.

The laptop's WSL lab loads `kvm_amd` with its default `nested=1`; HANDOFF §11.3 says to pass `nested=0` next time.

**Evidence (RW 4.71).** Windows: `kvm_posture_tests` 3 passed; mutants M530 (nesting never noted) and M531 (the device's
mode never noted) red, control green; `microvm_guest_kernel` green, red with `# CONFIG_TUN is not set` removed. Linux (the
laptop's WSL, `~/delulu-gnu-target`): `microvm::tests` and `kvm_posture_tests` 11 passed; `cargo clippy -p delulu
--all-targets -- -D warnings` clean; M532 (a `/network-interfaces/eth0` call added to `vm_api_calls`, on a copy of the
tree) red on `the_vm_is_given_one_vsock_and_nothing_else`. `scripts/clippy.sh -j 4` clean and stamped on Windows.

## 2026-10-20 — the laptop: `List` on the WASM engine; WASM-IFTY-1, WASM-INTERN-1 (RW 3.4, 3.15, 3.16, D-V2-164)

**Why.** The last two Study-C kernels, `wordcount_macro` (`Str.split`, `len`) and `list_map_macro` (a literal, `map`,
`len`), were DL1201 on `List` alone.

**What.** D-V2-164 records the design:
- the header `[len][cap][data]`, so lists are shared as on the interpreter;
- `__list_new` and `__list_slot`;
- a `for` over a snapshot;
- `push` computing its value first;
- `get`'s `None` outside the list;
- `delulu:list.str_split` (the interpreter's own `split`);
- `delulu:list.index_fault` (the interpreter's DL0903 words);
- the imports and helpers only in a module that uses a list;
- reference capabilities looked through;
- `run_int_fn` under a host with nothing granted.

Both kernels now print the interpreter's answers on WASM (18000, 16000); **6 of 6 compile**. The two that newly do are
not re-measured.

**Found on the way, each witnessed red first.**
- **WASM-IFTY-1.** `pick(a, b, c) = if c { a } else { b }` over `Str`s: the interpreter printed `a left`, while
  `--engine wasm` printed `DL0904: invalid WASM module: failed to compile`. The `if`'s block type had been guessed `Int`
  for a variable. A `match` arm binding a `Str` was refused for the same reason.
- **WASM-INTERN-1.** `apply(fn(s: Str) -> Str { s + " and a literal" }, "a value")` panicked the compiler at
  `codegen.rs` `expect("interned")`.

Block types are now patched from what the first branch compiled to, and the guess functions (`block_result_ty`,
`expr_result_ty`) are deleted. The collector descends into lambdas, lists, records, fields and indexes, and a missing
literal is refused rather than panicked over.

**Evidence.**
- `crates/delulu-wasm/tests/list_parity.rs` (new, 9 tests): literals of `Int`, `Str` and `Bool`; `get` inside and
  outside the list; sharing through `let ys = xs` and a `ref` parameter; `push` of the list's own length; a `for` that
  pushes during its walk; break and continue in nested `for`s; `map` with captured names and a bound function value;
  `split` (empty parts, multi-byte characters, empty separator); the index fault at 3, -1 and `Int::MAX`; and what stays
  refused.
- `loop_parity.rs`'s and `wasm_cli.rs`'s refused examples became a list of lists, since a `for` over a list now
  compiles.
- `random_loop_program` builds, pushes, indexes, `get`s, walks and maps a list, from a stream of its own. 176 of 400
  sampled programs do, asserted over 120: 298 agreed and 102 faulted on both engines.
- Mutants M533–M540 each red, control green.
- After the last edit:
  - the 50,000-program release differential: 45,007 agreed, 4,993 faulted on both, 0 mismatches;
  - `scripts/clippy.sh -j 4` clean and stamped;
  - `delulu-wasm` whole (lib 87, `list_parity` 9, `loop_parity` 7, `fault_parity` 6 and the rest) green;
  - the CLI targets that run the WASM engine green (artifact_review, budget_cli, cli, guard_e2e, json_contract,
    plugin_cli, release, sandbox_modes_cli, wasm_cli, message_spacing, book).

## 2026-10-20 — the laptop: `Str == Str` on WASM; Study C over all six kernels; two harness and host fixes

**`Str == Str` and `!=`** compile to `__str_eq` (same pointer, then length, then each byte). It is a late helper, emitted
only in a module that contains an `==` or `!=`. That is detected by `module_any`, a module walker that names every
expression form, after WASM-INTERN-1's lesson; the list detection uses it too. `list_parity.rs`
`str_equality_matches_the_interpreter` covers an interned literal against itself, a built string, differing lengths and
bytes, multi-byte characters, empty strings and split words. The console generator now writes a comparison in about
half its programs, from a stream of its own. `conformance_parity`'s 2,000 programs assert over 700 comparisons, over 200
of them equal and over 200 unequal. Mutants M541 (the bytes read from the length prefix) and M542 (`!=` as `==`) are red
in both tests; control green.

**Study C, all six kernels on one machine.** It ran in the laptop's WSL, release, from a fresh clone of
`laptop/wasm-list` at `e85fcb5`.

The first run (2026-10-19 18:43 UTC) found two things:
- the CPython lane reported UNRUN, "no CPython", on a machine that had only `python3`: the harness ran `python`;
- `wordcount_macro` took 12.01 ms on WASM against 5.35 on the interpreter, 5.58 against 1.69 above start-up. Each
  host-side `split` wrote every part with its own allocation and four export lookups.

Fixed: `python_command()` tries `python3` then `python` (`the_python_lane_finds_python3_or_python`), and `write_str_list`
writes a split's list in one allocation and one write.

The second run (18:46 UTC, the same machine, the fixes applied) is published:
- interpreter 8.9×–489.5× C; WASM **6 of 6**, 9.8×–15.7× C (geo-mean 12.4×);
- start-up 0.55 ms C, 3.58 interpreter, 6.24 WASM, 11.67 CPython;
- `wordcount_macro` on WASM 8.70 ms (2.46 above start-up) against the interpreter's 5.61 (2.03). It is still slower,
  and the table says so.

The previous published run (the cloud VM's, 14:26 UTC) is kept whole in `runs/2026-10-19-vm/`. HOT_PATH_TABLE,
METHODOLOGY §3.3, README, QUESTIONS (twice), for-agents and REMAINING_WORK 3.4 quote the new run. Criterion 1 is still
not met. Its coverage half is closed for this suite; its speed half stands.

**Evidence.** `delulu-measure` 15 unit tests (`study_c::tests` with the new one); `list_parity` 10; `conformance_parity` 6;
mutants above; the second run's `REPORT.md` and `results.json` as published.

## 2026-10-20 — the laptop: every `Str` method on the WASM engine (RW 3.4)

**`trim`, `to_upper`, `to_lower`, `chars`, `contains`, `starts_with`, `replace` and `slice`** compile. Each is one host
import, `delulu:str.str_method(s, method, a, b, lo, hi) -> i32`: the method's number in `STR_METHODS`, its `Str`
arguments in `a` and `b`, its `Int` arguments in `lo` and `hi`. The host reads the strings, calls the interpreter's own
`prim::call_str_method`, and writes the answer back as the guest reads it: a string cell, 0 or 1, or a list written in one
allocation. The WASM engine therefore cannot disagree with the interpreter about Unicode, because there is one
implementation, not two. Both sides read the argument slots from one table (`str_method_sig` through
`str_method_arity`). A module that calls none of these methods imports nothing new. With `len` and `split`, every `Str`
method the checker knows compiles.

The codegen half was committed unfinished at the end of the previous session (`3ae3b3f`, marked WIP); this entry
finishes it: the host function, the tests, the generator and the records.

`str_parity.rs`:
- `each_str_method_matches_the_interpreter` — whitespace beyond ASCII (U+3000, tab, newline), `straße` uppercasing to
  `STRASSE`, a precomposed `é` that does not contain `e`, an empty pattern for `contains` and `replace`, the empty
  string, and slices with a negative start, an end past the string and crossed bounds;
- `str_methods_compose_with_the_rest_of_the_fragment` — chained calls in a pure function, a word split from a line with
  an empty part, a `for` over `chars`, arguments computed from variables, answers compared and counted;
- `only_a_module_that_calls_a_str_method_imports_the_interface`.

The console generator writes a chain of `Str` methods in about half its programs, from a stream of its own, before the
comparison line. Its texts include U+3000, `ß`, `ǅ` and `é`; its patterns include the empty one; its slice bounds run
from −2 to 6. `conformance_parity` asserts each of the eight methods appears in over 100 of its 2,000 programs.
`fragment_external_programs_are_compile_errors` named `s.trim()` as outside the fragment; it now names `fold`.

Mutants, all red, control green:
- M543 — the argument counts swapped;
- M544 — the `Str` arguments reversed;
- M545 — a `Bool` answer inverted;
- M546 — `lo` and `hi` swapped;
- M547 — the import never declared;
- M548 — the import always declared (red only in the import test);
- M549 — the generator writing no method (red only in the per-method count).

Measured with `delulu build FILE --target wasm` over the corpus's 17 entry points: still 5 compile. The two that
stopped at `.trim` move on: `04_effects` now stops at a named function passed as a value (`apply(double, 21)`),
`telemetry_report` at `parse_float`.

**Evidence.** `str_parity` 3; `conformance_parity` 6; the whole `delulu-wasm` suite (87 unit tests and every test file)
green; the release differential with the new generator: 50,000 checked, 45,007 agreed, 4,993 both faulted, 0 mismatches.

## 2026-10-20 — the laptop: a function's name as a value on the WASM engine; a generic body brings what it needs (RW 3.4)

**A function named as a value compiles**: `apply(double, 21)`, `xs.map(double)`, `let h = double` and `let g = h`. The
backend keeps a table, `named`, built by `named_value`:
- a compilable function becomes a function value whose body is a call to it, so its recursion and an early `return`
  stay its own;
- a generic function becomes its own body, inlined as a direct call to it already is.

Either sees what a function's own body sees and nothing of where it is named. A function value of the same name (a
lambda bound in a block, a generic function's parameter) hides the table, as does a local in a `let`. Direct calls are
compiled as before, so a module that names no function as a value is unchanged.
`examples/guide/04_effects.delulu`, which stopped at `apply(double, 21)` once `.trim` compiled, now compiles and prints
the interpreter's output (`--grant console`, both engines byte for byte). The corpus's entry points: 6 of 17.

**A generic function's body now brings what it needs.** The module's imports and helpers (`Str` methods, `__str_eq`, the
list helpers, the console) were chosen from the compilable functions' bodies alone, and the string literals were
interned from them alone. Yet a generic body is inlined into every function that calls it. So a generic using `.trim()`,
`==` on `Str`s, a list, `println`, or a literal found nowhere else was refused with DL1201. It was refused, never
miscompiled, but the same body compiled in an ordinary function. Probe, before the fix: DL1201 ("a `Str` method in a
module compiled without the string interface"); after: the interpreter's output. `body_is_compiled` now decides the
walks, and a generic body's literals are interned after the others in declaration order, so a module without a generic
is byte-identical.

`named_fn_parity.rs`:
- `a_named_function_is_a_value_as_on_the_interpreter` — `apply`, `twice`, a recursive `fact` with an early `return`, a
  `Str` function, `let`-bound and aliased, `map`, a generic `identity`, and a function named inside a generic body;
- `a_function_value_or_a_local_of_the_same_name_hides_the_function` — a lambda `double` in a block, a generic parameter
  named `double`, a local `double` copied by `let`;
- `a_generic_body_brings_what_it_needs`.

The loop generator, from a stream of its own, declares a top-level `step` in a third of its programs, binds the outer
`h` to it (`let h = step`), and gives it to `map` in place of a lambda half the time. `random_loop_programs_agree_with_the_interpreter`:
63 of 400 bind it, 21 map it (asserted over 30 and over 10).

Mutants, all red, control green:
- M550 — the table empty;
- M551 — the table before the function values (red in the hiding test);
- M552 — `let y = double` without its local check;
- M553 — a compilable function inlined as its body instead of called (red on `fact`'s `return`);
- M554 — no `let h = double`;
- M555 — `body_is_compiled` as `is_compilable`;
- M556 — a generic body's literals not interned;
- M557 — the generator naming no function.

**Evidence.** `named_fn_parity` 3; `loop_parity` 7; the whole `delulu-wasm` suite green; the release differential with the
new loop generator: 50,000 checked, 45,027 agreed, 4,973 both faulted, 0 mismatches.

## 2026-10-20 — the laptop: `parse_int` on the WASM engine (RW 3.4)

**`parse_int` compiles.** It is one host import, `delulu:str.parse_int(s) -> (parsed, value)`, which uses WebAssembly's
multiple results. The host reads the text and calls the interpreter's own `prim::call_builtin("parse_int", …)`. It
answers 1 and the number for `Some`, or 0 and 0 for `None`, and refuses anything else rather than reading it as `None`.
The guest builds the `Option[Int]` from the two, as `List.get` builds its own. The import is declared only in a module
whose compiled bodies call `parse_int`, found by `module_calls_builtin` (a builtin's name cannot be redefined, DL0302).
`examples/guide/03_errors.delulu` now compiles and prints the interpreter's output on both engines, byte for byte. The
corpus's entry points: 7 of 17.

`str_parity.rs`:
- `parse_int_matches_the_interpreter` — padded, signed and tab-and-newline-wrapped text, `0` and `-0`, both ends of the
  `Int` range and one past it, the empty string, `1_000`, `0x10`, Arabic-Indic digits, an inner space, a word, and a sum
  over split words;
- `only_a_module_that_calls_parse_int_imports_it`.

The console generator writes a `match parse_int(…)` line in a third of its programs, from a stream of its own, before the
comparison line. Its texts are numbers that parse, numbers that do not, and `str` of a generated `Int`.
`conformance_parity` asserts over 300 programs call it and over 100 print its `None`.

Mutants, all red, control green:
- M558 — the host answering `None` for every number;
- M559 — the guest's test inverted;
- M560 — the import never declared;
- M561 — the import always declared (red in both import tests);
- M562 — the generator writing no `parse_int`.

**Evidence.** `str_parity` 5; `conformance_parity` 6; the whole `delulu-wasm` suite green; the release differential with
the new console generator: 50,000 checked, 45,027 agreed, 4,973 both faulted, 0 mismatches.

## 2026-10-20 — the laptop: `Float` on the WASM engine (RW 3.4)

**`Float` compiles**: `+ - * /`, the six comparisons, negation, `%`, `float(…)`, `int(…)`, `str(…)` and `parse_float`.
Both engines use IEEE 754, so the guest uses WebAssembly's own `f64` instructions and the answers agree to the bit:
- a NaN compares false, and `!=` true, on both;
- `float(i)` is `f64.convert_i64_s` (round to nearest, as Rust's `as f64`);
- `int(x)` is `i64.trunc_sat_f64_s`: toward zero, saturating at the `Int` range, a NaN to 0, exactly Rust's `as i64`;
- negation is `f64.neg`.

What WebAssembly lacks, the host computes with the interpreter's own code:
- `%` is `delulu:float.rem`, Rust's `%`;
- `str` of a `Float` is `delulu:float.to_str`, the interpreter's `Value::display` itself;
- `parse_float` is `delulu:str.parse_float`, through `call_builtin` as `parse_int` is.

The foreign interface's `float_to_str` now also calls `Value::display`. The copy of that rule, `format_float`, is
removed, so there is one implementation, not two. `parse_int` and `parse_float` share `parse_by_builtin`.

A module carries `delulu:float` only if a compiled body makes or takes a `Float`: a literal, `float(…)`,
`parse_float(…)`, or `Float` written in a signature (`module_uses_floats`). Every other module is byte-identical.

`tests/corpus/tier3-application/telemetry_report.delulu` now compiles. Given a telemetry file with good, bad, exponent,
`inf` and padded readings, it prints the interpreter's answer on both engines. The corpus's entry points: 8 of 17.
`01_types` now stops at a record literal.

`float_parity.rs`:
- `float_arithmetic_matches_the_interpreter` — `0.1 + 0.2`, a third, negative zero, an overflow to infinity, a NaN and
  its comparisons, `%` with signs and by zero, and the casts at both ends of their ranges;
- `parse_float_matches_the_interpreter` — padded, signed and exponent text, and `inf`, `nan`, `1e999`, a word, `1.2.3`;
- `floats_are_held_as_the_interpreter_holds_them` — function signatures, a `var` in a loop, a list pushed, indexed,
  walked and mapped, an `Option[Float]`;
- `only_a_module_with_a_float_imports_the_float_interface`.

The console generator writes a `Float` line in a third of its programs, from a stream of its own (`gen_float`). Its leaves
are exact and inexact, overflow-prone and signed-zero; it uses every operator and an `if` on a comparison; the line is
printed through `str`, or cast back with `int`. `conformance_parity`: 678 of 2,000 programs, 49 printing a NaN and 15 an
infinity, asserted over 300, 20 and 10. (A first run printed 10 infinities; a leaf `0.3` became `1.0e200`.)

Mutants, all red, control green:
- M563 — `+` as `-`;
- M564 — `<` as `<=` (red in the random programs only, which compare equal operands);
- M565 — `%` as `rem_euclid`;
- M566 — the trapping `i64.trunc_f64_s`;
- M567 — an unsigned conversion;
- M568 — negation dropped;
- M569 — the interface never declared;
- M570 — the interface always declared;
- M571 — `parse_float` off by an epsilon;
- M572 — `str` as Rust's `to_string`;
- M573 — the generator writing no `Float`.

**Evidence.** `float_parity` 4; `conformance_parity` 6; `foreign_parity` 8; the whole `delulu-wasm` suite green; the
release differential with the new console generator: 50,000 checked, 45,027 agreed, 4,973 both faulted, 0 mismatches.

## 2026-10-20 — the laptop: an empty list typed by the checker on the WASM engine (RW 3.4)

**An empty list whose element type no `let` writes compiles**: `let out = []`, filled by `out.push(i)` later, or `[]`
passed straight to a function. The backend saw only syntax, so it refused such a list (DL1201). Yet the checker had
already inferred its type, and `CheckResult::node_types` records the type of every expression node. A new entry point,
`delulu_wasm::compile_checked(module, check_result)`, threads that table into the codegen (`Cx::node_types`). An empty
list literal takes its element type from there when the backend holds it: `Int`, `Bool`, `Float` or `Str`
(`list_scalar`). The command line's three compile sites (`run --engine wasm` in `run_cmd.rs`, and `build --target
wasm` and the sealed-artifact path in `cli.rs`) now call `compile_checked`. `compile_module` and `compile_module_with`
pass no types and still refuse such a list rather than guess.

`examples/guide/02_option_and_lists.delulu` now compiles and prints the interpreter's output; the corpus's entry points:
9 of 17. Of the book's six guide chapters, four now run on the WASM engine: `02`, `03`, `04`, `06`.

Tests:
- `list_parity.rs` `an_empty_list_takes_the_type_the_checker_inferred` — `let evens = []` then `push`, a function's
  `let out = []`, `total([])`, lists of `Bool` and `Float` begun empty. The test confirms `compile_module` refuses the
  program and `compile_checked` compiles it with the interpreter's output.
- `what_the_list_fragment_leaves_out_is_refused` loses its empty-list case and asserts the rest refused by
  `compile_checked` too.
- `wasm_cli.rs` `the_guide_chapters_the_wasm_engine_runs_match_the_interpreter` — chapters `02`, `03` and `04` through
  the command line, both engines byte for byte.

Mutants, all red, control green:
- M574 — no inferred type;
- M575 — an `Int` list typed `Str`;
- M576 — `run --engine wasm` back on `compile_module_with` (red in the CLI test).

## 2026-10-20 — the laptop: `==` on records and maps; values that hold themselves (RW 2.13, 2.14; 2.15 and 2.16 found)

**EQ-RECORD-1.** While reading `Value::eq` for the WASM `Float` work: its doc says it is the structural equality for
the value types `==` accepts. The checker accepts `==` on a record or a map whose contents are not opaque; it even refuses
a map with an opaque side by name. Yet `eq` had no arm for either and fell to `_ => false`. A probe on the binary at
`4d200b6` printed `false false true` for `p == p`, `p == q` (equal fields) and `p != p`; `m == m` was `false`;
`[p].contains(p)` was `false`; `assert_eq(p, p)` would fail naming two identical records. Fixed: a record equals one
of the same type whose fields are equal, a map one with the same keys holding equal values.

**DISPLAY-CYCLE-1.** Before adding those arms: what does a structural walk meet? `n.next = Some(n)` is a legal program,
and `str(n)` already overflowed the stack and killed the process. So would the new `==`. Fixed in both:
- `display` keeps the cells it is inside. A `List`, `Map` or `Record` cell met again renders as its shape around `...`
  (`Node { v: 1, next: Some(Node { ... }) }`, `[...]`, `{...}`), as Python renders a list that holds itself.
- `eq` keeps the pairs of cells it is comparing. A pair met again is taken as equal, so two values that hold themselves
  compare in finite time and differ only if something else they hold does.
- No identity shortcut: `[nan] == [nan]` stays `false`, as `nan == nan` is.
- The sets are `BTreeSet`s, so a scalar `==` allocates nothing.

**Found, open:**
- EQ-FN-1 (RW 2.15) — `f == f` on a function value checks clean and answers `false`.
- MSG-CYCLE-1 (RW 2.16) — `recover`ing a self-referencing `iso Node` and sending it to an actor overflows the stack in
  `value_to_msg`; the channel's `WireValue::from_value` and `assert_unique_graph` walk the same way.

Tests:
- `crates/delulu-runtime/tests/value_semantics.rs` `records_and_maps_compare_as_their_contents` (records, `contains`, list
  equality in and out of order, maps, a NaN in a list) — red on the old `eq`;
- `a_value_that_holds_itself_renders_and_compares_in_finite_time` (a record through an `Option` field, a record through
  a list it holds) — on the old code the test binary died, `STATUS_STACK_OVERFLOW`;
- `value.rs` `cycle_tests::a_map_that_holds_itself_renders_and_compares_in_finite_time`.

Mutants, all red, control green:
- M577 — `display` not marking a cell (the binary dies);
- M578 — a record pair met again taken as unequal;
- M579 — the record arm answering `false`;
- M580 — map keys ignored;
- M581 — an identity shortcut on lists (red on `[nan] == [nan]`);
- M582 — a map pair met again taken as unequal.

**Evidence.** `value_semantics` 2; the whole `delulu-runtime` suite (280 unit tests and every test file) and
`delulu-conform` green; `conformance`, `core_invariance`, `corpus_cli`, `examples_run` green.

**The owner, 2026-10-20:** *"push everything to test github repo frequently. enable that windows task scheduler for
automatic github push"*. The `DeluluLang-autosave` task's 7-hour window of 2026-10-19 had run out. It was re-installed
for 7 days (`-Hours 168`, from 2026-10-19 22:42 UTC), and a run by hand pushed snapshot `c94cbba` to `autosave/laptop`.
Recorded in HANDOFF §11.1 and the laptop's memory. Every verified commit is pushed at once, as before.

## 2026-10-20 — the laptop: an `iso` move carries its graph whole; the debug race checker asks for the guarantee (RW 2.16, D-V2-165)

**MSG-CYCLE-1, fixed.** `recover { var k = Node { v: 7, next: None }; k.next = Some(k); k }` is a legal `iso Node`.
Moving it to an actor overflowed the stack in `value_to_msg` and killed the process. The move now numbers each `List`,
`Map` and `Record` cell where it is first met, before what it holds, and carries a cell met again as
`MsgValue::Back(n)`. `msg_to_value` makes each cell empty and records it before rebuilding what it holds, so a back
reference finds it. The graph arrives whole: a cycle as a cycle, and a cell held twice as one cell held twice, where it
used to arrive as two copies.

**Two more, found on the way:**
- **The debug race checker.** It reported the cyclic graph, and the legal `recover { let a = [1]; [a, a] }`, as a
  compiler bug (DL1610). It wanted every node below the top held exactly once, after spec §6.2's *"refcount==1 across
  the moved graph"*, a proxy for the guarantee. It also never looked inside a variant or a map, so an outside alias
  reached through `Some(xs)` passed. It now counts the references the moved graph holds to each node (the top node
  gets two more) and reports only one from outside, walking through variants (each payload once) and maps. The old
  test `an_aliased_nested_node_is_a_dl1610_class_violation` built exactly the legal `[a, a]` shape and called it a
  violation; it now holds the list outside the graph, and two new tests pin both directions.
- **The sandbox channel.** `WireValue::from_value` recursed the same way. Its wire is a tree and it refuses what
  cannot cross honestly, so a value that holds itself is now `Untransferable` (DL1401).

D-V2-165 records the decision and amends spec §6.2. Refusing the cyclic move, or narrowing `recover`, was rejected: the
program is legal and the move is sound.

**Observed once, not reproduced (RW 7.20).** Two `channel.rs` confinement tests failed in one `--lib channel` run, both
at `host.denied().1 == 1`. The panic text was not kept, and the next 25 runs were green. Recorded rather than dropped.

**Evidence.**
- `actors_cli.rs` `a_graph_that_holds_itself_moves_to_an_actor_whole` (plain and `--debug-rcaps`).
- `actors.rs` `move_graph_tests::a_moved_graph_keeps_its_shape` (pointer identity after the move).
- `actors_trace.rs` 8 (three uniqueness tests, one rewritten).
- `channel.rs` `a_value_that_holds_itself_is_refused_and_one_held_twice_crosses`.
- Mutants M583–M590, all red, control green:
  - M583, M590 — no back-references (the unit test and the CLI test, each dying of the overflow);
  - M584 — a cell recorded after its contents;
  - M585 — no allowance for the top node;
  - M586 — the graph's own references uncounted;
  - M587 — variants not walked;
  - M588 — maps not walked;
  - M589 — the channel walking into a cycle.
- The whole `delulu-runtime` suite (282 unit tests and every test file) and `actors_cli`, `actors_bounded_cli`,
  `actors_cycles_cli` green.

## 2026-10-20 — the laptop: no `==` on functions; an opaque type inside a record field is seen (RW 2.15, 2.17, D-V2-166)

Asking after EQ-RECORD-1 whether the runtime answers `==` for every type the checker admits found two more:
- **EQ-FN-1** — `g == g` on a closure checked clean and answered `false`. A function has no structural equality, so
  `==`, `!=`, `assert_eq` and `List.contains` now refuse a type holding one (DL0605, in words about functions). `str`
  of a function stays `<fn>`.
- **OPAQUE-FIELD-1** — the walk over a record's written field types named no `Map`, no `Plugin` and no actor. So
  `type Vault { m: Map[Str, Secret[Str]] }` compared and printed, and so did records holding a plugin or an actor
  reference, all refused outside a record. Nothing secret reached the program, but R-5 did not hold in a field. The
  field walk now names what the type walk names.

One walk answers both questions, `opaque_for(t, fns)`: `is_opaque` for `str`, `lacks_equality` for the equality sites.
The DL0605 `explain` text now names functions and opaque values in containers.

**Evidence.**
- `delulu-check` `equality_on_a_function_is_dl0605_in_its_own_words` and `an_opaque_type_inside_a_record_field_is_seen`,
  both red on the old `check.rs`.
- Mutants M591–M598, all red, control green:
  - M591, M592 — functions uncounted in the two walks;
  - M593 — no `Map`;
  - M594 — no actor;
  - M595 — no `Plugin`;
  - M596 — the opaque wording for a function;
  - M597 — `==` asking `is_opaque`;
  - M598 — `str` refusing a function.
- `delulu-check` (all), `delulu-conform`, `delulu-diag`, `conformance`, `core_invariance`, `corpus_cli`,
  `examples_run`, `actors_cli`, `cli`, `json_contract`, `palette_cli`, `message_spacing` green.

## 2026-10-20 — the laptop: records on the WASM engine (RW 3.4)

**Records compile.** A non-generic `type R { … }` whose fields the backend can hold (any scalar, `Float`, an enum,
another record) is `Ty::Record(id)`: an i32 pointer to a cell with one 8-byte slot per field, in declaration order
(`RecordDesc`, registered in `build_enum_env`). The cell is shared, as the interpreter's `Rc<RefCell<…>>` is, so
`var q = p; q.n = 70` is read through `p`. What compiles:
- a literal, whose fields are computed in the order written, as the interpreter computes them;
- a field read;
- a field write through a variable or a field (`p.x = v`, `l.a.x = v`), the value computed before the place, as on
  the interpreter.

A write through a list element (`ps[i].x = v`) is refused by the checker (DL1604), so the backend holds no path for
it. `str` and `==` of a whole record, and a record with an `Option` or `List` field, stay DL1201.
`examples/guide/01_types.delulu` now compiles and prints the interpreter's output; the corpus's entry points: 10 of 17,
five of the six guide chapters among them.

Tests:
- `record_parity.rs` `records_match_the_interpreter` (every scalar kind, sharing, a two-level write, records in a list,
  walked, indexed and read with `get`);
- `a_literal_computes_its_fields_in_the_order_written` (the index fault before the division);
- `what_the_record_fragment_leaves_out_is_refused`;
- `wasm_cli.rs`'s chapter test now runs `01` too.

Mutants M599–M603, all red, control green:
- M599 — fields computed in reverse;
- M600 — slots stored reversed;
- M601 — a read at slot 0;
- M602 — a write at slot 0;
- M603 — records not registered.

**Evidence.** `record_parity` 3; `wasm_cli` 12. The whole `delulu-wasm` suite and the release differential were
started before the owner's *"97% current session usage used. Commit and push everything"*, and their results were not
read before this commit. This branch's CI is the record.

## 2026-10-20 — the laptop: linked lists and trees on the WASM engine; RCAP-CTOR-1, WASM-SCAN-1 (RW 3.4, 2.18, 3.17)

**A record field may hold an `Option`, a `Result` or a `List`.** `RecordDesc` keeps each field's `Ty` (`field_ty`: a
scalar, or an `Option`, `Result` or `List` of scalars), stored and loaded by its width (`store_ty`, `load_ty`); the
three compound kinds are pointers to cells shared as the interpreter's are. A linked list (`next: Option[ref Node]`)
and a tree (`kids: ref List[ref Tree]`, grown through the field) run with the interpreter's output.

**A constructor where no type is written compiles.** `let c = Red` and `Circle(1.5)` build the sum that declares the
name (`enum_of_ctor`, unique, as the checker's `variant_ctor` is); `Some(x)` takes its payload's type, `x` computed
first (`compile_some`); `None`, `Ok` and `Err` take the type the checker inferred (`checked_ty`, scalars and their
`Option`/`Result`/`List`). A written `let` type is now the value's expected type, so `let k: Result[Color, Str] =
Ok(Green)` compiles.

**RCAP-CTOR-1 (RW 2.18), found writing the linked list.** `head.next = None` and `p.c = Red` were DL1603, "an
undetermined capability": the capability pass resolved a name as a local, a function or a constant and not as a
constructor, and a field store refuses an undetermined value where a literal does not. A nullary constructor now has
its type's default capability (`default_k`), as `Some(2)` has. A first version made it a fresh composite, as `[]` is;
that refused `var o = None; o = Some(2)` (a fresh `None` binds `ref`, and `Some(2)` is `val`), which the old checker
accepted — caught by comparing the two binaries by hand, and now held by the test. Nothing newly refused: `check` on
all 266 `.delulu` files in the repository gives the same verdict and codes on the old and the new binary, and so do
eleven written-`ref`/`iso` destinations probed by hand (`let o: ref Option[Int] = None` is DL1603 on both, as
`Some(1)` there is).

**WASM-SCAN-1 (RW 3.17), found probing records.** `Stamp { at: root.clock().now_ms(), … }` stopped with "DL0904:
invalid WASM module" on `master`: `module_calls_method` had its own walk, which passed over a record literal's fields, a
field's receiver, a `spawn`'s arguments and a `recover` body, so the module called a clock import it did not have. It
is now `module_any`'s walk, whose `match` names every expression form. RW 3.16's closure had said the import scan
descends into every expression; it did not, and 3.17 says so.

Tests:
- `record_parity.rs` `records_holding_options_results_and_lists_match_the_interpreter` (the list built by prepending,
  walked and cut; the tree grown through `t.kids.push`, summed over a `Result` field; an `Option[Str]` field written both
  ways, seen through the list that holds the record);
- `constructors_with_no_written_type_match_the_interpreter`;
- `a_capability_used_inside_a_record_literal_brings_its_import` (clock and rand, fixed clock and seed on both engines;
  the runner now takes a `HostConfig`);
- `what_the_record_fragment_leaves_out_is_refused`: its `Option[Int]` field compiles now, so the refused cases are an
  `Option[List[Int]]` field and a `Map` field;
- `codegen.rs` `ctor_tests::a_constructor_two_sums_declare_names_neither`;
- `delulu-check` `a_nullary_constructor_stored_into_a_field_has_its_types_capability` (red on the old checker).

Mutants, all red, control green each time:
- M604 — a field only a scalar;
- M605 — the old import walk;
- M606 — no `Some` synthesis;
- M607 — no sum for a constructor;
- M608 — a written `let` type not the expected one;
- M609 — a nullary constructor `Unknown` again (the old checker);
- M610 — no `Option` from the checker's type;
- M611 — a constructor two sums declare taken as the first's — survived `record_parity` (the checker refuses such a
  name before the backend sees it) and is red on `ctor_tests`, written for it;
- M612 — the first version of the checker fix (fresh).

**Evidence.** `delulu-check` and `delulu-wasm` whole, green (`--no-fail-fast`, exit 0). The release differential: 50,000
programs, 45,027 agreed, 4,973 both faulted, 0 mismatches — the same counts as the slice before, because its generator
writes no record and no bare constructor; the parity tests above are this slice's witnesses. The corpus's entry
points: still 10 of 17 — this slice widens the language, not those programs.

## 2026-10-20 — the laptop: `str` and `==` of compound values on the WASM engine; RECORD-ORDER-1 (RW 3.4, 2.19)

**`str`, `==` and `!=` of a list, an option, a result, a sum or a record compile**, answered by the interpreter's own
`Value::display` and `Value::eq` — one rule for both engines, as `Str` methods and `parse_int` already are. The guest
passes the value's pointer and its type's descriptor (`type_desc`: the type, then every record and sum it reaches —
`R0=Node{v:i,next:oR0.}`), laid out after the literals on first use (`Descs`, `desc_ptr`; the memory and the heap's
start are now set after the bodies are compiled, so a descriptor named in a body lands before the heap). The host
(`delulu:value.display`, `delulu:value.eq`) rebuilds the value as the interpreter's `Value` from the guest's memory
(`reify.rs`), each record and list cell once by its address, so a value that holds itself holds itself there too and
`display` prints `Node { ... }` where the interpreter does; `p == p` compares one cell with itself on both engines; a
`NaN` keeps `[nan] == [nan]` `false`. The module carries the two imports only when a compiled body takes `str` of, or
compares, a value the checker typed as one of these (or as a type variable, for an inlined generic body); every other
module is byte-identical to before, and without the checker's types (`compile_module`) such a `str` is refused.

The guest's memory is read as untrusted, since a `.dwx` can be written by hand: every read is bounds-checked, a list
header claiming more elements than the memory holds is refused before anything is allocated, a tag outside its type's
constructors is refused, and so is a variant cell met again inside itself (no compiled program can build one — a
variant is never written after it is made — and it would otherwise recurse without end). Each refusal is DL0903 in
words.

**RECORD-ORDER-1 (RW 2.19), found writing the probe for `str` of a record.** The interpreter held a record's fields in
the order its literal wrote them: `P { y: "a", x: 2 } == P { x: 2, y: "a" }` was `false` (2.13's `eq` pairs fields by
position) and the two printed differently. A literal's fields are still computed in the order written and are now held
in the order the type declares (`Home.records`) — the order the WASM engine's cell uses.

Tests:
- `value_parity.rs` (new): `str_and_equality_of_compound_values_match_the_interpreter`,
  `a_value_that_holds_itself_and_a_nan_match_the_interpreter` (a record held twice in a list is one cell),
  `str_and_equality_inside_a_generic_body_match_the_interpreter`, `what_the_value_interface_leaves_out_is_refused`;
- `reify.rs` `a_descriptor_parses_and_a_malformed_one_does_not`, `a_malformed_guest_value_is_refused`;
- `value_semantics.rs` `a_record_holds_its_fields_in_the_order_its_type_declares` (red on the old interpreter; it also
  holds the computing order);
- `record_parity.rs`'s refusal test loses its `str` and `==` cases, which compile now and are held above.

Mutants:
- M613 — a record rebuilt again where met again: red (the value that holds itself);
- M614 — a list rebuilt again where met again: **survived**, and cannot be seen in this fragment: no list can hold
  itself without a list of lists, which does not compile, and a list met twice compares and prints the same either way;
- M615 — the variant guard gone: red (the unit test's process overflowed its stack — the crash the guard prevents);
- M616 — the list bound gone: red (the unit test's process aborted on the allocation);
- M617 — a record's fields described in reverse; M618 — `!=` as `==`; M619 — no value interface; M620 — `Option`'s
  constructors swapped; M621 — a type variable not counted; M622 — a `Bool` read inverted: all red;
- M623 — the interpreter's sort gone: red.
Control green after each spec.

**The generator.** `random_console_program` gains a line, from a stream of its own so every other line is what each seed
always gave: `str`, `==` and `!=` of a list and an option of `Float`s — a NaN, an infinity and a negative zero rendered
and compared inside them, a list against itself as often as against another. `conformance_parity` and the differential
now compile with the checker's types (`compile_checked`), as the command line does — through `compile_module` the value
interface is never carried. `conformance_parity` asserts the line in more than 500 of its 2,000 programs, more than 150
equal, more than 150 unequal and more than 10 with a NaN inside (mutants M624 — the line never written — and M625 — a
list never compared with itself — red).

**Evidence.** `delulu-runtime`, `delulu-wasm` and `delulu-conform` whole, green (509 tests, `--no-fail-fast`); `delulu`'s
`wasm_cli`, `conformance`, `core_invariance`, `corpus_cli`, `examples_run`, `cli`, `json_contract`, `actors_cli`,
`budget_cli` and `book` green (114) — the interpreter's new field order changed no golden output there. The release
differential with the new line: 50,000 programs, all 50,000 compiled (with the checker's types), 45,027 agreed, 4,973
both faulted, 0 mismatches. By hand, a probe of every kind printed the same on both engines, the cycle and `[NaN]`
included.

## 2026-10-20 — the laptop: `narrow` on the WASM engine; the tier-5 attenuation program compiles (RW 3.4)

**`Cap[FsRead].narrow(sub)` compiles**, to the import `delulu:cap.fs_narrow(cap, path) -> cap`. The host mints the
narrower handle by the interpreter's own `prim::resolve_in_scope` — public now for it — so the two engines draw one
boundary in one set of words: a hostile spelling, a `..` or a link out of the handle's scope is DL0904, and no
narrowing reaches past the scope it starts from. Pure, as on the interpreter (`effect_for("FsRead", "narrow")` is
`None`): no trace record, no custody round-trip — the read through the narrowed handle is traced and gated as every
read is, against the narrowed scope. The import scan names `narrow`, so a module whose only filesystem operation is a
narrowing of a handle it was given still carries the filesystem imports. The corpus's entry points: **11 of 17**
compile, `tests/corpus/tier5-security/attenuation.delulu` the eleventh.

Tests:
- `narrow_parity.rs` (new): `a_narrowed_handle_reads_inside_its_subtree_on_both_engines` (two independent
  narrowings, a narrowing within a narrowing), `no_narrowing_or_read_reaches_past_the_scope_on_both_engines` (a
  narrowing back up, reads into the sibling and the parent, a narrowing to the sibling by way of `..` — DL0904 in the
  interpreter's words on both), `a_module_that_only_narrows_carries_the_filesystem_imports` (the module validated by
  wasmtime);
- `corpus_cli.rs`'s tier-5 test runs both of its grants on the WASM engine too, each with the interpreter's stdout and
  exit code (through the command line; on the old binary the WASM run was DL1201).

Mutants M626 (the narrowed path unchecked), M627 (no `narrow` arm), M628 (the scan without `narrow`), M629 (the
narrowed handle keeping the wider scope): all red, control green.

**Evidence.** `delulu-wasm` and `delulu-runtime` whole, green (484 tests); `delulu`'s `wasm_cli`, `corpus_cli`,
`conformance`, `examples_run`, `cli` and `core_invariance` green (78). By hand on the laptop: the attenuation program
and the four refusals printed and refused alike on both engines through `delulu run --grant fs.read=./data`.

## 2026-10-20 — the laptop: writing files on the WASM engine (RW 3.4)

**`root.fs_write(p)`, `write_text` and `append_text` compile**, to `delulu:cap.root_fs_write` and
`delulu:cap.fs_write_text(cap, path, body, append, file, start, end) -> Result[Unit, IoErr]`. The mint is the
interpreter's (`prim`'s root method): a hostile spelling refused, the path resolved from the working directory, granted
iff it lies on disk within a granted write subtree (`HostConfig.fs_write_roots`, filled by the command line from
`--grant fs.write=` on both of its WASM paths). The write follows the interpreter's order step for step: the dispatch
record (the relative path, where it pointed, the scope), the custody gate on the path the write will open
(GUARD-ALIAS-1: pinned and spelled as on disk), the scope by `prim::resolve_in_scope` in its words, then the write
through `beneath` (FS-RACE-1); a refusal after the record is recorded as one (RW 4.62). `push_fs_read_trace` became
`push_fs_trace`, for the three file effects. Two alignments made on the way: an ungranted `root.fs_read` on WASM now
names the grant to pass, as the interpreter's message does; and the read path's escape at open (reachable only by a
race between the check and the open) now records its refusal after the dispatch, as the interpreter does — by
reading, unwitnessed.

Chapter `05` now stops only at `.http`; the corpus count stays 11 of 17.

Tests:
- `wasm_cli.rs` `file_writes_on_the_wasm_engine_match_the_interpreter`, through the command line: a write, an append
  and a write into a directory that does not exist — the interpreter's output, the same bytes on disk, the same trace
  byte for byte; a write escaping the scope — DL0904 on both, no file, the same dispatch and refusal records; an
  ungranted mint — DL0703 on both in the interpreter's words;
- `lib.rs` `a_write_asks_the_custody_for_the_path_it_will_open`: a recording custody is asked once, about `FsWrite`,
  on the on-disk spelling — the program names the directory by another spelling (upper case on Windows, a link on
  Unix), and the test asserts it can tell the two apart; a denying custody refuses the run in its words and nothing
  is written.

Mutants:
- M630 (the scope unchecked), M631 (no dispatch record), M633 (the mint ungated), M636 (no refusal record), M634 (no
  custody gate), M635 (`FsRead` asked) — red;
- M632 — the append flag dropped: first aimed at the unpinned branch, which runs only for a file that does not exist
  yet, where appending and writing agree (an equivalent mutant, survived); aimed at the pinned branch, red;
- M637 — the custody asked about the path as written: survived while the test's directory had one spelling; red once
  the program named it by another.
Control green after each spec.

**Evidence.** `delulu-wasm` whole, green (168 tests); `delulu`'s `wasm_cli`, `corpus_cli`, `conformance`,
`examples_run`, `cli`, `json_contract`, `budget_cli` and `core_invariance` green (102). By hand: the write program, the
escape and the ungranted mint printed, wrote and refused alike on both engines, their traces byte-identical.

## 2026-10-20 — on-demand routine run 5: the laptop's #17–#19 landed; a host's egress window is its own thread's (RW 7.20)

**Landed from the laptop.** The laptop's session stopped at about 04:30 UTC with three pull requests open, each green:
#17 (`cba20d4`, `str` and `==` of compound values on WASM, RECORD-ORDER-1), #18 (`ce5f6ab`, `narrow` on WASM) and #19
(`275af08`, writing files on WASM). Their CI — `39367623331`, `39368481885` and `39369712525` — read success on every job
(`miri-slow` and `heavy-gates` skipped, as on every pull request), and so did `a7b6476`'s push run `39368506567`. `master`
was fast-forwarded `a7b6476..275af08` and pushed. On `275af08` in the VM: `survey check` ok (1543 nodes, 15500 edges);
`doctor --check` ok (26 checks); the two-engine differential (`cargo test -p delulu-wasm --release --test differential --
--ignored`, 50,000 programs — `heavy-gates`, which no pull request runs) passed in 151 s. **The autosave branch:** its eleven
snapshots since `bbb8ea3` were each compared with the commit that followed their base: six landed exactly (`c94cbba`,
`3174d05`, `fc0310c`, `301c018`, `ce67ca1`, `84eabf6`); five were mid-edit drafts that the next commit carries further
(`902e45d` — the commit adds a §11.4 lesson; `0012e9f` and `e500da6` — `in_block`'s walk, replaced in `a7b6476` by
`module_any`'s, WASM-SCAN-1; `3f5454e` — `compile_module` calls, replaced by `compile_checked`; `6d7557c` — `Ty`'s list
re-laid out by `rustfmt` around `FsWrite`). Nothing to recover; no snapshot after `275af08`.

**RW 7.20 — the confinement tests' flake, reproduced and explained.** The row said the two tests' count was a field of the
`HostChannel` each test makes "and nothing global feeds it that a read found". Something does: `HostChannel::answer` takes
`egress::mark()` on the way in and counts every refusal `egress::records_since` finds on the way out, and both read the
PROCESS-WIDE egress log — the run report's evidence, process-wide on purpose, because an actor's `http.get` runs on a
worker thread. Two tests in the same binary make refused requests (`a_guests_refused_network_request_is_in_the_hosts_denied_list_with_its_reason`,
`a_guests_request_with_a_refused_header_is_refused_by_the_host_and_recorded`); when one landed between a confinement test's
mark and its read, that test's host counted it as its guest's, and `denied().1` was 2. **Reproduced in the VM** with a frozen
copy of the pre-fix test binary, its `channel::tests::a_` filter run at `--test-threads 16`: 11 of 500 runs red, then 13
of 400 (6 at `channel.rs:1336`, 7 at `:1380` — the laptop's two lines — every one `left: 2`); the fixed binary, run side by
side under the same load, 0 of 400.

**The fix.** `egress::mark` and `records_since` read a thread-local tally of what THIS thread logged (`MINE`, bounded and
counted as the process log is; `log` writes both). The sandbox host performs a guest's request on the thread that answers
it — `LocalSink` calls the primitive table in place, and every `log` call is on the requesting thread — so the window holds
that request's refusals and no other thread's. The process-wide log, which `take_log` and `snapshot` give the run report,
is unchanged. The same rule already held for RW 4.46's `fetch_refusal`, read on the calling thread "never another
thread's". In the `delulu` command line one process hosts one guest, so no shipped run reported another's refusal; a
process hosting two guests (an embedder of `delulu-runtime`, or a test binary) could.

Tests:
- `channel.rs` `a_refusal_another_thread_logs_while_the_host_answers_is_not_the_guests`: a sink that, while performing the
  guest's call, has ANOTHER thread make a refused request — `denied().1` is 0; the control, the same refusal made on the
  answering thread — 1, naming the URL. Red on the old code (`left: 1`).
- `egress/tests.rs` `records_logged_after_a_mark_are_found_by_it`, tightened: it accepted "ours is among what the mark
  finds, or anything was refused" because other tests log concurrently; now exactly ours, with another thread's refusal
  logged beside it.

Mutants: M638 (the window read from the process log again) — red, both tests; M639 (`log` not writing the thread's tally)
— red, four tests (the two network tests' records gone from `denied[]`). Control green.

**Evidence.** `delulu-runtime`'s `channel` and `egress` tests green (70); on the runners at `18b0adf` (`witness.yml`,
`delulu-runtime`'s whole library): macOS `39388976507` 284 passed, 0 failed, the new test and the tightened one named;
Windows ``39388979829`` 283 passed, 0 failed, both tests named. The full suite alone at `18b0adf`: 2,431 passed, 0 failed, 16 ignored (173 binaries), cargo exit 0, the tree unmoved (a red-team pass loaded the VM during part of it). `scripts/clippy.sh -j 4` clean and
stamped; `cargo fmt --all` clean.

## 2026-10-20 — on-demand routine run 5 (continued): a red-team pass on the WASM engine's file capabilities; RW 4.72 — a file capability is anchored at its mint (D-V2-167)

**The pass.** The laptop's `narrow` and `fs_write` on the WASM engine (#18, #19) are security surfaces that had no
adversarial pass. One Sonnet 5.5 sous-chef was briefed to break the claim that on WASM they behave exactly as on the
interpreter and reach nothing outside the grant, against a frozen copy of the binary built from `18b0adf`'s tree in a
scratch directory outside the repository (it wrote nothing in the repository; `git status` held only this run's own
edits). About 250 cases on both engines, each compared by stdout, stderr, exit code, trace file and a dump of the disk. A
usage limit stopped it once (08:10 UTC); it was resumed by message, and its findings were already in its `FINDINGS.md`,
appended as found, as the brief asked. Each finding below was re-run by the head chef before use.

**Nothing found** (one line each in the pass's list): about 60 spellings of a write path (`..`, absolute, empty, `.`,
`//`, NUL, control and look-alike characters, names to 5,000 characters, 3,000-deep paths, `/proc/self/cwd`, `/dev/*`);
links as the last or a middle component, dangling, chained, absolute, to `/dev/null` and `/etc/passwd`, a FIFO (refused,
no hang); about 40 mint spellings and grant shapes; 18 `narrow` cases; file bodies to 200 MB and non-UTF-8; a race that
swaps a directory BELOW a handle's root for a link, 300 runs on each engine for read and write — 0 escapes, each run
ending in FS-RACE-1's refusal. `--sandbox --engine wasm` is refused in words. Not tried: a user method named `narrow` or
`read_text`, capabilities inside records or closures, `--broker daemon` and `--lease` on the WASM file path.

**F1 — fixed (`92d9119`, RW 3.18).** A read through a link whose target does not exist was refused on both engines, but on
WASM without SYMLINK-DANGLE-1's explanation, in the refusal and in its trace record — `fs_read_text` checked the scope with
a copy of `resolve_in_scope`'s rule. It also checked the scope BEFORE asking the custody, where the interpreter asks first.
Now the read follows the interpreter's order, as the write does since `275af08`: the dispatch record, the custody's
question about the path it would open, then the interpreter's own `resolve_in_scope`, in its words. Tests: `wasm_cli.rs`
`a_read_through_a_dangling_link_is_refused_in_the_interpreters_words_on_wasm` (unix; words and traces across engines) and
`delulu-wasm` `a_read_asks_the_custody_before_its_scope_as_the_interpreter_does` (a recording custody on both engines,
roots as the command line resolves them) — each red on the old host (the second's first form was red for the wrong
reason, the program's row missing `Write`; corrected and re-read red: WASM asked nothing, the interpreter once). Mutants
M640 (the old words), M641 (no refusal record), M642 (no custody question) red; control green. Runners at `92d9119`:
macOS `39393020777` and Windows `39393024452` (`delulu-wasm`'s custody tests, 3 passed each), macOS `39393029397`
(`wasm_cli`, 14 passed).

**F2 and F3 — fixed (`3cb335e`, RW 4.72, D-V2-167).** A capability's directory was resolved again at every use. With
`--grant fs.read=./scope`, a handle narrowed to `sub` — or a capability minted on `./scope/sub`, or on `./scope` itself —
followed `sub` once another process replaced it with a link to outside the grant: the pass read 31,552 of 40,000 reads
from outside on WASM and 33,114 on the interpreter, exit 0, and created a file outside on both. Re-run by the head chef:
the same in-process, deterministically (below). FS-RACE-1 refuses a link that appears BELOW a scope's root, but the root
itself was canonicalized again by `pin_fs`, by `resolve_in_scope`'s on-disk gate and by `beneath`'s check, so a swapped
root carried every check with it. F3, the granted directory itself swapped after the mint, was the documented residual
"the grant's own path" in `beneath.rs`; the same fix closes it.

The fix: `CapScope::Fs` carries `anchor`, where its root resolved when the capability was minted (`prim::mint_anchor`,
whose resolution is both the grant decision and the anchor) or narrowed (the subtree's own resolution, inside its
parent's anchor). Every use decides against the anchor and never resolves the root again — `resolve_in_anchor` (the scope,
in `resolve_in_scope`'s words), `pin_fs_at` (the path the custody is asked about and the path opened) — and the open always
goes through a path pinned inside the anchor (`pinned_in`), refusing one that no longer resolves there. The WASM host's
`FsRead`/`FsWrite` handles carry the anchor, through the same functions. The trace and the custody still name the root as
spelled; a grant that is a link when the capability is minted is followed then, as before.

Tests:
- `prim.rs` `a_capability_directory_swapped_for_a_link_after_its_mint_reaches_nothing_outside`: a narrowed read, a write
  minted on a sub-directory and a write minted on the grant, each pinned (as the interpreter and the sandbox host pin)
  and unpinned, after the swap — DL0904, nothing written outside; a narrowed handle whose directory became a link to its
  SIBLING inside the grant — refused too. Red on the old code (it read `OUTSIDE`).
- `delulu-wasm` `a_narrowed_directory_swapped_for_a_link_reads_nothing_outside`: a custody swaps the directory when asked
  about the read before, so the narrowed read follows the swap deterministically — DL0904. Red on the old code (it
  printed `OUTSIDE`).

Mutants: M643 (the pin re-resolving the root) and M644 (the scope's gate re-resolving it) each survive alone — two walls,
by design: the other refuses; M645 (both) red, and M647 (both, read by the WASM witness) red; M646 (a narrowing anchored at
its parent's anchor rather than its own) red, by the sibling case. Control green. `scripts/clippy.sh -j 4` clean and
stamped; `scripts/check-other-os.sh x86_64-pc-windows-msvc` clean. Runners at `3cb335e`: macOS `39394517196` (`delulu-runtime`'s library, 285 passed) and `39394523755` (`delulu-wasm`'s library, 98 passed), Windows `39394520536` (284), `39394527710` (93) and `39394530798` (`wasm_cli`, `corpus_cli`, `examples_run`, 22) — 0 failed, both new tests named and run on each OS. The full suite alone
at `3cb335e`: 2,435 passed, 0 failed, 16 ignored (173 binaries), cargo exit 0, the tree unmoved.

**F4 — open (RW 3.19).** `delulu run FILE.dwx` accepts `--trace-effects`, `--trace-out` and `--assert-trace` and applies
none: since `275af08` the `.dwx` path is given the grant's write roots, so a write-capable artifact writes with no effect
trace and no word about it. Its scope refusals hold (DL0904, DL0703; nothing written outside).

**Oddities, not defects:** every runtime refusal is framed three ways (the interpreter with a source snippet, `--engine
wasm` as "WASM engine: WASM trap: …", a `.dwx` as "`FILE`: WASM trap: …") with the same code and words; `Cap[Console].print`
is DL1201 on WASM (`println` compiles) — RW 3.4's subset.

## 2026-10-20 — on-demand routine run 5 (continued): a `.dwx` run refuses every flag it does not apply — DWX-FLAGS-1 (RW 3.19)

**Found.** The red-team pass's F4: `delulu run FILE.dwx` accepted `--trace-effects`, `--trace-out` and `--assert-trace` and
applied none — no trace file, no record, no word. The head chef's probe of the same path, against the binary at `3cb335e`,
found it worse: `--broker daemon` with no broker running, and `--lease not-a-token`, each ran the artifact and wrote its file
under the command line's grants, with no custody at all. `cmd_run_inner` dispatches a `.dwx` to `run_dwx_artifact` before the
broker, the lease and the trace are set up, and that function read the grants, `--json`, the seed and the clock — nothing
else; since `275af08` it is given the grant's write roots too. The sandboxed path had the same shape once and was closed by
an allowlist (`guest.rs`, `APPLIED_UNDER_SANDBOX`, PS-B-05); the `.dwx` path never got one.

**Fixed (`a7e50d6`).** `run_dwx_artifact` refuses, before anything runs, every flag outside `APPLIED_TO_A_DWX` — the grants,
`--json`, `--no-prompt`, `--seed`, `--clock`, and what `cmd_run` applies for every path (`--limits`, `--report-out`,
`--break-glass`, `--isolation`, `--sandbox=off`) — and `--engine` unless it says `wasm`: exit 2, in the sandbox's words ("a
flag that is silently ignored reads exactly like a flag that was applied"). An allowlist, so a flag added to `run` later is
refused here until this path applies it. Giving the path a trace and custody is the alternative the row named; refusing is
the honest state until then.

Test: `wasm_cli.rs` `a_dwx_run_refuses_the_flags_it_does_not_apply` — each of `--trace-effects`, `--trace-out`,
`--assert-trace`, `--broker daemon`, `--lease`, `--engine interp`: exit 2, the flag named, nothing written; the applied flags
still run the artifact and write. Red on the old path (exit 0, the file written). Mutants M648 (`--broker` and `--lease`
allowed), M649 (the trace flags allowed), M650 (`--engine` unchecked) red; control green.

**Evidence.** `scripts/clippy.sh -j 4` clean and stamped; `artifact_review`, `json_contract`, `package_run`, `cli_contract`,
`plugin_cli`, `signing_cli`, `pqc_cli`, `within_cli`, `unreadable_paths`, `completions_cli`, `wasm_cli` and `message_spacing`
green. Runners at `a7e50d6`: Windows `39396317161` (`wasm_cli`, filter `dwx`: 5 passed, the new test among them) and macOS `39396321250` (the same filter, success). The full suite alone at `a7e50d6`: 2,436 passed, 0 failed, 16 ignored (173 binaries), cargo exit 0, the tree unmoved.

**RW 4.72, end to end.** The red-team pass's own race scripts, re-run against the binary built from `a7e50d6`: the narrowed
read with the directory swapped at 0.25, 0.3 and 0.4 s, the write minted on a sub-directory at 0.25 and 0.5 s, and the
granted directory itself swapped (three runs) — on both engines, every run refused with DL0904 (exit 1), nothing read from
or written outside. Before the fix the same scripts read 25,707–33,114 of 40,000 reads from outside and wrote outside on
both engines.

## 2026-10-20 — on-demand routine run 5 (close): `launch/2.0.0` at `20666c5`; the run's CI

**`launch/2.0.0`** merged `master` twice this run: `3ee1370` (`d259cf2..7355808` — the laptop's #16–#19 and RW 7.20) and
`20666c5` (`7355808..ec4bc87` — F1, RW 4.72 with D-V2-167, RW 3.19). Each time `master`'s `## Unreleased` CHANGELOG sections moved
under `## [2.0.0]` as `###` entries (five, then three), and the second time the `Breaking` paragraph gained `run FILE.dwx`'s
refusal of the flags it does not apply (a command line that ran under 1.x is refused now). On `20666c5`'s tree: survey build 1545 nodes, 15597 edges, 0 errors, 14 warnings, check ok; `doctor --check` ok (26 checks); `scripts/clippy.sh -j 4` clean and stamped; doctor_cli, repository_structure, evidence_claims, governance, message_spacing, release, distribution and book green.
`3ee1370`'s release dry run `39392801546` success; its `everything` run `39392798450` success on every job but the two
`miri-slow` jobs, still running at this entry. `20666c5`'s `everything` and release dry run are dispatched.

## 2026-10-20 — on-demand routine run 5 (postscript): two of the pass's untried families, probed; the run's CI

**Under a live broker daemon** (`delulu broker start` with its state in `/tmp`, stopped with `broker stop`): a handle
narrowed to `sub` reads its file on both engines (`--broker daemon`, `--engine wasm` or not). The swap race of RW 4.72 — `sub`
replaced by a link to outside 0.2 s and 0.4 s into a 200,000-read loop — refused at the swap on both engines (DL0904; the
interpreter in `resolve_in_scope`'s words, WASM at the open's walk, "a component became a symbolic link … after the scope
check"), nothing read from outside. By hand, against the binary built from `318b210`; not a committed test.

**A capability held in a closure** carries its narrowed scope on both engines (the file inside read, `../f.txt` refused
DL0904 on both). **A capability held in a record field** runs on the interpreter (inside read, `../../outside/f.txt`
refused DL0904) and is DL1201 on WASM ("a record with a field of a type outside the WASM fragment") — RW 3.4's subset,
refused rather than run. Still untried: a user method named `narrow` or `read_text`, and `--lease` on the WASM file path.

**CI read.** push runs `39387600500` (`275af08`), `39390507228` (`7355808`), `39396073953` (`669664c` — F1 and RW 4.72 on all three operating systems) and `39397818042` (`ec4bc87` — RW 3.19) success on every job; the nightly `39391181701` (`7355808`) and `launch/2.0.0`'s `everything` `39392798450` (`3ee1370`) success on every job but their two `miri-slow` jobs, running at this entry; `20666c5`'s release dry run `39398473807` success and its `everything` `39398467836` running; `318b210`'s push run `39398823386` running.

**`--lease` on the WASM file path** (the last of the pass's untried families but one; by hand, a live broker, a fresh
single-use token per run from `grants delegate --effects Read --fs-read ./data/pub`): a read inside the lease ran on both
engines; a read through a narrowed handle reaching `../priv/f.txt` was refused at the lease's own custody gate on both, in
the same words ("lease `g_…` does not grant `…/data/priv/f.txt` in scope `fs.read`") — the WASM read asks the custody
before its scope since `92d9119`, so before F1's fix WASM would have answered with the scope's DL0904 instead; a mint of
`./data/priv` was DL0703 on both. **An oddity, both engines:** that DL0703 still says "pass `--grant fs.read=./data/priv`",
which a `--lease` run refuses ("a `--lease` run derives its authority from the delegated node") — the hint points a lease
holder outside its delegation, as PS-B-05 found under `--sandbox`. Untried still: a user method named `narrow`/`read_text`.
**CI read, last:** `318b210`'s push run `39398823386` and `8ac7ee5`'s `39400706197` success on every job.

## 2026-10-20 — on-demand routine run 5: stopped on the owner's word

The owner asked at 09:50 UTC for a safe stop. Nothing was half-done: every slice of the run (the laptop's #17–#19, RW 7.20,
F1, RW 4.72 with D-V2-167, RW 3.19, two merges into `launch/2.0.0`) was already verified, recorded and on `master`. At the stop
no job had failed; four runs were still going (`6a03165`'s push run, and `miri-slow` jobs on `launch/2.0.0` and the nightly) —
named in `docs/CLOUD_SYNC_LOG.md` for the next session to read.

## 2026-10-21 — the laptop: a lease run's DL0703 names its delegation, not a `--grant` it refuses (RW 6.15)

**Synced first.** `git fetch origin`: `master` `a7b6476..9c1d30b` (on-demand routine run 5 — the laptop's #17–#19, RW 7.20,
F1, RW 4.72 with D-V2-167, RW 3.19, two merges into `launch/2.0.0`), fast-forwarded. The four runs run 5 left going, read
to their end: `6a03165`'s push run `39403469304`, `launch/2.0.0`'s `everything` `39398467836` (`20666c5`) and `39392798450`
(`3ee1370`), and the nightly `39391181701` (`7355808`) — success on every job, `miri-slow` included; so are `e32b3c7`'s
`everything` `39364184892` and `9c1d30b`'s push run `39403663796`. The autosave branch's newest snapshot (`6d7557c`, 04:23
UTC on 2026-10-20) is older than `275af08`, which carries it — nothing to recover. On `9c1d30b`: `survey check` ok (1544
nodes, 15586 edges), `doctor --check` ok (29 checks on this machine).

**The defect** (run 5's oddity, reproduced on `9c1d30b`'s binary with a live broker and a `--multi` lease of
`--effects Read --fs-read ./data/pub`): a program minting `root.fs_read("./data/priv")` was refused DL0703 "filesystem read
of `./data/priv` was not granted — pass `--grant fs.read=./data/priv`" on both engines, and `root.console()` "pass `--grant
console`" — while the same run refuses every local `--grant` but a foreign library's path ("a `--lease` run derives its
authority from the delegated node"). The refusal sent the lease's holder outside its delegation. Under `--sandbox --lease`
the guest's refusal said the same. Beside it, the WASM engine's ungranted `console`, `clock` and `rand` named no remedy at
all ("DL0703: console was not granted to this program"), where the interpreter's say `pass --grant console`.

**The fix.** A run's grants say where they came from: `Grants::lease` (set by `grants_from_lease` to the redeemed node),
carried into `RootVal::lease` by `build_root`, across an actor boundary in `RootMsg`, and into the WASM host as
`HostConfig::lease`. Not authority — nothing decides a use by it. One function, `prim::grant_remedy`, words the remedy for
every mint on both engines: `pass --grant G` on an ordinary run; under a lease, "the lease this run redeemed (node `g_…`)
does not carry `G`, and a `--lease` run takes its authority from its delegation alone (a local `--grant` is refused)". The
twelve interpreter mints that answer DL0703 (console, fs_read, fs_write, http, clock, rand, declassify, secret, actuator,
sensor, compute, plugin_host) use it; an ordinary run's words are unchanged, each one. The WASM host's console, clock, rand,
fs_read and fs_write refusals now carry the interpreter's words exactly. The sandbox host serves its guest's mints from the
same root, so `--sandbox --lease` is covered without a change of its own (checked by hand: the guest's refusal names the
node).

**Evidence.** By hand on the fixed binary, a live broker: under `--lease` the interpreter, `--engine wasm` and `--sandbox`
each name the node and `does not carry fs.read=./data/priv` / `console`; without a lease each still says `pass --grant …`.
Witnesses: `prim.rs` `a_lease_runs_refusal_names_its_delegation_and_never_a_grant_it_refuses` (the twelve mints, both kinds
of root); `fault_parity.rs` `an_ungranted_mint_names_the_same_remedy_on_both_engines` (five mints × with and without a
lease, the WASM refusal ending in the interpreter's exact message); `grants_cli.rs`
`a_lease_runs_refusal_names_its_delegation_on_both_engines` (a live broker, both engines, and an ordinary run beside them);
`actors.rs` `a_roots_lease_survives_the_crossing_into_an_actor_and_back`. Mutants (`scripts/mutants.py`, one target each):
M651 `grant_remedy` ignores the lease → the unit test red; M652 `grants_from_lease` sets no lease → the end-to-end test red
(the old code's words on both engines); M653 the CLI's `HostConfig` drops it → the end-to-end test red (the WASM half);
M654 the WASM host state drops it, M657 the WASM console refusal back to its old words, M658 `build_root` drops it → the
parity test red; M655 and M656 either direction of the actor crossing drops it → the actor test red; the control green.
`scripts/clippy.sh -j 4` clean and stamped. `delulu-runtime` and `delulu-wasm` whole: 492 passed, 0 failed (27 targets); the 26 `delulu` CLI targets that assert a refusal's words (`cli`, `grants_cli`, `wasm_cli`, the sandbox, broker, budget, device and plugin files among them): 286 passed, 0 failed. The rest runs on GitHub (the pull request's CI).

## 2026-10-21 — the laptop: a behaviour named like a capability method is a send on WASM (RW 3.20, WASM-SEND-NAME-1)

**Found** probing the red-team pass's last untried family, "a user method named `narrow`/`read_text`" (`V2_LOG.md`,
2026-10-20). The language has no user methods on records; the only method a program names is an actor's behaviour, sent as
`w.job(n)`. So: an actor with one behaviour per capability-method name, sent from `main`. On the interpreter each ran. On
`--engine wasm` every name the codegen matched ahead of the receiver with the same argument count was refused — `w.narrow(5)`
DL1201 "narrow() on a receiver that is not a Cap[FsRead]", `w.int(2, 3)` "int(lo, hi) on a non-Rand receiver", `w.console()`
"console() on a non-Root receiver"; a behaviour named like one with another count (`w.len(5)`, `w.now_ms(5)`) ran. The
codegen's method arm tried fifteen names (`console`, `clock`, `now_ms`, `rand`, `println`, `int`, `fs_read`, `read_text`,
`narrow`, `fs_write`, `write_text`, `append_text`, `foreign_load`, `foreign`, `expose` — `expose` refused as "secret
contents cannot enter WASM guest memory") before it compiled the receiver and found an actor. It failed closed: nothing ran
as a capability, and an arm that found the wrong receiver refused. But a valid program was refused, for a reason it does
not have. `secret`, the sixteenth name, is a reserved word (DL0106), so no behaviour can carry it.

**The fix.** The receiver decides first: `recv_actor` reads from the scopes, without compiling anything, whether the
receiver is a local or parameter of actor-reference type, `self` in an actor member, or a `self.field` holding a reference —
the forms `compile_expr` compiles to `Ty::ActorRef`; such a method is a send whatever its name. Any other receiver is
compiled and dispatched as before (a `spawn W()` receiver still reaches the send by its compiled type). Peeking by compiling
the receiver and rolling back was rejected: each level of a method chain would compile its receiver twice.

**Evidence.** Witness `actors_parity.rs` `a_behaviour_named_like_a_capability_method_is_a_send_on_both_engines`: the
fifteen names as behaviours (none, one or two `Int` arguments), each sent from `main`; the one- and two-argument ones count
down by sending to `self` under their own name; and a second actor sends `narrow` through a `tag W` field. The two engines'
quiescence reports agree, and the WASM turn count is the arithmetic's, 55. Red on the old code (`Unsupported("console() on a
non-Root receiver")`). Mutants: M659 no receiver read → red; M660 `self` not read → red (the self-sends); M661 the field not
read → red (the relay); M662 a local not read → red; the control green. By hand on the fixed binary: every probe program
runs on `--engine wasm`. `scripts/clippy.sh -j 4` clean and stamped; `delulu-wasm` whole: 172 passed, 0 failed (17 targets).

## 2026-10-21 — the laptop: three new advisories, fixed by patch updates (Wasmtime 48.0.7, rustls-webpki 0.103.16); rustfmt

**Found by** pull request #20's CI `39555991114`: `supply-chain` red on three advisories that did not exist at `9c1d30b`'s
green push run the day before (cargo-deny's database moved, not the code), and `lints` red on `cargo fmt --check` for
`prim.rs` and `grants_cli.rs` (the laptop ran clippy and not rustfmt on the slice's new code — `scripts/clippy.sh` does not
format; run `cargo fmt --all` with it).

**What each reaches, read before updating.**
- RUSTSEC-2026-0379 (Cranelift on aarch64 can jump to the wrong target on a large `br_table`): the codegen emits no
  `br_table`, but Wasmtime compiles whatever module it is handed. `delulu run FILE.dwx` and a `.dpx` plugin hand it a
  module someone else built, and its code hash binds only to a manifest its builder wrote. So on the two aarch64 release
  targets (`aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`) a crafted artifact could reach the miscompile.
  **The one that matters.**
- RUSTSEC-2026-0382 (rustls-webpki may accept a path through the distrusted emSign intermediates): the egress client's
  certificate verification (`rustls` under `reqwest` and `delulu-runtime`).
- RUSTSEC-2026-0378 (a panic lowering a host-created component-model stream): the WASM engine runs core modules only — no
  `wasmtime::component` anywhere in `delulu-wasm`. Not reached; fixed by the same update anyway.

**The update.** `cargo update -p rustls-webpki` (0.103.15 → 0.103.16) and `cargo update -p wasmtime` (48.0.6 → 48.0.7, and
the packages released with it, twenty-seven: `cranelift-*` 0.135.7, `pulley-*` and `wasmtime-internal-*` 48.0.7). No other crate moved,
none added or removed. 49.x was not taken: a major version is its own slice.

**Evidence.** Locally `cargo deny check advisories`: `advisories ok`, and `check bans licenses sources`: all three ok
(cargo-deny 0.20.2). On the updated lock: `delulu-wasm` whole, 172 passed, 0 failed; `egress_cli` 8, `wasm_cli` 14,
`examples_run` 6 — all passed; `delulu-runtime --lib egress` 41 passed.
`cargo fmt --all` changed exactly the two files CI named. clippy clean and stamped. The pull request's CI is the rest.

## 2026-10-21 — the laptop: the network on the WASM engine — `root.http` and `Cap[Http].get` (RW 3.4); chapter five runs

**Why this, now.** `http` was chapter five's last blocker on WASM: with its two network lines removed, the chapter compiled
already (checked first, against `9c1d30b`'s binary — the uncalled `Secret` function does not stop it). And the network is
the effect where a second implementation would cost most: the allowlist's path scopes (D-V2-139), the special-use rule
(PS-B-02), redirects, the custody's chain (RW 4.46) — each written once, in the interpreter's code.

**So the WASM host calls that code, and copies none of it.**
- `root.http(hosts)`: the guest passes the list and its type descriptor (`desc_ptr`, #17's), and the host rebuilds the
  `List[Str]` by the reifier.
- It mints by `prim::call_root_method` over a root holding only the grants' `net`, `net_special` and `lease`. So each
  requested entry must lie within a granted one, the `net.special=` subset is carried, and a refusal is the interpreter's
  DL0703, remedy included (RW 6.15).
- `Cap[Http].get(url)` follows the interpreter's order (`Interp::call_method`):
  1. the dispatch record (the URL only, D-V2-128);
  2. the custody on the URL's host — except for a URL that is not `https://`, which the effect answers `Refused` before
     anything is reached (HTTP-SCHEME-1), so nothing is asked;
  3. `egress::begin_fetch`, then `prim::call_cap_method`, the allowlist and the egress client entire;
  4. an egress refusal after the custody allowed the host is said to the custody (`note_egress_refusal`);
  5. a fault's refusal record follows its dispatch (RW 4.62).
- The answer is built as `Result[Str, NetErr]` in the guest — `NetErr`'s cells are `IoErr`'s shape.
- The codegen:
  - `Ty::Http` (a `Cap[Http]` parameter lowers to it).
  - The two imports, gated on the mint alone: a `Cap[Http]` comes from nowhere else, and `get` is also a list's method,
    so a module whose only `get` is on a list imports no network interface.
  - `get` is dispatched on the receiver's type, after it is compiled.
  - `get_with` and `get_with_secret` stay DL1201.
- The CLI hands the grants' `net` and `net_special` to both WASM paths (the `.dwx` one included).

**Evidence.**
- Chapter five on both engines, by hand, with its grants and `--trace-effects`: the same output, and the same trace byte
  for byte, the real request to `https://example.com/health` included (answered 404 → "fetch failed" on both).
- `wasm_cli.rs` `http_on_the_wasm_engine_matches_the_interpreter`, on loopback only:
  - a scheme refusal (`Refused`, report reason `scheme`);
  - a request under `net.special=127.0.0.1` that leaves as TLS — the listener, shared by both engines' runs, sees a
    handshake record from each — to a server that hangs up (`Other`);
  - a host outside the allowlist (DL0904, its `get.refused` record);
  - an ungranted mint (DL0703 in the interpreter's words);
  - checked engine against engine: output, outcome, trace, and the report's egress reasons.
- `examples_run.rs`'s chapter-five test runs on both engines (the network line may be either answer, as before).
- `fault_parity.rs`'s remedy test gains the `http` mint.
- `lib.rs` `a_fetch_asks_the_custody_for_its_host_first_and_tells_it_of_a_refusal`, with a recording custody:
  - `Console`, then `Net 127.0.0.1` — nothing for the `http://` URL;
  - a denial is the run's refusal in the custody's words;
  - allowed, the egress client's refusal reaches the custody. This crate builds the runtime without its client, so the
    reason here is `no-client`; in the command line's build it is `special-use`.
- The corpus counter: **12 of 17** entry points compile, all six guide chapters (`05` the twelfth). The rest stop at:
  `demo` (DL1205, a secret in the guest), Python, a plugin, and two packages' other modules.
- Mutants (`scripts/mutants.py`, one target each), each red, and the control green:
  - M663 no import gate → the CLI test (DL1201);
  - M664 no dispatch record → the CLI test (the traces);
  - M665 the custody never asked → the unit test;
  - M666 asked for every URL → the unit test (an `http://` URL asked);
  - M667 no refusal record after the effect's fault → the CLI test;
  - M668 the mint over a root that grants every host → the CLI test (the DL0703);
  - M669 `Refused` built as `Timeout` → the CLI test;
  - M670 the egress refusal not told → the unit test;
  - M671 the mint's root without the lease → the remedy test.
- `scripts/clippy.sh -j 4` (rustfmt's check included) clean and stamped.
- `delulu-wasm` whole: 173 passed, 0 failed.
- `wasm_cli`, `examples_run`, `egress_cli`, `cli_contract`, `cli`, `corpus_cli`, `budget_cli`: 113 passed, 0 failed.

## 2026-10-21 — the laptop: a sealed artifact runs under a lease, the broker daemon and a trace (RW 3.19's rest)

**Why.** DWX-FLAGS-1 (RW 3.19, run 5) made `run FILE.dwx` refuse `--lease`, `--broker` and the trace flags it had ignored,
and left "giving the path a trace and custody" open. But running code someone else built, under a delegation, is what a
sealed artifact is for — and the dangerous way to give it custody is a second copy of the source route's.

**The change.** The source route's custody and trace code became functions, and both routes call them:
- `lease_for_run`: the redemption with its budget hold (PS-B-05) and break-glass binding (D-V2-116).
- `custody_for_run` and `RunCustody`:
  - the custody label;
  - a `--broker daemon` root issued and held (RW 4.57), or the lease's mints dead-manned;
  - `_run_node` still bound in the source route where the local was, so it drops after the devices.
- `wasm_trace_sink`, `emit_wasm_trace`, and `assert_wasm_trace`, against a row or a set.

The `.dwx` route then does what the source route does, in its order: redeem, choose the custody, refuse a runtime file in
a scope the program can write (D-V2-21 — a lease's scopes are known only after redemption), and run with the custody and
the sink. Its `--assert-trace` judges the trace against the effects the artifact's manifest declares.
`APPLIED_TO_A_DWX` gains `--broker`, `--epoch-ms`, `--lease`, `--trace-effects`, `--trace-out` and `--assert-trace`.
Every other flag is still refused (`--debug-rcaps`, `--foreign-isolation`, `--on-quiesce` and `--engine interp` in the
witness).

**Evidence.**
- The extraction first, before the `.dwx` route used it: 13 lease and daemon CLI targets, 144 passed, 0 failed (`grants_cli`,
  `broker_cli`, `budget_cli`, `guard_e2e`, `device_delegation_cli`, `plugin_load_cli`, `sandbox_guard_e2e`, `monitor_cli`,
  `federation_cli`, `cli_contract`, `wasm_cli`, `dead_man_cli`, `estop_cli`).
- `grants_cli.rs` `a_sealed_artifact_runs_under_a_lease_and_the_daemon_with_its_trace`, a live broker:
  - Under a single-use lease of `./data`, the artifact reads both files with daemon custody and writes its four-record
    trace. A second use of the token is DL1407, and runs nothing.
  - A lease of `./data/pub` refuses the artifact's mint of `./data` in the lease's words (RW 6.15).
  - An artifact reading `../priv/f.txt` through a `./data/pub` handle is refused by the lease's custody ("lease `g_…` does
    not grant"), which the host asks before its own scope check — so the words say the custody decided.
  - A lease whose write scope holds `--trace-out` is refused (exit 2, nothing run).
  - Under `--broker daemon` the run's root is issued, armed and revoked (the audit chain's last records).
- `wasm_cli.rs` `a_dwx_run_refuses_the_flags_it_does_not_apply`: the four flags still refused, and the applied run writes its
  trace and passes `--assert-trace`. An artifact whose manifest says `Clock` where it performs `Write` is DL1101 under it.
- Mutants, each red, the control green:
  - M672 the lease skipped;
  - M673 the custody not handed to the host — the escape refused in the scope's words instead;
  - M674 the trace not handed to it;
  - M675 the assertion skipped;
  - M676 the write-scope refusal skipped.

**Found while making the last witness: ARTIFACT-CLAIM-1** — next entry.

## 2026-10-21 — the laptop: ARTIFACT-CLAIM-1 — a `.dwx`'s declared effects were its builder's word; now they are a ceiling `run` enforces (RW 4.73)

**Found** while writing the `--assert-trace` witness of the previous entry: to make an artifact whose trace leaves its
declared effects, the laptop changed `"Write"` to `"Clock"` in a built `.dwx`'s `delulu:authority` section — five bytes for
five, no hash recomputed. It verified, and it ran. The run said:

> running `lie.dwx` — authority verified; declared effects: Read, Clock

and it read, printed and wrote. `code_blake3` hashes the bare module with the authority section excluded (`embed_authority`,
`read_and_verify`), so it binds the code to the manifest's own `code_blake3` field and to nothing else in the manifest.

Three claims were wrong:
- the book's "the authority claim and the code it describes cannot drift apart" (Chapter 9);
- the run banner's "authority verified";
- `delulu authority x.dwx`'s "effects: (none — provably pure)" for an artifact that only says so.

**Severity: LOW — words, not authority.** Every effect was still refused unless its grant covered it: the host mints and
performs everything under the run's grants, and the custody. So nothing escaped. But an operator chooses those grants from
this report, and a review surface that a builder can edit with a hex editor is the C65 class of failure (the review surface
vouching for what it never checked).

**The fix: enforce the claim, don't sign it.** A signature would say *who* built the artifact, not that its claim is true.
The run-time ceiling makes the claim true of what can run:
- `HostConfig::declared_effects` — the `.dwx` route passes the manifest's effects; the source route passes `None`, since
  there the checker computed the row.
- `HostState::custody_allows` — every effect the WASM host performs passes it, after its dispatch record. It now refuses an
  effect whose required effect (`Op::required_effect`, the grant tree's own rule) the manifest does not declare: DL1202,
  "the artifact performed `Write`, an effect its `delulu:authority` manifest does not declare (it declares Clock, Read)",
  before the custody is asked and before the effect happens. The trace shows the dispatch, then its refusal.
- Over-declaring stays harmless.
- No new code: DL1202 is the artifact refusal, and its explanation now says this half too.
- `delulu authority x.dwx` gains an `enforced:` line saying the effects are the builder's claim and that `run` holds the
  artifact to them.
- The book's sentence is corrected in place, the false version quoted.

**And a mislabel it exposed: WASM-CODE-1.** `wasm_fault_code` labelled a WASM refusal by a list of nine codes, so DL1202
printed as `error[DL0904]` — and so would a custody's guard refusal (DL1410) or seal (DL1413), whose messages carry their
codes. It now returns the message's first registered code (`delulu_diag::static_code`), or DL0904 when there is none.

**Evidence.**
- Witnesses, red on the old code:
  - `lib.rs` `an_artifact_runs_nothing_its_manifest_does_not_declare`: no ceiling and `{Write}` run; `{Read, Clock}` and `{}`
    are refused DL1202;
  - `wasm_cli.rs` `a_dwx_run_refuses_the_flags_it_does_not_apply`, the doctored artifact: `error[DL1202]`, exit 1, nothing
    written, nothing printed; under `--assert-trace`, exit 3 with DL1101 as well;
  - `cli.rs` `a_wasm_refusal_is_labelled_with_the_code_it_carries`: nine messages, DL1410, DL1413 and DL1202 among them.
- Mutants, each red, the control green:
  - M677 the ceiling never consulted;
  - M678 the `.dwx` route passes none;
  - M679 the host state drops it;
  - M680 and M681 the label back to a short list (the unit test and the CLI witness).
- By hand: the honest `r.dwx` runs as before; the doctored one stops at its first `Write`, after a declared `Read`.
- The whole of `delulu`, `delulu-wasm` and `delulu-diag` on this tree: 1,231 passed, 13 ignored, 3 failed in 120 targets
  — the three `doctor_cli` tests that read the Survey's freshness, red because the map was built before the edits (as
  they must be). After the rebuild: see the commit message.
- `scripts/clippy.sh -j 4` first refused to stamp: rustfmt's check, added this morning, caught the slice's unformatted code
  before any commit could carry it.

## 2026-10-21 — the laptop: DWX-ACTORS-1 — an actor program's `.dwx` was written and could not run; refused in words (RW 3.21)

**Found** probing the `.dwx` route after giving it custody (the entry before last). `delulu build
examples/guide/06_actors.delulu --target wasm` wrote a 1,523-byte artifact. `delulu run` of it failed: "WASM trap: actor
index 1 out of range", labelled DL0904. The source of the same chapter runs on `--engine wasm` (5 actors, 13 turns).

The `.dwx` route calls `run_main` with no actor table. The source route builds the table from the AST (`actor_table`):
each actor's behaviours, their arities, and its field count. The exports name the members (`actor$W$new`, `actor$W$job`)
but carry no field count.

**Refused, not made to run — this slice.** Making it run means the code carrying its own table: each field count as an
exported global, read from the module. It must never come from an unhashed custom section; that is ARTIFACT-CLAIM-1's
lesson, the entry before. That is a format change, a slice of its own (RW 3.21's "What closing it takes"). Until then:
- `build --target wasm` refuses a program that declares actors (DL1201, `dwx_actors_refusal`), and writes nothing;
- `run` refuses an artifact whose exports name an actor member (`delulu_wasm::exports_actor_members`, a read of the
  export section), in the same words, before anything runs.

**Evidence.**
- `wasm_cli.rs` `an_actor_program_has_no_dwx_yet_and_says_so_at_build_and_at_run`: the build refused, with no file
  written; an artifact assembled through the library, as the old build wrote it, refused at run with no "out of range".
  Red on the old code.
- Mutants, each red, the control green:
  - M682 the build's refusal removed;
  - M683 the run's;
  - M684 the scan's prefix;
  - M685 the scan reading the wrong section.
- The five CLI targets that build artifacts (`wasm_cli`, `release`, `plugin_cli`, `artifact_review`, `json_contract`):
  51 passed.
- Chapter five and the other non-actor programs still build.

## 2026-10-21 — the laptop's close: four pull requests landed; `launch/2.0.0` merged twice

Landed today by fast-forward, each on its own CI read green to the end:
- #20: RW 6.15, RW 3.20, the three advisories, rustfmt and `scripts/clippy.sh`'s format check;
- #21: the network on WASM;
- #22: a `.dwx` under custody and a trace; ARTIFACT-CLAIM-1; WASM-CODE-1;
- #23: DWX-ACTORS-1.

`launch/2.0.0` took them in two merges:
- `6863bf7`: release dry run `39562463148` success; `everything` `39562457688` success on every job but two `miri-slow`
  jobs still running.
- `5d5ba5a`: its CHANGELOG's Breaking paragraph says what an artifact is now refused (DL1202 at an undeclared effect; an
  actor program at the build). The checklist's supply-chain criterion is re-cited, since the advisory database moved
  under a fixed tree this morning. `everything` `39566831246` and the release dry run `39566838279` dispatched.

Nothing is tagged; the launch waits for the owner's word. `docs/CLOUD_SYNC_LOG.md`'s closing entry lists the runs to read
and the open items, in order.

## 2026-10-21 (evening) — the laptop: EPOCH-WINDOW-1 — a broker test timed the daemon's shutdown inside a 250 ms window (RW 7.21)

**The red.** `master`'s push run `39569868995` on `3af2b2a` (a records-only commit; its pull request's run `39567010105`
green on every job) failed one test on `test (windows-latest)`: `brokerd::tests::epoch_class_over_the_wire_uses_the_cached_snapshot`
— "fresh cache answers within the epoch window" (`brokerd.rs:2921`), 187 of 188 passing. Every other job of the run
succeeded. Never seen before.

**The mechanism, read before reproducing.** The test refreshed a client's epoch cache (250 ms, the longest window
`clamp_epoch_ms` allows), then `stop_daemon` (the `Shutdown` round trip, the handler loop ending, a connection to wake the
accept thread, the thread's join), then expected the still-fresh cache to answer. The whole shutdown stood inside the window.
On a 4-vCPU runner under the suite it can outlast 250 ms; the cache was then stale, the refresh met a dead broker, and the
custody failed closed — DL1401, exactly the behaviour the rest of the test asserts.

**Reproduced deterministically.** A 300 ms sleep added to `stop_daemon` turns the old test red with CI's message at CI's line.
Statistically it did not show: 48 runs at four at a time and 48 more beside six CPU-bound loops, the frozen pre-fix test
binary, 0 red — the laptop has 16 cores; the runner has 4 (`docs/assistant-memory/delulu-timing-tests.md`: starve, don't
guess; here the shape was readable and the sleep is the starvation).

**The fix: nothing slow inside the window.** The window now holds one revoke request:
- After the refresh, the node is revoked. A check inside the window still answers Allow: a round trip would have seen the
  revocation, so this witnesses the cache better than a dead broker did.
- Once the window lapses, the refresh sees the revocation (DL1403).
- The "inside the window" half counts only when the test reached it in time, measured from before the refresh (the cache is
  stamped after it, so the measure is conservative). An attempt that did not is retried with a fresh node, up to five. If none
  does, the test says the runner was too slow, with the slowest time.
- The dead-broker DL1401 stays, after the window, where no timing decides it.

**Evidence.**
- The new test is green under the 300 ms shutdown.
- Mutants on the client's cache, each red, the control green:
  - M686 never fresh — the inside-window Allow became DL1403;
  - M687 never stale;
  - M689 the double: the refresh error ignored and the stale cache kept.
- M688 (the error ignored alone) survives by design: `refresh_cache` drops the stale cache first, and the next guard answers
  DL1401 for an empty one — two walls, read before recording the survivor.
- `delulu`'s whole unit binary: 188 passed. clippy with rustfmt's check clean.

## 2026-10-21 (evening) — the laptop: an actor program's `.dwx` runs — the module carries its own actor table (RW 3.21 closed)

**From the refusal to the run.** DWX-ACTORS-1 was refused in words this afternoon (`a2fb0cd`): the `.dwx` route had no actor
table, and the module did not carry each actor's field count.

**The module now carries its table, as code.** The codegen exports, beside each actor's members, an immutable `i32` global
`actor$A$#fields` holding its field count; `#` spells no identifier, so no behaviour's export can collide with it. The host
reads the table from the module alone, which its code hash covers — never from a custom section, ARTIFACT-CLAIM-1's lesson
the same day (`actors::table_from_module`):
- the members are the `actor$A$new` and `actor$A$beh` exports, in the export section's order. The codegen writes each
  actor's constructor and then its behaviours, actors in declaration order, which is how `actor_table` numbers them, so
  actor and behaviour indices agree.
- each arity is the function type's, less the self slot.
- a field-count export must be an immutable `i32` global, or the module is refused.
- the counts are read once the module is instantiated, before `main` runs, so before any spawn sizes a slot
  (`field_counts`, bounded at 65,536).

`run_artifact` compiles the module once: with no actor member it runs `main` as before; with members, under the cooperative
scheduler (`run_actors_in`, shared with `run_main_actors`). The `.dwx` route uses it, prints the engine's actor label and a
death count as the source route does, and applies `--on-quiesce report` (`APPLIED_TO_A_DWX`; DWX-FLAGS-1's test now names
`--actors-threads` among the flags still refused). `build --target wasm` builds actor programs again. An actor artifact built
before carries no counts and is refused before it runs: DL1202, "rebuild it from its source".

**Evidence.**
- `wasm_cli.rs` `an_actor_programs_dwx_runs_as_its_source_does`: two actor types, self-sends, a send through a field, and a
  behaviour dying on a division by zero. The artifact's quiescence report — "2 surviving actor(s)" and "1 died" — equals the
  source's on `--engine wasm` and on the interpreter. An artifact with its count exports renamed is `error[DL1202]` and never
  "out of range".
- `actors_parity.rs`: `assert_actor_parity` also runs every parity program as an artifact and holds its report to the
  source's: the counter and ping-pong programs, the poisoning bomb, the fifteen-name program. 8 passed.
- By hand: `examples/guide/06_actors.delulu` built and run as a `.dwx` — 5 surviving actors, 13 turns, as its source on both
  engines.
- Mutants:
  - M690 every count zero → red (six parity tests);
  - M691 behaviours read in reverse → red;
  - M693 the counts not set → red;
  - M694 the count exports one global off (the heap pointer) → red: not immutable;
  - M695 a missing count read as zero → red;
  - M692 (the self slot not subtracted) survives as equivalent: arities are read only by the codegen, from the AST, at
    compile time, and Wasmtime type-checks every call itself.
- `delulu-wasm` whole: 174 passed. Six artifact CLI targets (`wasm_cli`, `release` — byte-identical builds among them —
  `examples_run`, `artifact_review`, `json_contract`, `plugin_cli`): 57 passed. clippy with rustfmt's check clean.

**For `launch/2.0.0`'s next merge:** its Breaking paragraph says `build --target wasm` refuses a program with actors — no
longer true once this lands; that clause is to go.

## 2026-10-21 (evening) — the laptop: P9-05 (c)'s static half — `authority --json` names each request by the checker's type (D-V2-168, RW 2.12)

**Why now.** D-V2-140 §2 kept `authority`'s static path report open for one reason: the AST walk could not tell an `Http`
capability's `get` from a `Map`'s without types, and guessing which call is a request is not a claim this project makes.
The checker's per-node types, which the WASM backend has used since RW 3.4, answer exactly that.

**What.** `within::request_destinations` is `secret_destinations`' walk (`SendWalk`), given the checker's `node_types`. At
each `get`, `get_with` and `get_with_secret` whose receiver's type is `Cap[Http]` it records:
- the method;
- the URL literal;
- its host and path by the egress client's own `parse_target`;
- the call's position.

A computed URL is `null`, and so are the host and path of a URL that client refuses (not `https://`). The report gains
`requests` only when the program makes one, and the closed schema admits it. Built for a program checked as one file: a
package's check keeps no per-node types, and node ids are per module — recorded open.

**The core's answers moved by one case** (`tests/core-invariance/SNAPSHOT.txt`, re-recorded with `DELULU_BLESS=1` as a
reviewed act, D-NE-3) — `examples/guide/05_capabilities.delulu :: authority --json`, which gains exactly:

```
    "requests": [
      {
        "at": { "column": 5, "file": "examples/guide/05_capabilities.delulu", "line": 30 },
        "host": null,
        "method": "get",
        "path": null,
        "url": null
      }
    ],
```

— the chapter's `client.get(url)` inside `fetch`, whose URL is a parameter. Intended: nothing else in any recorded case
moved. The file stays LF (`git ls-files --eol`: `i/lf w/lf`).

**Evidence.**
- `within_cli.rs` `authority_names_each_request_by_the_checkers_type_and_never_by_a_methods_name`: the parameter's URL
  (all null), a literal (`example.com`, `/v1/health`) and an `http://` literal (host and path null) are named, and the
  `List`'s `get` on line 8 is not. The report validates against the schema; a program with no request has no key.
- Mutants, each red, the control green:
  - M696 decided by the method's name;
  - M697 the key never stamped;
  - M698 the path taken from the host;
  - M699 the schema without the key.
- `delulu` and `delulu-conform` whole, before the snapshot was re-recorded: 1,029 passed, 4 failed in 103 targets. The
  failures were the three `doctor_cli` map tests (stale before the Survey's rebuild) and this snapshot case. Both are
  answered above; after them, see the commit message.

## 2026-10-21 (evening) — the laptop: a package's `requests` — each module by its own node types (D-V2-168 item 2)

The entry before left a package's report open: `check_program` kept no per-node types, and node ids are per module.
`Program` now has `node_types`, one map per module, keyed by its **unit name** — the key both package checks
(`check_program` for a loose directory, `check_workspace` for a manifest and its dependencies) already use for everything
else. `emit_authority` takes the unit names beside the modules and pairs them directly. It never recomputes a key from a
module's declared name: a workspace need not keep that unique, and the wrong map would put one module's types on
another's nodes. The single-file `Program` the CLI builds and the atlas's test helper carry theirs too.

**Evidence.** `within_cli.rs` `authority_names_each_request_by_the_checkers_type_and_never_by_a_methods_name` gains three
cases:
- a two-module package whose second module requests `/ping` (named in `src/net.delulu`; the `List` `get` in `main` is not);
- an app whose dependency `netlib` requests `/dep`;
- a loose directory (`src/` and no manifest) — which first read empty because the test had put its files beside `src/`
  instead of in it (the loader reads `src/`), a test mistake read before blaming the code.

Mutants, each red, the control green:
- M700 `check_program` drops the types;
- M701 `check_workspace` drops them;
- M702 the report pairs by a wrong key.

`delulu-check` whole: 275 passed. `delulu-atlas`: 18 passed. `within_cli`: 17 passed. clippy with rustfmt's check clean.

## 2026-10-21 (evening) — the laptop: a once-seen Windows red, made to say why next time (RW 7.22)

`master`'s push run `39599000467` on `eef9583` failed one test on `test (windows-latest)`:
`sandbox_confirm_cli::an_external_launcher_ends_when_its_host_is_killed_on_windows` — "the launcher never started", after
its 60 s wait (7 of 8 passing; every other job green). The same commit's pull-request run `39596126140` was green, and the
test has passed on every run since PS-E-02.

The test starts `delulu run --sandbox --sandbox-backend external:powershell …`, whose launcher writes its own pid. It had
sent the host's stderr to nowhere, so the red cannot say which happened: the host ended early (a sandbox refusal, an
AppContainer the runner would not make), PowerShell took over a minute to start, or the host waited on something else. A
guess is not a root cause.

**So the witness now speaks:**
- the host's stderr is kept in the lab directory;
- a host that ends before the pid appears is named at once — its exit status and its words — rather than after 60 s;
- the 60 s bound says the host was still running, and quotes it.

Falsified: a launcher that does not exist fails in 0.06 s, quoting the host's "the external launcher `no-such-launcher-exe`
could not be started: there is no such program". Locally (Windows 11) the test is green. Three `witness.yml` runs of the test
on `windows-latest` at `fa4021c` were dispatched to reproduce it: `39602082207`, `39602087235`, `39602091804`.

## 2026-10-21 (night) — the laptop: the path in the chain's record (P9-05, D-V2-169, RW 2.12)

P9-05's open slice since D-V2-139 §6: the broker's `Net` op carried the host, and a use's record named the host alone.
Reading the code for it found what that cost. Under the daemon, the custody gate runs before the effect's own first check,
and the broker compared the host alone. So a request to a granted host, for a path outside every pattern the grant names
for it, was recorded `use allow` and then refused at the call.

**Witnessed** with the pre-change build (`launch/2.0.0` `678dee6`'s binary, Windows, a daemon, granted
`net.special=*.example.invalid/v1/**`). `GET /v2/items?q=1` was refused DL0904 at the call ("outside every path the grant
names for its host"), and the chain's seq 9 reads `use allow … target=api.example.invalid`. No record anywhere named a path.

**Built** (D-V2-169):
- `prim::request_path` is the URL's path without its query or fragment, `/` when it names none.
- `Custody::check_at` carries the path, defaulting to `check` for a custody that records nothing (embedded).
- The interpreter's gate (`custody_path_for`), a sandboxed guest's host (`channel.rs`) and the WASM host
  (`custody_allows_at`) send it. The plugin wrapper forwards it.
- The daemon client sends it in `Check` as `url_path`, a key absent from the frame when there is none.
- The broker (`check_use_at`) judges it by `netscope::judge_all`, refusing as `Denial::NetPath` (DL0904, worded as the
  call words it). It names the path in the use's payload as `url_path`, beside `op`; `target` stays the host.
- `audit query` prints `path=`. The OCSF export maps the path to `http_request.url` (`hostname`, `path`).
- Records before this verify unchanged: the payload is hashed as a value, whatever its keys.

**After** (this branch's binary, the same scenario): `/v1/items` is recorded `allow … path=/v1/items`, and `/v2/items`
is recorded `deny … path=/v2/items`, refused by the broker in the call's words. The same holds for the WASM engine and for
a sandboxed guest, both read by hand on Windows and then in `broker_cli.rs`.

**Tests:**
- `validate.rs` `a_net_use_naming_its_path_is_judged_on_it_and_recorded_with_it`.
- `broker_cli.rs` `a_path_scoped_net_grant_is_honored_under_the_daemon_as_embedded`, extended: seven runs (the
  interpreter's three, the WASM engine's two, a sandboxed guest's two), each use's path and decision. The query is never
  recorded, and the text form prints `path=`.
- `audit_ocsf_cli.rs`: `http_request.url` for a use naming its path, none for one that names none.
- `delulu-runtime`: `a_network_use_carries_its_request_path_and_never_its_query` and
  `a_plugin_exports_custody_forwards_the_request_path`.

**Mutants** M703–M717, each red, the restored tree green (the list is in D-V2-169's evidence).

**Local results:** `delulu-broker`, `delulu-runtime` and `delulu-wasm` whole, 746 passed. The `delulu` binary's unit tests
with `broker_cli`, `audit_ocsf_cli`, `message_spacing`, `monitor_cli`, `plugin_load_cli` and `sandbox_guard_e2e`, 255
passed. `scripts/clippy.sh -j 4` (rustfmt's check, then clippy) clean and stamped. The whole suite runs on GitHub (the pull
request's CI).

**Left in P9-05:** methods (d), with the first non-GET primitive (RW 2.11).

## 2026-10-21 (night) — the laptop: `Map` on the WASM engine (RW 3.4, D-V2-170)

**Before:** the WASM backend had no `Map`. `delulu build --target wasm` of a program calling `Map()` answered "DL1201: WASM
codegen does not support a call to `Map`" (witnessed on this tree with the change stashed). `str` of a map was pinned as
refused by `value_parity.rs`.

**Built** (D-V2-170):
- `Ty::Map(K, V)`: a header like a list's, its data 16-byte pairs, ascending by key.
- `Map()`, typed by the checker's type for the call or by a declared type.
- The eight methods: `len` and `is_empty` read the header. `get`, `contains_key` and `remove` ask the host for the key's
  index. `insert` asks the host to place the key and fills the value slot it answers. `keys` and `values` are new lists
  the host copies out of one column.
- `str`, `==` and `!=` go through the host's value interface, with the descriptor `m<K><V>` that `reify.rs` rebuilds as the
  interpreter's `Value::Map`.
- The host's `delulu:map` (`new`, `find`, `put`, `remove_at`, `keys`, `values`) orders by the interpreter's own `MapKey`.
  It reads the guest's memory as untrusted, and on any failure it records DL0903 in words and traps.

**One flaw found and fixed before landing:** `keys`/`values` reserved the new list's slots before reading the pairs, so a
hand-written header claiming 2^32 − 1 pairs made the host reserve 32 GiB first. The pairs are now read first. The mutant
that reverses this (M725) **survives**: the operating system granted the untouched reservation, and the read then refused
as before. No test can see the order, which is recorded rather than claimed.

**Tests:** `delulu-wasm/tests/map_parity.rs`, four, each run on both engines (the outputs equal) where a program runs:
- every method, on each key kind and three value kinds, with sharing, `str` and `==`;
- 300 keys inserted out of order, half removed; `Str` keys by their bytes, a key built at run time replacing a literal's;
- a map of records holding the records themselves, and a map of sums;
- four hand-written modules, each refused DL0903.

`value_parity.rs`'s list of what stays outside now holds a list holding a map in place of `str` of a map (`str` of a map
compiles; its parity is `map_parity.rs`'s).

**Mutants** M718–M733: fifteen red, M725 surviving (above), the restored tree green.

**Local results:** `delulu-wasm` whole, 178 passed. `artifact_review`, `budget_cli` and `wasm_cli`, 30 passed.
`core_invariance` green. `scripts/clippy.sh -j 4` clean and stamped. By hand: a map program built to a `.dwx` and run
prints the interpreter's output.

## 2026-10-21 (night) — the laptop: every `List` method on the WASM engine (RW 3.4, D-V2-171)

**Before:** five of the fifteen `List` methods compiled. The pre-change build refused the others: "DL1201: WASM codegen
does not support the List method `.pop`" (and `.filter`, `.reverse`, each program of the new test built by hand).

**Built** (D-V2-171):
- **In the guest:** `pop`, `reverse`, `concat`, `slice` (clamped as the interpreter clamps), `contains` (as `==` compares),
  and `filter`, `find` and `fold`, their function values inlined as `map`'s is.
- **Host-side:** `sort`, stable by the interpreter's order, its strings compared where they lie, and `join`, its total
  bounded before one allocation.
- **Detection:** a module carries the list helpers for any method on a receiver the checker typed a `List`.

**Found while building it, by an existing test:** `list_parity.rs` pinned `pop` as refused, with its reason written beside
it: a `pop` and then a `push` reuse the popped slot, which a walk of the list's own slots could see. Making `pop` compile
turned that pin red. Read rather than deleted, it named a real divergence. With every walk reading the list's own slots, a
program popping and pushing inside each walk printed `1239` from its `for` (the interpreter `1234`), and `map`, `filter`,
`fold` and `find` diverged likewise (D-V2-171 item 5 has all five). The fix: in a module that pops a list, each walk that
runs code per element walks a private copy of the slots taken when it begins, which is what the interpreter's copy means.
A module that never pops is unchanged.

**Tests:** `delulu-wasm/tests/list_methods_parity.rs`, six (D-V2-171's evidence). `list_parity.rs` now lists a list of
lists and a list holding a map as outside, and `conformance_parity.rs` a list of lists in place of `fold`.

**Mutants** M734–M752, each red, the restored tree green. M748 first survived, and the test it showed missing (a string
cell whose length runs past the memory) was added.

**Local results:** `delulu-wasm` whole, 184 passed. `wasm_cli`, `budget_cli`, `artifact_review` and `core_invariance`, 31
passed. `scripts/clippy.sh -j 4` clean and stamped.

## 2026-10-22 (night) — the laptop: `contains` on records and sums on the WASM engine (RW 3.4, D-V2-171 amended)

The last `List` method shape the WASM engine refused: `contains` on a list of records or sums answered DL1201. It now
compares through the host's value interface, as `==` of two records or sums already did (`reify.rs`). The module carries
the interface when a list's `contains` takes such a value. Test: `list_methods_parity.rs`
`contains_on_records_and_sums_compares_as_equality_does`, both engines `true false false` / `true false true false`.
Mutants M753–M755 red. M753 removes the new arm, which is exactly the pre-change refusal. Local: `delulu-wasm` whole, 185
passed; clippy clean and stamped.

## 2026-10-22 (night) — the laptop: a `match` tested inside each constructor, and proved to cover every case (D-V2-172)

Found while reading the WASM `match` to widen RW 3.4. Three defects, each witnessed on `master` `30885b1`:

- **MATCH-NEST-1 (RW 3.22), the WASM engine, silent wrong answers.** `compile_match` sent each constructor to the first
  arm naming it, and `compile_arm` bound a field only when its pattern was a name, ignoring a constructor or a literal
  there. `Some(Blue)` took `Some(Red) => …`; `Some(7)` took `Some(0) => …`. The interpreter printed `2` and `7`, WASM
  `1` and `10`. It reached `run --engine wasm`, `build --target wasm` and every Contained plugin (`cli.rs:4913`). Such a
  match now compiles arm by arm, in order, as `eval_match` tries them; when no arm matches, `delulu:match.no_arm`
  records the interpreter's DL0907 word for word. A match testing nothing inside a constructor compiles as before.
- **MATCH-EXHAUST-1 (RW 2.20), the checker.** `check_exhaustive` asked only which constructors the arms named at the
  top. `Some(Red), None` on an `Option[Color]` checked clean, and so did literal arms on an `Int`, `Str` or `Bool` with
  no catch-all; each faulted at run time with *"no match arm applied (checker guarantees exhaustiveness)"*. The
  reference says *"A `match` must cover every case … Enforced by: DL0407 … Coverage: covered"*. A search over the
  arms (Maranget's) now names the first missing case: `missing Some(Blue)`, `missing _`, `missing false`.
  - It is bounded by the cells it reads (10,000,000 for one match). A first version counted calls and **passed** a
    match whose search ran out: a random 400-arm match over 40 `Bool`s was accepted, and it is not exhaustive. So a
    search that runs out now refuses, in words. The pigeonhole principle with 8 pigeons in 7 holes is exhaustive but
    needs an exponential search: refused in 0.37 s. With 7 pigeons in 6 holes it is proved in 0.25 s (debug build).
  - A row of wildcards ends the search at once; without that, single-column `Bool` arms cost 2ⁿ steps.
  - 252 programs in the repository were checked before and after: no verdict and no message changed.
- **WASM-IMPORTS-1 (RW 3.23).** The import section was written only when one of a hand-kept list of flags was set, and
  the list left `delulu:map` out. A pure function whose only imports were a map's compiled to bytes the engine refused
  (`failed to parse WebAssembly module`). The section is now written whenever it holds an import.

Tests: `match_parity.rs` (4), red on the old code (`red,red,red,none` for `red,other Green,blue,none`;
`Y,Y,Y,Y,zero,zero` for `Y,M,empty,ok no,zero,err 3`); `map_parity.rs`
`a_module_whose_only_imports_are_a_maps_loads`; three `delulu-check` tests (missing cases named, every-depth clean
cases, the pigeonhole bound); two conformance reject cases (`DL0407_nested_non_exhaustive`,
`DL0407_literal_non_exhaustive`). Mutants M756–M767 (WASM) and M768–M776 (checker) red. M774 survived first: swapping
which `Bool` literal a branch keeps is invisible when the arms are symmetric in `true` and `false`. A case that is not
symmetric was added (`P(true, true), P(false, _)` leaves `P(true, false)`). Local: `delulu-check` and `delulu-wasm`
whole, `conformance`, `message_spacing`, `wasm_cli` green; clippy clean and stamped. Commit `823673e`, PR #34.

## 2026-10-22 (night) — the laptop: deep values print and compare on both engines (RW 4.74)

Found while checking whether the WASM host's rebuild of a guest's value (`reify.rs`) bounded its depth before widening
what a list may hold (RW 3.4). It did not, and neither did the interpreter's own `display` and `eq`. INTERP-DROP-1 had
made teardown iterative and left them recursive.

- **DATA-DEPTH-1 (RW 4.74).** A linked list of records built in a `while` loop, then `str` of its head:
  - the interpreter prints it at 100,000 links and dies at 300,000 (`thread 'delulu-main' has overflowed its stack`,
    exit 127), inside the default 1 GiB budget;
  - the WASM engine dies at 100,000, so the guest's data killed its host. That covers `run --engine wasm`, a `.dwx` and
    every Contained plugin.
  - A hand-written module did the same through the descriptor parser, with a million `l`s.
- **Fixed.** `Value::display` and `Value::eq` walk explicit worklists. `display` keeps the cells it is inside, and
  leaves each when its rendering ends. `eq` keeps the pairs it has assumed equal, and compares every reachable pair once,
  so the order cannot change the answer. `reify.rs` builds post-order from a worklist with a stack of finished values:
  - a list's elements are read one per step from a single `Run`, so a long list never queues a step per element;
  - its descriptor parser refuses nesting past 256 (a written type stops at 128).
  - Both engines now print the 300,000-link list as `9488888` characters.
- **Not yet covered (RW 4.75, DATA-DEPTH-2).** Two more walks recurse: the copy into an actor message and the
  sandbox channel's serializing. Read, not run: whether a well-typed program can reach either with a deep value is open.

Tests:
- `delulu-runtime` `value::depth_tests`: a 200,000-level value through a variant, a list, a record and a map in turn,
  displayed and compared; and a cell held twice side by side.
- `delulu-wasm/tests/deep_value_parity.rs`: a 100,000-link list on both engines, a cell held twice side by side, and the
  deep descriptor.
- On the old code each kills its test binary with a stack overflow; each was witnessed separately.

Mutants M777–M785 red. M777 (`display` never leaves a cell) and M785 (the rebuild never leaves a variant) survived
first: no test held one cell twice side by side, under the old recursion either.

## 2026-10-22 (night) — the laptop: deep values and closures cross to actors (RW 4.75, 4.76)

- **DATA-DEPTH-2, the actor path (RW 4.75).** A chain of 300,000 links built in a loop and sent to an actor killed the
  process (exit 127). `value_to_msg`, `msg_to_value` and a message's derived teardown each recursed once per level.
  - Now each walks a worklist.
  - A part is read through the cell that holds it, never cloned onto the list. So the debug lane's
    `strong_count <= 2` check sees what the recursion saw: a first version that cloned each part would have flagged a
    list holding one cell twice.
  - `MsgParts` holds a message's parts and takes them apart iteratively when dropped. It, not `MsgValue`, carries the
    `Drop`, so a message can still be taken apart by value.
  - The root slice is boxed: `clippy` found `MsgValue` at 344 bytes, paid by every part of every message.
- **CLOSURE-SEND-1 (RW 4.76), found while writing 4.75's test.** A `let`-bound closure sent to an actor overflowed the
  old binary's stack, and after the worklist ran to the 1 GiB budget.
  - `flatten_env` carried every binding of every scope the closure was made in, and the closure is one of them.
  - It also carried values the closure never reads: an `iso` map beside it tripped the alias check.
  - Now a closure carries exactly `rcap_check::closure_captures`, the free variables the checker judges sendability by,
    each from the innermost scope below the globals. The judgement and the act are one set.
- A false alarm, recorded because it cost time. After a mutant run, `delulu.exe` still held mutant M791: the control run
  rebuilt only the library. A program then "failed" on my change. A clean build answered correctly. The fix: give the
  control run a command that rebuilds the binary.

Tests:
- `actors::move_graph_tests::a_value_nested_past_the_stack_crosses_whole`: on the old code the binary dies.
- `deep_values_cli.rs` (2, end to end: `str` of a 300,000-link list, and a 300,000-link chain to an actor).
- `actors_cli.rs`: a closure and a map cross with every part in place; a `let`-bound closure, with a closure inside it
  and a shadowed name, carries only what it reads.

Mutants M786–M793 red. M789–M791 went red only after the parts test: no test had sent a map of more than one key or a
closure capturing more than one value. Still recursive (4.75): the sandbox channel's `WireValue` and `--debug-rcaps`'
uniqueness walk.

## 2026-10-22 (night) — the laptop: lists, maps and options nest on the WASM engine (RW 3.4, D-V2-173)

The widening this night's work began with, reached once the defects it surfaced were fixed: MATCH-NEST-1,
MATCH-EXHAUST-1, WASM-IMPORTS-1, DATA-DEPTH-1 and -2, CLOSURE-SEND-1.
- A slot holds a pointer to any list, map, option or result (`Scalar::Boxed`, interned so `Scalar` stays `Copy`). So a
  list of lists, a list of options, a map of lists, an option of an option, a map in a record field compile.
- The host's value rebuild (`reify.rs`) and descriptors were already recursive in structure, and since DATA-DEPTH-1 they
  are iterative in walk. `match` tests inside each constructor since MATCH-NEST-1.
- The five pins that listed these shapes as refused were read first (POP-WALK-1). None named a hazard. Each now lists
  what stays outside: a list or a record field holding a function or a capability.

Tests: `nested_parity.rs` (3), refused by the old codegen with DL1201 (`a list of List(I64)`). It checks sharing (one
list held twice and in a record, pushed to after), `contains` and `==` by contents, a nested option matched, a result of
lists, a sum holding a list, `find` and `fold`, three levels of lists, and an inner index's fault in the interpreter's
words. The whole `delulu-wasm` suite passes. Mutants M794–M797 red: the type made a slot type for only lists and maps, `contains` of nested values by address, a record field refusing a map, a list element refusing a pointer.

## 2026-10-22 (night) — the laptop: `--debug-rcaps` walks a deep graph (RW 4.75 closed)

Witnessed: a 100,000-deep `iso` graph (records in options) moved to an actor runs without the flag. Under
`--debug-rcaps` it stopped at the 1 GiB budget. `assert_unique_graph` recursed once per level and built each node's
whole path as it went, quadratic in memory. It now walks the same worklist as `value_to_msg`: a part is read through the
cell holding it, and each node's strong count is read before the walk holds any reference to it. A path is kept as its
parent and its last step (`Holder::step`), spelled out only for a node reported, in the same words: the existing tests
(`arg[0].0`, `arg[0][k]`) pass unchanged.

Test: `actors_trace.rs` `a_graph_nested_past_the_stack_passes_the_uniqueness_walk_and_its_deepest_alias_is_named`. A
100,000-level graph passes, and an alias at its bottom is named by its whole 33,334-step path, exactly. On the old walk
the binary dies of a stack overflow. Mutants M798 (the path spelled backwards) and M800 (every step hung from the root)
are red. With this, RW 4.75 is closed. The channel's walk still recurses, but no well-typed program can reach it, and
the host's decoder stops at 256 levels.

## 2026-10-22 (night) — the laptop: the cycle collector keeps what a map holds (GC-MAP-1, RW 4.77)

Found by reading `cycles.rs` for the depth class while closing RW 4.75. The collector already used a worklist, but its
marking `match` ended in `_ => {}`, and `Map` fell there: the collector dates from 2026-07-20 and `Map` from 2026-09-20.
- **Witnessed on the old binary.** An actor fills `self.m` with `"k" → [1, 2, 3]`, churns 600 short-lived lists to force
  two sweeps, then prints `{k: []}`. The live list was alive and unmarked, so the sweep broke it in place, and the only
  trace was `1 garbage-cycle cell(s) collected`. Silent data corruption.
- **Fixed.** A map's values are marked. The `match` now names every `Value` kind, so a kind added later does not compile
  until the collector places it. Built with and without the `python` feature.
- **Test.** `actors_cycles_cli.rs` `a_list_held_in_an_actors_map_survives_the_sweep` asserts the two sweeps, so the
  witness is exercised. It is red on the old collector (`{k: []}`), and that mutant is exactly the old code.
- **Residual.** A cycle through a map is not collected, because a map cell is not registered. That is a leak, the safe
  direction.

## 2026-10-22 (night) — the laptop: a cycle through a map is collected (GC-MAP-1's residual)

Witnessed: an actor turn that makes 200 cycles through maps (`g.insert("self", Link(g))`) reached **0 sweeps**, and all
200 leaked. `note_value` registered lists, records and closures' scopes and passed every other kind to `_ => {}`, so a
map was never registered, the pressure never reached the threshold, and no sweep ran. Now a map is registered
(`RegCell::Map`, `note_map`) and broken like a list, and `note_value` names every `Value` kind, as the marking walk does.
The same turn: 1 sweep, 200 collected. The live map of GC-MAP-1's witness is now registered as well, and stays whole
through two sweeps. Test: `actors_cycles_cli.rs` `a_turn_full_of_cycles_through_maps_is_collected`, red on the old
collector (no sweep).

## 2026-10-22 (evening) — the laptop: the exhaustiveness limit's test fits Miri (`miri-slow` timed out twice)

Witnessed: `miri-slow (delulu-check)` ran out of its 240 minutes on `launch/2.0.0`'s `jobs=everything` run `39678647200`
(`7b5aec3`) and on `master`'s nightly `39746314492` (`35d8208`); on `81f34d1` (run `39646720253`) it had taken 57
minutes. Every other job of both runs passed. The gaps between the job log's test lines name one test:
`a_match_the_search_cannot_finish_is_refused_not_passed` (PR #34) took 11,873 s. It reads about 15 million search cells
— 5,042,506 to prove six holes, then the whole 10,000,000-cell limit refusing seven — and a cell costs Miri about a
millisecond. `miri-slow` runs on the nightly and on `jobs=everything`, never on a pull request, so the test landed green.
(The next slowest, `forward_references_chains_and_recursive_nominals_still_work`, took 632 s; it took 847 s in the green
run, so it is not new.)

Now `EXHAUSTIVE_BUDGET` is ten thousand under Miri, as `audit.rs`'s `LOCK_WAIT` is an hour there, and under Miri the
test's pair is four holes refused (43,367 cells to prove) and three proved (3,694). Measured with a native probe: every
other match the `delulu-check` and `delulu-atlas` lib tests check — the two Miri jobs that reach the checker — reads at
most 11 cells (307 searches; atlas reaches none). With both `cfg!(miri)` arms forced true natively, the test passes, the
255 lib tests pass, and a mutant passing a spent search is red. The native limit and native test are unchanged.

## 2026-10-22 (evening) — the laptop: autosave pushes what origin lacks, not what an upstream lacks

Read at the owner's question, *"is autosave working properly as intended, To which branch autosave windows task
scheduler pushing everything"*. The task runs `%LOCALAPPDATA%\DeluluLang\autosave.ps1` (byte-identical to
`scripts/autosave.ps1`) every 10 minutes. Its snapshot step works: at 15:33 UTC it pushed this session's uncommitted
records as `b35b6b1` to `autosave/laptop`, and its newest earlier snapshot `838c739` is tree-identical to `8a88466`, the
commit that landed it. Its log's 16-hour gap is the laptop switched off (Kernel-Power: shut down 04:59 IST, booted 20:49).

Its push step asked each branch's configured upstream, and that was wrong three ways, witnessed:
- **A branch with no upstream was skipped** — every branch made with `git switch -c X master` until its first
  `push -u`. A commit on one that the session had not pushed stayed on this disk alone, and the snapshot step, seeing
  a clean tree, logged `clean`. Witnessed on the real repository: a local branch with one commit no origin branch
  held, the installed script run once by hand, `clean`, the commit still not on origin.
- **A `master` ahead of `origin/master` was pushed onto it** — past the rule that `master` lands only by fast-forward
  on a green run (owner, 2026-10-19). Witnessed on a throwaway origin.
- **A branch whose upstream has another name** (`launch-merge-1020`, tracking `launch/2.0.0`) was pushed under its own
  name each run (the stray `origin/launch-merge-1020`, `8ff2b17`, contained in `launch/2.0.0`), and one tracking
  `origin/master` was pushed again every run, unchanged (`laptop/wasm-contains-compound`, three runs on 2026-10-21).

Now a branch is pushed, under its own name, when it holds a commit no branch of `origin` holds; the snapshot branches
are left out of that question, since each carries HEAD as a parent. `master`'s and `main`'s unpushed commits go to
`autosave/laptop-master` / `-main`, never merged. A branch diverged from its namesake is logged, and one sharing no
commit with `origin` is refused — after the launch, that is the development history. `-DryRun` logs what a run would
push. On two identical throwaway setups (bare origin plus a clone, one branch per case): the old script pushed `master`
onto origin's `master`, re-pushed the branch tracking `origin/master`, and left `no-upstream` and `snap-parent`
unpushed. The new one pushed both, sent `master` to `autosave/laptop-master` while origin's `master` stayed put, logged
`diverged`, refused the orphan, pushed nothing on a second run, followed one more `master` commit, and snapshotted an
uncommitted edit. A dry run over the real repository's 47 branches would push only the witness branch, deleted afterwards
(local only, never pushed).

Then, at the owner's word (*"also snapshot the dl-launch worktree"*): the snapshot step read only the repository's own
checkout, so a session's uncommitted edits in the linked worktree `D:\nelan\dl-launch` — where `launch/2.0.0` is merged,
CHANGELOG conflicts resolved by hand — reached no snapshot. Now each worktree `git worktree list` names is snapshotted:
the repository's own checkout to `autosave/laptop` as before, a linked one to `autosave/laptop-<its folder>`
(`autosave/laptop-dl-launch`), each through its own private index, under the same refusals; a worktree whose folder is
gone is logged and skipped. On a throwaway setup with a linked worktree holding an edit and a new file, and a removed
one: the snapshot carried both, named the worktree in its message, left the worktree's own index unstaged, was
`unchanged` on the next run, and the removed worktree was logged; the repository's own checkout still snapshotted to
`autosave/laptop`. A dry run on the real repository: `clean: launch-merge-1020 7b5aec3 in D:\nelan\dl-launch`.

## 2026-10-22 (evening) — the laptop: the broker log names a late answer (RW 7.17's instrument)

RW 7.17's next step was a measurement: how long the serial daemon takes to answer a device's 1 s authority probe beside a
run's own appends. Nothing measured it — the daemon timed no request, and a run's line says `operator-revoke` for a probe
that got no answer in time, as for a revoke (D-V2-99). Now the serve loop notes when it took each request off its queue,
and an answer that leaves 500 ms or more after its request arrived (half the probe's deadline) is named in `broker.log`:
the request's kind (`ReqBody::kind`, a name and none of its fields — a field can be a token or a beat's key), the time it
waited for the requests ahead of it, and the time it took itself. An answer that could not be written has its own line
already and is not named twice.

Test: `broker_cli.rs` `an_answer_given_late_is_named_in_the_broker_log`, on every OS. The handler is held as RW 4.35's red
team held it — twelve pending guard requests with long justifications, asked for on the daemon's own endpoint (the
socket, or the Windows pipe named from the state directory's FNV-1a) by a client that reads 32 KiB a quarter second — and
`broker status` behind it waits seconds. On this laptop: *"a Status request was answered 2499 ms after it arrived: 2495
ms waiting for the requests ahead of it, 4 ms handling and answering it"*, the client 2509 ms; three runs green. Three
mutants red: the line never written, the wait measured from arrival to arrival, the threshold at 50 s. Building it
found two things a first try missed: identical guard requests are folded into one (`deduped`), so twelve copies asked
for 30 kB, not 360 kB; and a Windows reply's flush must drain within 1 s, so a holder reading 8 KiB a quarter second was
dropped after a second (`the reader did not take its answer`) instead of holding the handler.

`estop_cli`'s two no-revoke controls — the arm64 red and the `windows-latest` red — now attach `broker.log` to their
failure, and print each late answer on a green run too, where `witness.yml`'s `--nocapture` shows it: the measurement is
`witness.yml -f os=windows-latest -f target=estop_cli`. On this laptop: eight green, no answer late.

**Read on `windows-latest`** (`witness.yml` at `86b8df5`): runs `39810107555` (`estop_cli`, 8 green) and `39810113455`
(`estop_cli` and `broker_cli`, 18 green, the new test among them) — in the four no-revoke control runs no answer was late
(no `late answer:` line). The red stays a rare timing-class event; when it recurs, its failure carries `broker.log`.

## 2026-10-22 (evening) — the laptop: a grant naming a network share is warned of on Windows (RW 4.17's UNC half)

RW 4.17 left one question: an operator's `fs.read`, `fs.write` or `plugin` grant may name `\\server\share` (a program's own
path may not, PS-0-06), and on Windows reaching that share can send the user's NTLM credentials to `server`. Decided as
the row proposed: said, not refused — the grant is the operator's informed choice. `prim::unc_server` names the machine a
UNC spelling reaches (either slash; none for a local path or the `\\?\` / `\\.\` prefixes the grant parser refuses), and
the option parser warns as it reads each `--grant`, on Windows only — elsewhere `//server/share` is an ordinary path. The
first try printed the warning three times: `run` reads its arguments three times, so it is said once per spelling.

On this laptop: `fs.read=\\fileserver.invalid\share`, `fs.write=//fileserver.invalid/share` and
`plugin=\\fileserver.invalid\p` each warn once, naming the server, and the program runs (16 ms — nothing touches the
share until a program opens it); `fs.read=\\?\UNC\srv\share` is refused as before; local paths and `net=` are silent.
Tests: `cli.rs` `a_grant_naming_a_network_share_is_warned_of_on_windows` (on every OS: warned on Windows, silent
elsewhere), `prim.rs` `a_unc_spelling_names_the_machine_it_reaches`. Three mutants red: no warning, the warning every
time the arguments are read, only `fs.read` warned. Also corrected: RW row 1.11 and its section still called the
constitution's guarantee 5 unedited and the owner's — it was corrected on 2026-10-17 (7.10a, D-V2-141).

Then, reading the open rows: two of this night's RW edits (4.17, 7.17) had wrapped a table row across lines, and five
older rows were the same (4.58, 4.59, 4.53, 4.46, 5.6) — a Markdown row is one line, so each rendered as a stub and loose
paragraphs, and nothing caught it. Joined (the file's words unchanged, 27,939 in the same order), and gated:
`repository_structure.rs` `every_remaining_work_row_is_one_line`, red on the file before (all seven named), and
`the_broken_row_rule_finds_a_wrapped_row_and_nothing_else` (a code span holding `|` is not a break).

## 2026-10-22 (evening) — the laptop: `broker stop` waits for the daemon to exit (RW 7.19 closed)

RW 7.19: on Windows `broker_cli`'s secret tests left their temp directories behind on most runs, the cause suspected
(the daemon runs with its working directory in the state directory) and not confirmed. Confirmed: `broker stop` returned
once the daemon ANSWERED `Shutdown`, and the daemon then still had to leave its loop, wake its accept thread, remove its
pid file and exit — a directory removed in that window fails with OS error 32, *"The process cannot access the file
because it is being used by another process"*. Witnessed on the old code: start, stop, remove at once — the first of five
tries failed, both times it was run. Now `stop` asks `Status` for the daemon's pid first and returns once that process is
gone (`process_gone`: `kill(pid, 0)` and `ESRCH` on Linux and macOS, `GetExitCodeProcess` on Windows — an exited process
has closed every handle it held), waiting at most 5 s and saying so if that passes; it says `ok: broker stopped`. Five
of five removed at once, three runs; the secret tests left no directory in three runs; `broker_cli` 11, `estop_cli` 8,
`guard_cli` 2 green. Test: `broker_cli` `a_stopped_broker_holds_nothing_its_state_directory_can_be_removed_at_once`; a
mutant whose exit check always says "gone" is red. An operator gains the same: `stop` then `start` no longer meets the
old daemon still there.

## 2026-10-22 (evening) — the laptop: a package builds to one `.dwx` (RW 3.4)

RW 3.4 counted 12 of the corpus's 17 entry points compiling to WASM, two stopping at "a package's other modules". Read
again: `run <dir> --engine wasm` already ran both packages on WASM — `run` flattens a package into one checked module
and the backend compiles that — and the four-package `station` printed and wrote what the interpreter did, byte for
byte. What stopped them was the counter's question, `build FILE --target wasm`: an entry file alone cannot see its
package's other modules (DL0301), and `build <dir> --target wasm` refused a directory with a note to try `delulu build
.` — what had been typed. So a program of more than one module had no artifact.

Now `build <dir> --target wasm` compiles the program `run <dir>` runs: `load_package_flattened` (the run's loader, its
diagnostics now naming the command) resolves, checks the workspace — the authoritative gate — and flattens, and
`emit_dwx`, shared with the file route, compiles and embeds the authority with the scopes of the package's own manifest,
by default at `target/<name>/<name>.dwx` beside its `interface.json`. `station`'s artifact (4,226 bytes) runs with its
authority verified (`Read, Write`; `FsRead ./data`, `FsWrite ./out`, `Console`), its output and both files equal to
`run .`'s; the greeter's too. Re-measured file by file the same day: the 12, `demo` DL1205, the plugin host and
`numpy_mean` DL1201, the two packages' entry files DL0301 alone — **14 of 17** as programs.

Test: `package_run.rs` `a_four_package_program_builds_to_one_dwx_that_runs_as_the_package_does` — built from the
directory above, so the scopes must come from the package's manifest. Three mutants red: no directory route; the scopes
read from the working directory (it survived the first version of the test, which built from inside the package, and
the test was moved); the artifact written beside the directory. `artifact_review`, `grants_cli`, `json_contract`,
`plugin_cli`, `release`, `wasm_cli`, `cli` (53) green.

Also: three of this night's `CLOUD_SYNC_LOG.md` entries had been scrambled — each new one inserted after an anchor in
the middle of the one before, ahead of its file list; the lists moved back, every line kept (checked).

## 2026-10-22 (night) — the laptop: a WASM program holds a secret and sends it (RW 3.4, D-V2-174)

Of the corpus's three entry points still off WASM, `demo` stopped at one line: `root.secret("API_KEY")`, a secret
minted and never used, DL1205. Phase 3n refused the mint because the bytes "would still be placed in guest memory"; the
Stage 3 spec's normative §4.4 had said a secret in WASM lives "only in the host secret table", the guest holding a
reference. D-V2-174 builds that table. `root.secret` mints by the interpreter's own rule against the run's grants (the
value under embedded custody, the broker's handle under the daemon) into the host's capability table, and the guest
gets the index. `get_with_secret` runs host-side in the interpreter's order: trace record `«opaque»`, custody on the
host, then `broker_secret_send`, now one function shared with the interpreter, then its egress client. `verify` is
compared by `secret_verify`, also shared, under a `.dwx`'s declared-effects ceiling. `expose`, `root.declassify()` and
`Secret.map` stay DL1205.

Evidence: after a run that minted a secret and sent it to the egress client, the guest's memory holds no byte of it
(`host::secret_handle_tests`, with a control that finds the program's own strings). On the interpreter, on
`--engine wasm` and from a `.dwx`, the same request leaves as TLS to the audience only, with identical traces
(`egress_cli`). Under the daemon the WASM run's audit chain is the interpreter's, record for record (`broker_cli`).
`demo` prints what the interpreter prints, from source and artifact (`wasm_cli`). `verify` answers alike, and a
manifest stripped of `Declassify` is DL1202 at it (`wasm_cli`). Eight mutants red, one per claim.

Two things found on the way. The first version of the change broke Phase 3n's own `expose` test: its program
declassifies before it exposes, and `root.declassify()` had only ever been unreachable, so it now meets DL1201 first.
`declassify` is refused by name as DL1205, its one use being `expose`. And re-measuring the corpus found chapter five of
the guide newly refused: its never-called `password_matches(held: Secret[Str], …)` used to be skipped because the
backend had no type for the parameter, and now compiled up to its `verify`. Hence `verify`. A unit test that waited for
a connection also hung: tested alone, `delulu-wasm` builds the egress client without a transport.

PR #48's first run (`39825874508`) was red on `test (ubuntu-24.04)`, twice over. The memory test asserted `no-client`,
which held on the laptop's `-p delulu-wasm` and not on CI's workspace build, where cargo turns `net` on and the same
request tried to connect. It now accepts either; both builds were run on the laptop. And the conformance witness for
`ref.diag.DL1205` still named the replaced pin, which the laptop's run, lacking `delulu-conform`, had not caught. It now
names the replacement (an entrenched edit, recorded), and `delulu-conform` is green (28). Both lessons are in HANDOFF §11.5.

Re-measured file by file: **15 of 17** build as programs; `numpy_mean` and the plugin host remain (DL1201). Green on the
laptop, 640 tests, none failed: every `delulu-wasm` target, `delulu-runtime`'s unit tests, `delulu-diag`, and `wasm_cli`
(17), `within_cli` (17), `json_contract` (13), `broker_cli` (12), `egress_cli` (9), `package_run` (6), `release` (6),
`grants_cli` (5), `artifact_review` (4), `conformance` (4).

## 2026-10-22 (night) — the laptop: Delulu Core's Theorems 1–3 and No-forgery, machine-checked for the monomorphic fragment (RW 5.1, D-V2-175)

RW 5.1 was the headline evidence gap: beyond P17-9's higher-order fragment, nothing of `DELULU_CORE.md` §1–§7 was
mechanized. `docs/design/models/lean/DeluluCoreSafety.lean` (903 lines, core Lean 4.32.2) now states §1–§4 for the
monomorphic, closed-row fragment: rows on arrows, `let`, products, sums, the capability store, `op` with `E-Op` and
`E-Refuse`, `hop`, `wrap`, `verify` and `expose`. Evaluation is call-by-value by frames, with faults propagating. It
proves progress-or-fault, preservation, effect soundness over every finite prefix of a run, safety along a run, and
no-forgery. `grind` closes the row bookkeeping; the substitution lemma substitutes closed values only, as call-by-value
evaluation of a closed program does. It rests on Lean's standard `propext`, `Classical.choice` and `Quot.sound`.
Assumed: `SigOk`, that each primitive's result is well typed. Left out: type variables, open rows.

A second pass added attenuation. The store became a list, with a fresh token at its end; a step may extend it, and
typing survives the extension (`weaken_store`). `narrow` mints a token of its parent's kind under its parent's scope,
or faults. `attenuation_monotone` proves every token a run's store holds descends from a root token, over an abstract
scope preorder with the one law `narrow(s, v) ⊑ s`, and `no_forgery` carries the Corollary's scope clause. Lean
promoted the input store to a parameter of `Step` (it is the same in every rule), which shifted every `@`-pattern by
one until noticed. The third mutant, the law weakened, first changed its arity and failed for that alone; with the
arity kept it fails where the proof uses the law.

Three things the work found:
- **Lean reports a `sorry` only as a warning, and exits 0.** A planted `theorem planted : False := by sorry` checked
  "clean", and CI's step read only the exit code. The step now fails on any `sorry` in the output. CI also requires
  two rule mutants to fail to check: `T-Op` without its label, and `T-HOp` without the callback's row (C88's shape).
- **`T-Verify` was stale.** §3 gave `verify` no label, while the checker has typed it `Declassify` since P17-IF1.
  Witnessed: a pure function calling it is DL0501, and the trace records `Declassify`. And `E-Expose` was never written.
  v0.4 states both. Stated, `T-Verify` makes Fact B and Theorem 3's capability-witness clause false for that one label,
  which is RW 2.6, now named in the calculus. The development takes `verify`'s label as a parameter, so both rules are
  proved.
- **`DELULU_CORE.md` §9 said `CONTRIBUTING.md` recorded the mechanization as invited work.** It never did; the pointer is
  now RW 5.1.
- **Attenuation was relied on and never stated.** §4 said tokens are created only by `narrow`, and the No-forgery
  corollary rests on it, but `narrow` was no term of the calculus. v0.4 gives it `T-Narrow`, `E-Narrow` and
  `E-NarrowRefuse`.

An entrenched edit (`DELULU_CORE.md`, recorded). The models README, `MATHEMATICS.md`'s category 2 and
`REPOSITORY_STRUCTURE.md`'s tree are updated. Also on the way: running `lean --version` made elan download its default
toolchain, 4.35.0, beside the pinned 4.32.2 (local only).

## 2026-10-23 (small hours) — the laptop: `deploy plan` checks every request against the profile (RW 6.6, D-V2-176), and DEPLOY-PKG-1

RW 6.6: `deploy plan` compared a service's effects with the profile's ceiling and nothing else, though a profile can name
paths, hosts, secrets and foreign libraries. The command that answers the rest already existed: `authority --within`
decides each request a program makes, in the `--grant` grammar, by the runtime's own mint, with the call that makes it.
So the profile becomes a boundary (`profile_boundary`):
- `fs.read`, `fs.write`, `net` and `foreign.python` go through `Grants::add`, so a special-use host under `net` is refused
  as no boundary at all, D-NE-28;
- secrets and `foreign.c` are matched by name;
- `console`, `clock`, `rand` and `declassify` are granted where its effects grant them.

Each service's requirements go to `within::verdict`. A plan is approved only if every service's effects fit and its
verdict is *within*. Anything else refuses the whole plan, DL1909, named at the call:
`fs.read=./data at src/station.delulu:16, exceeds: …`, or a computed path as *inconclusive*. `accept_manifest`, the
run's own reading of a manifest, was not usable: it takes a secret only if it is set in the environment, and no
`foreign.c`.

Trying it on the four-package `station` found **DEPLOY-PKG-1**. `deploy plan` refused it as a service that "does not
check cleanly" while `authority` reported it. `package_authority_value`, the helper behind `deploy plan`, `publish` and
`add`, still used the single-package loader that C51 moved `authority` off, so all three refused every package with a
dependency. Witnessed for `add` on the old binary ("does not check clean, so its authority cannot be computed"). The
helper is now `checked_package`, the loader `authority` uses.

Tests: four new in `deploy_cli.rs` and one in `add_path_cli.rs`. Four mutants are red: the old loader, the verdict
ignored, the profile's `fs.read` dropped, its secrets dropped. Five approving fixtures in `deploy_plan_adversarial.rs`
name the host their Net service calls, and the scope contract pins the new `compared` and `profile_cannot_grant`. A
breaking change for a profile that named effects only, under 2.0.0. Green: `deploy_cli` 9, `deploy_plan_adversarial`
15, `add_path_cli` 8, `signing_cli` 6, `cli` 53, `cli_contract` 19, `json_contract` 13, `delulu-diag`.

## 2026-10-23 (early morning) — the laptop: CI read to its end; a Windows witness made to say why (RW 7.23)

Read since the last entry:
- PR #50's run `39833210704`: **success**, every job (14, with `miri-slow` and `heavy-gates` skipped as on any pull
  request). #50 landed by fast-forward, master `05909ef`; its push runs `39836740749` and `39836740781`: **success**.
- `master`'s push run for `4af166e` (#48), `39831374181`: **success**.
- `launch/2.0.0`'s everything run `39816919025`: attempt 2 of `test (windows-latest)` **success**, so the run is green on
  `7e7b2ed`. Attempt 1 had failed three wall-clock tests (RW 7.23 (b)).
- PR #49's run `39830724209` had reported success while the jobs API still listed its `editor` job as in progress, with no
  conclusion, when #49 landed. Read again at 03:55 IST: `completed`, `success`. GitHub's job record lagged the run's; the
  run's result stands.
- `master`'s push run for `b8c4fde` (#49), `39833849066`: **failure**, one test on `test (windows-latest)`:
  `monitor_cli`'s `a_monitor_sees_nothing_outside_its_own_subtree`. The run's arm was revoked (`operator-revoke`) after
  four of five refusals. The same commit's pull-request run and both of `05909ef`'s push runs were green.

`operator-revoke` is the program's word for every dead probe (RW 7.17): a revoked node, which here would be the sibling's
monitor reaching outside its subtree, or a broker that did not answer within 1 s. The test printed only the program's
line. Its messages now carry the run's stderr, where the device journal's `lease.revoked` detail keeps the probe's own
words, and the monitor's report. Falsified by forcing the baseline red: both appear in the message. The test passes
otherwise, unchanged in what it asserts. RW 7.23 records all four Windows failures; none is root-caused.

## 2026-10-23 (morning) — the laptop: a WASM slot holds a capability or a secret (RW 3.4, D-V2-177)

The parked branch `laptop/wasm-handles-in-slots` (`5373158`) finished. `Scalar::Handle` stores and loads a capability's
or a secret's index as one word, so a list, an option, a record field and a map's value can hold one. What it had left
open: a `Map()` or a `None` whose type only the checker names stayed DL1201, because `checked_ty` did not map a handle
type. It does now, for the same types `wasm_ty` reads from an annotation.

Tests, `handle_parity.rs`, 2:
- a list walked and indexed, an option, a record field, an annotated map and an inferred one, each printing through the
  handle it holds, held to the interpreter;
- a secret in a list or an option compiles, and `expose` from a list stays DL1205.

Both are red on `master`'s code (DL1201). The `checked_ty` mutants, Console and Secret: red. The first form of the test
used only annotated maps, and the `checked_ty` mutant survived it, so the inferred map and `None` were added.
`list_parity.rs` and `record_parity.rs` had pinned a list, and a record field, of `Cap[Console]` as refused. Those pins
were about the fragment's edge, not about safety, and the cases moved to the parity test. The pins now hold the root,
which no slot holds: a list of it, a record field of it, both red when `Root` is let into a slot. A first attempt pinned
`Cap[Python]` instead and passed for the wrong reason: a function whose signature the backend cannot map is left out of
the module, so nothing about the list was compiled.

Green: all 22 of `delulu-wasm`'s test targets; `wasm_cli` 9, `egress_cli` 17, `repository_structure` 6. Clippy stamped.
Survey: ok, 1555 nodes, 16285 edges; findings 0 errors. doctor: ok, 29 checks.

The book's chapter 9 still told a reader that any program with a map, a list of lists or a list of options runs on
the interpreter, which stopped being true on 2026-10-21. It now says what compiles since then (maps, every `List`
method, nesting, a package's `.dwx`, a secret held on the host, handles in slots: 15 of 17 entry points) and what
still runs on the interpreter: embedded Python, a plugin host, reading a secret's value, a function in a list or a
record. `book`, `evidence_claims`: green.

## 2026-10-23 (morning) — the laptop: a Windows-only `channel` failure root-caused — a zero sleep (RW 7.23 (b))

Read since the last entry: PR #52's run `39867706347` **success**, every job; #52 landed by fast-forward, master
`36740a4`. `launch/2.0.0`'s release dry run `39865371700`: **success**. Its everything run `39865367900` on `e51b4f3`:
`test (windows-latest)` **failure**, one test, recorded as RW 7.23 (c): `plugin_load_cli`'s
`under_a_lease_a_killed_runs_minted_node_is_revoked_by_its_dead_man_and_the_lease_lives`, whose run printed `refused`
where it commands its device. The program prints no reason.

RW 7.23 (b)'s `channel` `from_now_counts_the_wait_before_the_first_byte` failed once on a Windows runner, in its control:
a frame already in memory, read at once, must be served within 100 ms. The project's rule is to starve a machine, not
lengthen a bound. 32 busy loops on the laptop's 16 CPUs, above normal priority, turned the frozen test binary red 200
times in 200; 0 in 100 unloaded. A first fix gave the control five attempts. Under 32 busy loops at normal priority,
all five attempts took ~162 ms, every time. That is not a stall but a cost per byte. The test reader `Drip` called
`thread::sleep` before every byte, even when its gap was zero, and on Windows a zero sleep is `Sleep(0)`, which yields
the processor. With no zero sleep: 0 red in 100 at normal priority and 0 in 100 above normal. The five attempts were
taken out again; the control is as strict as it was. Falsified: `from_now` counting from the first byte turns the test
red. No production code calls `yield_now` or sleeps zero (`crates/*/src` searched).

RW 7.23 (c)'s program now prints which refusal it was given and its words; falsified by commanding 30 degrees
against the rudder's 25: the message read ``refused: envelope: `deflect_deg` = 30 is outside the envelope [-25, 25]``.
`plugin_load_cli`'s three lease tests: green.

## 2026-10-23 (morning) — the laptop: a record's width no longer costs per field (RW 2.10, D-V2-178)

Read since the last entry: `launch/2.0.0` `0ef515f`'s everything run `39874332895` **success on all 18 jobs**, its two
`miri-slow` jobs (delulu-check, delulu-broker) included; with its release dry run `39874335998` (success) the launch tree
is green throughout. `e51b4f3`'s `39865367900`: `miri-slow` (delulu-broker) **success**; its one red stays RW 7.23 (c).
Autosave: the task ran at 09:31 IST, result 0; its newest snapshot `c7eecb2` holds exactly `2349cff`'s tree, nothing to
recover.

RW 2.10 (finding C55) said a field read costs time in the record's width. Re-measured before changing anything, with a
release build of `master` and programs of 4 to 1,600 fields: reads and writes of the last field grew linearly, as
published (0.116 s to 0.473 s for 200,000 reads, whole process). Literals were worse, and not on record:
RECORD-ORDER-1 (2026-10-20) sorts a literal's fields into declared order, and its sort key scanned the declaration, so
every comparison cost two scans. Written in a shuffled order, 250 literals of 1,600 fields took 9.79 s, and one of
6,400 fields 1.1 s. `delulu check` of the same programs is linear (6,400 fields: 0.18 s).

The fix (D-V2-178): `Home.records` maps each record type's field names to their declared places, built once. A literal
looks up each field's place once (`sort_by_cached_key`, the same order as before). A read or a write in a record wider
than 16 fields asks for the place and uses it only when the name held there is the one asked for; a narrower record,
or a record not laid out by this module's declaration, is scanned. Release, 1,600 fields: reads 0.128 s, writes
0.110 s, out-of-order literals 0.078 s; 4 and 20 fields unchanged. Not through the DIR, as the row had planned: that
changes a serialized format, and an index taken on trust would misread a record built elsewhere.

`crates/delulu-runtime/tests/record_width.rs`, four tests. Two time a cost against a control measured beside it, best of
three: a 2,000-field literal written out of order against a 2,000-item list (3,662x on the old interpreter, 17x now,
bound 100), and the last of 8,000 fields against the first, on a record built outside the timing (21.4x on the old,
1.00 now, bound 3). A first version of the read test built the record inside the timed function; the old interpreter's
quadratic build then hid the read cost (3.22x against a bound of 3). Two hold the answers: a 40-field record written
in a shuffled order prints, compares, reads and writes as declared, through a function, a list and a record field; a
record whose fields are not where the declaration puts them is read and written by name. Mutants red, each on its own
test: the name check removed, a scan at every width, no ordering of a literal.

Green: `delulu-runtime` and `delulu-wasm`, 34 targets, 532 passed, 0 failed, 2 ignored. Clippy stamped.
Survey and doctor: in the commit message. Records: `measurements/scale/RECORD.md` (the table, before and after),
`HARDENING_CAMPAIGN.md` and `AUTHORITY_GUARD_CAPSTONE.md` (C55 closed), `CHANGELOG.md`, RW 2.10, HANDOFF §11.5 (a
correctness fix can bring a cost no test measures).

## 2026-10-23 (morning) — the laptop: a type shared through its variables is checked once, and one too large to write out is refused (RW 2.3, D-V2-179; OPAQUE-FIELD-2)

Read since the last entry: PR #55's run `39900420977` **success**; #55 landed by fast-forward, master `118cf51`.

RW 2.3 was deferred to an RFC: a bound on type size is language-visible. Re-measured on a release build:
`let aN = Pair { l: aN-1, r: aN-1 }`, 22 lines, 5.8 s, doubling per line. The type is N small types through its
variables. The checker expanded it in several places, each found only after fixing the one before:
- **The DIR's side table** writes every node's type out in full. Skipping that left 0.08 s at 22, so it was most of the cost.
- **The occurs check** followed every binding at every place (about 10%).
- **Unification** of two such chains (`a == b`) took 261 s at 22, once the side table was bounded.
- **`==`'s equality walk** (`opaque_for`) wrote the type out again at every part it visited.

The walks now follow each binding once; their answers are unchanged, and D-V2-179 sets out why. Writing a type out is
bounded: 100,000 parts a type, and 2,000,000 in all for types over 1,000 parts. Past either, the program is refused with
DL0211 at the expression. Refused depths take about 0.1 s, `a40 == b40` 0.08 s, and depth 15 still checks clean.
Ordinary programs take about 3% longer; the cost was found by elimination over six builds (D-V2-179).

`crates/delulu-check/tests/type_size.rs`, four tests, each program checked on a thread within a minute. Mutants red: no
limit, no pair cache, an occurs check that re-follows, no DL0211, the walk writing out each part again, the settle's
replay naming no node. One change I made survived and was taken out: the walk visiting each argument again through its
record's fields. `visiting` cuts it off one level down; it had been credited with a second that was really the per-part
writing out.

**OPAQUE-FIELD-2 (RW 2.21)**, found reading that walk. `type W { b: Box[Secret[Str]] }` checked clean under `a == b` and
`str(a)`, and so did a `Box` of a function: a user generic named in a field had its written arguments replaced by `Unit`
unasked. The runtime failed safe (`false`, `«secret»`). The arguments are now asked; the test is red without it.

Green: `delulu-check` 6 targets, 283 passed. `delulu-diag`, `delulu-syntax`, `delulu-wasm` and `delulu-conform`: 31
targets, 414 passed. Clippy stamped. Survey and doctor: in the commit message. Records: RW 2.3 closed and 2.21, D-V2-179,
DL0211's explanation, the book's chapter on bounds, `for-agents.md`, `CHANGELOG.md`, HANDOFF §11.5 (a substitution makes a
type a graph; a walk that resolves as it goes makes it a tree again).

## 2026-10-23 (afternoon) — the laptop: a sent secret is asked for, and decided, with its audience (RW 2.11, D-V2-180)

RW 2.11 stayed open for one step: proposing the audience in `--grants`' lines. A program sending `API_TOKEN` to
`https://api.example.com:8443/…` was given `--grant secret:API_TOKEN=VALUE` to type. Under that grant the run holds the
secret and refuses the send: *"its grant names no audience"*. `authority --within-file` answered *within* for that
boundary, and for one whose audience named `other.example.com:443`. `deploy plan`, which uses the same verdict
(D-V2-176), approved both. The runtime was right every time; the static answers were not.

The requirement for a secret now carries the origins of its sends, from the walk that already fed `secret_destinations`.
The line names them (`secret:API_TOKEN@api.example.com:8443=VALUE`, `HOST` for a computed URL, `NAME` for a secret the
source does not name). `--within` decides each send against the boundary's audience for that secret, by host and port,
as the egress client compares. A `deploy plan` profile's `secrets` entry may name an audience, read by the run's own
parser.

Witnessed: `within_cli` `a_sent_secret_is_asked_for_and_decided_with_its_audience` and `deploy_cli`
`a_secret_a_service_sends_is_bounded_by_its_audience`, both red on `master`'s `cli.rs`, `within.rs` and `deploy.rs`.
Mutants red: no audience in the line, the sends not decided, the port ignored, an unnamed secret's send always within.
A first run of the mutants without `--no-fail-fast` stopped at the first red target and left the second unread; they
were run again with it.

Green: the binary's unit tests and `cli`, `within_cli`, `deploy_cli`, `deploy_plan_adversarial`, `egress_cli`,
`grants_cli`, `guest_cli`, `sandbox_openshell_cli`, `schema_cli`, `mcp_cli`, `lsp_cli`, `secret_verify_cli`: 13 targets,
357 passed, 0 failed, 1 ignored. Clippy stamped. Survey and doctor: in the commit message.

**The same question, asked of paths (D-V2-181).** A program minting `example.com/api/**` that requests
`https://example.com/admin` was *within* `net=example.com/api/**`; the run refuses the request at its call (P9-05 (a)).
`--within` decided the mint and never the requests. Each request the checker types is now decided against the boundary's
`net` entries by the call's own reading (`prim::host_of`, `path_of`, `netscope::judge_all`), in a file, in a package (each
module by its own types) and in `deploy plan`. Witnessed: `within_cli` `a_request_is_decided_by_its_path_against_the_boundary`
red on the previous code, and `deploy_cli` `a_request_outside_the_profiles_path_scope_is_refused`, whose plan the previous
code approved. Mutants red: outside-path *within*, a package's requests not wired, a computed URL always *within*, the
unreadable-URL arm removed. A `typed` flag kept requests out of `--grants`; its mutant survived (`--grants`' walk has
none), and it was taken out. `deploy_plan_adversarial`'s fixture requested `http://example.com`, which the run refuses
("only `https://` is fetched"); five of its tests went red with a misleading reason. Such a URL is now refused in the
client's words, and the fixture requests `https://example.com`, its assertions unchanged.

## 2026-10-23 (afternoon) — the laptop: SEND-REBIND-1, a reassigned secret's send (RW 4.78)

The walk behind `secret_destinations` (and D-V2-180's verdict) restored a block's bindings at its end, undoing assignments
inside it. `var tok = root.secret("A"); if 1 > 0 { tok = root.secret("B") }` then a send: attributed to `A`, and `--within`
said *within* with `B` granted no audience (witnessed on `85f6b5c`'s binary). Same in a loop that assigns after it sends,
and in a closure (closures capture by reference: `var x = 1; let f = fn() -> Int { x }; x = 2; f()` prints 2 on both
engines). Fixed by forgetting assigned names after blocks, arms and closures, before loop bodies, and inside closures for
names their function assigns. `within_cli` test red on master; four mutants red. Green: 8 `delulu` targets, 306 passed.
Also read: launch `cab60c7`'s release dry run `39911653388` **success**; conformance on the launch tree 355/355, reference
in sync. The owner saved `CLAUDE_CODE_EFFORT_LEVEL=xhigh` and a setup script (with `|| true` and `exit 0`) in the Default
cloud environment; not yet seen run.

## 2026-10-23 (evening) — the laptop: RUSTSEC-2026-0385, fixed by a patch update (ctutils 0.4.5)

**Found by** pull request #59's CI `39929350206`: `supply-chain` red on RUSTSEC-2026-0385 against `ctutils` 0.4.2, an
advisory published after `30e714b`'s green push run `39913548269`. The tree did not change; cargo-deny's database moved.
The same lockfile is on `master` and `launch/2.0.0`, so both are red the same way until this update lands
(`cargo deny check advisories` on the launch tree: `advisories FAILED`, exit 1).

**What it reaches.** Before 0.4.4, `ctutils::Choice`'s mask functions, which are `const fn` and cannot use `cmov`'s
assembly, could compile to a branch. Its `u128` comparisons branched on 32-bit targets. `ml-dsa` 0.1.1 takes its
constant-time comparisons and selects from `ctutils` (`algebra.rs`, `hint.rs`, `signing.rs`). DeluluLang signs with
ML-DSA only in `delulu sign --hybrid`, which also needs `--unstable` (`signing.rs`, `pqc::sign_hybrid`). Verifying handles
only public data. `ml-kem` 0.3.2 links `ctutils` too; outside the NIST known-answer tests, nothing calls it. This
entry does not show whether ML-DSA's code calls the affected mask functions; the update was taken regardless.

**The update.** `cargo update -p ctutils`: 0.4.2 → 0.4.5, the newest 0.4. No other crate moved, none added or removed.

**Evidence.** Locally `cargo deny check advisories`: `advisories ok` (exit 0), and `advisories FAILED` on the launch
tree's 0.4.2 lockfile (the witness). `delulu-runtime` `nist_kat` 5 passed, so the signatures and keys are byte-identical
to NIST's vectors. `--lib pqc` 11 passed, `delulu` `pqc_cli` 6 passed. Clippy stamped; the pull request's CI is the rest.

## 2026-10-23 (evening) — the laptop: a URL named by a `let` or a constant is decided by its literal

**Why.** D-V2-180/181 decide a send by its URL's origin and a request by its path, but the walk read a URL only where the
call wrote it out. `let u = "https://example.com/admin"; h.get(u)` under `net=example.com/api/**` was *inconclusive*,
though the run refuses that request; `--grants` proposed `secret:A@HOST=VALUE` for a send through such a name. Naming a
URL once and using it is the ordinary way to write one, so the precision was the common case, not an edge.

**What.** `SendWalk.vars` now maps a name to what it is known to hold, a secret or a string literal (`Held`), under the
rules SEND-REBIND-1 just gave the secrets: a block's names end with it, a name assigned in a branch, a loop or a closure
is forgotten. A module's constants (`let NAME: Str = "…"` among its items, `Item::Const`) are read where no local name
shadows them, and each body's parameters now shadow them (`body(params, block)`). Another module's constant, imported,
stays computed. A constant cannot be assigned (DL0301, checked), so nothing forgets it.

**Evidence.** `within_cli` `a_url_named_by_a_let_or_a_constant_is_decided_by_its_literal`: red on the previous code (the
`let` request *inconclusive*, not *exceeds*). Mutants, each red at its own assertion: a parameter not shadowing (K1), a
`let`-bound literal not read (K2), constants not read (K3), a shadowing local falling through to the constant (K4), a
literal not forgotten after a branch (K5), a send's audience from the call's literal only (K6). Green: the binary's unit
tests and 14 `delulu` targets (`add_path_cli`, `atlas_chain`, `cli`, `cli_contract`, `deploy_cli`,
`deploy_plan_adversarial`, `egress_cli`, `json_contract`, `mcp_cli`, `sandbox_modes_cli`, `within_cli`, `grants_cli`,
`guest_cli`, `schema_cli`): 370 passed, 0 failed. `docs/for-agents.md` says where the literal may be written.

## 2026-10-23 (evening) — the laptop: a `net` entry names its methods (P9-05 (d), first slice, D-V2-182)

**Why now.** The next feature an agent needs is a request other than GET: most APIs take a POST. Under today's grammar,
that primitive would let every `net=HOST` grant ever written POST, PUT or DELETE on its host, so the method dimension
comes first.

**What.** `net=HOST[:METHODS][/PATTERN]`; none named = GET alone; one spelling per set. The order (subset) and meet
(intersection) are in `netscope`, beside the paths'. Every check asks with GET, and the broker gains
`Denial::NetMethod` (DL0904). `--within` decides a computed URL as *within* only when every boundary entry takes GET
on every path. The OpenShell export refuses a method scope. A port in an entry, accepted before and matching nothing,
is now refused in words, while the broker's host check still reads such an entry as one host string, so a malformed
suffix gains no match. An entry naming no host now takes no request (it took one whose URL named no host;
latent: such a URL is refused earlier). See D-V2-182.

**Evidence.** Witnessed red on the previous code: `egress_cli` `a_method_scoped_grant_takes_only_its_methods` ("host … is
not in the granted allowlist") and `sandbox_openshell_cli`'s method case (exit 0). Mutants N1 to N6 were red in the
`netscope` unit tests, and N1, N2, N3 and N5 end to end as well. Z3 §2e: 9 obligations proved, 50 in all, and the
mutants `net-method-dropped` and `net-method-meet-union` each caught. CI's teeth step now names 11 mutants.
`delulu-broker` `netscope` 10 passed; the `delulu` binary's `within` unit tests 8 passed, the agreement test
generating method entries.

## 2026-10-23 (night) — the laptop: `http.post`, the first request that is not a GET (P9-05 (d), second slice, D-V2-183)

**Why.** An agent calls an authenticated API to change something, and that takes a POST. D-V2-182 made the methods a
dimension first, so this primitive widens no grant written before it.

**What.**
- `post(url, headers, body)`, typed as `get_with` plus a `Str` body, the URL still argument one.
- The egress client gained `request(policy, method, url, headers, body, secret, …)`, with `request_with_secret` as
  its GET. Each hop is judged by its own method. A 301, 302 or 303 after a non-GET continues as a GET with no body; a
  307 or 308 keeps both, to the named origin only (`Reason::BodyElsewhere`). The body is at most 1 MiB
  (`Reason::RequestTooLarge`).
- The reqwest transport sends `request(method, url).body(…)`.
- The custody's `check_at` carries the method (`custody_method_for`, `None` for a GET) to the daemon. `ReqBody::Check`
  gains an optional `method`, absent for a GET, so a GET's frame is unchanged. `Broker::check_use_by` judges the
  method and records `method` beside `url_path`. OCSF maps it to `http_request.http_method`.
- `--within`'s `Ask::Request` and its `RequestSite` carry the method. The requirement line for a POST names `:POST`.
- The conformance anchor `ref.prim.http.post` has four witnesses (entrenched; `ENTRENCHED_CHANGE_RECORD.md`), and the
  reference was regenerated. `PRIM_TABLE_VERSION` is 8, and the pinned table count 85.

**Evidence.**
- Tests:
  - egress unit tests: the POST, refusal under GET alone, each redirect status, the cross-origin refusal, the size
    limit, a GET's record unchanged;
  - over real TLS: the request line, `Content-Length` and the body whole;
  - the broker's method judgement and record, and OCSF's `http_method`;
  - `egress_cli` at L0 and from a guest;
  - `broker_cli` `a_post_is_judged_by_its_method_under_the_daemon_as_embedded`: embedded, the daemon, the daemon
    with a guest; the chain's records `POST allow` and `GET deny`;
  - `within_cli` `a_post_is_decided_by_its_method_against_the_boundary`, including a request decided by its method
    alone.
- Mutants, each red: P1 the first check asking GET, P2 every hop judged as GET, P3 a 303 keeping the POST, P4 the
  body allowed elsewhere, P5 the custody never told the method, P6 `--within` deciding GET, P7 the daemon dropping
  the method.
- `delulu-conform --coverage`: one gap before the witness rows, 356/356 after. `--check-reference` in sync.

**Found on the way:** the egress test server read only a request's head, so its body arrived unseen. It now reads to
the head's `Content-Length`.

## 2026-10-23 (night, continued) — the laptop: `http.post_with_secret`, the authenticated POST (D-V2-183)

**What.** `post_with_secret(url, headers, header, secret, body)`, on the same branch and table version as `post`. The
secret rides as `get_with_secret`'s does, and the body as `post`'s. Under the daemon, `broker_secret_send` now takes
the method and judges the request's own method before the broker is asked for a byte. The interpreter's dispatch routes
both secret-carrying methods to it. `--within`'s walk sees the send (the secret is argument four of both).

**Evidence.**
- Tests:
  - egress `a_post_carries_its_secret_and_its_body_only_inside_the_audience`: the secret and the body on a POST to
    the audience; refused outside it before anything is resolved; a 307 to another origin refused, the secret sent once;
  - `broker_cli` `a_broker_held_secret_rides_a_post_to_its_audience_and_the_use_names_the_method`: under the daemon the
    POST went out, the chain held the `use` with `method: POST` and one `send`, and no byte of the secret. A capability
    minted for GET alone refused it at the call: no `send`, nothing connected;
  - `within_cli`: `--grants` names `net=…:POST` and the secret's audience.
- Mutants, each red: Q1 the pre-judgement as a GET, Q3 the walk blind to the send, Q4 the dispatch not routing the
  broker-held secret.
- Conformance: anchor `ref.prim.http.post_with_secret`, four witnesses; 357/357.

## 2026-10-23 (night, later) — the laptop: `http.post` on the WASM engine

**What.** The import `delulu:cap.http_post(cap, url, headers, desc, body, file, start, end)` is in the module only where
a `Cap[Http]` is minted and `post` is called. Its index follows `http_get_with_secret`'s, so a module without `post` is
byte-identical to before. The host reads the URL and body, reifies the headers as `get_with_secret`'s are, asks the
custody with the method (`custody_allows_by`, beside `custody_allows_at`, which now asks it with none) and performs the
interpreter's own `post`. `post_with_secret` stays on the interpreter.

**Evidence.**
- `broker_cli` `a_post_is_judged_by_its_method_under_the_daemon_as_embedded` now also runs `--engine wasm`, embedded
  and under the daemon. The chain gains the WASM run's `POST allow` and `GET deny`.
- `delulu build --target wasm` of a POST program wrote 1,860 bytes, and its imports name `http_get` and `http_post`.
  So the run compiled; it did not fall back to the interpreter.
- Mutant red: the host's gate asking with no method.

**And `post_with_secret` on the WASM engine.** The host's `http_get_with_secret` body became one helper,
`send_with_secret(caller, method, …, body_ptr: Option<i32>, …)`, with `http_get_with_secret` and a new
`http_post_with_secret` as thin imports. The custody is asked by the method, and the body is read when its pointer is
given. Codegen: the import, its 10-argument type, and the call. `broker_cli`
`a_broker_held_secret_rides_a_post_to_its_audience_and_the_use_names_the_method` now runs the POST on both engines
under the daemon, one connection and one `send` each. `build --target wasm` imports `http_post_with_secret`. Mutant
red: the helper's custody asked with no method.

**Found on the way (RW 4.79, open, LOW).** The GET-only run's custody recorded `use allow` with `method: POST`: the
node's grant takes POST, while the program's own capability, narrower, refused the call after. The interpreter asks the
custody before `prim` judges the capability, for every op. So the chain records a use the custody allowed and the
program never made. Nothing is performed, and the run reports DL0904. The test counts that record and names the row.

## 2026-10-24 — the laptop: a certificate carries every dimension a delegation can (CERT-SECRET-1, RW 4.80)

**Found.** Setting up the agent test campaign, the head chef ran the broker as `DEPLOYMENT.md` documents for
production: strict anchored roots, the root adopted from a certificate the anchor key signed. Delegating a lease that
carries the broker-held secret `TOKEN` and its audience was refused DL0802: the adopted root held no secret. `grants
certify` wrote effects, `fs.read`, `fs.write`, `net` and devices only. The certificate's `authority` field already
carries secrets, audiences, `declassify`, foreign libraries and the budget, and `adopt` and `⊑` read them. Only the
command line could not write them. Under strict mode a root enters only by adoption (DL1421), so no lease there could
ever hold a broker-held secret, `declassify` or a foreign library.

**Fixed.** `certify` reads `--secret`, `--secret-to`, `--declassify`, `--foreign-c`, `--foreign-python` and `--budget`,
parsed as `delegate` parses them. `--secret NAME=VALUE` is refused: a certificate names a secret and never carries its
value. Each audience member is canonicalized by the egress client's `parse_audience`. A pair for a secret the
certificate does not name is refused in words. `delegate` and `certify` now share the two helpers, `secret_name_flag`
and `secret_audiences`, so the two cannot drift.

**Evidence.**
- `federation_cli` `a_strict_broker_delegates_the_secret_its_certificate_carries`: a certificate with a secret, its
  audience and `declassify`; `secrets set`; `broker start --require-anchored-roots`; adopt. The delegation of the secret
  to its audience is allowed, and one to `example.com` is refused DL0802.
- Red before the fix: `unknown grants certify argument --secret`. Mutant red: the certificate's audience dropped.
- `federation_cli` 10 passed.

**And.** `toolchain --json` described `net` as `net=HOST[/PATH]` a day after D-V2-182 gave it methods. `GRANT_FORMS`
now says `net=HOST[:METHODS][/PATH]` for `net` and `net.special`, with the default (no methods: GET alone). Its
example names two methods and still parses (`broker` grant-form tests, 12 passed).

## 2026-10-24 — the laptop: the effect row escaped through a generic; a generic's parameters are rigid in its body (GENERIC-RIGID-1, D-V2-184)

**Found by the agent test campaign.** The owner asked for agents to build and test DeluluLang under access conditions
only the head chef can change. The head chef ran a broker with strict anchored roots and delegated one lease per
condition. A wrapper (`dl`) was the agents' only way to run delulu, and a Claude Code PreToolUse gate refused any agent
call that would change access. The gate was falsified first by a test agent, and judged by its own log. Agents: A, the
builder (Sonnet 5.5); B, least authority (Sonnet 5.5); C, zero authority (Haiku 5.5). Models read from the transcripts.
C's programs `coerce[A, B](x: A) -> B { x }` and `hide[e](…) -> fn() -> Unit { f }` printed under a console grant while
`check` said `main` never performed `Write`. The head chef reproduced both on the current binary.

**Cause.** `check_fn` gave the body fresh *flexible* variables for its generics (`make_genv`). The body unified `T` with
`Int`, or closed `e` to `{}`, and each call site instantiated them afresh. `check_actor` did the same for actor generics.

**Fixed.** Rigid variables (D-V2-184): never bound; a flexible variable meeting one is bound to it; a rigid row tail is
never closed. The body sees rigid generics; call sites and `spawn` see flexible ones. Messages name `T` and `e`.

**And the laundering suite's own witnesses.** Writing the new witnesses showed that F-3, F-3b and F-4 never parsed: two
statements on one line, DL0209, passed through recovery. `rejects_with` now refuses a witness that does not parse. The
three are repaired, and each is still refused with its own code.

**Evidence.** `laundering` 17 passed. Mutants red: the type guard dropped, the closed-row guard dropped, actor generics
flexible. Agent C's eleven generic programs are now refused DL0401.
Suites on the laptop: `delulu-check` (all targets), `delulu-conform` and `delulu-wasm` with 0 failures; `delulu`'s `core_invariance` 53, `cli` 19, `examples_run` 1 and `cli_contract` 6 passed. The full suite on three OSes is the pull request's CI run.

## 2026-10-24 — the laptop: a top-level constant's value is checked (CONST-UNCHECKED-1, RW 4.82)

**Found by the agent test campaign** (agent C, Haiku 5.5): `let Y: Int = "not an int"` at module level checked clean
and ran; `let W = spawn Worker()` checked clean, and `authority` reported only `Write`. The head chef reproduced both.

**Cause.** `check_module`'s item loop matched `Fn`, `Actor` and `Test` and passed every other item with `_ => {}`. A
constant was registered (`resolve`), its declared type used at its uses, and its value never looked at. The runtime
evaluates every constant before `main` (`eval_consts`), trusting a "checker invariant" nothing checked. This is the
walk's catch-all again (MATCH-NEST-1's lesson, HANDOFF §11.4): the item the loop never named was never checked.

**Fixed.** `check_const` checks each constant, in order and before the functions. Its value is an expression with
nothing in scope but the module's items, held to its declared type, and refused DL0501 if it performs any effect.
An unannotated constant's uses share the type its value has (`const_types`), where each use had a fresh one.

**Evidence.** `delulu-check/tests/constants.rs`, five tests, red before the fix. Mutants red: the constant pass
removed, the purity check removed, the use-site type ignored. Agent C's `p5_01`, `p5_02`, `p5_05` and `p5_06` are
refused.
Full workspace suite on the laptop (`cargo test --workspace --no-fail-fast -j 4`): 2,472 passed; the 4 failures were `doctor_cli` (3) and the Survey's `freshness` (1), which fail until the map is regenerated after the last edit, and pass after it (below).

## 2026-10-24 — the laptop: `% 0` faults alike on both engines (RW 4.83); three stale doc passages (campaign B10)

**`% 0`.** The builder agent (Sonnet 5.5) found `str(id(1) % id(0))` reported DL0902 on the interpreter and DL0901 on
WASM, while `/` agreed. The head chef reproduced it. `chk_rem_fn` trapped `b == 0` with `unreachable`, which the host's
`clean_trap` can only read as overflow; HARDENING_CAMPAIGN C20 had recorded the mismatch as a residual. `i64.rem_s`
traps `IntegerDivisionByZero` by itself, so the helper now leaves `b == 0` to it and keeps `unreachable` for `INT_MIN %
-1` alone. `fault_parity` gains `7 % 0` and `INT_MIN % -1`; it was red before the fix and is green after. `delulu-wasm`
199 passed; `core_invariance` and `wasm_cli` passed.

**Docs (B10, the least-authority agent, Sonnet 5.5; confirmed by the head chef).** `for-agents.md` said `--lease` and
`--broker` are refused under `--sandbox`. A leased `--sandbox` run works, and the lease judges the guest's uses
(`guest: error[DL0904]: lease … does not grant …`); `APPLIED_UNDER_SANDBOX` has listed both since RW 4.20. Two
passages said a secret-carrying request is "not yet" under `--engine wasm`, a claim stale since D-V2-174. All three now
say what the binary does.

## 2026-10-25 — the laptop: #66, #67 and #68 landed; the nightly Miri run of the broker back inside its budget

**Landed.** PRs #66 (GENERIC-RIGID-1), #67 (CONST-UNCHECKED-1) and #68 (`% 0` on WASM) were green on every job of
their own runs (40015000010, 40016786869, 40017678424: 14 passed, `miri-slow` and `heavy-gates` skipped as on every
pull request). `master` was fast-forwarded through them in order: `bf922f6` → `85f5f7a` → `0f79ac4` → `0549645`.

**The nightly run of 2026-10-24 (40075722165) was cancelled.** Every job passed except `miri-slow (delulu-broker)`,
cancelled at its 240-minute cap. Its log shows why: `netscope::tests::the_order_and_the_meet_with_methods_are_exact_over_a_bounded_domain`
(D-V2-182) ran from 08:24:57 to 11:09:59, 2 h 45 min, and the run then reached
`path::tests::canonicalization_laws_hold_on_generated_paths`, which took 66 min on 2026-10-23. The whole crate had
taken 1 h 25 min that night (39922976338). The test judged every request afresh for both entries of each of the 1,521
pairs, about 410,000 `judge` calls, all identical to the 3,510 needed. It now judges each entry's requests once and
each distinct meet's once. Its domain is unchanged, nothing shrinks under `cfg!(miri)`, and two mutants still turn it
red: a meet that unions the methods (line 741) and an order that ignores them (line 734). The `ci.yml` comment that
said no run had confirmed the 240-minute budget now gives the measured times.

## 2026-10-25 — the laptop: `--within` decides a file use at its call (B9, D-V2-185)

See D-V2-185. `authority fsr.delulu --within console --within fs.read=./allowed` now answers *exceeds* at
`fsr.delulu:14:10`, with the runtime's own reason (`path `../forbidden/secret.txt` escapes the granted scope (DL0904)`).
`within_cli` 23 passed and `deploy_cli` 23 passed on the laptop. The full suite on three OSes is the pull request's CI run.

## 2026-10-25 — on-demand routine run 6: #69 and #70 landed; A18, A2 and RUN-PIN-1 closed (D-V2-186)

**The session.** Opus 5.5 (the system prompt names `claude-opus-5-5`; no fallback notice), `CLAUDE_CODE_EFFORT_LEVEL=xhigh`
and `CLAUDE_EFFORT=xhigh` in its environment — the first run since the owner set the variable in the cloud environment on
2026-10-23, and it shows `xhigh`. The owner's setup script ran: the 1.96.1 toolchain's directory was made at 09:37:57 UTC,
before the session's first command, and the cargo registry already held 356 `.crate` archives, Windows' among them. No sous-chef.

**Landed.** The laptop's session of 2026-10-25 stopped at about 05:35 UTC with two green pull requests open: #69
(`2ddd15c`, the methods order test judges each entry once) and #70 (`daff3ec`, D-V2-185). Their runs `40165160695` and
`40165944252` passed every job (`miri-slow` and `heavy-gates` skipped, as on every pull request), and #69's `miri-slow`
dispatch `40165159856` passed inside its budget: delulu-broker 1 h 47 min, delulu-check 1 h 31 min, delulu-syntax 21 min.
Re-run here before `master` moved: `within_cli` 23 passed, `deploy_cli` **11** passed (both records say 23; the target
holds 11 tests), `netscope` 10 passed, and a mutant of D-V2-185 that answers every file use *within* turned
`a_file_use_is_decided_by_its_path_against_its_capability` red. `master` was fast-forwarded `0549645` → `daff3ec`. The
autosave snapshot `875c373` (05:31 UTC) is `daff3ec`'s code before its test was carried further, and the 2026-10-23
snapshot of the launch worktree (`ccb3241`) equals `launch/2.0.0` outside the map: nothing to recover.

**A18 — a sandboxed run let the program write the runtime's record of its run** (the agent test campaign's builder,
Sonnet 5.5; reproduced). `run p.delulu --grant fs.write=./out --report-out out/r.json` is refused before anything runs
(exit 2, D-V2-21); the same command with `--sandbox` ran, exit 0, and the guest wrote a counterfeit report where the
runtime's would go. `cmd_run` returned to the sandboxed route before its D-V2-21 check, and `guest.rs` never made one. The
sandboxed route now makes both checks the ordinary route makes: the command line's scopes before anything else (one
function, `refuse_runtime_paths_in_command_line_scopes`, which both routes call; a dry run refuses too, since it says what
the run would do), and the final grants, a lease's included, once the lease is redeemed and before a root is issued or a
guest exists. Witnesses: `sandbox_modes_cli` `the_report_is_refused_inside_a_write_scope_inside_the_sandbox_too` (two
spellings, strict and audit, a control outside every scope) and `sandbox_guard_e2e`'s lease step, sandboxed and not; both
red on the old code. M801 (the first check removed) red on the audit row, M802 (the second) on the lease step, M803 (both)
on both; the control green. Runners at `601d9bb`: macOS `40182254723`, Windows `40182256417` (json_contract 13 there, one
test being Unix-only; sandbox_guard_e2e 1; sandbox_modes_cli 12). `master` took it before the next two slices.

**A2 — DL1001's own repair wrote a manifest that does not parse** (the builder; reproduced). With `ratlib = { path =
"../ratlib" }` the repair appended `[dependencies.ratlib.authority]`, which TOML refuses after an inline table, so applying
it turned DL1001 into DL1004. The inline table is the form `delulu add` writes. The pin now goes inside an inline table,
before its closing brace, at a place read from the TOML parser's own span of the value; a table or dotted keys keep the
appended sub-table. The label reads "add an `authority` pin to this dependency". `fix_cli` said `tooling.rs` covered the
repair, and nothing did. Witness: `delulu-check` `deps::tests::the_pin_repair_leaves_a_manifest_that_checks_whatever_form_the_dependency_takes`
— six forms, each repair applied byte for byte, parsed again, its pin read back and the workspace re-checked; red on the
old code. M804 (the old append) and M805 (inserted after the brace) red; the control green. The Stage 2 specification's
§3.2 example showed the same invalid TOML (its §12a said so); it now shows the inline form.

**RUN-PIN-1 — `run` ran a dependency its pin refuses** (found while verifying A2 on the binary; HIGH). A dependency pinned
pure that writes, or one with no pin, is DL1001 under `check`, `build` and `lock`. `delulu run app` ran it — the
interpreter, `--sandbox` and `--engine wasm` each printed the dependency's words and exited 0; `build --target wasm` wrote a
`.dwx` that `run` then ran; `test` ran the package's tests; `deploy plan` planned it; `sandbox policy` previewed its run.
Every loader but `build_workspace` and `cmd_lock` checked types and never called `check_self_authority` or `check_pins`,
while the documents said the gate "runs BEFORE any code". `delulu_check::deps::supply_chain_gate` (DL1009 and DL1001) is
now called by every route that runs, ships or plans a package (D-V2-186). Witness: `provenance`
`no_route_runs_or_ships_a_package_whose_dependency_breaks_its_pin` — seven routes, each refused with its reason, nothing of
the dependency run, no `.dwx` written, for a broken pin and a missing one; the control, pinned to what the dependency does,
goes through all seven. Red on the old code. M806 (`run`'s loader), M807 (the preview's), M808 (`test`'s), M809 (the
planner's) and M810 (the gate emptied) red; the control green. Runners at `4ee928a`: macOS `40183138954` and Windows
`40183140704`, provenance 17 passed (the witness among them) and fix_cli 13 on each.

**The full suite at `4ee928a`** (alone, `-j 4`, the tree unmoved): 2,544 passed, **3 failed**, 16 ignored, 183 binaries.
The three are D-V2-186 at work, each read: two `deploy_cli` services were packages `build` refuses (a secret read and never
declared, DL1009), and `sandbox_run_cli`'s package with no declared effects is refused DL1009 by the gate before `run`'s
DL0701. The fixtures declare their secrets now, and the package case asserts DL1009 (the single file beside a manifest
keeps DL0701); `deploy_cli` 11 and `sandbox_run_cli` 19 passed.

**B1 — a lease cannot carry `net.special`, and nothing said so until the connect** (the least-authority agent, Sonnet
5.5; re-run here: `grants delegate --net 127.0.0.1` exit 0, no word). A broker node has no `net.special` dimension, so a
leased run reaches no special-use address. `delegate` and `certify` now print a note naming the host, the missing
dimension and `--grant net.special=HOST`; the node is still made, because the Guard and the audit decide a leased send
before the connect and `broker_cli`'s and `federation_cli`'s witnesses rest on such a node (D-V2-187, which also sizes
the dimension for the next slice). Witness `broker_cli`
`a_special_use_host_is_named_where_a_delegation_is_written_not_first_at_a_connect` (four spellings, both verbs, a public
host as the control); M811–M814 red, the control green; broker_cli 16, federation_cli 10, grants_cli 5, json_contract 14
passed. A first form refused the host; reading the tests that delegate `127.0.0.1`, before running them, showed what such a
node is used for, and the refusal became a note.
