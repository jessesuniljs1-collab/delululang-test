//! `delulu-sandbox-channel/1` (PS-A-02): the wire between a guest interpreter and the host that
//! performs its effects.
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
pub const CHANNEL_VERSION: &str = "delulu-sandbox-channel/1";

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
    /// A capability, by host-minted handle. The guest holds the number and nothing else.
    Cap(Handle),
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
            Value::Cap(c) => WireValue::Cap(cap(c)),
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
            WireValue::Cap(h) => Value::Cap(cap(h).ok_or(Untransferable("an unknown capability handle"))?),
        })
    }
}

/// A name for a value kind that has no wire form, for the refusal message.
fn leaked_kind(_v: &Value) -> &'static str {
    "a value of a kind that does not cross the sandbox channel"
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
    /// The guest has finished; the host stops reading.
    Done { exit: i32 },
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
pub fn write_frame<T: Serialize>(w: &mut impl Write, msg: &T) -> io::Result<()> {
    let mut buf = Vec::new();
    ciborium::into_writer(msg, &mut buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    if buf.len() > MAX_FRAME as usize {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame exceeds the channel's bound"));
    }
    w.write_all(&(buf.len() as u32).to_le_bytes())?;
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
    ciborium::from_reader(&buf[..]).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let wire = WireValue::Cap(42);
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
