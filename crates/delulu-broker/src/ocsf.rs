//! PS-E-06 — the audit chain, exported as OCSF 1.8.0 events, and still verifiable
//! (`V2_OPENSHELL_STUDY.md` §4.6, D-V2-78).
//!
//! A security team reads events in the Open Cybersecurity Schema Framework; DeluluLang keeps a
//! hash-chained log ([`crate::audit`]). This module maps one to the other, one event per record, and
//! keeps the property OCSF records alone cannot have: **a removed or edited event is detectable from
//! the export itself.**
//!
//! # How the chain survives the export
//!
//! Every event carries its whole audit record — every hashed field and the `hash` — under
//! `unmapped.delulu`, and every OCSF field of the event is a pure function of that record and two
//! labels the exporter states once for the whole export (the product's version and the device's
//! name). So [`verify`] does two things: it recomputes each event from its record and requires the
//! line to be exactly that event — an edited class, time, severity or message is caught — and it
//! re-verifies the chain over the records, as [`crate::audit::Bundle::verify`] does — a removed or
//! reordered line breaks a `prev_hash`, and an edited record breaks its own hash.
//!
//! What the export cannot prove, it does not claim: a removed LAST line leaves every surviving link
//! correct (P17-C1, as for the log itself), so the verifier reports the head it reached, for the
//! reader to compare with the source's `delulu audit verify`; and the two labels are the exporter's
//! word, checked only to be the same on every line.
//!
//! # The mapping (each class UID checked against the published 1.8.0 schema)
//!
//! | record | OCSF class |
//! |---|---|
//! | any `deny`; a `break-glass`; `guard_bypass_on`; `guard_bypassed_use` | Detection Finding (2004), Create |
//! | `issue`, `delegate`, `attenuate`, `redeem`, `renew`, `adopt` (allowed) | User Access Management (3005), Assign Privileges |
//! | `revoke`, `guard_permit_revoke` (allowed) | User Access Management (3005), Revoke Privileges |
//! | `sandbox-launch` | Process Activity (1007), Launch |
//! | `sandbox-death` (either decision) | Process Activity (1007), Terminate |
//! | an allowed use whose record names `FsRead` / `FsWrite` | File System Activity (1001), Read / Update |
//! | an allowed use whose record names `Net` | HTTP Activity (4002), Get |
//! | everything else | Base Event (0), Other — the record's action as the activity's name |
//!
//! A use's class comes from the effect its record NAMES (`{"op": …}`, D-V2-80), never from its
//! argument: a path and a host name are both strings, and choosing a class from the spelling would be
//! the export inventing a fact. A record written before D-V2-80 names none and stays a Base Event.
//! `Net` is `Get` because the language's one network primitive is `http.get` — `custody_op_for` maps
//! nothing else to `Net` — so a method added to the language must reach the record before the export
//! may name it.
//!
//! Never secret bytes: the audit holds none (a secret's record names the secret), and the export adds
//! nothing the record does not hold.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::audit::{canonical_json, AuditError, AuditRecord, Bundle, BundleSummary};

/// The OCSF schema version every event names in `metadata.version`.
pub const OCSF_VERSION: &str = "1.8.0";

/// The product every event names in `metadata.product`.
pub const PRODUCT_NAME: &str = "DeluluLang";

/// The two things an export says that no record does, stated once for the whole export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// `metadata.product.version` — the `delulu` that exported.
    pub product_version: String,
    /// `device.hostname` — the machine whose chain this is, as the exporter names it.
    pub device: String,
}

/// What a successful [`verify`] proved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verified {
    /// The chain over the embedded records: count, first and last seq, head.
    pub chain: BundleSummary,
    /// The `prev_hash` the first record chains onto — [`crate::audit::GENESIS_HASH`] when the export
    /// begins at the chain's beginning.
    pub start: String,
    /// The labels every line carries.
    pub labels: Labels,
    /// Events per OCSF `class_uid`.
    pub classes: BTreeMap<u64, usize>,
}

/// The class an audit record maps to — the table in the module's documentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    /// Detection Finding (2004), activity Create.
    Finding,
    /// User Access Management (3005): 1 Assign Privileges, 2 Revoke Privileges.
    UserAccess(u64),
    /// Process Activity (1007): 1 Launch, 2 Terminate.
    Process(u64),
    /// File System Activity (1001): 2 Read, 3 Update.
    File(u64),
    /// HTTP Activity (4002): 3 Get.
    Http(u64),
    /// Base Event (0), activity Other.
    Base,
}

fn class_of(rec: &AuditRecord) -> Class {
    let action = rec.action.as_str();
    // A guest's end is a process's end whatever its outcome; the decision is its status. It comes
    // first so a failed program is not reported as a detection.
    if action == "sandbox-death" {
        return Class::Process(2);
    }
    if rec.decision != "allow" || is_special(action) {
        return Class::Finding;
    }
    match action {
        "issue" | "delegate" | "attenuate" | "redeem" | "renew" | "adopt" => Class::UserAccess(1),
        "revoke" | "guard_permit_revoke" => Class::UserAccess(2),
        "sandbox-launch" => Class::Process(1),
        "use" | "guard_permit_use" | "guard_warn" => match use_op(rec) {
            Some("FsRead") => Class::File(2),
            Some("FsWrite") => Class::File(3),
            Some("Net") => Class::Http(3),
            _ => Class::Base,
        },
        _ => Class::Base,
    }
}

/// The effect a use's record names (D-V2-80), if it names one.
fn use_op(rec: &AuditRecord) -> Option<&str> {
    rec.authority.as_ref().and_then(|a| a.get("op")).and_then(Value::as_str)
}

/// An allowed record that is still a finding: authority exercised past the ordinary checks.
fn is_special(action: &str) -> bool {
    matches!(action, "break-glass" | "guard_bypass_on" | "guard_bypassed_use")
}

/// The OCSF event for one record — a pure function of the record and the export's labels, which is
/// what lets [`verify`] recompute it.
pub fn event(rec: &AuditRecord, labels: &Labels) -> Value {
    let class = class_of(rec);
    let (class_uid, class_name, category_uid, category_name, activity_id, activity_name): (u64, &str, u64, &str, u64, &str) =
        match class {
            Class::Finding => (2004, "Detection Finding", 2, "Findings", 1, "Create"),
            Class::UserAccess(a) => (
                3005,
                "User Access Management",
                3,
                "Identity & Access Management",
                a,
                if a == 1 { "Assign Privileges" } else { "Revoke Privileges" },
            ),
            Class::Process(a) => (1007, "Process Activity", 1, "System Activity", a, if a == 1 { "Launch" } else { "Terminate" }),
            Class::File(a) => (1001, "File System Activity", 1, "System Activity", a, if a == 2 { "Read" } else { "Update" }),
            Class::Http(a) => (4002, "HTTP Activity", 4, "Network Activity", a, "Get"),
            Class::Base => (0, "Base Event", 0, "Uncategorized", 99, rec.action.as_str()),
        };
    let (severity_id, severity) = match class {
        Class::Finding if is_special(&rec.action) => (4, "High"),
        Class::Finding => (3, "Medium"),
        _ => (1, "Informational"),
    };
    let message = match &rec.target {
        Some(t) => format!("{} {}: {}", rec.action, rec.decision, t),
        None => format!("{} {}", rec.action, rec.decision),
    };
    let mut ev = json!({
        "activity_id": activity_id,
        "activity_name": activity_name,
        "category_name": category_name,
        "category_uid": category_uid,
        "class_name": class_name,
        "class_uid": class_uid,
        "device": { "hostname": labels.device, "type": "Unknown", "type_id": 0 },
        "message": message,
        "metadata": {
            "event_code": rec.action,
            "log_name": "delulu audit chain",
            "product": { "name": PRODUCT_NAME, "vendor_name": PRODUCT_NAME, "version": labels.product_version },
            "sequence": rec.seq,
            "uid": rec.hash,
            "version": OCSF_VERSION,
        },
        "severity": severity,
        "severity_id": severity_id,
        "time": rec.ts,
        // OCSF's caption for activity 99 is "Other"; the record's own action stays in `activity_name`.
        "type_name": format!("{class_name}: {}", if class == Class::Base { "Other" } else { activity_name }),
        "type_uid": class_uid * 100 + activity_id,
        "unmapped": { "delulu": rec.to_value() },
    });
    let obj = ev.as_object_mut().expect("json! built an object");
    // Findings carry their own status enum (New, In Progress, …), which says nothing a record knows;
    // every other class says whether the recorded operation succeeded.
    if class != Class::Finding {
        let (status_id, status) = if rec.decision == "allow" { (1, "Success") } else { (2, "Failure") };
        obj.insert("status_id".into(), json!(status_id));
        obj.insert("status".into(), json!(status));
    }
    let node = |uid: &str| json!({ "type": "Grant node", "type_id": 99, "uid": uid });
    let actor = match &rec.actor_node {
        Some(a) => json!({ "user": node(a) }),
        None => json!({ "app_name": "delulu" }),
    };
    match class {
        Class::Finding => {
            let mut types = vec![rec.action.clone()];
            types.extend(use_op(rec).map(str::to_string));
            let mut info = json!({
                "title": format!("{} {}", rec.action, rec.decision),
                "types": types,
                "uid": rec.hash,
            });
            if let Some(t) = &rec.target {
                info.as_object_mut().expect("json! built an object").insert("desc".into(), json!(t));
            }
            obj.insert("finding_info".into(), info);
            obj.insert("is_alert".into(), json!(is_special(&rec.action)));
            obj.insert("actor".into(), actor);
        }
        Class::UserAccess(_) => {
            // The subject is the node the authority went to or was taken from: the child of a
            // delegation, the target of a revocation, the new root of an issue (its actor).
            let subject = rec.target.as_deref().or(rec.actor_node.as_deref()).unwrap_or("unknown");
            let effects: Vec<String> = rec
                .authority
                .as_ref()
                .and_then(|a| a.get("effects"))
                .and_then(Value::as_array)
                .map(|arr| arr.iter().filter_map(Value::as_str).map(str::to_string).collect())
                .unwrap_or_default();
            // A revocation's record holds no authority of its own: what it takes is everything the
            // node held, and the privilege says exactly that rather than an empty list.
            let privileges = if effects.is_empty() { vec![format!("every effect held by {subject}")] } else { effects };
            obj.insert("privileges".into(), json!(privileges));
            obj.insert("user".into(), node(subject));
            obj.insert("actor".into(), actor);
        }
        Class::Process(_) => {
            // The guest is named by its run's generation (PS-E-01, a per-run hex string the launch and
            // the death records both carry), so a reader can pair them; the record's hash otherwise.
            let uid = rec
                .authority
                .as_ref()
                .and_then(|a| a.get("generation"))
                .and_then(Value::as_str)
                .map_or_else(|| rec.hash.clone(), str::to_string);
            obj.insert("process".into(), json!({ "name": "delulu guest", "uid": uid }));
            obj.insert("actor".into(), json!({ "app_name": "delulu" }));
        }
        Class::File(_) => {
            // The path as the record holds it: the one the use was decided on (the pinned spelling).
            let path = rec.target.clone().unwrap_or_default();
            let name = path.rsplit(['/', '\\']).next().unwrap_or(&path).to_string();
            obj.insert("file".into(), json!({ "name": name, "path": path, "type": "Unknown", "type_id": 0 }));
            obj.insert("actor".into(), actor);
        }
        Class::Http(_) => {
            // A network use's argument is the host (`custody_op_for`): never a path, never a query string.
            let host = rec.target.clone().unwrap_or_default();
            obj.insert("dst_endpoint".into(), json!({ "hostname": host }));
            obj.insert("http_request".into(), json!({ "http_method": "GET" }));
            obj.insert("actor".into(), actor);
        }
        Class::Base => {
            obj.insert("actor".into(), actor);
        }
    }
    ev
}

/// The export: one canonical JSON line per record, in chain order.
pub fn export(records: &[AuditRecord], labels: &Labels) -> String {
    let mut out = String::new();
    for r in records {
        out.push_str(&canonical_json(&event(r, labels)));
        out.push('\n');
    }
    out
}

/// Verify an export: every line is exactly the event its embedded record maps to, under labels that
/// are the same on every line, and the records form an unbroken chain — from `expected_start` when
/// it is given ([`crate::audit::GENESIS_HASH`] for an export of a whole chain).
pub fn verify(text: &str, expected_start: Option<&str>) -> Result<Verified, AuditError> {
    let mut labels: Option<Labels> = None;
    let mut records = Vec::new();
    let mut classes = BTreeMap::new();
    for (i, line) in text.lines().enumerate() {
        let n = i + 1;
        if line.trim().is_empty() {
            continue;
        }
        let v: Value =
            serde_json::from_str(line).map_err(|e| AuditError::corrupt(0, format!("line {n} is not JSON: {e}")))?;
        let rec = v
            .get("unmapped")
            .and_then(|u| u.get("delulu"))
            .and_then(AuditRecord::from_value)
            .ok_or_else(|| AuditError::corrupt(0, format!("line {n} carries no DeluluLang audit record")))?;
        let these = Labels {
            product_version: str_at(&v, &["metadata", "product", "version"]),
            device: str_at(&v, &["device", "hostname"]),
        };
        match &labels {
            None => labels = Some(these.clone()),
            Some(first) if *first != these => {
                return Err(AuditError::corrupt(
                    rec.seq,
                    format!("line {n}'s labels differ from the first line's — the export was edited or spliced"),
                ))
            }
            Some(_) => {}
        }
        if canonical_json(&event(&rec, &these)) != canonical_json(&v) {
            return Err(AuditError::corrupt(
                rec.seq,
                format!("line {n} is not the event its record maps to — its OCSF fields were edited after export"),
            ));
        }
        *classes.entry(v.get("class_uid").and_then(Value::as_u64).unwrap_or_default()).or_insert(0) += 1;
        records.push(rec);
    }
    let start = records.first().map(|r| r.prev_hash.clone()).unwrap_or_default();
    let chain = Bundle { records }.verify(expected_start)?;
    Ok(Verified { chain, start, labels: labels.expect("a verified chain has a record"), classes })
}

fn str_at(v: &Value, path: &[&str]) -> String {
    let mut cur = v;
    for key in path {
        match cur.get(key) {
            Some(next) => cur = next,
            None => return String::new(),
        }
    }
    cur.as_str().unwrap_or_default().to_string()
}
