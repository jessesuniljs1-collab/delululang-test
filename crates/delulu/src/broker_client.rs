//! Phase 5f — `BrokerClientCustody`: the daemon-mode [`Custody`] impl (spec §4).
//!
//! Every effectful op is authorized by the broker in ANOTHER process:
//! - **Synchronous class** (`FsWrite`/`Net`/`Declassify`/`ForeignBind`) — one IPC round-trip per use
//!   against live tree state.
//! - **Epoch class** (`FsRead`/`Clock`/`Rand`/`Console`) — validated locally against a cached
//!   [`Snapshot`] refreshed at most every `--epoch-ms` (default **50 ms**, ceiling **250 ms** —
//!   clamped in [`clamp_epoch_ms`], spec §4.1). The client caches its OWN node's authority (known
//!   from its `issue`) and refreshes only the effective state + epoch, then validates through the
//!   exact same `Snapshot::check` the broker uses — no duplicated policy logic (playbook §1).
//!
//! **FAIL CLOSED (invariant 27, playbook trap 4).** A broker that cannot be reached — daemon down,
//! pipe gone mid-run, malformed frame — makes every effectful op fail **DL1401** with the exact
//! start command in the message. There is NO fallback to embedded custody, silent or otherwise:
//! this type has no embedded path to fall back TO, and `mode()` is always `"daemon"`.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use delulu_broker::{Authority, EffState, GrantId, Op, OpClass, Snapshot};
use delulu_runtime::{Custody, CustodyDecision, CustodyDenial};

use crate::broker_ipc::{AuthoritySpec, ReqBody, Response};
use crate::brokerd;

/// The exact fail-closed message every unreachable-broker denial carries (invariant 27).
fn dl1401(detail: &str) -> CustodyDenial {
    CustodyDenial::new(
        "DL1401",
        format!(
            "broker unreachable: {detail} — start it with `delulu broker start` \
             (fail closed, invariant 27: effectful ops never fall back to embedded custody)"
        ),
    )
}

/// Map a wire diagnostic code to its `&'static` registry form (the `Denial` codes the daemon can
/// answer with). An UNKNOWN code maps to DL1401 (a protocol-level surprise is a broker failure —
/// fail closed, never invent an allow).
fn static_code(code: &str) -> &'static str {
    match code {
        "DL0802" => "DL0802",
        "DL0904" => "DL0904",
        "DL1401" => "DL1401",
        "DL1402" => "DL1402",
        "DL1403" => "DL1403",
        "DL1405" => "DL1405",
        "DL1406" => "DL1406",
        "DL1407" => "DL1407",
        _ => "DL1401",
    }
}

/// Clamp `--epoch-ms` to spec §4.1: default **50**, floor 1, ceiling **250**. Documented bound: a
/// larger value would stretch the honest revocation latency window beyond what §4.2 states.
pub fn clamp_epoch_ms(requested: Option<u64>) -> u64 {
    requested.unwrap_or(50).clamp(1, 250)
}

/// Daemon-mode custody: one node's lease, checked over IPC. See the module docs.
pub struct BrokerClientCustody {
    state_dir: PathBuf,
    node: GrantId,
    /// The node's authority, known client-side from our own `issue` — used to reconstruct the epoch
    /// snapshot without shipping the whole authority every refresh.
    authority: Authority,
    epoch_ms: u64,
    /// The cached epoch snapshot + when it was taken. `None` until the first epoch-class check.
    cache: Option<(Instant, Snapshot)>,
}

impl BrokerClientCustody {
    /// Issue a root node for this run (the daemon-mode `--grant` sugar: issue-then-run) and return
    /// the custody client bound to it. Fails DL1401 when the broker is unreachable.
    pub fn issue_root(
        state_dir: PathBuf,
        spec: AuthoritySpec,
        epoch_ms: Option<u64>,
    ) -> Result<BrokerClientCustody, CustodyDenial> {
        let authority = crate::brokerd::spec_to_authority(&spec);
        let resp = rpc(&state_dir, ReqBody::Issue(spec))?;
        match resp {
            Response::Issued { node } => Ok(BrokerClientCustody {
                state_dir,
                node: GrantId::from_trusted(node),
                authority,
                epoch_ms: clamp_epoch_ms(epoch_ms),
                cache: None,
            }),
            Response::Error { code, message, .. } => Err(CustodyDenial::new(static_code(&code), message)),
            other => Err(dl1401(&format!("unexpected issue response: {other:?}"))),
        }
    }

    /// Bind to an EXISTING node (a redeemed lease / tests). No issue round-trip; the authority is
    /// the client-side copy used for epoch snapshot reconstruction. Wired to `delulu run --lease
    /// <token>` in chunk 5 (phase 5j); exercised now by the daemon-mode tests.
    #[allow(dead_code)]
    pub fn for_node(
        state_dir: PathBuf,
        node: GrantId,
        authority: Authority,
        epoch_ms: Option<u64>,
    ) -> BrokerClientCustody {
        BrokerClientCustody { state_dir, node, authority, epoch_ms: clamp_epoch_ms(epoch_ms), cache: None }
    }

    /// The node this custody client holds (for `revoke` in tests / display).
    #[allow(dead_code)]
    pub fn node(&self) -> &GrantId {
        &self.node
    }

    /// Refresh the epoch cache from the daemon (one `NodeState` round-trip). Fail closed: an
    /// unreachable broker is an `Err`, and the STALE CACHE IS DISCARDED — a dead broker must not
    /// keep serving allows from an old snapshot beyond this refresh point.
    fn refresh_cache(&mut self) -> Result<(), CustodyDenial> {
        self.cache = None;
        let resp = rpc(&self.state_dir, ReqBody::NodeState { node: self.node.as_str().to_string() })?;
        match resp {
            Response::NodeState { epoch, state, by_seq, ttl_millis, now_millis } => {
                let eff = match state.as_str() {
                    "live" => EffState::Live,
                    "revoked" => EffState::Revoked { by_seq: by_seq.unwrap_or(0) },
                    "expired" => EffState::Expired {
                        ttl_millis: ttl_millis.unwrap_or(0),
                        now_millis: now_millis.unwrap_or(0),
                    },
                    other => return Err(dl1401(&format!("unknown node state `{other}`"))),
                };
                let snap = Snapshot::from_entries(
                    epoch,
                    vec![(self.node.clone(), eff, self.authority.clone())],
                );
                self.cache = Some((Instant::now(), snap));
                Ok(())
            }
            Response::Error { code, message, .. } => Err(CustodyDenial::new(static_code(&code), message)),
            other => Err(dl1401(&format!("unexpected node-state response: {other:?}"))),
        }
    }

    fn cache_is_fresh(&self) -> bool {
        matches!(&self.cache, Some((at, _)) if at.elapsed() < Duration::from_millis(self.epoch_ms))
    }
}

/// One request/response round-trip; any transport failure is DL1401 (fail closed, invariant 27).
fn rpc(state_dir: &std::path::Path, body: ReqBody) -> Result<Response, CustodyDenial> {
    brokerd::request(state_dir, body).map_err(|e| dl1401(&e.to_string()))
}

impl Custody for BrokerClientCustody {
    fn check(&mut self, op: Op, arg: Option<&str>) -> CustodyDecision {
        match op.class() {
            OpClass::Synchronous => {
                // One broker round-trip per use, against LIVE tree state (spec §4.1).
                let resp = match rpc(
                    &self.state_dir,
                    ReqBody::Check {
                        node: self.node.as_str().to_string(),
                        op: op.wire_name().to_string(),
                        arg: arg.map(str::to_string),
                    },
                ) {
                    Ok(r) => r,
                    Err(d) => return CustodyDecision::Deny(d),
                };
                match resp {
                    Response::Decision { allow: true, .. } => CustodyDecision::Allow,
                    Response::Decision { allow: false, code, message, .. } => {
                        CustodyDecision::Deny(CustodyDenial::new(
                            static_code(code.as_deref().unwrap_or("DL1401")),
                            message.unwrap_or_else(|| "broker denied the operation".to_string()),
                        ))
                    }
                    Response::Error { code, message, .. } => {
                        CustodyDecision::Deny(CustodyDenial::new(static_code(&code), message))
                    }
                    other => CustodyDecision::Deny(dl1401(&format!("unexpected check response: {other:?}"))),
                }
            }
            OpClass::Epoch => {
                // Validate against the cached snapshot, refreshed at most every epoch_ms (spec §4.1).
                // Revocation reaches epoch-class ops within ≤ one interval — the honest §4.2 bound,
                // never "immediate". An unreachable broker at refresh time is DL1401 (fail closed).
                if !self.cache_is_fresh() {
                    if let Err(d) = self.refresh_cache() {
                        return CustodyDecision::Deny(d);
                    }
                }
                let Some((_, snap)) = &self.cache else {
                    return CustodyDecision::Deny(dl1401("epoch snapshot unavailable"));
                };
                match snap.check(&self.node, op, arg) {
                    delulu_broker::Decision::Allow { .. } => CustodyDecision::Allow,
                    delulu_broker::Decision::Deny(denial) => {
                        let diag = denial.to_diagnostic();
                        CustodyDecision::Deny(CustodyDenial::new(static_code(diag.code), diag.message))
                    }
                }
            }
        }
    }

    fn expose(&mut self, name: &str, span: Option<&str>) -> Result<String, CustodyDenial> {
        // Synchronous declassification through the broker (phase 5g): the bytes cross HERE, for the
        // first time, audited daemon-side with the calling span.
        let resp = rpc(
            &self.state_dir,
            ReqBody::Expose {
                node: self.node.as_str().to_string(),
                name: name.to_string(),
                span: span.map(str::to_string),
            },
        )?;
        match resp {
            Response::Exposed { bytes } => Ok(bytes),
            Response::Error { code, message, .. } => Err(CustodyDenial::new(static_code(&code), message)),
            other => Err(dl1401(&format!("unexpected expose response: {other:?}"))),
        }
    }

    fn refresh_epoch(&mut self) {
        let _ = self.refresh_cache();
    }

    fn mode(&self) -> &'static str {
        "daemon"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_ms_clamps_to_spec_bounds() {
        assert_eq!(clamp_epoch_ms(None), 50, "default 50 ms (spec §4.1)");
        assert_eq!(clamp_epoch_ms(Some(10)), 10);
        assert_eq!(clamp_epoch_ms(Some(0)), 1, "floor 1 ms");
        assert_eq!(clamp_epoch_ms(Some(1000)), 250, "ceiling 250 ms (spec §4.1)");
    }

    #[test]
    fn unknown_wire_code_maps_to_dl1401_fail_closed() {
        assert_eq!(static_code("DL1403"), "DL1403");
        assert_eq!(static_code("DL9999"), "DL1401", "an unknown code is a broker failure, fail closed");
    }

    /// Invariant 27 / playbook trap 4: with NO daemon on this state dir, an effectful op fails
    /// DL1401 — fast (no hang), with the exact start command in the message, and with NO fallback
    /// to embedded custody (the decision is Deny; mode stays "daemon").
    #[test]
    fn broker_unreachable_is_dl1401_fast_never_embedded_fallback() {
        let dir = std::env::temp_dir().join(format!("delulu_no_daemon_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let mut custody = BrokerClientCustody::for_node(
            dir.clone(),
            GrantId::from_trusted("g_00000000000000000000000000000000"),
            Authority::default(),
            None,
        );

        let started = Instant::now();
        // Synchronous class: the round-trip itself fails.
        let d = custody.check(Op::FsWrite, Some("./out/x"));
        let CustodyDecision::Deny(denial) = d else { panic!("must deny when the broker is unreachable") };
        assert_eq!(denial.code, "DL1401");
        assert!(denial.message.contains("delulu broker start"), "message carries the exact start command: {}", denial.message);

        // Epoch class: the cache refresh fails — also DL1401, not a stale allow.
        let d2 = custody.check(Op::FsRead, Some("./data/x"));
        assert!(matches!(d2, CustodyDecision::Deny(ref dd) if dd.code == "DL1401"));

        // expose fails closed the same way (no bytes from nowhere).
        assert_eq!(custody.expose("API_KEY", None).unwrap_err().code, "DL1401");

        // Not a hang: both classes + expose failed well under the connect deadline.
        assert!(started.elapsed() < Duration::from_secs(10), "fail-closed must be fast, not a hang");
        // And there is no embedded fallback to observe: the mode is daemon, the decision was Deny.
        assert_eq!(custody.mode(), "daemon");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
