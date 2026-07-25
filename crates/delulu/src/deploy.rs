//! `delulu deploy plan` — the whole-deployment authority answer, computed and checked BEFORE
//! anything runs (Stage 10, phase 10j, Track H; spec §9.2, invariant 53: "no plan, no launch").
//!
//! A deployment is a set of named services, each backed by a package directory, checked against
//! an ENVIRONMENT PROFILE — a file in the exact same `[authority]\neffects = [...]` shape a
//! package's own `delulu.toml` uses, parsed with the SAME `delulu_runtime::parse_manifest` (this
//! is deliberate reuse, not a coincidence: an environment profile is an authority manifest for a
//! place a program runs, not a new format that needs its own parser). The profile's `effects`
//! list is the CEILING — the maximum authority any service in that environment may hold.
//!
//! `delulu deploy plan` computes each named service's checked authority the same way `delulu
//! authority` computes it for one package, and refuses the WHOLE plan — never partially — if any
//! service's effects go beyond the ceiling (**DL1909**). "What can this deployment do to my cloud
//! account?" gets the same mechanical answer as "what can this function do?" (spec §9.2).
//!
//! Honesty note, stated once here rather than left implicit: this checks EFFECTS only. Spec §9.2
//! describes the full authority answer as effects, capability scopes, and foreign holes, per
//! service — capability SCOPES and foreign holes are not compared against the profile by this
//! command. Recorded as a gap, not implied as covered.

use delulu_diag::{render_human, Diagnostic, SourceMap};
use delulu_runtime::parse_manifest;
use serde_json::{json, Value};

/// Dispatches `delulu deploy <subcommand>`. Only `plan` exists today.
pub fn cmd_deploy(rest: &[String], authority_of: impl Fn(&str) -> Option<Value>) -> i32 {
    let Some(sub) = rest.first().map(String::as_str) else {
        eprintln!(
            "error: `deploy` needs a subcommand: plan --service NAME=PACKAGE_DIR [--service ...] \
             --env ENVFILE.toml [--json]"
        );
        return 2;
    };
    match sub {
        "plan" => cmd_deploy_plan(&rest[1..], authority_of),
        other => {
            eprintln!("error: unknown `deploy` subcommand `{other}` (only `plan` exists)");
            2
        }
    }
}

/// `delulu deploy plan --service NAME=PACKAGE_DIR [--service ...] --env ENVFILE.toml [--json]`
/// (spec §9.2). Computes every named service's checked authority, then refuses the whole plan
/// (DL1909) if any service's effects exceed the environment profile's ceiling.
fn cmd_deploy_plan(rest: &[String], authority_of: impl Fn(&str) -> Option<Value>) -> i32 {
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

    // Compute every service's checked authority FIRST. A service that does not check cleanly is a
    // DIFFERENT failure from "checks fine but exceeds the profile" (DL1909 is only the second
    // kind) — and without a clean check there is no honest authority to compare, so this stops the
    // whole plan immediately rather than guessing what an unchecked package might do.
    let mut reports: Vec<(String, Vec<String>)> = Vec::new();
    for (name, dir) in &services {
        let Some(report) = authority_of(dir) else {
            return report_check_failure(&env_path, name, dir, json);
        };
        let effects: Vec<String> = report
            .get("effects")
            .and_then(Value::as_array)
            .map(|xs| xs.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        reports.push((name.clone(), effects));
    }

    // The ceiling check runs across EVERY service before any refusal is issued — the whole plan
    // is refused together, never partially, so "approved" never means anything less than "all of
    // it fits".
    let violations: Vec<(String, Vec<String>)> = reports
        .iter()
        .filter_map(|(name, effects)| {
            let exceeding = exceeding_effects(effects, ceiling);
            if exceeding.is_empty() { None } else { Some((name.clone(), exceeding)) }
        })
        .collect();

    if !violations.is_empty() {
        return report_ceiling_violation(&env_path, &violations, ceiling, json);
    }

    if json {
        let services_json: Vec<Value> =
            reports.iter().map(|(name, effects)| json!({ "name": name, "effects": effects })).collect();
        crate::cli::note_json_emitted();
        println!(
            "{}",
            json!({
                "command": "deploy", "subcommand": "plan", "env": env_path, "approved": true,
                "services": services_json,
            })
        );
    } else {
        for (name, effects) in &reports {
            println!("  {name}: {}", if effects.is_empty() { "pure".to_string() } else { format!("effects {effects:?}") });
        }
        println!("deploy plan: approved — {} service(s) within {env_path}'s authority ceiling", reports.len());
    }
    0
}

/// A service that failed its OWN check — not a ceiling violation. There is no honest authority to
/// compare without a clean check, so this is a plain error (never DL1909), naming the service.
fn report_check_failure(env_path: &str, name: &str, dir: &str, json: bool) -> i32 {
    let message =
        format!("service `{name}` ({dir}) does not check cleanly — fix its errors before it can be part of a deployment plan");
    if json {
        crate::cli::note_json_emitted();
        println!(
            "{}",
            json!({ "command": "deploy", "subcommand": "plan", "env": env_path, "approved": false, "error": message })
        );
    } else {
        eprintln!("error: {message}");
    }
    1
}

/// The DL1909 ceiling refusal. A PLAIN `Diagnostic::error` (DL1905's real precedent in `cli.rs`)
/// — deliberately not built with a `Repair`: `Repair` carries byte-offset edits into `.delulu`
/// SOURCE files, and there is no source span that would widen a TOML environment profile. The
/// "a human must decide" idea lives in the message's prose instead, exactly as DL1905 states its
/// own human-decision requirement in prose rather than as a `requires_human` repair flag.
fn report_ceiling_violation(env_path: &str, violations: &[(String, Vec<String>)], ceiling: &[String], json: bool) -> i32 {
    let detail = violations
        .iter()
        .map(|(name, exceeding)| {
            format!("service `{name}` requires {} that the environment profile does not grant", backticked(exceeding))
        })
        .collect::<Vec<_>>()
        .join("; ");
    let ceiling_str = if ceiling.is_empty() { "no effects at all".to_string() } else { backticked(ceiling) };
    let message = format!(
        "deploy plan for `{env_path}` refused: {detail} — the profile's authority ceiling is \
         {ceiling_str}, and a plan that would exceed it is refused as a WHOLE, never partially \
         approved [a human must decide whether to widen the environment profile; see `delulu \
         explain DL1909` and spec §9.2]"
    );
    let d = Diagnostic::error("DL1909", message);
    if json {
        crate::cli::note_json_emitted();
        println!(
            "{}",
            json!({
                "command": "deploy", "subcommand": "plan", "env": env_path, "approved": false,
                "code": d.code, "error": d.message,
            })
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
