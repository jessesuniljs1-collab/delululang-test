//! Command dispatch and the four Stage-1 commands (§9.5).

use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;

use delulu_check::{
    authority_report, authority_widened, check_pins, check_program, check_self_authority,
    check_source, check_workspace, compute_lockfile, enforce_semver_law, load_package,
    program_authority, resolve_workspace, verify_locked, Checked, Effect, Lockfile, Program, Row,
    Workspace,
};
use delulu_check::lockfile::api_row_hash;
use delulu_diag::{envelope_to_string, render_human, Diagnostic, SourceMap};
use delulu_runtime::{parse_manifest, Grants, Interp, Value};
use delulu_runtime::value::RootVal;
use delulu_syntax::ast::Item;
use serde_json::{json, Value as Json};

/// Parsed common options.
struct Opts {
    json: bool,
    grants: Vec<String>,
    grant_manifest: bool,
    no_prompt: bool,
    /// `--locked`: refuse any resolution not already pinned in delulu.lock.
    locked: bool,
    /// `--accept-authority <pkg>`: names packages whose authority widening is explicitly accepted.
    accept_authority: Vec<String>,
    /// `--diff <old.lock>`: for `authority`, the previous lockfile to diff against (§8).
    diff: Option<String>,
    /// `--trace-effects`: emit one JSON trace record per effectful operation (spec §6.1).
    trace_effects: bool,
    /// `--trace-out <file>`: write the trace there instead of stderr.
    trace_out: Option<String>,
    /// `--assert-trace`: verify trace ⊆ the checker's row of main; violations are DL1101, exit 3.
    assert_trace: bool,
    /// `--seed <u64>`: deterministic Cap[Rand] (spec §6.2).
    seed: Option<u64>,
    /// `--clock fixed:<ms>`: deterministic Cap[Clock] (spec §6.2).
    clock_ms: Option<i64>,
    /// `--engine wasm|interp`: which execution engine `run` uses (default interp).
    engine: Option<String>,
    /// `--target wasm`: for `build`, emit a `.dwx` WebAssembly artifact from a single source file.
    target: Option<String>,
    /// `-o <path>` / `--out <path>`: output path for `build --target wasm`.
    out: Option<String>,
    /// `--foreign-max-ret <bytes>`: ceiling on a returned foreign string (invariant 21; default
    /// 64 MiB). A return exceeding it is `ForeignErr::BadReturn`, never a truncated silent success.
    foreign_max_ret: Option<usize>,
    /// `--broker embedded|daemon` (Stage 5 phase 5f): where authority lives for this run. Default
    /// embedded (Stage 1–4 behavior, byte-identical — criterion 11); `daemon` routes every effectful
    /// op through the broker daemon (fail closed DL1401 when unreachable, invariant 27).
    broker: Option<String>,
    /// `--epoch-ms <N>` (spec §4.1): the epoch-class snapshot refresh interval in daemon mode.
    /// Clamped to default 50 / ceiling 250 by `broker_client::clamp_epoch_ms`.
    epoch_ms: Option<u64>,
    /// `--foreign-isolation inproc|process` (spec §5 phase 5h): where a granted C library executes.
    /// `inproc` (dev) loads it in the host process (Stage-4 behavior). `process` runs each library in
    /// an isolated worker subprocess (blast-radius containment; a worker crash is DL1409, host
    /// survives). Default: `process` in `--broker daemon` mode, `inproc` otherwise.
    foreign_isolation: Option<String>,
    /// `--isolation none|process|microvm` (Stage 5 phase 5i, spec §6): the run's isolation profile.
    /// `none` (default) = in-process, language + custody enforcement only. `process` = the code
    /// outside the proof (foreign libs) runs in minimum-privilege worker subprocesses; labeled.
    /// `microvm` = Firecracker-class guest — Linux+KVM only; anywhere else it is DL1408 with the
    /// documented, explicitly-weaker fallback (never a silent approximation — playbook trap 8).
    /// The honest capability matrix is spec §6.1.
    isolation: Option<String>,
    /// `--lease <token>` (Stage 5 phase 5j, spec §3.2/§3.3): redeem a delegated lease token and run
    /// under EXACTLY that node's authority — the orchestration payoff: whoever holds a grant
    /// delegates a slice, hands the token over, and this run can acquire nothing outside it.
    lease: Option<String>,
}

fn parse_opts(rest: &[String]) -> (Option<String>, Opts) {
    let mut file = None;
    let mut opts = Opts {
        json: false,
        grants: Vec::new(),
        grant_manifest: false,
        no_prompt: false,
        locked: false,
        accept_authority: Vec::new(),
        diff: None,
        trace_effects: false,
        trace_out: None,
        assert_trace: false,
        seed: None,
        clock_ms: None,
        engine: None,
        target: None,
        out: None,
        foreign_max_ret: None,
        broker: None,
        epoch_ms: None,
        foreign_isolation: None,
        isolation: None,
        lease: None,
    };
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--json" => opts.json = true,
            "--grant-manifest" => opts.grant_manifest = true,
            "--no-prompt" => opts.no_prompt = true,
            "--locked" => opts.locked = true,
            "--accept-authority" => {
                if i + 1 < rest.len() {
                    opts.accept_authority.push(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--accept-authority=") => {
                opts.accept_authority.push(s["--accept-authority=".len()..].to_string())
            }
            "--diff" => {
                if i + 1 < rest.len() {
                    opts.diff = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--diff=") => opts.diff = Some(s["--diff=".len()..].to_string()),
            "--trace-effects" => opts.trace_effects = true,
            "--assert-trace" => opts.assert_trace = true,
            "--trace-out" => {
                if i + 1 < rest.len() {
                    opts.trace_out = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            "--seed" => {
                if i + 1 < rest.len() {
                    opts.seed = rest[i + 1].parse().ok();
                    i += 1;
                }
            }
            "--clock" => {
                if i + 1 < rest.len() {
                    opts.clock_ms = rest[i + 1].strip_prefix("fixed:").and_then(|s| s.parse().ok());
                    i += 1;
                }
            }
            "--engine" => {
                if i + 1 < rest.len() {
                    opts.engine = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--engine=") => opts.engine = Some(s["--engine=".len()..].to_string()),
            "--target" => {
                if i + 1 < rest.len() {
                    opts.target = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--target=") => opts.target = Some(s["--target=".len()..].to_string()),
            "-o" | "--out" => {
                if i + 1 < rest.len() {
                    opts.out = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--out=") => opts.out = Some(s["--out=".len()..].to_string()),
            "--foreign-max-ret" => {
                if i + 1 < rest.len() {
                    opts.foreign_max_ret = rest[i + 1].parse().ok();
                    i += 1;
                }
            }
            s if s.starts_with("--foreign-max-ret=") => {
                opts.foreign_max_ret = s["--foreign-max-ret=".len()..].parse().ok();
            }
            "--broker" => {
                if i + 1 < rest.len() {
                    opts.broker = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--broker=") => opts.broker = Some(s["--broker=".len()..].to_string()),
            "--epoch-ms" => {
                if i + 1 < rest.len() {
                    opts.epoch_ms = rest[i + 1].parse().ok();
                    i += 1;
                }
            }
            s if s.starts_with("--epoch-ms=") => opts.epoch_ms = s["--epoch-ms=".len()..].parse().ok(),
            "--foreign-isolation" => {
                if i + 1 < rest.len() {
                    opts.foreign_isolation = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--foreign-isolation=") => {
                opts.foreign_isolation = Some(s["--foreign-isolation=".len()..].to_string())
            }
            "--isolation" => {
                if i + 1 < rest.len() {
                    opts.isolation = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--isolation=") => {
                opts.isolation = Some(s["--isolation=".len()..].to_string())
            }
            "--lease" => {
                if i + 1 < rest.len() {
                    opts.lease = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--lease=") => opts.lease = Some(s["--lease=".len()..].to_string()),
            "--grant" => {
                if i + 1 < rest.len() {
                    opts.grants.push(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--grant=") => opts.grants.push(s["--grant=".len()..].to_string()),
            s if !s.starts_with('-') && file.is_none() => file = Some(s.to_string()),
            _ => {}
        }
        i += 1;
    }
    (file, opts)
}

pub fn run(args: &[String]) -> i32 {
    let Some(cmd) = args.first() else {
        eprintln!("{}", usage());
        return 2;
    };
    let rest = &args[1..];
    match cmd.as_str() {
        "check" => cmd_check(rest),
        "build" => cmd_build(rest),
        "lock" => cmd_lock(rest),
        "run" => cmd_run(rest),
        "authority" => cmd_authority(rest),
        "why" => cmd_why(rest),
        "repl" => repl_cmd(rest),
        "audit" => cmd_audit(rest),
        "grants" => cmd_grants(rest),
        "broker" => crate::brokerd::cmd_broker(rest),
        // Hidden: the process-isolation foreign worker (spec §5 phase 5h), spawned by the host, not a
        // user-facing command. Loads one granted C library and serves marshalled calls over its pipe.
        s if s == crate::foreign_worker::WORKER_SUBCOMMAND => crate::foreign_worker::run_worker(rest),
        "secrets" => cmd_secrets(rest),
        "explain" => cmd_explain(rest),
        "--help" | "-h" | "help" => {
            println!("{}", usage());
            0
        }
        "--version" | "-V" => {
            println!("delulu {}", env!("CARGO_PKG_VERSION"));
            0
        }
        other => {
            eprintln!("unknown command `{other}`\n\n{}", usage());
            2
        }
    }
}

fn usage() -> &'static str {
    "delulu — the DeluluLang compiler and runtime\n\
     \n\
     USAGE:\n\
     \x20 delulu check     <file.delulu | package-dir> [--json]\n\
     \x20 delulu build     <package-dir> [--locked] [--json]   (resolve deps + verify pins/authority)\n\
     \x20 delulu build     <file.delulu> --target wasm [-o out.dwx]  (emit an authority-carrying .dwx)\n\
     \x20 delulu lock      [package-dir] [--accept-authority <pkg>]... [--json]\n\
     \x20 delulu run       <file.delulu | file.dwx> [--json] [--grant K[=V]]... [--grant-manifest] [--no-prompt]\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--trace-effects] [--trace-out F] [--assert-trace] [--seed N] [--clock fixed:MS]\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--engine wasm]  (run `main` on the WebAssembly backend instead of the interpreter)\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--broker embedded|daemon] [--epoch-ms N]  (custody: daemon routes ops through the broker)\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--lease TOKEN]  (run under a delegated lease — the authority is the delegated node's)\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--isolation none|process|microvm]  (microvm is Linux+KVM; elsewhere DL1408, see spec §6.1)\n\
     \x20 delulu authority <file.delulu | package-dir> [--json]\n\
     \x20 delulu authority --diff <old.lock> <new.lock-or-package-dir> [--json]\n\
     \x20 delulu why       <Effect> <file.delulu | package-dir> [--json]\n\
     \x20 delulu repl      [--grant K[=V]]...\n\
     \x20 delulu audit     tail [N] | query [--node g_ID] [--action A] [--effect E] | verify\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--dir DIR] [--json]  (default DIR: ~/.delulu/audit)\n\
     \x20 delulu broker    start [--foreground] | status | stop | rotate-key [--state-dir DIR]\n\
     \x20 delulu grants    list | tree | inspect <g_ID> | revoke <g_ID>\n\
     \x20 delulu grants    delegate [--parent g_ID] --effects E,.. [--fs-read P].. [--fs-write P]..\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--net H].. [--secret N].. [--declassify N].. [--ttl 1h] [--multi]  (prints a lease token)\n\
     \x20 delulu secrets   set NAME VALUE | list [--state-dir DIR]  (broker-resident secrets)\n\
     \x20 delulu explain   <DLxxxx | E-REVOKE>\n\
     \n\
     `delulu authority` prints the compiler-computed answer to \"what can this program do?\"\n\
     `delulu authority --diff` compares two lockfile states and reports authority widening.\n\
     `delulu why <Effect>` explains, at function granularity, why a program can perform an\n\
     effect — the shortest chain of calls from `main` down to the function that performs it.\n\
     `delulu audit` reads the broker's hash-chained audit log (observability, not enforcement);\n\
     `verify` recomputes the whole chain and reports DL1405 at the first broken record.\n\
     `delulu build` resolves path dependencies and verifies each dependency's authority against\n\
     its pin (DL1001); `delulu lock` writes delulu.lock and enforces the semver-authority law\n\
     (DL1003 — authority never widens silently across versions)."
}

fn load(file: &str) -> Result<(SourceMap, u32, String), i32> {
    match std::fs::read_to_string(file) {
        Ok(src) => {
            let mut map = SourceMap::new();
            let id = map.add_file(file, src.clone());
            Ok((map, id, src))
        }
        Err(e) => {
            eprintln!("error: cannot read `{file}`: {e}");
            Err(2)
        }
    }
}

fn print_diagnostics(command: &str, diags: &[Diagnostic], map: &SourceMap, authority: Option<Json>, json: bool) {
    if json {
        println!("{}", envelope_to_string(command, diags, authority, map));
    } else {
        for d in diags {
            eprint!("{}", render_human(d, map));
            eprintln!();
        }
    }
}

fn errors(diags: &[Diagnostic]) -> usize {
    diags.iter().filter(|d| d.is_error()).count()
}

// ----- check ---------------------------------------------------------------

fn cmd_check(rest: &[String]) -> i32 {
    let (file, opts) = parse_opts(rest);
    let Some(file) = file else {
        eprintln!("error: `check` needs a file or package directory");
        return 2;
    };
    if std::path::Path::new(&file).is_dir() {
        return build_workspace(&file, &opts, "check", opts.locked);
    }
    let (map, id, src) = match load(&file) {
        Ok(x) => x,
        Err(c) => return c,
    };
    let checked = check_source(id, &src);
    let n = errors(&checked.diagnostics);
    print_diagnostics("check", &checked.diagnostics, &map, None, opts.json);
    if !opts.json {
        if n == 0 {
            eprintln!("ok: {} checked clean", file);
        } else {
            eprintln!("{n} error(s)");
        }
    }
    if n == 0 {
        0
    } else {
        1
    }
}

// ----- authority (the flagship) --------------------------------------------

fn cmd_authority(rest: &[String]) -> i32 {
    let (file, opts) = parse_opts(rest);
    if let Some(old_path) = opts.diff.clone() {
        let Some(new_arg) = file else {
            eprintln!("error: `authority --diff` needs <old.lock> and <new.lock-or-package-dir>");
            return 2;
        };
        return cmd_authority_diff(&old_path, &new_arg, &opts);
    }
    let Some(file) = file else {
        eprintln!("error: `authority` needs a file or package directory");
        return 2;
    };
    if std::path::Path::new(&file).is_dir() {
        return authority_package(&file, &opts);
    }
    let (map, id, src) = match load(&file) {
        Ok(x) => x,
        Err(c) => return c,
    };
    let checked = check_source(id, &src);
    if errors(&checked.diagnostics) > 0 {
        print_diagnostics("authority", &checked.diagnostics, &map, None, opts.json);
        return 1;
    }
    let mut scopes = manifest_scopes(&file);
    // Foreign blocks the program declares + its embedded-Python use → the `foreign_calls` array under
    // the "outside the proof" separator (spec §6). A program with no `foreign` blocks and no Python
    // use yields `[]`, keeping the report byte-identical to Stage 3 (criterion 7).
    let python_allowlist = manifest_python_allowlist(&file);
    scopes.foreign_calls = foreign_calls_json(&checked.module, &map, &python_allowlist);
    let program = checked.module.name.dotted();
    let mut report = authority_report(&program, &checked.result, &scopes);
    stamp_custody(&mut report, &opts);
    stamp_foreign_isolation(&mut report, &opts);
    stamp_isolation(&mut report, &opts);
    if opts.json {
        println!("{}", envelope_to_string("authority", &[], Some(report), &map));
    } else {
        print!("{}", render_authority(&report));
    }
    0
}

/// Stamp the custody label on an authority report (Stage 5, playbook 5j): `embedded` unless the
/// caller passed `--broker daemon`. Additive JSON key; the human render prints it as a line.
fn stamp_custody(report: &mut Json, opts: &Opts) {
    let custody = if opts.broker.as_deref() == Some("daemon") { "daemon" } else { "embedded" };
    if let Some(obj) = report.as_object_mut() {
        obj.insert("custody".to_string(), json!(custody));
    }
}

/// Stamp the foreign-isolation label on an authority report (Stage 5 phase 5h, spec §5): the mode
/// that WOULD apply when this program runs — `process` if `--foreign-isolation process` (or daemon
/// mode's default), else `inproc`. Additive JSON key; the human render prints it only inside a
/// non-empty foreign section, so a no-foreign report stays byte-identical (criterion 7).
fn stamp_foreign_isolation(report: &mut Json, opts: &Opts) {
    let daemon = opts.broker.as_deref() == Some("daemon");
    let mode = match opts.foreign_isolation.as_deref() {
        Some("process") => "process",
        Some("inproc") => "inproc",
        _ => {
            if daemon {
                "process"
            } else {
                "inproc"
            }
        }
    };
    if let Some(obj) = report.as_object_mut() {
        obj.insert("foreign_isolation".to_string(), json!(mode));
    }
}

/// Why `--isolation microvm` is unavailable here (`Err(detail)`), or `Ok(())` once the Linux+KVM
/// guest launch exists. v0.5: always `Err` — on Linux the probe names the first missing
/// prerequisite (or the pending launch work); everywhere else it is a platform refusal. Honest by
/// construction (trap 8): there is no code path that quietly substitutes weaker isolation.
#[cfg(target_os = "linux")]
fn microvm_unavailable() -> Result<(), String> {
    crate::microvm::probe()
}

#[cfg(not(target_os = "linux"))]
fn microvm_unavailable() -> Result<(), String> {
    Err(format!(
        "`--isolation microvm` requires Linux x86_64/aarch64 with KVM; this platform is `{}`",
        std::env::consts::OS
    ))
}

/// Stamp the requested isolation profile on an authority report (Stage 5 phase 5i, spec §6) —
/// only when `--isolation` was passed, so a default report stays byte-identical to prior stages
/// (criterion 11). An unattainable `microvm` request is labeled unavailable, never affirmed.
fn stamp_isolation(report: &mut Json, opts: &Opts) {
    let Some(iso) = &opts.isolation else { return };
    let label = if iso == "microvm" && microvm_unavailable().is_err() {
        "microvm (unavailable here — a run refuses with DL1408; see spec §6.1)".to_string()
    } else {
        iso.clone()
    };
    if let Some(obj) = report.as_object_mut() {
        obj.insert("isolation".to_string(), json!(label));
    }
}

fn strs(v: &Json) -> Vec<String> {
    v.as_array()
        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

fn render_authority(report: &Json) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let name = report["program"].as_str().unwrap_or("?");
    let _ = writeln!(out, "Authority of `{name}` — what this program can do to your system:");
    let modules = strs(&report["modules"]);
    if !modules.is_empty() {
        let _ = writeln!(out, "  modules:      {}", modules.join(", "));
    }
    let effects = strs(&report["effects"]);
    let _ = writeln!(out, "  effects:      {}", if effects.is_empty() { "(none — provably pure)".to_string() } else { effects.join(", ") });
    let caps = report["capabilities"].as_array().cloned().unwrap_or_default();
    if caps.is_empty() {
        let _ = writeln!(out, "  capabilities: (none)");
    } else {
        let _ = writeln!(out, "  capabilities:");
        for c in &caps {
            let kind = c["kind"].as_str().unwrap_or("?");
            let scopes = strs(&c["scopes"]);
            let scope_str = if scopes.is_empty() { "(scope granted at runtime)".to_string() } else { scopes.join(", ") };
            let _ = writeln!(out, "    - {kind:<8} {scope_str}");
        }
    }
    let secrets = strs(&report["secrets"]);
    let _ = writeln!(out, "  secrets:      {}", if secrets.is_empty() { "(none)".to_string() } else { secrets.join(", ") });
    let pure = strs(&report["pure_functions"]);
    let _ = writeln!(out, "  pure fns:     {}", if pure.is_empty() { "(none)".to_string() } else { pure.join(", ") });
    // Foreign code lives OUTSIDE the effect proof. With none declared, the report is byte-identical
    // to Stage 3 (criterion 7); with foreign blocks, they list under the mandated separator line.
    // The custody label (Stage 5): where authority lives when this program runs. The JSON report
    // always carries it (criterion 11: "custody label present in reports"); the HUMAN render shows
    // it only when custody is non-default (daemon), so the default embedded report stays
    // byte-identical to Stages 1–4 (criterion 11's no-regression half, matching the run path).
    if let Some(custody) = report["custody"].as_str() {
        if custody != "embedded" {
            let _ = writeln!(out, "  custody:      {custody}");
        }
    }
    // The isolation profile (Stage 5 phase 5i): present only when explicitly requested, so the
    // default report is byte-identical to prior stages (criterion 11).
    if let Some(iso) = report["isolation"].as_str() {
        let _ = writeln!(out, "  isolation:    {iso}");
    }
    let foreign = report["foreign_calls"].as_array().cloned().unwrap_or_default();
    if foreign.is_empty() {
        let _ = writeln!(out, "  foreign:      (none — no code outside the guarantee)");
    } else {
        let _ = writeln!(out, "  foreign:");
        let _ = writeln!(out, "    -- outside the proof (contained at process level) --");
        // Stage 5 phase 5h: which isolation profile bounds these foreign libraries' blast radius when
        // the program runs (spec §5). `process` = each library in an isolated worker subprocess (a
        // crash is DL1409, the host survives); `inproc` = Stage-4 in-process (a crash takes the host).
        let iso = report["foreign_isolation"].as_str().unwrap_or("inproc");
        let _ = writeln!(out, "    isolation: {iso}");
        for f in &foreign {
            let abi = f["abi"].as_str().unwrap_or("c");
            if abi == "python" {
                // The embedded-Python entry: the import allowlist (interface gate, §5.3) and the
                // statically-seen imports — no lib/symbols/binary.
                let allow = strs(&f["allowlist"]);
                let seen = strs(&f["imports_seen"]);
                let allow_str = if allow.is_empty() { "(none granted)".to_string() } else { allow.join(", ") };
                let seen_str = if seen.is_empty() { "(none seen statically)".to_string() } else { seen.join(", ") };
                let _ = writeln!(out, "    - python  allowlist: [{allow_str}]  imports seen: [{seen_str}]");
                continue;
            }
            let lib = f["lib"].as_str().unwrap_or("?");
            let symbols = strs(&f["symbols"]);
            let binary = f["granted_path"].as_str().unwrap_or("(chosen by the human at grant time)");
            let _ = writeln!(out, "    - {abi} {lib} [{}]  binary: {binary}", symbols.join(", "));
        }
    }
    out
}

fn manifest_scopes(file: &str) -> delulu_check::ScopeInfo {
    let dir = std::path::Path::new(file).parent().unwrap_or_else(|| std::path::Path::new("."));
    scopes_in_dir(dir)
}

fn scopes_in_dir(dir: &std::path::Path) -> delulu_check::ScopeInfo {
    match std::fs::read_to_string(dir.join("delulu.toml")) {
        Ok(src) => parse_manifest(&src).scope_info(),
        Err(_) => delulu_check::ScopeInfo::default(),
    }
}

/// The manifest's declared `foreign.python` import allowlist next to `file` (empty when no manifest).
fn manifest_python_allowlist(file: &str) -> Vec<String> {
    let dir = std::path::Path::new(file).parent().unwrap_or_else(|| std::path::Path::new("."));
    match std::fs::read_to_string(dir.join("delulu.toml")) {
        Ok(src) => parse_manifest(&src).foreign_python,
        Err(_) => Vec::new(),
    }
}

// ----- package mode (multi-module, Stage 2) --------------------------------

fn cmd_build(rest: &[String]) -> i32 {
    let (path, opts) = parse_opts(rest);
    let Some(path) = path else {
        eprintln!("error: `build` needs a source file (`--target wasm`) or a package directory");
        return 2;
    };
    // `build --target wasm <file.delulu>` emits a `.dwx` artifact (Stage 3 §5).
    if opts.target.as_deref() == Some("wasm") {
        return build_wasm_artifact(&path, &opts);
    }
    if let Some(t) = &opts.target {
        eprintln!("error: unknown --target `{t}` (only `wasm` is supported)");
        return 2;
    }
    if !std::path::Path::new(&path).is_dir() {
        eprintln!("error: `build` expects a package directory (with src/ and delulu.toml)");
        return 2;
    }
    build_workspace(&path, &opts, "build", opts.locked)
}

/// `build --target wasm <file.delulu>`: check the program, compile `main` to WebAssembly, and write
/// a `.dwx` artifact with the compiler-computed authority embedded as a `delulu:authority` custom
/// section (hash-bound to the code). This is the single, self-describing, authority-carrying
/// artifact: `delulu run <file>.dwx` re-verifies it before running.
fn build_wasm_artifact(file: &str, opts: &Opts) -> i32 {
    let (map, id, src) = match load(file) {
        Ok(x) => x,
        Err(c) => return c,
    };
    let checked = check_source(id, &src);
    if errors(&checked.diagnostics) > 0 {
        print_diagnostics("build", &checked.diagnostics, &map, None, opts.json);
        return 1;
    }
    if !checked.result.main_present {
        let d = Diagnostic::error("DL1201", format!("`{file}` has no `fn main(root: Root)` — a `.dwx` artifact needs an entry point to run"));
        print_diagnostics("build", &[d], &map, None, opts.json);
        return 1;
    }
    let wasm = match delulu_wasm::compile_module_with(&checked.module, &checked.result.foreign_binds) {
        Ok(w) => w,
        Err(e) => {
            let d = Diagnostic::error(e.code(), format!("{} — this program can't be built to a `.dwx` yet (run it on the interpreter)", e.message()));
            print_diagnostics("build", &[d], &map, None, opts.json);
            return 1;
        }
    };
    // Embed the same authority answer `delulu authority` reports (incl. the `foreign_calls` entries),
    // so the artifact carries its own truthful manifest of what it can do — a foreign-using program's
    // manifest lists its foreign entries under the "outside the proof" separator (Stage 4 phase 4g).
    let mut scopes = manifest_scopes(file);
    let python_allowlist = manifest_python_allowlist(file);
    scopes.foreign_calls = foreign_calls_json(&checked.module, &map, &python_allowlist);
    let program = checked.module.name.dotted();
    let authority = authority_report(&program, &checked.result, &scopes);
    let dwx = delulu_wasm::embed_authority(&wasm, &authority);

    let out_path = opts.out.clone().unwrap_or_else(|| {
        std::path::Path::new(file).with_extension("dwx").to_string_lossy().to_string()
    });
    if let Err(e) = std::fs::write(&out_path, &dwx) {
        eprintln!("error: could not write `{out_path}`: {e}");
        return 2;
    }
    if opts.json {
        let report = json!({ "artifact": out_path, "bytes": dwx.len(), "authority": authority });
        println!("{}", envelope_to_string("build", &[], Some(report), &map));
    } else {
        eprintln!(
            "ok: wrote `{out_path}` ({} bytes) with authority embedded as `{}`",
            dwx.len(),
            delulu_wasm::AUTHORITY_SECTION
        );
    }
    0
}

/// Resolve the workspace (root + path dependencies), check every package under one global type
/// registry, and verify the whole-graph authority story: DL1009 (package exceeds its own
/// manifest), DL1001 (a dependency exceeds its pin), and — under `--locked` — DL1010/DL1002/DL1011
/// against `delulu.lock`. This is the compile-time supply-chain gate: it runs BEFORE any code.
fn build_workspace(dir: &str, opts: &Opts, command: &str, locked: bool) -> i32 {
    let ws = resolve_workspace(dir);
    let program = check_workspace(&ws);

    let mut diags: Vec<Diagnostic> = Vec::new();
    diags.extend(ws.diagnostics.iter().cloned());
    diags.extend(program.diagnostics.iter().cloned());
    diags.extend(check_self_authority(&ws, &program));
    diags.extend(check_pins(&ws, &program));

    if locked {
        match std::fs::read_to_string(std::path::Path::new(dir).join("delulu.lock")) {
            Ok(text) => {
                let lock = Lockfile::parse(&text);
                diags.extend(verify_locked(&ws, &program, &lock));
            }
            Err(_) => {
                for pkg in &ws.packages {
                    diags.push(Diagnostic::error(
                        "DL1011",
                        format!("`--locked` build but delulu.lock is missing — package `{}` is unresolved; run `delulu lock`", pkg.name),
                    ));
                }
            }
        }
    }

    let n = errors(&diags);
    print_diagnostics(command, &diags, &ws.source_map, None, opts.json);
    // Git dependencies are parsed and rev/tag-validated, but fetching is deferred to a later stage.
    // We refuse rather than pretend a git dependency's authority was verified.
    let git_blocked = !ws.git_deferred.is_empty();
    if git_blocked && !opts.json {
        eprintln!(
            "note: git dependency resolution is deferred to a later stage; cannot verify authority for: {}",
            ws.git_deferred.join(", ")
        );
    }
    let failed = n > 0 || git_blocked;
    // §5.5: `interface.json` is a build artifact, not a check artifact — only `build` writes it,
    // and only after a clean whole-graph check (never on a failed/diagnostic-bearing build).
    if command == "build" && !failed {
        write_interfaces(&ws, &program);
    }
    if !opts.json {
        if !failed {
            eprintln!(
                "ok: `{}` built clean ({} package(s), {} module(s); authority within manifest and pins)",
                ws.root_pkg().name,
                ws.packages.len(),
                ws.modules.len()
            );
        } else {
            eprintln!("{n} error(s)");
        }
    }
    if failed {
        1
    } else {
        0
    }
}

/// §5.5: write `<pkgdir>/target/<pkg>/interface.json` for every package in the workspace (root +
/// path dependencies) — its `pub` surface (name, full type, effect row) plus the same
/// `api_row_hash` the lockfile carries, so an agent can introspect a dependency without reading
/// its source. The spec text frames this as "per lib package"; this writes it for every package
/// regardless of `kind`, since a `bin` package's `pub` fns are equally legitimate machine surface
/// and the extra file is harmless when there are none (`exports: []`).
fn write_interfaces(ws: &Workspace, program: &Program) {
    for (idx, pkg) in ws.packages.iter().enumerate() {
        let mut exports: Vec<Json> = Vec::new();
        for wm in ws.modules.iter().filter(|m| m.pkg == idx) {
            for item in &wm.unit.module.items {
                if let Item::Fn(f) = item {
                    if !f.public {
                        continue;
                    }
                    let key = format!("{}::{}", wm.unit.name, f.name.name);
                    let ty = program.fn_types.get(&key).map(|t| t.to_string()).unwrap_or_default();
                    let row = program
                        .facts
                        .get(&key)
                        .map(|f| Row::closed(f.effects.clone()).to_string())
                        .unwrap_or_else(|| Row::pure().to_string());
                    exports.push(json!({ "name": f.name.name, "type": ty, "row": row }));
                }
            }
        }
        exports.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        let doc = json!({
            "package": pkg.name,
            "version": pkg.manifest.version,
            "exports": exports,
            "api_row_hash": api_row_hash(ws, idx),
        });
        let out_dir = pkg.dir.join("target").join(&pkg.name);
        if let Err(e) = std::fs::create_dir_all(&out_dir) {
            eprintln!("warning: cannot create `{}`: {e}", out_dir.display());
            continue;
        }
        match serde_json::to_string_pretty(&doc) {
            Ok(text) => {
                let out_file = out_dir.join("interface.json");
                if let Err(e) = std::fs::write(&out_file, text) {
                    eprintln!("warning: cannot write `{}`: {e}", out_file.display());
                }
            }
            Err(e) => eprintln!("warning: cannot serialize interface.json for `{}`: {e}", pkg.name),
        }
    }
}

/// `delulu lock`: (re)compute delulu.lock next to the root manifest and enforce the
/// semver-authority law (DL1003) against the previous lock.
fn cmd_lock(rest: &[String]) -> i32 {
    let (path, opts) = parse_opts(rest);
    let dir = path.unwrap_or_else(|| ".".to_string());
    if !std::path::Path::new(&dir).is_dir() {
        eprintln!("error: `lock` expects a package directory (with src/ and delulu.toml)");
        return 2;
    }

    let ws = resolve_workspace(&dir);
    let program = check_workspace(&ws);
    let mut diags: Vec<Diagnostic> = Vec::new();
    diags.extend(ws.diagnostics.iter().cloned());
    diags.extend(program.diagnostics.iter().cloned());
    diags.extend(check_self_authority(&ws, &program));
    diags.extend(check_pins(&ws, &program));
    if !ws.git_deferred.is_empty() {
        diags.push(Diagnostic::error(
            "DL1007",
            format!("cannot lock: git dependency resolution is deferred ({})", ws.git_deferred.join(", ")),
        ));
    }
    if errors(&diags) > 0 {
        print_diagnostics("lock", &diags, &ws.source_map, None, opts.json);
        if !opts.json {
            eprintln!("{} error(s) — refusing to write delulu.lock", errors(&diags));
        }
        return 1;
    }

    let mut newlock = compute_lockfile(&ws, &program);
    let lock_path = std::path::Path::new(&dir).join("delulu.lock");
    if let Ok(text) = std::fs::read_to_string(&lock_path) {
        let old = Lockfile::parse(&text);
        let law = enforce_semver_law(&old, &mut newlock, &opts.accept_authority);
        if !law.is_empty() {
            print_diagnostics("lock", &law, &ws.source_map, None, opts.json);
            if !opts.json {
                eprintln!("semver-authority law violated — refusing to write delulu.lock");
            }
            return 1;
        }
    }
    if let Err(e) = std::fs::write(&lock_path, newlock.render()) {
        eprintln!("error: cannot write {}: {e}", lock_path.display());
        return 2;
    }
    if opts.json {
        println!("{}", envelope_to_string("lock", &[], None, &ws.source_map));
    } else {
        eprintln!("ok: wrote {} ({} package(s))", lock_path.display(), newlock.packages.len());
    }
    0
}

fn authority_package(dir: &str, opts: &Opts) -> i32 {
    let pkg = load_package(dir);
    let program = check_program(&pkg);
    if errors(&program.diagnostics) > 0 {
        print_diagnostics("authority", &program.diagnostics, &pkg.source_map, None, opts.json);
        return 1;
    }
    let name = program.entry_module.clone().unwrap_or_else(|| "package".to_string());
    let scopes = scopes_in_dir(std::path::Path::new(dir));
    let mut report = program_authority(&program, &name, &scopes);
    stamp_custody(&mut report, opts);
    stamp_foreign_isolation(&mut report, opts);
    stamp_isolation(&mut report, opts);
    if opts.json {
        println!("{}", envelope_to_string("authority", &[], Some(report), &pkg.source_map));
    } else {
        print!("{}", render_authority(&report));
    }
    0
}

// ----- authority --diff (§8: the supply-chain review surface) --------------

/// `delulu authority --diff <old.lock> <new.lock-or-dir>`: compare two authority *states*.
/// `old_path` is always a `delulu.lock`; `new_arg` is either another lockfile or a package
/// directory to resolve+check+lock fresh (so a maintainer can diff "what's locked" against
/// "what HEAD would lock" without writing a file first).
fn cmd_authority_diff(old_path: &str, new_arg: &str, opts: &Opts) -> i32 {
    let old_text = match std::fs::read_to_string(old_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read `{old_path}`: {e}");
            return 2;
        }
    };
    let old_lock = Lockfile::parse(&old_text);

    let new_lock = if std::path::Path::new(new_arg).is_dir() {
        let ws = resolve_workspace(new_arg);
        let program = check_workspace(&ws);
        let mut diags: Vec<Diagnostic> = ws.diagnostics.clone();
        diags.extend(program.diagnostics.iter().cloned());
        if errors(&diags) > 0 {
            print_diagnostics("authority", &diags, &ws.source_map, None, opts.json);
            return 1;
        }
        compute_lockfile(&ws, &program)
    } else {
        let text = match std::fs::read_to_string(new_arg) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: cannot read `{new_arg}`: {e}");
                return 2;
            }
        };
        Lockfile::parse(&text)
    };

    // Union of every package name mentioned on either side, so additions/removals surface too.
    let mut names: BTreeSet<String> = BTreeSet::new();
    names.extend(old_lock.packages.iter().map(|p| p.name.clone()));
    names.extend(new_lock.packages.iter().map(|p| p.name.clone()));

    let mut added_effects = serde_json::Map::new();
    let mut removed_effects = serde_json::Map::new();
    let mut added_scopes = serde_json::Map::new();
    let mut api_row_changes = serde_json::Map::new();
    let mut widened = false;

    for name in &names {
        // A package missing on one side diffs against an empty (`Default`) entry: a package
        // that disappeared loses authority (never a widening); a brand-new package's authority
        // is entirely "added".
        let old_e = old_lock.get(name).cloned().unwrap_or_default();
        let new_e = new_lock.get(name).cloned().unwrap_or_default();

        let old_fx: BTreeSet<&str> = old_e.effects.iter().map(String::as_str).collect();
        let new_fx: BTreeSet<&str> = new_e.effects.iter().map(String::as_str).collect();
        let added: Vec<&str> = new_fx.difference(&old_fx).copied().collect();
        let removed: Vec<&str> = old_fx.difference(&new_fx).copied().collect();
        if !added.is_empty() {
            added_effects.insert(name.clone(), json!(added));
        }
        if !removed.is_empty() {
            removed_effects.insert(name.clone(), json!(removed));
        }

        let mut scope_obj = serde_json::Map::new();
        added_scope_field(&mut scope_obj, "cap_kinds", &old_e.cap_kinds, &new_e.cap_kinds);
        added_scope_field(&mut scope_obj, "net", &old_e.net, &new_e.net);
        added_scope_field(&mut scope_obj, "fs.read", &old_e.fs_read, &new_e.fs_read);
        added_scope_field(&mut scope_obj, "fs.write", &old_e.fs_write, &new_e.fs_write);
        if !scope_obj.is_empty() {
            added_scopes.insert(name.clone(), Json::Object(scope_obj));
        }

        if old_e.api_row_hash != new_e.api_row_hash {
            api_row_changes.insert(
                name.clone(),
                json!({ "old_hash": old_e.api_row_hash, "new_hash": new_e.api_row_hash }),
            );
        }

        // Reuses the same subset check the semver-authority law enforces (lockfile.rs) — a
        // package missing from one side compares against a `Default` (empty-authority) entry.
        if authority_widened(&old_e, &new_e) {
            widened = true;
        }
    }

    let verdict = if widened {
        "WIDENING — requires major version + --accept-authority"
    } else {
        "OK"
    };

    let report = json!({
        "added_effects": added_effects,
        "removed": removed_effects,
        "added_scopes": added_scopes,
        "api_row_changes": api_row_changes,
        "verdict": verdict,
    });

    if opts.json {
        println!("{}", serde_json::to_string_pretty(&report).expect("diff report serializes"));
    } else {
        println!("Authority diff: {old_path} -> {new_arg}");
        if added_effects.is_empty() && removed_effects.is_empty() && added_scopes.is_empty() && api_row_changes.is_empty() {
            println!("  (no authority or public-API changes)");
        } else {
            for (pkg, v) in &added_effects {
                println!("  {pkg}: + effects {}", strs(v).join(", "));
            }
            for (pkg, v) in &removed_effects {
                println!("  {pkg}: - effects {}", strs(v).join(", "));
            }
            for (pkg, v) in &added_scopes {
                println!("  {pkg}: + scopes {v}");
            }
            for (pkg, v) in &api_row_changes {
                println!(
                    "  {pkg}: api row changed ({} -> {})",
                    v["old_hash"].as_str().unwrap_or("?"),
                    v["new_hash"].as_str().unwrap_or("?")
                );
            }
        }
        println!("verdict: {verdict}");
    }

    0
}

fn added_scope_field(obj: &mut serde_json::Map<String, Json>, key: &str, old: &[String], new: &[String]) {
    let old_set: BTreeSet<&str> = old.iter().map(String::as_str).collect();
    let added: Vec<&str> = new.iter().map(String::as_str).filter(|s| !old_set.contains(s)).collect();
    if !added.is_empty() {
        obj.insert(key.to_string(), json!(added));
    }
}

// ----- why -------------------------------------------------------------------------------------

/// `delulu why <Effect> <file-or-dir>`: explain, at FUNCTION granularity, why a program can
/// perform an effect (§8). This is an honest approximation, not the spec's full "op with that
/// effect" story:
///
/// - It names the deepest *function* that originates the effect (the one where the effect is
///   introduced by a capability operation rather than by calling something else that has it),
///   not the specific call-site/operation within that function's body. `FnFacts` records a
///   function's row, not a per-statement trace, so finer granularity isn't available from these
///   facts without walking the body AST again.
/// - It reports exactly ONE origin path (the first one found, in `BTreeSet` — i.e. lexical —
///   callee order), not "all minimal paths, ≤ 10" as §8 literally asks for. A same-effect cycle
///   among callees is cut by the `visited` set so the walk always terminates.
///
/// Algorithm: resolve/check the program; if `main`'s declared row doesn't contain the effect,
/// report "cannot perform" (exit 0). Otherwise walk from `main`: at each step, look at the
/// current function's callees (resolved to their owning module via `call_owner`); descend into
/// the first one whose row still contains the effect; stop when none do — that function performs
/// it directly.
fn cmd_why(rest: &[String]) -> i32 {
    // `why` takes two positionals (`<Effect> <file-or-dir>`) where every other command takes one;
    // peel the effect name off the front and hand the rest to the shared `parse_opts`.
    let mut effect_name: Option<String> = None;
    let mut remainder: Vec<String> = Vec::new();
    for a in rest {
        if effect_name.is_none() && !a.starts_with('-') {
            effect_name = Some(a.clone());
        } else {
            remainder.push(a.clone());
        }
    }
    let Some(effect_name) = effect_name else {
        eprintln!("error: `why` needs an effect name and a file or package directory, e.g. `delulu why Net examples/greeter`");
        return 2;
    };
    let (path, opts) = parse_opts(&remainder);
    let Some(path) = path else {
        eprintln!("error: `why` needs a file or package directory");
        return 2;
    };

    let (diags, program, locations, map): (Vec<Diagnostic>, Program, HashMap<String, (String, u32)>, SourceMap) =
        if std::path::Path::new(&path).is_dir() {
            let ws = resolve_workspace(&path);
            let program = check_workspace(&ws);
            let mut diags: Vec<Diagnostic> = ws.diagnostics.clone();
            diags.extend(program.diagnostics.iter().cloned());
            let locations = locations_workspace(&ws);
            (diags, program, locations, ws.source_map)
        } else {
            let (map, id, src) = match load(&path) {
                Ok(x) => x,
                Err(c) => return c,
            };
            let checked = check_source(id, &src);
            let locations = locations_single(&checked, &map);
            let diags = checked.diagnostics.clone();
            let program = synth_single_program(&checked);
            (diags, program, locations, map)
        };

    if errors(&diags) > 0 {
        print_diagnostics("why", &diags, &map, None, opts.json);
        return 1;
    }

    // A "known" effect is a core primitive, or a user-declared `effect` that appears anywhere in
    // this program's facts (reachable or not — a program can ask "why" about a dead branch too).
    let is_known = Effect::core_from_name(&effect_name).is_some()
        || program.facts.values().any(|f| f.effects.iter().any(|e| e.name() == effect_name));
    if !is_known {
        eprintln!(
            "error: `{effect_name}` is not a known effect (core effects: Read, Write, Net, Clock, Rand, Declassify, ForeignCall; \
             or a user-declared `effect` visible in this program)"
        );
        return 2;
    }

    let Some(entry_mod) = program.entry_module.clone() else {
        eprintln!("error: `why` needs an executable entry point (`fn main`); none was found in `{path}`");
        return 2;
    };
    let main_key = format!("{entry_mod}::main");
    let Some(main_facts) = program.facts.get(&main_key) else {
        eprintln!("error: no `main` function found in `{path}`");
        return 2;
    };

    if !main_facts.effects.iter().any(|e| e.name() == effect_name) {
        if opts.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({ "effect": effect_name, "performs": false, "path": [] }))
                    .expect("why report serializes")
            );
        } else {
            println!("program cannot perform `{effect_name}`");
        }
        return 0;
    }

    // ----- the origin walk ---------------------------------------------------------------------
    let mut path_nodes = vec![main_key.clone()];
    let mut visited: std::collections::HashSet<String> = std::iter::once(main_key.clone()).collect();
    let mut current = main_key;
    loop {
        let Some((cur_mod, _)) = current.split_once("::") else { break };
        let Some(facts) = program.facts.get(&current) else { break };
        let mut next = None;
        for callee in &facts.callees {
            let Some(owner) = program.call_owner.get(cur_mod).and_then(|o| o.get(callee)) else { continue };
            let key = format!("{owner}::{callee}");
            if visited.contains(&key) {
                continue;
            }
            if let Some(cf) = program.facts.get(&key) {
                if cf.effects.iter().any(|e| e.name() == effect_name) {
                    next = Some(key);
                    break;
                }
            }
        }
        match next {
            Some(k) => {
                visited.insert(k.clone());
                path_nodes.push(k.clone());
                current = k;
            }
            None => break,
        }
    }

    if opts.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({ "effect": effect_name, "performs": true, "path": path_nodes }))
                .expect("why report serializes")
        );
    } else {
        let mut out = String::new();
        for (i, key) in path_nodes.iter().enumerate() {
            let fname = key.split_once("::").map(|(_, f)| f).unwrap_or(key.as_str());
            if i > 0 {
                out.push_str(" -> ");
            }
            out.push_str(fname);
            if let Some((file, line)) = locations.get(key) {
                out.push_str(&format!(" ({file}:{line})"));
            }
        }
        out.push_str(&format!(" — {effect_name}"));
        println!("{out}");
    }
    0
}

/// Function-declaration locations (display file + 1-based line) across a whole workspace,
/// keyed `"module::fn"` — the same key shape as `Program::facts`.
fn locations_workspace(ws: &Workspace) -> HashMap<String, (String, u32)> {
    let mut out = HashMap::new();
    for wm in &ws.modules {
        for item in &wm.unit.module.items {
            if let Item::Fn(f) = item {
                let (line, _col) = ws.source_map.position(f.name.span.file, f.name.span.start);
                out.insert(format!("{}::{}", wm.unit.name, f.name.name), (ws.source_map.name(f.name.span.file).to_string(), line));
            }
        }
    }
    out
}

/// Same as [`locations_workspace`], for a single checked file (one implicit module).
fn locations_single(checked: &Checked, map: &SourceMap) -> HashMap<String, (String, u32)> {
    let mod_name = checked.module.name.dotted();
    let mut out = HashMap::new();
    for item in &checked.module.items {
        if let Item::Fn(f) = item {
            let (line, _col) = map.position(f.name.span.file, f.name.span.start);
            out.insert(format!("{mod_name}::{}", f.name.name), (map.name(f.name.span.file).to_string(), line));
        }
    }
    out
}

/// Build a `Program`-shaped view of one checked file so `why` can walk it exactly like a
/// multi-module package. There is no cross-module structure for a lone file — the whole file is
/// treated as its own module (named by its `module` header), and `call_owner` maps every
/// declared top-level function back to that one module. `Program`'s fields are all `pub`, so this
/// is a plain struct literal — no new API surface needed in `delulu-check`.
fn synth_single_program(checked: &Checked) -> Program {
    let mod_name = checked.module.name.dotted();
    let mut facts = HashMap::new();
    let mut fn_types = HashMap::new();
    for (name, f) in &checked.result.facts {
        facts.insert(format!("{mod_name}::{name}"), f.clone());
    }
    for (name, t) in &checked.result.fn_types {
        fn_types.insert(format!("{mod_name}::{name}"), t.clone());
    }
    let mut owner = HashMap::new();
    for name in checked.table.fns.keys() {
        owner.insert(name.clone(), mod_name.clone());
    }
    let mut call_owner = HashMap::new();
    call_owner.insert(mod_name.clone(), owner);
    let entry_module = if checked.result.main_present { Some(mod_name) } else { None };
    Program { diagnostics: checked.diagnostics.clone(), facts, fn_types, call_owner, entry_module }
}

// ----- run -----------------------------------------------------------------

/// Map a WASM host refusal message to the diagnostic code it carries. The host tags refusals with
/// their code inline (DL0703 denied root slice, DL0903 out-of-bounds memory access, DL140x custody
/// denials from the Stage-5 broker seam); anything else is a capability-scope violation (DL0904).
fn wasm_fault_code(msg: &str) -> &'static str {
    for code in ["DL0703", "DL0903", "DL1306", "DL1401", "DL1402", "DL1403"] {
        if msg.contains(code) {
            // The registry stores codes as &'static str; return the matching literal.
            return match code {
                "DL0703" => "DL0703",
                "DL0903" => "DL0903",
                "DL1306" => "DL1306",
                "DL1401" => "DL1401",
                "DL1402" => "DL1402",
                _ => "DL1403",
            };
        }
    }
    "DL0904"
}

/// Map this run's grants to the root-node authority the daemon issues (Stage 5 phase 5f: the
/// `--grant` flags in daemon mode are sugar for issue-then-run at the root, spec §3.2). Paths are
/// the SAME absolute, lexically-normalized strings the embedded `RootVal` carries, so the broker's
/// path lattice sees exactly what the runtime resolves against.
fn authority_spec_from_grants(grants: &Grants, program: &str) -> crate::broker_ipc::AuthoritySpec {
    let root = grants.build_root();
    let mut effects: BTreeSet<&'static str> = BTreeSet::new();
    if root.console || !root.fs_write.is_empty() {
        effects.insert("Write");
    }
    if !root.fs_read.is_empty() {
        effects.insert("Read");
    }
    if !root.net.is_empty() {
        effects.insert("Net");
    }
    if root.clock {
        effects.insert("Clock");
    }
    if root.rand {
        effects.insert("Rand");
    }
    if root.declassify {
        effects.insert("Declassify");
    }
    if root.foreign_load {
        effects.insert("ForeignCall");
    }
    let mut secret_names: Vec<String> = grants.secrets.keys().cloned().collect();
    secret_names.sort();
    crate::broker_ipc::AuthoritySpec {
        effects: effects.into_iter().map(str::to_string).collect(),
        fs_read: root.fs_read.iter().map(|p| p.to_string_lossy().to_string()).collect(),
        fs_write: root.fs_write.iter().map(|p| p.to_string_lossy().to_string()).collect(),
        net: root.net.clone(),
        secrets: secret_names,
        declassify: Vec::new(),
        foreign_c: {
            let mut libs: Vec<String> = grants.foreign_c.keys().cloned().collect();
            libs.sort();
            libs
        },
        foreign_python: grants.foreign_python.clone(),
        holder_kind: "process".to_string(),
        holder_desc: program.to_string(),
        ttl_millis: None,
    }
}

/// The inverse of [`authority_spec_from_grants`], for `--lease` runs (phase 5j): derive the run's
/// local `Grants` from the delegated node's authority, so the in-process Stage 1–4 scope checks see
/// exactly the leased slice (the broker's per-§4 checks enforce it regardless — this keeps the two
/// layers agreeing instead of the local layer being wider). `foreign_c` binary PATHS are grant data
/// a human supplies via `--grant foreign.c=LIB:PATH`; the lease's `foreign.c` scope bounds which
/// lib NAMES the broker will actually bind.
fn grants_from_lease(info: &crate::broker_ipc::NodeInfo, foreign_c: HashMap<String, String>) -> Grants {
    let has = |e: &str| info.effects.iter().any(|x| x == e);
    Grants {
        // Console is Write-effect-gated at the broker (spec §4.1: Console requires Write).
        console: has("Write"),
        fs_read: info.fs_read.clone(),
        fs_write: info.fs_write.clone(),
        net: info.net.clone(),
        clock: has("Clock"),
        rand: has("Rand"),
        declassify: has("Declassify"),
        // Daemon mode strips local secret VALUES and keeps only broker-handle names (invariant 23);
        // the names come from the lease's `secrets` scope.
        secrets: info.secrets.iter().map(|n| (n.clone(), String::new())).collect(),
        foreign_c,
        foreign_python: info.foreign_python.clone(),
    }
}

/// `delulu secrets set NAME VALUE | list [--state-dir DIR]` (Stage 5 phase 5g, minimal v0.5 form —
/// full CLI polish is a later chunk). Writes the broker's secret store directly; a daemon started
/// AFTER the write sees the secret (the daemon loads the store at startup — restart to pick up new
/// names; flagged as a v0.5 limitation).
fn cmd_secrets(rest: &[String]) -> i32 {
    let Some(sub) = rest.first().map(String::as_str) else {
        eprintln!("error: `secrets` needs a subcommand: set NAME VALUE | list [--state-dir DIR]");
        return 2;
    };
    let state_flag = rest.iter().position(|a| a == "--state-dir").and_then(|i| rest.get(i + 1)).cloned();
    let Some(state_dir) = crate::brokerd::resolve_state_dir(state_flag.as_deref()) else {
        eprintln!("error: cannot resolve the broker state directory (no HOME/USERPROFILE) — pass --state-dir DIR");
        return 2;
    };
    let store = delulu_broker::SecretStore::load(state_dir.join("secrets.json"));
    match sub {
        "set" => {
            // Positionals = everything after `set` minus `--state-dir <DIR>` and other flags.
            let mut positionals: Vec<&String> = Vec::new();
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--state-dir" => i += 1, // skip its value too
                    s if s.starts_with("--") => {}
                    _ => positionals.push(&rest[i]),
                }
                i += 1;
            }
            let (Some(name), Some(value)) = (positionals.first(), positionals.get(1)) else {
                eprintln!("error: `secrets set` needs NAME and VALUE");
                return 2;
            };
            match store.set(name.as_str(), value.as_str()) {
                Ok(()) => {
                    eprintln!("ok: secret `{name}` stored (broker-resident; bytes enter a program only on `expose`)");
                    eprintln!("note: a running broker daemon loads the store at startup — restart it to pick up new names");
                    0
                }
                Err(e) => {
                    eprintln!("error: cannot write the secret store: {e}");
                    2
                }
            }
        }
        "list" => {
            for name in store.names() {
                println!("{name}");
            }
            0
        }
        other => {
            eprintln!("error: unknown secrets subcommand `{other}` (set | list)");
            2
        }
    }
}

// ----- grants (Stage 5 phase 5j: the grant-tree CLI surface, spec §3.2) --------------------------
//
// Every verb talks to the RUNNING daemon over `broker/1` — there is no local fallback: with the
// daemon down each verb fails DL1401 carrying the exact start command (invariant 27, playbook trap
// 4). The CLI is the human at the top of the tree (spec §3.1): `delegate` with no `--parent` first
// issues a root node holding exactly the requested authority — the typed command line IS the human
// action (the same footing as the `--grant` issue-then-run sugar) — then delegates under it.

/// Parse a TTL duration like `500ms`, `90s`, `30m`, `1h`, `2d` into a millisecond count.
fn parse_ttl_millis(s: &str) -> Option<i64> {
    let (num, mult) = if let Some(n) = s.strip_suffix("ms") {
        (n, 1)
    } else if let Some(n) = s.strip_suffix('s') {
        (n, 1_000)
    } else if let Some(n) = s.strip_suffix('m') {
        (n, 60_000)
    } else if let Some(n) = s.strip_suffix('h') {
        (n, 3_600_000)
    } else if let Some(n) = s.strip_suffix('d') {
        (n, 86_400_000)
    } else {
        return None;
    };
    let v: i64 = num.parse().ok()?;
    if v <= 0 {
        return None;
    }
    v.checked_mul(mult)
}

/// Wall-clock "now" in epoch millis — the CLI side of an absolute TTL deadline (the broker's own
/// clock does the enforcement; spec §4.3).
fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// One daemon round-trip for a `grants` verb. Fail closed (invariant 27): an unreachable daemon
/// prints DL1401 with the exact start command and yields `Err(1)`; a daemon `Error` reply prints
/// its diagnostic (e.g. a DL0802 whose message carries the computed intersection) and yields
/// `Err(1)`.
fn grants_rpc(
    state_dir: &std::path::Path,
    body: crate::broker_ipc::ReqBody,
    json: bool,
) -> Result<crate::broker_ipc::Response, i32> {
    let map = SourceMap::new();
    match crate::brokerd::request(state_dir, body) {
        Ok(crate::broker_ipc::Response::Error { code, message, .. }) => {
            let d = Diagnostic::error(crate::broker_client::static_code(&code), message);
            print_diagnostics("grants", &[d], &map, None, json);
            Err(1)
        }
        Ok(resp) => Ok(resp),
        Err(e) => {
            let d = Diagnostic::error(
                "DL1401",
                format!(
                    "broker unreachable: {e} — start it with `delulu broker start` \
                     (fail closed, invariant 27: `grants` verbs never fall back to local state)"
                ),
            );
            print_diagnostics("grants", &[d], &map, None, json);
            Err(1)
        }
    }
}

/// One wire node as a compact human line (`grants list`). Holder fields are display DATA — never a
/// decision input (criterion 9).
fn render_node_line(n: &crate::broker_ipc::NodeInfo) -> String {
    let authority = crate::brokerd::spec_to_authority(&n.authority_spec()).render_compact();
    let state = match (n.state.as_str(), n.by_seq) {
        ("revoked", Some(seq)) => format!("revoked@{seq}"),
        (s, _) => s.to_string(),
    };
    let parent = n.parent.as_deref().unwrap_or("-");
    format!("{}  [{}]  parent={}  {}  ({}) {}", n.id, state, parent, authority, n.holder_kind, n.holder_desc)
}

fn cmd_grants(rest: &[String]) -> i32 {
    let Some(sub) = rest.first().map(String::as_str) else {
        eprintln!(
            "error: `grants` needs a subcommand: list | tree | inspect <g_ID> | revoke <g_ID> | \
             delegate [--parent g_ID] --effects E,.. [--fs-read P].. [--ttl 1h] [--multi]"
        );
        return 2;
    };
    let args = &rest[1..];
    let json = args.iter().any(|a| a == "--json");
    let state_flag = args
        .iter()
        .position(|a| a == "--state-dir")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .or_else(|| args.iter().find_map(|a| a.strip_prefix("--state-dir=").map(str::to_string)));
    let Some(state_dir) = crate::brokerd::resolve_state_dir(state_flag.as_deref()) else {
        eprintln!("error: cannot resolve the broker state directory (no HOME/USERPROFILE) — pass --state-dir DIR");
        return 2;
    };
    // The first positional after the subcommand (`inspect`/`revoke` take a node id). `--state-dir`
    // is the only value-taking flag these verbs accept, so skip it and its value.
    let node_arg = {
        let mut found = None;
        let mut skip = false;
        for a in args {
            if skip {
                skip = false;
                continue;
            }
            if a == "--state-dir" {
                skip = true;
                continue;
            }
            if a.starts_with("--") {
                continue;
            }
            found = Some(a.clone());
            break;
        }
        found
    };

    match sub {
        "list" => {
            let resp = match grants_rpc(&state_dir, crate::broker_ipc::ReqBody::List, json) {
                Ok(r) => r,
                Err(c) => return c,
            };
            let crate::broker_ipc::Response::Listed { nodes } = resp else {
                eprintln!("error: unexpected list response: {resp:?}");
                return 2;
            };
            if json {
                let arr: Vec<Json> = nodes.iter().map(|n| serde_json::to_value(n).expect("node serializes")).collect();
                println!(
                    "{}",
                    json!({ "command": "grants", "subcommand": "list", "count": nodes.len(), "nodes": arr })
                );
            } else if nodes.is_empty() {
                println!("(no grants — the tree is empty)");
            } else {
                for n in &nodes {
                    println!("{}", render_node_line(n));
                }
            }
            0
        }
        "tree" => {
            let resp = match grants_rpc(&state_dir, crate::broker_ipc::ReqBody::Tree, json) {
                Ok(r) => r,
                Err(c) => return c,
            };
            let crate::broker_ipc::Response::Tree { text } = resp else {
                eprintln!("error: unexpected tree response: {resp:?}");
                return 2;
            };
            if json {
                println!("{}", json!({ "command": "grants", "subcommand": "tree", "tree": text }));
            } else if text.is_empty() {
                println!("(no grants — the tree is empty)");
            } else {
                print!("{text}");
            }
            0
        }
        "inspect" => {
            let Some(id) = node_arg else {
                eprintln!("error: `grants inspect` needs a node id (g_…)");
                return 2;
            };
            let resp = match grants_rpc(&state_dir, crate::broker_ipc::ReqBody::Inspect { node: id }, json) {
                Ok(r) => r,
                Err(c) => return c,
            };
            let crate::broker_ipc::Response::Inspected { node: n } = resp else {
                eprintln!("error: unexpected inspect response: {resp:?}");
                return 2;
            };
            if json {
                println!(
                    "{}",
                    json!({ "command": "grants", "subcommand": "inspect", "node": serde_json::to_value(&n).expect("node serializes") })
                );
            } else {
                println!("id:        {}", n.id);
                println!("parent:    {}", n.parent.as_deref().unwrap_or("(root)"));
                // Display only — the holder is data, never a decision input (criterion 9).
                println!("holder:    {} — {} [{}]", n.holder_kind, n.holder_desc, n.holder_peer);
                match (n.state.as_str(), n.by_seq) {
                    ("revoked", Some(seq)) => println!("state:     revoked (by audit seq {seq})"),
                    (s, _) => println!("state:     {s}"),
                }
                match n.ttl_millis {
                    Some(ttl) => println!("ttl:       {}", delulu_broker::render_ts_utc(ttl)),
                    None => println!("ttl:       (none)"),
                }
                println!("created:   {}", delulu_broker::render_ts_utc(n.created_millis));
                println!("audit_seq: {}", n.audit_seq);
                println!("authority: {}", crate::brokerd::spec_to_authority(&n.authority_spec()).render_compact());
            }
            0
        }
        "revoke" => {
            let Some(id) = node_arg else {
                eprintln!("error: `grants revoke` needs a node id (g_…)");
                return 2;
            };
            // The CLI revokes AS the named node (self-or-descendant is always satisfied by
            // caller == target, spec §3.2); transitivity kills the whole subtree.
            let resp = match grants_rpc(
                &state_dir,
                crate::broker_ipc::ReqBody::Revoke { caller: id.clone(), target: id },
                json,
            ) {
                Ok(r) => r,
                Err(c) => return c,
            };
            let crate::broker_ipc::Response::Revoked { by_seq, epoch, newly_revoked } = resp else {
                eprintln!("error: unexpected revoke response: {resp:?}");
                return 2;
            };
            if json {
                println!(
                    "{}",
                    json!({
                        "command": "grants", "subcommand": "revoke",
                        "by_seq": by_seq, "epoch": epoch, "newly_revoked": newly_revoked,
                        "revocation_takes_effect": delulu_diag::REVOCATION_BOUND,
                    })
                );
            } else {
                if newly_revoked.is_empty() {
                    eprintln!("ok: already revoked (idempotent; audit seq {by_seq}, epoch {epoch})");
                } else {
                    eprintln!(
                        "ok: revoked {} node(s) (audit seq {by_seq}, epoch {epoch}): {}",
                        newly_revoked.len(),
                        newly_revoked.join(", ")
                    );
                }
                // The honest §4.2 bound, stated at the point of revocation (playbook trap 3).
                eprintln!("takes effect: {}", delulu_diag::REVOCATION_BOUND);
            }
            0
        }
        "delegate" => cmd_grants_delegate(args, &state_dir, json),
        other => {
            eprintln!("error: unknown grants subcommand `{other}` (list | tree | inspect | revoke | delegate)");
            2
        }
    }
}

/// `delulu grants delegate` (spec §3.2): attenuate + mint a portable lease token, printed to
/// STDOUT so a script/orchestrator can capture it (`--json` for the structured form). With no
/// `--parent`, a root node holding exactly the requested authority is issued first — the typed
/// command line is the human action at the top of the tree (spec §3.1).
fn cmd_grants_delegate(args: &[String], state_dir: &std::path::Path, json: bool) -> i32 {
    use crate::broker_ipc::{AuthoritySpec, ReqBody, Response};

    /// `--name value` or `--name=value` (both accepted, like the shared `parse_opts`).
    fn flag_value(args: &[String], i: &mut usize, name: &str) -> Option<String> {
        let a = &args[*i];
        if let Some(v) = a.strip_prefix(name) {
            if let Some(v) = v.strip_prefix('=') {
                return Some(v.to_string());
            }
        }
        if a == name && *i + 1 < args.len() {
            *i += 1;
            return Some(args[*i].clone());
        }
        None
    }

    let mut effects: Vec<String> = Vec::new();
    let (mut fs_read, mut fs_write, mut net) = (Vec::new(), Vec::new(), Vec::new());
    let (mut secrets, mut declassify) = (Vec::new(), Vec::new());
    let (mut foreign_c, mut foreign_python) = (Vec::new(), Vec::new());
    let mut ttl: Option<String> = None;
    let mut parent: Option<String> = None;
    let mut multi = false;
    let mut holder_desc = "delegated lease".to_string();

    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--multi" {
            multi = true;
        } else if a == "--json" {
            // handled by the caller
        } else if let Some(_v) = flag_value(args, &mut i, "--state-dir") {
            // handled by the caller
        } else if let Some(v) = flag_value(args, &mut i, "--effects") {
            effects.extend(v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
        } else if let Some(v) = flag_value(args, &mut i, "--fs-read") {
            fs_read.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--fs-write") {
            fs_write.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--net") {
            net.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--secret") {
            secrets.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--declassify") {
            declassify.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--foreign-c") {
            foreign_c.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--foreign-python") {
            foreign_python.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--ttl") {
            ttl = Some(v);
        } else if let Some(v) = flag_value(args, &mut i, "--parent") {
            parent = Some(v);
        } else if let Some(v) = flag_value(args, &mut i, "--holder-desc") {
            holder_desc = v;
        } else {
            eprintln!("error: unknown `grants delegate` argument `{a}`");
            return 2;
        }
        i += 1;
    }

    // A typo'd effect name must not silently vanish (it would delegate LESS than asked — safe but
    // baffling). Validate up front.
    for e in &effects {
        if Effect::core_from_name(e).is_none() {
            eprintln!("error: unknown effect `{e}` (core effects: Read, Write, Net, Clock, Rand, Declassify, ForeignCall)");
            return 2;
        }
    }
    // fs scope paths are absolutized + lexically normalized against THIS command's cwd — the same
    // frame `--grant fs.*` uses (see `authority_spec_from_grants`: "the SAME absolute, lexically-
    // normalized strings the embedded RootVal carries") — so the broker's path lattice sees exactly
    // what an agent's runtime will resolve its file arguments against. Consequence for holders:
    // mint the delegation from the directory the paths are relative to.
    let norm = |v: Vec<String>| -> Vec<String> {
        v.into_iter()
            .map(|p| delulu_runtime::prim::granted_root(&p).to_string_lossy().to_string())
            .collect()
    };
    let fs_read = norm(fs_read);
    let fs_write = norm(fs_write);
    let ttl_millis = match &ttl {
        None => None,
        Some(s) => match parse_ttl_millis(s) {
            Some(d) => Some(now_millis() + d),
            None => {
                eprintln!("error: bad --ttl `{s}` (use e.g. 500ms, 90s, 30m, 1h, 2d)");
                return 2;
            }
        },
    };

    let parent = match parent {
        Some(p) => p,
        None => {
            // No parent named: issue a root holding EXACTLY the requested authority (nothing
            // wider), then delegate under it. Root issuance stays a human action — this typed
            // command line — never a programmatic path (Constitution §5.16 law 4).
            let root_spec = AuthoritySpec {
                effects: effects.clone(),
                fs_read: fs_read.clone(),
                fs_write: fs_write.clone(),
                net: net.clone(),
                secrets: secrets.clone(),
                declassify: declassify.clone(),
                foreign_c: foreign_c.clone(),
                foreign_python: foreign_python.clone(),
                holder_kind: "human".to_string(),
                holder_desc: "delulu grants delegate (root)".to_string(),
                ttl_millis: None,
            };
            match grants_rpc(state_dir, ReqBody::Issue(root_spec), json) {
                Ok(Response::Issued { node }) => node,
                Ok(other) => {
                    eprintln!("error: unexpected issue response: {other:?}");
                    return 2;
                }
                Err(code) => return code,
            }
        }
    };

    let child_spec = AuthoritySpec {
        effects,
        fs_read,
        fs_write,
        net,
        secrets,
        declassify,
        foreign_c,
        foreign_python,
        holder_kind: "delegate".to_string(),
        holder_desc,
        ttl_millis,
    };
    match grants_rpc(state_dir, ReqBody::Delegate { parent: parent.clone(), authority: child_spec, multi }, json) {
        Ok(Response::Delegated { node, token }) => {
            if json {
                println!(
                    "{}",
                    json!({
                        "command": "grants", "subcommand": "delegate",
                        "parent": parent, "node": node, "token": token,
                        "multi": multi, "ttl_millis": ttl_millis,
                    })
                );
            } else {
                eprintln!(
                    "ok: delegated `{node}` ⊑ `{parent}`{}{}",
                    match ttl_millis {
                        Some(t) => format!(", expires {}", delulu_broker::render_ts_utc(t)),
                        None => String::new(),
                    },
                    if multi { ", multi-redemption" } else { ", single-redemption" }
                );
                eprintln!("hand the token to the agent; it runs with: delulu run <file> --lease <token>");
                println!("{token}");
            }
            0
        }
        Ok(other) => {
            eprintln!("error: unexpected delegate response: {other:?}");
            2
        }
        Err(code) => code,
    }
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

fn cmd_run(rest: &[String]) -> i32 {
    let (file, mut opts) = parse_opts(rest);
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
                             `--isolation process` (worker-style OS containment of the code \
                             outside the proof; explicitly weaker: no guest boundary, no \
                             virtio-fs scope mounts, no default-deny egress) [a human must choose \
                             the weaker profile; see `delulu explain DL1408` and spec §6.1]"
                        ),
                    );
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
                "process" => {
                    "process — foreign code in minimum-privilege worker subprocesses; the \
                     verified program remains in-process (weaker than microvm; spec §6.1)"
                }
                _ => "none — in-process (language + custody enforcement only)",
            };
            eprintln!("isolation: {label}");
        }
    }

    // A `.dwx` is a pre-built, authority-carrying artifact — re-verify and run it directly.
    if file.ends_with(".dwx") {
        return run_dwx_artifact(&file, &opts);
    }
    let (map, id, src) = match load(&file) {
        Ok(x) => x,
        Err(c) => return c,
    };
    let checked = check_source(id, &src);
    if errors(&checked.diagnostics) > 0 {
        print_diagnostics("run", &checked.diagnostics, &map, None, opts.json);
        return 1;
    }
    if !checked.result.main_present {
        eprintln!("error: `{file}` has no `fn main(root: Root)` to run");
        return 2;
    }

    // Grant flow (§7.2): manifest DL0701 check, then reconcile grants.
    let dir = std::path::Path::new(&file).parent().unwrap_or_else(|| std::path::Path::new("."));
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
        grants = grants_from_lease(&info, std::mem::take(&mut grants.foreign_c));
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
        let spec = authority_spec_from_grants(&grants, &file);
        match crate::broker_client::BrokerClientCustody::issue_root(state_dir, spec, opts.epoch_ms) {
            Ok(c) => daemon_custody = Some(c),
            Err(d) => {
                let diag = Diagnostic::error(d.code, d.message);
                print_diagnostics("run", &[diag], &map, None, opts.json);
                return 1;
            }
        }
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
        let sink = if opts.trace_effects || opts.assert_trace {
            Some(delulu_runtime::TraceSink::new())
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
        let run_result = delulu_wasm::run_main(&wasm, &cfg);

        // Emit the trace before verdicts (the witness is available even on a fault).
        if let Some(s) = &sink {
            if opts.trace_effects {
                let lines = s.to_json_lines();
                match &opts.trace_out {
                    Some(path) => {
                        if let Err(e) = std::fs::write(path, lines + "\n") {
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
    let sink = if opts.trace_effects || opts.assert_trace {
        Some(delulu_runtime::TraceSink::new())
    } else {
        None
    };

    let mut root_val = build_root(&grants);
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
    let mut interp = Interp::new(&checked.module).with_foreign(
        checked.result.foreign_binds.clone(),
        grants.foreign_c.clone(),
        max_ret,
    );
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
    if let Some(c) = daemon_custody.take() {
        // Stage 5 phase 5f: route every effectful op through the broker daemon.
        interp = interp.with_custody(Box::new(c));
    }
    if let Some(s) = &sink {
        interp = interp.with_trace(s.clone());
    }
    let run_result = interp.run_main(root);

    // Emit the trace before verdicts, so the witness is available even on a fault.
    if let Some(s) = &sink {
        if opts.trace_effects {
            let lines = s.to_json_lines();
            match &opts.trace_out {
                Some(path) => {
                    if let Err(e) = std::fs::write(path, lines + "\n") {
                        eprintln!("error: cannot write trace to `{path}`: {e}");
                    }
                }
                None => eprintln!("{lines}"),
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

    code
}

fn build_root(grants: &Grants) -> RootVal {
    grants.build_root()
}

// ----- foreign grant flow + authority (Stage 4, spec §4.1/§6) --------------

/// One `foreign` block, extracted from the AST for the grant prompt and the authority report.
struct ForeignBlock {
    abi: String,
    lib: String,
    symbols: Vec<String>,
    span: delulu_diag::Span,
}

fn foreign_blocks_of(module: &delulu_syntax::ast::Module) -> Vec<ForeignBlock> {
    module
        .items
        .iter()
        .filter_map(|it| match it {
            Item::Foreign(fd) => Some(ForeignBlock {
                abi: fd.abi.clone(),
                lib: fd.name.name.clone(),
                symbols: fd.fns.iter().map(|f| f.name.name.clone()).collect(),
                span: fd.span,
            }),
            _ => None,
        })
        .collect()
}

/// Deny-by-default grant flow for the foreign libs a program binds (spec §4.1 / criterion 4).
/// Returns `Err(exit_code)` when a bound lib exceeds the manifest ceiling or has no grant; on success
/// `grants.foreign_c` holds a binary path for every bound lib. An interactive human is prompted with
/// the full symbol list; agents, `--json`, `--no-prompt`, and non-terminals are never prompted (they
/// get DL1303 and refuse). Everything happens BEFORE `main` runs — never mid-run.
fn foreign_grant_preflight(
    checked: &Checked,
    manifest: &Option<delulu_runtime::Manifest>,
    grants: &mut Grants,
    opts: &Opts,
    map: &SourceMap,
) -> Result<(), i32> {
    use std::io::IsTerminal;

    let mut diags: Vec<Diagnostic> = Vec::new();

    // ----- C libs (spec §4.1) ----------------------------------------------
    let bound: BTreeSet<&String> = checked.result.foreign_binds.values().collect();
    let blocks = foreign_blocks_of(&checked.module);
    let symbols_of = |lib: &str| -> Vec<String> {
        blocks.iter().find(|b| b.lib == lib).map(|b| b.symbols.clone()).unwrap_or_default()
    };
    for lib in bound {
        // Manifest ceiling: a bound lib must be listed in `[authority] foreign.c` when a manifest
        // exists. Exceeding it is the program reaching for authority it never declared (DL1303).
        if let Some(m) = manifest {
            if !m.foreign_c.iter().any(|l| l == lib) {
                diags.push(Diagnostic::error(
                    "DL1303",
                    format!("program uses foreign lib `{lib}` not permitted by the authority manifest (`[authority] foreign.c`)"),
                ));
                continue;
            }
        }
        if grants.foreign_c.contains_key(lib) {
            continue;
        }
        let interactive = !opts.json && !opts.no_prompt && std::io::stdin().is_terminal();
        if interactive {
            if let Some(path) = prompt_foreign_grant(lib, &symbols_of(lib)) {
                grants.foreign_c.insert(lib.clone(), path);
                continue;
            }
        }
        diags.push(Diagnostic::error(
            "DL1303",
            format!("foreign lib `{lib}` was not granted — pass `--grant foreign.c={lib}:PATH` (the human chooses which binary)"),
        ));
    }

    // ----- embedded Python (spec §5.1) -------------------------------------
    // A program that reaches `root.python` needs a granted `foreign.python` allowlist. Deny-by-
    // default at the grant flow (DL1303), before `main` runs — never mid-run. The runtime allowlist
    // check at `py.import` (DL1305) is a second, per-import gate on top of this.
    if python_usage(&checked.module).is_some() {
        if let Some(m) = manifest {
            if m.foreign_python.is_empty() {
                diags.push(Diagnostic::error(
                    "DL1303",
                    "program uses embedded Python not permitted by the authority manifest (`[authority] foreign.python`)".to_string(),
                ));
            } else {
                // Granted patterns must not exceed the manifest's declared allowlist (attenuation).
                for p in &grants.foreign_python {
                    if !m.foreign_python.iter().any(|d| d == p) {
                        diags.push(Diagnostic::error(
                            "DL1303",
                            format!("python import pattern `{p}` exceeds the authority manifest (`[authority] foreign.python`)"),
                        ));
                    }
                }
            }
        }
        if grants.foreign_python.is_empty() {
            diags.push(Diagnostic::error(
                "DL1303",
                "embedded Python was not granted — pass `--grant foreign.python=numpy` (an import allowlist pattern; the human chooses which modules)".to_string(),
            ));
        }
    }

    if diags.is_empty() {
        Ok(())
    } else {
        print_diagnostics("run", &diags, map, None, opts.json);
        Err(1)
    }
}

/// Prompt an interactive human for a foreign lib's binary path, showing the FULL symbol list first
/// (spec §4.1). Returns `None` on an empty line (deny) or EOF. Uses stderr so `--json`/piped stdout
/// stays clean — though this is only ever reached for a real terminal.
fn prompt_foreign_grant(lib: &str, symbols: &[String]) -> Option<String> {
    use std::io::Write as _;
    eprintln!("foreign lib `{lib}` (abi \"c\") requests these symbols:");
    if symbols.is_empty() {
        eprintln!("    (no symbols declared)");
    } else {
        for s in symbols {
            eprintln!("    - {s}");
        }
    }
    eprint!("grant a binary path for `{lib}` (empty line to deny): ");
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    if std::io::stdin().read_line(&mut line).ok()? == 0 {
        return None;
    }
    let p = line.trim().to_string();
    if p.is_empty() {
        None
    } else {
        Some(p)
    }
}

/// Build the `foreign_calls` JSON array (spec §6) from a program's `foreign` blocks AND its embedded
/// Python use. `granted_path` is null: `delulu authority` is a static command with no grants, and the
/// path is a runtime human decision, not program data. `used_at` reports the declaration/use site at
/// declaration granularity, the same altitude `delulu why` reports at. The `python_allowlist` is the
/// manifest's declared `foreign.python` (empty when there is no manifest).
fn foreign_calls_json(module: &delulu_syntax::ast::Module, map: &SourceMap, python_allowlist: &[String]) -> Vec<Json> {
    let mut out: Vec<Json> = foreign_blocks_of(module)
        .into_iter()
        .map(|b| {
            let (line, _col) = map.position(b.span.file, b.span.start);
            json!({
                "abi": b.abi,
                "lib": b.lib,
                "symbols": b.symbols,
                "granted_path": Json::Null,
                "used_at": [ { "file": map.name(b.span.file), "line": line } ],
            })
        })
        .collect();
    // The embedded-Python entry (spec §6): `{abi:"python", allowlist, imports_seen, used_at}`. Present
    // only when the program actually reaches for Python (`root.python`), so a non-Python program's
    // report is unchanged. `imports_seen` is the statically-known set of `py.import("literal")` names.
    if let Some(usage) = python_usage(module) {
        let (line, _col) = map.position(usage.used_at.file, usage.used_at.start);
        out.push(json!({
            "abi": "python",
            "allowlist": python_allowlist,
            "imports_seen": usage.imports_seen,
            "used_at": [ { "file": map.name(usage.used_at.file), "line": line } ],
        }));
    }
    out
}

/// A program's embedded-Python use, discovered by an AST walk: where `root.python` is first reached
/// (`used_at`) and the statically-known `py.import("literal")` module names (`imports_seen`).
struct PythonUsage {
    used_at: delulu_diag::Span,
    imports_seen: Vec<String>,
}

fn python_usage(module: &delulu_syntax::ast::Module) -> Option<PythonUsage> {
    let mut w = PyWalk { python_call: None, imports: Vec::new() };
    for item in &module.items {
        match item {
            Item::Fn(f) => w.walk_block(&f.body),
            Item::Const(c) => w.walk_expr(&c.value),
            _ => {}
        }
    }
    w.python_call.map(|used_at| PythonUsage { used_at, imports_seen: w.imports })
}

/// A read-only AST walk collecting embedded-Python use: the first `root.python(...)` method call
/// (which is what mints a `Cap[Python]`) and every `py.import("literal")` module name.
struct PyWalk {
    python_call: Option<delulu_diag::Span>,
    imports: Vec<String>,
}

impl PyWalk {
    fn walk_block(&mut self, b: &delulu_syntax::ast::Block) {
        for s in &b.stmts {
            self.walk_stmt(s);
        }
    }

    fn walk_stmt(&mut self, s: &delulu_syntax::ast::Stmt) {
        use delulu_syntax::ast::Stmt::*;
        match s {
            Let { value, .. } => self.walk_expr(value),
            Assign { value, .. } => self.walk_expr(value),
            While { cond, body, .. } => {
                self.walk_expr(cond);
                self.walk_block(body);
            }
            Return { value, .. } => {
                if let Some(e) = value {
                    self.walk_expr(e);
                }
            }
            Expr(e) => self.walk_expr(e),
        }
    }

    fn walk_expr(&mut self, e: &delulu_syntax::ast::Expr) {
        use delulu_syntax::ast::{Expr::*, LitKind};
        match e {
            Method { recv, name, args, span, .. } => {
                self.walk_expr(recv);
                for a in args {
                    self.walk_expr(a);
                }
                if name.name == "python" && self.python_call.is_none() {
                    self.python_call = Some(*span);
                }
                if name.name == "import" {
                    if let Some(Lit { kind: LitKind::Str(s), .. }) = args.first() {
                        if !self.imports.contains(s) {
                            self.imports.push(s.clone());
                        }
                    }
                }
            }
            List { items, .. } => items.iter().for_each(|it| self.walk_expr(it)),
            Record { fields, .. } => fields.iter().for_each(|(_, v)| self.walk_expr(v)),
            Call { callee, args, .. } => {
                self.walk_expr(callee);
                args.iter().for_each(|a| self.walk_expr(a));
            }
            Field { recv, .. } => self.walk_expr(recv),
            Index { recv, index, .. } => {
                self.walk_expr(recv);
                self.walk_expr(index);
            }
            Unary { operand, .. } => self.walk_expr(operand),
            Binary { lhs, rhs, .. } => {
                self.walk_expr(lhs);
                self.walk_expr(rhs);
            }
            If { cond, then_, else_, .. } => {
                self.walk_expr(cond);
                self.walk_block(then_);
                if let Some(e) = else_ {
                    self.walk_expr(e);
                }
            }
            Match { scrutinee, arms, .. } => {
                self.walk_expr(scrutinee);
                arms.iter().for_each(|a| self.walk_expr(&a.body));
            }
            Lambda { body, .. } => self.walk_block(body),
            Try { inner, .. } => self.walk_expr(inner),
            Block(b) => self.walk_block(b),
            Lit { .. } | Var { .. } => {}
        }
    }
}

// ----- audit (Stage 5 phase 5d: the hash-chained log's read surface) --------
//
// `delulu audit tail|query|verify` reads the broker's log files DIRECTLY — no daemon involved (the
// daemon is chunk 3). The log is observability, not enforcement (spec §7): nothing here feeds any
// authority decision. Revocation latency honesty (spec §4.2): records observe that revocations take
// effect synchronously before the next use / within one epoch interval — never "immediately".

/// Default log dir: `~/.delulu/audit` (HOME, else USERPROFILE on Windows). Only the CLI knows this
/// path — the `delulu-broker` library takes injected paths only.
fn default_audit_dir() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(std::path::PathBuf::from(home).join(".delulu").join("audit"))
}

/// One human line per record: seq, UTC time (display-only ISO render of the stored epoch millis),
/// action, decision, then whichever of actor/target the record carries.
fn render_audit_record(r: &delulu_broker::AuditRecord) -> String {
    let mut s = format!(
        "seq {:>5}  {}  {:<10} {:<5}",
        r.seq,
        delulu_broker::render_ts_utc(r.ts),
        r.action,
        r.decision
    );
    if let Some(a) = &r.actor_node {
        s.push_str(&format!("  actor={a}"));
    }
    if let Some(t) = &r.target {
        s.push_str(&format!("  target={t}"));
    }
    s
}

fn audit_record_json(r: &delulu_broker::AuditRecord) -> Json {
    let mut m = serde_json::Map::new();
    m.insert("seq".into(), json!(r.seq));
    m.insert("ts".into(), json!(r.ts));
    m.insert("prev_hash".into(), json!(r.prev_hash));
    m.insert("hash".into(), json!(r.hash));
    if let Some(a) = &r.actor_node {
        m.insert("actor_node".into(), json!(a));
    }
    m.insert("action".into(), json!(r.action));
    if let Some(t) = &r.target {
        m.insert("target".into(), json!(t));
    }
    if let Some(a) = &r.authority {
        m.insert("authority".into(), a.clone());
    }
    if let Some(s) = &r.span {
        m.insert("span".into(), json!(s));
    }
    m.insert("decision".into(), json!(r.decision));
    Json::Object(m)
}

fn cmd_audit(rest: &[String]) -> i32 {
    let Some(sub) = rest.first().map(String::as_str) else {
        eprintln!("error: `audit` needs a subcommand: tail [N] | query [--node g_ID] [--action A] [--effect E] | verify");
        return 2;
    };
    let args = &rest[1..];

    // The audit flag set (own small parser, like `why` peels its own positional).
    let mut json = false;
    let mut dir_flag: Option<String> = None;
    let mut filter = delulu_broker::QueryFilter::default();
    let mut tail_n: usize = 10;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--dir" => {
                if i + 1 < args.len() {
                    dir_flag = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--dir=") => dir_flag = Some(s["--dir=".len()..].to_string()),
            "--node" => {
                if i + 1 < args.len() {
                    filter.node = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--node=") => filter.node = Some(s["--node=".len()..].to_string()),
            "--action" => {
                if i + 1 < args.len() {
                    filter.action = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--action=") => filter.action = Some(s["--action=".len()..].to_string()),
            "--effect" => {
                if i + 1 < args.len() {
                    filter.effect = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--effect=") => filter.effect = Some(s["--effect=".len()..].to_string()),
            s if !s.starts_with('-') && s.chars().all(|c| c.is_ascii_digit()) => {
                tail_n = s.parse().unwrap_or(10);
            }
            _ => {}
        }
        i += 1;
    }

    let dir = match dir_flag.map(std::path::PathBuf::from).or_else(default_audit_dir) {
        Some(d) => d,
        None => {
            eprintln!("error: cannot resolve the audit directory (no HOME/USERPROFILE) — pass --dir DIR");
            return 2;
        }
    };
    let map = SourceMap::new(); // audit records carry no source text; diagnostics have no span

    match sub {
        "tail" | "query" => {
            let result = if sub == "tail" {
                delulu_broker::tail(&dir, tail_n)
            } else {
                delulu_broker::query(&dir, &filter)
            };
            let records = match result {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("error: cannot read audit log at `{}`: {e}", dir.display());
                    return 2;
                }
            };
            if json {
                let arr: Vec<Json> = records.iter().map(audit_record_json).collect();
                let report = json!({ "command": "audit", "subcommand": sub, "records": arr, "count": records.len() });
                println!("{}", serde_json::to_string_pretty(&report).expect("audit report serializes"));
            } else {
                if records.is_empty() {
                    eprintln!("(no matching audit records under `{}`)", dir.display());
                }
                for r in &records {
                    println!("{}", render_audit_record(r));
                }
            }
            0
        }
        "verify" => match delulu_broker::verify(&dir) {
            Ok(stats) => {
                if json {
                    let report = json!({
                        "command": "audit",
                        "subcommand": "verify",
                        "ok": true,
                        "files": stats.files,
                        "records": stats.records,
                        "head": stats.head,
                    });
                    println!("{}", serde_json::to_string_pretty(&report).expect("audit report serializes"));
                } else {
                    eprintln!(
                        "ok: audit chain verified — {} record(s) across {} file(s), head {}",
                        stats.records,
                        stats.files,
                        &stats.head[..16.min(stats.head.len())]
                    );
                }
                0
            }
            Err(e) => {
                // A broken chain is DL1405 at the failing seq (requires_human, spec §8); an I/O
                // problem is a plain CLI error (exit 2), not a tamper verdict.
                match e.denial() {
                    Some(d) => {
                        print_diagnostics("audit", &[d.to_diagnostic()], &map, None, json);
                        1
                    }
                    None => {
                        eprintln!("error: cannot verify audit log at `{}`: {e}", dir.display());
                        2
                    }
                }
            }
        },
        other => {
            eprintln!("error: unknown audit subcommand `{other}` (tail | query | verify)");
            2
        }
    }
}

// ----- explain / repl ------------------------------------------------------

fn cmd_explain(rest: &[String]) -> i32 {
    let code = rest.iter().find(|a| !a.starts_with('-')).map(|s| s.trim_start_matches("E-").to_string());
    let Some(code) = code else {
        eprintln!("error: `explain` needs a code, e.g. `delulu explain DL0501`");
        return 2;
    };
    // Named topics (Stage 5 phase 5j): `delulu explain E-REVOKE` states the spec §4.2 revocation
    // latency bound VERBATIM (playbook trap 3 — never "immediate").
    if let Some((title, body)) = delulu_diag::topic_explain(&code) {
        println!("E-{code}: {title}");
        println!("\n{body}");
        return 0;
    }
    match delulu_diag::code_title(&code) {
        Some(title) => {
            println!("{code}: {title}");
            // A longer explanation, where one exists (DL13xx foreign codes carry the spec §10
            // honesty caveats verbatim — reachability-not-behavior, link forward to Stage 5).
            if let Some(body) = delulu_diag::code_explain(&code) {
                println!("\n{body}");
            }
            0
        }
        None => {
            eprintln!("unknown code `{code}`");
            1
        }
    }
}

fn repl_cmd(rest: &[String]) -> i32 {
    let (_file, opts) = parse_opts(rest);
    let mut grants = Grants::default();
    for g in &opts.grants {
        let _ = grants.add(g);
    }
    crate::repl::run(grants)
}

// Silence an unused import in configurations where json! is not exercised.
#[allow(dead_code)]
fn _json_marker() -> Json {
    json!({})
}
