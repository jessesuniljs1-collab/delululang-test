//! Human/agent text formats for the atlas: the `tree` overview and the token-budgeted `digest`.
//!
//! Both are deterministic: they walk the already-id-sorted node and edge vectors, so two runs over
//! the same atlas produce byte-identical output (criterion 1). Color is applied by the caller's
//! Palette in phase A3; the strings produced here are plain and stable.

use std::fmt::Write as _;

use serde_json::Value;

use crate::model::{Atlas, EdgeKind, NodeKind};

/// The fixed footer that ends every digest — it teaches the query verbs so any agent that reads
/// `ATLAS.md` learns to drill down without being told (addendum §2.2 / criterion 3).
pub const QUERYING_FOOTER: &str = "\
## Querying further

This digest is a summary of a larger graph. To drill down without loading the whole graph, ask the \
atlas directly:

- `delulu atlas node <name-or-id>`      — one node: kind, effects, authority, and its edges
- `delulu atlas callers <fn>`           — which functions call this one
- `delulu atlas calls <fn>`             — which functions this one calls
- `delulu atlas path <A> <B>`           — the shortest typed path between two nodes
- `delulu atlas why <Effect|resource>`  — which functions perform/require it, and through which calls

Add `--json` for a machine answer, or `--budget <N>` to cap any answer at ~N tokens (a chars/4 \
heuristic).";

/// The default digest budget, in tokens (a chars/4 heuristic — stated as such wherever reported).
pub const DEFAULT_BUDGET: usize = 2000;

/// Estimate tokens as chars/4 (the stated heuristic).
pub fn tokens(s: &str) -> usize {
    s.chars().count() / 4
}

impl Atlas {
    pub(crate) fn functions(&self) -> impl Iterator<Item = &crate::model::Node> {
        self.nodes.iter().filter(|n| n.kind == NodeKind::Function)
    }

    /// The plain `tree` overview for a terminal (color is layered on by the caller in A3).
    pub fn render_tree(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(
            out,
            "atlas: {}  ({} nodes, {} edges)",
            self.root,
            self.nodes.len(),
            self.edges.len()
        );
        let _ = writeln!(out);

        for pkg in self.nodes.iter().filter(|n| n.kind == NodeKind::Package) {
            let _ = writeln!(out, "package {}", pkg.name);
            let mods: Vec<&crate::model::Node> = self
                .out_edges(&pkg.id)
                .filter(|e| e.kind == EdgeKind::Contains)
                .filter_map(|e| self.node(&e.to))
                .filter(|n| n.kind == NodeKind::Module)
                .collect();
            for m in mods {
                let fns: Vec<&crate::model::Node> = self
                    .out_edges(&m.id)
                    .filter(|e| e.kind == EdgeKind::Contains)
                    .filter_map(|e| self.node(&e.to))
                    .filter(|n| n.kind == NodeKind::Function)
                    .collect();
                let _ = writeln!(out, "  module {}  ({} fn)", m.name, fns.len());
                for f in fns {
                    let row = if f.effects.is_empty() {
                        "!{}".to_string()
                    } else {
                        format!("!{{{}}}", f.effects.join(", "))
                    };
                    let tag = if f.pure == Some(true) { "  [pure]" } else { "" };
                    let _ = writeln!(out, "    fn {} {}{}", f.name, row, tag);
                }
            }
        }

        let _ = writeln!(out);
        let _ = writeln!(out, "god nodes (top {} by degree):", self.gods.len());
        for g in &self.gods {
            let _ = writeln!(out, "  {:>3}  {} {}  ({})", g.degree, g.kind.as_str(), g.name, g.id);
        }

        let _ = writeln!(out);
        let _ = write!(out, "{}", self.authority_block("authority (mirrors `delulu authority`):"));

        let _ = writeln!(out);
        let _ = writeln!(out, "caveats:");
        for c in &self.caveats {
            let _ = writeln!(out, "  - {c}");
        }
        out
    }

    /// A compact authority section shared by tree + digest, rendered from the embedded report so it
    /// mirrors `delulu authority` exactly.
    fn authority_block(&self, header: &str) -> String {
        let a = &self.authority;
        let mut out = String::new();
        let _ = writeln!(out, "{header}");
        let effects = strs(a.get("effects"));
        let _ = writeln!(
            out,
            "  effects:      {}",
            if effects.is_empty() { "(none — provably pure)".into() } else { effects.join(", ") }
        );
        if let Some(caps) = a.get("capabilities").and_then(|c| c.as_array()) {
            if caps.is_empty() {
                let _ = writeln!(out, "  capabilities: (none)");
            } else {
                let _ = writeln!(out, "  capabilities:");
                for c in caps {
                    let kind = c.get("kind").and_then(|k| k.as_str()).unwrap_or("?");
                    let scopes = strs(c.get("scopes"));
                    let scope_str =
                        if scopes.is_empty() { "(scope granted at runtime)".into() } else { scopes.join(", ") };
                    let _ = writeln!(out, "    - {kind:<8} {scope_str}");
                }
            }
        }
        let secrets = strs(a.get("secrets"));
        if !secrets.is_empty() {
            let _ = writeln!(out, "  secrets:      {}", secrets.join(", "));
        }
        out
    }

    /// The foreign-boundary list for the digest, read from the authority report's `foreign_calls`
    /// (the single place the proof's holes are enumerated). Empty ⇒ no section.
    fn foreign_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(fc) = self.authority.get("foreign_calls").and_then(|c| c.as_array()) {
            for e in fc {
                let abi = e.get("abi").and_then(|a| a.as_str()).unwrap_or("?");
                if abi == "python" {
                    let allow = strs(e.get("allowlist"));
                    let seen = strs(e.get("imports_seen"));
                    lines.push(format!(
                        "- python: allowlist [{}]; imports seen [{}]",
                        allow.join(", "),
                        seen.join(", ")
                    ));
                } else {
                    let lib = e.get("lib").and_then(|l| l.as_str()).unwrap_or("?");
                    let syms = strs(e.get("symbols"));
                    lines.push(format!("- {abi} lib {lib}: {}", syms.join(", ")));
                }
            }
        }
        lines
    }

    /// The token-budgeted Markdown digest (addendum §2.3). `--budget` caps the WHOLE output at
    /// ~`budget` tokens (chars/4, stated as heuristic). An irreducible floor is always kept — the
    /// title line, the explicit truncation notice, and the fixed "Querying further" footer (the
    /// footer IS the remedy pointer: a truncated digest that lost it would strand the agent). Body
    /// lines are packed in document order into what remains; anything dropped is announced —
    /// explicit over silent, always (criteria 3 & 4; A3.2).
    pub fn render_digest(&self, budget: usize) -> String {
        let packages: Vec<&crate::model::Node> =
            self.nodes.iter().filter(|n| n.kind == NodeKind::Package).collect();
        let modules: Vec<&crate::model::Node> =
            self.nodes.iter().filter(|n| n.kind == NodeKind::Module).collect();

        // ----- the floor: title + footer (notice joins them only when truncating) ---------------
        let title = format!("# Atlas of `{}`\n\n", self.root);

        let mut head = String::new();
        let _ = writeln!(
            head,
            "{} packages, {} modules, {} functions, {} nodes, {} edges.",
            packages.len(),
            modules.len(),
            self.functions().count(),
            self.nodes.len(),
            self.edges.len()
        );
        let _ = writeln!(head);
        let _ = writeln!(head, "> Caveats:");
        for c in &self.caveats {
            let _ = writeln!(head, "> - {c}");
        }
        let _ = writeln!(head, "> - the token budget below is a chars/4 heuristic.");
        let _ = writeln!(head);

        let mut gods = String::new();
        let _ = writeln!(gods, "## God nodes (most-connected, top {})", self.gods.len());
        let _ = writeln!(gods);
        let _ = writeln!(gods, "| degree | kind | name | id |");
        let _ = writeln!(gods, "|---|---|---|---|");
        for g in &self.gods {
            let _ = writeln!(gods, "| {} | {} | {} | `{}` |", g.degree, g.kind.as_str(), g.name, g.id);
        }
        let _ = writeln!(gods);

        let mut auth = String::new();
        let _ = writeln!(auth, "## Authority (mirrors `delulu authority`)");
        let _ = writeln!(auth);
        let _ = writeln!(auth, "```");
        let _ = write!(auth, "{}", self.authority_block(&format!("program `{}`:", self.root)));
        let _ = writeln!(auth, "```");
        let _ = writeln!(auth);

        let foreign_lines = self.foreign_lines();
        let mut foreign = String::new();
        if !foreign_lines.is_empty() {
            let _ = writeln!(foreign, "## Foreign boundary (outside the effect proof)");
            let _ = writeln!(foreign);
            for l in &foreign_lines {
                let _ = writeln!(foreign, "{l}");
            }
            let _ = writeln!(foreign);
        }

        // ----- droppable section: the per-package / per-module map -------------------------------
        let mut map = String::new();
        let _ = writeln!(map, "## Structure");
        let _ = writeln!(map);
        for p in &packages {
            let _ = writeln!(map, "- package `{}`", p.name);
        }
        for m in &modules {
            let fns: Vec<&crate::model::Node> = self
                .out_edges(&m.id)
                .filter(|e| e.kind == EdgeKind::Contains)
                .filter_map(|e| self.node(&e.to))
                .filter(|n| n.kind == NodeKind::Function)
                .collect();
            let mut effs: std::collections::BTreeSet<&str> = Default::default();
            for f in &fns {
                for e in &f.effects {
                    effs.insert(e.as_str());
                }
            }
            let row = if effs.is_empty() {
                "!{}".to_string()
            } else {
                format!("!{{{}}}", effs.into_iter().collect::<Vec<_>>().join(", "))
            };
            let _ = writeln!(map, "  - module `{}` — {} fn — {}", m.name, fns.len(), row);
        }
        let _ = writeln!(map);

        // ----- assemble under the budget --------------------------------------------------------
        let body = format!("{head}{gods}{auth}{foreign}{map}");
        let footer = format!("{QUERYING_FOOTER}\n");
        let full = format!("{title}{body}{footer}");
        if tokens(&full) <= budget {
            return full;
        }

        // Over budget: keep the floor (title + notice + footer), pack whole body LINES in document
        // order into what remains. The notice length is reserved up front so the final assembly
        // respects the cap as far as the floor allows.
        const NOTICE_RESERVE: usize = 260;
        let char_budget = budget.saturating_mul(4);
        let remaining =
            char_budget.saturating_sub(title.chars().count() + footer.chars().count() + NOTICE_RESERVE);
        let mut kept = String::new();
        let mut kept_lines = 0usize;
        let total_lines = body.split_inclusive('\n').count();
        for line in body.split_inclusive('\n') {
            if kept.chars().count() + line.chars().count() > remaining {
                break;
            }
            kept.push_str(line);
            kept_lines += 1;
        }
        // Explicit, never silent (§2.2): the notice names the budget, states the heuristic, counts
        // what was dropped, and points at the remedies. When even the floor exceeds the budget the
        // floor is STILL emitted — and the notice says exactly that.
        let dropped = total_lines - kept_lines;
        let notice = if kept_lines == 0 {
            format!(
                "… truncated at budget (~{budget} tokens, chars/4 heuristic): the budget is below \
                 the digest floor (title + this notice + the querying footer), so the floor is \
                 emitted anyway — explicit over silent. All {total_lines} body line(s) omitted; \
                 narrow with `atlas node <id>` or raise `--budget`.\n\n"
            )
        } else {
            format!(
                "… truncated at budget (~{budget} tokens, chars/4 heuristic): {dropped} of \
                 {total_lines} body line(s) omitted; narrow with `atlas node <id>` or raise \
                 `--budget`.\n\n"
            )
        };
        format!("{title}{kept}{notice}{footer}")
    }
}

fn strs(v: Option<&Value>) -> Vec<String> {
    v.and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}
