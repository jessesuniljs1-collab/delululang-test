//! Stage 10 phase 10f — the device broker: dead-man leases, the reference simulator, and the
//! sim-to-hardware artifact gate (Track D, spec §5.2/§5.4, invariants 47 and 48).
//!
//! 10e opened the physical boundary and enforced the envelope at the call site. This module adds
//! the half that has to keep working when the program stops working.
//!
//! **The dead-man is not a library call.** A lease that only expires when the program asks
//! whether it has expired is not a dead-man switch — it is a comment. So the revoke decision runs
//! on a watchdog thread that owes the program nothing: it wakes on its own tick, compares each
//! lease's last beat against its `heartbeat_ms`, and when the beat is overdue it revokes the lease
//! and engages the device's declared fail-state whether or not the interpreter ever runs another
//! instruction. A program wedged in an infinite loop, blocked on a socket, or stopped at a
//! breakpoint loses its actuators on schedule.
//!
//! **The beat rides device activity** (build-order D11c). Spec §5.2 says the runtime heartbeats
//! "while the holding actor's turns are healthy"; this runtime beats a lease on every accepted
//! operation against that device. The consequence is deliberate and documented rather than
//! hidden: a control loop must touch its device at least once per `heartbeat_ms`, because a device
//! nobody is driving is exactly the device a dead-man exists to release.
//!
//! **The broker's envelope is the authority of record.** `interp` already checks the command
//! against the capability value's scope; the broker checks it again against the envelope it took
//! from the human's grant. In this in-process sim those two copies come from the same source, so
//! the doubling is structural rehearsal for the real split (spec §5.1's host-side *and*
//! adapter-side check), NOT the independent defense-in-depth a hardware deployment gets — and
//! this comment says so rather than letting the word "double" do work it has not earned. What it
//! does buy today is real: if a capability's scope and the grant ever disagree, the grant wins.
//!
//! **The simulator is honest about being a simulator.** It models kinematics no more deeply than
//! it must: a commanded dimension moves to its setpoint, a fail-state engages, and a sensor reads
//! back either a mirrored actuator position or a reproducible synthetic signal. Nothing here is a
//! physics engine, and no output of this module may be described as one.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::value::ActuatorEnvelope;

/// The fail-state a device adapter engages when its lease dies (spec §5.2). Declared by the human
/// at grant time, per device — never defaulted, because "what this machine does when the software
/// stops" is not a decision a runtime may make on an operator's behalf.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailState {
    /// Hold the last commanded setpoint with the drive engaged.
    Hold,
    /// Disengage the drive and let the mechanism coast.
    Coast,
    /// Drive to the device's declared safe park pose.
    SafePark,
}

impl FailState {
    pub fn parse(s: &str) -> Option<FailState> {
        Some(match s {
            "hold" => FailState::Hold,
            "coast" => FailState::Coast,
            "safe-park" => FailState::SafePark,
            _ => return None,
        })
    }

    pub fn name(&self) -> &'static str {
        match self {
            FailState::Hold => "hold",
            FailState::Coast => "coast",
            FailState::SafePark => "safe-park",
        }
    }
}

/// Which backend the run's devices are bound to (spec §5.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Profile {
    /// No adapter at all — 10e's behavior, preserved exactly: commands validate and go nowhere,
    /// sensors answer `NoDevice`. Invariant 50 lives here.
    Null,
    /// The in-tree reference simulator, deterministic under its seed.
    Sim { seed: u64 },
    /// A hardware adapter, named. No adapter ships in-tree (see `DeviceBroker::new`), and the
    /// profile exists so the artifact-hash gate has something real to gate.
    Hw { adapter: String },
}

/// How the dead-man lease clock advances (build-order D20).
///
/// `Wall` is real time, and it is the dead-man's actual guarantee: a program that stops beating
/// loses its actuator after `heartbeat_ms` of WALL-CLOCK silence — whether it is wedged, looping,
/// or paused at a breakpoint. Every hardware profile uses it, and it is the only mode in which
/// "the program stopped, so the machine stopped" is a real-time promise. The watchdog thread owns
/// expiry here.
///
/// `Stepped` is for the reference simulator only: the lease clock advances by a fixed amount of
/// SIMULATED time per device interaction (command, read, or beat), and expiry is swept
/// synchronously at each interaction. The interaction on which a lease dies is then a deterministic
/// function of the COMMAND SEQUENCE, not of how fast the interpreter runs between commands — so a
/// sim demonstration replays identically in a debug build, a release build, or under load. The
/// honest cost, stated where it is chosen: a program that stops interacting stops the clock, so
/// `Stepped` does NOT model the wedged-program dead-man — that guarantee is real-time and belongs
/// to `Wall`. `Stepped` is a determinism tool for demonstrations, never a safety mechanism.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockMode {
    Wall,
    Stepped { step_us: u64 },
}

/// One thing that happened to a device, for the effect trace. The watchdog runs on its own thread
/// and `TraceSink` is an `Rc`, so device events accumulate in this `Send` journal and the CLI
/// merges them into the trace at exit — the same shape Stage 7 uses for worker-collected records.
#[derive(Clone, Debug)]
pub struct DeviceEvent {
    /// The trace `op`: `lease.revoked`, `failstate.engaged`.
    pub op: String,
    pub device: String,
    pub detail: String,
    /// Milliseconds since the broker started, so the merged records stay orderable even though
    /// they are appended after the program's own.
    pub at_ms: u64,
}

/// Why a lease died.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevokeCause {
    /// No beat within `heartbeat_ms` — the program stopped driving the device.
    MissedHeartbeat,
    /// The lease reached `ttl_ms`. A lease is a loan with an end, not a transfer of ownership.
    TtlExpired,
    /// An operator revoked it (`delulu grants revoke` on the actuator subtree, spec §5.2 e-stop).
    Operator,
}

impl RevokeCause {
    pub fn name(&self) -> &'static str {
        match self {
            RevokeCause::MissedHeartbeat => "missed-heartbeat",
            RevokeCause::TtlExpired => "ttl-expired",
            RevokeCause::Operator => "operator-revoke",
        }
    }
}

/// What an [`AuthorityProbe`] found when it asked whether this run still holds its grant.
#[derive(Clone, Debug)]
pub enum AuthorityState {
    Live,
    /// The authority is gone — revoked by an operator, expired, or **unanswerable**. All three
    /// collapse to one variant on purpose: a run that cannot confirm it still holds a machine is
    /// in exactly the position of a run that has been told it does not.
    Dead(String),
}

/// The watchdog's link to the grant tree: asked once per tick **per device**, from the watchdog
/// thread, so an operator e-stop reaches the machine WITHOUT the program cooperating. A program
/// that has stopped commanding — wedged, looping, waiting on a socket — is precisely the case an
/// e-stop exists for, and exactly the case a check on the command path would miss.
///
/// Per device, not per run, because spec §5.2 says *subtree*: each device holds its own child node
/// under the run's, so revoking one arm stops that arm and leaves the supervisor its console to
/// report with. Revoking the parent still reaches every device — transitively, through the tree,
/// which is the broker's job and not this probe's.
pub type AuthorityProbe = Box<dyn Fn(&str) -> AuthorityState + Send>;

/// A revocation that has already happened, with the numbers the latency budget is made of.
#[derive(Clone, Copy, Debug)]
pub struct Revocation {
    pub cause: RevokeCause,
    /// How late the beat already was when the watchdog noticed. This is the term the tick
    /// interval contributes; publishing it is what keeps the latency claim honest.
    pub overdue_us: u64,
    /// Detection → fail-state engaged, at the adapter. The part the broker itself owns.
    pub engage_us: u64,
}

/// The per-device lease state. `heartbeat_ms`/`ttl_ms`/`fail_state` come from the grant and never
/// change; everything else is the running dead-man. Timestamps are microseconds on the broker's
/// clock (`BrokerInner::now_us`) — real elapsed time in `Wall`, simulated in `Stepped` — so the
/// same arithmetic serves both without the dead-man logic knowing which clock it is on.
struct LeaseState {
    env: ActuatorEnvelope,
    granted_at_us: u64,
    last_beat_us: u64,
    last_command_us: Option<u64>,
    revoked: Option<Revocation>,
}

/// One simulated device: a position per bounded dimension, and whether the drive is engaged.
#[derive(Default)]
struct SimDevice {
    pos: BTreeMap<String, f64>,
    drive: bool,
}

/// The reference simulator's whole state. Deterministic: identical seed plus identical command
/// sequence yields identical readings, on every platform, forever.
struct Sim {
    seed: u64,
    devices: BTreeMap<String, SimDevice>,
    /// Per-sensor read counters — the synthetic signal is a pure function of
    /// (seed, device name, read index), so replaying a run replays its measurements.
    reads: BTreeMap<String, u64>,
}

struct BrokerInner {
    profile: Profile,
    clock: ClockMode,
    /// Simulated microseconds, advanced per device interaction in `Stepped` mode; unread in `Wall`.
    virtual_us: AtomicU64,
    leases: Mutex<BTreeMap<String, LeaseState>>,
    sim: Mutex<Sim>,
    journal: Mutex<Vec<DeviceEvent>>,
    t0: Instant,
    /// The hardware adapter, under `Profile::Hw` only (RFC 0001 dish 3). `None` everywhere else —
    /// and `None` under `Hw` too if the CLI was not told which driver to run, in which case a
    /// command REFUSES rather than pretending to have reached a machine.
    adapter: Mutex<Option<crate::adapter::ProcessAdapter>>,
}

impl BrokerInner {
    /// "Now", in microseconds, on whichever clock this broker runs: real elapsed since `t0` in
    /// `Wall`, the simulated counter in `Stepped`. Every heartbeat/TTL comparison goes through here.
    fn now_us(&self) -> u64 {
        match self.clock {
            ClockMode::Wall => self.t0.elapsed().as_micros() as u64,
            ClockMode::Stepped { .. } => self.virtual_us.load(Ordering::SeqCst),
        }
    }
}

/// The device broker for one run. Owns the leases, the watchdog thread, and the adapter.
pub struct DeviceBroker {
    inner: Arc<BrokerInner>,
    stop: Arc<AtomicBool>,
    watchdog: Mutex<Option<std::thread::JoinHandle<()>>>,
}

/// Why a command did not reach the device. All three become `ActuateErr` values in the program —
/// the command dies, the process lives (10e's law, unchanged).
#[derive(Clone, Debug)]
pub enum CommandRefusal {
    /// The command is outside what the grant's envelope vouches for, or arrived faster than
    /// `rate_hz` allows. Reported as `Envelope(reason)`.
    Envelope(String),
    /// The lease is dead. Reported as `Revoked(reason)` — a distinct variant on purpose: a
    /// control program clamps a bad setpoint and retries, but it must STOP when it no longer
    /// holds the device (build-order D11b).
    Revoked(String),
}

impl DeviceBroker {
    /// Build the broker for a run and start its watchdog. `envelopes` are the granted actuator
    /// envelopes; `sensors` the granted sensor device names. Wall-clock dead-man — the default, and
    /// the only mode that carries a real-time guarantee.
    pub fn new(profile: Profile, envelopes: &[ActuatorEnvelope], sensors: &[String]) -> DeviceBroker {
        DeviceBroker::with_config(profile, envelopes, sensors, None, ClockMode::Wall)
    }

    /// The same broker, plus a probe against the grant tree (10g). This is the operator e-stop:
    /// `delulu grants revoke` kills the node in the broker daemon, the watchdog's next tick sees
    /// it, and every device this run holds engages its declared fail-state — with no cooperation
    /// from the program, which may well be the thing that needed stopping.
    pub fn with_authority_watch(
        profile: Profile,
        envelopes: &[ActuatorEnvelope],
        sensors: &[String],
        authority: Option<AuthorityProbe>,
    ) -> DeviceBroker {
        DeviceBroker::with_config(profile, envelopes, sensors, authority, ClockMode::Wall)
    }

    /// The full constructor: authority probe AND an explicit clock (D20). `ClockMode::Wall` is real
    /// time — every hardware run and every wall-clock dead-man test takes this path, unchanged.
    /// `ClockMode::Stepped` is the deterministic simulator clock, resolved by the CLI only under
    /// `Profile::Sim`.
    pub fn with_config(
        profile: Profile,
        envelopes: &[ActuatorEnvelope],
        sensors: &[String],
        authority: Option<AuthorityProbe>,
        clock: ClockMode,
    ) -> DeviceBroker {
        DeviceBroker::with_adapter(profile, envelopes, sensors, authority, clock, None)
    }

    /// The full constructor, with a hardware adapter attached (RFC 0001 dish 3).
    ///
    /// `adapter` is `Some` only under [`Profile::Hw`], and only when the CLI was told which driver
    /// to run. Under `Hw` with no adapter every command REFUSES — a hardware profile that silently
    /// commanded nothing would be the most dangerous possible default, because the program would
    /// report success while the machine never moved.
    pub fn with_adapter(
        profile: Profile,
        envelopes: &[ActuatorEnvelope],
        sensors: &[String],
        authority: Option<AuthorityProbe>,
        clock: ClockMode,
        adapter: Option<crate::adapter::ProcessAdapter>,
    ) -> DeviceBroker {
        let now = Instant::now();
        let mut leases = BTreeMap::new();
        let mut devices = BTreeMap::new();
        for env in envelopes {
            // The simulated device starts at its park pose: the in-envelope value nearest zero.
            // Starting anywhere else would be inventing a position nobody commanded.
            let mut dev = SimDevice { drive: false, ..SimDevice::default() };
            for (dim, lo, hi) in &env.dims {
                dev.pos.insert(dim.clone(), park_point(*lo, *hi));
            }
            devices.insert(env.device.clone(), dev);
            // Both clocks read 0 at construction (t0 == now; the simulated counter starts at 0), so
            // the lease is born beaten and its TTL begins counting from this instant on either.
            leases.insert(
                env.device.clone(),
                LeaseState {
                    env: env.clone(),
                    granted_at_us: 0,
                    last_beat_us: 0,
                    last_command_us: None,
                    revoked: None,
                },
            );
        }
        let seed = match &profile {
            Profile::Sim { seed } => *seed,
            _ => 0,
        };
        let inner = Arc::new(BrokerInner {
            profile,
            clock,
            virtual_us: AtomicU64::new(0),
            leases: Mutex::new(leases),
            sim: Mutex::new(Sim { seed, devices, reads: BTreeMap::new() }),
            journal: Mutex::new(Vec::new()),
            t0: now,
            adapter: Mutex::new(adapter),
        });
        for s in sensors {
            inner.sim.lock().unwrap().reads.insert(s.clone(), 0);
        }
        let stop = Arc::new(AtomicBool::new(false));
        let watchdog = spawn_watchdog(inner.clone(), stop.clone(), envelopes, authority);
        DeviceBroker { inner, stop, watchdog: Mutex::new(watchdog) }
    }

    pub fn profile(&self) -> &Profile {
        &self.inner.profile
    }

    /// Validate and dispatch a command. `fields` is the command record flattened to numbers by
    /// the caller — the broker deals in dimensions and magnitudes, never in language values.
    ///
    /// Order matters and is part of the contract: **the lease is checked first**. A revoked lease
    /// must not be able to hide behind a well-formed command, and an operator watching the trace
    /// must see "you do not hold this device" rather than "your torque is 0.1 too high".
    pub fn command(&self, device: &str, fields: &[(String, f64)]) -> Result<(), CommandRefusal> {
        // In stepped mode this advances the simulated clock one interaction and revokes any lease
        // now past its heartbeat or TTL — BEFORE this command is judged — so the command that
        // crosses a deadline is a deterministic function of the sequence. A no-op under `Wall`.
        step_and_sweep(&self.inner);
        let mut leases = self.inner.leases.lock().unwrap();
        let Some(lease) = leases.get_mut(device) else {
            // A device the broker never leased. Fail closed: the interpreter should not be able to
            // reach this (the mint checks the grant), and if it ever can, the answer is still no.
            return Err(CommandRefusal::Revoked(format!("no lease for `{device}`")));
        };
        if let Some(r) = lease.revoked {
            return Err(CommandRefusal::Revoked(format!(
                "the lease on `{device}` was revoked ({}); the `{}` fail-state is engaged",
                r.cause.name(),
                lease.env.fail_state.name()
            )));
        }
        let now = self.inner.now_us();
        // `rate_hz` becomes real here (build-order D10f closes its own gap): a command arriving
        // sooner than the granted period is refused, because a rate bound nobody enforces is a
        // comfort, not a control. Measured on the broker clock, so a stepped sim rate-limits in
        // simulated time exactly as a wall run does in real time.
        if let Some(hz) = lease.env.rate_hz {
            if hz > 0 {
                let period_us = 1_000_000 / hz as u64;
                if let Some(prev) = lease.last_command_us {
                    let since_us = now.saturating_sub(prev);
                    if since_us < period_us {
                        return Err(CommandRefusal::Envelope(format!(
                            "`{device}` is granted at most {hz} Hz; this command arrived {since_us} µs after the last (minimum {period_us} µs)"
                        )));
                    }
                }
            }
        }
        // The broker's own envelope check, against the grant rather than the capability value.
        envelope_check(&lease.env, fields).map_err(CommandRefusal::Envelope)?;

        // Accepted BY THE GRANT. Only now may the command leave this process.
        //
        // The ordering is the whole point of having an adapter at all (RFC 0001 dish 3): the
        // envelope was checked above, against the grant, before one byte reached vendor code. An
        // adapter can therefore refuse MORE — a hard stop, a thermal limit, a fault — and can never
        // permit more, whatever it replies. D11e observed that host-side and adapter-side checks
        // used to live in one process, making this "structural rehearsal"; with a subprocess the
        // split is real.
        if matches!(self.inner.profile, Profile::Hw { .. }) {
            let mut slot = self.inner.adapter.lock().unwrap();
            let Some(ad) = slot.as_mut() else {
                // A hardware profile with no driver attached must not look like success. A program
                // told "COMMANDED" while nothing moved is the worst failure mode available here.
                return Err(CommandRefusal::Revoked(format!(
                    "no hardware adapter is attached for `{device}` — pass `--adapter-cmd` \
                     (a `hw:` profile that commanded nothing while reporting success would be a lie)"
                )));
            };
            if let Err(e) = ad.command(device, fields) {
                // Every adapter failure is a refusal, reported as a VALUE like every other
                // actuation refusal (10e's law): a driver fault must not kill a supervisor that is
                // still holding three other arms. The dead-man governs what happens next — this
                // command did not beat, so silence from here on engages the declared fail-state.
                return Err(CommandRefusal::Envelope(format!("`{device}`: {e}")));
            }
        }
        if matches!(self.inner.profile, Profile::Sim { .. }) {
            let mut sim = self.inner.sim.lock().unwrap();
            if let Some(dev) = sim.devices.get_mut(device) {
                dev.drive = true;
                for (dim, x) in fields {
                    dev.pos.insert(dim.clone(), *x);
                }
            }
        }
        lease.last_command_us = Some(now);
        lease.last_beat_us = now;
        Ok(())
    }

    /// Read a sensor. `None` means "no device" — invariant 50's whole point: an absent measurement
    /// is absent, never a plausible-looking number a control loop would act on.
    pub fn read(&self, device: &str) -> Option<f64> {
        // A read is a device interaction too: in stepped mode it advances the simulated clock and
        // sweeps expiry, so a control loop that only reads — never beating its actuator — still
        // loses that actuator on schedule, deterministically. A no-op under `Wall`.
        step_and_sweep(&self.inner);
        match &self.inner.profile {
            Profile::Null => None,
            // A real sensor read, over the adapter (RFC 0001 dish 3). Every failure returns `None`
            // — invariant 50: an absent measurement is absent, never a plausible-looking number a
            // control loop would act on. The adapter itself distinguishes "no such device" from "I
            // could not tell" and poisons itself on the latter, so a broken driver stops answering
            // rather than answering wrongly.
            Profile::Hw { .. } => {
                let mut slot = self.inner.adapter.lock().unwrap();
                match slot.as_mut() {
                    None => None,
                    Some(ad) => ad.read(device).unwrap_or(None),
                }
            }
            Profile::Sim { .. } => {
                let mut sim = self.inner.sim.lock().unwrap();
                // A sensor named `ACTUATOR#DIM` mirrors that actuator's simulated position — the
                // only reading in this simulator with a physical meaning, and the one that lets a
                // control loop actually close. Everything else is a reproducible synthetic signal
                // and is described as exactly that.
                if let Some((dev, dim)) = device.split_once('#') {
                    // THE SKIP BRANCH. A `#` name is a claim about a specific dimension of a
                    // specific device. If the simulator cannot resolve it — no such actuator, no
                    // such dimension — the answer is absence, NOT a fall-through to the synthetic
                    // source. Handing back a plausible number for a mirror that mirrors nothing is
                    // the precise failure invariant 50 exists to forbid, and it would look
                    // completely normal in the output.
                    return sim.devices.get(dev).and_then(|d| d.pos.get(dim)).copied();
                }
                if !sim.reads.contains_key(device) {
                    return None;
                }
                let n = sim.reads.entry(device.to_string()).or_insert(0);
                let ix = *n;
                *n += 1;
                let seed = sim.seed;
                Some(synthetic_signal(seed, device, ix))
            }
        }
    }

    /// Beat every live lease. Used at quiescence-adjacent points where the runtime knows the
    /// program is healthy but may not be touching a device this instant.
    pub fn beat_all(&self) {
        // A quiescence beat is an interaction: advance and sweep first (so a TTL that already came
        // due is honored, not beaten past), then refresh every LIVE lease's beat.
        step_and_sweep(&self.inner);
        let now = self.inner.now_us();
        for lease in self.inner.leases.lock().unwrap().values_mut() {
            if lease.revoked.is_none() {
                lease.last_beat_us = now;
            }
        }
    }

    /// Operator e-stop (spec §5.2): revoke a device's lease now, engaging its fail-state.
    pub fn revoke(&self, device: &str) -> bool {
        let mut leases = self.inner.leases.lock().unwrap();
        let Some(lease) = leases.get_mut(device) else { return false };
        if lease.revoked.is_some() {
            return false;
        }
        revoke_lease(&self.inner, device, lease, RevokeCause::Operator, 0, Some("revoked in-process"));
        true
    }

    /// The revocation record for a device, if its lease died.
    pub fn revocation(&self, device: &str) -> Option<Revocation> {
        self.inner.leases.lock().unwrap().get(device).and_then(|l| l.revoked)
    }

    /// Every device event so far, in the order they happened.
    pub fn events(&self) -> Vec<DeviceEvent> {
        self.inner.journal.lock().unwrap().clone()
    }

    /// Stop the watchdog and join it. Called before the process reports its verdict, so a run
    /// never leaves a thread deciding things about devices after the program is over.
    pub fn shutdown(&self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(h) = self.watchdog.lock().unwrap().take() {
            let _ = h.join();
        }
    }
}

impl Drop for DeviceBroker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(h) = self.watchdog.lock().unwrap().take() {
            let _ = h.join();
        }
    }
}

/// The watchdog: the reason any of this is a dead-man rather than a promise. It holds no
/// reference to the interpreter and asks it for nothing.
fn spawn_watchdog(
    inner: Arc<BrokerInner>,
    stop: Arc<AtomicBool>,
    envelopes: &[ActuatorEnvelope],
    authority: Option<AuthorityProbe>,
) -> Option<std::thread::JoinHandle<()>> {
    if envelopes.is_empty() {
        return None;
    }
    // Tick fast enough that detection delay is a small fraction of the shortest heartbeat, and
    // never so fast that it burns a core. The tick is a term in the published latency budget.
    let min_hb = envelopes.iter().map(|e| e.heartbeat_ms).min().unwrap_or(200);
    let tick = Duration::from_millis((min_hb / 4).clamp(1, 25));
    Some(std::thread::spawn(move || {
        while !stop.load(Ordering::SeqCst) {
            std::thread::sleep(tick);
            // The e-stop, checked BEFORE the heartbeat sweep so that a run whose authority is gone
            // is never credited with a beat it has no right to. `Dead` covers "revoked", "expired"
            // and "the broker did not answer" alike — see `AuthorityState`.
            if let Some(probe) = &authority {
                let live: Vec<String> = {
                    let leases = inner.leases.lock().unwrap();
                    leases.iter().filter(|(_, l)| l.revoked.is_none()).map(|(d, _)| d.clone()).collect()
                };
                for device in live {
                    // Probed OUTSIDE the lease lock: this is an IPC round-trip, and holding the
                    // lock across it would stall every command in the interpreter for its
                    // duration — turning the safety mechanism into a latency problem.
                    let AuthorityState::Dead(why) = probe(&device) else { continue };
                    let mut leases = inner.leases.lock().unwrap();
                    let Some(lease) = leases.get_mut(&device) else { continue };
                    // THE SKIP BRANCH, re-checked after re-acquiring: the dead-man may have taken
                    // this same lease while the probe was in flight, and revoking twice would
                    // overwrite a genuine missed-heartbeat record with an operator one — falsifying
                    // the audit trail about why a machine stopped.
                    if lease.revoked.is_some() {
                        continue;
                    }
                    revoke_lease(&inner, &device, lease, RevokeCause::Operator, 0, Some(&why));
                }
            }
            // Heartbeat/TTL is the watchdog's job ONLY on the wall clock. Under a stepped sim the
            // lease clock advances on interactions and expiry is swept synchronously there
            // (`step_and_sweep`), so the watchdog leaves it be — a stepped run's timing owes nothing
            // to this thread's scheduling.
            if matches!(inner.clock, ClockMode::Wall) {
                let now_us = inner.now_us();
                let mut leases = inner.leases.lock().unwrap();
                let devices: Vec<String> = leases.keys().cloned().collect();
                for device in devices {
                    let Some(lease) = leases.get_mut(&device) else { continue };
                    if lease.revoked.is_some() {
                        continue;
                    }
                    let Some((cause, overdue)) = due(lease, now_us) else { continue };
                    revoke_lease(&inner, &device, lease, cause, overdue, None);
                }
            }
        }
    }))
}

/// Is this lease past a deadline at `now_us`? Returns the cause and how far past, in microseconds.
/// The single source of truth for "when does a lease die," shared by the wall-clock watchdog and
/// the stepped synchronous sweep so the two clocks apply identical arithmetic — heartbeat before
/// TTL, because a missed beat is the more urgent fact and `overdue_us` means a different thing in
/// each (`revoke_lease`'s journal keeps them distinct).
fn due(lease: &LeaseState, now_us: u64) -> Option<(RevokeCause, u64)> {
    let since_beat = now_us.saturating_sub(lease.last_beat_us);
    let alive = now_us.saturating_sub(lease.granted_at_us);
    let hb_us = lease.env.heartbeat_ms.saturating_mul(1000);
    let ttl_us = lease.env.ttl_ms.saturating_mul(1000);
    if since_beat > hb_us {
        Some((RevokeCause::MissedHeartbeat, since_beat - hb_us))
    } else if alive > ttl_us {
        Some((RevokeCause::TtlExpired, alive - ttl_us))
    } else {
        None
    }
}

/// Stepped mode's clock tick: advance the simulated clock one interaction, then revoke any lease
/// now past a deadline — synchronously, on the caller's thread. A no-op in `Wall`, where the
/// watchdog owns expiry against real time. This is what makes a sim demonstration replay
/// identically regardless of build speed: the interaction on which a lease dies is fixed by the
/// command sequence, not by how fast the interpreter runs between interactions or when a background
/// thread happens to wake.
fn step_and_sweep(inner: &Arc<BrokerInner>) {
    let ClockMode::Stepped { step_us } = inner.clock else { return };
    inner.virtual_us.fetch_add(step_us, Ordering::SeqCst);
    let now_us = inner.virtual_us.load(Ordering::SeqCst);
    let mut leases = inner.leases.lock().unwrap();
    let devices: Vec<String> = leases.keys().cloned().collect();
    for device in devices {
        let Some(lease) = leases.get_mut(&device) else { continue };
        if lease.revoked.is_some() {
            continue;
        }
        let Some((cause, overdue)) = due(lease, now_us) else { continue };
        revoke_lease(inner, &device, lease, cause, overdue, None);
    }
}

/// Revoke one lease and engage its fail-state, measuring the part of the latency the broker owns.
/// Called with the lease map already locked, so a revocation can never interleave with a command.
fn revoke_lease(
    inner: &Arc<BrokerInner>,
    device: &str,
    lease: &mut LeaseState,
    cause: RevokeCause,
    overdue_us: u64,
    why: Option<&str>,
) {
    let detected = Instant::now();
    let fail = lease.env.fail_state;
    if matches!(inner.profile, Profile::Sim { .. }) {
        let mut sim = inner.sim.lock().unwrap();
        if let Some(dev) = sim.devices.get_mut(device) {
            match fail {
                // The drive stays engaged on the last setpoint.
                FailState::Hold => dev.drive = true,
                // The drive lets go. This simulator models no gravity and no friction, so the
                // position simply stops changing — the sim is not claiming the mechanism holds
                // its pose, only that nothing further is commanded.
                FailState::Coast => dev.drive = false,
                // Driven to the park pose: the in-envelope value nearest zero on each dimension.
                // That interpretation belongs to THIS adapter; a real device declares its own.
                FailState::SafePark => {
                    dev.drive = true;
                    for (dim, lo, hi) in &lease.env.dims {
                        dev.pos.insert(dim.clone(), park_point(*lo, *hi));
                    }
                }
            }
        }
    }
    // Engage latency is a real measurement in `Wall`; in `Stepped` no simulated time passes during
    // a synchronous sweep, so the fail-state engages at the same simulated instant it is detected —
    // reporting a wall-clock micro-jitter would make the one number meant to be reproducible, not.
    let engage_us = match inner.clock {
        ClockMode::Wall => Instant::now().saturating_duration_since(detected).as_micros() as u64,
        ClockMode::Stepped { .. } => 0,
    };
    lease.revoked = Some(Revocation { cause, overdue_us, engage_us });
    // Ordering stamp on the broker's own clock: real ms since start, or simulated ms.
    let at_ms = inner.now_us() / 1000;
    let mut journal = inner.journal.lock().unwrap();
    journal.push(DeviceEvent {
        op: "lease.revoked".to_string(),
        device: device.to_string(),
        // The tail differs by cause, because `overdue_us` MEANS something different in each and a
        // single template would misreport two of the three. "beat overdue" is a fact about a missed
        // heartbeat: for a TTL expiry the beats were arriving perfectly and the *loan* ran out, and
        // for an operator revoke nothing was overdue at all — an e-stop reporting `overdue by 0 µs`
        // reads like a heartbeat that landed exactly on time, which is the opposite of what
        // happened. An audit trail that says why a machine stopped has to say the right why.
        detail: match cause {
            RevokeCause::MissedHeartbeat => format!(
                "{device}: lease revoked (missed-heartbeat), heartbeat_ms={}, ttl_ms={}, beat overdue by {overdue_us} µs",
                lease.env.heartbeat_ms, lease.env.ttl_ms
            ),
            RevokeCause::TtlExpired => format!(
                "{device}: lease revoked (ttl-expired), heartbeat_ms={}, ttl_ms={}, held {overdue_us} µs past its ttl",
                lease.env.heartbeat_ms, lease.env.ttl_ms
            ),
            RevokeCause::Operator => format!(
                "{device}: lease revoked (operator-revoke), heartbeat_ms={}, ttl_ms={}, {}",
                lease.env.heartbeat_ms,
                lease.env.ttl_ms,
                why.unwrap_or("authority withdrawn")
            ),
        },
        at_ms,
    });
    journal.push(DeviceEvent {
        op: "failstate.engaged".to_string(),
        device: device.to_string(),
        detail: format!(
            "{device}: `{}` fail-state engaged {engage_us} µs after detection",
            fail.name()
        ),
        at_ms,
    });
}

/// The in-envelope value nearest zero: zero itself when the range straddles it, otherwise the
/// nearer bound. A park pose has to be somewhere the envelope already permits — parking outside
/// the envelope to reach "safety" would be the envelope granting what it refused.
fn park_point(lo: f64, hi: f64) -> f64 {
    if lo <= 0.0 && 0.0 <= hi {
        0.0
    } else if lo > 0.0 {
        lo
    } else {
        hi
    }
}

/// A reproducible synthetic sensor signal: a pure function of (seed, device, read index), in
/// `[-1, 1]`. It is not a model of anything physical and is never described as one — it exists so
/// a sim run replays identically, which is what `--seed` promises.
fn synthetic_signal(seed: u64, device: &str, index: u64) -> f64 {
    let mut x = seed ^ fnv1a(device.as_bytes()) ^ index.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let v = splitmix64(&mut x);
    // 53 bits into [0, 1), then into [-1, 1]. Deterministic on every platform.
    let unit = (v >> 11) as f64 / (1u64 << 53) as f64;
    unit * 2.0 - 1.0
}

fn splitmix64(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The envelope law, broker-side. Identical in intent to the interpreter's (10e, build-order
/// D10b) and deliberately fail-closed in every branch: a dimension the envelope never bounded is
/// refused BY NAME, and a `NaN` fails `x >= lo && x <= hi` rather than passing a negated chain.
pub fn envelope_check(env: &ActuatorEnvelope, fields: &[(String, f64)]) -> Result<(), String> {
    for (fname, x) in fields {
        let Some((_, lo, hi)) = env.dims.iter().find(|(d, _, _)| d == fname) else {
            return Err(format!(
                "dimension `{fname}` is not bounded by the envelope for `{}`",
                env.device
            ));
        };
        if !(*x >= *lo && *x <= *hi) {
            return Err(format!("`{fname}` = {x} is outside the envelope [{lo}, {hi}]"));
        }
    }
    Ok(())
}

// ----- the sim-to-hardware artifact gate (spec §5.4, invariant 48, DL1905) -------------------

/// A sim sign-off: the content hash of the artifact that was exercised in simulation. Deliberately
/// tiny and human-readable — an approval nobody can read is an approval nobody gave.
#[derive(Clone, Debug)]
pub struct Approval {
    pub artifact: String,
    pub hash: String,
    pub profile: String,
}

impl Approval {
    pub fn to_json(&self) -> String {
        format!(
            "{{\n  \"approved_artifact\": {},\n  \"approved_hash\": {},\n  \"approved_under\": {}\n}}\n",
            json_str(&self.artifact),
            json_str(&self.hash),
            json_str(&self.profile)
        )
    }

    /// Parse an approval record. Any shape this cannot read is an ERROR, never an empty approval:
    /// a gate that reads a corrupt file as "nothing to check" is a gate that opens on damage.
    pub fn parse(src: &str) -> Result<Approval, String> {
        let field = |k: &str| -> Result<String, String> {
            let pat = format!("\"{k}\"");
            let i = src.find(&pat).ok_or_else(|| format!("approval record has no `{k}` field"))?;
            let rest = &src[i + pat.len()..];
            let c = rest.find(':').ok_or_else(|| format!("approval field `{k}` has no value"))?;
            let after = &rest[c + 1..];
            let a = after.find('"').ok_or_else(|| format!("approval field `{k}` is not a string"))?;
            let b = after[a + 1..]
                .find('"')
                .ok_or_else(|| format!("approval field `{k}` is unterminated"))?;
            Ok(after[a + 1..a + 1 + b].to_string())
        };
        let hash = field("approved_hash")?;
        if !hash.starts_with("blake3:") || hash.len() != "blake3:".len() + 64 {
            return Err(format!("approved_hash `{hash}` is not a blake3 content hash"));
        }
        Ok(Approval { artifact: field("approved_artifact")?, hash, profile: field("approved_under")? })
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(device: &str, hb: u64, ttl: u64) -> ActuatorEnvelope {
        ActuatorEnvelope {
            device: device.to_string(),
            dims: vec![("angle_deg".to_string(), -30.0, 95.0)],
            rate_hz: None,
            heartbeat_ms: hb,
            ttl_ms: ttl,
            fail_state: FailState::SafePark,
        }
    }

    /// The dead-man fires with no help from the program. Nothing in this test calls the broker
    /// after the grant — exactly the situation a wedged control program is in.
    #[test]
    fn a_lease_nobody_beats_is_revoked_by_the_watchdog_alone() {
        let e = env("arm0/elbow", 40, 60_000);
        let b = DeviceBroker::new(Profile::Sim { seed: 7 }, std::slice::from_ref(&e), &[]);
        let deadline = Instant::now() + Duration::from_secs(5);
        while b.revocation("arm0/elbow").is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        let r = b.revocation("arm0/elbow").expect("the watchdog revokes without being asked");
        assert_eq!(r.cause, RevokeCause::MissedHeartbeat);
        // And the command path is closed afterwards: a revoked lease is not a suggestion.
        let refusal = b.command("arm0/elbow", &[("angle_deg".to_string(), 10.0)]).unwrap_err();
        assert!(
            matches!(refusal, CommandRefusal::Revoked(_)),
            "a command after revocation is Revoked, not Envelope: {refusal:?}"
        );
        // Both halves of the story are in the journal, in order.
        let ops: Vec<String> = b.events().iter().map(|e| e.op.clone()).collect();
        assert_eq!(ops, vec!["lease.revoked", "failstate.engaged"]);
        b.shutdown();
    }

    /// The TTL is the other end of the lease: a program that beats perfectly still does not keep
    /// a device forever. A loan with no end is a transfer.
    #[test]
    fn a_perfectly_beaten_lease_still_expires_at_its_ttl() {
        let e = env("arm0/elbow", 10_000, 60);
        let b = DeviceBroker::new(Profile::Sim { seed: 7 }, std::slice::from_ref(&e), &[]);
        let deadline = Instant::now() + Duration::from_secs(5);
        while b.revocation("arm0/elbow").is_none() && Instant::now() < deadline {
            // Beat constantly — the heartbeat is nowhere near expiry, so only the TTL can end this.
            b.beat_all();
            std::thread::sleep(Duration::from_millis(5));
        }
        let r = b.revocation("arm0/elbow").expect("the TTL ends the lease regardless of beats");
        assert_eq!(r.cause, RevokeCause::TtlExpired);
        b.shutdown();
    }

    // ----- 10g: the operator e-stop ------------------------------------------------------------

    /// The e-stop's whole point: a program that is beating perfectly, driving legally, and doing
    /// nothing wrong still loses its device the moment an operator withdraws the authority — and
    /// loses it WITHOUT being asked anything. Note what this test does not do: it never calls
    /// `command` to trigger the revocation. The arm parks while the program is mid-loop.
    #[test]
    fn an_operator_revoke_parks_every_device_without_the_program_asking() {
        let e = env("arm0/elbow", 10_000, 60_000);
        let live = Arc::new(AtomicBool::new(true));
        let seen = live.clone();
        let b = DeviceBroker::with_authority_watch(
            Profile::Sim { seed: 7 },
            std::slice::from_ref(&e),
            &[],
            Some(Box::new(move |_device| {
                if seen.load(Ordering::SeqCst) {
                    AuthorityState::Live
                } else {
                    AuthorityState::Dead("grant node `g_arm` is revoked (by audit seq 9)".to_string())
                }
            })),
        );
        // Drive it somewhere the park pose is not, so parking is observable.
        b.command("arm0/elbow", &[("angle_deg".to_string(), 42.0)]).expect("in-envelope");
        assert!(b.revocation("arm0/elbow").is_none(), "a live grant keeps the device");
        // The operator pulls the lever.
        live.store(false, Ordering::SeqCst);
        let deadline = Instant::now() + Duration::from_secs(5);
        while b.revocation("arm0/elbow").is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(2));
        }
        let r = b.revocation("arm0/elbow").expect("the watchdog acts on the revoked grant alone");
        assert_eq!(r.cause, RevokeCause::Operator);
        // The fail-state actually engaged — `safe-park` means the joint is AT the park pose, not
        // merely that a flag was set. Checking the flag would pass with an empty fail-state.
        assert_eq!(b.read("arm0/elbow#angle_deg"), Some(0.0), "safe-park drove the joint home");
        // And the audit line says what happened, in the operator's words, without inventing an
        // overdue heartbeat that never occurred.
        let line = b
            .events()
            .into_iter()
            .find(|e| e.op == "lease.revoked")
            .expect("the revocation is journaled");
        assert!(line.detail.contains("operator-revoke"), "{}", line.detail);
        assert!(line.detail.contains("audit seq 9"), "the reason travels: {}", line.detail);
        assert!(!line.detail.contains("overdue"), "an e-stop is not a missed beat: {}", line.detail);
        b.shutdown();
    }

    /// THE SKIP BRANCH, and the one that decides whether any of this is real. A probe that cannot
    /// answer — dead daemon, garbled reply, severed link — must be treated exactly like a probe
    /// that answered "revoked". The tempting `else { continue }` here keeps the arm running
    /// whenever the broker is unreachable, which is the one moment nobody is watching it.
    #[test]
    fn a_probe_that_cannot_answer_parks_the_device_just_like_a_revoke() {
        let e = env("arm0/elbow", 10_000, 60_000);
        let b = DeviceBroker::with_authority_watch(
            Profile::Sim { seed: 7 },
            std::slice::from_ref(&e),
            &[],
            Some(Box::new(|_device| {
                AuthorityState::Dead("the broker did not answer for `g_arm`: pipe closed".to_string())
            })),
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        while b.revocation("arm0/elbow").is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(2));
        }
        let r = b.revocation("arm0/elbow").expect("silence from the broker is not consent");
        assert_eq!(r.cause, RevokeCause::Operator);
        let line = b.events().into_iter().find(|e| e.op == "lease.revoked").expect("journaled");
        assert!(line.detail.contains("did not answer"), "the cause is not hidden: {}", line.detail);
        b.shutdown();
    }

    /// "Revoke the subtree" means the subtree. One operator action must reach every device this
    /// run holds — an e-stop that parks the first arm and leaves the second one driving is worse
    /// than none, because the console now says the machine is safe.
    #[test]
    fn one_operator_revoke_reaches_every_device_the_run_holds() {
        let arm = env("arm0/elbow", 10_000, 60_000);
        let grip = env("arm0/gripper", 10_000, 60_000);
        let b = DeviceBroker::with_authority_watch(
            Profile::Sim { seed: 7 },
            &[arm, grip],
            &[],
            Some(Box::new(|_device| AuthorityState::Dead("g_fleet revoked".to_string()))),
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        while b.revocation("arm0/gripper").is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(2));
        }
        for d in ["arm0/elbow", "arm0/gripper"] {
            let r = b.revocation(d).unwrap_or_else(|| panic!("{d} must be revoked too"));
            assert_eq!(r.cause, RevokeCause::Operator, "{d}");
        }
        b.shutdown();
    }

    /// The subtree cuts where the operator aimed it, and nowhere else. Revoking one arm's node
    /// stops that arm; the other arm — a sibling, not a descendant — keeps working. Without this,
    /// a "revoke" that quietly stopped everything would look identical to a correct one in every
    /// test above, and a warehouse would halt when one robot was e-stopped.
    #[test]
    fn revoking_one_device_leaves_its_sibling_driving() {
        let elbow = env("arm0/elbow", 10_000, 60_000);
        let grip = env("arm1/gripper", 10_000, 60_000);
        let b = DeviceBroker::with_authority_watch(
            Profile::Sim { seed: 7 },
            &[elbow, grip],
            &[],
            Some(Box::new(|device: &str| {
                if device == "arm0/elbow" {
                    AuthorityState::Dead("grant node for arm0/elbow is revoked".to_string())
                } else {
                    AuthorityState::Live
                }
            })),
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        while b.revocation("arm0/elbow").is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(
            b.revocation("arm0/elbow").expect("the aimed device stops").cause,
            RevokeCause::Operator
        );
        assert!(
            b.revocation("arm1/gripper").is_none(),
            "a sibling device must not be collateral: e-stopping one robot is not e-stopping the fleet"
        );
        // And the sibling still accepts commands — it did not merely avoid the revocation record.
        b.command("arm1/gripper", &[("angle_deg".to_string(), 5.0)]).expect("the sibling still drives");
        b.shutdown();
    }

    /// The control case for the e-stop, mirroring the dead-man's: a probe that keeps answering
    /// `Live` never costs the program its device. Without this, a watchdog that revoked
    /// unconditionally would pass every test above.
    #[test]
    fn a_live_grant_is_never_revoked_by_the_authority_watch() {
        let e = env("arm0/elbow", 10_000, 60_000);
        let b = DeviceBroker::with_authority_watch(
            Profile::Sim { seed: 7 },
            std::slice::from_ref(&e),
            &[],
            Some(Box::new(|_device| AuthorityState::Live)),
        );
        std::thread::sleep(Duration::from_millis(400));
        assert!(
            b.revocation("arm0/elbow").is_none(),
            "a live grant, repeatedly confirmed, must not lose the device"
        );
        b.shutdown();
    }

    /// The safety half, and the one a bug would silently break: a lease that IS being beaten must
    /// never be revoked. A dead-man that fires under a healthy program is worse than none, because
    /// it teaches operators to disable it.
    #[test]
    fn a_beaten_lease_is_never_revoked() {
        let e = env("arm0/elbow", 120, 60_000);
        let b = DeviceBroker::new(Profile::Sim { seed: 7 }, std::slice::from_ref(&e), &[]);
        let until = Instant::now() + Duration::from_millis(600);
        while Instant::now() < until {
            b.command("arm0/elbow", &[("angle_deg".to_string(), 5.0)]).expect("in-envelope");
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(
            b.revocation("arm0/elbow").is_none(),
            "five heartbeat periods of healthy driving must not lose the device"
        );
        b.shutdown();
    }

    /// The grant is the authority of record. The capability value's scope is a copy; if the two
    /// ever disagree, the broker's copy decides — which is the only part of "validated twice" that
    /// is real while both checks live in one process.
    #[test]
    fn the_brokers_envelope_decides_even_when_a_wider_one_is_presented() {
        let e = env("arm0/elbow", 10_000, 60_000);
        let b = DeviceBroker::new(Profile::Null, std::slice::from_ref(&e), &[]);
        // A caller that had somehow acquired a widened envelope would pass its own check at 400°.
        let widened = ActuatorEnvelope { dims: vec![("angle_deg".to_string(), -400.0, 400.0)], ..e.clone() };
        assert!(envelope_check(&widened, &[("angle_deg".to_string(), 400.0)]).is_ok());
        // The broker refuses it anyway, because it checks the grant it was given at start.
        let refusal = b.command("arm0/elbow", &[("angle_deg".to_string(), 400.0)]).unwrap_err();
        assert!(matches!(refusal, CommandRefusal::Envelope(_)), "{refusal:?}");
        b.shutdown();
    }

    /// `rate_hz` stops being decorative (build-order D10f). Two commands back to back at 5 Hz
    /// cannot both be honored, and the refusal says so with numbers.
    #[test]
    fn a_command_faster_than_rate_hz_is_refused_with_the_period_named() {
        let e = ActuatorEnvelope { rate_hz: Some(5), ..env("arm0/elbow", 10_000, 60_000) };
        let b = DeviceBroker::new(Profile::Null, std::slice::from_ref(&e), &[]);
        b.command("arm0/elbow", &[("angle_deg".to_string(), 1.0)]).expect("the first is fine");
        let refusal = b.command("arm0/elbow", &[("angle_deg".to_string(), 2.0)]).unwrap_err();
        match refusal {
            CommandRefusal::Envelope(r) => assert!(r.contains("5 Hz"), "the rate is named: {r}"),
            other => panic!("a rate refusal is an envelope refusal: {other:?}"),
        }
        b.shutdown();
    }

    /// Determinism under `--seed` (spec §5.4), and its converse: a different seed must actually
    /// produce a different run, or the seed is decoration.
    #[test]
    fn the_simulator_replays_identically_under_a_seed_and_differs_across_seeds() {
        let sensors = vec!["arm0/strain".to_string()];
        let reads = |seed: u64| {
            let b = DeviceBroker::new(Profile::Sim { seed }, &[], &sensors);
            let v: Vec<f64> = (0..8).map(|_| b.read("arm0/strain").expect("bound in sim")).collect();
            b.shutdown();
            v
        };
        assert_eq!(reads(42), reads(42), "same seed, same measurements");
        assert_ne!(reads(42), reads(43), "a seed that changes nothing is not a seed");
        // And every reading is in the documented range, so nobody mistakes it for a raw unit.
        for x in reads(42) {
            assert!((-1.0..=1.0).contains(&x), "synthetic signal out of its documented range: {x}");
        }
    }

    /// The mirror sensor closes the loop: command the elbow, read the elbow back. This is the only
    /// physically meaningful reading the simulator offers, and it must track commands exactly.
    #[test]
    fn a_mirror_sensor_reads_back_the_position_that_was_commanded() {
        let e = env("arm0/elbow", 10_000, 60_000);
        let sensors = vec!["arm0/elbow#angle_deg".to_string()];
        let b = DeviceBroker::new(Profile::Sim { seed: 1 }, std::slice::from_ref(&e), &sensors);
        assert_eq!(b.read("arm0/elbow#angle_deg"), Some(0.0), "parked at the in-envelope zero");
        b.command("arm0/elbow", &[("angle_deg".to_string(), 42.5)]).unwrap();
        assert_eq!(b.read("arm0/elbow#angle_deg"), Some(42.5));
        b.shutdown();
    }

    /// Invariant 50 through the null adapter, unchanged from 10e: absence reads as absence.
    #[test]
    fn a_sensor_with_no_adapter_reads_nothing_at_all() {
        let b = DeviceBroker::new(Profile::Null, &[], &["arm0/strain".to_string()]);
        assert_eq!(b.read("arm0/strain"), None, "no adapter, no number");
        b.shutdown();
    }

    /// The simulator's own skip branch, and the one a plausible number would hide. A `#` sensor
    /// name claims to mirror a specific dimension of a specific device; when the simulator has no
    /// such device, it must answer absence rather than falling through to the synthetic signal.
    /// The failure this forbids is invisible in the output — a control loop would read a
    /// perfectly ordinary-looking float from a joint that does not exist.
    #[test]
    fn a_mirror_of_a_device_the_simulator_does_not_have_reads_nothing() {
        let e = env("arm0/elbow", 10_000, 60_000);
        let sensors = vec![
            "arm9/ghost#angle_deg".to_string(),
            "arm0/elbow#torque_nm".to_string(),
        ];
        let b = DeviceBroker::new(Profile::Sim { seed: 5 }, std::slice::from_ref(&e), &sensors);
        assert_eq!(b.read("arm9/ghost#angle_deg"), None, "no such simulated device");
        assert_eq!(
            b.read("arm0/elbow#torque_nm"),
            None,
            "the device exists but the envelope never bounded that dimension, so nothing models it"
        );
        b.shutdown();
    }

    /// The fail-state is a physical decision, so the simulator must actually take it. `safe-park`
    /// drives back to the park pose; a lease that dies at 90° does not stay at 90°.
    #[test]
    fn safe_park_returns_the_simulated_device_to_its_park_pose() {
        let e = env("arm0/elbow", 40, 60_000);
        let sensors = vec!["arm0/elbow#angle_deg".to_string()];
        let b = DeviceBroker::new(Profile::Sim { seed: 3 }, std::slice::from_ref(&e), &sensors);
        b.command("arm0/elbow", &[("angle_deg".to_string(), 90.0)]).unwrap();
        assert_eq!(b.read("arm0/elbow#angle_deg"), Some(90.0));
        let deadline = Instant::now() + Duration::from_secs(5);
        while b.revocation("arm0/elbow").is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(b.revocation("arm0/elbow").is_some(), "the lease must have died");
        assert_eq!(
            b.read("arm0/elbow#angle_deg"),
            Some(0.0),
            "safe-park is a movement, not a label"
        );
        b.shutdown();
    }

    /// The park point stays inside the envelope even when the envelope excludes zero — parking
    /// outside the granted range to reach "safety" would be the envelope granting what it refused.
    #[test]
    fn the_park_point_is_always_inside_the_envelope() {
        assert_eq!(park_point(-30.0, 95.0), 0.0);
        assert_eq!(park_point(10.0, 40.0), 10.0, "a range above zero parks at its floor");
        assert_eq!(park_point(-40.0, -10.0), -10.0, "a range below zero parks at its ceiling");
    }

    /// A corrupt approval record must not read as "nothing to check".
    #[test]
    fn an_unreadable_approval_is_an_error_not_an_empty_approval() {
        assert!(Approval::parse("{}").is_err(), "a record with no hash approves nothing");
        assert!(
            Approval::parse("{\"approved_hash\": \"sha256:abc\"}").is_err(),
            "an unrecognized hash form is refused rather than compared loosely"
        );
        let a = Approval {
            artifact: "arm.delulu".to_string(),
            hash: format!("blake3:{}", "a".repeat(64)),
            profile: "sim".to_string(),
        };
        let back = Approval::parse(&a.to_json()).expect("round-trips");
        assert_eq!(back.hash, a.hash);
        assert_eq!(back.artifact, "arm.delulu");
    }

    // ----- D20: the stepped simulator clock -------------------------------------------------------

    /// The point of the stepped clock: lease timing is a deterministic function of the command
    /// SEQUENCE, not of how fast the interpreter runs. The TTL fires at the same command count
    /// whether the caller sleeps between commands or not — and it fires as `ttl-expired`, not
    /// `missed-heartbeat`, because a lease beaten on every interaction never misses a beat. This is
    /// the exact property the satellite demo needs to replay identically in debug and release.
    #[test]
    fn a_stepped_clock_is_deterministic_in_command_count_and_wall_time_independent() {
        let run = |sleep: Option<Duration>| -> (usize, Option<RevokeCause>) {
            // Heartbeat huge (every command beats, so it is never missed), TTL 1000 ms, 100 ms/op.
            let e = env("arm0/elbow", 1_000_000, 1000);
            let b = DeviceBroker::with_config(
                Profile::Sim { seed: 1 },
                std::slice::from_ref(&e),
                &[],
                None,
                ClockMode::Stepped { step_us: 100_000 },
            );
            let mut count = 0usize;
            for _ in 0..100 {
                let r = b.command("arm0/elbow", &[("angle_deg".to_string(), 1.0)]);
                count += 1;
                if r.is_err() {
                    break;
                }
                if let Some(d) = sleep {
                    std::thread::sleep(d);
                }
            }
            let cause = b.revocation("arm0/elbow").map(|r| r.cause);
            b.shutdown();
            (count, cause)
        };
        let fast = run(None);
        let slow = run(Some(Duration::from_millis(12)));
        assert_eq!(fast, slow, "wall-clock sleep between commands must not change the outcome");
        // 1000 ms TTL at 100 ms/op: simulated time crosses the TTL on the 11th interaction.
        assert_eq!(fast.0, 11, "the TTL is a function of the op count, deterministically");
        assert_eq!(fast.1, Some(RevokeCause::TtlExpired), "beaten every op, so it dies by TTL not beat");
    }

    /// The heartbeat is measured in simulated time too: a granted actuator that is never commanded
    /// (only its mirror sensor read) is never beaten, and simulated time from those reads carries
    /// it past `heartbeat_ms` — deterministically, with no watchdog thread and no wall-clock wait.
    #[test]
    fn a_stepped_clock_heartbeat_fires_when_interaction_outpaces_the_beat() {
        let e = env("arm0/elbow", 300, 60_000); // hb 300 ms, ttl 60 s
        let sensors = vec!["arm0/elbow#angle_deg".to_string()];
        let b = DeviceBroker::with_config(
            Profile::Sim { seed: 1 },
            std::slice::from_ref(&e),
            &sensors,
            None,
            ClockMode::Stepped { step_us: 100_000 }, // 100 ms/op
        );
        // Never command the elbow — only read its mirror. Each read advances 100 ms of sim time;
        // once the beat is more than 300 ms stale the lease dies, on the read that crosses.
        let mut count = 0usize;
        for _ in 0..10 {
            let _ = b.read("arm0/elbow#angle_deg");
            count += 1;
            if b.revocation("arm0/elbow").is_some() {
                break;
            }
        }
        let r = b.revocation("arm0/elbow").expect("an unbeaten lease dies on the heartbeat in sim time");
        assert_eq!(r.cause, RevokeCause::MissedHeartbeat);
        assert_eq!(count, 4, "300 ms heartbeat at 100 ms/read: the 4th read crosses it");
        b.shutdown();
    }

    /// The honest limit of stepped mode, tested so it cannot be mistaken for the real dead-man: a
    /// program that STOPS interacting stops the simulated clock, so its lease does NOT expire.
    /// Stepped mode reproduces a demonstration deterministically; the wall-clock guarantee that a
    /// wedged controller loses its actuator is `ClockMode::Wall`, exercised by the tests above.
    #[test]
    fn stepped_mode_does_not_model_the_wedged_program_only_a_wall_clock_can_catch() {
        let e = env("arm0/elbow", 40, 60); // tiny hb + ttl: instant death on a wall clock
        let b = DeviceBroker::with_config(
            Profile::Sim { seed: 1 },
            std::slice::from_ref(&e),
            &[],
            None,
            ClockMode::Stepped { step_us: 1000 },
        );
        b.command("arm0/elbow", &[("angle_deg".to_string(), 5.0)]).expect("in-envelope");
        // The "program" wedges: no further interaction. Real time passes; simulated time does not.
        std::thread::sleep(Duration::from_millis(300)); // >> hb and ttl in wall ms
        assert!(
            b.revocation("arm0/elbow").is_none(),
            "stepped mode advances only on interaction — a wedged program is a WALL-clock concern (D20)"
        );
        b.shutdown();
    }
}
