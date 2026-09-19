//! The sandbox guest (PS-A-03): `delulu __guest`, the child that runs a program while holding no
//! authority of its own.
//!
//! The host sends one [`Hello`] frame — the program, the hash it must match, the seed and the clock
//! — and then answers the guest's requests on `delulu-sandbox-channel/1` until the guest says it is
//! done. The guest's own root is EMPTY: every capability it uses is a handle the host minted, so
//! "the guest performs no effects" is true by construction rather than by policy.
//!
//! Standard input and standard output belong to the CHANNEL here. The program's own output is
//! performed by the host, on the host's streams, which is also what keeps a guest from forging the
//! run's report (D-V2-21).
//!
//! Like the foreign worker, this is an INTERNAL subcommand: dispatched by constant through a guard
//! arm, so it is never offered in `--help` or in completions, and never typed by a caller. The gates
//! that read the dispatch see literal-string arms only, which is exactly why the worker uses this
//! shape. What covers it instead is `tests/guest_cli.rs`, which drives the real binary end to end,
//! including its refusal to run without a hello.

use std::io;
use std::rc::Rc;

use delulu_runtime::channel::{read_frame, write_frame, ChannelSink, Hello, HostChannel, CHANNEL_VERSION};
use delulu_runtime::interp::Interp;
use delulu_runtime::sink::LocalSink;
use delulu_runtime::value::{RootVal, Value};

/// How long either side waits for the other before giving up. A channel with no deadline is the
/// IPC-1 shape: one stalled peer hangs the other for ever (PS-0-07 fixed the same hole for the
/// foreign worker, which is why the guest borrows its transport rather than using pipes).
const CHANNEL_DEADLINE: std::time::Duration = std::time::Duration::from_secs(60);

/// How long the host waits for the guest to come up at all.
const CONNECT_DEADLINE: std::time::Duration = std::time::Duration::from_secs(10);

/// The internal subcommand name. Never advertised: the host passes it when it spawns the child.
pub const GUEST_SUBCOMMAND: &str = "__guest";

/// The ONLY environment variables a guest inherits: the ones its platform's LOADER needs to start a
/// process at all. One list, read by the spawn and by the test that guards it, so the two cannot
/// drift — the same reason P1-F3's flag allowlist is derived from the help text.
///
/// Learned twice from guests that died before running a line: Windows at 0xC0000135, DLL not found,
/// and Linux at 127, `libpython3.13.so.1.0: cannot open shared object file`, because this binary
/// links CPython. None of these names is authority.
#[cfg(windows)]
pub const LOADER_ENV: &[&str] = &["SystemRoot", "SystemDrive", "WINDIR", "PATH"];
#[cfg(target_os = "linux")]
pub const LOADER_ENV: &[&str] = &["LD_LIBRARY_PATH"];
#[cfg(target_os = "macos")]
pub const LOADER_ENV: &[&str] = &["DYLD_LIBRARY_PATH", "DYLD_FALLBACK_LIBRARY_PATH"];
#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub const LOADER_ENV: &[&str] = &[];

/// Run as the guest. Returns the process exit status.
pub fn run_guest(args: &[String]) -> i32 {
    // The guest is the server on its own channel, as the foreign worker is: the HOST's wait is then
    // bounded by a connect deadline rather than by an accept that could never return.
    let Some(dir) = args.first().map(std::path::PathBuf::from) else {
        eprintln!("error: the sandbox guest was started without a channel directory");
        return 2;
    };
    let listener = match crate::broker_transport::Listener::bind(&dir) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: the sandbox guest cannot open its channel: {e}");
            return 2;
        }
    };
    let mut conn = match listener.accept() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: the sandbox guest was never contacted by a host: {e}");
            return 2;
        }
    };
    if conn.set_read_timeout(Some(CHANNEL_DEADLINE)).is_err() {
        eprintln!("error: the sandbox guest cannot set its channel deadline — refusing to run unbounded");
        return 2;
    }
    let hello: Hello = match read_frame(&mut conn) {
        Ok(h) => h,
        // No hello, no run: a guest reached by anything other than its host does nothing at all.
        Err(e) => {
            eprintln!("error: the sandbox guest was started without a hello frame ({e})");
            return 2;
        }
    };
    if hello.version != CHANNEL_VERSION {
        eprintln!("error: the host speaks `{}`, this guest speaks `{CHANNEL_VERSION}`", hello.version);
        return 2;
    }
    // The hash pins WHAT runs: a program swapped in flight is refused, not executed.
    let got = blake3::hash(hello.program.as_bytes()).to_hex().to_string();
    if got != hello.hash {
        eprintln!("error: the program does not match the hash the host sent — refused");
        return 2;
    }

    let checked = delulu_check::check_source(0, &hello.program);
    if checked.diagnostics.iter().any(|d| d.is_error()) {
        eprintln!("error: the host sent a program that does not check");
        return 1;
    }

    delulu_runtime::prim::set_rand_seed(hello.seed);
    delulu_runtime::prim::set_fixed_clock_ms(hello.fixed_clock_ms);

    // The last thing before the program runs: the guest narrows itself to what interpreting needs.
    // Fail closed — a guest that cannot be locked down does not run the program.
    match crate::jail::lock_down_self() {
        Ok(applied) if !applied.is_empty() => eprintln!("sandbox: the guest locked itself down — {}", applied.join("; ")),
        Ok(_) => {}
        Err(why) => {
            eprintln!("error: the sandbox guest could not lock itself down ({why}) — nothing ran");
            return 2;
        }
    }

    let sink = Rc::new(ChannelSink::new(conn));
    let interp = Interp::new(&checked.module).with_effect_sink(sink.clone());
    // The guest's root grants NOTHING. Every capability the program obtains is minted by the host,
    // over the channel, from the root the operator actually granted.
    let exit = match interp.run_main(Value::Root(Rc::new(RootVal::default()))) {
        Ok(_) => 0,
        Err(f) => {
            eprintln!("error[{}]: {}", f.code, f.message);
            1
        }
    };
    // Say goodbye even on failure, so the host stops serving rather than waiting on a dead guest.
    let _ = sink.done(exit);
    exit
}

/// The internal runner that drives the host half, so the whole arrangement is exercised through the
/// real binary before `--sandbox` exists. Internal for the same reason the guest is: PS-A3 replaces
/// it with the flag, the profiles and the run report the owner ruled on (D-V2-25).
pub const SANDBOX_RUN_SUBCOMMAND: &str = "__sandbox_run";

/// Effects the channel carries today. A guest may use exactly these; anything else is REFUSED rather
/// than run unconfined, because "the sandbox quietly did not apply" is the failure this whole phase
/// exists to prevent (D-V2-25: refuse, never silently downgrade).
///
/// Actors, foreign C, Python, plugins, devices and secrets are not here yet: each needs its own
/// request kind on `delulu-sandbox-channel/1`, and a handle cannot stand in for a thread or a
/// library. They arrive with the rest of PS-A.
const CARRIED: &[delulu_check::ResourceKind] = &[
    delulu_check::ResourceKind::Console,
    delulu_check::ResourceKind::FsRead,
    delulu_check::ResourceKind::FsWrite,
    delulu_check::ResourceKind::Clock,
    delulu_check::ResourceKind::Rand,
];

/// What this program needs that a guest cannot be given yet, in the words a caller can act on.
pub fn unsupported_surface(program: &str) -> Option<String> {
    let checked = delulu_check::check_source(0, program);
    let mut missing: Vec<&'static str> = Vec::new();
    for facts in checked.result.facts.values() {
        for kind in &facts.cap_kinds {
            if !CARRIED.contains(kind) && !missing.contains(&kind.name()) {
                missing.push(kind.name());
            }
        }
    }
    // Capability kinds are not the whole surface: an actor is a language feature, not a capability,
    // and the channel has no request kind for a mailbox or a turn. Checking only the kinds let an
    // actor program through to a guest that then failed halfway, which is precisely the
    // "quietly not applied" shape this gate exists to prevent.
    for item in &checked.module.items {
        match item {
            delulu_syntax::ast::Item::Actor(_) if !missing.contains(&"actors") => missing.push("actors"),
            delulu_syntax::ast::Item::Foreign(_) if !missing.contains(&"foreign code") => {
                missing.push("foreign code")
            }
            _ => {}
        }
    }
    missing.sort_unstable();
    if missing.is_empty() {
        None
    } else {
        Some(missing.join(", "))
    }
}

/// `delulu run <file> --sandbox …` (PS-A-07): run the program as a jailed guest.
///
/// Refusals come first and are explicit, because the one thing a sandbox flag must never do is
/// quietly not apply: an unknown profile, limits that cannot be read, a surface the channel does not
/// carry, or a host with no jail at all are each refused with a reason and a way forward.
pub fn cmd_run_sandboxed(file: Option<&str>, opts: &crate::cli::Opts, _rest: &[String]) -> i32 {
    let Some(file) = file else {
        eprintln!("error: `run --sandbox` needs a file");
        return 2;
    };
    let program = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read `{file}`: {e}");
            return 2;
        }
    };
    let profile = match opts.sandbox_profile.as_deref() {
        None => crate::policy::Profile::Contained,
        Some(name) => match crate::policy::Profile::parse(name) {
            Some(p) => p,
            None => {
                eprintln!("error: `{name}` is not a sandbox profile (dev, contained, hostile-agent)");
                return 2;
            }
        },
    };
    let limits = match parse_limits(opts.limits.as_deref(), profile) {
        Ok(l) => l,
        Err(why) => {
            eprintln!("error: {why}");
            return 2;
        }
    };
    // PS-A-10, the AUDIT half: a dry run performs NOTHING. It reports the policy that would be in
    // force and the authority the program would need, which is what an agent wants before it lets
    // unfamiliar code run at all. Nothing is spawned, so nothing can slip through.
    let mode = match opts.sandbox_mode.as_deref() {
        None | Some("strict") => crate::policy::Mode::Strict,
        Some("audit") => crate::policy::Mode::Audit,
        Some(other) => {
            eprintln!("error: `--mode {other}` is not a mode this command knows (strict, audit)");
            return 2;
        }
    };
    if mode == crate::policy::Mode::Audit {
        let policy = crate::policy::SandboxPolicy::derive(1, profile, Some(limits), mode);
        let checked = delulu_check::check_source(0, &program);
        let required = crate::cli::required_grants_of(file, &checked);
        let carried = unsupported_surface(&program);
        let report = serde_json::json!({
            "command": "run",
            "schema": 1,
            "delulu_version": env!("CARGO_PKG_VERSION"),
            "diagnostics": [],
            "summary": { "errors": 0, "warnings": 0 },
            "sandbox": policy.to_json("none", &[]),
            "audit": {
                "required_grants": required,
                "unsupported_surface": carried,
            },
            "outcome": { "ran": false, "exit": 0 },
        });
        let text = serde_json::to_string_pretty(&report).expect("the audit report serializes");
        match opts.report_out.as_deref() {
            Some(path) => {
                if let Err(e) = std::fs::write(path, format!("{text}\n")) {
                    eprintln!("error: cannot write the audit report to `{path}`: {e}");
                    return 2;
                }
            }
            // With no report file the audit still has to reach its reader; nothing ran, so standard
            // output belongs to no program here.
            None => println!("{text}"),
        }
        return 0;
    }
    if let Some(missing) = unsupported_surface(&program) {
        eprintln!(
            "error: `--sandbox` cannot carry this program yet: it uses {missing}. \
             Nothing ran. Run it with `--sandbox=off` if you accept no confinement, or wait for the \
             channel to carry that surface — the sandbox will not quietly not apply."
        );
        return 2;
    }
    let mut grants = delulu_runtime::broker::Grants::default();
    for g in &opts.grants {
        if let Err(e) = grants.add(g) {
            eprintln!("error: {e}");
            return 2;
        }
    }
    let root = Rc::new(crate::cli::build_root(&grants));
    match spawn_and_serve_with(&program, root, 0xDE1, None, limits, profile, opts.report_out.as_deref()) {
        Ok(exit) => exit,
        Err(e) => {
            eprintln!("error: the sandboxed run failed: {e}");
            1
        }
    }
}

/// `--limits mem=N,cpu=S`: narrow the profile. A flag may ask for LESS than the profile allows and
/// never for more, so choosing a tight profile cannot be undone by a flag on the same line.
fn parse_limits(spec: Option<&str>, profile: crate::policy::Profile) -> Result<crate::jail::Limits, String> {
    let mut limits = profile.limits();
    let Some(spec) = spec else { return Ok(limits) };
    for part in spec.split(',').filter(|p| !p.is_empty()) {
        let (key, value) = part
            .split_once('=')
            .ok_or_else(|| format!("`--limits {part}` needs the form mem=BYTES or cpu=SECONDS"))?;
        let n: u64 = value.parse().map_err(|_| format!("`{value}` is not a number in `--limits {part}`"))?;
        match key.trim() {
            "mem" => limits.memory_bytes = n.min(limits.memory_bytes),
            "cpu" => limits.cpu_seconds = n.min(limits.cpu_seconds),
            other => return Err(format!("`--limits {other}=…` is not a limit this command knows (mem, cpu)")),
        }
    }
    Ok(limits)
}

/// `__sandbox_run <file.delulu> [--grant …]`: run a program in a guest, serving it from here.
pub fn run_sandboxed_cli(args: &[String]) -> i32 {
    let (file, opts) = crate::cli::parse_opts(args);
    let Some(file) = file else {
        eprintln!("error: `{SANDBOX_RUN_SUBCOMMAND}` needs a file");
        return 2;
    };
    let program = match std::fs::read_to_string(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read `{file}`: {e}");
            return 2;
        }
    };
    let mut grants = delulu_runtime::broker::Grants::default();
    for g in &opts.grants {
        if let Err(e) = grants.add(g) {
            eprintln!("error: {e}");
            return 2;
        }
    }
    let root = Rc::new(crate::cli::build_root(&grants));
    match spawn_and_serve(&program, root, 0xDE1, None) {
        Ok(exit) => exit,
        Err(e) => {
            eprintln!("error: the sandboxed run failed: {e}");
            1
        }
    }
}

/// The host half: launch a guest, hand it the program, and serve its effects under `root`.
///
/// Today the guest is an ordinary child process: it has no OS jail yet, which PS-A2 adds. What it
/// already has is no capability of its own — it can only ask.
pub fn spawn_and_serve(
    program: &str,
    root: Rc<RootVal>,
    seed: u64,
    fixed_clock_ms: Option<i64>,
) -> io::Result<i32> {
    spawn_and_serve_with(program, root, seed, fixed_clock_ms, crate::jail::Limits::default(), crate::policy::Profile::Contained, None)
}

/// The host half with the policy the operator chose, and the report it asked for.
#[allow(clippy::too_many_arguments)]
pub fn spawn_and_serve_with(
    program: &str,
    root: Rc<RootVal>,
    seed: u64,
    fixed_clock_ms: Option<i64>,
    limits: crate::jail::Limits,
    profile: crate::policy::Profile,
    report_out: Option<&str>,
) -> io::Result<i32> {
    let exe = std::env::current_exe()?;
    let dir = std::env::temp_dir().join(format!("delulu-guest-{}-{}", std::process::id(), channel_tag()));
    std::fs::create_dir_all(&dir)?;
    let (mut cmd, launched) = guest_command(&exe, &dir);
    // Where the platform allows it, the limits are in force from the guest's first instruction.
    let mut before_exec = crate::jail::harden(&mut cmd, limits);
    before_exec.extend(launched);
    let mut child = cmd.spawn()?;
    // PS-A-04: the OS jail, applied before the guest has been told what to run — it is still waiting
    // for a hello at this point, so it has executed no program bytes yet. (Creating the child
    // suspended and assigning before its first instruction is the stronger form, and needs a
    // raw-handle spawn that `std::process::Command` does not offer; it lands with the rest of PS-A2.)
    let (jail, enforced) = crate::jail::confine(&child, limits);
    let mut applied = before_exec;
    applied.extend(enforced.guarantees.iter().copied());
    if applied.is_empty() {
        // Never claim a boundary that was not applied: PS-0-04's rule, in the place it matters most.
        eprintln!("sandbox: no OS jail on this host yet — the guest still holds no authority of its own");
    } else {
        eprintln!("sandbox: the guest is confined — {}", applied.join("; "));
    }
    let _ = &jail;
    // Only now does the guest run: on Windows it was created suspended, so it meets its jail before
    // its first instruction rather than a moment after.
    if !crate::jail::resume(&child) {
        let _ = child.kill();
        let _ = child.wait();
        let _ = std::fs::remove_dir_all(&dir);
        return Err(io::Error::other("the sandbox guest could not be started under its jail"));
    }
    let served = converse(&mut child, &dir, program, root, seed, fixed_clock_ms);
    // Whatever happened on the channel, the child is not left running and the channel is removed.
    let status = child.wait();
    let _ = std::fs::remove_dir_all(&dir);
    // The run report (D-V2-21): written by the RUNTIME to the file the operator named, never on the
    // program's own output, which the program could forge. `granted` and `host_guarantees` carry
    // what this host actually applied, so a report never claims a boundary that was not there.
    if let Some(path) = report_out {
        let policy = crate::policy::SandboxPolicy::derive(
            if applied.is_empty() { 0 } else { 1 },
            profile,
            Some(limits),
            crate::policy::Mode::Strict,
        );
        let backend = if applied.is_empty() { "inproc" } else { "process" };
        let exit = served.as_ref().copied().unwrap_or(1);
        let report = serde_json::json!({
            "command": "run",
            "schema": 1,
            "delulu_version": env!("CARGO_PKG_VERSION"),
            "diagnostics": [],
            "summary": { "errors": if exit == 0 { 0 } else { 1 }, "warnings": 0 },
            "sandbox": policy.to_json(backend, &applied),
            "outcome": { "ran": true, "exit": exit },
        });
        let text = serde_json::to_string_pretty(&report).expect("the run report serializes");
        if let Err(e) = std::fs::write(path, format!("{text}\n")) {
            eprintln!("error: cannot write the run report to `{path}`: {e}");
        }
    }
    let status = status?;
    match served {
        Ok(exit) => Ok(exit),
        // A guest that dies without saying goodbye is a failure, never a silent success — and the
        // REASON is reported in both branches. This arm used to drop it whenever the child's exit
        // status was non-zero, which is exactly when the reason matters most: the channel diagnosis
        // was written, thrown away in a branch, and three CI runs read as an unexplained timeout.
        Err(e) => {
            eprintln!("sandbox: {e}");
            if status.success() {
                Err(io::Error::new(io::ErrorKind::UnexpectedEof, format!("the guest stopped mid-conversation: {e}")))
            } else {
                Ok(status.code().unwrap_or(1))
            }
        }
    }
}

/// How the guest is started (PS-A-05): an EMPTY environment plus the few variables the operating
/// system needs to load a process at all, no arguments beyond the channel directory, and no standard
/// input.
///
/// Environment variables are where secrets live in practice — tokens, keys, cloud credentials — and
/// none of them are authority DeluluLang granted. Inheriting the parent's environment would hand a
/// guest everything the operator's shell happens to hold, which is the opposite of the arrangement.
/// The program's own output is performed by the host, so the guest needs no standard input and
/// writes nothing to standard output; its standard error stays attached, because a guest that fails
/// must be able to say so.
fn guest_command(exe: &std::path::Path, dir: &std::path::Path) -> (std::process::Command, Vec<&'static str>) {
    // On macOS the guest is launched THROUGH the Seatbelt profile, so the confinement is in force
    // from its first instruction, as the suspended start is on Windows and `pre_exec` is on Linux.
    let plain = || {
        let mut cmd = std::process::Command::new(exe);
        cmd.arg(GUEST_SUBCOMMAND).arg(dir);
        cmd
    };
    #[cfg(target_os = "macos")]
    let (mut cmd, launched) = {
        let args: Vec<&std::ffi::OsStr> = vec![GUEST_SUBCOMMAND.as_ref(), dir.as_os_str()];
        match crate::jail::seatbelt_launcher(exe, dir, &args) {
            Some(pair) => pair,
            None => (plain(), Vec::new()),
        }
    };
    #[cfg(not(target_os = "macos"))]
    let (mut cmd, launched) = (plain(), Vec::new());
    cmd.env_clear();
    // Every platform needs the few variables its LOADER uses, and nothing else. Learned twice, both
    // times by a guest that died before running a line: Windows at 0xC0000135, DLL not found
    // (CI-free, found locally), and Linux at 127, `libpython3.13.so.1.0: cannot open shared object
    // file`, because this binary links CPython (CI run 35394515395). None of these is authority.
    for name in LOADER_ENV {
        if let Ok(v) = std::env::var(name) {
            cmd.env(name, v);
        }
    }
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::null());
    (cmd, launched)
}

/// Connect to the guest, tell it what to run, and serve it until it is done.
fn converse(
    child: &mut std::process::Child,
    dir: &std::path::Path,
    program: &str,
    root: Rc<RootVal>,
    seed: u64,
    fixed_clock_ms: Option<i64>,
) -> io::Result<i32> {
    let deadline = std::time::Instant::now() + CONNECT_DEADLINE;
    let mut conn = loop {
        match crate::broker_transport::connect(dir) {
            Ok(c) => break c,
            Err(e) if std::time::Instant::now() >= deadline => {
                // Say WHY, not just that it timed out. A guest killed by its own jail — a resource
                // limit at startup, a profile that refused its socket — otherwise prints nothing at
                // all, and the host sits out the deadline against a process that died in the first
                // millisecond (CI run 35391962354 cost two red runs to that silence).
                let died = child.try_wait().ok().flatten();
                let how = match died {
                    Some(status) => format!("the guest had already exited ({status})"),
                    None => "the guest is running but never opened its channel".to_string(),
                };
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("no channel after {CONNECT_DEADLINE:?}: {how} — {e}"),
                ));
            }
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(20)),
        }
    };
    conn.set_read_timeout(Some(CHANNEL_DEADLINE))?;
    let hello = Hello {
        version: CHANNEL_VERSION.to_string(),
        program: program.to_string(),
        hash: blake3::hash(program.as_bytes()).to_hex().to_string(),
        seed,
        fixed_clock_ms,
    };
    write_frame(&mut conn, &hello)?;
    let mut host = HostChannel::new(LocalSink).with_root(root);
    host.serve(&mut conn)
}


/// A per-call channel name: the clock alone collides when runs start together.
fn channel_tag() -> String {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("{t}-{n}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PS-A-05: a guest inherits nothing. The environment is where secrets actually live, so the
    /// child gets an empty one plus only what the OS needs to start a process at all.
    #[test]
    fn a_guest_inherits_no_environment_and_no_standard_input() {
        let (cmd, _) = guest_command(std::path::Path::new("delulu"), std::path::Path::new("chan"));
        let names: Vec<String> =
            cmd.get_envs().map(|(k, _)| k.to_string_lossy().to_string()).collect();
        // Read from the one list the spawn uses: a second copy here went stale twice, and a test
        // that disagrees with the code it guards is worse than no test (CI runs 35390738394 and
        // 35396024254 were red on exactly that).
        for n in &names {
            assert!(LOADER_ENV.contains(&n.as_str()), "the guest would inherit `{n}`");
        }
        assert!(names.len() <= LOADER_ENV.len(), "nothing beyond the loader list: {names:?}");
        // The arguments carry the channel directory and nothing else: no secret is ever on a command
        // line, where every process on the machine can read it.
        let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        assert_eq!(args, vec![GUEST_SUBCOMMAND.to_string(), "chan".to_string()]);
    }
}
