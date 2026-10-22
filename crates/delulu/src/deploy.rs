//! `delulu deploy plan` — the whole-deployment authority answer, computed and checked BEFORE
//! anything runs (Stage 10, phase 10j, Track H; spec §9.2, invariant 53: "no plan, no launch").
//!
//! A deployment is a set of named services, each backed by a package directory, checked against
//! an ENVIRONMENT PROFILE — a file in the exact same `[authority]` shape a package's own
//! `delulu.toml` uses, parsed with the SAME `delulu_runtime::parse_manifest` (this is deliberate
//! reuse, not a coincidence: an environment profile is an authority manifest for a place a program
//! runs, not a new format that needs its own parser). The profile is the CEILING — the most any
//! service in that environment may hold.
//!
//! `delulu deploy plan` checks each named service as `delulu authority <dir>` checks it (the
//! workspace loader for a package with a `delulu.toml`, so its dependencies are seen — DEPLOY-PKG-1),
//! and refuses the WHOLE plan — never partially — if any service goes beyond the ceiling
//! (**DL1909**), in either of the two halves of the answer spec §9.2 asks for:
//!
//! - **effects** — the service's effect row against the profile's `effects`;
//! - **capability scopes and foreign holes** (RW 6.6, 2026-10-22) — every request the service makes,
//!   in the `--grant` grammar, decided against the profile's boundary by `delulu authority --within`'s
//!   own verdict (`within.rs`): the runtime's mint for each path, host and secret, the boundary's
//!   `foreign.c` and `foreign.python`. The boundary is the profile's `fs.read`, `fs.write`, `net`,
//!   `secrets`, `foreign.c` and `foreign.python`, with `console`, `clock`, `rand` and `declassify`
//!   where its effects grant `Write`, `Clock`, `Rand` and `Declassify`. A request the source does not
//!   show (a computed path) is *inconclusive* unless the profile grants none of its kind, and an
//!   inconclusive plan is refused like an exceeding one: no plan, no launch.
//!
//! "What can this deployment do to my cloud account?" gets the same mechanical answer as "what can
//! this function do?" (spec §9.2). A profile has no spelling for `net.special`, plugin roots or
//! devices, so a service that needs one is refused, by name.

use delulu_diag::{render_human, Diagnostic, SourceMap};
use delulu_runtime::{parse_manifest, Grants, Manifest};
use serde_json::{json, Value};

/// The authority dimensions `deploy plan` compares against the environment profile, and what a profile cannot grant.
/// Named in the approval itself, not only in this module's header: a verdict that travels without its scope is read as
/// covering everything (`HARDENING_CAMPAIGN.md` C45).
const COMPARED_DIMENSIONS: &[&str] = &["effects", "capability scopes (fs.read, fs.write, net, secrets)", "foreign holes"];
const PROFILE_CANNOT_GRANT: &[&str] = &["net.special", "plugin roots", "devices (actuator, sensor, compute)"];

/// The environment profile as a boundary in the `--grant` grammar — the form `delulu authority --within` decides against —
/// and the lines it was built from. Paths, hosts and Python patterns go through `Grants::add`, the parser `run --grant` uses,
/// so a profile is read as a run's grants would be (a special-use host under `net` is refused, D-NE-28). Secrets and
/// `foreign.c` libraries are matched by name; the profile says which may be held, never with what value or binary.
fn profile_boundary(profile: &Manifest) -> Result<(Grants, Vec<String>), String> {
    let mut boundary = Grants::default();
    let mut lines: Vec<String> = Vec::new();
    for (effect, grant) in [("Write", "console"), ("Clock", "clock"), ("Rand", "rand"), ("Declassify", "declassify")] {
        if profile.effects.iter().any(|e| e == effect) {
            lines.push(grant.to_string());
        }
    }
    lines.extend(profile.fs_read.iter().map(|p| format!("fs.read={p}")));
    lines.extend(profile.fs_write.iter().map(|p| format!("fs.write={p}")));
    lines.extend(profile.net.iter().map(|h| format!("net={h}")));
    lines.extend(profile.foreign_python.iter().map(|p| format!("foreign.python={p}")));
    for line in &lines {
        boundary.add(line).map_err(|e| format!("`{line}` is not a boundary this profile can give: {e}"))?;
    }
    for name in &profile.secrets {
        boundary.secrets.insert(name.clone(), String::new());
        lines.push(format!("secret:{name}"));
    }
    for lib in &profile.foreign_c {
        boundary.foreign_c.insert(lib.clone(), String::new());
        lines.push(format!("foreign.c={lib}"));
    }
    Ok((boundary, lines))
}

/// One service of the plan, as decided: its effects, the effects beyond the ceiling, and its `within` verdict as JSON.
struct Decided {
    name: String,
    effects: Vec<String>,
    exceeding: Vec<String>,
    within: Value,
}

impl Decided {
    fn fits(&self) -> bool {
        self.exceeding.is_empty() && self.within["result"] == "within"
    }
}

/// Dispatches `delulu deploy <subcommand>`. Only `plan` exists today.
pub fn cmd_deploy(rest: &[String], checked_package: impl Fn(&str) -> Option<crate::cli::CheckedPackage>) -> i32 {
    let Some(sub) = rest.first().map(String::as_str) else {
        eprintln!(
            "error: `deploy` needs a subcommand: plan --service NAME=PACKAGE_DIR [--service ...] \
             --env ENVFILE.toml [--json]"
        );
        return 2;
    };
    match sub {
        "plan" => cmd_deploy_plan(&rest[1..], checked_package),
        other => {
            eprintln!("error: unknown `deploy` subcommand `{other}` (only `plan` exists)");
            2
        }
    }
}

/// `delulu deploy plan --service NAME=PACKAGE_DIR [--service ...] --env ENVFILE.toml [--json]`
/// (spec §9.2). Checks every named service, then refuses the whole plan (DL1909) if any service's effects exceed the
/// environment profile's ceiling or any of its requests falls outside the profile's boundary.
fn cmd_deploy_plan(rest: &[String], checked_package: impl Fn(&str) -> Option<crate::cli::CheckedPackage>) -> i32 {
    let json = rest.iter().any(|a| a == "--json");

    let raw_services = flags_all(rest, "--service");
    if raw_services.is_empty() {
        eprintln!("error: `deploy plan` needs at least one --service NAME=PACKAGE_DIR");
        return 2;
    }
    let Some(env_path) = flag(rest, "--env") else {
        eprintln!("error: `deploy plan` needs --env ENVFILE.toml");
        return 2;
    };

    let mut services: Vec<(String, String)> = Vec::new();
    for raw in &raw_services {
        let Some((name, dir)) = raw.split_once('=') else {
            eprintln!("error: `--service {raw}` is not NAME=PACKAGE_DIR (missing `=`)");
            return 2;
        };
        services.push((name.trim().to_string(), dir.trim().to_string()));
    }

    let env_src = match std::fs::read_to_string(&env_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read environment profile {env_path}: {e}");
            return 2;
        }
    };
    let profile = parse_manifest(&env_src);
    let ceiling = &profile.effects;
    let (boundary, boundary_lines) = match profile_boundary(&profile) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: environment profile {env_path}: {e}");
            return 2;
        }
    };

    // Check every service FIRST. A service that does not check cleanly is a DIFFERENT failure from "checks fine but
    // exceeds the profile" (DL1909 is only the second kind) — and without a clean check there is no honest authority to
    // compare, so this stops the whole plan immediately rather than guessing what an unchecked package might do.
    let mut decided: Vec<Decided> = Vec::new();
    for (name, dir) in &services {
        let Some(pkg) = checked_package(dir) else {
            return report_check_failure(&env_path, name, dir, json);
        };
        let effects: Vec<String> = pkg
            .report
            .get("effects")
            .and_then(Value::as_array)
            .map(|xs| xs.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        let verdict = crate::within::verdict(&pkg.requirements, &pkg.unsupported, &boundary);
        decided.push(Decided {
            name: name.clone(),
            exceeding: exceeding_effects(&effects, ceiling),
            effects,
            within: crate::within::verdict_json(&verdict, &boundary_lines, &pkg.map),
        });
    }

    // The ceiling is checked across EVERY service before any refusal is issued — the whole plan is refused together,
    // never partially, so "approved" never means anything less than "all of it fits".
    if decided.iter().any(|d| !d.fits()) {
        return report_ceiling_violation(&env_path, &decided, ceiling, json);
    }

    if json {
        let services_json: Vec<Value> =
            decided.iter().map(|d| json!({ "name": d.name, "effects": d.effects, "within": d.within })).collect();
        crate::cli::print_success_envelope(
            "deploy",
            json!({
                "subcommand": "plan", "env": env_path, "approved": true,
                "services": services_json,
                // What the approval rests on, carried in the approval (C45): a caller reading `"approved": true` must
                // be able to see which dimensions it covers, and what the profile could not have granted.
                "compared": COMPARED_DIMENSIONS, "profile_cannot_grant": PROFILE_CANNOT_GRANT,
                "boundary": boundary_lines,
            }),
        );
    } else {
        for d in &decided {
            println!(
                "  {}: {}",
                d.name,
                if d.effects.is_empty() { "pure".to_string() } else { format!("effects {:?}", d.effects) }
            );
        }
        // The scope goes ABOVE the verdict, deliberately: `deploy_plan_adversarial.rs` contracts that the last non-empty
        // line is the verdict, which is what a human scanning a CI log and a script reading the tail both rely on.
        println!(
            "  compared: {}; a profile cannot grant: {} (spec §9.2; see `delulu explain DL1909`)",
            COMPARED_DIMENSIONS.join(", "),
            PROFILE_CANNOT_GRANT.join(", "),
        );
        println!("deploy plan: approved — {} service(s) within {env_path}'s authority ceiling", decided.len());
    }
    0
}

/// A service that failed its OWN check — not a ceiling violation. There is no honest authority to
/// compare without a clean check, so this is a plain error (never DL1909), naming the service.
fn report_check_failure(env_path: &str, name: &str, dir: &str, json: bool) -> i32 {
    let message =
        format!("service `{name}` ({dir}) does not check cleanly — fix its errors before it can be part of a deployment plan");
    if json {
        crate::cli::print_success_envelope(
            "deploy",
            json!({
                "subcommand": "plan", "env": env_path, "approved": false, "error": message,
                "summary": { "errors": 1 },
            }),
        );
    } else {
        eprintln!("error: {message}");
    }
    1
}

/// The DL1909 refusal, naming each service that does not fit and why: the effects beyond the ceiling, and each request
/// outside the boundary with the call that makes it (`within`'s findings — the runtime's own refusal for a path, host or
/// secret; what no form bounds; what no static reading decides). A PLAIN `Diagnostic::error` (DL1905's real precedent in
/// `cli.rs`) — deliberately not built with a `Repair`: `Repair` carries byte-offset edits into `.delulu` SOURCE files,
/// and there is no source span that would widen a TOML environment profile. The "a human must decide" idea lives in the
/// message's prose instead, exactly as DL1905 states its own human-decision requirement in prose.
fn report_ceiling_violation(env_path: &str, decided: &[Decided], ceiling: &[String], json: bool) -> i32 {
    let detail = decided
        .iter()
        .filter(|d| !d.fits())
        .map(|d| {
            let mut parts: Vec<String> = Vec::new();
            if !d.exceeding.is_empty() {
                parts.push(format!("requires {} that the environment profile does not grant", backticked(&d.exceeding)));
            }
            for f in d.within["findings"].as_array().into_iter().flatten() {
                let asks = match f["grant"].as_str() {
                    Some(g) if !g.is_empty() => format!("asks for `{g}`"),
                    _ => format!("holds a `{}` capability", f["dimension"].as_str().unwrap_or("?")),
                };
                let at = match (f["at"]["file"].as_str(), f["at"]["line"].as_u64()) {
                    (Some(file), Some(line)) => format!(" at {file}:{line}"),
                    _ => String::new(),
                };
                parts.push(format!(
                    "{asks}{at}, {}: {}",
                    f["result"].as_str().unwrap_or("?"),
                    f["reason"].as_str().unwrap_or("")
                ));
            }
            format!("service `{}` {}", d.name, parts.join("; "))
        })
        .collect::<Vec<_>>()
        .join("; ");
    let ceiling_str = if ceiling.is_empty() { "no effects at all".to_string() } else { backticked(ceiling) };
    let message = format!(
        "deploy plan for `{env_path}` refused: {detail} — the profile's authority ceiling is \
         {ceiling_str} with the scopes it names, and a plan that would exceed it is refused as a WHOLE, never partially \
         approved [a human must decide whether to widen the environment profile; see `delulu explain DL1909` and spec §9.2]"
    );
    let d = Diagnostic::error("DL1909", message);
    if json {
        let services: Vec<Value> = decided
            .iter()
            .map(|d| json!({ "name": d.name, "effects": d.effects, "exceeding_effects": d.exceeding, "within": d.within }))
            .collect();
        crate::cli::print_success_envelope(
            "deploy",
            json!({
                "subcommand": "plan", "env": env_path, "approved": false,
                "code": d.code, "error": d.message, "services": services,
                "summary": { "errors": 1 },
            }),
        );
    } else {
        eprint!("{}", render_human(&d, &SourceMap::new()));
    }
    1
}

// ----- tiny helpers (no new deps) ------------------------------------------

/// The first value of a `--name value` / `--name=value` flag (mirrors `signing.rs`'s `flag`).
fn flag(rest: &[String], name: &str) -> Option<String> {
    flags_all(rest, name).into_iter().next()
}

/// EVERY value of a repeatable `--name value` / `--name=value` flag, in order — `--service` is
/// the one repeatable flag here, like `--grant` repeats elsewhere in this CLI.
fn flags_all(rest: &[String], name: &str) -> Vec<String> {
    let eq = format!("{name}=");
    let mut out = Vec::new();
    let mut it = rest.iter();
    while let Some(a) = it.next() {
        if a == name {
            if let Some(v) = it.next() {
                out.push(v.clone());
            }
            continue;
        }
        if let Some(v) = a.strip_prefix(&eq) {
            out.push(v.to_string());
        }
    }
    out
}

/// The effects present in `service_effects` but absent from `ceiling` — "authority widening
/// beyond the profile" (spec §9.2). Pure and total: no CLI, no filesystem, no JSON, so the rule
/// itself is testable in isolation from the plumbing that calls it.
fn exceeding_effects(service_effects: &[String], ceiling: &[String]) -> Vec<String> {
    service_effects.iter().filter(|e| !ceiling.iter().any(|c| c == *e)).cloned().collect()
}

/// `["Write", "Net"]` → `` `Write`, `Net` `` — the naming style this CLI uses throughout Stage 10
/// for "name what triggered it" refusals.
fn backticked(items: &[String]) -> String {
    items.iter().map(|s| format!("`{s}`")).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn exceeding_effects_finds_only_what_the_ceiling_does_not_grant() {
        let ceiling = v(&["Write", "Net"]);
        assert_eq!(exceeding_effects(&v(&["Write"]), &ceiling), Vec::<String>::new());
        assert_eq!(exceeding_effects(&v(&["Write", "Net"]), &ceiling), Vec::<String>::new());
        assert_eq!(exceeding_effects(&v(&["Write", "FsWrite"]), &ceiling), v(&["FsWrite"]));
    }

    #[test]
    fn exceeding_effects_against_an_empty_ceiling_is_everything() {
        assert_eq!(exceeding_effects(&v(&["Write"]), &[]), v(&["Write"]));
        assert_eq!(exceeding_effects(&[], &[]), Vec::<String>::new());
    }

    /// THE SKIP-BRANCH CASE: a pure service (no effects at all) must never be reported as
    /// exceeding, even against an empty ceiling — the rule is about what a service ADDS, and pure
    /// services add nothing.
    #[test]
    fn a_pure_service_never_exceeds_any_ceiling() {
        assert_eq!(exceeding_effects(&[], &v(&["Write"])), Vec::<String>::new());
        assert_eq!(exceeding_effects(&[], &[]), Vec::<String>::new());
    }

    #[test]
    fn flags_all_collects_every_occurrence_in_order() {
        let rest = v(&["--service", "api=dir1", "--service", "worker=dir2", "--env", "envs/prod.toml"]);
        assert_eq!(flags_all(&rest, "--service"), v(&["api=dir1", "worker=dir2"]));
        assert_eq!(flag(&rest, "--env"), Some("envs/prod.toml".to_string()));
    }

    #[test]
    fn flags_all_supports_the_equals_form_too() {
        let rest = v(&["--service=api=dir1"]);
        assert_eq!(flags_all(&rest, "--service"), v(&["api=dir1"]));
    }

    #[test]
    fn flags_all_is_empty_when_the_flag_never_appears() {
        let rest = v(&["--env", "e.toml"]);
        assert!(flags_all(&rest, "--service").is_empty());
    }

    #[test]
    fn backticked_joins_with_commas() {
        assert_eq!(backticked(&v(&["Net"])), "`Net`");
        assert_eq!(backticked(&v(&["Write", "Net"])), "`Write`, `Net`");
        assert_eq!(backticked(&[]), "");
    }
}
