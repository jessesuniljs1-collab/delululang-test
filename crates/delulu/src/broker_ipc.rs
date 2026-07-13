//! Phase 5f — the broker IPC wire protocol (spec §2, head-chef ruling 2).
//!
//! Length-prefixed (u32 little-endian) canonical CBOR frames, one request → one response per
//! connection. Every request carries the version tag `"broker/1"`; a mismatch is answered with
//! [`Response::Error`] carrying `DL1406` (spec §8). Messages are fixed-field serde structs / enums,
//! so ciborium encodes them deterministically by construction (no map-ordering ambiguity that a
//! free-form CBOR value would have).
//!
//! Fail-closed (invariant 27): a malformed frame, an oversize length, or an unreadable stream is a
//! transport failure, never a silent success. The client turns any such failure into `DL1401`.

use std::io::{self, Read, Write};

use serde::{Deserialize, Serialize};

/// The wire protocol version, present in every request frame (head-chef ruling 2).
pub const WIRE_VERSION: &str = "broker/1";

/// Hard ceiling on a single frame (16 MiB) — a defensive bound so a corrupt/hostile length prefix
/// cannot make the peer allocate unboundedly. Secrets/tokens are far smaller.
const MAX_FRAME: u32 = 16 * 1024 * 1024;

/// A descriptive authority the client asks the broker to issue/attenuate/delegate (spec §3.1). The
/// broker maps `effects`/scopes to its `Authority`; `holder_*` is descriptive metadata, never
/// switched on (criterion 9).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AuthoritySpec {
    pub effects: Vec<String>,
    pub fs_read: Vec<String>,
    pub fs_write: Vec<String>,
    pub net: Vec<String>,
    pub secrets: Vec<String>,
    pub declassify: Vec<String>,
    pub foreign_c: Vec<String>,
    pub foreign_python: Vec<String>,
    pub holder_kind: String,
    pub holder_desc: String,
    /// Absolute epoch-millis TTL deadline, or `None` for no expiry.
    pub ttl_millis: Option<i64>,
}

/// One grant-tree node on the wire (spec §3.1), for `delulu grants list|inspect` (phase 5j) and
/// for a `--lease` run learning its OWN node's authority after redemption. Fixed-field struct →
/// deterministic CBOR. Additive to `broker/1` (both ends are the same binary).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NodeInfo {
    pub id: String,
    pub parent: Option<String>,
    /// Holder fields are descriptive metadata — stored and displayed, never switched on
    /// (criterion 9).
    pub holder_kind: String,
    pub holder_desc: String,
    pub holder_peer: String,
    /// Effective state: `"live"`, `"revoked"` (with `by_seq`), or `"expired"`.
    pub state: String,
    pub by_seq: Option<u64>,
    pub ttl_millis: Option<i64>,
    pub created_millis: i64,
    pub audit_seq: u64,
    pub effects: Vec<String>,
    pub fs_read: Vec<String>,
    pub fs_write: Vec<String>,
    pub net: Vec<String>,
    pub secrets: Vec<String>,
    pub declassify: Vec<String>,
    pub foreign_c: Vec<String>,
    pub foreign_python: Vec<String>,
}

/// One guard rule on the wire (Stage 5 chunk 6): `class:pattern → tier`. Fixed-field struct →
/// deterministic CBOR. Additive to `broker/1` (both ends are one binary).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GuardRuleWire {
    pub class: String,
    pub pattern: String,
    pub tier: String,
}

impl NodeInfo {
    /// This node's authority as an [`AuthoritySpec`] (holder/ttl fields blank — authority only).
    /// Used by `delulu run --lease` to reconstruct the client-side epoch-snapshot authority.
    pub fn authority_spec(&self) -> AuthoritySpec {
        AuthoritySpec {
            effects: self.effects.clone(),
            fs_read: self.fs_read.clone(),
            fs_write: self.fs_write.clone(),
            net: self.net.clone(),
            secrets: self.secrets.clone(),
            declassify: self.declassify.clone(),
            foreign_c: self.foreign_c.clone(),
            foreign_python: self.foreign_python.clone(),
            holder_kind: String::new(),
            holder_desc: String::new(),
            ttl_millis: None,
        }
    }
}

/// A request to the broker daemon (spec §3.2 operations + lifecycle).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReqBody {
    /// Liveness ping / introspection.
    Status,
    /// Ask the daemon to exit cleanly.
    Shutdown,
    /// Rotate the broker MAC key (invalidates outstanding tokens).
    RotateKey,
    /// Issue a root node (CLI-only human action at the top of the tree).
    Issue(AuthoritySpec),
    /// Attenuate `parent` into a child (`⊑`-checked, DL0802 otherwise). `owner` (Stage 5 chunk 6)
    /// is the guard owner code: minting guarded/sealed authority into the child requires it (or a
    /// covering permit) — the principal minting directly (addendum §2.4.3).
    Attenuate { parent: String, authority: AuthoritySpec, owner: Option<String> },
    /// Attenuate + mint a portable lease token. `owner` gates minting guarded authority (see
    /// [`ReqBody::Attenuate`]).
    Delegate { parent: String, authority: AuthoritySpec, multi: bool, owner: Option<String> },
    /// Redeem a token, binding it to `peer`.
    Redeem { token: String, peer: String },
    /// Revoke `target` on behalf of `caller` (transitive).
    Revoke { caller: String, target: String },
    /// Per-use validation of a synchronous-class op (spec §4.4).
    Check { node: String, op: String, arg: Option<String> },
    /// The epoch-class client cache refresh: the current epoch + this node's effective state.
    NodeState { node: String },
    /// Synchronous declassification of a broker-held secret (phase 5g); audited with `span`.
    Expose { node: String, name: String, span: Option<String> },
    /// A broker-side `Secret.map` whitelist op (phase 5g): apply `op` to the held bytes, returning a
    /// fresh handle name — bytes never cross.
    SecretMap { node: String, name: String, op: String, arg: Option<String> },
    /// A human-readable tree render.
    Tree,
    /// All nodes, sorted by id (phase 5j `delulu grants list` — a CLI/human read surface).
    /// Additive `broker/1` variant (chunk-5; both ends are one binary).
    List,
    /// One node's full detail (phase 5j `delulu grants inspect <id>`; also how a `--lease` run
    /// learns its OWN node's authority after redeeming). Additive `broker/1` variant.
    Inspect { node: String },

    // ----- The Guard (Stage 5 chunk 6, phases 5k–5m). Read verbs carry no owner (awareness is
    // free, addendum §2.2); admin verbs carry `owner`. All additive `broker/1` variants. ---------
    /// Guard status: mode, bypass, rule digest, queue sizes (the machine surface, read).
    GuardStatus,
    /// Guard policy show (read; same payload as [`ReqBody::GuardStatus`]).
    GuardPolicyShow,
    /// Guard policy set `class:pattern → tier` (owner-gated).
    GuardPolicySet { owner: Option<String>, class: String, pattern: String, tier: String },
    /// Guard policy unset `class:pattern` (owner-gated).
    GuardPolicyUnset { owner: Option<String>, class: String, pattern: String },
    /// Guard bypass on|off (owner-gated, `--dangerously-bypass-guard` at runtime).
    GuardBypass { owner: Option<String>, on: bool },
}

/// The versioned request envelope.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Request {
    pub version: String,
    pub body: ReqBody,
}

impl Request {
    pub fn new(body: ReqBody) -> Request {
        Request { version: WIRE_VERSION.to_string(), body }
    }
}

/// A response from the broker daemon.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Response {
    Ok,
    /// A refusal carrying the diagnostic code (DL0802/DL1401/DL1402/DL1403/DL1406/DL1407/DL0904…).
    Error { code: String, message: String, requires_human: bool },
    Issued { node: String },
    Delegated { node: String, token: String },
    Redeemed { node: String },
    Revoked { by_seq: u64, epoch: u64, newly_revoked: Vec<String> },
    /// A per-use decision (synchronous-class). `allow=false` carries the denial code/message.
    /// `warn` (Stage 5 chunk 6) is an agent-side note for a `warn`-tier or bypassed-guarded use that
    /// PROCEEDED — the client surfaces it once per rule per run (addendum §2.6/§2.7).
    Decision { allow: bool, code: Option<String>, message: Option<String>, audit_seq: Option<u64>, warn: Option<String> },
    /// The node's effective state + the current epoch (client epoch-cache refresh). The guard fields
    /// `guarded_classes` and `guard_bypass` (Stage 5 chunk 6) let the client route guarded epoch-class
    /// ops synchronously (addendum §2.4.2 / criterion 11).
    NodeState {
        epoch: u64,
        state: String,
        by_seq: Option<u64>,
        ttl_millis: Option<i64>,
        now_millis: Option<i64>,
        guarded_classes: Vec<String>,
        guard_bypass: bool,
    },
    /// The declassified bytes (phase 5g — the only response that carries secret material).
    Exposed { bytes: String },
    /// A fresh broker-held secret handle name from a broker-side `Secret.map` (phase 5g).
    Mapped { name: String },
    Status { pid: u32, nodes: usize, epoch: u64 },
    Tree { text: String },
    /// `List` reply: every node, sorted by id (additive `broker/1` variant, chunk 5).
    Listed { nodes: Vec<NodeInfo> },
    /// `Inspect` reply (additive `broker/1` variant, chunk 5). Boxed: `NodeInfo` is by far the
    /// widest payload and would otherwise bloat every `Response` on the stack.
    Inspected { node: Box<NodeInfo> },

    // ----- The Guard (Stage 5 chunk 6). Additive `broker/1` variants. --------------------------
    /// Guard status / policy show reply: mode, bypass, rule digest, queue sizes.
    GuardStatus {
        bypass: bool,
        poisoned: bool,
        rules: Vec<GuardRuleWire>,
        pending: usize,
        permits: usize,
    },
}

/// Write one length-prefixed CBOR frame: `[u32-le len][CBOR bytes]`.
pub fn write_frame<W: Write, T: Serialize>(w: &mut W, msg: &T) -> io::Result<()> {
    let mut buf = Vec::new();
    ciborium::into_writer(msg, &mut buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    if buf.len() as u64 > MAX_FRAME as u64 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame exceeds the maximum size"));
    }
    w.write_all(&(buf.len() as u32).to_le_bytes())?;
    w.write_all(&buf)?;
    w.flush()
}

/// Read one length-prefixed CBOR frame. An oversize length or a short read is a hard error
/// (fail-closed), never a partial/best-effort decode.
pub fn read_frame<R: Read, T: for<'de> Deserialize<'de>>(r: &mut R) -> io::Result<T> {
    let mut len_bytes = [0u8; 4];
    r.read_exact(&mut len_bytes)?;
    let len = u32::from_le_bytes(len_bytes);
    if len > MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "incoming frame exceeds the maximum size"));
    }
    let mut buf = vec![0u8; len as usize];
    r.read_exact(&mut buf)?;
    ciborium::from_reader(&buf[..]).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_roundtrips_a_request() {
        let req = Request::new(ReqBody::Check { node: "g_abc".into(), op: "FsWrite".into(), arg: Some("/tmp/x".into()) });
        let mut buf = Vec::new();
        write_frame(&mut buf, &req).unwrap();
        // The first four bytes are the LE length of the remainder.
        let len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        assert_eq!(len, buf.len() - 4);
        let got: Request = read_frame(&mut &buf[..]).unwrap();
        assert_eq!(got, req);
        assert_eq!(got.version, WIRE_VERSION);
    }

    #[test]
    fn frame_roundtrips_a_response() {
        let resp = Response::Revoked { by_seq: 7, epoch: 3, newly_revoked: vec!["g_a".into(), "g_b".into()] };
        let mut buf = Vec::new();
        write_frame(&mut buf, &resp).unwrap();
        let got: Response = read_frame(&mut &buf[..]).unwrap();
        assert_eq!(got, resp);
    }

    /// The chunk-5 additive variants (List/Inspect/Listed/Inspected) round-trip on the SAME
    /// `broker/1` wire — additive enum variants, no version bump (head-chef ruling 5).
    #[test]
    fn additive_grants_variants_roundtrip_on_broker_1() {
        let req = Request::new(ReqBody::Inspect { node: "g_abc".into() });
        let mut buf = Vec::new();
        write_frame(&mut buf, &req).unwrap();
        let got: Request = read_frame(&mut &buf[..]).unwrap();
        assert_eq!(got, req);
        assert_eq!(got.version, WIRE_VERSION, "still broker/1");

        let resp = Response::Listed {
            nodes: vec![NodeInfo {
                id: "g_a".into(),
                state: "live".into(),
                effects: vec!["Read".into()],
                fs_read: vec!["./data".into()],
                ..Default::default()
            }],
        };
        let mut buf = Vec::new();
        write_frame(&mut buf, &resp).unwrap();
        let got: Response = read_frame(&mut &buf[..]).unwrap();
        assert_eq!(got, resp);
    }

    #[test]
    fn truncated_frame_is_an_error_not_a_hang() {
        // Only the length prefix, no body: read_exact of the body must error (fail-closed).
        let mut buf = Vec::new();
        buf.extend_from_slice(&100u32.to_le_bytes());
        let got: io::Result<Response> = read_frame(&mut &buf[..]);
        assert!(got.is_err());
    }
}
