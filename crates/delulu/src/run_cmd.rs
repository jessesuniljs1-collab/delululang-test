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
}

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

/// The run report envelope. At L0 the `sandbox` object is the in-process backend: level 0, no host
/// guarantees, no limits, strict mode, no break-glass — measured facts, nothing claimed.
fn run_report(opts: &Opts, exit: i32) -> Json {
    let (ran, _) = RUN_STATE.with(|s| s.get());
    let requested = opts.isolation.clone().unwrap_or_else(|| "none".to_string());
    let mut env = success_envelope(
        "run",
        json!({
            "sandbox": {
                "backend": "inproc",
                "level": 0,
                "requested": requested,
                "granted": "none",
                "host_guarantees": [],
                "limits": null,
                "mode": "strict",
                "break_glass": false,
            },
            "outcome": { "ran": ran, "exit": exit },
        }),
    );
    if exit != 0 {
        env["summary"]["errors"] = json!(1);
    }
    env
}

pub(crate) fn cmd_run(rest: &[String]) -> i32 {
    RUN_STATE.with(|s| s.set((false, false)));
    let (_, opts) = parse_opts(rest);
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
    let code = cmd_run_inner(rest);
    if let Some(path) = &opts.report_out {
        let refused = RUN_STATE.with(|s| s.get().1);
        if !refused {
            let text = serde_json::to_string_pretty(&run_report(&opts, code)).expect("the run report serializes");
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
                             `delulu run {file} --isolation process` (worker-style OS containment of the code \
                             outside the proof; explicitly weaker: no guest boundary, no \
                             virtio-fs scope mounts, no default-deny egress) [a human must choose \
                             the weaker profile; see `delulu explain DL1408` and spec §6.1]"
                        ),
                    )
                    // NE-16c / PS-0-03: the repair STAGE5 §8 promised. No edit — choosing a weaker
                    // boundary is a person's decision — and the fallback command, ready to run.
                    .with_repair(delulu_diag::Repair {
                        id: "fall-back-to-isolation-process",
                        confidence: delulu_diag::Confidence::Suggest,
                        authority_widening: false,
                        requires_human: true,
                        edits: Vec::new(),
                        reason: Some(
                            "the fallback, `--isolation process`, is explicitly weaker (it contains \
                             foreign code only; no guest boundary, no scope mounts, no default-deny \
                             egress), so a human must choose it; the message names the exact \
                             command: `delulu run <file> --isolation process`",
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
                "process" => {
                    "process — isolates FOREIGN CODE ONLY: foreign libraries run in \
                     minimum-privilege worker subprocesses; the verified program itself stays \
                     in-process and is not sandboxed (weaker than microvm; spec §6.1)"
                }
                _ => "none — in-process (language + custody enforcement only)",
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
