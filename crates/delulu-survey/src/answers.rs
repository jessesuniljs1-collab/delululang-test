//! The machine answers to the questions asked of the map: `query`/`rdeps` (one hop) and
//! `impact`/`affected-by` (transitive), as JSON.
//!
//! They lived in the binary until P4-03 (`delulu mcp`) needed the same answers: the MCP server's
//! `survey_query` and `survey_impact` tools are these functions, not a second rendering of the map
//! that could come to disagree with `delulu-survey --json`. The binary prints them; the server returns
//! them; the shape is one shape.

use serde_json::{json, Value};

use crate::{Dir, EdgeKind, Survey};

/// The short word an edge kind is printed as, in both the human and the machine rendering.
pub fn kind_word(k: EdgeKind) -> &'static str {
    match k {
        EdgeKind::DependsOn => "depends-on",
        EdgeKind::DependsOnExternal => "depends-on-ext",
        EdgeKind::DeclaresModule => "declares",
        EdgeKind::Uses => "uses",
        EdgeKind::DefinesCode => "defines-code",
        EdgeKind::RaisesCode => "raises",
        EdgeKind::ExpectsCode => "expects",
        EdgeKind::DocumentsCode => "documents",
        EdgeKind::Defines => "defines",
        EdgeKind::Cites => "cites",
        EdgeKind::LinksTo => "links-to",
        EdgeKind::References => "references",
        EdgeKind::TestsCrate => "tests",
    }
}

/// An edge or hop, rendered for machines with the citation intact.
///
/// The provenance law — *"a relation that cannot be pointed at in the text is not in the map"* — is
/// not a property of the human rendering. It has to survive into the machine channel, or an agent
/// reading this map has strictly less ability to check it than a human reading the same map.
pub fn via_json(kind: EdgeKind, file: &str, line: u32) -> Value {
    json!({ "kind": kind_word(kind), "file": file, "line": line })
}

/// The object every `--json` answer is: the tool, the verb and the schema first, so a caller can
/// branch before reading anything else, then the payload's own fields.
pub fn envelope(verb: &str, payload: Value) -> Value {
    let mut root = json!({
        "tool": "delulu-survey",
        "verb": verb,
        "schema": 1,
        "delulu_version": env!("CARGO_PKG_VERSION"),
    });
    if let (Some(o), Some(p)) = (root.as_object_mut(), payload.as_object()) {
        for (k, v) in p {
            o.insert(k.clone(), v.clone());
        }
    }
    root
}

/// Up to ten node ids that contain `id` in their id or name — the "did you mean" of a miss.
pub fn near(survey: &Survey, id: &str) -> Vec<String> {
    survey
        .nodes
        .iter()
        .filter(|n| n.id.contains(id) || n.name.contains(id))
        .map(|n| n.id.clone())
        .take(10)
        .collect()
}

/// `query` (both directions) or `rdeps` (what points at it only): the node, its edges with their
/// citations, and — always present, `null` when ordinary — whether it is entrenched. `None` when the
/// map has no such node.
pub fn query_json(survey: &Survey, id: &str, rdeps_only: bool) -> Option<Value> {
    let node = survey.node(id)?;
    let edge_json = |edges: &[&crate::Edge], incoming: bool| -> Vec<Value> {
        edges
            .iter()
            .map(|e| {
                let other = if incoming { &e.from } else { &e.to };
                json!({ "node": other, "via": via_json(e.kind, &e.file, e.line) })
            })
            .collect()
    };
    let incoming = survey.into_(id);
    let mut payload = json!({
        "node": {
            "id": node.id,
            "kind": format!("{:?}", node.kind),
            "path": node.path,
            "lines": node.lines,
            "summary": node.summary,
            "holds": node.contents,
        },
        "pointed_at_by": edge_json(&incoming, true),
    });
    if !rdeps_only {
        let outgoing = survey.out(id);
        payload["points_at"] = json!(edge_json(&outgoing, false));
    }
    // Entrenchment is the one field a maintainer must read BEFORE deciding to act, so it is always
    // present — `null` when the node is ordinary, rather than absent. An agent that keys on a missing
    // field cannot tell "not entrenched" from "this tool did not tell me".
    payload["entrenched"] = match &node.entrenched {
        Some(e) => json!({
            "owner": e.owner,
            "pattern": e.pattern,
            "matched_at": { "file": e.file, "line": e.line },
            "means": "changing this needs that owner specifically, not any maintainer; \
                      Constitution §10 requires an entrenchment analysis (invariant 44) first",
        }),
        None => Value::Null,
    };
    Some(envelope(if rdeps_only { "rdeps" } else { "query" }, payload))
}

/// `impact` (what breaks if this changes — `reverse`) or `affected-by` (what this rests on): every
/// node reached, with its depth, the node it was reached from and the citation of that hop. The
/// machine answer is deliberately UNCAPPED (campaign finding C32): the human render is truncated
/// because a saturating list stops informing a reader, but a caller that asked for the whole blast
/// radius gets it. `None` when the map has no such node.
pub fn walk_json(survey: &Survey, id: &str, reverse: bool, depth: u32) -> Option<Value> {
    survey.node(id)?;
    let dir = if reverse { Dir::Incoming } else { Dir::Outgoing };
    let reached = survey.walk(id, dir, depth);
    let question = if reverse { "what breaks if this changes" } else { "what this rests on" };
    let hops: Vec<Value> = reached
        .iter()
        .map(|r| json!({ "id": r.id, "depth": r.depth, "from": r.from, "via": via_json(r.via.kind, &r.via.file, r.via.line) }))
        .collect();
    Some(envelope(
        if reverse { "impact" } else { "affected-by" },
        json!({
            "node": id,
            "question": question,
            "depth_limit": depth,
            "reached": hops.len(),
            "hops": hops,
        }),
    ))
}
