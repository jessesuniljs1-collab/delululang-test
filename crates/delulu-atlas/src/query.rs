//! Agent-side query verbs: `node`, `path`, `callers`, `calls`, `why`. Each answers a small
//! question from the graph so an agent never needs the whole atlas in context (addendum §2.2).
//!
//! `--budget N` caps any textual answer at ~N tokens (chars/4 heuristic); truncation is always
//! explicit, never silent (criterion 4).

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::fmt::Write as _;

use serde_json::{json, Value};

use crate::model::{Atlas, Edge, EdgeKind, Node, NodeKind};
use crate::render::tokens;

/// The result of resolving a user query string to a node.
pub enum Resolved<'a> {
    One(&'a Node),
    Ambiguous(Vec<&'a Node>),
    NotFound,
}

impl Atlas {
    /// Resolve a query string to node(s): by exact id, then by exact name, then by `mod::fn` or a
    /// bare function name matching a function-node id suffix.
    pub fn resolve(&self, query: &str) -> Resolved<'_> {
        if let Some(n) = self.node(query) {
            return Resolved::One(n);
        }
        // `mod::fn` → the function whose id ends with `/<mod>.<fn>`.
        let by_qual: Vec<&Node> = if let Some((m, f)) = query.split_once("::") {
            let suffix = format!("/{m}.{f}");
            self.nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Function && n.id.ends_with(&suffix))
                .collect()
        } else {
            Vec::new()
        };
        if !by_qual.is_empty() {
            return one_or_many(by_qual);
        }
        // Exact display name (functions, effects, types, resources).
        let by_name: Vec<&Node> = self.nodes.iter().filter(|n| n.name == query).collect();
        if !by_name.is_empty() {
            return one_or_many(by_name);
        }
        // A bare function name matching a function id `…​.<name>`.
        let suffix = format!(".{query}");
        let by_fn: Vec<&Node> = self
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Function && n.id.ends_with(&suffix))
            .collect();
        if !by_fn.is_empty() {
            return one_or_many(by_fn);
        }
        Resolved::NotFound
    }

    /// `atlas node <name-or-id>` — one node: kind, effects, authority, grouped + capped edges.
    pub fn query_node(&self, query: &str, budget: Option<usize>) -> String {
        let node = match self.resolve(query) {
            Resolved::One(n) => n,
            Resolved::Ambiguous(cands) => return ambiguous_msg(query, &cands),
            Resolved::NotFound => return not_found(query),
        };
        let mut out = String::new();
        let _ = writeln!(out, "{} {}  ({})", node.kind.as_str(), node.name, node.id);
        if let Some(m) = &node.module {
            let _ = writeln!(out, "  module:    {m}");
        }
        if let Some(sp) = &node.span {
            let _ = writeln!(out, "  defined:   {}:{}", sp.file, sp.line);
        }
        if node.kind == NodeKind::Function {
            let row = if node.effects.is_empty() { "!{}".into() } else { format!("!{{{}}}", node.effects.join(", ")) };
            let _ = writeln!(out, "  effects:   {row}");
            let _ = writeln!(out, "  pure:      {}", node.pure.unwrap_or(false));
        }
        if let (Some(class), Some(pat)) = (&node.resource_class, &node.pattern) {
            let _ = writeln!(out, "  resource:  {class}:{pat}");
        }

        // Grouped, capped edges.
        let cap = 12usize;
        self.write_edge_group(&mut out, "calls", node, EdgeKind::Calls, true, cap);
        self.write_edge_group(&mut out, "called by", node, EdgeKind::Calls, false, cap);
        self.write_edge_group(&mut out, "performs", node, EdgeKind::Performs, true, cap);
        self.write_edge_group(&mut out, "requires", node, EdgeKind::Requires, true, cap);
        self.write_edge_group(&mut out, "declassifies", node, EdgeKind::Declassifies, true, cap);
        self.write_edge_group(&mut out, "uses type", node, EdgeKind::UsesType, true, cap);
        self.write_edge_group(&mut out, "contains", node, EdgeKind::Contains, true, cap);
        self.write_edge_group(&mut out, "performed by", node, EdgeKind::Performs, false, cap);
        self.write_edge_group(&mut out, "required by", node, EdgeKind::Requires, false, cap);

        cap_to_budget(out, budget)
    }

    fn write_edge_group(&self, out: &mut String, label: &str, node: &Node, kind: EdgeKind, outgoing: bool, cap: usize) {
        let items: Vec<&str> = if outgoing {
            self.out_edges(&node.id).filter(|e| e.kind == kind).map(|e| e.to.as_str()).collect()
        } else {
            self.in_edges(&node.id).filter(|e| e.kind == kind).map(|e| e.from.as_str()).collect()
        };
        if items.is_empty() {
            return;
        }
        let shown = items.len().min(cap);
        let _ = writeln!(out, "  {label} ({}):", items.len());
        for &id in items.iter().take(shown) {
            let name = self.node(id).map(|n| n.name.as_str()).unwrap_or(id);
            let _ = writeln!(out, "    - {name}  ({id})");
        }
        if items.len() > shown {
            let _ = writeln!(out, "    … {} more; narrow with `atlas node <id>`", items.len() - shown);
        }
    }

    /// `atlas callers <fn>` — the reverse call slice.
    pub fn query_callers(&self, query: &str, budget: Option<usize>) -> String {
        self.call_slice(query, false, budget)
    }

    /// `atlas calls <fn>` — the forward call slice.
    pub fn query_calls(&self, query: &str, budget: Option<usize>) -> String {
        self.call_slice(query, true, budget)
    }

    fn call_slice(&self, query: &str, forward: bool, budget: Option<usize>) -> String {
        let node = match self.resolve(query) {
            Resolved::One(n) if n.kind == NodeKind::Function => n,
            Resolved::One(n) => return format!("`{}` is a {}, not a function\n", n.name, n.kind.as_str()),
            Resolved::Ambiguous(cands) => return ambiguous_msg(query, &cands),
            Resolved::NotFound => return not_found(query),
        };
        let verb = if forward { "calls" } else { "callers of" };
        let ids: Vec<&str> = if forward {
            self.out_edges(&node.id).filter(|e| e.kind == EdgeKind::Calls).map(|e| e.to.as_str()).collect()
        } else {
            self.in_edges(&node.id).filter(|e| e.kind == EdgeKind::Calls).map(|e| e.from.as_str()).collect()
        };
        let mut out = String::new();
        if ids.is_empty() {
            let _ = writeln!(out, "{verb} `{}`: (none)", node.name);
        } else {
            let _ = writeln!(out, "{verb} `{}` ({}):", node.name, ids.len());
            for id in ids {
                let name = self.node(id).map(|n| n.name.as_str()).unwrap_or(id);
                let _ = writeln!(out, "  - {name}  ({id})");
            }
        }
        cap_to_budget(out, budget)
    }

    /// `atlas path <A> <B>` — shortest path with typed hops printed.
    pub fn query_path(&self, a: &str, b: &str, budget: Option<usize>) -> String {
        let from = match self.resolve(a) {
            Resolved::One(n) => n,
            Resolved::Ambiguous(c) => return ambiguous_msg(a, &c),
            Resolved::NotFound => return not_found(a),
        };
        let to = match self.resolve(b) {
            Resolved::One(n) => n,
            Resolved::Ambiguous(c) => return ambiguous_msg(b, &c),
            Resolved::NotFound => return not_found(b),
        };
        match self.shortest_path(&from.id, &to.id) {
            None => format!("no path from `{}` to `{}`\n", from.name, to.name),
            Some(path) => {
                let mut out = String::new();
                let _ = writeln!(out, "path `{}` → `{}` ({} hop(s)):", from.name, to.name, path.len());
                let start = self.node(&from.id).map(|n| n.name.as_str()).unwrap_or(from.id.as_str());
                let _ = writeln!(out, "  {start}");
                for e in &path {
                    let name = self.node(&e.to).map(|n| n.name.as_str()).unwrap_or(e.to.as_str());
                    let _ = writeln!(out, "    --{}--> {name}  ({})", e.kind.as_str(), e.to);
                }
                cap_to_budget(out, budget)
            }
        }
    }

    /// A shortest directed path A→B over all edges, as the sequence of edges taken.
    fn shortest_path(&self, from: &str, to: &str) -> Option<Vec<Edge>> {
        if from == to {
            return Some(Vec::new());
        }
        let mut adj: HashMap<&str, Vec<&Edge>> = HashMap::new();
        for e in &self.edges {
            adj.entry(e.from.as_str()).or_default().push(e);
        }
        let mut prev: HashMap<&str, &Edge> = HashMap::new();
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut q: VecDeque<&str> = VecDeque::new();
        q.push_back(from);
        seen.insert(from);
        while let Some(cur) = q.pop_front() {
            if let Some(outs) = adj.get(cur) {
                for e in outs {
                    if seen.insert(e.to.as_str()) {
                        prev.insert(e.to.as_str(), e);
                        if e.to == to {
                            // Reconstruct.
                            let mut path = Vec::new();
                            let mut at = to;
                            while at != from {
                                let e = prev[at];
                                path.push((*e).clone());
                                at = e.from.as_str();
                            }
                            path.reverse();
                            return Some(path);
                        }
                        q.push_back(e.to.as_str());
                    }
                }
            }
        }
        None
    }

    /// `atlas why <Effect|resource>` — which functions perform/require it, and (for a few) a call
    /// chain from an entry point. The graph-shaped sibling of `delulu why` (which stays byte-stable).
    pub fn query_why(&self, query: &str, budget: Option<usize>) -> String {
        let target = match self.resolve(query) {
            Resolved::One(n) => n,
            Resolved::Ambiguous(c) => return ambiguous_msg(query, &c),
            Resolved::NotFound => return not_found(query),
        };
        if !matches!(target.kind, NodeKind::Effect | NodeKind::Resource) {
            return format!(
                "`{}` is a {} — `atlas why` explains an effect or a resource; try `atlas node {}`\n",
                target.name,
                target.kind.as_str(),
                target.name
            );
        }
        // Functions with a performs/requires/declassifies edge to the target.
        let mut origins: Vec<&str> = self
            .in_edges(&target.id)
            .filter(|e| matches!(e.kind, EdgeKind::Performs | EdgeKind::Requires | EdgeKind::Declassifies))
            .map(|e| e.from.as_str())
            .filter(|id| self.node(id).map(|n| n.kind == NodeKind::Function).unwrap_or(false))
            .collect();
        origins.sort();
        origins.dedup();

        let mut out = String::new();
        if origins.is_empty() {
            let _ = writeln!(out, "no function performs/requires `{}`", target.name);
            return out;
        }
        let _ = writeln!(
            out,
            "`{}` — {} function(s) perform/require it:",
            target.name,
            origins.len()
        );
        // Entry points: functions named `main` (bin) or any function if there is none.
        let entries: Vec<&str> = self
            .functions()
            .filter(|n| n.name == "main")
            .map(|n| n.id.as_str())
            .collect();
        for &id in &origins {
            let name = self.node(id).map(|n| n.name.as_str()).unwrap_or(id);
            let _ = writeln!(out, "  - {name}  ({id})");
            // Show one call chain from an entry to this origin, if one exists.
            for &entry in &entries {
                if entry == id {
                    let _ = writeln!(out, "      (entry point)");
                    break;
                }
                if let Some(path) = self.shortest_call_path(entry, id) {
                    let mut chain = self.node(entry).map(|n| n.name.to_string()).unwrap_or_default();
                    for step in &path {
                        let sn = self.node(step).map(|n| n.name.as_str()).unwrap_or(step);
                        let _ = write!(chain, " → {sn}");
                    }
                    let _ = writeln!(out, "      via: {chain}");
                    break;
                }
            }
        }
        cap_to_budget(out, budget)
    }

    /// Shortest path over `calls` edges only, returned as the sequence of node ids after `from`.
    fn shortest_call_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        if from == to {
            return Some(Vec::new());
        }
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for e in self.edges.iter().filter(|e| e.kind == EdgeKind::Calls) {
            adj.entry(e.from.as_str()).or_default().push(e.to.as_str());
        }
        let mut prev: HashMap<&str, &str> = HashMap::new();
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut q: VecDeque<&str> = VecDeque::new();
        q.push_back(from);
        seen.insert(from);
        while let Some(cur) = q.pop_front() {
            if let Some(outs) = adj.get(cur) {
                for &nx in outs {
                    if seen.insert(nx) {
                        prev.insert(nx, cur);
                        if nx == to {
                            let mut chain = Vec::new();
                            let mut at = to;
                            while at != from {
                                chain.push(at.to_string());
                                at = prev[at];
                            }
                            chain.reverse();
                            return Some(chain);
                        }
                        q.push_back(nx);
                    }
                }
            }
        }
        None
    }
}

impl Atlas {
    /// The machine (`--json`) form of a query verb. Structured, never colored — an agent parses this
    /// directly instead of scraping the human text.
    pub fn query_json(&self, verb: &str, a: &str, b: Option<&str>) -> Value {
        let resolve_one = |q: &str| -> Result<&Node, Value> {
            match self.resolve(q) {
                Resolved::One(n) => Ok(n),
                Resolved::Ambiguous(c) => Err(json!({
                    "error": "ambiguous", "query": q,
                    "candidates": c.iter().map(|n| n.id.clone()).collect::<Vec<_>>()
                })),
                Resolved::NotFound => Err(json!({ "error": "not_found", "query": q })),
            }
        };
        let node_ref = |id: &str| json!({ "id": id, "name": self.node(id).map(|n| n.name.as_str()).unwrap_or(id) });
        match verb {
            "node" => {
                let n = match resolve_one(a) {
                    Ok(n) => n,
                    Err(e) => return json!({ "command": "atlas", "verb": verb, "result": e }),
                };
                let group = |kind: EdgeKind, out: bool| -> Vec<Value> {
                    if out {
                        self.out_edges(&n.id).filter(|e| e.kind == kind).map(|e| node_ref(&e.to)).collect()
                    } else {
                        self.in_edges(&n.id).filter(|e| e.kind == kind).map(|e| node_ref(&e.from)).collect()
                    }
                };
                json!({
                    "command": "atlas", "verb": "node",
                    "node": serde_json::to_value(n).unwrap(),
                    "edges": {
                        "calls": group(EdgeKind::Calls, true),
                        "called_by": group(EdgeKind::Calls, false),
                        "performs": group(EdgeKind::Performs, true),
                        "requires": group(EdgeKind::Requires, true),
                        "declassifies": group(EdgeKind::Declassifies, true),
                        "uses_type": group(EdgeKind::UsesType, true),
                        "contains": group(EdgeKind::Contains, true),
                    }
                })
            }
            "callers" | "calls" => {
                let n = match resolve_one(a) {
                    Ok(n) => n,
                    Err(e) => return json!({ "command": "atlas", "verb": verb, "result": e }),
                };
                let fns: Vec<Value> = if verb == "calls" {
                    self.out_edges(&n.id).filter(|e| e.kind == EdgeKind::Calls).map(|e| node_ref(&e.to)).collect()
                } else {
                    self.in_edges(&n.id).filter(|e| e.kind == EdgeKind::Calls).map(|e| node_ref(&e.from)).collect()
                };
                json!({ "command": "atlas", "verb": verb, "of": n.id, "functions": fns })
            }
            "path" => {
                let (from, to) = match (resolve_one(a), resolve_one(b.unwrap_or(""))) {
                    (Ok(f), Ok(t)) => (f, t),
                    (Err(e), _) | (_, Err(e)) => return json!({ "command": "atlas", "verb": "path", "result": e }),
                };
                match self.shortest_path(&from.id, &to.id) {
                    None => json!({ "command": "atlas", "verb": "path", "from": from.id, "to": to.id, "found": false, "hops": [] }),
                    Some(path) => {
                        let hops: Vec<Value> = path
                            .iter()
                            .map(|e| json!({ "kind": e.kind.as_str(), "to": e.to, "name": self.node(&e.to).map(|n| n.name.as_str()).unwrap_or(&e.to) }))
                            .collect();
                        json!({ "command": "atlas", "verb": "path", "from": from.id, "to": to.id, "found": true, "hops": hops })
                    }
                }
            }
            "why" => {
                let target = match resolve_one(a) {
                    Ok(n) => n,
                    Err(e) => return json!({ "command": "atlas", "verb": "why", "result": e }),
                };
                let mut origins: Vec<&str> = self
                    .in_edges(&target.id)
                    .filter(|e| matches!(e.kind, EdgeKind::Performs | EdgeKind::Requires | EdgeKind::Declassifies))
                    .map(|e| e.from.as_str())
                    .filter(|id| self.node(id).map(|n| n.kind == NodeKind::Function).unwrap_or(false))
                    .collect();
                origins.sort();
                origins.dedup();
                let entries: Vec<&str> = self.functions().filter(|n| n.name == "main").map(|n| n.id.as_str()).collect();
                let list: Vec<Value> = origins
                    .iter()
                    .map(|&id| {
                        let via = entries
                            .iter()
                            .find_map(|&e| if e == id { Some(vec![]) } else { self.shortest_call_path(e, id) })
                            .unwrap_or_default();
                        json!({ "id": id, "name": self.node(id).map(|n| n.name.as_str()).unwrap_or(id), "via": via })
                    })
                    .collect();
                json!({ "command": "atlas", "verb": "why", "target": target.id, "origins": list })
            }
            _ => json!({ "command": "atlas", "verb": verb, "result": { "error": "unknown_verb" } }),
        }
    }
}

fn one_or_many(v: Vec<&Node>) -> Resolved<'_> {
    if v.len() == 1 {
        Resolved::One(v[0])
    } else {
        Resolved::Ambiguous(v)
    }
}

fn not_found(query: &str) -> String {
    format!("no node matches `{query}` — try `atlas <file>` first, or an id like `fn:pkg/mod.name`\n")
}

fn ambiguous_msg(query: &str, cands: &[&Node]) -> String {
    let mut out = format!("`{query}` is ambiguous — {} matches; use a full id:\n", cands.len());
    for n in cands {
        let _ = writeln!(out, "  - {}", n.id);
    }
    out
}

/// Cap `text` at `budget` tokens (chars/4). Truncation is explicit — never silent (criterion 4).
fn cap_to_budget(text: String, budget: Option<usize>) -> String {
    let Some(b) = budget else { return text };
    if tokens(&text) <= b {
        return text;
    }
    let char_budget = b.saturating_mul(4);
    let mut kept = String::new();
    for line in text.lines() {
        if kept.chars().count() + line.chars().count() + 1 > char_budget {
            break;
        }
        kept.push_str(line);
        kept.push('\n');
    }
    kept.push_str("… truncated at budget; narrow with `atlas node <id>`\n");
    kept
}
