# Red-team note — execution attack surfaces and the adversarial sandbox test matrix

**Author:** sous-chef (Opus 5), 2026-09-17. **Read-only pass.** Nothing was implemented, no existing
file was modified, nothing was built, nothing was pushed. This is the only file written.

**Binary driven:** `D:\nelan\DeluluLang\target\release\delulu.exe`, `delulu --version` →
`delulu 1.0.0`. I did not rebuild it; per `VERIFICATION_FINDINGS.md` the head chef built it from
`52eecbe` on 2026-09-17. All probes ran on Windows 11 with `DELULU_NO_FIRST_RUN=1`,
`DELULU_COLOR=never` and `DELULU_STATE_DIR` pointed at a scratch directory under the session
scratchpad. No probe wrote outside that scratchpad; no probe sent a byte to a network.

**Source read:** the worktree at `D:\nelan\DeluluLang\.claude\worktrees\agent-ab8e3c649dff8d3e7`.
Line numbers are that tree's.

**Builds on, does not redo:** `VERIFICATION_FINDINGS.md` NE-01…NE-16. Existing findings referenced
by their recorded ids (C84, C85, C86, C87, CONTAIN-TOCTOU-1, SYMLINK-DANGLE-1, P20-R1, ADAPTER-LINE-1,
IPC-1, DEADMAN-1, DISC-1, GUARD-SPELL-1) are the project's, not mine.

**Honesty convention used throughout:** *verified* = I ran it and pasted the output. *code claim* =
read in the source, cited, not executed. *not verified* = stated as unknown.

---

## 0. The one-sentence state of the isolation story

`--isolation none` and `--isolation process` put the program-under-execution **in the `delulu`
process**; `process` differs only in that it sets `foreign_isolation = process`
(`run_cmd.rs:246-252`), and `microvm` is refused before anything runs (`run_cmd.rs:216-241`). So
today **there is no OS sandbox beneath the runtime at all** — the containment is the language, the
capability checks in `prim.rs`, and the operator's `--grant` flags. Everything in §1 follows from
that.

---

## 1. Attack-surface map

Column *enforced by* answers "what actually stops this today". "Future layer" is what an OS jail
beneath the runtime would have to do for the residual to disappear.

| # | Surface | Enforced today by | `file:line` | Residual / evidence | What an OS layer beneath the runtime must enforce |
|---|---|---|---|---|---|
| 1 | **FS read** (`read_text`) | runtime: lexical `..` check **then** `canonicalize`-both-sides prefix check | `prim.rs:395-401` → `resolve_in_scope` `prim.rs:158`, `contains_on_disk` `prim.rs:147`, `canonical_existing` `prim.rs:76` | **CONTAIN-TOCTOU-1**: check-then-open; a concurrent writer can swap the checked path (`prim.rs:821-841`, `DEPLOYMENT.md` §5). Junction escape refused — **verified**, probe P1. | mount namespace / bind-mount of exactly the granted subtree (`openat2(RESOLVE_BENEATH)` on Linux); then the race has nothing outside to point at |
| 2 | **FS write** (`write_text`, `append_text`) | same two gates; then `fs::write` / `OpenOptions::append` | `prim.rs:422-435` | Same TOCTOU. **No quota, no rate limit, no file-count bound** — **verified**, probe P2 (20,000 appends, 1.3 MB, exit 0, nothing complained). Windows **device-name escape**, see row 20. | read-only bind mount unless write-granted; a filesystem quota or a size-capped overlay/tmpfs |
| 3 | **Directory listing** (`list_dir`) | the same `resolve_in_scope` | `prim.rs:403-418` | Returns every entry name unbounded and with no type; it **discloses the existence and name of a link aimed outside the grant** even though following it is refused — **verified**, probe P1 printed `jn`. Metadata leak, not an escape. | the guest simply does not see siblings — nothing to enumerate |
| 4 | **Network egress** (`http.get`) | runtime: `https://` scheme check, then exact/`*.`-suffix host allowlist; **then the call always returns `Err(Refused)` — v1.x bundles no HTTP client** | `prim.rs:437-450`; mint at `prim.rs:273-280`; `host_allowed` `prim.rs:206`; `host_of` `prim.rs:190` | No byte leaves today (`prim.rs:449` comment, and **verified** in probe P3). The **decision** and the **audit record** are already live and are made on a **name**. `grep` for `reqwest|ureq|hyper|curl|to_socket_addrs|lookup_host` over `crates/` returns nothing outside `delulu-registry`'s own test server. | default-deny egress; a host-side proxy that re-checks the **address** it connects to, not the name it was handed |
| 5 | **DNS** | **nothing — the runtime never resolves a name.** There is no resolver, no library, no `to_socket_addrs` call on the program's path. | absence, verified by the grep above | The allowlist is therefore a **name** check with **no address check anywhere**. The moment a client is added: allowlisted-name→private-address (rebinding / a hostile authoritative server) is wholly unhandled, and `169.254.169.254` / `localhost` / RFC1918 are not special-cased anywhere — **verified**, probe P3. | resolve inside the jail against a pinned resolver, pin the resolved address, and re-check it against the policy *after* resolution and *at connect time* |
| 6 | **Secrets in memory** | language: `Secret[T]` is opaque; `expose` is the only exit and is `Declassify`-gated + trace-redacted | `trace.rs:19` (`OPAQUE`), redaction decided in `interp.rs`; grant parse `broker.rs:153-163` | In embedded mode the **plaintext lives in the process** (`broker.rs:279-286`: "Embedded mode: secrets carry their bytes in `secrets`"). `--grant secret:NAME=VALUE` takes the literal from **argv** unless prefixed `env:` (`broker.rs:155-160`) → visible in `ps` / Task Manager to any same-user process. | a separate address space; and the broker-resident secret path of spec §5 (bytes enter only on `expose`) rather than the embedded cache |
| 7 | **Secrets / env into children** | **nothing.** Neither child-spawning path clears the environment. | foreign worker: `build_worker_command` sets only `DELULU_STATE_DIR` (`foreign_worker.rs:243-257`); adapter: `Command::new(program).args(...)` with no `env` call (`adapter.rs:141-149`) | The full parent environment — including anything an operator put there for `secret:NAME=env:VAR`, plus `DELULU_GUARD_OWNER` — is inherited by both the foreign worker and the operator-supplied adapter. **Code claim**, not executed. | `env_clear()` plus an explicit allowlist, and the jail's own empty environment |
| 8 | **stdout / stderr** | nothing beyond the `Console` grant | `prim.rs:378-386` | Program output is written raw to the operator's terminal; so is a hostile **adapter's stderr**, which is `Stdio::inherit()` **on purpose** (`adapter.rs:146-147`). Terminal-escape injection from untrusted subprocess output is unmitigated. **Code claim.** | nothing an OS jail fixes — this is an output-sanitisation decision at the CLI |
| 9 | **stdin** (`readline`) | the `Console` grant only | `prim.rs:387-393` | `std::io::stdin().read_line(&mut line)` with **no length bound**. The project already fixed exactly this shape for the adapter (ADAPTER-LINE-1, `MAX_LINE_BYTES = 64 KiB`, `adapter.rs:89`) and did not apply it here. Memory consequence **not verified** by execution. | a pipe the jail controls; but the right fix is `take(N)`, as the adapter does |
| 10 | **Broker IPC — current in-process model** | **nothing separates them.** The program *is* the `delulu` process that holds the broker client. | `run_cmd.rs` (the run path); transport `broker_transport.rs:168` (Win, owner-only DACL + peer-SID `:239-263`) / `:454-470` (Unix, `0700` dir) | The program cannot *speak* the protocol because DeluluLang has no socket primitive — that is a **language** bound, not an OS one. Any foreign C code (row 12) is in the same address space under the default `inproc` and can. | a jail that does not map the socket / pipe at all |
| 11 | **Broker IPC — hypothetical child-process model** | would be: the `0700` dir (Unix) / owner-only DACL + peer-SID (Windows) | same lines | Both admit **any same-user process**, by construction (`broker_transport.rs:5`, `MATHEMATICS.md` §12 item 11). A child of the run would inherit the same uid and therefore be admitted. Non-disclosure (not handing over the address) is the only barrier — and `foreign_worker.rs:23-28` says so in those words. | a distinct uid / user namespace, or a socket passed only as an already-open fd into the jail |
| 12 | **Foreign workers** (`--foreign-isolation process`) | OS: Windows Job Object `KILL_ON_JOB_CLOSE` (`foreign_worker.rs:567-620`), Linux `PR_SET_PDEATHSIG` (`:529-545`); null stdio; `DELULU_STATE_DIR` redirected | `foreign_worker.rs:243-257`, `:260-336` | (a) **No read timeout on the worker channel** — `set_read_timeout` appears only at `brokerd.rs:867` and `:918`; `WorkerConn::call` (`foreign_worker.rs:351-368`) does a bare `read_frame`. A foreign function that hangs blocks the host **forever**. This is the IPC-1 defect shape at a site the IPC-1 fix did not reach. (b) seccomp is a **documented stub** (`foreign_worker.rs:34-36`). (c) The channel dir is a predictable path under `std::env::temp_dir()` — `delulu_fw_<pid>_<n>` (`foreign_worker.rs:233`) — and `create_dir_all` succeeds on a pre-existing directory; on a shared `/tmp` that is a squat surface. (d) Full env inherited (row 7). All four are **code claims**; I ran no foreign library. | seccomp/landlock in the worker; a private `/tmp`; the read bound is a code fix, not an OS one |
| 13 | **Hardware adapter subprocess** | runtime: envelope validated **host-side before dispatch** (`adapter.rs:49-54`); per-exchange `recv_timeout`; poison-on-failure; `MAX_LINE_BYTES` 64 KiB | `adapter.rs:89`, `:141-180`, `:205-235`, `:285-290` | Operator-supplied program, **no signature check, not sandboxed** (`DEPLOYMENT.md` §5; memory: D23). Inherits stderr (row 8) and the environment (row 7). It is a full OS process with the operator's privileges — the envelope bounds what DeluluLang *asks* it to do, never what it does. | run the adapter in its own jail with only the device node it needs |
| 14 | **Plugins — `Verified`** | runtime: full DIR replay + per-export row re-check, never falls back to `Contained` (`plugin.rs:493-560`) | `plugin.rs:513` `step5_verified` | **A program cannot load one at run time at all** — NE-01, `prim.rs:366`. So the attack surface is `delulu plugin *` (operator-driven), not the program. | n/a while NE-01 stands |
| 15 | **Plugins — `Contained` (WASM)** | engine: whitelist of `(namespace, name, exact signature)` imports derived from the grant (`plugin.rs:282-340`); store fuel / memory limiter / epoch wall watchdog (`limits.rs:230+`, defaults 50 M fuel / 64 MiB / 5 s at `limits.rs:38-44`); trap cause attributed from evidence only (`limits.rs:178-215`) | as cited | **Refused outright on Windows** (`limits.rs:245-252`, `EnforcementUnsupported`) because the trap unwind fastfails the host. So the one path in the whole product with real CPU/memory/wall limits does not run on one of the two verified platforms. | a jail makes the Windows refusal unnecessary: limits enforced outside the process cannot fastfail it |
| 16 | **Main program on the WASM engine** | the same capability checks, a **smaller** surface (no `fs_write`, no `http`, no `readline` in the linker — `host.rs:504-1000`); hardened feature set (`wasm_simd/threads/memory64/component_model` all off, `host.rs:61-67`) | `host.rs:1132` `run_main`, store at `:1136` | **No fuel, no memory limiter, no epoch deadline** on the main-program store — `Store::new(&engine, state)` with none of them. The limits live only in `limits.rs`'s separate Contained path. So `--engine wasm` is *not* a resource bound. | same as row 19 |
| 17 | **Actors / threads** | language: actors are entries on a fixed worker-thread pool, not one OS thread each (`actors.rs:322-390`); mailboxes bounded **only if configured** (`actors.rs:208-231`) | `actors.rs:210` `spawn` | **Verified, probe P5: a program granted only `console` reached 1,200 MB RSS in 30 s and had to be killed.** `spawn`/`send` need no grant at all — the `Async` effect is not a `--grant` key. Default mailbox is unbounded ("no config = unbounded (1.0 behavior)", `actors.rs:209-210`). | a memory cgroup / Job Object memory cap on the whole run |
| 18 | **Process lifecycle** | Windows: Job Object kill-on-close + explicit `kill()`+`wait()` in `WorkerGuard::drop` (`foreign_worker.rs:392-404`); Linux: `PR_SET_PDEATHSIG` | as cited | macOS/BSD have **neither** — `set_pdeathsig` is `#[cfg(target_os = "linux")]` (`foreign_worker.rs:529-545`), so on a Mac an orphaned worker survives a hard host kill; only the `Drop` guard (which a `SIGKILL` skips) covers it. The adapter child is killed on `Drop` only (`adapter.rs:285-290`) — same gap. **Code claim.** | a process group / jail that the kernel reaps as a unit |
| 19 | **Resource exhaustion (CPU / memory / wall)** | **nothing.** Only a call-depth bound: `DEFAULT_MAX_DEPTH = 10_000` (`interp.rs:33`) → DL0905. | `interp.rs:33`, and the absence of any limit flag | `delulu run --help` lists `[--json] [--grant] [--grant-manifest] [--no-prompt]` — **verified**; `grep` for `--timeout/--max-*/--cpu/--wall` in `cli.rs` finds none. `DEPLOYMENT.md` §5 says so plainly: "Authority is the containment, not a quota." | cgroups v2 (`cpu.max`, `memory.max`, `pids.max`) / a Job Object with `ProcessMemoryLimit` + `JobMemoryLimit`, and a wall-clock killer |
| 20 | **Device access (Windows reserved names)** | **nothing — and containment actively mis-approves it.** | `canonical_existing` `prim.rs:76-100` re-appends a non-symlink final component onto the canonicalized parent | **Verified, probe P6/P8:** under `--grant fs.write=.`, `write_text("NUL", …)` → `Ok` and **no file named `NUL` was created** (directory empty afterwards); `write_text("CON", …)` → `Ok`. The path passes as `<grant>\NUL`, the Win32 parser resolves it to a device object. `COM1`–`COM9` / `LPT1`–`LPT9` / `AUX` / `PRN` are the same construction — **not verified here** (this machine reports no serial ports). The trace record says `detail: "CON"`, i.e. it attests a filesystem write inside the grant that never happened. | a jail with no device nodes mounted; on Windows, an explicit reserved-name refusal in `resolve_in_scope` (there is no OS layer that fixes this) |
| 21 | **Path spelling / confusion** | lexical `..` rejection + `canonicalize` both sides | `prim.rs:158-176` | **Verified, probe P9:** `trail.txt.` created `trail.txt`; `space.txt␠` created `space.txt` — the Win32 parser strips trailing dots/spaces, so two grant-internal spellings name one file while the trace records the unnormalized string. Inside one grant this escapes nothing; it breaks any **per-file** policy or audit key. `NUL.txt` created a real file. 8.3 aliasing **inconclusive** — this volume generated no alias for `inside.txt`. | nothing; this is a normalisation bug in the decision + record, the class of C86 / GUARD-SPELL-1 |
| 22 | **Grant spelling (operator side)** | `Grants::add` (`broker.rs:152-190`); C87 refuses an empty value | `broker.rs:173-190` | **Verified:** `--grant fs.read=C:` is accepted and means **the current directory on drive C**, not the drive — `root.fs_read("..")` was still refused, identical to `--grant fs.read=.`. `--grant "fs.read=\\?\<abs path>"` is accepted silently and then grants **nothing**: `root.fs_read(".")` on that exact directory → DL0703. Fail-closed, but an operator gets no signal that their grant is inert. | n/a — a CLI validation fix |
| 23 | **Trace buffer** | bounded at `TRACE_RECORD_CAP = 200_000` for `--trace-effects` (`cli.rs:784`, `run_cmd.rs:620`, `:734`) | `trace.rs:154-193` | `--assert-trace` asks for an **unbounded** sink deliberately (`trace.rs:143-148`, `run_cmd.rs:618`): dropping records there would be a fail-open on a security-adjacent check. So `--assert-trace` on a long effectful run is an unbounded-growth path, consciously chosen. | a memory cap on the run makes it a kill rather than a swap-death |
| 24 | **Metadata endpoint class** | **nothing** — see rows 4 and 5 | `prim.rs:206-224` | `--grant net=169.254.169.254` is accepted with no warning and `https://169.254.169.254/…` passes `host_allowed` — **verified**, probe P3; the only reason nothing happened is the missing client. Nothing anywhere knows that link-local, loopback or RFC1918 are special. | default-deny egress plus an explicit address-class deny list in the proxy |
| 25 | **`--isolation` label on the machine channel** | human stderr line only (`run_cmd.rs:253-266`, guarded by `if !opts.json`) | `run_cmd.rs:254` | **Verified:** `run … --isolation process --json` prints **no** isolation label and no envelope on success. Spec §6.1's "Output label" row is honoured on the human channel only — and the audience for `--json` is exactly the agent orchestrator that most needs to know. | n/a — an envelope fix |

---

## 2. Probes I ran

Every command below was run with `DELULU_NO_FIRST_RUN=1`, `DELULU_COLOR=never`,
`DELULU_STATE_DIR=<scratch>\state`; `<scratch>` is the session scratchpad directory. Programs are in
`<scratch>\redteam\*.delulu` (never in `examples/` or `tests/`). Exit codes are the process's own.

**P1 — junction inside a grant, aimed outside.** Prepared `mklink /J <scratch>\granted\jn
<scratch>\outside`, cwd `<scratch>\granted`.
`delulu run p1_listdir.delulu --grant console --grant "fs.read=."` →
```
entries = 2
  inside.txt
  jn
error[DL0904]: path `jn/crown.txt` escapes the granted scope
```
exit 1. **C84 holds, live.** Note two things: the escape is a hard fault that *terminates the
program*, not an `Err` value; and `list_dir` disclosed `jn`.

**P2 — unbounded disk growth.** `delulu run p2_append.delulu --grant console --grant "fs.write=."` →
`appended 20000 lines`, exit 0, elapsed 6,917 ms, `grow.log` = **1,300,000 bytes**. Nothing bounds
it, nothing warns.

**P3 — the metadata address.**
`delulu run p3_http.delulu --grant console --grant "net=169.254.169.254" --trace-effects` →
```
metadata Err (value, not refusal)
{"seq":0,"effect":"Net","op":"get","cap_kind":"Http","detail":"https://169.254.169.254/latest/meta-data/",…}
```
exit 0. The grant was accepted; the host check passed; the `Err` came from `prim.rs:449` (no client),
not from any policy. `delulu authority p3_http.delulu` reports `Http (scope granted at runtime)` —
the requested host is absent from the report, which is NE-10/NE-14, unchanged.

**P4 — actor spawn count.** 50,000 `spawn`s with only `--grant console` → `spawned 50000`, exit 0,
402 ms. Cheap: no OS thread per actor.

**P5 — unbounded mailbox.** One actor with a slow behaviour, 3,000,000 sends, `--grant console` only.
`main` printed `sent 3000000`; peak working set reached **1,200.2 MB** and I killed it at my own 30 s
cap (it had not finished draining). **A program with no filesystem, network or secret grant can
exhaust host memory.**

**P6 / P8 — Windows device names.** cwd `<scratch>\dev`, `--grant fs.write=.`:
`write_text("NUL", …)` → `NUL: Ok`, exit 0, and `Get-ChildItem <scratch>\dev -Force` afterwards
listed **nothing** — the bytes went to the device, no file was made. `write_text("CON", …)` → `CON:
Ok` with trace `{"effect":"Write","op":"write_text","cap_kind":"FsWrite","detail":"CON",…}`.
`[System.IO.Ports.SerialPort]::GetPortNames()` on this machine returned empty, so `COM1` was **not**
tested — the mechanism is identical but I am not claiming it.

**P9 — trailing dot / space / `NUL.txt`.** `--grant fs.write=.`, all three `Ok`; the directory
afterwards contained `[NUL.txt] 1`, `[space.txt] 12`, `[trail.txt] 10` — the trailing dot and space
were stripped by the OS, `NUL.txt` became a real file.

**P10 — grant spellings.** `--grant fs.read=C:` → `root.fs_read("..")` refused DL0703, byte-identical
to `--grant fs.read=.`. `--grant "fs.read=\\?\<abs granted dir>"` → `root.fs_read(".")` refused
DL0703 (the grant is inert). `--grant fs.read=..\outside` → `root.fs_read(".")` correctly refused.

**P11 — the isolation label.** Same program, `--isolation process`: human channel prints
`isolation: process — foreign code in minimum-privilege worker subprocesses; …`; adding `--json`
prints **no label at all**. `delulu run --help` lists only
`[--json] [--grant K[=V]]... [--grant-manifest] [--no-prompt]`.

Not run, deliberately: any fork bomb (modelled in §3), anything touching a real serial port, any
network request, any write outside the scratchpad.

---

## 3. The adversarial test matrix

*Layer* = where the defence belongs: **L**anguage / **R**untime / **J**ail (OS) / **H**ypervisor.
*Today* = ✅ runnable now as a **characterization** test pinning the current residual;
⛔ needs the sandbox layer to exist first.
*Generator* = how a generator produces many variants (house rule: generate inputs, do not write
examples).

### 3.1 Traversal, symlink, hardlink, spelling

| Attacker's move | Layer | Code path / future component | Generator | Today |
|---|---|---|---|---|
| `read_text("../../etc/passwd")` and 200 encodings of it | R | `resolve_in_scope` `prim.rs:158` | cross-product of separators × `.`/`..` runs × URL/UTF-8/overlong encodings × depth 1-40 | ✅ |
| Symlink inside the grant → outside, target **exists** | R | `contains_on_disk` `prim.rs:147` | generate link topologies: (depth, chain length, relative vs absolute, loop yes/no) | ✅ |
| Symlink inside the grant → outside, target **absent** (SYMLINK-DANGLE-1) | R | `canonical_existing` `prim.rs:76-100` | as above, with target existence as a toggled bit — the bit that flipped the verdict | ✅ |
| Link delivered **by the workspace** (git mode `120000`) | R+J | the clone/`delulu add` path | generate repos whose tree contains links aimed at N escape targets | ✅ |
| Hardlink inside the grant to an outside file (P20-R1) | J | documented boundary, `prim.rs:800-820` | generate (inode fan-out × creation order × grant shape) | ✅ (pin the boundary) |
| Win32 junction / mount point (C84) | R | `contains_on_disk` | junction vs symlink-D vs `\??\` object-manager reparse, × 3 depths | ✅ (P1 shape) |
| **Trailing dot / trailing space** spellings of one file | R | `resolve_in_scope`; the trace/audit key | generate name × {"", ".", " ", ". ", "..", "  "} × {ASCII, unicode} | ✅ (P9 shape) |
| **8.3 short name** of a long name on an 8dot3-enabled volume | R | same | generate long names, query the alias via `GetShortPathName`, feed both spellings | ✅ (my run was inconclusive — needs a volume with 8dot3 on) |
| `\\?\`, `\\.\`, `\\?\UNC\`, `//?/` spellings in the **grant** and in the **program path** | R | `Grants::add` `broker.rs:173`; `resolve_in_scope` | the 6 verbatim prefixes × {grant side, program side} × {absolute, relative} | ✅ (P10 shape) |
| Case-only difference on a case-insensitive volume; NFC vs NFD on macOS | R | `contains_on_disk` prefix compare | generate every name in both normal forms and both cases | ✅ on Win; macOS ⛔ (no Mac here) |
| Concurrent swap of a checked path between check and open (CONTAIN-TOCTOU-1) | J | `prim.rs:821-841` | a racer thread flipping file↔symlink at N µs offsets; run K trials, report the hit rate | ✅ (expect it to succeed — that is the point) |

### 3.2 Device access, process and namespace escape

| Attacker's move | Layer | Code path / future component | Generator | Today |
|---|---|---|---|---|
| `write_text("NUL"/"CON"/"COM1"/"LPT1"/"AUX"/"PRN")` inside a write grant | R | `canonical_existing` `prim.rs:76` re-append | enumerate all 22 reserved names × {bare, `NAME:`, `NAME.txt`, `NAME `, `NAME.`} × {read, write, append, list_dir, narrow} | ✅ (**P6/P8 already show `NUL` and `CON` pass**) |
| `read_text("/dev/mem")`, `/proc/self/mem`, `/proc/<pid>/environ` under a `/`-adjacent grant | J | none today | generate the `/dev`, `/proc`, `/sys` name space and grant shapes that contain them | ✅ on Linux CI |
| Program spawns a process | L | **no spawn/exec primitive exists in the language** — pin that with a refusal test | generate every `Root`/`Cap` method name and assert the prim table has no exec arm (`prim.rs` match) | ✅ |
| Foreign C code calls `fork`/`CreateProcess`; grandchild escapes the Job Object with `CREATE_BREAKAWAY_FROM_JOB` | J | `foreign_worker.rs:567-620` sets `KILL_ON_JOB_CLOSE` but **not** `JOB_OBJECT_LIMIT_BREAKAWAY_OK=0`/`SILENT_BREAKAWAY` | generate C stubs: fork depth × breakaway flag × detach method (`setsid`, `daemon`, `DETACHED_PROCESS`) | ✅ (needs a C toolchain in the harness) |
| Foreign code `ptrace`s / `OpenProcess(PROCESS_VM_READ)`s the host and reads the broker key | J | seccomp stub, `foreign_worker.rs:34-36` | generate the syscall set a worker actually needs, then a stub per denied syscall | ⛔ (needs seccomp/landlock) |
| Worker survives host `SIGKILL` on macOS/BSD (no `PR_SET_PDEATHSIG`) | J | `foreign_worker.rs:529-545` | matrix: OS × kill signal × worker state (idle/in-call/hung) | ✅ on a Mac runner |
| Adapter subprocess re-parents itself and outlives the run | J | `adapter.rs:285-290` (`Drop` only) | adapter stubs: {exit, ignore SIGTERM, double-fork, spawn grandchild, `setsid`} | ✅ |
| Mount / user / PID namespace escape from the guest | J+H | future jail | fuzz the jail's own config surface | ⛔ |

### 3.3 Network, DNS, localhost, metadata

| Attacker's move | Layer | Code path / future component | Generator | Today |
|---|---|---|---|---|
| Host-string confusion: userinfo `@`, `*.` suffix, IPv6 brackets, port, case, trailing dot, percent-encoding, `\` for `/`, CR/LF injection | R | `host_of` `prim.rs:190`, `host_allowed` `prim.rs:206` | a URL grammar generator (scheme × userinfo × host form × port × path × fragment), **differentially** compared against a real WHATWG parser — the bug is a *disagreement*, not a crash | ✅ (pure, fast, and the exact shape that found C85/C86) |
| Allowlisted name resolves to `127.0.0.1` / `169.254.169.254` / RFC1918 (rebinding) | R+J | **no resolver exists** — belongs to the future egress proxy | generate (name, A-record sequence) pairs including TTL-0 flips mid-connection | ⛔ |
| Exfiltration through the resolver itself (`<base32-payload>.attacker.tld`) | J | future egress proxy + DNS policy | generate payload sizes × label chunking × query types (A/AAAA/TXT/NULL) | ⛔ |
| Direct connect to `169.254.169.254` / `::ffff:169.254.169.254` / `0xA9FEA9FE` / `2852039166` | R+J | `host_allowed` has no address awareness | enumerate every textual form of the same 32-bit address (dotted, decimal, octal, hex, mixed, IPv4-mapped IPv6) | ✅ as a *decision* test (assert the grant/allowlist verdict), ⛔ as a *connection* test |
| `localhost` reaching the broker's own listener, the registry server, or a sibling run | J | `broker_transport.rs`; `delulu-registry/src/lib.rs:438` | generate loopback aliases (`localhost`, `127.0.0.1`, `127.0.0.2`, `[::1]`, `0.0.0.0`) × ports | ⛔ (needs a client) |

### 3.4 Credential, env and secret leakage

| Attacker's move | Layer | Code path / future component | Generator | Today |
|---|---|---|---|---|
| Secret written to a granted file / printed / sent | L | `Secret[T]` opacity + `Declassify`; trace redaction `trace.rs:19` | generate every sink (`write_text`, `append_text`, `println`, `http.get` URL, actor `send`, foreign arg) × every laundering path (`str()`, concat, list, record field, `map`, `verify`) and assert the checker refuses or the trace says `«opaque»` | ✅ |
| Secret read out of **argv** by a same-user process | J | `broker.rs:155-160` (literal from the command line) | generate grant forms {literal, `env:VAR`, store} and assert argv never carries plaintext | ✅ (characterization: it does today) |
| Environment inherited by the foreign worker / the adapter | J+R | `foreign_worker.rs:243-257`, `adapter.rs:141-149` | seed N marker variables in the parent, have the child dump its own environ, diff | ✅ |
| Secret recovered from a **core dump** / crash report of the host | J | none | force a crash at K points after `expose` and grep the dump | ✅ (Linux) |
| Plaintext resident in the embedded secret cache before `expose` | R | `broker.rs:279-286` | memory-scan test (the project already has a Stage-3 scan test to extend) | ✅ |

### 3.5 Exhaustion, timeout bypass, persistence

| Attacker's move | Layer | Code path / future component | Generator | Today |
|---|---|---|---|---|
| Unbounded mailbox growth (**P5: 1.2 GB, no grant needed**) | R+J | `actors.rs:208-231` | generate (actor count, send rate, payload size, behaviour cost, mailbox config present/absent) and record RSS-vs-time; the assertion is a **cap**, not a shape | ✅ |
| Tight loop with no effect, forever | J | nothing; no `--timeout` exists | generate loop bodies (pure, allocating, effectful, recursive-under-10 000) × durations | ✅ (as a "no bound exists" pin) |
| Disk fill via `append_text` (**P2**) | J | `prim.rs:422-435` | generate (chunk size, file count, path depth, sparse/dense) until a quota fires — today none does | ✅ |
| Inode / file-count exhaustion in the granted dir | J | same | generate N distinct names per second | ✅ |
| Fork bomb from foreign code — **model, never run** | J | `pids.max` / Job Object `ActiveProcessLimit` | generate the *shape* (breadth, depth, spawn rate) and assert the limit refuses at K, in a disposable VM only | ⛔ |
| Foreign worker that accepts a call and never replies → host hangs forever | R | **`WorkerConn::call` `foreign_worker.rs:351-368` sets no read timeout** | stub library: {return, sleep(∞), exit(0), abort, close the pipe, reply twice, reply to the wrong call} | ✅ (needs a C stub; this is the highest-value single test in the table) |
| Adapter that stalls, floods, or replies late | R | `adapter.rs:205-235` (already bounded + poisoned) | the same stub matrix over the line protocol, plus a 64 KiB+1 line | ✅ (regression pins for ADAPTER-LINE-1) |
| A program that ignores a kill / a worker that outlives its host | J | `foreign_worker.rs:392-404`, `:529-545` | OS × signal × child state | ✅ |
| `--assert-trace` unbounded sink | R | `trace.rs:143-148` (deliberate) | generate effect rates × run lengths; assert the *documented* trade-off still holds | ✅ |

### 3.6 Sandbox-to-sandbox, lifecycle, artifacts, supply chain

| Attacker's move | Layer | Code path / future component | Generator | Today |
|---|---|---|---|---|
| Two concurrent runs sharing one `DELULU_STATE_DIR` | R | `brokerd.rs:858-890` — the serve loop is **single-connection, serial**, 5 s read timeout | generate (N clients × request mix × stall pattern) and measure the e-stop revoke's latency under load | ✅ |
| One run squats the next run's predictable worker channel dir | J | `foreign_worker.rs:233` `delulu_fw_<pid>_<n>` | generate pid/seq guesses and pre-create the directory + a listening socket | ✅ on Linux (shared `/tmp`); on Windows temp is per-user |
| Two runs racing on one granted directory (both write `out.txt`) | J | `prim.rs:422` | generate interleavings at µs offsets; assert only "no corruption of the *broker's* state", since file races are the program's problem | ✅ |
| Run A reads run B's leftover temp / channel dir after a crash | J | `WorkerGuard::drop` skips on `SIGKILL` | crash the host at K points, then enumerate what survived | ✅ |
| Malformed `.dwx` / `.dpx` artifact (truncated, bit-flipped, hostile manifest, signature mismatch, wrong class) | R | `plugin.rs:331` (import slice), `:513` (`step5_verified`), `:419` (class never downgraded) | structure-aware mutation over a valid artifact: field-wise flips, length lies, duplicate sections, class swap Verified↔Contained | ✅ |
| WASM module using a disabled proposal (SIMD/threads/memory64/component model) | R | `host.rs:61-67` | generate one module per disabled proposal and assert **validation** refuses (not codegen) | ✅ |
| Contained plugin that exhausts fuel / memory / wall, or traps for a **non**-limit reason | R | `limits.rs:178-215` attribution | generate guests: {infinite loop, `memory.grow` storm, `unreachable`, div-by-zero, OOB} × limit settings; assert DL1506 **only** for real limits | ✅ on Linux; **refused on Windows** (`limits.rs:245`) |
| Dependency pinned by content hash, then swapped at the registry | R | `delulu-registry/src/lib.rs` (server-side authority recomputation) | generate (index line, artifact) pairs that disagree; assert the server discards the client's claim | ✅ |
| Guest image / rootfs tampering; snapshot contamination between runs | H | future microVM component | generate image mutations; and for snapshots, generate (secret written pre-snapshot, restore, read) triples | ⛔ |

### 3.7 IPC and the broker

| Attacker's move | Layer | Code path / future component | Generator | Today |
|---|---|---|---|---|
| Same-user process connects to the broker pipe/socket and issues `Issue` (DISC-1) | J | `brokerd.rs:288` | generate request bodies from the `ReqBody` grammar; assert LEGACY accepts and STRICT refuses DL1421 | ✅ (this is the DISC-1 repro; keep it passing as a *pin*) |
| Malformed / truncated / oversized frame; connection churn | R | `brokerd.rs:858-890` | CBOR-grammar fuzz × frame-length lies × slow-loris dribble | ✅ |
| A run's own foreign worker reaches the broker (criterion 7b) | J | `foreign_worker.rs:20-28` + the `--probe-broker` diagnostic mode | generate worker environments: inherited `DELULU_STATE_DIR`, guessed default path, argv leak, open-fd scan | ✅ (the binary already has the probe subcommand) |
| Peer-SID / `0700` check bypass on a filesystem that ignores `chmod` (P21-F1) | J | `broker_transport.rs:454-470`, `signing::set_owner_only_or_warn` | generate filesystem types (9p, DrvFs, NFS, SMB, exFAT, tmpfs) × state-dir placement | ✅ on WSL |

---

## 4. Architectural mistakes and limitations I see

Each is a claim to verify before use.

**A1 — `--isolation` names a *program* property but only configures *foreign* code.** `run_cmd.rs:246-252`
turns `--isolation process` into `foreign_isolation = process` and nothing else; the verified program
runs in-process either way. Spec §6.1's honesty note says exactly this, so the code is not lying —
but the **flag name** is. An operator who types `--isolation process` around an untrusted program
gets no jail for that program. *Correction:* keep the honest note, and rename the axis so the two
things are not one flag — e.g. `--foreign-isolation {inproc,process}` (already exists) and
`--isolation {none,process,microvm}` reserved for what actually jails the whole run, with `process`
refusing (DL1408-style) until a whole-program jail exists rather than accepting and doing nothing to
the program.

**A2 — the isolation label is on the human channel only.** `run_cmd.rs:253-266` is guarded by
`if !opts.json`; verified in P11. The consumer that most needs "which profile am I actually under"
is the machine one. *Correction:* an `isolation` field in the `run` envelope, and in
`delulu authority --json`.

**A3 — the containment function's fail-open case: a final component that exists but is not a
symlink is re-appended unchecked.** `canonical_existing` (`prim.rs:76-100`) asks exactly one question
of a component that did not canonicalize — "is it a symlink?" — and re-appends otherwise. On Windows
the set of things that are neither a symlink nor a plain file is not empty: reserved device names
resolve to device objects. **Verified:** `NUL` and `CON` both passed containment and reached the
device (P6/P8). The SYMLINK-DANGLE-1 reasoning was right and its *implementation* is narrower than
its argument — the argument is "a component whose destination we could not verify must not be
re-appended", and a device name is such a component. *Correction:* on Windows, refuse a final (or
any) component whose stem, upper-cased and stripped of trailing dots/spaces, is a reserved name;
state it in `STAGE3_SPECIFICATION.md` §4.3 beside the canonicalize-then-prefix rule; and pin it with
a generator over all 22 names × 5 spellings. This is a **runtime** fix; no OS jail supplies it,
because `<grant>\COM1` really is inside the grant as far as the kernel's path parser is concerned.

**A4 — the decision and the record are made on the unnormalized string, again.** The trace record for
the `CON` write says `detail: "CON"`, and for `space.txt␠` it will say `space.txt␠` while the file is
`space.txt` (P9). The project has already paid for this twice (C86: the audit attested the wrong
host; GUARD-SPELL-1: a seal written the natural relative way gated nothing). *Correction:* record
the **resolved** path alongside the requested one, and make the guard/audit key the resolved form —
the P22 search key applied to its own next instance.

**A5 — the IPC-1 fix was applied to the broker and not to the structurally identical foreign-worker
channel.** `set_read_timeout` appears at `brokerd.rs:867` and `:918` and nowhere else;
`WorkerConn::call` (`foreign_worker.rs:351-368`) reads unbounded. The `process` profile exists to
bound the damage untrusted foreign code can do, and the one thing it does **not** bound is the thing
untrusted code does most cheaply: not answering. *Correction:* a per-call read timeout on the worker
connection, surfaced as `ForeignErr::Unavailable`/`WorkerDied` rather than a hang, plus the
generator in §3.5. (Also: the worker's `Bind` handshake at `foreign_worker.rs:318-336` has the same
gap.)

**A6 — `process` isolation redirects `DELULU_STATE_DIR` but never clears the environment.**
`build_worker_command` (`foreign_worker.rs:243-257`) sets one variable. Everything else — including
whatever an operator exported for `secret:NAME=env:VAR` — crosses into the process that exists
*because we do not trust its contents*. The same is true of the adapter (`adapter.rs:141-149`).
*Correction:* `env_clear()` + an explicit allowlist in both; assert it with a marker-variable test.

**A7 — the only place in the product with real CPU/memory/wall enforcement is switched off on
Windows.** `limits.rs:245-252` refuses Contained execution there rather than risk a fastfail — the
right call given in-process enforcement. But it means the resource story is: none for the program on
either engine (`interp.rs:33` depth only; `host.rs:1136` a bare `Store`), and for plugins, Linux/macOS
only. *Correction:* move enforcement **out of the process**. A Job Object with
`ProcessMemoryLimit`/`JobMemoryLimit`/`ActiveProcessLimit` and a wall-clock killer on Windows, cgroups
v2 on Linux, applied to the whole run, is both stronger and immune to the fastfail problem — and it
is the same mechanism the jail needs anyway.

**A8 — the network policy is a name policy, and there is no resolution step to attach an address
policy to.** Rows 4/5/24. Today that is harmless because no byte leaves. It becomes load-bearing the
instant a client or the spec §6 egress proxy lands, and the design in spec §6 says the proxy
"enforces the `net` allowlist" — an allowlist of **names**. *Correction:* decide now that the proxy
checks the **connected address** against a policy that includes a default deny of loopback,
link-local, RFC1918/ULA and the metadata address, with the name allowlist as a second, independent
gate; write that into the spec before the code.

**A9 — `spawn`/`send` are outside the grant model entirely.** `Async` is an effect but not a
`--grant` key (P5 ran with `--grant console` alone and reached 1.2 GB), and the mailbox bound is
opt-in per declaration or manifest (`actors.rs:208-231`). The language's own thesis is that a program
can only do what it was granted; concurrency and memory are the exception. *Correction:* a
default mailbox bound with an explicit opt-out, and/or fold "how much" into the grant vocabulary
(`--grant actors=N`, `--grant mem=…`) — a ruling, not a quiet change.

**A10 — `list_dir` grants enumeration, which is not the same authority as reading.** `prim.rs:403-418`
gives every entry name in scope, including the names of links aimed outside (P1). *Correction:* at
minimum say so in the capability's documentation; consider whether `FsRead` should imply enumeration
or whether it wants its own scope bit.

**A11 — an inert grant is accepted silently.** `--grant "fs.read=\\?\<dir>"` parses, prints nothing,
and grants nothing (P10). C87 established the principle for the empty value; the verbatim-prefix case
is the same principle with a different spelling. *Correction:* normalize grant paths through the
same `canonical_existing` the checks use, and refuse a grant that can never match.

---

## 5. What I could not determine

1. **`COM1`–`COM9` / `LPT1`–`LPT9` / `AUX` / `PRN`.** The mechanism is identical to the verified `NUL`
   and `CON`, but this machine reports no serial ports, so I did not demonstrate a write reaching
   real hardware. Needs a machine with a port, or a virtual COM pair.
2. **8.3 short names.** Inconclusive: `dir /x` showed no alias for `inside.txt` on this volume, so the
   probe proved nothing either way. Needs a long name on a volume with 8dot3 generation enabled.
3. **Anything about macOS.** Not run: the `PR_SET_PDEATHSIG` gap, NFC/NFD path confusion, the
   Contained-plugin limits path on Apple Silicon.
4. **Anything about Linux specifics.** `/proc`, `/dev`, seccomp, the `/tmp` squat on the worker channel
   dir, `PR_SET_PDEATHSIG` actually firing — all code claims here, none executed.
5. **The foreign-worker hang (A5).** I did not build a C stub, so the "no read timeout" claim is from
   the source and the grep, not from a hung host. It is the single test I would run first.
6. **Whether the Job Object sets `JOB_OBJECT_LIMIT_BREAKAWAY_OK` / `SILENT_BREAKAWAY`.** I read only
   the `KILL_ON_JOB_CLOSE` assignment at `foreign_worker.rs:597-620`; I did not enumerate the full
   `JOBOBJECT_EXTENDED_LIMIT_INFORMATION` the code writes, so I cannot say whether a grandchild can
   break away.
7. **Whether the guard/audit key is the requested or the resolved path.** A4 is inferred from the
   trace record's `detail` field (which is the requested string, verified) plus `interp.rs`'s call
   sites, which I did not read. The audit chain's `target` field may already carry the resolved form.
8. **The microVM design's residuals.** There is no code to attack — `run_cmd.rs:216-241` refuses
   before anything runs — so §3's ⛔ rows are designed against the spec, not against an implementation.
9. **Whether any of this is already covered by an existing test.** I did not read
   `crates/*/tests/`; the head chef should check before treating any ✅ row as new work.
