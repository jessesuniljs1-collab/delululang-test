//! P4-11: `delulu atlas chain` — the V2 chain as one view over `atlas/1`.
//!
//! What is held here:
//! - **the snapshot on the corpus** (the roadmap's verification): the chain of every shipped example
//!   — each single-file program and each package — is byte-for-byte the committed snapshot under
//!   `tests/snapshots/atlas_chain/`. The chain carries nothing about the host, so one snapshot holds
//!   on every operating system. `DELULU_BLESS=1` rewrites them; a diff is then a reviewable change.
//! - **it is a view, not a second model**: for every example file the chain agrees with the commands
//!   it joins — the sandbox link with `delulu sandbox policy` (same hash, same refusal words), the
//!   authority link with `delulu authority`, the resources with the Atlas's own resource nodes, and
//!   every function it names is an Atlas function.
//! - **the links that were empty before P4-11**: an actuator, a sensor, a compute device and a plugin
//!   host are resources with the scopes the code names; an actor program lists its actors.
//! - the profile flag, and refusals.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn delulu(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_delulu")).args(args).current_dir(repo()).env("DELULU_NO_FIRST_RUN", "1").output().unwrap()
}

fn json_of(args: &[&str]) -> Value {
    let out = delulu(args);
    assert_eq!(out.status.code(), Some(0), "{args:?}: {}", String::from_utf8_lossy(&out.stderr));
    serde_json::from_slice(&out.stdout).unwrap()
}

const LINKS: [&str; 10] = [
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

/// The corpus: every example file that checks on its own, and every example package. A package's
/// module that only checks inside its package (it imports a sibling) is covered by the package.
fn corpus() -> Vec<String> {
    let mut files = Vec::new();
    let mut stack = vec![repo().join("examples")];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.join("delulu.toml").is_file() {
                    files.push(p.clone());
                }
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "delulu") {
                files.push(p);
            }
        }
    }
    let root = repo();
    let mut out: Vec<String> = files
        .iter()
        .map(|p| p.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/"))
        .filter(|t| {
            // Package modules that import a sibling cannot be checked alone; their package is.
            !(t.ends_with(".delulu") && delulu(&["check", t, "--json"]).status.code() != Some(0) && in_package(t))
        })
        .collect();
    out.sort();
    out
}

fn in_package(target: &str) -> bool {
    let mut p = repo().join(target);
    while p.pop() {
        if p.join("delulu.toml").is_file() {
            return true;
        }
        if p == repo() {
            break;
        }
    }
    false
}

fn snapshot_name(target: &str) -> String {
    target.trim_start_matches("examples/").trim_end_matches(".delulu").replace('/', "__") + ".json"
}

#[test]
fn the_chain_of_every_example_is_its_committed_snapshot() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/atlas_chain");
    let bless = std::env::var_os("DELULU_BLESS").is_some();
    if bless {
        std::fs::create_dir_all(&dir).unwrap();
    }
    let targets = corpus();
    assert!(targets.len() >= 12, "the corpus shrank: {targets:?}");
    let mut drift = Vec::new();
    let mut expected_names = Vec::new();
    for t in &targets {
        let v = json_of(&["atlas", "chain", t, "--json"]);
        assert_eq!(v["view"], "chain");
        assert_eq!(v["of"], "atlas/1");
        assert_eq!(v["links"], json!(LINKS), "the order is stated as data");
        let keys: Vec<&str> = v["chain"].as_object().unwrap().keys().map(String::as_str).collect();
        let mut want = LINKS.to_vec();
        want.sort_unstable();
        assert_eq!(keys, want, "{t}: exactly the ten links");
        let text = serde_json::to_string_pretty(&v["chain"]).unwrap() + "\n";
        assert!(!text.contains("\\\\"), "{t}: a host path separator leaked into the chain");
        let name = snapshot_name(t);
        expected_names.push(name.clone());
        let path = dir.join(&name);
        if bless {
            std::fs::write(&path, &text).unwrap();
            continue;
        }
        let committed = std::fs::read_to_string(&path).unwrap_or_default().replace("\r\n", "\n");
        if committed != text {
            drift.push(format!("{t} → {}", path.display()));
        }
    }
    // No snapshot is left behind for an example that no longer exists.
    let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
        .map(|rd| rd.flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect())
        .unwrap_or_default();
    on_disk.sort();
    expected_names.sort();
    assert_eq!(on_disk, expected_names, "the snapshot set is the corpus");
    assert!(
        drift.is_empty(),
        "the chain changed for: {drift:#?}\nIf the change is intended, rerun with DELULU_BLESS=1 and review the diff."
    );
}

#[test]
fn the_chain_is_a_view_that_agrees_with_every_command_it_joins() {
    for t in corpus().iter().filter(|t| t.ends_with(".delulu")) {
        let chain = json_of(&["atlas", "chain", t, "--json"])["chain"].clone();
        let policy = json_of(&["sandbox", "policy", t, "--json"])["policy"].clone();
        let sp = &chain["sandbox_policy"];
        assert_eq!(sp["policy_hash"], policy["policy_hash"], "{t}");
        assert_eq!(sp["limits"], policy["limits"], "{t}");
        assert_eq!(sp["unsupported_surface"], policy["unsupported_surface"], "{t}: the same refusal words");
        assert_eq!(sp["carried"], json!(policy["unsupported_surface"].is_null()), "{t}");

        let auth = json_of(&["authority", t, "--json"])["authority"].clone();
        assert_eq!(chain["authority"]["effects"], auth["effects"], "{t}");
        assert_eq!(chain["authority"]["required_grants"], auth["required_grants"], "{t}");
        assert_eq!(chain["execution_boundary"]["custody"], auth["custody"], "{t}: an ordinary run's custody");
        assert_eq!(chain["execution_boundary"]["foreign"]["isolation"], auth["foreign_isolation"], "{t}");

        let atlas = json_of(&["atlas", t, "--json"])["atlas"].clone();
        let nodes = atlas["nodes"].as_array().unwrap();
        let mut res: Vec<&str> = nodes.iter().filter(|n| n["kind"] == "resource").map(|n| n["id"].as_str().unwrap()).collect();
        res.sort_unstable();
        let chain_res: Vec<&str> = chain["resources"].as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap()).collect();
        assert_eq!(chain_res, res, "{t}: the resources ARE the Atlas's");
        let fns: Vec<&str> = nodes.iter().filter(|n| n["kind"] == "function").map(|n| n["id"].as_str().unwrap()).collect();
        for e in chain["effects"].as_array().unwrap() {
            for f in e["performed_by"].as_array().unwrap() {
                assert!(fns.contains(&f.as_str().unwrap()), "{t}: {f} is not an Atlas function");
            }
        }
    }
}

#[test]
fn devices_plugins_and_actors_are_on_the_chain_with_the_scopes_the_code_names() {
    let dir = std::env::temp_dir().join(format!("delulu-atlas-chain-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("rig.delulu");
    std::fs::write(
        &f,
        "module rig\n\nfn main(root: Root) ! {Read, Write, Actuate} {\n    let out = root.console()\n    \
         let arm = root.actuator(\"arm0/elbow\")\n    let temp = root.sensor(\"bay0/temp\")\n    \
         let gpu = root.compute(\"gpu0\")\n    let h = root.plugin_host()\n    out.println(\"ready\")\n}\n",
    )
    .unwrap();
    let f = f.to_string_lossy().to_string();
    let v = json_of(&["atlas", "chain", &f, "--json"]);
    let c = &v["chain"];
    let devices: Vec<(&str, &Value)> =
        c["devices"]["devices"].as_array().unwrap().iter().map(|d| (d["class"].as_str().unwrap(), &d["requested_scopes"])).collect();
    assert_eq!(
        devices,
        vec![("actuator", &json!(["arm0/elbow"])), ("compute", &json!(["gpu0"])), ("sensor", &json!(["bay0/temp"]))],
        "{c:#}"
    );
    assert_eq!(c["devices"]["actuate_performed_by"], json!(["fn:rig/rig.main"]));
    assert_eq!(c["plugins"]["hosts"][0]["id"], "res:plugin_host:*");
    assert_eq!(c["sandbox_policy"]["carried"], false);
    assert_eq!(c["sandbox_policy"]["unsupported_surface"], "Actuator, Compute, PluginHost, Sensor");
    // The Atlas itself now holds them — additive: new ids, and an attribute only where there are scopes.
    let atlas = json_of(&["atlas", &f, "--json"])["atlas"].clone();
    let arm = atlas["nodes"].as_array().unwrap().iter().find(|n| n["id"] == "res:actuator:*").expect("an actuator resource");
    assert_eq!(arm["requested_scopes"], json!(["arm0/elbow"]));
    let console = atlas["nodes"].as_array().unwrap().iter().find(|n| n["id"] == "res:console:stdio").unwrap();
    assert!(console.get("requested_scopes").is_none(), "no scopes, no attribute: {console}");
    assert!(atlas["edges"].as_array().unwrap().iter().any(|e| e["from"] == "fn:rig/rig.main" && e["to"] == "res:sensor:*" && e["kind"] == "requires"));

    let actors = json_of(&["atlas", "chain", "examples/guide/06_actors.delulu", "--json"])["chain"]["actors"].clone();
    let names: Vec<&str> = actors["actors"].as_array().unwrap().iter().map(|a| a["actor"].as_str().unwrap()).collect();
    assert_eq!(names, vec!["Supervisor", "Worker"]);
    assert!(actors["async_performed_by"].as_array().unwrap().iter().any(|f| f == "fn:guide.actors/guide.actors.Supervisor.dispatch"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_profile_is_the_sandboxs_and_a_package_says_it_has_no_sandboxed_run() {
    let t = "examples/guide/04_effects.delulu";
    let hostile = json_of(&["atlas", "chain", t, "--sandbox-profile", "hostile-agent", "--json"])["chain"]["sandbox_policy"].clone();
    let policy = json_of(&["sandbox", "policy", t, "--sandbox-profile", "hostile-agent", "--json"])["policy"].clone();
    assert_eq!(hostile["profile"], "hostile-agent");
    assert_eq!(hostile["policy_hash"], policy["policy_hash"]);
    let contained = json_of(&["atlas", "chain", t, "--json"])["chain"]["sandbox_policy"].clone();
    assert_ne!(hostile["policy_hash"], contained["policy_hash"], "the profile is part of the policy");

    let pkg = json_of(&["atlas", "chain", "examples/greeter", "--json"])["chain"]["sandbox_policy"].clone();
    assert_eq!(pkg["carried"], Value::Null);
    assert!(pkg["carried_note"].as_str().unwrap().contains("one file"), "{pkg}");

    for bad in [
        vec!["atlas", "chain", t, "--sandbox-profile", "lax"],
        vec!["atlas", "chain", t, "--force"],
        vec!["atlas", "chain", t, t],
    ] {
        assert_eq!(delulu(&bad).status.code(), Some(2), "{bad:?}");
    }
    let human = String::from_utf8(delulu(&["atlas", "chain", t]).stdout).unwrap();
    for word in ["program", "authority", "effects", "sandbox policy", "execution boundary"] {
        assert!(human.contains(word), "the human reading names `{word}`:\n{human}");
    }
}
