//! `delulu doctor` — one command that says whether this machine, and this checkout, are healthy.
//!
//! Two sections, and the split matters. **Environment** checks the things that decide whether any
//! `delulu` command will work at all, and runs everywhere. **Repository** checks the DeluluLang
//! source tree against its own map, and runs *only* when doctor is standing in that tree — a
//! contributor's check, not something to pretend at a user who installed the compiler.
//!
//! The repository section regenerates the Survey when it is behind and then verifies the map's
//! integrity, so the whole "is the map current, and is it sound?" loop is one command.
//!
//! # Why it does not write unless something is wrong
//!
//! Regeneration only touches files that actually differ, and each write goes through a temporary
//! file and a rename ([`delulu_survey::sync_outputs`]). In a healthy checkout `delulu doctor`
//! writes nothing at all, so running it is not a change to the repository — which is the property
//! that makes it safe to run from a script, a hook, or several test processes at once. Campaign
//! finding C69 was a short-lived process writing a shared artifact concurrently; the shape is the
//! same here and is designed out rather than hoped away.

use std::path::{Path, PathBuf};

/// What a single check concluded.
#[derive(PartialEq, Clone, Copy)]
enum Status {
    /// Healthy.
    Ok,
    /// Was wrong, and doctor corrected it.
    Fixed,
    /// True and worth a human's eye. Never fails the run.
    Note,
    /// Wrong, and doctor could not or would not fix it. Fails the run.
    Problem,
}

impl Status {
    fn word(self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::Fixed => "fixed",
            Status::Note => "note",
            Status::Problem => "problem",
        }
    }
}

struct Check {
    section: &'static str,
    name: &'static str,
    status: Status,
    detail: String,
}

#[derive(Default)]
struct Report {
    checks: Vec<Check>,
    regenerated: Vec<&'static str>,
    survey: Option<(usize, usize, [usize; 3])>,
}

impl Report {
    fn push(&mut self, section: &'static str, name: &'static str, status: Status, detail: impl Into<String>) {
        self.checks.push(Check { section, name, status, detail: detail.into() });
    }
    fn problems(&self) -> usize {
        self.checks.iter().filter(|c| c.status == Status::Problem).count()
    }
}

pub fn cmd_doctor(args: &[String]) -> i32 {
    let mut json = false;
    let mut check_only = false;
    for a in args {
        match a.as_str() {
            "--json" => json = true,
            // The read-only mode. Used by the CLI contract suite, which runs subcommands from the
            // workspace root in parallel: a doctor that regenerated there would have several
            // processes writing `docs/survey/` while another test reads it.
            "--check" => check_only = true,
            "--help" | "-h" => {
                println!("{}", help());
                return 0;
            }
            other => {
                // Human reason to stderr, exit 2, and nothing on stdout. The single JSON envelope
                // for a usage error is emitted by the outer layer (`cli.rs`), which is why this
                // must not emit one too — doing so produced *two* objects on stdout, the exact
                // shape campaign C2's fix was gated against, caught by `json_contract`'s rule that
                // stdout must parse as exactly one JSON value.
                eprintln!("error: unknown option `{other}`\n\n{}", help());
                return 2;
            }
        }
    }

    let mut r = Report::default();
    environment(&mut r);
    match source_tree() {
        Some(root) => repository(&mut r, &root, check_only),
        None => r.push(
            "repository",
            "delulu source tree",
            Status::Note,
            "not inside the DeluluLang source tree — repository checks skipped",
        ),
    }

    if json {
        println!("{}", envelope(&r));
    } else {
        render(&r);
    }
    if r.problems() > 0 {
        1
    } else {
        0
    }
}

fn help() -> String {
    "delulu doctor — check this machine and this checkout\n\n\
     USAGE:\n  \
       delulu doctor            check, and regenerate the repository map if it is behind\n  \
       delulu doctor --check    report only; never write\n  \
       delulu doctor --json     one JSON envelope\n\n\
     Exit 0 when healthy, 1 when a problem remains, 2 on a bad invocation."
        .to_string()
}

// --- environment ---------------------------------------------------------------------------------

fn environment(r: &mut Report) {
    r.push("environment", "delulu", Status::Ok, format!("version {}", env!("CARGO_PKG_VERSION")));

    // Whether embedded Python was compiled in. This is a build-time fact, so it is knowable
    // exactly — unlike whether an interpreter will initialize, which is only knowable by trying.
    #[cfg(feature = "python")]
    r.push("environment", "embedded Python", Status::Ok, "compiled in; `root.python(...)` is available with a grant");
    #[cfg(not(feature = "python"))]
    r.push(
        "environment",
        "embedded Python",
        Status::Note,
        "not compiled in — `root.python(...)` returns Unavailable (DL1307). Build without `--no-default-features` to include it",
    );

    match state_dir() {
        Some(dir) => {
            let shown = dir.display().to_string();
            if !dir.exists() {
                // Absent is normal before first use, and says nothing is wrong.
                r.push("environment", "state directory", Status::Ok, format!("{shown} (not created yet)"));
            } else if writable(&dir) {
                r.push("environment", "state directory", Status::Ok, shown);
            } else {
                r.push(
                    "environment",
                    "state directory",
                    Status::Problem,
                    format!("{shown} is not writable — keys, credentials and locales cannot be stored"),
                );
            }
        }
        None => r.push(
            "environment",
            "state directory",
            Status::Problem,
            "neither DELULU_HOME nor HOME/USERPROFILE is set, so there is nowhere to keep state",
        ),
    }

    let mode = std::env::var("DELULU_BROKER").unwrap_or_else(|_| "embedded".into());
    match mode.as_str() {
        "embedded" | "daemon" => r.push("environment", "broker mode", Status::Ok, mode),
        other => r.push(
            "environment",
            "broker mode",
            Status::Problem,
            format!("DELULU_BROKER is `{other}`; only `embedded` and `daemon` exist"),
        ),
    }

    audit_chain(r);
}

/// The audit chain is the one piece of durable state a broken machine can silently damage, so it is
/// worth verifying rather than assuming.
fn audit_chain(r: &mut Report) {
    let Some(dir) = state_dir().map(|h| h.join("audit")) else { return };
    if !dir.exists() {
        r.push("environment", "audit chain", Status::Ok, "no audit log yet");
        return;
    }
    match delulu_broker::verify(&dir) {
        Ok(stats) => r.push(
            "environment",
            "audit chain",
            Status::Ok,
            format!("verified — {} record(s) across {} file(s)", stats.records, stats.files),
        ),
        Err(e) => r.push(
            "environment",
            "audit chain",
            Status::Problem,
            format!("{} failed verification: {e}. Do not discard it — `delulu audit verify` reports where", dir.display()),
        ),
    }
}

fn state_dir() -> Option<PathBuf> {
    std::env::var_os("DELULU_HOME").map(PathBuf::from).or_else(|| {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(|h| PathBuf::from(h).join(".delulu"))
    })
}

fn writable(dir: &Path) -> bool {
    let probe = dir.join(format!(".delulu-doctor-{}.tmp", std::process::id()));
    match std::fs::write(&probe, b"") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

// --- repository ----------------------------------------------------------------------------------

/// The DeluluLang source tree, found by walking up from the working directory.
///
/// Identified by what it contains rather than by its name: a workspace manifest beside the crate
/// that owns the diagnostic registry and the crate that builds the map. A directory that merely
/// happens to be called `DeluluLang` is not one.
fn source_tree() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join("Cargo.toml").is_file()
            && dir.join("crates/delulu-diag/src/codes.rs").is_file()
            && dir.join("crates/delulu-survey/Cargo.toml").is_file()
        {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn repository(r: &mut Report, root: &Path, check_only: bool) {
    r.push("repository", "delulu source tree", Status::Ok, root.display().to_string());

    let survey = delulu_survey::Survey::build(root);
    let stale = delulu_survey::stale_outputs(root, &survey);

    if stale.is_empty() {
        r.push(
            "repository",
            "survey freshness",
            Status::Ok,
            format!("the map matches the tree — {} nodes, {} edges", survey.nodes.len(), survey.edges.len()),
        );
    } else if check_only {
        r.push(
            "repository",
            "survey freshness",
            Status::Problem,
            format!("{} behind the tree: {}. Run `delulu doctor` without --check", stale.len(), stale.join(", ")),
        );
    } else {
        match delulu_survey::sync_outputs(root, &survey) {
            Ok(written) => {
                r.regenerated = written.clone();
                r.push(
                    "repository",
                    "survey freshness",
                    Status::Fixed,
                    format!("was behind the tree; regenerated {} — commit it with the change that caused it", written.join(", ")),
                );
            }
            Err(e) => r.push("repository", "survey freshness", Status::Problem, format!("cannot write docs/survey: {e}")),
        }
    }

    integrity(r, &survey);

    let (mut errors, mut warnings, mut notes) = (0, 0, 0);
    for f in &survey.findings {
        match f.severity {
            delulu_survey::Severity::Error => errors += 1,
            delulu_survey::Severity::Warning => warnings += 1,
            delulu_survey::Severity::Note => notes += 1,
        }
    }
    r.survey = Some((survey.nodes.len(), survey.edges.len(), [errors, warnings, notes]));
    // A discrepancy is a report, not a verdict: errors fail the run, the rest are for a reader.
    let status = if errors > 0 { Status::Problem } else if warnings > 0 { Status::Note } else { Status::Ok };
    r.push(
        "repository",
        "discrepancies",
        status,
        format!("{errors} error, {warnings} warning, {notes} note — docs/survey/DISCREPANCIES.md"),
    );
}

/// The map's own standard, checked here for the same reason the test suite checks it: a map that
/// has stopped checking itself is worse than none, because it is still believed.
fn integrity(r: &mut Report, survey: &delulu_survey::Survey) {
    let uncited = survey.edges.iter().filter(|e| e.file.is_empty() || e.line == 0).count();
    r.push(
        "repository",
        "every edge cites a line",
        if uncited == 0 { Status::Ok } else { Status::Problem },
        if uncited == 0 {
            format!("{} edges, all with a file and line", survey.edges.len())
        } else {
            format!("{uncited} edge(s) carry no citation")
        },
    );

    let dangling =
        survey.edges.iter().flat_map(|e| [&e.from, &e.to]).filter(|id| survey.node(id).is_none()).count();
    r.push(
        "repository",
        "no edge dangles",
        if dangling == 0 { Status::Ok } else { Status::Problem },
        if dangling == 0 { "every endpoint is a node".to_string() } else { format!("{dangling} dangling endpoint(s)") },
    );

    let crates =
        survey.nodes.iter().filter(|n| n.kind == delulu_survey::NodeKind::Crate && n.id != "workspace").count();
    let agrees = survey.facts.crates as usize == crates && survey.facts.crates_shipped <= survey.facts.crates;
    r.push(
        "repository",
        "totals agree with contents",
        if agrees { Status::Ok } else { Status::Problem },
        if agrees {
            format!("{} workspace members, {} shipped", survey.facts.crates, survey.facts.crates_shipped)
        } else {
            format!("facts say {} crates; the map holds {crates}", survey.facts.crates)
        },
    );
}

// --- output ---------------------------------------------------------------------------------------

fn render(r: &Report) {
    let mut section = "";
    for c in &r.checks {
        if c.section != section {
            println!("\n{}", c.section);
            section = c.section;
        }
        println!("  {:<8} {:<28} {}", c.status.word(), c.name, c.detail);
    }

    let problems = r.problems();
    let fixed = r.checks.iter().filter(|c| c.status == Status::Fixed).count();
    println!();
    if problems == 0 {
        let tail = if fixed > 0 { format!(", {fixed} fixed") } else { String::new() };
        println!("ok: {} check(s) passed{tail}", r.checks.len());
    } else {
        println!("{problems} problem(s) remain — see the lines marked `problem` above");
    }
}

/// Exactly one JSON object (`docs/for-agents.md`). No DL code is invented for a doctor report — the
/// code registry is a stable contract and a health check is not a language diagnostic — so
/// `diagnostics` stays empty and the findings ride in an additive `doctor` object.
fn envelope(r: &Report) -> String {
    let mut checks = String::new();
    for (i, c) in r.checks.iter().enumerate() {
        if i > 0 {
            checks.push(',');
        }
        checks.push_str(&serde_json::json!({
            "section": c.section,
            "name": c.name,
            "status": c.status.word(),
            "detail": c.detail,
        }).to_string());
    }

    let mut root = serde_json::json!({
        "command": "doctor",
        "schema": 1,
        "delulu_version": env!("CARGO_PKG_VERSION"),
        "diagnostics": [],
        "summary": { "errors": r.problems(), "warnings": 0 },
    });

    let mut doctor = serde_json::json!({
        "checks": serde_json::from_str::<serde_json::Value>(&format!("[{checks}]")).unwrap_or(serde_json::json!([])),
        "regenerated": r.regenerated,
    });
    if let Some((nodes, edges, [e, w, n])) = r.survey {
        doctor["survey"] = serde_json::json!({
            "nodes": nodes,
            "edges": edges,
            "discrepancies": { "error": e, "warning": w, "note": n },
        });
    }
    root["doctor"] = doctor;
    root.to_string()
}
