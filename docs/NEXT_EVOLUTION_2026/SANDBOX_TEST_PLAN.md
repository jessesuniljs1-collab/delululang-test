# Sandbox test plan — generated adversaries, escape matrices, and the gates that must be able to fail

**Written:** 2026-09-17. **Status:** proposed; executed in phases PS-0…PS-C of
`SANDBOX_IMPLEMENTATION_PLAN.md`. **House rules applied:** every gate is falsified with a mutant
before it counts; inputs are generated, not hand-written, wherever a generator can express the
shape; silence is not evidence — every test asserts on something that happened; exit codes are
read from the process, never from a pipe; a witness must reproduce the *baseline* effect before
its refusal is believed.

**Inputs to this plan:** the head chef's own probes (`VERIFICATION_FINDINGS.md`, NE-16…NE-22),
the Opus 5 red team's attack-surface map and matrix
(`agent-notes/RED-TEAM-SANDBOX-SURFACES-opus5.md`, ~55 attacker moves; the claims used here were
re-verified by the head chef against the source and the binary — see NE-17…NE-22), and the field's
own benchmark shape (SandboxEscapeBench's layered scenarios, ControlArena's task-pair-plus-monitor,
Inspect's per-sample sandbox; `SANDBOX_RESEARCH.md` §1.1).

---

## 1. Shape of the program

Five families of tests, each owning one question:

| Family | Question | Runs where |
|---|---|---|
| **Characterization** | what does the *current* binary do at each surface? (pins today's residuals so a fix is a visible diff) | every push, all OSes, **now** — nothing to build |
| **Channel** | can the guest hurt the host through the only thing it can reach? | every push, all OSes (the parser is host code) |
| **Policy** | is the policy a pure, honest function of authority? | every push |
| **Boundary** | can the guest reach anything but the channel? | per level, per OS; L2 only on a KVM runner |
| **Lifecycle** | does a guest die when it should, and only then? | per level |

Plus a **measurement** family (not a gate): the round-trip cost per effect and the launch cost per
level, recorded under `measurements` in a new `sandbox` record in the style of the existing ones.

**Layered like the field's benchmark.** Every boundary scenario is tagged with the layer that
should stop it — *language* (the checker), *runtime* (the primitive table and proxy), *jail* (L1's
OS controls), *hypervisor* (L2) — and with a difficulty 1–5 in SandboxEscapeBench's sense, so the
published result can say "every difficulty-1 and -2 scenario is refused at layer X" rather than
"the sandbox is secure".

## 2. Characterization tests — runnable today (category 4; pins, not proofs)

These exist to make the current residuals *visible in the suite* before any sandbox code lands,
so that PS-A's fixes show up as red-to-green diffs and nothing regresses silently later. Each is a
CLI-level test against the real binary, each asserting on the observed effect, each with the
platform stated.

| Id | Surface | Today (verified 2026-09-17) | The test pins | Fixed by |
|---|---|---|---|---|
| C-01 | Windows reserved device names inside a grant | `write_text("NUL", …)` → `Ok`, nothing written anywhere; `write_text("CON", …)` → `Ok` and a real file named `CON` is created through the verbatim (`\\?\`) path (NE-19) | both outcomes, on Windows | PS-0-06 (refuse reserved names at the primitive table) |
| C-02 | Trailing dot / trailing space on Windows | `trail.txt.` and `space.txt ` create `trail.txt` / `space.txt`; the trace records the unstripped spelling (NE-20) | the created name ≠ the requested name | PS-0-06 (normalize-or-refuse; the trace records the resolved name) |
| C-03 | `http.get` | always `Err(Refused)` after the allowlist check — no network client exists (NE-17) | that no bytes leave the process (a listening socket on the granted host sees nothing) | PS-B-02 (the egress proxy is the first client) |
| C-04 | Special-use addresses | `--grant net=169.254.169.254` and `net=localhost` are accepted without a warning (NE-18) | accepted today | PS-B-02 (refuse unless explicitly spelled) |
| C-05 | Foreign-worker channel | no read timeout on `WorkerConn::call` (NE-21): a worker that never answers hangs the host | a fake worker that accepts and never replies hangs `run` past N seconds → today the test must use its own watchdog and *expect* the hang | PS-0-07 (apply IPC-1's fix) |
| C-06 | Resource bounds on the main program | none on either engine; an unbounded mailbox grows to >1 GB with only `--grant console` (NE-22, red team's measurement, head chef re-ran at smaller scale — see NE-22) | a bounded-time run exceeds a memory threshold | PS-B-01 (limits) |
| C-07 | `--isolation` under `--json` | no field at all (NE-16b) | absence | PS-0-02 |
| C-08 | DL1408 repair | `repairs: []` where the Stage 5 table promises `requires_human: true` and the fallback | absence | PS-0-03 |
| C-09 | Child environment | the foreign worker inherits the parent's environment but one variable; the adapter inherits everything; `--grant secret:NAME=VALUE` is argv (visible in process listings) | inheritance | PS-A-05 (empty environment for guests; secrets never on argv for guests) |
| C-10 | `list_dir` on a directory holding a junction to outside | the junction's *name* is listed; following it is refused (C84 holds) | both facts | documented; unchanged |
| C-11 | `--grant fs.read=C:` | means "the working directory on drive C:", silently | current meaning | PS-0-06 (refuse drive-relative spellings) |

Every characterization test is written so that the *fix* turns it red, and is then rewritten as a
refusal witness in the same commit — the pair is the evidence that the fix changed the binary.

## 3. Channel tests (host parser; categories 4 and 6)

- A `cargo-fuzz` target over the frame decoder and the request dispatcher: arbitrary bytes,
  arbitrary-but-valid CBOR, valid frames with out-of-range handles, kinds and spans, frames at
  and over the size bound, truncated frames, a valid frame followed by garbage. Success criterion:
  no panic, no allocation beyond the bound, every input answered with a violation or a refusal
  within the read deadline. (This also discharges `REMAINING_WORK.md` 5.4's "fuzz targets are not
  written" for the component that most needs it.)
- Property tests: every primitive-table row has exactly one request kind and the guest cannot name
  a kind for a primitive it was not granted a handle for; a handle issued to guest A is refused
  from guest B; handle numbers are not guessable across guests (a fresh random space per guest).
- The IPC-1 lesson as a test: a guest that connects and never sends, or sends half a frame, does
  not hang the host past the deadline; a guest that floods frames is throttled or killed, and the
  host's own memory stays bounded (the trace-buffer lesson).
- The audit link: every brokered request that is *performed* has exactly one audit record carrying
  the guest id and the guest's span; a refused request has one refusal record; no record without a
  request (the C29 shape — "allow" written for a request that was refused — as a mutant).

## 4. Policy tests (category 4)

- `derive` is pure: byte-identical output across two runs and across OSes for the guide corpus, the
  book samples and the conformance accept set, pinned like the core-invariance snapshot.
- Every `--grant` kind the runtime parses appears in the policy as *brokered* — no grant can produce
  a guest-side permission (a test that scans the policy for any guest-side allowance and expects
  none).
- Profiles refuse honestly: `hostile-agent` on a host without L2 refuses with the new code and a
  `requires_human: true` repair; `contained` on a host without Landlock runs only if the profile's
  best-effort flag is set, and `host_guarantees` omits `landlock`; a profile requested in
  `delulu.toml` is overridden by the command line and the envelope says which won.
- The manifest ceiling still bounds: a `[sandbox]` table cannot widen `[authority]`; the widening
  is `DL1009`-class, witnessed.

## 5. Boundary tests (categories 4 and 7)

**Generated, not hand-written.** A scenario generator emits guest programs — DeluluLang where the
attempt is expressible through the primitive table (paths, hosts, device names, sizes, counts), and
a tiny native probe binary (Rust, built once per OS, launched *as the guest* by the same launcher)
where the attempt is a raw syscall or API call the language cannot express. Each scenario names its
layer and difficulty, the expected refusal, and the *baseline* (the same attempt from an unjailed
process must succeed, or the test proves nothing).

### 5.1 Filesystem and path family
- **Traversal** — generated relative paths with `..` at every depth, mixed separators, URL-encoded
  and Unicode-normalized dots, over-long components, embedded NUL; expected: `DL0904` at the proxy
  (runtime layer) *and*, under L1, `EACCES`/`ERROR_ACCESS_DENIED` for the native probe (jail layer).
- **Symlinks and junctions** — created by the test harness (not by the guest, which cannot create
  them): dangling, chained, to a sibling, to a parent, to a device, to the broker's state directory;
  expected: refused by the proxy (the C84/SYMLINK-DANGLE witnesses re-run *through the channel*),
  and unreachable by the native probe under L1.
- **Hardlinks** — the P20-R1 boundary re-stated: pinned as a residual at the proxy; under L1 the
  guest cannot create links (Landlock without `MAKE_*`/`REFER`; seccomp without `link*`).
- **Spelling** — the 2026-08-10 search key as a generator: case, `\\?\`, `\\.\`, 8.3 short names,
  trailing dots and spaces, reserved names (`CON`, `PRN`, `AUX`, `NUL`, `COM1..9`, `LPT1..9`, with
  and without extensions), NFC/NFD on macOS, drive-relative `C:x`, UNC; expected: refusal at the
  primitive table (PS-0-06) and, if any spelling slips, the jail layer.
- **Devices** — `/dev/*`, `/proc/self/*`, `/proc/1/*`, `/sys/kernel/*`, Windows device namespace,
  macOS `/dev/*`; the native probe attempts `open` on each; expected: refused at the jail.
- **Host file access** — the broker's state directory, `~/.ssh`, the user's home, the repository
  itself, the guest binary's own path (read-only allowed for the loader only where the OS needs
  it); expected: refused at the jail; a **mutant launcher** that skips Landlock/the restricted
  token/Seatbelt must make these tests fail (the gate must be able to fail).

### 5.2 Process family
- **Spawn/exec** — the native probe calls `fork`, `execve`, `posix_spawn`, `CreateProcess`,
  `ShellExecute`; expected: refused (seccomp; Job Object with the active-process limit; Seatbelt
  `deny process-fork`/`process-exec`).
- **Job breakaway** (Windows) — `CREATE_BREAKAWAY_FROM_JOB`; expected: refused because the job
  denies breakaway; mutant: a job without the limit must fail the test.
- **ptrace / debugging** — `ptrace(PTRACE_ATTACH)` on the host's pid, `OpenProcess` on the parent;
  expected: refused (seccomp denies `ptrace`; restricted token denies `PROCESS_VM_READ`; Seatbelt).
- **Namespace escape** (Linux) — `unshare`, `setns`, `mount`, `pivot_root`, `chroot`; expected:
  refused by seccomp; on hosts where user namespaces are available and used, the probe attempts
  to map uid 0; expected: no capability gain.
- **Signals and IPC** — sending signals to the host (Landlock `SCOPE_SIGNAL`), abstract Unix sockets
  (`SCOPE_ABSTRACT_UNIX_SOCKET`), System V IPC, named pipes by guessing the broker's pipe name
  (the Stage 5 §11 note that a worker *could* reconstruct the default pipe name — now a test that
  it *cannot open it* under L1).
- **Zombies and persistence** — a guest that double-forks, detaches, or spawns a thread that sleeps
  past the wall limit; expected: nothing survives `sandbox kill` or the wall limit (job/cgroup/VM
  kill), verified by scanning for the guest's process tree after the host reports "dead".
- **Timeout bypass** — a guest that ignores SIGTERM, masks signals, or blocks in an uninterruptible
  read; expected: SIGKILL/`TerminateJobObject`/VM stop after the grace period; "killed" is reported
  only after the pid or VMM is verified gone (T10).

### 5.3 Network family
- **Any socket** — the native probe calls `socket`, `connect`, `bind`, `sendto`; expected: refused
  at the jail (no netns / `LANDLOCK_ACCESS_NET_*` empty / AppContainer without network capability /
  Seatbelt `deny network*`); the DeluluLang program cannot express these at all.
- **DNS abuse** — a guest asks the proxy for `http.get` to an allowlisted name that resolves to a
  private, loopback or link-local address (the harness controls resolution through a test
  resolver); expected: the proxy resolves once, pins the address, refuses special-use ranges unless
  explicitly granted by their own spelling (PS-B-02), and never resolves names outside the
  allowlist (no resolver exfiltration channel).
- **Metadata endpoints** — `169.254.169.254`, `fd00:ec2::254`, `169.254.170.2`, `100.100.100.200`,
  `metadata.google.internal`; expected: refused by the proxy unless explicitly granted; refused by
  the jail at the socket layer regardless.
- **Localhost** — the broker's own IPC, the host's LSP/MCP ports, the operator's services;
  expected: refused unless `net=localhost` is granted *and* the proxy is told loopback is intended.
- **Domain fronting and content channels** — recorded as category 7 (the proxy does not inspect
  content); a test pins that the proxy *does* enforce SNI/Host agreement with the allowlisted name.

### 5.4 Secrets and credentials family
- **Environment** — the guest dumps its environment and argv over `println`; expected: empty but
  for the channel handle; no `DELULU_*`, no secrets, no tokens (PS-A-05).
- **Memory** — after `secret_verify`, the harness scans the guest's memory image for the plaintext;
  expected: absent (the plaintext never entered the guest); under `Secret.map` (§8.3 of the
  architecture) the ruling's outcome is the test.
- **Files, stdout, network** — the guest tries to write a secret handle's bytes anywhere; expected:
  the handle carries no bytes; the existing DL0602/0604/0605 witnesses re-run in guest mode.
- **Child processes** — n/a (no spawning); foreign workers and adapters are the host's children and
  get an empty environment (PS-A-05).
- **Core dumps** — a crashing guest must not leave a dump containing host state: the guest holds
  none; the harness asserts the dump (if any) contains no key material.

### 5.5 Resource family
- **CPU** — a spin loop; expected: killed at `cpu_ms` (Job Object process-time limit; RLIMIT_CPU or
  the cgroup; the VMM's cgroup) with the attribution rule (never a widening repair).
- **Memory** — a growing list, an unbounded mailbox (the C-06 program); expected: killed at
  `mem_mb`; the host survives and its own RSS stays bounded.
- **Disk** — `append_text` in a loop; expected: `disk_mb` enforced by the proxy (bytes counted per
  guest) — the jail has no filesystem to fill.
- **Processes/threads** — actor spawn in a loop; expected: bounded by `pids` (Job active-process
  limit counts processes only; threads are bounded by the interpreter's `--actors-threads` and a
  guest-side thread cap); a fork bomb is *modelled* by the native probe attempting to exceed the
  limit N times, never by an actual unbounded bomb.
- **Trace buffer** — `--trace-effects` inside a guest is bounded as today; `--assert-trace`'s
  unbounded sink stays host-side.

### 5.6 Sandbox-to-sandbox and lifecycle family
- **Two concurrent guests** — distinct channels, jails, jobs; each probes for the other's pipe,
  socket, temp directory, cgroup; expected: nothing found or refused; the audit log attributes every
  effect to the right guest (T11).
- **Shared state directory** — two guests under one broker; expected: neither can enumerate the
  other's node (the broker serves the *host*, and the host holds one node per guest).
- **Cleanup races** — kill during launch, kill during a brokered effect, host death mid-run (the
  harness kills the host): expected: no orphan process, no orphan jail directory, no orphan vsock
  socket; a sweep on the next start reports what it removed.
- **Restart** — a guest killed for a limit is not restarted silently; the CLI exits 1 with the
  diagnosis.

### 5.7 Artifact and plugin family
- **Malformed `.dwx` / `.dpx`** — the existing DL1202/DL1508 witnesses re-run through a guest
  that asks the host to load them; expected: identical verdicts (criterion 9's spirit).
- **Malicious plugin** — a Contained plugin that exhausts fuel or memory, or a Verified DIR that
  fails re-check, loaded from a guest; expected: the host refuses or kills the plugin, the guest
  gets `PluginErr`, the guest survives, the host survives.
- **Malicious dependency** — a path dependency whose authority widens after the pin; expected:
  `DL1001`, unchanged by sandboxing (the check is at `build`).
- **Guest image** (L2) — a flipped byte in the initramfs or kernel; expected: refused before
  launch (T13); the image hash in the `sandbox-launch` audit record matches the manifest.

### 5.8 Kernel and hypervisor surfaces (L2)
- The native probe inside the VM enumerates devices (`/sys/bus/virtio`, `/proc/net/dev`); expected:
  exactly one vsock device, no NIC, no block, no fs; `connect` fails at socket creation.
- The VMM API socket and the jail directory are unreachable from the guest by construction; a test
  confirms the guest cannot even name them (they are host paths).
- Kernel exploits are **category 7**: no test claims to prove their absence; the published
  difficulty-4/5 scenarios of SandboxEscapeBench are *out of scope for a pass/fail gate* and are
  run, if at all, as a recorded experiment.

## 6. Mutants — the gates that must be able to fail

For every boundary the launcher applies, a **mutant launcher** exists behind a test-only flag
(never shipped in a release build; a `cfg(test)`/env-var gate is refused in release) that skips
exactly one control: no Landlock, no seccomp, no restricted token, no Job Object limit, no Seatbelt
profile, no wall watchdog, no image hash check, no read deadline. The suite runs each boundary test
against the real launcher (must pass) and against each mutant (the corresponding tests **must
fail**). A mutant that changes nothing observable is itself a finding (the 2026-08-10 lesson: a
falsification that does not change the binary proves nothing).

## 7. Measurement (not a gate)

Recorded once per level per OS, methodology in the record: launch cost (spawn to first channel
frame), round-trip cost per brokered effect (a `println` loop of N; a `read_text` of 1 KB and
1 MB), throughput of the channel, and the interpreter's compute speed inside the guest versus
in-process (expected: identical, since compute never crosses). These numbers decide PS-B's batching
work and are published beside Study C in the same honest voice.

## 8. What this plan does not claim

It does not claim kernel or hypervisor correctness, side-channel resistance, or anything about an
L3 environment. It claims that, for the listed scenario families, the listed layer refuses the
listed move, that each refusal was witnessed against a mutant that let it through, and that the
current binary's residuals are pinned so that they cannot be quietly forgotten.
