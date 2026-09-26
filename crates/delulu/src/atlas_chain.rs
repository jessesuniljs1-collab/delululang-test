//! P4-11: `delulu atlas chain` — the Atlas's V2 chain, one ordered view an auditor can read top to
//! bottom and query by name.
//!
//! The Atlas maps what a program is made of; the authority report says what it may do; the sandbox
//! preview says what a confined run would hold it to. Each answered its own question, and an auditor
//! asking "from this program down to the machine, what is there?" had to join three documents by hand.
//! The chain is that join, in a fixed order:
//!
//! **program → authority → effects → capabilities → sandbox policy → resources → plugins → actors →
//! devices → execution boundary**
//!
//! It is a VIEW over `atlas/1`, not a new graph and not a second model: every fact in it is read from
//! the Atlas this command builds (its nodes, its `performs`/`requires` edges, its embedded authority
//! report) and from the sandbox policy `delulu sandbox policy` derives — the same pure derivation, so
//! the same hash. Nothing is inferred that those do not state. And it is not the Survey: it describes
//! one PROGRAM, never the repository.
//!
//! Deterministic by construction — every list sorted, nothing about the host in it — which is what
//! lets `tests/atlas_chain.rs` hold a snapshot of the whole corpus.

use serde_json::{json, Value};

/// The chain's links, in order. The JSON carries exactly these keys, in this order, under `chain`.
pub(crate) const LINKS: [&str; 10] = [
    "program",
    "authority",
    "effects",
    "capabilities",
    "sandbox_policy",
    "resources",
    "plugins",
    "actors",
    "devices",
    "execution_boundary",
];

/// Resource classes that are physical or accelerator devices (Constitution §7, Stage 10 Tracks D/F).
const DEVICE_CLASSES: [&str; 3] = ["actuator", "compute", "sensor"];

fn strs(v: &Value) -> Vec<String> {
    v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default()
}

/// The Atlas's nodes of one kind.
fn of_kind<'a>(nodes: &'a [Value], kind: &'a str) -> impl Iterator<Item = &'a Value> + 'a {
    nodes.iter().filter(move |n| n["kind"] == kind)
}

/// Every node id with an edge of `kind` INTO `to`, sorted.
fn sources(edges: &[Value], kind: &str, to: &str) -> Vec<String> {
    let mut out: Vec<String> = edges
        .iter()
        .filter(|e| e["kind"] == kind && e["to"] == to)
        .filter_map(|e| e["from"].as_str())
        .filter(|f| f.starts_with("fn:"))
        .map(str::to_string)
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Whether a sandboxed run would carry the program: `Some(None)` it would, `Some(Some(words))` it
/// would refuse and why, `None` when there is no sandboxed run of this target to ask about.
type Carried = Option<Option<String>>;

/// The chain for an Atlas, under the sandbox `profile`. `atlas` is the `atlas/1` document.
pub(crate) fn chain(atlas: &Value, profile: crate::policy::Profile, carried: Carried) -> Value {
    let nodes = atlas["nodes"].as_array().cloned().unwrap_or_default();
    let edges = atlas["edges"].as_array().cloned().unwrap_or_default();
    let auth = &atlas["authority"];
    let ids = |k: &str| -> Vec<String> {
        let mut v: Vec<String> = of_kind(&nodes, k).filter_map(|n| n["id"].as_str().map(str::to_string)).collect();
        v.sort();
        v
    };

    // ---- effects: each effect and the functions whose checked row performs it.
    let effects: Vec<Value> = {
        let mut names: Vec<String> = of_kind(&nodes, "effect").filter_map(|n| n["name"].as_str().map(str::to_string)).collect();
        names.sort();
        names
            .iter()
            .map(|e| json!({ "effect": e, "performed_by": sources(&edges, "performs", &format!("effect:{e}")) }))
            .collect()
    };
    let performers = |e: &str| sources(&edges, "performs", &format!("effect:{e}"));

    // ---- resources: every resource node, with who requires or declassifies it.
    let resource = |n: &Value| {
        let id = n["id"].as_str().unwrap_or("");
        json!({
            "id": id,
            "class": n["resource_class"],
            "pattern": n["pattern"],
            "requested_scopes": n.get("requested_scopes").cloned().unwrap_or_else(|| json!([])),
            "required_by": sources(&edges, "requires", id),
            "declassified_by": sources(&edges, "declassifies", id),
        })
    };
    let mut resources: Vec<Value> = of_kind(&nodes, "resource").map(resource).collect();
    resources.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    let in_classes = |classes: &[&str]| -> Vec<Value> {
        resources.iter().filter(|r| r["class"].as_str().is_some_and(|c| classes.contains(&c))).cloned().collect()
    };

    // ---- capabilities: as the authority report states them, each with the resources it designates.
    let mut capabilities: Vec<Value> = auth["capabilities"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|mut c| {
            let class = delulu_atlas::class_of_kind(c["kind"].as_str().unwrap_or(""));
            let designates: Vec<Value> = match class {
                Some(cl) => resources.iter().filter(|r| r["class"] == cl).map(|r| r["id"].clone()).collect(),
                None => Vec::new(),
            };
            c["resource_class"] = json!(class);
            c["resources"] = json!(designates);
            c
        })
        .collect();
    capabilities.sort_by(|a, b| a["kind"].as_str().cmp(&b["kind"].as_str()));

    // ---- actors: the Atlas names an actor's members `Actor.member`.
    let mut actors: std::collections::BTreeMap<String, (Vec<String>, std::collections::BTreeSet<String>)> =
        std::collections::BTreeMap::new();
    for n in of_kind(&nodes, "function") {
        let (Some(id), Some(name)) = (n["id"].as_str(), n["name"].as_str()) else { continue };
        if let Some((actor, _)) = name.split_once('.') {
            let entry = actors.entry(actor.to_string()).or_default();
            entry.0.push(id.to_string());
            entry.1.extend(strs(&n["effects"]));
        }
    }
    let actors: Vec<Value> = actors
        .into_iter()
        .map(|(actor, (mut members, effects))| {
            members.sort();
            json!({ "actor": actor, "members": members, "effects": effects })
        })
        .collect();

    // ---- the sandbox policy: the same pure derivation as `delulu sandbox policy`, so the same hash.
    let policy = crate::policy::SandboxPolicy::derive(1, profile, None, crate::policy::Mode::Strict);
    let preview = policy.to_json("process", &[]);

    let foreign_nodes = ids("foreign");
    // Custody and foreign isolation belong to a RUN (`--broker daemon`, `--foreign-isolation`), so the
    // Atlas leaves them out; the boundary states an ordinary run's, and says that is what they are.
    let run_mode = crate::cli::ordinary_run_mode();
    json!({
        "program": {
            "name": auth["program"],
            "root": atlas["root"],
            "packages": ids("package"),
            "modules": ids("module"),
            "functions": of_kind(&nodes, "function").count(),
        },
        "authority": {
            "effects": auth["effects"],
            "required_grants": auth["required_grants"],
            "pure": auth["effects"].as_array().is_none_or(|e| e.is_empty()),
            "secrets": auth["secrets"],
        },
        "effects": effects,
        "capabilities": capabilities,
        "sandbox_policy": {
            "profile": profile.name(),
            "requested_level": preview["requested_level"],
            "backend": preview["backend"],
            "mode": preview["mode"],
            "limits": preview["limits"],
            "policy_hash": preview["policy_hash"],
            "carried": carried.as_ref().map(Option::is_none),
            "unsupported_surface": carried.clone().flatten(),
            "carried_note": if carried.is_some() {
                Value::Null
            } else {
                json!("`--sandbox` runs one file: ask `atlas chain` of the file to be run")
            },
        },
        "resources": resources,
        "plugins": {
            "hosts": in_classes(&["plugin_host"]),
            "load_performed_by": performers("Load"),
            "contained": auth["contained_plugins"],
        },
        "actors": {
            "actors": actors,
            "async_performed_by": performers("Async"),
        },
        "devices": {
            "devices": in_classes(&DEVICE_CLASSES),
            "actuate_performed_by": performers("Actuate"),
        },
        "execution_boundary": {
            "ordinary_run": "in-process (L0): the language and custody in one process, no OS boundary",
            "run_mode_note": "custody and foreign isolation are an ordinary run's; `--broker daemon` and `--foreign-isolation process` change them",
            "sandboxed_run": {
                "level": 1,
                "backend": "process",
                "guest_performs_effects": false,
                "carried": carried.as_ref().map(Option::is_none),
            },
            "custody": run_mode["custody"],
            "foreign": {
                "isolation": run_mode["foreign_isolation"],
                "calls": auth["foreign_calls"],
                "boundaries": foreign_nodes,
            },
        },
    })
}

fn usage() -> i32 {
    eprintln!("usage: delulu atlas chain [<file.delulu | package-dir | atlas.json>] [--sandbox-profile dev|contained|hostile-agent] [--json]");
    2
}

/// `delulu atlas chain [target] [--sandbox-profile P] [--json]`.
pub(crate) fn cmd_chain(args: &[String]) -> i32 {
    let mut json_out = false;
    let mut profile = crate::policy::Profile::Contained;
    let mut target: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        let name = if a == "--sandbox-profile" {
            i += 1;
            match args.get(i) {
                Some(v) => Some(v.clone()),
                None => return usage(),
            }
        } else {
            a.strip_prefix("--sandbox-profile=").map(str::to_string)
        };
        if let Some(name) = name {
            match crate::policy::Profile::parse(&name) {
                Some(p) => profile = p,
                None => {
                    eprintln!("error: `{name}` is not a sandbox profile (dev, contained, hostile-agent)");
                    return 2;
                }
            }
        } else if a == "--json" {
            json_out = true;
        } else if a.starts_with('-') {
            eprintln!("error: `atlas chain` does not know the option `{a}`");
            eprintln!("  nothing was done — an option nobody understood is refused, never ignored");
            return 2;
        } else if target.replace(a.to_string()).is_some() {
            eprintln!("error: `atlas chain` takes one target");
            return 2;
        }
        i += 1;
    }
    let target = target.unwrap_or_else(|| ".".to_string());
    let atlas = match crate::cli::atlas_from_target(&target, 10, json_out) {
        Ok(a) => a,
        Err(c) => return c,
    };
    // Whether `--sandbox` would carry it is a question about ONE file — the guest's own rule on that
    // file's text, exactly what `delulu sandbox policy` answers. A package or a saved Atlas has no
    // sandboxed run to ask about, and says so rather than guessing.
    let carried: Carried = if std::path::Path::new(&target).is_file() && target.ends_with(".delulu") {
        std::fs::read_to_string(&target).ok().map(|src| crate::guest::unsupported_surface(&src))
    } else {
        None
    };
    let c = chain(&atlas.to_json(), profile, carried);
    if json_out {
        // A JSON object has no order, so the order is stated as data: `links` names the chain's keys
        // top to bottom. `of` says which graph this is a view over.
        crate::cli::print_success_envelope(
            "atlas",
            json!({ "view": "chain", "of": delulu_atlas::SCHEMA, "links": LINKS, "chain": c }),
        );
        return 0;
    }
    print!("{}", render(&c));
    0
}

/// The human reading: one section per link, in order.
fn render(c: &Value) -> String {
    use std::fmt::Write as _;
    let list = |v: &Value| -> String {
        let xs = strs(v);
        if xs.is_empty() { "none".to_string() } else { xs.join(", ") }
    };
    let mut s = String::new();
    let p = &c["program"];
    let _ = writeln!(s, "{:<18} {} (root {}; {} function(s))", "program", p["name"].as_str().unwrap_or("?"), p["root"].as_str().unwrap_or("?"), p["functions"]);
    let a = &c["authority"];
    let _ = writeln!(s, "{:<18} effects {}; grants {}", "authority", list(&a["effects"]), list(&a["required_grants"]));
    for e in c["effects"].as_array().into_iter().flatten() {
        let _ = writeln!(s, "{:<18} {} ← {}", "effects", e["effect"].as_str().unwrap_or(""), list(&e["performed_by"]));
    }
    for cap in c["capabilities"].as_array().into_iter().flatten() {
        let _ = writeln!(s, "{:<18} {} {} → {}", "capabilities", cap["kind"].as_str().unwrap_or(""), list(&cap["requested_scopes"]), list(&cap["resources"]));
    }
    let sp = &c["sandbox_policy"];
    let carried = match (sp["carried"].as_bool(), sp["unsupported_surface"].as_str()) {
        (Some(true), _) => "the channel carries this program".to_string(),
        (Some(false), Some(u)) => format!("NOT carried yet: {u} — `--sandbox` would refuse it"),
        _ => sp["carried_note"].as_str().unwrap_or("").to_string(),
    };
    let _ = writeln!(s, "{:<18} {} (L{}, {} bytes, {} s); {carried}", "sandbox policy", sp["profile"].as_str().unwrap_or(""), sp["requested_level"], sp["limits"]["memory_bytes"], sp["limits"]["cpu_seconds"]);
    for r in c["resources"].as_array().into_iter().flatten() {
        let _ = writeln!(s, "{:<18} {} ← {}", "resources", r["id"].as_str().unwrap_or(""), list(&r["required_by"]));
    }
    let pl = &c["plugins"];
    let hosts: Vec<Value> = pl["hosts"].as_array().cloned().unwrap_or_default();
    let _ = writeln!(s, "{:<18} {} host(s); Load ← {}", "plugins", hosts.len(), list(&pl["load_performed_by"]));
    for act in c["actors"]["actors"].as_array().into_iter().flatten() {
        let _ = writeln!(s, "{:<18} {}: {}", "actors", act["actor"].as_str().unwrap_or(""), list(&act["members"]));
    }
    for d in c["devices"]["devices"].as_array().into_iter().flatten() {
        let _ = writeln!(s, "{:<18} {} ← {}", "devices", d["id"].as_str().unwrap_or(""), list(&d["required_by"]));
    }
    let b = &c["execution_boundary"];
    let _ = writeln!(s, "{:<18} {}; sandboxed: L1 guest performs no effect, the host performs every one; custody {}", "execution boundary", b["ordinary_run"].as_str().unwrap_or(""), b["custody"].as_str().unwrap_or("?"));
    s
}
