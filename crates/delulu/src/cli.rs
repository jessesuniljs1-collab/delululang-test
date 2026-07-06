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
     \x20 delulu authority <file.delulu | package-dir> [--json]\n\
     \x20 delulu authority --diff <old.lock> <new.lock-or-package-dir> [--json]\n\
     \x20 delulu why       <Effect> <file.delulu | package-dir> [--json]\n\
     \x20 delulu repl      [--grant K[=V]]...\n\
     \x20 delulu explain   <DLxxxx>\n\
     \n\
     `delulu authority` prints the compiler-computed answer to \"what can this program do?\"\n\
     `delulu authority --diff` compares two lockfile states and reports authority widening.\n\
     `delulu why <Effect>` explains, at function granularity, why a program can perform an\n\
     effect — the shortest chain of calls from `main` down to the function that performs it.\n\
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
    let wasm = match delulu_wasm::compile_module(&checked.module) {
        Ok(w) => w,
        Err(e) => {
            let d = Diagnostic::error(e.code(), format!("{} — this program can't be built to a `.dwx` yet (run it on the interpreter)", e.message()));
            print_diagnostics("build", &[d], &map, None, opts.json);
            return 1;
        }
    };
    // Embed the same authority answer `delulu authority` reports, so the artifact carries its own
    // truthful manifest of what it can do.
    let scopes = manifest_scopes(file);
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
    let report = program_authority(&program, &name, &scopes);
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
            "error: `{effect_name}` is not a known effect (core effects: Read, Write, Net, Clock, Rand, Declassify; \
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
/// their code inline (DL0703 denied root slice, DL0903 out-of-bounds memory access); anything else
/// is a capability-scope violation (DL0904).
fn wasm_fault_code(msg: &str) -> &'static str {
    if msg.contains("DL0703") {
        "DL0703"
    } else if msg.contains("DL0903") {
        "DL0903"
    } else {
        "DL0904"
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

    let cfg = delulu_wasm::HostConfig {
        console: grants.console,
        clock: grants.clock,
        rand: grants.rand,
        fs_read_roots: grants.build_root().fs_read,
        fixed_clock_ms: opts.clock_ms,
        rand_seed: opts.seed,
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
    let (file, opts) = parse_opts(rest);
    let Some(file) = file else {
        eprintln!("error: `run` needs a file");
        return 2;
    };
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

    // WASM engine (spec §5.11 / Stage 3): compile `main` to WebAssembly and run it under the
    // deny-by-default Wasmtime host, with the console capability minted and checked host-side.
    // Constructs the backend can't compile are DL1201 — omit `--engine wasm` to use the interpreter.
    if opts.engine.as_deref() == Some("wasm") {
        let wasm = match delulu_wasm::compile_module(&checked.module) {
            Ok(w) => w,
            Err(e) => {
                let d = Diagnostic::error(e.code(), format!("{} — omit `--engine wasm` to run it on the interpreter", e.message()));
                print_diagnostics("run", &[d], &map, None, opts.json);
                return 1;
            }
        };
        let cfg = delulu_wasm::HostConfig {
            console: grants.console,
            clock: grants.clock,
            rand: grants.rand,
            fs_read_roots: grants.build_root().fs_read,
            fixed_clock_ms: opts.clock_ms,
            rand_seed: opts.seed,
        };
        return match delulu_wasm::run_main(&wasm, &cfg) {
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

    let root = Value::Root(Rc::new(build_root(&grants)));
    let mut interp = Interp::new(&checked.module);
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
