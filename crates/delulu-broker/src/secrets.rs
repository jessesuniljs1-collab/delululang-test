//! Phase 5g — the broker-resident secret store (spec §4.4, invariant 23).
//!
//! In daemon mode secret BYTES live here — in the broker process — and enter a program process only
//! through `Broker::expose` (a synchronous, audited declassification). The program holds opaque
//! handle names. `Secret.map` whitelist ops run broker-side ([`Broker::secret_map`]): the transform
//! executes here and hands back a fresh *derived* handle name; bytes never cross.
//!
//! Storage is a small JSON object file (`{ "NAME": "value", … }`) at an INJECTED path (ruling 2 —
//! the CLI resolves `~/.delulu/secrets.json`; the library never chooses it). On Unix the file is
//! written `0600`. Derived (`Secret.map`) values are session-transient: they live in memory with a
//! generated `s_…` name and are NOT persisted (a daemon restart drops them — deliberate: they are
//! re-derivable from their sources).
//!
//! Threat model (spec §10): this defends against the *program*, not the OS user — any same-user
//! process can read the store file, exactly as it could read the broker key.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The broker's secret store: named persistent secrets + session-transient derived secrets.
pub struct SecretStore {
    /// Persistence target; `None` for a purely in-memory store (tests).
    path: Option<PathBuf>,
    /// The persistent entries (name → bytes), mirrored to `path` on every `set`.
    entries: RefCell<BTreeMap<String, String>>,
    /// Session-transient derived entries from `Secret.map` (name → bytes). Never persisted.
    derived: RefCell<BTreeMap<String, DerivedSecret>>,
    /// Counter for generated derived names (deterministic within a session).
    next_derived: RefCell<u64>,
}

struct DerivedSecret {
    value: String,
    /// The PERSISTENT source secret this was (transitively) derived from — scope checks resolve a
    /// derived name back to its root source, so a node may expose a derivative iff it could expose
    /// the source (deriving never widens authority).
    source: String,
}

impl SecretStore {
    /// Load the store from `path` (missing file ⇒ empty store; the file is created on first `set`).
    pub fn load(path: impl Into<PathBuf>) -> SecretStore {
        let path = path.into();
        let entries = match std::fs::read_to_string(&path) {
            Ok(text) => parse_entries(&text),
            Err(_) => BTreeMap::new(),
        };
        SecretStore {
            path: Some(path),
            entries: RefCell::new(entries),
            derived: RefCell::new(BTreeMap::new()),
            next_derived: RefCell::new(0),
        }
    }

    /// A purely in-memory store (tests / embedded experiments). Nothing persists.
    pub fn in_memory() -> SecretStore {
        SecretStore {
            path: None,
            entries: RefCell::new(BTreeMap::new()),
            derived: RefCell::new(BTreeMap::new()),
            next_derived: RefCell::new(0),
        }
    }

    /// Set a persistent secret and mirror the store to disk (`delulu secrets set`).
    pub fn set(&self, name: impl Into<String>, value: impl Into<String>) -> std::io::Result<()> {
        self.entries.borrow_mut().insert(name.into(), value.into());
        self.persist()
    }

    /// The bytes of `name` — persistent or session-derived.
    pub fn get(&self, name: &str) -> Option<String> {
        if let Some(v) = self.entries.borrow().get(name) {
            return Some(v.clone());
        }
        self.derived.borrow().get(name).map(|d| d.value.clone())
    }

    /// The names of the persistent secrets (for `delulu secrets list` — never the values).
    pub fn names(&self) -> Vec<String> {
        self.entries.borrow().keys().cloned().collect()
    }

    /// Resolve a name to the PERSISTENT source secret scope checks authorize against: a persistent
    /// name resolves to itself; a derived name resolves to its recorded root source. `None` for an
    /// unknown name (fail closed).
    pub fn source_of(&self, name: &str) -> Option<String> {
        if self.entries.borrow().contains_key(name) {
            return Some(name.to_string());
        }
        self.derived.borrow().get(name).map(|d| d.source.clone())
    }

    /// Record a session-transient derived secret (a `Secret.map` result), returning its generated
    /// handle name. `source` must be the ROOT persistent source (the caller passes `source_of` of
    /// the input), so derivation chains always resolve in one hop.
    pub fn insert_derived(&self, value: impl Into<String>, source: impl Into<String>) -> String {
        let mut n = self.next_derived.borrow_mut();
        let name = format!("s_derived_{:08x}", *n);
        *n += 1;
        self.derived
            .borrow_mut()
            .insert(name.clone(), DerivedSecret { value: value.into(), source: source.into() });
        name
    }

    fn persist(&self) -> std::io::Result<()> {
        let Some(path) = &self.path else { return Ok(()) };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let entries = self.entries.borrow();
        let map: serde_json::Map<String, serde_json::Value> =
            entries.iter().map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone()))).collect();
        let text = serde_json::to_string_pretty(&serde_json::Value::Object(map))
            .expect("secret store serializes");
        write_secret_file(path, text.as_bytes())
    }
}

fn parse_entries(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(text) {
        for (k, v) in map {
            if let serde_json::Value::String(s) = v {
                out.insert(k, s);
            }
        }
    }
    out
}

#[cfg(unix)]
fn write_secret_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600) // owner-only, like the broker key
        .open(path)?;
    f.write_all(bytes)
}

#[cfg(not(unix))]
fn write_secret_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    // Windows v0.5: the user-profile ACL bounds access (same caveat as broker.key; DACL hardening
    // is chunk-3+ polish). Documented, not silently weaker.
    std::fs::write(path, bytes)
}

// ----- the broker-side expose / secret_map operations (spec §3.2, §4.4) --------------------------

use crate::diag::Denial;
use crate::tree::{Broker, EffState, GrantId};

impl Broker {
    /// Synchronous declassification of a broker-held secret (spec §3.2 `expose`, §4.4): the ONLY
    /// operation that hands secret bytes to a program process. Requires a live node whose authority
    /// carries the `Declassify` effect and whose `secrets` scope covers the (source of the) name.
    /// Always consumes one audit seq and emits one `"expose"` record — allow or deny — carrying the
    /// calling `span` (invariant 26; feedback quality for agents).
    pub fn expose(
        &mut self,
        node_id: &GrantId,
        name: &str,
        store: &SecretStore,
        span: Option<String>,
    ) -> Result<String, Denial> {
        let seq = self.consume_seq();
        let result = self.expose_inner(node_id, name, store);
        let decision = if result.is_ok() { "allow" } else { "deny" };
        self.record_op(
            seq,
            "expose",
            Some(node_id.as_str().to_string()),
            Some(name.to_string()),
            None,
            decision,
            span,
        );
        result
    }

    /// Guard-aware `expose` (Stage 5 chunk 6): a declassification from a delegated node is guarded by
    /// the default policy, so it is gated exactly like a synchronous `check` (addendum §2.4.1) — one
    /// audit event replaces the plain `"expose"` record. Returns the bytes/denial plus an optional
    /// agent-side warn note (warn-tier / bypassed-guarded). Root nodes are ungated (principal use).
    pub fn expose_guarded(
        &mut self,
        node_id: &GrantId,
        name: &str,
        store: &SecretStore,
        span: Option<String>,
    ) -> (Result<String, Denial>, Option<String>) {
        use crate::guard::GuardVerdict;
        let actor = || Some(node_id.as_str().to_string());
        let tgt = || Some(name.to_string());
        // 1. Authorize normally (Declassify effect + secrets scope + liveness) — the guard sits ON
        //    TOP, exactly like `check_use`, so a node lacking the authority is DL0904, not DL1410.
        let auth = self.expose_inner(node_id, name, store);
        if let Err(d) = auth {
            let seq = self.consume_seq();
            self.record_op(seq, "expose", actor(), tgt(), None, "deny", span);
            return (Err(d), None);
        }
        // 2. Authorized. Apply the Guard (declassify is guarded by default) — one event, one seq.
        //    The declassify axis token is the secret name (matched by `declassify:*`).
        match self.guard_verdict_use(node_id, crate::validate::Op::Declassify, Some(name)) {
            GuardVerdict::Block(d) => {
                let seq = self.consume_seq();
                self.record_op(seq, "guard_block", actor(), tgt(), None, "deny", span);
                (Err(d), None)
            }
            GuardVerdict::Ungated => {
                let seq = self.consume_seq();
                self.record_op(seq, "expose", actor(), tgt(), None, "allow", span);
                (auth, None)
            }
            GuardVerdict::PermitUse => {
                let seq = self.consume_seq();
                self.record_op(seq, "guard_permit_use", actor(), tgt(), None, "allow", span);
                (auth, None)
            }
            GuardVerdict::BypassedUse { note } => {
                let seq = self.consume_seq();
                self.record_op(seq, "guard_bypassed_use", actor(), tgt(), None, "allow", span);
                (auth, Some(note))
            }
            GuardVerdict::Warn { note } => {
                let seq = self.consume_seq();
                self.record_op(seq, "guard_warn", actor(), tgt(), None, "allow", span);
                (auth, Some(note))
            }
        }
    }

    fn expose_inner(&self, node_id: &GrantId, name: &str, store: &SecretStore) -> Result<String, Denial> {
        let authority = self.live_authority(node_id)?;
        // The Declassify effect gates byte-crossing (defense in depth; the checker bounds kind).
        if !authority.effects.contains(&delulu_check::Effect::Declassify) {
            return Err(Denial::OutOfScope {
                node: node_id.clone(),
                dimension: "effects",
                arg: "Declassify".to_string(),
            });
        }
        // Scope: the node's `secrets` set must cover the ROOT SOURCE of the name — a `Secret.map`
        // derivative is exposable iff its source is (deriving never widens authority).
        let Some(source) = store.source_of(name) else {
            // Unknown to the store (fail closed) — distinct dimension so the message is honest.
            return Err(Denial::OutOfScope {
                node: node_id.clone(),
                dimension: "secrets.store",
                arg: name.to_string(),
            });
        };
        if !authority.scopes.secrets.contains(&source) {
            return Err(Denial::OutOfScope {
                node: node_id.clone(),
                dimension: "secrets",
                arg: name.to_string(),
            });
        }
        store.get(name).ok_or_else(|| Denial::OutOfScope {
            node: node_id.clone(),
            dimension: "secrets.store",
            arg: name.to_string(),
        })
    }

    /// A broker-side `Secret.map` whitelist op (spec §4.4): apply `op` to the held bytes and return
    /// a fresh session-derived handle name — the bytes NEVER cross to the program. Whitelist: `trim`,
    /// `upper`, `lower` (anything else is refused, fail closed). Requires the same live-node +
    /// `secrets`-scope authority as `expose` but NOT the `Declassify` effect (no bytes cross).
    /// Consumes one audit seq and emits one `"secret_map"` record per call.
    pub fn secret_map(
        &mut self,
        node_id: &GrantId,
        name: &str,
        op: &str,
        _arg: Option<&str>,
        store: &SecretStore,
    ) -> Result<String, Denial> {
        let seq = self.consume_seq();
        let result = self.secret_map_inner(node_id, name, op, store);
        let decision = if result.is_ok() { "allow" } else { "deny" };
        self.record_op(
            seq,
            "secret_map",
            Some(node_id.as_str().to_string()),
            Some(format!("{name}:{op}")),
            None,
            decision,
            None,
        );
        result
    }

    fn secret_map_inner(&self, node_id: &GrantId, name: &str, op: &str, store: &SecretStore) -> Result<String, Denial> {
        let authority = self.live_authority(node_id)?;
        let Some(source) = store.source_of(name) else {
            return Err(Denial::OutOfScope {
                node: node_id.clone(),
                dimension: "secrets.store",
                arg: name.to_string(),
            });
        };
        if !authority.scopes.secrets.contains(&source) {
            return Err(Denial::OutOfScope {
                node: node_id.clone(),
                dimension: "secrets",
                arg: name.to_string(),
            });
        }
        let Some(value) = store.get(name) else {
            return Err(Denial::OutOfScope {
                node: node_id.clone(),
                dimension: "secrets.store",
                arg: name.to_string(),
            });
        };
        // The whitelist (fail closed on anything else).
        let mapped = match op {
            "trim" => value.trim().to_string(),
            "upper" => value.to_uppercase(),
            "lower" => value.to_lowercase(),
            other => {
                return Err(Denial::OutOfScope {
                    node: node_id.clone(),
                    dimension: "secret.map",
                    arg: other.to_string(),
                })
            }
        };
        Ok(store.insert_derived(mapped, source))
    }

    /// The authority of a LIVE node, or the liveness denial (DL1403/DL1402/DL0904 unknown).
    fn live_authority(&self, node_id: &GrantId) -> Result<crate::authority::Authority, Denial> {
        let Some(eff) = self.effective_state(node_id) else {
            return Err(Denial::UnknownNode { node: node_id.clone() });
        };
        match eff {
            EffState::Revoked { by_seq } => Err(Denial::Revoked { node: node_id.clone(), by_seq }),
            EffState::Expired { ttl_millis, now_millis } => {
                Err(Denial::Expired { node: node_id.clone(), ttl_millis, now_millis })
            }
            EffState::Live => Ok(self
                .inspect(node_id)
                .expect("effective_state implies the node exists")
                .authority
                .clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_persist_roundtrip() {
        let dir = std::env::temp_dir().join(format!("delulu_secrets_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("secrets.json");
        {
            let store = SecretStore::load(&path);
            assert!(store.get("API_KEY").is_none());
            store.set("API_KEY", "hunter2").unwrap();
            assert_eq!(store.get("API_KEY").as_deref(), Some("hunter2"));
        }
        // A fresh load reads the persisted value back.
        let store = SecretStore::load(&path);
        assert_eq!(store.get("API_KEY").as_deref(), Some("hunter2"));
        assert_eq!(store.names(), vec!["API_KEY"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn derived_secrets_resolve_to_their_root_source_and_do_not_persist() {
        let dir = std::env::temp_dir().join(format!("delulu_secrets_der_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("secrets.json");
        let d1;
        {
            let store = SecretStore::load(&path);
            store.set("TOKEN", "  abc  ").unwrap();
            d1 = store.insert_derived("abc", "TOKEN");
            assert_eq!(store.get(&d1).as_deref(), Some("abc"));
            assert_eq!(store.source_of(&d1).as_deref(), Some("TOKEN"), "derived resolves to its source");
            assert_eq!(store.source_of("TOKEN").as_deref(), Some("TOKEN"));
            assert!(store.source_of("nope").is_none(), "unknown name has no source (fail closed)");
            // Chained derivation: the caller passes the ROOT source, so it still resolves in one hop.
            let d2 = store.insert_derived("ABC", store.source_of(&d1).unwrap());
            assert_eq!(store.source_of(&d2).as_deref(), Some("TOKEN"));
        }
        // Restart: the derived entry is gone (session-transient); the persistent one remains.
        let store = SecretStore::load(&path);
        assert!(store.get(&d1).is_none(), "derived secrets do not survive a restart");
        assert_eq!(store.get("TOKEN").as_deref(), Some("  abc  "));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ----- Broker::expose / Broker::secret_map (phase 5g) ------------------------------------

    use crate::authority::{Authority, Scopes};
    use crate::ids::SeqIdSource;
    use crate::time::ManualClock;
    use crate::tree::Holder;
    use delulu_check::Effect;
    use std::collections::BTreeSet;
    use std::rc::Rc;

    fn eff(names: &[&str]) -> BTreeSet<Effect> {
        names.iter().map(|n| Effect::core_from_name(n).unwrap()).collect()
    }
    fn names(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    /// A broker + sink + a root node holding Declassify over {API_KEY} and a store with two secrets.
    fn expose_fixture() -> (Broker, crate::audit::MemSink, GrantId, SecretStore) {
        let sink = crate::audit::MemSink::new();
        let mut b = Broker::with_sources(
            Box::new(SeqIdSource::new()),
            Box::new(Rc::new(ManualClock::new(1000))),
        )
        .with_sink(Box::new(sink.clone()));
        let root = b.issue(
            Holder::new("process", "app", "pid:1"),
            Authority::new(eff(&["Declassify"]), Scopes { secrets: names(&["API_KEY"]), ..Default::default() }),
            None,
        );
        let store = SecretStore::in_memory();
        store.set("API_KEY", "hunter2").unwrap();
        store.set("OTHER", "nope").unwrap();
        (b, sink, root, store)
    }

    #[test]
    fn expose_returns_bytes_and_audits_with_span() {
        let (mut b, sink, root, store) = expose_fixture();
        let bytes = b.expose(&root, "API_KEY", &store, Some("app.delulu:10:24".into())).unwrap();
        assert_eq!(bytes, "hunter2");
        let recs = sink.records();
        let last = recs.last().unwrap();
        assert_eq!(last.action, "expose");
        assert_eq!(last.decision, "allow");
        assert_eq!(last.span.as_deref(), Some("app.delulu:10:24"), "expose is audited with the calling span");
        assert_eq!(last.target.as_deref(), Some("API_KEY"));
    }

    #[test]
    fn expose_out_of_scope_and_unknown_are_denied_and_audited() {
        let (mut b, sink, root, store) = expose_fixture();
        // In the store but NOT in the node's secrets scope.
        let err = b.expose(&root, "OTHER", &store, None).unwrap_err();
        assert_eq!(err.code(), "DL0904");
        // In scope name-wise but absent from the store (fail closed).
        let err2 = b.expose(&root, "MISSING", &store, None).unwrap_err();
        assert_eq!(err2.code(), "DL0904");
        // Both denies audited (one record each).
        let denies = sink.records().iter().filter(|r| r.action == "expose" && r.decision == "deny").count();
        assert_eq!(denies, 2);
    }

    #[test]
    fn expose_requires_declassify_effect_and_a_live_node() {
        let sink = crate::audit::MemSink::new();
        let mut b = Broker::with_sources(
            Box::new(SeqIdSource::new()),
            Box::new(Rc::new(ManualClock::new(1000))),
        )
        .with_sink(Box::new(sink.clone()));
        // No Declassify effect.
        let node = b.issue(
            Holder::new("process", "app", "pid:1"),
            Authority::new(eff(&["Read"]), Scopes { secrets: names(&["API_KEY"]), ..Default::default() }),
            None,
        );
        let store = SecretStore::in_memory();
        store.set("API_KEY", "x").unwrap();
        assert_eq!(b.expose(&node, "API_KEY", &store, None).unwrap_err().code(), "DL0904");
        // Revoked node → DL1403 with the revoking seq.
        let (mut b2, _s2, root2, store2) = expose_fixture();
        b2.revoke(&root2, &root2).unwrap();
        let err = b2.expose(&root2, "API_KEY", &store2, None).unwrap_err();
        assert_eq!(err.code(), "DL1403");
        assert!(err.revoking_seq().is_some());
    }

    #[test]
    fn secret_map_runs_broker_side_and_derivative_exposes_iff_source_does() {
        let (mut b, sink, root, store) = expose_fixture();
        store.set("API_KEY", "  abc  ").unwrap();
        // map trim → a derived handle; NO bytes returned.
        let derived = b.secret_map(&root, "API_KEY", "trim", None, &store).unwrap();
        assert!(derived.starts_with("s_derived_"), "map returns a handle name, never bytes: {derived}");
        assert_eq!(sink.records().last().unwrap().action, "secret_map");
        // The derivative exposes because its SOURCE is in scope.
        assert_eq!(b.expose(&root, &derived, &store, None).unwrap(), "abc");
        // A non-whitelisted op is refused (fail closed).
        assert_eq!(b.secret_map(&root, "API_KEY", "reverse", None, &store).unwrap_err().code(), "DL0904");
        // Mapping an out-of-scope secret is refused.
        assert_eq!(b.secret_map(&root, "OTHER", "trim", None, &store).unwrap_err().code(), "DL0904");
    }
}
