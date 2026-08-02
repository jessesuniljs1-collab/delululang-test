//! Tool + browser formats: `dot` (Graphviz), `mermaid` (module-level), and a self-contained
//! `html` visualization. All three walk the id-sorted node/edge vectors, so they are deterministic.
//!
//! The HTML is a SINGLE self-contained file: data + JS are inlined, there are ZERO external URLs
//! (no CDN, no web-font, not even an SVG namespace — it draws on a `<canvas>`), and above 3000
//! nodes it collapses to module level with an explicit notice (criterion 7).

use std::fmt::Write as _;

use crate::model::{Atlas, EdgeKind, Node, NodeKind};

/// Above this many nodes the HTML collapses to a module-level view (criterion 7).
pub const HTML_NODE_CAP: usize = 3000;

/// A stable color per node kind (used by dot + html; hand-picked, colorblind-aware).
fn kind_color(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Package => "#6a4c93",
        NodeKind::Module => "#1982c4",
        NodeKind::Function => "#2a9d8f",
        NodeKind::Type => "#8ac926",
        NodeKind::Effect => "#ff924c",
        NodeKind::Resource => "#ff595e",
        NodeKind::Foreign => "#b5651d",
        NodeKind::Grant => "#c77dff",
    }
}

/// A function's dominant-effect color (the HTML colors functions "by effect row" — §1 adopt/improve).
fn effect_color(effects: &[String]) -> &'static str {
    // Priority order: the most authority-bearing effect wins the color.
    for (name, color) in [
        ("Declassify", "#9d4edd"),
        ("ForeignCall", "#b5651d"),
        ("Net", "#ff595e"),
        ("Write", "#ff924c"),
        ("Read", "#1982c4"),
    ] {
        if effects.iter().any(|e| e == name) {
            return color;
        }
    }
    "#2a9d8f" // pure / other
}

impl Atlas {
    /// Graphviz DOT. One node per atlas node (shape/color by kind), one edge per atlas edge
    /// (labeled with the edge kind; authority edges dashed).
    pub fn render_dot(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "digraph atlas {{");
        let _ = writeln!(out, "  rankdir=LR;");
        let _ = writeln!(out, "  node [fontname=\"monospace\", style=filled];");
        let _ = writeln!(out, "  labelloc=\"t\";");
        let _ = writeln!(out, "  label=\"atlas: {}\";", dot_escape(&self.root));
        for n in &self.nodes {
            let shape = match n.kind {
                NodeKind::Package => "box3d",
                NodeKind::Module => "box",
                NodeKind::Function => "ellipse",
                NodeKind::Type => "component",
                NodeKind::Effect => "diamond",
                NodeKind::Resource => "note",
                NodeKind::Foreign => "hexagon",
                NodeKind::Grant => "octagon",
            };
            let color = if n.kind == NodeKind::Function {
                effect_color(&n.effects)
            } else {
                kind_color(n.kind)
            };
            let _ = writeln!(
                out,
                "  \"{}\" [label=\"{}\", shape={shape}, fillcolor=\"{color}\"];",
                dot_escape(&n.id),
                dot_escape(&n.name)
            );
        }
        for e in &self.edges {
            let dashed = matches!(
                e.kind,
                EdgeKind::Performs | EdgeKind::Requires | EdgeKind::Declassifies | EdgeKind::Delegates
            );
            let style = if dashed { ", style=dashed, color=\"#888888\"" } else { "" };
            let _ = writeln!(
                out,
                "  \"{}\" -> \"{}\" [label=\"{}\"{style}];",
                dot_escape(&e.from),
                dot_escape(&e.to),
                e.kind.as_str()
            );
        }
        let _ = writeln!(out, "}}");
        out
    }

    /// Mermaid, at MODULE level (packages + modules + their depends_on/imports/contains). The
    /// function-level detail belongs in tree/json/html; mermaid is the shareable overview.
    pub fn render_mermaid(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "graph LR");
        // Say what this diagram is, inside the diagram (`HARDENING_CAMPAIGN.md` C36).
        //
        // Module level is the right scope for a shareable overview, and it is documented — in a
        // design addendum. The problem is where a mermaid diagram ENDS UP: pasted into a README, an
        // issue, or an agent's context, permanently separated from the command that produced it and
        // from any documentation about it. A reader then sees two boxes for a program with twelve
        // nodes and twenty-three edges, and the honest conclusion available to them is that the
        // program has no functions and no effects.
        //
        // A graph that under-reports authority and does not say so is the same defect as C23's inert
        // declarations and C34's phantom ceiling: safe, and misleading. So the artifact describes
        // itself. `%%` is a mermaid comment; renderers drop it, humans and agents reading the source
        // see it, and it comes after `graph LR` so the declaration stays first.
        let _ = writeln!(
            out,
            "  %% MODULE-LEVEL overview: packages and modules only. Functions, effects, capabilities,"
        );
        let _ = writeln!(
            out,
            "  %% and call edges are NOT shown here — use `--format dot`, `json`, or `tree` for those."
        );
        // Stable short ids `n0, n1, …` for the module-level nodes, assigned in id order.
        let mut short: std::collections::BTreeMap<&str, String> = std::collections::BTreeMap::new();
        for (idx, n) in self
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, NodeKind::Package | NodeKind::Module))
            .enumerate()
        {
            let sid = format!("n{idx}");
            let kindword = if n.kind == NodeKind::Package { "package" } else { "module" };
            let _ = writeln!(out, "  {sid}[\"{kindword} {}\"]", mermaid_escape(&n.name));
            short.insert(n.id.as_str(), sid);
        }
        for e in &self.edges {
            if let (Some(a), Some(b)) = (short.get(e.from.as_str()), short.get(e.to.as_str())) {
                if matches!(e.kind, EdgeKind::DependsOn | EdgeKind::Imports | EdgeKind::Contains) {
                    let _ = writeln!(out, "  {a} -->|{}| {b}", e.kind.as_str());
                }
            }
        }
        out
    }

    /// A single self-contained HTML file (data + JS inlined, zero external URLs). Above
    /// [`HTML_NODE_CAP`] nodes it collapses to module level with a visible notice (criterion 7).
    pub fn render_html(&self) -> String {
        let collapsed = self.nodes.len() > HTML_NODE_CAP;
        // The data actually embedded: full graph, or a module-level reduction when collapsed.
        let (nodes, edges): (Vec<&Node>, Vec<&crate::model::Edge>) = if collapsed {
            let ns: Vec<&Node> = self
                .nodes
                .iter()
                .filter(|n| matches!(n.kind, NodeKind::Package | NodeKind::Module))
                .collect();
            let keep: std::collections::BTreeSet<&str> = ns.iter().map(|n| n.id.as_str()).collect();
            let es: Vec<&crate::model::Edge> = self
                .edges
                .iter()
                .filter(|e| keep.contains(e.from.as_str()) && keep.contains(e.to.as_str()))
                .filter(|e| matches!(e.kind, EdgeKind::DependsOn | EdgeKind::Imports | EdgeKind::Contains))
                .collect();
            (ns, es)
        } else {
            (self.nodes.iter().collect(), self.edges.iter().collect())
        };

        // Build the embedded data object by hand (kind color + effect color precomputed in Rust so
        // the JS stays tiny and deterministic).
        let mut data_nodes = String::from("[");
        for (i, n) in nodes.iter().enumerate() {
            if i > 0 {
                data_nodes.push(',');
            }
            let color = if n.kind == NodeKind::Function { effect_color(&n.effects) } else { kind_color(n.kind) };
            let _ = write!(
                data_nodes,
                "{{\"id\":{},\"name\":{},\"kind\":{},\"color\":\"{}\",\"effects\":{}}}",
                js_str(&n.id),
                js_str(&n.name),
                js_str(n.kind.as_str()),
                color,
                js_str(&n.effects.join(", "))
            );
        }
        data_nodes.push(']');
        let mut data_edges = String::from("[");
        for (i, e) in edges.iter().enumerate() {
            if i > 0 {
                data_edges.push(',');
            }
            let auth = matches!(
                e.kind,
                EdgeKind::Performs | EdgeKind::Requires | EdgeKind::Declassifies | EdgeKind::Delegates
            );
            let _ = write!(
                data_edges,
                "{{\"from\":{},\"to\":{},\"kind\":{},\"auth\":{}}}",
                js_str(&e.from),
                js_str(&e.to),
                js_str(e.kind.as_str()),
                auth
            );
        }
        data_edges.push(']');

        let notice = if collapsed {
            format!(
                "<div class=\"notice\">This program has {} nodes (over the {} cap) — the map is \
                 collapsed to package + module level. Use <code>delulu atlas node &lt;id&gt;</code> \
                 to drill into a module.</div>",
                self.nodes.len(),
                HTML_NODE_CAP
            )
        } else {
            String::new()
        };
        let caveats: String = self.caveats.iter().map(|c| format!("<li>{}</li>", html_escape(c))).collect();

        // NOTE: no `http(s)://` anywhere — inline canvas rendering, no SVG namespace, no web fonts.
        format!(
            "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
             <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
             <title>Atlas of {root}</title>\n<style>{css}</style>\n</head>\n<body>\n\
             <header><h1>Atlas of <code>{root}</code></h1>\
             <p>{ncount} nodes, {ecount} edges. A static map of checked facts — not a runtime trace.</p></header>\n\
             {notice}\n<canvas id=\"c\" width=\"1200\" height=\"760\"></canvas>\n\
             <aside id=\"legend\"></aside>\n<div id=\"tip\"></div>\n\
             <section class=\"caveats\"><h2>Caveats</h2><ul>{caveats}</ul></section>\n\
             <script>\nconst ATLAS={{nodes:{nodes},edges:{edges}}};\nconst COLLAPSED={collapsed};\n{js}\n</script>\n\
             </body>\n</html>\n",
            root = html_escape(&self.root),
            ncount = self.nodes.len(),
            ecount = self.edges.len(),
            css = HTML_CSS,
            notice = notice,
            caveats = caveats,
            nodes = data_nodes,
            edges = data_edges,
            collapsed = collapsed,
            js = HTML_JS,
        )
    }
}

fn dot_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
fn mermaid_escape(s: &str) -> String {
    s.replace('"', "&quot;").replace('[', "(").replace(']', ")")
}
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
/// Emit a JSON/JS string literal (escaping quotes/backslashes/controls). No URL can appear here —
/// ids and names are compiler tokens.
fn js_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

const HTML_CSS: &str = "\
:root{color-scheme:light dark}\
body{margin:0;font-family:system-ui,sans-serif;background:#fbfbfd;color:#111}\
@media(prefers-color-scheme:dark){body{background:#0e0e12;color:#e8e8ee}}\
header{padding:16px 20px}h1{margin:0 0 4px;font-size:1.3rem}header p{margin:0;opacity:.7;font-size:.85rem}\
canvas{display:block;margin:0 auto;max-width:100%;background:transparent}\
.notice{margin:8px 20px;padding:10px 14px;border-left:4px solid #ff924c;background:#ff924c22;font-size:.9rem}\
#legend{display:flex;flex-wrap:wrap;gap:10px;padding:8px 20px;font-size:.8rem}\
#legend span{display:inline-flex;align-items:center;gap:5px}\
#legend i{width:11px;height:11px;border-radius:50%;display:inline-block}\
#tip{position:fixed;pointer-events:none;padding:5px 8px;border-radius:6px;background:#000c;color:#fff;font:12px monospace;opacity:0;transition:opacity .1s}\
.caveats{padding:8px 20px 24px;font-size:.8rem;opacity:.8}.caveats h2{font-size:.9rem}";

const HTML_JS: &str = r#"
(function(){
  const cv=document.getElementById('c'), ctx=cv.getContext('2d'), tip=document.getElementById('tip');
  const layers=['package','module','type','function','effect','resource','foreign','grant'];
  const byId={}; ATLAS.nodes.forEach(n=>byId[n.id]=n);
  // Deterministic layered layout: x by kind-layer, y by index within layer (nodes are id-sorted).
  const cols={}; layers.forEach(l=>cols[l]=[]);
  ATLAS.nodes.forEach(n=>{(cols[n.kind]||(cols[n.kind]=[])).push(n)});
  const W=cv.width, H=cv.height, used=layers.filter(l=>cols[l]&&cols[l].length);
  const colW=W/(used.length||1);
  used.forEach((l,li)=>{const arr=cols[l]; const gap=H/(arr.length+1);
    arr.forEach((n,i)=>{n.x=colW*li+colW/2; n.y=gap*(i+1);});});
  function draw(){
    ctx.clearRect(0,0,W,H);
    ATLAS.edges.forEach(e=>{const a=byId[e.from],b=byId[e.to]; if(!a||!b)return;
      ctx.beginPath(); ctx.moveTo(a.x,a.y); ctx.lineTo(b.x,b.y);
      ctx.strokeStyle=e.auth?'#ff595e88':'#88888855'; ctx.lineWidth=1;
      if(e.auth){ctx.setLineDash([4,3]);}else{ctx.setLineDash([]);} ctx.stroke();});
    ctx.setLineDash([]);
    ATLAS.nodes.forEach(n=>{ctx.beginPath(); ctx.arc(n.x,n.y,6,0,7); ctx.fillStyle=n.color; ctx.fill();
      ctx.fillStyle=getComputedStyle(document.body).color; ctx.font='10px monospace';
      ctx.fillText(n.name,n.x+8,n.y+3);});
  }
  draw();
  cv.addEventListener('mousemove',ev=>{const r=cv.getBoundingClientRect();
    const sx=cv.width/r.width, sy=cv.height/r.height, mx=(ev.clientX-r.left)*sx, my=(ev.clientY-r.top)*sy;
    let hit=null; ATLAS.nodes.forEach(n=>{if((n.x-mx)**2+(n.y-my)**2<64)hit=n;});
    if(hit){tip.style.opacity=1; tip.style.left=(ev.clientX+12)+'px'; tip.style.top=(ev.clientY+12)+'px';
      tip.textContent=hit.kind+' '+hit.id+(hit.effects?'  !{'+hit.effects+'}':'');}
    else{tip.style.opacity=0;}});
  const seen={}; ATLAS.nodes.forEach(n=>{seen[n.kind]=n.color;});
  document.getElementById('legend').innerHTML=Object.keys(seen).sort().map(k=>
    '<span><i style="background:'+seen[k]+'"></i>'+k+'</span>').join('')+
    '<span><i style="background:#ff595e"></i>authority edge (dashed)</span>';
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Atlas, Edge, EdgeKind, Node, NodeKind, SCHEMA};

    fn tiny() -> Atlas {
        let mut nodes = vec![
            Node::new("pkg:app", NodeKind::Package, "app"),
            Node::new("mod:app/app", NodeKind::Module, "app"),
            {
                let mut f = Node::new("fn:app/app.main", NodeKind::Function, "main");
                f.effects = vec!["Write".to_string()];
                f
            },
            Node::new("effect:Write", NodeKind::Effect, "Write"),
        ];
        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        let edges = vec![
            Edge { from: "mod:app/app".into(), to: "fn:app/app.main".into(), kind: EdgeKind::Contains },
            Edge { from: "fn:app/app.main".into(), to: "effect:Write".into(), kind: EdgeKind::Performs },
            Edge { from: "pkg:app".into(), to: "mod:app/app".into(), kind: EdgeKind::Contains },
        ];
        Atlas {
            version: SCHEMA.to_string(),
            root: "app".to_string(),
            nodes,
            edges,
            gods: vec![],
            authority: serde_json::json!({"effects":["Write"]}),
            custody: None,
            caveats: vec![crate::model::CAVEAT_STATIC.to_string()],
        }
    }

    /// A mermaid diagram must say what it leaves out (`HARDENING_CAMPAIGN.md` C36).
    ///
    /// Module level is the right scope for a shareable overview and it is documented — in a design
    /// addendum. But a mermaid diagram gets pasted into READMEs, issues, and agent context,
    /// permanently separated from the command that made it. A reader seeing two boxes for a program
    /// with a dozen nodes cannot tell that functions and effects were excluded BY DESIGN rather than
    /// absent from the program, and "this program appears to have no effects" is the worst wrong
    /// conclusion this particular graph could invite.
    ///
    /// The HTML renderer already got this right — full graph normally, collapsed above a node cap
    /// *with a visible notice*. Mermaid now describes itself the same way.
    #[test]
    fn the_mermaid_overview_declares_its_own_scope() {
        let a = tiny();
        // The premise: this atlas really does contain a function and an effect that mermaid drops.
        assert!(a.nodes.iter().any(|n| n.kind == NodeKind::Function), "premise: a function exists");
        assert!(a.nodes.iter().any(|n| n.kind == NodeKind::Effect), "premise: an effect exists");

        let m = a.render_mermaid();
        assert!(m.starts_with("graph LR"), "the diagram declaration stays first:
{m}");
        assert!(m.contains("%% MODULE-LEVEL"), "the scope note is a mermaid comment:
{m}");
        assert!(m.contains("NOT shown"), "it says what is MISSING, not only what is present:
{m}");
        assert!(m.contains("--format dot"), "it points at a format that shows the rest:
{m}");
        assert!(!m.contains("main"), "premise: the function really is absent from the diagram:
{m}");
        assert!(m.contains("n0["), "the nodes still render:
{m}");
    }

    #[test]
    fn dot_mermaid_html_are_deterministic() {
        let a = tiny();
        assert_eq!(a.render_dot(), a.render_dot());
        assert_eq!(a.render_mermaid(), a.render_mermaid());
        assert_eq!(a.render_html(), a.render_html());
    }

    #[test]
    fn dot_and_mermaid_have_expected_shape() {
        let a = tiny();
        let dot = a.render_dot();
        assert!(dot.starts_with("digraph atlas {"));
        assert!(dot.contains("\"fn:app/app.main\" -> \"effect:Write\" [label=\"performs\""));
        let m = a.render_mermaid();
        assert!(m.starts_with("graph LR"));
        assert!(m.contains("package app") && m.contains("module app"));
        // Module-level only: no function nodes leak into mermaid.
        assert!(!m.contains("main"));
    }

    #[test]
    fn html_is_self_contained_with_zero_external_urls() {
        let a = tiny();
        let h = a.render_html();
        assert!(h.contains("<!DOCTYPE html>"));
        assert!(h.contains("const ATLAS="), "data is embedded");
        assert!(h.contains("<script>") && h.contains("getContext('2d')"), "JS is inlined, canvas-drawn");
        // Criterion 7: zero external URLs — not even an SVG namespace.
        assert!(!h.contains("http://"), "no http URLs");
        assert!(!h.contains("https://"), "no https URLs");
        assert!(!h.contains("xmlns"), "no SVG namespace URL");
    }

    #[test]
    fn html_collapses_above_the_node_cap() {
        // Synthesize a graph larger than the cap: 1 package, 1 module, and many functions.
        let mut nodes = vec![
            Node::new("pkg:big", NodeKind::Package, "big"),
            Node::new("mod:big/big", NodeKind::Module, "big"),
        ];
        let mut edges = vec![Edge { from: "pkg:big".into(), to: "mod:big/big".into(), kind: EdgeKind::Contains }];
        for i in 0..(HTML_NODE_CAP + 5) {
            let id = format!("fn:big/big.f{i:05}");
            nodes.push(Node::new(&id, NodeKind::Function, format!("f{i:05}")));
            edges.push(Edge { from: "mod:big/big".into(), to: id, kind: EdgeKind::Contains });
        }
        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        let a = Atlas {
            version: SCHEMA.to_string(),
            root: "big".to_string(),
            nodes,
            edges,
            gods: vec![],
            authority: serde_json::json!({}),
            custody: None,
            caveats: vec![],
        };
        let h = a.render_html();
        assert!(h.contains("const COLLAPSED=true"), "collapsed flag set");
        assert!(h.contains("collapsed to package + module level"), "explicit collapse notice");
        // The embedded node data must NOT contain a function node when collapsed.
        assert!(!h.contains("fn:big/big.f00000"), "functions dropped in the collapsed view");
    }
}

