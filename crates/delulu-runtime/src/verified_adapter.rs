//! P8-02: a device driver whose **logic is a Verified-class plugin** (`V2_P8_DESIGN.md`).
//!
//! A [`crate::adapter::ProcessAdapter`] runs an operator's driver as a process: the code that turns a
//! command into the bytes a device speaks is code DeluluLang cannot read, holding whatever its process
//! can reach. A [`VerifiedAdapter`] splits a driver in two:
//!
//! - **The logic** — a command to the frame the device speaks, the device's reply to an outcome — is a
//!   Verified plugin's DIR, re-proved at load (`plugin::step5_verified`) and interpreted by the host.
//!   Its exports are pure: R-Get ([`crate::plugin::r_get_verified`]) holds each one to an EMPTY row
//!   before the adapter exists, and it is handed no capability — strings and a map of floats, nothing
//!   else — so it can compute frames and can do nothing at all.
//! - **The transport** — writing a frame, reading the reply — is the host's ([`Transport`]). The plugin
//!   reaches nothing the host does not write for it.
//!
//! # The interface
//!
//! Four exports, each R-Get-checked at exactly this type (a missing one, a different type, or a row
//! that names any effect refuses the adapter — the existing DL1502/DL1504 of `p.get`):
//!
//! ```text
//! encode_command(device: Str, fields: Map[Str, Float]) -> Result[Str, Str]   the frame, or the driver's own refusal
//! decode_command(device: Str, reply: Str) -> Option[Str]                      the device's refusal, if it refused
//! encode_read(device: Str) -> Result[Str, Str]                                the frame that asks for a reading
//! decode_read(device: Str, reply: Str) -> Result[Option[Float], Str]          the reading, absence, or a refusal
//! ```
//!
//! Four rather than an `encode`/`decode` pair, so the meaning of a reply never depends on the plugin
//! guessing which question it answers: an `OK` that accepts a command and a `NODEV` that answers a read
//! are different types here, not one value read two ways.
//!
//! # The laws of `adapter.rs`, kept
//!
//! 1. **The envelope is checked before the adapter is called** — by the broker
//!    ([`crate::device::DeviceBroker::command`]), unchanged; a plugin can refuse more and never permit
//!    more.
//! 2. **A deadline on the whole exchange**, not on each part (FRAME-DRIP-1's lesson): the logic's two
//!    calls and the transport's exchange share one [`EXCHANGE_TIMEOUT`]. The logic runs on its own
//!    thread so a computation that runs long cannot hold the broker past it; the plugin's step and
//!    memory budget bounds the computation itself (best-effort, DL1506).
//! 3. **A failed adapter stays failed.** A fault in the plugin's code, a value outside its type, a frame
//!    that is not one line, a non-finite reading, or a transport failure poisons the adapter; every later
//!    call fails without computing or writing anything.
//! 4. **Silence is never success**: there is no default branch that answers `Ok`.
//!
//! **A frame is one line** — no line break, at most [`MAX_FRAME_BYTES`]. A transport frames by line, so
//! a frame holding a line break would put a second request on the wire that no grant check ever saw.
//! Checked before the transport is called.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use delulu_check::ty::{Row, Type};

use crate::adapter::{Adapter, AdapterError, EXCHANGE_TIMEOUT};
use crate::interp::{max_depth_for_stack, Interp};
use crate::plugin::{plugin_err_code, r_get_verified, Limits};
use crate::value::{MapKey, Value};

/// The longest frame a plugin may hand the transport, in bytes — the same bound `adapter.rs` puts on a
/// reply line. Every frame of the reference protocol is a few dozen bytes.
pub const MAX_FRAME_BYTES: usize = 64 * 1024;

/// The native stack of the thread that interprets a driver's logic, and with it the call depth the
/// interpreter allows there ([`max_depth_for_stack`]) — the pair is the invariant, so deep recursion in a
/// driver is DL0905, never an overflow of the host. A driver computes a frame; it needs no deep stack.
const LOGIC_STACK_BYTES: usize = 16 * 1024 * 1024;

/// The host's half of a Verified driver: write one frame to the device and read its one-line reply.
///
/// A transport is the only part of a driver that touches the machine — a serial line, a CAN socket, a
/// process speaking a line protocol (P8-03's reference). It is given the time left of the exchange's one
/// deadline and must answer within it. `Timeout`, `Closed` and `Protocol` poison the adapter; a transport
/// never returns `Refused` (a refusal is the plugin's reading of a reply).
pub trait Transport: Send {
    fn exchange(&mut self, frame: &str, within: Duration) -> Result<String, AdapterError>;
}

/// Why a Verified driver could not be built: the registered code and the reason, as a load refusal says it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterfaceRefusal {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for InterfaceRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// The four exports and the exact type each must have — the interface a Verified driver plugs into.
fn interface() -> [(&'static str, Type); 4] {
    let str_result = || Type::result(Type::Str, Type::Str);
    let pure = |params: Vec<Type>, ret: Type| Type::Fn { params, ret: Box::new(ret), row: Row::pure() };
    [
        (
            "encode_command",
            pure(vec![Type::Str, Type::Map(Box::new(Type::Str), Box::new(Type::Float))], str_result()),
        ),
        ("decode_command", pure(vec![Type::Str, Type::Str], Type::Option(Box::new(Type::Str)))),
        ("encode_read", pure(vec![Type::Str], str_result())),
        (
            "decode_read",
            pure(vec![Type::Str, Type::Str], Type::result(Type::Option(Box::new(Type::Float)), Type::Str)),
        ),
    ]
}

/// Is this re-proved DIR a device driver? Each of the four exports must exist with exactly the
/// interface's type and an empty row — R-Get, as `p.get` applies it. A caller checks this before it starts
/// anything a driver would use (`VerifiedAdapter::new` checks it again).
pub fn check_interface(name: &str, dir: &delulu_check::Dir) -> Result<(), InterfaceRefusal> {
    for (export, ty) in interface() {
        r_get_verified(dir.fn_types.get(export), &ty, export).map_err(|e| InterfaceRefusal {
            code: plugin_err_code(&e),
            message: format!("`{name}` is not a device driver: {}", plugin_err_text(&e)),
        })?;
    }
    Ok(())
}

/// One call into the driver's logic. Plain data: the logic's thread builds the interpreter's values.
enum Call {
    EncodeCommand { device: String, fields: Vec<(String, f64)> },
    DecodeCommand { device: String, reply: String },
    EncodeRead { device: String },
    DecodeRead { device: String, reply: String },
}

/// What a call answered, read back into plain data on the logic's thread. `Fault` is everything that is
/// not an answer of the export's type: a fault in the plugin's code, a spent budget, a value its type
/// does not allow.
enum Answer {
    Frame(Result<String, String>),
    Refusal(Option<String>),
    Reading(Result<Option<f64>, String>),
    Fault(String),
}

/// A device driver whose logic is a re-proved Verified plugin and whose transport is the host's.
pub struct VerifiedAdapter {
    name: String,
    /// `None` once dropped, so the logic's thread sees its channel close and ends.
    calls: Option<Sender<Call>>,
    answers: Receiver<Answer>,
    transport: Box<dyn Transport>,
    /// Set on the first unrecoverable failure and never cleared — law 3.
    poisoned: bool,
}

impl VerifiedAdapter {
    /// Build a driver from a re-proved DIR (the caller ran `step5_verified` and the signature policy on
    /// these very bytes) and a transport. The interface is checked here, by R-Get, before anything runs:
    /// an adapter that exists has four pure exports of exactly the interface's types.
    pub fn new(
        name: &str,
        dir: delulu_check::Dir,
        limits: Limits,
        transport: Box<dyn Transport>,
    ) -> Result<VerifiedAdapter, InterfaceRefusal> {
        check_interface(name, &dir)?;
        let (calls, inbox) = mpsc::channel::<Call>();
        let (outbox, answers) = mpsc::channel::<Answer>();
        std::thread::Builder::new()
            .name("delulu-driver-logic".into())
            .stack_size(LOGIC_STACK_BYTES)
            .spawn(move || serve_logic(dir, limits, inbox, outbox))
            .map_err(|e| InterfaceRefusal {
                code: "DL1508",
                message: format!("`{name}`: the driver's logic could not be given a thread: {e}"),
            })?;
        Ok(VerifiedAdapter { name: name.to_string(), calls: Some(calls), answers, transport, poisoned: false })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Has this adapter failed unrecoverably?
    pub fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    fn poison(&mut self, e: AdapterError) -> AdapterError {
        self.poisoned = true;
        e
    }

    /// Ask the logic one question and wait for its answer until `deadline`.
    fn ask(&mut self, call: Call, deadline: Instant) -> Result<Answer, AdapterError> {
        let sent = self.calls.as_ref().map(|c| c.send(call).is_ok()).unwrap_or(false);
        if !sent {
            return Err(self.poison(AdapterError::Closed));
        }
        match self.answers.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
            Ok(Answer::Fault(f)) => {
                Err(self.poison(AdapterError::Protocol(format!("the driver's logic failed: {f}"))))
            }
            Ok(a) => Ok(a),
            // Poisoned BEFORE returning: an answer that arrives late would otherwise be read as the
            // answer to the next question.
            Err(RecvTimeoutError::Timeout) => Err(self.poison(AdapterError::Timeout)),
            Err(RecvTimeoutError::Disconnected) => Err(self.poison(AdapterError::Closed)),
        }
    }

    /// Hand a frame to the transport: one line, bounded, within the time left.
    fn send_frame(&mut self, frame: String, deadline: Instant) -> Result<String, AdapterError> {
        if frame.is_empty() || frame.len() > MAX_FRAME_BYTES || frame.contains(['\n', '\r']) {
            let shown: String = frame.chars().take(80).collect::<String>().escape_debug().to_string();
            return Err(self.poison(AdapterError::Protocol(format!(
                "the driver's logic computed a frame that is not one line of at most {MAX_FRAME_BYTES} bytes: \"{shown}\""
            ))));
        }
        let within = deadline.saturating_duration_since(Instant::now());
        if within.is_zero() {
            return Err(self.poison(AdapterError::Timeout));
        }
        match self.transport.exchange(&frame, within) {
            Ok(reply) if Instant::now() <= deadline => Ok(reply),
            // A transport that answered after the deadline it was given answered too late, whatever it says.
            Ok(_) => Err(self.poison(AdapterError::Timeout)),
            Err(AdapterError::Refused(r)) => Err(self.poison(AdapterError::Protocol(format!(
                "the transport refused a frame (a refusal is the driver's to read, not the transport's): {r}"
            )))),
            Err(e) => Err(self.poison(e)),
        }
    }

    /// Command a device with fields the broker has already checked against the grant.
    pub fn command(&mut self, device: &str, fields: &[(String, f64)]) -> Result<(), AdapterError> {
        if self.poisoned {
            return Err(AdapterError::Poisoned);
        }
        let deadline = Instant::now() + EXCHANGE_TIMEOUT;
        let frame = match self.ask(Call::EncodeCommand { device: device.into(), fields: fields.to_vec() }, deadline)? {
            Answer::Frame(Ok(frame)) => frame,
            // The driver's own refusal: nothing was written, and a driver saying no is working.
            Answer::Frame(Err(r)) => return Err(AdapterError::Refused(r)),
            _ => return Err(self.poison(AdapterError::Protocol("the logic answered out of turn".into()))),
        };
        let reply = self.send_frame(frame, deadline)?;
        match self.ask(Call::DecodeCommand { device: device.into(), reply }, deadline)? {
            Answer::Refusal(None) => Ok(()),
            Answer::Refusal(Some(r)) => Err(AdapterError::Refused(r)),
            _ => Err(self.poison(AdapterError::Protocol("the logic answered out of turn".into()))),
        }
    }

    /// Read a sensor. `None` is the device's "no such device" as the driver read it — never a number.
    pub fn read(&mut self, device: &str) -> Result<Option<f64>, AdapterError> {
        if self.poisoned {
            return Err(AdapterError::Poisoned);
        }
        let deadline = Instant::now() + EXCHANGE_TIMEOUT;
        let frame = match self.ask(Call::EncodeRead { device: device.into() }, deadline)? {
            Answer::Frame(Ok(frame)) => frame,
            Answer::Frame(Err(r)) => return Err(AdapterError::Refused(r)),
            _ => return Err(self.poison(AdapterError::Protocol("the logic answered out of turn".into()))),
        };
        let reply = self.send_frame(frame, deadline)?;
        match self.ask(Call::DecodeRead { device: device.into(), reply }, deadline)? {
            Answer::Reading(Ok(Some(x))) if x.is_finite() => Ok(Some(x)),
            // A non-finite reading is refused rather than propagated: NaN in a control loop silently
            // defeats every comparison it touches — whoever computed it.
            Answer::Reading(Ok(Some(x))) => Err(self.poison(AdapterError::Protocol(format!(
                "the driver read a non-finite value ({x}) for `{device}`"
            )))),
            Answer::Reading(Ok(None)) => Ok(None),
            Answer::Reading(Err(r)) => Err(AdapterError::Refused(r)),
            _ => Err(self.poison(AdapterError::Protocol("the logic answered out of turn".into()))),
        }
    }
}

impl Adapter for VerifiedAdapter {
    fn command(&mut self, device: &str, fields: &[(String, f64)]) -> Result<(), AdapterError> {
        VerifiedAdapter::command(self, device, fields)
    }
    fn read(&mut self, device: &str) -> Result<Option<f64>, AdapterError> {
        VerifiedAdapter::read(self, device)
    }
}

impl Drop for VerifiedAdapter {
    fn drop(&mut self) {
        // Close the channel the logic waits on: its thread ends after the call it is in, which its
        // step budget bounds. It is not joined — a run never waits on a driver to finish exiting.
        self.calls = None;
    }
}

/// The logic's thread: one fresh interpreter per call over the re-proved module, so nothing a call
/// computes survives into the next — the plugin is a function of its arguments, call by call.
fn serve_logic(dir: delulu_check::Dir, limits: Limits, inbox: Receiver<Call>, outbox: Sender<Answer>) {
    let (steps, mem_bytes) = crate::plugin::effective_interp_limits(&limits);
    let depth = max_depth_for_stack(LOGIC_STACK_BYTES);
    while let Ok(call) = inbox.recv() {
        let interp = Interp::new(&dir.module).with_max_depth(depth).with_plugin_budget(steps, mem_bytes);
        let answer = match call {
            Call::EncodeCommand { device, fields } => {
                let map: BTreeMap<MapKey, Value> =
                    fields.into_iter().map(|(d, x)| (MapKey::Str(d), Value::Float(x))).collect();
                let args = vec![Value::str(device), Value::Map(Rc::new(RefCell::new(map)))];
                run(&interp, "encode_command", args, read_frame)
            }
            Call::DecodeCommand { device, reply } => {
                run(&interp, "decode_command", vec![Value::str(device), Value::str(reply)], read_refusal)
            }
            Call::EncodeRead { device } => run(&interp, "encode_read", vec![Value::str(device)], read_frame),
            Call::DecodeRead { device, reply } => {
                run(&interp, "decode_read", vec![Value::str(device), Value::str(reply)], read_reading)
            }
        };
        if outbox.send(answer).is_err() {
            return; // the adapter is gone
        }
    }
}

fn run(interp: &Interp, export: &str, args: Vec<Value>, read: fn(&Value) -> Option<Answer>) -> Answer {
    match interp.call_with(export, args) {
        Ok(v) => read(&v).unwrap_or_else(|| {
            Answer::Fault(format!("`{export}` returned `{}`, which its type does not allow", v.display()))
        }),
        Err(f) => Answer::Fault(format!("`{export}`: {} {}", f.code, f.message)),
    }
}

fn plugin_err_text(e: &crate::plugin::PluginErr) -> String {
    use crate::plugin::PluginErr as E;
    match e {
        E::NotGranted(m) | E::VerifyFailed(m) | E::BadArtifact(m) | E::LimitExceeded(m) | E::ApiMismatch(m) => {
            m.clone()
        }
        E::Revoked(n) => format!("revoked ({n})"),
    }
}

fn variant(v: &Value) -> Option<(&str, &[Value])> {
    match v {
        Value::Variant { name, fields } => Some((&**name, &fields[..])),
        _ => None,
    }
}

fn string(v: &Value) -> Option<String> {
    match v {
        Value::Str(s) => Some(s.to_string()),
        _ => None,
    }
}

/// `Result[Str, Str]`.
fn read_frame(v: &Value) -> Option<Answer> {
    match variant(v)? {
        ("Ok", [s]) => Some(Answer::Frame(Ok(string(s)?))),
        ("Err", [s]) => Some(Answer::Frame(Err(string(s)?))),
        _ => None,
    }
}

/// `Option[Str]`.
fn read_refusal(v: &Value) -> Option<Answer> {
    match variant(v)? {
        ("None", []) => Some(Answer::Refusal(None)),
        ("Some", [s]) => Some(Answer::Refusal(Some(string(s)?))),
        _ => None,
    }
}

/// `Result[Option[Float], Str]`.
fn read_reading(v: &Value) -> Option<Answer> {
    match variant(v)? {
        ("Ok", [o]) => match variant(o)? {
            ("None", []) => Some(Answer::Reading(Ok(None))),
            ("Some", [Value::Float(x)]) => Some(Answer::Reading(Ok(Some(*x)))),
            _ => None,
        },
        ("Err", [s]) => Some(Answer::Reading(Err(string(s)?))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::{DeviceBroker, FailState, Profile};
    use crate::plugin::{step5_verified, PluginArtifact};
    use crate::value::ActuatorEnvelope;
    use std::sync::{Arc, Mutex};

    /// The reference driver as it ships (`examples/line_driver`): `adapter.rs`'s line protocol as a Verified plugin.
    const LINE_DRIVER: &str = include_str!("../../../examples/line_driver/src/lib.delulu");

    /// The rest of the reference driver, for a test that replaces one export.
    fn driver_with(encode_command: &str) -> String {
        let start = LINE_DRIVER.find("pub fn encode_command").unwrap();
        let end = LINE_DRIVER.find("pub fn decode_command").unwrap();
        format!("{}{encode_command}\n\n{}", &LINE_DRIVER[..start], &LINE_DRIVER[end..])
    }

    /// Re-prove `code` the way a load does (`step5_verified`) and hand back its DIR.
    fn reproved(code: &str) -> delulu_check::Dir {
        let c = delulu_check::check_source(0, code);
        assert!(!c.has_errors(), "test driver must check clean: {:?}", c.diagnostics);
        let art = PluginArtifact {
            manifest: serde_json::json!({
                "name": "line_driver", "version": "0.1.0", "api": 1, "class": "verified",
                "authority": { "effects": [], "requires": [] }, "exports": {},
            }),
            class: "verified".into(),
            api: 1,
            dir: Some(delulu_check::dir_serialize(&c.module, &c.result)),
            wasm: None,
            sig: None,
            lock: None,
            wasm_cache_valid: true,
        };
        step5_verified(&art).expect("the driver re-proves").dir
    }

    type Answerer = Box<dyn FnMut(&str) -> Result<String, AdapterError> + Send>;

    /// An in-process device: logs every frame the host writes and answers from a script. The log is
    /// the witness for "nothing was written" — what crossed to the machine, seen from the machine's side.
    struct Fake {
        log: Arc<Mutex<Vec<String>>>,
        within: Arc<Mutex<Vec<Duration>>>,
        answer: Answerer,
    }

    impl Transport for Fake {
        fn exchange(&mut self, frame: &str, within: Duration) -> Result<String, AdapterError> {
            self.log.lock().unwrap().push(frame.to_string());
            self.within.lock().unwrap().push(within);
            (self.answer)(frame)
        }
    }

    type Log = Arc<Mutex<Vec<String>>>;

    fn rig(
        code: &str,
        limits: Limits,
        answer: impl FnMut(&str) -> Result<String, AdapterError> + Send + 'static,
    ) -> (VerifiedAdapter, Log, Arc<Mutex<Vec<Duration>>>) {
        let log: Log = Arc::default();
        let within = Arc::default();
        let fake = Fake { log: Arc::clone(&log), within: Arc::clone(&within), answer: Box::new(answer) };
        let a = VerifiedAdapter::new("line_driver", reproved(code), limits, Box::new(fake)).expect("an adapter");
        (a, log, within)
    }

    fn written(log: &Log) -> Vec<String> {
        log.lock().unwrap().clone()
    }

    fn angle(x: f64) -> Vec<(String, f64)> {
        vec![("angle_deg".to_string(), x)]
    }

    #[test]
    fn a_verified_driver_commands_and_reads_through_the_hosts_transport() {
        let mut replies = vec!["OK", "ERR hard stop at 60", "VAL 21.5", "NODEV", "OK"].into_iter();
        let (mut a, log, _) =
            rig(LINE_DRIVER, Limits::default(), move |_| Ok(replies.next().expect("a reply").to_string()));
        assert_eq!(a.command("arm0/elbow", &angle(10.5)), Ok(()));
        // The device's own refusal is a refusal, and the driver keeps working after it.
        assert_eq!(a.command("arm0/elbow", &angle(70.0)), Err(AdapterError::Refused("hard stop at 60".into())));
        assert!(!a.is_poisoned(), "a device saying no is a working device");
        assert_eq!(a.read("arm0/strain"), Ok(Some(21.5)));
        assert_eq!(a.read("arm0/gone"), Ok(None), "NODEV is absence, never a number");
        let two = vec![("x".to_string(), 1.0), ("y".to_string(), -2.25)];
        assert_eq!(a.command("gantry", &two), Ok(()));
        assert_eq!(
            written(&log),
            [
                "CMD arm0/elbow angle_deg=10.5",
                "CMD arm0/elbow angle_deg=70.0",
                "READ arm0/strain",
                "READ arm0/gone",
                "CMD gantry x=1.0,y=-2.25",
            ],
            "the frames the plugin computed are the frames the host wrote"
        );
    }

    #[test]
    fn the_interface_is_checked_before_the_adapter_exists() {
        let refusal = |code: &str| {
            let fake = Fake { log: Arc::default(), within: Arc::default(), answer: Box::new(|_| Ok("OK".into())) };
            VerifiedAdapter::new("d", reproved(code), Limits::default(), Box::new(fake)).err().expect("refused")
        };
        // An export missing.
        let start = LINE_DRIVER.find("pub fn decode_read").unwrap();
        let r = refusal(&LINE_DRIVER[..start]);
        assert_eq!(r.code, "DL1502", "{r}");
        assert!(r.message.contains("decode_read"), "{r}");
        // An export of another type.
        let r = refusal(&driver_with(
            "pub fn encode_command(device: Str, fields: Map[Str, Int]) -> Result[Str, Str] { Ok(device) }",
        ));
        assert_eq!(r.code, "DL1504", "{r}");
        assert!(r.message.contains("encode_command"), "{r}");
        // A driver that asks for a capability does not fit: its parameters are not the interface's.
        let r = refusal(&driver_with(
            "pub fn encode_command(c: Cap[Console], device: Str, fields: Map[Str, Float]) -> Result[Str, Str] ! {Write} {\n  c.println(device)\n  Ok(device)\n}",
        ));
        assert_eq!(r.code, "DL1504", "{r}");
        // A row that names an effect, even one the body never performs, is refused: the interface is pure.
        let r = refusal(&driver_with(
            "pub fn encode_command(device: Str, fields: Map[Str, Float]) -> Result[Str, Str] ! {Write} { Ok(device) }",
        ));
        assert_eq!(r.code, "DL1504", "{r}");
        assert!(r.message.contains("Write"), "the refusal names the effect: {r}");
    }

    #[test]
    fn a_frame_that_is_not_one_line_is_never_written() {
        for frame in [r#""CMD arm0/elbow angle_deg=10\nCMD arm0/elbow angle_deg=900""#, r#""CMD a x=1\r""#, r#""""#] {
            let code = driver_with(&format!(
                "pub fn encode_command(device: Str, fields: Map[Str, Float]) -> Result[Str, Str] {{ Ok({frame}) }}"
            ));
            let (mut a, log, _) = rig(&code, Limits::default(), |_| Ok("OK".into()));
            let e = a.command("arm0/elbow", &angle(10.0)).expect_err("refused");
            assert!(matches!(e, AdapterError::Protocol(ref m) if m.contains("not one line")), "{frame}: {e:?}");
            assert!(a.is_poisoned(), "{frame}");
            assert_eq!(a.command("arm0/elbow", &angle(10.0)), Err(AdapterError::Poisoned));
            assert!(written(&log).is_empty(), "{frame}: nothing reached the device: {:?}", written(&log));
        }
    }

    #[test]
    fn a_fault_in_the_drivers_logic_poisons_it_before_anything_is_written() {
        let cases: [(&str, Limits, &str); 3] = [
            // The plugin's own runtime bug.
            (
                "pub fn encode_command(device: Str, fields: Map[Str, Float]) -> Result[Str, Str] { let z = fields.len() - fields.len()\n  Ok(device + str(10 / z)) }",
                Limits::default(),
                "DL0902",
            ),
            // A spin, ended by the best-effort step budget.
            (
                "pub fn encode_command(device: Str, fields: Map[Str, Float]) -> Result[Str, Str] { while true { let _k = 1 }\n  Ok(device) }",
                Limits { fuel: 5_000, mem_mb: 0, wall_ms: 0 },
                "DL1506",
            ),
            // Recursion deeper than the logic's thread allows: the depth bound, never the host's stack.
            (
                "fn down(n: Int) -> Int { if n == 0 { 0 } else { 1 + down(n - 1) } }\npub fn encode_command(device: Str, fields: Map[Str, Float]) -> Result[Str, Str] { Ok(device + str(down(1000000))) }",
                Limits::default(),
                "DL0905",
            ),
        ];
        for (export, limits, code) in cases {
            let (mut a, log, _) = rig(&driver_with(export), limits, |_| Ok("OK".into()));
            let e = a.command("arm0/elbow", &angle(10.0)).expect_err("a faulted driver refuses");
            assert!(matches!(e, AdapterError::Protocol(ref m) if m.contains(code)), "{code}: {e:?}");
            assert!(a.is_poisoned(), "{code}");
            assert_eq!(a.read("arm0/strain"), Err(AdapterError::Poisoned), "{code}");
            assert!(written(&log).is_empty(), "{code}: nothing reached the device");
        }
    }

    #[test]
    fn a_non_finite_reading_is_refused_and_poisons() {
        // The language's `parse_float` refuses "NaN" and "inf" — the reference driver reads them as a
        // device's unreadable value, a refusal — but its arithmetic can still compute one, so the host
        // checks what the plugin returns, not what the device sent.
        let (mut a, _, _) = rig(LINE_DRIVER, Limits::default(), |_| Ok("VAL NaN".into()));
        assert!(matches!(a.read("arm0/strain"), Err(AdapterError::Refused(_))));
        for value in ["0.0 / 0.0", "1.0 / 0.0", "-1.0e308 * 10.0"] {
            let start = LINE_DRIVER.find("pub fn decode_read").unwrap();
            let code = format!(
                "{}pub fn decode_read(device: Str, reply: Str) -> Result[Option[Float], Str] {{ Ok(Some({value})) }}\n",
                &LINE_DRIVER[..start]
            );
            let (mut a, _, _) = rig(&code, Limits::default(), |_| Ok("VAL 1".into()));
            let e = a.read("arm0/strain").expect_err("refused");
            assert!(matches!(e, AdapterError::Protocol(ref m) if m.contains("non-finite")), "{value}: {e:?}");
            assert!(a.is_poisoned(), "{value}");
        }
    }

    #[test]
    fn a_transport_failure_poisons_and_a_transport_cannot_refuse() {
        for (failure, expect) in [
            (AdapterError::Timeout, AdapterError::Timeout),
            (AdapterError::Closed, AdapterError::Closed),
            (AdapterError::Refused("nope".into()), AdapterError::Protocol(String::new())),
        ] {
            let f = failure.clone();
            let (mut a, log, _) = rig(LINE_DRIVER, Limits::default(), move |_| Err(f.clone()));
            let e = a.command("arm0/elbow", &angle(10.0)).expect_err("failed");
            assert_eq!(std::mem::discriminant(&e), std::mem::discriminant(&expect), "{failure:?} → {e:?}");
            assert!(a.is_poisoned(), "{failure:?}");
            assert_eq!(a.command("arm0/elbow", &angle(10.0)), Err(AdapterError::Poisoned));
            assert_eq!(written(&log).len(), 1, "{failure:?}: one frame, then never another");
        }
    }

    #[test]
    fn the_exchange_has_one_deadline_and_a_late_answer_is_too_late() {
        // The transport is handed what is LEFT of one deadline, not a fresh one of its own.
        let (mut a, _, within) = rig(LINE_DRIVER, Limits::default(), |_| Ok("OK".into()));
        a.command("arm0/elbow", &angle(1.0)).unwrap();
        let w = within.lock().unwrap()[0];
        assert!(w < EXCHANGE_TIMEOUT, "the transport got {w:?}, the whole of a fresh deadline");
        // An answer that comes after that deadline is a timeout, whatever it says.
        let (mut a, _, _) = rig(LINE_DRIVER, Limits::default(), |_| {
            std::thread::sleep(EXCHANGE_TIMEOUT + Duration::from_millis(50));
            Ok("OK".into())
        });
        assert_eq!(a.command("arm0/elbow", &angle(1.0)), Err(AdapterError::Timeout));
        assert!(a.is_poisoned());
    }

    #[test]
    fn the_broker_checks_the_envelope_before_a_verified_driver_is_called() {
        let (a, log, _) = rig(LINE_DRIVER, Limits::default(), |_| Ok("OK".into()));
        let env = ActuatorEnvelope {
            device: "arm0/elbow".into(),
            dims: vec![("angle_deg".to_string(), -30.0, 95.0)],
            rate_hz: None,
            heartbeat_ms: 60_000,
            ttl_ms: 60_000,
            fail_state: FailState::SafePark,
        };
        let b = DeviceBroker::with_adapter(
            Profile::Hw { adapter: "line_driver".into() },
            std::slice::from_ref(&env),
            &[],
            None,
            crate::device::ClockMode::Wall,
            Some(Box::new(a)),
        );
        assert!(b.command("arm0/elbow", &angle(400.0)).is_err(), "out of the envelope");
        assert!(written(&log).is_empty(), "refused before the driver: {:?}", written(&log));
        assert!(b.command("arm0/elbow", &angle(45.0)).is_ok());
        assert_eq!(written(&log), ["CMD arm0/elbow angle_deg=45.0"]);
        b.shutdown();
    }
}
