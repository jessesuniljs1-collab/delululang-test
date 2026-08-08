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

use delulu_check::Effect;

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
    /// A physical device (RFC 0001 F1 / Stage 10 Track D). Added by campaign C31: `device` was the
    /// only scope dimension with no guard class, so actuation could be gated all-or-nothing through
    /// `effect:Actuate` but never per device — while every other axis supported per-item rules.
    Device,
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
            GuardClass::Device => "device",
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
            "device" => GuardClass::Device,
            _ => return None,
        })
    }
}

/// The use-time axis class an op is gated on. `None` means the axis has no per-use broker op —
/// `secret` and `foreign_python` gate at MINT time only, because their use never crosses the broker.
///
/// **Exhaustive on purpose (campaign C31).** This match used to end in `_ => None`, and that catch-all
/// is how `Op::Actuate` — commanding a physical device — ended up with no axis of its own: the variant
/// was added, the arm moved, and the compiler had nothing to say. Ops that genuinely have no scope
/// dimension are listed explicitly, so adding a new `Op` breaks this build and forces the question
/// "what gates it?" to be answered by a person rather than by a wildcard.
fn use_axis_class(op: Op) -> Option<GuardClass> {
    match op {
        Op::FsRead => Some(GuardClass::FsRead),
        Op::FsWrite => Some(GuardClass::FsWrite),
        Op::Net => Some(GuardClass::Net),
        Op::Declassify => Some(GuardClass::Declassify),
        Op::ForeignBind => Some(GuardClass::ForeignC),
        Op::Actuate => Some(GuardClass::Device),
        // No scope dimension exists for these, so there is no per-item axis to gate on; the
        // cross-cutting `effect` class still applies to every one of them.
        Op::Clock | Op::Rand | Op::Console => None,
        // Stage 6 plugin loading does not route through the broker's op check (it is gated by the
        // plugin ceiling in `delulu-runtime::plugin`), so there is nothing to gate here yet.
        Op::PluginLoad => None,
    }
}

/// The effect a use of this guard class requires, if any — the dual of [`use_axis_class`] composed
/// with [`Op::required_effect`] (each axis class corresponds to exactly one `use_axis_class` op, whose
/// `required_effect` this returns). A sealed `effect:<name>` rule gates every use in the classes that
/// map to it, so a request/approval touching such a class is sealed even when its own axis rule is
/// only `guarded`; [`GuardPolicy::tier_for_subset`] uses this to mirror `tier_for_use`'s cross-cut.
/// The three classes with no use-time broker op map to `None`: `effect` items are matched directly,
/// and `secret`/`foreign_python` gate at MINT time only (their use never crosses the broker).
/// Exhaustive on purpose (campaign C31 discipline): a new class breaks this build rather than
/// silently escaping the seal cross-cut.
fn class_use_effect(class: GuardClass) -> Option<Effect> {
    match class {
        GuardClass::FsRead => Some(Effect::Read),
        GuardClass::FsWrite => Some(Effect::Write),
        GuardClass::Net => Some(Effect::Net),
        GuardClass::Declassify => Some(Effect::Declassify),
        GuardClass::ForeignC => Some(Effect::ForeignCall),
        GuardClass::Device => Some(Effect::Actuate),
        GuardClass::Effect | GuardClass::Secret | GuardClass::ForeignPython => None,
    }
}

/// The axis guard classes with an associated use-time effect (the domain where [`class_use_effect`]
/// is `Some`). [`GuardPolicy::tier_for_subset`] iterates these to resolve an `effect:E` request back
/// to the axis rules its covered uses would hit. Kept beside `class_use_effect` so the two cannot
/// drift; a missing entry only ever UNDER-seals, which a falsification test guards against.
const EFFECT_BEARING_CLASSES: [GuardClass; 6] = [
    GuardClass::FsRead,
    GuardClass::FsWrite,
    GuardClass::Net,
    GuardClass::Declassify,
    GuardClass::ForeignC,
    GuardClass::Device,
];

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
        // `device` is a BTreeMap (one envelope per device), so it cannot join the array above — which
        // is exactly how it was missed when RFC 0001 F1 added it (campaign C31). Gate on the device
        // NAME; the envelope itself is bounded by the `⊑` lattice, not by policy patterns.
        for name in s.device.keys() {
            for r in self.rules.iter().filter(|r| r.class == GuardClass::Device) {
                if pattern_matches(&r.pattern, Some(name)) {
                    consider(r.tier, rule_label(r.class, &r.pattern));
                }
            }
        }
        best
    }

    /// The strongest tier gating any use that a requested/approved subset could later authorize, with
    /// the matched rule's `class:pattern` label — or `None` when nothing gates it. This is the
    /// request-time dual of [`Self::tier_for_use`]: `guard_request`/`guard_approve` consult it so a
    /// **sealed** subset is refused (DL1413) exactly as use-time and mint-time refuse one. It closes
    /// finding F-CUSTODY-1 — a permit minted while a class is sealed would otherwise sit inert (a
    /// permit is never consulted for a sealed op, `guard_verdict_use`), then silently become effective
    /// the instant the class was unsealed to `guarded`, with no fresh approval reflecting that change.
    /// (This does NOT contradict `guard_check_mint` letting the owner mint sealed authority: a minted
    /// child's USE is still sealed-gated, whereas a permit's whole purpose is to lift the gate.)
    ///
    /// Matching is the symmetric closure of [`pattern_matches`] (`overlaps`): the requested pattern may
    /// itself be `*`, so `*` on either side matches. Each subset item is judged against (a) rules of
    /// its own class and (b) the cross-cutting `effect` class in BOTH directions — an axis item (e.g.
    /// `declassify:x`) is gated by a sealed `effect:Declassify` rule, and an `effect:E` item is gated
    /// by any rule of a class whose use requires `E` (an `effect:E` permit covers EVERY token of that
    /// class, so a single sealed token seals the request). Anything `tier_for_use` refuses at use time
    /// is therefore refused here at request time; an over-broad subset straddling a seal is refused
    /// whole, which is correct — the holder can re-request the un-sealed items narrowly.
    pub fn tier_for_subset(&self, subset: &GuardSubset) -> Option<(GuardTier, String)> {
        fn overlaps(a: &str, b: &str) -> bool {
            a == "*" || b == "*" || a == b
        }
        let mut best: Option<(GuardTier, String)> = None;
        let mut consider = |tier: GuardTier, label: String| {
            if best.as_ref().is_none_or(|(t, _)| tier > *t) {
                best = Some((tier, label));
            }
        };
        for (class, pat) in &subset.0 {
            // (a) rules of the item's own class.
            for r in self.rules.iter().filter(|r| r.class == *class) {
                if overlaps(&r.pattern, pat) {
                    consider(r.tier, rule_label(r.class, &r.pattern));
                }
            }
            // (b) cross-cutting effect, axis → effect: a use of this axis class requires an effect, so
            // a sealed `effect:<that>` rule gates it (exactly as `tier_for_use` folds in the effect).
            if let Some(eff) = class_use_effect(*class) {
                for r in self.rules.iter().filter(|r| r.class == GuardClass::Effect) {
                    if overlaps(&r.pattern, eff.name()) {
                        consider(r.tier, rule_label(r.class, &r.pattern));
                    }
                }
            }
            // (b) cross-cutting effect, effect → axis: an `effect:E` item covers every token of any
            // axis class whose use requires an effect named by the item, so every rule on that class
            // applies (not just an overlapping-token one — the request is unbounded over tokens).
            if *class == GuardClass::Effect {
                for other in EFFECT_BEARING_CLASSES {
                    let Some(eff) = class_use_effect(other) else { continue };
                    if overlaps(pat, eff.name()) {
                        for r in self.rules.iter().filter(|r| r.class == other) {
                            consider(r.tier, rule_label(r.class, &r.pattern));
                        }
                    }
                }
            }
        }
        best
    }

    /// Class wire-names that have ANY rule — the set a client uses to decide which epoch-class ops
    /// must round-trip synchronously (addendum §2.4.2 / criterion 11). Guarded/sealed ops cannot
    /// consult permits from a cached snapshot; `warn`-tier ops cannot audit `guard_warn` or surface
    /// the agent note from one either (deviation §7) — so every RULED class routes synchronously;
    /// ungated classes keep epoch caching unchanged. Sorted, deduped.
    pub fn guarded_classes(&self) -> Vec<String> {
        let mut set: BTreeSet<&'static str> = BTreeSet::new();
        for r in &self.rules {
            set.insert(r.class.wire_name());
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
    /// Monotone id counters for minted permits / requests (phase 5l).
    next_permit: u64,
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
        // A poisoned store extends no trust (criterion 10): mint refuses guarded without owner code.
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
        Err(Denial::GuardMintBlocked { parent: parent.clone(), rule })
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

    // ----- the approval flow (phase 5l, addendum §2.5) ------------------------------------------

    /// An agent requests guarded access with a mandatory justification (no owner — the agent
    /// explaining itself is free). Records `guard_request` (with the `why`). A repeat request for the
    /// same (node, subset) that is still pending returns the existing id (dedup). Returns `(id,
    /// deduped)`.
    pub fn guard_request(
        &mut self,
        node: &GrantId,
        subset: GuardSubset,
        why: String,
    ) -> Result<(String, bool), Denial> {
        // A sealed subset is not runtime-approvable (addendum §2.6): refuse the request the same way
        // use-time (DL1413) and mint-time do, BEFORE queueing it. Checked ahead of dedup so a seal set
        // AFTER a first request still refuses retries. This is fail-fast; `guard_approve` re-checks —
        // that is the decisive gate, since a request queued while `guarded` may be sealed before it is
        // approved (F-CUSTODY-1).
        if let Some((GuardTier::Sealed, rule)) = self.guard.policy.tier_for_subset(&subset) {
            return Err(Denial::GuardSealed { node: node.clone(), rule });
        }
        self.guard_gc();
        if let Some(r) = self
            .guard
            .requests
            .iter()
            .find(|r| r.node == *node && r.subset == subset && r.status == ReqStatus::Pending)
        {
            return Ok((r.id.clone(), true));
        }
        let id = format!("gr_{:04}", self.guard.next_request);
        self.guard.next_request += 1;
        let now = self.effective_now();
        self.guard.requests.push(GuardRequest {
            id: id.clone(),
            node: node.clone(),
            subset: subset.clone(),
            why: why.clone(),
            created_millis: now,
            status: ReqStatus::Pending,
        });
        let seq = self.consume_seq();
        self.record_op(
            seq,
            "guard_request",
            Some(node.as_str().to_string()),
            Some(subset.labels().join(",")),
            Some(serde_json::json!({ "why": why, "request": id })),
            "allow",
            None,
        );
        Ok((id, false))
    }

    /// The pending/decided request queue (with justifications), after a GC of expired pendings.
    pub fn guard_pending(&mut self) -> Vec<GuardRequest> {
        self.guard_gc();
        self.guard.requests.clone()
    }

    /// The broker-held permits, after a GC of expired ones.
    pub fn guard_permits(&mut self) -> Vec<Permit> {
        self.guard_gc();
        self.guard.permits.clone()
    }

    /// The principal approves a request, minting a permit (owner-gated, addendum §2.5). Default TTL
    /// 15 min (ruling 3); unlimited uses within scope unless `uses`. Records `guard_approve` (comment,
    /// ttl, uses). `Ok(None)` = no such pending request; `Ok(Some(permit_id))` = approved.
    pub fn guard_approve(
        &mut self,
        owner: Option<&str>,
        id: &str,
        ttl_millis: Option<i64>,
        uses: Option<u64>,
        comment: Option<String>,
    ) -> Result<Option<String>, Denial> {
        if !self.guard.owner_ok(owner) {
            return Err(Denial::GuardOwner { detail: "approve".into() });
        }
        self.guard_gc();
        let Some(pos) =
            self.guard.requests.iter().position(|r| r.id == id && r.status == ReqStatus::Pending)
        else {
            return Ok(None);
        };
        let subset = self.guard.requests[pos].subset.clone();
        let node = self.guard.requests[pos].node.clone();
        // The decisive seal gate (F-CUSTODY-1). The class may have been sealed AFTER this request was
        // queued (requested while `guarded`, then `guard policy set … sealed`). Sealed authority is
        // not runtime-approvable: refuse DL1413 and leave the request PENDING, so no permit is ever
        // minted for a sealed subset. The owner's path to grant it is to unseal first (a policy edit),
        // then approve — which mints a permit that correctly reflects the now-`guarded` status.
        if let Some((GuardTier::Sealed, rule)) = self.guard.policy.tier_for_subset(&subset) {
            return Err(Denial::GuardSealed { node, rule });
        }
        self.guard.requests[pos].status = ReqStatus::Approved;
        let permit_id = format!("gp_{:04}", self.guard.next_permit);
        self.guard.next_permit += 1;
        let now = self.effective_now();
        let expires_millis = Some(now + ttl_millis.unwrap_or(DEFAULT_PERMIT_TTL_MILLIS));
        self.guard.permits.push(Permit {
            id: permit_id.clone(),
            node: node.clone(),
            subset,
            expires_millis,
            remaining_uses: uses,
        });
        let seq = self.consume_seq();
        self.record_op(
            seq,
            "guard_approve",
            Some(node.as_str().to_string()),
            Some(permit_id.clone()),
            Some(serde_json::json!({ "comment": comment, "ttl_millis": ttl_millis, "uses": uses })),
            "allow",
            None,
        );
        Ok(Some(permit_id))
    }

    /// The principal denies a request with a mandatory comment (owner-gated, addendum §2.5). Records
    /// `guard_deny` (comment). A retried use then refuses DL1412 carrying the comment. `Ok(false)` =
    /// no such pending request.
    pub fn guard_deny(&mut self, owner: Option<&str>, id: &str, comment: String) -> Result<bool, Denial> {
        if !self.guard.owner_ok(owner) {
            return Err(Denial::GuardOwner { detail: "deny".into() });
        }
        self.guard_gc();
        let Some(pos) =
            self.guard.requests.iter().position(|r| r.id == id && r.status == ReqStatus::Pending)
        else {
            return Ok(false);
        };
        let node = self.guard.requests[pos].node.clone();
        self.guard.requests[pos].status = ReqStatus::Denied { comment: comment.clone() };
        let seq = self.consume_seq();
        self.record_op(
            seq,
            "guard_deny",
            Some(node.as_str().to_string()),
            Some(comment),
            None,
            "deny",
            None,
        );
        Ok(true)
    }

    /// Revoke a permit (owner-gated). Records `guard_permit_revoke`. `Ok(false)` = no such permit.
    pub fn guard_permit_revoke(&mut self, owner: Option<&str>, id: &str) -> Result<bool, Denial> {
        if !self.guard.owner_ok(owner) {
            return Err(Denial::GuardOwner { detail: "permits revoke".into() });
        }
        let node = self.guard.permits.iter().find(|p| p.id == id).map(|p| p.node.clone());
        let before = self.guard.permits.len();
        self.guard.permits.retain(|p| p.id != id);
        let removed = self.guard.permits.len() != before;
        if removed {
            let seq = self.consume_seq();
            self.record_op(
                seq,
                "guard_permit_revoke",
                node.map(|n| n.as_str().to_string()),
                Some(id.to_string()),
                None,
                "allow",
                None,
            );
        }
        Ok(removed)
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
    fn a_single_device_can_be_sealed_by_name() {
        // C31. `device` was the only scope dimension with no guard class. Actuation was still gated —
        // through the cross-cutting `effect:Actuate` axis — but ONLY all-or-nothing, while every other
        // axis took per-item rules (`net:api.example.com`, `secret:DB_PASSWORD`). For the one axis that
        // moves physical hardware, that was the wrong asymmetry: an operator could not seal a thruster
        // while leaving a status LED at `warn`.
        //
        // Two mechanical causes, both now closed: `tier_for_mint` walked a fixed `[…; 7]` array that
        // could not fail to compile when RFC 0001 F1 added the 8th dimension, and `use_axis_class`
        // ended in `_ => None`, so `Op::Actuate` was born ungated on its own axis and nothing said so.
        let mut b = broker();
        b.guard_policy_set(Some("gow1_testowner"), GuardClass::Device, "sat0/thruster".into(), GuardTier::Sealed)
            .unwrap();

        let scopes = || Scopes {
            device: ["sat0/thruster:pitch=-5..5,heartbeat_ms=1000,ttl_ms=60000,fail=safe-park",
                "sat0/led:brightness=0..1,heartbeat_ms=1000,ttl_ms=60000,fail=safe-park"]
                .iter()
                .map(|g| crate::device_scope::parse(g).expect("device grant parses"))
                .map(|d| (d.device.clone(), d))
                .collect(),
            ..Default::default()
        };
        let root = b.issue(holder(), Authority::new(eff(&["Actuate"]), scopes()), None);
        let child = b.attenuate(&root, Authority::new(eff(&["Actuate"]), scopes()), holder(), None).unwrap();

        // The sealed device refuses at USE time, by name, even though the grant covers it.
        match b.guard_verdict_use(&child, Op::Actuate, Some("sat0/thruster")) {
            GuardVerdict::Block(d) => assert_eq!(d.code(), "DL1413", "sealed device must be DL1413"),
            _ => panic!("the sealed device must be refused"),
        }
        // The device NOT named by the rule is unaffected — this is the granularity that was missing.
        assert!(
            !matches!(b.guard_verdict_use(&child, Op::Actuate, Some("sat0/led")), GuardVerdict::Block(_)),
            "an unnamed device must not be swept up by another device's rule"
        );
    }

    #[test]
    fn minting_a_sealed_device_is_refused() {
        // The mint half: a child requesting a sealed device cannot be minted without the owner code,
        // exactly as for every other dimension. Before C31 this mint was invisible to the guard.
        let mut b = broker();
        b.guard_policy_set(Some("gow1_testowner"), GuardClass::Device, "sat0/thruster".into(), GuardTier::Sealed)
            .unwrap();
        let dev = |g: &str| {
            let d = crate::device_scope::parse(g).expect("device grant parses");
            Scopes { device: [(d.device.clone(), d)].into_iter().collect(), ..Default::default() }
        };
        let root = b.issue(holder(), Authority::new(eff(&["Actuate"]), dev("sat0/thruster:pitch=-5..5,heartbeat_ms=1000,ttl_ms=60000,fail=safe-park")), None);
        let child_auth = Authority::new(eff(&["Actuate"]), dev("sat0/thruster:pitch=-1..1,heartbeat_ms=1000,ttl_ms=60000,fail=safe-park"));
        let d = b.guard_check_mint(&child_auth, &root, None).unwrap_err();
        assert_eq!(d.code(), "DL1413", "minting a sealed device must be refused");
        assert!(d.to_diagnostic().message.contains("device:sat0/thruster"), "the rule must be named: {}", d.to_diagnostic().message);
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

    /// A root holding Declassify over {foo, bar}, and a delegated child minted with the owner code.
    fn declassify_root_and_child(b: &mut Broker) -> (GrantId, GrantId) {
        let scopes = || Scopes { declassify: names(&["foo", "bar"]), ..Default::default() };
        let root = b.issue(holder(), Authority::new(eff(&["Declassify"]), scopes()), None);
        // Minting the guarded slice needs the owner code (criterion 3).
        b.guard_check_mint(&Authority::new(eff(&["Declassify"]), scopes()), &root, Some("gow1_testowner")).unwrap();
        let child = b.attenuate(&root, Authority::new(eff(&["Declassify"]), scopes()), holder(), None).unwrap();
        (root, child)
    }

    /// Criterion 4 (unit): request → approve → the retried use succeeds via the permit.
    /// Criterion 5 (unit): request → deny → the retried use is DL1412 carrying the comment VERBATIM.
    #[test]
    fn request_approve_and_deny_flow() {
        let mut b = broker();
        let (_root, child) = declassify_root_and_child(&mut b);

        // Blocked first (no permit).
        assert!(matches!(b.guard_verdict_use(&child, Op::Declassify, Some("foo")), GuardVerdict::Block(d) if d.code() == "DL1410"));

        // Request (with why) → pending; a retried use is DL1411 carrying the request id.
        let subset = GuardSubset::parse(&["declassify:foo".to_string()]).unwrap();
        let (id, deduped) =
            b.guard_request(&child, subset.clone(), "need the api key to call home".into()).unwrap();
        assert!(!deduped);
        match b.guard_verdict_use(&child, Op::Declassify, Some("foo")) {
            GuardVerdict::Block(d) => {
                assert_eq!(d.code(), "DL1411");
                assert!(d.to_diagnostic().message.contains(&id), "DL1411 carries the request id");
            }
            _ => panic!("a pending request must yield DL1411"),
        }
        // Dedup: a repeat request for the same (node, subset) returns the existing id.
        let (id2, deduped2) = b.guard_request(&child, subset, "again".into()).unwrap();
        assert!(deduped2 && id2 == id, "repeat request dedups to the existing id");

        // Approve → the retried use succeeds via the permit.
        let pid = b.guard_approve(Some("gow1_testowner"), &id, None, None, Some("ok, one call".into())).unwrap().unwrap();
        assert!(pid.starts_with("gp_"));
        assert!(matches!(b.guard_verdict_use(&child, Op::Declassify, Some("foo")), GuardVerdict::PermitUse));

        // Criterion 5: a SECOND request (bar) → deny (comment required) → DL1412 carrying the comment.
        let (id3, _) = b
            .guard_request(&child, GuardSubset::parse(&["declassify:bar".to_string()]).unwrap(), "and bar too".into())
            .unwrap();
        assert!(b.guard_deny(Some("gow1_testowner"), &id3, "no — bar is out of bounds".into()).unwrap());
        match b.guard_verdict_use(&child, Op::Declassify, Some("bar")) {
            GuardVerdict::Block(d) => {
                assert_eq!(d.code(), "DL1412");
                assert!(d.to_diagnostic().message.contains("no — bar is out of bounds"), "DL1412 carries the comment verbatim: {}", d.to_diagnostic().message);
            }
            _ => panic!("a denied request must yield DL1412"),
        }
    }

    /// Head-chef soundness requirement: a permit for `declassify:foo` does NOT cover `declassify:bar`
    /// (the USE must be covered by the PERMIT, never the reverse).
    #[test]
    fn a_permit_covers_only_its_own_subset() {
        let mut b = broker();
        let (_root, child) = declassify_root_and_child(&mut b);
        let (id, _) = b
            .guard_request(&child, GuardSubset::parse(&["declassify:foo".to_string()]).unwrap(), "foo only".into())
            .unwrap();
        b.guard_approve(Some("gow1_testowner"), &id, None, None, None).unwrap().unwrap();
        // foo is covered…
        assert!(matches!(b.guard_verdict_use(&child, Op::Declassify, Some("foo")), GuardVerdict::PermitUse));
        // …bar is NOT (the permit for foo does not widen to bar) → a fresh DL1410.
        assert!(matches!(b.guard_verdict_use(&child, Op::Declassify, Some("bar")), GuardVerdict::Block(d) if d.code() == "DL1410"));
    }

    /// Criterion 6 (unit): a `sealed` rule refuses DL1413 even with a pre-existing approved permit —
    /// sealed short-circuits before any permit is consulted (and bypass never lifts it, see the 5k
    /// `sealed_use_is_dl1413_even_under_bypass`).
    ///
    /// The permit is minted the ONLY way a permit can now exist for a class that ends up sealed: while
    /// the class was still `guarded`. (F-CUSTODY-1 closed the other route — `guard_approve` refuses a
    /// subset that is sealed AT approval time, so you can no longer mint a permit for an already-sealed
    /// class.) This is the realistic inert-permit scenario: an approval predates a later seal.
    #[test]
    fn sealed_refuses_even_with_an_approved_permit() {
        let mut b = broker();
        // Start GUARDED so a permit can be minted through the normal request/approve flow…
        b.guard_policy_set(Some("gow1_testowner"), GuardClass::FsWrite, "*".into(), GuardTier::Guarded).unwrap();
        let root = b.issue(holder(), Authority::new(eff(&["Write"]), Scopes { fs_write: names(&["./out"]), ..Default::default() }), None);
        b.guard_check_mint(&Authority::new(eff(&["Write"]), Scopes { fs_write: names(&["./out"]), ..Default::default() }), &root, Some("gow1_testowner")).unwrap();
        let child = b.attenuate(&root, Authority::new(eff(&["Write"]), Scopes { fs_write: names(&["./out"]), ..Default::default() }), holder(), None).unwrap();
        let (id, _) = b
            .guard_request(&child, GuardSubset::parse(&["fs_write:*".to_string()]).unwrap(), "want it".into())
            .unwrap();
        b.guard_approve(Some("gow1_testowner"), &id, None, None, None).unwrap().unwrap();
        // …the permit works while guarded…
        assert!(matches!(b.guard_verdict_use(&child, Op::FsWrite, Some("./out/x")), GuardVerdict::PermitUse));
        // …now SEAL the class. The pre-existing permit becomes inert: use-time refuses DL1413 before
        // any permit is consulted, so the seal is airtight even against an approval that predates it.
        b.guard_policy_set(Some("gow1_testowner"), GuardClass::FsWrite, "*".into(), GuardTier::Sealed).unwrap();
        assert!(matches!(b.guard_verdict_use(&child, Op::FsWrite, Some("./out/x")), GuardVerdict::Block(d) if d.code() == "DL1413"));
    }

    // F-CUSTODY-1: a sealed subset is NOT runtime-approvable. Split into focused cases so that
    // neutering `tier_for_subset` makes EACH fail independently (an assert panic no longer hides the
    // cases after it) — the falsification that proves none of these is vacuous.
    fn foo_subset() -> GuardSubset {
        GuardSubset::parse(&["declassify:foo".to_string()]).unwrap()
    }

    /// (F-CUSTODY-1, direct) Sealing the class refuses the request immediately, before it is queued.
    #[test]
    fn sealing_a_class_refuses_its_request_dl1413() {
        let mut b = broker();
        let (_root, child) = declassify_root_and_child(&mut b);
        b.guard_policy_set(Some("gow1_testowner"), GuardClass::Declassify, "*".into(), GuardTier::Sealed).unwrap();
        assert!(matches!(b.guard_request(&child, foo_subset(), "want it".into()), Err(d) if d.code() == "DL1413"));
    }

    /// (F-CUSTODY-1, the decisive gate) A request made while `guarded`, then sealed, must be refused at
    /// APPROVE time — the request-time check provably cannot catch this, so it isolates the approve
    /// gate. No permit is minted; the request is left pending for a later unseal.
    #[test]
    fn sealing_after_a_request_refuses_the_approval_dl1413() {
        let mut b = broker();
        let (_root, child) = declassify_root_and_child(&mut b);
        let (id, _) = b.guard_request(&child, foo_subset(), "want it".into()).unwrap();
        b.guard_policy_set(Some("gow1_testowner"), GuardClass::Declassify, "*".into(), GuardTier::Sealed).unwrap();
        assert!(matches!(b.guard_approve(Some("gow1_testowner"), &id, None, None, None), Err(d) if d.code() == "DL1413"));
        assert!(b.guard_permits().is_empty(), "no permit may be minted for a sealed subset");
        assert!(
            b.guard_pending().iter().any(|r| r.id == id && matches!(r.status, ReqStatus::Pending)),
            "a refused approve leaves the request pending — the owner must unseal first"
        );
    }

    /// (F-CUSTODY-1, cross-cut axis → effect) `declassify:*` only GUARDED, but `effect:Declassify`
    /// SEALED. A declassify use would be DL1413 at use time (the effect folds in), so the request must
    /// be refused here too — a sealed verdict reachable ONLY through the `class_use_effect` cross-cut.
    #[test]
    fn a_sealed_effect_rule_seals_an_axis_request() {
        let mut b = broker();
        let (_root, child) = declassify_root_and_child(&mut b);
        b.guard_policy_set(Some("gow1_testowner"), GuardClass::Effect, "Declassify".into(), GuardTier::Sealed).unwrap();
        assert!(matches!(b.guard_request(&child, foo_subset(), "want it".into()), Err(d) if d.code() == "DL1413"));
    }

    /// (F-CUSTODY-1, cross-cut effect → axis) A specific token `declassify:secret` SEALED, requested as
    /// the broad `effect:Declassify` (which covers EVERY declassify token). Must refuse — otherwise the
    /// broad permit would cover `declassify:secret` the instant it was unsealed. Reachable ONLY through
    /// the effect→axis cross-cut (there is no sealed `effect:*` rule for the direct branch to find).
    #[test]
    fn a_sealed_axis_token_seals_a_broad_effect_request() {
        let mut b = broker();
        let (_root, child) = declassify_root_and_child(&mut b);
        b.guard_policy_set(Some("gow1_testowner"), GuardClass::Declassify, "secret".into(), GuardTier::Sealed).unwrap();
        let eff_decl = GuardSubset::parse(&["effect:Declassify".to_string()]).unwrap();
        assert!(matches!(b.guard_request(&child, eff_decl, "broad".into()), Err(d) if d.code() == "DL1413"));
    }

    /// (F-CUSTODY-1, negative) A merely GUARDED class is unaffected: request and approve still work
    /// end-to-end, so the seal gate does not over-refuse the ordinary approval path.
    #[test]
    fn a_guarded_subset_still_approves_normally() {
        let mut b = broker();
        let (_root, child) = declassify_root_and_child(&mut b);
        let (id, _) = b.guard_request(&child, foo_subset(), "want it".into()).unwrap();
        let pid = b.guard_approve(Some("gow1_testowner"), &id, None, None, None).unwrap().unwrap();
        assert!(pid.starts_with("gp_"), "a guarded subset still approves normally");
    }

    /// Permits and pending requests are daemon-memory only — an owner-gated revoke drops a permit,
    /// and a fresh `GuardState` (a restart) starts with none (asserted structurally below).
    #[test]
    fn permits_are_session_scoped_and_revocable() {
        let mut b = broker();
        let (_root, child) = declassify_root_and_child(&mut b);
        let (id, _) = b
            .guard_request(&child, GuardSubset::parse(&["declassify:foo".to_string()]).unwrap(), "why".into())
            .unwrap();
        let pid = b.guard_approve(Some("gow1_testowner"), &id, None, None, None).unwrap().unwrap();
        assert!(matches!(b.guard_verdict_use(&child, Op::Declassify, Some("foo")), GuardVerdict::PermitUse));
        // Revoke (owner-gated) → the use is blocked again.
        assert!(b.guard_permit_revoke(Some("gow1_testowner"), &pid).unwrap());
        assert!(matches!(b.guard_verdict_use(&child, Op::Declassify, Some("foo")), GuardVerdict::Block(_)));
        // A wrong owner cannot revoke.
        assert_eq!(b.guard_permit_revoke(Some("nope"), "gp_0000").unwrap_err().code(), "DL1414");
        // A fresh GuardState (the restart shape) holds no permits.
        assert_eq!(GuardState::new().permits.len(), 0);
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
