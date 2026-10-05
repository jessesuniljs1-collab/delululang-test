# P8 — safe autonomy, as far as software reaches: the design

**Status:** designed 2026-09-28 by the first cloud routine run (D-V2-51), under the owner's mandate of
the same day (*"finish all the phases and verify"* — `docs/CLOUD_ROUTINE.md` names P8 "as far as software
reaches"). **P8-01 is complete** (routine run 13, 2026-10-05, D-V2-95 — `tests/sandbox_devices_cli.rs`, and at L2
`tests/microvm_cli.rs`); P8-02 to P8-04 are not. Each slice below is sized for one routine run and ends with its CI run read green. A real device stays environment-blocked, and every surface that mentions P8 says so.

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

### P8-03 — the reference transport and the sim as a device

A transport the tests and a lab can both use: a line or byte stream to the in-tree simulator run as a
separate process, so P8-02 is witnessed across a real process boundary. A serial port or a CAN socket is
a transport of the same shape, and is the operator's to wire — a real device stays out of reach here.

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

## What stays out, and why

- **A real device, and certification** — environment-blocked (no hardware in a cloud VM), and D23 and
  `STAGE10_AUTONOMY_ADDENDUM.md` §4 keep saying so.
- **Federation model-checking** (RW 5.5) — separate work, not a precondition for P8.
- **A lease-level "only attested" or "only Verified adapter" constraint** — changes the authority model:
  RFC territory, the owner's governance.
