//! `delulu run` — the command that actually executes a program.
//!
//! Split out of `cli.rs` by measurement rather than by taste (ruling D69). `cmd_run` alone was 945
//! lines, the largest item in an 8,856-line file, and the coupling was checked before the cut: of
//! the 143 top-level items in `cli.rs` this subsystem reaches **24**, and two thirds of those are
//! its own helpers. What it needs from the dispatcher is a short, stable list — option parsing,
//! diagnostic printing, the palette, the positional refusal — which is exactly the shape a seam
//! should have.
//!
//! **What deliberately did NOT move**: `mint_device_nodes`, `authority_spec_from_grants`,
//! `grants_from_lease` and friends have call sites in `grants`/`guard` as well. Moving shared code
//! into one command's module would trade a large file for a false ownership claim, which is worse
//! architecture rather than better — so they stay where both callers can see them.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_diag::{render_human_with, Diagnostic, SourceMap};
use delulu_runtime::{parse_manifest, Grants, Interp, Value};
use serde_json::{json, Value as Json};

use super::cli::*;
/// `run <package-dir>`: resolve the dependency graph, check it, and flatten it for execution.
///
/// Closes campaign finding **C59** (ruling D61): `delulu run` took a single `.delulu` file or a
/// `.dwx`, so a multi-package program — and even the shipped two-module `examples/greeter/` — could
/// only ever be *checked*. `kind = "bin"` was a manifest field the toolchain could not honour.
///
/// **The order here is the ruling.** `check_workspace` is authoritative: per-module visibility,
/// package authority ceilings, dependency pins, `pub`/`pub import` export rules. Nothing runs unless
/// it passes. Only then is the program flattened into one module for the interpreter, and the
/// flattened form's own check exists solely to *discover a name collision between modules* — never
/// to admit a program the workspace rejected.
///
/// A collision is refused with its own message rather than silently resolved by merge order. That is
/// the same bargain the WASM backend makes with DL1201: a bounded capability with a fail-closed edge
/// beats an unbounded one that sometimes runs the wrong function. Lifting it needs per-module
/// resolution inside `Interp`, for which the checker already computes `Program::call_owner`.
fn load_package_for_run(dir: &str, opts: &Opts) -> Result<(SourceMap, delulu_check::Checked), i32> {
    // A directory that is not a package at all. Before D61 this said "is a directory — try `delulu
    // build`", which was right then and is wrong now: `run` DOES take a directory. What it must never
    // do is leak the raw OS error (`Access is denied. (os error 5)` on Windows, `Is a directory` on
    // Linux) — two different misleading texts for one mistake, which is what C27 was.
    if !std::path::Path::new(dir).join("delulu.toml").is_file() {
        eprintln!("error: `{dir}` is a directory and not a DeluluLang package — there is no `delulu.toml` in it");
        eprintln!(
            "note: name the file to run, e.g. `{}`, or add a `delulu.toml` to run the directory as a package",
            std::path::Path::new(dir).join("main.delulu").display()
        );
        return Err(2);
    }

    let ws = delulu_check::deps::resolve_workspace(dir);
    // An empty package: a manifest with no sources. `build` already refuses this with its own note
    // (C26/D33 — "nothing checked, success claimed" was the defect); `run` says the same thing rather
    // than inventing a second vocabulary for one condition.
    if ws.modules.is_empty() {
        let mut diags: Vec<Diagnostic> = ws.diagnostics.clone();
        if errors(&diags) > 0 {
            print_diagnostics("run", &diags, &ws.source_map, None, opts.json);
        } else {
            diags.clear();
            eprintln!(
                "error: no `.delulu` modules found under `{}` — a DeluluLang package keeps its \
                 sources in `src/`, so there is nothing to run",
                std::path::Path::new(dir).join("src").display()
            );
        }
        return Err(2);
    }
    let program = delulu_check::deps::check_workspace(&ws);

    // The authoritative gate.
    let mut diags: Vec<Diagnostic> = ws.diagnostics.clone();
    diags.extend(program.diagnostics.iter().cloned());
    if errors(&diags) > 0 {
        print_diagnostics("run", &diags, &ws.source_map, None, opts.json);
        return Err(1);
    }

    let Some(entry_name) = program.entry_module.clone().or_else(|| ws.root_entry_module()) else {
        eprintln!("error: `{dir}` declares no entry module — a runnable package needs one");
        return Err(2);
    };
    if !ws.modules.iter().any(|m| m.unit.name == entry_name) {
        eprintln!("error: `{dir}`'s entry module `{entry_name}` was not loaded");
        return Err(2);
    }

    // Flatten the SOURCE, then parse once. Merging the module ASTs instead looks equivalent and is
    // not: each was parsed separately so their `NodeId`s overlap, and every checker side table keyed
    // by node id then reads one module's entry for another module's expression. The first attempt did
    // that and produced a nonsense reference-capability complaint about a correct program.
    let module_count = ws.modules.len();
    let texts: Vec<&str> =
        ws.modules.iter().map(|m| ws.source_map.file(m.unit.file).src.as_str()).collect();
    let flat_src = delulu_check::flatten_sources(&entry_name, &texts);

    // The flattened program gets its own file in the map, so a diagnostic from it can still be
    // rendered with a real snippet rather than pointing into a file whose offsets no longer apply.
    let mut map = ws.source_map;
    let fid = map.add_file(format!("{dir} (flattened for execution)"), flat_src.clone());
    let flat = check_source(fid, &flat_src);

    if errors(&flat.diagnostics) > 0 {
        // The workspace accepted this program, so these are not the author's errors: flattening
        // collided. Say which name, and say that the program is CORRECT — the runner is what is
        // limited. Anything less would send someone hunting a bug in code the checker just approved.
        let dup: Vec<String> = flat
            .diagnostics
            .iter()
            .filter(|d| d.is_error())
            .map(|d| d.message.clone())
            .take(3)
            .collect();
        eprintln!(
            "error: `{dir}` checks clean as a {module_count}-module program, but two of its modules \
             declare the same top-level name, and running it flattens them into one scope"
        );
        for m in &dup {
            eprintln!("  collision: {m}");
        }
        eprintln!(
            "note: the program is not wrong — `delulu check`/`build`/`authority` all handle it. The \
             RUNNER cannot yet distinguish two same-named declarations from different modules \
             (campaign finding C59, ruling D61). Rename one, or run the entry module as a single \
             file if it does not import."
        );
        return Err(1);
    }

    Ok((map, flat))
}

/// `run <file>.dwx`: re-verify a pre-built artifact's embedded `delulu:authority` manifest against
/// its code (DL1202 on a missing/tampered section, DL1204 on an incompatible version), announce
/// what it declares it can do, then run `main` under the deny-by-default Wasmtime host.
fn run_dwx_artifact(file: &str, opts: &Opts) -> i32 {
    let bytes = match std::fs::read(file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: cannot read `{file}`: {e}");
            return 2;
        }
    };
    let map = SourceMap::new(); // a .dwx carries no source text; diagnostics have no span
    let artifact = match delulu_wasm::read_and_verify(&bytes) {
        Ok(a) => a,
        Err(e) => {
            let d = Diagnostic::error(e.code(), format!("`{file}`: {}", e.message()));
            print_diagnostics("run", &[d], &map, None, opts.json);
            return 1;
        }
    };

    let mut grants = Grants::default();
    for g in &opts.grants {
        if let Err(e) = grants.add(g) {
            eprintln!("error: bad --grant: {e}");
            return 2;
        }
    }

    // The thesis made concrete: the artifact travels with its authority. Announce it before running
    // (human mode only; JSON mode keeps stdout clean for the program's own output).
    if !opts.json {
        let effects = artifact.authority.get("effects").map(strs).unwrap_or_default();
        let effects_str = if effects.is_empty() { "(none — provably pure)".to_string() } else { effects.join(", ") };
        eprintln!("running `{file}` — authority verified; declared effects: {effects_str}");
    }

    // A `.dwx` carries no source AST, so it has no foreign signatures to bind: a foreign-using `.dwx`
    // is not executable in phase 4g (its authority manifest DOES record the foreign entries — the
    // manifest requirement — but running foreign code needs the source's marshalling signatures).
    let cfg = delulu_wasm::HostConfig {
        console: grants.console,
        clock: grants.clock,
        rand: grants.rand,
        fs_read_roots: grants.build_root().fs_read,
        fixed_clock_ms: opts.clock_ms,
        rand_seed: opts.seed,
        ..delulu_wasm::HostConfig::default()
    };
    note_program_started();
    match delulu_wasm::run_main(&artifact.wasm, &cfg) {
        Ok(output) => {
            print!("{output}");
            0
        }
        Err(e) => {
            let msg = e.message();
            let d = Diagnostic::error(wasm_fault_code(&msg), format!("`{file}`: {msg}"));
            print_diagnostics("run", &[d], &map, None, opts.json);
            1
        }
    }
}

// ----- the run report (D-V2-21, PS-0-02) ---------------------------------------------------------
//
// `--report-out <path>`: the runtime writes one envelope (`command: "run"`) carrying the `sandbox`
// object and the outcome, whether the program ran or was refused before it started. It never goes
// on the program's stdout or stderr — the program writes there too and could print a counterfeit.
// The same two rules guard the report and `--trace-out`, the other file the runtime writes on the
// program's behalf: a path inside a scope the program may WRITE is refused before the run (a
// program must never write or redirect the record of its own confinement), and the file is opened
// without following a symbolic link at its final component.

thread_local! {
    /// Whether `main` was reached (the report's `outcome.ran`), and whether a runtime-written path
    /// was refused — a refused report path is never written, not even with the refusal.
    static RUN_STATE: std::cell::Cell<(bool, bool)> = const { std::cell::Cell::new((false, false)) };
}

pub(crate) fn note_program_started() {
    RUN_STATE.with(|s| s.set((true, s.get().1)));
    start_budget_watchdog();
}

/// PS-B-01: what the budget watchdog needs to stop a run and still report it, captured before the
/// program starts — the watchdog runs on its own thread and must not need anything the interpreter
/// holds.
#[derive(Clone)]
struct BudgetRun {
    budget: crate::budget::Budget,
    report_out: Option<String>,
    requested: String,
    /// Set when the watchdog starts, so a second [`note_program_started`] cannot start a second one.
    /// The record itself stays, because the run report written after the program reads the budget
    /// back from it (a lease may have narrowed it after it was first set, PS-B-05).
    armed: bool,
}

static BUDGET_RUN: std::sync::Mutex<Option<BudgetRun>> = std::sync::Mutex::new(None);

/// Arm the watchdog the moment the program starts, on whichever engine runs it — the interpreter,
/// the WASM engine and a `.dwx` artifact all pass through [`note_program_started`], which is what
/// makes one budget hold on every engine (PS-B-01's "on every engine" is this one call site).
fn start_budget_watchdog() {
    let Some(ctx) = BUDGET_RUN.lock().ok().and_then(|mut g| {
        let r = g.as_mut().filter(|r| !r.armed)?;
        r.armed = true;
        Some(r.clone())
    }) else {
        return;
    };
    let budget = ctx.budget;
    crate::budget::watch(budget, move |breach| stop_for_budget(&ctx, breach));
}

/// The budget this run is held to, once decided (before `main`). `None` outside `delulu run`.
fn run_budget() -> Option<crate::budget::Budget> {
    BUDGET_RUN.lock().ok().and_then(|g| g.as_ref().map(|r| r.budget))
}

/// PS-B-05: replace the run's budget before the program starts — the only caller is a `--lease` run
/// whose node carries a delegated budget. After the watchdog is armed this would change nothing it
/// enforces, so it refuses to pretend: it is an error to call it then, and it says so.
fn hold_run_to(b: crate::budget::Budget) -> Result<(), &'static str> {
    let mut g = BUDGET_RUN.lock().map_err(|_| "the run's budget record is poisoned")?;
    match g.as_mut() {
        Some(r) if !r.armed => {
            r.budget = b;
            Ok(())
        }
        Some(_) => Err("the budget watchdog is already running"),
        None => Err("no budget was recorded for this run"),
    }
}

/// A budget was spent: say which, from the watchdog's own measurement; write the report; end the run
/// as a failure. Never returns.
fn stop_for_budget(ctx: &BudgetRun, breach: crate::budget::Breach) -> ! {
    eprintln!("error: {}", breach.explain());
    if let Some(path) = &ctx.report_out {
        let egress = delulu_runtime::egress::snapshot();
        let mut env = success_envelope(
            "run",
            json!({
                "sandbox": l0_sandbox(&ctx.requested, &ctx.budget),
                "outcome": { "ran": true, "exit": 1, "stopped_by": breach.to_json() },
                "egress": egress.to_json(),
            }),
        );
        env["summary"]["errors"] = json!(1);
        let text = serde_json::to_string_pretty(&env).expect("the run report serializes");
        if let Err(e) = write_nofollow(path, format!("{text}\n").as_bytes()) {
            eprintln!("error: cannot write the run report to `{path}`: {e}");
        }
    }
    // What the program already printed is flushed if it can be — but not waited on forever: a
    // program blocked writing to a pipe nobody reads holds the lock, and a stop that waited for it
    // would be a stop that never happened.
    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::Builder::new().name("delulu-budget-flush".into()).spawn(move || {
        use std::io::Write as _;
        let _ = std::io::stdout().flush();
        let _ = tx.send(());
    });
    let _ = rx.recv_timeout(std::time::Duration::from_millis(200));
    std::process::exit(1)
}

/// The `sandbox` object of an L0 run report: the in-process backend, no OS boundary, strict mode —
/// since PS-B-01 the budgets the run is actually held to, and since PS-B-06 whether it ran by breaking
/// the glass, and on which ticket.
fn l0_sandbox(requested: &str, budget: &crate::budget::Budget) -> Json {
    let ticket = BREAK_GLASS.lock().ok().and_then(|g| g.clone());
    let mut v = json!({
        "backend": "inproc",
        "level": 0,
        "requested": requested,
        "granted": "none",
        "host_guarantees": [],
        "limits": budget.to_json(),
        "mode": "strict",
        "break_glass": ticket.is_some(),
    });
    if let Some(t) = ticket {
        v["break_glass_ticket"] = t;
    }
    v
}

/// PS-B-06: the accepted ticket of a break-glass run, for its report.
static BREAK_GLASS: std::sync::Mutex<Option<Json>> = std::sync::Mutex::new(None);

/// Refuse `--report-out` / `--trace-out` inside any filesystem scope the program may write,
/// comparing RESOLVED paths (links, `..`, case — `prim::resolve_for_decision`, the containment
/// code's own resolution). A path that cannot be resolved is refused too: cannot tell is no.
pub(crate) fn refuse_runtime_paths_in_write_scopes(opts: &Opts, write_roots: &[std::path::PathBuf]) -> Option<i32> {
    for (flag, path) in [("--report-out", &opts.report_out), ("--trace-out", &opts.trace_out)] {
        let Some(path) = path else { continue };
        // A bare file name (`rep.json`) has an EMPTY parent, which no resolver can resolve; it means
        // the current directory. Without this the commonest spelling was refused as unresolvable.
        let p = std::path::Path::new(path);
        let p = if p.parent().is_some_and(|d| d.as_os_str().is_empty()) {
            std::path::Path::new(".").join(p)
        } else {
            p.to_path_buf()
        };
        let Some(target) = delulu_runtime::prim::resolve_for_decision(&p) else {
            eprintln!("error: {flag} `{path}` cannot be resolved (a dangling link on the way?) — refused before the run");
            RUN_STATE.with(|s| s.set((s.get().0, true)));
            return Some(2);
        };
        for root in write_roots {
            // A write scope that does not resolve cannot be compared, so it cannot be excluded:
            // cannot tell is no (the skip-branch rule), never "skip this root".
            let Some(r) = delulu_runtime::prim::resolve_for_decision(root) else {
                eprintln!(
                    "error: {flag} `{path}` cannot be checked against the write scope `{}`, which does not \
                     resolve — refused before the run",
                    root.display()
                );
                RUN_STATE.with(|s| s.set((s.get().0, true)));
                return Some(2);
            };
            if target.starts_with(&r) {
                eprintln!(
                    "error: {flag} `{path}` lies inside `{}`, a scope this program is granted to write — \
                     a program must never be able to write or redirect the runtime's own record of \
                     its run (D-V2-21); nothing ran. Name a path outside every `fs.write` grant.",
                    root.display()
                );
                RUN_STATE.with(|s| s.set((s.get().0, true)));
                return Some(2);
            }
        }
    }
    None
}

/// Write a runtime-owned file without following a symbolic link at its final component.
pub(crate) fn write_nofollow(path: &str, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write as _;
    let p = std::path::Path::new(path);
    if std::fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err(std::io::Error::other("the path is a symbolic link; the runtime will not follow it"));
    }
    let mut o = std::fs::OpenOptions::new();
    o.write(true).create(true).truncate(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // FILE_FLAG_OPEN_REPARSE_POINT: a link raced in after the check is opened AS a link.
        o.custom_flags(0x0020_0000);
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.custom_flags(0o400_000); // O_NOFOLLOW
    }
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.custom_flags(0x0100); // O_NOFOLLOW
    }
    let mut f = o.open(p)?;
    if f.metadata()?.file_type().is_symlink() {
        return Err(std::io::Error::other("the path became a symbolic link; the runtime will not follow it"));
    }
    f.write_all(bytes)
}

/// PS-B-02: tell the operator, on stderr, about every `http.get` that did not deliver — the reason
/// the PROGRAM is deliberately not told (it sees only `NetErr`). Human prose, so it is withheld under
/// `--json`, where the same records are in the run report's `egress` object.
pub(crate) fn print_egress_notes(log: &delulu_runtime::egress::EgressLog) {
    for r in &log.records {
        if let Err(reason) = &r.outcome {
            eprintln!(
                "note: `http.get` of `{}` was not delivered: {} [egress: {}]",
                r.url.escape_debug(),
                reason.explain(),
                reason.code()
            );
        }
    }
    let shown = log.records.iter().filter(|r| r.outcome.is_err()).count() as u64;
    if log.refused > shown {
        eprintln!(
            "note: {} more request(s) were not delivered; the log keeps the first {} records",
            log.refused - shown,
            delulu_runtime::egress::MAX_RECORDED
        );
    }
}

/// The run report envelope. At L0 the `sandbox` object is the in-process backend: level 0, no host
/// guarantees, the budgets the run is held to (PS-B-01), strict mode, no break-glass — measured
/// facts, nothing claimed. `egress` is every network request the program made and what became of it
/// (PS-B-02), always present so a reader never has to ask whether its absence means "none".
fn run_report(opts: &Opts, exit: i32, egress: &delulu_runtime::egress::EgressLog, budget: &crate::budget::Budget) -> Json {
    let (ran, _) = RUN_STATE.with(|s| s.get());
    let requested = opts.isolation.clone().unwrap_or_else(|| "none".to_string());
    let mut env = success_envelope(
        "run",
        json!({
            "sandbox": l0_sandbox(&requested, budget),
            "outcome": { "ran": ran, "exit": exit },
            "egress": egress.to_json(),
        }),
    );
    if exit != 0 {
        env["summary"]["errors"] = json!(1);
    }
    env
}

pub(crate) fn cmd_run(rest: &[String]) -> i32 {
    RUN_STATE.with(|s| s.set((false, false)));
    let (file, opts) = parse_opts(rest);
    // PS-A-07: `--sandbox` runs the program as a jailed guest holding no authority of its own.
    // `--sandbox=off` is the explicit opposite, and saying it is the point: a run that is not
    // confined should be a sentence someone wrote. Anything else is refused rather than guessed.
    // PS-A-10, the transition matrix of `V2_SECURITY_MODEL.md` §5. Two of its rules are decided
    // right here, before a line of the program is read.
    //
    // ON -> OFF must never happen QUIETLY. Two contradictory `--sandbox` requests on one command
    // line used to be resolved by whichever came last, which means `--sandbox --sandbox=off` — a
    // wrapper script's default followed by an argument a caller appended, or the other way round —
    // silently decided the boundary. A downgrade must be a sentence someone wrote, so a command
    // line that asks for both is refused and neither wins.
    {
        let mut distinct: Vec<&str> = opts.sandbox_said.iter().map(String::as_str).collect();
        distinct.sort_unstable();
        distinct.dedup();
        if distinct.len() > 1 {
            eprintln!(
                "error: this command line asks for `--sandbox` {} times with different answers ({}). \
                 Nothing ran: a sandbox that goes on or off by argument order is not a decision \
                 anyone made. Say it once.",
                opts.sandbox_said.len(),
                distinct.join(", ")
            );
            return 2;
        }
    }
    // A sandbox flag that is not read is worse than one that refuses: it reads as applied. `--mode`
    // and `--sandbox-profile` describe a sandboxed run and nothing else, so asking for them without
    // `--sandbox` is refused rather than dropped on the floor. `--limits` left this list at PS-B-01:
    // an ordinary run has budgets now, so the flag is APPLIED to it rather than refused.
    if opts.sandbox.as_deref() != Some("on") {
        for (flag, asked) in [
            ("--mode", opts.sandbox_mode.is_some()),
            ("--sandbox-profile", opts.sandbox_profile.is_some()),
        ] {
            if asked {
                eprintln!(
                    "error: `{flag}` describes a sandboxed run, and this run is not one. Nothing ran, \
                     because a flag that is silently ignored reads exactly like a flag that was \
                     applied. Add `--sandbox`, or drop `{flag}`."
                );
                return 2;
            }
        }
    }
    // PS-B-06: a host whose operator requires the sandbox runs nothing outside it, unless this run
    // carries a ticket they signed for exactly this program. Decided before either path reads a line.
    {
        let program = file.as_deref().and_then(|f| std::fs::read(f).ok());
        let sandboxed = opts.sandbox.as_deref() == Some("on");
        match crate::breakglass::gate("run", sandboxed, opts.break_glass.as_deref(), program.as_deref()) {
            Err(code) => return code,
            Ok(Some(accepted)) => {
                if let Ok(mut g) = BREAK_GLASS.lock() {
                    *g = Some(crate::breakglass::record(&accepted));
                }
            }
            Ok(None) => {}
        }
    }
    if let Some(choice) = opts.sandbox.as_deref() {
        match choice {
            "off" => {}
            "on" => return crate::guest::cmd_run_sandboxed(file.as_deref(), &opts, rest),
            other => {
                eprintln!("error: `--sandbox={other}` is not a value this command knows (use `--sandbox` or `--sandbox=off`)");
                return 2;
            }
        }
    }
    // The grants named on the command line are refused here, before anything else; a lease's or a
    // manifest's are refused again once known (`cmd_run_inner`).
    if opts.report_out.is_some() || opts.trace_out.is_some() {
        let mut g = Grants::default();
        for spec in &opts.grants {
            let _ = g.add(spec);
        }
        if let Some(code) = refuse_runtime_paths_in_write_scopes(&opts, &g.build_root().fs_write) {
            return code;
        }
    }
    // PS-B-01 (D-V2-25): the main program's budgets, decided before anything runs — a budget that
    // cannot be meant (zero, or a dimension nobody enforces) is refused here, not discovered later.
    // The sandboxed path above keeps its own rule: there `--limits` may only narrow a profile.
    let budget = match crate::budget::Budget::parse(opts.limits.as_deref()) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: {e}");
            return 2;
        }
    };
    if let Ok(mut g) = BUDGET_RUN.lock() {
        *g = Some(BudgetRun {
            budget,
            report_out: opts.report_out.clone(),
            requested: opts.isolation.clone().unwrap_or_else(|| "none".to_string()),
            armed: false,
        });
    }
    let code = cmd_run_inner(rest);
    // A lease may have narrowed the budget inside `cmd_run_inner` (PS-B-05); report what was held.
    let budget = run_budget().unwrap_or(budget);
    // If the watchdog is already stopping this run, it owns the exit: it has written the report and
    // chosen the code, and a second report here would contradict it.
    if !crate::budget::claim_normal_exit() {
        loop {
            std::thread::park();
        }
    }
    let egress = delulu_runtime::egress::take_log();
    if !opts.json {
        print_egress_notes(&egress);
    }
    if let Some(path) = &opts.report_out {
        let refused = RUN_STATE.with(|s| s.get().1);
        if !refused {
            let text = serde_json::to_string_pretty(&run_report(&opts, code, &egress, &budget)).expect("the run report serializes");
            if let Err(e) = write_nofollow(path, format!("{text}\n").as_bytes()) {
                eprintln!("error: cannot write the run report to `{path}`: {e}");
                return if code == 0 { 2 } else { code };
            }
        }
    }
    code
}

fn cmd_run_inner(rest: &[String]) -> i32 {
    let (file, mut opts) = parse_opts(rest);
    if let Some(code) = refuse_extra_positionals("run", &opts) {
        return code;
    }
    // `run` is the command where a dropped flag costs the most: a mistyped `--grant`, `--engine`
    // or `--isolation` used to be discarded and the program ran anyway, under whatever the
    // defaults were, reporting success.
    if let Some(code) = refuse_unknown_flags("run", &opts) {
        return code;
    }
    let Some(file) = file else {
        eprintln!("error: `run` needs a file");
        return 2;
    };

    // ----- isolation profile gate (Stage 5 phase 5i, spec §6) -----------------------------------
    // Decided BEFORE anything runs. `microvm` is Linux+KVM-first and refused honestly everywhere
    // else — DL1408 with the documented fallback, NEVER a silent substitution of weaker isolation
    // (playbook trap 8). The capability matrix is spec §6.1.
    match opts.isolation.as_deref() {
        None | Some("none") | Some("process") => {}
        Some("microvm") => {
            match microvm_unavailable() {
                Err(detail) => {
                    let d = Diagnostic::error(
                        "DL1408",
                        format!(
                            "isolation profile `microvm` is unavailable: {detail} — fall back to \
                             `delulu run {file} --sandbox` (a jailed guest process holding no authority of \
                             its own; the host performs every effect. Weaker than microvm: one OS account, \
                             no guest kernel, no default-deny egress — `delulu explain E-SANDBOX`), or to \
                             `delulu run {file} --isolation process` (worker-style OS containment of the \
                             code outside the proof; weaker still: it confines FOREIGN code only and leaves \
                             the program itself unconfined) [a human must choose the weaker profile; see \
                             `delulu explain DL1408` and spec §6.1]"
                        ),
                    )
                    // NE-16c / PS-0-03: the repair STAGE5 §8 promised. No edit — choosing a weaker
                    // boundary is a person's decision — and the fallback command, ready to run.
                    .with_repair(delulu_diag::Repair {
                        // Renamed when a second, stronger fallback appeared: an id naming only one
                        // of two options is a small lie in a machine-readable field.
                        id: "fall-back-to-a-weaker-profile",
                        confidence: delulu_diag::Confidence::Suggest,
                        authority_widening: false,
                        requires_human: true,
                        edits: Vec::new(),
                        reason: Some(
                            "both fallbacks are explicitly weaker, so a human must choose. `--sandbox` is \
                             the stronger of the two — a jailed guest process that holds no authority, and \
                             available on all three systems — so it is named first; it is still one OS \
                             account with no guest kernel and no default-deny egress. `--isolation process` \
                             is weaker still: it confines foreign code only and leaves the verified program \
                             in-process. The message names both exact commands: \
                             `delulu run <file> --sandbox` and `delulu run <file> --isolation process`",
                        ),
                    });
                    print_diagnostics("run", &[d], &SourceMap::new(), None, opts.json);
                    return 1;
                }
                Ok(()) => {
                    // The Linux+KVM guest-launch path (spec §6) lands with criterion-8 CI work;
                    // v0.5's probe never returns Ok. Refuse loudly rather than pretend.
                    eprintln!("error: the microVM guest launch is not wired in this build (v0.5)");
                    return 2;
                }
            }
        }
        Some(other) => {
            eprintln!("error: unknown --isolation `{other}` (none | process | microvm)");
            return 2;
        }
    }
    if opts.isolation.as_deref() == Some("process") && opts.foreign_isolation.is_none() {
        // The process profile's v0.5 mechanism: force the code OUTSIDE the proof (foreign libs)
        // into minimum-privilege worker subprocesses (phase 5h). The verified program itself stays
        // in-process, bounded by the effect system + custody — stated honestly in the label below
        // and in the spec §6.1 matrix (never sold as microVM-equivalent).
        opts.foreign_isolation = Some("process".to_string());
    }
    if let Some(iso) = &opts.isolation {
        if !opts.json {
            let label = match iso.as_str() {
                // PS-A-07: announced, not changed. `--isolation process` still means exactly what it
                // always meant, because changing a profile under a caller who asked for it by name is
                // the silent substitution trap 8 forbids. But an operator reaching for it FOR
                // CONTAINMENT now has a stronger option that did not exist when this label was written,
                // and not naming it would leave them holding the weaker one with no way to find out.
                "process" => {
                    "process — isolates FOREIGN CODE ONLY: foreign libraries run in \
                     minimum-privilege worker subprocesses; the verified program itself stays \
                     in-process and is not sandboxed (weaker than microvm; spec §6.1). If you want \
                     the PROGRAM confined, `--sandbox` runs it as a jailed guest holding no \
                     authority of its own — `delulu explain E-SANDBOX`"
                }
                _ => {
                    "none — in-process (language + custody enforcement only). `--sandbox` runs the \
                     program as a jailed guest instead; `delulu explain E-SANDBOX`"
                }
            };
            eprintln!("isolation: {label}");
        }
    }

    // ----- device profile + the sim-to-hardware gate (Stage 10 phase 10f, spec §5.4) ------------
    // Decided BEFORE anything runs, for the same reason the isolation profile is: a hardware
    // grant for an artifact nobody approved must never reach the point of moving something.
    let device_profile = match resolve_device_profile(&file, &opts) {
        Ok(p) => p,
        Err(code) => return code,
    };
    // D20: the deterministic simulator clock. Only meaningful under `sim`, where it replaces the
    // wall-clock dead-man with a step-per-interaction clock so a demonstration's timing is a
    // function of the command sequence, not of interpreter speed. Refused loudly elsewhere: a
    // hardware run's dead-man is a real-time promise and must never be quietly stepped.
    let device_clock = match opts.sim_step {
        None => delulu_runtime::ClockMode::Wall,
        Some(0) => {
            eprintln!("error: --sim-step needs a positive number of simulated milliseconds");
            return 2;
        }
        Some(ms) => {
            if !matches!(device_profile, delulu_runtime::Profile::Sim { .. }) {
                eprintln!(
                    "error: --sim-step is only meaningful with --broker-profile sim; the wall-clock \
                     dead-man is a real-time guarantee and is not stepped"
                );
                return 2;
            }
            delulu_runtime::ClockMode::Stepped { step_us: ms.saturating_mul(1000) }
        }
    };


    // A `.dwx` is a pre-built, authority-carrying artifact — re-verify and run it directly.
    if file.ends_with(".dwx") {
        return run_dwx_artifact(&file, &opts);
    }
    // A package DIRECTORY: resolve the dependency graph, check it authoritatively, then flatten it
    // for execution (ruling D61, closing C59 — `kind = "bin"` was declarable and unexecutable).
    let is_package = std::path::Path::new(&file).is_dir();
    let (map, checked) = if is_package {
        match load_package_for_run(&file, &opts) {
            Ok(x) => x,
            Err(c) => return c,
        }
    } else {
        let (map, id, src) = match load(&file) {
            Ok(x) => x,
            Err(c) => return c,
        };
        let checked = check_source(id, &src);
        (map, checked)
    };
    if errors(&checked.diagnostics) > 0 {
        print_diagnostics("run", &checked.diagnostics, &map, None, opts.json);
        return 1;
    }
    if !checked.result.main_present {
        eprintln!("error: `{file}` has no `fn main(root: Root)` to run");
        return 2;
    }

    // Grant flow (§7.2): manifest DL0701 check, then reconcile grants. For a package the manifest
    // is the package's own; for a single file it is whatever sits beside it.
    let file_dir = std::path::Path::new(&file).parent().unwrap_or_else(|| std::path::Path::new("."));
    let dir = if is_package { std::path::Path::new(&file) } else { file_dir };
    let manifest = std::fs::read_to_string(dir.join("delulu.toml")).ok().map(|s| parse_manifest(&s));
    let empty = std::collections::BTreeSet::new();
    let main_row = checked.result.main_row.as_ref().unwrap_or(&empty);

    let mut grants = Grants::default();
    if let Some(m) = &manifest {
        let d = m.check_main_row(main_row);
        if !d.is_empty() {
            print_diagnostics("run", &d, &map, None, opts.json);
            return 1;
        }
        if opts.grant_manifest {
            grants.accept_manifest(m);
        }
    }
    for g in &opts.grants {
        if let Err(e) = grants.add(g) {
            eprintln!("error: bad --grant: {e}");
            return 2;
        }
    }

    // ----- the compute attestation gate (Stage 10 phase 10h, spec §7.1 — DL1911) -----------------
    // Decided here, before a line of the program runs, for a sharper reason than convenience: the
    // claim this gate protects is that a device envelope is enforced in TWO places. If the adapter
    // cannot attest a layer below itself, the claim is single — and the run must either say so out
    // loud, via an explicit human waiver in the grant, or not happen. *Silently* single is the one
    // outcome this code exists to prevent.
    for env in &grants.computes {
        if let Err(refusal) = delulu_runtime::compute::check_grant(env) {
            let d = Diagnostic::error(
                "DL1911",
                format!(
                    "`{}`: {} [a human must accept single enforcement for this device; see \
                     `delulu explain DL1911` and spec §7.1]",
                    env.device,
                    refusal.message()
                ),
            );
            print_diagnostics("run", &[d], &map, None, opts.json);
            return 1;
        }
    }

    // Stage 10 (invariant 46): a `@jit` hint without the `exec.native` grant is IGNORED — the
    // program runs interpreted, sandbox intact — and DL1906 says so on the human channel
    // (stderr, the DL1790 surface-warning pattern; never inside `--json`, whose bytes for
    // untouched programs are law). A hint may not change whether a program runs (invariant 45),
    // so this is a warning by construction, never a refusal.
    if module_requests_native(&checked.module) && !grants.exec_native {
        let mut w = Diagnostic::warning(
            "DL1906",
            "`@jit` hint ignored: native-code emission was not granted (`--grant exec.native`)",
        );
        if manifest.as_ref().is_some_and(|m| !m.exec_native) {
            w.message.push_str(
                " — and the manifest does not declare the request (`[authority] exec.native = true`)",
            );
        }
        if !opts.json {
            let wmap = SourceMap::new();
            let palette = palette_stderr();
            eprint!("{}", render_human_with(&w, &wmap, &palette));
        }
    }

    // ----- lease redemption (Stage 5 phase 5j, spec §3.2/§3.3) ----------------------------------
    // `--lease <token>` redeems a delegated lease and runs under EXACTLY that node: the local
    // grants are DERIVED from the node's authority (never widened locally — the broker enforces per
    // §4 regardless), and custody binds to the redeemed node via `for_node`. This is the
    // orchestration payoff: whoever holds a grant delegates a slice, hands the token to an agent,
    // and the agent runs under exactly that attenuated authority. Fail closed: broker down ⇒
    // DL1401; a bad/expired/already-redeemed token ⇒ DL1407/DL1402 — before `main` ever runs.
    let lease_mode = opts.lease.is_some();
    let mut lease_custody: Option<crate::broker_client::BrokerClientCustody> = None;
    if let Some(token) = &opts.lease {
        if opts.broker.as_deref() == Some("embedded") {
            eprintln!("error: `--lease` runs under the broker daemon; it cannot be combined with `--broker embedded`");
            return 2;
        }
        // The lease IS the authority. Local `--grant` flags may only supply `foreign.c` binary
        // PATHS (the path is grant data — a human decision; the lease's `foreign.c` scope still
        // bounds WHICH libs, enforced by the broker's ForeignBind check). Anything else would be a
        // confusing local widening the broker would deny anyway — refuse it up front.
        // 10g: device grants are named here explicitly, and the reason is worth recording. This
        // list enumerates what a lease run may NOT be given locally, so every grant kind added
        // after it was written fell through to `else` and was silently DISCARDED by
        // `grants_from_lease` below. That is exactly what happened to `actuator=`/`sensor=`: the
        // operator typed a device grant, was told nothing, and the program then died at the mint
        // with `DL0703: actuator was not granted` — a diagnostic that blames the program for the
        // CLI having thrown the grant away. A refusal list is a skip branch wearing a disguise.
        if grants.console
            || grants.clock
            || grants.rand
            || grants.declassify
            || !grants.fs_read.is_empty()
            || !grants.fs_write.is_empty()
            || !grants.net.is_empty()
            || !grants.secrets.is_empty()
            || !grants.foreign_python.is_empty()
            || opts.grant_manifest
        {
            eprintln!(
                "error: a `--lease` run derives its authority from the delegated node — only \
                 `--grant foreign.c=LIB:PATH` (the binary path, which is grant data) may accompany it"
            );
            return 2;
        }
        // Devices get their own refusal, because the honest answer is not "you may not" but "this
        // cannot be delegated yet". A grant tree node carries the authority to actuate; it cannot
        // yet carry an ENVELOPE (`Scopes` has no actuator dimension), so there is no way for the
        // delegating side to say *how far* the holder may move a machine. Rather than run with the
        // device silently absent, say which grant was refused and why.
        // Two different refusals now, because the two cases stopped having the same reason.
        //
        // An actuator IS expressible in a grant node since RFC 0001 F1, so the refusal is no longer
        // "this cannot be bounded" — it is "you do not get to bound it yourself." A holder that
        // could hand itself a local envelope would be choosing its own corridor, which is precisely
        // the authority the delegating side is supposed to hold.
        if !grants.actuators.is_empty() {
            eprintln!(
                "error: a `--lease` run cannot take a local `--grant actuator=`: its device \
                 authority comes FROM the delegation, bounded by whoever delegated it. Put the \
                 envelope on the delegation instead:\n  \
                 delulu grants delegate --effects Actuate --device \
                 'arm0/elbow:angle_deg=-30..95,heartbeat_ms=200,ttl_ms=60000,fail=hold'"
            );
            return 2;
        }
        // A sensor still has no scope dimension at all, so this half of the old refusal stands
        // unchanged, with its original reason (build-order D12e, narrowed to sensors).
        if !grants.sensors.is_empty() {
            eprintln!(
                "error: a `--lease` run cannot take a local `--grant sensor=`: a sensor read is \
                 `Read` under a sensor scope, and `Scopes` has no sensor dimension yet, so the \
                 delegating side could not bound it. Run the program under `--broker daemon` with \
                 its own sensor grants instead (Stage 10 build order, ruling D12e)."
            );
            return 2;
        }
        let Some(state_dir) = crate::brokerd::resolve_state_dir(None) else {
            eprintln!("error: cannot resolve the broker state directory (no HOME/USERPROFILE)");
            return 2;
        };
        let fail = |code: &str, message: String| -> i32 {
            let d = Diagnostic::error(crate::broker_client::static_code(code), message);
            print_diagnostics("run", &[d], &map, None, opts.json);
            1
        };
        let unreachable_msg = |e: &dyn std::fmt::Display| {
            format!(
                "broker unreachable: {e} — start it with `delulu broker start` \
                 (fail closed, invariant 27: a lease run never falls back to embedded custody)"
            )
        };
        let peer = format!("pid:{}", std::process::id());
        let node = match crate::brokerd::request(
            &state_dir,
            crate::broker_ipc::ReqBody::Redeem { token: token.clone(), peer },
        ) {
            Ok(crate::broker_ipc::Response::Redeemed { node }) => node,
            Ok(crate::broker_ipc::Response::Error { code, message, .. }) => return fail(&code, message),
            Ok(other) => return fail("DL1401", format!("unexpected redeem response: {other:?}")),
            Err(e) => return fail("DL1401", unreachable_msg(&e)),
        };
        // Learn this run's OWN node's authority (its slice — never a parent's or a sibling's;
        // invariant 25 is upheld by what the ops expose, and this asks only about itself).
        let info = match crate::brokerd::request(
            &state_dir,
            crate::broker_ipc::ReqBody::Inspect { node: node.clone() },
        ) {
            Ok(crate::broker_ipc::Response::Inspected { node }) => node,
            Ok(crate::broker_ipc::Response::Error { code, message, .. }) => return fail(&code, message),
            Ok(other) => return fail("DL1401", format!("unexpected inspect response: {other:?}")),
            Err(e) => return fail("DL1401", unreachable_msg(&e)),
        };
        let authority = crate::brokerd::spec_to_authority(&info.authority_spec());
        // PS-B-05: a node that carries a budget holds its runs to it — the delegation is the ceiling
        // and, for what `--limits` leaves unnamed, the default. Decided here, before `main`, from the
        // same converted authority the custody below enforces (so an unreadable budget on the wire is
        // already the smallest one, never none). A node without one leaves the operator's budget
        // exactly as it was.
        if let Some(ceiling) = authority.scopes.budget {
            let held = match crate::budget::Budget::under_delegation(opts.limits.as_deref(), &ceiling) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("error: {e}");
                    return 2;
                }
            };
            if let Err(e) = hold_run_to(held) {
                eprintln!("error: cannot hold this lease run to its delegated budget: {e}");
                return 2;
            }
            if !opts.json {
                eprintln!(
                    "lease: held to the delegated budget, mem={} bytes, cpu={} s",
                    held.memory_bytes, held.cpu_seconds
                );
            }
        }
        grants = grants_from_lease(&info, std::mem::take(&mut grants.foreign_c));
        // The guard awareness line (addendum §2.7, criterion 8): EVERY `run --lease` prints one
        // guard status line before user code output — stderr always (in `--json` mode too: stdout
        // stays the machine surface, decorations ride stderr — the dcg robot-mode convention).
        // Sourced from a GuardStatus wire call post-redeem; an unreachable broker here would already
        // have failed the redeem above, but fail closed anyway.
        match crate::brokerd::request(&state_dir, crate::broker_ipc::ReqBody::GuardStatus) {
            Ok(crate::broker_ipc::Response::GuardStatus { bypass, poisoned, rules, .. }) => {
                eprintln!("{}", lease_guard_status_line(bypass, poisoned, &rules));
            }
            Ok(other) => return fail("DL1401", format!("unexpected guard status response: {other:?}")),
            Err(e) => return fail("DL1401", unreachable_msg(&e)),
        }
        let custody = crate::broker_client::BrokerClientCustody::for_node(
            state_dir,
            delulu_broker::GrantId::from_trusted(node),
            authority,
            opts.epoch_ms,
        );
        if !opts.json {
            eprintln!("lease: running under delegated node `{}`", custody.node());
        }
        lease_custody = Some(custody);
    }

    // Foreign grant flow (Stage 4, spec §4.1 / criterion 4): every `foreign` lib the program binds
    // must be permitted by the manifest (if any) and granted a binary at startup. An ungranted lib
    // is DL1303 HERE — before `main` runs — never mid-run. The path is grant data: the human/broker
    // decides which binary satisfies the logical name, prompted with the full symbol list.
    if let Err(code) = foreign_grant_preflight(&checked, &manifest, &mut grants, &opts, &map) {
        return code;
    }

    // ----- custody selection (Stage 5 phase 5f) ------------------------------------------------
    // Default: embedded — byte-identical Stage 1–4 behavior (criterion 11). `--broker daemon`
    // routes every effectful op through the broker daemon: the `--grant` flags become sugar for
    // issue-then-run at the root (spec §3.2), and an unreachable broker is DL1401 BEFORE `main`
    // runs (fail closed, invariant 27 — never a silent fallback to embedded).
    let daemon_mode = match opts.broker.as_deref() {
        // A `--lease` run's custody is the daemon by definition (the node lives there).
        None | Some("embedded") => lease_mode,
        Some("daemon") => true,
        Some(other) => {
            eprintln!("error: unknown --broker mode `{other}` (embedded | daemon)");
            return 2;
        }
    };
    if (opts.broker.is_some() || lease_mode) && !opts.json {
        // The custody label (playbook 5j; printed when custody was explicitly chosen so default
        // embedded output stays byte-identical to prior stages).
        eprintln!("custody: {}", if daemon_mode { "daemon" } else { "embedded" });
    }
    // A lease run already bound custody to the redeemed node above; a plain `--broker daemon` run
    // issues its root here (the `--grant` flags as issue-then-run sugar, spec §3.2).
    let mut daemon_custody: Option<crate::broker_client::BrokerClientCustody> = lease_custody;
    if daemon_mode && daemon_custody.is_none() {
        let Some(state_dir) = crate::brokerd::resolve_state_dir(None) else {
            eprintln!("error: cannot resolve the broker state directory (no HOME/USERPROFILE)");
            return 2;
        };
        let mut spec = authority_spec_from_grants(&grants, &file);
        // PS-B-05: the root records the budget this run is held to, so anything the program
        // delegates from it inherits that budget or narrows it, and never outspends its own run.
        spec.budget = run_budget().map(|b| b.to_scope().to_grant_string());
        match crate::broker_client::BrokerClientCustody::issue_root(state_dir, spec, opts.epoch_ms) {
            Ok(c) => daemon_custody = Some(c),
            Err(d) => {
                let diag = Diagnostic::error(d.code, d.message);
                print_diagnostics("run", &[diag], &map, None, opts.json);
                return 1;
            }
        }
    }

    // D-V2-21 / PS-0-02, the second time: the grants are final now (a lease's or a manifest's
    // scopes included), and neither runtime-written file may lie in a scope the program can write.
    if let Some(code) = refuse_runtime_paths_in_write_scopes(&opts, &grants.build_root().fs_write) {
        return code;
    }

    // ----- foreign isolation selection (Stage 5 phase 5h) --------------------------------------
    // `process` runs each granted C library in an isolated worker subprocess (a worker crash is
    // DL1409, the host survives — spec §5, criterion 7); `inproc` is the Stage-4 in-process path.
    // Default: `process` in daemon mode, `inproc` otherwise. Labeled below.
    let foreign_process = match opts.foreign_isolation.as_deref() {
        Some("process") => true,
        Some("inproc") => false,
        Some(other) => {
            eprintln!("error: unknown --foreign-isolation `{other}` (inproc | process)");
            return 2;
        }
        None => daemon_mode,
    };
    let program_binds_foreign = !checked.result.foreign_binds.is_empty();
    if program_binds_foreign && (opts.foreign_isolation.is_some() || daemon_mode) && !opts.json {
        eprintln!("foreign-isolation: {}", if foreign_process { "process" } else { "inproc" });
    }

    // WASM engine (spec §5.11 / Stage 3; foreign C FFI in Stage 4 phase 4g): compile `main` to
    // WebAssembly and run it under the deny-by-default Wasmtime host, with capabilities minted and
    // checked host-side and all foreign FFI performed host-side (§4.3). Constructs the backend can't
    // compile are DL1201 — omit `--engine wasm` to use the interpreter.
    if opts.engine.as_deref() == Some("wasm") {
        // Phase-5h scope note: worker isolation is wired on the INTERPRETER engine (the criterion-7
        // reference). The WASM host's foreign path stays in-process for v0.5 — flagged in spec §11.
        if foreign_process && program_binds_foreign && !opts.json {
            eprintln!("note: --foreign-isolation process is not yet applied on the WASM engine (foreign runs in-process); use the interpreter engine for worker isolation");
        }
        let wasm = match delulu_wasm::compile_module_with(&checked.module, &checked.result.foreign_binds) {
            Ok(w) => w,
            Err(e) => {
                let d = Diagnostic::error(e.code(), format!("{} — omit `--engine wasm` to run it on the interpreter", e.message()));
                print_diagnostics("run", &[d], &map, None, opts.json);
                return 1;
            }
        };
        // Effect tracing on the WASM engine (spec §6.1; criterion 6): the host records the same
        // `TraceRecord`s the interpreter would, so a foreign program's trace is byte-identical.
        let sink = if opts.assert_trace {
            Some(delulu_runtime::TraceSink::new())
        } else if opts.trace_effects {
            Some(delulu_runtime::TraceSink::bounded(TRACE_RECORD_CAP))
        } else {
            None
        };
        let max_ret = opts.foreign_max_ret.unwrap_or(delulu_runtime::foreign::DEFAULT_MAX_RET);
        let root = grants.build_root();
        // Stage 5 phase 5f: the WASM host calls the SAME custody seam as the interpreter. A broker
        // denial inside a host callback is recorded in HostState and surfaced after the call —
        // never an Err across the wasm frame (playbook trap 5).
        let wasm_custody: Option<delulu_wasm::CustodyHandle> = daemon_custody
            .take()
            .map(|c| Rc::new(std::cell::RefCell::new(Box::new(c) as Box<dyn delulu_runtime::Custody>)));
        let cfg = delulu_wasm::HostConfig {
            console: grants.console,
            clock: grants.clock,
            rand: grants.rand,
            fs_read_roots: root.fs_read.clone(),
            fixed_clock_ms: opts.clock_ms,
            rand_seed: opts.seed,
            foreign_load: root.foreign_load,
            foreign_grants: grants.foreign_c.clone(),
            foreign_sigs: delulu_wasm::foreign_sigs(&checked.module),
            foreign_max_ret: max_ret,
            trace: sink.clone(),
            custody: wasm_custody.clone(), // Stage 5 phase 5f: Some(...) in daemon mode, None embedded
        };
        // Stage 7 phase 7h: a module with actors runs under the WASM engine's COOPERATIVE
        // single-threaded scheduler (spec §6.5 — identical semantics, no parallelism,
        // labeled in output). A module without actors takes the byte-identical v0.6 path.
        let wasm_has_actors =
            checked.module.items.iter().any(|it| matches!(it, delulu_syntax::ast::Item::Actor(_)));
        note_program_started();
        let run_result = if wasm_has_actors {
            eprintln!("engine: wasm (actors: cooperative single-threaded — semantics identical, parallelism absent)");
            let table = delulu_wasm::actor_table(&checked.module);
            delulu_wasm::run_main_actors(&wasm, &cfg, &table).map(|(output, report)| {
                if report.dead_actors > 0 || report.dropped_sends > 0 {
                    eprintln!(
                        "actors: {} died; {} message(s) to dead actors dropped",
                        report.dead_actors, report.dropped_sends
                    );
                }
                if opts.on_quiesce_report {
                    eprintln!(
                        "quiesce: {} surviving actor(s), {} turn(s) run",
                        report.surviving_actors, report.total_turns
                    );
                }
                output
            })
        } else {
            delulu_wasm::run_main(&wasm, &cfg)
        };

        // Emit the trace before verdicts (the witness is available even on a fault).
        if let Some(s) = &sink {
            if opts.trace_effects {
                let lines = s.to_json_lines();
                match &opts.trace_out {
                    Some(path) => {
                        if let Err(e) = write_nofollow(path, (lines + "\n").as_bytes()) {
                            eprintln!("error: cannot write trace to `{path}`: {e}");
                        }
                    }
                    None => eprintln!("{lines}"),
                }
            }
        }

        let code = match run_result {
            Ok(output) => {
                print!("{output}");
                0
            }
            Err(e) => {
                let msg = e.message();
                let d = Diagnostic::error(wasm_fault_code(&msg), format!("WASM engine: {msg}"));
                print_diagnostics("run", &[d], &map, None, opts.json);
                1
            }
        };

        // The trace ⊆ row law (spec §6.2, invariant 12): a violation is compiler-bug class (DL1101).
        if opts.assert_trace {
            if let Some(s) = &sink {
                let allowed: std::collections::BTreeSet<String> =
                    main_row.iter().map(|e| e.name().to_string()).collect();
                let violations = delulu_runtime::assert_trace(&allowed, &s.records());
                if !violations.is_empty() {
                    let diags: Vec<Diagnostic> = violations
                        .iter()
                        .map(|v| Diagnostic::error("DL1101", format!("effect-trace assertion violation: {v} — this is a compiler-bug class failure; please report it")))
                        .collect();
                    print_diagnostics("run", &diags, &map, None, opts.json);
                    return 3;
                }
            }
        }
        return code;
    }

    // Determinism knobs (spec §6.2).
    if let Some(seed) = opts.seed {
        delulu_runtime::set_rand_seed(seed);
    }
    if let Some(ms) = opts.clock_ms {
        delulu_runtime::set_fixed_clock_ms(Some(ms));
    }

    // Effect tracing (spec §6.1) — also attached when --assert-trace needs the witness.
    let sink = if opts.assert_trace {
        // `--assert-trace` proves no effect outside the declared set occurred, so it needs EVERY
        // record: a dropped one could hide the violation (C56/D49).
        Some(delulu_runtime::TraceSink::new())
    } else if opts.trace_effects {
        Some(delulu_runtime::TraceSink::bounded(TRACE_RECORD_CAP))
    } else {
        None
    };

    let mut root_val = build_root(&grants);
    // P2 (D-V2-27): the manifest's `[plugins] allow` hash list is attached whether or not
    // `--grant-manifest` was passed, and that is deliberate. `--grant-manifest` accepts the
    // manifest's declared AUTHORITY; this list is a CEILING — it can only narrow which artifacts may
    // load. A restriction that applied only when the operator opted into accepting grants would be a
    // restriction an operator could drop by accident.
    if let Some(m) = &manifest {
        root_val.plugins_allow = m.plugins_allow.clone();
    }
    if daemon_mode {
        // Phase 5g: in daemon mode the program gets opaque broker-secret HANDLES — the byte values
        // live in the broker's store, never in this process pre-`expose` (invariant 23). The grant
        // names select which broker secrets are reachable; any locally-supplied values are DROPPED.
        let mut names: Vec<String> = root_val.secrets.keys().cloned().collect();
        names.sort();
        root_val.broker_secrets = names;
        root_val.secrets.clear();
    }
    let root = Value::Root(Rc::new(root_val));
    let max_ret = opts.foreign_max_ret.unwrap_or(delulu_runtime::foreign::DEFAULT_MAX_RET);
    let mut interp = Interp::new(&checked.module)
        .with_foreign(checked.result.foreign_binds.clone(), grants.foreign_c.clone(), max_ret)
        // P2: the plugin container reader. `delulu-wasm` implements the trait `delulu-runtime`
        // DEFINES, so the runtime cannot reach it and the binary wires it — the same arrangement the
        // foreign binder uses. Unconditional: a program that loads no plugin never touches it, and a
        // program that does must not depend on a flag to have found its reader.
        .with_plugin_engine(Rc::new(delulu_wasm::WasmPluginEngine::new()));
    if foreign_process {
        // Stage 5 phase 5h: run each granted C library in an isolated worker subprocess. Additive —
        // a program with no foreign binds never spawns a worker (the binder is only consulted by
        // `root.foreign(load)`).
        match crate::foreign_worker::WorkerBinder::new() {
            Ok(b) => interp = interp.with_foreign_binder(Rc::new(b)),
            Err(e) => {
                eprintln!("error: cannot set up foreign process isolation (locating the worker executable): {e}");
                return 2;
            }
        }
    }
    // 10g: give each actuator its own child node under this run's, then build the e-stop probe
    // from them BEFORE custody moves into the interpreter. Two reasons for the child nodes rather
    // than watching the run's own node: `grants revoke` on a device stops that device and leaves
    // the program its console to report the loss with (spec §5.2's *subtree*), and revoking the
    // parent still reaches every device transitively — the tree already does that.
    // The device nodes' ids travel separately from the probe so the run can revoke them on the way
    // out; the probe itself has moved into the watchdog thread by then.
    let mut device_nodes: Vec<String> = Vec::new();
    let device_state_dir: Option<std::path::PathBuf> =
        daemon_custody.as_ref().map(|c| c.state_dir().to_path_buf());
    let authority_probe: Option<delulu_runtime::AuthorityProbe> = if grants.actuators.is_empty() {
        None
    } else {
        match daemon_custody.as_mut().map(|c| mint_device_nodes(c, &grants.actuators)) {
            None => None,
            Some(Ok((probe, ids))) => {
                device_nodes = ids;
                Some(probe)
            }
            Some(Err(d)) => {
                // Fail closed: if a device's own node cannot be minted, the e-stop has nothing to
                // aim at, and a run holding a machine with no way to stop it must not start.
                let diag = Diagnostic::error(d.code, d.message);
                print_diagnostics("run", &[diag], &map, None, opts.json);
                return 1;
            }
        }
    };
    if let Some(c) = daemon_custody.take() {
        // Stage 5 phase 5f: route every effectful op through the broker daemon.
        interp = interp.with_custody(Box::new(c));
    } else if !grants.plugins.is_empty() {
        // P2: a run that MAY load a plugin needs a grant tree for the load's step-4 holder check, and
        // in embedded mode there is none by default. The root is the RUN's own authority, derived from
        // what the operator granted — so a plugin's grant attenuates under it and the broker's `⊑`
        // enforces the property that matters: a plugin can never hold more than its host.
        //
        // Only when plugin loading was granted. `EmbeddedCustody::new()` stays the default for every
        // other run, byte-identical to Stage 1–5, because minting a tree for runs that will never use
        // one changes behaviour nobody asked to change.
        let spec = crate::cli::authority_spec_from_grants(&grants, &checked.module.name.dotted());
        let authority = crate::brokerd::spec_to_authority(&spec);
        interp = interp.with_custody(Box::new(delulu_runtime::EmbeddedCustody::with_root(authority)));
    }
    if let Some(s) = &sink {
        interp = interp.with_trace(s.clone());
    }
    // Stage 10 (10f): the device broker starts its dead-man the moment the leases exist, which is
    // BEFORE `main` runs. A program that never reaches its first command still holds a lease it
    // is not beating, and the watchdog treats that exactly like any other silence.
    let devices = if grants.actuators.is_empty() && grants.sensors.is_empty() {
        None
    } else {
        // RFC 0001 dish 3: under `hw:` the driver is a separate process. It is spawned HERE,
        // after the DL1905 approval gate above has already refused an unapproved artifact — so a
        // hardware driver is never started for bytes a human did not sign off on.
        let hw_adapter = match (&device_profile, &opts.adapter_cmd) {
            (delulu_runtime::Profile::Hw { adapter }, Some(cmd)) => {
                let mut parts = cmd.split_whitespace();
                let Some(prog) = parts.next() else {
                    eprintln!("error: --adapter-cmd is empty");
                    return 2;
                };
                let args: Vec<String> = parts.map(str::to_string).collect();
                // Provenance, BEFORE the driver is spawned (D52, closing the gap D23 named).
                let (prov, gate) = check_adapter_signature(
                    prog,
                    opts.adapter_artifact.as_deref(),
                    opts.require_signed_adapter,
                    opts.adapter_signer.as_deref(),
                );
                // Recorded BEFORE the refusal is acted on, so a run stopped because the driver was
                // signed by the wrong key leaves the evidence that it happened (C60).
                if let Err(code) = record_adapter_provenance(&prov, opts.adapter_record.as_deref()) {
                    return code;
                }
                if let Some(code) = gate {
                    return code;
                }
                match delulu_runtime::adapter::ProcessAdapter::spawn(adapter, prog, &args) {
                    Ok(a) => Some(a),
                    Err(e) => {
                        // Fail closed and BEFORE `main`: a run that could not start its driver must
                        // not begin, or the program would discover the machine is unreachable
                        // partway through a motion.
                        eprintln!("error: {e}");
                        return 1;
                    }
                }
            }
            _ => None,
        };
        let b = std::sync::Arc::new(delulu_runtime::DeviceBroker::with_adapter(
            device_profile.clone(),
            &grants.actuators,
            &grants.sensors,
            authority_probe,
            device_clock,
            hw_adapter,
        ));
        interp = interp.with_devices(b.clone());
        Some(b)
    };
    // Stage 10 (10h): the compute broker, on the same gate — no compute grants, no adapter bound,
    // and a dispatch would answer `NoAdapter` rather than a number nobody computed.
    let computes = if grants.computes.is_empty() {
        None
    } else {
        // Kernels are DATA: every artifact is read, its detached signature verified, and its
        // content hashed BEFORE the program starts. A dispatch later resolves against these
        // verified artifacts, never against the grant string — so an artifact that failed
        // verification is not merely reported, it is absent, and nothing can run it.
        let mut broker = delulu_runtime::compute::ComputeBroker::new(&grants.computes);
        for env in &grants.computes {
            let mut arts = Vec::new();
            for (name, path) in &env.kernels {
                match delulu_runtime::compute::load_kernel(name, path, &env.formats) {
                    Ok(a) => arts.push(a),
                    Err(refusal) => {
                        let d = Diagnostic::error(
                            refusal.code(),
                            format!("`{}`: {}", env.device, refusal.message()),
                        );
                        print_diagnostics("run", &[d], &map, None, opts.json);
                        return 1;
                    }
                }
            }
            broker = broker.with_artifacts(&env.device, arts);
        }
        let b = std::sync::Arc::new(broker);
        interp = interp.with_computes(b.clone());
        Some(b)
    };
    // Stage 7 (phase 7g): a module with actors gets the actor system. Gated on declaration —
    // a program with no actors takes the identical path to v0.6, byte for byte.
    let has_actors =
        checked.module.items.iter().any(|it| matches!(it, delulu_syntax::ast::Item::Actor(_)));
    let mut actor_trace: Option<std::sync::Arc<std::sync::Mutex<Vec<delulu_runtime::TraceRecord>>>> = None;
    let actor_system = if has_actors {
        let threads = opts
            .actors_threads
            .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1));
        if sink.is_some() {
            actor_trace = Some(std::sync::Arc::new(std::sync::Mutex::new(Vec::new())));
        }
        let debug_set = opts
            .debug_rcaps
            .then(|| std::sync::Arc::new(checked.result.iso_moves.clone()));
        // Stage 10 (10c): the manifest's `[actors]` defaults. A decl-level `(mailbox = N)`
        // wins per actor; anything but the two known overflow values is warned about rather
        // than silently meaning `block` (the skip branch, made audible).
        let mb_default = manifest.as_ref().and_then(|m| m.actors_mailbox).map(|n| n as usize);
        let mb_overflow = manifest.as_ref().and_then(|m| m.actors_overflow.clone());
        if let Some(o) = &mb_overflow {
            if o != "block" && o != "drop-new" {
                eprintln!(
                    "warning: unknown `[actors] overflow = \"{o}\"` — valid values are \"block\" and \"drop-new\"; using \"block\""
                );
            }
        }
        let system = delulu_runtime::actors::ActorSystem::start_with(
            &checked.module,
            threads,
            opts.on_actor_death_abort,
            actor_trace.clone(),
            debug_set.clone(),
            mb_default,
            mb_overflow.as_deref() == Some("drop-new"),
        );
        // An honest label for a degraded capability (D67). Actor workers reserve a stack sized for
        // the interpreter's documented depth bound; where the host refuses that reservation the
        // worker takes what it can get and lowers its bound to match, so a deep recursion is still
        // DL0905 rather than an abort. Saying so matters because the alternative is a program that
        // works on one machine and reports DL0905 on another with nothing to explain the difference
        // — the silent-fallback shape `ref.rule.portability.isolation-labels-are-honest` forbids.
        if let Some((bytes, depth)) = system.reduced_depth_bound() {
            eprintln!(
                "warning: this host would not reserve a full interpreter stack for the actor \
                 workers ({} MiB granted), so recursion inside a behavior is bounded at {depth} \
                 instead of {} — deeper recursion is DL0905, never a crash",
                bytes / (1024 * 1024),
                delulu_runtime::DEFAULT_MAX_DEPTH
            );
        }
        interp = interp.with_actors(system.host());
        if let Some(d) = debug_set {
            interp = interp.with_debug_rcaps(d);
        }
        Some(system)
    } else {
        None
    };
    note_program_started();
    let run_result = interp.run_main(root);
    // Quiescence exit (spec §6.1): `main` returned AND all mailboxes empty AND no turn
    // running — then report per flags.
    let mut actor_abort = false;
    if let Some(system) = actor_system {
        let report = system.finish();
        // Merge worker-collected causal records into the main sink (seq renumbered to stay
        // strictly increasing; cross-actor order is by globally monotonic turn id).
        if let (Some(collector), Some(s)) = (&actor_trace, &sink) {
            let mut recs = collector.lock().unwrap().clone();
            recs.sort_by_key(|r| r.turn);
            // The sequence continues from what the sink already holds, so this is an offset into a
            // shared log rather than a position in `recs` — `enumerate()` would restart at 0 and
            // silently renumber records that are already written.
            let base = s.len() as u64;
            for (i, mut r) in recs.into_iter().enumerate() {
                r.seq = base + i as u64;
                s.append(r);
            }
        }
        actor_abort = report.aborted;
        if report.dead_actors > 0 || report.dropped_sends > 0 {
            eprintln!(
                "actors: {} died; {} message(s) to dead actors dropped",
                report.dead_actors, report.dropped_sends
            );
        }
        // Stage 10 (10c): overflow drops are never silent — counted and reported whenever they
        // happened, and itemized per actor under `--trace-memory` (spec §3 B3, mailbox half).
        if report.overflow_drops > 0 {
            eprintln!("actors: {} message(s) dropped by mailbox overflow (`drop-new`)", report.overflow_drops);
        }
        if opts.trace_memory {
            eprintln!("trace-memory: {} bounded mailbox(es)", report.mailbox.len());
            for m in &report.mailbox {
                eprintln!(
                    "  {}: bound {}, peak depth {}, overflow drops {}",
                    m.actor, m.bound, m.peak, m.drops
                );
            }
            // 10d: collector accounting — cell COUNTS, deliberately not bytes (build-order D9:
            // a byte figure without a real size walk would be an invented number).
            eprintln!(
                "  cycle collector: {} sweep(s), {} garbage-cycle cell(s) collected",
                report.cycle_runs, report.cycle_collected
            );
        }
        if opts.on_quiesce_report {
            eprintln!(
                "quiesce: {} surviving actor(s), {} turn(s) run",
                report.surviving_actors, report.total_turns
            );
        }
    }
    if actor_abort {
        eprintln!("aborting: an actor died and --on-actor-death abort is set");
        return 1;
    }

    // Stage 10 (10f): stop the watchdog before any verdict is reported, so a run never leaves a
    // thread deciding things about physical devices after the program is over. Then merge its
    // journal into the trace — the watchdog runs on its own thread and `TraceSink` is an `Rc`, so
    // device events arrive here the same way Stage 7's worker records do. They carry `at_ms`
    // because appending them last would otherwise misrepresent when they happened.
    if let Some(b) = &devices {
        b.shutdown();
        let events = b.events();
        if let Some(s) = &sink {
            let base = s.len() as u64;
            for (i, e) in events.iter().enumerate() {
                s.append(delulu_runtime::TraceRecord {
                    seq: base + i as u64,
                    effect: "Actuate".to_string(),
                    op: e.op.clone(),
                    cap_kind: "Actuator".to_string(),
                    detail: Some(format!("[+{} ms] {}", e.at_ms, e.detail)),
                    span: None,
                    ..Default::default()
                });
            }
        }
        // A lost device is never silent, trace or no trace: losing an actuator mid-run is the
        // single most consequential thing that can happen to a program in this language.
        for e in events.iter().filter(|e| e.op == "lease.revoked") {
            eprintln!("devices: {}", e.detail);
        }
        // 10g: and the device's grant node dies with the run that minted it, so `grants list`
        // never offers an operator an arm that nobody holds (see `revoke_device_nodes`).
        if let Some(dir) = &device_state_dir {
            revoke_device_nodes(dir, &device_nodes);
        }
    }

    // Stage 10 (10h): compute telemetry. Every dispatch that reached the adapter is summarised on
    // stderr — how many elements crossed, how long the kernel took — and every refusal is named.
    // A dispatch is foreign code doing work on a device the human paid to bound; the run says what
    // actually happened rather than leaving it to be inferred from a return value.
    if let Some(cb) = &computes {
        let records = cb.records();
        if !records.is_empty() && !opts.json {
            let landed = records.iter().filter(|r| r.refused.is_none()).count();
            let refused = records.len() - landed;
            eprintln!("compute: {landed} dispatch(es) ran, {refused} refused");
            for r in records.iter() {
                if let Some((code, why)) = &r.refused {
                    eprintln!("compute: {code} {}/{}: {why}", r.device, r.kernel);
                }
            }
        }
    }

    // Emit the trace before verdicts, so the witness is available even on a fault.
    if let Some(s) = &sink {
        if opts.trace_effects {
            let lines = s.to_json_lines();
            match &opts.trace_out {
                Some(path) => {
                    if let Err(e) = write_nofollow(path, (lines + "\n").as_bytes()) {
                        eprintln!("error: cannot write trace to `{path}`: {e}");
                    }
                }
                None => eprintln!("{lines}"),
            }
            // A truncated trace says so, with the number withheld and how to get the rest — the
            // same contract D38 gave the diagnostic flood. A trace that silently stopped recording
            // would be worse than one that grew: the reader would conclude the effects stopped.
            if s.dropped() > 0 {
                eprintln!(
                    "note: {} further effect record(s) not traced — `--trace-effects` retains the \
                     first {TRACE_RECORD_CAP} so a long run cannot exhaust memory (campaign C56). \
                     `--assert-trace` is never capped, because it must see every effect to be sound.",
                    s.dropped()
                );
            }
        }
    }

    let code = match run_result {
        Ok(_) => 0,
        Err(fault) => {
            let mut d = Diagnostic::error(fault.code, fault.message.clone());
            if let Some(span) = fault.span {
                d = d.with_span(span, "runtime fault here");
            }
            print_diagnostics("run", &[d], &map, None, opts.json);
            1
        }
    };

    // The trace ⊆ row law (spec §6.2, invariant 12): a violation is compiler-bug class (DL1101)
    // and gets the dedicated exit code 3.
    if opts.assert_trace {
        if let Some(s) = &sink {
            let allowed: std::collections::BTreeSet<String> =
                main_row.iter().map(|e| e.name().to_string()).collect();
            // Stage 7 (invariant 35 executable): actor-attributed records check against the
            // EXECUTING member's row AND the send site's row via the cause chain; main-line
            // records keep the original law byte-for-byte.
            let member_rows: std::collections::HashMap<String, std::collections::BTreeSet<String>> = checked
                .result
                .facts
                .iter()
                .filter(|(k, _)| k.contains('.'))
                .map(|(k, f)| (k.clone(), f.effects.iter().map(|e| e.name().to_string()).collect()))
                .collect();
            let violations = if has_actors {
                delulu_runtime::assert_trace_causal(&allowed, &member_rows, &s.records())
            } else {
                delulu_runtime::assert_trace(&allowed, &s.records())
            };
            if !violations.is_empty() {
                let diags: Vec<Diagnostic> = violations
                    .iter()
                    .map(|v| Diagnostic::error("DL1101", format!("effect-trace assertion violation: {v} — this is a compiler-bug class failure; please report it")))
                    .collect();
                print_diagnostics("run", &diags, &map, None, opts.json);
                return 3;
            }
        }
    }

    // Stage 10 (10f, invariant 48): the sign-off is written only for a run that actually finished
    // clean under the simulator. Approving an artifact whose sim run FAULTED would be the gate
    // certifying the thing it exists to catch.
    if let Some(path) = &opts.signoff {
        if code == 0 {
            if let Err(e) = write_signoff(&file, path, &device_profile) {
                eprintln!("error: {e}");
                return 2;
            }
        } else {
            eprintln!("sign-off withheld: the run did not complete cleanly, so `{path}` was not written");
        }
    }

    code
}
