//! P9-01 — `delulu authority FILE --within BOUNDARY` (`V2_OPENSHELL_STUDY.md` §4.8, `V2_P9_DESIGN.md`).
//!
//! The question OpenShell's prover asks of a POLICY, asked of a PROGRAM: given a boundary written in
//! the `--grant` grammar, does everything this program asks for fall inside it? The answer is one of
//! four — **within**, **exceeds** (with a counterexample: the first request the boundary does not
//! cover and the call in the source that makes it — a line of code, which a policy prover cannot
//! give), **unsupported** (a capability no grant form bounds, named) and **inconclusive** (a request
//! no static reading decides, named) — and it is a claim about every run: `within` is never answered
//! while a request remains that the checker has not read.
//!
//! **What decides each request is the runtime's own mint.** The program's side is the requirement list
//! `delulu authority --grants` prints ([`crate::cli::requirements`] — one list, so the verdict and the
//! flags an operator types cannot disagree about what is asked for). The boundary's side is parsed by
//! `Grants::add`, the parser `run --grant` uses, and every literal request is put to
//! `prim::call_root_method` — the very decision `run --grant BOUNDARY` makes at that call. The broker's
//! `Authority` is NOT used to decide, though the design first said so (`V2_P9_DESIGN.md` step 2): it is
//! coarser than a grant set — `console` is folded into the `Write` effect (`authority_spec_from_grants`),
//! and it has no `net.special`, sensor, compute or plugin dimension — so a verdict read through it would
//! answer *within* for a printing program against an `fs.write`-only boundary that `run` refuses. Where
//! the two speak of the same thing (paths, hosts, secret names), a test holds them to one answer over
//! generated pairs.
//!
//! **A request whose scope the source computes** (a hidden site — SCOPE-HIDDEN-1, SECRET-HIDDEN-1) is
//! *exceeds* against a boundary that grants nothing of its kind (every value it could compute is
//! outside), *within* against one that grants the kind without bound (only `fs.read=/` and
//! `fs.write=/` on Unix — no other dimension has a widest form), and otherwise *inconclusive*, named:
//! which path or host the call asks for is decided only when it runs (D-V2-118).

use std::collections::BTreeMap;

use serde_json::{json, Value as Json};

use delulu_diag::{SourceMap, Span};
use delulu_runtime::value::{RootVal, Value};
use delulu_runtime::Grants;

use crate::cli::{Ask, Requirement};

/// The four answers, ordered by precedence: the verdict is the greatest outcome among the requests.
/// A definite counterexample (*exceeds*) outranks a capability no form bounds (*unsupported*), which
/// outranks a request no static reading decides (*inconclusive*).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Outcome {
    Within,
    Inconclusive,
    Unsupported,
    Exceeds,
}

impl Outcome {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Outcome::Within => "within",
            Outcome::Inconclusive => "inconclusive",
            Outcome::Unsupported => "unsupported",
            Outcome::Exceeds => "exceeds",
        }
    }
}

/// Every dimension of the grant grammar a program can ask for — `GRANT_FORMS`' keys but
/// `exec.native`, which a program never requires (invariant 45). The coverage names each one, so a
/// reader sees what was decided and what the program never asked for.
pub(crate) const DIMENSIONS: &[&str] = &[
    "console",
    "clock",
    "rand",
    "declassify",
    "fs.read",
    "fs.write",
    "net",
    "net.special",
    "secret",
    "foreign.c",
    "foreign.python",
    "actuator",
    "sensor",
    "compute",
    "plugin",
];

/// One request the boundary does not plainly cover.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Finding {
    /// The `--grant` line the program needs (empty for a capability no form bounds).
    pub(crate) grant: String,
    pub(crate) dimension: String,
    pub(crate) outcome: Outcome,
    pub(crate) reason: String,
    pub(crate) at: Option<Span>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Verdict {
    pub(crate) outcome: Outcome,
    /// Every request that is not *within*, in source order (a request with no call site last).
    pub(crate) findings: Vec<Finding>,
    /// Each dimension of [`DIMENSIONS`] → the greatest outcome among its requests, or `None` when the
    /// program asks for nothing of it.
    pub(crate) coverage: BTreeMap<String, Option<Outcome>>,
    /// How many requests were decided.
    pub(crate) requests: usize,
}

impl Verdict {
    /// The first finding whose outcome is the verdict's — the counterexample for *exceeds*, the first
    /// named request for *unsupported* and *inconclusive*; `None` for *within*.
    pub(crate) fn counterexample(&self) -> Option<&Finding> {
        self.findings.iter().find(|f| f.outcome == self.outcome && self.outcome != Outcome::Within)
    }
}

/// The dimension a `--grant` line belongs to: its key, up to the first `=` or `:`.
fn dimension_of(grant: &str) -> String {
    grant.split(['=', ':']).next().unwrap_or(grant).to_string()
}

/// Decide every requirement against the boundary. Pure but for the filesystem reads the runtime's own
/// path containment makes (a granted path and a requested one are resolved on disk, as at the mint).
pub(crate) fn verdict(reqs: &[Requirement], unsupported: &[String], boundary: &Grants) -> Verdict {
    let root = boundary.build_root();
    let mut findings: Vec<(usize, Finding)> = Vec::new();
    let mut coverage: BTreeMap<String, Option<Outcome>> = DIMENSIONS.iter().map(|d| (d.to_string(), None)).collect();
    for (i, r) in reqs.iter().enumerate() {
        let (outcome, reason) = decide(&r.ask, &root, boundary);
        let dimension = dimension_of(&r.grant);
        let slot = coverage.entry(dimension.clone()).or_insert(None);
        *slot = Some(slot.map_or(outcome, |o| o.max(outcome)));
        if outcome != Outcome::Within {
            findings.push((i, Finding { grant: r.grant.clone(), dimension, outcome, reason, at: r.at }));
        }
    }
    for (j, kind) in unsupported.iter().enumerate() {
        coverage.insert(kind.clone(), Some(Outcome::Unsupported));
        findings.push((
            reqs.len() + j,
            Finding {
                grant: String::new(),
                dimension: kind.clone(),
                outcome: Outcome::Unsupported,
                reason: format!("the program holds a `{kind}` capability, and no `--grant` form bounds that kind"),
                at: None,
            },
        ));
    }
    // Source order: by the call's position; a request no call site is attributed to keeps its place
    // after every one that has one.
    findings.sort_by_key(|(i, f)| (f.at.map_or((1, 0, 0), |s| (0, s.file, s.start)), *i));
    let findings: Vec<Finding> = findings.into_iter().map(|(_, f)| f).collect();
    let outcome = findings.iter().map(|f| f.outcome).max().unwrap_or(Outcome::Within);
    Verdict { outcome, findings, coverage, requests: reqs.len() + unsupported.len() }
}

/// One request against the boundary.
fn decide(ask: &Ask, root: &RootVal, boundary: &Grants) -> (Outcome, String) {
    let span = Span::new(0, 0, 0);
    match ask {
        Ask::Mint { method, arg } => {
            let args: Vec<Value> = match (method, arg) {
                (_, None) => Vec::new(),
                // `root.http` takes the host LIST; one host at a time is the same decision, host by host.
                (&"http", Some(h)) => vec![Value::List(std::rc::Rc::new(std::cell::RefCell::new(vec![Value::str(h.clone())])))],
                (_, Some(a)) => vec![Value::str(a.clone())],
            };
            // D-NE-28 needs no second check here: `Grants::add` refuses a special-use host under a plain
            // `net=` (it must be spelled `net.special=`, which also puts it in `net`), so a special-use host
            // the mint accepts is one the boundary granted by its explicit spelling. A check written for it
            // was unreachable — mutant M272 survived every witness — and was removed (§11.5, M222).
            match delulu_runtime::prim::call_root_method(root, method, &args, span) {
                Ok(_) => (Outcome::Within, String::new()),
                // The runtime's own refusal, in its own words — the reason `run --grant BOUNDARY` would
                // give at this call.
                Err(f) => (Outcome::Exceeds, format!("{} ({})", f.message, f.code)),
            }
        }
        Ask::Hidden { method } => {
            let (granted_none, unbounded, what) = match *method {
                "fs_read" => (root.fs_read.is_empty(), any_unix_root(&root.fs_read), "path to read"),
                "fs_write" => (root.fs_write.is_empty(), any_unix_root(&root.fs_write), "path to write"),
                "http" => (root.net.is_empty(), false, "host"),
                "secret" => (root.secrets.is_empty(), false, "secret's name"),
                "actuator" => (root.actuators.is_empty(), false, "actuator"),
                "sensor" => (root.sensors.is_empty(), false, "sensor"),
                "compute" => (root.computes.is_empty(), false, "compute device"),
                "foreign" => (boundary.foreign_c.is_empty(), false, "foreign library"),
                // A hidden site of a kind this function does not know is never answered within.
                _ => (false, false, "scope"),
            };
            // Not "this call": a capability can reach a function as a parameter, with no minting call
            // anywhere in the file (`10_actuate_errors`) — the source shows no scope, and no call.
            let unseen = format!(
                "the source does not show this {what} (it is computed, or reached through a value the checker \
                 does not follow)"
            );
            if granted_none {
                (Outcome::Exceeds, format!("{unseen}, and the boundary grants none of its kind"))
            } else if unbounded {
                (Outcome::Within, String::new())
            } else {
                (
                    Outcome::Inconclusive,
                    format!("{unseen}; the boundary grants some, and which one it is is decided only when the program runs"),
                )
            }
        }
        Ask::ForeignLib(lib) => {
            if boundary.foreign_c.contains_key(lib) {
                (Outcome::Within, String::new())
            } else {
                (Outcome::Exceeds, format!("the boundary grants no foreign library `{lib}` (`foreign.c={lib}:PATH`)"))
            }
        }
        Ask::PythonImport { name, pattern } => {
            // A manifest's PATTERN is compared as the broker compares the dimension, by exact string; a
            // module the source imports is matched as the runtime matches an import (`py.import`).
            let ok = if *pattern {
                boundary.foreign_python.iter().any(|p| p == name)
            } else {
                delulu_runtime::python::allowlist_allows(&boundary.foreign_python, name)
            };
            if ok {
                (Outcome::Within, String::new())
            } else {
                (Outcome::Exceeds, format!("the boundary's Python allowlist does not allow `{name}`"))
            }
        }
        Ask::PythonUnknown => {
            if boundary.foreign_python.is_empty() {
                (Outcome::Exceeds, "the program embeds Python, and the boundary grants no `foreign.python`".to_string())
            } else {
                (
                    Outcome::Inconclusive,
                    "the program embeds Python and names no import the checker can read; the boundary allows some".to_string(),
                )
            }
        }
        Ask::PluginLoad => {
            if root.plugins.is_empty() {
                (Outcome::Exceeds, "the program loads a plugin, and the boundary grants no `plugin=` root".to_string())
            } else {
                (
                    Outcome::Inconclusive,
                    "a plugin's artifact is checked at its load — against the plugin roots and the manifest's hash \
                     list — not by this reading"
                        .to_string(),
                )
            }
        }
    }
}

/// Does a granted path set hold the filesystem's root — the one widest form a path dimension has? Unix
/// only: on Windows a drive's root bounds that drive, and a computed path may name another.
fn any_unix_root(roots: &[std::path::PathBuf]) -> bool {
    cfg!(unix) && roots.iter().any(|p| p == std::path::Path::new("/"))
}

fn at_json(map: &SourceMap, at: Option<Span>) -> Json {
    match at {
        Some(s) if map.has(s.file) => {
            let (line, column) = map.position(s.file, s.start);
            json!({ "file": map.name(s.file), "line": line, "column": column })
        }
        _ => Json::Null,
    }
}

fn finding_json(map: &SourceMap, f: &Finding) -> Json {
    json!({
        "grant": f.grant,
        "dimension": f.dimension,
        "result": f.outcome.name(),
        "reason": f.reason,
        "at": at_json(map, f.at),
    })
}

/// The `within` object of the authority envelope: additive, beside every field the report had.
pub(crate) fn verdict_json(v: &Verdict, boundary: &[String], map: &SourceMap) -> Json {
    let coverage: serde_json::Map<String, Json> =
        v.coverage.iter().map(|(d, o)| (d.clone(), json!(o.map_or("not-requested", |o| o.name())))).collect();
    json!({
        "result": v.outcome.name(),
        "boundary": boundary,
        "counterexample": v.counterexample().map_or(Json::Null, |f| finding_json(map, f)),
        "findings": v.findings.iter().map(|f| finding_json(map, f)).collect::<Vec<_>>(),
        "coverage": coverage,
        "requests": v.requests,
    })
}

/// The human answer. A program's strings (a path, a host, a quoted line of its source) reach the
/// operator's terminal here, so each is put on one line with its control characters shown escaped
/// (TERMINAL-TEXT-1's `terminal_line`).
pub(crate) fn render_verdict(v: &Verdict, program: &str, map: &SourceMap) -> String {
    let esc = |s: &str| delulu_diag::terminal_line(s).into_owned();
    let mut out = String::new();
    let decided = v.coverage.values().filter(|o| o.is_some()).count();
    match v.outcome {
        Outcome::Within => out.push_str(&format!(
            "within: `{}` asks for nothing outside the boundary — {} request(s) in {} dimension(s), each decided\n",
            esc(program),
            v.requests,
            decided
        )),
        other => {
            let head = match other {
                Outcome::Exceeds => "exceeds: the program asks for authority the boundary does not grant",
                Outcome::Unsupported => "unsupported: the program holds a capability no grant form can bound",
                _ => "inconclusive: no static reading decides every request",
            };
            out.push_str(head);
            out.push('\n');
            for f in &v.findings {
                let label = if f.grant.is_empty() { f.dimension.clone() } else { f.grant.clone() };
                out.push_str(&format!("  {:<12} {}\n", f.outcome.name(), esc(&label)));
                out.push_str(&format!("               {}\n", esc(&f.reason)));
                if let Some(s) = f.at.filter(|s| map.has(s.file)) {
                    let (line, column) = map.position(s.file, s.start);
                    out.push_str(&format!("               --> {}:{line}:{column}\n", esc(map.name(s.file))));
                    let text = map.line_text(s.file, line).trim();
                    out.push_str(&format!("               |   {}\n", esc(text)));
                }
            }
        }
    }
    out
}

/// One thing a change of grant set ADDS (P9-02, `V2_OPENSHELL_STUDY.md` §4.9): a category, no severity grade.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DiffFinding {
    /// `effect_expansion` (a dimension OLD grants none of), `scope_expansion` (a wider scope in a dimension OLD grants),
    /// `special_use_reach` (a `net.special=` host OLD does not name), `device_reach_expansion` (a device OLD does not name,
    /// or an envelope wider than OLD's). `budget_expansion` belongs to lease nodes, which carry a budget; a grant set
    /// carries none.
    pub(crate) category: &'static str,
    pub(crate) dimension: String,
    /// NEW's line, as written.
    pub(crate) grant: String,
    pub(crate) detail: String,
}

/// P9-02 — `delulu grants diff OLD NEW` between two grant sets: each line of NEW, decided against OLD exactly as
/// `authority --within OLD` decides a request — through the runtime's own mint where one decides it (a path, a host, a
/// flag), and through the broker's device lattice for an actuator's envelope, which the mint does not read. A line NEW
/// shares with OLD, or narrows, adds nothing. `Err` is a line that does not parse.
pub(crate) fn diff(old: &Grants, new_lines: &[String]) -> Result<Vec<DiffFinding>, String> {
    let root = old.build_root();
    let span = Span::new(0, 0, 0);
    let mint = |method: &str, arg: Option<&str>| -> Result<(), String> {
        let args: Vec<Value> = match arg {
            None => Vec::new(),
            Some(h) if method == "http" => {
                vec![Value::List(std::rc::Rc::new(std::cell::RefCell::new(vec![Value::str(h.to_string())])))]
            }
            Some(a) => vec![Value::str(a.to_string())],
        };
        delulu_runtime::prim::call_root_method(&root, method, &args, span).map(|_| ()).map_err(|f| f.message)
    };
    let devices =
        |g: &Grants| crate::brokerd::spec_to_authority(&crate::cli::authority_spec_from_grants(g, "diff")).scopes.device;
    let mut out = Vec::new();
    for line in new_lines {
        let mut one = Grants::default();
        one.add(line).map_err(|e| format!("bad grant in NEW: {e}"))?;
        let dimension = if line.trim_start().starts_with("secret:") { "secret".to_string() } else { dimension_of(line.trim()) };
        let none = |what: &str| Err(format!("OLD grants no {what}"));
        let covered: Result<(), String> = match dimension.as_str() {
            "console" | "clock" | "rand" | "declassify" => mint(&dimension, None),
            "exec.native" => {
                if old.exec_native {
                    Ok(())
                } else {
                    none("native emission")
                }
            }
            "fs.read" => one.fs_read.iter().try_for_each(|p| mint("fs_read", Some(p))),
            "fs.write" => one.fs_write.iter().try_for_each(|p| mint("fs_write", Some(p))),
            // `net.special=` puts its host in `net` too: decided once, as special-use reach.
            "net" => one.net.iter().try_for_each(|h| mint("http", Some(h))),
            "net.special" => one.net_special.iter().try_for_each(|h| {
                // P9-05 (b): within a special-use grant of OLD, by the broker's order (a narrower path is no new reach).
                if old.net_special.iter().any(|g| delulu_broker::netscope::entry_within(h, g)) {
                    Ok(())
                } else {
                    Err(format!("OLD does not grant `{h}` as special-use reach"))
                }
            }),
            // P9-04 (c), D-V2-131: a secret's REACH is its audience — every origin NEW lets it be sent to must be one OLD
            // names for it too. No audience is the bottom (sent nowhere), so one is reach a secret without one never had.
            "secret" => one.secrets.keys().try_for_each(|n| {
                if !old.secrets.contains_key(n) {
                    return Err(format!("OLD grants no secret `{n}`"));
                }
                let old_to = old.secret_audiences.get(n).map(Vec::as_slice).unwrap_or(&[]);
                match one.secret_audiences.get(n).into_iter().flatten().find(|a| !old_to.contains(a)) {
                    Some(a) => {
                        Err(format!("NEW lets the secret `{n}` be sent to `{a}`, which OLD's audience for it does not name"))
                    }
                    None => Ok(()),
                }
            }),
            "foreign.c" => one.foreign_c.keys().try_for_each(|l| {
                if old.foreign_c.contains_key(l) {
                    Ok(())
                } else {
                    Err(format!("OLD grants no foreign library `{l}`"))
                }
            }),
            // A pattern by exact string (the broker's rule); a plain module name as an import is matched.
            "foreign.python" => one.foreign_python.iter().try_for_each(|p| {
                let ok = old.foreign_python.contains(p)
                    || (!p.contains('*') && delulu_runtime::python::allowlist_allows(&old.foreign_python, p));
                if ok {
                    Ok(())
                } else {
                    Err(format!("OLD's Python allowlist does not allow `{p}`"))
                }
            }),
            "actuator" => {
                let (n, o) = (devices(&one), devices(old));
                if delulu_broker::device_scope::all_within(&n, &o) {
                    Ok(())
                } else if n.keys().all(|d| o.contains_key(d)) {
                    Err("its envelope is wider than OLD's for the same device".to_string())
                } else {
                    Err("OLD grants no such actuator".to_string())
                }
            }
            "sensor" => one.sensors.iter().try_for_each(|d| mint("sensor", Some(d))),
            "compute" => one.computes.iter().try_for_each(|c| match root.computes.iter().find(|o| o.device == c.device) {
                None => Err(format!("OLD grants no compute device `{}`", c.device)),
                Some(o) => {
                    // Every bound no wider, the same adapter, no format or kernel name OLD lacks — and no waiver of the
                    // adapter's attestation that OLD did not already make (DL1911): each is reach the device did not have.
                    let within = c.memory_bytes <= o.memory_bytes
                        && c.queue_depth <= o.queue_depth
                        && c.kernel_ms.0 >= o.kernel_ms.0
                        && c.kernel_ms.1 <= o.kernel_ms.1
                        && c.power_w.0 >= o.power_w.0
                        && c.power_w.1 <= o.power_w.1
                        && c.adapter == o.adapter
                        && c.formats.iter().all(|f| o.formats.contains(f))
                        && c.kernels.iter().all(|(k, _)| o.kernels.iter().any(|(ok, _)| ok == k))
                        && (!c.waived || o.waived);
                    if within {
                        Ok(())
                    } else {
                        Err(format!("its envelope is wider than OLD's for `{}`", c.device))
                    }
                }
            }),
            "plugin" => one.plugins.iter().try_for_each(|p| {
                let want = delulu_runtime::prim::granted_root(p);
                if root.plugins.iter().any(|g| delulu_runtime::prim::contains_on_disk(g, &want)) {
                    Ok(())
                } else {
                    Err(format!("OLD grants no plugin root holding `{p}`"))
                }
            }),
            // A form this diff does not decide is reported as added — never skipped (§11.2).
            _ => Err("a grant form this diff does not decide".to_string()),
        };
        if let Err(detail) = covered {
            let granted_any = match dimension.as_str() {
                "console" => old.console,
                "clock" => old.clock,
                "rand" => old.rand,
                "declassify" => old.declassify,
                "exec.native" => old.exec_native,
                "fs.read" => !old.fs_read.is_empty(),
                "fs.write" => !old.fs_write.is_empty(),
                "net" => !old.net.is_empty(),
                "secret" => !old.secrets.is_empty(),
                "foreign.c" => !old.foreign_c.is_empty(),
                "foreign.python" => !old.foreign_python.is_empty(),
                "plugin" => !old.plugins.is_empty(),
                _ => false,
            };
            let category = match dimension.as_str() {
                "net.special" => "special_use_reach",
                // Every name NEW grants is one OLD grants: what failed is where the secret may go.
                "secret" if one.secrets.keys().all(|n| old.secrets.contains_key(n)) => "secret_reach_expansion",
                "actuator" | "sensor" | "compute" => "device_reach_expansion",
                _ if granted_any => "scope_expansion",
                _ => "effect_expansion",
            };
            out.push(DiffFinding { category, dimension, grant: line.trim().to_string(), detail });
        }
    }
    Ok(out)
}

/// P9-02 between two lease NODES: here the broker's `Authority` IS the domain — a node carries no `console` flag, no
/// `net.special`, no sensor — so the lattice decides, and each item of NEW is asked of `attenuation_check` as a one-item
/// authority. The probe carries OLD's budget, because an absent budget is the TOP of that dimension and would make every
/// probe fail; the budget itself is decided once, by `budget_scope::within`. An effect NEW adds is an effect expansion;
/// a path, host, name or library is a scope expansion; a device a device-reach expansion; a budget a budget expansion.
pub(crate) fn diff_nodes(old: &delulu_broker::Authority, new: &delulu_broker::Authority) -> Vec<DiffFinding> {
    use delulu_broker::{Authority, Scopes};
    let covered = |set: &dyn Fn(&mut Scopes)| -> bool {
        let mut scopes = Scopes { budget: old.scopes.budget, ..Default::default() };
        set(&mut scopes);
        delulu_broker::attenuation_check(&Authority { effects: Default::default(), scopes }, old).is_ok()
    };
    let mut out = Vec::new();
    let mut push = |category: &'static str, dimension: &str, grant: String, detail: String| {
        out.push(DiffFinding { category, dimension: dimension.to_string(), grant, detail });
    };
    for e in &new.effects {
        if !old.effects.contains(e) {
            push("effect_expansion", "effect", e.name().to_string(), format!("OLD's node has no `{}` effect", e.name()));
        }
    }
    let n = &new.scopes;
    type Pick = fn(&mut Scopes) -> &mut std::collections::BTreeSet<String>;
    let dims: [(&str, &std::collections::BTreeSet<String>, Pick); 7] = [
        ("fs.read", &n.fs_read, |s| &mut s.fs_read),
        ("fs.write", &n.fs_write, |s| &mut s.fs_write),
        ("net", &n.net, |s| &mut s.net),
        ("secret", &n.secrets, |s| &mut s.secrets),
        ("declassify", &n.declassify, |s| &mut s.declassify),
        ("foreign.c", &n.foreign_c, |s| &mut s.foreign_c),
        ("foreign.python", &n.foreign_python, |s| &mut s.foreign_python),
    ];
    for (dimension, items, pick) in dims {
        for item in items {
            if !covered(&|s: &mut Scopes| {
                pick(s).insert(item.clone());
            }) {
                push("scope_expansion", dimension, format!("{dimension}={item}"), format!("OLD's node does not cover `{item}`"));
            }
        }
    }
    // P9-04 (d), D-V2-133: a secret's REACH between nodes — each `NAME@host:port` pair NEW holds must be one OLD holds,
    // the lattice's own inclusion. A pair for a secret OLD does not hold at all is that secret's `scope_expansion`,
    // already named above, so it is not counted twice (the grant-file diff's rule, D-V2-131).
    for pair in &n.secret_to {
        let held = delulu_broker::secret_to_name(pair).is_some_and(|name| old.scopes.secrets.contains(name));
        if held
            && !covered(&|s: &mut Scopes| {
                s.secret_to.insert(pair.clone());
            })
        {
            let (name, origin) = pair.split_once('@').unwrap_or((pair.as_str(), ""));
            push(
                "secret_reach_expansion",
                "secret",
                pair.clone(),
                format!("NEW's node lets the secret `{name}` be sent to `{origin}`, which OLD's audience for it does not name"),
            );
        }
    }
    for (dev, scope) in &n.device {
        if !covered(&|s: &mut Scopes| {
            s.device.insert(dev.clone(), scope.clone());
        }) {
            let detail = if old.scopes.device.contains_key(dev) {
                "its envelope is wider than OLD's for the same device".to_string()
            } else {
                "OLD's node grants no such device".to_string()
            };
            push("device_reach_expansion", "device", scope.to_grant_string(), detail);
        }
    }
    if !delulu_broker::budget_scope::within(n.budget.as_ref(), old.scopes.budget.as_ref()) {
        let shown = n.budget.map_or("none — the operator's own".to_string(), |b| b.to_grant_string());
        push("budget_expansion", "budget", shown, "a budget OLD's node does not bound".to_string());
    }
    out
}

/// P9-04 (e), D-V2-135: every place a program SENDS a secret (`http.get_with_secret`), with the secret and the
/// destination as far as the source shows them — the static half of a secret's reach. `secret` is the grant name when
/// the argument is `root.secret("NAME")` itself, a `let`-bound name for one, or a `Secret.map` of either; `to` is the
/// URL literal's origin (`host:port`, the egress client's own parse and spelling — what an audience must name). Either
/// is `null` when the source does not show it — a computed URL, a secret that arrives as a parameter, a field, a
/// collection's element — and is never guessed: a name a later `let`, assignment, `for`, lambda parameter or match
/// binding rebinds is forgotten from there on, so a stale binding cannot attribute a different value.
pub(crate) fn secret_destinations(modules: &[&delulu_syntax::ast::Module], map: &SourceMap) -> Vec<Json> {
    use delulu_syntax::ast::Item;
    let mut w = SendWalk { map, vars: BTreeMap::new(), out: Vec::new() };
    for m in modules {
        for item in &m.items {
            match item {
                // Each body is a block, and a block's names end with it: a parameter, which no `let` in the body
                // bound, is never attributed.
                Item::Fn(f) => w.block(&f.body),
                Item::Test(t) => w.block(&t.body),
                Item::Actor(a) => {
                    w.block(&a.ctor.body);
                    for b in &a.behaviors {
                        w.block(&b.body);
                    }
                    for f in &a.fns {
                        w.block(&f.body);
                    }
                }
                _ => {}
            }
        }
    }
    w.out
}

/// The walk behind [`secret_destinations`]: `vars` maps a name in scope to the secret it is known to hold (`None`: a
/// name known NOT to be one the source shows), restored at each block's end.
struct SendWalk<'a> {
    map: &'a SourceMap,
    vars: BTreeMap<String, Option<String>>,
    out: Vec<Json>,
}

impl SendWalk<'_> {
    fn block(&mut self, b: &delulu_syntax::ast::Block) {
        let saved = self.vars.clone();
        for s in &b.stmts {
            self.stmt(s);
        }
        self.vars = saved;
    }

    fn stmt(&mut self, s: &delulu_syntax::ast::Stmt) {
        use delulu_syntax::ast::{LValue, Stmt};
        match s {
            Stmt::Let { name, value, .. } => {
                self.expr(value);
                let held = self.secret_of(value);
                self.vars.insert(name.name.clone(), held);
            }
            Stmt::Assign { target, value, .. } => {
                self.expr(value);
                if let LValue::Var(name) = target {
                    let held = self.secret_of(value);
                    self.vars.insert(name.name.clone(), held);
                }
            }
            Stmt::While { cond, body, .. } => {
                self.expr(cond);
                self.block(body);
            }
            Stmt::For { var, iter, body, .. } => {
                self.expr(iter);
                let saved = self.vars.clone();
                self.vars.insert(var.name.clone(), None);
                self.block(body);
                self.vars = saved;
            }
            Stmt::Return { value: Some(e), .. } | Stmt::Expr(e) => self.expr(e),
            Stmt::Return { value: None, .. } | Stmt::Break { .. } | Stmt::Continue { .. } => {}
        }
    }

    /// The grant name of the secret `e` evaluates to, when the source shows it.
    fn secret_of(&self, e: &delulu_syntax::ast::Expr) -> Option<String> {
        use delulu_syntax::ast::Expr;
        match e {
            // Only `Root` has a `secret` method (the primitive table), so a literal `.secret("NAME")` is its mint under
            // whatever name the receiver goes by — `requested_scopes`' rule.
            Expr::Method { name, args, .. } if name.name == "secret" && args.len() == 1 => match &args[0] {
                Expr::Lit { kind: delulu_syntax::ast::LitKind::Str(s), .. } => Some(s.clone()),
                _ => None,
            },
            // `Secret.map` keeps the secret's grant name (D-V2-131: a value derived from its bytes goes where they may).
            Expr::Method { recv, name, .. } if name.name == "map" => self.secret_of(recv),
            Expr::Var { path, .. } if path.segs.len() == 1 => self.vars.get(&path.segs[0].name).cloned().flatten(),
            _ => None,
        }
    }

    fn expr(&mut self, e: &delulu_syntax::ast::Expr) {
        use delulu_syntax::ast::{Expr, Pattern};
        match e {
            Expr::Method { recv, name, args, span, .. } => {
                self.expr(recv);
                for a in args {
                    self.expr(a);
                }
                if name.name == "get_with_secret" && args.len() == 4 {
                    let to = match &args[0] {
                        Expr::Lit { kind: delulu_syntax::ast::LitKind::Str(u), .. } => {
                            delulu_runtime::egress::parse_target(u).ok().map(|t| format!("{}:{}", t.host, t.port))
                        }
                        _ => None,
                    };
                    let secret = self.secret_of(&args[3]);
                    self.out.push(json!({ "secret": secret, "to": to, "at": at_json(self.map, Some(*span)) }));
                }
            }
            Expr::List { items, .. } => items.iter().for_each(|it| self.expr(it)),
            Expr::Record { fields, .. } => fields.iter().for_each(|(_, v)| self.expr(v)),
            Expr::Call { callee, args, .. } => {
                self.expr(callee);
                args.iter().for_each(|a| self.expr(a));
            }
            Expr::Field { recv, .. } => self.expr(recv),
            Expr::Index { recv, index, .. } => {
                self.expr(recv);
                self.expr(index);
            }
            Expr::Unary { operand, .. } => self.expr(operand),
            Expr::Binary { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            Expr::If { cond, then_, else_, .. } => {
                self.expr(cond);
                self.block(then_);
                if let Some(e) = else_ {
                    self.expr(e);
                }
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.expr(scrutinee);
                for arm in arms {
                    // Every name the pattern binds shadows, for the arm (a `Bind` that names a variant over-forgets:
                    // the safe direction).
                    fn binds(p: &Pattern, out: &mut Vec<String>) {
                        match p {
                            Pattern::Bind(i) => out.push(i.name.clone()),
                            Pattern::Variant { fields, .. } => fields.iter().for_each(|f| binds(f, out)),
                            Pattern::Wildcard(_) | Pattern::Lit(..) => {}
                        }
                    }
                    let mut names = Vec::new();
                    binds(&arm.pattern, &mut names);
                    let saved = self.vars.clone();
                    for n in names {
                        self.vars.insert(n, None);
                    }
                    self.expr(&arm.body);
                    self.vars = saved;
                }
            }
            Expr::Lambda { params, body, .. } => {
                let saved = self.vars.clone();
                for p in params {
                    self.vars.insert(p.name.name.clone(), None);
                }
                self.block(body);
                self.vars = saved;
            }
            Expr::Try { inner, .. } => self.expr(inner),
            Expr::Block(b) => self.block(b),
            Expr::Spawn { args, .. } => args.iter().for_each(|a| self.expr(a)),
            Expr::Recover { body, .. } => self.block(body),
            Expr::Lit { .. } | Expr::Var { .. } | Expr::Consume { .. } => {}
        }
    }
}

/// P9-03 — what `grants propose` proposes for a refusal at `at` (`V2_P9_DESIGN.md`).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Proposal {
    /// The `--grant` lines of the refused call's own requirement.
    Lines(Vec<String>),
    /// Nothing, and why; an empty reason is "no minting call is there" (the caller words it: the program changed).
    Nothing(String),
}

/// P9-03: the static requirement of the capability-minting call at exactly `at`, as the requirement list names it — the
/// narrowest grant for that call site (`V2_OPENSHELL_STUDY.md` §4.10), never a line the runtime or the program worded. Nothing
/// when no minting call is there, when the call computes its scope (no static narrowest grant exists for it), or when its
/// grant names data only the operator holds (a library's path, a plugin's artifact).
pub(crate) fn propose(reqs: &[Requirement], sites: &[crate::cli::MintSite], at: Span) -> Proposal {
    let Some(site) = sites.iter().find(|s| s.at.file == at.file && s.at.start == at.start && s.at.end == at.end) else {
        return Proposal::Nothing(String::new());
    };
    let method = match site.kind {
        "FsRead" => "fs_read",
        "FsWrite" => "fs_write",
        "Http" => "http",
        "Secret" => "secret",
        "Actuator" => "actuator",
        "Sensor" => "sensor",
        "Compute" => "compute",
        "Console" => "console",
        "Clock" => "clock",
        "Rand" => "rand",
        "Declassify" => "declassify",
        other => {
            return Proposal::Nothing(format!(
                "the call mints `{other}`, whose grant names what only the operator holds (a library's or an artifact's path)"
            ))
        }
    };
    if site.hidden {
        let shape = reqs.iter().find(|r| r.ask == Ask::Hidden { method }).map(|r| format!(" — its shape is `{}`", r.grant));
        return Proposal::Nothing(format!(
            "the call computes its scope, so no grant is the narrowest for it: the operator chooses the value{}",
            shape.unwrap_or_default()
        ));
    }
    let lines: Vec<String> = reqs
        .iter()
        .filter(|r| match &r.ask {
            Ask::Mint { method: m, arg: None } => *m == method && site.lits.is_empty(),
            Ask::Mint { method: m, arg: Some(a) } => *m == method && site.lits.contains(a),
            _ => false,
        })
        .map(|r| r.grant.clone())
        .collect();
    if lines.is_empty() {
        return Proposal::Nothing(format!("no requirement of the program names the call's `{}`", site.kind));
    }
    Proposal::Lines(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grants(lines: &[&str]) -> Grants {
        let mut g = Grants::default();
        for l in lines {
            g.add(l).unwrap_or_else(|e| panic!("{l}: {e}"));
        }
        g
    }

    fn mint(grant: &str, method: &'static str, arg: Option<&str>, at: u32) -> Requirement {
        Requirement {
            grant: grant.to_string(),
            ask: Ask::Mint { method, arg: arg.map(str::to_string) },
            at: Some(Span::new(0, at, at + 1)),
        }
    }

    fn hidden(grant: &str, method: &'static str) -> Requirement {
        Requirement { grant: grant.to_string(), ask: Ask::Hidden { method }, at: None }
    }

    /// The design's agreement test (`V2_P9_DESIGN.md` step 2): where the broker's lattice and a grant set
    /// speak of the same thing — paths, hosts, secret names — the verdict and `attenuation_check` give one
    /// answer, over generated programs-and-boundaries. Paths are absolute, under a real directory tree, so
    /// the runtime's on-disk containment and the lattice's lexical one are asked the same question.
    #[test]
    fn the_verdict_agrees_with_the_lattice_where_both_speak() {
        // Not canonicalized: on Windows that spells `\\?\C:\…`, which a program's path may not (D-NE-29), and
        // both sides resolve a link or an 8.3 name themselves — the runtime on disk, the lattice's side through
        // `broker_path`.
        let base = std::env::temp_dir().join(format!("delulu-within-agree-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        for d in ["a/b/c", "a/x", "d/e"] {
            std::fs::create_dir_all(base.join(d)).unwrap();
        }
        let paths: Vec<String> =
            ["a", "a/b", "a/b/c", "a/x", "d", "d/e"].iter().map(|p| base.join(p).to_string_lossy().to_string()).collect();
        // P9-05 (b), D-V2-139: path-scoped entries too — the runtime's mint and the lattice must order them alike.
        let hosts = [
            "example.com",
            "api.example.com",
            "cdn.example.org",
            "api.example.com/v1/**",
            "api.example.com/v1/*/x",
            "api.example.com/v1/items/x",
            "api.example.com/v2/**",
            "api.example.com/**",
        ];
        let names = ["API_KEY", "DB_PASS"];
        // A small deterministic generator (xorshift) — inputs generated, not written (§11.2).
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = |n: usize| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state % n as u64) as usize
        };
        let mut agreed = 0;
        let mut within_seen = 0;
        for _ in 0..400 {
            let mut prog: Vec<String> = Vec::new();
            let mut bound: Vec<String> = Vec::new();
            for side in [&mut prog, &mut bound] {
                for _ in 0..next(4) {
                    let line = match next(4) {
                        0 => format!("fs.read={}", paths[next(paths.len())]),
                        1 => format!("fs.write={}", paths[next(paths.len())]),
                        2 => format!("net={}", hosts[next(hosts.len())]),
                        _ => format!("secret:{}=v", names[next(names.len())]),
                    };
                    side.push(line);
                }
            }
            let reqs: Vec<Requirement> = prog
                .iter()
                .map(|l| {
                    let (k, v) = l.split_once('=').unwrap();
                    match k {
                        "fs.read" => mint(l, "fs_read", Some(v), 0),
                        "fs.write" => mint(l, "fs_write", Some(v), 0),
                        "net" => mint(l, "http", Some(v), 0),
                        _ => mint(l, "secret", Some(k.trim_start_matches("secret:")), 0),
                    }
                })
                .collect();
            let refs: Vec<&str> = bound.iter().map(String::as_str).collect();
            let b = grants(&refs);
            let ours = verdict(&reqs, &[], &b).outcome == Outcome::Within;
            let prefs: Vec<&str> = prog.iter().map(String::as_str).collect();
            let auth = |g: &Grants| crate::brokerd::spec_to_authority(&crate::cli::authority_spec_from_grants(g, "t"));
            let lattice = delulu_broker::attenuation_check(&auth(&grants(&prefs)), &auth(&b)).is_ok();
            assert_eq!(ours, lattice, "program {prog:?} within boundary {bound:?}: verdict {ours}, lattice {lattice}");
            agreed += 1;
            within_seen += usize::from(ours);
        }
        // The control: a generator that never produced a `within` (or never anything else) proved nothing.
        assert!(within_seen > 20 && within_seen < agreed - 20, "{within_seen} of {agreed} within");
        let _ = std::fs::remove_dir_all(&base);
    }

    /// Why the verdict decides by the runtime's mint and not by the broker's `Authority`: the node a
    /// boundary would mint folds `console` into `Write`, so the lattice calls a printing program within an
    /// `fs.write`-only boundary — and `run` with that boundary refuses its `root.console()` (DL0703).
    #[test]
    fn a_printing_program_exceeds_a_boundary_that_grants_only_a_write() {
        let b = grants(&["fs.write=./out"]);
        let v = verdict(&[mint("console", "console", None, 7)], &[], &b);
        assert_eq!(v.outcome, Outcome::Exceeds, "{v:?}");
        assert!(v.findings[0].reason.contains("DL0703"), "the runtime's own refusal: {v:?}");
        let auth = |g: &Grants| crate::brokerd::spec_to_authority(&crate::cli::authority_spec_from_grants(g, "t"));
        assert!(
            delulu_broker::attenuation_check(&auth(&grants(&["console"])), &auth(&b)).is_ok(),
            "the coarser lattice says within — the reason it does not decide"
        );
    }

    #[test]
    fn a_hidden_site_is_exceeds_against_nothing_inconclusive_against_some_and_never_within_unless_unbounded() {
        let none = verdict(&[hidden("fs.read=PATH", "fs_read")], &[], &grants(&["console"]));
        assert_eq!(none.outcome, Outcome::Exceeds, "{none:?}");
        let some = verdict(&[hidden("fs.read=PATH", "fs_read")], &[], &grants(&["fs.read=./data"]));
        assert_eq!(some.outcome, Outcome::Inconclusive, "{some:?}");
        let host = verdict(&[hidden("net=HOST", "http")], &[], &grants(&["net=example.com"]));
        assert_eq!(host.outcome, Outcome::Inconclusive, "a host set has no widest form: {host:?}");
        let secret = verdict(&[hidden("secret:NAME=VALUE", "secret")], &[], &grants(&["console"]));
        assert_eq!(secret.outcome, Outcome::Exceeds, "SECRET-HIDDEN-1 against no secret: {secret:?}");
        if cfg!(unix) {
            let all = verdict(&[hidden("fs.read=PATH", "fs_read")], &[], &grants(&["fs.read=/"]));
            assert_eq!(all.outcome, Outcome::Within, "the whole filesystem bounds every computed path: {all:?}");
        }
    }

    #[test]
    fn the_counterexample_is_the_first_call_in_source_order_and_exceeds_outranks_the_rest() {
        // Source order differs from the list's order AND from its reverse (M280: a reversed list survived a
        // witness whose source-first request happened to be listed last).
        let reqs = vec![
            mint("fs.read=./middle", "fs_read", Some("./middle"), 50),
            mint("fs.read=./first", "fs_read", Some("./first"), 10),
            hidden("net=HOST", "http"),
            mint("fs.read=./last", "fs_read", Some("./last"), 90),
        ];
        let v = verdict(&reqs, &["Teleport".to_string()], &grants(&["net=example.com"]));
        assert_eq!(v.outcome, Outcome::Exceeds, "{v:?}");
        let c = v.counterexample().expect("a counterexample");
        assert_eq!(c.grant, "fs.read=./first", "the first call in the source, not the first in the list: {v:?}");
        let order: Vec<&str> = v.findings.iter().map(|f| f.grant.as_str()).collect();
        assert_eq!(
            order,
            ["fs.read=./first", "fs.read=./middle", "fs.read=./last", "net=HOST", ""],
            "source order, unplaced last: {v:?}"
        );
        assert_eq!(v.findings.len(), 5, "every request that is not within is named: {v:?}");
        assert_eq!(v.coverage["net"], Some(Outcome::Inconclusive));
        assert_eq!(v.coverage["Teleport"], Some(Outcome::Unsupported));
        assert_eq!(v.coverage["clock"], None, "a dimension the program never asks for is not-requested");
        let only = verdict(&[hidden("net=HOST", "http")], &["Teleport".to_string()], &grants(&["net=example.com"]));
        assert_eq!(only.outcome, Outcome::Unsupported, "unsupported outranks inconclusive: {only:?}");
    }

    #[test]
    fn a_special_use_host_is_within_only_under_its_explicit_spelling() {
        let req = [mint("net.special=127.0.0.1", "http", Some("127.0.0.1"), 3)];
        assert_eq!(verdict(&req, &[], &grants(&["net.special=127.0.0.1"])).outcome, Outcome::Within);
        assert_eq!(verdict(&req, &[], &grants(&["net=example.com"])).outcome, Outcome::Exceeds);
    }

    /// Every key the grant grammar parses, but `exec.native`, is a dimension the coverage names — so a
    /// form added to `GRANT_FORMS` cannot be asked for and left out of the answer.
    #[test]
    fn every_grant_form_is_a_dimension_of_the_coverage() {
        for f in delulu_runtime::broker::GRANT_FORMS {
            let key = f.key.trim_end_matches(':');
            if key == "exec.native" {
                continue;
            }
            assert!(DIMENSIONS.contains(&key), "grant form `{key}` is not in within::DIMENSIONS");
        }
    }

    /// Every capability kind the checker can report on a capability line has a grant form the
    /// requirement list names — and an unknown kind is reported unsupported, never dropped (the skip
    /// branch, §11.2). The match below has no wildcard: a new `ResourceKind` does not compile until it is
    /// placed here.
    #[test]
    fn every_capability_kind_has_a_grant_form_and_an_unknown_one_is_unsupported() {
        use delulu_check::ResourceKind as K;
        let all = [
            K::FsRead,
            K::FsWrite,
            K::Console,
            K::Http,
            K::Clock,
            K::Rand,
            K::Declassify,
            K::PluginHost,
            K::Python,
            K::ForeignLoad,
            K::Actuator,
            K::Sensor,
            K::Compute,
        ];
        let walk = crate::cli::requested_scopes(&[]);
        for k in all {
            // Declassify, ForeignLoad and Python are never capability LINES (`authority_report` reports
            // them as an effect and in the foreign section, each with its own grant form).
            let on_a_line = match k {
                K::Declassify | K::ForeignLoad | K::Python => false,
                K::FsRead
                | K::FsWrite
                | K::Console
                | K::Http
                | K::Clock
                | K::Rand
                | K::PluginHost
                | K::Actuator
                | K::Sensor
                | K::Compute => true,
            };
            if !on_a_line {
                continue;
            }
            let report = json!({ "capabilities": [{ "kind": k.name(), "scopes": [] }], "effects": [], "secrets": [], "foreign_calls": [] });
            let (reqs, unsupported) = crate::cli::requirements(&report, &walk);
            assert!(!reqs.is_empty() && unsupported.is_empty(), "{}: {reqs:?} {unsupported:?}", k.name());
        }
        let report =
            json!({ "capabilities": [{ "kind": "Teleport", "scopes": [] }], "effects": [], "secrets": [], "foreign_calls": [] });
        let (reqs, unsupported) = crate::cli::requirements(&report, &walk);
        assert!(reqs.is_empty() && unsupported == ["Teleport"], "{reqs:?} {unsupported:?}");
        assert_eq!(verdict(&reqs, &unsupported, &grants(&["console"])).outcome, Outcome::Unsupported);
    }
}
