//! The Atlas data model: typed nodes and edges, stable ids, and the versioned `atlas/1` envelope.
//!
//! Every field here is a *checked fact* — there are no confidence tags, because a fact either is a
//! checked fact (and appears) or it is not (and does not). Determinism is structural: `Atlas::build`
//! sorts every collection by stable id before it is stored, so two runs over the same input produce
//! byte-identical output in every format (criterion 1).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The verbatim honesty caveat that ships in `caveats` in every format (addendum §2.6). The atlas
/// maps *checked structure*, not a runtime trace.
pub const CAVEAT_STATIC: &str =
    "the atlas is a static map of checked facts, not a runtime trace; calls through function values \
     may be under-approximated";

/// The kinds of node the atlas contains (addendum §2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Package,
    Module,
    Function,
    Type,
    Effect,
    /// A resource a capability designates: an fs path class, a net host, a secret name.
    Resource,
    /// A foreign boundary: a C symbol or a Python module.
    Foreign,
}

impl NodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            NodeKind::Package => "package",
            NodeKind::Module => "module",
            NodeKind::Function => "function",
            NodeKind::Type => "type",
            NodeKind::Effect => "effect",
            NodeKind::Resource => "resource",
            NodeKind::Foreign => "foreign",
        }
    }
}

/// The kinds of edge the atlas contains (addendum §2.1). Every edge is a checked relationship.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// package → module, module → function/type.
    Contains,
    /// package → package.
    DependsOn,
    /// module → module.
    Imports,
    /// function → function (from `FnFacts.callees` + `Program.call_owner`).
    Calls,
    /// function → type (from a checked signature).
    UsesType,
    /// function → effect (from the checked effect row).
    Performs,
    /// function/package → resource (from the authority report).
    Requires,
    /// function → foreign boundary.
    Foreign,
    /// function → resource of kind secret (declassification).
    Declassifies,
    /// grant → node (custody overlay only, phase A3).
    Delegates,
}

impl EdgeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EdgeKind::Contains => "contains",
            EdgeKind::DependsOn => "depends_on",
            EdgeKind::Imports => "imports",
            EdgeKind::Calls => "calls",
            EdgeKind::UsesType => "uses_type",
            EdgeKind::Performs => "performs",
            EdgeKind::Requires => "requires",
            EdgeKind::Foreign => "foreign",
            EdgeKind::Declassifies => "declassifies",
            EdgeKind::Delegates => "delegates",
        }
    }
}

/// A source location: a file name and 1-based line (byte-precise column is intentionally omitted —
/// the atlas orients, it does not replace the source).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpanLoc {
    pub file: String,
    pub line: u32,
}

/// One node. `id` is the stable, diff-friendly identity; kind-specific fields are present only when
/// they apply (a function carries `effects`/`pure`, a resource carries `resource_class`/`pattern`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub span: Option<SpanLoc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub effects: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub pure: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub resource_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub pattern: Option<String>,
}

impl Node {
    pub fn new(id: impl Into<String>, kind: NodeKind, name: impl Into<String>) -> Node {
        Node {
            id: id.into(),
            kind,
            name: name.into(),
            module: None,
            package: None,
            span: None,
            effects: Vec::new(),
            pure: None,
            resource_class: None,
            pattern: None,
        }
    }
}

/// One directed, typed edge between two node ids.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
}

/// A "god node" — a most-connected node, reported in every format (addendum §2.1). Cheap (degree
/// count), genuinely orienting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct God {
    pub id: String,
    pub name: String,
    pub kind: NodeKind,
    pub degree: usize,
}

/// The whole atlas — the versioned envelope of `atlas/1` (addendum §2.4). Additive evolution only.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Atlas {
    /// The schema tag, always `"atlas/1"`.
    #[serde(rename = "atlas")]
    pub version: String,
    pub root: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub gods: Vec<God>,
    /// The authority report — byte-for-byte the same object `delulu authority` prints (parity).
    pub authority: Value,
    /// The custody overlay (phase A3); `null` unless `--custody` succeeded.
    pub custody: Option<Value>,
    pub caveats: Vec<String>,
}

/// The current schema version string.
pub const SCHEMA: &str = "atlas/1";

impl Atlas {
    /// Serialize to the `atlas/1` JSON envelope. Machine channel — NEVER colored (criterion 2).
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).expect("atlas serializes")
    }

    /// Pretty JSON string (stable key order via serde_json's map ordering on our structs).
    pub fn to_json_string(&self) -> String {
        serde_json::to_string_pretty(&self.to_json()).expect("atlas serializes")
    }

    /// Look up a node by exact id.
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Degree (in + out) of a node id.
    pub fn degree(&self, id: &str) -> usize {
        self.edges.iter().filter(|e| e.from == id || e.to == id).count()
    }

    /// Outgoing edges from `id`.
    pub fn out_edges<'a>(&'a self, id: &'a str) -> impl Iterator<Item = &'a Edge> + 'a {
        self.edges.iter().filter(move |e| e.from == id)
    }

    /// Incoming edges to `id`.
    pub fn in_edges<'a>(&'a self, id: &'a str) -> impl Iterator<Item = &'a Edge> + 'a {
        self.edges.iter().filter(move |e| e.to == id)
    }
}

// ----- stable id constructors (addendum §2.1) -----------------------------------------------------

pub fn pkg_id(pkg: &str) -> String {
    format!("pkg:{pkg}")
}
pub fn mod_id(pkg: &str, module: &str) -> String {
    format!("mod:{pkg}/{module}")
}
pub fn fn_id(pkg: &str, module: &str, name: &str) -> String {
    format!("fn:{pkg}/{module}.{name}")
}
pub fn type_id(pkg: &str, module: &str, name: &str) -> String {
    format!("type:{pkg}/{module}.{name}")
}
pub fn effect_id(name: &str) -> String {
    format!("effect:{name}")
}
pub fn resource_id(class: &str, pattern: &str) -> String {
    format!("res:{class}:{pattern}")
}
pub fn foreign_c_id(symbol: &str) -> String {
    format!("foreign:c:{symbol}")
}
pub fn foreign_py_id(module: &str) -> String {
    format!("foreign:py:{module}")
}
pub fn grant_id(node_id: &str) -> String {
    format!("grant:{node_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_follow_the_scheme() {
        assert_eq!(pkg_id("app"), "pkg:app");
        assert_eq!(mod_id("app", "app.util"), "mod:app/app.util");
        assert_eq!(fn_id("app", "app.util", "announce"), "fn:app/app.util.announce");
        assert_eq!(effect_id("Net"), "effect:Net");
        assert_eq!(resource_id("fs_write", "./out"), "res:fs_write:./out");
        assert_eq!(foreign_c_id("cos"), "foreign:c:cos");
    }

    #[test]
    fn atlas_round_trips_through_serde() {
        let atlas = Atlas {
            version: SCHEMA.to_string(),
            root: "app".to_string(),
            nodes: vec![{
                let mut n = Node::new(fn_id("app", "app", "main"), NodeKind::Function, "main");
                n.module = Some("app".to_string());
                n.effects = vec!["Write".to_string()];
                n.pure = Some(false);
                n
            }],
            edges: vec![Edge {
                from: fn_id("app", "app", "main"),
                to: effect_id("Write"),
                kind: EdgeKind::Performs,
            }],
            gods: vec![God {
                id: fn_id("app", "app", "main"),
                name: "main".to_string(),
                kind: NodeKind::Function,
                degree: 1,
            }],
            authority: serde_json::json!({"program": "app", "effects": ["Write"]}),
            custody: None,
            caveats: vec![CAVEAT_STATIC.to_string()],
        };
        let json = atlas.to_json();
        assert_eq!(json["atlas"], "atlas/1");
        assert_eq!(json["custody"], serde_json::Value::Null);
        let back: Atlas = serde_json::from_value(json.clone()).expect("round-trips");
        assert_eq!(back.to_json(), json, "serde round-trip is lossless");
    }

    #[test]
    fn json_carries_zero_ansi_bytes() {
        let atlas = Atlas {
            version: SCHEMA.to_string(),
            root: "r".to_string(),
            nodes: vec![],
            edges: vec![],
            gods: vec![],
            authority: serde_json::json!({}),
            custody: None,
            caveats: vec![CAVEAT_STATIC.to_string()],
        };
        assert!(!atlas.to_json_string().contains('\u{1b}'), "atlas/1 is never colored");
    }
}
