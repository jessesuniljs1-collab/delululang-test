# P8 — safe autonomy, as far as software reaches: the design

**Status:** designed 2026-09-28 by the first cloud routine run (D-V2-51), under the owner's mandate of
the same day (*"finish all the phases and verify"* — `docs/CLOUD_ROUTINE.md` names P8 "as far as software
reaches"). **P8-01 is complete** (routine run 13, 2026-10-05, D-V2-95 — `tests/sandbox_devices_cli.rs`, and at L2
`tests/microvm_cli.rs`) and **P8-02 is complete** (routine run 14, 2026-10-05, D-V2-96 and D-V2-97 —
`crates/delulu-runtime/src/verified_adapter.rs`, `tests/hw_dpx_cli.rs`, `examples/line_driver`) and **P8-03 is
complete** (routine run 14, D-V2-98 — `crates/delulu-runtime/src/sim.rs`, `delulu device sim`,
`tests/sim_device_cli.rs`); P8-04 is not. Each slice below is sized for one routine run and ends with its CI run read green. A real device stays environment-blocked, and every surface that mentions P8 says so.

## What exists, measured on 2026-09-28

- **The device broker is already host-side** (`crates/delulu-runtime/src/device.rs`): leases per granted
  actuator and sensor, the envelope and rate checks before a command leaves the process, the dead-man
  watchdog on its own thread (`ClockMode::Wall`), the operator e-stop through the grant tree
  (`with_authority_watch`), the declared fail-state. Invariant 52's "the adapter and the dead-man stay
  host-side" is true today.
- **The driver** is an operator-supplied subprocess (`crates/delulu-runtime/src/adapter.rs`, a line
  protocol with a 2,000 ms first exchange). Its provenance is checked before it is started (D52/D53: a
  detached signature, refused when bad, optionally required and pinned; D66: recorded), and since
  ADAPTER-SPELL-1 (D-V2-50) the file checked is the file started. What it is NOT: specification §5.4's
  Verified-class signed plugin — and a file can still be replaced between the check and the start.
- **A control program cannot run as a guest.** `guest::CARRIED` holds Console, FsRead, FsWrite, Clock,
  Rand and Http; a program that names an actuator or a sensor is refused under `--sandbox` — witnessed:
  "`--sandbox` cannot carry this program yet: it uses Actuator. Nothing ran.", exit 2. The channel itself
  is generic — `RootMethod` mints a handle on the host,
  `CapMethod` performs an operation by handle — so what is missing is the host's side, not a new wire.

## The slices

### P8-01 — the control program in a guest (after PS-E)

**Stands on PS-E** (D-V2-53, `V2_OPENSHELL_STUDY.md`): the guest that holds an actuator handle is sent
its program only from PS-E-01's `Confirmed` state, and PS-E-02's "host loss ends the guest" is one of
the properties a device run's profile requires — a control program must never outlive the host that
holds its watchdog.

A program holding `Actuator`/`Sensor` capabilities runs under `--sandbox` (L1) and `--isolation microvm`
(L2). `root.actuator("arm0/elbow")` and `root.sensor(…)` become `RootMethod`s the host answers with a
handle; `command`, `read` and the beat become `CapMethod`s the host performs through the run's
`DeviceBroker` — the same broker, envelope, rate, lease, dead-man and e-stop an ordinary run uses. The
guest holds a handle and nothing else, so a guest that stops talking is a program that stopped beating:
the host's watchdog engages the fail-state with no cooperation from the guest.

Witnesses, all against the simulator (`--broker-profile sim`) and on every OS CI runs:
1. the same program, sandboxed and not, produces the same commanded values and the same refusals (an
   out-of-envelope command never reaches the simulator — the adapter's log is the witness, as in
   `hw_adapter_cli.rs`);
2. a guest that wedges (loops without beating) loses its actuator within `heartbeat_ms` of wall time and
   the fail-state is engaged — measured on the host, the guest's own clock not consulted;
3. `delulu grants revoke` of the run's node engages the fail-state while the guest is mid-motion;
4. the DL1905 sign-off gate still binds: a sandboxed hardware run of bytes nobody signed off starts no
   driver;
5. falsified: a host that forwards a command without the envelope check, and a guest-side beat that the
   host believes without a live channel, each turn a witness red.
`CARRIED` gains the two kinds; `unsupported_surface`'s refusal loses "devices"; the report's `granted`
and `host_guarantees` gain nothing a device run did not measure.

**Where the work is (read before starting):** the host channel performs a guest's `CapMethod` through
`prim::call_cap_method`, and device operations do not live there — they live in the interpreter
(`Interp::call_actuator` / `call_sensor` in `crates/delulu-runtime/src/interp.rs`, with the envelope
check, the refused-attempt sweep of C39, and the DL1904 trace records). So P8-01 first moves that logic
behind one function both the interpreter and `HostChannel` call, with the `DeviceBroker` handed to the
host channel the way custody is (`HostChannel::with_custody`); and the sandboxed path in
`crates/delulu/src/guest.rs` must build the broker exactly as `run_cmd.rs` does for an ordinary run —
the DL1905 sign-off gate, the adapter's provenance check and `resolve_driver`, the clock mode — by
sharing that code, not copying it.

**The build order, read against the code by routine run 12 (2026-10-05, after PS-E closed — D-V2-93):**
1. *Minting already crosses.* `root.actuator(d)` / `root.sensor(d)` are `RootMethod`s the host answers through
   `LocalSink::root_method` → `prim::call_root_method` (`prim.rs`, the "actuator"/"sensor" arms): the handle carries the
   granted envelope, deny-by-default. Nothing to build there but `CARRIED` (`guest.rs`) gaining the two kinds.
2. *Using does not.* A guest's `command`/`read` arrive as `CapMethod`s and reach `prim::call_cap_method_pinned`, which
   knows no device. Move the bodies of `Interp::call_actuator` and `Interp::call_sensor` (`interp.rs`, about 100 lines:
   the envelope check, C39's `note_refused_attempt` sweep, `DeviceBroker::command`/`read`, the `Envelope`/`LeaseRevoked`/
   `NoDevice` values) into `device.rs` as two functions taking `Option<&DeviceBroker>`, the capability's scope, the
   arguments and a sink for the DL1904 trace records — the interpreter passes `trace_actuate_refusal`; the host channel
   records the same refusals in its denied list. Behaviour-preserving first: the existing device tests are the witness.
3. `HostChannel::with_devices(Arc<DeviceBroker>)`, consulted in `CapMethod` for an `Actuator`/`Sensor` capability after
   the custody gate, exactly where the interpreter consults it.
4. `run_cmd.rs`'s broker construction (lines ~1375–1445: the hw adapter's `resolve_driver`, `check_adapter_signature`,
   `record_adapter_provenance`, `ProcessAdapter::spawn`, `DeviceBroker::with_adapter`) becomes one shared function the
   sandboxed path in `guest.rs` calls too; the device clock is the wall's for a guest (its own clock is never consulted).
5. The witnesses (1–5 above), then the falsifiers. `contained` already requires `host_loss_ends_guest` (D-V2-92): a
   control program never outlives the host that holds its watchdog.

**Built as written (D-V2-95), with three departures:** the broker starts when the guest is SENT its program, not before
its launch (a microVM's boot is not charged against the first heartbeat); `--sim-step` and `--signoff` were refused
under `--sandbox` at first and applied the same run (`5b89f9f`); and witness 2 has two more forms — a guest wedged for ever loses the arm all the same, and a
guest that only talks to its host is not beating it. M123 (the capability's own envelope check removed) survives on the
broker's own check, the second wall; M123b (both removed) is red. Level 2 is read on the KVM runner, where M128 — the broker
started before the launch — is red (a VM's boot revoked a 150 ms lease) though it survives every L1 witness.

### P8-02 — the Verified-class adapter as a `.dpx`

The driver's LOGIC becomes a DeluluLang plugin of the Verified class: re-proved at load like every
Verified plugin (P2), holding no capability, a pure function from a command to the frames the device
speaks (and from a device's reply to a reading). The host keeps the TRANSPORT — the only part that
touches the machine — and writes the frames the plugin computed, so the plugin cannot reach anything the
host does not write for it. Its bytes are read once, the detached signature verified against a pinned key
(`--adapter-signer`, now required for the Verified class), and executed from those same bytes: the
check-then-start residual of D-V2-50 is gone, because nothing is started by name. Refusals are the
existing codes (DL1510 bad or wrong-key signature, DL1511 unsigned or unverifiable, DL1905 not signed
off); no new code without a decision recorded.

Witnesses against the simulator: a signed adapter drives the simulated arm; a tampered one, one signed by
a stranger, and one that fails its proof at load are each refused before a frame is written; the envelope
still refuses before the plugin is called; a plugin that tries to name a capability does not check.

**The build order, read against the code by routine run 13 (2026-10-05, after P8-01 completed — D-V2-95):**
1. *A Verified plugin already runs as a pure function the host can call.* "The DIR is what runs, not the WASM"
   (`plugin.rs`, `LoadedHandle`): `Interp::call_plugin_export` builds `Interp::new(&verified.dir.module)` and calls the export
   by name (`call_exported`, DL1508 for a missing one). `load_verified` is the gate, in its normative order: container and
   API (DL1507), class (never inferred), ceiling `grant ⊑ plugin.authority` (DL1502), holder (DL0802), the DIR replayed and
   each export's verified type matched EXACTLY to its manifest signature (`step5_verified`, DL1504: "the verified type …
   does not match the manifest signature"), then the signature policy (DL1510 bad, DL1511 unsigned under `require_signed`).
   So purity needs no new rule: an adapter's grant is empty, and an export whose manifest signature has an empty row cannot
   be matched by a body that performs an effect — "a plugin that tries to name a capability does not check" is that DL1504.
2. *The interface the plugin plugs into.* `DeviceBroker` keeps `Mutex<Option<ProcessAdapter>>` and calls the adapter only
   under `Profile::Hw`, AFTER its lease, rate and envelope checks (`DeviceBroker::command`, `read`) — the ordering P8-02
   needs, already. Make the slot an `Adapter` trait (`command(device, fields)`, `read(device)`, `Send`), implemented by
   `ProcessAdapter` unchanged and by a new `VerifiedAdapter`; the broker's code does not otherwise change.
3. *Threads.* The broker is an `Arc` whose watchdog runs on its own thread; an `Interp` and its values are `Rc`. The DIR
   (`delulu_check::Dir`, an AST `Module`) is plain data — `delulu-syntax/src/ast.rs` holds no `Rc`, `RefCell` or `Cell`
   (assert `Dir: Send` at compile time as the first step). So the `VerifiedAdapter` owns ONE thread that owns the module and
   builds an `Interp` per call, and the broker holds only its `Send` side (two channels). The exchange keeps `adapter.rs`'s
   three laws: a bounded reply, a deadline, poisoned after the first failure.
4. *The exports.* `encode` (a command → the frame line the device speaks) and `decode` (the device's reply → a reading),
   with empty rows and value types `lower_export_signature` accepts — choose them from `step5_verified`'s own tests. The
   frames' reference is `adapter.rs`'s `CMD`/`READ` line protocol, so P8-03's transport can speak to the in-tree simulator
   run as a process and a `.dpx` can be witnessed against it.
5. *Provenance, read once.* `verify_signature(manifest, dir, sig)` checks the artifact's embedded signature (a 32-byte key
   ‖ a 64-byte signature); P8-02 adds the PIN — the key must be `--adapter-signer`, required for the Verified class
   (DL1510 for another key, DL1511 unsigned) — and interprets the very bytes it verified, so D-V2-50's check-then-start
   residual is gone. `record_adapter_provenance` records it, as for a driver.
6. *The flag.* `--adapter-dpx FILE`, refused beside `--adapter-cmd`; it joins `device_terms`'s hardware-only list (refused
   without `--broker-profile hw:`) and the sandbox's allowlist; DL1905 is unchanged.
7. *Witnesses,* the list above — first against an in-process transport that logs the frames it is handed (so "refused
   before a frame is written" is read from that log), then against P8-03's.

**Done by routine run 13 (`418dcca`):** step 2 (`adapter::Adapter`, the broker's slot `Option<Box<dyn Adapter>>`) and step
3's precondition (`Dir: Send`, asserted at compile time). **The next run starts at step 3 proper:** the `VerifiedAdapter`'s
thread.

**Done by routine run 14 (`45d1181`, D-V2-96):** steps 3 and 4 — `delulu_runtime::verified_adapter`: the logic's thread, a
fresh interpreter per call, one deadline over the exchange, the poison laws, and the interface as FOUR exports (a departure:
`encode_command`, `decode_command`, `encode_read`, `decode_read`, each R-Get-checked to an empty row before the adapter
exists), witnessed against an in-process device. **Steps 5–7 the same run (`a8040fd`, D-V2-97): P8-02 is complete** —
`--adapter-dpx` read once and pinned to `--adapter-signer`, re-proved and interpreted from those bytes; the interface
checked before `--adapter-transport` is looked up; every dropped-flag combination refused; both flags carried under
`--sandbox`; `adapter::LineTransport` and `examples/line_driver` as the reference transport and driver; five end-to-end
witnesses and M145–M153 red. **Next: P8-03**, the simulator as a device behind this transport across a real process
boundary, then P8-04.

### P8-03 — the reference transport and the sim as a device — **complete** (routine run 14, `e92870a`, D-V2-98)

A transport the tests and a lab can both use: a line or byte stream to the in-tree simulator run as a
separate process, so P8-02 is witnessed across a real process boundary. A serial port or a CAN socket is
a transport of the same shape, and is the operator's to wire — a real device stays out of reach here.

**Built:** the transport is `adapter::LineTransport` (P8-02, `a8040fd`); the device is `delulu device sim`, running
`delulu_runtime::sim` — the simulator moved out of `device.rs`, so the broker and the device process drive ONE
simulator. The bench's `--actuator` bounds are the device's own limits, not the grant's, so a hard stop is simulated by
giving it a narrower range; the host's envelope refusal never reaches it. Witnessed by parity: the same program, seed and
command sequence read the same numbers in-process and across the boundary (`sim_device_cli.rs`); M154–M158 red.

### P8-04 — an out-of-band monitor (D-V2-54)

The shape of NVIDIA's Sentry (announced 2026-09-28: out-of-band watching from a DPU, quarantine in
milliseconds), in software. A **monitor** is a separate principal — its own process, a separate OS
identity where the host allows one — holding exactly one authority: **revoke** over one run's grant
node. It reads only what the host recorded (the audit as it is appended, or PS-E-06's OCSF stream),
applies declarative rules (N refusals within T; any envelope refusal on an actuator; any break-glass
use; any special-use reach), and quarantines by revoking: the e-stop and the declared fail-state engage
(implemented), the guest is ended. It cannot grant and cannot perform an effect, and it trusts nothing
the guest says.

Witnesses against the simulator: a guest that keeps probing refused operations loses its actuator
within a measured bound after the Nth refusal; a monitor that tries to grant (a mutant) is refused by the
broker; the monitor's own death is recorded and then handled as the operator's rule says —
`quarantine` (the default for a run that holds a device: a watchdog that has gone quiet is treated as
one that fired, as OpenShell treats the loss of its fence's controller) or `continue`.

**The build order, read against the code by routine run 14 (2026-10-05, after P8-03 — D-V2-98):**

1. *Everything the monitor READS exists.* `delulu_broker::audit::query(dir, &QueryFilter { node, action, effect })`
   reads the hash-chained log a run writes (one shared reader, `audit::read_regular` — AUDIT-FIFO-1), and every record
   carries `seq`, `ts`, `actor_node`, `action`, `target`, `authority`, `decision` and its hash. A device command
   round-trips to the broker per command (`validate::Op::Actuate`, 10g — "what makes an operator e-stop reach a running
   arm at all"), so a refused command is a `deny` record on the device's own node. PS-E-06's OCSF export (D-V2-78) is
   the same records in another spelling, so a monitor reading either sees the same thing. Nothing new to build for
   reading — the rules read `query` at a `--since SEQ` and never anything the host did not record.
   **Measured by routine run 15 (2026-10-09): two of those claims were false.** (i) A refused command was NOT a `deny`:
   the broker records `use allow` on the RUN's node for the device's identity, and the envelope's refusal reached only
   standard error — fixed (AUDIT-REFUSAL-1, `5cd4474`, D-V2-100): each envelope refusal is now a `use` `deny` citing the
   `allow` it overrides. (ii) `--since SEQ` was not a cursor: the daemon counted from 1 on every start and a sandboxed run's host
   numbered its own records, and `--since` dropped a later revocation (AUDIT-SEQ-1 — closed the same run, `2546425`,
   D-V2-102). The monitor resumes after the last record HASH it read all the same: a hash also shows a chain rewritten
   under it. Still journaled only on the run's stderr: a dead lease's revocation (missed heartbeat, TTL),
   which a rule on "the device was lost" would need.
2. *Everything the monitor DOES exists too.* `tree::revoke(caller, target)` is transitive over the subtree, idempotent,
   consumes one audit seq and stamps `revoked_by_seq` on every node; the device's fail-state and the guest's death
   follow from it already (`AuthorityProbe` → `AuthorityState::Dead` → the declared fail-state, and `close_devices`).
   `brokerd::request(state_dir, ReqBody::Revoke { caller, target })` is the whole action, one line.
3. **The tension to resolve first, and it is the slice's real design question.** `revoke` is allowed only when the
   target is the caller's own node or a DESCENDANT of it (`tree.rs`: `is_self_or_descendant`, else
   `Denial::NotRevocable` — no upward or lateral reach). So a monitor that can revoke a run must be an ANCESTOR of the
   run's node. But a parent bounds its child (R-7: no grantee exceeds its grantor), so an ancestor of a device run
   necessarily holds at least `Actuate`. "A separate principal holding exactly revoke" is therefore not expressible in
   today's tree. Three ways, and the choice belongs in a `D-V2-nn`:
   - **(a) The monitor is an automated operator** — it holds the operator's own root node id. Buildable today with no
     model change, and honest only if stated plainly: it is as strong as the operator, which is what the design wanted
     to avoid.
   - **(b) A monitor node between the operator and the run** (recommended): the operator mints a monitor node holding
     exactly the run's authority, and the run attenuates under IT (`watch_devices`/`mint_device_nodes` already mint the
     run's device nodes under whatever node the client holds, so this is a parameter, not a new path). The monitor can
     then revoke its own subtree — the run — and nothing else: not laterally, not upward. The residual to name: its node
     *holds* the run's authority because a parent must, and what keeps it from using that authority is that the monitor
     program performs no effect, plus the same-uid boundary (§11.4, category 7).
   - **(c) A revoke-only principal in the tree** — a watcher registered on a node, allowed to revoke that subtree and
     nothing else. This is the only form that makes "exactly revoke" true. The argument that it is HARDENING rather than
     redefining: revocation is monotone — it only ever removes authority, so a principal that can only revoke cannot
     widen what any program may do. It still changes who may revoke, which is the Authority model's own rule, so it is
     put in front of the owner rather than taken under the delegation.
   Build (b) now, name (c) as what would make the claim exact, and record (a) as refused.
   **Built by routine run 15 (`b0a5a80`, D-V2-101):** `delulu monitor watch`, with steps 4 (two of its four rules —
   `envelope`, `denies=N/MS`), 6 (the surface, its five gates) and 7 (the witnesses: the quarantine measured at 13 ms
   after the third refusal; a sibling's monitor reads everything and quarantines nothing; a grant attempt has no path —
   the monitor sends only `Quarantine`). **Step 5 built by routine run 17 (`c299e47`, `3e9c496`, D-V2-106)**; next: the other two rules.
4. **Measured by routine run 15:** `envelope` and `denies=N/MS` are built (D-V2-101); `special-use` waits on RW 4.46 (the
   egress client's refusal is not in the chain); `break-glass` waits on its record carrying the run's node (`audit_required`
   writes none). *The rules, declarative and few* (`V2_OPENSHELL_STUDY.md` §4.7): N `deny` records within T on one run's subtree; any
   envelope refusal on an actuator; any break-glass use (`breakglass.rs` records one); any special-use reach. Each is a
   predicate over the records of step 1, evaluated at a poll interval the operator sets, with the window and the count
   printed in the quarantine's own record — a monitor that fires without saying what it saw is a monitor nobody can
   audit.
5. *The monitor's own death* — **BUILT by routine run 17 (2026-10-10, D-V2-106) as option (c)**: `delulu-broker`'s
   `deadman.rs`, the daemon's `ArmDeadman`/`Beat`/`DisarmDeadman`, `monitor watch --on-monitor-death quarantine|continue
   --death-after MS`; a beat carries a key only the monitor holds; witnessed by killing and by stopping a monitor mid-run (the run's
   node revoked about one period after, the cause in the chain). Read against the code by routine run 15 first: A dead monitor cannot act, so
   whatever quarantines on its death must live elsewhere: (a) a short TTL on `g_M` that the monitor renews — but the tree
   has no renewal for a local node (RFC 0001 F4's `Renew` needs a receipt signed by the ground), and a holder extending its
   own deadline is a widening in time; (b) each run probes its monitor's liveness beside its grant node — a run-side opt-in
   that couples every run to a monitor it cannot see; (c) **recommended:** the broker holds a dead-man FOR A NODE — `monitor
   watch` arms one on `g_M` and beats it each poll, and if the beats stop the broker revokes `g_M`'s subtree with the
   cause recorded (`continue` = arm none). The argument that (c) hardens rather than redefines: any node may already revoke
   itself (`caller == target`), so arming a FUTURE self-revocation adds no authority — it only ever removes some, as the
   device dead-man (10f) does for a lease. The next run builds (c), witnessed by killing a monitor mid-run.
   Original text: `quarantine` is the default for a run holding a device (a watchdog gone quiet is treated
   as one that fired, as OpenShell treats the loss of its fence's controller) and `continue` is the operator's explicit
   opt-out; the choice is recorded. The run's own dead-man is unaffected and remains the real-time guarantee — the
   monitor is a second, slower line, never a replacement for it.
6. *The surface.* `delulu monitor watch --state-dir DIR --node g_ID [--rule ...]... [--poll MS] [--on-monitor-death
   quarantine|continue] [--json]`. **A new subcommand is FIVE gates** (routine run 14): the help line, the dispatcher's
   arm and `cli::SUBCOMMANDS`; `json_contract`'s failing sweep AND its success table; and `mcp.rs`'s door rule — and
   `monitor watch` is an EFFECTOR (it revokes), so no tool may start it.
7. *Witnesses, against the simulator and `delulu device sim`* (P8-03): a guest that keeps probing refused operations
   loses its actuator within a measured bound after the Nth refusal (the bound printed, as D-V2-90's sampler is); a
   monitor that tries to grant is refused by the broker (a mutant that asks for `Issue` gets DL1421/`NotRevocable`); a
   monitor given a node it is not an ancestor of revokes nothing (`Denial::NotRevocable`, the step-3 rule as a test);
   and the monitor's death handled both ways.

## What stays out, and why

- **A real device, and certification** — environment-blocked (no hardware in a cloud VM), and D23 and
  `STAGE10_AUTONOMY_ADDENDUM.md` §4 keep saying so.
- **Federation model-checking** (RW 5.5) — separate work, not a precondition for P8.
- **A lease-level "only attested" or "only Verified adapter" constraint** — changes the authority model:
  RFC territory, the owner's governance.
