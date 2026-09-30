//! PS-E-05 (a), D-V2-82: `delulu sandbox policy <file> --format openshell` — the wall an OpenShell
//! sandbox should put around `delulu run`, written by DeluluLang from the program's own authority.
//!
//! OpenShell (NVIDIA's open agent runtime, studied in `docs/DELULULANG_V2/V2_OPENSHELL_STUDY.md`)
//! governs binaries it cannot read, so its policy is a document someone writes. DeluluLang can write
//! that document instead of a person, because a program's authority is computed from its text. This
//! module EMITS a document in OpenShell's published policy format (`version: 1`); nothing of OpenShell
//! is copied or linked (D-V2-52), and the format is named only to identify it.
//!
//! **The one rule (study §4.5): what this emits never allows more than the program's authority and
//! the operator's grants allow.** So:
//!
//! - a grant is emitted only for a capability KIND the program's authority report names (kinds are
//!   static and sound — P16; a scope the program computes is still bounded by the grant, which is why
//!   the grant's scope, not the source's literal, is what is emitted);
//! - a grant the program cannot use is listed as `omitted`, never written;
//! - a grant OpenShell cannot express (a special-use address, a device, code loading) REFUSES the
//!   export by name — never dropped in silence, never widened;
//! - what OpenShell does not model at all (the console, the clock, budgets) is listed as
//!   `unrepresented`, because DeluluLang's own grant still governs it inside the wall;
//! - where the wall is NARROWER than the grant (HTTPS on port 443 only), that is listed too.
//!
//! The runtime's own read-only paths (`RUNTIME_PATHS`) and the program's file are the two things on
//! the wall that no grant names; both are reported as such. `openshell-prover` checks the document
//! against a boundary the operator writes (`.github/workflows/openshell.yml`), so two independent
//! tools — this derivation and NVIDIA's SMT model — answer the same question about the same program.

use serde_json::{json, Value};

/// Where the repository's image (`Dockerfile`, RW 7.3) installs the binary: the ONE executable the
/// emitted network rules name, because only `delulu` performs a program's effects.
pub(crate) const DEFAULT_BINARY: &str = "/usr/local/bin/delulu";

/// OpenShell's own name for its unprivileged workload identity (it refuses root in any spelling).
pub(crate) const DEFAULT_RUN_AS: &str = "sandbox";

/// What the `delulu` executable itself reads to start and to reach a granted host (its libraries, the
/// dynamic loader, CA certificates, the resolver's files) — the runtime's, never a program's. They are
/// three of the read-only paths OpenShell's own baseline adds for a policy with a network rule.
pub(crate) const RUNTIME_PATHS: [&str; 3] = ["/usr", "/lib", "/etc"];

/// OpenShell's baseline additions (read at `36b0386`, `default-policy.mdx`): when the EFFECTIVE policy
/// has a network rule, it adds these unless the policy lists them — reported, so nobody reads the
/// emitted document as the whole wall.
const BASELINE_READ_ONLY: [&str; 7] = ["/usr", "/lib", "/etc", "/app", "/var/log", "/proc", "/dev/urandom"];
const BASELINE_READ_WRITE: [&str; 2] = ["/tmp", "/dev/null"];

/// The only port the emitted endpoints name. DeluluLang fetches `https://` on any port a URL spells;
/// the wall allows 443, and says so under `narrowed`.
const PORT: u16 = 443;

/// The only method the language's `http` performs (`Http.get`, the primitive table). OpenShell's
/// `read-only` preset also allows `HEAD` and `OPTIONS` — wider than the program's authority — so the
/// rule is written out instead. P9-05 widens it only when the language does.
const METHOD: &str = "GET";

/// OpenShell's limits on a filesystem policy (`schema.mdx`).
const MAX_PATH_BYTES: usize = 4096;
const MAX_PATHS: usize = 256;

pub(crate) struct Options {
    /// The directory `delulu run` will run in INSIDE the sandbox: relative paths resolve against it,
    /// never against this machine's working directory, which names a path on the wrong machine.
    pub workdir: Option<String>,
    pub binary: String,
    pub run_as_user: String,
    pub run_as_group: String,
}

/// Why no document was written. `Usage` is exit 2 (the request was malformed); `Unrepresentable` is
/// exit 1 (a well-formed request that OpenShell's format cannot state without widening it).
#[derive(Debug)]
pub(crate) enum Refused {
    Usage(String),
    Unrepresentable(String),
}

pub(crate) struct Export {
    /// The policy document, YAML as OpenShell reads it.
    pub document: String,
    /// The `openshell` payload of `--json`: the same policy as JSON, and every account above.
    pub payload: Value,
}

/// Parse `--run-as UID[:GID]` (numeric, never root) or `sandbox`.
pub(crate) fn run_as(spec: &str) -> Result<(String, String), Refused> {
    let (u, g) = spec.split_once(':').unwrap_or((spec, spec));
    for id in [u, g] {
        if id == DEFAULT_RUN_AS {
            continue;
        }
        match id.parse::<u64>() {
            Ok(n) if (1..=4_294_967_294).contains(&n) && id == n.to_string() => {}
            _ => {
                return Err(Refused::Usage(format!(
                    "`--run-as {spec}`: each id is `sandbox` or a number from 1 to 4294967294 — OpenShell never runs a workload as root"
                )))
            }
        }
    }
    Ok((u.to_string(), g.to_string()))
}

/// A path as OpenShell takes it: absolute, no `.` or empty segment, no `..`, no control character. A
/// relative path is joined to `workdir`; without one it is refused rather than resolved on this machine.
fn absolute(path: &str, workdir: Option<&str>, what: &str) -> Result<String, Refused> {
    if path.chars().any(char::is_control) {
        return Err(Refused::Usage(format!("{what} holds a control character, which names no path a policy should carry")));
    }
    // A Windows spelling names a path on the machine running `delulu`, never one inside an OpenShell
    // sandbox, which is Linux: `C:\x` is not absolute there, and `.\x` is a file named with a backslash.
    let b = path.as_bytes();
    if path.contains('\\') || (b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':') {
        return Err(Refused::Usage(format!(
            "{what} is a Windows spelling (a backslash or a drive); an OpenShell sandbox is Linux — name the path \
             as it is inside the sandbox, with `/`"
        )));
    }
    let joined = if path.starts_with('/') {
        path.to_string()
    } else {
        match workdir {
            Some(w) => format!("{w}/{path}"),
            None => {
                return Err(Refused::Usage(format!(
                    "{what} is relative, and OpenShell takes only absolute paths — pass `--workdir DIR`, the directory \
                     `delulu run` will run in inside the sandbox (this machine's working directory names a path on the \
                     wrong machine)"
                )))
            }
        }
    };
    let mut parts: Vec<&str> = Vec::new();
    for seg in joined.split('/') {
        match seg {
            "" | "." => {}
            // Refused, never collapsed: the kernel resolves `..` AFTER following a link, so `a/link/..`
            // collapsed here could name a directory the sandbox never reaches — and the wall would
            // then cover a path no grant does (HANDOFF §11.4: what else spells this?). OpenShell
            // refuses `..` too.
            ".." => {
                return Err(Refused::Usage(format!(
                    "{what} holds `..`, which only the sandbox's own filesystem can resolve (through its links); \
                     spell the path without it"
                )))
            }
            s => parts.push(s),
        }
    }
    let out = format!("/{}", parts.join("/"));
    if out.len() > MAX_PATH_BYTES {
        return Err(Refused::Unrepresentable(format!("{what} is longer than OpenShell's {MAX_PATH_BYTES} bytes")));
    }
    Ok(out)
}

/// A granted host as an OpenShell endpoint host. DeluluLang's `*.suffix` matches one label or more
/// before the suffix and never the apex (`prim::host_matches`, C85) — OpenShell's `**.suffix`, which
/// needs at least one label. OpenShell refuses a wildcard of fewer than three labels.
fn endpoint_host(grant: &str, host: &str) -> Result<String, Refused> {
    let h = host.to_ascii_lowercase();
    if h.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(Refused::Usage(format!("`--grant {grant}` names no host")));
    }
    match h.strip_prefix("*.") {
        Some(suffix) => {
            if suffix.contains('*') {
                return Err(Refused::Unrepresentable(format!("`--grant {grant}`: a wildcard is written only as the first label")));
            }
            if suffix.split('.').filter(|l| !l.is_empty()).count() < 2 {
                return Err(Refused::Unrepresentable(format!(
                    "`--grant {grant}` matches every name under a top-level domain; OpenShell refuses a wildcard that wide, \
                     and this export will not widen it to `**`"
                )));
            }
            Ok(format!("**.{suffix}"))
        }
        None if h.contains('*') => {
            Err(Refused::Unrepresentable(format!("`--grant {grant}`: a wildcard is written only as the first label")))
        }
        None => Ok(h),
    }
}

/// Derive the document. `report` is the program's authority report (`delulu authority --json`'s
/// `authority`, from `cli::authority_of_text` — the same pipeline, not a second derivation);
/// `program_path` is the file or package `delulu run` will be given inside the sandbox; `grants` the
/// `--grant` specs, parsed by the runtime's own parser first.
pub(crate) fn export(
    program_path: &str,
    report: &Value,
    grants: &[String],
    limits: &crate::jail::Limits,
    opts: &Options,
) -> Result<Export, Refused> {
    let kinds: Vec<&str> = report["capabilities"]
        .as_array()
        .map(|caps| caps.iter().filter_map(|c| c["kind"].as_str()).collect())
        .unwrap_or_default();
    let uses = |kind: &str| kinds.contains(&kind);
    if let Some(w) = &opts.workdir {
        if !w.starts_with('/') {
            return Err(Refused::Usage(format!("`--workdir {w}` must be absolute: it names a directory inside the sandbox")));
        }
    }
    if !opts.binary.starts_with('/') || opts.binary.contains("..") {
        return Err(Refused::Usage(format!(
            "`--binary {}` must be the absolute, real path of `delulu` in the sandbox's image",
            opts.binary
        )));
    }
    let workdir = match &opts.workdir {
        Some(w) => Some(absolute(w, None, "`--workdir`")?),
        None => None,
    };
    let workdir = workdir.as_deref();

    let mut read_only: Vec<String> = RUNTIME_PATHS.iter().map(|p| p.to_string()).collect();
    let program = absolute(program_path, workdir, "the program's path")?;
    let mut read_write: Vec<String> = Vec::new();
    let mut hosts: Vec<(String, String)> = Vec::new(); // (endpoint host, the grant it came from)
    let mut emitted: Vec<Value> = Vec::new();
    let mut omitted: Vec<Value> = Vec::new();
    let mut unrepresented: Vec<Value> = Vec::new();
    let mut narrowed: Vec<Value> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    for spec in grants {
        // A secret is named, never parsed here: `secret:NAME=env:VAR` would read the environment, and
        // no value of it may reach the document or the envelope.
        if let Some(rest) = spec.strip_prefix("secret:") {
            let name = rest.split_once('=').map(|(n, _)| n).unwrap_or(rest);
            unrepresented.push(json!({
                "what": format!("secret:{name}"),
                "why": "a secret is held by DeluluLang's runtime and never enters the guest; the wall grants nothing for it (endpoint-bound secrets are P9-04)",
            }));
            continue;
        }
        let mut probe = delulu_runtime::broker::Grants::default();
        probe.add(spec).map_err(Refused::Usage)?;
        let (key, value) = match spec.split_once('=') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => (spec.trim(), ""),
        };
        match key {
            "console" | "clock" | "rand" | "declassify" => unrepresented.push(json!({
                "what": key,
                "why": "OpenShell does not fence it; DeluluLang's grant still governs it inside the wall",
            })),
            "fs.read" | "fs.write" => {
                let (kind, reads) = if key == "fs.read" { ("FsRead", true) } else { ("FsWrite", false) };
                if !uses(kind) {
                    omitted.push(json!({ "grant": spec, "why": format!("the program's authority names no {kind} capability") }));
                    continue;
                }
                let p = absolute(value, workdir, &format!("`--grant {spec}`"))?;
                if reads {
                    read_only.push(p.clone());
                } else {
                    if p == "/" {
                        return Err(Refused::Unrepresentable(format!(
                            "`--grant {spec}` writes the whole filesystem; OpenShell refuses `/` as a read-write path"
                        )));
                    }
                    read_write.push(p.clone());
                }
                emitted.push(json!({ "grant": spec, "as": if reads { "read_only" } else { "read_write" }, "path": p }));
            }
            "net" => {
                if !uses("Http") {
                    omitted.push(json!({ "grant": spec, "why": "the program's authority names no Http capability" }));
                    continue;
                }
                let h = endpoint_host(spec, value)?;
                if !hosts.iter().any(|(e, _)| *e == h) {
                    hosts.push((h.clone(), spec.clone()));
                }
                emitted.push(json!({ "grant": spec, "as": "network", "host": h, "port": PORT, "method": METHOD }));
                narrowed.push(json!({
                    "grant": spec,
                    "why": format!("HTTPS on port {PORT} only: DeluluLang would fetch any port a URL names"),
                }));
            }
            "net.special" => {
                return Err(Refused::Unrepresentable(format!(
                    "`--grant {spec}` names a special-use address; OpenShell never authorizes a loopback, link-local \
                     or unspecified destination, and this export does not map a private one yet"
                )))
            }
            "actuator" | "sensor" | "compute" => {
                return Err(Refused::Unrepresentable(format!(
                    "`--grant {spec}` is a device; OpenShell's policy has no device model"
                )))
            }
            "plugin" | "foreign.c" | "foreign.python" | "exec.native" => {
                return Err(Refused::Unrepresentable(format!(
                    "`--grant {spec}` loads code whose files and libraries this export does not map yet"
                )))
            }
            other => return Err(Refused::Usage(format!("`--grant {other}` has no OpenShell mapping"))),
        }
    }

    // The program's own file is on the wall because `delulu run` must read it; no grant names it.
    if !read_only.contains(&program) {
        read_only.push(program.clone());
    }
    read_only.sort();
    read_only.dedup();
    read_write.sort();
    read_write.dedup();
    // A path both read and written is read-write once: OpenShell lists each path with one access.
    read_only.retain(|p| !read_write.contains(p));
    if read_only.len() + read_write.len() > MAX_PATHS {
        return Err(Refused::Unrepresentable(format!("more than OpenShell's {MAX_PATHS} filesystem paths")));
    }
    hosts.sort();
    unrepresented.push(json!({
        "what": "budget",
        "why": format!(
            "OpenShell's policy has no budget; set the sandbox's own limits at creation — DeluluLang still holds each \
             run to {} bytes of memory and {} s of processor time",
            limits.memory_bytes, limits.cpu_seconds
        ),
    }));
    if !hosts.is_empty() {
        let added_ro: Vec<&str> = BASELINE_READ_ONLY.iter().copied().filter(|p| !read_only.iter().any(|r| r == p) && !read_write.iter().any(|r| r == p)).collect();
        let added_rw: Vec<&str> = BASELINE_READ_WRITE.iter().copied().filter(|p| !read_only.iter().any(|r| r == p) && !read_write.iter().any(|r| r == p)).collect();
        notes.push(format!(
            "OpenShell adds its baseline paths to a policy with a network rule unless the policy lists them: read-only {}, \
             read-write {} (where they exist) — run a boundary check on the EFFECTIVE policy (`openshell policy get --base`)",
            added_ro.join(", "),
            added_rw.join(", ")
        ));
    }
    if !read_write.is_empty() {
        notes.push(
            "OpenShell has no write-only path: `read_write` also lets `delulu` read what it may write; the program's \
             grant still refuses it a read there"
                .into(),
        );
    }

    let rules: Vec<(String, String)> = hosts
        .iter()
        .enumerate()
        .map(|(i, (h, _))| (format!("delulu_https_{}", i + 1), h.clone()))
        .collect();
    let endpoint = |h: &str| {
        json!({
            "host": h, "port": PORT, "protocol": "rest", "enforcement": "enforce",
            "rules": [
                { "allow": { "method": METHOD, "path": "/" } },
                { "allow": { "method": METHOD, "path": "/**" } },
            ],
        })
    };
    let mut network = serde_json::Map::new();
    for (name, h) in &rules {
        network.insert(name.clone(), json!({ "endpoints": [endpoint(h)], "binaries": [{ "path": opts.binary }] }));
    }
    let policy = json!({
        "version": 1,
        "filesystem_policy": { "include_workdir": false, "read_only": read_only, "read_write": read_write },
        "landlock": { "compatibility": "hard_requirement" },
        "process": { "run_as_user": opts.run_as_user, "run_as_group": opts.run_as_group },
        "network_policies": Value::Object(network),
    });

    let mut y = String::new();
    y.push_str("# An OpenShell sandbox policy for `delulu run`, derived by `delulu sandbox policy --format openshell`\n");
    y.push_str("# from the program's authority and the grants below. It allows nothing the program's authority and\n");
    y.push_str("# its grants do not; `delulu sandbox policy --format openshell --json` accounts for every grant.\n");
    y.push_str(&format!("# program: {}\n", quote(&program)));
    for u in &unrepresented {
        y.push_str(&format!("# not represented: {} — {}\n", quote(u["what"].as_str().unwrap_or("")), u["why"].as_str().unwrap_or("")));
    }
    for o in &omitted {
        y.push_str(&format!("# omitted: {} — {}\n", quote(o["grant"].as_str().unwrap_or("")), o["why"].as_str().unwrap_or("")));
    }
    for n in &narrowed {
        y.push_str(&format!("# narrowed: {} — {}\n", quote(n["grant"].as_str().unwrap_or("")), n["why"].as_str().unwrap_or("")));
    }
    for n in &notes {
        y.push_str(&format!("# note: {n}\n"));
    }
    // The document is written FROM `policy`, so what the YAML states and what `--json` reports cannot
    // differ (a mutant adding a method to the text alone passed the unit gate while they were two paths).
    y.push_str(&yaml_of(&policy));

    let payload = json!({
        "format": "openshell",
        "program": program,
        "binary": opts.binary,
        "runtime_paths": RUNTIME_PATHS,
        "policy": policy,
        "document": y,
        "emitted": emitted,
        "omitted": omitted,
        "unrepresented": unrepresented,
        "narrowed": narrowed,
        "notes": notes,
    });
    Ok(Export { document: y, payload })
}

/// A YAML double-quoted scalar: every string the document carries goes through here, so a path or a
/// host can never close its quote, start a new key, or break a line.
fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c.is_control() || matches!(c, '\u{85}' | '\u{2028}' | '\u{2029}' | '\u{feff}') => {
                out.push_str(&format!("\\u{:04x}", c as u32))
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The policy as OpenShell's YAML, in the schema's order. Every string is double-quoted through
/// [`quote`]; only keys (fixed here), integers and booleans are bare.
fn yaml_of(policy: &Value) -> String {
    let s = |v: &Value| quote(v.as_str().unwrap_or_default());
    let strings = |v: &Value| -> Vec<String> { v.as_array().map(|a| a.iter().map(s).collect()).unwrap_or_default() };
    let mut y = String::new();
    y.push_str(&format!("version: {}\n", policy["version"]));
    let fs = &policy["filesystem_policy"];
    y.push_str(&format!("filesystem_policy:\n  include_workdir: {}\n", fs["include_workdir"]));
    for key in ["read_only", "read_write"] {
        let items = strings(&fs[key]);
        if items.is_empty() {
            y.push_str(&format!("  {key}: []\n"));
        } else {
            y.push_str(&format!("  {key}:\n"));
            for i in items {
                y.push_str(&format!("    - {i}\n"));
            }
        }
    }
    y.push_str(&format!("landlock:\n  compatibility: {}\n", s(&policy["landlock"]["compatibility"])));
    let pr = &policy["process"];
    y.push_str(&format!("process:\n  run_as_user: {}\n  run_as_group: {}\n", s(&pr["run_as_user"]), s(&pr["run_as_group"])));
    let net = policy["network_policies"].as_object().cloned().unwrap_or_default();
    if net.is_empty() {
        y.push_str("network_policies: {}\n");
        return y;
    }
    y.push_str("network_policies:\n");
    for (name, rule) in &net {
        y.push_str(&format!("  {}:\n    endpoints:\n", quote(name)));
        for ep in rule["endpoints"].as_array().into_iter().flatten() {
            y.push_str(&format!("      - host: {}\n        port: {}\n", s(&ep["host"]), ep["port"]));
            y.push_str(&format!("        protocol: {}\n        enforcement: {}\n        rules:\n", s(&ep["protocol"]), s(&ep["enforcement"])));
            for r in ep["rules"].as_array().into_iter().flatten() {
                y.push_str(&format!(
                    "          - allow:\n              method: {}\n              path: {}\n",
                    s(&r["allow"]["method"]),
                    s(&r["allow"]["path"])
                ));
            }
        }
        y.push_str("    binaries:\n");
        for b in rule["binaries"].as_array().into_iter().flatten() {
            y.push_str(&format!("      - path: {}\n", s(&b["path"])));
        }
    }
    y
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(workdir: Option<&str>) -> Options {
        Options {
            workdir: workdir.map(str::to_string),
            binary: DEFAULT_BINARY.into(),
            run_as_user: DEFAULT_RUN_AS.into(),
            run_as_group: DEFAULT_RUN_AS.into(),
        }
    }

    /// The soundness rule over every shipped example (study §4.5's in-tree witness): with the grants its
    /// own report requires (a placeholder filled with a value of its shape) and one grant of every kind it
    /// does NOT use, each path on the wall is a runtime path, the program's file, or a grant the program's
    /// authority can use; each endpoint is such a grant's host; each rule is `GET`; and every grant given
    /// is accounted for exactly once — emitted, omitted, unrepresented, or the export refused.
    #[test]
    fn every_example_exports_nothing_its_authority_and_grants_do_not_allow() {
        let limits = crate::policy::Profile::Contained.limits();
        let mut exported = 0;
        for (path, src) in crate::examples::EMBEDDED {
            let report = crate::cli::authority_of_text(path, src).expect("a shipped example checks");
            let mut grants: Vec<String> = report["required_grants"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|g| g.as_str())
                .map(|g| g.replace("=PATH", "=./filled").replace("=HOST", "=filled.example.com").replace("=VALUE", "=v"))
                .collect();
            // Kinds the example may not use: each must come back omitted when it does not.
            grants.extend(["fs.read=./extra-read", "fs.write=./extra-write", "net=unused.example.com"].map(String::from));
            let kinds: Vec<String> = report["capabilities"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|c| c["kind"].as_str().map(str::to_string))
                .collect();
            let e = match export(path, &report, &grants, &limits, &opts(Some("/sandbox"))) {
                Ok(e) => e,
                Err(Refused::Unrepresentable(why)) => {
                    // Refused by name: only for a grant this export has no mapping for.
                    assert!(
                        grants.iter().any(|g| ["foreign.", "plugin", "exec.native", "actuator", "sensor", "compute", "net.special"]
                            .iter()
                            .any(|k| g.starts_with(k))),
                        "{path}: refused without a grant it cannot map: {why}"
                    );
                    continue;
                }
                Err(Refused::Usage(why)) => panic!("{path}: {why}"),
            };
            exported += 1;
            let p = &e.payload["policy"];
            let allowed_path = |x: &str| {
                RUNTIME_PATHS.contains(&x)
                    || x == format!("/sandbox/{path}")
                    || e.payload["emitted"].as_array().unwrap().iter().any(|m| m["path"] == x)
            };
            for x in p["filesystem_policy"]["read_only"].as_array().unwrap() {
                assert!(allowed_path(x.as_str().unwrap()), "{path}: `{x}` on the wall traces to nothing");
            }
            for x in p["filesystem_policy"]["read_write"].as_array().unwrap() {
                let x = x.as_str().unwrap();
                assert!(
                    e.payload["emitted"].as_array().unwrap().iter().any(|m| m["path"] == x && m["as"] == "read_write"),
                    "{path}: `{x}` is writable and no write grant names it"
                );
            }
            for m in e.payload["emitted"].as_array().unwrap() {
                let g = m["grant"].as_str().unwrap();
                let kind = match m["as"].as_str().unwrap() {
                    "read_only" => "FsRead",
                    "read_write" => "FsWrite",
                    _ => "Http",
                };
                assert!(kinds.iter().any(|k| k == kind), "{path}: `{g}` emitted for a kind ({kind}) the program never uses");
            }
            for (_, rule) in p["network_policies"].as_object().unwrap() {
                assert_eq!(rule["binaries"], json!([{ "path": DEFAULT_BINARY }]), "{path}: one binary, delulu");
                for ep in rule["endpoints"].as_array().unwrap() {
                    assert!(ep.get("access").is_none(), "{path}: a preset is wider than GET: {ep}");
                    assert_eq!(ep["enforcement"], "enforce", "{path}: {ep}");
                    for r in ep["rules"].as_array().unwrap() {
                        assert_eq!(r["allow"]["method"], METHOD, "{path}: {ep}");
                    }
                    let h = ep["host"].as_str().unwrap();
                    assert!(
                        e.payload["emitted"].as_array().unwrap().iter().any(|m| m["host"] == h),
                        "{path}: endpoint `{h}` traces to no grant"
                    );
                }
            }
            // Every grant accounted for exactly once.
            let accounted = |g: &str| {
                let key = g.split('=').next().unwrap_or(g);
                ["emitted", "omitted"]
                    .iter()
                    .map(|k| e.payload[*k].as_array().unwrap().iter().filter(|m| m["grant"] == g).count())
                    .sum::<usize>()
                    + e.payload["unrepresented"].as_array().unwrap().iter().filter(|m| m["what"] == key).count()
            };
            for g in &grants {
                let n = accounted(g);
                assert!(n >= 1, "{path}: `{g}` was dropped in silence: {:#}", e.payload);
            }
            assert!(
                e.payload["omitted"].as_array().unwrap().iter().all(|m| !e.payload["emitted"].as_array().unwrap().iter().any(|x| x["grant"] == m["grant"])),
                "{path}: a grant both emitted and omitted"
            );
        }
        assert!(exported >= 6, "the gate must read real exports, not refusals: {exported}");
    }

    #[test]
    fn paths_are_absolute_and_normal_or_refused() {
        assert_eq!(absolute("./out/./x", Some("/w"), "p").unwrap(), "/w/out/x");
        assert_eq!(absolute("/a//b/", None, "p").unwrap(), "/a/b");
        assert!(matches!(absolute("data", None, "p"), Err(Refused::Usage(_))));
        assert!(matches!(absolute("./data/../out", Some("/w"), "p"), Err(Refused::Usage(_))));
        assert!(matches!(absolute("/../etc", None, "p"), Err(Refused::Usage(_))));
        assert!(matches!(absolute("/a\nb", None, "p"), Err(Refused::Usage(_))));
        assert!(matches!(absolute("C:\\x\\p.delulu", None, "p"), Err(Refused::Usage(_))));
        assert!(matches!(absolute(".\\data", Some("/w"), "p"), Err(Refused::Usage(_))));
        assert!(matches!(absolute("c:/x", Some("/w"), "p"), Err(Refused::Usage(_))));
    }

    #[test]
    fn hosts_map_to_openshell_patterns_or_refuse() {
        assert_eq!(endpoint_host("g", "API.Example.com").unwrap(), "api.example.com");
        assert_eq!(endpoint_host("g", "*.cdn.example.org").unwrap(), "**.cdn.example.org");
        assert!(matches!(endpoint_host("g", "*.com"), Err(Refused::Unrepresentable(_))));
        assert!(matches!(endpoint_host("g", "a.*.com"), Err(Refused::Unrepresentable(_))));
    }

    #[test]
    fn a_quoted_scalar_cannot_break_out_of_its_line() {
        assert_eq!(quote("a\"b\\c\nd"), "\"a\\\"b\\\\c\\u000ad\"");
        assert_eq!(quote("x\u{2028}y"), "\"x\\u2028y\"");
    }

    #[test]
    fn run_as_refuses_root_and_names() {
        assert_eq!(run_as("1500:1500").unwrap(), ("1500".into(), "1500".into()));
        assert_eq!(run_as("sandbox").unwrap(), ("sandbox".into(), "sandbox".into()));
        for bad in ["0", "0:0", "root", "01500", "4294967295", "-1"] {
            assert!(run_as(bad).is_err(), "{bad}");
        }
    }
}
