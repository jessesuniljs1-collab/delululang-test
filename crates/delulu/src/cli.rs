//! Command dispatch and the four Stage-1 commands (§9.5).

use std::rc::Rc;

use delulu_check::{
    authority_report, check_pins, check_program, check_self_authority, check_source,
    check_workspace, compute_lockfile, enforce_semver_law, load_package, program_authority,
    resolve_workspace, verify_locked, Lockfile,
};
use delulu_diag::{envelope_to_string, render_human, Diagnostic, SourceMap};
use delulu_runtime::{parse_manifest, Grants, Interp, Value};
use delulu_runtime::value::RootVal;
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
        "repl" => repl_cmd(rest),
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
     \x20 delulu lock      [package-dir] [--accept-authority <pkg>]... [--json]\n\
     \x20 delulu run       <file.delulu> [--json] [--grant K[=V]]... [--grant-manifest] [--no-prompt]\n\
     \x20 delulu authority <file.delulu | package-dir> [--json]\n\
     \x20 delulu repl      [--grant K[=V]]...\n\
     \x20 delulu explain   <DLxxxx>\n\
     \n\
     `delulu authority` prints the compiler-computed answer to \"what can this program do?\"\n\
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
    let scopes = manifest_scopes(&file);
    let program = checked.module.name.dotted();
    let report = authority_report(&program, &checked.result, &scopes);
    if opts.json {
        println!("{}", envelope_to_string("authority", &[], Some(report), &map));
    } else {
        print!("{}", render_authority(&report));
    }
    0
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
    let _ = writeln!(out, "  foreign:      (none — no code outside the guarantee)");
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

// ----- package mode (multi-module, Stage 2) --------------------------------

fn cmd_build(rest: &[String]) -> i32 {
    let (path, opts) = parse_opts(rest);
    let Some(path) = path else {
        eprintln!("error: `build` needs a package directory");
        return 2;
    };
    if !std::path::Path::new(&path).is_dir() {
        eprintln!("error: `build` expects a package directory (with src/ and delulu.toml)");
        return 2;
    }
    build_workspace(&path, &opts, "build", opts.locked)
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
    let report = program_authority(&program, &name, &scopes);
    if opts.json {
        println!("{}", envelope_to_string("authority", &[], Some(report), &pkg.source_map));
    } else {
        print!("{}", render_authority(&report));
    }
    0
}

// ----- run -----------------------------------------------------------------

fn cmd_run(rest: &[String]) -> i32 {
    let (file, opts) = parse_opts(rest);
    let Some(file) = file else {
        eprintln!("error: `run` needs a file");
        return 2;
    };
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

    let root = Value::Root(Rc::new(build_root(&grants)));
    let interp = Interp::new(&checked.module);
    match interp.run_main(root) {
        Ok(_) => 0,
        Err(fault) => {
            let mut d = Diagnostic::error(fault.code, fault.message.clone());
            if let Some(span) = fault.span {
                d = d.with_span(span, "runtime fault here");
            }
            print_diagnostics("run", &[d], &map, None, opts.json);
            1
        }
    }
}

fn build_root(grants: &Grants) -> RootVal {
    grants.build_root()
}

// ----- explain / repl ------------------------------------------------------

fn cmd_explain(rest: &[String]) -> i32 {
    let code = rest.iter().find(|a| !a.starts_with('-')).map(|s| s.trim_start_matches("E-").to_string());
    let Some(code) = code else {
        eprintln!("error: `explain` needs a code, e.g. `delulu explain DL0501`");
        return 2;
    };
    match delulu_diag::code_title(&code) {
        Some(title) => {
            println!("{code}: {title}");
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
