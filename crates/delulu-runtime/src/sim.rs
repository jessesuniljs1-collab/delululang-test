//! The reference device simulator, and the device PROCESS it can be run as (P8-03).
//!
//! The simulator was private to [`crate::device`]: a `Profile::Sim` broker drove it in its own
//! process, and nothing else could. P8-03 gives it a second caller — a process that speaks
//! `adapter.rs`'s `CMD`/`READ` line protocol on its standard input and output — so a Verified driver
//! ([`crate::verified_adapter`]) and the transport that carries its frames
//! ([`crate::adapter::LineTransport`]) can be witnessed against a real process boundary, with a
//! device whose readings this repository can predict.
//!
//! **One simulator, two callers.** [`Sim`] is the state and the physics; the broker drives it
//! directly under `Profile::Sim`, and [`serve`] drives the same code from the wire. So a run through
//! a device process and an in-process `sim` run with the same seed and the same command sequence
//! produce the same readings — asserted, not assumed (`sim_device_cli.rs`).
//!
//! **What it is, said plainly.** It models no physics: a commanded dimension becomes that
//! dimension's position, and a sensor that is not a position mirror answers a reproducible synthetic
//! signal that is a pure function of (seed, device, read index). It exists so a demonstration
//! replays identically, and so the parts that DO matter — the envelope, the lease, the dead-man, a
//! driver's frames — can be exercised without a laboratory. Nothing here means hardware moved.
//!
//! **Its own limits are its own.** A device process refuses a command outside the bounds IT was
//! given (`ERR`), which is a device's own hard stop, nothing to do with the grant: the host checked
//! the grant's envelope before the frame was written. Two refusals from two different places, and
//! the run can tell them apart — that is the whole point of the split.

use std::collections::BTreeMap;
use std::io::{BufRead, Write};

use crate::value::ActuatorEnvelope;

/// One simulated device: a position per bounded dimension, and whether the drive is engaged.
#[derive(Default)]
pub struct SimDevice {
    pub pos: BTreeMap<String, f64>,
    pub drive: bool,
}

/// The reference simulator's whole state. Deterministic: identical seed plus identical command
/// sequence yields identical readings, on every platform, forever.
pub struct Sim {
    seed: u64,
    devices: BTreeMap<String, SimDevice>,
    /// Per-sensor read counters — the synthetic signal is a pure function of
    /// (seed, device name, read index), so replaying a run replays its measurements.
    reads: BTreeMap<String, u64>,
}

impl Sim {
    /// The simulator for one run: every actuator at its park pose, every sensor's read counter at 0.
    pub fn new(seed: u64, envelopes: &[ActuatorEnvelope], sensors: &[String]) -> Sim {
        let mut devices = BTreeMap::new();
        for env in envelopes {
            // The simulated device starts at its park pose: the in-envelope value nearest zero.
            // Starting anywhere else would be inventing a position nobody commanded.
            let mut dev = SimDevice { drive: false, ..SimDevice::default() };
            for (dim, lo, hi) in &env.dims {
                dev.pos.insert(dim.clone(), park_point(*lo, *hi));
            }
            devices.insert(env.device.clone(), dev);
        }
        let reads = sensors.iter().map(|s| (s.clone(), 0)).collect();
        Sim { seed, devices, reads }
    }

    /// Drive `device`: each field becomes that dimension's position. Unknown device: nothing happens
    /// (the broker has already refused it; a simulator does not invent a device).
    pub fn drive(&mut self, device: &str, fields: &[(String, f64)]) {
        if let Some(dev) = self.devices.get_mut(device) {
            dev.drive = true;
            for (dim, x) in fields {
                dev.pos.insert(dim.clone(), *x);
            }
        }
    }

    /// Read a sensor. `None` is absence — invariant 50: an absent measurement is absent, never a
    /// plausible-looking number a control loop would act on.
    pub fn read(&mut self, device: &str) -> Option<f64> {
        // A sensor named `ACTUATOR#DIM` mirrors that actuator's simulated position — the only
        // reading in this simulator with a physical meaning, and the one that lets a control loop
        // actually close. Everything else is a reproducible synthetic signal and is described as
        // exactly that.
        if let Some((dev, dim)) = device.split_once('#') {
            // THE SKIP BRANCH. A `#` name is a claim about a specific dimension of a specific
            // device. If the simulator cannot resolve it — no such actuator, no such dimension —
            // the answer is absence, NOT a fall-through to the synthetic source. Handing back a
            // plausible number for a mirror that mirrors nothing is the precise failure invariant 50
            // exists to forbid, and it would look completely normal in the output.
            return self.devices.get(dev).and_then(|d| d.pos.get(dim)).copied();
        }
        if !self.reads.contains_key(device) {
            return None;
        }
        let n = self.reads.entry(device.to_string()).or_insert(0);
        let ix = *n;
        *n += 1;
        Some(synthetic_signal(self.seed, device, ix))
    }

    /// Engage a device's declared fail-state (the broker's dead-man calls this).
    pub fn engage(&mut self, device: &str, fail: crate::device::FailState, env: &ActuatorEnvelope) {
        let Some(dev) = self.devices.get_mut(device) else { return };
        match fail {
            // The drive stays engaged on the last setpoint.
            crate::device::FailState::Hold => dev.drive = true,
            // The drive lets go. This simulator models no gravity and no friction, so the position
            // simply stops changing — the sim is not claiming the mechanism holds its pose, only
            // that nothing further is commanded.
            crate::device::FailState::Coast => dev.drive = false,
            // Driven to the park pose: the in-envelope value nearest zero on each dimension. That
            // interpretation belongs to THIS adapter; a real device declares its own.
            crate::device::FailState::SafePark => {
                dev.drive = true;
                for (dim, lo, hi) in &env.dims {
                    dev.pos.insert(dim.clone(), park_point(*lo, *hi));
                }
            }
        }
    }

    /// Is a device's drive engaged? (Read by the broker's journal and by tests.)
    pub fn drive_engaged(&self, device: &str) -> Option<bool> {
        self.devices.get(device).map(|d| d.drive)
    }
}

/// The park pose for one bounded dimension: the in-envelope value nearest zero.
pub fn park_point(lo: f64, hi: f64) -> f64 {
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
pub fn synthetic_signal(seed: u64, device: &str, index: u64) -> f64 {
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

/// The longest request line the device process will read, matching the bound on a reply line in
/// `adapter.rs`: a peer on the other side of a pipe is untrusted in both directions.
const MAX_LINE_BYTES: u64 = 64 * 1024;

/// Read one line of at most [`MAX_LINE_BYTES`], the way `adapter.rs` reads a reply: a peer that
/// streams without a newline is answered with nothing rather than buffered without limit. Written
/// against `&mut dyn BufRead` (`Read::take` needs a sized receiver, which a trait object is not).
fn read_bounded_line(input: &mut dyn BufRead, buf: &mut Vec<u8>) -> std::io::Result<usize> {
    let mut read = 0usize;
    while read < MAX_LINE_BYTES as usize {
        let mut byte = [0u8; 1];
        match input.read(&mut byte) {
            Ok(0) => break,
            Ok(_) => {
                read += 1;
                buf.push(byte[0]);
                if byte[0] == b'\n' {
                    break;
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(read)
}

/// Run the simulator as a device on a line: read `CMD`/`READ` requests, answer `OK`/`ERR`/`VAL`/
/// `NODEV`, one line per request, until the input ends.
///
/// Every refusal is the DEVICE's: a request it cannot parse, a device it does not have, or a
/// setpoint outside the bounds this process was given (its own hard stop). It never answers `OK` for
/// something it did not do — the law of `adapter.rs` rule 4, on this side of the pipe.
pub fn serve(input: &mut dyn BufRead, output: &mut dyn Write, sim: &mut Sim, bounds: &[ActuatorEnvelope]) {
    loop {
        let mut buf: Vec<u8> = Vec::new();
        // Bounded, like every read of an untrusted peer: a host that streams without a newline gets
        // no answer and ends the conversation, rather than growing this process without limit.
        let n = match read_bounded_line(input, &mut buf) {
            Ok(0) | Err(_) => return,
            Ok(n) => n,
        };
        if n == MAX_LINE_BYTES as usize && buf.last() != Some(&b'\n') {
            return; // over-cap with no line end in sight — fail closed, never keep buffering
        }
        let Ok(line) = String::from_utf8(buf) else { return };
        let reply = answer(line.trim_end_matches(['\n', '\r']), sim, bounds);
        if writeln!(output, "{reply}").is_err() || output.flush().is_err() {
            return;
        }
    }
}

/// One request to one reply. Factored out so the protocol is testable without pipes.
pub fn answer(request: &str, sim: &mut Sim, bounds: &[ActuatorEnvelope]) -> String {
    if let Some(rest) = request.strip_prefix("CMD ") {
        let Some((device, fields)) = rest.split_once(' ') else {
            return "ERR a CMD names a device and its dimensions".to_string();
        };
        let mut parsed: Vec<(String, f64)> = Vec::new();
        for field in fields.split(',') {
            let Some((dim, value)) = field.split_once('=') else {
                return format!("ERR `{}` is not DIM=VALUE", one_line(field));
            };
            let Ok(x) = value.trim().parse::<f64>() else {
                return format!("ERR `{}` is not a number", one_line(value));
            };
            if !x.is_finite() {
                return format!("ERR `{dim}` = {x} is not a finite setpoint");
            }
            parsed.push((dim.trim().to_string(), x));
        }
        if parsed.is_empty() {
            return "ERR a CMD commands at least one dimension".to_string();
        }
        let Some(env) = bounds.iter().find(|e| e.device == device) else {
            return format!("ERR no device `{}` on this bench", one_line(device));
        };
        // The DEVICE's own limit, which is not the grant's: this process was told what the bench can
        // do, and it refuses beyond that whatever the host permitted. `ERR` is a working device
        // saying no.
        if let Err(why) = crate::device::envelope_check(env, &parsed) {
            return format!("ERR the bench refuses it: {}", one_line(&why));
        }
        sim.drive(device, &parsed);
        "OK".to_string()
    } else if let Some(device) = request.strip_prefix("READ ") {
        match sim.read(device.trim()) {
            Some(x) if x.is_finite() => format!("VAL {x}"),
            // A non-finite reading is not a reading. The simulator cannot produce one today; saying
            // so here means a future signal that could never becomes a number a control loop acts on.
            Some(_) => "NODEV".to_string(),
            None => "NODEV".to_string(),
        }
    } else {
        format!("ERR `{}` is not a request this bench speaks", one_line(request))
    }
}

/// One line of at most 80 characters, with the control bytes escaped: this process's answers travel
/// back to an operator's terminal and into a host's diagnostics, and a device that can forge a line
/// of them is the hazard TERMINAL-TEXT-1 recorded.
fn one_line(s: &str) -> String {
    s.chars().take(80).collect::<String>().escape_debug().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::FailState;

    fn env(device: &str) -> ActuatorEnvelope {
        ActuatorEnvelope {
            device: device.to_string(),
            dims: vec![("angle_deg".to_string(), -30.0, 95.0)],
            rate_hz: None,
            heartbeat_ms: 60_000,
            ttl_ms: 60_000,
            fail_state: FailState::SafePark,
        }
    }

    fn rig() -> (Sim, Vec<ActuatorEnvelope>) {
        let bounds = vec![env("arm0/elbow")];
        let sim = Sim::new(7, &bounds, &["arm0/strain".to_string()]);
        (sim, bounds)
    }

    #[test]
    fn the_protocol_answers_each_request_and_refuses_everything_else() {
        let (mut sim, bounds) = rig();
        assert_eq!(answer("CMD arm0/elbow angle_deg=12.5", &mut sim, &bounds), "OK");
        assert_eq!(answer("READ arm0/elbow#angle_deg", &mut sim, &bounds), "VAL 12.5", "the mirror reads the pose");
        assert_eq!(answer("READ arm0/nothing", &mut sim, &bounds), "NODEV", "absence is absence");
        // The device's own hard stop, and the grant has nothing to do with it.
        let refused = answer("CMD arm0/elbow angle_deg=400", &mut sim, &bounds);
        assert!(refused.starts_with("ERR the bench refuses it:"), "{refused}");
        assert_eq!(
            answer("READ arm0/elbow#angle_deg", &mut sim, &bounds),
            "VAL 12.5",
            "a refused command moved nothing"
        );
        // Everything it cannot do is an ERR naming why — never an OK, and never silence.
        for bad in [
            "CMD arm0/elbow",
            "CMD arm0/elbow angle_deg",
            "CMD arm0/elbow angle_deg=x",
            "CMD arm0/elbow angle_deg=inf",
            "CMD arm9/elbow angle_deg=1",
            "CMD arm0/elbow twist_deg=1",
            "HELLO",
            "",
        ] {
            let r = answer(bad, &mut sim, &bounds);
            assert!(r.starts_with("ERR "), "`{bad}` must be refused, got `{r}`");
            assert_eq!(r.lines().count(), 1, "one line per request: `{r}`");
        }
    }

    #[test]
    fn a_devices_answer_is_one_line_whatever_it_is_asked() {
        let (mut sim, bounds) = rig();
        // TERMINAL-TEXT-1's shape, from the device's side: a host's frame cannot make this process
        // write a second line, which a transport would read as the answer to the NEXT frame.
        for hostile in [
            "CMD arm0/elbow angle_deg=1\nOK",
            "CMD x\u{1b}[2J y=1",
            "READ arm0/strain\rVAL 99",
            "CMD arm0/elbow a\nb=1",
        ] {
            let r = answer(hostile, &mut sim, &bounds);
            assert!(!r.contains('\n') && !r.contains('\r'), "`{hostile:?}` produced `{r:?}`");
            assert!(!r.contains('\u{1b}'), "an escape sequence reached the answer: `{r:?}`");
        }
    }

    #[test]
    fn the_same_seed_and_sequence_reads_the_same_numbers() {
        // The promise of `--seed`, now that two callers drive the same simulator.
        let read_three = || {
            let bounds = vec![env("arm0/elbow")];
            let mut sim = Sim::new(42, &bounds, &["arm0/strain".to_string()]);
            let mut out = Vec::new();
            for _ in 0..3 {
                out.push(answer("READ arm0/strain", &mut sim, &bounds));
            }
            out
        };
        let first = read_three();
        assert_eq!(first, read_three(), "same seed, same sequence, same readings");
        assert!(first.iter().all(|r| r.starts_with("VAL ")), "{first:?}");
        assert_ne!(first[0], first[1], "the signal advances with the read index");
    }

    #[test]
    fn serve_speaks_the_protocol_over_a_stream_and_ends_with_its_input() {
        let (mut sim, bounds) = rig();
        let input = "CMD arm0/elbow angle_deg=4\nREAD arm0/elbow#angle_deg\nCMD arm0/elbow angle_deg=900\nREAD gone\n";
        let mut out: Vec<u8> = Vec::new();
        serve(&mut std::io::BufReader::new(input.as_bytes()), &mut out, &mut sim, &bounds);
        let said: Vec<&str> = std::str::from_utf8(&out).unwrap().lines().collect();
        assert_eq!(said.len(), 4, "one line per request: {said:?}");
        assert_eq!(said[0], "OK");
        assert_eq!(said[1], "VAL 4");
        assert!(said[2].starts_with("ERR the bench refuses it:"), "{said:?}");
        assert_eq!(said[3], "NODEV");
    }

    #[test]
    fn serve_answers_nothing_to_a_line_past_the_bound_and_stops() {
        let (mut sim, bounds) = rig();
        // A host that streams without a newline: bounded, no answer, the conversation ends — rather
        // than this process growing until the machine is out of memory.
        let flood = format!("CMD arm0/elbow angle_deg={}", "1".repeat(70 * 1024));
        let mut out: Vec<u8> = Vec::new();
        serve(&mut std::io::BufReader::new(flood.as_bytes()), &mut out, &mut sim, &bounds);
        assert!(out.is_empty(), "it answered an unbounded line: {:?}", String::from_utf8_lossy(&out));
    }

    #[test]
    fn the_fail_state_a_lease_death_engages_is_the_declared_one() {
        let bounds = vec![env("arm0/elbow")];
        let mut sim = Sim::new(1, &bounds, &[]);
        answer("CMD arm0/elbow angle_deg=80", &mut sim, &bounds);
        sim.engage("arm0/elbow", FailState::SafePark, &bounds[0]);
        assert_eq!(sim.read("arm0/elbow#angle_deg"), Some(0.0), "safe-park drives to the park pose");
        assert_eq!(sim.drive_engaged("arm0/elbow"), Some(true));
        sim.engage("arm0/elbow", FailState::Coast, &bounds[0]);
        assert_eq!(sim.drive_engaged("arm0/elbow"), Some(false), "coast lets go");
    }
}
