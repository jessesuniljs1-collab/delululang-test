//! `delulu-sandbox-channel/3` (PS-A-02; `/2` since RW 4.23 added [`ReqBody::Confined`]; `/3` since
//! PS-E-01 split the host's first frame in two): the wire between a guest interpreter and the host that
//! performs its effects.
//!
//! **The order, by construction (PS-E-01).** The host opens with an [`Open`] frame — the version and
//! this run's generation, and no program. The guest locks itself down and answers with
//! [`ReqBody::Confined`], its self-applied layers and the generation echoed back. Only when the host
//! has accepted that report does it send the [`Program`] frame. Until `/3` the host's first frame WAS
//! the program, so a guest that never confined itself had already been handed it.
//!
//! The framing is the broker's, deliberately: length-prefixed (u32 little-endian) canonical CBOR
//! with a hard per-frame bound and a version tag in every request (`broker_ipc`, head-chef ruling 2).
//! Reusing it adds no dependency and no second set of transport bugs to find.
//!
//! **What may cross.** Plain data crosses by value. A capability crosses only as an opaque
//! [`Handle`] minted by the host: the guest never holds a path, a socket or a secret, so a guest
//! that lies about its own state still cannot name a resource the host did not give it. Everything
//! else — a closure, the root, a foreign handle or pointer, a Python object, an actor reference —
//! is **refused** rather than approximated, because a value that cannot cross honestly must not
//! cross at all (fail-closed, invariant 27's habit).
//!
//! The host performs the operation with the checks it already makes: the sink is a transport, never
//! a second authority.

use std::io::{self, Read, Write};

use serde::{Deserialize, Serialize};

use crate::value::{CapVal, Value, VariantFields};

/// The wire protocol version, present in every request frame.
pub const CHANNEL_VERSION: &str = "delulu-sandbox-channel/3";

/// The only words a guest may report in [`ReqBody::Confined`]: the boundaries a guest applies to
/// ITSELF (Landlock and seccomp on Linux, `delulu`'s `jail.rs`). A report of anything else is refused,
/// so a guest cannot put a claim into the run report that the host would not have made — it can only
/// say which of these known layers took hold on its kernel. `jail.rs` is bound to this list by the
/// refusal itself: a word it adds without adding it here fails every Linux sandboxed run, loudly.
pub const SELF_APPLIED: &[&str] = &[
    "no file writes",
    "no file writes but truncation",
    "reads only from the system paths",
    "no TCP bind or connect",
    "no new programs",
    "no debugger",
    "no namespace or module tricks",
    // PS-E-03 H2 (GUEST-SOCKET-1): the Linux guest's filter refuses every new socket, of every family —
    // Landlock's "no TCP bind or connect" alone left UDP, netlink and Unix sockets open.
    "no sockets but the channel",
    // PS-C-03: the microVM guest asks its own kernel for an IPv4 and an IPv6 socket before its program
    // runs, and reports this only when BOTH are refused as an unsupported family — a measurement, not
    // a reading of the kernel's configuration.
    "no network stack in its kernel",
    // SANDBOX-STOP-1 at L2: the microVM guest caps its own data below the VM's free memory, so a
    // runaway allocation is refused to the allocator (and named) before the guest kernel runs out.
    "memory refused to the guest before its kernel runs out",
];

/// Text a GUEST chose — a word in its confinement report, a method name, a decoder's quotation of its
/// frame — as it may reach the operator: control characters escaped, so it cannot break a line, move the
/// cursor or forge a row of `audit query`; and at most `max_chars` characters, with how many bytes were
/// cut, so a guest cannot make the host write megabytes into a report or the audit chain. (The red-team
/// pass on `/3`, 2026-09-28: a refused word carrying `\n` and ANSI escapes forged audit rows and terminal
/// lines, and a 10 MB word became a 10 MB report and a 20 MB chain.)
pub fn shown(text: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (taken, (i, c)) in text.char_indices().enumerate() {
        if taken == max_chars {
            out.push_str(&format!("… (+{} bytes)", text.len() - i));
            return out;
        }
        if c.is_control() || c == '\u{2028}' || c == '\u{2029}' {
            out.extend(c.escape_default());
        } else {
            out.push(c);
        }
    }
    out
}

/// How much of one refusal a report and the chain keep.
pub const MAX_DENIED_CHARS: usize = 512;

/// Hard ceiling on one frame (16 MiB), as on the broker wire: a corrupt or hostile length prefix
/// must not make the peer allocate unboundedly.
pub const MAX_FRAME: u32 = 16 * 1024 * 1024;

/// An opaque capability reference. The number means nothing outside the host's table for one run.
pub type Handle = u64;

/// A value as it crosses the channel.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WireValue {
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<WireValue>),
    Record { name: String, fields: Vec<(String, WireValue)> },
    Variant { name: String, fields: Vec<WireValue> },
    /// A capability, by host-minted handle plus its KIND. The guest needs the kind to type-check its
    /// own use of the value; it never learns the scope, which is the part that names a resource.
    Cap { handle: Handle, kind: String },
}

/// Why a value may not cross. Carried as a refusal, never silently dropped or coerced.
#[derive(Debug, PartialEq, Eq)]
pub struct Untransferable(pub &'static str);

impl WireValue {
    /// Encode a runtime value for the wire. `cap` maps a capability to the handle the host minted.
    pub fn from_value(v: &Value, cap: &mut impl FnMut(&std::rc::Rc<CapVal>) -> Handle) -> Result<WireValue, Untransferable> {
        Ok(match v {
            Value::Unit => WireValue::Unit,
            Value::Bool(b) => WireValue::Bool(*b),
            Value::Int(i) => WireValue::Int(*i),
            Value::Float(f) => WireValue::Float(*f),
            Value::Str(s) => WireValue::Str(s.to_string()),
            Value::List(items) => WireValue::List(
                items.borrow().iter().map(|x| WireValue::from_value(x, cap)).collect::<Result<_, _>>()?,
            ),
            Value::Record { name, fields } => WireValue::Record {
                name: name.to_string(),
                fields: fields
                    .borrow()
                    .iter()
                    .map(|(k, x)| Ok((k.clone(), WireValue::from_value(x, cap)?)))
                    .collect::<Result<Vec<_>, Untransferable>>()?,
            },
            Value::Variant { name, fields } => WireValue::Variant {
                name: name.to_string(),
                fields: fields.iter().map(|x| WireValue::from_value(x, cap)).collect::<Result<_, _>>()?,
            },
            Value::Cap(c) => WireValue::Cap { handle: cap(c), kind: c.kind.name().to_string() },
            Value::Closure(_) => return Err(Untransferable("a closure")),
            Value::Root(_) => return Err(Untransferable("the root")),
            Value::Secret(_) => return Err(Untransferable("a secret")),
            Value::Foreign(_) => return Err(Untransferable("a foreign library handle")),
            Value::ForeignPtr(_) => return Err(Untransferable("a foreign pointer")),
            #[cfg(feature = "python")]
            Value::PyObj(_) => return Err(Untransferable("a Python object")),
            other => return Err(Untransferable(leaked_kind(other))),
        })
    }

    /// Decode a wire value. `cap` resolves a handle the peer sent; an unknown handle is refused.
    pub fn into_value(self, cap: &mut impl FnMut(Handle) -> Option<std::rc::Rc<CapVal>>) -> Result<Value, Untransferable> {
        Ok(match self {
            WireValue::Unit => Value::Unit,
            WireValue::Bool(b) => Value::Bool(b),
            WireValue::Int(i) => Value::Int(i),
            WireValue::Float(f) => Value::Float(f),
            WireValue::Str(s) => Value::Str(s.into()),
            WireValue::List(items) => Value::List(std::rc::Rc::new(std::cell::RefCell::new(
                items.into_iter().map(|x| x.into_value(cap)).collect::<Result<Vec<_>, _>>()?,
            ))),
            WireValue::Record { name, fields } => Value::Record {
                name: name.into(),
                fields: std::rc::Rc::new(std::cell::RefCell::new(
                    fields
                        .into_iter()
                        .map(|(k, x)| Ok((k, x.into_value(cap)?)))
                        .collect::<Result<Vec<_>, Untransferable>>()?,
                )),
            },
            WireValue::Variant { name, fields } => Value::Variant {
                name: name.into(),
                fields: VariantFields::new(
                    fields.into_iter().map(|x| x.into_value(cap)).collect::<Result<Vec<_>, _>>()?,
                ),
            },
            WireValue::Cap { handle, .. } => {
                Value::Cap(cap(handle).ok_or(Untransferable("an unknown capability handle"))?)
            }
        })
    }
}

impl WireValue {
    /// Decode as the GUEST does: every capability becomes a handle-scoped [`CapVal`], carrying the
    /// kind the host named and no scope at all. The guest cannot manufacture a scope this way, and
    /// [`crate::prim::call_cap_method`] refuses such a capability if it ever reaches the local path.
    pub fn into_guest_value(self) -> Result<Value, Untransferable> {
        Ok(match self {
            WireValue::Cap { handle, kind } => {
                let kind = delulu_check::ResourceKind::from_name(&kind).ok_or(Untransferable("an unknown capability kind"))?;
                Value::Cap(std::rc::Rc::new(CapVal { kind, scope: crate::value::CapScope::Handle(handle) }))
            }
            WireValue::List(items) => Value::List(std::rc::Rc::new(std::cell::RefCell::new(
                items.into_iter().map(WireValue::into_guest_value).collect::<Result<Vec<_>, _>>()?,
            ))),
            WireValue::Record { name, fields } => Value::Record {
                name: name.into(),
                fields: std::rc::Rc::new(std::cell::RefCell::new(
                    fields
                        .into_iter()
                        .map(|(k, x)| Ok((k, x.into_guest_value()?)))
                        .collect::<Result<Vec<_>, Untransferable>>()?,
                )),
            },
            WireValue::Variant { name, fields } => Value::Variant {
                name: name.into(),
                fields: VariantFields::new(
                    fields.into_iter().map(WireValue::into_guest_value).collect::<Result<Vec<_>, _>>()?,
                ),
            },
            plain => plain.into_value(&mut |_| None)?,
        })
    }
}

/// A name for a value kind that has no wire form, for the refusal message.
fn leaked_kind(_v: &Value) -> &'static str {
    "a value of a kind that does not cross the sandbox channel"
}

/// The host's FIRST frame (PS-E-01): the protocol's version and this run's generation — a fresh nonce
/// for every run — and nothing to run. The guest locks itself down, then reports what it applied with
/// the generation echoed ([`ReqBody::Confined`]); a report for another generation is refused, so a
/// confirmation belongs to one run.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Open {
    pub version: String,
    /// Defaulted when absent so that a host of an older protocol, whose first frame had no generation,
    /// is refused on its VERSION, in words, rather than on a missing field.
    #[serde(default)]
    pub generation: String,
}

/// The frame that travels HOST to GUEST once the guest's boundary is confirmed, before the
/// conversation turns around: what to run, the hash it must match, and the two knobs that make a run
/// reproducible.
///
/// It carries no grant and no scope. The guest is told what to execute, never what it may reach —
/// that stays with the host, which is the whole point of the arrangement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub program: String,
    /// The program's blake3 hash, so a guest refuses anything swapped in flight.
    pub hash: String,
    pub seed: u64,
    pub fixed_clock_ms: Option<i64>,
}

/// One request from the guest to the host.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Request {
    /// Always [`CHANNEL_VERSION`]; a mismatch is refused, never guessed at.
    pub version: String,
    /// Monotonic per connection, so a reply can never be taken for another call's reply.
    pub seq: u64,
    pub body: ReqBody,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ReqBody {
    /// One capability operation: the primitive table's shape, by handle.
    CapMethod { cap: Handle, method: String, args: Vec<WireValue>, file: u32, start: u32, end: u32 },
    /// Minting from the root (`root.console()`, `root.fs_read("./x")`, …). The guest never holds a
    /// root: the host mints from the real one and answers with a handle (PS-A-03).
    RootMethod { method: String, args: Vec<WireValue>, file: u32, start: u32, end: u32 },
    /// The guest has finished; the host stops reading.
    Done { exit: i32 },
    /// What the guest applied to ITSELF, sent once, as its FIRST request: after it locked itself down
    /// and before a line of the program ran, while it is still the toolchain's own code (RW 4.23). A
    /// report at the goodbye would come from a guest the program had already been running in. Since
    /// `/3` (PS-E-01) it comes before the guest has even been SENT the program, and it carries the
    /// generation of the [`Open`] frame it answers.
    Confined { applied: Vec<String>, generation: String },
}

/// The host's answer. A fault is the program's own error (an `IoErr`, a refusal the checks made);
/// an error is the channel's (a bad frame, an unknown handle, a version mismatch).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Response {
    Ok(WireValue),
    Fault { code: String, message: String },
    Error { code: String, message: String },
}

/// Write one length-prefixed canonical CBOR frame.
///
/// In ONE write: the length is reserved at the front of the buffer and filled in after encoding.
/// Written as two (the length, then the body), a frame could reach the other side as two arrivals,
/// and a reader blocked in `read_exact` then woke twice for one message — on Windows, where the pipe
/// is drained by a thread, twice across threads. PS-B-04's measurement is what showed the channel's
/// cost was worth reading line by line (`measurements/sandbox-channel/RECORD.md`).
pub fn write_frame<T: Serialize>(w: &mut impl Write, msg: &T) -> io::Result<()> {
    let mut buf = vec![0u8; 4];
    ciborium::into_writer(msg, &mut buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    let len = buf.len() - 4;
    if len > MAX_FRAME as usize {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame exceeds the channel's bound"));
    }
    buf[..4].copy_from_slice(&(len as u32).to_le_bytes());
    w.write_all(&buf)?;
    w.flush()
}

/// Read one frame. An oversize length is refused before a single byte of it is allocated.
pub fn read_frame<T: for<'de> Deserialize<'de>>(r: &mut impl Read) -> io::Result<T> {
    let mut len = [0u8; 4];
    r.read_exact(&mut len)?;
    let len = u32::from_le_bytes(len);
    if len > MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame length exceeds the channel's bound"));
    }
    let mut buf = vec![0u8; len as usize];
    r.read_exact(&mut buf)?;
    let mut rest: &[u8] = &buf[..];
    let value = ciborium::from_reader(&mut rest).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    // RW 4.32 (the red-team pass's F12): one frame, one value. Bytes after it would give a frame many
    // spellings, and a peer that adds them is not speaking this protocol.
    if !rest.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("the frame carries {} byte(s) after its value", rest.len()),
        ));
    }
    Ok(value)
}

/// RW 4.32 (the red-team pass on `/3`): a deadline on each FRAME, not only on each read.
///
/// The host's channel had a deadline on every read, and a read returns as soon as ONE byte arrives —
/// so a guest that sent a byte every 40 s met each read's 60 s and held one frame open for as long as
/// it liked: a 16 MiB frame, one byte at a time, is months. This wraps the host's end of the channel
/// and times the frame the host is waiting for from its first byte: when a read returns later than
/// `deadline` after that byte, the frame is refused (`TimedOut`) and the channel's failure ends the run.
///
/// The host reads one request and then answers it, so its WRITE ends the frame: the clock stops there
/// and starts again at the next request's first byte. Silence BEFORE a frame is not the frame's time —
/// the per-read deadline already bounds it, as before.
///
/// What it bounds, exactly: the frame's bytes are judged when a read returns, and a read waits at most
/// the per-read deadline — so a frame is refused at most one per-read deadline after its own ran out
/// (under two minutes at 60 s and 60 s), where before it was never refused at all.
pub struct FrameDeadline<C> {
    inner: C,
    deadline: std::time::Duration,
    /// When the first byte of the frame now arriving came in; `None` between frames.
    since: Option<std::time::Instant>,
}

impl<C> FrameDeadline<C> {
    pub fn new(inner: C, deadline: std::time::Duration) -> Self {
        FrameDeadline { inner, deadline, since: None }
    }

    fn overdue(&self) -> io::Result<()> {
        match self.since {
            Some(t) if t.elapsed() > self.deadline => Err(io::Error::new(io::ErrorKind::TimedOut, FrameOverdue(self.deadline))),
            _ => Ok(()),
        }
    }
}

/// The error [`FrameDeadline`] fails with. Its kind is `TimedOut`, like a silent peer's, so every path
/// that ends a run on a deadline ends it on this one; its words are its own, because this peer was NOT
/// silent — telling the operator "the guest said nothing" would be false (see [`is_frame_overdue`]).
#[derive(Debug)]
pub struct FrameOverdue(pub std::time::Duration);

impl std::fmt::Display for FrameOverdue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the guest took longer than {:?} to send one frame, and the frame's deadline ended the run", self.0)
    }
}

impl std::error::Error for FrameOverdue {}

/// Whether `e` is a frame refused by [`FrameDeadline`], rather than a peer that said nothing at all.
pub fn is_frame_overdue(e: &io::Error) -> bool {
    e.get_ref().is_some_and(|inner| inner.is::<FrameOverdue>())
}

impl<C: Read> Read for FrameDeadline<C> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        if n > 0 && self.since.is_none() {
            self.since = Some(std::time::Instant::now());
        }
        // Judged when the read returns — the last piece included: a frame is whole within its deadline,
        // or it is refused.
        self.overdue()?;
        Ok(n)
    }
}

impl<C: Write> Write for FrameDeadline<C> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.since = None;
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// The host side of the channel: it mints the handles, resolves them, and performs each operation
/// through an [`EffectSink`] — [`crate::sink::LocalSink`] today, so a guest's effect goes through
/// exactly the checks a local run makes. The table is per run and per connection: a handle means
/// nothing anywhere else, and a number the host never minted resolves to nothing.
pub struct HostChannel<S: crate::sink::EffectSink> {
    sink: S,
    caps: Vec<std::rc::Rc<CapVal>>,
    /// The REAL root, held only here. A guest asks; the host mints. Absent means the run granted
    /// nothing, and every mint is refused rather than defaulted.
    root: Option<std::rc::Rc<crate::value::RootVal>>,
    /// PS-A-07's `denied[]`: every refusal this host gave the guest, in order.
    ///
    /// A run report that lists only what was allowed tells an operator nothing about what the program
    /// TRIED. These are the attempts that were refused — an ungranted scope, an unknown handle, a
    /// value the channel will not carry, a version mismatch — and they are the most interesting line
    /// in the report when the program is one nobody wrote.
    ///
    /// Bounded: a guest could otherwise make the host allocate without limit simply by being refused
    /// in a loop, which would be a denial of service through the evidence channel. The count keeps
    /// going after the list stops, so a truncated report still says how many there were.
    denied: Vec<String>,
    denied_total: u64,
    /// Requests answered so far — [`ReqBody::Confined`] is accepted only as the first.
    answered: u64,
    /// What the guest reported applying to itself, each word one of [`SELF_APPLIED`].
    self_applied: Vec<&'static str>,
    /// PS-E-01: the generation this host opened the run with. A confinement report is accepted only
    /// for it; a host that opened none accepts no report at all.
    generation: Option<String>,
    /// The custody every capability operation is authorized by BEFORE it is performed — the
    /// interpreter's gate (`Interp::with_custody`), at the same point, for a guest. Absent means
    /// embedded custody: the root's own scopes are the enforcement, as in an embedded local run.
    /// Present when the run is under the broker daemon (`--broker daemon`, `--lease`): then every use
    /// is decided by the broker — revocation, expiry, the Guard's `guarded` and `sealed` tiers and
    /// its permits — exactly as for a program the host interprets itself (REMAINING_WORK 4.20).
    custody: Option<Box<dyn crate::custody::Custody>>,
}

/// How many refusals a report keeps. Past this the count still rises but nothing more is stored.
pub const MAX_DENIED_RECORDED: usize = 64;

impl<S: crate::sink::EffectSink> HostChannel<S> {
    pub fn new(sink: S) -> Self {
        HostChannel {
            sink,
            caps: Vec::new(),
            root: None,
            denied: Vec::new(),
            denied_total: 0,
            answered: 0,
            self_applied: Vec::new(),
            generation: None,
            custody: None,
        }
    }

    /// PS-E-01: the generation this run was opened with ([`Open`]); the guest's confinement report must
    /// echo it.
    pub fn with_generation(mut self, generation: &str) -> Self {
        self.generation = Some(generation.to_string());
        self
    }

    /// Authorize every capability operation through `custody` before performing it (see the field).
    pub fn with_custody(mut self, custody: Box<dyn crate::custody::Custody>) -> Self {
        self.custody = Some(custody);
        self
    }

    /// The custody label a report states: `"daemon"` when a broker decides, `"embedded"` otherwise.
    pub fn custody_mode(&self) -> &'static str {
        self.custody.as_ref().map_or("embedded", |c| c.mode())
    }

    /// The boundaries the guest reported applying to itself, before its program ran (RW 4.23).
    pub fn self_applied(&self) -> &[&'static str] {
        &self.self_applied
    }

    /// Every refusal this host gave, oldest first, and how many there were in total — which is larger
    /// than the list when a guest was refused more than [`MAX_DENIED_RECORDED`] times.
    pub fn denied(&self) -> (&[String], u64) {
        (&self.denied, self.denied_total)
    }

    /// Record one refusal. Called on the way out, so no refusal path can forget: a rule that has to
    /// be remembered at every `return` is a rule that dies in one branch (the skip-branch lesson).
    fn note_denied(&mut self, what: &str) {
        self.denied_total += 1;
        if self.denied.len() < MAX_DENIED_RECORDED {
            // Bounded and escaped: the text quotes what the guest sent (`shown`).
            self.denied.push(shown(what, MAX_DENIED_CHARS));
        }
    }

    /// Record a refusal the host made WITHOUT answering — a guest's first request that was not its
    /// confinement report (PS-E-01): not answered, because an answer could be an effect, and still on
    /// the record (the red-team pass on `/3`, F8: it had left `denied` empty).
    pub fn record_unanswered(&mut self, what: &str) {
        self.note_denied(what);
    }

    /// Give the host the root this run was granted, so the guest can mint from it by asking.
    pub fn with_root(mut self, root: std::rc::Rc<crate::value::RootVal>) -> Self {
        self.root = Some(root);
        self
    }

    /// Mint a handle for a capability the host holds. Handles start at 1, so 0 is never valid.
    pub fn mint(&mut self, cap: std::rc::Rc<CapVal>) -> Handle {
        self.caps.push(cap);
        self.caps.len() as Handle
    }

    pub fn resolve(&self, h: Handle) -> Option<std::rc::Rc<CapVal>> {
        self.caps.get((h.checked_sub(1)?) as usize).cloned()
    }

    /// Answer one request. Every refusal is an answer: the guest is never left waiting, and the
    /// host never guesses at a frame it does not understand.
    pub fn answer(&mut self, req: &Request) -> Response {
        let egress_mark = crate::egress::mark();
        let resp = self.decide(req);
        self.answered += 1;
        // ONE place, on the way out. Recording at each refusal site would mean a new refusal added
        // later is silently absent from the report — which is exactly the shape of defect this
        // project keeps finding in `else { continue }` branches.
        match &resp {
            Response::Ok(_) => {}
            Response::Fault { code, message } | Response::Error { code, message } => {
                let what = match &req.body {
                    ReqBody::CapMethod { method, .. } => format!("{code} on a capability method `{method}`: {message}"),
                    ReqBody::RootMethod { method, .. } => format!("{code} on `root.{method}`: {message}"),
                    ReqBody::Done { .. } => format!("{code} on goodbye: {message}"),
                    ReqBody::Confined { .. } => format!("{code} on the guest's own confinement report: {message}"),
                };
                self.note_denied(&what);
            }
        }
        // PS-B-02: an egress refusal reaches the guest as a VALUE (`Err(Refused)`), so the match above
        // never sees it — and a report of what the host refused that omits the network would be the
        // same hole in a new place. The egress client logs every request host-side; the ones this
        // request refused are recorded here, with the machine-readable reason the guest is not told.
        let (records, refused) = crate::egress::records_since(egress_mark);
        let mut noted = 0u64;
        for r in records.iter() {
            if let Err(reason) = &r.outcome {
                self.note_denied(&format!("egress `{}` refused: {} ({})", r.url, reason.code(), reason.explain()));
                noted += 1;
            }
        }
        // Refusals past the log's bound are counted even though their records were not kept.
        for _ in noted..refused {
            self.note_denied("egress request refused (its record was past the log's bound)");
        }
        resp
    }

    fn decide(&mut self, req: &Request) -> Response {
        if req.version != CHANNEL_VERSION {
            return Response::Error {
                code: "DL1401".into(),
                message: format!("channel version `{}` is not `{CHANNEL_VERSION}`", req.version),
            };
        }
        match &req.body {
            ReqBody::Done { .. } => Response::Ok(WireValue::Unit),
            ReqBody::Confined { applied, generation } => {
                // Once, and first: after anything else, the guest has been running the program, and
                // what it says about itself is no longer the toolchain speaking.
                if self.answered > 0 {
                    return Response::Error {
                        code: "DL1401".into(),
                        message: "a confinement report is accepted only as a guest's first request".into(),
                    };
                }
                // PS-E-01: for THIS run. A report carrying another generation — replayed, relayed from
                // another run, or made up — confirms nothing here.
                if self.generation.as_deref() != Some(generation.as_str()) {
                    return Response::Error {
                        code: "DL1401".into(),
                        message: "the confinement report names another run's generation".into(),
                    };
                }
                let mut known: Vec<&'static str> = Vec::with_capacity(applied.len());
                for word in applied {
                    match SELF_APPLIED.iter().find(|k| **k == word.as_str()) {
                        Some(k) if !known.contains(k) => known.push(k),
                        Some(_) => {}
                        None => {
                            return Response::Error {
                                code: "DL1401".into(),
                                message: format!("`{}` is not a boundary a guest applies to itself", shown(word, 64)),
                            }
                        }
                    }
                }
                self.self_applied = known;
                Response::Ok(WireValue::Unit)
            }
            ReqBody::RootMethod { method, args, file, start, end } => {
                let Some(root) = self.root.clone() else {
                    return Response::Error {
                        code: "DL1401".into(),
                        message: "this run has no root to mint from".into(),
                    };
                };
                let mut resolve = |h: Handle| self.caps.get(h.wrapping_sub(1) as usize).cloned();
                let mut decoded = Vec::with_capacity(args.len());
                for a in args {
                    match a.clone().into_value(&mut resolve) {
                        Ok(v) => decoded.push(v),
                        Err(Untransferable(why)) => {
                            return Response::Error { code: "DL1401".into(), message: format!("an argument is {why}") }
                        }
                    }
                }
                let span = delulu_diag::Span::new(*file, *start, *end);
                match self.sink.root_method(&root, method, &decoded, span) {
                    Ok(v) => self.encode_result(v),
                    Err(f) => Response::Fault { code: f.code.to_string(), message: f.message.clone() },
                }
            }
            ReqBody::CapMethod { cap, method, args, file, start, end } => {
                let Some(capv) = self.resolve(*cap) else {
                    return Response::Error {
                        code: "DL1401".into(),
                        message: format!("no capability handle {cap} in this run"),
                    };
                };
                let mut resolve = |h: Handle| self.caps.get((h.wrapping_sub(1)) as usize).cloned();
                let mut decoded = Vec::with_capacity(args.len());
                for a in args {
                    match a.clone().into_value(&mut resolve) {
                        Ok(v) => decoded.push(v),
                        Err(Untransferable(why)) => {
                            return Response::Error { code: "DL1401".into(), message: format!("an argument is {why}") }
                        }
                    }
                }
                let span = delulu_diag::Span::new(*file, *start, *end);
                // The custody gate, where the interpreter has it: after the arguments are known,
                // before the effect happens, with the same op and argument (`custody_op_for` is the
                // interpreter's own mapping, so the two cannot name an operation differently). A
                // denial is the broker's code and words — DL1403 revoked, DL1410 guarded, DL1413
                // sealed — returned to the guest as the fault the program would have raised locally.
                // GUARD-ALIAS-1: the file effect's path is pinned once; the custody decides on it
                // and the sink opens exactly it.
                let recv = Value::Cap(capv.clone());
                let fs_pin = crate::interp::fs_pin_for(&recv, method, &decoded);
                if let Some(custody) = self.custody.as_mut() {
                    if let Some((op, arg)) = crate::interp::custody_op_for(&recv, method, &decoded, fs_pin.as_deref()) {
                        if let crate::custody::CustodyDecision::Deny(d) = custody.check(op, arg.as_deref()) {
                            return Response::Fault { code: d.code.to_string(), message: d.message };
                        }
                    }
                }
                match self.sink.cap_method_pinned(&capv, method, &decoded, span, fs_pin.as_deref()) {
                    Ok(v) => self.encode_result(v),
                    Err(f) => Response::Fault { code: f.code.to_string(), message: f.message.clone() },
                }
            }
        }
    }

    /// Encode a result for the guest. A capability in it — a freshly minted one, or a narrowed one —
    /// becomes a handle; nothing that names a resource is ever sent.
    fn encode_result(&mut self, v: Value) -> Response {
        let mut minted: Vec<std::rc::Rc<CapVal>> = Vec::new();
        let wire = WireValue::from_value(&v, &mut |c: &std::rc::Rc<CapVal>| {
            minted.push(c.clone());
            0
        });
        match wire {
            Ok(w) => Response::Ok(self.remint(w, &minted)),
            Err(Untransferable(why)) => Response::Error {
                code: "DL1401".into(),
                message: format!("the result is {why}, which does not cross the channel"),
            },
        }
    }

    /// Replace the placeholder handles in an encoded result with real minted ones, in the order the
    /// encoder met them.
    fn remint(&mut self, w: WireValue, minted: &[std::rc::Rc<CapVal>]) -> WireValue {
        let mut next = 0usize;
        self.walk(w, minted, &mut next)
    }

    fn walk(&mut self, w: WireValue, minted: &[std::rc::Rc<CapVal>], next: &mut usize) -> WireValue {
        match w {
            WireValue::Cap { kind, .. } => {
                let cap = minted[*next].clone();
                *next += 1;
                WireValue::Cap { handle: self.mint(cap), kind }
            }
            WireValue::List(items) => {
                WireValue::List(items.into_iter().map(|x| self.walk(x, minted, next)).collect())
            }
            WireValue::Record { name, fields } => WireValue::Record {
                name,
                fields: fields.into_iter().map(|(k, x)| (k, self.walk(x, minted, next))).collect(),
            },
            WireValue::Variant { name, fields } => WireValue::Variant {
                name,
                fields: fields.into_iter().map(|x| self.walk(x, minted, next)).collect(),
            },
            plain => plain,
        }
    }

    /// Serve one guest until it says it is done, or the connection fails. Returns the guest's exit.
    ///
    /// One duplex, not a read half and a write half: the transports that carry this channel (a Unix
    /// socket, a Windows named pipe) are single objects, and splitting them would mean cloning a
    /// handle for no reason.
    pub fn serve(&mut self, io: &mut (impl Read + Write)) -> io::Result<i32> {
        loop {
            let req: Request = read_frame(io)?;
            let done = matches!(req.body, ReqBody::Done { .. });
            let exit = if let ReqBody::Done { exit } = req.body { exit } else { 0 };
            let resp = self.answer(&req);
            write_frame(io, &resp)?;
            if done {
                return Ok(exit);
            }
        }
    }
}

/// The guest side: every capability operation becomes one request and one reply. The guest holds no
/// scope of its own — each of its capabilities is a [`CapScope::Handle`] — so this sink cannot widen
/// anything: the host decides what the handle means.
pub struct ChannelSink<T: Read + Write> {
    io: std::cell::RefCell<T>,
    seq: std::cell::Cell<u64>,
}

impl<T: Read + Write> ChannelSink<T> {
    pub fn new(io: T) -> Self {
        ChannelSink { io: std::cell::RefCell::new(io), seq: std::cell::Cell::new(0) }
    }

    /// Tell the host the guest is finished, so it stops serving.
    pub fn done(&self, exit: i32) -> io::Result<()> {
        let req = Request { version: CHANNEL_VERSION.into(), seq: self.next_seq(), body: ReqBody::Done { exit } };
        let mut io = self.io.borrow_mut();
        write_frame(&mut *io, &req)?;
        let _: Response = read_frame(&mut *io)?;
        Ok(())
    }

    /// Tell the host what this guest applied to itself, for the generation it was opened with. Called
    /// once, before the guest is sent its program (PS-E-01); a host that refuses the report is an error,
    /// and the guest then never receives a program at all.
    pub fn confined(&self, applied: &[&str], generation: &str) -> io::Result<()> {
        let req = Request {
            version: CHANNEL_VERSION.into(),
            seq: self.next_seq(),
            body: ReqBody::Confined {
                applied: applied.iter().map(|s| s.to_string()).collect(),
                generation: generation.to_string(),
            },
        };
        let mut io = self.io.borrow_mut();
        write_frame(&mut *io, &req)?;
        match read_frame::<Response>(&mut *io)? {
            Response::Ok(_) => Ok(()),
            Response::Fault { message, .. } | Response::Error { message, .. } => Err(io::Error::other(message)),
        }
    }

    /// One frame FROM the host that is not an answer: the [`Program`], once the host has confirmed this
    /// guest's boundary (PS-E-01).
    pub fn receive<M: serde::de::DeserializeOwned>(&self) -> io::Result<M> {
        read_frame(&mut *self.io.borrow_mut())
    }

    fn next_seq(&self) -> u64 {
        let n = self.seq.get() + 1;
        self.seq.set(n);
        n
    }

    /// One request, one reply: the only shape this side ever uses. A transport failure is a fault,
    /// never a value, so a guest can never mistake a broken channel for a performed effect.
    fn exchange(&self, req: Request, span: delulu_diag::Span) -> Result<Value, crate::value::Fault> {
        let fault = |m: String| crate::value::Fault::at("DL1401", m, span);
        let mut io = self.io.borrow_mut();
        write_frame(&mut *io, &req).map_err(|e| fault(format!("the sandbox channel failed: {e}")))?;
        let resp: Response = read_frame(&mut *io).map_err(|e| fault(format!("the sandbox channel failed: {e}")))?;
        match resp {
            Response::Ok(w) => {
                w.into_guest_value().map_err(|Untransferable(why)| fault(format!("the reply carried {why}")))
            }
            // The host's own answer, carried through unchanged: an unregistered code would be a lie
            // about which diagnostic this is, so it becomes the channel's own DL1401 instead.
            Response::Fault { code, message } | Response::Error { code, message } => Err(crate::value::Fault::at(
                delulu_diag::static_code(&code).unwrap_or("DL1401"),
                message,
                span,
            )),
        }
    }
}

impl<T: Read + Write> crate::sink::EffectSink for ChannelSink<T> {
    fn cap_method(
        &self,
        cap: &CapVal,
        method: &str,
        args: &[Value],
        span: delulu_diag::Span,
    ) -> Result<Value, crate::value::Fault> {
        let fault = |m: String| crate::value::Fault::at("DL1401", m, span);
        let crate::value::CapScope::Handle(h) = cap.scope else {
            return Err(fault("a guest capability must be a host handle".into()));
        };
        let mut encode = |c: &std::rc::Rc<CapVal>| match c.scope {
            crate::value::CapScope::Handle(h) => h,
            // A guest that somehow holds a scoped capability must not send it: 0 is never a handle
            // the host minted, so the host refuses it rather than acting on a scope it did not give.
            _ => 0,
        };
        let mut wire_args = Vec::with_capacity(args.len());
        for a in args {
            match WireValue::from_value(a, &mut encode) {
                Ok(w) => wire_args.push(w),
                Err(Untransferable(why)) => return Err(fault(format!("an argument is {why}"))),
            }
        }
        let req = Request {
            version: CHANNEL_VERSION.into(),
            seq: self.next_seq(),
            body: ReqBody::CapMethod {
                cap: h,
                method: method.to_string(),
                args: wire_args,
                file: span.file,
                start: span.start,
                end: span.end,
            },
        };
        self.exchange(req, span)
    }

    fn root_method(
        &self,
        _root: &crate::value::RootVal,
        method: &str,
        args: &[Value],
        span: delulu_diag::Span,
    ) -> Result<Value, crate::value::Fault> {
        // The guest's own root value carries nothing: the HOST holds the real one and mints from it.
        // So the request names the method and the arguments, never the root's contents.
        let fault = |m: String| crate::value::Fault::at("DL1401", m, span);
        let mut encode = |c: &std::rc::Rc<CapVal>| match c.scope {
            crate::value::CapScope::Handle(h) => h,
            _ => 0,
        };
        let mut wire_args = Vec::with_capacity(args.len());
        for a in args {
            match WireValue::from_value(a, &mut encode) {
                Ok(w) => wire_args.push(w),
                Err(Untransferable(why)) => return Err(fault(format!("an argument is {why}"))),
            }
        }
        let req = Request {
            version: CHANNEL_VERSION.into(),
            seq: self.next_seq(),
            body: ReqBody::RootMethod {
                method: method.to_string(),
                args: wire_args,
                file: span.file,
                start: span.start,
                end: span.end,
            },
        };
        self.exchange(req, span)
    }

    fn backend(&self) -> &'static str {
        "channel"
    }
}

/// The whole arrangement in one process: whatever the guest writes is answered by a real
/// [`HostChannel`], over the real frames, with no socket and no child.
///
/// Public because it is not only a test fixture. It is how the guest path can be exercised at fuzz
/// volume — every accepted program run twice, once locally and once as a guest — which is the only
/// way to state the property that matters: routing an effect through the channel does not change
/// WHICH effects happen. A second process per program would make that campaign unaffordable, and an
/// unaffordable gate is one that does not run.
pub struct Loopback<S: crate::sink::EffectSink> {
    host: HostChannel<S>,
    inbox: Vec<u8>,
    replies: std::collections::VecDeque<u8>,
}

impl<S: crate::sink::EffectSink> Loopback<S> {
    pub fn new(host: HostChannel<S>) -> Self {
        Loopback { host, inbox: Vec::new(), replies: std::collections::VecDeque::new() }
    }
}

impl<S: crate::sink::EffectSink> Write for Loopback<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inbox.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        // A whole frame has arrived: answer it exactly as the host would on a socket.
        let taken = std::mem::take(&mut self.inbox);
        if taken.is_empty() {
            return Ok(());
        }
        let req: Request = read_frame(&mut &taken[..])?;
        let resp = self.host.answer(&req);
        let mut out = Vec::new();
        write_frame(&mut out, &resp)?;
        self.replies.extend(out);
        Ok(())
    }
}

impl<S: crate::sink::EffectSink> Read for Loopback<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = buf.len().min(self.replies.len());
        for slot in buf.iter_mut().take(n) {
            *slot = self.replies.pop_front().expect("checked length");
        }
        Ok(n)
    }
}

/// PS-A-02's fuzz property, in ONE place: the `cargo-fuzz` target and the in-suite corpus replay
/// both call this function, so there is no second copy to go stale. (A test holding its own copy of
/// a rule has already gone stale twice in this phase — once for the loader's environment allowlist,
/// once for a per-platform jail report.)
///
/// These bytes are the most hostile input in the whole system. They arrive from a guest the host has
/// deliberately assumed is compromised, and the host reads them before it knows anything about them.
/// Three claims are checked on every input:
///
/// 1. **No panic and no unbounded allocation.** Every byte string is a refusal or a value. A panic
///    in the host's reader would be a guest crashing its host, which is a denial of service the
///    guest is not supposed to be able to cause.
/// 2. **Re-encoding is byte-stable.** Whatever decodes must encode back to bytes that decode to the
///    same thing and encode again identically. A frame the host acts on must mean exactly one thing:
///    two spellings of one request is the shape that produced GUARD-SPELL-1 and the `net.special`
///    finding, one layer down. (Byte equality rather than value equality, deliberately: `WireValue`
///    carries an `f64`, and a NaN is not equal to itself, so value equality would be a weaker claim
///    dressed as a stronger one.)
/// 3. **An empty host grants nothing.** The decoded request is answered by a [`HostChannel`] that
///    holds no root and has minted no handle, and the answer must never be `Ok` — except for `Done`,
///    which performs nothing. This is the arrangement's central claim, checked against arbitrary
///    frames rather than the ones the tests thought to write: whatever a guest sends, it cannot make
///    a host that holds nothing perform something.
pub fn fuzz_one_frame(data: &[u8]) {
    // The framing rule first, on the raw bytes, exactly as the wire delivers them. A length beyond
    // the bound must be refused before the body is touched.
    let _ = read_frame::<Request>(&mut &data[..]);

    // Then the BODY decoder, given a correct prefix, because that is the decoder a guest actually
    // reaches once it has framed its frame. Fuzzing the prefix alone would spend every iteration in
    // the length check and never reach the CBOR.
    if data.len() > u32::MAX as usize {
        return;
    }
    let mut framed = Vec::with_capacity(4 + data.len());
    framed.extend_from_slice(&(data.len() as u32).to_le_bytes());
    framed.extend_from_slice(data);

    // Every type that crosses this channel, in both directions.
    if let Ok(v) = read_frame::<WireValue>(&mut &framed[..]) {
        stable(&v);
    }
    if let Ok(o) = read_frame::<Open>(&mut &framed[..]) {
        stable(&o);
    }
    if let Ok(p) = read_frame::<Program>(&mut &framed[..]) {
        stable(&p);
    }
    if let Ok(r) = read_frame::<Response>(&mut &framed[..]) {
        stable(&r);
    }
    if let Ok(req) = read_frame::<Request>(&mut &framed[..]) {
        stable(&req);
        // `Done` and a guest's report of its own confinement perform nothing, so they may be `Ok`.
        let performs_nothing = matches!(req.body, ReqBody::Done { .. } | ReqBody::Confined { .. });
        // Opened with a generation a well-formed corpus frame can carry, so an accepted confinement
        // report is reached, not only refused ones.
        let mut host = HostChannel::new(crate::sink::LocalSink).with_generation(&"ab".repeat(32));
        match host.answer(&req) {
            Response::Ok(_) => assert!(
                performs_nothing,
                "a host holding no root and no handle answered `Ok` to {req:?} — a guest that sends \
                 the right bytes must still not be able to make an empty host do anything"
            ),
            Response::Fault { .. } | Response::Error { .. } => {}
        }
    }
}

/// Encoding is idempotent: encode, decode, encode again, same bytes.
fn stable<T: Serialize + for<'de> Deserialize<'de>>(v: &T) {
    let mut once = Vec::new();
    // A value that decoded from a frame is within the bound by construction, so a write failure here
    // is a real inconsistency and not a case to skip.
    write_frame(&mut once, v).expect("a value that decoded must re-encode");
    let again: T = read_frame(&mut &once[..]).expect("a re-encoded value must decode");
    let mut twice = Vec::new();
    write_frame(&mut twice, &again).expect("and re-encode again");
    assert_eq!(once, twice, "this frame has two spellings, so it does not mean exactly one thing");
}

#[cfg(test)]
mod tests {
    use super::*;

    const GEN: &str = "0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f";

    fn confined(seq: u64, words: &[&str]) -> Request {
        confined_for(seq, words, GEN)
    }

    fn confined_for(seq: u64, words: &[&str], generation: &str) -> Request {
        Request {
            version: CHANNEL_VERSION.into(),
            seq,
            body: ReqBody::Confined { applied: words.iter().map(|w| w.to_string()).collect(), generation: generation.into() },
        }
    }

    /// RW 4.23: a guest's report of its own confinement is taken once, first, and only in known words
    /// — so the report can gain what the guest really applied and nothing a guest makes up.
    #[test]
    fn a_confinement_report_is_accepted_first_once_and_only_in_known_words() {
        let mut host = HostChannel::new(crate::sink::LocalSink).with_generation(GEN);
        let r = host.answer(&confined(1, &["no file writes", "no new programs", "no file writes"]));
        assert!(matches!(r, Response::Ok(_)), "{r:?}");
        assert_eq!(host.self_applied(), ["no file writes", "no new programs"], "known words, each once");

        // Not a second time: by then the program has been running in the guest.
        let r = host.answer(&confined(2, &["no debugger"]));
        assert!(matches!(r, Response::Error { .. }), "{r:?}");
        assert_eq!(host.self_applied(), ["no file writes", "no new programs"], "unchanged by the refused one");
        assert_eq!(host.denied().1, 1, "and the refusal is on the record");

        // A word nobody applies is refused, whole: nothing of that report is kept.
        let mut host = HostChannel::new(crate::sink::LocalSink).with_generation(GEN);
        let r = host.answer(&confined(1, &["no new programs", "a separate identity: trust me"]));
        assert!(matches!(r, Response::Error { .. }), "{r:?}");
        assert!(host.self_applied().is_empty());

        // And not after an ordinary request either.
        let mut host = HostChannel::new(crate::sink::LocalSink).with_generation(GEN);
        let _ = host.answer(&Request { version: CHANNEL_VERSION.into(), seq: 1, body: ReqBody::Done { exit: 0 } });
        assert!(matches!(host.answer(&confined(2, &["no new programs"])), Response::Error { .. }));
        assert!(host.self_applied().is_empty());
    }

    /// PS-E-01: a confinement report belongs to ONE run — the generation the host opened it with. A
    /// report for another generation, or to a host that opened none, confirms nothing and is recorded.
    /// The red-team pass on `/3`: a guest's text is escaped and bounded before it can reach the operator.
    #[test]
    fn a_guests_text_is_shown_escaped_and_bounded() {
        assert_eq!(shown("no new programs", 64), "no new programs");
        let s = shown("a\nseq 4 FORGED\x1b[31m\r\u{2028}é", 64);
        assert!(!s.contains('\n') && !s.contains('\x1b') && !s.contains('\r') && !s.contains('\u{2028}'), "{s:?}");
        assert!(s.contains("\\n") && s.contains('é'), "{s:?}");
        let long = "y".repeat(1_000_000);
        let s = shown(&long, 64);
        assert!(s.starts_with(&"y".repeat(64)) && s.ends_with("(+999936 bytes)"), "{}", &s[..80]);
        // Through the host: a refused word and the refusal it records are both bounded.
        let mut host = HostChannel::new(crate::sink::LocalSink).with_generation(GEN);
        let word = format!("x\n{}", "z".repeat(100_000));
        let r = host.answer(&confined(1, &[word.as_str()]));
        let Response::Error { message, .. } = &r else { panic!("{r:?}") };
        assert!(message.len() < 300 && !message.contains('\n'), "{message:?}");
        let (denied, _) = host.denied();
        assert!(denied[0].len() <= MAX_DENIED_CHARS * 4 + 32 && !denied[0].contains('\n'), "{}", denied[0].len());
    }

    #[test]
    fn a_confinement_report_is_accepted_only_for_the_generation_the_host_opened() {
        let other = "1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e";
        let mut host = HostChannel::new(crate::sink::LocalSink).with_generation(GEN);
        let r = host.answer(&confined_for(1, &["no new programs"], other));
        assert!(matches!(&r, Response::Error { message, .. } if message.contains("another run's generation")), "{r:?}");
        assert!(host.self_applied().is_empty(), "nothing of a report for another run is kept");
        assert_eq!(host.denied().1, 1, "and the refusal is on the record");

        let mut host = HostChannel::new(crate::sink::LocalSink);
        let r = host.answer(&confined(1, &["no new programs"]));
        assert!(matches!(r, Response::Error { .. }), "a host that opened no generation takes no report: {r:?}");
        assert!(host.self_applied().is_empty());

        let mut host = HostChannel::new(crate::sink::LocalSink).with_generation(GEN);
        assert!(matches!(host.answer(&confined(1, &["no new programs"])), Response::Ok(_)), "its own generation is taken");
        assert_eq!(host.self_applied(), ["no new programs"]);
    }

    fn no_caps() -> impl FnMut(&std::rc::Rc<CapVal>) -> Handle {
        |_: &std::rc::Rc<CapVal>| 0
    }

    #[test]
    fn plain_data_crosses_and_comes_back_the_same() {
        let v = Value::List(std::rc::Rc::new(std::cell::RefCell::new(vec![
            Value::Int(7),
            Value::Str("hi".into()),
            Value::Bool(true),
            Value::Unit,
        ])));
        let wire = WireValue::from_value(&v, &mut no_caps()).expect("plain data crosses");
        let mut buf = Vec::new();
        write_frame(&mut buf, &wire).unwrap();
        let back: WireValue = read_frame(&mut &buf[..]).unwrap();
        assert_eq!(wire, back);
        let value = back.into_value(&mut |_| None).expect("no handles in this value");
        assert_eq!(value.display(), v.display());
    }

    /// The point of the channel: a guest may not send anything that would carry authority or a
    /// host-side identity across it.
    #[test]
    fn a_value_that_cannot_cross_honestly_is_refused() {
        let root = Value::Root(std::rc::Rc::new(crate::value::RootVal::default()));
        assert_eq!(WireValue::from_value(&root, &mut no_caps()), Err(Untransferable("the root")));
    }

    /// An unknown handle is a refusal, never a capability conjured from a number.
    #[test]
    fn an_unknown_capability_handle_is_refused() {
        let wire = WireValue::Cap { handle: 42, kind: "Clock".into() };
        // `Value` has no equality (a capability must not be comparable), so check the refusal itself.
        let err = wire.into_value(&mut |_| None).expect_err("an unknown handle must be refused");
        assert_eq!(err, Untransferable("an unknown capability handle"));
    }

    /// A hostile length prefix must not make the reader allocate: the bound is checked first.
    #[test]
    fn an_oversize_frame_is_refused_before_it_is_allocated() {
        let mut bytes = (MAX_FRAME + 1).to_le_bytes().to_vec();
        bytes.extend_from_slice(&[0u8; 8]);
        let err = read_frame::<WireValue>(&mut &bytes[..]).expect_err("the bound must hold");
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    /// RW 4.32 (the red-team pass's F12): a frame means exactly one value. Bytes after it inside the
    /// frame's length were ignored, so one frame had many spellings — now it is refused.
    #[test]
    fn a_frame_with_bytes_after_its_value_is_refused() {
        let open = Open { version: CHANNEL_VERSION.into(), generation: GEN.into() };
        let mut frame = Vec::new();
        write_frame(&mut frame, &open).unwrap();
        assert_eq!(read_frame::<Open>(&mut &frame[..]).unwrap(), open, "the exact frame decodes");
        let len = u32::from_le_bytes(frame[..4].try_into().unwrap()) + 16;
        frame[..4].copy_from_slice(&len.to_le_bytes());
        frame.extend([0xAA_u8; 16]);
        let e = read_frame::<Open>(&mut &frame[..]).unwrap_err();
        assert_eq!(e.kind(), io::ErrorKind::InvalidData, "{e}");
        assert!(e.to_string().contains("after its value"), "{e}");
    }

    /// A peer that sends what it was scripted to, in pieces: each piece after its own pause.
    struct Scripted {
        pieces: std::collections::VecDeque<(std::time::Duration, Vec<u8>)>,
        written: Vec<u8>,
    }

    impl Scripted {
        /// `frames`, each cut into `cuts` pieces with `pause` before every piece.
        fn dripping(frames: &[Vec<u8>], cuts: usize, pause: std::time::Duration) -> Scripted {
            let mut pieces = std::collections::VecDeque::new();
            for f in frames {
                for chunk in f.chunks(f.len().div_ceil(cuts)) {
                    pieces.push_back((pause, chunk.to_vec()));
                }
            }
            Scripted { pieces, written: Vec::new() }
        }
    }

    impl Read for Scripted {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let Some((pause, mut piece)) = self.pieces.pop_front() else { return Ok(0) };
            std::thread::sleep(pause);
            let n = piece.len().min(buf.len());
            buf[..n].copy_from_slice(&piece[..n]);
            if n < piece.len() {
                self.pieces.push_front((std::time::Duration::ZERO, piece.split_off(n)));
            }
            Ok(n)
        }
    }

    impl Write for Scripted {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.written.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn framed(req: &Request) -> Vec<u8> {
        let mut f = Vec::new();
        write_frame(&mut f, req).unwrap();
        f
    }

    /// RW 4.32: a read returns at its first byte, so a peer that dripped a frame met every read's
    /// deadline and held the frame open for as long as it liked. The frame now has a deadline of its
    /// own, from its first byte — and the refusal says the peer was slow, not silent.
    #[test]
    #[cfg_attr(miri, ignore)] // its witness is the wall clock (D-V2-45)
    fn a_frame_dripped_past_its_deadline_is_refused() {
        let frame = framed(&confined(1, &["no file writes", "no new programs", "no debugger"]));
        assert!(frame.len() >= 40, "a frame long enough to drip: {} bytes", frame.len());
        // One byte every 20 ms: the whole frame would take about a second and more; its deadline is 100 ms.
        let pause = std::time::Duration::from_millis(20);
        let deadline = std::time::Duration::from_millis(100);
        let mut peer = FrameDeadline::new(Scripted::dripping(std::slice::from_ref(&frame), frame.len(), pause), deadline);
        let t = std::time::Instant::now();
        let e = read_frame::<Request>(&mut peer).expect_err("a frame dripped past its deadline must be refused");
        let took = t.elapsed();
        assert_eq!(e.kind(), io::ErrorKind::TimedOut, "{e}");
        assert!(is_frame_overdue(&e), "the frame's own refusal: {e}");
        assert!(e.to_string().contains("to send one frame"), "{e}");
        assert!(took < pause * (frame.len() as u32) / 2, "refused near the deadline, not at the frame's end: {took:?}");
        assert!(!peer.inner.pieces.is_empty(), "the frame was never read whole");
    }

    /// The frame's clock starts at its first byte: a peer that is SILENT before a frame and then sends
    /// it whole is the per-read deadline's business, as it always was — not a slow frame.
    #[test]
    #[cfg_attr(miri, ignore)] // its witness is the wall clock (D-V2-45)
    fn silence_before_a_frame_is_not_the_frames_time() {
        let req = confined(1, &["no file writes"]);
        let deadline = std::time::Duration::from_millis(100);
        let mut peer = FrameDeadline::new(Scripted::dripping(&[framed(&req)], 1, deadline * 3), deadline);
        assert_eq!(read_frame::<Request>(&mut peer).expect("a whole frame after a silence"), req);
    }

    /// The host reads one request and then answers it: its answer ends the frame it waited for, so the
    /// next request's clock starts at that request's first byte. Two frames that each arrive within
    /// the deadline, but not together, are both taken when an answer lies between them — and the second
    /// is refused when none does, which is what shows the answer is what restarts the clock.
    #[test]
    #[cfg_attr(miri, ignore)] // its witness is the wall clock (D-V2-45)
    fn the_hosts_answer_starts_the_next_frames_clock() {
        let (a, b) = (confined(1, &["no file writes"]), confined(2, &["no new programs"]));
        // Each frame in ten pieces 60 ms apart — about 0.6 s — against a 1 s deadline: within it
        // alone, past it together.
        let pause = std::time::Duration::from_millis(60);
        let deadline = std::time::Duration::from_secs(1);
        let script = || Scripted::dripping(&[framed(&a), framed(&b)], 10, pause);

        let mut peer = FrameDeadline::new(script(), deadline);
        assert_eq!(read_frame::<Request>(&mut peer).expect("the first frame"), a);
        write_frame(&mut peer, &Response::Ok(WireValue::Unit)).unwrap();
        assert_eq!(read_frame::<Request>(&mut peer).expect("the second frame, after the answer"), b);
        assert!(!peer.inner.written.is_empty(), "the answer went through to the peer");

        let mut peer = FrameDeadline::new(script(), deadline);
        assert_eq!(read_frame::<Request>(&mut peer).expect("the first frame"), a);
        let e = read_frame::<Request>(&mut peer).expect_err("with no answer between them, one clock runs over both");
        assert!(is_frame_overdue(&e), "{e}");
    }

    /// A truncated frame is a transport failure, never a half-read value.
    #[test]
    fn a_truncated_frame_is_a_failure_not_a_value() {
        let mut buf = Vec::new();
        write_frame(&mut buf, &WireValue::Int(1)).unwrap();
        buf.pop();
        assert!(read_frame::<WireValue>(&mut &buf[..]).is_err());
    }

    /// The version tag rides in every request, so a peer speaking another protocol is caught.
    #[test]
    fn a_request_carries_the_version_and_the_sequence() {
        let req = Request {
            version: CHANNEL_VERSION.to_string(),
            seq: 1,
            body: ReqBody::CapMethod {
                cap: 3,
                method: "println".into(),
                args: vec![WireValue::Str("x".into())],
                file: 0,
                start: 0,
                end: 1,
            },
        };
        let mut buf = Vec::new();
        write_frame(&mut buf, &req).unwrap();
        let back: Request = read_frame(&mut &buf[..]).unwrap();
        assert_eq!(back, req);
        assert_eq!(back.version, CHANNEL_VERSION);
    }

    /// The whole round trip: a guest holding nothing but a handle asks for the clock, and the host
    /// performs it with the primitive table it always uses.
    #[test]
    fn a_guest_effect_is_performed_by_the_host_through_the_handle() {
        use crate::sink::EffectSink;
        crate::prim::set_fixed_clock_ms(Some(1_234));
        let mut host = HostChannel::new(crate::sink::LocalSink);
        let handle = host.mint(std::rc::Rc::new(CapVal {
            kind: delulu_check::ResourceKind::Clock,
            scope: crate::value::CapScope::Clock,
        }));
        let guest_cap = CapVal { kind: delulu_check::ResourceKind::Clock, scope: crate::value::CapScope::Handle(handle) };
        let sink = ChannelSink::new(Loopback::new(host));

        let got = sink
            .cap_method(&guest_cap, "now_ms", &[], delulu_diag::Span::new(0, 0, 1))
            .expect("the host performs it");
        assert_eq!(got.display(), Value::Int(1_234).display());
        assert_eq!(sink.backend(), "channel");
        crate::prim::set_fixed_clock_ms(None);
    }

    /// PS-B-02: a guest's network request is performed by the HOST, and when the host refuses it the
    /// guest is told only `Err(Refused)` — a value, which the fault-recording branch above never sees.
    /// The refusal must still reach `denied[]`, with the reason the guest was not told. Offline: the
    /// scheme refusal is decided before any resolution.
    #[test]
    fn a_guests_refused_network_request_is_in_the_hosts_denied_list_with_its_reason() {
        let mut host = HostChannel::new(crate::sink::LocalSink);
        let h = host.mint(std::rc::Rc::new(CapVal {
            kind: delulu_check::ResourceKind::Http,
            scope: crate::value::CapScope::Net { allow: vec!["example.com".into()], special: Vec::new() },
        }));
        let url = format!("http://example.com/denied-{}", std::process::id());
        let resp = host.answer(&Request {
            version: CHANNEL_VERSION.to_string(),
            seq: 1,
            body: ReqBody::CapMethod { cap: h, method: "get".into(), args: vec![WireValue::Str(url.clone())], file: 0, start: 0, end: 1 },
        });
        // The guest learns the opaque refusal and nothing else.
        let Response::Ok(WireValue::Variant { name, fields }) = &resp else { panic!("a value, not a fault: {resp:?}") };
        assert_eq!(name, "Err");
        assert!(matches!(fields.first(), Some(WireValue::Variant { name, fields }) if name == "Refused" && fields.is_empty()), "{resp:?}");
        // The host records what it refused, and why.
        let (denied, total) = host.denied();
        assert!(total >= 1);
        assert!(
            denied.iter().any(|d| d.contains(&url) && d.contains("scheme")),
            "the refusal and its machine-readable reason must be in denied[]: {denied:?}"
        );
    }

    /// The shape a real program takes: the guest mints from a root it does not hold, gets a handle,
    /// and uses it. Both halves cross the channel, and the host performs both.
    #[test]
    fn a_guest_mints_from_the_hosts_root_and_then_uses_the_handle() {
        use crate::sink::EffectSink;
        let root = crate::value::RootVal { console: true, ..Default::default() };
        let host = HostChannel::new(crate::sink::LocalSink).with_root(std::rc::Rc::new(root));
        let sink = ChannelSink::new(Loopback::new(host));
        // The guest's own root value is empty — it grants nothing and is never consulted.
        let empty_root = crate::value::RootVal::default();
        let span = delulu_diag::Span::new(0, 0, 1);

        let console = sink.root_method(&empty_root, "console", &[], span).expect("the host mints it");
        let Value::Cap(c) = &console else { panic!("minting must answer with a capability") };
        assert!(
            matches!(c.scope, crate::value::CapScope::Handle(_)),
            "the guest must receive a handle, never a scope"
        );

        crate::prim::set_capture(true);
        sink.cap_method(c, "println", &[Value::str("from the guest".to_string())], span).expect("the host performs it");
        let out = crate::prim::take_capture().unwrap_or_default();
        assert!(out.contains("from the guest"), "the host performed the effect: {out:?}");
    }

    /// A run that granted nothing mints nothing: the refusal is explicit, never an empty default.
    #[test]
    fn a_guest_cannot_mint_when_the_host_has_no_root() {
        let mut host = HostChannel::new(crate::sink::LocalSink);
        let req = Request {
            version: CHANNEL_VERSION.into(),
            seq: 1,
            body: ReqBody::RootMethod { method: "console".into(), args: vec![], file: 0, start: 0, end: 1 },
        };
        match host.answer(&req) {
            Response::Error { message, .. } => assert!(message.contains("no root"), "{message}"),
            other => panic!("minting without a root must be refused, got {other:?}"),
        }
    }

    /// A handle the host never minted buys nothing: the answer is a refusal, not an effect.
    #[test]
    fn a_handle_the_host_never_minted_is_refused() {
        let mut host = HostChannel::new(crate::sink::LocalSink);
        let req = Request {
            version: CHANNEL_VERSION.into(),
            seq: 1,
            body: ReqBody::CapMethod { cap: 99, method: "now_ms".into(), args: vec![], file: 0, start: 0, end: 1 },
        };
        match host.answer(&req) {
            Response::Error { code, message } => {
                assert_eq!(code, "DL1401");
                assert!(message.contains("no capability handle 99"), "{message}");
            }
            other => panic!("a forged handle must be refused, got {other:?}"),
        }
    }

    /// A peer speaking another protocol is refused before anything is performed.
    #[test]
    fn a_version_mismatch_is_refused() {
        let mut host = HostChannel::new(crate::sink::LocalSink);
        let req = Request {
            version: "something-else/9".into(),
            seq: 1,
            body: ReqBody::CapMethod { cap: 1, method: "now_ms".into(), args: vec![], file: 0, start: 0, end: 1 },
        };
        assert!(matches!(host.answer(&req), Response::Error { .. }));
    }

    /// The local path must refuse a host-held capability rather than pretend it did the work.
    #[test]
    fn the_local_path_refuses_a_host_held_capability() {
        let cap = CapVal { kind: delulu_check::ResourceKind::Clock, scope: crate::value::CapScope::Handle(7) };
        let err = crate::prim::call_cap_method(&cap, "now_ms", &[], delulu_diag::Span::new(0, 0, 1))
            .expect_err("a handle cannot be performed in this process");
        assert_eq!(err.code, "DL1401");
    }

    /// Hostile bytes reach [`read_frame`] before anything else on this wire, so it must fail, never
    /// panic and never trust a length. Ten thousand seeded pseudo-random frames, deterministic so a
    /// failure is reproducible from the seed alone. (The roadmap's `cargo-fuzz` target is a separate
    /// piece of infrastructure; this is the part that can run in the ordinary suite.)
    #[test]
    fn random_bytes_are_refused_and_never_panic() {
        let mut state = 0x5EED_1234_5678_9ABCu64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for i in 0..20_000u32 {
            // A third of the corpus is noise, a third is a VALID frame, and a third is a valid frame
            // with one byte corrupted. Noise alone almost never decodes, so a property about what
            // the host does with a frame it accepted would never be reached — which is how a fuzz
            // test comes to exercise only its own length check. The valid third is generated from
            // the types themselves, so it follows them when they change.
            let body = match i % 3 {
                0 => (0..(next() % 64) as usize).map(|_| (next() & 0xff) as u8).collect::<Vec<u8>>(),
                _ => {
                    let mut buf = Vec::new();
                    let req = Request {
                        version: if next().is_multiple_of(8) { "wrong/1".into() } else { CHANNEL_VERSION.into() },
                        seq: next(),
                        body: match next() % 4 {
                            0 => ReqBody::Done { exit: (next() % 8) as i32 },
                            3 => ReqBody::Confined {
                                applied: (0..next() % 3)
                                    .map(|_| ["no new programs", "no file writes", "root access", ""][(next() % 4) as usize].into())
                                    .collect(),
                                generation: ["", "00", &"ab".repeat(32)][(next() % 3) as usize].into(),
                            },
                            1 => ReqBody::RootMethod {
                                method: ["console", "fs_read", "fs_write", "clock", "nope"][(next() % 5) as usize].into(),
                                args: vec![random_wire(&mut next, 2)],
                                file: 0,
                                start: 0,
                                end: 1,
                            },
                            _ => ReqBody::CapMethod {
                                // Handles the host never minted, 0 among them: every one must be refused.
                                cap: next() % 5,
                                method: ["print", "read_text", "write_text", "now_ms"][(next() % 4) as usize].into(),
                                args: vec![random_wire(&mut next, 2)],
                                file: 0,
                                start: 0,
                                end: 1,
                            },
                        },
                    };
                    write_frame(&mut buf, &req).expect("the generated request encodes");
                    // Drop the length prefix: `fuzz_one_frame` adds its own, and the body is the part
                    // worth mutating.
                    let mut b = buf[4..].to_vec();
                    if i % 3 == 2 && !b.is_empty() {
                        let at = (next() as usize) % b.len();
                        b[at] ^= (next() & 0xff) as u8;
                    }
                    b
                }
            };
            // The same property the `cargo-fuzz` target runs, so the two can never disagree.
            crate::channel::fuzz_one_frame(&body);
        }
    }

    /// A small arbitrary `WireValue`, bounded in depth so the generator cannot run away.
    fn random_wire(next: &mut impl FnMut() -> u64, depth: u32) -> WireValue {
        match next() % if depth == 0 { 6 } else { 9 } {
            0 => WireValue::Unit,
            1 => WireValue::Bool(next().is_multiple_of(2)),
            2 => WireValue::Int(next() as i64),
            3 => WireValue::Float(f64::from_bits(next())),
            4 => WireValue::Str("s".repeat((next() % 4) as usize)),
            5 => WireValue::Cap { handle: next() % 5, kind: "Console".into() },
            6 => WireValue::List((0..next() % 3).map(|_| random_wire(next, depth - 1)).collect()),
            7 => WireValue::Record {
                name: "R".into(),
                fields: (0..next() % 3).map(|_| ("f".to_string(), random_wire(next, depth - 1))).collect(),
            },
            _ => WireValue::Variant {
                name: "V".into(),
                fields: (0..next() % 3).map(|_| random_wire(next, depth - 1)).collect(),
            },
        }
    }

    /// Canonical encoding: the same value is the same bytes, which is what makes a frame hashable
    /// and an audit record reproducible.
    #[test]
    fn the_same_value_is_the_same_bytes() {
        let v = WireValue::Record {
            name: "R".into(),
            fields: vec![("a".into(), WireValue::Int(1)), ("b".into(), WireValue::Str("s".into()))],
        };
        let (mut one, mut two) = (Vec::new(), Vec::new());
        write_frame(&mut one, &v).unwrap();
        write_frame(&mut two, &v).unwrap();
        assert_eq!(one, two);
    }
}
