//! Broker denials and their mapping to `delulu_diag::Diagnostic`.
//!
//! **Design note (ruling 7).** `delulu_diag::Diagnostic` has no free-form structured-payload field,
//! and Stage 4 did not add one: its DL13xx runtime refusals are carried as a native error type
//! (`runtime::Fault { code, message, span }`) with the rich detail in the message, and `Repair`
//! edits are byte-range source edits (which the broker, operating on the runtime grant tree rather
//! than source text, has no spans for). We follow that exact pattern here:
//!
//! - [`Denial`] is the broker-native, machine-consumable error. It carries the FULL structured
//!   payload — the DL0802 intersection [`Authority`], the DL1403 revoking audit seq, the DL1402
//!   TTL/now — as typed Rust fields. This is the single source of truth that chunk 3 wires in.
//! - [`Denial::to_diagnostic`] renders the human/JSON surface: the stable DL code plus a rich
//!   message embedding the same facts. DL0802 additionally carries a typed [`Repair`] flagged
//!   `authority_widening: false` / `Confidence::Exact` — the machine-checkable proof that the
//!   attenuation repair *never widens* (spec §8). The concrete text edit awaits a source span
//!   (chunk 3); the narrowing target itself is the intersection on the `Denial`.

use delulu_diag::{Confidence, Diagnostic, Repair};

use crate::authority::Authority;
use crate::tree::GrantId;

/// A refusal from the broker, with its full structured payload. Every variant maps to exactly one
/// registered diagnostic code via [`Denial::code`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Denial {
    /// DL0802 — the requested authority is not `⊑` the holder's grant. Carries the exact,
    /// never-widening `intersection` (`requested ⊓ holder`) as the repair target.
    Attenuation {
        holder: GrantId,
        requested: Box<Authority>,
        intersection: Box<Authority>,
    },
    /// DL1402 — the lease's TTL has passed (checked against the pluggable clock, ruling 3).
    Expired { node: GrantId, ttl_millis: i64, now_millis: i64 },
    /// DL1403 — the lease was revoked. Carries the revoking op's audit seq, so an agent's JSON
    /// error says *why and when* its authority died (spec §4.3 — feedback quality is a feature).
    Revoked { node: GrantId, by_seq: u64 },
    /// DL0904 — a use whose scope argument is outside the node's granted authority (defense in
    /// depth: the type checker already bounds *kind*; the broker bounds *scope* at runtime).
    OutOfScope { node: GrantId, dimension: &'static str, arg: String },
    /// DL0904 — `revoke` refused because the target is neither the caller's node nor a descendant
    /// of it (spec §3.2). The caller's authority (its own subtree) does not reach the target.
    NotRevocable { caller: GrantId, target: GrantId },
    /// DL0904 — an operation named a grant id that does not exist in the tree (fail-closed: an
    /// unknown lease confers no authority).
    UnknownNode { node: GrantId },
    /// DL1405 — the append-only audit chain failed verification at `seq` (a recomputed hash or a
    /// cross-record `prev_hash` linkage did not match). `requires_human: true` (spec §8): the log
    /// is observability, not enforcement — a break signals possible tampering, never a policy
    /// decision. `detail` states what mismatched (hash vs prev-link).
    AuditChainBroken { seq: u64, detail: String },
}

impl Denial {
    /// The stable diagnostic code for this denial.
    pub fn code(&self) -> &'static str {
        match self {
            Denial::Attenuation { .. } => "DL0802",
            Denial::Expired { .. } => "DL1402",
            Denial::Revoked { .. } => "DL1403",
            Denial::OutOfScope { .. }
            | Denial::NotRevocable { .. }
            | Denial::UnknownNode { .. } => "DL0904",
            Denial::AuditChainBroken { .. } => "DL1405",
        }
    }

    /// Whether resolving this denial requires a human decision (spec §8: DL1402/DL1403/DL1405 are
    /// `requires_human: true`; the attenuation and scope denials are mechanically resolvable).
    pub fn requires_human(&self) -> bool {
        matches!(
            self,
            Denial::Expired { .. } | Denial::Revoked { .. } | Denial::AuditChainBroken { .. }
        )
    }

    /// The never-widening repair target for a DL0802 attenuation denial, else `None`.
    pub fn intersection(&self) -> Option<&Authority> {
        match self {
            Denial::Attenuation { intersection, .. } => Some(intersection),
            _ => None,
        }
    }

    /// The revoking audit seq for a DL1403 revoked-lease denial, else `None`.
    pub fn revoking_seq(&self) -> Option<u64> {
        match self {
            Denial::Revoked { by_seq, .. } => Some(*by_seq),
            _ => None,
        }
    }

    /// The failing audit seq for a DL1405 audit-chain-break denial, else `None`.
    pub fn audit_break_seq(&self) -> Option<u64> {
        match self {
            Denial::AuditChainBroken { seq, .. } => Some(*seq),
            _ => None,
        }
    }

    /// Render to a `delulu_diag::Diagnostic` (the human/JSON surface).
    pub fn to_diagnostic(&self) -> Diagnostic {
        match self {
            Denial::Attenuation { holder, requested, intersection } => {
                // DL0802: the exact repair is the intersection, which NEVER widens (spec §8). We
                // encode that guarantee as a typed repair (authority_widening: false, Exact). The
                // concrete edit needs a source span (wired in chunk 3); the narrowing target is the
                // intersection carried structurally on this `Denial`.
                let repair = Repair {
                    id: "R-DL0802-attenuate-to-intersection",
                    confidence: Confidence::Exact,
                    authority_widening: false,
                    requires_human: false,
                    edits: Vec::new(),
                };
                let _ = requested; // full requested authority is carried structurally on the Denial
                Diagnostic::error(
                    "DL0802",
                    format!(
                        "requested authority exceeds the holder's grant `{}` (attenuation \
                         violation); narrow to the intersection — {}",
                        holder.as_str(),
                        intersection.render_compact()
                    ),
                )
                .with_repair(repair)
            }
            Denial::Expired { node, ttl_millis, now_millis } => Diagnostic::error(
                "DL1402",
                format!(
                    "lease `{}` expired: ttl {ttl_millis} ms, now {now_millis} ms — re-delegate",
                    node.as_str()
                ),
            ),
            Denial::Revoked { node, by_seq } => Diagnostic::error(
                "DL1403",
                format!(
                    "lease `{}` was revoked by audit seq {by_seq} (why and when your authority died)",
                    node.as_str()
                ),
            ),
            Denial::OutOfScope { node, dimension, arg } => Diagnostic::error(
                "DL0904",
                format!(
                    "lease `{}` does not grant `{arg}` in scope `{dimension}` (capability scope violation)",
                    node.as_str()
                ),
            ),
            Denial::NotRevocable { caller, target } => Diagnostic::error(
                "DL0904",
                format!(
                    "lease `{}` cannot revoke `{}` — the target is neither the caller nor a \
                     descendant of it (no upward/lateral reach)",
                    caller.as_str(),
                    target.as_str()
                ),
            ),
            Denial::UnknownNode { node } => Diagnostic::error(
                "DL0904",
                format!("no such lease `{}` (fail closed)", node.as_str()),
            ),
            Denial::AuditChainBroken { seq, detail } => Diagnostic::error(
                "DL1405",
                format!(
                    "audit chain verification failed at seq {seq}: {detail} — the log may have been \
                     tampered with (observability, not enforcement; this detects, it does not prevent)"
                ),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority::Scopes;
    use crate::tree::GrantId;
    use delulu_check::Effect;
    use std::collections::BTreeSet;

    fn gid(s: &str) -> GrantId {
        GrantId(s.to_string())
    }
    fn eff(names: &[&str]) -> BTreeSet<Effect> {
        names.iter().map(|n| Effect::core_from_name(n).unwrap()).collect()
    }

    #[test]
    fn dl0802_repair_never_widens_and_is_exact() {
        let inter = Authority::new(eff(&["Read"]), Scopes::default());
        let d = Denial::Attenuation {
            holder: gid("g_root"),
            requested: Box::new(Authority::new(eff(&["Read", "Net"]), Scopes::default())),
            intersection: Box::new(inter.clone()),
        };
        assert_eq!(d.code(), "DL0802");
        assert!(!d.requires_human(), "DL0802 is mechanically resolvable");
        assert_eq!(d.intersection(), Some(&inter));
        let diag = d.to_diagnostic();
        assert_eq!(diag.code, "DL0802");
        let repair = diag.repairs.first().expect("DL0802 carries a repair");
        assert!(!repair.authority_widening, "the attenuation repair NEVER widens (spec §8)");
        assert_eq!(repair.confidence, delulu_diag::Confidence::Exact);
        assert!(!repair.requires_human);
    }

    #[test]
    fn dl1403_carries_the_revoking_seq_and_requires_human() {
        let d = Denial::Revoked { node: gid("g_leaf"), by_seq: 42 };
        assert_eq!(d.code(), "DL1403");
        assert_eq!(d.revoking_seq(), Some(42));
        assert!(d.requires_human());
        assert!(d.to_diagnostic().message.contains("42"), "message states the revoking seq");
    }

    #[test]
    fn dl1402_expiry_requires_human() {
        let d = Denial::Expired { node: gid("g_x"), ttl_millis: 5000, now_millis: 6000 };
        assert_eq!(d.code(), "DL1402");
        assert!(d.requires_human());
    }

    #[test]
    fn audit_chain_break_is_dl1405_and_requires_human() {
        let d = Denial::AuditChainBroken { seq: 7, detail: "record hash mismatch".into() };
        assert_eq!(d.code(), "DL1405");
        assert_eq!(d.audit_break_seq(), Some(7));
        assert!(d.requires_human(), "DL1405 requires a human (spec §8)");
        let diag = d.to_diagnostic();
        assert_eq!(diag.code, "DL1405");
        assert!(diag.message.contains('7'), "message states the failing seq");
    }

    #[test]
    fn scope_and_topology_denials_map_to_dl0904() {
        for d in [
            Denial::OutOfScope { node: gid("g_x"), dimension: "net", arg: "evil.com".into() },
            Denial::NotRevocable { caller: gid("g_a"), target: gid("g_b") },
            Denial::UnknownNode { node: gid("g_?") },
        ] {
            assert_eq!(d.code(), "DL0904");
            assert!(!d.requires_human());
            // Every denial maps to a registered code (the Diagnostic builder debug-asserts this).
            let _ = d.to_diagnostic();
        }
    }
}
