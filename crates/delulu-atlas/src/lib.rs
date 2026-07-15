//! The Atlas — a typed, deterministic graph of a checked Delulu program, derived ONLY from
//! compiler (and optionally broker) facts (Surface addendum §2).
//!
//! Consumers and their native formats:
//! - human, terminal → `render_tree` (the default overview)
//! - AI agent / LLM  → `render_digest` (token-budgeted Markdown) + the query verbs + `to_json`
//! - other tools     → `dot` / `mermaid` (phase A3)
//! - human, browser  → self-contained `html` (phase A3)
//!
//! Determinism is structural: [`Atlas::build`] sorts every collection by stable id, so with color
//! off two runs produce byte-identical output in every format. The `atlas/1` JSON is a versioned
//! machine channel and is never colored.

mod build;
mod formats;
mod model;
mod query;
mod render;

pub use build::{recompute_gods, BuildInput, ModuleView, PackageView};
pub use formats::HTML_NODE_CAP;
pub use model::{
    effect_id, fn_id, foreign_c_id, foreign_py_id, grant_id, mod_id, pkg_id, resource_id, type_id,
    Atlas, Edge, EdgeKind, God, Node, NodeKind, SpanLoc, CAVEAT_CUSTODY, CAVEAT_STATIC, SCHEMA,
};
pub use query::Resolved;
pub use render::{tokens, DEFAULT_BUDGET, QUERYING_FOOTER};

#[cfg(test)]
mod tests {
    use super::*;
    use delulu_check::authority::ScopeInfo;
    use delulu_check::{authority_report, check_source};

    /// Build an atlas for a single checked source file (the same shape the CLI's file path uses).
    fn atlas_of(src: &str) -> Atlas {
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "test source must check clean: {:?}", checked.diagnostics);
        let root = checked.module.name.dotted();
        // Synthesize a single-module Program (Program fields are all pub).
        let mut facts = std::collections::HashMap::new();
        for (name, f) in &checked.result.facts {
            facts.insert(format!("{root}::{name}"), f.clone());
        }
        let mut owner = std::collections::HashMap::new();
        for name in checked.table.fns.keys() {
            owner.insert(name.clone(), root.clone());
        }
        let mut call_owner = std::collections::HashMap::new();
        call_owner.insert(root.clone(), owner);
        let entry_module = if checked.result.main_present { Some(root.clone()) } else { None };
        let program = delulu_check::program::Program {
            diagnostics: vec![],
            facts,
            fn_types: std::collections::HashMap::new(),
            call_owner,
            entry_module,
        };
        let scopes = ScopeInfo::default();
        let authority = authority_report(&root, &checked.result, &scopes);
        let map = &{
            let mut m = delulu_diag::SourceMap::new();
            m.add_file("test.delulu", src.to_string());
            m
        };
        Atlas::build(BuildInput {
            root: root.clone(),
            packages: vec![PackageView { name: root.clone(), deps: vec![], is_root: true }],
            modules: vec![ModuleView { package: root.clone(), name: root.clone(), module: &checked.module }],
            program: &program,
            source_map: map,
            authority,
            god_n: 10,
            custody: None,
        })
    }

    const SAMPLE: &str = "module app\n\
        fn helper(out: Cap[Console], n: Str) ! {Write} { out.println(n) }\n\
        fn fib(n: Int) -> Int { if n < 2 { n } else { fib(n-1) + fib(n-2) } }\n\
        fn main(root: Root) ! {Write} { let out = root.console()\n helper(out, \"hi\") }\n";

    #[test]
    fn build_is_deterministic() {
        let a = atlas_of(SAMPLE);
        let b = atlas_of(SAMPLE);
        assert_eq!(a.to_json_string(), b.to_json_string(), "json is byte-identical across runs");
        assert_eq!(a.render_tree(), b.render_tree(), "tree is byte-identical across runs");
        assert_eq!(a.render_digest(DEFAULT_BUDGET), b.render_digest(DEFAULT_BUDGET), "digest byte-identical");
    }

    #[test]
    fn functions_effects_and_calls_are_edges() {
        let a = atlas_of(SAMPLE);
        // main calls helper (calls edge).
        let main = fn_id("app", "app", "main");
        let helper = fn_id("app", "app", "helper");
        assert!(a.edges.iter().any(|e| e.from == main && e.to == helper && e.kind == EdgeKind::Calls));
        // helper performs Write.
        assert!(a
            .edges
            .iter()
            .any(|e| e.from == helper && e.to == effect_id("Write") && e.kind == EdgeKind::Performs));
        // fib is pure.
        let fib = a.node(&fn_id("app", "app", "fib")).unwrap();
        assert_eq!(fib.pure, Some(true));
    }

    #[test]
    fn authority_parity_effects_match_delulu_authority() {
        // Criterion 5: the union of `performs` over reachable functions equals `delulu authority`.
        let a = atlas_of(SAMPLE);
        let auth_effects: std::collections::BTreeSet<String> = a.authority["effects"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        // Reachable functions from main via calls edges.
        let main = fn_id("app", "app", "main");
        let mut reach = std::collections::BTreeSet::new();
        let mut stack = vec![main.clone()];
        while let Some(f) = stack.pop() {
            if !reach.insert(f.clone()) {
                continue;
            }
            for e in a.out_edges(&f).filter(|e| e.kind == EdgeKind::Calls) {
                stack.push(e.to.clone());
            }
        }
        let mut performs: std::collections::BTreeSet<String> = Default::default();
        for e in a.edges.iter().filter(|e| e.kind == EdgeKind::Performs) {
            if reach.contains(&e.from) {
                if let Some(n) = a.node(&e.to) {
                    performs.insert(n.name.clone());
                }
            }
        }
        assert_eq!(performs, auth_effects, "performs edges must agree with delulu authority");
    }

    #[test]
    fn digest_has_footer_gods_authority_and_caveat() {
        let a = atlas_of(SAMPLE);
        let d = a.render_digest(DEFAULT_BUDGET);
        assert!(d.contains("## Querying further"), "footer present");
        assert!(d.contains("delulu atlas node"), "footer teaches the verbs");
        assert!(d.contains("## God nodes"), "gods present");
        assert!(d.contains("Authority (mirrors `delulu authority`)"), "authority table present");
        assert!(d.contains(CAVEAT_STATIC), "the verbatim static caveat ships");
        assert!(tokens(&d) <= DEFAULT_BUDGET, "digest respects the default budget");
    }

    #[test]
    fn query_verbs_answer_from_the_graph() {
        let a = atlas_of(SAMPLE);
        assert!(a.query_callers("helper", None).contains("main"), "callers of helper include main");
        assert!(a.query_calls("main", None).contains("helper"), "main calls helper");
        let path = a.query_path("main", "Write", None);
        assert!(path.contains("--calls-->") || path.contains("--performs-->"), "typed hops: {path}");
        let why = a.query_why("Write", None);
        assert!(why.contains("helper") || why.contains("main"), "why Write names a performer: {why}");
        let node = a.query_node("fib", None);
        assert!(node.contains("pure:      true"), "node view shows purity: {node}");
    }

    #[test]
    fn budget_truncation_is_explicit() {
        let a = atlas_of(SAMPLE);
        // A tiny budget forces truncation on a verb with several lines.
        let capped = a.query_node("main", Some(1));
        assert!(capped.contains("truncated at budget"), "truncation is explicit: {capped}");
    }

    /// A3.2: the digest honors `--budget` — the floor (title + notice + footer) is always kept,
    /// body lines are packed into what remains, and truncation is explicit, never silent.
    #[test]
    fn digest_honors_the_budget_with_floor_and_explicit_notice() {
        let a = atlas_of(SAMPLE);
        let full = a.render_digest(DEFAULT_BUDGET);
        assert!(!full.contains("truncated at budget"), "under budget ⇒ no notice");

        // A tiny budget (below the floor): the floor is STILL emitted and the notice says so.
        let tiny = a.render_digest(10);
        assert!(tiny.starts_with("# Atlas of `app`"), "title survives: {tiny}");
        assert!(tiny.contains("truncated at budget"), "explicit: {tiny}");
        assert!(tiny.contains("below the digest floor"), "the floor case is named: {tiny}");
        assert!(tiny.contains("## Querying further"), "the footer (the remedy pointer) survives");
        assert!(tiny.len() < full.len(), "smaller than the full digest");
        assert_eq!(tiny, a.render_digest(10), "deterministic");

        // A mid budget (floor fits): partial body + notice, and the cap actually binds.
        let mid = a.render_digest(300);
        assert!(mid.contains("truncated at budget"), "explicit: {mid}");
        assert!(mid.contains("body line(s) omitted"), "counts what was dropped: {mid}");
        assert!(mid.contains("## Querying further"));
        assert!(tokens(&mid) <= 300, "the cap binds: {} tokens", tokens(&mid));
        assert!(mid.len() > tiny.len(), "a larger budget keeps more body");
    }

    #[test]
    fn foreign_c_symbols_get_nodes_and_gated_edges() {
        // A declared C symbol always gets a node; the edge appears only for a function whose
        // CHECKED row carries ForeignCall and whose body calls the symbol (A3.1).
        let a = atlas_of(
            "module m\n\
             foreign \"c\" lib mathlib { fn cos(x: Float) -> Float\n fn sin(x: Float) -> Float }\n\
             fn compute(h: mathlib) -> Float ! {ForeignCall} { h.cos(1.0) }\n\
             fn pure_math(n: Int) -> Int { n + 1 }\n",
        );
        // Both declared symbols have nodes (checked declarations), even the uncalled `sin`.
        let cos = a.node(&foreign_c_id("cos")).expect("cos node");
        assert_eq!(cos.kind, NodeKind::Foreign);
        assert_eq!(cos.pattern.as_deref(), Some("c lib mathlib"), "the declaring lib is named");
        assert!(a.node(&foreign_c_id("sin")).is_some(), "declared-but-uncalled symbol still a node");
        // compute → cos edge, kind foreign.
        let compute = fn_id("m", "m", "compute");
        assert!(a
            .edges
            .iter()
            .any(|e| e.from == compute && e.to == foreign_c_id("cos") && e.kind == EdgeKind::Foreign));
        // No edge to the uncalled symbol, and none from the pure function.
        assert!(!a.edges.iter().any(|e| e.to == foreign_c_id("sin") && e.kind == EdgeKind::Foreign));
        let pure = fn_id("m", "m", "pure_math");
        assert!(!a.edges.iter().any(|e| e.from == pure && e.kind == EdgeKind::Foreign));
        // Query verbs see the boundary: a typed path routes through it.
        let path = a.query_path("compute", "cos", None);
        assert!(path.contains("--foreign-->"), "typed foreign hop: {path}");
    }

    #[test]
    fn python_imports_get_nodes_and_edges() {
        let a = atlas_of(
            "module m\n\
             fn work(py: Cap[Python]) -> Result[Float, PyErr] ! {ForeignCall} {\n\
               let np = py.import(\"numpy\")?\n\
               py.to_float(np.call_method(\"mean\", [py.list([py.of_float(1.0)])])?) }\n",
        );
        let numpy = a.node(&foreign_py_id("numpy")).expect("foreign:py:numpy node");
        assert_eq!(numpy.kind, NodeKind::Foreign);
        assert_eq!(numpy.name, "numpy");
        let work = fn_id("m", "m", "work");
        assert!(a
            .edges
            .iter()
            .any(|e| e.from == work && e.to == foreign_py_id("numpy") && e.kind == EdgeKind::Foreign));
        // `call_method("mean", …)` is a PyObj call, not an import — no `foreign:py:mean` node.
        assert!(a.node(&foreign_py_id("mean")).is_none(), "only literal imports mint py nodes");
        // Determinism: nodes remain id-sorted with the foreign nodes in place.
        let ids: Vec<&str> = a.nodes.iter().map(|n| n.id.as_str()).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn attach_custody_adds_grants_delegates_and_caveat() {
        let mut a = atlas_of(SAMPLE);
        assert_eq!(a.custody, None, "no overlay unless attached");
        let overlay = serde_json::json!({ "grants": [
            { "id": "g_root", "parent": null, "state": "active", "authority": "Read+Write" },
            { "id": "g_agent", "parent": "g_root", "state": "active", "authority": "Read" },
        ]});
        a.attach_custody(overlay, 10);
        assert!(a.node("grant:g_root").is_some(), "grant nodes added");
        assert!(a
            .edges
            .iter()
            .any(|e| e.from == "grant:g_root" && e.to == "grant:g_agent" && e.kind == EdgeKind::Delegates));
        assert!(a.caveats.iter().any(|c| c == CAVEAT_CUSTODY), "the custody caveat ships verbatim");
        assert!(a.custody.is_some());
        // Determinism preserved: nodes still id-sorted.
        let ids: Vec<&str> = a.nodes.iter().map(|n| n.id.as_str()).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted, "nodes remain id-sorted after the overlay");
    }
}
