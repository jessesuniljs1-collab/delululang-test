//! P4-02: `delulu toolchain --json` — what this toolchain IS, as data.
//!
//! The zero-shot loop (`V2_AI_NATIVE_DESIGN.md` §2) starts with a model that has never seen the
//! language asking the installed binary what it can do. `--help` answers a person; this answers a
//! program: every command with the invocations and options its help documents, the ten core effects,
//! the grant grammar with an example of each form that parses, the primitive table, the budgets and
//! sandbox profiles, the isolation levels, the channel version, and every diagnostic code and
//! explanation topic.
//!
//! **Nothing here is written twice.** Each field is read from the table the binary itself uses —
//! `cli::SUBCOMMANDS` and the usage text the help is cut from, `check::CORE_EFFECT_NAMES`,
//! `broker::GRANT_FORMS` (bound to the grant parser by a test that reads its match arms),
//! `prim_table::PRIM_TABLE` (bound to the checker), the budget defaults, `policy::Profile::ALL`,
//! `sandbox::LEVELS` (the probe's own names), `delulu_diag::REGISTRY` and `delulu_diag::TOPICS`. A description
//! that is a copy is a description that drifts; this one can only change when the thing it describes
//! does.

use serde_json::{json, Value};

pub fn cmd_toolchain(rest: &[String]) -> i32 {
    if let Some(bad) = rest.iter().find(|a| a.starts_with('-') && a.as_str() != "--json") {
        eprintln!("error: `toolchain` does not know the option `{bad}`");
        eprintln!("  nothing was done — an option nobody understood is refused, never ignored");
        return 2;
    }
    if let Some(extra) = rest.iter().find(|a| !a.starts_with('-')) {
        eprintln!("error: `toolchain` takes no arguments (got `{extra}`)");
        return 2;
    }
    let doc = describe();
    if rest.iter().any(|a| a == "--json") {
        crate::cli::print_success_envelope("toolchain", json!({ "toolchain": doc }));
    } else {
        print!("{}", summary(&doc));
    }
    0
}

/// The whole description. Public to the crate so a test can hold it against `--help` and the parsers.
pub(crate) fn describe() -> Value {
    let commands: Vec<Value> = crate::cli::SUBCOMMANDS
        .iter()
        .map(|name| {
            let flags = crate::cli::documented_flags(name);
            let usage: Vec<String> =
                crate::cli::usage_lines(name).iter().map(|l| l.trim().to_string()).collect();
            json!({
                "name": name,
                "json": flags.iter().any(|f| f == "--json"),
                "flags": flags,
                "usage": usage,
            })
        })
        .collect();
    let grants: Vec<Value> = delulu_runtime::broker::GRANT_FORMS
        .iter()
        .map(|g| json!({ "key": g.key, "form": g.form, "example": g.example, "grants": g.grants }))
        .collect();
    let primitives: Vec<Value> = delulu_check::prim_table::PRIM_TABLE
        .iter()
        .map(|p| json!({ "receiver": p.receiver, "method": p.method, "arity": p.arity }))
        .collect();
    let profiles: Vec<Value> = crate::policy::Profile::ALL
        .iter()
        .map(|p| {
            let l = p.limits();
            json!({ "name": p.name(), "memory_bytes": l.memory_bytes, "cpu_seconds": l.cpu_seconds })
        })
        .collect();
    let levels: Vec<Value> =
        crate::sandbox::LEVELS.iter().map(|(n, name)| json!({ "level": n, "name": name })).collect();
    let codes: Vec<Value> =
        delulu_diag::REGISTRY.iter().map(|c| json!({ "code": c.code, "title": c.title })).collect();
    let topics: Vec<String> = delulu_diag::TOPICS.iter().map(|t| format!("E-{t}")).collect();
    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "commands": commands,
        "effects": delulu_check::check::CORE_EFFECT_NAMES,
        "grants": grants,
        "primitives": { "table_version": delulu_check::dir::PRIM_TABLE_VERSION, "entries": primitives },
        "budgets": {
            // Every ordinary run, unless `--limits` or a lease says otherwise (D-V2-25).
            "run_default": {
                "memory_bytes": crate::budget::DEFAULT_MEMORY_BYTES,
                "cpu_seconds": crate::budget::DEFAULT_CPU_SECONDS,
            },
            "sandbox_profiles": profiles,
        },
        // `run` runs `main` on the interpreter unless `--engine wasm` says otherwise.
        "engines": ["interpreter", "wasm"],
        "sandbox": {
            "levels": levels,
            "modes": [crate::policy::Mode::Strict.name(), crate::policy::Mode::Audit.name()],
            "channel": delulu_runtime::channel::CHANNEL_VERSION,
        },
        "diagnostics": { "codes": codes, "topics": topics },
    })
}

/// The human form: counts and names, and where the full description is.
fn summary(doc: &Value) -> String {
    let n = |k: &str| doc[k].as_array().map_or(0, Vec::len);
    let names = |v: &Value, field: &str| -> String {
        v.as_array().map(|a| a.iter().filter_map(|x| x[field].as_str()).collect::<Vec<_>>().join(", ")).unwrap_or_default()
    };
    let effects: Vec<&str> = doc["effects"].as_array().map(|a| a.iter().filter_map(|e| e.as_str()).collect()).unwrap_or_default();
    format!(
        "delulu {} — this toolchain, as data: `delulu toolchain --json`\n\
         \x20 {:<13}{} (each with the invocations and options its `--help` documents)\n\
         \x20 {:<13}{}\n\
         \x20 {:<13}{} forms, each with an example that parses\n\
         \x20 {:<13}{} (table v{})\n\
         \x20 {:<13}{} codes, {} explanation topics\n\
         \x20 {:<13}levels {}; profiles {}; channel {}\n",
        doc["version"].as_str().unwrap_or(""),
        "commands",
        n("commands"),
        "effects",
        effects.join(", "),
        "grants",
        n("grants"),
        "primitives",
        doc["primitives"]["entries"].as_array().map_or(0, Vec::len),
        doc["primitives"]["table_version"],
        "diagnostics",
        doc["diagnostics"]["codes"].as_array().map_or(0, Vec::len),
        doc["diagnostics"]["topics"].as_array().map_or(0, Vec::len),
        "sandbox",
        names(&doc["sandbox"]["levels"], "name"),
        names(&doc["budgets"]["sandbox_profiles"], "name"),
        doc["sandbox"]["channel"].as_str().unwrap_or(""),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The description agrees with what it describes: every command the dispatcher knows, with the
    /// options its help documents; the grant forms parse; each explanation topic resolves; the levels
    /// are the probe's; the engines are the ones `run --help` names.
    #[test]
    fn the_description_is_read_from_the_binarys_own_tables() {
        let d = describe();
        let cmds = d["commands"].as_array().unwrap();
        assert_eq!(cmds.len(), crate::cli::SUBCOMMANDS.len());
        for c in cmds {
            let name = c["name"].as_str().unwrap();
            assert!(!c["usage"].as_array().unwrap().is_empty(), "`{name}` has no documented invocation");
            let flags: Vec<&str> = c["flags"].as_array().unwrap().iter().map(|f| f.as_str().unwrap()).collect();
            assert_eq!(flags, crate::cli::documented_flags(name), "`{name}`'s flags are its help's");
        }
        let check = cmds.iter().find(|c| c["name"] == "check").unwrap();
        assert_eq!(check["json"], true, "`check --json` is documented");
        assert_eq!(d["effects"].as_array().unwrap().len(), 10, "the ten core effects");
        for g in d["grants"].as_array().unwrap() {
            let mut grants = delulu_runtime::broker::Grants::default();
            assert!(grants.add(g["example"].as_str().unwrap()).is_ok(), "{g}");
        }
        for t in d["diagnostics"]["topics"].as_array().unwrap() {
            let t = t.as_str().unwrap().strip_prefix("E-").unwrap();
            assert!(delulu_diag::topic_explain(t).is_some(), "E-{t}");
        }
        let levels: Vec<&str> =
            d["sandbox"]["levels"].as_array().unwrap().iter().map(|l| l["name"].as_str().unwrap()).collect();
        assert_eq!(levels, ["none", "process", "microvm", "external", "attested"]);
        let run_usage = crate::cli::usage_lines("run").join("\n");
        assert!(run_usage.contains("--engine wasm"), "the engines listed are the ones `run` documents");
        assert!(summary(&d).contains("toolchain --json"));
    }
}
