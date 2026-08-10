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
    /// DL0904 — a guard policy edit named a pattern that could never match, so the rule would gate
    /// nothing (campaign finding GUARD-SPELL-1). Refused rather than stored, because the failure
    /// mode is an operator who believes a seal is in place while the program writes the file anyway.
    DeadGuardPattern { label: String, why: String },
    /// DL0904 — an operation named a grant id that does not exist in the tree (fail-closed: an
    /// unknown lease confers no authority).
    UnknownNode { node: GrantId },
    /// DL1405 — the append-only audit chain failed verification at `seq` (a recomputed hash or a
    /// cross-record `prev_hash` linkage did not match). `requires_human: true` (spec §8): the log
    /// is observability, not enforcement — a break signals possible tampering, never a policy
    /// decision. `detail` states what mismatched (hash vs prev-link).
    AuditChainBroken { seq: u64, detail: String },
    /// DL1407 — a delegation lease token is invalid or already redeemed: a bad/garbled MAC, a MAC
    /// signed by a rotated-away key, a malformed token, an unknown bound node, or a second
    /// redemption of a single-use token. `requires_human: true` (spec §8) — re-mint via `delegate`.
    TokenInvalid { detail: String },
    // ----- RFC 0001 F2: grant certificates (federation). DL1401–DL1414 were taken. -------------
    /// DL1415 — a certificate chain does not verify to a configured trust anchor: an unknown
    /// issuer, a signature that does not verify, a signature by a key other than the named issuer,
    /// a chain that does not link, or a non-holder issuing onward. All one code because they are
    /// one question — *is this chain rooted in something we trust?* — and the `detail` says which.
    CertUntrusted { detail: String },
    /// DL1416 — a hop in a certificate chain is not `⊑` its parent. Carries the never-widening
    /// intersection, exactly as DL0802 does for a local delegation; `index` is the failing hop.
    CertAttenuation {
        index: usize,
        requested: Box<Authority>,
        intersection: Box<Authority>,
    },
    /// DL1417 — a certificate is outside its `not_before`/`not_after` window. Checked per hop: a
    /// chain is only as live as its shortest link.
    CertExpired { not_before: i64, not_after: i64, now_millis: i64 },
    /// DL1418 — this build cannot fully understand the certificate: an unimplemented signature
    /// algorithm, an unknown effect, or an unknown scope dimension. **Refused whole, never honored
    /// in part** — an authority dimension a verifier cannot see is one it cannot enforce, so
    /// ignoring it would silently widen the grant (RFC 0001 §4.9.3).
    CertUnsupported { detail: String },
    /// DL1418 — a certificate that is not well-formed at all. Shares the code with
    /// [`Denial::CertUnsupported`] because both mean "this build will not act on these bytes".
    CertMalformed { detail: String },
    /// DL1410 — the Guard: a delegated node used guarded authority with no permit. Carries the
    /// matched rule and (in the message) the exact `delulu guard request …` escalation command.
    GuardBlocked { node: GrantId, rule: String },
    /// DL1410 — the Guard at MINT time: a `Delegate`/`Attenuate` would hand guarded authority to a
    /// child with neither the owner code nor a covering permit. The message names BOTH ways forward
    /// (owner code for the principal; `delulu guard request` for an agent) — addendum §2.4.3.
    GuardMintBlocked { parent: GrantId, rule: String },
    /// DL1411 — the Guard: a request for this access is already pending. Carries the request id.
    GuardPending { node: GrantId, request_id: String },
    /// DL1412 — the Guard: the principal denied this access. Carries the principal's comment VERBATIM.
    GuardDenied { node: GrantId, comment: String },
    /// DL1413 — the Guard: the matched rule is `sealed` — not runtime-approvable; a principal policy
    /// edit (owner-coded) is the only path, and bypass does not lift it.
    GuardSealed { node: GrantId, rule: String },
    /// DL1414 — the Guard: an admin verb was refused because the owner code was missing or invalid.
    GuardOwner { detail: String },
    /// DL1421 — strict root-issuance mode (DISC-1): this broker refuses to create root authority from
    /// an unsigned request. A root may enter ONLY by adopting a certificate that verifies against the
    /// configured trust anchor. Carries the pinned anchor pubkey (hex) so the message can name the
    /// exact `grants adopt` path. Not `requires_human`: it is resolved mechanically, by adopting a cert.
    StrictRootRequiresAnchor { anchor: String },
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
            | Denial::DeadGuardPattern { .. }
            | Denial::UnknownNode { .. } => "DL0904",
            Denial::AuditChainBroken { .. } => "DL1405",
            Denial::TokenInvalid { .. } => "DL1407",
            Denial::CertUntrusted { .. } => "DL1415",
            Denial::CertAttenuation { .. } => "DL1416",
            Denial::CertExpired { .. } => "DL1417",
            Denial::CertUnsupported { .. } | Denial::CertMalformed { .. } => "DL1418",
            Denial::GuardBlocked { .. } | Denial::GuardMintBlocked { .. } => "DL1410",
            Denial::GuardPending { .. } => "DL1411",
            Denial::GuardDenied { .. } => "DL1412",
            Denial::GuardSealed { .. } => "DL1413",
            Denial::GuardOwner { .. } => "DL1414",
            Denial::StrictRootRequiresAnchor { .. } => "DL1421",
        }
    }

    /// Whether resolving this denial requires a human decision (spec §8: DL1402/DL1403/DL1405/DL1407
    /// are `requires_human: true`; the attenuation and scope denials are mechanically resolvable).
    pub fn requires_human(&self) -> bool {
        matches!(
            self,
            Denial::Expired { .. }
                | Denial::Revoked { .. }
                | Denial::AuditChainBroken { .. }
                | Denial::TokenInvalid { .. }
                // Every guard refusal needs a principal action (approve / unseal / provide the code).
                | Denial::GuardBlocked { .. }
                | Denial::GuardMintBlocked { .. }
                | Denial::GuardPending { .. }
                | Denial::GuardDenied { .. }
                | Denial::GuardSealed { .. }
                | Denial::GuardOwner { .. }
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
            Denial::DeadGuardPattern { label, why } => Diagnostic::error(
                "DL0904",
                format!(
                    "guard policy `{label}` was NOT set: {why} A rule that cannot match is worse than \
                     no rule, because it reports success while gating nothing (GUARD-SPELL-1)."
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
            Denial::CertUntrusted { detail } => Diagnostic::error(
                "DL1415",
                format!("grant certificate chain does not verify to a trust anchor: {detail}"),
            ),
            Denial::CertAttenuation { index, requested, intersection } => Diagnostic::error(
                "DL1416",
                format!(
                    "grant certificate {index} is not an attenuation of its issuer: requested \
                     `{}`, but the most that hop can carry is `{}`",
                    requested.render_compact(),
                    intersection.render_compact()
                ),
            ),
            Denial::CertExpired { not_before, not_after, now_millis } => Diagnostic::error(
                "DL1417",
                format!(
                    "grant certificate is outside its validity window [{not_before}, {not_after}) \
                     at {now_millis} ms — a chain is only as live as its shortest hop"
                ),
            ),
            Denial::CertUnsupported { detail } | Denial::CertMalformed { detail } => {
                Diagnostic::error("DL1418", format!("grant certificate refused: {detail}"))
            }
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
            Denial::TokenInvalid { detail } => Diagnostic::error(
                "DL1407",
                format!("delegation token invalid or already redeemed: {detail} — re-mint via `delegate`"),
            ),
            // The Guard (addendum §3.3). DL1410's message MUST contain the exact request command
            // (dcg-style remediation); DL1412's MUST carry the principal's comment verbatim.
            Denial::GuardBlocked { node, rule } => Diagnostic::error(
                "DL1410",
                format!(
                    "guard: delegated node `{}` needs approval to use `{rule}` (guarded, no permit) — \
                     request access: delulu guard request {} --use {rule} --why \"<why>\"",
                    node.as_str(),
                    node.as_str()
                ),
            ),
            Denial::GuardMintBlocked { parent, rule } => Diagnostic::error(
                "DL1410",
                format!(
                    "guard: minting guarded authority `{rule}` into a delegated child under `{}` is \
                     refused (no owner code, no covering permit) — the principal may mint directly \
                     with `--owner <code>`, or an agent can request it: delulu guard request {} \
                     --use {rule} --why \"<why>\"",
                    parent.as_str(),
                    parent.as_str()
                ),
            ),
            Denial::GuardPending { node, request_id } => Diagnostic::error(
                "DL1411",
                format!(
                    "guard: access for `{}` is pending principal approval (request `{request_id}`) — \
                     the principal decides with `delulu guard approve {request_id}` / `deny {request_id}`",
                    node.as_str()
                ),
            ),
            Denial::GuardDenied { node, comment } => Diagnostic::error(
                "DL1412",
                // The comment is the PRINCIPAL'S WORDS, carried verbatim (addendum §3.3).
                format!("guard: the principal denied this access for `{}` — {comment}", node.as_str()),
            ),
            Denial::GuardSealed { node, rule } => Diagnostic::error(
                "DL1413",
                format!(
                    "guard: `{rule}` is sealed on `{}` — sealed authority is not runtime-approvable; \
                     only a principal policy edit can unseal it (`delulu guard policy unset {rule} \
                     --owner <code>`, or set a weaker tier). Bypass does not lift a seal.",
                    node.as_str()
                ),
            ),
            Denial::StrictRootRequiresAnchor { anchor } => Diagnostic::error(
                "DL1421",
                format!(
                    "root issuance refused: this broker requires anchored roots (strict mode, DISC-1). \
                     An unsigned root cannot be created — a root may enter only by adopting a \
                     certificate chain that verifies against the configured trust anchor `{anchor}` \
                     (`delulu grants adopt <chain> --anchor {anchor}`). A same-OS-user process without \
                     the anchor private key cannot manufacture root authority this way; the anchor \
                     key's custody is the residual boundary (see ROOT_ISSUANCE_TRUST_BOUNDARY.md)."
                ),
            ),
            Denial::GuardOwner { detail } => Diagnostic::error(
                "DL1414",
                format!(
                    "guard admin verb refused ({detail}): a valid owner code is required — pass \
                     `--owner <code>` or set DELULU_GUARD_OWNER (the daemon prints it once at `broker \
                     start`; it rotates each run and is never written to disk)"
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
    fn token_invalid_is_dl1407_and_requires_human() {
        let d = Denial::TokenInvalid { detail: "MAC mismatch".into() };
        assert_eq!(d.code(), "DL1407");
        assert!(d.requires_human(), "DL1407 requires a human (spec §8)");
        assert_eq!(d.to_diagnostic().code, "DL1407");
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
