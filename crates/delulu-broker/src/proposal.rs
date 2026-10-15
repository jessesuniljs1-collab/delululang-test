//! P9-03 step 3 (D-V2-125): proposals — a refused run asks for authority, and an approver answers it.
//!
//! A proposal is a **request, never a grant**. A node's holder files the authority one refused call needed (`delulu grants
//! propose REPORT --node g_ID` reads it from the run's report and the program's source, D-V2-123), and it waits. An
//! approval is a **delegation under the approver's own node** of the proposer's authority joined with what it asked — so
//! the tree's own `⊑` bounds it, as it bounds every delegation, and no new check is needed: a proposer approving under its
//! own node is refused DL0802 (a program cannot relax its own authority), and an approver cannot grant past what it holds.
//! The proposer's node is never changed — the program is re-run under the new node (V2 §5: restart, never a widening
//! while it runs). A rejection keeps its reason for the proposer to read. A proposal from a revoked or expired node is
//! not filed, and one whose node dies before its approval is not approved: a quarantined run's ask cannot bring its
//! authority back. Daemon memory only, like the tree itself.

use std::collections::BTreeSet;

use crate::authority::{attenuation_check, Authority, Scopes};
use crate::diag::Denial;
use crate::lease::Token;
use crate::tree::{Broker, GrantId, Holder};

/// The most bytes of each field of a proposer's claim, and of a rejection's reason, the broker keeps: text a party outside
/// the broker chose, stored and printed later (`crate::bounded_text`).
pub const CLAIM_MAX: usize = 512;
pub const REASON_MAX: usize = 2048;

/// Where a proposal stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProposalStatus {
    Pending,
    /// Approved: `node` was delegated under `under`, the approver's node.
    Approved { node: GrantId, under: GrantId },
    Rejected { reason: String },
}

/// The proposer's own account of its refusal — the program, the call, where it is, and the refusal's code. DATA: shown to
/// the approver and recorded, never a decision input (an approval decides from the node and [`Proposal::asks`] alone).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Claim {
    pub program: String,
    pub call: String,
    pub at: String,
    pub code: String,
}

impl Claim {
    fn bounded(self) -> Claim {
        let b = |s: String| crate::bounded_text(&s, CLAIM_MAX);
        Claim { program: b(self.program), call: b(self.call), at: b(self.at), code: b(self.code) }
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({ "at": self.at, "call": self.call, "code": self.code, "program": self.program })
    }
}

/// One proposal: `node` asks for `asks` beside what it holds.
#[derive(Clone, Debug)]
pub struct Proposal {
    pub id: String,
    pub node: GrantId,
    pub asks: Authority,
    pub claim: Claim,
    pub created_millis: i64,
    pub status: ProposalStatus,
}

/// The broker's proposals, in the order they were filed.
#[derive(Default)]
pub struct ProposalState {
    items: Vec<Proposal>,
    next: u64,
}

/// Why `asks` cannot be proposed beside `held`, or `None` when it can. A proposal asks for grants: never a budget (the
/// delegating side's to set), never nothing, never what the node already holds, and never a second envelope for a device
/// the node already has one for (one device has one envelope, and joining two would be a widening nobody wrote).
fn unproposable(held: &Authority, asks: &Authority) -> Option<String> {
    if asks.scopes.budget.is_some() {
        return Some("a proposal asks for grants, never a budget — a budget is the delegating side's to set".into());
    }
    if *asks == Authority::default() {
        return Some("it asks for nothing".into());
    }
    if let Some(device) = asks.scopes.device.keys().find(|d| held.scopes.device.contains_key(*d)) {
        return Some(format!(
            "the node already holds an envelope for `{device}`, and a proposal cannot widen one — ask its delegating side \
             for a new delegation"
        ));
    }
    if attenuation_check(asks, held).is_ok() {
        return Some("the node already holds everything it asks for".into());
    }
    None
}

/// `held ⊔ asks`: every effect and every scope of both, the node's budget kept (a proposal carries none), path scopes
/// re-canonicalized. [`unproposable`] has refused a device both name, so a device comes from exactly one side.
pub fn join(held: &Authority, asks: &Authority) -> Authority {
    let u = |a: &BTreeSet<String>, b: &BTreeSet<String>| a.union(b).cloned().collect::<BTreeSet<String>>();
    let (h, a) = (&held.scopes, &asks.scopes);
    let mut device = h.device.clone();
    for (name, envelope) in &a.device {
        device.entry(name.clone()).or_insert_with(|| envelope.clone());
    }
    Authority {
        effects: held.effects.union(&asks.effects).cloned().collect(),
        scopes: Scopes {
            fs_read: u(&h.fs_read, &a.fs_read),
            fs_write: u(&h.fs_write, &a.fs_write),
            net: u(&h.net, &a.net),
            secrets: u(&h.secrets, &a.secrets),
            declassify: u(&h.declassify, &a.declassify),
            foreign_c: u(&h.foreign_c, &a.foreign_c),
            foreign_python: u(&h.foreign_python, &a.foreign_python),
            device,
            budget: h.budget,
            secret_to: u(&h.secret_to, &a.secret_to),
        },
    }
    .canonicalized()
}

impl Broker {
    /// File a proposal: `node` asks for `asks` (D-V2-125). Records `proposal` (the authority asked for and the proposer's
    /// claim). A proposal still pending for the same node and the same authority is answered with its id (`deduped`).
    /// Nothing is granted. Returns `(id, deduped)`.
    pub fn propose(&mut self, node: &GrantId, asks: Authority, claim: Claim) -> Result<(String, bool), Denial> {
        let asks = asks.canonicalized();
        let held = self.live_authority(node)?;
        if let Some(detail) = unproposable(&held, &asks) {
            return Err(Denial::Proposal { node: node.clone(), detail });
        }
        if let Some(p) =
            self.proposals.items.iter().find(|p| p.node == *node && p.asks == asks && p.status == ProposalStatus::Pending)
        {
            return Ok((p.id.clone(), true));
        }
        let id = format!("pp_{:04}", self.proposals.next);
        self.proposals.next += 1;
        let claim = claim.bounded();
        let record = serde_json::json!({ "asks": asks.to_json(), "claim": claim.to_json() });
        let created_millis = self.effective_now();
        self.proposals.items.push(Proposal {
            id: id.clone(),
            node: node.clone(),
            asks,
            claim,
            created_millis,
            status: ProposalStatus::Pending,
        });
        let seq = self.consume_seq();
        self.record_op(seq, "proposal", Some(node.as_str().to_string()), Some(id.clone()), Some(record), "allow", None);
        Ok((id, false))
    }

    /// Every proposal, pending and decided, in the order filed.
    pub fn proposals(&self) -> Vec<Proposal> {
        self.proposals.items.clone()
    }

    /// What approving a pending proposal would mint now — its node's authority joined with what it asks — so an approver
    /// reads the whole new node, not only the ask. `None` for a decided proposal, or one whose node is no longer live.
    pub fn proposal_mints(&self, p: &Proposal) -> Option<Authority> {
        if p.status != ProposalStatus::Pending {
            return None;
        }
        self.live_authority(&p.node).ok().map(|held| join(&held, &p.asks))
    }

    /// Approve a pending proposal under `under`, the approver's own node (D-V2-125): a delegation of the proposer's
    /// authority joined with what it asked, through the Guard's mint gate (`owner` for guarded authority, as `grants
    /// delegate`) and the tree's `⊑` — refused DL0802 when it is not within `under`, which is every approval under the
    /// proposer's own node. Refused while the proposer's node is revoked or expired. A refusal leaves the proposal pending.
    /// Records the delegation's own `delegate` and then `proposal_approve`. `Ok(None)`: no such pending proposal.
    pub fn approve_proposal(
        &mut self,
        id: &str,
        under: &GrantId,
        owner: Option<&str>,
        ttl_millis: Option<i64>,
    ) -> Result<Option<(GrantId, Token)>, Denial> {
        let Some(pos) = self.proposals.items.iter().position(|p| p.id == id && p.status == ProposalStatus::Pending) else {
            return Ok(None);
        };
        let (proposer, asks) = (self.proposals.items[pos].node.clone(), self.proposals.items[pos].asks.clone());
        let held = self.live_authority(&proposer)?;
        let child = join(&held, &asks);
        self.guard_check_mint(&child, under, owner)?;
        let holder = Holder::new("proposal", format!("approved proposal {id} of {proposer}"), "pending");
        let (node, token) = self.delegate(under, child, holder, ttl_millis, false)?;
        self.proposals.items[pos].status = ProposalStatus::Approved { node: node.clone(), under: under.clone() };
        let seq = self.consume_seq();
        self.record_op(
            seq,
            "proposal_approve",
            Some(under.as_str().to_string()),
            Some(id.to_string()),
            Some(serde_json::json!({ "node": node.as_str(), "proposer": proposer.as_str() })),
            "allow",
            None,
        );
        Ok(Some((node, token)))
    }

    /// Reject a pending proposal, keeping `reason` for the proposer to read (D-V2-125). Records `proposal_reject` — a
    /// `deny` naming the proposer's node, as a Guard denial does. `false`: no such pending proposal.
    pub fn reject_proposal(&mut self, id: &str, reason: &str) -> bool {
        let Some(p) = self.proposals.items.iter_mut().find(|p| p.id == id && p.status == ProposalStatus::Pending) else {
            return false;
        };
        let reason = crate::bounded_text(reason, REASON_MAX);
        p.status = ProposalStatus::Rejected { reason: reason.clone() };
        let node = p.node.as_str().to_string();
        let seq = self.consume_seq();
        self.record_op(
            seq,
            "proposal_reject",
            Some(node),
            Some(id.to_string()),
            Some(serde_json::json!({ "reason": reason })),
            "deny",
            None,
        );
        true
    }

    /// RW 4.61's rule for a proposal whose record could not be written: taken off the list — an approval of it would
    /// approve something the chain never saw asked. Only the daemon calls this, in the request that filed it.
    pub fn withdraw_unrecorded_proposal(&mut self, id: &str) {
        self.proposals.items.retain(|p| p.id != id);
    }

    /// RW 4.61's rule for an approval whose records could not be written: its node withdrawn (a leaf, just minted) and
    /// the proposal pending again, so the tree never holds a node the chain did not see approved. Only the daemon calls
    /// this, in the request that approved it.
    pub fn withdraw_unrecorded_proposal_approval(&mut self, id: &str, node: &GrantId) {
        self.withdraw_unrecorded(node);
        if let Some(p) = self.proposals.items.iter_mut().find(|p| p.id == id) {
            p.status = ProposalStatus::Pending;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::MemSink;
    use crate::ids::SeqIdSource;
    use crate::time::ManualClock;
    use delulu_check::Effect;
    use std::rc::Rc;

    fn names(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }
    fn auth(effects: &[&str], scopes: Scopes) -> Authority {
        Authority::new(effects.iter().map(|e| Effect::core_from_name(e).unwrap()), scopes)
    }
    fn read(paths: &[&str]) -> Authority {
        auth(&["Read"], Scopes { fs_read: names(paths), ..Default::default() })
    }
    fn holder() -> Holder {
        Holder::new("process", "t", "pid:1")
    }
    fn claim() -> Claim {
        Claim { program: "p.delulu".into(), call: "root.fs_read(\"./b\")".into(), at: "3:5".into(), code: "DL0703".into() }
    }
    /// A broker, its records, the operator's root R (reads `./a` and `./b`, writes), and an agent's node A under it that
    /// reads `./a` only.
    fn setup() -> (Broker, MemSink, GrantId, GrantId) {
        let sink = MemSink::new();
        let mut b = Broker::with_sources(Box::new(SeqIdSource::new()), Box::new(Rc::new(ManualClock::new(1000))))
            .with_sink(Box::new(sink.clone()))
            .with_owner_code("gow1_testowner");
        let root = b.issue(holder(), auth(&["Read", "Write"], Scopes { fs_read: names(&["./a", "./b"]), ..Default::default() }), None);
        let agent = b.attenuate(&root, read(&["./a"]), holder(), None).unwrap();
        (b, sink, root, agent)
    }

    #[test]
    fn an_approval_delegates_the_join_under_the_approver_and_leaves_the_proposers_node_as_it_was() {
        let (mut b, sink, root, agent) = setup();
        let before = b.inspect(&agent).unwrap().authority.clone();
        let (id, deduped) = b.propose(&agent, read(&["./b"]), claim()).unwrap();
        assert_eq!((id.as_str(), deduped), ("pp_0000", false));
        assert_eq!(b.proposal_mints(&b.proposals()[0]), Some(read(&["./a", "./b"])), "the approver reads the whole new node");
        assert_eq!(b.propose(&agent, read(&["./b"]), claim()).unwrap(), (id.clone(), true), "a pending proposal is answered");
        let filed = sink.records().last().cloned().unwrap();
        assert_eq!((filed.action.as_str(), filed.target.as_deref(), filed.decision.as_str()), ("proposal", Some("pp_0000"), "allow"));
        assert_eq!(filed.actor_node.as_deref(), Some(agent.as_str()));
        let (node, _token) = b.approve_proposal(&id, &root, None, None).unwrap().unwrap();
        let minted = b.inspect(&node).unwrap();
        assert_eq!(minted.parent.as_ref(), Some(&root), "under the approver's node");
        assert_eq!(minted.authority, read(&["./a", "./b"]), "the proposer's authority and what it asked — nothing else");
        assert_eq!(b.inspect(&agent).unwrap().authority, before, "the proposer's node is never widened");
        let actions: Vec<String> = sink.records().iter().rev().take(2).map(|r| r.action.clone()).collect();
        assert_eq!(actions, ["proposal_approve", "delegate"]);
        assert_eq!(b.proposals()[0].status, ProposalStatus::Approved { node, under: root.clone() });
        assert_eq!(b.proposal_mints(&b.proposals()[0]), None, "a decided proposal mints nothing more");
        assert_eq!(b.approve_proposal(&id, &root, None, None).unwrap().map(|(n, _)| n), None, "decided once");
    }

    #[test]
    fn the_proposer_cannot_approve_its_own_widening_nor_an_approver_grant_past_its_node() {
        let (mut b, _sink, root, agent) = setup();
        let (id, _) = b.propose(&agent, read(&["./b"]), claim()).unwrap();
        let refused = b.approve_proposal(&id, &agent, None, None).unwrap_err();
        assert_eq!(refused.code(), "DL0802", "under its own node the join is not ⊑ it");
        // A write the root does not hold: the approver cannot grant it.
        let write = auth(&["Write"], Scopes { fs_write: names(&["./out"]), ..Default::default() });
        let (id2, _) = b.propose(&agent, write, claim()).unwrap();
        assert_eq!(b.approve_proposal(&id2, &root, None, None).unwrap_err().code(), "DL0802");
        assert!(b.proposals().iter().all(|p| p.status == ProposalStatus::Pending), "a refused approval leaves it pending");
        assert_eq!(b.len(), 2, "no node was minted");
    }

    #[test]
    fn a_dead_proposers_ask_is_neither_filed_nor_approved() {
        let (mut b, _sink, root, agent) = setup();
        let (id, _) = b.propose(&agent, read(&["./b"]), claim()).unwrap();
        b.revoke(&agent, &agent).unwrap();
        assert_eq!(b.approve_proposal(&id, &root, None, None).unwrap_err().code(), "DL1403");
        assert_eq!(b.len(), 2, "nothing minted for a revoked node's proposal");
        assert_eq!(b.proposals()[0].status, ProposalStatus::Pending);
        assert_eq!(b.propose(&agent, read(&["./a", "./b"]), claim()).unwrap_err().code(), "DL1403");
        assert_eq!(b.propose(&GrantId::from_trusted("g_nobody"), read(&["./b"]), claim()).unwrap_err().code(), "DL0904");
    }

    #[test]
    fn what_cannot_be_proposed_is_refused_in_words() {
        let (mut b, _sink, _root, agent) = setup();
        let refused = |b: &mut Broker, asks: Authority| b.propose(&agent, asks, claim()).unwrap_err().to_diagnostic().message;
        assert!(refused(&mut b, Authority::default()).contains("asks for nothing"));
        assert!(refused(&mut b, read(&["./a/inner"])).contains("already holds everything"));
        let budgeted = Authority {
            scopes: Scopes { budget: Some(crate::BudgetScope { memory_bytes: 1, cpu_seconds: 1 }), ..read(&["./b"]).scopes },
            ..read(&["./b"])
        };
        assert!(refused(&mut b, budgeted).contains("never a budget"));
        assert!(b.proposals().is_empty());
    }

    #[test]
    fn a_rejection_keeps_its_reason_and_is_recorded_as_a_deny() {
        let (mut b, sink, root, agent) = setup();
        let (id, _) = b.propose(&agent, read(&["./b"]), claim()).unwrap();
        assert!(b.reject_proposal(&id, "not ./b"));
        assert_eq!(b.proposals()[0].status, ProposalStatus::Rejected { reason: "not ./b".into() });
        let r = sink.records().last().cloned().unwrap();
        assert_eq!((r.action.as_str(), r.decision.as_str(), r.actor_node.as_deref()), ("proposal_reject", "deny", Some(agent.as_str())));
        assert!(!b.reject_proposal(&id, "again"), "decided once");
        assert_eq!(b.approve_proposal(&id, &root, None, None).unwrap().map(|(n, _)| n), None);
        // Asked again after a rejection, it is a new proposal.
        assert_eq!(b.propose(&agent, read(&["./b"]), claim()).unwrap(), ("pp_0001".to_string(), false));
    }

    #[test]
    fn an_approval_that_hands_down_guarded_authority_needs_the_owner_code() {
        let (mut b, _sink, _root, _agent) = setup();
        let declassify = |paths: &[&str]| {
            auth(&["Read", "Declassify"], Scopes { fs_read: names(paths), declassify: names(&["S"]), ..Default::default() })
        };
        let root = b.issue(holder(), declassify(&["./a", "./b"]), None);
        let agent = b.attenuate(&root, declassify(&["./a"]), holder(), None).unwrap();
        let (id, _) = b.propose(&agent, read(&["./b"]), claim()).unwrap();
        assert_eq!(b.approve_proposal(&id, &root, None, None).unwrap_err().code(), "DL1410", "the Guard's mint gate");
        assert_eq!(b.proposals()[0].status, ProposalStatus::Pending);
        let (node, _) = b.approve_proposal(&id, &root, Some("gow1_testowner"), None).unwrap().unwrap();
        assert_eq!(b.inspect(&node).unwrap().authority, declassify(&["./a", "./b"]));
    }

    #[test]
    fn the_join_keeps_both_sides_and_the_nodes_budget() {
        let held = Authority {
            scopes: Scopes { budget: Some(crate::BudgetScope { memory_bytes: 64, cpu_seconds: 2 }), ..read(&["./a"]).scopes },
            ..read(&["./a"])
        };
        let asks = auth(
            &["Net"],
            Scopes {
                net: names(&["example.com"]),
                secrets: names(&["K"]),
                secret_to: names(&["K@example.com:443"]),
                ..Default::default()
            },
        );
        let j = join(&held, &asks);
        assert_eq!(j.scopes.secret_to, names(&["K@example.com:443"]), "P9-04 (d): an audience asked for is in the join");
        assert_eq!(j.effects, auth(&["Read", "Net"], Scopes::default()).effects);
        assert_eq!((&j.scopes.fs_read, &j.scopes.net), (&held.scopes.fs_read, &names(&["example.com"])));
        assert_eq!(j.scopes.budget, held.scopes.budget);
        // Everything asked is in it — compared with the node's budget, since an absent budget is the dimension's top.
        let asked = Authority { scopes: Scopes { budget: held.scopes.budget, ..asks.scopes.clone() }, ..asks.clone() };
        assert!(attenuation_check(&held, &j).is_ok() && attenuation_check(&asked, &j).is_ok(), "both sides are within it");
    }
}
