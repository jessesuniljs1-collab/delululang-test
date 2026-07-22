//! The first real hardware adapter: a **line-protocol subprocess** (`Profile::Hw`).
//!
//! Until now `Profile::Hw { adapter }` existed "so the artifact-hash gate has something real to
//! gate" and nothing was behind it. This module puts something behind it — a device driver that is
//! a *separate operating-system process*, speaking a tiny text protocol over stdin/stdout.
//!
//! # Why a subprocess is the right first adapter, and not a cop-out
//!
//! The property that matters for custody is not "does it wiggle a servo" — it is **does the command
//! leave DeluluLang's guarantee**. A subprocess crosses exactly that boundary: the bytes go to code
//! this project did not write, cannot type-check, and must not trust. Everything the architecture
//! has to get right — envelope enforcement *before* dispatch, a hung driver not wedging the control
//! loop, a lying driver being unable to widen anything — is already fully in play, and is testable
//! without a laboratory.
//!
//! It is also how real device drivers actually get attached in the field: a serial bridge, a CAN
//! gateway, a ROS node, a vendor SDK shim. Each is a process that speaks a protocol. Writing the
//! serial framing into this crate would have picked one bus and one vendor; writing a process
//! boundary picks none.
//!
//! **What it is not.** It is not a driver for any specific hardware, and running it does not mean
//! anything physical moved. `RECORD.md` for the robotics demo and
//! `STAGE10_AUTONOMY_ADDENDUM.md` §4 both still say no hardware ships in-tree, and they stay true:
//! this ships the *socket the driver plugs into*, not the driver.
//!
//! # The protocol
//!
//! Line-oriented, UTF-8, one exchange per command. Requests from DeluluLang:
//!
//! ```text
//! CMD <device> <dim>=<value>[,<dim>=<value>...]
//! READ <device>
//! ```
//!
//! Replies from the adapter:
//!
//! ```text
//! OK                 the command was accepted by the hardware
//! ERR <reason>       the hardware refused it (a REFUSAL, reported as a value)
//! VAL <float>        a sensor reading
//! NODEV              no such device (invariant 50: absent is absent, never a fabricated number)
//! ```
//!
//! Anything else — a malformed line, a closed pipe, silence past the timeout — is a **failure**, not
//! a pass. See [`AdapterError`].
//!
//! # The four rules this module exists to enforce
//!
//! 1. **The envelope is checked HOST-SIDE BEFORE the adapter is spoken to.** The caller
//!    ([`crate::device::DeviceBroker::command`]) validates against the grant first and only then
//!    dispatches. An adapter can therefore refuse *more* and can never permit *more*, whatever it
//!    replies. Build-order D11e noted that host-side and adapter-side checks previously lived in
//!    one process, making that ordering "structural rehearsal" rather than a real split; with a
//!    subprocess the split is real, and the ordering is what makes it worth having.
//! 2. **A hung adapter must not wedge the control loop.** Every exchange has a timeout. Blocking
//!    stdio has no portable deadline, so a reader thread feeds an `mpsc` channel and the exchange
//!    uses `recv_timeout` — std only, no async runtime (head-chef ruling 1), and the same
//!    thread-plus-channel shape the dead-man watchdog already uses.
//! 3. **A failed adapter STAYS failed.** After a timeout or a protocol violation the stream's
//!    framing is no longer trustworthy — a late reply would be read as the answer to the *next*
//!    command. The adapter is poisoned, and every later call fails without touching the pipe. A
//!    half-open device connection is how a robot ends up executing yesterday's instruction.
//! 4. **Silence is never success.** There is no default-OK branch anywhere in this file.
//!
//! The dead-man lease (`device.rs`, invariant 47) sits above all of this: an adapter that dies takes
//! the lease's heartbeat with it, so the declared fail-state engages on schedule without anyone
//! noticing the process is gone.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::Duration;

/// How long any single exchange may take before the adapter is declared unresponsive.
///
/// Deliberately NOT configurable per call: a per-command timeout an operator can stretch is a
/// timeout that will be stretched until it stops protecting anything. The dead-man's `heartbeat_ms`
/// is the tunable that governs real-time behaviour, and it is set at grant time by a human.
pub const EXCHANGE_TIMEOUT: Duration = Duration::from_millis(2000);

/// Why an adapter exchange failed. Every variant is a refusal; none is recoverable in place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdapterError {
    /// The adapter process could not be started.
    Spawn(String),
    /// No reply within [`EXCHANGE_TIMEOUT`]. The adapter is now poisoned.
    Timeout,
    /// The adapter closed its output, or died.
    Closed,
    /// A reply this protocol does not define. Poisons the adapter: once framing is in doubt, a
    /// later line cannot be attributed to the command that caused it.
    Protocol(String),
    /// A previous failure poisoned this adapter; it will not be spoken to again.
    Poisoned,
    /// The adapter refused the command. This is the hardware's own "no" — the only variant that
    /// reflects a working adapter, and the reason it is separated from the failures above.
    Refused(String),
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterError::Spawn(e) => write!(f, "adapter could not be started: {e}"),
            AdapterError::Timeout => {
                write!(f, "adapter did not answer within {} ms", EXCHANGE_TIMEOUT.as_millis())
            }
            AdapterError::Closed => write!(f, "adapter closed its output (died?)"),
            AdapterError::Protocol(l) => write!(f, "adapter spoke outside the protocol: `{l}`"),
            AdapterError::Poisoned => {
                write!(f, "adapter is poisoned by an earlier failure and will not be reused")
            }
            AdapterError::Refused(r) => write!(f, "the hardware refused the command: {r}"),
        }
    }
}

/// A device driver running as a separate process.
pub struct ProcessAdapter {
    name: String,
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<String>,
    /// Set on the first unrecoverable failure and never cleared — see rule 3 in the module docs.
    poisoned: bool,
}

impl ProcessAdapter {
    /// Spawn `program` (with `args`) as a device adapter. `name` is the identity the DL1905
    /// approval record binds, so it is carried for diagnostics rather than derived from the path.
    pub fn spawn(name: &str, program: &str, args: &[String]) -> Result<ProcessAdapter, AdapterError> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // stderr is INHERITED on purpose: a driver's own logging goes to the operator's
            // terminal rather than into a pipe nobody drains, which would eventually block it.
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| AdapterError::Spawn(format!("{program}: {e}")))?;
        let stdin = child.stdin.take().ok_or_else(|| AdapterError::Spawn("no stdin pipe".into()))?;
        let stdout = child.stdout.take().ok_or_else(|| AdapterError::Spawn("no stdout pipe".into()))?;
        // The reader thread exists solely so an exchange can have a deadline: blocking stdio reads
        // have no portable timeout. It ends when the pipe closes, which happens when the child dies
        // or when `Drop` closes our handle.
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        if tx.send(l).is_err() {
                            return; // the adapter was dropped; nothing left to feed
                        }
                    }
                    Err(_) => return,
                }
            }
        });
        Ok(ProcessAdapter { name: name.to_string(), child, stdin, rx, poisoned: false })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Send one request and read one reply, with a deadline. Any failure poisons the adapter.
    fn exchange(&mut self, request: &str) -> Result<String, AdapterError> {
        if self.poisoned {
            return Err(AdapterError::Poisoned);
        }
        if writeln!(self.stdin, "{request}").is_err() || self.stdin.flush().is_err() {
            self.poisoned = true;
            return Err(AdapterError::Closed);
        }
        match self.rx.recv_timeout(EXCHANGE_TIMEOUT) {
            Ok(line) => Ok(line.trim().to_string()),
            Err(RecvTimeoutError::Timeout) => {
                // Poison BEFORE returning: a reply that arrives late would otherwise be read as the
                // answer to whatever command came next.
                self.poisoned = true;
                Err(AdapterError::Timeout)
            }
            Err(RecvTimeoutError::Disconnected) => {
                self.poisoned = true;
                Err(AdapterError::Closed)
            }
        }
    }

    /// Command a device. `fields` are already envelope-validated by the caller — this function must
    /// never be reachable with values the grant did not permit, and it does not re-derive them.
    pub fn command(&mut self, device: &str, fields: &[(String, f64)]) -> Result<(), AdapterError> {
        let dims: Vec<String> = fields.iter().map(|(d, v)| format!("{d}={v}")).collect();
        let reply = self.exchange(&format!("CMD {device} {}", dims.join(",")))?;
        if reply == "OK" {
            Ok(())
        } else if let Some(r) = reply.strip_prefix("ERR ") {
            // A refusal does NOT poison: the adapter is working correctly and saying no, which is
            // exactly what a device is supposed to be able to do.
            Err(AdapterError::Refused(r.to_string()))
        } else {
            self.poisoned = true;
            Err(AdapterError::Protocol(reply))
        }
    }

    /// Read a sensor. `None` means the adapter reported no such device — invariant 50's whole
    /// point: an absent measurement is absent, never a plausible-looking number a control loop
    /// would act on. A protocol violation is an ERROR, never a `None`, so a broken adapter can
    /// never be mistaken for an unplugged sensor.
    pub fn read(&mut self, device: &str) -> Result<Option<f64>, AdapterError> {
        let reply = self.exchange(&format!("READ {device}"))?;
        if reply == "NODEV" {
            return Ok(None);
        }
        if let Some(v) = reply.strip_prefix("VAL ") {
            return match v.trim().parse::<f64>() {
                // A non-finite reading is refused rather than propagated: NaN in a control loop
                // silently defeats every comparison it touches.
                Ok(x) if x.is_finite() => Ok(Some(x)),
                _ => {
                    self.poisoned = true;
                    Err(AdapterError::Protocol(reply))
                }
            };
        }
        self.poisoned = true;
        Err(AdapterError::Protocol(reply))
    }

    /// Has this adapter failed unrecoverably?
    pub fn is_poisoned(&self) -> bool {
        self.poisoned
    }
}

impl Drop for ProcessAdapter {
    fn drop(&mut self) {
        // Kill rather than wait: a driver that has stopped answering will not start now, and a run
        // that hangs on exit is a run an operator has to kill by hand. The dead-man has already
        // engaged the declared fail-state by this point if anything was still held.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drive the adapter with a tiny script instead of real hardware — no fixture binary, no
    /// laboratory. PowerShell on Windows and `sh` elsewhere; both ship with the OS, and both take
    /// the script as a single argument so nothing depends on shell quoting surviving `Command`.
    fn echo_adapter(script: &str) -> ProcessAdapter {
        #[cfg(windows)]
        let (prog, args) = (
            "powershell",
            vec!["-NoProfile".to_string(), "-Command".to_string(), script.to_string()],
        );
        #[cfg(not(windows))]
        let (prog, args) = ("sh", vec!["-c".to_string(), script.to_string()]);
        ProcessAdapter::spawn("test-adapter", prog, &args).expect("spawn")
    }

    /// A "reply the same line to every request" driver.
    fn responder(reply: &str) -> String {
        #[cfg(windows)]
        {
            format!("while ($l = [Console]::In.ReadLine()) {{ Write-Output '{reply}' }}")
        }
        #[cfg(not(windows))]
        {
            format!("while IFS= read -r line; do echo '{reply}'; done")
        }
    }

    /// A driver that echoes the request back inside an `ERR`, so a test can assert on the exact
    /// bytes DeluluLang put on the wire.
    fn echoer() -> String {
        #[cfg(windows)]
        {
            "while ($l = [Console]::In.ReadLine()) { Write-Output \"ERR saw $l\" }".to_string()
        }
        #[cfg(not(windows))]
        {
            "while IFS= read -r line; do echo \"ERR saw $line\"; done".to_string()
        }
    }

    /// A driver that reads forever and never answers — the pathological hung device.
    fn silent() -> String {
        #[cfg(windows)]
        {
            "while ($l = [Console]::In.ReadLine()) { }".to_string()
        }
        #[cfg(not(windows))]
        {
            "cat > /dev/null".to_string()
        }
    }

    #[test]
    fn a_working_adapter_accepts_a_command() {
        let mut a = echo_adapter(&responder("OK"));
        assert!(a.command("arm0/elbow", &[("angle_deg".into(), 12.0)]).is_ok());
        assert!(!a.is_poisoned(), "a normal exchange must not poison the adapter");
    }

    #[test]
    fn a_hardware_refusal_is_a_value_and_does_not_poison() {
        let mut a = echo_adapter(&responder("ERR joint at hard stop"));
        match a.command("arm0/elbow", &[("angle_deg".into(), 12.0)]) {
            Err(AdapterError::Refused(r)) => assert!(r.contains("hard stop"), "{r}"),
            other => panic!("expected a refusal, got {other:?}"),
        }
        assert!(
            !a.is_poisoned(),
            "an adapter saying no is an adapter WORKING; only failures poison it"
        );
    }

    #[test]
    fn a_sensor_reads_a_number_and_an_absent_device_reads_absent() {
        let mut a = echo_adapter(&responder("VAL 21.5"));
        assert_eq!(a.read("arm0/strain").unwrap(), Some(21.5));
        let mut b = echo_adapter(&responder("NODEV"));
        assert_eq!(b.read("arm0/nothing").unwrap(), None, "absent is absent, not 0.0");
    }

    /// Invariant 50 through the adapter: a broken driver must never be mistaken for an unplugged
    /// sensor, because "no device" and "I cannot tell" call for different responses.
    #[test]
    fn a_garbled_reading_is_an_error_never_a_none_and_never_a_number() {
        for bad in ["VAL not-a-number", "VAL NaN", "VAL inf", "hello", ""] {
            let mut a = echo_adapter(&responder(bad));
            let got = a.read("arm0/strain");
            assert!(
                matches!(got, Err(AdapterError::Protocol(_))),
                "`{bad}` must be a protocol error, got {got:?}"
            );
            assert!(a.is_poisoned(), "`{bad}` must poison the adapter");
        }
    }

    #[test]
    fn an_unknown_reply_to_a_command_is_a_protocol_error_not_a_pass() {
        // The dangerous case: something that is not "OK" must never be read as acceptance.
        for bad in ["ok", "OKAY", "ACK", "true", "1", ""] {
            let mut a = echo_adapter(&responder(bad));
            let got = a.command("arm0/elbow", &[("angle_deg".into(), 1.0)]);
            assert!(
                matches!(got, Err(AdapterError::Protocol(_))),
                "`{bad}` must NOT be accepted as success, got {got:?}"
            );
        }
    }

    /// A driver that never answers must fail the command on a deadline, not hang the control loop.
    #[test]
    fn a_silent_adapter_times_out_and_stays_poisoned() {
        let mut a = echo_adapter(&silent());

        let t0 = std::time::Instant::now();
        let got = a.command("arm0/elbow", &[("angle_deg".into(), 1.0)]);
        let waited = t0.elapsed();
        assert_eq!(got, Err(AdapterError::Timeout));
        assert!(
            waited < EXCHANGE_TIMEOUT * 3,
            "the deadline must bound the wait; waited {waited:?}"
        );

        // Rule 3: poisoned stays poisoned. A late reply must never be attributed to a later command.
        assert!(a.is_poisoned());
        assert_eq!(a.command("arm0/elbow", &[("angle_deg".into(), 1.0)]), Err(AdapterError::Poisoned));
        assert_eq!(a.read("arm0/strain"), Err(AdapterError::Poisoned));
    }

    #[test]
    fn a_dead_adapter_fails_closed() {
        // Exits immediately: the pipe closes before any request is answered.
        let mut a = echo_adapter("exit");
        let got = a.command("arm0/elbow", &[("angle_deg".into(), 1.0)]);
        assert!(
            matches!(got, Err(AdapterError::Closed) | Err(AdapterError::Timeout)),
            "a dead adapter must fail, got {got:?}"
        );
        assert!(a.is_poisoned());
    }

    #[test]
    fn spawning_a_program_that_does_not_exist_is_a_refusal_not_a_panic() {
        let got = ProcessAdapter::spawn("nope", "definitely-not-a-real-program-xyzzy", &[]);
        assert!(matches!(got, Err(AdapterError::Spawn(_))), "a missing program must refuse, not panic");
    }

    #[test]
    fn the_request_wire_form_carries_every_commanded_dimension() {
        // The driver echoes the request back inside an ERR, so this reads the exact bytes sent.
        let mut a = echo_adapter(&echoer());
        let got = a.command("arm0/elbow", &[("angle_deg".into(), 12.5), ("torque_nm".into(), 1.4)]);
        match got {
            Err(AdapterError::Refused(r)) => {
                assert!(r.contains("CMD arm0/elbow"), "the device is named: {r}");
                assert!(r.contains("angle_deg=12.5"), "every dimension travels: {r}");
                assert!(r.contains("torque_nm=1.4"), "…including the second: {r}");
            }
            other => panic!("expected the echoed request, got {other:?}"),
        }
    }
}
