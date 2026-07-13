//! Stage 5 chunk 6 (phases 5k–5m) — **the Guard**: a dcg-inspired principal-approval layer.
//!
//! The guard supervises what *delegated (non-root) grants* may do with the classes where the type
//! system's guarantees end or a mistake is catastrophic (declassification, native code). It is part
//! of the authority boundary — **fail closed** (invariant 27), never dcg's fail-open ergonomic hook.
//!
//! **Holder-neutral by construction (criterion 9 / addendum ruling 2).** Enforcement keys on TREE
//! POSITION, never holder identity: root nodes (no parent) are principal-held by construction and
//! pass without guard interaction; delegated nodes (any parent) are agent-held by construction and
//! are gated. No `holder.kind` is ever read here.
//!
//! **Model (addendum §2.3).** A policy is a set of rules `class:pattern → tier`:
//! - `warn` — the use proceeds; a one-line note reaches the agent and a `guard_warn` event is audited.
//! - `guarded` — refused with **DL1410** unless a matching permit exists (approvable at runtime, §2.5).
//! - `sealed` — refused with **DL1413** always; not runtime-approvable; **bypass does not lift it**.
//!
//! Permits/pending requests/the owner code all live in daemon memory only (a restart clears them —
//! approvals are deliberately session-scoped). Persistence is the policy alone, at a path the CLI
//! injects; the library never chooses a path (ruling 2).

use std::collections::BTreeSet;

use serde_json::{json, Value};

use crate::authority::Authority;
use crate::diag::Denial;
use crate::tree::{Broker, GrantId};
use crate::validate::Op;

// ================================================================================================
// Policy vocabulary
// ================================================================================================

/// The three graduated response tiers (addendum §2.3), simplified from dcg's WARN/SOFT/HARD. Ordered
/// so `Sealed > Guarded > Warn`: the STRONGEST matching rule wins (fail-safe).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GuardTier {
    Warn,
    Guarded,
    Sealed,
}

impl GuardTier {
    pub fn wire_name(self) -> &'static str {
        match self {
            GuardTier::Warn => "warn",
            GuardTier::Guarded => "guarded",
            GuardTier::Sealed => "sealed",
        }
    }
    pub fn from_wire(s: &str) -> Option<GuardTier> {
        Some(match s {
            "warn" => GuardTier::Warn,
            "guarded" => GuardTier::Guarded,
            "sealed" => GuardTier::Sealed,
            _ => return None,
        })
    }
}

/// The eight guard rule classes (addendum §2.3) — one per authority axis the broker already knows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GuardClass {
    Effect,
    FsRead,
    FsWrite,
    Net,
    Secret,
    Declassify,
    ForeignC,
    ForeignPython,
}

impl GuardClass {
    pub fn wire_name(self) -> &'static str {
        match self {
            GuardClass::Effect => "effect",
            GuardClass::FsRead => "fs_read",
            GuardClass::FsWrite => "fs_write",
            GuardClass::Net => "net",
            GuardClass::Secret => "secret",
            GuardClass::Declassify => "declassify",
            GuardClass::ForeignC => "foreign_c",
            GuardClass::ForeignPython => "foreign_python",
        }
    }
    pub fn from_wire(s: &str) -> Option<GuardClass> {
        Some(match s {
            "effect" => GuardClass::Effect,
            "fs_read" => GuardClass::FsRead,
            "fs_write" => GuardClass::FsWrite,
            "net" => GuardClass::Net,
            "secret" => GuardClass::Secret,
            "declassify" => GuardClass::Declassify,
            "foreign_c" => GuardClass::ForeignC,
            "foreign_python" => GuardClass::ForeignPython,
            _ => return None,
        })
    }
}

/// The use-time axis class an op is gated on (`None` for classes with no per-use broker op — e.g.
/// `secret`/`foreign_python` gate only at MINT time in v0.5; their use never crosses the broker).
fn use_axis_class(op: Op) -> Option<GuardClass> {
    match op {
        Op::FsRead => Some(GuardClass::FsRead),
        Op::FsWrite => Some(GuardClass::FsWrite),
        Op::Net => Some(GuardClass::Net),
        Op::Declassify => Some(GuardClass::Declassify),
        Op::ForeignBind => Some(GuardClass::ForeignC),
        _ => None,
    }
}

/// Pattern match against a use token (addendum §2.3 vocabulary). v0.5 (deviation §7): `*` matches
/// everything; otherwise EXACT string equality — the conservative, sound choice consistent with the
/// attenuation lattice's exact-string discipline (authority.rs ruling 4: no pattern implication).
fn pattern_matches(pattern: &str, token: Option<&str>) -> bool {
    pattern == "*" || token == Some(pattern)
}

fn rule_label(class: GuardClass, pattern: &str) -> String {
    format!("{}:{}", class.wire_name(), pattern)
}

// ================================================================================================
// Policy
// ================================================================================================

/// One guard rule: `class:pattern → tier`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardRule {
    pub class: GuardClass,
    pub pattern: String,
    pub tier: GuardTier,
}

/// The broker-held guard policy: an ordered set of rules (addendum §2.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardPolicy {
    rules: Vec<GuardRule>,
}

impl GuardPolicy {
    /// The zero-config default (addendum §2.3): the classes where the type system's guarantees end
    /// or a mistake is catastrophic — secret material leaving custody, and native code.
    pub fn default_policy() -> GuardPolicy {
        let g = GuardTier::Guarded;
        GuardPolicy {
            rules: vec![
                GuardRule { class: GuardClass::Declassify, pattern: "*".into(), tier: g },
                GuardRule { class: GuardClass::ForeignC, pattern: "*".into(), tier: g },
                GuardRule { class: GuardClass::ForeignPython, pattern: "*".into(), tier: g },
            ],
        }
    }

    pub fn rules(&self) -> &[GuardRule] {
        &self.rules
    }

    /// Set (add or replace) the rule for `class:pattern`. Returns whether it replaced an existing one.
    pub fn set(&mut self, class: GuardClass, pattern: String, tier: GuardTier) -> bool {
        if let Some(r) = self.rules.iter_mut().find(|r| r.class == class && r.pattern == pattern) {
            r.tier = tier;
            true
        } else {
            self.rules.push(GuardRule { class, pattern, tier });
            self.rules.sort_by(|a, b| (a.class, a.pattern.as_str()).cmp(&(b.class, b.pattern.as_str())));
            false
        }
    }

    /// Remove the rule for `class:pattern`. Returns whether one was removed.
    pub fn unset(&mut self, class: GuardClass, pattern: &str) -> bool {
        let before = self.rules.len();
        self.rules.retain(|r| !(r.class == class && r.pattern == pattern));
        self.rules.len() != before
    }

    /// The strongest tier gating a use-time op+arg, with the matched rule's `class:pattern` label —
    /// or `None` when ungated. Considers both the op's axis class and the cross-cutting `effect`
    /// class (addendum §2.3).
    pub fn tier_for_use(&self, op: Op, arg: Option<&str>) -> Option<(GuardTier, String)> {
        let mut best: Option<(GuardTier, String)> = None;
        let mut consider = |tier: GuardTier, label: String| {
            if best.as_ref().is_none_or(|(t, _)| tier > *t) {
                best = Some((tier, label));
            }
        };
        if let Some(class) = use_axis_class(op) {
            for r in self.rules.iter().filter(|r| r.class == class) {
                if pattern_matches(&r.pattern, arg) {
                    consider(r.tier, rule_label(r.class, &r.pattern));
                }
            }
        }
        if let Some(effect) = op.required_effect() {
            let effname = effect.name();
            for r in self.rules.iter().filter(|r| r.class == GuardClass::Effect) {
                if pattern_matches(&r.pattern, Some(effname)) {
                    consider(r.tier, rule_label(r.class, &r.pattern));
                }
            }
        }
        best
    }

    /// The strongest tier gating a MINT whose child authority is `auth`, with the matched rule label
    /// (addendum §2.4.3) — or `None` when the child requests nothing guarded. A mint gates only if
    /// the child actually REQUESTS items in a guarded dimension.
    pub fn tier_for_mint(&self, auth: &Authority) -> Option<(GuardTier, String)> {
        let mut best: Option<(GuardTier, String)> = None;
        let mut consider = |tier: GuardTier, label: String| {
            if best.as_ref().is_none_or(|(t, _)| tier > *t) {
                best = Some((tier, label));
            }
        };
        for e in &auth.effects {
            for r in self.rules.iter().filter(|r| r.class == GuardClass::Effect) {
                if pattern_matches(&r.pattern, Some(e.name())) {
                    consider(r.tier, rule_label(r.class, &r.pattern));
                }
            }
        }
        let s = &auth.scopes;
        let dims: [(GuardClass, &BTreeSet<String>); 7] = [
            (GuardClass::FsRead, &s.fs_read),
            (GuardClass::FsWrite, &s.fs_write),
            (GuardClass::Net, &s.net),
            (GuardClass::Secret, &s.secrets),
            (GuardClass::Declassify, &s.declassify),
            (GuardClass::ForeignC, &s.foreign_c),
            (GuardClass::ForeignPython, &s.foreign_python),
        ];
        for (class, set) in dims {
            for item in set {
                for r in self.rules.iter().filter(|r| r.class == class) {
                    if pattern_matches(&r.pattern, Some(item)) {
                        consider(r.tier, rule_label(r.class, &r.pattern));
                    }
                }
            }
        }
        best
    }

    /// Class wire-names that have ANY guarded/sealed rule — the set a client uses to decide which
    /// epoch-class ops must round-trip synchronously (addendum §2.4.2 / criterion 11). Sorted, deduped.
    pub fn guarded_classes(&self) -> Vec<String> {
        let mut set: BTreeSet<&'static str> = BTreeSet::new();
        for r in &self.rules {
            if matches!(r.tier, GuardTier::Guarded | GuardTier::Sealed) {
                set.insert(r.class.wire_name());
            }
        }
        set.into_iter().map(str::to_string).collect()
    }

    /// Canonical JSON for persistence (the daemon writes this beside its state; edits audited).
    pub fn to_json(&self) -> Value {
        let rules: Vec<Value> = self
            .rules
            .iter()
            .map(|r| json!({ "class": r.class.wire_name(), "pattern": r.pattern, "tier": r.tier.wire_name() }))
            .collect();
        json!({ "version": 1, "rules": rules })
    }

    /// Parse a policy from its persisted JSON. `None` on any structural error (the caller treats a
    /// corrupt store as **poisoned** — fail closed, addendum §2.4 / criterion 10).
    pub fn from_json(v: &Value) -> Option<GuardPolicy> {
        let arr = v.get("rules")?.as_array()?;
        let mut rules = Vec::new();
        for item in arr {
            let class = GuardClass::from_wire(item.get("class")?.as_str()?)?;
            let pattern = item.get("pattern")?.as_str()?.to_string();
            let tier = GuardTier::from_wire(item.get("tier")?.as_str()?)?;
            rules.push(GuardRule { class, pattern, tier });
        }
        Some(GuardPolicy { rules })
    }
}

// ================================================================================================
// Permits, requests, and the in-memory guard state
// ================================================================================================

/// An approved subset of guard classes/patterns (addendum §2.5). A permit/request carries one; a
/// use is "covered" iff some entry matches the use's axis class or effect class.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct GuardSubset(pub Vec<(GuardClass, String)>);

impl GuardSubset {
    /// Parse `class:pattern` items (the `--use` request vocabulary). `None` on any malformed item.
    pub fn parse(items: &[String]) -> Option<GuardSubset> {
        let mut out = Vec::new();
        for it in items {
            let (c, p) = it.split_once(':')?;
            out.push((GuardClass::from_wire(c)?, p.to_string()));
        }
        Some(GuardSubset(out))
    }

    pub fn labels(&self) -> Vec<String> {
        self.0.iter().map(|(c, p)| rule_label(*c, p)).collect()
    }

    /// Does this subset cover a use-time op+arg (addendum §2.5 — the use must be within the approved
    /// subset, on the permit's node; deviation §7: subset-cover rather than a literal `⊑` because
    /// `*` is not representable in the exact-string scope lattice)?
    fn covers_use(&self, op: Op, arg: Option<&str>) -> bool {
        if let Some(class) = use_axis_class(op) {
            if self.0.iter().any(|(c, p)| *c == class && pattern_matches(p, arg)) {
                return true;
            }
        }
        if let Some(effect) = op.required_effect() {
            let effname = effect.name();
            if self.0.iter().any(|(c, p)| *c == GuardClass::Effect && pattern_matches(p, Some(effname))) {
                return true;
            }
        }
        false
    }
}

/// A broker-held permit (addendum §2.5): NEVER a bearer token. The agent simply retries and the
/// broker honors the permit. Daemon-memory only.
#[derive(Clone, Debug)]
pub struct Permit {
    pub id: String,
    pub node: GrantId,
    pub subset: GuardSubset,
    /// Absolute epoch-millis expiry deadline (`None` = never within the session, but a daemon
    /// restart clears all permits regardless).
    pub expires_millis: Option<i64>,
    /// Remaining uses (`None` = unlimited within scope). Decremented on each honored use.
    pub remaining_uses: Option<u64>,
}

/// The lifecycle of a pending guard request (addendum §2.5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReqStatus {
    Pending,
    Approved,
    Denied { comment: String },
}

/// A pending/decided guard request `{id, node, requested subset, why, created}` (addendum §2.5).
#[derive(Clone, Debug)]
pub struct GuardRequest {
    pub id: String,
    pub node: GrantId,
    pub subset: GuardSubset,
    pub why: String,
    pub created_millis: i64,
    pub status: ReqStatus,
}

/// Generate a fresh guard owner code: `gow1_` + 96 bits of OS randomness in hex (addendum §2.2).
/// Daemon-memory only, printed once at `broker start`, rotates each run. The `gow1_` prefix reuses
/// the token-randomness machinery discipline (getrandom, ruling 3).
pub fn generate_owner_code() -> String {
    use std::fmt::Write as _;
    let mut b = [0u8; 12];
    getrandom::fill(&mut b).expect("OS randomness (getrandom) unavailable");
    let mut s = String::from("gow1_");
    for x in b {
        let _ = write!(s, "{x:02x}");
    }
    s
}

/// Pending request time-to-live: 30 min (addendum §2.5).
pub const REQUEST_TTL_MILLIS: i64 = 30 * 60 * 1000;
/// Default permit TTL: 15 min (addendum ruling 3).
pub const DEFAULT_PERMIT_TTL_MILLIS: i64 = 15 * 60 * 1000;

/// The daemon-memory guard state owned by every [`Broker`].
pub struct GuardState {
    policy: GuardPolicy,
    /// Fail closed (criterion 10): a corrupt/unreadable policy store poisons the guard — guarded
    /// classes refuse and permits are NOT consulted (an untrustworthy store extends no trust).
    poisoned: bool,
    bypass: bool,
    owner_code: Option<String>,
    permits: Vec<Permit>,
    requests: Vec<GuardRequest>,
    // The id counters are consumed by the approval flow (phase 5l: request/approve mint ids).
    #[allow(dead_code)]
    next_permit: u64,
    #[allow(dead_code)]
    next_request: u64,
}

impl Default for GuardState {
    fn default() -> Self {
        GuardState::new()
    }
}

impl GuardState {
    pub fn new() -> GuardState {
        GuardState {
            policy: GuardPolicy::default_policy(),
            poisoned: false,
            bypass: false,
            owner_code: None,
            permits: Vec::new(),
            requests: Vec::new(),
            next_permit: 0,
            next_request: 0,
        }
    }

    fn owner_ok(&self, provided: Option<&str>) -> bool {
        // Same-user threat model (spec §10) — a plain equality check is sufficient; the owner code
        // raises the bar only while the principal keeps it out of agent-visible context.
        matches!((&self.owner_code, provided), (Some(c), Some(p)) if c == p)
    }
}

// ================================================================================================
// The verdict a guard use-gate produces (recording happens in `validate::check_use`)
// ================================================================================================

/// The guard verdict for an otherwise-authorized use (addendum §2.4.1). `validate::check_use` maps
/// each arm to its audit action + decision; `Ungated` means the normal `"use"` path.
pub enum GuardVerdict {
    Ungated,
    /// Allowed via a matching permit → action `guard_permit_use`.
    PermitUse,
    /// Allowed under bypass → action `guard_bypassed_use`, agent-side warn note.
    BypassedUse { note: String },
    /// `warn` tier → action `guard_warn`, agent-side warn note.
    Warn { note: String },
    /// Refused (guarded no-permit / pending / denied / sealed) → action `guard_block`.
    Block(Denial),
}

// ================================================================================================
// Broker guard methods (the recording + orchestration layer)
// ================================================================================================

impl Broker {
    /// Inject a persisted guard policy + poison flag at daemon start (addendum §2.3). Chainable.
    pub fn with_guard_policy(mut self, policy: GuardPolicy, poisoned: bool) -> Broker {
        self.guard.policy = policy;
        self.guard.poisoned = poisoned;
        self
    }

    /// Inject the print-once owner code at daemon start (addendum §2.2). Chainable. Kept only in
    /// daemon memory — the caller never persists it.
    pub fn with_owner_code(mut self, code: impl Into<String>) -> Broker {
        self.guard.owner_code = Some(code.into());
        self
    }

    /// Enable bypass at daemon start (`--dangerously-bypass-guard`, addendum §2.6). Chainable.
    pub fn with_bypass(mut self, on: bool) -> Broker {
        self.guard.bypass = on;
        self
    }

    pub fn guard_bypass(&self) -> bool {
        self.guard.bypass
    }
    pub fn guard_poisoned(&self) -> bool {
        self.guard.poisoned
    }
    pub fn guard_rules(&self) -> &[GuardRule] {
        self.guard.policy.rules()
    }
    /// Guarded/sealed class wire-names (client epoch-routing set, criterion 11).
    pub fn guard_guarded_classes(&self) -> Vec<String> {
        self.guard.policy.guarded_classes()
    }
    pub fn guard_pending_count(&self) -> usize {
        self.guard.requests.iter().filter(|r| r.status == ReqStatus::Pending).count()
    }
    pub fn guard_permit_count(&self) -> usize {
        self.guard.permits.len()
    }

    /// Whether `node` is guard-gated (delegated / non-root). Unknown nodes are treated as gated for
    /// safety, but the normal check reports the unknown-node denial first (fail closed).
    fn guard_node_is_gated(&self, node: &GrantId) -> bool {
        self.inspect(node).map(|n| n.parent.is_some()).unwrap_or(true)
    }

    /// Purge expired permits/requests against the clock (called before any guard evaluation so an
    /// expired permit never honors a use — invariant 27).
    fn guard_gc(&mut self) {
        let now = self.effective_now();
        self.guard.permits.retain(|p| p.expires_millis.map(|e| now < e).unwrap_or(true));
        self.guard.requests.retain(|r| match r.status {
            // Pending requests time out (addendum §2.5); decided ones persist so a retried use still
            // sees the DL1412 comment / DL1411 id until the session ends.
            ReqStatus::Pending => now < r.created_millis + REQUEST_TTL_MILLIS,
            _ => true,
        });
    }

    /// The guard verdict for an otherwise-authorized use (addendum §2.4.1). Pure of audit — the
    /// caller (`validate::check_use`) records the mapped event. Consults the EFFECTIVE policy
    /// (default when poisoned) and, for guarded, permits → pending → denied → DL1410.
    pub(crate) fn guard_verdict_use(&mut self, node: &GrantId, op: Op, arg: Option<&str>) -> GuardVerdict {
        if !self.guard_node_is_gated(node) {
            return GuardVerdict::Ungated; // root (principal-held by construction) — never gated
        }
        let Some((tier, rule)) = self.guard.policy.tier_for_use(op, arg) else {
            return GuardVerdict::Ungated;
        };
        match tier {
            GuardTier::Warn => GuardVerdict::Warn {
                note: format!("guard warn: `{rule}` used by delegated node `{node}` (audited)"),
            },
            GuardTier::Sealed => GuardVerdict::Block(Denial::GuardSealed { node: node.clone(), rule }),
            GuardTier::Guarded => {
                // Bypass lifts guarded to audited-and-warned; it NEVER lifts sealed (checked above).
                if self.guard.bypass {
                    return GuardVerdict::BypassedUse {
                        note: format!(
                            "guard BYPASSED: `{rule}` proceeding for delegated node `{node}` — audited (guard_bypassed_use)"
                        ),
                    };
                }
                // Fail closed (criterion 10): a poisoned store extends no trust — guarded refuses and
                // neither permits nor pending/denied requests are consulted.
                if self.guard.poisoned {
                    return GuardVerdict::Block(Denial::GuardBlocked { node: node.clone(), rule });
                }
                self.guard_gc();
                // A matching permit honors the use (and decrements a bounded use count).
                if let Some(idx) = self
                    .guard
                    .permits
                    .iter()
                    .position(|p| p.node == *node && p.subset.covers_use(op, arg))
                {
                    if let Some(rem) = self.guard.permits[idx].remaining_uses.as_mut() {
                        *rem = rem.saturating_sub(1);
                    }
                    let exhausted = matches!(self.guard.permits[idx].remaining_uses, Some(0));
                    if exhausted {
                        self.guard.permits.remove(idx);
                    }
                    return GuardVerdict::PermitUse;
                }
                // No permit: a pending request → DL1411; a denial → DL1412 (comment verbatim); else DL1410.
                if let Some(req) =
                    self.guard.requests.iter().find(|r| r.node == *node && r.subset.covers_use(op, arg))
                {
                    return match &req.status {
                        ReqStatus::Pending => GuardVerdict::Block(Denial::GuardPending {
                            node: node.clone(),
                            request_id: req.id.clone(),
                        }),
                        ReqStatus::Denied { comment } => GuardVerdict::Block(Denial::GuardDenied {
                            node: node.clone(),
                            comment: comment.clone(),
                        }),
                        // An approved-but-consumed request falls through to a fresh DL1410.
                        ReqStatus::Approved => GuardVerdict::Block(Denial::GuardBlocked {
                            node: node.clone(),
                            rule,
                        }),
                    };
                }
                GuardVerdict::Block(Denial::GuardBlocked { node: node.clone(), rule })
            }
        }
    }

    /// Guard gate for a MINT (addendum §2.4.3). A child authority that INCLUDES guarded/sealed items
    /// is refused unless the request carries a valid owner code (the principal minting directly) or a
    /// permit on `parent` covers the guarded subset — an agent can never quietly re-delegate guarded
    /// authority (a parent's own USE permit does not cover minting; the mint needs its own approval).
    /// Records one `guard_block` on refusal (consuming a seq); records nothing on pass (`delegate`
    /// records its own event).
    pub fn guard_check_mint(
        &mut self,
        child_authority: &Authority,
        parent: &GrantId,
        owner: Option<&str>,
    ) -> Result<(), Denial> {
        let Some((tier, rule)) = self.guard.policy.tier_for_mint(child_authority) else {
            return Ok(()); // nothing guarded in the child spec
        };
        // The principal minting directly (owner code) always may (sealed included — a policy edit is
        // the same principal act). This is the ONLY way to hand a seal down deliberately.
        if self.guard.owner_ok(owner) {
            return Ok(());
        }
        if tier == GuardTier::Sealed {
            let seq = self.consume_seq();
            let denial = Denial::GuardSealed { node: parent.clone(), rule };
            self.record_op(seq, "guard_block", Some(parent.as_str().to_string()), None, None, "deny", None);
            return Err(denial);
        }
        // A permit on the parent covering the minted subset authorizes the mint (an approved slice).
        // A poisoned store extends no trust (criterion 10) — skip permits, refuse.
        if !self.guard.poisoned {
            self.guard_gc();
            let covered = self
                .guard
                .permits
                .iter()
                .any(|p| p.node == *parent && mint_within_subset(&p.subset, child_authority));
            if covered {
                return Ok(());
            }
        }
        let seq = self.consume_seq();
        self.record_op(seq, "guard_block", Some(parent.as_str().to_string()), None, None, "deny", None);
        Err(Denial::GuardBlocked { node: parent.clone(), rule })
    }

    // ----- owner-gated policy administration (addendum §2.3) -------------------------------------

    /// `guard policy set class:pattern tier` (owner-gated). Bumps the epoch (addendum ruling 4) and
    /// records `guard_policy_edit`. Returns the edited policy for the daemon to persist.
    pub fn guard_policy_set(
        &mut self,
        owner: Option<&str>,
        class: GuardClass,
        pattern: String,
        tier: GuardTier,
    ) -> Result<(), Denial> {
        if !self.guard.owner_ok(owner) {
            return Err(Denial::GuardOwner { detail: "policy set".into() });
        }
        let label = rule_label(class, &pattern);
        self.guard.policy.set(class, pattern, tier);
        self.bump_epoch();
        let seq = self.consume_seq();
        self.record_op(
            seq,
            "guard_policy_edit",
            None,
            Some(format!("set {label} {}", tier.wire_name())),
            None,
            "allow",
            None,
        );
        Ok(())
    }

    /// `guard policy unset class:pattern` (owner-gated). Bumps the epoch and records the edit.
    pub fn guard_policy_unset(
        &mut self,
        owner: Option<&str>,
        class: GuardClass,
        pattern: &str,
    ) -> Result<bool, Denial> {
        if !self.guard.owner_ok(owner) {
            return Err(Denial::GuardOwner { detail: "policy unset".into() });
        }
        let removed = self.guard.policy.unset(class, pattern);
        self.bump_epoch();
        let seq = self.consume_seq();
        self.record_op(
            seq,
            "guard_policy_edit",
            None,
            Some(format!("unset {}", rule_label(class, pattern))),
            None,
            if removed { "allow" } else { "deny" },
            None,
        );
        Ok(removed)
    }

    /// `guard bypass on|off` (owner-gated, addendum §2.6). Bumps the epoch and records
    /// `guard_bypass_on`/`guard_bypass_off`.
    pub fn guard_set_bypass(&mut self, owner: Option<&str>, on: bool) -> Result<(), Denial> {
        if !self.guard.owner_ok(owner) {
            return Err(Denial::GuardOwner { detail: "bypass toggle".into() });
        }
        self.guard.bypass = on;
        self.bump_epoch();
        let seq = self.consume_seq();
        self.record_op(seq, if on { "guard_bypass_on" } else { "guard_bypass_off" }, None, None, None, "allow", None);
        Ok(())
    }

    /// The current policy (for persistence after an edit).
    pub fn guard_policy_snapshot(&self) -> GuardPolicy {
        self.guard.policy.clone()
    }
}

/// Does a permit subset cover EVERY guarded item in a mint's child authority (addendum §2.4.3)?
fn mint_within_subset(subset: &GuardSubset, auth: &Authority) -> bool {
    // Every dimension item that the mint requests must be covered by some subset entry of the same
    // class (or an `effect`-class entry for effects). Only guarded-eligible items need coverage, but
    // requiring coverage of ALL requested items is the conservative (never-widening) choice.
    let covered_in = |class: GuardClass, item: &str| {
        subset.0.iter().any(|(c, p)| *c == class && pattern_matches(p, Some(item)))
    };
    let s = &auth.scopes;
    let dims: [(GuardClass, &BTreeSet<String>); 7] = [
        (GuardClass::FsRead, &s.fs_read),
        (GuardClass::FsWrite, &s.fs_write),
        (GuardClass::Net, &s.net),
        (GuardClass::Secret, &s.secrets),
        (GuardClass::Declassify, &s.declassify),
        (GuardClass::ForeignC, &s.foreign_c),
        (GuardClass::ForeignPython, &s.foreign_python),
    ];
    for (class, set) in dims {
        for item in set {
            if !covered_in(class, item) {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority::Scopes;
    use crate::ids::SeqIdSource;
    use crate::time::ManualClock;
    use crate::tree::Holder;
    use delulu_check::Effect;
    use std::rc::Rc;

    fn eff(names: &[&str]) -> BTreeSet<Effect> {
        names.iter().map(|n| Effect::core_from_name(n).unwrap()).collect()
    }
    fn names(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }
    fn holder() -> Holder {
        Holder::new("process", "t", "pid:1")
    }
    fn broker() -> Broker {
        Broker::with_sources(Box::new(SeqIdSource::new()), Box::new(Rc::new(ManualClock::new(1000))))
            .with_owner_code("gow1_testowner")
    }

    #[test]
    fn default_policy_guards_the_three_dangerous_classes() {
        let p = GuardPolicy::default_policy();
        assert!(matches!(p.tier_for_use(Op::Declassify, Some("API_KEY")), Some((GuardTier::Guarded, _))));
        assert!(matches!(p.tier_for_use(Op::ForeignBind, Some("mathlib")), Some((GuardTier::Guarded, _))));
        // Ungated by default.
        assert!(p.tier_for_use(Op::FsRead, Some("./data")).is_none());
        assert!(p.tier_for_use(Op::FsWrite, Some("./out")).is_none());
        assert_eq!(p.guarded_classes(), vec!["declassify", "foreign_c", "foreign_python"]);
    }

    #[test]
    fn policy_json_roundtrips() {
        let mut p = GuardPolicy::default_policy();
        p.set(GuardClass::FsWrite, "*".into(), GuardTier::Sealed);
        let v = p.to_json();
        let back = GuardPolicy::from_json(&v).unwrap();
        assert_eq!(back, p);
        // A structurally broken store fails to parse (→ poisoned at the daemon).
        assert!(GuardPolicy::from_json(&json!({ "rules": [ { "class": "bogus", "pattern": "*", "tier": "guarded" } ] })).is_none());
    }

    #[test]
    fn root_use_of_guarded_authority_passes_without_interaction() {
        // Criterion 2: a ROOT node using declassify passes with no guard interaction.
        let mut b = broker();
        let root = b.issue(
            holder(),
            Authority::new(eff(&["Declassify"]), Scopes { declassify: names(&["API_KEY"]), ..Default::default() }),
            None,
        );
        assert!(matches!(b.guard_verdict_use(&root, Op::Declassify, Some("API_KEY")), GuardVerdict::Ungated));
    }

    #[test]
    fn delegated_guarded_use_without_permit_is_dl1410() {
        // Criterion 1: a delegated node using guarded authority without a permit is DL1410.
        let mut b = broker();
        let root = b.issue(
            holder(),
            Authority::new(eff(&["Declassify"]), Scopes { declassify: names(&["API_KEY"]), ..Default::default() }),
            None,
        );
        let child = b
            .attenuate(&root, Authority::new(eff(&["Declassify"]), Scopes { declassify: names(&["API_KEY"]), ..Default::default() }), holder(), None)
            .unwrap();
        match b.guard_verdict_use(&child, Op::Declassify, Some("API_KEY")) {
            GuardVerdict::Block(d) => {
                assert_eq!(d.code(), "DL1410");
                let msg = d.to_diagnostic().message;
                assert!(msg.contains("delulu guard request"), "names the request command: {msg}");
                assert!(msg.contains(child.as_str()) && msg.contains("declassify:*"), "{msg}");
            }
            _ => panic!("a delegated guarded use must be blocked"),
        }
    }

    #[test]
    fn sealed_use_is_dl1413_even_under_bypass() {
        // Criterion 6 (unit form): sealed refuses even under bypass.
        let mut b = broker().with_bypass(true);
        b.guard_policy_set(Some("gow1_testowner"), GuardClass::FsWrite, "*".into(), GuardTier::Sealed).unwrap();
        let root = b.issue(holder(), Authority::new(eff(&["Write"]), Scopes { fs_write: names(&["./out"]), ..Default::default() }), None);
        let child = b
            .attenuate(&root, Authority::new(eff(&["Write"]), Scopes { fs_write: names(&["./out"]), ..Default::default() }), holder(), None)
            .unwrap();
        match b.guard_verdict_use(&child, Op::FsWrite, Some("./out/x")) {
            GuardVerdict::Block(d) => assert_eq!(d.code(), "DL1413"),
            _ => panic!("sealed must refuse even under bypass"),
        }
    }

    #[test]
    fn owner_gated_admin_verbs_refuse_dl1414_without_the_code() {
        // Criterion 9 (unit form): admin verbs without a valid owner code refuse DL1414.
        let mut b = broker();
        assert_eq!(
            b.guard_policy_set(Some("wrong"), GuardClass::Net, "*".into(), GuardTier::Guarded).unwrap_err().code(),
            "DL1414"
        );
        assert_eq!(b.guard_set_bypass(None, true).unwrap_err().code(), "DL1414");
        // With the right code it succeeds and bumps the epoch (addendum ruling 4).
        let e0 = b.epoch();
        b.guard_policy_set(Some("gow1_testowner"), GuardClass::Net, "*".into(), GuardTier::Guarded).unwrap();
        assert_eq!(b.epoch(), e0 + 1, "a policy edit bumps the epoch");
    }

    #[test]
    fn mint_of_guarded_authority_needs_owner_code() {
        // Criterion 3: Delegate/Attenuate cannot mint guarded authority without a permit or owner code.
        let mut b = broker();
        let root = b.issue(
            holder(),
            Authority::new(eff(&["Declassify"]), Scopes { declassify: names(&["API_KEY"]), ..Default::default() }),
            None,
        );
        let child_auth = Authority::new(eff(&["Declassify"]), Scopes { declassify: names(&["API_KEY"]), ..Default::default() });
        // No owner code → blocked (DL1410, minting context = the parent node).
        let d = b.guard_check_mint(&child_auth, &root, None).unwrap_err();
        assert_eq!(d.code(), "DL1410");
        // With the owner code → the principal may mint directly.
        assert!(b.guard_check_mint(&child_auth, &root, Some("gow1_testowner")).is_ok());
    }

    #[test]
    fn poisoned_policy_refuses_guarded_but_not_ungated() {
        // Criterion 10 (unit form): a poisoned store refuses guarded classes; ungated unaffected.
        let mut b = broker().with_guard_policy(GuardPolicy::default_policy(), true);
        let root = b.issue(
            holder(),
            Authority::new(eff(&["Declassify", "Write"]), Scopes { declassify: names(&["K"]), fs_write: names(&["./out"]), ..Default::default() }),
            None,
        );
        let child = b
            .attenuate(&root, Authority::new(eff(&["Declassify", "Write"]), Scopes { declassify: names(&["K"]), fs_write: names(&["./out"]), ..Default::default() }), holder(), None)
            .unwrap();
        assert!(matches!(b.guard_verdict_use(&child, Op::Declassify, Some("K")), GuardVerdict::Block(_)), "guarded refuses when poisoned");
        assert!(matches!(b.guard_verdict_use(&child, Op::FsWrite, Some("./out/x")), GuardVerdict::Ungated), "ungated unaffected");
        assert!(b.guard_poisoned());
    }
}
