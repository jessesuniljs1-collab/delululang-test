//! Phase 5e — portable lease tokens: `delegate` / `redeem` / `rotate_key` (spec §2, §3.2).
//!
//! This is the orchestration primitive: an LLM holding `g_orch` hands each parallel agent a token
//! minted by [`Broker::delegate`]; each agent [`Broker::redeem`]s it to bind the child node to itself.
//! No party kind is ever inspected (criterion 9) — redemption writes `holder.peer`, never reads
//! `holder.kind`.
//!
//! **MAC (head-chef ruling 1).** Tokens are authenticated with `blake3::keyed_hash` — no `hmac`/`sha2`
//! crate. The format is versioned:
//!
//! ```text
//! dlt1_<hex payload>.<hex mac>
//! payload = canonical JSON { v: 1, node: "g_…", exp_millis, multi: bool, nonce: "<hex>" }
//! mac     = blake3::keyed_hash(broker_key, payload_bytes)
//! ```
//!
//! MAC comparison is constant-time via `blake3::Hash`'s constant-time `PartialEq` (we compare `Hash`
//! values, never hex strings). A bad/garbled/rotated-key/tampered MAC, a malformed token, an unknown
//! bound node, or a second redemption of a single-use token is DL1407; an expired token is DL1402.
//!
//! **Key storage (ruling 2).** [`load_or_create_key`] takes an INJECTED path (the CLI passes
//! `~/.delulu/broker.key`); on Unix the file is created `0600`. On Windows v0.5 relies on the
//! user-profile ACL — full DACL hardening arrives with the named pipe in chunk 3 (noted below).

use std::path::Path;

use serde_json::{json, Value};

use crate::audit::canonical_json;
use crate::authority::Authority;
use crate::diag::Denial;
use crate::tree::{Broker, GrantId, Holder};

/// The token payload version this build mints and accepts.
const TOKEN_V: u64 = 1;
/// The token string prefix (versioned envelope — a future format is `dlt2_…`).
const TOKEN_PREFIX: &str = "dlt1_";

/// A portable, MAC-signed lease token. Opaque: hand it to another process; it carries no authority
/// bytes, only a reference to the bound node plus its expiry and single-use nonce.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token(String);

impl Token {
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn into_string(self) -> String {
        self.0
    }

    /// Reconstruct a token from its wire/display string (chunk 3 IPC `Redeem`, `--lease <token>`).
    /// No validation happens here — `redeem` verifies the MAC and rejects garbage with DL1407, so a
    /// malformed string is safe to wrap (fail closed at the check, not the parse).
    pub fn from_wire(s: impl Into<String>) -> Token {
        Token(s.into())
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Broker {
    /// Delegate: attenuate `parent` into a child node (all phase-5a/5b rules — `⊑`, DL0802 with the
    /// intersection, fail-closed under a dead parent) AND mint a token bound to that child (spec §3.2).
    /// Emits exactly one `"delegate"` audit record (never a separate `"attenuate"` record). `ttl_millis`
    /// is the child node's absolute epoch-millis deadline (as for [`Broker::attenuate`]); the token's
    /// `exp_millis` is that deadline, or "never" when `None`. `multi` mints a multi-redemption token.
    pub fn delegate(
        &mut self,
        parent: &GrantId,
        authority: Authority,
        holder: Holder,
        ttl_millis: Option<i64>,
        multi: bool,
    ) -> Result<(GrantId, Token), Denial> {
        let auth_json = authority.to_json();
        let (seq, res) = self.attenuate_core(parent, authority, holder, ttl_millis);
        match res {
            Ok(child) => {
                let token = self.mint_token(&child, ttl_millis, multi);
                self.record_op(
                    seq,
                    "delegate",
                    Some(parent.as_str().to_string()),
                    Some(child.as_str().to_string()),
                    Some(auth_json),
                    "allow",
                    None,
                );
                Ok((child, token))
            }
            Err(d) => {
                self.record_op(
                    seq,
                    "delegate",
                    Some(parent.as_str().to_string()),
                    None,
                    Some(auth_json),
                    "deny",
                    None,
                );
                Err(d)
            }
        }
    }

    /// Redeem a token, binding its node to `peer_desc` (spec §3.2). Verifies the MAC (bad/garbled/
    /// rotated-key/tampered → DL1407), checks expiry (→ DL1402), and enforces single-redemption
    /// unless the token was minted `multi` (a second redemption → DL1407). Binding writes only
    /// `holder.peer` (display/storage — NEVER a decision input; criterion 9 stays green). Emits one
    /// `"redeem"` record (allow, or deny with `decision: "deny"` for a refused redemption).
    pub fn redeem(&mut self, token: &Token, peer_desc: impl Into<String>) -> Result<GrantId, Denial> {
        let seq = self.consume_seq();
        let peer_desc = peer_desc.into();
        let result = self.redeem_inner(token, &peer_desc);
        let (actor, decision) = match &result {
            Ok(node) => (Some(node.as_str().to_string()), "allow"),
            Err(_) => (None, "deny"),
        };
        self.record_op(seq, "redeem", actor, Some(peer_desc), None, decision, None);
        result
    }

    /// Rotate the broker key: generate a fresh random 256-bit key (spec §2). This invalidates ALL
    /// outstanding tokens — their MACs no longer verify (→ DL1407) — which is the deliberate point.
    /// Emits one `"rotate_key"` audit record.
    pub fn rotate_key(&mut self) {
        let mut k = [0u8; 32];
        getrandom::fill(&mut k).expect("OS randomness (getrandom) unavailable");
        self.set_key(k);
        let seq = self.consume_seq();
        self.record_op(seq, "rotate_key", None, None, None, "allow", None);
    }

    // ----- internals -----------------------------------------------------------------------------

    fn mint_token(&mut self, node: &GrantId, ttl_millis: Option<i64>, multi: bool) -> Token {
        let key = self.ensure_key();
        let mut nonce_bytes = [0u8; 16];
        getrandom::fill(&mut nonce_bytes).expect("OS randomness (getrandom) unavailable");
        let nonce = to_hex(&nonce_bytes);
        let payload = json!({
            "v": TOKEN_V,
            "node": node.as_str(),
            "exp_millis": ttl_millis.unwrap_or(i64::MAX),
            "multi": multi,
            "nonce": nonce,
        });
        let payload_bytes = canonical_json(&payload).into_bytes();
        let mac = blake3::keyed_hash(&key, &payload_bytes);
        Token(format!("{TOKEN_PREFIX}{}.{}", to_hex(&payload_bytes), mac.to_hex()))
    }

    fn redeem_inner(&mut self, token: &Token, peer_desc: &str) -> Result<GrantId, Denial> {
        let key = self.ensure_key();
        // Parse the versioned envelope.
        let Some((payload_bytes, mac_bytes)) = parse_token(token.as_str()) else {
            return Err(Denial::TokenInvalid { detail: "malformed token envelope".into() });
        };
        // Constant-time MAC check: compare blake3::Hash values, never hex strings.
        let expected = blake3::keyed_hash(&key, &payload_bytes);
        if expected != blake3::Hash::from_bytes(mac_bytes) {
            return Err(Denial::TokenInvalid {
                detail: "MAC verification failed (bad, tampered, or rotated-away key)".into(),
            });
        }
        // Decode the (authenticated) payload.
        let Some(claims) = TokenClaims::parse(&payload_bytes) else {
            return Err(Denial::TokenInvalid { detail: "unreadable token payload".into() });
        };
        if claims.v != TOKEN_V {
            return Err(Denial::TokenInvalid {
                detail: format!("unsupported token version {}", claims.v),
            });
        }
        let node = GrantId::from_trusted(claims.node);
        // The bound node must still exist (fail-closed: a token for a vanished node grants nothing).
        if self.inspect(&node).is_none() {
            return Err(Denial::TokenInvalid { detail: "token references an unknown node".into() });
        }
        // Expiry (DL1402), checked against the pluggable clock.
        let now = self.effective_now();
        if now >= claims.exp_millis {
            return Err(Denial::Expired {
                node: node.clone(),
                ttl_millis: claims.exp_millis,
                now_millis: now,
            });
        }
        // Single-redemption (DL1407) unless minted `multi`.
        if !claims.multi {
            if self.is_redeemed(&claims.nonce) {
                return Err(Denial::TokenInvalid {
                    detail: "single-use token already redeemed".into(),
                });
            }
            self.mark_redeemed(&claims.nonce);
        }
        // Bind the node to the redeeming peer (storage/display only).
        self.set_holder_peer(&node, peer_desc);
        Ok(node)
    }
}

/// The authenticated token claims (only read after the MAC verifies).
struct TokenClaims {
    v: u64,
    node: String,
    exp_millis: i64,
    multi: bool,
    nonce: String,
}

impl TokenClaims {
    fn parse(payload_bytes: &[u8]) -> Option<TokenClaims> {
        let v: Value = serde_json::from_slice(payload_bytes).ok()?;
        let obj = v.as_object()?;
        Some(TokenClaims {
            v: obj.get("v")?.as_u64()?,
            node: obj.get("node")?.as_str()?.to_string(),
            exp_millis: obj.get("exp_millis")?.as_i64()?,
            multi: obj.get("multi")?.as_bool()?,
            nonce: obj.get("nonce")?.as_str()?.to_string(),
        })
    }
}

/// Parse `dlt1_<hex payload>.<hex mac>` into (payload bytes, 32-byte MAC). `None` on any shape error.
fn parse_token(tok: &str) -> Option<(Vec<u8>, [u8; 32])> {
    let rest = tok.strip_prefix(TOKEN_PREFIX)?;
    let (phex, mhex) = rest.split_once('.')?;
    let payload = from_hex(phex)?;
    let macv = from_hex(mhex)?;
    if macv.len() != 32 {
        return None;
    }
    let mut mac = [0u8; 32];
    mac.copy_from_slice(&macv);
    Some((payload, mac))
}

fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn from_hex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok()).collect()
}

// ----- key store (CLI injects the path — ruling 2) -----------------------------------------------

/// Load the broker MAC key from `path`, creating a random 256-bit key (and the file) on first use
/// (spec §2). On Unix the file is created with mode `0600`. On Windows v0.5 relies on the
/// user-profile ACL — full owner-only DACL hardening arrives with the named pipe in chunk 3.
///
/// The library never chooses the path (ruling 2); the CLI resolves `~/.delulu/broker.key`.
pub fn load_or_create_key(path: impl AsRef<Path>) -> std::io::Result<[u8; 32]> {
    let path = path.as_ref();
    // Existence check first (rather than matching the read error's ErrorKind — which would also trip
    // the holder-neutrality grep on `.kind`). The same-user TOCTOU window this opens is inside the
    // threat model: the broker does not defend against the same OS user (spec §10).
    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut k = [0u8; 32];
        getrandom::fill(&mut k).expect("OS randomness (getrandom) unavailable");
        write_key_file(path, &k)?;
        return Ok(k);
    }
    let bytes = std::fs::read(path)?;
    if bytes.len() != 32 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "broker key file is not exactly 32 bytes",
        ));
    }
    let mut k = [0u8; 32];
    k.copy_from_slice(&bytes);
    Ok(k)
}

#[cfg(unix)]
fn write_key_file(path: &Path, key: &[u8; 32]) -> std::io::Result<()> {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600) // owner read/write only — the key must not be world/group readable
        .open(path)?;
    f.write_all(key)
}

#[cfg(not(unix))]
fn write_key_file(path: &Path, key: &[u8; 32]) -> std::io::Result<()> {
    // Windows (v0.5): rely on the user-profile ACL of `%USERPROFILE%\.delulu`. The threat model
    // (spec §10) already states the broker does not defend against the same OS user; owner-only DACL
    // hardening lands with the named pipe in chunk 3. Documented, not silently weaker.
    std::fs::write(path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority::Scopes;
    use crate::ids::SeqIdSource;
    use crate::time::ManualClock;
    use crate::tree::Holder;
    use crate::validate::Op;
    use delulu_check::Effect;
    use std::collections::BTreeSet;
    use std::rc::Rc;

    fn eff(names: &[&str]) -> BTreeSet<Effect> {
        names.iter().map(|n| Effect::core_from_name(n).unwrap()).collect()
    }
    fn names(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }
    fn holder() -> Holder {
        Holder::new("process", "orchestrator", "pid:1")
    }
    fn broker(clock: Rc<ManualClock>) -> Broker {
        Broker::with_sources(Box::new(SeqIdSource::new()), Box::new(clock)).with_key([7u8; 32])
    }

    fn root_with_readnet(b: &mut Broker) -> GrantId {
        b.issue(
            holder(),
            Authority::new(
                eff(&["Read", "Net"]),
                Scopes { fs_read: names(&["./data"]), net: names(&["a.com"]), ..Default::default() },
            ),
            None,
        )
    }

    #[test]
    fn delegate_redeem_then_use_works() {
        let clock = Rc::new(ManualClock::new(1000));
        let mut b = broker(clock);
        let root = root_with_readnet(&mut b);
        let (child, token) = b
            .delegate(
                &root,
                Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data/sub"]), ..Default::default() }),
                Holder::new("delegate", "agent-1", "pending"),
                None,
                false,
            )
            .unwrap();
        // Redeem binds the node to the peer; then check() works on the redeemed node.
        let redeemed = b.redeem(&token, "pid:4711").unwrap();
        assert_eq!(redeemed, child);
        assert!(b.check(&child, Op::FsRead, Some("./data/sub/x")).is_allow());
        // The peer was bound (storage/display only).
        assert_eq!(b.inspect(&child).unwrap().holder.peer, "pid:4711");
    }

    #[test]
    fn delegate_widening_is_dl0802_with_intersection() {
        let clock = Rc::new(ManualClock::new(1000));
        let mut b = broker(clock);
        let root = root_with_readnet(&mut b);
        // Ask for Write (not held) + a wider net → DL0802 carrying the never-widening intersection.
        let err = b
            .delegate(
                &root,
                Authority::new(eff(&["Read", "Write"]), Scopes { net: names(&["evil.com"]), ..Default::default() }),
                holder(),
                None,
                false,
            )
            .unwrap_err();
        assert_eq!(err.code(), "DL0802");
        let inter = err.intersection().expect("DL0802 carries an intersection");
        assert_eq!(inter.effects, eff(&["Read"]), "meet keeps only the held effect");
        assert!(inter.scopes.net.is_empty(), "evil.com is not in the parent's net set");
    }

    #[test]
    fn second_redeem_of_single_use_token_is_dl1407() {
        let clock = Rc::new(ManualClock::new(1000));
        let mut b = broker(clock);
        let root = root_with_readnet(&mut b);
        let (_c, token) = b
            .delegate(&root, Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data"]), ..Default::default() }), holder(), None, false)
            .unwrap();
        assert!(b.redeem(&token, "peer-1").is_ok());
        let err = b.redeem(&token, "peer-2").unwrap_err();
        assert_eq!(err.code(), "DL1407");
        assert!(err.requires_human());
    }

    #[test]
    fn multi_token_redeems_more_than_once() {
        let clock = Rc::new(ManualClock::new(1000));
        let mut b = broker(clock);
        let root = root_with_readnet(&mut b);
        let (_c, token) = b
            .delegate(&root, Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data"]), ..Default::default() }), holder(), None, true)
            .unwrap();
        assert!(b.redeem(&token, "peer-1").is_ok());
        assert!(b.redeem(&token, "peer-2").is_ok(), "a multi token redeems repeatedly");
    }

    #[test]
    fn expired_token_at_redeem_is_dl1402() {
        let clock = Rc::new(ManualClock::new(1000));
        let mut b = broker(clock.clone());
        let root = root_with_readnet(&mut b);
        // TTL deadline at 5000 ms.
        let (_c, token) = b
            .delegate(&root, Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data"]), ..Default::default() }), holder(), Some(5000), false)
            .unwrap();
        clock.set(6000); // advance the fake clock past the deadline — no sleep
        let err = b.redeem(&token, "peer").unwrap_err();
        assert_eq!(err.code(), "DL1402");
    }

    #[test]
    fn token_signed_with_a_rotated_away_key_is_dl1407() {
        let clock = Rc::new(ManualClock::new(1000));
        let mut b = broker(clock);
        let root = root_with_readnet(&mut b);
        let (_c, token) = b
            .delegate(&root, Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data"]), ..Default::default() }), holder(), None, false)
            .unwrap();
        b.rotate_key(); // the outstanding token's MAC no longer verifies
        let err = b.redeem(&token, "peer").unwrap_err();
        assert_eq!(err.code(), "DL1407");
    }

    #[test]
    fn tampered_payload_is_dl1407() {
        let clock = Rc::new(ManualClock::new(1000));
        let mut b = broker(clock);
        let root = root_with_readnet(&mut b);
        let (_c, token) = b
            .delegate(&root, Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data"]), ..Default::default() }), holder(), None, false)
            .unwrap();
        // Flip one hex nibble in the payload half → MAC mismatch.
        let s = token.as_str();
        let dot = s.find('.').unwrap();
        let mut bytes: Vec<char> = s.chars().collect();
        let flip = dot - 1; // last hex char of the payload
        bytes[flip] = if bytes[flip] == '0' { '1' } else { '0' };
        let tampered = Token(bytes.into_iter().collect());
        let err = b.redeem(&tampered, "peer").unwrap_err();
        assert_eq!(err.code(), "DL1407");
    }

    #[test]
    fn hex_roundtrips() {
        let bytes = [0u8, 1, 15, 16, 255, 128, 7];
        assert_eq!(from_hex(&to_hex(&bytes)).unwrap(), bytes);
        assert!(from_hex("xyz").is_none());
        assert!(from_hex("abc").is_none()); // odd length
    }

    #[test]
    fn load_or_create_key_is_stable() {
        let dir = std::env::temp_dir().join(format!("delulu_key_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("broker.key");
        let k1 = load_or_create_key(&path).unwrap();
        let k2 = load_or_create_key(&path).unwrap(); // reads the same key back
        assert_eq!(k1, k2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
